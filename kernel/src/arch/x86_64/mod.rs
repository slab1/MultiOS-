//! x86_64 Architecture-Specific Module
//! 
//! This module provides x86_64 specific functionality and initialization.

use crate::log::{info, warn, error};
use crate::KernelError;
use crate::ArchType;

/// Desktop PC support module
pub mod desktop_pc;

/// x86_64 specific initialization
pub fn init() -> Result<(), KernelError> {
    info!("Initializing x86_64 architecture support...");
    
    // Initialize interrupt system
    crate::arch::interrupts::init_interrupt_system(ArchType::X86_64)?;
    
    info!("x86_64 architecture initialization complete");
    Ok(())
}

/// Get x86_64 specific CPU information
pub fn get_cpu_info() -> crate::arch::CpuInfo {
    crate::arch::CpuInfo {
        vendor: "GenuineIntel", // Placeholder
        model: "Unknown x86_64 CPU",
        family: 6,
        model_id: 0,
        stepping: 0,
        frequency_mhz: 2400,
        cores: 1,
        threads_per_core: 1,
    }
}

/// Get x86_64 specific system configuration
pub fn get_system_config() -> crate::arch::SystemConfig {
    crate::arch::SystemConfig {
        page_size: 4096,
        max_phys_addr: 0xFFFFFFFFFFFFF000, // 48-bit physical addressing
        max_virt_addr: 0xFFFFFFFFFFFFFFFF, // 64-bit virtual addressing
        pointer_size: 8,
        endianness: crate::arch::Endianness::Little,
        interrupt_controller: crate::arch::InterruptController::Apic,
    }
}

/// Get x86_64 CPU features
pub fn get_cpu_features() -> crate::arch::CpuFeatures {
    // This would be populated by CPUID instruction in a real implementation
    crate::arch::CpuFeatures {
        sse: true,
        sse2: true,
        sse3: true,
        sse4_1: true,
        sse4_2: true,
        avx: true,
        avx2: true,
        fma: true,
        nx_bit: true,
        pae: true,
        la57: true,
        smep: true,
        smap: true,
        pge: true,
    }
}

/// Check if running in 64-bit mode
pub fn is_long_mode() -> bool {
    let mut cr0: u64;
    let mut cr4: u64;
    
    unsafe {
        core::arch::asm!(
            "mov %cr0, {}",
            "mov %cr4, {}",
            out(reg) cr0,
            out(reg) cr4
        );
    }
    
    // Long mode requires:
    // - CR0.PE = 1 (protected mode)
    // - CR0.PG = 1 (paging enabled)
    // - CR4.PAE = 1 (physical address extensions)
    // - EFER.LME = 1 (long mode enabled)
    
    (cr0 & 1) != 0 && (cr0 & (1 << 31)) != 0 && (cr4 & (1 << 5)) != 0
}

/// Get current privilege level
pub fn get_current_privilege_level() -> u8 {
    // CPL is in bits 0-1 of CS register
    let cs: u64;
    unsafe {
        core::arch::asm!(
            "mov %cs, {}",
            out(reg) cs
        );
    }
    
    (cs & 3) as u8
}

/// Check if interrupts are enabled
pub fn are_interrupts_enabled() -> bool {
    let rflags: u64;
    unsafe {
        core::arch::asm!(
            "pushfq",
            "pop {}",
            out(reg) rflags
        );
    }
    
    (rflags & (1 << 9)) != 0
}

/// Enable hardware interrupts
pub fn enable_interrupts() {
    unsafe {
        core::arch::asm!("sti");
    }
}

/// Disable hardware interrupts
pub fn disable_interrupts() {
    unsafe {
        core::arch::asm!("cli");
    }
}

/// Get current page table root
pub fn get_page_table_root() -> usize {
    let cr3: u64;
    unsafe {
        core::arch::asm!(
            "mov %cr3, {}",
            out(reg) cr3
        );
    }
    
    cr3 as usize
}

/// Set page table root
pub fn set_page_table_root(root: usize) {
    unsafe {
        core::arch::asm!(
            "mov {}, %cr3",
            in(reg) root as u64
        );
    }
}

/// Read control register
fn read_cr(reg: u8) -> u64 {
    let mut value: u64;
    unsafe {
        match reg {
            0 => core::arch::asm!("mov %cr0, {}", out(reg) value),
            3 => core::arch::asm!("mov %cr3, {}", out(reg) value),
            4 => core::arch::asm!("mov %cr4, {}", out(reg) value),
            8 => core::arch::asm!("mov %cr8, {}", out(reg) value),
            _ => return 0,
        }
    }
    value
}

/// Write control register
fn write_cr(reg: u8, value: u64) {
    unsafe {
        match reg {
            0 => core::arch::asm!("mov {}, %cr0", in(reg) value),
            3 => core::arch::asm!("mov {}, %cr3", in(reg) value),
            4 => core::arch::asm!("mov {}, %cr4", in(reg) value),
            8 => core::arch::asm!("mov {}, %cr8", in(reg) value),
            _ => return,
        }
    }
}

/// Halt the CPU until next interrupt
pub fn halt() {
    unsafe {
        core::arch::asm!("hlt");
    }
}

/// Flush TLB (Translation Lookaside Buffer)
pub fn flush_tlb() {
    unsafe {
        core::arch::asm!(
            "mov %cr3, %rax",
            "mov %rax, %cr3"
        );
    }
}

/// Flush TLB for specific page
pub fn flush_tlb_page(address: usize) {
    // INVLPG instruction flushes TLB entry for specific virtual address
    unsafe {
        core::arch::asm!(
            "invlpg [{}]",
            in(reg) address
        );
    }
}

/// Get timestamp counter
pub fn get_tsc() -> u64 {
    let tsc_low: u32;
    let tsc_high: u32;
    
    unsafe {
        core::arch::asm!(
            "rdtsc",
            "mov %edx, {}",
            "mov %eax, {}",
            out(reg) tsc_high,
            out(reg) tsc_low
        );
    }
    
    ((tsc_high as u64) << 32) | (tsc_low as u64)
}

/// Store floating point state
pub fn fnsave(address: usize) {
    unsafe {
        core::arch::asm!(
            "fnsave [{}]",
            in(reg) address
        );
    }
}

/// Restore floating point state
pub fn frstor(address: usize) {
    unsafe {
        core::arch::asm!(
            "frstor [{}]",
            in(reg) address
        );
    }
}

/// Initialize x86_64 specific subsystem
pub mod subsystem {
    use super::*;
    
    /// Initialize performance monitoring
    pub fn init_performance_monitoring() -> Result<(), KernelError> {
        info!("Initializing x86_64 performance monitoring...");
        
        // Enable performance monitoring counters if available
        // This would involve MSR access in a real implementation
        
        Ok(())
    }
    
    /// Initialize debugging support
    pub fn init_debugging() -> Result<(), KernelError> {
        info!("Initializing x86_64 debugging support...");
        
        // Set up debug registers DR0-DR3 and DR7
        // Enable breakpoints and watchpoints
        
        Ok(())
    }
    
    /// Initialize thermal management
    pub fn init_thermal_management() -> Result<(), KernelError> {
        info!("Initializing x86_64 thermal management...");
        
        // Set up thermal monitoring and management
        
        Ok(())
    }
    
    /// Initialize power management
    pub fn init_power_management() -> Result<(), KernelError> {
        info!("Initializing x86_64 power management...");
        
        // Configure ACPI and power states
        
        Ok(())
    }
}

/// Desktop PC support module
pub mod desktop_pc {
    pub use super::desktop_pc::*;
    
    /// Initialize comprehensive desktop PC support
    pub fn init_desktop_pc() -> Result<(), KernelError> {
        use crate::log::info;
        info!("Initializing x86_64 Desktop PC support...");
        
        // This would call the comprehensive desktop PC initialization
        // For now, just log that it's available
        
        info!("x86_64 Desktop PC support initialized");
        Ok(())
    }
}