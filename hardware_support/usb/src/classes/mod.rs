//! USB Device Class Drivers Module
//! 
//! Contains implementations for standard USB device classes:
//! - HID (Human Interface Device) - keyboards, mice, game controllers
//! - MSC (Mass Storage Class) - flash drives, external hard drives  
//! - CDC (Communications Device Class) - modems, network adapters
//! - Audio - speakers, microphones, audio interfaces

pub mod hid;
pub mod msc;
pub mod cdc;
pub mod audio;

// Re-export classes for easy access
pub use hid::{HidDriver, HidEvent, HidUsagePage, HidGenericDesktopUsage};
pub use msc::{MscDriver, MscCommandResult, ScsiOperationCode, ScsiResponseCode};
pub use cdc::{CdcDriver, CdcAcmDriver, CdcNcmDriver, CdcSubclass, CdcProtocol};
pub use audio::{AudioDriver, AudioStreamFormat, AudioDataFormat, AudioTerminalType};

use crate::*;

/// USB Device Class Registration
#[derive(Debug, Clone)]
pub struct UsbDeviceClass {
    pub class_code: UsbClass,
    pub subclass_code: u8,
    pub protocol_code: u8,
    pub driver_factory: fn(u8) -> UsbResult<Box<dyn UsbClassDriver>>,
}

/// USB Class Driver Trait
pub trait UsbClassDriver {
    /// Initialize the device
    fn initialize(&mut self) -> UsbResult<()>;
    
    /// Get device information
    fn get_device_info(&self) -> UsbResult<String>;
    
    /// Process USB data
    fn process_data(&mut self, endpoint: u8, data: &[u8]) -> UsbResult<()>;
    
    /// Get driver status
    fn get_status(&self) -> UsbResult<UsbDriverStatus>;
    
    /// Check if driver is active
    fn is_active(&self) -> bool;
    
    /// Clean up resources
    fn cleanup(&mut self);
}

/// USB Driver Status
#[derive(Debug, Clone)]
pub struct UsbDriverStatus {
    pub state: UsbDeviceState,
    pub configuration: u8,
    pub interface: u8,
    pub endpoints_active: u8,
    pub bytes_processed: u64,
    pub last_error: Option<UsbDriverError>,
}

/// USB Device Class Manager
pub struct UsbClassManager {
    pub drivers: BTreeMap<u8, Box<dyn UsbClassDriver>>,
    pub class_registrations: Vec<UsbDeviceClass>,
    pub device_addresses: BTreeMap<u8, u8>, // Maps device address to driver ID
    pub initialized: bool,
}

impl UsbClassManager {
    /// Create a new class manager
    pub fn new() -> Self {
        let mut manager = Self {
            drivers: BTreeMap::new(),
            class_registrations: Vec::new(),
            device_addresses: BTreeMap::new(),
            initialized: false,
        };

        // Register standard USB classes
        manager.register_standard_classes();
        manager
    }

    /// Register standard USB device classes
    fn register_standard_classes(&mut self) {
        // Register HID class
        self.class_registrations.push(UsbDeviceClass {
            class_code: UsbClass::HID,
            subclass_code: 0x00, // Boot Interface
            protocol_code: 0x00, // No specific protocol
            driver_factory: |address| -> UsbResult<Box<dyn UsbClassDriver>> {
                Ok(Box::new(HidDriver::new(address)))
            },
        });

        // Register MSC class
        self.class_registrations.push(UsbDeviceClass {
            class_code: UsbClass::MassStorage,
            subclass_code: 0x06, // SCSI
            protocol_code: 0x50, // Bulk-Only Transport
            driver_factory: |address| -> UsbResult<Box<dyn UsbClassDriver>> {
                Ok(Box::new(MscDriver::new(address)))
            },
        });

        // Register CDC class
        self.class_registrations.push(UsbDeviceClass {
            class_code: UsbClass::Communications,
            subclass_code: 0x02, // Abstract Control Model (ACM)
            protocol_code: 0x01, // V.25ter (AT commands)
            driver_factory: |address| -> UsbResult<Box<dyn UsbClassDriver>> {
                Ok(Box::new(CdcAcmDriver::new(address)))
            },
        });

        // Register Audio class
        self.class_registrations.push(UsbDeviceClass {
            class_code: UsbClass::Audio,
            subclass_code: 0x01, // Audio Control
            protocol_code: 0x00, // No specific protocol
            driver_factory: |address| -> UsbResult<Box<dyn UsbClassDriver>> {
                Ok(Box::new(AudioDriver::new(address)))
            },
        });

        log::info!("Registered {} USB device classes", self.class_registrations.len());
    }

    /// Initialize the class manager
    pub fn initialize(&mut self) -> UsbResult<()> {
        if self.initialized {
            return Ok(());
        }

        self.initialized = true;
        log::info!("USB Class Manager initialized");
        Ok(())
    }

    /// Create a driver for a specific device
    pub fn create_driver(&mut self, device_address: u8, class_code: UsbClass, 
                        subclass_code: u8, protocol_code: u8) -> UsbResult<u8> {
        if !self.initialized {
            return Err(UsbDriverError::ControllerNotInitialized);
        }

        // Find matching class registration
        let registration = self.class_registrations.iter()
            .find(|reg| {
                reg.class_code == class_code &&
                reg.subclass_code == subclass_code &&
                reg.protocol_code == protocol_code
            })
            .or_else(|| {
                // Try with any subclass/protocol if exact match fails
                self.class_registrations.iter()
                    .find(|reg| reg.class_code == class_code)
            })
            .ok_or_else(|| {
                log::warn!("No driver found for class {:#x}/{:#x}/{:#x}", 
                          class_code as u8, subclass_code, protocol_code);
                UsbDriverError::UnsupportedFeature
            })?;

        // Create driver instance
        let driver = (registration.driver_factory)(device_address)
            .map_err(|e| {
                log::warn!("Failed to create driver for device {}: {:?}", device_address, e);
                e
            })?;

        let driver_id = self.drivers.len() as u8;
        self.drivers.insert(driver_id, driver);
        self.device_addresses.insert(device_address, driver_id);

        log::info!("Created driver for device {} (class {:#x})", device_address, class_code as u8);
        Ok(driver_id)
    }

    /// Initialize a specific driver
    pub fn initialize_driver(&mut self, driver_id: u8) -> UsbResult<()> {
        let driver = self.drivers.get_mut(&driver_id)
            .ok_or(UsbDriverError::DeviceNotFound { address: driver_id })?;

        driver.initialize()
    }

    /// Process USB data for a device
    pub fn process_device_data(&mut self, device_address: u8, endpoint: u8, data: &[u8]) -> UsbResult<()> {
        let driver_id = *self.device_addresses.get(&device_address)
            .ok_or(UsbDriverError::DeviceNotFound { address: device_address })?;

        let driver = self.drivers.get_mut(&driver_id)
            .ok_or(UsbDriverError::DeviceNotFound { address: driver_id })?;

        driver.process_data(endpoint, data)
    }

    /// Get driver information
    pub fn get_driver_info(&self, driver_id: u8) -> UsbResult<String> {
        let driver = self.drivers.get(&driver_id)
            .ok_or(UsbDriverError::DeviceNotFound { address: driver_id })?;

        driver.get_device_info()
    }

    /// Get all drivers
    pub fn get_drivers(&self) -> Vec<u8> {
        self.drivers.keys().cloned().collect()
    }

    /// Get driver by device address
    pub fn get_driver_id(&self, device_address: u8) -> UsbResult<u8> {
        self.device_addresses.get(&device_address)
            .copied()
            .ok_or(UsbDriverError::DeviceNotFound { address: device_address })
    }

    /// Get all registered classes
    pub fn get_registered_classes(&self) -> &[UsbDeviceClass] {
        &self.class_registrations
    }

    /// Remove driver
    pub fn remove_driver(&mut self, driver_id: u8) -> UsbResult<()> {
        let driver = self.drivers.remove(&driver_id)
            .ok_or(UsbDriverError::DeviceNotFound { address: driver_id })?;

        driver.cleanup();

        // Remove from device address mapping
        if let Some((&device_address, &id)) = self.device_addresses.iter().find(|&(_, &id)| id == driver_id) {
            if id == driver_id {
                self.device_addresses.remove(&device_address);
            }
        }

        log::info!("Removed driver {}", driver_id);
        Ok(())
    }

    /// Get system statistics
    pub fn get_system_stats(&self) -> UsbDriverSystemStats {
        let mut total_bytes = 0;
        let mut active_drivers = 0;
        let mut error_count = 0;

        for driver in self.drivers.values() {
            if let Ok(status) = driver.get_status() {
                total_bytes += status.bytes_processed;
                if status.state == UsbDeviceState::Configured {
                    active_drivers += 1;
                }
                if status.last_error.is_some() {
                    error_count += 1;
                }
            }
        }

        UsbDriverSystemStats {
            total_drivers: self.drivers.len() as u32,
            active_drivers,
            total_bytes_processed: total_bytes,
            error_count,
            driver_types: self.get_driver_type_distribution(),
        }
    }

    /// Get driver type distribution
    fn get_driver_type_distribution(&self) -> BTreeMap<String, u32> {
        let mut distribution = BTreeMap::new();

        for driver in self.drivers.values() {
            let info = match driver.get_device_info() {
                Ok(info) => info,
                Err(_) => "Unknown".to_string(),
            };

            let class_name = if info.starts_with("HID") {
                "HID".to_string()
            } else if info.starts_with("MSC") {
                "MSC".to_string()
            } else if info.starts_with("CDC") {
                "CDC".to_string()
            } else if info.starts_with("Audio") {
                "Audio".to_string()
            } else {
                "Unknown".to_string()
            };

            *distribution.entry(class_name).or_insert(0) += 1;
        }

        distribution
    }

    /// Check if manager is initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Get number of active drivers
    pub fn active_driver_count(&self) -> usize {
        self.drivers.values()
            .filter(|driver| driver.is_active())
            .count()
    }
}

/// USB Driver System Statistics
#[derive(Debug, Clone)]
pub struct UsbDriverSystemStats {
    pub total_drivers: u32,
    pub active_drivers: u32,
    pub total_bytes_processed: u64,
    pub error_count: u32,
    pub driver_types: BTreeMap<String, u32>,
}

/// USB Device Type Detection
pub struct UsbDeviceTypeDetector {
    pub known_devices: BTreeMap<(u16, u16), String>,
    pub class_detectors: Vec<fn(&UsbDeviceDescriptor) -> bool>,
}

impl UsbDeviceTypeDetector {
    /// Create a new device type detector
    pub fn new() -> Self {
        let mut detector = Self {
            known_devices: BTreeMap::new(),
            class_detectors: Vec::new(),
        };

        // Add known devices
        detector.known_devices.insert((0x046D, 0xC52B), "Logitech Unifying Receiver".to_string());
        detector.known_devices.insert((0x046D, 0xC534), "Logitech Wireless Mouse".to_string());
        detector.known_devices.insert((0x0C45, 0x7401), "Cypress USB Keyboard".to_string());
        detector.known_devices.insert((0x045E, 0x0745), "Microsoft USB Mouse".to_string());
        detector.known_devices.insert((0x0789, 0x0198), "PlayStation USB Controller".to_string());

        // Add class-based detectors
        detector.class_detectors.push(detect_hid_device);
        detector.class_detectors.push(detect_msc_device);
        detector.class_detectors.push(detect_cdc_device);
        detector.known_devices.insert((0x1058, 0x0630), "Western Digital External HDD".to_string());
        detector.known_devices.insert((0x046D, 0x0870), "Logitech Webcam".to_string());
        detector.known_devices.insert((0x13D3, 0x3367), "USB Audio Device".to_string());

        detector
    }

    /// Detect device type from descriptor
    pub fn detect_device_type(&self, descriptor: &UsbDeviceDescriptor) -> String {
        // Check known devices first
        if let Some(device_name) = self.known_devices.get(&(descriptor.idVendor, descriptor.idProduct)) {
            return device_name.clone();
        }

        // Try class-based detection
        for detector in &self.class_detectors {
            if detector(descriptor) {
                return self.get_class_description(descriptor.bDeviceClass);
            }
        }

        // Fallback to generic description
        format!("USB Device {:#06X}:{:#06X}", descriptor.idVendor, descriptor.idProduct)
    }

    /// Get description for device class
    fn get_class_description(&self, class_code: u8) -> String {
        match class_code {
            0x01 => "USB Audio Device".to_string(),
            0x02 => "USB Communications Device".to_string(),
            0x03 => "USB HID Device".to_string(),
            0x08 => "USB Mass Storage Device".to_string(),
            0x09 => "USB Hub".to_string(),
            0x0E => "USB Video Device".to_string(),
            _ => "USB Device".to_string(),
        }
    }
}

/// HID Device Detection
fn detect_hid_device(descriptor: &UsbDeviceDescriptor) -> bool {
    descriptor.bDeviceClass == 0x00 && descriptor.bDeviceSubClass == 0x00
}

/// MSC Device Detection  
fn detect_msc_device(descriptor: &UsbDeviceDescriptor) -> bool {
    descriptor.bDeviceClass == 0x00 && 
    (descriptor.bDeviceSubClass == 0x06 || descriptor.bDeviceSubClass == 0x02)
}

/// CDC Device Detection
fn detect_cdc_device(descriptor: &UsbDeviceDescriptor) -> bool {
    descriptor.bDeviceClass == 0x02 || descriptor.bDeviceSubClass == 0x02
}

impl Default for UsbClassManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for UsbDeviceTypeDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_class_manager_creation() {
        let manager = UsbClassManager::new();
        assert!(manager.is_initialized());
        assert!(!manager.active_driver_count() > 0);
    }

    #[test]
    fn test_device_type_detection() {
        let detector = UsbDeviceTypeDetector::new();
        
        let descriptor = UsbDeviceDescriptor {
            bLength: 18,
            bDescriptorType: 1,
            bcdUSB: 0x0200,
            bDeviceClass: 0x03,
            bDeviceSubClass: 0x01,
            bDeviceProtocol: 0x01,
            bMaxPacketSize0: 64,
            idVendor: 0x046D,
            idProduct: 0xC52B,
            bcdDevice: 0x0100,
            iManufacturer: 1,
            iProduct: 2,
            iSerialNumber: 3,
            bNumConfigurations: 1,
        };

        let device_type = detector.detect_device_type(&descriptor);
        assert!(device_type.contains("Logitech") || device_type.contains("HID"));
    }

    #[test]
    fn test_driver_creation() {
        let mut manager = UsbClassManager::new();
        
        let result = manager.create_driver(1, UsbClass::HID, 0x00, 0x00);
        assert!(result.is_ok());
        
        let driver_id = result.unwrap();
        assert!(manager.drivers.contains_key(&driver_id));
    }

    #[test]
    fn test_driver_initialization() {
        let mut manager = UsbClassManager::new();
        let driver_id = manager.create_driver(1, UsbClass::HID, 0x00, 0x00).unwrap();
        
        let result = manager.initialize_driver(driver_id);
        assert!(result.is_ok());
    }

    #[test]
    fn test_system_stats() {
        let manager = UsbClassManager::new();
        let stats = manager.get_system_stats();
        
        assert_eq!(stats.total_drivers, 0);
        assert_eq!(stats.active_drivers, 0);
    }
}