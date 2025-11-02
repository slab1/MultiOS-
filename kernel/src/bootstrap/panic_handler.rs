//! Bootstrap Panic Handler
//! 
//! This module provides panic handling during bootstrap with
//! detailed error reporting and system state preservation.

use crate::bootstrap::{BootstrapContext, BootstrapStage};
use crate::KernelError;
use crate::log::error;
use core::fmt::Write;

/// Panic information structure
#[derive(Debug, Clone)]
pub struct BootstrapPanicInfo {
    pub panic_message: String,
    pub panic_location: PanicLocation,
    pub bootstrap_stage: BootstrapStage,
    pub architecture: crate::ArchType,
    pub boot_time: u64,
    pub stack_trace: Vec<u64>,
    pub register_state: RegisterState,
    pub memory_info: MemoryPanicInfo,
}

/// Panic location information
#[derive(Debug, Clone)]
pub struct PanicLocation {
    pub file: &'static str,
    pub line: u32,
    pub column: u32,
    pub function: &'static str,
}

/// Register state at panic time
#[derive(Debug, Clone)]
pub struct RegisterState {
    // x86_64 registers
    pub rax: Option<u64>,
    pub rbx: Option<u64>,
    pub rcx: Option<u64>,
    pub rdx: Option<u64>,
    pub rsi: Option<u64>,
    pub rdi: Option<u64>,
    pub rbp: Option<u64>,
    pub rsp: Option<u64>,
    pub r8: Option<u64>,
    pub r9: Option<u64>,
    pub r10: Option<u64>,
    pub r11: Option<u64>,
    pub r12: Option<u64>,
    pub r13: Option<u64>,
    pub r14: Option<u64>,
    pub r15: Option<u64>,
    pub rip: Option<u64>,
    pub rflags: Option<u64>,
    
    // ARM64 registers
    pub x0: Option<u64>,
    pub x1: Option<u64>,
    pub x2: Option<u64>,
    pub x3: Option<u64>,
    pub x4: Option<u64>,
    pub x5: Option<u64>,
    pub x6: Option<u64>,
    pub x7: Option<u64>,
    pub x8: Option<u64>,
    pub x9: Option<u64>,
    pub x10: Option<u64>,
    pub x11: Option<u64>,
    pub x12: Option<u64>,
    pub x13: Option<u64>,
    pub x14: Option<u64>,
    pub x15: Option<u64>,
    pub x16: Option<u64>,
    pub x17: Option<u64>,
    pub x18: Option<u64>,
    pub x19: Option<u64>,
    pub x20: Option<u64>,
    pub x21: Option<u64>,
    pub x22: Option<u64>,
    pub x23: Option<u64>,
    pub x24: Option<u64>,
    pub x25: Option<u64>,
    pub x26: Option<u64>,
    pub x27: Option<u64>,
    pub x28: Option<u64>,
    pub fp: Option<u64>,
    pub lr: Option<u64>,
    pub sp: Option<u64>,
    pub pc: Option<u64>,
    
    // RISC-V registers
    pub zero: Option<u64>,
    pub ra: Option<u64>,
    pub sp: Option<u64>,
    pub gp: Option<u64>,
    pub tp: Option<u64>,
    pub t0: Option<u64>,
    pub t1: Option<u64>,
    pub t2: Option<u64>,
    pub s0: Option<u64>,
    pub s1: Option<u64>,
    pub a0: Option<u64>,
    pub a1: Option<u64>,
    pub a2: Option<u64>,
    pub a3: Option<u64>,
    pub a4: Option<u64>,
    pub a5: Option<u64>,
    pub a6: Option<u64>,
    pub a7: Option<u64>,
    pub s2: Option<u64>,
    pub s3: Option<u64>,
    pub s4: Option<u64>,
    pub s5: Option<u64>,
    pub s6: Option<u64>,
    pub s7: Option<u64>,
    pub s8: Option<u64>,
    pub s9: Option<u64>,
    pub s10: Option<u64>,
    pub s11: Option<u64>,
    pub t3: Option<u64>,
    pub t4: Option<u64>,
    pub t5: Option<u64>,
    pub t6: Option<u64>,
    pub pc_riscv: Option<u64>,
}

/// Memory information at panic time
#[derive(Debug, Clone)]
pub struct MemoryPanicInfo {
    pub total_memory: u64,
    pub available_memory: u64,
    pub used_memory: u64,
    pub panic_location_addr: Option<u64>,
    pub stack_pointer: Option<u64>,
}

/// Global panic information
static PANIC_INFO: core::sync::atomic::AtomicPtr<BootstrapPanicInfo> = 
    core::sync::atomic::AtomicPtr::new(core::ptr::null_mut());

/// Bootstrap panic handler
pub fn bootstrap_panic_handler(
    info: &core::panic::PanicInfo,
    context: Option<&BootstrapContext>,
) -> ! {
    disable_interrupts();
    
    let panic_info = gather_panic_info(info, context);
    save_panic_info(&panic_info);
    
    print_panic_report(&panic_info);
    
    save_crash_dump(&panic_info);
    
    halt_system()
}

/// Disable interrupts
fn disable_interrupts() {
    unsafe {
        match get_current_architecture() {
            crate::ArchType::X86_64 => {
                core::arch::asm!("cli");
            },
            crate::ArchType::AArch64 => {
                core::arch::asm!("msr daifset, #2");
            },
            crate::ArchType::Riscv64 => {
                let mut mstatus: usize;
                core::arch::asm!("csrr {}, mstatus", out(reg) mstatus);
                mstatus &= !(1 << 3); // Clear MIE
                core::arch::asm!("csrw mstatus, {}", in(reg) mstatus);
            },
            _ => {},
        }
    }
}

/// Gather panic information
fn gather_panic_info(
    info: &core::panic::PanicInfo,
    context: Option<&BootstrapContext>,
) -> BootstrapPanicInfo {
    let location = extract_panic_location(info);
    let message = extract_panic_message(info);
    let registers = capture_registers();
    let memory_info = capture_memory_info(context);
    let stack_trace = capture_stack_trace();
    
    let bootstrap_stage = context.map(|c| c.current_stage).unwrap_or(BootstrapStage::EarlyInit);
    let architecture = context.map(|c| c.config.architecture).unwrap_or(crate::ArchType::Unknown);
    let boot_time = context.map(|c| c.boot_info.boot_time).unwrap_or(0);
    
    BootstrapPanicInfo {
        panic_message: message,
        panic_location: location,
        bootstrap_stage,
        architecture,
        boot_time,
        stack_trace,
        register_state: registers,
        memory_info,
    }
}

/// Extract panic location
fn extract_panic_location(info: &core::panic::PanicInfo) -> PanicLocation {
    let location = info.location();
    
    PanicLocation {
        file: location.map(|l| l.file()).unwrap_or("unknown"),
        line: location.map(|l| l.line()).unwrap_or(0),
        column: location.map(|l| l.column()).unwrap_or(0),
        function: "unknown_function",
    }
}

/// Extract panic message
fn extract_panic_message(info: &core::panic::PanicInfo) -> String {
    let mut message = String::new();
    
    if let Some(msg) = info.message() {
        write!(message, "{}", msg).unwrap();
    } else {
        message = "Panic occurred".to_string();
    }
    
    message
}

/// Capture register state
fn capture_registers() -> RegisterState {
    unsafe {
        let (rax, rbx, rcx, rdx, rsi, rdi, rbp, rsp, rip, rflags);
        
        // Read general purpose registers for x86_64
        core::arch::asm!(
            "mov {}, rax", out(reg) rax,
        );
        core::arch::asm!(
            "mov {}, rbx", out(reg) rbx,
        );
        core::arch::asm!(
            "mov {}, rcx", out(reg) rcx,
        );
        core::arch::asm!(
            "mov {}, rdx", out(reg) rdx,
        );
        core::arch::asm!(
            "mov {}, rsi", out(reg) rsi,
        );
        core::arch::asm!(
            "mov {}, rdi", out(reg) rdi,
        );
        core::arch::asm!(
            "mov {}, rbp", out(reg) rbp,
        );
        core::arch::asm!(
            "mov {}, rsp", out(reg) rsp,
        );
        core::arch::asm!(
            "mov {}, r8", out(reg) _,
        );
        core::arch::asm!(
            "pushfq; pop {}", out(reg) rflags,
        );
        
        // Get instruction pointer
        let rip_addr: u64;
        core::arch::asm!(
            "lea {}, [rip]", out(reg) rip_addr,
        );
        
        RegisterState {
            rax: Some(rax),
            rbx: Some(rbx),
            rcx: Some(rcx),
            rdx: Some(rdx),
            rsi: Some(rsi),
            rdi: Some(rdi),
            rbp: Some(rbp),
            rsp: Some(rsp),
            r8: None, r9: None, r10: None, r11: None,
            r12: None, r13: None, r14: None, r15: None,
            rip: Some(rip_addr),
            rflags: Some(rflags),
            
            // ARM64 and RISC-V registers (not available on x86_64)
            x0: None, x1: None, x2: None, x3: None, x4: None,
            x5: None, x6: None, x7: None, x8: None, x9: None,
            x10: None, x11: None, x12: None, x13: None, x14: None,
            x15: None, x16: None, x17: None, x18: None, x19: None,
            x20: None, x21: None, x22: None, x23: None, x24: None,
            x25: None, x26: None, x27: None, x28: None,
            fp: None, lr: None, sp: None, pc: None,
            
            zero: None, ra: None, sp_riscv: None, gp: None, tp: None,
            t0: None, t1: None, t2: None, s0: None, s1: None,
            a0: None, a1: None, a2: None, a3: None, a4: None,
            a5: None, a6: None, a7: None, s2: None, s3: None,
            s4: None, s5: None, s6: None, s7: None, s8: None,
            s9: None, s10: None, s11: None, t3: None, t4: None,
            t5: None, t6: None, pc_riscv: None,
        }
    }
}

/// Capture memory information
fn capture_memory_info(context: Option<&BootstrapContext>) -> MemoryPanicInfo {
    let mut total_memory = 0;
    let mut available_memory = 0;
    let mut used_memory = 0;
    
    if let Some(ctx) = context {
        for entry in &ctx.boot_info.memory_map {
            if entry.entry_type == crate::MemoryType::Usable {
                available_memory += entry.size;
            }
            total_memory += entry.size;
        }
        used_memory = total_memory - available_memory;
    }
    
    MemoryPanicInfo {
        total_memory,
        available_memory,
        used_memory,
        panic_location_addr: None,
        stack_pointer: None,
    }
}

/// Capture stack trace
fn capture_stack_trace() -> Vec<u64> {
    let mut stack_trace = Vec::new();
    
    // Simple stack trace capturing
    // In a real implementation, this would walk the stack frames
    // using frame pointers or debug information
    
    unsafe {
        let mut frame_ptr: u64;
        let mut return_addr: u64;
        
        // Get frame pointer
        core::arch::asm!("mov {}, rbp", out(reg) frame_ptr);
        
        // Walk stack frames
        for _ in 0..10 { // Limit depth
            if frame_ptr == 0 {
                break;
            }
            
            // Read return address from stack
            return_addr = *(frame_ptr as *const u64).add(1);
            stack_trace.push(return_addr);
            
            // Move to next frame
            frame_ptr = *(frame_ptr as *const u64);
            
            if frame_ptr == 0 {
                break;
            }
        }
    }
    
    stack_trace
}

/// Get current architecture
fn get_current_architecture() -> crate::ArchType {
    crate::ArchType::X86_64 // Assume x86_64 for panic handler
}

/// Save panic information
fn save_panic_info(panic_info: &BootstrapPanicInfo) {
    let panic_ptr = Box::into_raw(Box::new(panic_info.clone()));
    PANIC_INFO.store(panic_ptr, core::sync::atomic::Ordering::SeqCst);
}

/// Print panic report
fn print_panic_report(panic_info: &BootstrapPanicInfo) {
    error!("");
    error!("==============================================");
    error!("           BOOTSTRAP PANIC REPORT");
    error!("==============================================");
    error!("");
    
    error!("PANIC MESSAGE: {}", panic_info.panic_message);
    error!("");
    
    error!("LOCATION:");
    error!("  File: {}", panic_info.panic_location.file);
    error!("  Line: {}", panic_info.panic_location.line);
    error!("  Column: {}", panic_info.panic_location.column);
    error!("  Function: {}", panic_info.panic_location.function);
    error!("");
    
    error!("BOOTSTRAP CONTEXT:");
    error!("  Stage: {:?}", panic_info.bootstrap_stage);
    error!("  Architecture: {:?}", panic_info.architecture);
    error!("  Boot Time: {}", panic_info.boot_time);
    error!("");
    
    error!("REGISTER STATE:");
    print_register_state(&panic_info.register_state);
    error!("");
    
    error!("MEMORY INFO:");
    error!("  Total Memory: {} MB", panic_info.memory_info.total_memory / 1024 / 1024);
    error!("  Available: {} MB", panic_info.memory_info.available_memory / 1024 / 1024);
    error!("  Used: {} MB", panic_info.memory_info.used_memory / 1024 / 1024);
    error!("");
    
    error!("STACK TRACE:");
    for (i, addr) in panic_info.stack_trace.iter().enumerate() {
        error!("  Frame {}: 0x{:016x}", i, addr);
    }
    error!("");
    
    error!("==============================================");
    error!("");
}

/// Print register state
fn print_register_state(registers: &RegisterState) {
    if let Some(rip) = registers.rip {
        error!("  RIP: 0x{:016x}", rip);
    }
    if let Some(rsp) = registers.rsp {
        error!("  RSP: 0x{:016x}", rsp);
    }
    if let Some(rbp) = registers.rbp {
        error!("  RBP: 0x{:016x}", rbp);
    }
    if let Some(rax) = registers.rax {
        error!("  RAX: 0x{:016x}", rax);
    }
    if let Some(rbx) = registers.rbx {
        error!("  RBX: 0x{:016x}", rbx);
    }
    if let Some(rcx) = registers.rcx {
        error!("  RCX: 0x{:016x}", rcx);
    }
    if let Some(rdx) = registers.rdx {
        error!("  RDX: 0x{:016x}", rdx);
    }
    if let Some(rflags) = registers.rflags {
        error!("  RFLAGS: 0x{:016x}", rflags);
    }
}

/// Save crash dump
fn save_crash_dump(panic_info: &BootstrapPanicInfo) {
    error!("Saving crash dump to memory...");
    
    // Simple crash dump - save to a known memory location
    // In a real implementation, this would save to non-volatile storage
    
    let dump_location = 0x10000; // Known memory location for crash dump
    
    unsafe {
        let dump_ptr = dump_location as *mut BootstrapPanicInfo;
        dump_ptr.write_volatile(*panic_info);
    }
    
    error!("Crash dump saved to memory location 0x{:x}", dump_location);
}

/// Halt system
fn halt_system() -> ! {
    error!("System halted. Waiting for power cycle or reset...");
    
    loop {
        unsafe {
            // Halt CPU and wait for interrupts
            core::arch::asm!("hlt");
        }
    }
}

/// Get saved panic information
pub fn get_saved_panic_info() -> Option<Box<BootstrapPanicInfo>> {
    let panic_ptr = PANIC_INFO.load(core::sync::atomic::Ordering::SeqCst);
    
    if panic_ptr.is_null() {
        None
    } else {
        Some(Box::from_raw(panic_ptr))
    }
}

/// Check if a panic has occurred
pub fn has_panic_occurred() -> bool {
    !PANIC_INFO.load(core::sync::atomic::Ordering::SeqCst).is_null()
}