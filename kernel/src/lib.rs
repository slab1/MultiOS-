//! MultiOS Hybrid Microkernel
//! 
//! This is the main kernel crate for the MultiOS operating system.
//! It implements a hybrid microkernel architecture with support for
//! multiple architectures and device types.

#![no_std]
#![feature(alloc)]
#![feature(asm)]
#![feature(const_fn)]
#![feature(core_intrinsics)]

// Core kernel modules
pub mod bootstrap;
pub mod memory;
pub mod scheduler;
pub mod drivers;
pub mod ipc;
pub mod filesystem;
pub mod arch;
pub mod syscall; // System call interface

// Hardware Abstraction Layer
pub mod hal;

// System Services
pub mod services;

pub mod log; // Simple bootstrap logger

// Fonts and text rendering
pub mod fonts;

use alloc::vec::Vec;
use spin::Mutex;
use log::{info, warn, error};

// Version information
const KERNEL_VERSION: &str = env!("CARGO_PKG_VERSION");
const KERNEL_NAME: &str = "MultiOS";

// Global kernel state
static KERNEL_STATE: Mutex<Option<KernelState>> = Mutex::new(None);

/// Main kernel state structure
#[derive(Debug)]
pub struct KernelState {
    pub initialized: bool,
    pub boot_time: u64,
    pub architecture: ArchType,
    pub version: String,
    pub memory_stats: memory::MemoryStats,
    pub interrupt_stats: arch::interrupts::InterruptStats,
}

/// Supported architectures
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ArchType {
    X86_64 = 0,
    AArch64 = 1,
    Riscv64 = 2,
    Unknown = 255,
}

/// Kernel initialization result
pub type KernelResult<T> = Result<T, KernelError>;
pub type Result<T> = core::result::Result<T, KernelError>;

/// Error types for kernel operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelError {
    AlreadyInitialized,
    InitializationFailed,
    NotInitialized,
    UnsupportedArchitecture,
    MemoryInitFailed,
    SchedulerInitFailed,
    DriverInitFailed,
    InvalidConfig,
    FeatureNotSupported,
    CapabilityNotAvailable,
    // HAL-specific errors
    NoTimerAvailable,
    UnsupportedInterruptController,
    InvalidParameter,
    InvalidAddress,
    OutOfMemory,
    // Graphics-specific errors
    DeviceNotFound,
    InvalidMode,
    ResourceBusy,
    NotSupported,
    GraphicsInitFailed,
    FramebufferInitFailed,
    FontLoadFailed,
    AlreadyExists,
    NotFound,
}

/// Initialize the kernel
/// 
/// This is the main entry point for kernel initialization.
/// It must be called from the bootloader with appropriate parameters.
pub fn kernel_main(arch: ArchType, boot_info: &BootInfo, boot_method: bootstrap::BootMethod) -> KernelResult<()> {
    let mut kernel_guard = KERNEL_STATE.lock();
    
    if kernel_guard.is_some() {
        return Err(KernelError::AlreadyInitialized);
    }
    
    info!("Initializing {} kernel version {}", KERNEL_NAME, KERNEL_VERSION);
    info!("Architecture: {:?}", arch);
    
    // Initialize bootstrap configuration
    let config = bootstrap::BootstrapConfig {
        architecture: arch,
        boot_method,
        enable_debug: true,
        enable_logging: true,
        memory_test: false,
        recovery_mode: true,
    };
    
    // Initialize bootstrap system
    let mut bootstrap_context = bootstrap::init_bootstrap(config, *boot_info)?;
    
    // Execute complete bootstrap sequence
    bootstrap::execute_bootstrap(bootstrap_context)?;
    
    // Initialize Hardware Abstraction Layer
    info!("Initializing Hardware Abstraction Layer...");
    hal::init()
        .map_err(|e| {
            error!("HAL initialization failed: {:?}", e);
            KernelError::InitializationFailed
        })?;
    
    // Initialize architecture support
    info!("Initializing architecture support...");
    arch::init()
        .map_err(|e| {
            error!("Architecture initialization failed: {:?}", e);
            KernelError::InitializationFailed
        })?;
    
    // Initialize interrupt handling system
    info!("Initializing interrupt handling system...");
    crate::arch::interrupts::init_interrupt_system(arch)
        .map_err(|e| {
            error!("Interrupt system initialization failed: {:?}", e);
            KernelError::InitializationFailed
        })?;
    
    // Initialize System Services
    info!("Initializing System Services...");
    services::init()
        .map_err(|e| {
            error!("System Services initialization failed: {:?}", e);
            KernelError::InitializationFailed
        })?;
    
    // Create kernel state
    let kernel_state = KernelState {
        initialized: true,
        boot_time: boot_info.boot_time,
        architecture: arch,
        version: KERNEL_VERSION.to_string(),
        memory_stats: memory::get_memory_stats(),
        interrupt_stats: arch::interrupts::InterruptStats {
            total_interrupts: 0,
            exceptions: 0,
            hardware_interrupts: 0,
            system_calls: 0,
            software_interrupts: 0,
            last_interrupt: 0,
            interrupt_rate: 0.0,
        },
    };

    // System call handler would be initialized here for user processes
    // Note: Global system call handler initialization deferred until user processes are created
    
    *kernel_guard = Some(kernel_state);
    
    info!("{} kernel initialization complete!", KERNEL_NAME);
    
    // Start the scheduler and begin normal operation
    start_main_scheduler()?;
    
    Ok(())
}

/// 64-bit kernel entry point for Multiboot2
/// 
/// This function serves as the entry point when transitioning to long mode.
/// It receives the boot information pointer as the first argument.
#[no_mangle]
pub extern "C" fn kernel_main_64bit(boot_info_ptr: *const BootInfo) -> ! {
    // Initialize bootstrap logger first
    log::init_logger(log::LogLevel::Info);
    
    log::info!("64-bit kernel starting with Multiboot2 boot info");
    
    if boot_info_ptr.is_null() {
        log::error!("Null boot information pointer");
        bootstrap::panic_handler::bootstrap_panic_handler(
            &core::panic::PanicInfo::new(
                Some("Null boot information pointer"),
                core::panic::Location::new(file!(), line!(), column!()),
            ),
            None,
        );
    }
    
    unsafe {
        let boot_info = &*boot_info_ptr;
        log::info!("Boot time: {}", boot_info.boot_time);
        
        if let Some(cmdline) = boot_info.command_line {
            log::info!("Command line: {}", cmdline);
        }
        
        log::info!("Memory map entries: {}", boot_info.memory_map.len());
        log::info!("Modules: {}", boot_info.modules.len());
        
        // Initialize kernel with detected architecture and boot method
        let arch = ArchType::X86_64; // Assume x86_64 for now
        let boot_method = bootstrap::BootMethod::Multiboot2;
        
        match kernel_main(arch, boot_info, boot_method) {
            Ok(_) => log::info!("Kernel initialized successfully"),
            Err(e) => {
                log::error!("Kernel initialization failed: {:?}", e);
                bootstrap::panic_handler::bootstrap_panic_handler(
                    &core::panic::PanicInfo::new(
                        Some(&format!("Kernel initialization failed: {:?}", e)),
                        core::panic::Location::new(file!(), line!(), column!()),
                    ),
                    None,
                );
            }
        }
    }
    
    // Main kernel loop (should not return)
    kernel_main_loop()
}

/// Main kernel loop after initialization
fn kernel_main_loop() -> ! {
    info!("Entering main kernel loop...");
    
    loop {
        // Main kernel processing loop
        // This would handle system calls, interrupts, etc.
        
        // For now, just halt and wait for interrupts
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}

/// Get the current kernel state
pub fn get_kernel_state() -> KernelResult<KernelState> {
    let kernel_guard = KERNEL_STATE.lock();
    
    kernel_guard
        .as_ref()
        .copied()
        .ok_or(KernelError::NotInitialized)
}

/// Check if kernel is initialized
pub fn is_initialized() -> bool {
    let kernel_guard = KERNEL_STATE.lock();
    kernel_guard.is_some() && kernel_guard.as_ref().unwrap().initialized
}

/// Start the main scheduler loop
fn start_main_scheduler() -> KernelResult<()> {
    info!("Starting main scheduler loop...");
    
    loop {
        // This would normally be implemented by architecture-specific code
        // that switches between threads based on the scheduling algorithm
        
        // For now, we'll just yield the CPU
        scheduler::yield_current_thread();
        
        // In a real implementation, this would:
        // 1. Save current thread state
        // 2. Select next thread to run
        // 3. Restore next thread state
        // 4. Switch to next thread
        // 
        // For now, we just break to avoid infinite loop in single-threaded tests
        break;
    }
    
    Ok(())
}

/// Boot information passed from bootloader
#[derive(Debug, Clone, Copy)]
pub struct BootInfo {
    pub boot_time: u64,
    pub memory_map: Vec<MemoryMapEntry>,
    pub command_line: Option<&'static str>,
    pub modules: Vec<BootModule>,
    pub framebuffer: Option<FramebufferInfo>,
}

/// Memory map entry
#[derive(Debug, Clone, Copy)]
pub struct MemoryMapEntry {
    pub base: u64,
    pub size: u64,
    pub entry_type: MemoryType,
}

/// Boot module information
#[derive(Debug, Clone, Copy)]
pub struct BootModule {
    pub start: u64,
    pub end: u64,
    pub cmdline: Option<&'static str>,
}

/// Framebuffer information
#[derive(Debug, Clone, Copy)]
pub struct FramebufferInfo {
    pub address: u64,
    pub pitch: u32,
    pub width: u32,
    pub height: u32,
    pub bpp: u8,
}

/// Memory types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MemoryType {
    Usable = 1,
    Reserved = 2,
    AcpiReclaimable = 3,
    AcpiNvs = 4,
    BadMemory = 5,
    BootloaderReclaimable = 0x100,
    KernelAndModules = 0x101,
}

/// Get system information
pub fn get_system_info() -> KernelResult<SystemInfo> {
    let kernel_state = get_kernel_state()?;
    
    Ok(SystemInfo {
        kernel_name: KERNEL_NAME.to_string(),
        kernel_version: kernel_state.version,
        architecture: kernel_state.architecture,
        boot_time: kernel_state.boot_time,
        memory_total: kernel_state.memory_stats.total_pages * 4096, // Assume 4KB pages
        memory_used: kernel_state.memory_stats.used_pages * 4096,
        memory_available: kernel_state.memory_stats.available_pages * 4096,
    })
}

/// System information structure
#[derive(Debug, Clone)]
pub struct SystemInfo {
    pub kernel_name: String,
    pub kernel_version: String,
    pub architecture: ArchType,
    pub boot_time: u64,
    pub memory_total: usize,
    pub memory_used: usize,
    pub memory_available: usize,
}

/// Panic handler
#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    bootstrap::panic_handler::bootstrap_panic_handler(info, None);
}

/// Exception handler
#[no_mangle]
pub extern "C" fn exception_handler() {
    error!("Unhandled exception occurred!");
    
    // Use bootstrap panic handler for exceptions during early boot
    bootstrap::panic_handler::bootstrap_panic_handler(
        &core::panic::PanicInfo::new(
            Some("Unhandled exception occurred"),
            core::panic::Location::new(file!(), line!(), column!()),
        ),
        None,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kernel_state_creation() {
        let state = KernelState {
            initialized: false,
            boot_time: 1000,
            architecture: ArchType::X86_64,
            version: "1.0.0".to_string(),
            memory_stats: memory::MemoryStats {
                total_pages: 1024,
                used_pages: 256,
                available_pages: 768,
            },
        };
        
        assert!(!state.initialized);
        assert_eq!(state.architecture, ArchType::X86_64);
        assert_eq!(state.memory_stats.total_pages, 1024);
    }

    #[test]
    fn test_memory_type_ordering() {
        assert_eq!(MemoryType::Usable as u8, 1);
        assert_eq!(MemoryType::Reserved as u8, 2);
        assert_eq!(MemoryType::KernelAndModules as u8, 0x101);
    }

    #[test]
    fn test_arch_type_ordering() {
        assert_eq!(ArchType::X86_64 as u8, 0);
        assert_eq!(ArchType::Unknown as u8, 255);
    }
}