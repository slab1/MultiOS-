//! MultiOS Kernel
//! 
//! A hybrid microkernel for MultiOS with interrupt handling,
//! system calls, and multi-architecture support.

#![no_std]
#![feature(asm)]
#![feature(alloc)]
#![feature(const_fn)]
#![feature(core_intrinsics)]
#![feature(custom_test_frameworks)]
#![test_runner(test_runner)]
#![allow(dead_code)]

extern crate alloc;
extern crate x86_64;
extern crate spin;
extern crate log;
extern crate bootloader;
extern crate pic8259;

use alloc::vec::Vec;
use spin::Mutex;
use log::{info, warn, error, debug};

// Architecture-specific modules
pub mod arch;

// Core kernel modules
pub mod interrupts;
pub mod syscall;
pub mod memory;
pub mod scheduler;
pub mod drivers;
pub mod ipc;
pub mod filesystem;
pub mod serial;
pub mod exception;

// GUI module
pub mod gui;

// Version information
const KERNEL_VERSION: &str = "0.1.0";
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
    InterruptInitFailed,
    SyscallInitFailed,
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

/// Initialize the kernel
/// 
/// This is the main entry point for kernel initialization.
pub fn kernel_main(arch: ArchType, boot_info: &BootInfo) -> KernelResult<()> {
    let mut kernel_guard = KERNEL_STATE.lock();
    
    if kernel_guard.is_some() {
        return Err(KernelError::AlreadyInitialized);
    }
    
    info!("Initializing {} kernel version {}", KERNEL_NAME, KERNEL_VERSION);
    info!("Architecture: {:?}", arch);
    
    // Initialize architecture-specific code
    info!("Initializing architecture-specific code...");
    arch::init(arch, boot_info)?;
    
    // Initialize memory management
    info!("Initializing memory management...");
    memory::init(boot_info)
        .map_err(|e| {
            error!("Memory initialization failed: {:?}", e);
            KernelError::MemoryInitFailed
        })?;
    
    // Initialize interrupt handling
    info!("Initializing interrupt handling...");
    interrupts::init()
        .map_err(|e| {
            error!("Interrupt initialization failed: {:?}", e);
            KernelError::InterruptInitFailed
        })?;
    
    // Initialize system call interface
    info!("Initializing system call interface...");
    syscall::init()
        .map_err(|e| {
            error!("System call initialization failed: {:?}", e);
            KernelError::SyscallInitFailed
        })?;
    
    // Initialize scheduler
    info!("Initializing scheduler...");
    scheduler::init()
        .map_err(|e| {
            error!("Scheduler initialization failed: {:?}", e);
            KernelError::SchedulerInitFailed
        })?;
    
    // Initialize device drivers
    info!("Initializing device drivers...");
    drivers::init()
        .map_err(|e| {
            error!("Driver initialization failed: {:?}", e);
            KernelError::DriverInitFailed
        })?;
    
    // Initialize IPC system
    info!("Initializing IPC system...");
    ipc::init()?;
    
    // Initialize file system
    info!("Initializing file system...");
    filesystem::init()?;
    
    // Initialize GUI system
    info!("Initializing GUI system...");
    gui::init()
        .map_err(|e| {
            error!("GUI initialization failed: {:?}", e);
            KernelError::DriverInitFailed
        })?;
    
    // Create kernel state
    let kernel_state = KernelState {
        initialized: true,
        boot_time: boot_info.boot_time,
        architecture: arch,
        version: KERNEL_VERSION.to_string(),
        memory_stats: memory::get_memory_stats(),
    };
    
    *kernel_guard = Some(kernel_state);
    
    info!("{} kernel initialization complete!", KERNEL_NAME);
    
    Ok(())
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

/// Get system information
pub fn get_system_info() -> KernelResult<SystemInfo> {
    let kernel_state = get_kernel_state()?;
    
    Ok(SystemInfo {
        kernel_name: KERNEL_NAME.to_string(),
        kernel_version: kernel_state.version,
        architecture: kernel_state.architecture,
        boot_time: kernel_state.boot_time,
        memory_total: kernel_state.memory_stats.total_pages * 4096,
        memory_used: kernel_state.memory_stats.used_pages * 4096,
        memory_available: kernel_state.memory_stats.available_pages * 4096,
    })
}

// Test runner for kernel tests
fn test_runner(tests: &[&dyn Fn()]) {
    info!("Running kernel tests...");
    for test in tests {
        test();
    }
    info!("All kernel tests passed!");
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
