//! Boot Device Detection
//! 
//! Provides functionality to detect and enumerate available boot devices
//! across different architectures and boot modes (UEFI, BIOS, ARM, RISC-V).

use crate::{BootMode, BootError};
use core::fmt;

/// Boot device types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootDeviceType {
    // UEFI/BIOS devices
    HardDisk,
    USB,
    CDROM,
    Network,
    
    // ARM devices
    SDCard,
    eMMC,
    UART,
    
    // RISC-V devices
    SPI,
    I2C,
    
    // Virtual devices
    VirtualDisk,
    RAMDisk,
    
    // Firmware
    Firmware,
}

/// Boot device information
#[derive(Debug, Clone)]
pub struct BootDevice {
    pub device_type: BootDeviceType,
    pub device_path: &'static str,
    pub description: &'static str,
    pub is_bootable: bool,
    pub is_removable: bool,
    pub priority: u8, // Lower numbers = higher priority
    pub supported_modes: Vec<BootMode>,
}

/// Boot device detection result
pub type BootDeviceResult<T> = Result<T, BootDeviceError>;

/// Boot device detection errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootDeviceError {
    DetectionFailed,
    DeviceNotFound,
    UnsupportedDevice,
    AccessDenied,
    NotBootable,
}

/// Boot device enumeration context
#[derive(Debug, Clone)]
pub struct BootDeviceContext {
    pub architecture: BootArchitecture,
    pub boot_mode: BootMode,
    pub detected_devices: Vec<BootDevice>,
}

/// Supported architectures for boot device detection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootArchitecture {
    X86_64,
    ARM64,
    RISCV64,
    Unknown,
}

/// Global boot device context
static BOOT_DEVICE_CONTEXT: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);

impl BootDevice {
    /// Create a new boot device
    pub fn new(
        device_type: BootDeviceType,
        device_path: &'static str,
        description: &'static str,
        is_bootable: bool,
        is_removable: bool,
        priority: u8,
        supported_modes: Vec<BootMode>,
    ) -> Self {
        Self {
            device_type,
            device_path,
            description,
            is_bootable,
            is_removable,
            priority,
            supported_modes,
        }
    }

    /// Check if device supports a specific boot mode
    pub fn supports_mode(&self, mode: BootMode) -> bool {
        self.supported_modes.contains(&mode)
    }

    /// Check if device has higher priority than another
    pub fn has_higher_priority_than(&self, other: &Self) -> bool {
        self.priority < other.priority
    }
}

impl fmt::Display for BootDevice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let bootable_marker = if self.is_bootable { " [BOOTABLE]" } else { "" };
        let removable_marker = if self.is_removable { " [REMOVABLE]" } else { "" };
        write!(
            f,
            "{}: {}{}{}",
            self.device_type, self.description, bootable_marker, removable_marker
        )
    }
}

impl BootDeviceContext {
    /// Create a new boot device context
    pub fn new(architecture: BootArchitecture, boot_mode: BootMode) -> Self {
        Self {
            architecture,
            boot_mode,
            detected_devices: Vec::new(),
        }
    }

    /// Add a device to the context
    pub fn add_device(&mut self, device: BootDevice) {
        self.detected_devices.push(device);
    }

    /// Get all detected devices
    pub fn devices(&self) -> &[BootDevice] {
        &self.detected_devices
    }

    /// Get bootable devices sorted by priority
    pub fn bootable_devices(&self) -> Vec<&BootDevice> {
        let mut bootable: Vec<_> = self.detected_devices
            .iter()
            .filter(|d| d.is_bootable && d.supports_mode(self.boot_mode))
            .collect();
        
        bootable.sort_by(|a, b| a.priority.cmp(&b.priority));
        bootable
    }

    /// Find device by path
    pub fn find_device_by_path(&self, path: &str) -> Option<&BootDevice> {
        self.detected_devices.iter().find(|d| d.device_path == path)
    }

    /// Find devices by type
    pub fn find_devices_by_type(&self, device_type: BootDeviceType) -> Vec<&BootDevice> {
        self.detected_devices.iter()
            .filter(|d| d.device_type == device_type)
            .collect()
    }
}

impl fmt::Display for BootDeviceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BootDeviceType::HardDisk => write!(f, "Hard Disk"),
            BootDeviceType::USB => write!(f, "USB Device"),
            BootDeviceType::CDROM => write!(f, "CD/DVD Drive"),
            BootDeviceType::Network => write!(f, "Network Boot"),
            BootDeviceType::SDCard => write!(f, "SD Card"),
            BootDeviceType::eMMC => write!(f, "eMMC Storage"),
            BootDeviceType::UART => write!(f, "UART/Serial"),
            BootDeviceType::SPI => write!(f, "SPI Device"),
            BootDeviceType::I2C => write!(f, "I2C Device"),
            BootDeviceType::VirtualDisk => write!(f, "Virtual Disk"),
            BootDeviceType::RAMDisk => write!(f, "RAM Disk"),
            BootDeviceType::Firmware => write!(f, "Firmware"),
        }
    }
}

impl BootArchitecture {
    /// Detect current architecture
    pub fn current() -> Self {
        #[cfg(target_arch = "x86_64")]
        {
            BootArchitecture::X86_64
        }
        #[cfg(target_arch = "aarch64")]
        {
            BootArchitecture::ARM64
        }
        #[cfg(target_arch = "riscv64")]
        {
            BootArchitecture::RISCV64
        }
        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64", target_arch = "riscv64")))]
        {
            BootArchitecture::Unknown
        }
    }

    /// Check if architecture supports UEFI
    pub fn supports_uefi(&self) -> bool {
        matches!(self, BootArchitecture::X86_64 | BootArchitecture::ARM64)
    }

    /// Check if architecture supports legacy BIOS
    pub fn supports_legacy_bios(&self) -> bool {
        matches!(self, BootArchitecture::X86_64)
    }

    /// Check if architecture requires device tree
    pub fn requires_device_tree(&self) -> bool {
        matches!(self, BootArchitecture::ARM64 | BootArchitecture::RISCV64)
    }
}

/// Initialize boot device detection for current architecture
pub fn init_device_detection(boot_mode: BootMode) -> BootDeviceResult<BootDeviceContext> {
    let architecture = BootArchitecture::current();
    let mut context = BootDeviceContext::new(architecture, boot_mode);
    
    match architecture {
        BootArchitecture::X86_64 => detect_x86_64_devices(&mut context),
        BootArchitecture::ARM64 => detect_arm64_devices(&mut context),
        BootArchitecture::RISCV64 => detect_riscv64_devices(&mut context),
        BootArchitecture::Unknown => return Err(BootDeviceError::UnsupportedDevice),
    }
    
    Ok(context)
}

/// Detect devices for x86_64 architecture
fn detect_x86_64_devices(context: &mut BootDeviceContext) {
    // Primary hard disk
    context.add_device(BootDevice::new(
        BootDeviceType::HardDisk,
        "/dev/sda",
        "Primary SATA Hard Drive",
        true,
        false,
        1,
        vec![BootMode::UEFI, BootMode::LegacyBIOS],
    ));

    // Secondary hard disk
    context.add_device(BootDevice::new(
        BootDeviceType::HardDisk,
        "/dev/sdb",
        "Secondary SATA Hard Drive",
        true,
        false,
        2,
        vec![BootMode::UEFI, BootMode::LegacyBIOS],
    ));

    // USB device
    context.add_device(BootDevice::new(
        BootDeviceType::USB,
        "/dev/sdc",
        "USB Flash Drive",
        true,
        true,
        3,
        vec![BootMode::UEFI, BootMode::LegacyBIOS],
    ));

    // CD/DVD drive
    context.add_device(BootDevice::new(
        BootDeviceType::CDROM,
        "/dev/sr0",
        "CD/DVD Drive",
        true,
        true,
        4,
        vec![BootMode::LegacyBIOS],
    ));

    // Network boot (PXE)
    context.add_device(BootDevice::new(
        BootDeviceType::Network,
        "pxe",
        "Network Boot (PXE)",
        true,
        false,
        10,
        vec![BootMode::UEFI, BootMode::LegacyBIOS],
    ));
}

/// Detect devices for ARM64 architecture
fn detect_arm64_devices(context: &mut BootDeviceContext) {
    // SD Card (common on Raspberry Pi)
    context.add_device(BootDevice::new(
        BootDeviceType::SDCard,
        "/dev/mmcblk0",
        "SD Card",
        true,
        true,
        1,
        vec![BootMode::UEFI],
    ));

    // eMMC (common on embedded boards)
    context.add_device(BootDevice::new(
        BootDeviceType::eMMC,
        "/dev/mmcblk1",
        "eMMC Storage",
        true,
        false,
        2,
        vec![BootMode::UEFI],
    ));

    // USB storage
    context.add_device(BootDevice::new(
        BootDeviceType::USB,
        "/dev/sda",
        "USB Storage",
        true,
        true,
        3,
        vec![BootMode::UEFI],
    ));

    // UART (for serial boot/installation)
    context.add_device(BootDevice::new(
        BootDeviceType::UART,
        "ttyAMA0",
        "Serial Console",
        false,
        false,
        100,
        vec![BootMode::UEFI],
    ));
}

/// Detect devices for RISC-V architecture
fn detect_riscv64_devices(context: &mut BootDeviceContext) {
    // SPI flash (common on RISC-V boards)
    context.add_device(BootDevice::new(
        BootDeviceType::SPI,
        "/dev/mtd0",
        "SPI Flash",
        true,
        false,
        1,
        vec![BootMode::UEFI],
    ));

    // eMMC storage
    context.add_device(BootDevice::new(
        BootDeviceType::eMMC,
        "/dev/mmcblk0",
        "eMMC Storage",
        true,
        false,
        2,
        vec![BootMode::UEFI],
    ));

    // SD Card
    context.add_device(BootDevice::new(
        BootDeviceType::SDCard,
        "/dev/mmcblk1",
        "SD Card",
        true,
        true,
        3,
        vec![BootMode::UEFI],
    ));

    // USB storage
    context.add_device(BootDevice::new(
        BootDeviceType::USB,
        "/dev/sda",
        "USB Storage",
        true,
        true,
        4,
        vec![BootMode::UEFI],
    ));

    // I2C devices
    context.add_device(BootDevice::new(
        BootDeviceType::I2C,
        "i2c0",
        "I2C Bus 0",
        false,
        false,
        100,
        vec![BootMode::UEFI],
    ));
}

/// Get the best boot device for the current configuration
pub fn get_best_boot_device() -> BootDeviceResult<BootDevice> {
    let architecture = BootArchitecture::current();
    let boot_mode = crate::detect_boot_mode();
    
    let mut context = init_device_detection(boot_mode)?;
    let bootable_devices = context.bootable_devices();
    
    if bootable_devices.is_empty() {
        return Err(BootDeviceError::NotBootable);
    }
    
    // Return the highest priority device
    Ok(bootable_devices[0].clone())
}

/// Check if a specific device is available and bootable
pub fn check_device_availability(device_path: &str) -> BootDeviceResult<bool> {
    let architecture = BootArchitecture::current();
    let boot_mode = crate::detect_boot_mode();
    
    let context = init_device_detection(boot_mode)?;
    
    match context.find_device_by_path(device_path) {
        Some(device) => {
            if device.is_bootable && device.supports_mode(boot_mode) {
                Ok(true)
            } else {
                Err(BootDeviceError::NotBootable)
            }
        }
        None => Err(BootDeviceError::DeviceNotFound),
    }
}

/// Display all detected boot devices
pub fn display_detected_devices() -> BootDeviceResult<()> {
    let architecture = BootArchitecture::current();
    let boot_mode = crate::detect_boot_mode();
    
    let context = init_device_detection(boot_mode)?;
    
    println!("\n=== Detected Boot Devices ({} Architecture) ===", architecture);
    println!("Boot Mode: {:?}", boot_mode);
    
    if context.devices().is_empty() {
        println!("No devices detected.");
        return Ok(());
    }
    
    println!("\nAll Devices:");
    for device in context.devices() {
        println!("  {}", device);
    }
    
    println!("\nBootable Devices (sorted by priority):");
    let bootable_devices = context.bootable_devices();
    if bootable_devices.is_empty() {
        println!("  No bootable devices found for current configuration.");
    } else {
        for device in bootable_devices {
            println!("  [P{}] {}", device.priority, device);
        }
    }
    
    Ok(())
}

/// Get device type from device path
pub fn get_device_type_from_path(path: &str) -> Option<BootDeviceType> {
    // Simple heuristics to determine device type from path
    if path.starts_with("/dev/sd") {
        Some(BootDeviceType::HardDisk)
    } else if path.starts_with("/dev/mmcblk") {
        Some(BootDeviceType::SDCard)
    } else if path.starts_with("/dev/sr") {
        Some(BootDeviceType::CDROM)
    } else if path == "pxe" {
        Some(BootDeviceType::Network)
    } else if path.starts_with("tty") {
        Some(BootDeviceType::UART)
    } else if path.starts_with("/dev/mtd") {
        Some(BootDeviceType::SPI)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boot_device_creation() {
        let device = BootDevice::new(
            BootDeviceType::HardDisk,
            "/dev/sda",
            "Test Hard Drive",
            true,
            false,
            1,
            vec![BootMode::UEFI],
        );
        
        assert_eq!(device.device_type, BootDeviceType::HardDisk);
        assert_eq!(device.device_path, "/dev/sda");
        assert!(device.is_bootable);
        assert!(!device.is_removable);
        assert_eq!(device.priority, 1);
        assert!(device.supports_mode(BootMode::UEFI));
        assert!(!device.supports_mode(BootMode::LegacyBIOS));
    }

    #[test]
    fn test_architecture_detection() {
        let arch = BootArchitecture::current();
        
        #[cfg(target_arch = "x86_64")]
        assert_eq!(arch, BootArchitecture::X86_64);
        
        #[cfg(target_arch = "aarch64")]
        assert_eq!(arch, BootArchitecture::ARM64);
        
        #[cfg(target_arch = "riscv64")]
        assert_eq!(arch, BootArchitecture::RISCV64);
    }

    #[test]
    fn test_device_priority() {
        let device1 = BootDevice::new(
            BootDeviceType::HardDisk,
            "/dev/sda",
            "Device 1",
            true,
            false,
            1,
            vec![BootMode::UEFI],
        );
        
        let device2 = BootDevice::new(
            BootDeviceType::USB,
            "/dev/sdb",
            "Device 2",
            true,
            false,
            3,
            vec![BootMode::UEFI],
        );
        
        assert!(device1.has_higher_priority_than(&device2));
        assert!(!device2.has_higher_priority_than(&device1));
    }

    #[test]
    fn test_device_type_display() {
        assert_eq!(format!("{}", BootDeviceType::HardDisk), "Hard Disk");
        assert_eq!(format!("{}", BootDeviceType::USB), "USB Device");
        assert_eq!(format!("{}", BootDeviceType::Network), "Network Boot");
    }

    #[test]
    fn test_get_device_type_from_path() {
        assert_eq!(get_device_type_from_path("/dev/sda"), Some(BootDeviceType::HardDisk));
        assert_eq!(get_device_type_from_path("/dev/mmcblk0"), Some(BootDeviceType::SDCard));
        assert_eq!(get_device_type_from_path("/dev/sr0"), Some(BootDeviceType::CDROM));
        assert_eq!(get_device_type_from_path("pxe"), Some(BootDeviceType::Network));
        assert_eq!(get_device_type_from_path("ttyAMA0"), Some(BootDeviceType::UART));
        assert_eq!(get_device_type_from_path("/dev/mtd0"), Some(BootDeviceType::SPI));
        assert_eq!(get_device_type_from_path("unknown"), None);
    }
}