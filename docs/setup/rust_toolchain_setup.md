# Rust Cross-Compilation Toolchain Setup Guide

## Overview

This guide describes the setup and configuration of Rust cross-compilation targets for bare-metal development on multiple architectures.

**Date:** November 2, 2025  
**Rust Version:** 1.91.0 (f8297e351 2025-10-28)

## Installed Targets

The following bare-metal cross-compilation targets have been configured:

1. **x86_64-unknown-none** - x86_64 64-bit bare metal
2. **aarch64-unknown-none** - ARM64 AArch64 bare metal
3. **riscv64gc-unknown-none-elf** - RISC-V 64-bit bare metal (with compressed instructions)

## Prerequisites

### System Requirements
- Linux (Debian/Ubuntu-based distribution)
- Internet connection for package downloads
- Minimum 2GB free disk space for toolchain installations

### Rust Installation

Rust was installed using `rustup`:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source $HOME/.cargo/env
```

**Installed Components:**
- rustc 1.91.0 (stable)
- cargo 1.91.0
- clippy
- rust-docs
- rust-std
- rustfmt

## Installation Steps

### 1. Install Rust Targets

Add the bare-metal targets using rustup:

```bash
# Add x86_64 bare-metal target
rustup target add x86_64-unknown-none

# Add ARM64 bare-metal target
rustup target add aarch64-unknown-none

# Add RISC-V bare-metal target
rustup target add riscv64gc-unknown-none-elf
```

### 2. Install Cross-Compilation Tools

Install GCC cross-compilers and binutils for each target architecture:

```bash
# Update package lists
apt update

# Install ARM64 cross-compilation tools
apt install -y gcc-aarch64-linux-gnu \
    binutils-aarch64-linux-gnu \
    libc6-arm64-cross \
    libgcc-12-dev-arm64-cross \
    linux-libc-dev-arm64-cross

# Install RISC-V cross-compilation tools
apt install -y gcc-riscv64-linux-gnu \
    binutils-riscv64-linux-gnu \
    libc6-riscv64-cross \
    libgcc-12-dev-riscv64-cross \
    linux-libc-dev-riscv64-cross
```

**Installed Packages:**
- `gcc-aarch64-linux-gnu` - ARM64 cross-compiler (GCC 12.2.0)
- `gcc-riscv64-linux-gnu` - RISC-V cross-compiler (GCC 12.2.0)
- `binutils-aarch64-linux-gnu` - ARM64 binary utilities
- `binutils-riscv64-linux-gnu` - RISC-V binary utilities
- Cross-compilation libraries and headers for each architecture

### 3. Configure Cargo

The `.cargo/config.toml` file has been created with appropriate linker configurations for each target.

## Configuration Details

### .cargo/config.toml Structure

```toml
[build]
default-target = "x86_64-unknown-linux-gnu"

[target.x86_64-unknown-none]
linker = "rust-lld"
rustflags = [
    "-C", "link-arg=--script=/path/to/linker_script.ld",
]

[target.aarch64-unknown-none]
linker = "aarch64-linux-gnu-gcc"
rustflags = [
    "-C", "link-arg=-nostartfiles",
    "-C", "link-arg=-static",
]

[target.riscv64gc-unknown-none-elf]
linker = "riscv64-linux-gnu-gcc"
rustflags = [
    "-C", "link-arg=-nostartfiles",
    "-C", "link-arg=-static",
]
```

### Target-Specific Configurations

#### x86_64-unknown-none (Bare Metal x86_64)
- **Linker:** rust-lld (LLVM linker)
- **Flags:** Uses custom linker script
- **Use Case:** Bootloaders, kernels, bare-metal applications
- **Note:** Requires custom linker script for proper memory layout

#### aarch64-unknown-none (Bare Metal ARM64)
- **Linker:** aarch64-linux-gnu-gcc
- **Flags:** `-nostartfiles -static`
- **Use Case:** ARM64 kernels, bootloaders, embedded systems
- **C Runtime:** No standard C library, fully statically linked

#### riscv64gc-unknown-none-elf (Bare Metal RISC-V 64-bit)
- **Linker:** riscv64-linux-gnu-gcc
- **Flags:** `-nostartfiles -static`
- **Use Case:** RISC-V kernels, bootloaders, embedded systems
- **ISA:** Supports RISC-V GC (General) instruction set including compressed instructions

## Usage Examples

### Building for a Specific Target

```bash
# Build for x86_64 bare metal
cargo build --target x86_64-unknown-none

# Build for ARM64 bare metal
cargo build --target aarch64-unknown-none

# Build for RISC-V bare metal
cargo build --target riscv64gc-unknown-none-elf
```

### Release Builds

```bash
cargo build --release --target aarch64-unknown-none
cargo build --release --target riscv64gc-unknown-none-elf
cargo build --release --target x86_64-unknown-none
```

### Using cargo-xbuild (Optional)

For advanced use cases, you can install cargo-xbuild for building the standard library:

```bash
cargo install cargo-xbuild
cargo xbuild --target aarch64-unknown-none
cargo xbuild --target riscv64gc-unknown-none-elf
cargo xbuild --target x86_64-unknown-none
```

## Verification

### Check Installed Targets

```bash
rustup target list --installed
```

Expected output:
```
aarch64-unknown-none (installed)
riscv64gc-unknown-none-elf (installed)
x86_64-unknown-linux-gnu (installed)
x86_64-unknown-none (installed)
```

### Verify Cross-Compilers

```bash
# Check ARM64 cross-compiler
aarch64-linux-gnu-gcc --version

# Check RISC-V cross-compiler
riscv64-linux-gnu-gcc --version

# Check ARM64 binary utilities
aarch64-linux-gnu-objdump --version

# Check RISC-V binary utilities
riscv64-linux-gnu-objdump --version
```

## Project Structure

### Recommended Bare-Metal Project Structure

```
your-project/
├── .cargo/
│   └── config.toml          # Created during setup
├── src/
│   ├── main.rs              # Entry point
│   └── lib.rs               # Library code
├── linker-scripts/
│   ├── x86_64.ld            # x86_64 linker script
│   ├── aarch64.ld           # ARM64 linker script
│   └── riscv64.ld           # RISC-V linker script
└── Cargo.toml               # Cargo configuration
```

### Example Cargo.toml for Bare-Metal

```toml
[package]
name = "bare-metal-app"
version = "0.1.0"
edition = "2021"

[dependencies]
# No std for bare-metal
# panic = "0.2.0"  # For panic handling

[profile.dev]
panic = "abort"

[profile.release]
panic = "abort"
```

## Troubleshooting

### Common Issues

1. **Missing Linker Errors**
   ```
   error: linker `aarch64-linux-gnu-gcc` not found
   ```
   **Solution:** Ensure cross-compiler packages are installed:
   ```bash
   apt install gcc-aarch64-linux-gnu
   ```

2. **Undefined Reference Errors**
   ```
   error: undefined reference to `__gnu_f2h_ieee`
   ```
   **Solution:** Ensure proper linker flags are set in config.toml

3. **Linker Script Not Found**
   ```
   error: cannot find linker script
   ```
   **Solution:** Create appropriate linker scripts for each target

### Getting Help

- **Rust Cross-Compilation:** https://rust-embedded.github.io/book/intro/cross.html
- **Rust Reference:** https://doc.rust-lang.org/rustc/targets/
- **GCC Cross-Compilation:** https://gcc.gnu.org/install/configure.html

## Advanced Configuration

### Building Custom Linker Scripts

For x86_64, you may need to create a custom linker script. Example:

```ld
/* linker.ld - Simple linker script for x86_64 bare metal */
OUTPUT_FORMAT(binary)
OUTPUT_ARCH(i386:x86-64)
ENTRY(_start)

SECTIONS
{
    . = 1M;
    
    .text : {
        *(.text)
    }
    
    .data : {
        *(.data)
    }
    
    .bss : {
        *(.bss)
    }
}
```

### Using LLVM Linker (rust-lld)

For x86_64 targets, rust-lld is recommended:

```bash
# Install rust-lld (usually included with Rust)
rustup component add rust-src

# Use in .cargo/config.toml
[target.x86_64-unknown-none]
linker = "rust-lld"
rustflags = ["-C", "link-arg=-T/linker.ld"]
```

## Maintenance

### Updating Targets

To update installed targets:
```bash
rustup target add --update-existing x86_64-unknown-none
rustup target add --update-existing aarch64-unknown-none
rustup target add --update-existing riscv64gc-unknown-none-elf
```

### Updating Rust

```bash
rustup update stable
```

### Removing Targets

```bash
rustup target remove x86_64-unknown-none
rustup target remove aarch64-unknown-none
rustup target remove riscv64gc-unknown-none-elf
```

## Security Considerations

- Bare-metal targets have no operating system security
- Always validate and sanitize inputs
- Use secure coding practices for embedded systems
- Consider memory protection mechanisms where available

## Performance Notes

- Cross-compilation is generally faster than native compilation
- Link time optimizations may require additional memory
- Consider using `cargo build --release` for final binaries

## References

1. [The Rustonomicon - Advanced](https://doc.rust-lang.org/nomicon/)
2. [Embedded Rust Book](https://rust-embedded.github.io/book/)
3. [Rust Reference - Targets](https://doc.rust-lang.org/rustc/targets/)
4. [GCC Cross-Compilation](https://gcc.gnu.org/onlinedocs/gcc/Configured-Targets.html)
5. [Binutils Documentation](https://sourceware.org/binutils/)

---

**Last Updated:** November 2, 2025  
**Setup Version:** 1.0  
**Maintainer:** System Administrator
