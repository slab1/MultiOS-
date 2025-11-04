//! Architecture-specific code for MultiOS
//! 
//! This module contains architecture-specific implementations for
//! different CPU architectures (x86_64, AArch64, RISC-V64).

use crate::{ArchType, BootInfo, KernelResult};

pub mod x86_64;
pub mod aarch64;
pub mod riscv64;

/// Initialize architecture-specific components
pub fn init(arch: ArchType, boot_info: &BootInfo) -> KernelResult<()> {
    match arch {
        ArchType::X86_64 => x86_64::init(boot_info),
        ArchType::AArch64 => aarch64::init(boot_info),
        ArchType::Riscv64 => riscv64::init(boot_info),
        ArchType::Unknown => Err(crate::KernelError::UnsupportedArchitecture),
    }
}

/// Get current privilege level
pub fn get_privilege_level() -> u8 {
    unsafe {
        core::arch::asm!("mov {}, cs", out(reg) _);
        // Ring 0 (kernel) or Ring 3 (user)
        0
    }
}

/// Check if running in user space
pub fn is_user_mode() -> bool {
    get_privilege_level() == 3
}

/// Check if running in kernel space
pub fn is_kernel_mode() -> bool {
    get_privilege_level() == 0
}

/// Save current processor state
pub fn save_state() -> ProcessorState {
    ProcessorState {
        rflags: read_flags(),
        // Additional state would be saved here
    }
}

/// Restore processor state
pub fn restore_state(state: ProcessorState) {
    write_flags(state.rflags);
    // Additional state would be restored here
}

/// Processor state structure
#[derive(Debug, Clone, Copy)]
pub struct ProcessorState {
    pub rflags: u64,
    // Additional state fields would be added here
}

/// Read RFLAGS register
fn read_flags() -> u64 {
    unsafe {
        let flags: u64;
        core::arch::asm!("pushfq; pop {}", out(reg) flags);
        flags
    }
}

/// Write RFLAGS register
fn write_flags(flags: u64) {
    unsafe {
        core::arch::asm!("push {}", in(reg) flags);
        core::arch::asm!("popfq");
    }
}

/// Common interrupt handling types and constants
pub mod interrupts {
    /// Maximum number of interrupt vectors
    pub const MAX_INTERRUPT_VECTORS: usize = 256;
    
    /// Timer interrupt vector
    pub const TIMER_INTERRUPT: u8 = 0x20;
    
    /// Keyboard interrupt vector
    pub const KEYBOARD_INTERRUPT: u8 = 0x21;
    
    /// Cascade interrupt (PIC)
    pub const CASCADE_INTERRUPT: u8 = 0x22;
    
    /// COM2 interrupt
    pub const COM2_INTERRUPT: u8 = 0x23;
    
    /// COM1 interrupt
    pub const COM1_INTERRUPT: u8 = 0x24;
    
    /// LPT2 interrupt
    pub const LPT2_INTERRUPT: u8 = 0x25;
    
    /// Floppy disk interrupt
    pub const FLOPPY_INTERRUPT: u8 = 0x26;
    
    /// LPT1 interrupt
    pub const LPT1_INTERRUPT: u8 = 0x27;
    
    /// CMOS clock interrupt
    pub const CMOS_INTERRUPT: u8 = 0x28;
    
    /// Free interrupt (available)
    pub const FREE_INTERRUPT_1: u8 = 0x29;
    
    /// Free interrupt (available)
    pub const FREE_INTERRUPT_2: u8 = 0x2A;
    
    /// Free interrupt (available)
    pub const FREE_INTERRUPT_3: u8 = 0x2B;
    
    /// Free interrupt (available)
    pub const FREE_INTERRUPT_4: u8 = 0x2C;
    
    /// Free interrupt (available)
    pub const FREE_INTERRUPT_5: u8 = 0x2D;
    
    /// Free interrupt (available)
    pub const FREE_INTERRUPT_6: u8 = 0x2E;
    
    /// Free interrupt (available)
    pub const FREE_INTERRUPT_7: u8 = 0x2F;
    
    /// System call interrupt vector (x86_64)
    pub const SYSCALL_INTERRUPT: u8 = 0x80;
    
    /// Start of IRQs from PIC
    pub const IRQ_BASE: u8 = 0x20;
    
    /// Maximum IRQ number
    pub const MAX_IRQ: u8 = 0x2F;
    
    /// System call number for test
    pub const SYSCALL_TEST: u64 = 0;
}

/// Common system call interface
pub mod syscall {
    /// System call numbers
    pub const SYSCALL_EXIT: u64 = 1;
    pub const SYSCALL_WRITE: u64 = 2;
    pub const SYSCALL_READ: u64 = 3;
    pub const SYSCALL_OPEN: u64 = 4;
    pub const SYSCALL_CLOSE: u64 = 5;
    pub const SYSCALL_MMAP: u64 = 6;
    pub const SYSCALL_MUNMAP: u64 = 7;
    pub const SYSCALL_GETPID: u64 = 8;
    pub const SYSCALL_FORK: u64 = 9;
    pub const SYSCALL_EXEC: u64 = 10;
    
    /// Maximum number of system call parameters
    pub const MAX_SYSCALL_PARAMS: usize = 6;
    
    /// System call parameter types
    pub type SyscallParam = u64;
    pub type SyscallResult = i64;
    
    /// System call context
    #[derive(Debug, Clone, Copy)]
    pub struct SyscallContext {
        pub syscall_number: SyscallParam,
        pub parameters: [SyscallParam; MAX_SYSCALL_PARAMS],
        pub return_value: SyscallResult,
    }
}
