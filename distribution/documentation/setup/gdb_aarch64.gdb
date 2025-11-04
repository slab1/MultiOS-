# MultiOS ARM64 GDB Configuration
# This file contains ARM64-specific GDB settings and commands for MultiOS debugging

# Set architecture
set architecture aarch64
set osabi none

# Useful ARM64 breakpoints
define set-kernel-breakpoints-aarch64
    break kernel_main
    break _start
    break panic
    break hal::arch::aarch64::start::boot_main
    break aarch64::entry::boot::start_kernel
    break memory::init::init_heap
    break scheduler::init
end

# Multi-architecture boot commands
define boot-aarch64
    target remote | qemu-system-aarch64 -gdb stdio -S -machine virt -cpu cortex-a57 -kernel $arg0
end
document boot-aarch64
    Boot ARM64 kernel in QEMU with GDB server
    Usage: boot-aarch64 <kernel_binary>
end

define qemu-aarch64
    target remote | qemu-system-aarch64 -gdb stdio -S -machine virt -cpu cortex-a57 -kernel $arg0
end
document qemu-aarch64
    Launch QEMU for ARM64 debugging
    Usage: qemu-aarch64 <kernel_binary>
end

# Memory analysis for ARM64
define analyze-aarch64-memory
    echo Analyzing ARM64 Memory Layout...\n
    
    # ARM64 uses EL (Exception Level) specific registers
    printf "Memory Analysis for ARM64\n"
    printf "============================\n"
    
    # Print TTBR (Translation Table Base Registers) info if available
    # This is architecture-specific and may not be directly accessible
    printf "Check SCTLR (System Control Register) for MMU status\n"
    printf "Translation tables are managed by EL1/EL2 firmware\n"
end
document analyze-aarch64-memory
    Analyze ARM64 memory layout and translation tables
end

# Register analysis for ARM64
define print-aarch64-registers
    printf "ARM64 General Purpose Registers (X0-X30):\n"
    printf "X0:  0x%016llx  X1:  0x%016llx  X2:  0x%016llx  X3:  0x%016llx\n", $x0, $x1, $x2, $x3
    printf "X4:  0x%016llx  X5:  0x%016llx  X6:  0x%016llx  X7:  0x%016llx\n", $x4, $x5, $x6, $x7
    printf "X8:  0x%016llx  X9:  0x%016llx  X10: 0x%016llx  X11: 0x%016llx\n", $x8, $x9, $x10, $x11
    printf "X12: 0x%016llx  X13: 0x%016llx  X14: 0x%016llx  X15: 0x%016llx\n", $x12, $x13, $x14, $x15
    printf "X16: 0x%016llx  X17: 0x%016llx  X18: 0x%016llx  X19: 0x%016llx\n", $x16, $x17, $x18, $x19
    printf "X20: 0x%016llx  X21: 0x%016llx  X22: 0x%016llx  X23: 0x%016llx\n", $x20, $x21, $x22, $x23
    printf "X24: 0x%016llx  X25: 0x%016llx  X26: 0x%016llx  X27: 0x%016llx\n", $x24, $x25, $x26, $x27
    printf "X28: 0x%016llx  X29: 0x%016llx  X30: 0x%016llx\n", $x28, $x29, $x30
    
    printf "\nSpecial Registers:\n"
    printf "SP (Stack Pointer): 0x%016llx\n", $sp
    printf "PC (Program Counter): 0x%016llx\n", $pc
    
    # FP (Frame Pointer) is X29 in ARM64
    printf "FP (Frame Pointer/X29): 0x%016llx\n", $x29
    
    # LR (Link Register) is X30 in ARM64  
    printf "LR (Link Register/X30): 0x%016llx\n", $x30
end
document print-aarch64-registers
    Print all ARM64 general purpose registers
end

# Stack analysis for ARM64
define analyze-aarch64-stack
    printf "Analyzing ARM64 Stack (SP: 0x%016llx)\n", $sp
    x/32x $sp
end
document analyze-aarch64-stack
    Analyze the current stack contents for ARM64
end

# ARM64 specific breakpoint helpers
define set-aarch64-exception-breakpoints
    echo Setting ARM64 exception handling breakpoints...\n
    break aarch64::exception::handle_synchronous_exception
    break aarch64::exception::handle_irq
    break aarch64::exception::handle_fiq
    break aarch64::exception::handle_serror
end
document set-aarch64-exception-breakpoints
    Set breakpoints for ARM64 exception handling
end

# ARM64 MMU and memory management helpers
define show-aarch64-mmu-state
    printf "ARM64 MMU State (check with 'info registers' for actual values):\n"
    printf "SCTLR_EL1: System Control Register (EL1)\n"
    printf "TTBR0_EL1: Translation Table Base Register 0 (EL1)\n"
    printf "TTBR1_EL1: Translation Table Base Register 1 (EL1)\n"
    printf "TCR_EL1: Translation Control Register (EL1)\n"
    printf "\nNote: Actual register values require privileged access\n"
end
document show-aarch64-mmu-state
    Show ARM64 MMU and memory management state
end

# Quick ARM64 setup
define setup-aarch64
    echo Setting up ARM64 debugging environment...\n
    set architecture aarch64
    set pagination off
    set print pretty on
    set confirm off
    set-kernel-breakpoints-aarch64
    echo ARM64 debugging ready.\n
end
document setup-aarch64
    Complete ARM64 debugging environment setup
end

# Print help for ARM64 commands
define help-aarch64
    printf "MultiOS ARM64 Debugging Commands:\n"
    printf "  set-kernel-breakpoints-aarch64     - Set common kernel breakpoints\n"
    printf "  boot-aarch64 <binary>              - Boot kernel with GDB\n"
    printf "  qemu-aarch64 <binary>              - Launch QEMU for debugging\n"
    printf "  analyze-aarch64-memory             - Analyze memory layout\n"
    printf "  print-aarch64-registers            - Print all registers\n"
    printf "  analyze-aarch64-stack              - Analyze stack contents\n"
    printf "  set-aarch64-exception-breakpoints  - Set exception handlers\n"
    printf "  show-aarch64-mmu-state             - Show MMU state\n"
    printf "  setup-aarch64                      - Complete environment setup\n"
end
document help-aarch64
    Show ARM64-specific debugging commands
end
