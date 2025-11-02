//! Interrupt Hardware Abstraction Layer
//!
//! This module provides unified interrupt management interfaces across architectures
//! for interrupt controllers, handling, routing, and masking.

use crate::log::{info, warn, error};
use crate::{KernelError, Result};
use spin::RwLock;
use spin::Mutex;
use core::sync::atomic::{AtomicU64, Ordering};

/// Interrupt subsystem initialization
pub fn init() -> Result<()> {
    info!("Initializing Interrupt HAL...");
    
    // Detect interrupt controller
    detect_interrupt_controller()?;
    
    // Initialize interrupt controller
    init_interrupt_controller()?;
    
    // Set up interrupt routing
    setup_interrupt_routing()?;
    
    // Configure interrupt handling
    configure_interrupt_handling()?;
    
    Ok(())
}

/// Interrupt subsystem shutdown
pub fn shutdown() -> Result<()> {
    info!("Shutting down Interrupt HAL...");
    Ok(())
}

/// Interrupt controller types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum InterruptControllerType {
    Pic = 0,     // 8259 PIC
    Apic = 1,    // APIC (x86)
    Gic = 2,     // GIC (ARM)
    Clint = 3,   // CLINT (RISC-V)
    Plic = 4,    // PLIC (RISC-V)
    Unknown = 255,
}

/// Interrupt sources
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum InterruptSource {
    Timer = 0,
    Keyboard = 1,
    Serial = 2,
    Network = 3,
    Storage = 4,
    Graphics = 5,
    Usb = 6,
    Audio = 7,
    Power = 8,
    Thermal = 9,
    Ipc = 10,
    Custom = 11,
    Spurious = 255,
}

/// Interrupt priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum InterruptPriority {
    Lowest = 0,
    Low = 32,
    Normal = 64,
    High = 96,
    Highest = 127,
}

/// Interrupt state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum InterruptState {
    Disabled = 0,
    Enabled = 1,
    Pending = 2,
    Active = 3,
    InService = 4,
}

/// Interrupt descriptor
#[derive(Debug, Clone)]
pub struct InterruptDescriptor {
    pub source: InterruptSource,
    pub vector: u32,
    pub priority: InterruptPriority,
    pub state: InterruptState,
    pub enabled: bool,
    pub cpu_mask: u64,
    pub count: AtomicU64,
}

/// Interrupt statistics
#[derive(Debug, Clone, Copy)]
pub struct InterruptStats {
    pub total_interrupts: AtomicU64,
    pub timer_interrupts: AtomicU64,
    pub keyboard_interrupts: AtomicU64,
    pub serial_interrupts: AtomicU64,
    pub spurious_interrupts: AtomicU64,
    pub interrupt_latency_ns: AtomicU64,
}

/// Current interrupt controller
static INTERRUPT_CONTROLLER: RwLock<InterruptControllerType> = 
    RwLock::new(InterruptControllerType::Unknown);

/// Interrupt table
static INTERRUPT_TABLE: RwLock<Vec<InterruptDescriptor>> = 
    RwLock::new(Vec::new());

/// Interrupt statistics
static INTERRUPT_STATS: InterruptStats = InterruptStats {
    total_interrupts: AtomicU64::new(0),
    timer_interrupts: AtomicU64::new(0),
    keyboard_interrupts: AtomicU64::new(0),
    serial_interrupts: AtomicU64::new(0),
    spurious_interrupts: AtomicU64::new(0),
    interrupt_latency_ns: AtomicU64::new(0),
};

/// Interrupt handling lock
static INTERRUPT_LOCK: Mutex<()> = Mutex::new(());

/// Detect interrupt controller
fn detect_interrupt_controller() -> Result<()> {
    info!("Detecting interrupt controller...");
    
    let controller = detect_interrupt_controller_arch()?;
    *INTERRUPT_CONTROLLER.write() = controller;
    
    match controller {
        InterruptControllerType::Pic => info!("Detected 8259 PIC"),
        InterruptControllerType::Apic => info!("Detected APIC"),
        InterruptControllerType::Gic => info!("Detected GIC"),
        InterruptControllerType::Clint => info!("Detected CLINT"),
        InterruptControllerType::Plic => info!("Detected PLIC"),
        InterruptControllerType::Unknown => warn!("Unknown interrupt controller"),
    }
    
    Ok(())
}

/// Architecture-specific interrupt controller detection
#[cfg(target_arch = "x86_64")]
fn detect_interrupt_controller_arch() -> Result<InterruptControllerType> {
    // Check for APIC presence
    let has_apic = check_x86_64_apic();
    if has_apic {
        Ok(InterruptControllerType::Apic)
    } else {
        Ok(InterruptControllerType::Pic)
    }
}

#[cfg(target_arch = "aarch64")]
fn detect_interrupt_controller_arch() -> Result<InterruptControllerType> {
    // ARM64 uses GIC
    Ok(InterruptControllerType::Gic)
}

#[cfg(target_arch = "riscv64")]
fn detect_interrupt_controller_arch() -> Result<InterruptControllerType> {
    // RISC-V uses CLINT and PLIC
    // Assume both are present
    Ok(InterruptControllerType::Clint)
}

#[cfg(target_arch = "x86_64")]
fn check_x86_64_apic() -> bool {
    // Check if APIC is present by reading CPUID
    // Simplified for now
    true
}

/// Initialize interrupt controller
fn init_interrupt_controller() -> Result<()> {
    let controller = *INTERRUPT_CONTROLLER.read();
    
    info!("Initializing {} interrupt controller", 
          match controller {
              InterruptControllerType::Pic => "8259 PIC",
              InterruptControllerType::Apic => "APIC",
              InterruptControllerType::Gic => "GIC",
              InterruptControllerType::Clint => "CLINT",
              InterruptControllerType::Plic => "PIC",
              _ => "Unknown",
          });
    
    match controller {
        InterruptControllerType::Pic => init_pic_controller(),
        InterruptControllerType::Apic => init_apic_controller(),
        InterruptControllerType::Gic => init_gic_controller(),
        InterruptControllerType::Clint => init_clint_controller(),
        InterruptControllerType::Plic => init_plic_controller(),
        _ => Err(KernelError::Unsupported),
    }
}

/// Initialize PIC controller
fn init_pic_controller() -> Result<()> {
    #[cfg(target_arch = "x86_64")]
    {
        use crate::arch::x86_64::pic;
        pic::init_8259_pic()?;
        info!("PIC controller initialized");
    }
    Ok(())
}

/// Initialize APIC controller
fn init_apic_controller() -> Result<()> {
    #[cfg(target_arch = "x86_64")]
    {
        use crate::arch::x86_64::apic;
        apic::init_local_apic()?;
        info!("APIC controller initialized");
    }
    Ok(())
}

/// Initialize GIC controller
fn init_gic_controller() -> Result<()> {
    #[cfg(target_arch = "aarch64")]
    {
        use crate::arch::aarch64::gic;
        gic::init_gic()?;
        info!("GIC controller initialized");
    }
    Ok(())
}

/// Initialize CLINT controller
fn init_clint_controller() -> Result<()> {
    #[cfg(target_arch = "riscv64")]
    {
        use crate::arch::riscv64::clint;
        clint::init_clint()?;
        info!("CLINT controller initialized");
    }
    Ok(())
}

/// Initialize PLIC controller
fn init_plic_controller() -> Result<()> {
    #[cfg(target_arch = "riscv64")]
    {
        use crate::arch::riscv64::plic;
        plic::init_plic()?;
        info!("PLIC controller initialized");
    }
    Ok(())
}

/// Set up interrupt routing
fn setup_interrupt_routing() -> Result<()> {
    info!("Setting up interrupt routing...");
    
    // Initialize interrupt table
    init_interrupt_table()?;
    
    // Configure routing for each interrupt source
    route_interrupts()?;
    
    Ok(())
}

/// Initialize interrupt table
fn init_interrupt_table() -> Result<()> {
    let mut table = Vec::new();
    
    // Add standard interrupts
    table.push(InterruptDescriptor {
        source: InterruptSource::Timer,
        vector: 32, // Timer interrupt vector
        priority: InterruptPriority::High,
        state: InterruptState::Disabled,
        enabled: false,
        cpu_mask: 0x1, // CPU 0
        count: AtomicU64::new(0),
    });
    
    table.push(InterruptDescriptor {
        source: InterruptSource::Keyboard,
        vector: 33,
        priority: InterruptPriority::Normal,
        state: InterruptState::Disabled,
        enabled: false,
        cpu_mask: 0x1,
        count: AtomicU64::new(0),
    });
    
    table.push(InterruptDescriptor {
        source: InterruptSource::Serial,
        vector: 34,
        priority: InterruptPriority::Normal,
        state: InterruptState::Disabled,
        enabled: false,
        cpu_mask: 0x1,
        count: AtomicU64::new(0),
    });
    
    *INTERRUPT_TABLE.write() = table;
    
    Ok(())
}

/// Route interrupts to CPUs
fn route_interrupts() -> Result<()> {
    let controller = *INTERRUPT_CONTROLLER.read();
    
    match controller {
        InterruptControllerType::Apic => route_x86_64_apic(),
        InterruptControllerType::Gic => route_arm64_gic(),
        InterruptControllerType::Clint | InterruptControllerType::Plic => route_riscv64(),
        _ => Ok(()),
    }
}

#[cfg(target_arch = "x86_64")]
fn route_x86_64_apic() -> Result<()> {
    info!("Routing interrupts via APIC");
    Ok(())
}

#[cfg(target_arch = "aarch64")]
fn route_arm64_gic() -> Result<()> {
    info!("Routing interrupts via GIC");
    Ok(())
}

#[cfg(target_arch = "riscv64")]
fn route_riscv64() -> Result<()> {
    info!("Routing interrupts via CLINT/PLIC");
    Ok(())
}

/// Configure interrupt handling
fn configure_interrupt_handling() -> Result<()> {
    info!("Configuring interrupt handling...");
    
    // Set up interrupt handlers
    setup_interrupt_handlers()?;
    
    // Enable interrupts globally
    enable_global_interrupts()?;
    
    Ok(())
}

/// Set up interrupt handlers
fn setup_interrupt_handlers() -> Result<()> {
    #[cfg(target_arch = "x86_64")]
    {
        use crate::arch::x86_64::interrupt;
        interrupt::setup_idt()?;
    }
    
    #[cfg(target_arch = "aarch64")]
    {
        use crate::arch::aarch64::interrupt;
        interrupt::init_exception_level_handlers()?;
    }
    
    #[cfg(target_arch = "riscv64")]
    {
        use crate::arch::riscv64::interrupt;
        interrupt::init_exception_handlers()?;
    }
    
    Ok(())
}

/// Enable global interrupts
fn enable_global_interrupts() {
    #[cfg(target_arch = "x86_64")]
    {
        crate::arch::x86_64::enable_interrupts();
    }
    
    #[cfg(target_arch = "aarch64")]
    {
        // ARM64 uses PSTATE.ICONFIG
        unsafe {
            core::arch::asm!("msr daifclrm, #0xF");
        }
    }
    
    #[cfg(target_arch = "riscv64")]
    {
        crate::arch::riscv64::registers::enable_interrupts();
    }
}

/// Disable global interrupts
fn disable_global_interrupts() {
    #[cfg(target_arch = "x86_64")]
    {
        crate::arch::x86_64::disable_interrupts();
    }
    
    #[cfg(target_arch = "aarch64")]
    {
        unsafe {
            core::arch::asm!("msr daifsetm, #0xF");
        }
    }
    
    #[cfg(target_arch = "riscv64")]
    {
        crate::arch::riscv64::registers::disable_interrupts();
    }
}

/// Enable specific interrupt
pub fn enable_interrupt(source: InterruptSource) -> Result<()> {
    let controller = *INTERRUPT_CONTROLLER.read();
    
    match source {
        InterruptSource::Timer => enable_timer_interrupt(),
        InterruptSource::Keyboard => enable_keyboard_interrupt(),
        InterruptSource::Serial => enable_serial_interrupt(),
        _ => warn!("Unknown interrupt source: {:?}", source),
    }
    
    // Update interrupt table
    update_interrupt_state(source, InterruptState::Enabled)?;
    
    Ok(())
}

/// Enable timer interrupt
fn enable_timer_interrupt() {
    info!("Enabling timer interrupt");
    
    // Increment timer interrupt count
    INTERRUPT_STATS.timer_interrupts.fetch_add(1, Ordering::SeqCst);
    
    // Start timer
    start_system_timer()?;
}

/// Start system timer
fn start_system_timer() -> Result<()> {
    let controller = *INTERRUPT_CONTROLLER.read();
    
    match controller {
        InterruptControllerType::Apic => start_apic_timer(),
        InterruptControllerType::Gic => start_gic_timer(),
        InterruptControllerType::Clint => start_clint_timer(),
        _ => {},
    }
    
    Ok(())
}

#[cfg(target_arch = "x86_64")]
fn start_apic_timer() {
    // Start APIC timer
    info!("Starting APIC timer");
}

#[cfg(target_arch = "aarch64")]
fn start_gic_timer() {
    // Start GIC timer
    info!("Starting GIC timer");
}

#[cfg(target_arch = "riscv64")]
fn start_clint_timer() {
    // Start CLINT timer
    info!("Starting CLINT timer");
}

/// Enable keyboard interrupt
fn enable_keyboard_interrupt() {
    info!("Enabling keyboard interrupt");
    INTERRUPT_STATS.keyboard_interrupts.fetch_add(1, Ordering::SeqCst);
}

/// Enable serial interrupt
fn enable_serial_interrupt() {
    info!("Enabling serial interrupt");
    INTERRUPT_STATS.serial_interrupts.fetch_add(1, Ordering::SeqCst);
}

/// Disable specific interrupt
pub fn disable_interrupt(source: InterruptSource) -> Result<()> {
    warn!("Disabling interrupt: {:?}", source);
    update_interrupt_state(source, InterruptState::Disabled)?;
    Ok(())
}

/// Update interrupt state
fn update_interrupt_state(source: InterruptSource, state: InterruptState) -> Result<()> {
    let mut table = INTERRUPT_TABLE.write();
    
    for descriptor in &mut table {
        if descriptor.source == source {
            descriptor.state = state;
            break;
        }
    }
    
    Ok(())
}

/// Send interrupt to CPU
pub fn send_interrupt_to_cpu(source: InterruptSource, cpu_id: usize) -> Result<()> {
    let controller = *INTERRUPT_CONTROLLER.read();
    
    match controller {
        InterruptControllerType::Apic => send_apic_interrupt(source, cpu_id),
        InterruptControllerType::Gic => send_gic_interrupt(source, cpu_id),
        InterruptControllerType::Clint | InterruptControllerType::Plic => send_riscv_interrupt(source, cpu_id),
        _ => Err(KernelError::Unsupported),
    }
}

#[cfg(target_arch = "x86_64")]
fn send_apic_interrupt(source: InterruptSource, cpu_id: usize) -> Result<()> {
    // Send IPI (Inter-Processor Interrupt)
    info!("Sending {} interrupt to CPU {}", source, cpu_id);
    Ok(())
}

#[cfg(target_arch = "aarch64")]
fn send_gic_interrupt(source: InterruptSource, cpu_id: usize) -> Result<()> {
    // Send GIC interrupt
    info!("Sending {} interrupt to CPU {}", source, cpu_id);
    Ok(())
}

#[cfg(target_arch = "riscv64")]
fn send_riscv_interrupt(source: InterruptSource, cpu_id: usize) -> Result<()> {
    // Send RISC-V interrupt
    info!("Sending {} interrupt to CPU {}", source, cpu_id);
    Ok(())
}

/// Check if global interrupts are enabled
pub fn are_global_interrupts_enabled() -> bool {
    #[cfg(target_arch = "x86_64")]
    {
        crate::arch::x86_64::are_interrupts_enabled()
    }
    
    #[cfg(target_arch = "aarch64")]
    {
        // ARM64 PSTATE check
        let pstate: u64;
        unsafe {
            core::arch::asm!("mrs {}, pstate", out(reg) pstate);
        }
        (pstate & 0x80) == 0
    }
    
    #[cfg(target_arch = "riscv64")]
    {
        let status = crate::arch::riscv64::registers::csrr(0x100); // mstatus
        (status & 0x88) != 0
    }
}

/// Get current interrupt controller
pub fn get_interrupt_controller() -> InterruptControllerType {
    *INTERRUPT_CONTROLLER.read()
}

/// Get interrupt statistics
pub fn get_stats() -> InterruptStats {
    INTERRUPT_STATS
}

/// Get interrupt table
pub fn get_interrupt_table() -> Vec<InterruptDescriptor> {
    INTERRUPT_TABLE.read().clone()
}

/// Get interrupt statistics for specific source
pub fn get_interrupt_count(source: InterruptSource) -> u64 {
    let table = INTERRUPT_TABLE.read();
    for descriptor in table.iter() {
        if descriptor.source == source {
            return descriptor.count.load(Ordering::SeqCst);
        }
    }
    0
}

/// Increment interrupt count
pub fn increment_interrupt_count(source: InterruptSource) {
    INTERRUPT_STATS.total_interrupts.fetch_add(1, Ordering::SeqCst);
    
    let mut table = INTERRUPT_TABLE.write();
    for descriptor in &mut table {
        if descriptor.source == source {
            descriptor.count.fetch_add(1, Ordering::SeqCst);
            break;
        }
    }
}

/// Set interrupt priority
pub fn set_interrupt_priority(source: InterruptSource, priority: InterruptPriority) -> Result<()> {
    let mut table = INTERRUPT_TABLE.write();
    
    for descriptor in &mut table {
        if descriptor.source == source {
            descriptor.priority = priority;
            break;
        }
    }
    
    Ok(())
}

/// Benchmark interrupt latency
pub fn benchmark_latency() -> u64 {
    // Measure interrupt latency by enabling/disabling interrupts
    let start = crate::hal::cpu::get_cycles();
    
    disable_global_interrupts();
    let _ = are_global_interrupts_enabled(); // Trigger interrupt check
    enable_global_interrupts();
    
    let end = crate::hal::cpu::get_cycles();
    let latency = end - start;
    
    // Store average latency
    let current_avg = INTERRUPT_STATS.interrupt_latency_ns.load(Ordering::SeqCst);
    let new_avg = (current_avg + latency) / 2;
    INTERRUPT_STATS.interrupt_latency_ns.store(new_avg, Ordering::SeqCst);
    
    latency
}

/// Interrupt handlers module
pub mod handlers {
    use super::*;
    
    /// Timer interrupt handler
    pub fn timer_interrupt_handler() {
        increment_interrupt_count(InterruptSource::Timer);
        info!("Timer interrupt received");
        
        // Update system time
        update_system_time();
        
        // Call scheduler tick
        // scheduler::tick();
    }
    
    /// Keyboard interrupt handler
    pub fn keyboard_interrupt_handler() {
        increment_interrupt_count(InterruptSource::Keyboard);
        info!("Keyboard interrupt received");
        
        // Read keyboard input
        // handle_keyboard_input();
    }
    
    /// Serial interrupt handler
    pub fn serial_interrupt_handler() {
        increment_interrupt_count(InterruptSource::Serial);
        info!("Serial interrupt received");
        
        // Handle serial communication
        // handle_serial_io();
    }
    
    /// Spurious interrupt handler
    pub fn spurious_interrupt_handler() {
        INTERRUPT_STATS.spurious_interrupts.fetch_add(1, Ordering::SeqCst);
        warn!("Spurious interrupt received");
    }
    
    /// Update system time
    fn update_system_time() {
        // Update kernel time tracking
        // time::tick();
    }
    
    /// Generic interrupt handler
    pub fn generic_interrupt_handler(interrupt_vector: u32) {
        increment_interrupt_count(InterruptSource::Custom);
        info!("Generic interrupt: vector {}", interrupt_vector);
        
        // Find the interrupt source and call appropriate handler
        let table = INTERRUPT_TABLE.read();
        for descriptor in table.iter() {
            if descriptor.vector == interrupt_vector {
                match descriptor.source {
                    InterruptSource::Timer => timer_interrupt_handler(),
                    InterruptSource::Keyboard => keyboard_interrupt_handler(),
                    InterruptSource::Serial => serial_interrupt_handler(),
                    _ => spurious_interrupt_handler(),
                }
                break;
            }
        }
    }
    
    /// Exception handler
    pub fn exception_handler(exception_type: u32, error_code: u32, address: usize) {
        error!("Exception: type={}, code={}, address={:#x}", 
               exception_type, error_code, address);
        
        match exception_type {
            14 => { // Page fault
                page_fault_handler(address, error_code, error_code);
            }
            13 => { // General protection fault
                general_protection_fault_handler(error_code, address);
            }
            _ => {
                error!("Unhandled exception type: {}", exception_type);
            }
        }
    }
    
    /// Page fault handler
    pub fn page_fault_handler(address: usize, error_code: usize, _register: usize) {
        warn!("Page fault at address {:#x}, error code {:#x}", address, error_code);
        
        // Handle page fault
        // memory::handle_page_fault(address, error_code)?;
    }
    
    /// General protection fault handler
    pub fn general_protection_fault_handler(error_code: u32, address: usize) {
        error!("General protection fault at {:#x}, error code {:#x}", address, error_code);
        
        // This is typically a critical error
        // In a real kernel, this might trigger a panic or process termination
    }
}