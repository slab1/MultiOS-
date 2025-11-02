//! MultiOS Hardware Abstraction Layer (HAL)
//!
//! This module provides a comprehensive hardware abstraction layer that unifies
//! access to hardware components across different architectures (x86_64, ARM64, RISC-V).
//!
//! The HAL provides safe, unified interfaces for:
//! - CPU features and detection
//! - Memory management
//! - Interrupt controllers
//! - Timers
//! - I/O operations
//! - Multi-core support
//! - NUMA configurations

use crate::log::{info, warn, error};
use crate::{KernelError, Result};

pub mod cpu;
pub mod memory;
pub mod interrupts;
pub mod timers;
pub mod io;
pub mod multicore;
pub mod numa;

/// HAL initialization
pub fn init() -> Result<()> {
    info!("Initializing Hardware Abstraction Layer...");
    
    // Initialize subsystem in order
    cpu::init()?;
    memory::init()?;
    interrupts::init()?;
    timers::init()?;
    io::init()?;
    multicore::init()?;
    numa::init()?;
    
    info!("HAL initialization complete");
    Ok(())
}

/// HAL shutdown
pub fn shutdown() -> Result<()> {
    info!("Shutting down Hardware Abstraction Layer...");
    
    // Shutdown subsystem in reverse order
    numa::shutdown()?;
    multicore::shutdown()?;
    io::shutdown()?;
    timers::shutdown()?;
    interrupts::shutdown()?;
    memory::shutdown()?;
    cpu::shutdown()?;
    
    info!("HAL shutdown complete");
    Ok(())
}

/// HAL version information
#[derive(Debug, Clone, Copy)]
pub struct HalVersion {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
    pub build: u32,
}

impl HalVersion {
    pub const fn new(major: u16, minor: u16, patch: u16, build: u32) -> Self {
        Self { major, minor, patch, build }
    }
    
    pub fn to_string(&self) -> String {
        format!("{}.{}.{}-{}", self.major, self.minor, self.patch, self.build)
    }
}

/// Current HAL version
pub const HAL_VERSION: HalVersion = HalVersion::new(1, 0, 0, 1);

/// HAL capabilities
#[derive(Debug, Clone)]
pub struct HalCapabilities {
    pub supports_smp: bool,
    pub supports_numa: bool,
    pub supports_virt: bool,
    pub max_cpus: usize,
    pub max_memory_gb: usize,
    pub supported_features: Vec<String>,
}

/// System state information
#[derive(Debug, Clone)]
pub struct SystemState {
    pub uptime_ticks: u64,
    pub interrupts_count: u64,
    pub context_switches: u64,
    pub page_faults: u64,
}

/// Panic handler
pub fn panic_handler(info: &str, location: &str, registers: &[u64]) {
    error!("HAL PANIC: {}", info);
    error!("Location: {}", location);
    error!("Architecture-specific registers:");
    
    // Architecture-specific panic handling
    #[cfg(target_arch = "x86_64")]
    {
        error!("RAX: {:#x} RBX: {:#x} RCX: {:#x} RDX: {:#x}", 
               registers.get(0).unwrap_or(&0), registers.get(1).unwrap_or(&0),
               registers.get(2).unwrap_or(&0), registers.get(3).unwrap_or(&0));
        error!("RSI: {:#x} RDI: {:#x} RBP: {:#x} RSP: {:#x}", 
               registers.get(4).unwrap_or(&0), registers.get(5).unwrap_or(&0),
               registers.get(6).unwrap_or(&0), registers.get(7).unwrap_or(&0));
    }
    
    #[cfg(target_arch = "aarch64")]
    {
        error!("X0: {:#x} X1: {:#x} X2: {:#x} X3: {:#x}", 
               registers.get(0).unwrap_or(&0), registers.get(1).unwrap_or(&0),
               registers.get(2).unwrap_or(&0), registers.get(3).unwrap_or(&0));
        error!("X4: {:#x} X5: {:#x} X6: {:#x} X7: {:#x}", 
               registers.get(4).unwrap_or(&0), registers.get(5).unwrap_or(&0),
               registers.get(6).unwrap_or(&0), registers.get(7).unwrap_or(&0));
    }
    
    #[cfg(target_arch = "riscv64")]
    {
        error!("A0: {:#x} A1: {:#x} A2: {:#x} A3: {:#x}", 
               registers.get(0).unwrap_or(&0), registers.get(1).unwrap_or(&0),
               registers.get(2).unwrap_or(&0), registers.get(3).unwrap_or(&0));
        error!("A4: {:#x} A5: {:#x} A6: {:#x} A7: {:#x}", 
               registers.get(4).unwrap_or(&0), registers.get(5).unwrap_or(&0),
               registers.get(6).unwrap_or(&0), registers.get(7).unwrap_or(&0));
    }
    
    // Halt the system
    cpu::halt_cpu();
}

/// Architecture type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ArchType {
    X86_64 = 0,
    AArch64 = 1,
    Riscv64 = 2,
}

/// Get current architecture type
pub fn get_arch_type() -> ArchType {
    #[cfg(target_arch = "x86_64")]
    return ArchType::X86_64;
    
    #[cfg(target_arch = "aarch64")]
    return ArchType::AArch64;
    
    #[cfg(target_arch = "riscv64")]
    return ArchType::Riscv64;
}

/// HAL statistics
pub struct HalStats {
    pub cpu_stats: cpu::CpuStats,
    pub memory_stats: memory::MemoryStats,
    pub interrupt_stats: interrupts::InterruptStats,
    pub timer_stats: timers::TimerStats,
    pub io_stats: io::IoStats,
}

/// Get HAL statistics
pub fn get_stats() -> HalStats {
    HalStats {
        cpu_stats: cpu::get_stats(),
        memory_stats: memory::get_stats(),
        interrupt_stats: interrupts::get_stats(),
        timer_stats: timers::get_stats(),
        io_stats: io::get_stats(),
    }
}

/// HAL benchmark operations
pub mod benchmark {
    use super::*;
    
    /// Benchmark CPU performance
    pub fn cpu_performance_test() -> Result<u64> {
        Ok(cpu::benchmark_cpu())
    }
    
    /// Benchmark memory performance
    pub fn memory_performance_test() -> Result<u64> {
        Ok(memory::benchmark_memory())
    }
    
    /// Benchmark interrupt latency
    pub fn interrupt_latency_test() -> Result<u64> {
        Ok(interrupts::benchmark_latency())
    }
    
    /// Comprehensive system benchmark
    pub fn system_performance_test() -> Result<SystemBenchmarkResult> {
        let cpu_result = cpu_performance_test()?;
        let memory_result = memory_performance_test()?;
        let interrupt_result = interrupt_latency_test()?;
        
        Ok(SystemBenchmarkResult {
            cpu_score: cpu_result,
            memory_score: memory_result,
            interrupt_latency_ns: interrupt_result,
        })
    }
}

/// System benchmark result
#[derive(Debug, Clone)]
pub struct SystemBenchmarkResult {
    pub cpu_score: u64,
    pub memory_score: u64,
    pub interrupt_latency_ns: u64,
}

/// Get current time in milliseconds since system boot
pub fn get_current_time() -> u64 {
    crate::hal::timers::get_system_time_ms()
}

/// Get current time in microseconds since system boot
pub fn get_current_time_us() -> u64 {
    crate::hal::timers::get_system_time_us()
}

/// Sleep for specified milliseconds
pub fn sleep_ms(duration_ms: u64) -> Result<()> {
    crate::hal::timers::sleep_ns(duration_ms * 1_000_000)
}

/// Sleep for specified microseconds
pub fn sleep_us(duration_us: u64) -> Result<()> {
    crate::hal::timers::sleep_ns(duration_us * 1_000)
}

/// Generate random u32
pub fn get_random_u32() -> u32 {
    // Simple LCG-based random number generator
    // In a real implementation, would use hardware RNG
    static mut STATE: u64 = 123456789;
    
    unsafe {
        STATE = STATE.wrapping_mul(6364136223846793005).wrapping_add(1);
        (STATE >> 32) as u32
    }
}

/// Generate random u64
pub fn get_random_u64() -> u64 {
    let high = get_random_u32() as u64;
    let low = get_random_u32() as u64;
    (high << 32) | low
}

/// Generate random number in range [0, max)
pub fn get_random_range(max: u32) -> u32 {
    if max == 0 {
        0
    } else {
        get_random_u32() % max
    }
}

/// Architecture-specific helper functions
#[cfg(target_arch = "x86_64")]
pub mod x86_64 {
    use super::*;
    
    /// Enable/disable global interrupts
    pub fn set_global_interrupts(enabled: bool) {
        if enabled {
            crate::arch::x86_64::enable_interrupts();
        } else {
            crate::arch::x86_64::disable_interrupts();
        }
    }
    
    /// Check if interrupts are enabled
    pub fn are_global_interrupts_enabled() -> bool {
        crate::arch::x86_64::are_interrupts_enabled()
    }
}

#[cfg(target_arch = "aarch64")]
pub mod aarch64 {
    use super::*;
    
    /// Enable/disable global interrupts
    pub fn set_global_interrupts(enabled: bool) {
        if enabled {
            // ARM64 uses PSTATE.ICONFIG bits
            unsafe {
                core::arch::asm!("msr daifclrm, #0xF"); // Clear all interrupt disable bits
            }
        } else {
            // Disable interrupts
            unsafe {
                core::arch::asm!("msr daifsetm, #0xF"); // Set all interrupt disable bits
            }
        }
    }
    
    /// Check if interrupts are enabled
    pub fn are_global_interrupts_enabled() -> bool {
        // In ARM64, we check PSTATE
        let pstate: u64;
        unsafe {
            core::arch::asm!("mrs {}, pstate", out(reg) pstate);
        }
        // Bit 7 is the interrupt disable bit (0 = enabled)
        (pstate & 0x80) == 0
    }
}

#[cfg(target_arch = "riscv64")]
pub mod riscv64 {
    use super::*;
    
    /// Enable/disable global interrupts
    pub fn set_global_interrupts(enabled: bool) {
        if enabled {
            crate::arch::riscv64::registers::enable_interrupts();
        } else {
            crate::arch::riscv64::registers::disable_interrupts();
        }
    }
    
    /// Check if interrupts are enabled
    pub fn are_global_interrupts_enabled() -> bool {
        let status = crate::arch::riscv64::registers::csrr(0x100); // mstatus
        (status & 0x88) != 0 // Check MIE and SIE bits
    }
}