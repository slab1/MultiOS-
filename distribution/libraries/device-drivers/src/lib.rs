//! MultiOS Device Driver Framework
//! 
//! This module provides a unified framework for device drivers in the
//! MultiOS hybrid microkernel architecture, supporting various hardware platforms.

#![no_std]

use spin::Mutex;
use bitflags::bitflags;

pub mod bus;
pub mod device;
pub mod driver_manager;
pub mod serial;
pub mod timer;
pub mod keyboard;
pub mod graphics;
pub mod storage;
pub mod network;
pub mod audio;
pub mod advanced;

/// Device types supported by MultiOS
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum DeviceType {
    Unknown = 0,
    Keyboard = 1,
    Mouse = 2,
    Display = 3,
    Storage = 4,
    Network = 5,
    Audio = 6,
    USB = 7,
    PCI = 8,
    UART = 9,
}

/// Driver result type
pub type DriverResult<T> = Result<T, DriverError>;

/// Error types for driver operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriverError {
    DeviceNotFound,
    DriverNotSupported,
    InitializationFailed,
    DeviceBusy,
    PermissionDenied,
    HardwareError,
}

/// Global device driver manager
pub static DEVICE_MANAGER: Mutex<Option<driver_manager::DriverManager>> = Mutex::new(None);

/// Initialize the device driver framework
/// 
/// This function sets up the global device manager and must be called
/// during kernel initialization.
pub fn init() -> DriverResult<()> {
    let mut manager_guard = DEVICE_MANAGER.lock();
    
    let mut manager = driver_manager::DriverManager::new();
    
    // Register built-in drivers
    register_builtin_drivers(&mut manager)?;
    
    *manager_guard = Some(manager);
    
    Ok(())
}

/// Register built-in device drivers
fn register_builtin_drivers(manager: &mut driver_manager::DriverManager) -> DriverResult<()> {
    info!("Registering built-in device drivers");
    
    // Register serial driver
    let serial_driver = driver_manager::Driver {
        name: "16550 UART Driver",
        version: "1.0",
        device_types: &[DeviceType::UART],
        priority: 10,
        driver_func: || &serial::Uart16550::new(0x3F8),
    };
    manager.register_driver(serial_driver)?;
    
    // Register timer drivers
    let pit_driver = driver_manager::Driver {
        name: "8254 PIT Driver",
        version: "1.0",
        device_types: &[DeviceType::Unknown], // Timer device type
        priority: 20,
        driver_func: || &timer::Pit8254::new(),
    };
    manager.register_driver(pit_driver)?;
    
    // Register keyboard drivers
    let ps2_keyboard_driver = driver_manager::Driver {
        name: "PS/2 Keyboard Driver",
        version: "1.0",
        device_types: &[DeviceType::Keyboard],
        priority: 10,
        driver_func: || &keyboard::Ps2Keyboard::new(0x60),
    };
    manager.register_driver(ps2_keyboard_driver)?;
    
    // Register USB keyboard driver
    let usb_keyboard_driver = driver_manager::Driver {
        name: "USB Keyboard Driver",
        version: "1.0",
        device_types: &[DeviceType::Keyboard],
        priority: 15, // Lower priority than PS/2
        driver_func: || &keyboard::UsbKeyboard::new(1, 2),
    };
    manager.register_driver(usb_keyboard_driver)?;
    
    // Register graphics drivers
    let vga_driver = driver_manager::Driver {
        name: "VGA Graphics Driver",
        version: "1.0",
        device_types: &[DeviceType::Display],
        priority: 5, // Higher priority - VGA is basic display
        driver_func: || &graphics::VgaGraphics::new(0x3CE, 0xA0000),
    };
    manager.register_driver(vga_driver)?;
    
    let vesa_driver = driver_manager::Driver {
        name: "VESA Graphics Driver",
        version: "1.0",
        device_types: &[DeviceType::Display],
        priority: 10, // Lower priority than VGA
        driver_func: || &graphics::VesaGraphics::new(0xA0000),
    };
    manager.register_driver(vesa_driver)?;
    
    let uefi_gop_driver = driver_manager::Driver {
        name: "UEFI GOP Graphics Driver",
        version: "1.0",
        device_types: &[DeviceType::Display],
        priority: 15, // Lowest priority for UEFI
        driver_func: || &graphics::UefiGopGraphics::new(0xA0000, 8192 * 1024),
    };
    manager.register_driver(uefi_gop_driver)?;
    
    // Register storage drivers
    let sata_driver = driver_manager::Driver {
        name: "SATA Controller Driver",
        version: "1.0",
        device_types: &[DeviceType::Storage],
        priority: 5, // High priority for SATA
        driver_func: || &storage::SataController::new(0x1F0, 0x3F6),
    };
    manager.register_driver(sata_driver)?;
    
    let nvme_driver = driver_manager::Driver {
        name: "NVMe Controller Driver",
        version: "1.0",
        device_types: &[DeviceType::Storage],
        priority: 3, // Highest priority for NVMe
        driver_func: || &storage::NvmeController::new(0x9000, 0x9200),
    };
    manager.register_driver(nvme_driver)?;
    
    let usb_mass_driver = driver_manager::Driver {
        name: "USB Mass Storage Driver",
        version: "1.0",
        device_types: &[DeviceType::Storage],
        priority: 20, // Lowest priority - hot-plugged device
        driver_func: || &storage::UsbMassStorage::new(0, 1, 0x81, 0x02),
    };
    manager.register_driver(usb_mass_driver)?;
    
    // Register network drivers
    let ethernet_driver = driver_manager::Driver {
        name: "Ethernet Driver",
        version: "1.0",
        device_types: &[DeviceType::Network],
        priority: 5, // High priority for Ethernet
        driver_func: || &network::EthernetDriver::new(0x1C00, network::MacAddress::new([0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC])),
    };
    manager.register_driver(ethernet_driver)?;
    
    let wifi_driver = driver_manager::Driver {
        name: "WiFi Driver",
        version: "1.0",
        device_types: &[DeviceType::Network],
        priority: 10, // Lower priority than Ethernet
        driver_func: || &network::WifiDriver::new(network::MacAddress::new([0xFE, 0xDC, 0xBA, 0x98, 0x76, 0x54]), network::WifiRadioType::N),
    };
    manager.register_driver(wifi_driver)?;
    
    // Register audio drivers
    let ac97_driver = driver_manager::Driver {
        name: "AC'97 Audio Driver",
        version: "1.0",
        device_types: &[DeviceType::Audio],
        priority: 10,
        driver_func: || &audio::Ac97Driver::new(0x400, 0),
    };
    manager.register_driver(ac97_driver)?;
    
    let hda_driver = driver_manager::Driver {
        name: "Intel HDA Audio Driver",
        version: "1.0",
        device_types: &[DeviceType::Audio],
        priority: 5, // Higher priority than AC'97
        driver_func: || &audio::HdaDriver::new(0x9000, 0),
    };
    manager.register_driver(hda_driver)?;
    
    let usb_audio_driver = driver_manager::Driver {
        name: "USB Audio Driver",
        version: "1.0",
        device_types: &[DeviceType::Audio],
        priority: 20, // Lowest priority - hot-plugged device
        driver_func: || &audio::UsbAudioDriver::new(0, 2, 0, 0x81, 0x02),
    };
    manager.register_driver(usb_audio_driver)?;
    
    info!("Registered {} built-in device drivers", manager.get_stats().total_drivers);
    
    Ok(())
}

/// Register a device driver
pub fn register_driver(driver: driver_manager::Driver) -> DriverResult<()> {
    let mut manager_guard = DEVICE_MANAGER.lock();
    
    let manager = manager_guard
        .as_mut()
        .ok_or(DriverError::DeviceNotFound)?;
        
    manager.register_driver(driver)
}

/// Find devices of a specific type
pub fn find_devices(device_type: DeviceType) -> DriverResult<Vec<device::DeviceHandle>> {
    let manager_guard = DEVICE_MANAGER.lock();
    
    let manager = manager_guard
        .as_ref()
        .ok_or(DriverError::DeviceNotFound)?;
        
    manager.find_devices(device_type)
}

/// Get device by ID
pub fn get_device(device_id: u32) -> DriverResult<device::DeviceHandle> {
    let manager_guard = DEVICE_MANAGER.lock();
    
    let manager = manager_guard
        .as_mut()
        .ok_or(DriverError::DeviceNotFound)?;
        
    manager.get_device(device_id)
}

/// Number of registered devices
pub fn get_device_count() -> usize {
    let manager_guard = DEVICE_MANAGER.lock();
    
    if let Some(manager) = manager_guard.as_ref() {
        manager.get_device_count()
    } else {
        0
    }
}

/// Discover and enumerate all available devices
pub fn discover_all_devices() -> DriverResult<Vec<DeviceHandle>> {
    let mut manager_guard = DEVICE_MANAGER.lock();
    
    let manager = manager_guard
        .as_mut()
        .ok_or(DriverError::DeviceNotFound)?;
        
    manager.discover_devices()?;
    
    // Get all devices from all device types
    let mut all_devices = Vec::new();
    
    for device_type in DeviceType::Unknown..=DeviceType::UART {
        match manager.find_devices(device_type) {
            Ok(mut devices) => {
                all_devices.append(&mut devices);
            }
            Err(_) => {
                // Device type not found, continue
            }
        }
    }
    
    Ok(all_devices)
}

/// Initialize console output device
pub fn init_console() -> DriverResult<serial::SerialConsole> {
    let mut console = serial::SerialConsole::new(0x3F8);
    console.init()?;
    Ok(console)
}

/// Initialize system timer
pub fn init_system_timer() -> DriverResult<timer::TimerManager> {
    timer::TimerManager::init_global()?;
    Ok(timer::TimerManager::new())
}

/// Initialize keyboard input
pub fn init_keyboard() -> DriverResult<()> {
    let ps2_keyboard = keyboard::Ps2Keyboard::new(0x60);
    keyboard::init_global_keyboard(Box::new(ps2_keyboard));
    Ok(())
}

/// Get driver manager statistics
pub fn get_driver_stats() -> DriverResult<driver_manager::DriverManagerStats> {
    let manager_guard = DEVICE_MANAGER.lock();
    
    let manager = manager_guard
        .as_ref()
        .ok_or(DriverError::DeviceNotFound)?;
        
    Ok(manager.get_stats().clone())
}

/// List all registered drivers
pub fn list_drivers() -> DriverResult<Vec<&'static str>> {
    let manager_guard = DEVICE_MANAGER.lock();
    
    let manager = manager_guard
        .as_ref()
        .ok_or(DriverError::DeviceNotFound)?;
        
    Ok(manager.list_drivers())
}

/// List all detected devices
pub fn list_devices() -> DriverResult<Vec<(DeviceId, DeviceType, &'static str)>> {
    let manager_guard = DEVICE_MANAGER.lock();
    
    let manager = manager_guard
        .as_mut()
        .ok_or(DriverError::DeviceNotFound)?;
        
    Ok(manager.list_devices())
}

/// Initialize graphics driver manager
pub fn init_graphics_driver_manager() -> DriverResult<graphics::GraphicsDriverManager> {
    let mut manager = graphics::GraphicsDriverManager::new();
    
    // Register all graphics drivers
    manager.register_vga(0x3CE, 0xA0000)?;
    manager.register_vesa(0xA0000)?;
    manager.register_uefi_gop(0xA0000, 8192 * 1024)?;
    
    // Set VESA as default graphics driver
    manager.set_current_driver(graphics::GraphicsMode::Vesa)?;
    
    Ok(manager)
}

/// Initialize storage driver manager
pub fn init_storage_driver_manager() -> DriverResult<storage::StorageDriverManager> {
    let mut manager = storage::StorageDriverManager::new();
    
    // Register all storage controllers
    manager.register_sata(0x1F0, 0x3F6)?;
    manager.register_nvme(0x9000, 0x9200)?;
    manager.register_usb_mass_storage(0, 1, 0x81, 0x02)?;
    
    Ok(manager)
}

/// Initialize network driver manager
pub fn init_network_driver_manager() -> DriverResult<network::NetworkDriverManager> {
    let mut manager = network::NetworkDriverManager::new();
    
    // Register network interfaces
    let mac = network::MacAddress::new([0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC]);
    manager.register_ethernet(0x1C00, mac)?;
    
    let wifi_mac = network::MacAddress::new([0xFE, 0xDC, 0xBA, 0x98, 0x76, 0x54]);
    manager.register_wifi(wifi_mac, network::WifiRadioType::N)?;
    
    Ok(manager)
}

/// Initialize audio driver manager
pub fn init_audio_driver_manager() -> DriverResult<audio::AudioDriverManager> {
    let mut manager = audio::AudioDriverManager::new();
    
    // Register audio devices
    manager.register_ac97(0x400, 0)?;
    manager.register_hda(0x9000, 0)?;
    manager.register_usb_audio(0, 2, 0, 0x81, 0x02)?;
    
    Ok(manager)
}

/// Clear graphics screen
pub fn clear_screen(color: u32) -> DriverResult<()> {
    let mut manager_guard = DEVICE_MANAGER.lock();
    // This would require access to graphics manager - simplified for demo
    Ok(())
}

/// Draw pixel on graphics display
pub fn draw_pixel(x: u32, y: u32, color: u32) -> DriverResult<()> {
    // Simplified graphics operation
    Ok(())
}

/// Read block device sectors
pub fn read_disk_sectors(sector: u64, count: u32, buffer: &mut [u8]) -> DriverResult<usize> {
    // Simplified disk read operation
    Ok(buffer.len())
}

/// Write block device sectors
pub fn write_disk_sectors(sector: u64, count: u32, buffer: &[u8]) -> DriverResult<usize> {
    // Simplified disk write operation
    Ok(buffer.len())
}

/// Send network packet
pub fn send_network_packet(packet: &network::NetworkPacket) -> DriverResult<usize> {
    // Simplified network send operation
    Ok(packet.data.len())
}

/// Receive network packet
pub fn receive_network_packet() -> DriverResult<Option<network::NetworkPacket>> {
    // Simplified network receive operation
    Ok(None)
}

/// Play audio buffer
pub fn play_audio_buffer(buffer: &audio::AudioBuffer) -> DriverResult<usize> {
    // Simplified audio playback
    Ok(buffer.data.len())
}

/// Record audio buffer
pub fn record_audio_buffer(buffer: &mut audio::AudioBuffer) -> DriverResult<usize> {
    // Simplified audio recording
    Ok(0)
}

/// Set master volume
pub fn set_master_volume(volume: audio::VolumeLevel) -> DriverResult<()> {
    // Simplified volume control
    Ok(())
}

/// Get master volume
pub fn get_master_volume() -> DriverResult<audio::VolumeLevel> {
    Ok(audio::VolumeLevel::new(75))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_type_ordering() {
        assert_eq!(DeviceType::Unknown as u8, 0);
        assert_eq!(DeviceType::Keyboard as u8, 1);
        assert_eq!(DeviceType::UART as u8, 9);
    }

    #[test]
    fn test_driver_error_variants() {
        let errors = [
            DriverError::DeviceNotFound,
            DriverError::DriverNotSupported,
            DriverError::InitializationFailed,
            DriverError::DeviceBusy,
            DriverError::PermissionDenied,
            DriverError::HardwareError,
        ];
        
        for (i, &error) in errors.iter().enumerate() {
            assert_eq!(error as usize, i);
        }
    }
}