//! Unified Device Interface
//! 
//! This module provides a consistent interface for devices across all supported
//! architectures, abstracting away platform-specific details.

use crate::{DeviceClass, ArchitectureType, CompatibilityError};
use core::sync::atomic::{AtomicU32, AtomicUsize, Ordering};
use spin::Mutex;
use bitflags::bitflags;

/// Device identifier type
pub type DeviceId = u32;

/// Unique device ID counter
static NEXT_DEVICE_ID: AtomicU32 = AtomicU32::new(1);

/// Device base class
#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub id: DeviceId,
    pub name: &'static str,
    pub class: DeviceClass,
    pub vendor: &'static str,
    pub model: &'static str,
    pub subsystem: &'static str,
    pub flags: DeviceFlags,
    pub capabilities: DeviceCapabilities,
}

/// Device-specific capabilities
#[derive(Debug, Clone)]
pub struct DeviceCapabilities {
    pub interrupt_support: bool,
    pub dma_support: bool,
    pub hotplug_support: bool,
    pub power_management: bool,
    pub performance_counters: bool,
}

/// Device flags for configuration and status
bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct DeviceFlags: u32 {
        const PRESENT = 0x001;
        const ENABLED = 0x002;
        const POWERED = 0x004;
        const READY = 0x008;
        const ERROR = 0x010;
        const REMOVABLE = 0x020;
        const HOTPLUGGABLE = 0x040;
        const DMA_CAPABLE = 0x080;
        const INTERRUPT_CAPABLE = 0x100;
    }
}

/// Common device operations
pub trait Device: Send + Sync {
    /// Get device information
    fn info(&self) -> &DeviceInfo;
    
    /// Initialize device
    fn init(&mut self) -> Result<(), CompatibilityError>;
    
    /// Shutdown device
    fn shutdown(&mut self) -> Result<(), CompatibilityError>;
    
    /// Reset device to initial state
    fn reset(&mut self) -> Result<(), CompatibilityError>;
    
    /// Enable device
    fn enable(&mut self) -> Result<(), CompatibilityError>;
    
    /// Disable device
    fn disable(&mut self) -> Result<(), CompatibilityError>;
    
    /// Get device status
    fn get_status(&self) -> DeviceStatus;
    
    /// Configure device with specified options
    fn configure(&mut self, config: &DeviceConfig) -> Result<(), CompatibilityError>;
}

/// Device status information
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceStatus {
    NotPresent,
    Present,
    Initializing,
    Ready,
    Suspended,
    Error,
}

/// Device configuration options
#[derive(Debug, Clone)]
pub struct DeviceConfig {
    pub enable_interrupts: bool,
    pub enable_power_mgmt: bool,
    pub dma_enabled: bool,
    pub performance_monitoring: bool,
}

/// Block device interface for storage devices
pub trait BlockDevice: Device {
    /// Read block from device
    fn read_block(&self, block: u64, data: &mut [u8]) -> Result<(), CompatibilityError>;
    
    /// Write block to device
    fn write_block(&self, block: u64, data: &[u8]) -> Result<(), CompatibilityError>;
    
    /// Get total number of blocks
    fn get_block_count(&self) -> u64;
    
    /// Get block size in bytes
    fn get_block_size(&self) -> usize;
    
    /// Get device size in bytes
    fn get_size(&self) -> u64 {
        self.get_block_count() * self.get_block_size() as u64
    }
}

/// Character device interface
pub trait CharacterDevice: Device {
    /// Read from device
    fn read(&self, data: &mut [u8]) -> Result<usize, CompatibilityError>;
    
    /// Write to device
    fn write(&self, data: &[u8]) -> Result<usize, CompatibilityError>;
    
    /// Get available bytes for reading
    fn available(&self) -> usize;
    
    /// Check if device is ready for I/O
    fn ready(&self) -> bool;
}

/// Network device interface
pub trait NetworkDevice: Device {
    /// Send packet
    fn send_packet(&self, packet: &[u8]) -> Result<(), CompatibilityError>;
    
    /// Receive packet (non-blocking)
    fn receive_packet(&self) -> Result<Option<Vec<u8>>, CompatibilityError>;
    
    /// Get MAC address
    fn get_mac_address(&self) -> [u8; 6];
    
    /// Set MAC address
    fn set_mac_address(&self, mac: &[u8; 6]) -> Result<(), CompatibilityError>;
    
    /// Get device state
    fn get_link_state(&self) -> LinkState;
}

/// Network link states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkState {
    Down,
    Up,
    Unknown,
}

/// Graphics device interface
pub trait GraphicsDevice: Device {
    /// Get framebuffer information
    fn get_framebuffer(&self) -> Option<FramebufferInfo>;
    
    /// Set video mode
    fn set_mode(&mut self, width: u32, height: u32, depth: u32) -> Result<(), CompatibilityError>;
    
    /// Clear screen
    fn clear(&self, color: u32) -> Result<(), CompatibilityError>;
    
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

/// Audio device interface
pub trait AudioDevice: Device {
    /// Set volume
    fn set_volume(&mut self, volume: u8) -> Result<(), CompatibilityError>;
    
    /// Play audio buffer
    fn play(&self, buffer: &[i16]) -> Result<(), CompatibilityError>;
    
    /// Record audio
    fn record(&self, buffer: &mut [i16]) -> Result<usize, CompatibilityError>;
    
    /// Get current volume
    fn get_volume(&self) -> Result<u8, CompatibilityError>;
}

/// Input device interface
pub trait InputDevice: Device {
    /// Check if input event is available
    fn has_event(&self) -> bool;
    
    /// Get next input event
    fn get_event(&self) -> Option<InputEvent>;
}

/// Input event types
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InputEventType {
    KeyPress,
    KeyRelease,
    MouseMove,
    MouseButton,
    MouseScroll,
}

/// Input event structure
#[derive(Debug, Clone, Copy)]
pub struct InputEvent {
    pub event_type: InputEventType,
    pub timestamp: u64,
    pub data: [u64; 4],
}

/// USB device interface
pub trait UsbDevice: Device {
    /// Get USB address
    fn get_usb_address(&self) -> u8;
    
    /// Get USB device class
    fn get_device_class(&self) -> u8;
    
    /// Control transfer
    fn control_transfer(&self, request: &UsbRequest, data: &mut [u8]) -> Result<usize, CompatibilityError>;
    
    /// Bulk transfer
    fn bulk_transfer(&self, endpoint: u8, data: &mut [u8]) -> Result<usize, CompatibilityError>;
}

/// USB request structure
#[derive(Debug, Clone, Copy)]
pub struct UsbRequest {
    pub request_type: u8,
    pub request: u8,
    pub value: u16,
    pub index: u16,
    pub length: u16,
}

/// PCI device interface
pub trait PciDevice: Device {
    /// Get PCI configuration space
    fn read_config(&self, offset: u8) -> Result<u32, CompatibilityError>;
    
    /// Write PCI configuration space
    fn write_config(&self, offset: u8, value: u32) -> Result<(), CompatibilityError>;
    
    /// Enable memory space
    fn enable_memory_space(&mut self) -> Result<(), CompatibilityError>;
    
    /// Get bus, device, function numbers
    fn get_bdf(&self) -> (u8, u8, u8);
}

/// Device manager for tracking all devices
pub struct DeviceManager {
    devices: Mutex<Vec<Box<dyn Device>>>,
    device_by_class: spin::Mutex<[spin::Mutex<Vec<Box<dyn Device>>>; 12]>, // 12 device classes
}

impl DeviceManager {
    pub fn new() -> Self {
        let mut class_vecs = Vec::new();
        for _ in 0..12 {
            class_vecs.push(Mutex::new(Vec::new()));
        }
        
        DeviceManager {
            devices: Mutex::new(Vec::new()),
            device_by_class: Mutex::from_array(class_vecs.try_into().unwrap()),
        }
    }
    
    /// Register a new device
    pub fn register_device(&self, device: Box<dyn Device>) -> Result<DeviceId, CompatibilityError> {
        let device_id = NEXT_DEVICE_ID.fetch_add(1, Ordering::SeqCst);
        
        // Update device info with assigned ID
        // In practice, this would be done by the device itself
        // For now, we just add to the registry
        
        {
            let mut devices = self.devices.lock();
            devices.push(device);
        }
        
        // Add to class-specific list
        let class = 0; // Would be determined from device info
        {
            let mut class_devices = self.device_by_class.lock()[class].lock();
            class_devices.push(Box::new(())); // Placeholder
        }
        
        Ok(device_id)
    }
    
    /// Find device by ID
    pub fn find_device(&self, device_id: DeviceId) -> Option<&dyn Device> {
        let devices = self.devices.lock();
        
        // Find device by ID
        // In practice, this would use a hashmap or other efficient lookup
        for device in devices.iter() {
            // Would check device.info().id == device_id
            // For now, return first device
            return Some(device.as_ref());
        }
        None
    }
    
    /// Find devices by class
    pub fn find_devices_by_class(&self, class: DeviceClass) -> Vec<&dyn Device> {
        let class_idx = class as usize;
        let class_devices = self.device_by_class.lock()[class_idx].lock();
        class_devices.iter().map(|d| d.as_ref()).collect()
    }
}

/// Global device manager instance
static DEVICE_MANAGER: spin::Mutex<Option<DeviceManager>> = spin::Mutex::new(None);

/// Initialize device abstraction layer
pub fn init() -> Result<(), CompatibilityError> {
    let mut manager_lock = DEVICE_MANAGER.lock();
    
    if manager_lock.is_some() {
        return Ok(());
    }
    
    *manager_lock = Some(DeviceManager::new());
    
    // Perform architecture-specific device initialization
    arch_init_devices()?;
    
    Ok(())
}

/// Architecture-specific device initialization
fn arch_init_devices() -> Result<(), CompatibilityError> {
    // This would detect and initialize platform-specific devices
    // For now, we'll create placeholder devices
    
    Ok(())
}

/// Get global device manager
pub fn get_device_manager() -> Option<&'static DeviceManager> {
    DEVICE_MANAGER.lock().as_ref()
}

/// Helper function to register a device
pub fn register_device(device: Box<dyn Device>) -> Result<DeviceId, CompatibilityError> {
    let manager = get_device_manager()
        .ok_or(CompatibilityError::InitializationFailed("Device manager not initialized"))?;
    
    manager.register_device(device)
}