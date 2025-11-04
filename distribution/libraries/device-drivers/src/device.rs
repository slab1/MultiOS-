//! Device Abstraction Layer
//! 
//! Provides unified device interface for all hardware devices in MultiOS,
//! including plug-and-play detection and safe device operations.

use crate::{DeviceType, DriverResult, DriverError};
use spin::Mutex;
use core::fmt;
use core::sync::atomic::{AtomicU32, Ordering};

/// Unique device identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeviceId(pub u32);

/// Device handle for safe device access
#[derive(Debug, Clone, Copy)]
pub struct DeviceHandle {
    pub id: DeviceId,
    pub device_type: DeviceType,
    pub name: &'static str,
    pub is_available: bool,
}

/// Device state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceState {
    Uninitialized,
    Initializing,
    Ready,
    Suspended,
    Error,
    Removed,
}

/// Device capabilities and features
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum DeviceCapability {
    None = 0,
    Read = 1 << 0,
    Write = 1 << 1,
    Interrupt = 1 << 2,
    Dma = 1 << 3,
    Pnp = 1 << 4,
    HotPlug = 1 << 5,
    PowerManagement = 1 << 6,
}

bitflags! {
    /// Device capability flags
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct DeviceCapabilities: u32 {
        const NONE = DeviceCapability::None as u32;
        const READ = DeviceCapability::Read as u32;
        const WRITE = DeviceCapability::Write as u32;
        const INTERRUPT = DeviceCapability::Interrupt as u32;
        const DMA = DeviceCapability::Dma as u32;
        const PNP = DeviceCapability::Pnp as u32;
        const HOT_PLUG = DeviceCapability::HotPlug as u32;
        const POWER_MANAGEMENT = DeviceCapability::PowerManagement as u32;
    }
}

/// Device information structure
#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub id: DeviceId,
    pub name: &'static str,
    pub device_type: DeviceType,
    pub vendor_id: Option<u16>,
    pub product_id: Option<u16>,
    pub capabilities: DeviceCapabilities,
    pub state: DeviceState,
    pub is_hot_plug: bool,
    pub parent_bus: Option<BusHandle>,
}

/// Hardware address types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HardwareAddress {
    Port(u16),           // x86 I/O port
    Memory(u64),         // Memory-mapped I/O address
    Pci(u8, u8, u8),     // PCI bus:device:function
    Usb(u8, u8),         // USB bus:device
    I2c(u8, u8),         // I2C bus:address
    Spi(u8),             // SPI chip select
    None,
}

/// Device driver interface
pub trait DeviceDriver: Send + Sync {
    /// Get the device name
    fn name(&self) -> &'static str;
    
    /// Get supported device types
    fn supported_devices(&self) -> &[DeviceType];
    
    /// Initialize the device
    fn init(&self, device: &Device) -> DriverResult<()>;
    
    /// Clean up resources when device is removed
    fn remove(&self, device: &Device) -> DriverResult<()>;
    
    /// Read data from device
    fn read(&self, device: &Device, buffer: &mut [u8]) -> DriverResult<usize>;
    
    /// Write data to device
    fn write(&self, device: &Device, buffer: &[u8]) -> DriverResult<usize>;
    
    /// Control device operations
    fn ioctl(&self, device: &Device, command: u32, data: usize) -> DriverResult<usize>;
    
    /// Get device capabilities
    fn capabilities(&self) -> DeviceCapabilities;
}

/// Core device structure
pub struct Device {
    pub info: DeviceInfo,
    pub hardware_addr: HardwareAddress,
    pub driver: Option<&'static dyn DeviceDriver>,
    pub private_data: Option<*mut u8>,
}

/// Safe device handle wrapper
pub struct SafeDevice {
    pub handle: DeviceHandle,
    lock: Mutex<()>,
}

impl DeviceHandle {
    /// Create a new device handle
    pub fn new(id: DeviceId, device_type: DeviceType, name: &'static str) -> Self {
        Self {
            id,
            device_type,
            name,
            is_available: true,
        }
    }
    
    /// Mark device as unavailable
    pub fn set_unavailable(&mut self) {
        self.is_available = false;
    }
    
    /// Check if device is available for operations
    pub fn is_available(&self) -> bool {
        self.is_available
    }
}

impl fmt::Display for DeviceHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} (ID: {:?})", self.name, self.id)
    }
}

impl Device {
    /// Create a new device
    pub fn new(
        info: DeviceInfo,
        hardware_addr: HardwareAddress,
        driver: Option<&'static dyn DeviceDriver>,
    ) -> Self {
        Self {
            info,
            hardware_addr,
            driver,
            private_data: None,
        }
    }
    
    /// Initialize the device
    pub fn init(&self) -> DriverResult<()> {
        if let Some(driver) = self.driver {
            driver.init(self)
        } else {
            Ok(())
        }
    }
    
    /// Remove and clean up the device
    pub fn remove(&self) -> DriverResult<()> {
        if let Some(driver) = self.driver {
            driver.remove(self)
        } else {
            Ok(())
        }
    }
    
    /// Read from device
    pub fn read(&self, buffer: &mut [u8]) -> DriverResult<usize> {
        if let Some(driver) = self.driver {
            driver.read(self, buffer)
        } else {
            Err(DriverError::DriverNotSupported)
        }
    }
    
    /// Write to device
    pub fn write(&self, buffer: &[u8]) -> DriverResult<usize> {
        if let Some(driver) = self.driver {
            driver.write(self, buffer)
        } else {
            Err(DriverError::DriverNotSupported)
        }
    }
    
    /// Device I/O control
    pub fn ioctl(&self, command: u32, data: usize) -> DriverResult<usize> {
        if let Some(driver) = self.driver {
            driver.ioctl(self, command, data)
        } else {
            Err(DriverError::DriverNotSupported)
        }
    }
    
    /// Update device state
    pub fn set_state(&mut self, state: DeviceState) {
        self.info.state = state;
    }
    
    /// Get device state
    pub fn state(&self) -> DeviceState {
        self.info.state
    }
    
    /// Check if device supports a capability
    pub fn has_capability(&self, capability: DeviceCapability) -> bool {
        self.info.capabilities.contains(DeviceCapabilities::from_bits_truncate(capability as u32))
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        // Clean up when device is dropped
        if let Err(e) = self.remove() {
            // Log error but don't panic
            crate::log::warn!("Failed to properly remove device {}: {:?}", self.info.name, e);
        }
    }
}

impl fmt::Display for Device {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.info.name)
    }
}

/// Bus handle for device hierarchy
#[derive(Debug, Clone, Copy)]
pub struct BusHandle {
    pub bus_type: BusType,
    pub bus_id: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum BusType {
    Pci = 0,
    Usb = 1,
    I2c = 2,
    Spi = 3,
    Isa = 4,
    Platform = 5,
}

/// Device manager internal device registry
static NEXT_DEVICE_ID: AtomicU32 = AtomicU32::new(1);

impl Device {
    /// Allocate a new unique device ID
    pub fn allocate_id() -> DeviceId {
        DeviceId(NEXT_DEVICE_ID.fetch_add(1, Ordering::SeqCst))
    }
}

impl SafeDevice {
    /// Create a new safe device wrapper
    pub fn new(handle: DeviceHandle) -> Self {
        Self {
            handle,
            lock: Mutex::new(()),
        }
    }
    
    /// Acquire exclusive access to device
    pub fn lock(&self) -> spin::MutexGuard<()> {
        self.lock.lock()
    }
}

/// Device discovery and enumeration
pub struct DeviceEnumerator {
    pub bus: BusHandle,
    pub devices_found: usize,
}

impl DeviceEnumerator {
    /// Create a new device enumerator
    pub fn new(bus: BusHandle) -> Self {
        Self {
            bus,
            devices_found: 0,
        }
    }
    
    /// Enumerate devices on the bus
    pub fn enumerate(&mut self) -> DriverResult<Vec<DeviceInfo>> {
        self.devices_found = 0;
        let mut devices = Vec::new();
        
        // This will be implemented by specific bus drivers
        // For now, return empty vector
        Ok(devices)
    }
    
    /// Increment device counter
    fn increment_count(&mut self) {
        self.devices_found += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockDriver;

    impl DeviceDriver for MockDriver {
        fn name(&self) -> &'static str {
            "Mock Driver"
        }
        
        fn supported_devices(&self) -> &[DeviceType] {
            &[DeviceType::Keyboard]
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
    fn test_device_creation() {
        let device_info = DeviceInfo {
            id: Device::allocate_id(),
            name: "Test Device",
            device_type: DeviceType::Keyboard,
            vendor_id: Some(0x1234),
            product_id: Some(0x5678),
            capabilities: DeviceCapabilities::READ | DeviceCapabilities::WRITE,
            state: DeviceState::Uninitialized,
            is_hot_plug: false,
            parent_bus: None,
        };
        
        let device = Device::new(
            device_info,
            HardwareAddress::Port(0x3F8),
            Some(&MockDriver),
        );
        
        assert_eq!(device.info.name, "Test Device");
        assert_eq!(device.info.device_type, DeviceType::Keyboard);
        assert!(device.has_capability(DeviceCapability::Read));
        assert!(device.has_capability(DeviceCapability::Write));
        assert!(!device.has_capability(DeviceCapability::Interrupt));
    }

    #[test]
    fn test_device_capabilities() {
        let caps = DeviceCapabilities::READ | DeviceCapabilities::WRITE | DeviceCapabilities::INTERRUPT;
        
        assert!(caps.contains(DeviceCapabilities::READ));
        assert!(caps.contains(DeviceCapabilities::WRITE));
        assert!(caps.contains(DeviceCapabilities::INTERRUPT));
        assert!(!caps.contains(DeviceCapabilities::DMA));
    }

    #[test]
    fn test_hardware_address_types() {
        assert_eq!(
            HardwareAddress::Pci(0, 1, 2),
            HardwareAddress::Pci(0, 1, 2)
        );
        
        match HardwareAddress::Port(0x3F8) {
            HardwareAddress::Port(0x3F8) => {},
            _ => panic!("Expected Port address"),
        }
    }

    #[test]
    fn test_device_enumerator() {
        let enumerator = DeviceEnumerator::new(BusHandle {
            bus_type: BusType::Pci,
            bus_id: 0,
        });
        
        assert_eq!(enumerator.bus.bus_type, BusType::Pci);
        assert_eq!(enumerator.bus.bus_id, 0);
        assert_eq!(enumerator.devices_found, 0);
    }

    #[test]
    fn test_device_handle_display() {
        let handle = DeviceHandle::new(DeviceId(1), DeviceType::Keyboard, "Test Keyboard");
        assert_eq!(format!("{}", handle), "Test Keyboard (ID: DeviceId(1))");
    }
}