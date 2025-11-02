//! Interrupt handling and management
//! 
//! This module provides interrupt management, including:
//! - Hardware interrupt handling
//! - Interrupt controller management
//! - Interrupt masking and priority
//! - Interrupt statistics

use spin::Mutex;
use alloc::vec::Vec;
use log::{debug, warn, error};
use crate::KernelResult;
use crate::arch::interrupts::*;

/// Interrupt statistics
#[derive(Debug, Clone)]
pub struct InterruptStats {
    pub total_interrupts: u64,
    pub timer_interrupts: u64,
    pub keyboard_interrupts: u64,
    pub serial_interrupts: u64,
    pub timer_last_trigger: u64,
    pub timer_rate_hz: u32,
}

static INTERRUPT_STATS: Mutex<InterruptStats> = Mutex::new(InterruptStats {
    total_interrupts: 0,
    timer_interrupts: 0,
    keyboard_interrupts: 0,
    serial_interrupts: 0,
    timer_last_trigger: 0,
    timer_rate_hz: 100, // Default 100 Hz
});

/// Interrupt handler function type
pub type InterruptHandler = fn();

/// Registered interrupt handlers
static INTERRUPT_HANDLERS: Mutex<[Option<InterruptHandler>; MAX_INTERRUPT_VECTORS]> = 
    Mutex::new([None; MAX_INTERRUPT_VECTORS]);

/// Initialize interrupt system
pub fn init() -> KernelResult<()> {
    debug!("Initializing interrupt system...");
    
    // Initialize interrupt statistics
    let mut stats = INTERRUPT_STATS.lock();
    stats.total_interrupts = 0;
    stats.timer_interrupts = 0;
    stats.keyboard_interrupts = 0;
    stats.serial_interrupts = 0;
    stats.timer_last_trigger = 0;
    stats.timer_rate_hz = 100;
    
    // Initialize interrupt handlers
    let mut handlers = INTERRUPT_HANDLERS.lock();
    for handler in &mut *handlers {
        *handler = None;
    }
    
    debug!("Interrupt system initialized");
    
    Ok(())
}

/// Register an interrupt handler
pub fn register_interrupt_handler(vector: u8, handler: InterruptHandler) -> KernelResult<()> {
    if vector as usize >= MAX_INTERRUPT_VECTORS {
        return Err(crate::KernelError::InterruptInitFailed);
    }
    
    let mut handlers = INTERRUPT_HANDLERS.lock();
    handlers[vector as usize] = Some(handler);
    
    debug!("Registered interrupt handler for vector {:#x}", vector);
    
    Ok(())
}

/// Unregister an interrupt handler
pub fn unregister_interrupt_handler(vector: u8) -> KernelResult<()> {
    if vector as usize >= MAX_INTERRUPT_VECTORS {
        return Err(crate::KernelError::InterruptInitFailed);
    }
    
    let mut handlers = INTERRUPT_HANDLERS.lock();
    handlers[vector as usize] = None;
    
    debug!("Unregistered interrupt handler for vector {:#x}", vector);
    
    Ok(())
}

/// Enable a specific interrupt
pub fn enable_interrupt(vector: u8) {
    debug!("Enabling interrupt vector {:#x}", vector);
    
    // For x86_64, this would configure the PIC
    // For now, just log the action
}

/// Disable a specific interrupt
pub fn disable_interrupt(vector: u8) {
    debug!("Disabling interrupt vector {:#x}", vector);
    
    // For x86_64, this would configure the PIC
    // For now, just log the action
}

/// Mask an interrupt line
pub fn mask_interrupt(irq: u8) {
    debug!("Masking IRQ {}", irq);
    
    // For x86_64, this would send command to PIC
    // For now, just log the action
}

/// Unmask an interrupt line
pub fn unmask_interrupt(irq: u8) {
    debug!("Unmasking IRQ {}", irq);
    
    // For x86_64, this would send command to PIC
    // For now, just log the action
}

/// Check if an interrupt is enabled
pub fn is_interrupt_enabled(vector: u8) -> bool {
    // For x86_64, check PIC status
    // For now, always return true
    true
}

/// Get current interrupt statistics
pub fn get_interrupt_stats() -> InterruptStats {
    *INTERRUPT_STATS.lock()
}

/// Update timer interrupt statistics
pub fn handle_timer_interrupt() {
    let mut stats = INTERRUPT_STATS.lock();
    stats.total_interrupts += 1;
    stats.timer_interrupts += 1;
    stats.timer_last_trigger = get_system_time();
}

/// Update keyboard interrupt statistics
pub fn handle_keyboard_interrupt() {
    let mut stats = INTERRUPT_STATS.lock();
    stats.total_interrupts += 1;
    stats.keyboard_interrupts += 1;
}

/// Update serial interrupt statistics
pub fn handle_serial_interrupt() {
    let mut stats = INTERRUPT_STATS.lock();
    stats.total_interrupts += 1;
    stats.serial_interrupts += 1;
}

/// Set timer interrupt rate
pub fn set_timer_rate(hz: u32) -> KernelResult<()> {
    if hz == 0 || hz > 1000 {
        return Err(crate::KernelError::InterruptInitFailed);
    }
    
    let mut stats = INTERRUPT_STATS.lock();
    stats.timer_rate_hz = hz;
    
    debug!("Timer interrupt rate set to {} Hz", hz);
    
    Ok(())
}

/// Get timer interrupt rate
pub fn get_timer_rate() -> u32 {
    INTERRUPT_STATS.lock().timer_rate_hz
}

/// Get timer period in nanoseconds
pub fn get_timer_period_ns() -> u64 {
    let rate = get_timer_rate();
    1_000_000_000 / rate as u64 // Convert Hz to nanoseconds
}

/// Check if it's time for a timer interrupt
pub fn should_trigger_timer_interrupt() -> bool {
    let stats = INTERRUPT_STATS.lock();
    let current_time = get_system_time();
    
    if stats.timer_last_trigger == 0 {
        return true; // First time
    }
    
    let elapsed = current_time - stats.timer_last_trigger;
    elapsed >= get_timer_period_ns()
}

/// Process an interrupt
pub fn process_interrupt(vector: u8) {
    debug!("Processing interrupt vector {:#x}", vector);
    
    // Update statistics
    let mut stats = INTERRUPT_STATS.lock();
    stats.total_interrupts += 1;
    
    // Call registered handler if any
    let handlers = INTERRUPT_HANDLERS.lock();
    if let Some(handler) = handlers[vector as usize] {
        handler();
    }
    
    // Handle specific interrupt types
    match vector {
        TIMER_INTERRUPT => {
            stats.timer_interrupts += 1;
            stats.timer_last_trigger = get_system_time();
        }
        KEYBOARD_INTERRUPT => {
            stats.keyboard_interrupts += 1;
        }
        COM1_INTERRUPT | COM2_INTERRUPT => {
            stats.serial_interrupts += 1;
        }
        _ => {
            // Other interrupt types
        }
    }
}

/// Handle interrupt from interrupt handler context
pub fn handle_interrupt_from_isr(vector: u8) {
    // This function is called from the actual interrupt handler
    // to do additional processing that can't be done in the ISR
    process_interrupt(vector);
    
    // Signal scheduler if it's a timer interrupt
    if vector == TIMER_INTERRUPT {
        crate::scheduler::schedule_next();
    }
}

/// Get system time (simplified)
fn get_system_time() -> u64 {
    // In a real implementation, this would read from a high-resolution timer
    // For now, just return a dummy value
    0
}

/// Get human-readable interrupt name
pub fn get_interrupt_name(vector: u8) -> &'static str {
    match vector {
        TIMER_INTERRUPT => "Timer",
        KEYBOARD_INTERRUPT => "Keyboard",
        COM1_INTERRUPT => "COM1 Serial",
        COM2_INTERRUPT => "COM2 Serial",
        CASCADE_INTERRUPT => "PIC Cascade",
        FLOPPY_INTERRUPT => "Floppy Disk",
        CMOS_INTERRUPT => "CMOS/RTC",
        SYSCALL_INTERRUPT => "System Call",
        _ => "Unknown",
    }
}

/// Print interrupt statistics
pub fn print_interrupt_stats() {
    let stats = get_interrupt_stats();
    
    debug!("Interrupt Statistics:");
    debug!("  Total Interrupts: {}", stats.total_interrupts);
    debug!("  Timer Interrupts: {}", stats.timer_interrupts);
    debug!("  Keyboard Interrupts: {}", stats.keyboard_interrupts);
    debug!("  Serial Interrupts: {}", stats.serial_interrupts);
    debug!("  Timer Rate: {} Hz", stats.timer_rate_hz);
    if stats.timer_last_trigger > 0 {
        debug!("  Last Timer Interrupt: {} ns ago", get_system_time() - stats.timer_last_trigger);
    }
}
