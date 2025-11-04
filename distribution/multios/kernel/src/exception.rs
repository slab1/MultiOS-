//! Exception and panic handling for MultiOS
//! 
//! This module provides exception handling and panic recovery.

use log::{error, warn};
use crate::arch::ProcessorState;

/// Panic handler
#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    error!("KERNEL PANIC: {}", info);
    
    // Log panic location
    if let Some(location) = info.location() {
        error!("Panic occurred at {}:{}:{}", location.file(), location.line(), location.column());
    }
    
    // Disable interrupts to prevent further damage
    disable_interrupts();
    
    // Try to save system state
    let state = save_current_state();
    error!("Saved processor state: {:?}", state);
    
    // Halt the system
    loop {
        halt_cpu();
    }
}

/// Exception handler for unhandled exceptions
#[no_mangle]
pub extern "C" fn exception_handler() {
    error!("Unhandled exception occurred!");
    
    // Disable interrupts
    disable_interrupts();
    
    // Get current state
    let state = save_current_state();
    error!("Exception state: {:?}", state);
    
    // For now, halt the system
    loop {
        halt_cpu();
    }
}

/// Save current processor state
fn save_current_state() -> ProcessorState {
    crate::arch::save_state()
}

/// Disable interrupts
fn disable_interrupts() {
    unsafe {
        core::arch::asm!("cli");
    }
}

/// Halt CPU
fn halt_cpu() {
    unsafe {
        core::arch::asm!("hlt");
    }
}

/// Handle double fault
#[no_mangle]
pub extern "C" fn double_fault_handler() -> ! {
    error!("DOUBLE FAULT - System will halt");
    error!("This is a critical error that indicates stack corruption or other serious issues");
    
    disable_interrupts();
    loop {
        halt_cpu();
    }
}

/// Handle page fault
#[no_mangle]
pub extern "C" fn page_fault_handler() -> ! {
    error!("PAGE FAULT - System will halt");
    error!("Invalid memory access detected");
    
    disable_interrupts();
    loop {
        halt_cpu();
    }
}

/// Handle general protection fault
#[no_mangle]
pub extern "C" fn general_protection_fault_handler() -> ! {
    error!("GENERAL PROTECTION FAULT - System will halt");
    error!("Privilege level violation or invalid segment access");
    
    disable_interrupts();
    loop {
        halt_cpu();
    }
}
