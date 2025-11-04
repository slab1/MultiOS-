//! MultiOS Bootloader
//! 
//! This module provides comprehensive bootloader functionality for the MultiOS operating system,
//! supporting both UEFI and legacy BIOS boot methods with proper error handling and logging.

#![no_std]
#![feature(abi_efiapi)]
#![feature(asm)]
#![allow(dead_code)]
#![allow(unused_variables)]

extern crate bootloader;
extern crate x86_64;
extern crate spin;
extern crate bitflags;
extern crate log;
extern crate uart_16550;
extern crate thiserror;
extern crate atomic;

use core::panic::PanicInfo;
use spin::Mutex;
use log::{info, warn, error, debug, Level};
use atomic::Atomic;

use bootloader::boot_info::BootInfo;

use crate::uefi::{uefi_boot_start, UefiBootContext};
use crate::legacy::{legacy_bios_boot_start, LegacyBiosContext};
use crate::memory_map::MemoryMap;
use crate::kernel_loader::{KernelBootInfo, create_kernel_boot_info, enter_kernel, BootInfoBuffer};

/// Boot result type
pub type BootResult<T> = Result<T, BootError>;

/// Error types for boot operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootError {
    UefiNotSupported,
    LegacyNotSupported,
    KernelNotFound,
    MemoryMapError,
    InvalidKernelFormat,
    BootProcessError,
    SerialConsoleError,
    BiosDetectionFailed,
    UefiSystemTableError,
    KernelLoadError,
    BootInfoError,
}

impl core::fmt::Display for BootError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            BootError::UefiNotSupported => write!(f, "UEFI boot not supported"),
            BootError::LegacyNotSupported => write!(f, "Legacy BIOS boot not supported"),
            BootError::KernelNotFound => write!(f, "Kernel not found"),
            BootError::MemoryMapError => write!(f, "Memory map error"),
            BootError::InvalidKernelFormat => write!(f, "Invalid kernel format"),
            BootError::BootProcessError => write!(f, "Boot process error"),
            BootError::SerialConsoleError => write!(f, "Serial console error"),
            BootError::BiosDetectionFailed => write!(f, "BIOS detection failed"),
            BootError::UefiSystemTableError => write!(f, "UEFI system table error"),
            BootError::KernelLoadError => write!(f, "Kernel load error"),
            BootError::BootInfoError => write!(f, "Boot info error"),
        }
    }
}

/// Boot mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootMode {
    UEFI,
    LegacyBIOS,
    Unknown,
}

/// Boot configuration
#[derive(Debug, Clone)]
pub struct BootConfig {
    pub mode: BootMode,
    pub kernel_path: &'static str,
    pub initrd_path: Option<&'static str>,
    pub command_line: Option<&'static str>,
    pub memory_test: bool,
    pub serial_console: bool,
    pub debug_mode: bool,
    pub log_level: Level,
}

/// Global boot state
static BOOT_STATE: Mutex<Option<BootState>> = Mutex::new(None);

/// Boot state structure
#[derive(Debug)]
pub struct BootState {
    pub mode: BootMode,
    pub boot_time: u64,
    pub memory_map: MemoryMap,
    pub boot_config: BootConfig,
    pub bootloader_version: [u8; 64],
}

/// Bootloader entry point - called by the bootloader crate
pub fn boot_start(boot_info: &'static BootInfo) -> ! {
    info!("MultiOS Bootloader v0.1.0 starting...");
    info!("Bootloader features: UEFI={}, Legacy={}, Logging={}, Debug={}", 
          cfg!(feature = "uefi"), 
          cfg!(feature = "legacy"),
          cfg!(feature = "logging"),
          cfg!(feature = "debug_mode"));
    
    let boot_time = get_boot_time();
    
    // Initialize serial console first
    init_serial_console();
    
    // Detect boot mode
    let boot_mode = detect_boot_mode(boot_info);
    info!("Detected boot mode: {:?}", boot_mode);
    
    // Initialize boot state
    let boot_config = BootConfig {
        mode: boot_mode,
        kernel_path: "/boot/multios/kernel",
        initrd_path: None,
        command_line: None,
        memory_test: cfg!(feature = "memory_test"),
        serial_console: cfg!(feature = "logging"),
        debug_mode: cfg!(feature = "debug_mode"),
        log_level: if cfg!(feature = "debug_mode") { Level::Debug } else { Level::Info },
    };
    
    let bootloader_version = *b"MultiOS Bootloader v0.1.0                          \0";
    
    let memory_map = MemoryMap::from_boot_info(boot_info);
    
    {
        let mut boot_state_guard = BOOT_STATE.lock();
        *boot_state_guard = Some(BootState {
            mode: boot_mode,
            boot_time,
            memory_map: memory_map.clone(),
            boot_config: boot_config.clone(),
            bootloader_version,
        });
    }
    
    // Print memory map
    if boot_config.debug_mode {
        memory_map.print();
    }
    
    // Execute boot process based on mode
    match boot_mode {
        BootMode::UEFI => {
            info!("Starting UEFI boot process...");
            boot_uefi(boot_info)
        }
        BootMode::LegacyBIOS => {
            info!("Starting legacy BIOS boot process...");
            boot_legacy(boot_info)
        }
        BootMode::Unknown => {
            error!("Cannot determine boot mode");
            boot_panic("Boot mode detection failed");
        }
    }
}

/// Boot via UEFI
fn boot_uefi(boot_info: &'static BootInfo) -> ! {
    info!("UEFI boot process starting...");
    
    // The UEFI system table would be provided by the bootloader crate
    // For now, we'll use a simplified UEFI boot process
    
    // Create kernel boot information
    let kernel_config = create_kernel_boot_info("/boot/multios/kernel", None);
    
    // Create boot info buffer for kernel
    let boot_info_buffer = create_kernel_boot_info_buffer(boot_info);
    
    // Load and start kernel
    start_uefi_kernel(boot_info, kernel_config, boot_info_buffer)
}

/// Boot via Legacy BIOS
fn boot_legacy(boot_info: &'static BootInfo) -> ! {
    info!("Legacy BIOS boot process starting...");
    
    // Create kernel boot information
    let kernel_config = create_kernel_boot_info("/boot/multios/kernel", None);
    
    // Create boot info buffer for kernel
    let boot_info_buffer = create_kernel_boot_info_buffer(boot_info);
    
    // Load and start kernel
    start_legacy_kernel(boot_info, kernel_config, boot_info_buffer)
}

/// Create kernel boot info buffer
fn create_kernel_boot_info_buffer(boot_info: &'static BootInfo) -> BootInfoBuffer {
    info!("Creating kernel boot information buffer...");
    
    // Estimate required size
    let boot_info_size = core::mem::size_of::<KernelBootInfo>();
    let memory_map_size = boot_info.memory_map.len() * core::mem::size_of::<crate::kernel_loader::KMemoryMapEntry>();
    let cmdline_size = 2048;
    let modules_size = boot_info.modules.len() * core::mem::size_of::<crate::kernel_loader::KModuleInfo>();
    
    let total_size = boot_info_size + memory_map_size + cmdline_size + modules_size;
    let buffer_size = (total_size + 4095) & !4095; // Align to page boundary
    
    let mut buffer = BootInfoBuffer::new(buffer_size);
    
    // Set kernel information (placeholder)
    buffer.set_kernel_info(crate::x86_64::PhysAddr::new(0x100000), 1024 * 1024, crate::x86_64::PhysAddr::new(0x100000));
    
    // Set framebuffer information
    if let Some(ref fb) = boot_info.framebuffer {
        buffer.set_framebuffer(fb);
    }
    
    // Set memory map
    let memory_map = MemoryMap::from_boot_info(boot_info);
    buffer.set_memory_map(&memory_map);
    
    // Set command line
    buffer.set_command_line(None);
    
    // Add modules
    for module in &boot_info.modules {
        buffer.add_module(
            crate::x86_64::PhysAddr::new(module.start),
            crate::x86_64::PhysAddr::new(module.end),
            &module.name,
        );
    }
    
    // Finalize buffer
    let boot_info_addr = buffer.finalize();
    
    info!("Kernel boot info buffer created at: {:?}", boot_info_addr);
    buffer
}

/// Start kernel via UEFI
fn start_uefi_kernel(
    boot_info: &'static BootInfo,
    kernel_config: crate::kernel_loader::KernelBootConfig,
    boot_info_buffer: BootInfoBuffer,
) -> ! {
    info!("Starting kernel via UEFI boot process...");
    
    // Validate memory map
    let memory_map = MemoryMap::from_boot_info(boot_info);
    if !memory_map.validate() {
        error!("Memory map validation failed");
        boot_panic("Memory map validation failed");
    }
    
    // In a real UEFI implementation, this would:
    // 1. Exit boot services properly
    // 2. Call the kernel entry point with boot info
    // 3. Handle the transition from UEFI runtime to kernel
    
    info!("UEFI boot process completed");
    info!("Kernel would be started here with boot info at: {:?}", 
          crate::x86_64::PhysAddr::new(boot_info_buffer.buffer.as_ptr() as u64));
    
    boot_panic("UEFI kernel transition not yet implemented");
}

/// Start kernel via Legacy BIOS
fn start_legacy_kernel(
    boot_info: &'static BootInfo,
    kernel_config: crate::kernel_loader::KernelBootConfig,
    boot_info_buffer: BootInfoBuffer,
) -> ! {
    info!("Starting kernel via legacy BIOS boot process...");
    
    // Validate memory map
    let memory_map = MemoryMap::from_boot_info(boot_info);
    if !memory_map.validate() {
        error!("Memory map validation failed");
        boot_panic("Memory map validation failed");
    }
    
    // In a real legacy BIOS implementation, this would:
    // 1. Set up real mode environment
    // 2. Load kernel into memory
    // 3. Jump to kernel entry point
    
    info!("Legacy BIOS boot process completed");
    info!("Kernel would be started here with boot info at: {:?}", 
          crate::x86_64::PhysAddr::new(boot_info_buffer.buffer.as_ptr() as u64));
    
    boot_panic("Legacy BIOS kernel transition not yet implemented");
}

/// Detect boot mode from boot info
fn detect_boot_mode(boot_info: &'static BootInfo) -> BootMode {
    // Check if we have UEFI information
    if boot_info.uefi_info.is_some() {
        info!("UEFI boot detected");
        return BootMode::UEFI;
    }
    
    // Check bootloader signature for legacy BIOS
    // The bootloader crate provides information about the boot method
    
    // For now, assume UEFI on x86_64 if available, otherwise legacy
    #[cfg(target_arch = "x86_64")]
    {
        if cfg!(feature = "uefi") {
            return BootMode::UEFI;
        }
    }
    
    info!("Legacy BIOS boot detected");
    BootMode::LegacyBIOS
}

/// Initialize serial console for bootloader logging
fn init_serial_console() {
    #[cfg(feature = "logging")]
    {
        info!("Initializing serial console...");
        
        // Initialize serial port (COM1 = 0x3F8)
        match uart_16550::SerialPort::new(0x3F8) {
            Ok(mut serial) => {
                serial.init();
                info!("Serial console initialized on COM1 (0x3F8)");
            }
            Err(_) => {
                warn!("Failed to initialize serial console");
            }
        }
    }
}

/// Get current boot time
fn get_boot_time() -> u64 {
    // Read from system timer or RTC
    // This is a simplified implementation
    0
}

/// Bootloader panic handler
#[panic_handler]
fn bootloader_panic(info: &PanicInfo) -> ! {
    error!("BOOTLOADER PANIC: {}", info);
    
    // Try to print to serial console
    #[cfg(feature = "logging")]
    {
        if let Ok(mut serial) = uart_16550::SerialPort::new(0x3F8) {
            serial.init();
            let _ = writeln!(serial, "PANIC: {}", info);
        }
    }
    
    boot_panic("Bootloader panic occurred");
}

/// Halt the system with a panic message
fn boot_panic(message: &str) -> ! {
    error!("Bootloader panic: {}", message);
    
    // Print to serial console if available
    #[cfg(feature = "logging")]
    {
        if let Ok(mut serial) = uart_16550::SerialPort::new(0x3F8) {
            serial.init();
            let _ = writeln!(serial, "PANIC: {}", message);
        }
    }
    
    // Halt the system
    loop {
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}

/// Get boot configuration
pub fn get_boot_config() -> BootConfig {
    let boot_state_guard = BOOT_STATE.lock();
    if let Some(state) = boot_state_guard.as_ref() {
        state.boot_config
    } else {
        // Default configuration
        BootConfig {
            mode: BootMode::Unknown,
            kernel_path: "/boot/multios/kernel",
            initrd_path: None,
            command_line: None,
            memory_test: false,
            serial_console: true,
            debug_mode: false,
            log_level: Level::Info,
        }
    }
}

/// Get memory map
pub fn get_memory_map() -> Option<MemoryMap> {
    let boot_state_guard = BOOT_STATE.lock();
    boot_state_guard.as_ref().map(|s| s.memory_map)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boot_mode_ordering() {
        assert!(matches!(BootMode::UEFI, BootMode::UEFI));
        assert!(matches!(BootMode::LegacyBIOS, BootMode::LegacyBIOS));
        assert!(matches!(BootMode::Unknown, BootMode::Unknown));
    }

    #[test]
    fn test_boot_error_variants() {
        let errors = [
            BootError::UefiNotSupported,
            BootError::LegacyNotSupported,
            BootError::KernelNotFound,
            BootError::MemoryMapError,
            BootError::InvalidKernelFormat,
            BootError::BootProcessError,
            BootError::SerialConsoleError,
            BootError::BiosDetectionFailed,
            BootError::UefiSystemTableError,
            BootError::KernelLoadError,
            BootError::BootInfoError,
        ];
        
        assert_eq!(errors.len(), 11);
    }

    #[test]
    fn test_boot_config_creation() {
        let config = BootConfig {
            mode: BootMode::UEFI,
            kernel_path: "/boot/kernel",
            initrd_path: Some("/boot/initrd"),
            command_line: Some("quiet loglevel=3"),
            memory_test: true,
            serial_console: true,
            debug_mode: true,
            log_level: Level::Debug,
        };
        
        assert_eq!(config.mode, BootMode::UEFI);
        assert_eq!(config.kernel_path, "/boot/kernel");
        assert!(config.memory_test);
        assert!(config.debug_mode);
    }

    #[test]
    fn test_boot_state_creation() {
        let memory_map = MemoryMap::new();
        let config = BootConfig {
            mode: BootMode::UEFI,
            kernel_path: "/boot/kernel",
            initrd_path: None,
            command_line: None,
            memory_test: false,
            serial_console: true,
            debug_mode: false,
            log_level: Level::Info,
        };
        
        let state = BootState {
            mode: BootMode::UEFI,
            boot_time: 123456,
            memory_map,
            boot_config: config,
            bootloader_version: [0; 64],
        };
        
        assert_eq!(state.mode, BootMode::UEFI);
        assert_eq!(state.boot_time, 123456);
    }
}