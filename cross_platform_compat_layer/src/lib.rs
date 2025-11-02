//! MultiOS Cross-Platform Compatibility Layer
//! 
//! This module provides a unified abstraction layer for MultiOS to support
//! multiple architectures (x86_64, ARM64, RISC-V) with minimal code duplication.
//!
//! # Key Features
//!
//! - Unified device interface across all supported architectures
//! - Portable application framework
//! - Cross-platform driver abstraction
//! - Unified API layer for system calls and services
//! - Platform abstraction for applications
//! - Comprehensive compatibility testing framework

#![no_std]
#![allow(dead_code)]
#![allow(non_camel_case_types)]

extern crate spin;
extern crate bitflags;
extern crate log;

pub mod arch;
pub mod devices;
pub mod drivers;
pub mod framework;
pub mod api;
pub mod platform;
pub mod testing;

use core::sync::atomic::{AtomicU64, Ordering};
use spin::Mutex;
use log::{info, debug, warn, error};

/// Version information for the compatibility layer
pub const VERSION: &str = "1.0.0";
pub const VERSION_MAJOR: u16 = 1;
pub const VERSION_MINOR: u16 = 0;
pub const VERSION_PATCH: u16 = 0;

/// Global compatibility layer state
static COMPAT_STATE: Mutex<Option<CompatibilityState>> = Mutex::new(None);

/// Initialize the cross-platform compatibility layer
pub fn init(arch_type: ArchitectureType) -> Result<(), CompatibilityError> {
    let mut state = COMPAT_STATE.lock();
    
    if state.is_some() {
        return Err(CompatibilityError::AlreadyInitialized);
    }

    info!("Initializing MultiOS Cross-Platform Compatibility Layer v{}", VERSION);
    debug!("Target architecture: {:?}", arch_type);

    let compatibility_state = CompatibilityState::new(arch_type);
    
    // Initialize architecture-specific components
    arch::init(arch_type)?;
    
    // Initialize device abstraction layer
    devices::init()?;
    
    // Initialize driver interface
    drivers::init()?;
    
    // Initialize application framework
    framework::init()?;
    
    // Initialize API layer
    api::init()?;
    
    // Initialize platform abstraction
    platform::init()?;
    
    *state = Some(compatibility_state);
    
    info!("Cross-platform compatibility layer initialized successfully");
    Ok(())
}

/// Get current compatibility layer state
pub fn get_state() -> Option<&'static CompatibilityState> {
    COMPAT_STATE.lock().as_ref()
}

/// Current architecture type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ArchitectureType {
    X86_64,
    ARM64,
    RISCV64,
    Unknown,
}

impl ArchitectureType {
    /// Get string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            ArchitectureType::X86_64 => "x86_64",
            ArchitectureType::ARM64 => "aarch64",
            ArchitectureType::RISCV64 => "riscv64",
            ArchitectureType::Unknown => "unknown",
        }
    }
    
    /// Check if this architecture is supported
    pub fn is_supported(&self) -> bool {
        matches!(self, 
            ArchitectureType::X86_64 | 
            ArchitectureType::ARM64 | 
            ArchitectureType::RISCV64)
    }
}

/// Device class enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum DeviceClass {
    Processor,
    Memory,
    Storage,
    Network,
    Audio,
    Graphics,
    Input,
    USB,
    PCI,
    Serial,
    Unknown,
}

/// Architecture-specific features
#[derive(Debug, Clone)]
pub struct ArchitectureFeatures {
    pub has_fpu: bool,
    pub has_sse: bool,
    pub has_avx: bool,
    pub has_aes: bool,
    pub has_neon: bool,
    pub has_vector_extensions: bool,
    pub word_size: u8,
    pub pointer_size: u8,
    pub max_physical_address: u64,
    pub page_size: u64,
}

/// Compatibility layer global state
pub struct CompatibilityState {
    pub arch_type: ArchitectureType,
    pub features: ArchitectureFeatures,
    pub boot_time: AtomicU64,
    pub device_count: AtomicU32,
}

impl CompatibilityState {
    fn new(arch_type: ArchitectureType) -> Self {
        let features = ArchitectureFeatures::detect(arch_type);
        
        info!("Detected architecture features:");
        debug!("  FPU: {}", features.has_fpu);
        debug!("  SSE: {}", features.has_sse);
        debug!("  AVX: {}", features.has_avx);
        debug!("  Neon: {}", features.has_neon);
        debug!("  Word size: {} bits", features.word_size * 8);
        debug!("  Pointer size: {} bits", features.pointer_size * 8);
        debug!("  Max physical address: 0x{:016x}", features.max_physical_address);
        debug!("  Page size: {} bytes", features.page_size);

        CompatibilityState {
            arch_type,
            features,
            boot_time: AtomicU64::new(0),
            device_count: AtomicU32::new(0),
        }
    }
}

/// Error types for compatibility layer
#[derive(Debug)]
pub enum CompatibilityError {
    AlreadyInitialized,
    UnsupportedArchitecture,
    InitializationFailed(&'static str),
    DeviceNotFound,
    DriverNotCompatible,
    ApiNotAvailable,
}

impl core::fmt::Display for CompatibilityError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            CompatibilityError::AlreadyInitialized => {
                write!(f, "Compatibility layer already initialized")
            }
            CompatibilityError::UnsupportedArchitecture => {
                write!(f, "Unsupported architecture")
            }
            CompatibilityError::InitializationFailed(msg) => {
                write!(f, "Initialization failed: {}", msg)
            }
            CompatibilityError::DeviceNotFound => {
                write!(f, "Device not found")
            }
            CompatibilityError::DriverNotCompatible => {
                write!(f, "Driver not compatible with target architecture")
            }
            CompatibilityError::ApiNotAvailable => {
                write!(f, "API not available on this architecture")
            }
        }
    }
}

impl ArchitectureFeatures {
    fn detect(arch_type: ArchitectureType) -> Self {
        match arch_type {
            ArchitectureType::X86_64 => ArchitectureFeatures {
                has_fpu: true,
                has_sse: true,
                has_avx: true,
                has_aes: true,
                has_neon: false,
                has_vector_extensions: true,
                word_size: 8,
                pointer_size: 8,
                max_physical_address: 0x1FF_FFFF_FFFF_FFFF,
                page_size: 0x1000,
            },
            ArchitectureType::ARM64 => ArchitectureFeatures {
                has_fpu: true,
                has_sse: false,
                has_avx: false,
                has_aes: true,
                has_neon: true,
                has_vector_extensions: true,
                word_size: 8,
                pointer_size: 8,
                max_physical_address: 0x3_FFFF_FFFF_FFFF,
                page_size: 0x1000,
            },
            ArchitectureType::RISCV64 => ArchitectureFeatures {
                has_fpu: true,
                has_sse: false,
                has_avx: false,
                has_aes: false,
                has_neon: false,
                has_vector_extensions: false,
                word_size: 8,
                pointer_size: 8,
                max_physical_address: 0x1FF_FFFF_FFFF_FFFF,
                page_size: 0x1000,
            },
            ArchitectureType::Unknown => ArchitectureFeatures {
                has_fpu: false,
                has_sse: false,
                has_avx: false,
                has_aes: false,
                has_neon: false,
                has_vector_extensions: false,
                word_size: 8,
                pointer_size: 8,
                max_physical_address: 0,
                page_size: 0x1000,
            },
        }
    }
}