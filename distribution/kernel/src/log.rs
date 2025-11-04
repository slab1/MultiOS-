//! MultiOS Bootstrap Logger
//! 
//! This module provides logging functionality for the bootstrap sequence.
//! It uses a simple console output mechanism that works during early boot.

use core::fmt::{self, Write};
use crate::ArchType;

/// Log levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum LogLevel {
    Error = 0,
    Warn = 1,
    Info = 2,
    Debug = 3,
}

/// Simple logger that outputs to VGA/SERIAL during bootstrap
pub struct BootstrapLogger {
    level: LogLevel,
}

/// Global logger instance
static mut LOGGER: BootstrapLogger = BootstrapLogger { level: LogLevel::Info };

/// Initialize the bootstrap logger
pub fn init_logger(level: LogLevel) {
    unsafe {
        LOGGER.level = level;
    }
}

/// Log error message
pub fn error(msg: &str) {
    log(LogLevel::Error, msg);
}

/// Log warning message
pub fn warn(msg: &str) {
    log(LogLevel::Warn, msg);
}

/// Log info message
pub fn info(msg: &str) {
    log(LogLevel::Info, msg);
}

/// Log debug message
pub fn debug(msg: &str) {
    log(LogLevel::Debug, msg);
}

/// Internal log function
fn log(level: LogLevel, msg: &str) {
    unsafe {
        if level <= LOGGER.level {
            let level_str = match level {
                LogLevel::Error => "[ERROR] ",
                LogLevel::Warn => "[WARN]  ",
                LogLevel::Info => "[INFO]  ",
                LogLevel::Debug => "[DEBUG] ",
            };
            
            // Try serial console first
            if try_serial_log(level_str, msg) {
                return;
            }
            
            // Fall back to VGA console
            vga_log(level_str, msg);
        }
    }
}

/// Try to log via serial port
fn try_serial_log(level_str: &str, msg: &str) -> bool {
    unsafe {
        // Check if serial port is available
        let serial_ready = inb(0x3F8 + 5) & 0x20;
        if serial_ready != 0 {
            serial_write_str(level_str);
            serial_write_str(msg);
            serial_write_str("\r\n");
            return true;
        }
    }
    false
}

/// VGA console logging
fn vga_log(level_str: &str, msg: &str) {
    unsafe {
        // VGA text mode buffer
        let vga_buffer = 0xb8000 as *mut u8;
        
        let combined = format!("{}{}\r\n", level_str, msg);
        
        for (i, byte) in combined.bytes().enumerate() {
            if i < 80 * 25 * 2 {
                vga_buffer.add(i * 2).write(byte);
                vga_buffer.add(i * 2 + 1).write(0x07); // Light gray on black
            }
        }
    }
}

/// Write string to serial port
fn serial_write_str(s: &str) {
    unsafe {
        for byte in s.bytes() {
            outb(0x3F8, byte);
        }
    }
}

/// Output byte to port
unsafe fn outb(port: u16, value: u8) {
    core::arch::asm!("out dx, al", in("dx") port, in("al") value);
}

/// Input byte from port
unsafe fn inb(port: u16) -> u8 {
    let result: u8;
    core::arch::asm!("in al, dx", out("al") result, in("dx") port);
    result
}

/// Simple Write implementation for core::fmt
impl fmt::Write for BootstrapLogger {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        info(s);
        Ok(())
    }
}