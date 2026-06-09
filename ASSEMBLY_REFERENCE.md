# Quick Reference: Assembly Conversion Cheat Sheet

## AT&T vs Intel Syntax Quick Guide

### Basic Operation Formats

| Operation | AT&T | Intel | Example |
|-----------|------|-------|---------|
| **Move register** | `mov %src, %dst` | `mov dst, src` | `mov eax, ebx` |
| **Move immediate** | `mov $value, %reg` | `mov reg, value` | `mov eax, 1` |
| **Move memory** | `mov (%addr), %reg` | `mov reg, [addr]` | `mov eax, [rsp]` |
| **Load Effective Address** | `lea value, %reg` | `lea reg, [value]` | `lea rax, [rip+8]` |
| **Add** | `add %src, %dst` | `add dst, src` | `add rax, rbx` |
| **Subtract** | `sub %src, %dst` | `sub dst, src` | `sub eax, 1` |
| **Push** | `push %reg` | `push reg` | `push rbx` |
| **Pop** | `pop %reg` | `pop reg` | `pop rax` |

### Key Differences

```
AT&T Syntax (Old - Won't compile in Rust asm! macro):
├── Register prefix: %rax, %ebx, %r10
├── Immediate prefix: $1, $0xFF, $0x1B
├── Operand order: source, destination (mov %src, %dst)
├── Memory format: offset(%base,%index,scale)
└── Mnemonic suffix: movb, movw, movl, movq (size in instruction)

Intel Syntax (New - Required for Rust asm! macro):
├── Register prefix: (none) rax, ebx, r10
├── Immediate prefix: (none) 1, 0xFF, 0x1B
├── Operand order: destination, source (mov dst, src)
├── Memory format: [base + index*scale + offset]
└── Mnemonic suffix: (no size suffix, inferred from operand)
```

## Most Common Fixes in MultiOS

### Fix #1: Control Register Access
```rust
// BEFORE (AT&T)
asm!("mov %cr0, {}", out(reg) value);
asm!("mov {}, %cr4", in(reg) value);

// AFTER (Intel)
asm!("mov {}, cr0", out(reg) value, options(nomem, nostack));
asm!("mov cr4, {}", in(reg) value, options(nomem, nostack));
```

### Fix #2: Syscall Arguments
```rust
// BEFORE (AT&T)
asm!(
    "mov %rdi, {}",    // rdi (1st arg)
    "mov %rsi, {}",    // rsi (2nd arg)
    "mov %rdx, {}",    // rdx (3rd arg)
    out(reg) arg0,
    out(reg) arg1,
    out(reg) arg2
);

// AFTER (Intel)
asm!(
    "mov {}, rdi",     // 1st arg
    "mov {}, rsi",     // 2nd arg
    "mov {}, rdx",     // 3rd arg
    out(reg) arg0,
    out(reg) arg1,
    out(reg) arg2,
    options(nomem, nostack)
);
```

### Fix #3: MSR Operations
```rust
// BEFORE (AT&T)
asm!(
    "mov $0x1B, %ecx",     // MSR_APIC_BASE
    "rdmsr",
    "mov %rax, {}",
    out(reg) apic_base
);

// AFTER (Intel)
asm!(
    "mov ecx, 0x1B",       // MSR_APIC_BASE
    "rdmsr",
    "mov {}, rax",
    out(reg) apic_base,
    options(nomem, nostack)
);
```

### Fix #4: SIMD Instructions
```rust
// BEFORE (AT&T)
asm!(
    "movups {}, %xmm0",
    "addps %xmm1, %xmm0",
    "movups %xmm0, {}",
    in(reg) src1,
    in(reg) src2,
    out(reg) dst
);

// AFTER (Intel)
asm!(
    "movups xmm0, {}",
    "addps xmm0, xmm1",
    "movups {}, xmm0",
    in(reg) src1,
    in(reg) src2,
    out(reg) dst,
    options(nomem, nostack)
);
```

### Fix #5: Port I/O
```rust
// BEFORE (AT&T)
asm!("out %dx, %al", in("edx") port, in("al") value);

// AFTER (Intel)
asm!("out al, dx", in("edx") port, in("al") value, options(nomem, nostack));
```

## Register Reference

### General Purpose (x86_64)
```
64-bit:  rax, rbx, rcx, rdx, rsi, rdi, r8-r15
32-bit:  eax, ebx, ecx, edx, esi, edi, r8d-r15d
16-bit:  ax,  bx,  cx,  dx,  si,  di,  r8w-r15w
8-bit:   al,  bl,  cl,  dl,  sil, dil, r8b-r15b
```

### Special Registers
```
rsp  - Stack pointer
rbp  - Base pointer
rip  - Instruction pointer (often implicit)
```

### SIMD Registers
```
xmm0-xmm15   - 128-bit SSE registers
ymm0-ymm15   - 256-bit AVX registers
zmm0-zmm31   - 512-bit AVX-512 registers
```

### Control Registers
```
cr0  - Control register 0 (PE, PG bits)
cr2  - Page fault address
cr3  - Page table base
cr4  - Control register 4 (PAE, PSE bits)
cr8  - Task priority
```

## Rust asm! Macro Syntax

```rust
unsafe {
    core::arch::asm!(
        // Template strings with {} for operands
        "instruction {}, {}",
        // Operand constraints
        in("reg") value_in,          // Input in specific register
        in(reg) value_in,            // Input in any register
        out(reg) value_out,          // Output in any register
        out("rax") eax_val,          // Output in specific register
        inout(reg) rw_value,         // Input and output same register
        options(nomem, nostack)      // Optimization hints
    );
}
```

### Common Options
```
nomem    - Doesn't read or write memory
nostack  - Doesn't use stack
readonly - Doesn't write inputs
pure     - No side effects
volatile - Must not be optimized away
att_syntax - Use AT&T syntax (NOT RECOMMENDED)
```

## Debugging Tips

### 1. Check Line Numbers
```bash
# Error message shows file and line
# Open file and look at that line's asm! block
cargo check --target x86_64-unknown-none 2>&1 | grep "error\|-->" | head -5
```

### 2. Compare with Original
```bash
git diff HEAD~1 kernel/src/arch/x86_64/mod.rs | grep "asm!"
```

### 3. Validate Operand Count
```
Error: "template string references operand 2, but only 2 operands were provided"
→ means {} count doesn't match constraints
→ count: "mov {}, {}, {}" = 3 templates, need 3 operand lines
```

### 4. Verify Register Names
- All lowercase: rax, not RAX
- No prefix: rax, not %rax
- Valid names: Check x86_64 register reference above

## Validation Checklist

- [ ] No `%` before register names (except `%%` for literal %)
- [ ] Operands in correct order: destination, source
- [ ] All `{}` placeholders have matching `in()` or `out()` constraints
- [ ] Options clause present: `options(nomem, nostack)` or similar
- [ ] CPUID uses named registers: `in("eax")`, `out("edx")`
- [ ] Control reg access uses correct names: cr0, cr3, cr4, cr8
- [ ] No size suffixes on mnemonics: `mov` not `movq`
- [ ] Memory access uses brackets: `[rsp]` not `(%rsp)`

## Files to Review

Primary files changed:
1. `kernel/src/arch/x86_64/mod.rs` - 12 functions
2. `kernel/src/arch/x86_64/interrupt.rs` - Syscall/exception handlers
3. `kernel/src/arch/x86_64/apic.rs` - APIC operations
4. `kernel/src/arch/features.rs` - SIMD operations
5. `kernel/src/arch/multicore.rs` - IPI messaging
6. `kernel/src/arch/performance.rs` - PMU control
7-9. `kernel/src/hal/{cpu,multicore}.rs` + `bootstrap/`.rs

---

**Quick Test:**
```bash
# Verify all assembly syntax is correct
find kernel/src -name "*.rs" | xargs grep '".*%[a-z]' | grep -v '%Y\|%m\|%d' | wc -l
# Expected output: 0
```
