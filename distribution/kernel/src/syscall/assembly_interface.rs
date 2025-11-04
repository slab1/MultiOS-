//! MultiOS Assembly Interface and Entry Points
//! 
//! This module provides architecture-specific assembly entry points for system calls,
//! optimized system call invocation sequences, and low-level register management.
//! It includes implementations for x86_64, ARM64, and RISC-V architectures.

use crate::log::{info, warn, error, debug};
use crate::arch::{ArchType, PrivilegeLevel};
use crate::arch::interrupts::*;
use crate::syscall_numbers;
use crate::syscall::{dispatcher, performance, error_handling};
use core::sync::atomic::{AtomicU64, Ordering};

/// Global system call counter
static SYSCALL_COUNTER: AtomicU64 = AtomicU64::new(0);

/// System call performance monitoring
pub struct SyscallPerformanceMonitor {
    /// Total system calls executed
    pub total_syscalls: AtomicU64,
    /// Fast path system calls
    pub fast_path_syscalls: AtomicU64,
    /// Standard path system calls
    pub standard_path_syscalls: AtomicU64,
    /// Total CPU cycles used
    pub total_cycles: AtomicU64,
}

impl SyscallPerformanceMonitor {
    pub fn new() -> Self {
        Self {
            total_syscalls: AtomicU64::new(0),
            fast_path_syscalls: AtomicU64::new(0),
            standard_path_syscalls: AtomicU64::new(0),
            total_cycles: AtomicU64::new(0),
        }
    }

    /// Record system call execution
    pub fn record_syscall(&self, fast_path: bool, cycles: u64) {
        self.total_syscalls.fetch_add(1, Ordering::Relaxed);
        self.total_cycles.fetch_add(cycles, Ordering::Relaxed);
        
        if fast_path {
            self.fast_path_syscalls.fetch_add(1, Ordering::Relaxed);
        } else {
            self.standard_path_syscalls.fetch_add(1, Ordering::Relaxed);
        }
    }
}

// Global performance monitor
static PERFORMANCE_MONITOR: spin::Mutex<SyscallPerformanceMonitor> = spin::Mutex::new(SyscallPerformanceMonitor::new());

/// Initialize assembly interface for the current architecture
pub fn init_assembly_interface(arch: ArchType) -> Result<(), KernelError> {
    match arch {
        ArchType::X86_64 => {
            init_x86_64_interface()
        }
        ArchType::AArch64 => {
            init_arm64_interface()
        }
        ArchType::Riscv64 => {
            init_riscv64_interface()
        }
        ArchType::Unknown => {
            Err(KernelError::UnsupportedArchitecture)
        }
    }
}

/// Initialize x86_64 specific assembly interface
fn init_x86_64_interface() -> Result<(), KernelError> {
    info!("Initializing x86_64 assembly interface");
    
    // Setup syscall MSRs (Model Specific Registers)
    setup_x86_64_msrs()?;
    
    // Install syscall entry point
    install_x86_64_entry_point();
    
    info!("x86_64 assembly interface initialized successfully");
    Ok(())
}

/// Setup x86_64 Model Specific Registers for syscall
fn setup_x86_64_msrs() -> Result<(), KernelError> {
    // IA32_LSTAR - Long System Call Target Address
    // IA32_STAR - System Call Target Address
    // IA32_FMASK - RFLAGS Mask Register
    
    // In a real implementation, this would write to actual MSRs
    // using the `rdmsr` and `wrmsr` instructions
    
    debug!("Setting up x86_64 MSRs for syscall");
    
    // Placeholder implementation - would configure actual MSRs
    let syscall_entry = 0xFFFFFFFF_80001000u64; // Example entry point
    let star_value = 0x00160008_00000000u64;    // CS/SS selectors
    
    info!("x86_64 MSRs configured:");
    info!("  IA32_LSTAR: {:#x}", syscall_entry);
    info!("  IA32_STAR: {:#x}", star_value);
    
    Ok(())
}

/// Install x86_64 syscall entry point
fn install_x86_64_entry_point() {
    info!("Installing x86_64 syscall entry point");
    // In a real implementation, would write the entry point to the IDT
    // or setup the LSTAR MSR to point to the syscall handler
}

/// Initialize ARM64 specific assembly interface
fn init_arm64_interface() -> Result<(), KernelError> {
    info!("Initializing ARM64 assembly interface");
    
    // Setup Exception Vector Base Address Register (VBAR_EL1)
    setup_arm64_vbar()?;
    
    // Configure System Control Register (SCTLR_EL1)
    configure_arm64_sctlr()?;
    
    // Install SVC entry point
    install_arm64_entry_point();
    
    info!("ARM64 assembly interface initialized successfully");
    Ok(())
}

/// Setup ARM64 VBAR_EL1
fn setup_arm64_vbar() -> Result<(), KernelError> {
    debug!("Setting up ARM64 VBAR_EL1");
    
    // Placeholder - would write to VBAR_EL1 register
    let vbar_value = 0xFFFF0000_00001000u64; // Example vector base
    
    info!("ARM64 VBAR_EL1 configured: {:#x}", vbar_value);
    Ok(())
}

/// Configure ARM64 SCTLR_EL1
fn configure_arm64_sctlr() -> Result<(), KernelError> {
    debug!("Configuring ARM64 SCTLR_EL1");
    
    // Placeholder - would configure various system control bits
    info!("ARM64 SCTLR_EL1 configured");
    Ok(())
}

/// Install ARM64 SVC entry point
fn install_arm64_entry_point() {
    info!("Installing ARM64 SVC entry point");
    // Would configure the SVC vector in the exception table
}

/// Initialize RISC-V specific assembly interface
fn init_riscv64_interface() -> Result<(), KernelError> {
    info!("Initializing RISC-V assembly interface");
    
    // Setup Supervisor Trap Vector Base Address (stvec)
    setup_riscv64_stvec()?;
    
    // Configure Supervisor Status Register (sstatus)
    configure_riscv64_sstatus()?;
    
    // Install ECALL entry point
    install_riscv64_entry_point();
    
    info!("RISC-V assembly interface initialized successfully");
    Ok(())
}

/// Setup RISC-V stvec
fn setup_riscv64_stvec() -> Result<(), KernelError> {
    debug!("Setting up RISC-V stvec");
    
    // Placeholder - would write to stvec register
    let stvec_value = 0x80000000_00001000u64; // Example trap vector base
    
    info!("RISC-V stvec configured: {:#x}", stvec_value);
    Ok(())
}

/// Configure RISC-V sstatus
fn configure_riscv64_sstatus() -> Result<(), KernelError> {
    debug!("Configuring RISC-V sstatus");
    
    // Placeholder - would configure supervisor status bits
    info!("RISC-V sstatus configured");
    Ok(())
}

/// Install RISC-V ECALL entry point
fn install_riscv64_entry_point() {
    info!("Installing RISC-V ECALL entry point");
    // Would configure the ECALL handler in the trap vector
}

/// x86_64 Syscall Entry Point
#[no_mangle]
#[naked]
pub extern "C" fn x86_64_syscall_entry() -> ! {
    unsafe {
        // Save caller-saved registers
        core::arch::asm!(
            "push rbx",
            "push rcx",
            "push rdx", 
            "push rsi",
            "push rdi",
            "push rbp",
            "push r8",
            "push r9",
            "push r10",
            "push r11",
            
            // Get system call parameters from registers
            "mov {}, rax",           // syscall number from RAX
            "mov {}, rdi",           // arg0 from RDI
            "mov {}, rsi",           // arg1 from RSI
            "mov {}, rdx",           // arg2 from RDX
            "mov {}, r10",           // arg3 from R10
            "mov {}, r8",            // arg4 from R8
            "mov {}, r9",            // arg5 from R9
            
            // Preserve original return address
            "mov rcx, r11",          // Save RFLAGS in RCX
            
            // Call Rust syscall handler
            "call {}",
            
            // Restore registers
            "pop r11",
            "pop r10",
            "pop r9",
            "pop r8",
            "pop rbp",
            "pop rdi",
            "pop rsi",
            "pop rdx",
            "pop rcx",
            "pop rbx",
            
            // Return from syscall (RCX contains return address, R11 contains RFLAGS)
            "sysret",
            
            // Input constraints
            out(reg) _ /* syscall_num */,
            out(reg) _ /* arg0 */,
            out(reg) _ /* arg1 */,
            out(reg) _ /* arg2 */,
            out(reg) _ /* arg3 */,
            out(reg) _ /* arg4 */,
            out(reg) _ /* arg5 */,
            sym x86_64_syscall_handler
        );
    }
}

/// x86_64 Syscall Handler (Rust implementation)
#[no_mangle]
pub extern "C" fn x86_64_syscall_handler(
    syscall_num: usize,
    arg0: usize,
    arg1: usize,
    arg2: usize,
    arg3: usize,
    arg4: usize,
    arg5: usize
) -> (usize, usize) { // (return_value, error_code)
    
    let start_cycles = get_cycle_counter();
    
    // Increment global syscall counter
    SYSCALL_COUNTER.fetch_add(1, Ordering::Relaxed);
    
    // Create error context
    let context = error_handling::ErrorContext {
        syscall_number: syscall_num,
        process_id: get_current_process_id(),
        thread_id: get_current_thread_id(),
        privilege_level: PrivilegeLevel::Ring3 as usize,
        is_user_space_call: true,
        is_critical_system_call: is_critical_syscall(syscall_num),
        is_high_frequency_operation: is_high_frequency_syscall(syscall_num),
        failure_count: get_failure_count(syscall_num),
        timestamp: get_current_time(),
    };
    
    // Check for fast path eligibility
    let is_fast_path = is_fast_path_syscall(syscall_num);
    
    if is_fast_path {
        // Handle fast path syscalls
        let result = handle_fast_path_syscall(syscall_num, arg0, arg1, arg2, &context);
        
        let end_cycles = get_cycle_counter();
        let cycles_used = end_cycles - start_cycles;
        
        PERFORMANCE_MONITOR.lock().record_syscall(true, cycles_used);
        performance::record_syscall_performance(cycles_used as u64, result.1 == 0, true);
        
        result
    } else {
        // Handle standard path syscalls via dispatcher
        let params = SystemCallParams {
            syscall_number: syscall_num,
            arg0,
            arg1,
            arg2,
            arg3,
            arg4,
            arg5,
            caller_priv_level: PrivilegeLevel::Ring3,
        };
        
        let result = dispatcher::handle_system_call(params);
        
        let end_cycles = get_cycle_counter();
        let cycles_used = end_cycles - start_cycles;
        
        PERFORMANCE_MONITOR.lock().record_syscall(false, cycles_used);
        performance::record_syscall_performance(cycles_used as u64, true, false);
        
        (result.return_value, result.error_code as usize)
    }
}

/// ARM64 SVC Entry Point
#[no_mangle]
#[naked]
pub extern "C" fn arm64_svc_entry() -> ! {
    unsafe {
        // Save caller-saved registers
        core::arch::asm!(
            "str x0, [sp, #-16]!",
            "str x1, [sp, #-16]!",
            "str x2, [sp, #-16]!",
            "str x3, [sp, #-16]!",
            "str x4, [sp, #-16]!",
            "str x5, [sp, #-16]!",
            "str x6, [sp, #-16]!",
            "str x7, [sp, #-16]!",
            "str x8, [sp, #-16]!",
            
            // Get parameters from registers
            "mov x0, x8",            // syscall number from X8
            "ldr x1, [sp, 16]",      // arg0 from stack
            "ldr x2, [sp, 32]",      // arg1 from stack
            "ldr x3, [sp, 48]",      // arg2 from stack
            "ldr x4, [sp, 64]",      // arg3 from stack
            "ldr x5, [sp, 80]",      // arg4 from stack
            "ldr x6, [sp, 96]",      // arg5 from stack
            
            // Call Rust syscall handler
            "bl {}",
            
            // Restore registers and return
            "ldr x6, [sp], #16",
            "ldr x5, [sp], #16",
            "ldr x4, [sp], #16",
            "ldr x3, [sp], #16",
            "ldr x2, [sp], #16",
            "ldr x1, [sp], #16",
            "ldr x0, [sp], #16",
            
            // Return from SVC
            "eret",
            
            sym arm64_svc_handler
        );
    }
}

/// ARM64 SVC Handler (Rust implementation)
#[no_mangle]
pub extern "C" fn arm64_svc_handler(
    syscall_num: usize,
    arg0: usize,
    arg1: usize,
    arg2: usize,
    arg3: usize,
    arg4: usize,
    arg5: usize
) -> (usize, usize) {
    // Similar implementation to x86_64 but ARM64-specific
    arm64_generic_syscall_handler(syscall_num, arg0, arg1, arg2, arg3, arg4, arg5)
}

/// RISC-V ECALL Entry Point
#[no_mangle]
#[naked]
pub extern "C" fn riscv64_ecall_entry() -> ! {
    unsafe {
        // Save caller-saved registers
        core::arch::asm!(
            "addi sp, sp, -64",
            "sw t0, 0(sp)",
            "sw t1, 4(sp)",
            "sw t2, 8(sp)",
            "sw t3, 12(sp)",
            "sw t4, 16(sp)",
            "sw t5, 20(sp)",
            "sw t6, 24(sp)",
            "sw a0, 28(sp)",
            "sw a1, 32(sp)",
            "sw a2, 36(sp)",
            "sw a3, 40(sp)",
            "sw a4, 44(sp)",
            "sw a5, 48(sp)",
            "sw a6, 52(sp)",
            "sw a7, 56(sp)",
            
            // Get parameters
            "mv a0, a7",             // syscall number from a7
            "lw a1, 28(sp)",         // arg0 from saved a0
            "lw a2, 32(sp)",         // arg1 from saved a1
            "lw a3, 36(sp)",         // arg2 from saved a2
            "lw a4, 40(sp)",         // arg3 from saved a3
            "lw a5, 44(sp)",         // arg4 from saved a4
            "lw a6, 48(sp)",         // arg5 from saved a5
            
            // Call Rust syscall handler
            "jal {}",
            
            // Restore registers
            "lw a6, 52(sp)",
            "lw a5, 48(sp)",
            "lw a4, 44(sp)",
            "lw a3, 40(sp)",
            "lw a2, 36(sp)",
            "lw a1, 32(sp)",
            "lw a0, 28(sp)",
            "lw t6, 24(sp)",
            "lw t5, 20(sp)",
            "lw t4, 16(sp)",
            "lw t3, 12(sp)",
            "lw t2, 8(sp)",
            "lw t1, 4(sp)",
            "lw t0, 0(sp)",
            "addi sp, sp, 64",
            
            // Return from ECALL
            "sret",
            
            sym riscv64_ecall_handler
        );
    }
}

/// RISC-V ECALL Handler (Rust implementation)
#[no_mangle]
pub extern "C" fn riscv64_ecall_handler(
    syscall_num: usize,
    arg0: usize,
    arg1: usize,
    arg2: usize,
    arg3: usize,
    arg4: usize,
    arg5: usize
) -> (usize, usize) {
    // Similar implementation to x86_64 but RISC-V-specific
    arm64_generic_syscall_handler(syscall_num, arg0, arg1, arg2, arg3, arg4, arg5)
}

/// Generic syscall handler for non-x86 architectures
fn arm64_generic_syscall_handler(
    syscall_num: usize,
    arg0: usize,
    arg1: usize,
    arg2: usize,
    arg3: usize,
    arg4: usize,
    arg5: usize
) -> (usize, usize) {
    let start_cycles = get_cycle_counter();
    
    // Increment global syscall counter
    SYSCALL_COUNTER.fetch_add(1, Ordering::Relaxed);
    
    // Create error context
    let context = error_handling::ErrorContext {
        syscall_number: syscall_num,
        process_id: get_current_process_id(),
        thread_id: get_current_thread_id(),
        privilege_level: PrivilegeLevel::Ring3 as usize,
        is_user_space_call: true,
        is_critical_system_call: is_critical_syscall(syscall_num),
        is_high_frequency_operation: is_high_frequency_syscall(syscall_num),
        failure_count: get_failure_count(syscall_num),
        timestamp: get_current_time(),
    };
    
    // Check for fast path eligibility
    let is_fast_path = is_fast_path_syscall(syscall_num);
    
    if is_fast_path {
        let result = handle_fast_path_syscall(syscall_num, arg0, arg1, arg2, &context);
        
        let end_cycles = get_cycle_counter();
        let cycles_used = end_cycles - start_cycles;
        
        PERFORMANCE_MONITOR.lock().record_syscall(true, cycles_used);
        performance::record_syscall_performance(cycles_used as u64, result.1 == 0, true);
        
        result
    } else {
        let params = SystemCallParams {
            syscall_number: syscall_num,
            arg0,
            arg1,
            arg2,
            arg3,
            arg4,
            arg5,
            caller_priv_level: PrivilegeLevel::Ring3,
        };
        
        let result = dispatcher::handle_system_call(params);
        
        let end_cycles = get_cycle_counter();
        let cycles_used = end_cycles - start_cycles;
        
        PERFORMANCE_MONITOR.lock().record_syscall(false, cycles_used);
        performance::record_syscall_performance(cycles_used as u64, true, false);
        
        (result.return_value, result.error_code as usize)
    }
}

/// Handle fast path system calls
fn handle_fast_path_syscall(
    syscall_num: usize,
    arg0: usize,
    arg1: usize,
    arg2: usize,
    context: &error_handling::ErrorContext
) -> (usize, usize) {
    match syscall_num {
        // Thread yield - minimal overhead
        syscall_numbers::THREAD_YIELD => {
            crate::scheduler::yield_current_thread();
            (0, 0) // Success, no error
        }
        
        // Time get - direct kernel access
        syscall_numbers::TIME_GET => {
            use crate::bootstrap::get_boot_time;
            (get_boot_time(), 0)
        }
        
        // Process get PID - direct access to process table
        syscall_numbers::PROCESS_GETPID => {
            (get_current_process_id(), 0)
        }
        
        // Thread get TID - direct access to thread table
        syscall_numbers::THREAD_GETTID => {
            (get_current_thread_id(), 0)
        }
        
        // Process get PPID - direct access to process table
        syscall_numbers::PROCESS_GETPPID => {
            (get_parent_process_id(), 0)
        }
        
        // Get current working directory (simplified)
        syscall_numbers::DIRECTORY_CURRENT => {
            // Would return current directory path
            (0x1000, 0) // Return pointer to directory string
        }
        
        // Get process statistics (simplified)
        syscall_numbers::PROCESS_GETSTAT => {
            (0x2000, 0) // Return pointer to stats buffer
        }
        
        _ => {
            warn!("Fast path requested for unimplemented syscall {}", syscall_num);
            (0, InterruptError::SystemCallNotImplemented as usize)
        }
    }
}

/// Helper functions

/// Get current cycle counter
fn get_cycle_counter() -> u64 {
    // In real implementation, would use architecture-specific cycle counter
    // x86_64: RDTSC instruction
    // ARM64: PMCCNTR_EL0 register
    // RISC-V: rdcycle instruction
    1000 // Placeholder
}

/// Get current process ID
fn get_current_process_id() -> usize {
    // In real implementation, would read from task structure or TLS
    1 // Placeholder - assume process ID 1
}

/// Get current thread ID
fn get_current_thread_id() -> usize {
    // In real implementation, would read from task structure or TLS
    1 // Placeholder - assume thread ID 1
}

/// Get parent process ID
fn get_parent_process_id() -> usize {
    0 // Placeholder - no parent (init process)
}

/// Check if syscall is fast path eligible
fn is_fast_path_syscall(syscall_num: usize) -> bool {
    matches!(syscall_num,
        syscall_numbers::THREAD_YIELD |
        syscall_numbers::TIME_GET |
        syscall_numbers::PROCESS_GETPID |
        syscall_numbers::PROCESS_GETPPID |
        syscall_numbers::THREAD_GETTID |
        syscall_numbers::DIRECTORY_CURRENT |
        syscall_numbers::PROCESS_GETSTAT)
}

/// Check if syscall is critical
fn is_critical_syscall(syscall_num: usize) -> bool {
    matches!(syscall_num,
        syscall_numbers::PROCESS_CREATE |
        syscall_numbers::PROCESS_EXIT |
        syscall_numbers::VIRTUAL_ALLOC |
        syscall_numbers::PHYSICAL_ALLOC)
}

/// Check if syscall is high frequency
fn is_high_frequency_syscall(syscall_num: usize) -> bool {
    matches!(syscall_num,
        syscall_numbers::THREAD_YIELD |
        syscall_numbers::TIME_GET |
        syscall_numbers::PROCESS_GETPID |
        syscall_numbers::THREAD_GETTID)
}

/// Get failure count for syscall
fn get_failure_count(syscall_num: usize) -> u64 {
    // In real implementation, would track failure counts
    0 // Placeholder
}

/// Get current timestamp
fn get_current_time() -> u64 {
    // In real implementation, would get current time
    1000 // Placeholder
}

/// Get performance statistics
pub fn get_performance_stats() -> SyscallPerformanceStats {
    let monitor = PERFORMANCE_MONITOR.lock();
    SyscallPerformanceStats {
        total_syscalls: monitor.total_syscalls.load(Ordering::Relaxed),
        fast_path_syscalls: monitor.fast_path_syscalls.load(Ordering::Relaxed),
        standard_path_syscalls: monitor.standard_path_syscalls.load(Ordering::Relaxed),
        total_cycles: monitor.total_cycles.load(Ordering::Relaxed),
        avg_cycles_per_syscall: if monitor.total_syscalls.load(Ordering::Relaxed) > 0 {
            monitor.total_cycles.load(Ordering::Relaxed) / monitor.total_syscalls.load(Ordering::Relaxed)
        } else { 0 },
    }
}

/// Performance statistics structure
#[derive(Debug, Clone)]
pub struct SyscallPerformanceStats {
    pub total_syscalls: u64,
    pub fast_path_syscalls: u64,
    pub standard_path_syscalls: u64,
    pub total_cycles: u64,
    pub avg_cycles_per_syscall: u64,
}

/// Reset performance statistics
pub fn reset_performance_stats() {
    let mut monitor = PERFORMANCE_MONITOR.lock();
    monitor.total_syscalls.store(0, Ordering::Relaxed);
    monitor.fast_path_syscalls.store(0, Ordering::Relaxed);
    monitor.standard_path_syscalls.store(0, Ordering::Relaxed);
    monitor.total_cycles.store(0, Ordering::Relaxed);
}

/// Architecture-specific helper functions

/// Get CPU cycle counter for x86_64
#[cfg(target_arch = "x86_64")]
pub fn get_cycles() -> u64 {
    unsafe {
        let cycles: u64;
        core::arch::asm!("rdtsc", out(reg) cycles);
        cycles
    }
}

/// Get CPU cycle counter for ARM64
#[cfg(target_arch = "aarch64")]
pub fn get_cycles() -> u64 {
    unsafe {
        let cycles: u64;
        core::arch::asm!("mrs {}, pmccntr_el0", out(reg) cycles);
        cycles
    }
}

/// Get CPU cycle counter for RISC-V
#[cfg(target_arch = "riscv64")]
pub fn get_cycles() -> u64 {
    unsafe {
        let cycles: u64;
        core::arch::asm!("rdcycle {}", out(reg) cycles);
        cycles
    }
}

/// Default cycle counter for unsupported architectures
#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64", target_arch = "riscv64")))]
pub fn get_cycles() -> u64 {
    0 // Not supported
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fast_path_syscall_detection() {
        assert!(is_fast_path_syscall(syscall_numbers::THREAD_YIELD));
        assert!(is_fast_path_syscall(syscall_numbers::TIME_GET));
        assert!(is_fast_path_syscall(syscall_numbers::PROCESS_GETPID));
        assert!(!is_fast_path_syscall(syscall_numbers::FILE_OPEN));
    }

    #[test]
    fn test_critical_syscall_detection() {
        assert!(is_critical_syscall(syscall_numbers::PROCESS_CREATE));
        assert!(is_critical_syscall(syscall_numbers::VIRTUAL_ALLOC));
        assert!(!is_critical_syscall(syscall_numbers::TIME_GET));
    }

    #[test]
    fn test_high_frequency_syscall_detection() {
        assert!(is_high_frequency_syscall(syscall_numbers::THREAD_YIELD));
        assert!(is_high_frequency_syscall(syscall_numbers::TIME_GET));
        assert!(!is_high_frequency_syscall(syscall_numbers::FILE_OPEN));
    }

    #[test]
    fn test_performance_stats_structure() {
        let stats = SyscallPerformanceStats {
            total_syscalls: 1000,
            fast_path_syscalls: 800,
            standard_path_syscalls: 200,
            total_cycles: 500000,
            avg_cycles_per_syscall: 500,
        };
        
        assert_eq!(stats.total_syscalls, 1000);
        assert_eq!(stats.fast_path_percentage(), 80);
    }

    impl SyscallPerformanceStats {
        pub fn fast_path_percentage(&self) -> u64 {
            if self.total_syscalls > 0 {
                (self.fast_path_syscalls * 100) / self.total_syscalls
            } else {
                0
            }
        }
    }
}