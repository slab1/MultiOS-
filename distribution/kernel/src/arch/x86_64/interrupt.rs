//! x86_64 Interrupt Descriptor Table (IDT) Implementation
//! 
//! This module provides x86_64 specific interrupt handling including IDT setup,
//! exception handlers, and system call support.

use crate::arch::interrupts::*;
use crate::log::{info, warn, error};
use crate::KernelError;

const IDT_ENTRIES: usize = 256;
const IDT_GATE_DESCRIPTOR_SIZE: usize = 16;

/// IDT gate types
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum GateType {
    TaskGate = 0x5,
    InterruptGate16 = 0x6,
    TrapGate16 = 0x7,
    InterruptGate32 = 0xE,
    TrapGate32 = 0xF,
    InterruptGate64 = 0xE,
    TrapGate64 = 0xF,
}

/// IDT entry structure
#[repr(C)]
#[derive(Clone, Copy)]
struct IdtEntry {
    offset_low: u16,     // Offset bits 0..15
    selector: u16,       // Code segment selector
    ist_index: u8,       // IST (Interrupt Stack Table) index
    type_attr: u8,       // Type and attributes
    offset_mid: u16,     // Offset bits 16..31
    offset_high: u32,    // Offset bits 32..63
    zero: u16,           // Reserved
}

impl IdtEntry {
    fn new(offset: usize, selector: u16, gate_type: GateType, dpl: u8) -> Self {
        Self {
            offset_low: (offset & 0xFFFF) as u16,
            selector,
            ist_index: 0, // No IST for now
            type_attr: (gate_type as u8) | (1 << 7) | (dpl << 5), // Present bit and DPL
            offset_mid: ((offset >> 16) & 0xFFFF) as u16,
            offset_high: (offset >> 32) as u32,
            zero: 0,
        }
    }
}

/// IDT pointer structure
#[repr(C)]
#[derive(Clone, Copy)]
struct IdtPtr {
    limit: u16,
    base: u64,
}

impl IdtPtr {
    fn new(base: *const IdtEntry, limit: usize) -> Self {
        Self {
            limit: (limit as u16) - 1,
            base: base as u64,
        }
    }
}

/// IDT memory location (aligned to 16 bytes)
static mut IDT_ENTRIES_DATA: [IdtEntry; IDT_ENTRIES] = [IdtEntry {
    offset_low: 0,
    selector: 0,
    ist_index: 0,
    type_attr: 0,
    offset_mid: 0,
    offset_high: 0,
    zero: 0,
}; IDT_ENTRIES_DATA.len()];

/// Global IDT pointer
static mut IDT_PTR: Option<IdtPtr> = None;

/// Initialize the Interrupt Descriptor Table
pub fn init_idt() -> InterruptResult<()> {
    info!("Initializing x86_64 IDT...");
    
    unsafe {
        // Initialize all IDT entries to zero
        IDT_ENTRIES_DATA.fill(IdtEntry {
            offset_low: 0,
            selector: 0,
            ist_index: 0,
            type_attr: 0,
            offset_mid: 0,
            offset_high: 0,
            zero: 0,
        });
        
        // Set up exception handlers
        setup_exception_handlers()?;
        
        // Set up system call handler
        setup_system_call_handler()?;
        
        // Load IDT
        let idt_ptr = IdtPtr::new(IDT_ENTRIES_DATA.as_ptr(), IDT_ENTRIES * IDT_GATE_DESCRIPTOR_SIZE);
        IDT_PTR = Some(idt_ptr);
        
        load_idt(&idt_ptr);
    }
    
    info!("x86_64 IDT initialized successfully");
    Ok(())
}

/// Load IDT using the lidt instruction
unsafe fn load_idt(idt_ptr: &IdtPtr) {
    core::arch::asm!(
        "lidt [{}]",
        in(reg) idt_ptr as *const IdtPtr,
        options(nostack)
    );
}

/// Set up exception handlers in the IDT
pub fn setup_exception_handlers() -> InterruptResult<()> {
    info!("Setting up x86_64 exception handlers...");
    
    // Exception handler addresses
    let handler_ptr = exception_handler_wrapper as usize;
    let code_selector = 0x08; // Ring 0 code segment
    
    // Map all exception vectors to the same handler initially
    for vector in 0..32 {
        unsafe {
            IDT_ENTRIES_DATA[vector] = IdtEntry::new(
                handler_ptr,
                code_selector,
                GateType::InterruptGate64,
                0, // Ring 0 only
            );
        }
    }
    
    info!("x86_64 exception handlers configured");
    Ok(())
}

/// Set up system call handler
pub fn setup_system_call_handler() -> InterruptResult<()> {
    info!("Setting up x86_64 system call handler...");
    
    unsafe {
        let handler_ptr = syscall_handler_wrapper as usize;
        let code_selector = 0x08; // Ring 0 code segment
        
        // System call interrupt vector (0x80 for x86_64)
        IDT_ENTRIES_DATA[0x80] = IdtEntry::new(
            handler_ptr,
            code_selector,
            GateType::InterruptGate64,
            3, // Ring 3 allowed
        );
    }
    
    info!("x86_64 system call handler configured");
    Ok(())
}

/// Common interrupt/exception handler wrapper
extern "C" fn exception_handler_wrapper() {
    unsafe {
        // Get interrupt vector number from CPU
        let vector: u8;
        core::arch::asm!("pushal", "mov $0xFF, {}", out(reg) vector);
        
        // Call the appropriate handler based on vector
        handle_exception(vector as usize);
    }
}

/// System call handler wrapper
extern "C" fn syscall_handler_wrapper() {
    unsafe {
        let syscall_number: usize;
        let arg0: usize;
        let arg1: usize;
        let arg2: usize;
        let arg3: usize;
        let arg4: usize;
        let arg5: usize;
        
        // Read system call number and arguments from registers
        core::arch::asm!(
            "mov %rdi, {}",    // First argument
            "mov %rsi, {}",    // Second argument
            "mov %rdx, {}",    // Third argument
            "mov %r10, {}",    // Fourth argument
            "mov %r8, {}",     // Fifth argument
            "mov %r9, {}",     // Sixth argument
            out(reg) syscall_number,
            out(reg) arg0,
            out(reg) arg1,
            out(reg) arg2,
            out(reg) arg3,
            out(reg) arg4,
            out(reg) arg5,
        );
        
        // Handle system call
        let result = handle_system_call(syscall_number, arg0, arg1, arg2, arg3, arg4, arg5);
        
        // Set return value in RAX
        core::arch::asm!(
            "mov {}, %rax",
            in(reg) result.return_value,
            options(nostack)
        );
    }
}

/// Handle exception based on vector number
fn handle_exception(vector: usize) {
    match vector {
        interrupt_numbers::EXCEPTION_DE => {
            error!("Divide by zero exception (vector {})", vector);
            crate::arch::interrupts::handlers::divide_error_handler();
        }
        interrupt_numbers::EXCEPTION_PF => {
            let fault_addr: usize;
            let error_code: usize;
            let instruction_ptr: usize;
            
            unsafe {
                core::arch::asm!(
                    "push %cr2",
                    "pop {}",
                    out(reg) fault_addr,
                    options(nostack)
                );
                
                // Get error code from stack
                core::arch::asm!(
                    "pop {}",
                    out(reg) error_code,
                    options(nostack)
                );
                
                // Get instruction pointer
                core::arch::asm!(
                    "mov (%rsp), {}",
                    out(reg) instruction_ptr,
                    options(nostack)
                );
            }
            
            error!("Page fault at address {:#x}, error code {:#x}", fault_addr, error_code);
            crate::arch::interrupts::handlers::page_fault_handler(fault_addr, error_code, instruction_ptr);
        }
        interrupt_numbers::EXCEPTION_UD => {
            error!("Invalid opcode exception (vector {})", vector);
            crate::arch::interrupts::handlers::invalid_opcode_handler();
        }
        interrupt_numbers::EXCEPTION_GP => {
            let error_code: usize;
            unsafe {
                core::arch::asm!(
                    "pop {}",
                    out(reg) error_code,
                    options(nostack)
                );
            }
            error!("General protection fault, error code {:#x}", error_code);
            crate::arch::interrupts::handlers::general_protection_fault_handler(error_code);
        }
        interrupt_numbers::IRQ_TIMER => {
            crate::arch::interrupts::handlers::timer_interrupt_handler();
        }
        interrupt_numbers::IRQ_KEYBOARD => {
            crate::arch::interrupts::handlers::keyboard_interrupt_handler();
        }
        _ => {
            warn!("Unhandled interrupt/exception vector: {}", vector);
        }
    }
}

/// Handle system call
fn handle_system_call(syscall_number: usize, arg0: usize, arg1: usize, arg2: usize, 
                     arg3: usize, arg4: usize, arg5: usize) -> SystemCallResult {
    // Validate system call number is within bounds
    if syscall_number >= 1000 { // Leave room for architecture-specific syscalls
        return SystemCallResult {
            return_value: 0,
            error_code: InterruptError::SystemCallInvalid,
        };
    }
    
    info!("System call {} called with args: ({:#x}, {:#x}, {:#x}, {:#x}, {:#x}, {:#x})",
          syscall_number, arg0, arg1, arg2, arg3, arg4, arg5);
    
    // Dispatch to appropriate system call handler
    let result = match syscall_number {
        syscall_numbers::SYSTEM_INFO => handle_syscall_system_info(),
        syscall_numbers::PROCESS_GETPID => handle_syscall_getpid(),
        syscall_numbers::TIME_GET => handle_syscall_time_get(),
        syscall_numbers::MEMORY_INFO => handle_syscall_memory_info(),
        
        // Process management
        syscall_numbers::PROCESS_CREATE => handle_syscall_process_create(arg0, arg1),
        syscall_numbers::PROCESS_EXIT => handle_syscall_process_exit(arg0),
        
        // Thread management
        syscall_numbers::THREAD_CREATE => handle_syscall_thread_create(arg0, arg1, arg2),
        syscall_numbers::THREAD_YIELD => handle_syscall_thread_yield(),
        syscall_numbers::THREAD_GETTID => handle_syscall_gettid(),
        
        // Memory management
        syscall_numbers::VIRTUAL_ALLOC => handle_syscall_virtual_alloc(arg0, arg1, arg2),
        syscall_numbers::VIRTUAL_FREE => handle_syscall_virtual_free(arg0, arg1),
        
        // Device I/O
        syscall_numbers::DEVICE_OPEN => handle_syscall_device_open(arg0, arg1),
        syscall_numbers::DEVICE_CLOSE => handle_syscall_device_close(arg0),
        syscall_numbers::DEVICE_READ => handle_syscall_device_read(arg0, arg1, arg2),
        syscall_numbers::DEVICE_WRITE => handle_syscall_device_write(arg0, arg1, arg2),
        
        _ => {
            warn!("Unimplemented system call: {}", syscall_number);
            SystemCallResult {
                return_value: 0,
                error_code: InterruptError::SystemCallNotImplemented,
            }
        }
    };
    
    result
}

/// System call implementations
fn handle_syscall_system_info() -> SystemCallResult {
    if let Ok(info) = crate::get_system_info() {
        // Return pointer to system info structure
        SystemCallResult {
            return_value: (&info as *const crate::SystemInfo) as usize,
            error_code: InterruptError::SystemCallInvalid,
        }
    } else {
        SystemCallResult {
            return_value: 0,
            error_code: InterruptError::SystemCallInvalid,
        }
    }
}

fn handle_syscall_getpid() -> SystemCallResult {
    SystemCallResult {
        return_value: 1, // Placeholder PID
        error_code: InterruptError::SystemCallInvalid,
    }
}

fn handle_syscall_time_get() -> SystemCallResult {
    use crate::bootstrap::get_boot_time;
    SystemCallResult {
        return_value: get_boot_time(),
        error_code: InterruptError::SystemCallInvalid,
    }
}

fn handle_syscall_memory_info() -> SystemCallResult {
    let stats = crate::memory::get_memory_stats();
    SystemCallResult {
        return_value: (&stats as *const crate::memory::MemoryStats) as usize,
        error_code: InterruptError::SystemCallInvalid,
    }
}

fn handle_syscall_process_create(entry_point: usize, stack_size: usize) -> SystemCallResult {
    warn!("Process creation not yet implemented");
    SystemCallResult {
        return_value: 0,
        error_code: InterruptError::SystemCallNotImplemented,
    }
}

fn handle_syscall_process_exit(exit_code: usize) -> SystemCallResult {
    warn!("Process exit not yet implemented");
    SystemCallResult {
        return_value: 0,
        error_code: InterruptError::SystemCallNotImplemented,
    }
}

fn handle_syscall_thread_create(entry_point: usize, arg: usize, stack_size: usize) -> SystemCallResult {
    warn!("Thread creation not yet implemented");
    SystemCallResult {
        return_value: 0,
        error_code: InterruptError::SystemCallNotImplemented,
    }
}

fn handle_syscall_thread_yield() -> SystemCallResult {
    crate::scheduler::yield_current_thread();
    SystemCallResult {
        return_value: 0,
        error_code: InterruptError::SystemCallInvalid,
    }
}

fn handle_syscall_gettid() -> SystemCallResult {
    SystemCallResult {
        return_value: 1, // Placeholder TID
        error_code: InterruptError::SystemCallInvalid,
    }
}

fn handle_syscall_virtual_alloc(address: usize, size: usize, flags: usize) -> SystemCallResult {
    warn!("Virtual memory allocation not yet implemented");
    SystemCallResult {
        return_value: 0,
        error_code: InterruptError::SystemCallNotImplemented,
    }
}

fn handle_syscall_virtual_free(address: usize, size: usize) -> SystemCallResult {
    warn!("Virtual memory freeing not yet implemented");
    SystemCallResult {
        return_value: 0,
        error_code: InterruptError::SystemCallNotImplemented,
    }
}

fn handle_syscall_device_open(device_path: usize, flags: usize) -> SystemCallResult {
    warn!("Device opening not yet implemented");
    SystemCallResult {
        return_value: 0,
        error_code: InterruptError::SystemCallNotImplemented,
    }
}

fn handle_syscall_device_close(fd: usize) -> SystemCallResult {
    warn!("Device closing not yet implemented");
    SystemCallResult {
        return_value: 0,
        error_code: InterruptError::SystemCallNotImplemented,
    }
}

fn handle_syscall_device_read(fd: usize, buffer: usize, size: usize) -> SystemCallResult {
    warn!("Device reading not yet implemented");
    SystemCallResult {
        return_value: 0,
        error_code: InterruptError::SystemCallNotImplemented,
    }
}

fn handle_syscall_device_write(fd: usize, buffer: usize, size: usize) -> SystemCallResult {
    warn!("Device writing not yet implemented");
    SystemCallResult {
        return_value: 0,
        error_code: InterruptError::SystemCallNotImplemented,
    }
}