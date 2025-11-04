# MultiOS x86_64 GDB Configuration
# This file contains x86_64-specific GDB settings and commands for MultiOS debugging

# Set architecture
set architecture i386:x86-64
set osabi none

# Set up assembly flavor
set disassembly-flavor intel

# Useful x86_64 breakpoints
define set-kernel-breakpoints-x86_64
    break kernel_main
    break _start
    break panic
    break hal::arch::x86_64::start::boot_main
    break x86_64::entry::boot::start_kernel
    break memory::init::init_heap
    break scheduler::init
end

# Multi-architecture boot commands
define boot-x86_64
    target remote | qemu-system-x86_64 -gdb stdio -S -kernel $arg0
end
document boot-x86_64
    Boot x86_64 kernel in QEMU with GDB server
    Usage: boot-x86_64 <kernel_binary>
end

define qemu-x86_64
    target remote | qemu-system-x86_64 -gdb stdio -S -kernel $arg0
end
document qemu-x86_64
    Launch QEMU for x86_64 debugging
    Usage: qemu-x86_64 <kernel_binary>
end

# Memory analysis for x86_64
define analyze-x86_64-memory
    echo Analyzing x86_64 Memory Layout...\n
    printf "CR3 (Page Table Base): 0x%016llx\n", $cr3
    
    # Print PML4 entries (first level page tables)
    set $pml4_base = $cr3 & 0xFFFFFFFFFFFF000
    printf "PML4 Base: 0x%016llx\n", $pml4_base
    
    # Display first few PML4 entries
    set $i = 0
    while $i < 4
        set $pml4_entry = *(unsigned long*)($pml4_base + $i * 8)
        if ($pml4_entry & 1)
            printf "PML4[%d]: 0x%016llx (Present, U/S=%d, R/W=%d)\n", $i, $pml4_entry, ($pml4_entry >> 2) & 1, ($pml4_entry >> 1) & 1
        else
            printf "PML4[%d]: 0x%016llx (Not Present)\n", $i, $pml4_entry
        end
        set $i = $i + 1
    end
end
document analyze-x86_64-memory
    Analyze x86_64 memory layout including page tables
end

# Register analysis
define print-x86_64-registers
    printf "General Purpose Registers:\n"
    printf "RAX: 0x%016llx  RBX: 0x%016llx  RCX: 0x%016llx  RDX: 0x%016llx\n", $rax, $rbx, $rcx, $rdx
    printf "RSI: 0x%016llx  RDI: 0x%016llx  RBP: 0x%016llx  RSP: 0x%016llx\n", $rsi, $rdi, $rbp, $rsp
    printf "R8:  0x%016llx  R9:  0x%016llx  R10: 0x%016llx  R11: 0x%016llx\n", $r8, $r9, $r10, $r11
    printf "R12: 0x%016llx  R13: 0x%016llx  R14: 0x%016llx  R15: 0x%016llx\n", $r12, $r13, $r14, $r15
    
    printf "\nInstruction Pointer and Flags:\n"
    printf "RIP: 0x%016llx\n", $rip
    
    # Print some flags manually (EFLAGS is complex to parse)
    printf "RFLAGS: 0x%016llx\n", $rflags
end
document print-x86_64-registers
    Print all x86_64 general purpose registers
end

# Stack analysis
define analyze-x86_64-stack
    printf "Analyzing x86_64 Stack (RSP: 0x%016llx)\n", $rsp
    x/32x $rsp
end
document analyze-x86_64-stack
    Analyze the current stack contents
end

# Quick x86_64 setup
define setup-x86_64
    echo Setting up x86_64 debugging environment...\n
    set architecture i386:x86-64
    set disassembly-flavor intel
    set pagination off
    set print pretty on
    set confirm off
    set-kernel-breakpoints-x86_64
    echo x86_64 debugging ready.\n
end
document setup-x86_64
    Complete x86_64 debugging environment setup
end

# Print help for x86_64 commands
define help-x86_64
    printf "MultiOS x86_64 Debugging Commands:\n"
    printf "  set-kernel-breakpoints-x86_64  - Set common kernel breakpoints\n"
    printf "  boot-x86_64 <binary>           - Boot kernel with GDB\n"
    printf "  qemu-x86_64 <binary>           - Launch QEMU for debugging\n"
    printf "  analyze-x86_64-memory          - Analyze memory layout\n"
    printf "  print-x86_64-registers         - Print all registers\n"
    printf "  analyze-x86_64-stack           - Analyze stack contents\n"
    printf "  setup-x86_64                   - Complete environment setup\n"
end
document help-x86_64
    Show x86_64-specific debugging commands
end
