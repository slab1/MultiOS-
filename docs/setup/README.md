# MultiOS Debugging Setup - Quick Start Guide

## Overview

This directory contains the complete debugging setup for MultiOS development, providing comprehensive tools for cross-platform kernel debugging across x86_64, ARM64, and RISC-V architectures.

## Quick Setup

Run the automated setup script to configure your development environment:

```bash
# Make the script executable and run it
chmod +x scripts/setup_dev_env.sh
bash scripts/setup_dev_env.sh
```

## What's Included

### 1. Debugging Tools Configuration
- **GDB Configuration**: Architecture-specific GDB setups for x86_64, ARM64, and RISC-V
- **Python Scripts**: Advanced memory and process analysis utilities
- **QEMU Scripts**: Monitoring and debugging environment setup
- **Serial Console**: Terminal-based debugging interface

### 2. IDE Integration (VS Code)
- **Tasks**: Automated build and test tasks for all architectures
- **Launch Configurations**: Debug configurations for each target architecture
- **Settings**: Optimized editor settings for Rust development
- **Extensions**: Recommended extensions list

### 3. Documentation
- **Complete Setup Guide**: `debugging_setup.md` - Comprehensive debugging documentation
- **Quick Start**: This README with essential information
- **Architecture Notes**: Specific guidance for each supported platform

## Directory Structure

```
docs/setup/
├── debugging_setup.md          # Main debugging documentation
├── README.md                   # This file
├── .gdbinit                    # Global GDB configuration
├── gdb_x86_64.gdb             # x86_64 GDB configuration
├── gdb_aarch64.gdb            # ARM64 GDB configuration
├── gdb_riscv64.gdb            # RISC-V GDB configuration
└── gdb_scripts/               # Python debugging utilities
    ├── memory.py              # Memory analysis tools
    └── process.py             # Process and scheduler analysis

scripts/
├── setup_dev_env.sh           # Automated setup script
├── qemu_monitor.sh            # QEMU monitoring script
└── serial_console.sh          # Serial console helper

.vscode/
├── tasks.json                 # VS Code tasks
├── launch.json                # Debug configurations
├── settings.json              # Editor settings
└── extensions.json            # Recommended extensions
```

## Getting Started

### 1. Initial Setup
```bash
# Run the automated setup (installs dependencies and configures environment)
bash scripts/setup_dev_env.sh
```

### 2. Build the Kernel
```bash
# Build for x86_64 (default target)
cargo build --target x86_64-unknown-none-elf

# Build for all architectures
cargo build --target x86_64-unknown-none-elf
cargo build --target aarch64-unknown-none-elf
cargo build --target riscv64gc-unknown-none-elf
```

### 3. Start Debugging

#### Using QEMU and GDB (Command Line)
```bash
# Terminal 1: Start QEMU with GDB server
./scripts/qemu_monitor.sh x86_64

# Terminal 2: Connect GDB
gdb-multiarch target/x86_64-unknown-none-elf/release/multios
(gdb) target remote localhost:1234
(gdb) continue
```

#### Using VS Code
1. Open VS Code in the project directory
2. Install recommended extensions when prompted
3. Press `Ctrl+Shift+P` → "Tasks: Run Task" → "QEMU x86_64"
4. Press `F5` → "Debug x86_64"

### 4. Access Serial Console
```bash
# In a separate terminal
./scripts/serial_console.sh
```

## Key Commands

### GDB Commands
```gdb
# General help
(gdb) multios-help

# Architecture-specific setup
(gdb) setup-x86_64
(gdb) setup-aarch64
(gdb) setup-riscv64

# Memory analysis
(gdb) multios-memory leak          # Complete memory analysis
(gdb) multios-memory heap          # Heap analysis
(gdb) multios-memory pagetables    # Page table analysis
(gdb) multios-memory stack         # Stack analysis

# Process analysis
(gdb) multios-process current      # Current task info
(gdb) multios-process scheduler    # Scheduler state
(gdb) multios-process context      # Context switching
(gdb) multios-process interrupt    # Interrupt handling

# Architecture-specific commands
(gdb) analyze-x86_64-memory        # x86_64 memory analysis
(gdb) print-aarch64-registers      # ARM64 register dump
(gdb) show-riscv64-csr-state       # RISC-V CSR state
```

### VS Code Tasks
- **Build x86_64 / aarch64 / riscv64**: Build kernel for specific architecture
- **Build All**: Build for all architectures in parallel
- **QEMU x86_64 / aarch64 / riscv64**: Start QEMU for that architecture
- **Debug x86_64 / aarch64 / riscv64**: Start debugging session
- **Serial Console**: Open serial console

### Cargo Commands
```bash
# Build for specific architecture
cargo build --target x86_64-unknown-none-elf

# Build with debug info
cargo build --target x86_64-unknown-none-elf --features debug

# Run tests
cargo test --target x86_64-unknown-none-elf

# Cross-compilation
cross build --target x86_64-unknown-none-elf
```

## Architecture Support

### x86_64 (Intel/AMD)
- **Debug Port**: TCP 1234
- **QEMU Command**: `qemu-system-x86_64 -kernel <binary> -gdb tcp::1234`
- **Features**: 
  - Page table analysis (CR3, PML4, PDPT, PD, PT)
  - Register dump (RAX, RBX, ..., RSP, RIP)
  - Stack analysis with Intel syntax
  - KVM support for better performance

### ARM64 (AArch64)
- **Debug Port**: TCP 1235
- **QEMU Command**: `qemu-system-aarch64 -machine virt -cpu cortex-a57 -kernel <binary> -gdb tcp::1235`
- **Features**:
  - Exception level (EL) analysis
  - AArch64 register dump (X0-X30, SP, PC)
  - System register access
  - Memory translation analysis

### RISC-V (RV64GC)
- **Debug Port**: TCP 1236
- **QEMU Command**: `qemu-system-riscv64 -machine virt -kernel <binary> -gdb tcp::1236`
- **Features**:
  - CSR (Control and Status Register) analysis
  - RISC-V register dump (X0-X31 with names)
  - PMP (Physical Memory Protection) analysis
  - Privilege level analysis

## Advanced Debugging

### Memory Analysis
The memory analysis utilities can detect:
- Memory leaks in kernel heap
- Page table corruption
- Stack overflow/underflow
- Memory access patterns

### Process Analysis
The process analysis tools provide insights into:
- Current task identification
- Scheduler state and behavior
- Context switching mechanisms
- Interrupt handling

### Serial Console Features
- Real-time boot messages
- Kernel panic output
- Debug print statements
- Interactive debugging with breakpoints

## Troubleshooting

### Common Issues

#### QEMU Won't Start
```bash
# Check if kernel binary exists
ls -la target/x86_64-unknown-none-elf/release/multios

# Try without KVM
qemu-system-x86_64 -kernel target/.../multios -nographic -enable-kvm=off

# Check QEMU version
qemu-system-x86_64 --version
```

#### GDB Connection Fails
```bash
# Check if port is in use
netstat -an | grep 1234

# Try different connection method
(gdb) target remote | qemu-system-x86_64 -gdb stdio

# Check architecture
(gdb) set architecture i386:x86-64
```

#### Serial Console Issues
```bash
# Check socket exists
ls -la /tmp/multios_serial

# Recreate socket directory
mkdir -p /tmp

# Use alternative tools
socat -,raw,echo=0 UNIX-CONNECT:/tmp/multios_serial
```

### Debug Scripts Not Loading
```bash
# Verify Python support in GDB
gdb-multiarch -ex "python print('Python works')"

# Check script syntax
python3 docs/setup/gdb_scripts/memory.py

# Load scripts manually
(gdb) source docs/setup/gdb_scripts/memory.py
```

## Best Practices

### 1. Development Workflow
1. Build kernel with debug symbols
2. Start QEMU with GDB server
3. Connect GDB and set breakpoints
4. Use serial console for output
5. Analyze memory and process state

### 2. Debug Session Setup
1. Always start with serial console to see boot messages
2. Set architecture-specific breakpoints early
3. Use memory analysis tools to investigate issues
4. Leverage VS Code integration for convenience

### 3. Educational Use
1. Use the analysis tools to demonstrate OS concepts
2. Explore different architectures to understand portability
3. Modify debugging scripts to add custom analysis
4. Create tutorials using the serial console output

## Documentation References

- **Main Guide**: `debugging_setup.md` - Complete debugging documentation
- **Cross-Compilation**: `../cross_compilation/cross_compilation_guide.md` - Build system setup
- **Technical Specs**: `../../multios_technical_specifications.md` - System architecture
- **Architecture Analysis**: `../os_architectures/os_architectures_analysis.md` - Design patterns

## Getting Help

### GDB Help
```gdb
(gdb) help
(gdb) help <command>
(gdb) multios-help
```

### Documentation
- Read the comprehensive guide in `debugging_setup.md`
- Check architecture-specific GDB files
- Review VS Code configuration examples

### Common Resources
- [GDB Documentation](https://sourceware.org/gdb/documentation/)
- [QEMU Documentation](https://www.qemu.org/documentation/)
- [Rust Debugging Guide](https://doc.rust-embedded.org/book/debugging/index.html)

## Contributing

To improve the debugging setup:

1. **Add new analysis tools**: Extend the Python scripts
2. **Improve GDB commands**: Add new architecture-specific commands
3. **Enhance VS Code integration**: Add new tasks or debug configurations
4. **Update documentation**: Keep guides current with changes

---

**MultiOS Development Team**  
*Educational Operating System Project*  
*Last Updated: 2025-11-02*
