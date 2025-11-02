//! Architecture Abstraction Layer
//! 
//! This module provides unified interfaces for different CPU architectures
//! (x86_64, ARM64, RISC-V) to ensure consistent behavior across platforms.

use crate::{ArchitectureType, CompatibilityError, log};
use core::sync::atomic::{AtomicU32, Ordering};

/// Architecture-specific initialization
pub fn init(arch_type: ArchitectureType) -> Result<(), CompatibilityError> {
    log::debug!("Initializing architecture abstraction layer for {:?}", arch_type);
    
    match arch_type {
        ArchitectureType::X86_64 => x86_64::init(),
        ArchitectureType::ARM64 => arm64::init(),
        ArchitectureType::RISCV64 => riscv64::init(),
        ArchitectureType::Unknown => Err(CompatibilityError::UnsupportedArchitecture),
    }
}

/// Common CPU features across all architectures
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CpuFeatures {
    pub has_fpu: bool,
    pub has_simd: bool,
    pub has_crypto: bool,
    pub word_size: u8,
}

/// CPU information structure
#[derive(Debug, Clone)]
pub struct CpuInfo {
    pub vendor: &'static str,
    pub model_name: &'static str,
    pub features: CpuFeatures,
    pub max_frequency: u32,
    pub core_count: u32,
    pub thread_count: u32,
}

/// Generic CPU interface
pub trait CpuInterface {
    /// Get CPU information
    fn get_cpu_info() -> CpuInfo;
    
    /// Enable/disable interrupts
    fn set_interrupts_enabled(enabled: bool);
    
    /// Check if interrupts are enabled
    fn are_interrupts_enabled() -> bool;
    
    /// Get current privilege level
    fn get_privilege_level() -> PrivilegeLevel;
    
    /// Flush instruction cache
    fn flush_instruction_cache();
    
    /// Memory barrier instruction
    fn memory_barrier();
}

/// Privilege levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PrivilegeLevel {
    Ring0, // Kernel
    Ring1, // Reserved
    Ring2, // Reserved
    Ring3, // User
}

/// Memory management unit (MMU) interface
pub trait MemoryManagementUnit {
    /// Enable/disable paging
    fn set_paging_enabled(enabled: bool);
    
    /// Get current page directory root
    fn get_page_directory() -> usize;
    
    /// Set page directory root
    fn set_page_directory(root: usize);
    
    /// Invalidate TLB entry
    fn invalidate_tlb_entry(addr: usize);
    
    /// Invalidate entire TLB
    fn invalidate_tlb();
    
    /// Map virtual page to physical frame
    fn map_page(virt_addr: usize, phys_addr: usize, flags: PageFlags);
    
    /// Unmap virtual page
    fn unmap_page(virt_addr: usize);
}

/// Memory page flags
bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct PageFlags: u64 {
        const PRESENT = 0x001;
        const WRITEABLE = 0x002;
        const USER_ACCESSIBLE = 0x004;
        const WRITE_THROUGH = 0x008;
        const CACHE_DISABLED = 0x010;
        const ACCESSED = 0x020;
        const DIRTY = 0x040;
        const HUGE_PAGE = 0x080;
        const GLOBAL = 0x100;
        const EXECUTE_DISABLE = 0x200;
    }
}

/// Timer interface for architecture-specific timing
pub trait TimerInterface {
    /// Get current timer value
    fn get_time() -> u64;
    
    /// Get timer frequency in Hz
    fn get_frequency() -> u64;
    
    /// Sleep for specified microseconds
    fn sleep_us(microseconds: u64);
    
    /// Busy wait for specified nanoseconds
    fn busy_wait_ns(nanoseconds: u64);
}

/// Interrupt controller interface
pub trait InterruptController {
    /// Enable/disable interrupt
    fn set_interrupt_enabled(irq: u8, enabled: bool);
    
    /// Get interrupt pending status
    fn is_interrupt_pending(irq: u8) -> bool;
    
    /// Send end-of-interrupt signal
    fn send_eoi(irq: u8);
    
    /// Get number of available IRQ lines
    fn get_irq_count() -> u8;
}

/// Global CPU and architecture state
static CURRENT_CPU: spin::Mutex<Option<ArchitectureCpu>> = spin::Mutex::new(None);

/// Architecture-specific CPU state
#[derive(Debug, Clone)]
pub struct ArchitectureCpu {
    pub cpu_info: CpuInfo,
    pub arch_type: ArchitectureType,
    pub features: CpuFeatures,
}

impl ArchitectureCpu {
    fn new(arch_type: ArchitectureType) -> Self {
        let (cpu_info, features) = match arch_type {
            ArchitectureType::X86_64 => x86_64::get_cpu_info(),
            ArchitectureType::ARM64 => arm64::get_cpu_info(),
            ArchitectureType::RISCV64 => riscv64::get_cpu_info(),
            ArchitectureType::Unknown => unreachable!(),
        };

        ArchitectureCpu {
            cpu_info,
            arch_type,
            features,
        }
    }
}

/// Set current CPU information
pub fn set_current_cpu(arch_type: ArchitectureType) {
    let mut cpu_lock = CURRENT_CPU.lock();
    *cpu_lock = Some(ArchitectureCpu::new(arch_type));
}

/// Get current CPU information
pub fn get_current_cpu() -> Option<&'static ArchitectureCpu> {
    CURRENT_CPU.lock().as_ref()
}

// Architecture-specific implementations

#[cfg(target_arch = "x86_64")]
mod x86_64 {
    use super::*;
    use crate::{CompatibilityError, log};

    pub fn init() -> Result<(), CompatibilityError> {
        log::info!("Initializing x86_64 architecture support");
        
        // Initialize CPU features detection
        detect_cpu_features();
        
        // Initialize interrupt controller (APIC)
        init_apic();
        
        Ok(())
    }

    pub fn get_cpu_info() -> (CpuInfo, CpuFeatures) {
        // Simplified CPU detection - in real implementation would query CPUID
        let features = CpuFeatures {
            has_fpu: true,
            has_simd: true,
            has_crypto: true,
            word_size: 8,
        };

        let cpu_info = CpuInfo {
            vendor: "GenuineIntel",
            model_name: "MultiOS x86_64 CPU",
            features,
            max_frequency: 3000000, // 3GHz
            core_count: 4,
            thread_count: 8,
        };

        (cpu_info, features)
    }

    fn detect_cpu_features() {
        log::debug!("Detecting x86_64 CPU features");
        // CPUID detection would go here
    }

    fn init_apic() {
        log::debug!("Initializing APIC interrupt controller");
        // APIC initialization would go here
    }
}

#[cfg(target_arch = "aarch64")]
mod arm64 {
    use super::*;
    use crate::{CompatibilityError, log};

    pub fn init() -> Result<(), CompatibilityError> {
        log::info!("Initializing ARM64 architecture support");
        
        // Initialize GIC interrupt controller
        init_gic();
        
        // Initialize ARM-specific MMU features
        init_mmu();
        
        Ok(())
    }

    pub fn get_cpu_info() -> (CpuInfo, CpuFeatures) {
        let features = CpuFeatures {
            has_fpu: true,
            has_simd: true,
            has_crypto: true,
            word_size: 8,
        };

        let cpu_info = CpuInfo {
            vendor: "ARM",
            model_name: "MultiOS ARM64 CPU",
            features,
            max_frequency: 2000000, // 2GHz
            core_count: 8,
            thread_count: 8,
        };

        (cpu_info, features)
    }

    fn init_gic() {
        log::debug!("Initializing GIC interrupt controller");
        // GIC initialization would go here
    }

    fn init_mmu() {
        log::debug!("Initializing ARM MMU");
        // ARM MMU initialization would go here
    }
}

#[cfg(target_arch = "riscv64")]
mod riscv64 {
    use super::*;
    use crate::{CompatibilityError, log};

    pub fn init() -> Result<(), CompatibilityError> {
        log::info!("Initializing RISC-V64 architecture support");
        
        // Initialize CLINT/PLIC interrupt controller
        init_interrupts();
        
        // Initialize RISC-V MMU
        init_mmu();
        
        Ok(())
    }

    pub fn get_cpu_info() -> (CpuInfo, CpuFeatures) {
        let features = CpuFeatures {
            has_fpu: true,
            has_simd: false, // RISC-V doesn't have traditional SIMD
            has_crypto: false,
            word_size: 8,
        };

        let cpu_info = CpuInfo {
            vendor: "RISC-V",
            model_name: "MultiOS RISC-V64 CPU",
            features,
            max_frequency: 1500000, // 1.5GHz
            core_count: 4,
            thread_count: 4,
        };

        (cpu_info, features)
    }

    fn init_interrupts() {
        log::debug!("Initializing RISC-V interrupt controller");
        // CLINT/PLIC initialization would go here
    }

    fn init_mmu() {
        log::debug!("Initializing RISC-V MMU");
        // RISC-V MMU initialization would go here
    }
}

// Export common types
pub use crate::CpuInterface;
pub use crate::MemoryManagementUnit;
pub use crate::TimerInterface;
pub use crate::InterruptController;
pub use crate::CpuInfo;
pub use crate::CpuFeatures;