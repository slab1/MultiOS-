//! MultiOS Interrupt Handling Infrastructure
//! 
//! This module provides comprehensive interrupt handling for multiple architectures,
//! including interrupt descriptor tables, interrupt handlers, and system call support.

use crate::log::{info, warn, error, debug};
use crate::ArchType;
use crate::KernelError;

/// Interrupt handling result
pub type InterruptResult<T> = Result<T, InterruptError>;

/// Interrupt handling errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterruptError {
    InvalidInterruptNumber,
    HandlerNotFound,
    HandlerRegistrationFailed,
    InvalidHandler,
    IdtInitializationFailed,
    SystemCallInvalid,
    SystemCallNotImplemented,
    ParameterValidationFailed,
    PrivilegeViolation,
}

/// Interrupt types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum InterruptType {
    Exception = 0,      // CPU exceptions (faults, traps, aborts)
    Hardware = 1,      // Hardware interrupts (IRQ)
    SystemCall = 2,    // Software interrupts (syscalls)
    Software = 3,      // Other software-generated interrupts
}

/// Interrupt privilege levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PrivilegeLevel {
    Ring0 = 0, // Kernel level
    Ring1 = 1,
    Ring2 = 2,
    Ring3 = 3, // User level
}

/// Interrupt descriptor flags
#[derive(Debug, Clone, Copy)]
pub struct InterruptFlags {
    pub present: bool,
    pub privilege_level: PrivilegeLevel,
    pub interrupt_type: InterruptType,
    pub dpl: u8, // Descriptor privilege level
}

/// System call parameters
#[derive(Debug, Clone, Copy)]
pub struct SystemCallParams {
    pub syscall_number: usize,
    pub arg0: usize,
    pub arg1: usize,
    pub arg2: usize,
    pub arg3: usize,
    pub arg4: usize,
    pub arg5: usize,
    pub caller_priv_level: PrivilegeLevel,
}

/// System call result
#[derive(Debug, Clone, Copy)]
pub struct SystemCallResult {
    pub return_value: usize,
    pub error_code: InterruptError,
}

/// Hardware interrupt information
#[derive(Debug, Clone, Copy)]
pub struct HardwareInterrupt {
    pub interrupt_number: usize,
    pub device_id: u32,
    pub interrupt_count: u64,
    pub last_timestamp: u64,
}

/// System call number definitions
pub mod syscall_numbers {
    // Process management
    pub const PROCESS_CREATE: usize = 1;
    pub const PROCESS_EXIT: usize = 2;
    pub const PROCESS_WAIT: usize = 3;
    pub const PROCESS_GETPID: usize = 4;
    pub const PROCESS_GETPPID: usize = 5;
    
    // Thread management
    pub const THREAD_CREATE: usize = 10;
    pub const THREAD_EXIT: usize = 11;
    pub const THREAD_JOIN: usize = 12;
    pub const THREAD_YIELD: usize = 13;
    pub const THREAD_GETTID: usize = 14;
    pub const THREAD_SET_PRIORITY: usize = 15;
    pub const THREAD_GET_PRIORITY: usize = 16;
    
    // Memory management
    pub const VIRTUAL_ALLOC: usize = 20;
    pub const VIRTUAL_FREE: usize = 21;
    pub const VIRTUAL_MAP: usize = 22;
    pub const VIRTUAL_UNMAP: usize = 23;
    pub const PHYSICAL_ALLOC: usize = 24;
    pub const PHYSICAL_FREE: usize = 25;
    
    // File and I/O
    pub const FILE_OPEN: usize = 30;
    pub const FILE_CLOSE: usize = 31;
    pub const FILE_READ: usize = 32;
    pub const FILE_WRITE: usize = 33;
    pub const FILE_SEEK: usize = 34;
    pub const FILE_STAT: usize = 35;
    pub const DIRECTORY_CREATE: usize = 36;
    pub const DIRECTORY_READ: usize = 37;
    
    // Inter-process communication
    pub const IPC_SEND: usize = 40;
    pub const IPC_RECEIVE: usize = 41;
    pub const IPC_POLL: usize = 42;
    pub const MESSAGE_QUEUE_CREATE: usize = 43;
    pub const MESSAGE_QUEUE_SEND: usize = 44;
    pub const MESSAGE_QUEUE_RECEIVE: usize = 45;
    
    // Synchronization
    pub const MUTEX_CREATE: usize = 50;
    pub const MUTEX_LOCK: usize = 51;
    pub const MUTEX_UNLOCK: usize = 52;
    pub const CONDITION_CREATE: usize = 53;
    pub const CONDITION_WAIT: usize = 54;
    pub const CONDITION_SIGNAL: usize = 55;
    pub const SEMAPHORE_CREATE: usize = 56;
    pub const SEMAPHORE_WAIT: usize = 57;
    pub const SEMAPHORE_POST: usize = 58;
    
    // Device I/O
    pub const DEVICE_OPEN: usize = 60;
    pub const DEVICE_CLOSE: usize = 61;
    pub const DEVICE_READ: usize = 62;
    pub const DEVICE_WRITE: usize = 63;
    pub const DEVICE_IOCTL: usize = 64;
    pub const INTERRUPT_REGISTER: usize = 65;
    pub const INTERRUPT_UNREGISTER: usize = 66;
    
    // System information
    pub const SYSTEM_INFO: usize = 70;
    pub const MEMORY_INFO: usize = 71;
    pub const CPU_INFO: usize = 72;
    pub const TIME_GET: usize = 73;
    pub const TIME_SET: usize = 74;
    pub const CLOCK_GETTIME: usize = 75;
    
    // Security and access control
    pub const SECURITY_CHECK: usize = 80;
    pub const RESOURCE_LIMIT: usize = 81;
    pub const PERMISSION_SET: usize = 82;
    pub const AUDIT_LOG: usize = 83;
    
    // File operations - extended
    pub const FILE_LOCK: usize = 84;
    pub const FILE_UNLOCK: usize = 85;
    pub const FILE_TRUNCATE: usize = 86;
    pub const FILE_DUP: usize = 87;
    pub const FILE_DUP2: usize = 88;
    pub const FILE_CHMOD: usize = 89;
    pub const FILE_CHOWN: usize = 90;
    pub const FILE_RENAME: usize = 91;
    pub const FILE_REMOVE: usize = 92;
    pub const FILE_SYMLINK_CREATE: usize = 93;
    pub const FILE_READLINK: usize = 94;
    
    // Debug and monitoring
    pub const DEBUG_SET_BREAKPOINT: usize = 90;
    pub const DEBUG_REMOVE_BREAKPOINT: usize = 91;
    pub const PROFILING_START: usize = 92;
    pub const PROFILING_STOP: usize = 93;
    pub const TRACE_MARKER: usize = 94;
}

/// Common interrupt numbers across architectures
pub mod interrupt_numbers {
    // Exception numbers (x86_64)
    pub const EXCEPTION_DE: usize = 0;  // Divide Error
    pub const EXCEPTION_DB: usize = 1;  // Debug
    pub const EXCEPTION_NMI: usize = 2; // Non-maskable Interrupt
    pub const EXCEPTION_BP: usize = 3;  // Breakpoint
    pub const EXCEPTION_OF: usize = 4;  // Overflow
    pub const EXCEPTION_BR: usize = 5;  // BOUND Range Exceeded
    pub const EXCEPTION_UD: usize = 6;  // Invalid Opcode
    pub const EXCEPTION_NM: usize = 7;  // Device Not Available
    pub const EXCEPTION_DF: usize = 8;  // Double Fault
    pub const EXCEPTION_TS: usize = 10; // Invalid TSS
    pub const EXCEPTION_NP: usize = 11; // Segment Not Present
    pub const EXCEPTION_SS: usize = 12; // Stack-Segment Fault
    pub const EXCEPTION_GP: usize = 13; // General Protection Fault
    pub const EXCEPTION_PF: usize = 14; // Page Fault
    pub const EXCEPTION_MF: usize = 16; // x87 Floating-Point Exception
    pub const EXCEPTION_AC: usize = 17; // Alignment Check
    pub const EXCEPTION_MC: usize = 18; // Machine Check
    pub const EXCEPTION_XF: usize = 19; // SIMD Floating-Point Exception
    pub const EXCEPTION_VE: usize = 20; // Virtualization Exception
    
    // Hardware interrupts (IRQ base)
    pub const IRQ_TIMER: usize = 32;           // Timer interrupt
    pub const IRQ_KEYBOARD: usize = 33;        // Keyboard interrupt
    pub const IRQ_COM1: usize = 36;            // Serial port 1 (COM1)
    pub const IRQ_COM2: usize = 35;            // Serial port 2 (COM2)
    pub const IRQ_LPT2: usize = 37;            // Parallel port 2
    pub const IRQ_FLOPPY: usize = 38;          // Floppy disk
    pub const IRQ_COM4: usize = 39;            // Serial port 4 (COM4)
    pub const IRQ_COM3: usize = 34;            // Serial port 3 (COM3)
    pub const IRQ_LPT1: usize = 41;            // Parallel port 1
    pub const IRQ_CMOS: usize = 42;            // CMOS clock
    pub const IRQ_FREE1: usize = 43;           // Free for peripherals
    pub const IRQ_FREE2: usize = 44;           // Free for peripherals
    pub const IRQ_FREE3: usize = 45;           // Free for peripherals
    pub const IRQ_PS2_MOUSE: usize = 47;       // PS/2 mouse
    pub const IRQ_FPU: usize = 48;             // FPU
    pub const IRQ_PRIMARY_ATA: usize = 49;     // Primary ATA
    pub const IRQ_SECONDARY_ATA: usize = 50;   // Secondary ATA
    pub const IRQ_ATA3: usize = 51;            // ATA channel 3
    pub const IRQ_ATA4: usize = 52;            // ATA channel 4
    pub const IRQ_MSI0: usize = 32 + 16;       // Message Signaled Interrupt 0
    pub const IRQ_MSI15: usize = 47 + 16;      // Message Signaled Interrupt 15
    
    // Software interrupts
    pub const SYSCALL_INTERRUPT: usize = 128;  // System call interrupt (x86_64)
    pub const SYSCALL_INTERRUPT_RISCV: usize = 8;  // System call interrupt (RISC-V)
    pub const SYSCALL_INTERRUPT_ARM: usize = 8;    // System call interrupt (ARM64)
}

/// Interrupt statistics
#[derive(Debug, Clone, Copy)]
pub struct InterruptStats {
    pub total_interrupts: u64,
    pub exceptions: u64,
    pub hardware_interrupts: u64,
    pub system_calls: u64,
    pub software_interrupts: u64,
    pub last_interrupt: u64,
    pub interrupt_rate: f64, // Interrupts per second
}

/// Initialize interrupt handling system
pub fn init_interrupt_system(arch: ArchType) -> InterruptResult<()> {
    info!("Initializing interrupt handling system for {:?}", arch);
    
    match arch {
        ArchType::X86_64 => {
            // Initialize x86_64 interrupt handling
            crate::arch::x86_64::interrupt::init_idt()?;
            crate::arch::x86_64::interrupt::setup_exception_handlers()?;
            crate::arch::x86_64::interrupt::setup_system_call_handler()?;
            
            // Initialize PIC/APIC for hardware interrupts
            crate::arch::x86_64::pic::init_pic()?;
            crate::arch::x86_64::apic::init_apic()?;
        }
        ArchType::AArch64 => {
            // Initialize ARM64 interrupt handling
            crate::arch::aarch64::interrupt::init_exception_level_handlers()?;
            crate::arch::aarch64::interrupt::setup_system_call_handler()?;
            
            // Initialize GIC for hardware interrupts
            crate::arch::aarch64::gic::init_gic()?;
        }
        ArchType::Riscv64 => {
            // Initialize RISC-V interrupt handling
            crate::arch::riscv64::interrupt::init_exception_handlers()?;
            crate::arch::riscv64::interrupt::setup_system_call_handler()?;
            
            // Initialize CLINT/PLIC for hardware interrupts
            crate::arch::riscv64::clint::init_clint()?;
            crate::arch::riscv64::plic::init_plic()?;
        }
        ArchType::Unknown => {
            return Err(InterruptError::IdtInitializationFailed);
        }
    }
    
    info!("Interrupt handling system initialized successfully");
    Ok(())
}

/// Common interrupt handler functions
pub mod handlers {
    use super::*;
    
    /// Handle timer interrupt
    pub fn timer_interrupt_handler() {
        crate::scheduler::timer_interrupt_occurred();
        
        // Update timer statistics
        // This will be used for scheduling and time-related operations
        // debug!("Timer interrupt occurred");
    }
    
    /// Handle keyboard interrupt
    pub fn keyboard_interrupt_handler() {
        // Read keyboard input and place in buffer
        debug!("Keyboard interrupt occurred");
        
        // Process keyboard input
        if let Some(keycode) = crate::drivers::keyboard::read_keycode() {
            crate::drivers::keyboard::process_keycode(keycode);
        }
    }
    
    /// Handle page fault exception
    pub fn page_fault_handler(fault_addr: usize, error_code: usize, instruction_ptr: usize) {
        error!("Page fault at address {:#x}, error code {:#x}, instruction {:#x}", 
               fault_addr, error_code, instruction_ptr);
        
        // Handle page fault through memory manager
        let result = crate::memory::get_global_memory_manager()
            .map(|mut manager| manager.handle_page_fault(fault_addr as u64, error_code as u64, instruction_ptr as u64));
            
        if let Err(e) = result {
            error!("Page fault handling failed: {:?}", e);
        }
    }
    
    /// Handle division by zero exception
    pub fn divide_error_handler() {
        error!("Divide by zero exception occurred");
        // Kill the current process/thread
    }
    
    /// Handle invalid opcode exception
    pub fn invalid_opcode_handler() {
        error!("Invalid opcode exception occurred");
        // Kill the current process/thread
    }
    
    /// Handle general protection fault
    pub fn general_protection_fault_handler(error_code: usize) {
        error!("General protection fault with error code {:#x}", error_code);
        // Kill the current process/thread
    }
}