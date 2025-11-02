//! Cross-Platform Driver Interface
//! 
//! This module provides a unified interface for drivers across different
//! architectures, ensuring compatibility and ease of development.

use crate::{DeviceId, DeviceClass, ArchitectureType, CompatibilityError};
use spin::Mutex;
use bitflags::bitflags;

/// Driver manager type
pub type DriverId = u32;

/// Driver state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriverState {
    Uninitialized,
    Initializing,
    Running,
    Suspended,
    Stopped,
    Error,
}

/// Driver capabilities
bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct DriverCapabilities: u32 {
        const HOTPLUG = 0x001;
        const POWER_MANAGEMENT = 0x002;
        const THREAD_SAFE = 0x004;
        const DMA = 0x008;
        const INTERRUPTS = 0x010;
        const PERFORMANCE_MONITORING = 0x020;
        const CONFIGURATION = 0x040;
    }
}

/// Base driver trait
pub trait Driver: Send + Sync {
    /// Get driver information
    fn get_info(&self) -> DriverInfo;
    
    /// Initialize driver
    fn init(&mut self) -> Result<(), CompatibilityError>;
    
    /// Start driver operation
    fn start(&mut self) -> Result<(), CompatibilityError>;
    
    /// Stop driver operation
    fn stop(&mut self) -> Result<(), CompatibilityError>;
    
    /// Get current driver state
    fn get_state(&self) -> DriverState;
    
    /// Configure driver
    fn configure(&mut self, config: &DriverConfig) -> Result<(), CompatibilityError>;
    
    /// Get driver capabilities
    fn get_capabilities(&self) -> DriverCapabilities;
}

/// Driver information structure
#[derive(Debug, Clone)]
pub struct DriverInfo {
    pub id: DriverId,
    pub name: &'static str,
    pub version: &'static str,
    pub author: &'static str,
    pub description: &'static str,
    pub supported_devices: Vec<DeviceClass>,
    pub supported_architectures: Vec<ArchitectureType>,
    pub capabilities: DriverCapabilities,
}

/// Driver configuration
#[derive(Debug, Clone)]
pub struct DriverConfig {
    pub enable_hotplug: bool,
    pub enable_power_management: bool,
    pub enable_interrupts: bool,
    pub enable_performance_monitoring: bool,
    pub debug_level: u8,
    pub custom_options: Vec<(String, String)>,
}

/// Character driver interface
pub trait CharacterDriver: Driver {
    /// Read data from device
    fn read(&self, data: &mut [u8]) -> Result<usize, CompatibilityError>;
    
    /// Write data to device
    fn write(&self, data: &[u8]) -> Result<usize, CompatibilityError>;
    
    /// Check if data is available for reading
    fn is_readable(&self) -> bool;
    
    /// Check if device can accept writes
    fn is_writable(&self) -> bool;
    
    /// Handle ioctl command
    fn ioctl(&self, command: u32, arg: usize) -> Result<usize, CompatibilityError>;
}

/// Block driver interface
pub trait BlockDriver: Driver {
    /// Read block from device
    fn read_block(&self, block: u64, data: &mut [u8]) -> Result<(), CompatibilityError>;
    
    /// Write block to device
    fn write_block(&self, block: u64, data: &[u8]) -> Result<(), CompatibilityError>;
    
    /// Get device geometry
    fn get_geometry(&self) -> BlockDeviceGeometry;
    
    /// Flush buffers to device
    fn flush(&self) -> Result<(), CompatibilityError>;
}

/// Block device geometry information
#[derive(Debug, Clone, Copy)]
pub struct BlockDeviceGeometry {
    pub sectors: u64,
    pub sector_size: u32,
    pub heads: u16,
    pub tracks_per_cylinder: u16,
}

/// Network driver interface
pub trait NetworkDriver: Driver {
    /// Send packet
    fn send_packet(&self, packet: &[u8]) -> Result<(), CompatibilityError>;
    
    /// Receive packet (non-blocking)
    fn receive_packet(&self) -> Result<Option<Vec<u8>>, CompatibilityError>;
    
    /// Get MAC address
    fn get_mac_address(&self) -> Result<[u8; 6], CompatibilityError>;
    
    /// Set MAC address
    fn set_mac_address(&self, mac: &[u8; 6]) -> Result<(), CompatibilityError>;
    
    /// Get link status
    fn get_link_status(&self) -> Result<NetworkLinkStatus, CompatibilityError>;
    
    /// Enable/disable interface
    fn set_interface_enabled(&self, enabled: bool) -> Result<(), CompatibilityError>;
}

/// Network link status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkLinkStatus {
    Down,
    Up,
    Unknown,
}

/// Audio driver interface
pub trait AudioDriver: Driver {
    /// Set audio format
    fn set_format(&mut self, sample_rate: u32, channels: u8, bits_per_sample: u8) -> Result<(), CompatibilityError>;
    
    /// Play audio buffer
    fn play_buffer(&self, buffer: &[i16]) -> Result<(), CompatibilityError>;
    
    /// Record audio buffer
    fn record_buffer(&self, buffer: &mut [i16]) -> Result<usize, CompatibilityError>;
    
    /// Set volume
    fn set_volume(&mut self, volume: u8) -> Result<(), CompatibilityError>;
    
    /// Get volume
    fn get_volume(&self) -> Result<u8, CompatibilityError>;
}

/// Graphics driver interface
pub trait GraphicsDriver: Driver {
    /// Get framebuffer information
    fn get_framebuffer(&self) -> Result<FramebufferInfo, CompatibilityError>;
    
    /// Set video mode
    fn set_mode(&mut self, width: u32, height: u32, depth: u32) -> Result<(), CompatibilityError>;
    
    /// Clear screen
    fn clear(&self, color: u32) -> Result<(), CompatibilityError>;
    
    /// Draw pixel
    fn draw_pixel(&self, x: u32, y: u32, color: u32) -> Result<(), CompatibilityError>;
    
    /// Present framebuffer
    fn present(&self) -> Result<(), CompatibilityError>;
}

/// Framebuffer information
#[derive(Debug, Clone)]
pub struct FramebufferInfo {
    pub address: *mut u8,
    pub width: u32,
    pub height: u32,
    pub pitch: u32,
    pub bits_per_pixel: u32,
    pub red_mask: u32,
    pub green_mask: u32,
    pub blue_mask: u32,
    pub alpha_mask: u32,
}

/// USB driver interface
pub trait UsbDriver: Driver {
    /// Get USB address
    fn get_usb_address(&self) -> Result<u8, CompatibilityError>;
    
    /// Control transfer
    fn control_transfer(&self, request: &UsbControlRequest, data: &mut [u8]) -> Result<usize, CompatibilityError>;
    
    /// Bulk transfer
    fn bulk_transfer(&self, endpoint: u8, data: &mut [u8]) -> Result<usize, CompatibilityError>;
    
    /// Interrupt transfer
    fn interrupt_transfer(&self, endpoint: u8, data: &mut [u8]) -> Result<usize, CompatibilityError>;
}

/// USB control request
#[derive(Debug, Clone, Copy)]
pub struct UsbControlRequest {
    pub request_type: u8,
    pub request: u8,
    pub value: u16,
    pub index: u16,
    pub length: u16,
}

/// PCI driver interface
pub trait PciDriver: Driver {
    /// Read PCI configuration
    fn read_config(&self, offset: u8) -> Result<u32, CompatibilityError>;
    
    /// Write PCI configuration
    fn write_config(&self, offset: u8, value: u32) -> Result<(), CompatibilityError>;
    
    /// Get device BDF (Bus, Device, Function)
    fn get_bdf(&self) -> Result<(u8, u8, u8), CompatibilityError>;
    
    /// Enable memory space access
    fn enable_memory_space(&mut self) -> Result<(), CompatibilityError>;
    
    /// Enable I/O space access
    fn enable_io_space(&mut self) -> Result<(), CompatibilityError>;
}

/// Driver manager
pub struct DriverManager {
    drivers: Mutex<Vec<Box<dyn Driver>>>,
    drivers_by_class: spin::Mutex<[spin::Mutex<Vec<Box<dyn Driver>>>; 12]>,
}

impl DriverManager {
    pub fn new() -> Self {
        let mut drivers_by_class = Vec::new();
        for _ in 0..12 {
            drivers_by_class.push(Mutex::new(Vec::new()));
        }
        
        DriverManager {
            drivers: Mutex::new(Vec::new()),
            drivers_by_class: Mutex::from_array(drivers_by_class.try_into().unwrap()),
        }
    }
    
    /// Register a driver
    pub fn register_driver(&self, mut driver: Box<dyn Driver>) -> Result<DriverId, CompatibilityError> {
        // Check if driver is compatible with current architecture
        let arch_type = crate::get_state()
            .map(|s| s.arch_type)
            .ok_or(CompatibilityError::InitializationFailed("Compatibility state not initialized"))?;
        
        if !driver.get_info().supported_architectures.contains(&arch_type) {
            return Err(CompatibilityError::DriverNotCompatible);
        }
        
        let driver_id = driver.get_info().id;
        
        // Register in global list
        {
            let mut drivers = self.drivers.lock();
            drivers.push(driver.clone());
        }
        
        // Register in class-specific lists
        for device_class in driver.get_info().supported_devices.iter() {
            let class_idx = *device_class as usize;
            let mut class_drivers = self.drivers_by_class.lock()[class_idx].lock();
            class_drivers.push(driver.clone());
        }
        
        Ok(driver_id)
    }
    
    /// Find driver by ID
    pub fn find_driver(&self, driver_id: DriverId) -> Option<&dyn Driver> {
        let drivers = self.drivers.lock();
        for driver in drivers.iter() {
            if driver.get_info().id == driver_id {
                return Some(driver.as_ref());
            }
        }
        None
    }
    
    /// Find drivers for device class
    pub fn find_drivers_for_class(&self, device_class: DeviceClass) -> Vec<&dyn Driver> {
        let class_idx = device_class as usize;
        let class_drivers = self.drivers_by_class.lock()[class_idx].lock();
        class_drivers.iter().map(|d| d.as_ref()).collect()
    }
    
    /// Initialize all registered drivers
    pub fn init_all_drivers(&self) -> Result<(), CompatibilityError> {
        let drivers = self.drivers.lock();
        for driver in drivers.iter() {
            // Clone Arc to get mutable reference for initialization
            // In practice, this would use Rc<RefCell<Driver>> or similar
        }
        Ok(())
    }
    
    /// Start all drivers
    pub fn start_all_drivers(&self) -> Result<(), CompatibilityError> {
        let drivers = self.drivers.lock();
        for driver in drivers.iter() {
            // Start all drivers
        }
        Ok(())
    }
    
    /// Stop all drivers
    pub fn stop_all_drivers(&self) -> Result<(), CompatibilityError> {
        let drivers = self.drivers.lock();
        for driver in drivers.iter() {
            // Stop all drivers
        }
        Ok(())
    }
}

/// Global driver manager
static DRIVER_MANAGER: spin::Mutex<Option<DriverManager>> = spin::Mutex::new(None);

/// Initialize driver interface
pub fn init() -> Result<(), CompatibilityError> {
    let mut manager_lock = DRIVER_MANAGER.lock();
    
    if manager_lock.is_some() {
        return Ok(());
    }
    
    *manager_lock = Some(DriverManager::new());
    
    // Load built-in drivers
    load_builtin_drivers()?;
    
    Ok(())
}

/// Load architecture-specific built-in drivers
fn load_builtin_drivers() -> Result<(), CompatibilityError> {
    let manager = DRIVER_MANAGER.lock();
    let manager_ref = manager.as_ref()
        .ok_or(CompatibilityError::InitializationFailed("Driver manager not initialized"))?;
    
    // This would load platform-specific drivers
    // For now, we'll just initialize with basic drivers
    
    Ok(())
}

/// Register a driver
pub fn register_driver(driver: Box<dyn Driver>) -> Result<DriverId, CompatibilityError> {
    let manager = DRIVER_MANAGER.lock();
    let manager_ref = manager.as_ref()
        .ok_or(CompatibilityError::InitializationFailed("Driver manager not initialized"))?;
    
    manager_ref.register_driver(driver)
}

/// Get driver manager
pub fn get_driver_manager() -> Option<&'static DriverManager> {
    DRIVER_MANAGER.lock().as_ref()
}

/// Find driver by ID
pub fn find_driver(driver_id: DriverId) -> Option<&dyn Driver> {
    let manager = DRIVER_MANAGER.lock();
    let manager_ref = manager.as_ref()?;
    manager_ref.find_driver(driver_id)
}

/// Find drivers for device class
pub fn find_drivers_for_class(device_class: DeviceClass) -> Vec<&dyn Driver> {
    let manager = DRIVER_MANAGER.lock();
    let manager_ref = match manager.as_ref() {
        Some(manager) => manager,
        None => return Vec::new(),
    };
    manager_ref.find_drivers_for_class(device_class)
}