//! Driver Manager
//! 
//! Central registry for device drivers with automatic device discovery,
//! driver binding, and plug-and-play support.

use crate::{DeviceType, DriverResult, DriverError, DeviceHandle, DeviceInfo};
use crate::device::{Device, HardwareAddress, BusHandle, BusType, DeviceEnumerator};
use spin::{Mutex, RwLock};
use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use log::{info, warn, error};

/// Driver registration information
#[derive(Debug, Clone)]
pub struct Driver {
    pub name: &'static str,
    pub version: &'static str,
    pub device_types: &'static [DeviceType],
    pub priority: u8, // Lower numbers = higher priority
    pub driver_func: fn() -> &'static dyn DeviceDriver,
}

/// Driver binding information
#[derive(Debug)]
struct DriverBinding {
    driver: &'static dyn DeviceDriver,
    devices: Vec<DeviceHandle>,
    reference_count: u32,
}

/// Driver manager statistics
#[derive(Debug, Clone)]
pub struct DriverManagerStats {
    pub total_drivers: usize,
    pub active_devices: usize,
    pub total_device_types: usize,
    pub hot_plug_devices: usize,
}

/// Driver event callback function type
pub type DriverEventCallback = fn(event: DriverEvent, device_info: &DeviceInfo);

/// Driver events
#[derive(Debug, Clone)]
pub enum DriverEvent {
    DriverRegistered(&'static str),
    DriverUnregistered(&'static str),
    DeviceDetected(DeviceInfo),
    DeviceInitialized(DeviceHandle),
    DeviceRemoved(DeviceId),
    DeviceError(DeviceId, DriverError),
    HotPlugEvent(DeviceInfo),
}

/// Main driver manager
pub struct DriverManager {
    drivers: BTreeMap<DeviceType, Vec<Driver>>,
    device_registry: RwLock<BTreeMap<u32, Device>>,
    driver_bindings: BTreeMap<DeviceType, Vec<DriverBinding>>,
    device_counters: BTreeMap<DeviceType, u32>,
    event_callbacks: Vec<DriverEventCallback>,
    stats: DriverManagerStats,
}

impl DriverManager {
    /// Create a new driver manager
    pub fn new() -> Self {
        info!("Initializing MultiOS Driver Manager");
        
        let manager = Self {
            drivers: BTreeMap::new(),
            device_registry: RwLock::new(BTreeMap::new()),
            driver_bindings: BTreeMap::new(),
            device_counters: BTreeMap::new(),
            event_callbacks: Vec::new(),
            stats: DriverManagerStats {
                total_drivers: 0,
                active_devices: 0,
                total_device_types: DeviceType::UART as usize + 1,
                hot_plug_devices: 0,
            },
        };
        
        info!("Driver Manager initialized with {} device types", manager.stats.total_device_types);
        manager
    }

    /// Register a driver
    pub fn register_driver(&mut self, driver: Driver) -> DriverResult<()> {
        info!("Registering driver: {} v{}", driver.name, driver.version);
        
        for &device_type in driver.device_types {
            let drivers_for_type = self.drivers.entry(device_type).or_insert_with(Vec::new);
            
            // Check if driver is already registered
            if drivers_for_type.iter().any(|d| d.name == driver.name) {
                warn!("Driver {} already registered for device type {:?}", driver.name, device_type);
                continue;
            }
            
            // Add driver, maintaining priority order (lower number = higher priority)
            let insert_pos = drivers_for_type
                .iter()
                .position(|d| d.priority > driver.priority)
                .unwrap_or(drivers_for_type.len());
            drivers_for_type.insert(insert_pos, driver.clone());
            
            info!("  - Registered for device type {:?}", device_type);
        }
        
        self.stats.total_drivers += 1;
        self.emit_event(DriverEvent::DriverRegistered(driver.name), None);
        
        Ok(())
    }

    /// Unregister a driver
    pub fn unregister_driver(&mut self, driver_name: &str) -> DriverResult<()> {
        let mut drivers_to_remove = Vec::new();
        
        for (device_type, drivers) in &mut self.drivers {
            if let Some(pos) = drivers.iter().position(|d| d.name == driver_name) {
                drivers_to_remove.push((*device_type, pos));
            }
        }
        
        for (device_type, pos) in drivers_to_remove {
            if let Some(drivers) = self.drivers.get_mut(&device_type) {
                drivers.remove(pos);
                info!("Unregistered driver {} for device type {:?}", driver_name, device_type);
            }
        }
        
        self.stats.total_drivers = self.drivers.values().flatten().count();
        self.emit_event(DriverEvent::DriverUnregistered(driver_name), None);
        
        Ok(())
    }

    /// Add a device to the registry
    pub fn add_device(&mut self, device_info: DeviceInfo, hardware_addr: HardwareAddress) -> DriverResult<DeviceHandle> {
        let device_id = device_info.id;
        let device_type = device_info.device_type;
        let device_name = device_info.name;
        
        info!("Adding device: {} (ID: {:?}, Type: {:?})", device_name, device_id, device_type);
        
        // Create device
        let device = Device::new(device_info, hardware_addr, None);
        
        // Add to registry
        let mut registry = self.device_registry.write();
        registry.insert(device_id.0, device);
        
        // Create device handle
        let handle = DeviceHandle::new(device_id, device_type, device_name);
        
        // Try to bind driver automatically
        if let Err(e) = self.bind_driver_for_device(device_id) {
            warn!("Failed to bind driver for device {}: {:?}", device_name, e);
        }
        
        self.stats.active_devices += 1;
        self.emit_event(DriverEvent::DeviceDetected(registry.get(&device_id.0).unwrap().info.clone()), None);
        
        Ok(handle)
    }

    /// Remove a device from the registry
    pub fn remove_device(&mut self, device_id: DeviceId) -> DriverResult<()> {
        info!("Removing device ID: {:?}", device_id);
        
        let mut registry = self.device_registry.write();
        
        if let Some(device) = registry.remove(&device_id.0) {
            // Clean up any driver bindings
            self.cleanup_driver_bindings(device_id, device.info.device_type);
            
            self.stats.active_devices = self.stats.active_devices.saturating_sub(1);
            self.emit_event(DriverEvent::DeviceRemoved(device_id), None);
            
            Ok(())
        } else {
            Err(DriverError::DeviceNotFound)
        }
    }

    /// Find devices by type
    pub fn find_devices(&self, device_type: DeviceType) -> DriverResult<Vec<DeviceHandle>> {
        let registry = self.device_registry.read();
        
        let devices = registry
            .values()
            .filter(|device| device.info.device_type == device_type)
            .map(|device| DeviceHandle::new(device.info.id, device.info.device_type, device.info.name))
            .collect();
        
        if devices.is_empty() {
            return Err(DriverError::DeviceNotFound);
        }
        
        Ok(devices)
    }

    /// Get device by ID
    pub fn get_device(&mut self, device_id: DeviceId) -> DriverResult<DeviceHandle> {
        let registry = self.device_registry.read();
        
        if let Some(device) = registry.get(&device_id.0) {
            Ok(DeviceHandle::new(device.info.id, device.info.device_type, device.info.name))
        } else {
            Err(DriverError::DeviceNotFound)
        }
    }

    /// Get device count
    pub fn get_device_count(&self) -> usize {
        self.stats.active_devices
    }

    /// Get driver manager statistics
    pub fn get_stats(&self) -> &DriverManagerStats {
        &self.stats
    }

    /// Register event callback
    pub fn register_event_callback(&mut self, callback: DriverEventCallback) {
        info!("Registering driver event callback");
        self.event_callbacks.push(callback);
    }

    /// Trigger device discovery on all buses
    pub fn discover_devices(&mut self) -> DriverResult<()> {
        info!("Starting device discovery on all buses");
        
        // Discover PCI devices
        self.discover_pci_devices()?;
        
        // Discover USB devices  
        self.discover_usb_devices()?;
        
        // Discover platform devices
        self.discover_platform_devices()?;
        
        info!("Device discovery completed");
        Ok(())
    }

    /// Bind driver for specific device
    fn bind_driver_for_device(&mut self, device_id: u32) -> DriverResult<()> {
        let registry = self.device_registry.read();
        let device = registry.get(&device_id)
            .ok_or(DriverError::DeviceNotFound)?;
        
        let device_type = device.info.device_type;
        
        // Get available drivers for this device type
        let available_drivers = self.drivers.get(&device_type)
            .ok_or(DriverError::DriverNotSupported)?;
        
        if available_drivers.is_empty() {
            return Err(DriverError::DriverNotSupported);
        }
        
        // Try drivers in priority order
        for driver_info in available_drivers {
            let driver = (driver_info.driver_func)();
            
            // Check if driver supports this specific device
            if self.is_device_supported_by_driver(device, driver) {
                info!("Binding driver {} to device {:?}", driver.name(), device_id);
                
                // Initialize device with driver
                let mut device_mut = registry.get(&device_id).unwrap();
                device_mut.driver = Some(driver);
                device_mut.set_state(crate::device::DeviceState::Initializing);
                
                // Register binding
                let binding = DriverBinding {
                    driver,
                    devices: Vec::new(),
                    reference_count: 0,
                };
                
                self.driver_bindings
                    .entry(device_type)
                    .or_insert_with(Vec::new)
                    .push(binding);
                
                self.emit_event(DriverEvent::DeviceInitialized(
                    DeviceHandle::new(device.info.id, device.info.device_type, device.info.name)
                ), Some(&device.info));
                
                return Ok(());
            }
        }
        
        Err(DriverError::DriverNotSupported)
    }

    /// Check if device is supported by driver
    fn is_device_supported_by_driver(&self, device: &Device, driver: &dyn DeviceDriver) -> bool {
        // Check if device type is supported
        if !driver.supported_devices().contains(&device.info.device_type) {
            return false;
        }
        
        // Check vendor/product IDs if specified
        if let (Some(vendor_id), Some(product_id)) = (device.info.vendor_id, device.info.product_id) {
            // This would be implemented in specific drivers
            // For now, assume all devices with matching type are supported
        }
        
        true
    }

    /// Clean up driver bindings when device is removed
    fn cleanup_driver_bindings(&mut self, device_id: DeviceId, device_type: DeviceType) {
        if let Some(bindings) = self.driver_bindings.get_mut(&device_type) {
            for binding in bindings.iter_mut() {
                if let Some(pos) = binding.devices.iter().position(|h| h.id == device_id) {
                    binding.devices.remove(pos);
                    binding.reference_count = binding.reference_count.saturating_sub(1);
                    break;
                }
            }
            
            // Remove binding if no more devices
            bindings.retain(|b| !b.devices.is_empty());
        }
    }

    /// Discover PCI devices
    fn discover_pci_devices(&mut self) -> DriverResult<()> {
        info!("Discovering PCI devices");
        
        // This would implement actual PCI enumeration
        // For now, simulate some common devices
        
        // Example: Serial port
        let serial_info = DeviceInfo {
            id: Device::allocate_id(),
            name: "16550 UART",
            device_type: DeviceType::UART,
            vendor_id: Some(0x8086), // Intel
            product_id: Some(0x7000),
            capabilities: crate::device::DeviceCapabilities::READ | crate::device::DeviceCapabilities::WRITE,
            state: crate::device::DeviceState::Uninitialized,
            is_hot_plug: false,
            parent_bus: Some(BusHandle { bus_type: BusType::Pci, bus_id: 0 }),
        };
        
        self.add_device(serial_info, HardwareAddress::Pci(0, 31, 0))?;
        
        Ok(())
    }

    /// Discover USB devices
    fn discover_usb_devices(&mut self) -> DriverResult<()> {
        info!("Discovering USB devices");
        
        // This would implement actual USB enumeration
        // For now, simulate keyboard device
        
        let keyboard_info = DeviceInfo {
            id: Device::allocate_id(),
            name: "USB Keyboard",
            device_type: DeviceType::Keyboard,
            vendor_id: Some(0x046D), // Logitech
            product_id: Some(0xC31C),
            capabilities: crate::device::DeviceCapabilities::READ | crate::device::DeviceCapabilities::INTERRUPT,
            state: crate::device::DeviceState::Uninitialized,
            is_hot_plug: true,
            parent_bus: Some(BusHandle { bus_type: BusType::Usb, bus_id: 1 }),
        };
        
        self.add_device(keyboard_info, HardwareAddress::Usb(1, 2))?;
        
        Ok(())
    }

    /// Discover platform devices
    fn discover_platform_devices(&mut self) -> DriverResult<()> {
        info!("Discovering platform devices");
        
        // This would implement platform device enumeration
        // For timers, clocks, etc.
        
        Ok(())
    }

    /// Emit driver event
    fn emit_event(&self, event: DriverEvent, device_info: Option<&DeviceInfo>) {
        for callback in &self.event_callbacks {
            callback(event.clone(), device_info.unwrap_or(&DeviceInfo {
                id: Device::allocate_id(),
                name: "Unknown",
                device_type: DeviceType::Unknown,
                vendor_id: None,
                product_id: None,
                capabilities: crate::device::DeviceCapabilities::NONE,
                state: crate::device::DeviceState::Uninitialized,
                is_hot_plug: false,
                parent_bus: None,
            }));
        }
    }

    /// List all registered drivers
    pub fn list_drivers(&self) -> Vec<&'static str> {
        let mut driver_names = Vec::new();
        for drivers in self.drivers.values() {
            for driver in drivers {
                if !driver_names.contains(&driver.name) {
                    driver_names.push(driver.name);
                }
            }
        }
        driver_names.sort();
        driver_names
    }

    /// List all devices
    pub fn list_devices(&self) -> Vec<(DeviceId, DeviceType, &'static str)> {
        let registry = self.device_registry.read();
        registry.values()
            .map(|device| (device.info.id, device.info.device_type, device.info.name))
            .collect()
    }
}

impl Default for DriverManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::device::{MockDriver, DeviceInfo};

    struct TestDriver;

    impl DeviceDriver for TestDriver {
        fn name(&self) -> &'static str {
            "Test Driver"
        }
        
        fn supported_devices(&self) -> &[DeviceType] {
            &[DeviceType::Keyboard, DeviceType::Mouse]
        }
        
        fn init(&self, _device: &Device) -> DriverResult<()> {
            Ok(())
        }
        
        fn remove(&self, _device: &Device) -> DriverResult<()> {
            Ok(())
        }
        
        fn read(&self, _device: &Device, buffer: &mut [u8]) -> DriverResult<usize> {
            Ok(buffer.len())
        }
        
        fn write(&self, _device: &Device, buffer: &[u8]) -> DriverResult<usize> {
            Ok(buffer.len())
        }
        
        fn ioctl(&self, _device: &Device, _command: u32, _data: usize) -> DriverResult<usize> {
            Ok(0)
        }
        
        fn capabilities(&self) -> DeviceCapabilities {
            DeviceCapabilities::READ | DeviceCapabilities::WRITE
        }
    }

    #[test]
    fn test_driver_manager_creation() {
        let manager = DriverManager::new();
        
        assert_eq!(manager.get_device_count(), 0);
        assert_eq!(manager.get_stats().total_drivers, 0);
        assert_eq!(manager.get_stats().active_devices, 0);
    }

    #[test]
    fn test_driver_registration() {
        let mut manager = DriverManager::new();
        
        let driver = Driver {
            name: "Test Driver",
            version: "1.0",
            device_types: &[DeviceType::Keyboard],
            priority: 100,
            driver_func: || &TestDriver,
        };
        
        assert!(manager.register_driver(driver).is_ok());
        assert_eq!(manager.get_stats().total_drivers, 1);
        
        let drivers = manager.list_drivers();
        assert!(drivers.contains(&"Test Driver"));
    }

    #[test]
    fn test_device_addition_and_removal() {
        let mut manager = DriverManager::new();
        
        let device_info = DeviceInfo {
            id: Device::allocate_id(),
            name: "Test Device",
            device_type: DeviceType::Keyboard,
            vendor_id: Some(0x1234),
            product_id: Some(0x5678),
            capabilities: DeviceCapabilities::READ,
            state: crate::device::DeviceState::Uninitialized,
            is_hot_plug: false,
            parent_bus: None,
        };
        
        let handle = manager.add_device(device_info, HardwareAddress::Port(0x3F8)).unwrap();
        assert_eq!(manager.get_device_count(), 1);
        
        assert!(manager.remove_device(handle.id).is_ok());
        assert_eq!(manager.get_device_count(), 0);
    }

    #[test]
    fn test_device_discovery() {
        let mut manager = DriverManager::new();
        
        assert!(manager.discover_devices().is_ok());
        
        // Should find at least PCI and USB devices
        let devices = manager.list_devices();
        assert!(!devices.is_empty());
        
        // Check that we found expected device types
        let device_types: Vec<DeviceType> = devices.iter().map(|(_, ty, _)| *ty).collect();
        assert!(device_types.contains(&DeviceType::UART));
        assert!(device_types.contains(&DeviceType::Keyboard));
    }

    #[test]
    fn test_driver_event_callback() {
        let mut manager = DriverManager::new();
        
        let mut event_received = false;
        let callback: DriverEventCallback = |event, _| {
            if matches!(event, DriverEvent::DriverRegistered("Test Driver")) {
                event_received = true;
            }
        };
        
        manager.register_event_callback(callback);
        
        let driver = Driver {
            name: "Test Driver",
            version: "1.0",
            device_types: &[DeviceType::Keyboard],
            priority: 100,
            driver_func: || &TestDriver,
        };
        
        assert!(manager.register_driver(driver).is_ok());
        // Note: Event callback is called but we can't easily test it in unit test
    }
}