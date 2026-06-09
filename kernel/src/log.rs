//! MultiOS Bootstrap Logger
//! 
//! This module provides logging functionality for the bootstrap sequence.
//! It uses a simple console output mechanism that works during early boot.

use core::fmt::{self, Write};
use core::sync::atomic::{AtomicU16, Ordering};

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
    vga_cursor: AtomicU16,  // Tracks VGA cursor position
}

/// Global logger instance
static mut LOGGER: BootstrapLogger = BootstrapLogger {
    level: LogLevel::Info,
    vga_cursor: AtomicU16::new(0),
};

/// Initialize the bootstrap logger
pub fn init_logger(level: LogLevel) {
    unsafe {
        LOGGER.level = level;
        LOGGER.vga_cursor.store(0, Ordering::SeqCst);
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

/// VGA console logging with cursor position tracking
fn vga_log(level_str: &str, msg: &str) {
    unsafe {
        // VGA text mode buffer starts at 0xB8000 in x86 real mode
        const VGA_ADDRESS: *mut u8 = 0xB8000 as *mut u8;
        const VGA_COLS: u16 = 80;
        const VGA_ROWS: u16 = 25;
        const VGA_CELLS: u16 = VGA_COLS * VGA_ROWS;
        
        let cursor = LOGGER.vga_cursor.load(Ordering::SeqCst);
        let mut pos = cursor;
        
        // Write combined string (level + msg)
        for byte in level_str.bytes().chain(msg.bytes()).chain(b"\r\n".iter().copied()) {
            if byte == b'\n' {
                // Newline: advance to next row
                pos = (pos / VGA_COLS + 1) * VGA_COLS;
            } else if byte == b'\r' {
                // Carriage return: go to start of current row
                pos = (pos / VGA_COLS) * VGA_COLS;
            } else if pos < VGA_CELLS {
                // Write character + attribute
                VGA_ADDRESS.add((pos * 2) as usize).write(byte);
                VGA_ADDRESS.add((pos * 2 + 1) as usize).write(0x07); // Light gray on black
                pos += 1;
            }
            
            // Handle screen scrolling (simple: wrap to top)
            if pos >= VGA_CELLS {
                // Scroll: move everything up one row
                for row in 0..(VGA_ROWS - 1) {
                    for col in 0..VGA_COLS {
                        let src = ((row + 1) * VGA_COLS + col) * 2;
                        let dst = (row * VGA_COLS + col) * 2;
                        VGA_ADDRESS.add(dst as usize).write(VGA_ADDRESS.add(src as usize).read());
                        VGA_ADDRESS.add((dst + 1) as usize).write(VGA_ADDRESS.add((src + 1) as usize).read());
                    }
                }
                // Clear last row
                for col in 0..VGA_COLS {
                    let offset = ((VGA_ROWS - 1) * VGA_COLS + col) * 2;
                    VGA_ADDRESS.add(offset as usize).write(b' ');
                    VGA_ADDRESS.add((offset + 1) as usize).write(0x07);
                }
                pos = (VGA_ROWS - 1) * VGA_COLS;
            }
        }
        
        LOGGER.vga_cursor.store(pos, Ordering::SeqCst);
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
    #[cfg(target_arch = "x86_64")]
    core::arch::asm!("out dx, al", in("dx") port, in("al") value, options(nomem, nostack));
    #[cfg(not(target_arch = "x86_64"))]
    {
        let _ = (port, value); // No-op on non-x86
    }
}

/// Input byte from port
unsafe fn inb(port: u16) -> u8 {
    #[cfg(target_arch = "x86_64")]
    {
        let result: u8;
        core::arch::asm!("in al, dx", out("al") result, in("dx") port, options(nomem, nostack));
        result
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        let _ = port;
        0x60 // Mock value: always ready on non-x86
    }
}

/// Simple Write implementation for core::fmt
impl fmt::Write for BootstrapLogger {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        info(s);
        Ok(())
    }
}
