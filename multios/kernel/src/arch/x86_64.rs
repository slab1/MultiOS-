//! x86_64 architecture-specific implementation
//! 
//! This module implements x86_64-specific functionality including
//! IDT setup, interrupt handling, and system calls.

use x86_64::{
    structures::{
        idt::{InterruptDescriptorTable, InterruptStackFrame, InterruptDescriptorTableEntry},
        tss::TaskStateSegment,
    },
    VirtAddr,
};
use alloc::vec::Vec;
use spin::Mutex;
use log::{info, warn, error};
use crate::{BootInfo, KernelResult, interrupts};
use crate::syscall;

static IDT: Mutex<InterruptDescriptorTable> = Mutex::new(InterruptDescriptorTable::new());

/// Task State Segment for interrupt handling
static TSS: Mutex<TaskStateSegment> = Mutex::new(TaskStateSegment::new());

/// Initialize x86_64-specific components
pub fn init(boot_info: &BootInfo) -> KernelResult<()> {
    info!("Initializing x86_64 architecture...");
    
    // Set up the Interrupt Descriptor Table
    setup_idt()?;
    
    // Load the IDT
    load_idt();
    
    // Set up TSS for privilege level transitions
    setup_tss();
    
    // Enable interrupts
    enable_interrupts();
    
    info!("x86_64 architecture initialized successfully");
    
    Ok(())
}

/// Set up the Interrupt Descriptor Table
fn setup_idt() -> KernelResult<()> {
    info!("Setting up Interrupt Descriptor Table...");
    
    let mut idt = IDT.lock();
    
    // Set up exception handlers (traps)
    idt.alignment_check.set_handler_fn(exception_handler_alignment_check);
    idt.bound_range_exceeded.set_handler_fn(exception_handler_bound_range_exceeded);
    idt.breakpoint.set_handler_fn(exception_handler_breakpoint);
    idt.overflow.set_handler_fn(exception_handler_overflow);
    idt.devide_error.set_handler_fn(exception_handler_devide_error);
    idt.legacy_match.set_handler_fn(exception_handler_legacy_match);
    idt.invalid_opcode.set_handler_fn(exception_handler_invalid_opcode);
    idt.segment_not_present.set_handler_fn(exception_handler_segment_not_present);
    idt.stack_segment_fault.set_handler_fn(exception_handler_stack_segment_fault);
    idt.general_protection_fault.set_handler_fn(exception_handler_general_protection_fault);
    idt.page_fault.set_handler_fn(exception_handler_page_fault);
    idt.x87_floating_point.set_handler_fn(exception_handler_x87_floating_point);
    idt.simd_floating_point.set_handler_fn(exception_handler_simd_floating_point);
    idt.virtualization.set_handler_fn(exception_handler_virtualization);
    idt.security_exception.set_handler_fn(exception_handler_security);
    idt.backup_debug.set_handler_fn(exception_handler_debug);
    idt.debug.set_handler_fn(exception_handler_debug);
    
    // Set up hardware interrupt handlers
    idt[interrupts::TIMER_INTERRUPT as usize].set_handler_fn(timer_interrupt_handler);
    idt[interrupts::KEYBOARD_INTERRUPT as usize].set_handler_fn(keyboard_interrupt_handler);
    idt[interrupts::CASCADE_INTERRUPT as usize].set_handler_fn(cascade_interrupt_handler);
    idt[interrupts::COM2_INTERRUPT as usize].set_handler_fn(com2_interrupt_handler);
    idt[interrupts::COM1_INTERRUPT as usize].set_handler_fn(com1_interrupt_handler);
    idt[interrupts::FLOPPY_INTERRUPT as usize].set_handler_fn(floppy_interrupt_handler);
    idt[interrupts::CMOS_INTERRUPT as usize].set_handler_fn(cmos_interrupt_handler);
    
    // Set up system call interrupt handler (if using int 0x80)
    idt[interrupts::SYSCALL_INTERRUPT as usize].set_handler_fn(syscall_interrupt_handler);
    
    info!("Interrupt Descriptor Table setup complete");
    
    Ok(())
}

/// Load the IDT
fn load_idt() {
    let idt = &IDT;
    unsafe {
        idt.load();
    }
}

/// Set up Task State Segment
fn setup_tss() {
    info!("Setting up Task State Segment...");
    
    let mut tss = TSS.lock();
    
    // Set up interrupt stack table (IST)
    // For now, use a dummy stack - in real implementation we'd allocate proper stacks
    tss.interrupt_stack_table[0] = VirtAddr::new_truncate(0xFFFF_FFFF_FFFF_FFFF); // Dummy stack
    
    unsafe {
        tss.load();
    }
    
    info!("Task State Segment initialized");
}

/// Enable interrupts
fn enable_interrupts() {
    unsafe {
        core::arch::asm!("sti");
    }
    info!("Interrupts enabled");
}

/// Disable interrupts
fn disable_interrupts() {
    unsafe {
        core::arch::asm!("cli");
    }
}

/// Pause current instruction and wait for interrupt
fn hlt() {
    unsafe {
        core::arch::asm!("hlt");
    }
}

// Exception Handlers

fn exception_handler_alignment_check(stack_frame: &mut InterruptStackFrame, error_code: u64) {
    error!("Alignment check exception!");
    error!("Error code: {:#x}", error_code);
    error!("Stack frame: {:#?}", stack_frame);
    panic!("Alignment check exception");
}

fn exception_handler_bound_range_exceeded(stack_frame: &mut InterruptStackFrame) {
    error!("Bound range exceeded exception!");
    error!("Stack frame: {:#?}", stack_frame);
    panic!("Bound range exceeded exception");
}

fn exception_handler_breakpoint(stack_frame: &mut InterruptStackFrame) {
    warn!("Breakpoint exception (debug breakpoint)");
    warn!("Stack frame: {:#?}", stack_frame);
}

fn exception_handler_overflow(stack_frame: &mut InterruptStackFrame) {
    error!("Overflow exception!");
    error!("Stack frame: {:#?}", stack_frame);
    panic!("Overflow exception");
}

fn exception_handler_devide_error(stack_frame: &mut InterruptStackFrame) {
    error!("Divide error exception!");
    error!("Stack frame: {:#?}", stack_frame);
    panic!("Divide error exception");
}

fn exception_handler_legacy_match(stack_frame: &mut InterruptStackFrame) {
    error!("Legacy match exception!");
    error!("Stack frame: {:#?}", stack_frame);
    panic!("Legacy match exception");
}

fn exception_handler_invalid_opcode(stack_frame: &mut InterruptStackFrame) {
    error!("Invalid opcode exception!");
    error!("Stack frame: {:#?}", stack_frame);
    panic!("Invalid opcode exception");
}

fn exception_handler_segment_not_present(stack_frame: &mut InterruptStackFrame, error_code: u64) {
    error!("Segment not present exception!");
    error!("Error code: {:#x}", error_code);
    error!("Stack frame: {:#?}", stack_frame);
    panic!("Segment not present exception");
}

fn exception_handler_stack_segment_fault(stack_frame: &mut InterruptStackFrame, error_code: u64) {
    error!("Stack segment fault!");
    error!("Error code: {:#x}", error_code);
    error!("Stack frame: {:#?}", stack_frame);
    panic!("Stack segment fault");
}

fn exception_handler_general_protection_fault(stack_frame: &mut InterruptStackFrame, error_code: u64) {
    error!("General protection fault!");
    error!("Error code: {:#x}", error_code);
    error!("Stack frame: {:#?}", stack_frame);
    panic!("General protection fault");
}

fn exception_handler_page_fault(stack_frame: &mut InterruptStackFrame, error_code: u64) {
    error!("Page fault!");
    error!("Error code: {:#x}", error_code);
    error!("Stack frame: {:#?}", stack_frame);
    
    // Get faulting address from CR2 register
    let faulting_address = get_cr2();
    error!("Faulting address: {:#x}", faulting_address);
    
    panic!("Page fault");
}

fn exception_handler_x87_floating_point(stack_frame: &mut InterruptStackFrame) {
    error!("x87 floating point exception!");
    error!("Stack frame: {:#?}", stack_frame);
    panic!("x87 floating point exception");
}

fn exception_handler_simd_floating_point(stack_frame: &mut InterruptStackFrame) {
    error!("SIMD floating point exception!");
    error!("Stack frame: {:#?}", stack_frame);
    panic!("SIMD floating point exception");
}

fn exception_handler_virtualization(stack_frame: &mut InterruptStackFrame) {
    error!("Virtualization exception!");
    error!("Stack frame: {:#?}", stack_frame);
    panic!("Virtualization exception");
}

fn exception_handler_security(stack_frame: &mut InterruptStackFrame, error_code: u64) {
    error!("Security exception!");
    error!("Error code: {:#x}", error_code);
    error!("Stack frame: {:#?}", stack_frame);
    panic!("Security exception");
}

fn exception_handler_debug(stack_frame: &mut InterruptStackFrame) {
    warn!("Debug exception!");
    warn!("Stack frame: {:#?}", stack_frame);
}

// Hardware Interrupt Handlers

fn timer_interrupt_handler(stack_frame: &mut InterruptStackFrame) {
    debug!("Timer interrupt received");
    
    // Acknowledge the interrupt
    acknowledge_interrupt(interrupts::TIMER_INTERRUPT);
    
    // Update system time or scheduling
    update_system_timer();
    
    // Trigger scheduler if needed
    if should_schedule() {
        trigger_scheduler();
    }
}

fn keyboard_interrupt_handler(stack_frame: &mut InterruptStackFrame) {
    debug!("Keyboard interrupt received");
    
    // Acknowledge the interrupt
    acknowledge_interrupt(interrupts::KEYBOARD_INTERRUPT);
    
    // Read keyboard scancode
    let scancode = read_keyboard_data();
    
    // Process keyboard input
    process_keyboard_input(scancode);
}

fn cascade_interrupt_handler(stack_frame: &mut InterruptStackFrame) {
    debug!("Cascade interrupt received");
    acknowledge_interrupt(interrupts::CASCADE_INTERRUPT);
}

fn com2_interrupt_handler(stack_frame: &mut InterruptStackFrame) {
    debug!("COM2 interrupt received");
    acknowledge_interrupt(interrupts::COM2_INTERRUPT);
    process_serial_data(2);
}

fn com1_interrupt_handler(stack_frame: &mut InterruptStackFrame) {
    debug!("COM1 interrupt received");
    acknowledge_interrupt(interrupts::COM1_INTERRUPT);
    process_serial_data(1);
}

fn floppy_interrupt_handler(stack_frame: &mut InterruptStackFrame) {
    debug!("Floppy disk interrupt received");
    acknowledge_interrupt(interrupts::FLOPPY_INTERRUPT);
    process_floppy_operation();
}

fn cmos_interrupt_handler(stack_frame: &mut InterruptStackFrame) {
    debug!("CMOS/RTC interrupt received");
    acknowledge_interrupt(interrupts::CMOS_INTERRUPT);
    update_rtc_time();
}

// System Call Interrupt Handler

fn syscall_interrupt_handler(
    stack_frame: &mut InterruptStackFrame,
    syscall_number: u64,
    arg1: u64,
    arg2: u64,
    arg3: u64,
    arg4: u64,
    arg5: u64,
    arg6: u64,
) -> u64 {
    debug!("System call: {} (args: {:#x}, {:#x}, {:#x}, {:#x}, {:#x}, {:#x})",
           syscall_number, arg1, arg2, arg3, arg4, arg5, arg6);
    
    // Create system call context
    let context = syscall::SyscallContext {
        syscall_number,
        parameters: [arg1, arg2, arg3, arg4, arg5, arg6],
        return_value: 0,
    };
    
    // Dispatch the system call
    match handle_syscall(context) {
        Ok(result) => {
            debug!("System call {} completed successfully: {:#x}", syscall_number, result);
            result as u64
        }
        Err(error_code) => {
            error!("System call {} failed with error: {:#x}", syscall_number, error_code);
            (-(error_code as i64)) as u64 // Return negative error code
        }
    }
}

// System Call Handling

fn handle_syscall(mut context: syscall::SyscallContext) -> Result<i64, i32> {
    let syscall_num = context.syscall_number as u64;
    
    match syscall_num {
        syscall::SYSCALL_EXIT => {
            debug!("SYSCALL_EXIT: process exit");
            handle_syscall_exit(context.parameters[0] as i32)
        }
        
        syscall::SYSCALL_WRITE => {
            debug!("SYSCALL_WRITE: write to file descriptor");
            handle_syscall_write(
                context.parameters[0] as i32,
                context.parameters[1] as *const u8,
                context.parameters[2] as usize,
            )
        }
        
        syscall::SYSCALL_READ => {
            debug!("SYSCALL_READ: read from file descriptor");
            handle_syscall_read(
                context.parameters[0] as i32,
                context.parameters[1] as *mut u8,
                context.parameters[2] as usize,
            )
        }
        
        syscall::SYSCALL_OPEN => {
            debug!("SYSCALL_OPEN: open file");
            handle_syscall_open(
                context.parameters[0] as *const u8,
                context.parameters[1] as i32,
            )
        }
        
        syscall::SYSCALL_CLOSE => {
            debug!("SYSCALL_CLOSE: close file descriptor");
            handle_syscall_close(context.parameters[0] as i32)
        }
        
        syscall::SYSCALL_GETPID => {
            debug!("SYSCALL_GETPID: get process ID");
            handle_syscall_getpid()
        }
        
        syscall::SYSCALL_MMAP => {
            debug!("SYSCALL_MMAP: memory map");
            handle_syscall_mmap(
                context.parameters[0] as usize,
                context.parameters[1] as usize,
                context.parameters[2] as i32,
                context.parameters[3] as i32,
                context.parameters[4] as i32,
                context.parameters[5] as off_t,
            )
        }
        
        syscall::SYSCALL_MUNMAP => {
            debug!("SYSCALL_MUNMAP: unmap memory");
            handle_syscall_munmap(
                context.parameters[0] as *const u8,
                context.parameters[1] as usize,
            )
        }
        
        syscall::SYSCALL_FORK => {
            debug!("SYSCALL_FORK: fork process");
            handle_syscall_fork()
        }
        
        syscall::SYSCALL_EXEC => {
            debug!("SYSCALL_EXEC: execute program");
            handle_syscall_exec(
                context.parameters[0] as *const u8,
                context.parameters[1] as *const *const u8,
            )
        }
        
        0 => {
            // Test system call
            debug!("Test system call received");
            Ok(42) // Return 42 for testing
        }
        
        _ => {
            warn!("Unknown system call: {}", syscall_num);
            Err(libc::ENOSYS)
        }
    }
}

// Individual System Call Implementations

fn handle_syscall_exit(status: i32) -> Result<i64, i32> {
    debug!("Process exiting with status: {}", status);
    // In a real implementation, this would clean up the process
    Ok(0)
}

fn handle_syscall_write(fd: i32, buf: *const u8, count: usize) -> Result<i64, i32> {
    if buf.is_null() {
        return Err(libc::EFAULT);
    }
    
    // Simple implementation - write to serial console for testing
    if fd == 1 || fd == 2 { // stdout or stderr
        unsafe {
            let slice = core::slice::from_raw_parts(buf, count);
            for &byte in slice {
                if byte == b'\n' {
                    crate::serial::write_byte(b'\r');
                }
                crate::serial::write_byte(byte);
            }
        }
        Ok(count as i64)
    } else {
        Err(libc::EBADF)
    }
}

fn handle_syscall_read(fd: i32, buf: *mut u8, count: usize) -> Result<i64, i32> {
    if buf.is_null() {
        return Err(libc::EFAULT);
    }
    
    // For now, return 0 (no input available)
    Ok(0)
}

fn handle_syscall_open(pathname: *const u8, flags: i32) -> Result<i64, i32> {
    if pathname.is_null() {
        return Err(libc::EFAULT);
    }
    
    // Simple implementation - just return a dummy file descriptor
    Ok(3)
}

fn handle_syscall_close(fd: i32) -> Result<i64, i32> {
    if fd < 0 {
        return Err(libc::EBADF);
    }
    
    Ok(0)
}

fn handle_syscall_getpid() -> Result<i64, i32> {
    Ok(1) // Dummy process ID
}

fn handle_syscall_mmap(
    addr: usize,
    len: usize,
    prot: i32,
    flags: i32,
    fd: i32,
    offset: off_t,
) -> Result<i64, i32> {
    if len == 0 {
        return Ok(addr as i64);
    }
    
    // Simple implementation - just return the requested address
    // In a real implementation, this would allocate and map memory
    Ok(addr as i64)
}

fn handle_syscall_munmap(addr: *const u8, len: usize) -> Result<i64, i32> {
    if addr.is_null() && len == 0 {
        return Ok(0);
    }
    
    // Simple implementation
    Ok(0)
}

fn handle_syscall_fork() -> Result<i64, i32> {
    // Simple implementation - return child process ID
    Ok(2)
}

fn handle_syscall_exec(pathname: *const u8, argv: *const *const u8) -> Result<i64, i32> {
    if pathname.is_null() {
        return Err(libc::EFAULT);
    }
    
    // Simple implementation
    Ok(0)
}

// Helper Functions

fn get_cr2() -> u64 {
    unsafe {
        let value: u64;
        core::arch::asm!("mov {}, cr2", out(reg) value);
        value
    }
}

fn acknowledge_interrupt(irq: u8) {
    // Send End of Interrupt (EOI) to PIC
    unsafe {
        if irq < 8 {
            // Master PIC
            x86_64::instructions::port::Port::new(0x20).write(0x20u8);
        } else {
            // Slave PIC or cascade
            x86_64::instructions::port::Port::new(0x20).write(0x20u8);
            x86_64::instructions::port::Port::new(0xA0).write(0x20u8);
        }
    }
}

fn read_keyboard_data() -> u8 {
    unsafe {
        x86_64::instructions::port::Port::new(0x60).read()
    }
}

fn should_schedule() -> bool {
    // Simple timer-based scheduling
    true // For now, always schedule
}

fn trigger_scheduler() {
    // Trigger the scheduler
    crate::scheduler::schedule_next();
}

fn update_system_timer() {
    // Update system timer
}

fn process_keyboard_input(_scancode: u8) {
    // Process keyboard input
}

fn process_serial_data(_port: u32) {
    // Process serial port data
}

fn process_floppy_operation() {
    // Process floppy disk operation
}

fn update_rtc_time() {
    // Update real-time clock
}

// Mock libc constants
mod libc {
    pub const ENOSYS: i32 = 38;
    pub const EFAULT: i32 = 14;
    pub const EBADF: i32 = 9;
}

// Mock types
type off_t = u64;
