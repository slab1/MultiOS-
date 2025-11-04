# MultiOS Quick Start Guide

Welcome to MultiOS! This guide will get you up and running with MultiOS in just a few minutes.

## What is MultiOS?

MultiOS is a modern, educational operating system written in Rust that supports multiple architectures (x86_64, ARM64, RISC-V). It features:

- 🏗️ **Hybrid microkernel architecture** for modularity and safety
- 🌍 **Cross-platform support** - runs on desktop, mobile, embedded, and server platforms  
- 🛡️ **Memory-safe implementation** using Rust's guarantees
- 🔧 **Educational design** - perfect for learning OS concepts
- 🚀 **High performance** with modern optimization techniques
- 📱 **Multi-device support** - works across different form factors

## Prerequisites

Before you begin, ensure you have:

### System Requirements
- **Operating System**: Linux, macOS, or Windows 10+
- **RAM**: 4GB minimum, 8GB recommended
- **Storage**: 2GB free space for source code and builds
- **CPU**: Any x86_64, ARM64, or RISC-V processor

### Required Software

#### Install Rust
```bash
# Install Rust using rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add Rust to your PATH
source ~/.cargo/env

# Verify installation
rustc --version
cargo --version
```

#### Install Build Dependencies

**On Ubuntu/Debian:**
```bash
sudo apt-get update
sudo apt-get install -y build-essential qemu-system-x86 qemu-system-aarch64 qemu-system-riscv64
sudo apt-get install -y gcc-aarch64-linux-gnu gcc-riscv64-linux-gnu
sudo apt-get install -y doxygen graphviz
```

**On macOS:**
```bash
# Install Homebrew if not already installed
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Install dependencies
brew install qemu

# Install cross-compilation tools (using Homebrew)
brew install aarch64-elf-gcc riscv64-elf-gcc
```

**On Windows:**
```bash
# Install QEMU for Windows
# Download from: https://www.qemu.org/download/#windows

# Install Visual Studio Build Tools
# Download from: https://visualstudio.microsoft.com/visual-cpp-build-tools/

# Install Windows Subsystem for Linux (WSL2) recommended for development
wsl --install
```

#### Install Additional Rust Tools
```bash
cargo install cargo-audit cargo-tarpaulin cross
```

## Download and Build MultiOS

### 1. Clone the Repository
```bash
git clone https://github.com/multios/multios.git
cd multios
```

### 2. Quick Build
```bash
# Build for x86_64 (default)
make build-x86_64

# Or build for all supported architectures
make build-all
```

### 3. Run Tests
```bash
# Run the test suite
make test-x86_64

# Run all tests
make test-all
```

## Running MultiOS

### Using QEMU (Recommended)

**For x86_64:**
```bash
# Build and run
make run-x86_64

# Or run manually
qemu-system-x86_64 -kernel target/x86_64-unknown-none-elf/release/multios -m 256M -nographic
```

**For ARM64:**
```bash
# Build and run
make run-arm64

# Or run manually
qemu-system-aarch64 -machine virt -cpu cortex-a57 -kernel target/aarch64-unknown-none-elf/release/multios -m 256M -nographic
```

**For RISC-V:**
```bash
# Build and run
make run-riscv64

# Or run manually
qemu-system-riscv64 -machine virt -kernel target/riscv64gc-unknown-none-elf/release/multios -m 256M -nographic
```

### Using Serial Console

To see boot messages and interact with the system:

```bash
# Terminal 1: Start QEMU
make run-x86_64

# Terminal 2: Open serial console
./scripts/serial_console.sh
```

## Your First MultiOS Application

Create your first application to test the system:

### 1. Create a Simple Application

Create `examples/hello_multios.rs`:

```rust
use multios_syscalls::*;

fn main() -> Result<(), MultiOSError> {
    // Print to serial console
    println!("Hello from MultiOS!");
    
    // Get system information
    let system_info = system_info()?;
    println!("Architecture: {:?}", system_info.architecture);
    println!("Memory: {} MB", system_info.total_memory / (1024 * 1024));
    
    // Create a simple process
    let process_params = ProcessCreateParams {
        name: b"hello_process".to_vec(),
        priority: ProcessPriority::Normal,
        stack_size: 4096,
        entry_point: Some(hello_thread),
    };
    
    let process_id = create_process(process_params)?;
    println!("Created process with ID: {}", process_id);
    
    // Sleep for a bit
    sleep(1000)?;
    
    println!("Hello from MultiOS - Completed successfully!");
    Ok(())
}

fn hello_thread() -> ! {
    println!("Thread running in MultiOS!");
    loop {}
}
```

### 2. Build and Run

```bash
# Add to your Cargo.toml
# [dependencies]
# multios-syscalls = { path = "kernel/syscalls" }

# Build the application
cargo build --example hello_multios

# Run in MultiOS environment
make run-x86_64
```

## Basic Commands

Once MultiOS is running, you can use these commands:

### Serial Console Commands
```
help           - Show available commands
info           - Display system information
memory         - Show memory usage
processes      - List running processes
drivers        - Show loaded drivers
reboot         - Restart the system
shutdown       - Power off
```

### Example Interaction
```
MultiOS> help
Available commands:
  help     - Show this help message
  info     - Display system information
  memory   - Show memory statistics
  processes - List running processes
  drivers  - Show loaded device drivers
  
MultiOS> info
MultiOS System Information
Architecture: x86_64
CPU Cores: 1
Memory: 256 MB
Uptime: 5 seconds
MultiOS>
```

## Development Workflow

### Daily Development Cycle

1. **Make changes** to source code
2. **Build** the kernel: `make build-x86_64`
3. **Test** your changes: `make test-x86_64`
4. **Run** the system: `make run-x86_64`
5. **Debug** if needed: See [Debugging Guide](debugging.md)

### Using the IDE

MultiOS works great with VS Code:

1. **Install VS Code** and the Rust extension
2. **Open the project**: `code .`
3. **Accept recommended extensions** when prompted
4. **Use build tasks**: Ctrl+Shift+P → "Tasks: Run Task"
5. **Debug**: F5 to start debugging session

## Next Steps

Congratulations! You now have MultiOS running. Here are some recommended next steps:

### Learn More
1. Read the [User Manual](user_guide/README.md) for detailed features
2. Explore the [Architecture Guide](architecture/README.md) to understand the design
3. Try the [Tutorials](tutorials/README.md) for hands-on learning

### Build Something
1. Create a [kernel module](tutorials/kernel_module.md)
2. Write a [device driver](tutorials/device_driver.md)
3. Build a [GUI application](tutorials/gui_app.md)
4. Explore [network programming](tutorials/network_programming.md)

### Contribute
1. Check out the [Contributing Guidelines](developer/contributing.md)
2. Look for [good first issues](https://github.com/multios/multios/issues)
3. Join the [community discussions](https://github.com/multios/multios/discussions)

## Troubleshooting

### Common Issues

#### Build Fails with "Rust toolchain not found"
```bash
# Ensure Rust is properly installed and in PATH
source ~/.cargo/env
rustc --version
```

#### QEMU not installed or not found
```bash
# Ubuntu/Debian
sudo apt-get install qemu-system-x86 qemu-system-aarch64 qemu-system-riscv64

# macOS
brew install qemu

# Verify installation
qemu-system-x86_64 --version
```

#### Permission denied when running QEMU
```bash
# Add user to kvm group (Linux)
sudo usermod -a -G kvm $USER
newgrp kvm
```

#### Cross-compilation fails
```bash
# Install cross-compilation targets
rustup target add x86_64-unknown-none-elf
rustup target add aarch64-unknown-none-elf
rustup target add riscv64gc-unknown-none-elf
```

### Getting Help

If you encounter issues:

1. Check the [Troubleshooting Guide](troubleshooting/README.md)
2. Search [existing issues](https://github.com/multios/multios/issues)
3. Create a [new issue](https://github.com/multios/multios/issues/new) with:
   - Your operating system and version
   - Complete error messages
   - Steps to reproduce the problem
   - Your system specifications

## Summary

You now have:
- ✅ MultiOS built and running on your system
- ✅ Basic understanding of the development workflow
- ✅ Your first application running on MultiOS
- ✅ Knowledge of where to find help and documentation

Welcome to the MultiOS community! 🚀

---

**Next**: [Installation Guide](installation.md) for detailed installation instructions  
**Up**: [Documentation Index](../README.md)  
**Related**: [System Requirements](requirements.md) | [Development Guide](../developer/README.md)