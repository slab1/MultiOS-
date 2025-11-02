//! System call interface and handling
//! 
//! This module provides system call infrastructure including:
//! - System call entry and exit handling
//! - Parameter validation and copying
//! - System call number definitions
//! - Error handling and reporting

use spin::Mutex;
use log::{debug, warn, error};
use crate::KernelResult;
use crate::arch::syscall::{SyscallContext, SyscallParam, SyscallResult, MAX_SYSCALL_PARAMS};

/// System call error types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyscallError {
    InvalidArgument,
    PermissionDenied,
    ResourceUnavailable,
    InvalidAddress,
    InvalidFileDescriptor,
    NotImplemented,
    SystemError,
    Timeout,
}

/// Maximum system call name length for debugging
const MAX_SYSCALL_NAME_LEN: usize = 32;

/// System call statistics
#[derive(Debug, Clone)]
pub struct SyscallStats {
    pub total_calls: u64,
    pub successful_calls: u64,
    pub failed_calls: u64,
    pub last_call_time: u64,
    pub average_execution_time_ns: u64,
}

static SYSCALL_STATS: Mutex<SyscallStats> = Mutex::new(SyscallStats {
    total_calls: 0,
    successful_calls: 0,
    failed_calls: 0,
    last_call_time: 0,
    average_execution_time_ns: 0,
});

/// System call name lookup table
const SYSCALL_NAMES: &[&str] = &[
    "test",           // 0
    "exit",           // 1
    "write",          // 2
    "read",           // 3
    "open",           // 4
    "close",          // 5
    "mmap",           // 6
    "munmap",         // 7
    "getpid",         // 8
    "fork",           // 9
    "exec",           // 10
];

/// Initialize system call interface
pub fn init() -> KernelResult<()> {
    debug!("Initializing system call interface...");
    
    // Initialize statistics
    let mut stats = SYSCALL_STATS.lock();
    stats.total_calls = 0;
    stats.successful_calls = 0;
    stats.failed_calls = 0;
    stats.last_call_time = 0;
    stats.average_execution_time_ns = 0;
    
    debug!("System call interface initialized");
    
    Ok(())
}

/// Handle a system call
pub fn handle_syscall(context: SyscallContext) -> KernelResult<SyscallResult> {
    let start_time = get_current_time_ns();
    let syscall_num = context.syscall_number as usize;
    
    debug!("System call {} ({}) started",
           syscall_num,
           get_syscall_name(syscall_num));
    
    // Update statistics
    let mut stats = SYSCALL_STATS.lock();
    stats.total_calls += 1;
    
    // Validate system call number
    if syscall_num >= SYSCALL_NAMES.len() {
        warn!("Invalid system call number: {}", syscall_num);
        stats.failed_calls += 1;
        return Err(SyscallError::NotImplemented.into());
    }
    
    // Validate parameters based on system call
    match validate_parameters(&context) {
        Ok(_) => debug!("Parameters validated for syscall {}", syscall_num),
        Err(e) => {
            warn!("Parameter validation failed for syscall {}: {:?}", syscall_num, e);
            stats.failed_calls += 1;
            return Err(e.into());
        }
    }
    
    // Dispatch to appropriate handler
    let result = dispatch_syscall(context);
    
    // Update statistics based on result
    let end_time = get_current_time_ns();
    let execution_time = end_time - start_time;
    
    match result {
        Ok(_) => {
            stats.successful_calls += 1;
            debug!("System call {} completed successfully in {} ns",
                   syscall_num, execution_time);
        }
        Err(_) => {
            stats.failed_calls += 1;
            warn!("System call {} failed in {} ns", syscall_num, execution_time);
        }
    }
    
    // Update timing statistics
    stats.last_call_time = end_time;
    let total_time = stats.average_execution_time_ns * (stats.successful_calls + stats.failed_calls - 1) + execution_time;
    stats.average_execution_time_ns = total_time / stats.total_calls;
    
    result
}

/// Validate system call parameters
fn validate_parameters(context: &SyscallContext) -> KernelResult<()> {
    let syscall_num = context.syscall_number as usize;
    
    match syscall_num {
        SYSCALL_WRITE => {
            // Validate buffer pointer and length
            let buf = context.parameters[1];
            let len = context.parameters[2];
            
            if buf == 0 {
                return Err(SyscallError::InvalidAddress.into());
            }
            
            if len > MAX_SYSCALL_BUFFER_SIZE {
                return Err(SyscallError::InvalidArgument.into());
            }
        }
        
        SYSCALL_READ => {
            // Validate buffer pointer and length
            let buf = context.parameters[1];
            let len = context.parameters[2];
            
            if buf == 0 {
                return Err(SyscallError::InvalidAddress.into());
            }
            
            if len > MAX_SYSCALL_BUFFER_SIZE {
                return Err(SyscallError::InvalidArgument.into());
            }
        }
        
        SYSCALL_OPEN => {
            // Validate pathname pointer
            let pathname = context.parameters[0];
            
            if pathname == 0 {
                return Err(SyscallError::InvalidAddress.into());
            }
        }
        
        SYSCALL_CLOSE => {
            // Validate file descriptor
            let fd = context.parameters[0] as i32;
            
            if fd < 0 || fd > MAX_SYSCALL_FD {
                return Err(SyscallError::InvalidFileDescriptor.into());
            }
        }
        
        SYSCALL_MMAP => {
            // Validate memory mapping parameters
            let len = context.parameters[1];
            
            if len == 0 || len > MAX_SYSCALL_MMAP_SIZE {
                return Err(SyscallError::InvalidArgument.into());
            }
        }
        
        SYSCALL_MUNMAP => {
            // Validate munmap parameters
            let addr = context.parameters[0];
            let len = context.parameters[1];
            
            if addr == 0 && len == 0 {
                return Ok(()); // This is valid (no-op)
            }
            
            if addr == 0 || len == 0 {
                return Err(SyscallError::InvalidAddress.into());
            }
        }
        
        // Add validation for other system calls as needed
        _ => {
            // For now, other system calls don't have additional validation
        }
    }
    
    Ok(())
}

/// Dispatch system call to appropriate handler
fn dispatch_syscall(context: SyscallContext) -> KernelResult<SyscallResult> {
    let syscall_num = context.syscall_number as u64;
    
    match syscall_num {
        SYSCALL_TEST => {
            debug!("Test system call received");
            Ok(42) // Return 42 for testing
        }
        
        SYSCALL_EXIT => {
            handle_syscall_exit(context.parameters[0] as i32)
        }
        
        SYSCALL_WRITE => {
            handle_syscall_write(
                context.parameters[0] as i32,
                context.parameters[1] as *const u8,
                context.parameters[2] as usize,
            )
        }
        
        SYSCALL_READ => {
            handle_syscall_read(
                context.parameters[0] as i32,
                context.parameters[1] as *mut u8,
                context.parameters[2] as usize,
            )
        }
        
        SYSCALL_OPEN => {
            handle_syscall_open(
                context.parameters[0] as *const u8,
                context.parameters[1] as i32,
            )
        }
        
        SYSCALL_CLOSE => {
            handle_syscall_close(context.parameters[0] as i32)
        }
        
        SYSCALL_GETPID => {
            handle_syscall_getpid()
        }
        
        SYSCALL_MMAP => {
            handle_syscall_mmap(
                context.parameters[0] as usize,
                context.parameters[1] as usize,
                context.parameters[2] as i32,
                context.parameters[3] as i32,
                context.parameters[4] as i32,
                context.parameters[5] as off_t,
            )
        }
        
        SYSCALL_MUNMAP => {
            handle_syscall_munmap(
                context.parameters[0] as *const u8,
                context.parameters[1] as usize,
            )
        }
        
        SYSCALL_FORK => {
            handle_syscall_fork()
        }
        
        SYSCALL_EXEC => {
            handle_syscall_exec(
                context.parameters[0] as *const u8,
                context.parameters[1] as *const *const u8,
            )
        }
        
        _ => {
            warn!("Unknown system call: {}", syscall_num);
            Err(SyscallError::NotImplemented.into())
        }
    }
}

// Individual System Call Implementations

fn handle_syscall_exit(status: i32) -> KernelResult<SyscallResult> {
    debug!("Process exit requested with status: {}", status);
    
    // In a real implementation, this would:
    // 1. Clean up process resources
    // 2. Release memory
    // 3. Close file descriptors
    // 4. Signal parent process if applicable
    // 5. Remove process from scheduler
    
    Ok(0)
}

fn handle_syscall_write(fd: i32, buf: *const u8, count: usize) -> KernelResult<SyscallResult> {
    debug!("Write syscall: fd={}, buf={:#x}, count={}", fd, buf as usize, count);
    
    if buf.is_null() {
        return Err(SyscallError::InvalidAddress.into());
    }
    
    if count > MAX_SYSCALL_BUFFER_SIZE {
        return Err(SyscallError::InvalidArgument.into());
    }
    
    // Simple implementation - write to serial console for stdout/stderr
    if fd == 1 || fd == 2 {
        unsafe {
            let slice = core::slice::from_raw_parts(buf, count);
            for &byte in slice {
                if byte == b'\n' {
                    write_serial_byte(b'\r');
                }
                write_serial_byte(byte);
            }
        }
        Ok(count as i64)
    } else {
        Err(SyscallError::InvalidFileDescriptor.into())
    }
}

fn handle_syscall_read(fd: i32, buf: *mut u8, count: usize) -> KernelResult<SyscallResult> {
    debug!("Read syscall: fd={}, buf={:#x}, count={}", fd, buf as usize, count);
    
    if buf.is_null() {
        return Err(SyscallError::InvalidAddress.into());
    }
    
    if count > MAX_SYSCALL_BUFFER_SIZE {
        return Err(SyscallError::InvalidArgument.into());
    }
    
    // For now, return 0 (no input available)
    // In a real implementation, this would read from console or files
    Ok(0)
}

fn handle_syscall_open(pathname: *const u8, flags: i32) -> KernelResult<SyscallResult> {
    debug!("Open syscall: pathname={:#x}, flags={:#x}", pathname as usize, flags as u32);
    
    if pathname.is_null() {
        return Err(SyscallError::InvalidAddress.into());
    }
    
    // Simple implementation - just return a dummy file descriptor
    Ok(3)
}

fn handle_syscall_close(fd: i32) -> KernelResult<SyscallResult> {
    debug!("Close syscall: fd={}", fd);
    
    if fd < 0 || fd > MAX_SYSCALL_FD {
        return Err(SyscallError::InvalidFileDescriptor.into());
    }
    
    Ok(0)
}

fn handle_syscall_getpid() -> KernelResult<SyscallResult> {
    debug!("GetPID syscall");
    
    // Simple implementation - return dummy process ID
    Ok(1)
}

fn handle_syscall_mmap(
    addr: usize,
    len: usize,
    prot: i32,
    flags: i32,
    fd: i32,
    offset: off_t,
) -> KernelResult<SyscallResult> {
    debug!("MMap syscall: addr={:#x}, len={}, prot={:#x}, flags={:#x}, fd={}, offset={}",
           addr, len, prot as u32, flags as u32, fd, offset);
    
    if len == 0 || len > MAX_SYSCALL_MMAP_SIZE {
        return Err(SyscallError::InvalidArgument.into());
    }
    
    // Simple implementation - just return the requested address
    // In a real implementation, this would allocate and map memory
    Ok(addr as i64)
}

fn handle_syscall_munmap(addr: *const u8, len: usize) -> KernelResult<SyscallResult> {
    debug!("MUnMap syscall: addr={:#x}, len={}", addr as usize, len);
    
    if addr.is_null() && len == 0 {
        return Ok(0); // This is valid (no-op)
    }
    
    if addr.is_null() || len == 0 {
        return Err(SyscallError::InvalidAddress.into());
    }
    
    // Simple implementation
    Ok(0)
}

fn handle_syscall_fork() -> KernelResult<SyscallResult> {
    debug!("Fork syscall");
    
    // Simple implementation - return child process ID
    Ok(2)
}

fn handle_syscall_exec(pathname: *const u8, argv: *const *const u8) -> KernelResult<SyscallResult> {
    debug!("Exec syscall: pathname={:#x}, argv={:#x}", pathname as usize, argv as usize);
    
    if pathname.is_null() {
        return Err(SyscallError::InvalidAddress.into());
    }
    
    // Simple implementation
    Ok(0)
}

// Helper Functions

fn get_syscall_name(syscall_num: usize) -> &'static str {
    if syscall_num < SYSCALL_NAMES.len() {
        SYSCALL_NAMES[syscall_num]
    } else {
        "unknown"
    }
}

fn write_serial_byte(byte: u8) {
    // Write byte to serial console
    // In a real implementation, this would write to UART
    debug!("Serial write: {}", byte as char);
}

fn get_current_time_ns() -> u64 {
    // In a real implementation, this would read from a high-resolution timer
    0
}

// Constants
const MAX_SYSCALL_BUFFER_SIZE: usize = 4096;
const MAX_SYSCALL_FD: i32 = 1024;
const MAX_SYSCALL_MMAP_SIZE: usize = 64 * 1024 * 1024; // 64 MB

/// Get system call statistics
pub fn get_syscall_stats() -> SyscallStats {
    *SYSCALL_STATS.lock()
}

/// Print system call statistics
pub fn print_syscall_stats() {
    let stats = get_syscall_stats();
    
    debug!("System Call Statistics:");
    debug!("  Total Calls: {}", stats.total_calls);
    debug!("  Successful: {}", stats.successful_calls);
    debug!("  Failed: {}", stats.failed_calls);
    if stats.total_calls > 0 {
        let success_rate = (stats.successful_calls * 100) / stats.total_calls;
        debug!("  Success Rate: {}%", success_rate);
    }
    debug!("  Average Execution Time: {} ns", stats.average_execution_time_ns);
}
