# MultiOS RISC-V GDB Configuration
# This file contains RISC-V-specific GDB settings and commands for MultiOS debugging

# Set architecture
set architecture riscv:rv64gc
set osabi none

# Set up register display
set print rep on

# Useful RISC-V breakpoints
define set-kernel-breakpoints-riscv64
    break kernel_main
    break _start
    break panic
    break hal::arch::riscv64::start::boot_main
    break riscv64::entry::boot::start_kernel
    break memory::init::init_heap
    break scheduler::init
end

# Multi-architecture boot commands
define boot-riscv64
    target remote | qemu-system-riscv64 -gdb stdio -S -machine virt -kernel $arg0
end
document boot-riscv64
    Boot RISC-V kernel in QEMU with GDB server
    Usage: boot-riscv64 <kernel_binary>
end

define qemu-riscv64
    target remote | qemu-system-riscv64 -gdb stdio -S -machine virt -kernel $arg0
end
document qemu-riscv64
    Launch QEMU for RISC-V debugging
    Usage: qemu-riscv64 <kernel_binary>
end

# Memory analysis for RISC-V
define analyze-riscv64-memory
    echo Analyzing RISC-V Memory Layout...\n
    
    printf "RISC-V Memory Analysis\n"
    printf "======================\n"
    
    # RISC-V uses PMP (Physical Memory Protection) and virtual memory
    printf "Check satp (Supervisor Address Translation and Protection) register\n"
    printf "for virtual memory status\n"
    printf "\nMemory regions:\n"
    printf "0x00000000-0x0000FFFF: Boot ROM / firmware\n"
    printf "0x00010000-0x80000000: DRAM\n"
    printf "0x80000000-: Kernel virtual address space\n"
end
document analyze-riscv64-memory
    Analyze RISC-V memory layout and protection
end

# Register analysis for RISC-V
define print-riscv64-registers
    printf "RISC-V General Purpose Registers (X0-X31):\n"
    printf "X0 (zero): 0x%016llx  X1 (ra):  0x%016llx  X2 (sp):  0x%016llx  X3 (gp):  0x%016llx\n", $x0, $x1, $x2, $x3
    printf "X4 (tp):   0x%016llx  X5 (t0):  0x%016llx  X6 (t1):  0x%016llx  X7 (t2):  0x%016llx\n", $x4, $x5, $x6, $x7
    printf "X8 (s0):   0x%016llx  X9 (s1):  0x%016llx  X10 (a0): 0x%016llx  X11 (a1): 0x%016llx\n", $x8, $x9, $x10, $x11
    printf "X12 (a2):  0x%016llx  X13 (a3): 0x%016llx  X14 (a4): 0x%016llx  X15 (a5): 0x%016llx\n", $x12, $x13, $x14, $x15
    printf "X16 (a6):  0x%016llx  X17 (a7): 0x%016llx  X18 (s2): 0x%016llx  X19 (s3): 0x%016llx\n", $x16, $x17, $x18, $x19
    printf "X20 (s4):  0x%016llx  X21 (s5): 0x%016llx  X22 (s6): 0x%016llx  X23 (s7): 0x%016llx\n", $x20, $x21, $x22, $x23
    printf "X24 (s8):  0x%016llx  X25 (s9): 0x%016llx  X26 (s10):0x%016llx  X27 (s11):0x%016llx\n", $x24, $x25, $x26, $x27
    printf "X28 (t3):  0x%016llx  X29 (t4): 0x%016llx  X30 (t5): 0x%016llx  X31 (t6): 0x%016llx\n", $x28, $x29, $x30, $x31
    
    printf "\nSpecial Registers:\n"
    printf "PC (Program Counter): 0x%016llx\n", $pc
    
    printf "\nRegister Usage Notes:\n"
    printf "X0 (zero): Always zero\n"
    printf "X1 (ra):   Return address\n"  
    printf "X2 (sp):   Stack pointer\n"
    printf "X3 (gp):   Global pointer\n"
    printf "X4 (tp):   Thread pointer\n"
    printf "X8 (s0):   Frame pointer\n"
    printf "X10-X17 (a0-a7): Function arguments/return values\n"
end
document print-riscv64-registers
    Print all RISC-V general purpose registers with usage notes
end

# Stack analysis for RISC-V
define analyze-riscv64-stack
    printf "Analyzing RISC-V Stack (SP: 0x%016llx)\n", $x2
    x/32x $x2
end
document analyze-riscv64-stack
    Analyze the current stack contents for RISC-V
end

# RISC-V specific breakpoint helpers
define set-riscv64-exception-breakpoints
    echo Setting RISC-V exception handling breakpoints...\n
    break riscv64::exception::handle_exception
    break riscv64::interrupt::handle_m_software_interrupt
    break riscv64::interrupt::handle_m_timer_interrupt
    break riscv64::interrupt::handle_m_external_interrupt
end
document set-riscv64-exception-breakpoints
    Set breakpoints for RISC-V exception and interrupt handling
end

# RISC-V control and status register display
define show-riscv64-csr-state
    printf "RISC-V Control and Status Registers (CSR)\n"
    printf "==========================================\n"
    printf "Note: CSR access requires proper RISC-V GDB setup\n"
    printf "Common CSRs to monitor:\n"
    printf "  satp: Supervisor Address Translation and Protection\n"
    printf "  stvec: Supervisor Trap Vector Base Address\n"
    printf "  sstatus: Supervisor Status\n"
    printf "  sie: Supervisor Interrupt Enable\n"
    printf "  sip: Supervisor Interrupt Pending\n"
    printf "  scause: Supervisor Trap Cause\n"
    printf "  stval: Supervisor Trap Value\n"
    printf "  time: Hardware Timer\n"
    printf "  cycle: Cycle Counter\n"
end
document show-riscv64-csr-state
    Show RISC-V CSR (Control and Status Register) information
end

# RISC-V privilege level analysis
define show-riscv64-privilege-level
    printf "RISC-V Privilege Level Analysis\n"
    printf "===============================\n"
    printf "Current privilege level: Check current_mode from CSR status\n"
    printf "Privilege levels:\n"
    printf "  0x00: User (U)\n"
    printf "  0x01: Supervisor (S)\n"
    printf "  0x10: Reserved\n"
    printf "  0x11: Machine (M)\n"
    printf "\nNote: Kernel typically runs in Supervisor (S) or Machine (M) mode\n"
end
document show-riscv64-privilege-level
    Show RISC-V privilege level information
end

# RISC-V PMP (Physical Memory Protection) analysis
define show-riscv64-pmp-state
    printf "RISC-V PMP (Physical Memory Protection)\n"
    printf "=======================================\n"
    printf "PMP configuration is stored in pmpcfg and pmpaddr CSRs\n"
    printf "Each PMP entry has:\n"
    printf "  - Address range (pmpaddr)\n"
    printf "  - Permissions and mode (pmpcfg)\n"
    printf "  - Available on RV64: 0-15 PMP entries\n"
    printf "  - Available on RV32: 0-63 PMP entries\n"
    printf "\nNote: PMP provides physical memory access control\n"
end
document show-riscv64-pmp-state
    Show RISC-V PMP (Physical Memory Protection) configuration
end

# Quick RISC-V setup
define setup-riscv64
    echo Setting up RISC-V debugging environment...\n
    set architecture riscv:rv64gc
    set pagination off
    set print pretty on
    set print rep on
    set confirm off
    set-kernel-breakpoints-riscv64
    echo RISC-V debugging ready.\n
end
document setup-riscv64
    Complete RISC-V debugging environment setup
end

# Print help for RISC-V commands
define help-riscv64
    printf "MultiOS RISC-V Debugging Commands:\n"
    printf "  set-kernel-breakpoints-riscv64      - Set common kernel breakpoints\n"
    printf "  boot-riscv64 <binary>               - Boot kernel with GDB\n"
    printf "  qemu-riscv64 <binary>               - Launch QEMU for debugging\n"
    printf "  analyze-riscv64-memory              - Analyze memory layout\n"
    printf "  print-riscv64-registers             - Print all registers\n"
    printf "  analyze-riscv64-stack               - Analyze stack contents\n"
    printf "  set-riscv64-exception-breakpoints   - Set exception handlers\n"
    printf "  show-riscv64-csr-state              - Show CSR state\n"
    printf "  show-riscv64-privilege-level        - Show privilege level\n"
    printf "  show-riscv64-pmp-state              - Show PMP configuration\n"
    printf "  setup-riscv64                       - Complete environment setup\n"
end
document help-riscv64
    Show RISC-V-specific debugging commands
end
