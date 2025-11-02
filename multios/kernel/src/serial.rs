//! Serial console module
//! 
//! This module provides serial console functionality for output.

use log::debug;
use x86_64::instructions::port::Port;

/// Serial port base addresses
const COM1_BASE: u16 = 0x3F8;
const COM2_BASE: u16 = 0x2F8;

/// Serial port registers (offset from base)
const REG_DATA: u16 = 0;
const REG_INTERRUPT_ENABLE: u16 = 1;
const REG_LINE_CONTROL: u16 = 3;
const REG_LINE_STATUS: u16 = 5;

/// Initialize serial console
pub fn init() {
    debug!("Initializing serial console...");
    
    // Initialize COM1
    init_port(COM1_BASE);
    
    debug!("Serial console initialized");
}

/// Initialize a serial port
fn init_port(base: u16) {
    // Disable interrupts
    let mut interrupt_enable = Port::new(base + REG_INTERRUPT_ENABLE);
    interrupt_enable.write(0x00u8);
    
    // Enable DLAB (divisor latch access bit)
    let mut line_control = Port::new(base + REG_LINE_CONTROL);
    line_control.write(0x80u8);
    
    // Set divisor to 1 (115200 baud)
    let mut data = Port::new(base + REG_DATA);
    data.write(0x01u8);
    let mut interrupt_enable_2 = Port::new(base + REG_INTERRUPT_ENABLE);
    interrupt_enable_2.write(0x00u8);
    
    // 8 bits, no parity, 1 stop bit
    line_control.write(0x03u8);
}

/// Check if serial port is ready to send
fn is_transmit_ready(base: u16) -> bool {
    let mut line_status = Port::new(base + REG_LINE_STATUS);
    let status = line_status.read();
    status & 0x20 != 0
}

/// Write a byte to serial port
fn write_byte_to_port(base: u16, byte: u8) {
    // Wait for port to be ready
    while !is_transmit_ready(base) {
        core::arch::asm!("pause");
    }
    
    let mut data = Port::new(base + REG_DATA);
    data.write(byte);
}

/// Write a byte to COM1
pub fn write_byte(byte: u8) {
    write_byte_to_port(COM1_BASE, byte);
}

/// Write a byte to COM2
pub fn write_byte_com2(byte: u8) {
    write_byte_to_port(COM2_BASE, byte);
}

/// Write a string to serial console
pub fn write_string(text: &str) {
    for byte in text.bytes() {
        write_byte(byte);
    }
}

/// Write a formatted string to serial console
pub fn write_formatted(format: &str, args: &[&dyn core::fmt::Display]) {
    // Simple formatting - just concatenate for now
    // In a real implementation, would use proper formatting
    for arg in args {
        write_string(&format!("{}", arg));
    }
}
