//! Timer Hardware Abstraction Layer
//!
//! This module provides unified timer interfaces across architectures for
//! system time, scheduling, and time-based operations.

use crate::log::{info, warn, error};
use crate::{KernelError, Result};
use spin::RwLock;
use core::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

/// Timer subsystem initialization
pub fn init() -> Result<()> {
    info!("Initializing Timer HAL...");
    
    // Detect available timers
    detect_timers()?;
    
    // Initialize system timer
    init_system_timer()?;
    
    // Initialize high-resolution timer
    init_high_res_timer()?;
    
    // Set up timer interrupts
    setup_timer_interrupts()?;
    
    Ok(())
}

/// Timer subsystem shutdown
pub fn shutdown() -> Result<()> {
    info!("Shutting down Timer HAL...");
    Ok(())
}

/// Timer types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TimerType {
    SystemTimer = 0,       // Used for system time keeping
    HighResolution = 1,    // High-resolution timer
    RealTimeClock = 2,     // Wall clock time
    PerformanceCounter = 3, // CPU performance counter
    LocalTimer = 4,        // Per-CPU local timer
    GlobalTimer = 5,       // Global system timer
}

/// Timer sources
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TimerSource {
    Tsc = 0,           // x86_64 TSC
    HpET = 1,          // High Precision Event Timer
    PIT = 2,           // Programmable Interval Timer
    RTC = 3,           // Real Time Clock
    ARMArchTimer = 4,  // ARMv8 Architecture Timer
    RiscVTimer = 5,    // RISC-V Timer
    Custom = 6,
}

/// Timer configuration
#[derive(Debug, Clone)]
pub struct TimerConfig {
    pub frequency: u64,          // Timer frequency in Hz
    pub resolution_ns: u64,      // Timer resolution in nanoseconds
    pub min_interval_ns: u64,    // Minimum interrupt interval
    pub max_interval_ns: u64,    // Maximum interrupt interval
    pub supports_periodic: bool, // Can generate periodic interrupts
    pub supports_one_shot: bool, // Can generate one-shot interrupts
}

/// Timer information
#[derive(Debug, Clone)]
pub struct TimerInfo {
    pub timer_type: TimerType,
    pub source: TimerSource,
    pub config: TimerConfig,
    pub enabled: bool,
}

/// System time structure
#[derive(Debug, Clone, Copy)]
pub struct SystemTime {
    pub seconds: u64,
    pub nanoseconds: u64,
    pub uptime_ticks: u64,
    pub ticks_per_second: u64,
}

/// Timer statistics
#[derive(Debug, Clone, Copy)]
pub struct TimerStats {
    pub timer_interrupts: AtomicU64,
    pub timer_overflows: AtomicU64,
    pub missed_interrupts: AtomicU64,
    pub drift_ns: AtomicU64,
    pub latency_ns: AtomicU64,
}

/// System time (atomic for thread safety)
static SYSTEM_TIME: AtomicU64 = AtomicU64::new(0); // Stored as seconds

/// Timer information table
static TIMER_INFO: RwLock<Vec<TimerInfo>> = RwLock::new(Vec::new());

/// System timer configuration
static TIMER_CONFIG: RwLock<Option<TimerConfig>> = RwLock::new(None);

/// Timer statistics
static TIMER_STATS: TimerStats = TimerStats {
    timer_interrupts: AtomicU64::new(0),
    timer_overflows: AtomicU64::new(0),
    missed_interrupts: AtomicU64::new(0),
    drift_ns: AtomicU64::new(0),
    latency_ns: AtomicU64::new(0),
};

/// Timer callback type
pub type TimerCallback = fn(u64); // Takes ticks as parameter

/// Timer callback table
static TIMER_CALLBACKS: RwLock<Vec<(TimerType, TimerCallback)>> = RwLock::new(Vec::new());

/// Detect available timers
fn detect_timers() -> Result<()> {
    info!("Detecting available timers...");
    
    let timers = detect_timers_arch()?;
    
    for timer in &timers {
        info!("Detected {} timer: {} Hz ({} ns resolution)", 
              timer.timer_type, timer.config.frequency, timer.config.resolution_ns);
    }
    
    *TIMER_INFO.write() = timers;
    
    Ok(())
}

/// Architecture-specific timer detection
#[cfg(target_arch = "x86_64")]
fn detect_timers_arch() -> Result<Vec<TimerInfo>> {
    let mut timers = Vec::new();
    
    // Detect TSC
    let tsc_config = TimerConfig {
        frequency: 2_400_000_000, // 2.4GHz typical
        resolution_ns: 1, // 1ns theoretical resolution
        min_interval_ns: 1,
        max_interval_ns: u64::MAX,
        supports_periodic: false,
        supports_one_shot: false,
    };
    
    timers.push(TimerInfo {
        timer_type: TimerType::HighResolution,
        source: TimerSource::Tsc,
        config: tsc_config,
        enabled: false,
    });
    
    // Detect HPET if available
    let hpet_config = TimerConfig {
        frequency: 14_318_180, // ~14.3MHz typical
        resolution_ns: 70, // ~70ns resolution
        min_interval_ns: 100,
        max_interval_ns: u64::MAX,
        supports_periodic: true,
        supports_one_shot: true,
    };
    
    timers.push(TimerInfo {
        timer_type: TimerType::SystemTimer,
        source: TimerSource::HpET,
        config: hpet_config,
        enabled: false,
    });
    
    Ok(timers)
}

#[cfg(target_arch = "aarch64")]
fn detect_timers_arch() -> Result<Vec<TimerInfo>> {
    let mut timers = Vec::new();
    
    // ARMv8 Architecture Timer
    let arch_timer_config = TimerConfig {
        frequency: 100_000_000, // 100MHz typical
        resolution_ns: 10, // 10ns resolution
        min_interval_ns: 100,
        max_interval_ns: u64::MAX,
        supports_periodic: true,
        supports_one_shot: true,
    };
    
    timers.push(TimerInfo {
        timer_type: TimerType::SystemTimer,
        source: TimerSource::ARMArchTimer,
        config: arch_timer_config,
        enabled: false,
    });
    
    timers.push(TimerInfo {
        timer_type: TimerType::HighResolution,
        source: TimerSource::Custom, // ARM performance counter
        config: TimerConfig {
            frequency: 2_000_000_000, // 2GHz typical
            resolution_ns: 1,
            min_interval_ns: 1,
            max_interval_ns: u64::MAX,
            supports_periodic: false,
            supports_one_shot: false,
        },
        enabled: false,
    });
    
    Ok(timers)
}

#[cfg(target_arch = "riscv64")]
fn detect_timers_arch() -> Result<Vec<TimerInfo>> {
    let mut timers = Vec::new();
    
    // RISC-V Timer (mtime)
    let riscv_timer_config = TimerConfig {
        frequency: 100_000_000, // 100MHz typical
        resolution_ns: 10,
        min_interval_ns: 100,
        max_interval_ns: u64::MAX,
        supports_periodic: true,
        supports_one_shot: true,
    };
    
    timers.push(TimerInfo {
        timer_type: TimerType::SystemTimer,
        source: TimerSource::RiscVTimer,
        config: riscv_timer_config,
        enabled: false,
    });
    
    Ok(timers)
}

/// Initialize system timer
fn init_system_timer() -> Result<()> {
    info!("Initializing system timer...");
    
    let timers = TIMER_INFO.read();
    
    // Find best system timer
    let system_timer = timers.iter()
        .find(|t| t.timer_type == TimerType::SystemTimer)
        .cloned()
        .or_else(|| timers.first().cloned())
        .ok_or(KernelError::NoTimerAvailable)?;
    
    info!("Using {} as system timer", system_timer.source);
    *TIMER_CONFIG.write() = Some(system_timer.config);
    
    // Enable the timer
    enable_timer(&system_timer)?;
    
    Ok(())
}

/// Initialize high-resolution timer
fn init_high_res_timer() -> Result<()> {
    info!("Initializing high-resolution timer...");
    
    let timers = TIMER_INFO.read();
    
    // Find high-resolution timer
    if let Some(high_res_timer) = timers.iter()
        .find(|t| t.timer_type == TimerType::HighResolution) 
    {
        enable_timer(high_res_timer)?;
        info!("High-resolution timer initialized: {}", high_res_timer.source);
    }
    
    Ok(())
}

/// Set up timer interrupts
fn setup_timer_interrupts() -> Result<()> {
    info!("Setting up timer interrupts...");
    
    // Configure timer interrupt
    configure_timer_interrupt()?;
    
    Ok(())
}

/// Configure timer interrupt
fn configure_timer_interrupt() -> Result<()> {
    let config = *TIMER_CONFIG.read();
    if let Some(config) = config {
        let interrupt_interval_ns = 1_000_000; // 1ms intervals
        let ticks_per_interrupt = interrupt_interval_ns * config.frequency / 1_000_000_000;
        
        info!("Timer interrupt interval: {} ticks", ticks_per_interrupt);
        
        // Set up periodic timer interrupt
        setup_periodic_interrupt(ticks_per_interrupt)?;
    }
    
    Ok(())
}

/// Setup periodic interrupt
fn setup_periodic_interrupt(ticks: u64) -> Result<()> {
    let controller = crate::hal::interrupts::get_interrupt_controller();
    
    match controller {
        crate::hal::interrupts::InterruptControllerType::Apic => {
            setup_apic_periodic_interrupt(ticks)?;
        }
        crate::hal::interrupts::InterruptControllerType::Gic => {
            setup_gic_periodic_interrupt(ticks)?;
        }
        crate::hal::interrupts::InterruptControllerType::Clint => {
            setup_clint_periodic_interrupt(ticks)?;
        }
        _ => {}
    }
    
    Ok(())
}

#[cfg(target_arch = "x86_64")]
fn setup_apic_periodic_interrupt(ticks: u64) -> Result<()> {
    info!("Setting up APIC periodic interrupt with {} ticks", ticks);
    // This would configure the APIC timer
    Ok(())
}

#[cfg(target_arch = "aarch64")]
fn setup_gic_periodic_interrupt(ticks: u64) -> Result<()> {
    info!("Setting up GIC periodic interrupt with {} ticks", ticks);
    // This would configure the GIC timer
    Ok(())
}

#[cfg(target_arch = "riscv64")]
fn setup_clint_periodic_interrupt(ticks: u64) -> Result<()> {
    info!("Setting up CLINT periodic interrupt with {} ticks", ticks);
    // This would configure the CLINT timer
    Ok(())
}

/// Enable timer
fn enable_timer(timer: &TimerInfo) -> Result<()> {
    info!("Enabling {} timer", timer.source);
    
    match timer.source {
        TimerSource::Tsc => enable_tsc(),
        TimerSource::HpET => enable_hpet(),
        TimerSource::ARMArchTimer => enable_arm_timer(),
        TimerSource::RiscVTimer => enable_riscv_timer(),
        _ => warn!("Unknown timer source: {:?}", timer.source),
    }
    
    Ok(())
}

fn enable_tsc() {
    #[cfg(target_arch = "x86_64")]
    {
        info!("TSC timer enabled");
        // x86_64 TSC is always available in long mode
    }
}

fn enable_hpet() {
    #[cfg(target_arch = "x86_64")]
    {
        info!("HPET timer enabled");
        // Configure HPET
    }
}

fn enable_arm_timer() {
    #[cfg(target_arch = "aarch64")]
    {
        info!("ARM Architecture Timer enabled");
        // Configure ARM timer
    }
}

fn enable_riscv_timer() {
    #[cfg(target_arch = "riscv64")]
    {
        info!("RISC-V Timer enabled");
        // Configure RISC-V timer
    }
}

/// Get current system time
pub fn get_system_time() -> SystemTime {
    let seconds = SYSTEM_TIME.load(Ordering::SeqCst);
    let uptime_ticks = get_uptime_ticks();
    let ticks_per_second = get_ticks_per_second();
    
    SystemTime {
        seconds,
        nanoseconds: 0, // Would be calculated from sub-second ticks
        uptime_ticks,
        ticks_per_second,
    }
}

/// Get uptime in ticks
pub fn get_uptime_ticks() -> u64 {
    get_timer_ticks()
}

/// Get timer ticks
pub fn get_timer_ticks() -> u64 {
    #[cfg(target_arch = "x86_64")]
    {
        crate::arch::x86_64::get_tsc()
    }
    
    #[cfg(target_arch = "aarch64")]
    {
        // ARM64 cycle counter
        let cycles: u64;
        unsafe {
            core::arch::asm!("mrs {}, pmccntr_el0", out(reg) cycles);
        }
        cycles
    }
    
    #[cfg(target_arch = "riscv64")]
    {
        crate::arch::riscv64::registers::get_cycle()
    }
}

/// Get high-resolution time
pub fn get_high_res_time() -> u64 {
    get_timer_ticks()
}

/// Get timer frequency
pub fn get_timer_frequency() -> u64 {
    let config = TIMER_CONFIG.read();
    if let Some(config) = config.as_ref() {
        config.frequency
    } else {
        1_000_000 // Default 1MHz
    }
}

/// Get ticks per second
pub fn get_ticks_per_second() -> u64 {
    get_timer_frequency()
}

/// Update system time (called from timer interrupt)
pub fn update_system_time(increment_ticks: u64) {
    let ticks_per_second = get_ticks_per_second();
    let nanoseconds_per_tick = 1_000_000_000 / ticks_per_second;
    
    let total_ns = increment_ticks * nanoseconds_per_tick;
    let seconds_to_add = total_ns / 1_000_000_000;
    let nanoseconds_to_add = total_ns % 1_000_000_000;
    
    // Update system time atomically
    let current_time = SYSTEM_TIME.load(Ordering::SeqCst);
    let new_time = current_time + seconds_to_add;
    SYSTEM_TIME.store(new_time, Ordering::SeqCst);
    
    // Call registered timer callbacks
    call_timer_callbacks(increment_ticks);
    
    // Update statistics
    TIMER_STATS.timer_interrupts.fetch_add(1, Ordering::SeqCst);
}

/// Call registered timer callbacks
fn call_timer_callbacks(ticks: u64) {
    let callbacks = TIMER_CALLBACKS.read();
    for (timer_type, callback) in callbacks.iter() {
        match timer_type {
            TimerType::SystemTimer => {
                if *timer_type == TimerType::SystemTimer {
                    callback(ticks);
                }
            }
            _ => callback(ticks),
        }
    }
}

/// Register timer callback
pub fn register_timer_callback(timer_type: TimerType, callback: TimerCallback) {
    let mut callbacks = TIMER_CALLBACKS.write();
    callbacks.push((timer_type, callback));
    info!("Timer callback registered for {:?}", timer_type);
}

/// Sleep for specified duration
pub fn sleep_ns(duration_ns: u64) -> Result<()> {
    if duration_ns < 1_000_000 {
        // Short sleep - use busy waiting
        busy_sleep(duration_ns);
    } else {
        // Long sleep - use timer interrupt
        timer_sleep(duration_ns)?;
    }
    
    Ok(())
}

/// Busy wait for specified nanoseconds
fn busy_sleep(duration_ns: u64) {
    let start = get_high_res_time();
    let target_ticks = start + duration_ns * get_timer_frequency() / 1_000_000_000;
    
    while get_high_res_time() < target_ticks {
        // Busy wait
        core::hint::spin_loop();
    }
}

/// Sleep using timer interrupt
fn timer_sleep(duration_ns: u64) -> Result<()> {
    info!("Sleeping for {} ns", duration_ns);
    
    // Disable interrupts during sleep
    crate::hal::interrupts::disable_global_interrupts();
    
    // Set up wake timer
    setup_wake_timer(duration_ns)?;
    
    // Wait for wake timer interrupt
    crate::hal::cpu::halt_cpu();
    
    // Re-enable interrupts
    crate::hal::interrupts::enable_global_interrupts();
    
    Ok(())
}

/// Setup wake timer
fn setup_wake_timer(duration_ns: u64) -> Result<()> {
    // Configure one-shot timer
    let config = TIMER_CONFIG.read();
    if let Some(config) = config {
        let ticks = duration_ns * config.frequency / 1_000_000_000;
        setup_one_shot_interrupt(ticks)?;
    }
    
    Ok(())
}

/// Setup one-shot interrupt
fn setup_one_shot_interrupt(ticks: u64) -> Result<()> {
    // Configure timer for one-shot interrupt
    info!("Setting up one-shot interrupt in {} ticks", ticks);
    
    Ok(())
}

/// Convert ticks to nanoseconds
pub fn ticks_to_ns(ticks: u64) -> u64 {
    let frequency = get_timer_frequency();
    (ticks * 1_000_000_000) / frequency
}

/// Convert nanoseconds to ticks
pub fn ns_to_ticks(ns: u64) -> u64 {
    let frequency = get_timer_frequency();
    (ns * frequency) / 1_000_000_000
}

/// Get timer statistics
pub fn get_stats() -> TimerStats {
    TIMER_STATS
}

/// Get all timer information
pub fn get_timer_info() -> Vec<TimerInfo> {
    TIMER_INFO.read().clone()
}

/// Benchmark timer resolution
pub fn benchmark_timer_resolution() -> Result<u64> {
    info!("Benchmarking timer resolution...");
    
    let mut min_interval = u64::MAX;
    
    // Measure minimum interruptible time
    for _ in 0..1000 {
        let start = get_high_res_time();
        
        // Small busy loop
        for _ in 0..10 {
            core::hint::spin_loop();
        }
        
        let end = get_high_res_time();
        let interval = end - start;
        
        if interval > 0 && interval < min_interval {
            min_interval = interval;
        }
    }
    
    let resolution_ns = ticks_to_ns(min_interval);
    info!("Timer resolution: {} ns", resolution_ns);
    
    Ok(resolution_ns)
}

/// Timer utility functions
pub mod utils {
    use super::*;
    
    /// Convert seconds to ticks
    pub fn seconds_to_ticks(seconds: u64) -> u64 {
        seconds * get_ticks_per_second()
    }
    
    /// Convert milliseconds to ticks
    pub fn ms_to_ticks(milliseconds: u64) -> u64 {
        milliseconds * get_ticks_per_second() / 1000
    }
    
    /// Convert microseconds to ticks
    pub fn us_to_ticks(microseconds: u64) -> u64 {
        microseconds * get_ticks_per_second() / 1_000_000
    }
    
    /// Convert ticks to seconds
    pub fn ticks_to_seconds(ticks: u64) -> u64 {
        ticks / get_ticks_per_second()
    }
    
    /// Convert ticks to milliseconds
    pub fn ticks_to_ms(ticks: u64) -> u64 {
        ticks * 1000 / get_ticks_per_second()
    }
    
    /// Convert ticks to microseconds
    pub fn ticks_to_us(ticks: u64) -> u64 {
        ticks * 1_000_000 / get_ticks_per_second()
    }
}

/// Timer callback functions
pub mod callbacks {
    use super::*;
    
    /// System tick callback
    pub fn system_tick(_ticks: u64) {
        // Update kernel time
        // Update scheduler
        // Process timer queues
    }
    
    /// High-resolution timer callback
    pub fn high_res_tick(_ticks: u64) {
        // Update high-resolution time tracking
    }
    
    /// Performance monitoring callback
    pub fn performance_tick(_ticks: u64) {
        // Update performance counters
        // Sample CPU usage
    }
}

/// Get system time in milliseconds (convenience function)
pub fn get_system_time_ms() -> u64 {
    let time = get_system_time();
    time.seconds * 1000 + (time.uptime_ticks % time.ticks_per_second) * 1000 / time.ticks_per_second
}

/// Get system time in microseconds (convenience function)
pub fn get_system_time_us() -> u64 {
    let time = get_system_time();
    time.seconds * 1_000_000 + (time.uptime_ticks % time.ticks_per_second) * 1_000_000 / time.ticks_per_second
}