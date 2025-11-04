//! Hardware Bus Interfaces
//! 
//! Provides support for various hardware buses (PCI, USB, I2C, SPI)
//! with automatic device detection and enumeration capabilities.

use crate::{DeviceType, DriverResult, DriverError};
use crate::device::{DeviceInfo, HardwareAddress, BusHandle, BusType};
use spin::Mutex;
use alloc::vec::Vec;
use log::{info, warn, error};

/// Bus descriptor
#[derive(Debug)]
pub struct Bus {
    pub handle: BusHandle,
    pub name: &'static str,
    pub description: &'static str,
    pub devices_found: usize,
    pub driver: Option<&'static dyn BusDriver>,
}

/// Bus driver interface
pub trait BusDriver: Send + Sync {
    /// Get the bus type
    fn bus_type(&self) -> BusType;
    
    /// Initialize the bus
    fn init(&self, bus_handle: BusHandle) -> DriverResult<()>;
    
    /// Enumerate devices on the bus
    fn enumerate_devices(&self, bus_handle: BusHandle) -> DriverResult<Vec<DeviceInfo>>;
    
    /// Enable/disable a device on the bus
    fn enable_device(&self, bus_handle: BusHandle, device_addr: u32) -> DriverResult<()>;
    
    /// Disable a device on the bus
    fn disable_device(&self, bus_handle: BusHandle, device_addr: u32) -> DriverResult<()>;
    
    /// Read configuration space
    fn read_config(&self, bus_handle: BusHandle, device_addr: u32, offset: u16, size: usize) -> DriverResult<u32>;
    
    /// Write configuration space
    fn write_config(&self, bus_handle: BusHandle, device_addr: u32, offset: u16, value: u32) -> DriverResult<()>;
}

/// PCI Bus Driver
pub struct PciBusDriver {
    pub base_address: Option<u64>,
    pub configuration_space_size: usize,
}

impl PciBusDriver {
    /// Create a new PCI bus driver
    pub fn new() -> Self {
        Self {
            base_address: None,
            configuration_space_size: 256,
        }
    }

    /// Scan PCI bus for devices
    pub fn scan_bus(&self, bus_id: u8) -> DriverResult<Vec<PciDeviceInfo>> {
        info!("Scanning PCI bus {}", bus_id);
        let mut devices = Vec::new();
        
        // Simulate PCI device enumeration
        // In real implementation, this would scan actual PCI configuration space
        
        // Common devices to simulate
        let common_devices = [
            // Device 00:00.0 - Host bridge
            (0x00, 0x00, 0x00, 0x8086, 0x1234, "Host Bridge"),
            // Device 00:1f.0 - ISA bridge  
            (0x00, 0x1f, 0x00, 0x8086, 0x7000, "ISA Bridge"),
            // Device 00:1f.2 - SATA controller
            (0x00, 0x1f, 0x02, 0x8086, 0x2922, "AHCI SATA Controller"),
            // Device 00:1f.3 - SMBus
            (0x00, 0x1f, 0x03, 0x8086, 0x2930, "SMBus Controller"),
        ];
        
        for (bus, dev, func, vendor_id, device_id, description) in &common_devices {
            let pci_info = PciDeviceInfo {
                bus: *bus,
                device: *dev,
                function: *func,
                vendor_id: *vendor_id,
                device_id: *device_id,
                class_code: self.get_device_class(*vendor_id, *device_id),
                interrupt_line: Some(self.get_interrupt_line(*bus, *dev)),
                base_addresses: self.get_base_addresses(*vendor_id, *device_id),
                description: *description,
            };
            
            info!("  Found PCI device: {} ({:04x}:{:04x})", description, vendor_id, device_id);
            devices.push(pci_info);
        }
        
        Ok(devices)
    }

    /// Get device class code based on vendor/device IDs
    fn get_device_class(&self, vendor_id: u16, device_id: u16) -> u32 {
        match (vendor_id, device_id) {
            (0x8086, 0x1234) => 0x060000, // Host bridge
            (0x8086, 0x7000) => 0x060100, // ISA bridge
            (0x8086, 0x2922) => 0x010601, // SATA controller
            (0x8086, 0x2930) => 0x0c0500, // SMBus
            _ => 0x000000, // Unknown
        }
    }

    /// Get interrupt line for device
    fn get_interrupt_line(&self, bus: u8, device: u8) -> u8 {
        match (bus, device) {
            (0x00, 0x1f) => 0, // Legacy ISA interrupts
            _ => device % 16,  // Simplified interrupt assignment
        }
    }

    /// Get base addresses for device
    fn get_base_addresses(&self, vendor_id: u16, device_id: u16) -> Vec<u64> {
        match (vendor_id, device_id) {
            (0x8086, 0x2922) => vec![0xfebf1000], // SATA controller base address
            (0x8086, 0x2930) => vec![0xfebf0000], // SMBus base address
            _ => Vec::new(),
        }
    }
}

impl BusDriver for PciBusDriver {
    fn bus_type(&self) -> BusType {
        BusType::Pci
    }

    fn init(&self, bus_handle: BusHandle) -> DriverResult<()> {
        info!("Initializing PCI bus driver for bus {}", bus_handle.bus_id);
        Ok(())
    }

    fn enumerate_devices(&self, bus_handle: BusHandle) -> DriverResult<Vec<DeviceInfo>> {
        let pci_devices = self.scan_bus(bus_handle.bus_id)?;
        
        let mut devices = Vec::new();
        for pci_device in pci_devices {
            let device_info = DeviceInfo {
                id: crate::device::Device::allocate_id(),
                name: pci_device.description,
                device_type: self.map_pci_class_to_device_type(pci_device.class_code),
                vendor_id: Some(pci_device.vendor_id),
                product_id: Some(pci_device.device_id),
                capabilities: crate::device::DeviceCapabilities::READ | crate::device::DeviceCapabilities::WRITE,
                state: crate::device::DeviceState::Uninitialized,
                is_hot_plug: false,
                parent_bus: Some(bus_handle),
            };
            devices.push(device_info);
        }
        
        Ok(devices)
    }

    fn enable_device(&self, bus_handle: BusHandle, device_addr: u32) -> DriverResult<()> {
        info!("Enabling PCI device {} on bus {}", device_addr, bus_handle.bus_id);
        Ok(())
    }

    fn disable_device(&self, bus_handle: BusHandle, device_addr: u32) -> DriverResult<()> {
        info!("Disabling PCI device {} on bus {}", device_addr, bus_handle.bus_id);
        Ok(())
    }

    fn read_config(&self, bus_handle: BusHandle, device_addr: u32, offset: u16, size: usize) -> DriverResult<u32> {
        // Simplified PCI configuration space read
        info!("Reading PCI config space: bus {}, device {}, offset 0x{:04x}", 
              bus_handle.bus_id, device_addr, offset);
        Ok(0x00000000)
    }

    fn write_config(&self, bus_handle: BusHandle, device_addr: u32, offset: u16, value: u32) -> DriverResult<()> {
        // Simplified PCI configuration space write
        info!("Writing PCI config space: bus {}, device {}, offset 0x{:04x}, value 0x{:08x}", 
              bus_handle.bus_id, device_addr, offset, value);
        Ok(())
    }
}

impl Default for PciBusDriver {
    fn default() -> Self {
        Self::new()
    }
}

/// USB Bus Driver
pub struct UsbBusDriver {
    pub bus_number: u8,
    pub root_hub_ports: u8,
}

impl UsbBusDriver {
    /// Create a new USB bus driver
    pub fn new(bus_number: u8) -> Self {
        Self {
            bus_number,
            root_hub_ports: 8,
        }
    }

    /// Scan USB bus for devices
    pub fn scan_bus(&self) -> DriverResult<Vec<UsbDeviceInfo>> {
        info!("Scanning USB bus {}", self.bus_number);
        let mut devices = Vec::new();
        
        // Simulate USB device enumeration
        let usb_devices = [
            (2, 0x046d, 0xc31c, "USB Keyboard"),
            (3, 0x045e, 0x0745, "USB Mouse"),
        ];
        
        for (port, vendor_id, product_id, description) in &usb_devices {
            let usb_info = UsbDeviceInfo {
                port: *port,
                speed: UsbSpeed::High,
                device_class: 0x00, // Interface-specific
                vendor_id: *vendor_id,
                product_id: *product_id,
                description: *description,
            };
            
            info!("  Found USB device: {} (port {}, {:04x}:{:04x})", 
                  description, port, vendor_id, product_id);
            devices.push(usb_info);
        }
        
        Ok(devices)
    }

    /// Detect USB device type
    pub fn detect_usb_device_type(&self, vendor_id: u16, product_id: u16) -> DeviceType {
        match (vendor_id, product_id) {
            (0x046d, _) => DeviceType::Mouse,  // Logitech
            (0x045e, _) => DeviceType::Keyboard, // Microsoft
            (0x0a5c, _) => DeviceType::Network, // Broadcom Bluetooth
            _ => DeviceType::USB,
        }
    }
}

impl BusDriver for UsbBusDriver {
    fn bus_type(&self) -> BusType {
        BusType::Usb
    }

    fn init(&self, bus_handle: BusHandle) -> DriverResult<()> {
        info!("Initializing USB bus driver for bus {}", bus_handle.bus_id);
        Ok(())
    }

    fn enumerate_devices(&self, bus_handle: BusHandle) -> DriverResult<Vec<DeviceInfo>> {
        let usb_devices = self.scan_bus()?;
        
        let mut devices = Vec::new();
        for usb_device in usb_devices {
            let device_info = DeviceInfo {
                id: crate::device::Device::allocate_id(),
                name: usb_device.description,
                device_type: self.detect_usb_device_type(usb_device.vendor_id, usb_device.product_id),
                vendor_id: Some(usb_device.vendor_id),
                product_id: Some(usb_device.product_id),
                capabilities: crate::device::DeviceCapabilities::READ | crate::device::DeviceCapabilities::INTERRUPT | crate::device::DeviceCapabilities::HOT_PLUG,
                state: crate::device::DeviceState::Uninitialized,
                is_hot_plug: true,
                parent_bus: Some(bus_handle),
            };
            devices.push(device_info);
        }
        
        Ok(devices)
    }

    fn enable_device(&self, bus_handle: BusHandle, device_addr: u32) -> DriverResult<()> {
        info!("Enabling USB device {} on bus {}", device_addr, bus_handle.bus_id);
        Ok(())
    }

    fn disable_device(&self, bus_handle: BusHandle, device_addr: u32) -> DriverResult<()> {
        info!("Disabling USB device {} on bus {}", device_addr, bus_handle.bus_id);
        Ok(())
    }

    fn read_config(&self, bus_handle: BusHandle, device_addr: u32, offset: u16, size: usize) -> DriverResult<u32> {
        warn!("USB bus read_config not implemented");
        Ok(0)
    }

    fn write_config(&self, bus_handle: BusHandle, device_addr: u32, offset: u16, value: u32) -> DriverResult<()> {
        warn!("USB bus write_config not implemented");
        Ok(())
    }
}

/// I2C Bus Driver
pub struct I2cBusDriver {
    pub bus_id: u8,
    pub scl_pin: u8,
    pub sda_pin: u8,
}

impl I2cBusDriver {
    /// Create a new I2C bus driver
    pub fn new(bus_id: u8, scl_pin: u8, sda_pin: u8) -> Self {
        Self {
            bus_id,
            scl_pin,
            sda_pin,
        }
    }

    /// Scan I2C bus for devices
    pub fn scan_bus(&self) -> DriverResult<Vec<I2cDeviceInfo>> {
        info!("Scanning I2C bus {} (SCL: {}, SDA: {})", self.bus_id, self.scl_pin, self.sda_pin);
        let mut devices = Vec::new();
        
        // Common I2C addresses to scan
        let common_addresses = [0x50, 0x51, 0x68, 0x69]; // EEPROM, RTC, etc.
        
        for &address in &common_addresses {
            if self.detect_device(address) {
                let device_info = I2cDeviceInfo {
                    address,
                    device_type: self.get_device_type(address),
                    description: self.get_device_description(address),
                };
                
                info!("  Found I2C device: {} at address 0x{:02x}", 
                      device_info.description, address);
                devices.push(device_info);
            }
        }
        
        Ok(devices)
    }

    /// Detect if device responds at address
    fn detect_device(&self, address: u8) -> bool {
        // Simplified detection - always detect for common addresses
        matches!(address, 0x50 | 0x51 | 0x68 | 0x69)
    }

    /// Get device type from I2C address
    fn get_device_type(&self, address: u8) -> &'static str {
        match address {
            0x50 | 0x51 => "EEPROM",
            0x68 => "RTC",
            0x69 => "Temperature Sensor",
            _ => "Unknown I2C Device",
        }
    }

    /// Get device description
    fn get_device_description(&self, address: u8) -> &'static str {
        match address {
            0x50 => "24C32 EEPROM",
            0x51 => "24C64 EEPROM", 
            0x68 => "DS3231 RTC",
            0x69 => "TMP102 Temperature Sensor",
            _ => "I2C Device",
        }
    }
}

impl BusDriver for I2cBusDriver {
    fn bus_type(&self) -> BusType {
        BusType::I2c
    }

    fn init(&self, bus_handle: BusHandle) -> DriverResult<()> {
        info!("Initializing I2C bus driver for bus {}", bus_handle.bus_id);
        Ok(())
    }

    fn enumerate_devices(&self, bus_handle: BusHandle) -> DriverResult<Vec<DeviceInfo>> {
        let i2c_devices = self.scan_bus()?;
        
        let mut devices = Vec::new();
        for i2c_device in i2c_devices {
            let device_info = DeviceInfo {
                id: crate::device::Device::allocate_id(),
                name: i2c_device.description,
                device_type: DeviceType::Unknown, // I2C devices need specific drivers
                vendor_id: None,
                product_id: Some(i2c_device.address as u16),
                capabilities: crate::device::DeviceCapabilities::READ | crate::device::DeviceCapabilities::WRITE,
                state: crate::device::DeviceState::Uninitialized,
                is_hot_plug: false,
                parent_bus: Some(bus_handle),
            };
            devices.push(device_info);
        }
        
        Ok(devices)
    }

    fn enable_device(&self, bus_handle: BusHandle, device_addr: u32) -> DriverResult<()> {
        info!("Enabling I2C device {} on bus {}", device_addr, bus_handle.bus_id);
        Ok(())
    }

    fn disable_device(&self, bus_handle: BusHandle, device_addr: u32) -> DriverResult<()> {
        info!("Disabling I2C device {} on bus {}", device_addr, bus_handle.bus_id);
        Ok(())
    }

    fn read_config(&self, bus_handle: BusHandle, device_addr: u32, offset: u16, size: usize) -> DriverResult<u32> {
        warn!("I2C bus read_config not applicable");
        Ok(0)
    }

    fn write_config(&self, bus_handle: BusHandle, device_addr: u32, offset: u16, value: u32) -> DriverResult<()> {
        warn!("I2C bus write_config not applicable");
        Ok(())
    }
}

/// Platform Bus Driver (for ARM/RISC-V)
pub struct PlatformBusDriver {
    pub bus_name: &'static str,
    pub compatible: &'static str,
}

impl PlatformBusDriver {
    /// Create a new platform bus driver
    pub fn new(bus_name: &'static str, compatible: &'static str) -> Self {
        Self {
            bus_name,
            compatible,
        }
    }

    /// Scan platform bus for devices
    pub fn scan_bus(&self) -> DriverResult<Vec<PlatformDeviceInfo>> {
        info!("Scanning platform bus: {}", self.bus_name);
        let mut devices = Vec::new();
        
        // Simulate platform device discovery based on compatible property
        match self.compatible {
            "arm,gic-v3" => {
                devices.push(PlatformDeviceInfo {
                    name: "ARM GIC v3",
                    compatible: "arm,gic-v3",
                    base_address: 0x2c000000,
                    size: 0x100000,
                    interrupts: vec![25, 26, 27],
                });
            }
            "ns16550a" => {
                devices.push(PlatformDeviceInfo {
                    name: "16550 UART",
                    compatible: "ns16550a",
                    base_address: 0x3f8,
                    size: 0x8,
                    interrupts: vec![4],
                });
            }
            _ => {}
        }
        
        for device in &devices {
            info!("  Found platform device: {} at 0x{:08x}", device.name, device.base_address);
        }
        
        Ok(devices)
    }
}

impl BusDriver for PlatformBusDriver {
    fn bus_type(&self) -> BusType {
        BusType::Platform
    }

    fn init(&self, bus_handle: BusHandle) -> DriverResult<()> {
        info!("Initializing platform bus driver for {}", self.bus_name);
        Ok(())
    }

    fn enumerate_devices(&self, bus_handle: BusHandle) -> DriverResult<Vec<DeviceInfo>> {
        let platform_devices = self.scan_bus()?;
        
        let mut devices = Vec::new();
        for platform_device in platform_devices {
            let device_type = self.map_compatible_to_device_type(platform_device.compatible);
            
            let device_info = DeviceInfo {
                id: crate::device::Device::allocate_id(),
                name: platform_device.name,
                device_type,
                vendor_id: None,
                product_id: None,
                capabilities: crate::device::DeviceCapabilities::READ | crate::device::DeviceCapabilities::WRITE | crate::device::DeviceCapabilities::INTERRUPT,
                state: crate::device::DeviceState::Uninitialized,
                is_hot_plug: false,
                parent_bus: Some(bus_handle),
            };
            devices.push(device_info);
        }
        
        Ok(devices)
    }

    fn enable_device(&self, bus_handle: BusHandle, device_addr: u32) -> DriverResult<()> {
        info!("Enabling platform device {} on bus {}", device_addr, bus_handle.bus_id);
        Ok(())
    }

    fn disable_device(&self, bus_handle: BusHandle, device_addr: u32) -> DriverResult<()> {
        info!("Disabling platform device {} on bus {}", device_addr, bus_handle.bus_id);
        Ok(())
    }

    fn read_config(&self, bus_handle: BusHandle, device_addr: u32, offset: u16, size: usize) -> DriverResult<u32> {
        warn!("Platform bus read_config not applicable");
        Ok(0)
    }

    fn write_config(&self, bus_handle: BusHandle, device_addr: u32, offset: u16, value: u32) -> DriverResult<()> {
        warn!("Platform bus write_config not applicable");
        Ok(())
    }
}

// Supporting structures
#[derive(Debug)]
pub struct PciDeviceInfo {
    pub bus: u8,
    pub device: u8,
    pub function: u8,
    pub vendor_id: u16,
    pub device_id: u16,
    pub class_code: u32,
    pub interrupt_line: Option<u8>,
    pub base_addresses: Vec<u64>,
    pub description: &'static str,
}

#[derive(Debug)]
pub struct UsbDeviceInfo {
    pub port: u8,
    pub speed: UsbSpeed,
    pub device_class: u8,
    pub vendor_id: u16,
    pub product_id: u16,
    pub description: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub enum UsbSpeed {
    Low,
    Full,
    High,
    Super,
    SuperPlus,
}

#[derive(Debug)]
pub struct I2cDeviceInfo {
    pub address: u8,
    pub device_type: &'static str,
    pub description: &'static str,
}

#[derive(Debug)]
pub struct PlatformDeviceInfo {
    pub name: &'static str,
    pub compatible: &'static str,
    pub base_address: u64,
    pub size: usize,
    pub interrupts: Vec<u32>,
}

// Extension methods for BusDriver
impl PciBusDriver {
    fn map_pci_class_to_device_type(&self, class_code: u32) -> DeviceType {
        match class_code {
            0x0c0000 => DeviceType::USB,     // Serial bus controller - USB
            0x020000 => DeviceType::Network, // Network controller - Ethernet
            0x030000 => DeviceType::Display, // Display controller
            0x040000 => DeviceType::Audio,   // Multimedia controller
            0x010601 => DeviceType::Storage, // Mass storage controller - SATA
            0x010400 => DeviceType::Storage, // Mass storage controller - RAID
            0x060000 => DeviceType::Unknown, // Bridge device
            _ => DeviceType::Unknown,
        }
    }
}

impl PlatformBusDriver {
    fn map_compatible_to_device_type(&self, compatible: &str) -> DeviceType {
        match compatible {
            "ns16550a" => DeviceType::UART,
            "arm,gic-v3" => DeviceType::Unknown, // Interrupt controller
            "arm,pl011" => DeviceType::UART,
            "qemu,platform-device" => DeviceType::Unknown,
            _ => DeviceType::Unknown,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pci_bus_driver_creation() {
        let driver = PciBusDriver::new();
        assert_eq!(driver.bus_type(), BusType::Pci);
        assert_eq!(driver.configuration_space_size, 256);
    }

    #[test]
    fn test_pci_device_scanning() {
        let driver = PciBusDriver::new();
        let devices = driver.scan_bus(0).unwrap();
        
        assert!(!devices.is_empty());
        
        // Check for expected devices
        let device_descriptions: Vec<&str> = devices.iter().map(|d| d.description).collect();
        assert!(device_descriptions.contains(&"Host Bridge"));
        assert!(device_descriptions.contains(&"ISA Bridge"));
    }

    #[test]
    fn test_usb_bus_driver_creation() {
        let driver = UsbBusDriver::new(1);
        assert_eq!(driver.bus_type(), BusType::Usb);
        assert_eq!(driver.bus_number, 1);
        assert_eq!(driver.root_hub_ports, 8);
    }

    #[test]
    fn test_i2c_bus_driver_creation() {
        let driver = I2cBusDriver::new(0, 18, 19);
        assert_eq!(driver.bus_type(), BusType::I2c);
        assert_eq!(driver.bus_id, 0);
        assert_eq!(driver.scl_pin, 18);
        assert_eq!(driver.sda_pin, 19);
    }

    #[test]
    fn test_platform_bus_driver_creation() {
        let driver = PlatformBusDriver::new("ARM Platform", "arm,gic-v3");
        assert_eq!(driver.bus_type(), BusType::Platform);
        assert_eq!(driver.bus_name, "ARM Platform");
        assert_eq!(driver.compatible, "arm,gic-v3");
    }

    #[test]
    fn test_device_type_mapping() {
        let pci_driver = PciBusDriver::new();
        
        assert_eq!(pci_driver.map_pci_class_to_device_type(0x0c0000), DeviceType::USB);
        assert_eq!(pci_driver.map_pci_class_to_device_type(0x020000), DeviceType::Network);
        assert_eq!(pci_driver.map_pci_class_to_device_type(0x030000), DeviceType::Display);
        assert_eq!(pci_driver.map_pci_class_to_device_type(0x010601), DeviceType::Storage);
    }

    #[test]
    fn test_usb_device_detection() {
        let driver = UsbBusDriver::new(1);
        
        assert_eq!(driver.detect_usb_device_type(0x046d, 0xc31c), DeviceType::Mouse);
        assert_eq!(driver.detect_usb_device_type(0x045e, 0x0745), DeviceType::Keyboard);
        assert_eq!(driver.detect_usb_device_type(0x0a5c, 0x2045), DeviceType::Network);
    }
}