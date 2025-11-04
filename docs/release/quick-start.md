# MultiOS Quick Start Guide

Welcome to **MultiOS** - the revolutionary educational operating system written in Rust!

## What is MultiOS?

MultiOS is a modern, cross-platform operating system that supports **x86_64**, **ARM64**, and **RISC-V** architectures. It's designed for:
- **Educational purposes** - Learn OS development with modern Rust practices
- **Cross-platform deployment** - Run the same OS on different hardware
- **Production use** - Enterprise-grade reliability and performance
- **Research and experimentation** - Platform for OS research

## System Requirements

### Minimum Requirements
- **CPU**: Any 64-bit processor (x86_64, ARM64, or RISC-V)
- **Memory**: 512 MB RAM
- **Storage**: 2 GB available disk space
- **Boot**: UEFI or Legacy BIOS support

### Recommended Requirements
- **CPU**: Multi-core 64-bit processor
- **Memory**: 2 GB RAM or more
- **Storage**: 10 GB available disk space
- **Display**: VGA-compatible display
- **Network**: Ethernet or Wi-Fi adapter

## Quick Installation

### Method 1: Using Pre-built Images (Recommended)

1. **Download the MultiOS image**
   ```bash
   # Download for your architecture
   wget https://releases.multios.org/v1.0/x86_64/multios-x86_64.iso
   # or for ARM64
   wget https://releases.multios.org/v1.0/arm64/multios-arm64.iso
   ```

2. **Create bootable media**
   ```bash
   # Linux/macOS
   dd if=multios-x86_64.iso of=/dev/sdX bs=4M status=progress
   
   # Windows (using Rufus)
   # 1. Download Rufus from https://rufus.ie/
   # 2. Select MultiOS ISO and USB drive
   # 3. Click START
   ```

3. **Boot from the USB drive**
   - Restart your computer
   - Enter BIOS/UEFI settings (F2, F12, DEL, or ESC)
   - Set USB as first boot device
   - Save and restart

### Method 2: Build from Source

1. **Install dependencies**
   ```bash
   # Ubuntu/Debian
   sudo apt-get install -y build-essential qemu-system-x86 \
       gcc-aarch64-linux-gnu gcc-riscv64-linux-gnu doxygen \
       graphviz git
   
   # Install Rust
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   
   # Install cargo tools
   cargo install cargo-audit cargo-tarpaulin cross
   ```

2. **Clone the repository**
   ```bash
   git clone https://github.com/multios/multios.git
   cd multios
   ```

3. **Build MultiOS**
   ```bash
   # For your architecture
   make build-x86_64
   # or for all architectures
   make build-all
   ```

## Running MultiOS

### Option 1: Hardware Installation
After installation to your hard drive, MultiOS will boot automatically on system startup.

### Option 2: Virtual Machine Testing
Test MultiOS without installation using QEMU:

```bash
# Test x86_64 version
qemu-system-x86_64 -cdrom multios-x86_64.iso -m 1024 -boot d

# Test ARM64 version
qemu-system-aarch64 -cdrom multios-arm64.iso -m 1024 \
    -machine virt -cpu cortex-a57 -boot d
```

## First Boot Experience

1. **Boot Menu**: MultiOS presents a boot menu with options:
   - MultiOS (Normal Boot)
   - MultiOS (Safe Mode)
   - MultiOS (Debug Mode)

2. **Initial Setup**: On first boot:
   - Configure language and region
   - Set up user account
   - Configure network settings
   - Create disk partitions (if needed)

3. **Desktop Environment**: MultiOS features:
   - Modern GUI with Rust-based toolkit
   - File manager and terminal
   - Network connectivity tools
   - System monitoring dashboard

## Key Features at a Glance

### Cross-Platform Architecture
```rust
// Same codebase runs on all platforms
#[cfg(target_arch = "x86_64")]
fn platform_init() { /* x86_64 specific code */ }

#[cfg(target_arch = "aarch64")]
fn platform_init() { /* ARM64 specific code */ }

#[cfg(target_arch = "riscv64")]
fn platform_init() { /* RISC-V specific code */ }
```

### Advanced System Services
- **Memory Management**: Efficient allocation with safety guarantees
- **Process Scheduler**: Multi-core optimized scheduling
- **IPC System**: Fast inter-process communication
- **Network Stack**: Modern networking with protocols
- **File System**: MFS (MultiOS File System) with journaling
- **Driver Framework**: Hardware abstraction with safety

### Educational Resources
- **Interactive Tutorials**: Built-in OS development lessons
- **Code Examples**: Extensive examples in the documentation
- **Debug Tools**: Advanced debugging with GDB integration
- **Performance Profiling**: Built-in performance analysis

## Command Line Basics

Once booted, open the terminal and try these commands:

```bash
# Check system information
uname -a
cat /etc/multios-release

# List installed packages
multios-pkg list

# Install a package
multios-pkg install hello-world

# System monitoring
systemctl status
top

# Get help
help
man multios-pkg
```

## Development Quick Start

### Build a Simple Program

1. **Create a new project**
   ```bash
   multios-new-project hello-world
   cd hello-world
   ```

2. **Write your code** (`main.rs`)
   ```rust
   use multios::prelude::*;
   
   fn main() {
       println!("Hello from MultiOS!");
       println!("Running on: {}", multios::current_arch());
   }
   ```

3. **Build and run**
   ```bash
   cargo build --release
   cargo run
   ```

### Customize the Kernel

1. **Clone the source**
   ```bash
   git clone https://github.com/multios/kernel.git
   cd kernel
   ```

2. **Modify configuration**
   ```bash
   nano config/kernel.toml
   ```

3. **Rebuild**
   ```bash
   make clean
   make build-all
   ```

## Troubleshooting

### Common Issues

**Boot fails with "No bootable device":**
- Ensure USB is set as first boot device
- Try different USB port
- Check BIOS settings for UEFI/Legacy compatibility

**"Insufficient memory" error:**
- Allocate more RAM to virtual machine (minimum 512MB)
- Close other applications

**Build errors during compilation:**
- Update Rust toolchain: `rustup update`
- Install missing dependencies: `make install-deps`

**Network not working:**
- Check network adapter support in hardware compatibility list
- Try different network adapter in VM settings

### Getting Help

- **Documentation**: Visit https://docs.multios.org
- **Community Forum**: https://community.multios.org
- **GitHub Issues**: https://github.com/multios/multios/issues
- **IRC Chat**: #multios on Libera.Chat

## Next Steps

1. **Explore the Desktop**: Get familiar with the user interface
2. **Read Documentation**: Browse the comprehensive guides
3. **Try Development**: Build your first MultiOS application
4. **Join Community**: Contribute to the project
5. **Report Issues**: Help improve MultiOS

## Advanced Usage

### Multi-Boot Setup

Install MultiOS alongside other operating systems:

```bash
# Run the installer
multios-installer --multi-boot

# The installer will:
# 1. Detect existing OS installations
# 2. Offer to create new partition
# 3. Install bootloader
# 4. Configure boot menu
```

### Server Installation

For server deployments:

```bash
# Install server edition
multios-installer --server

# Features included:
# - Minimal GUI or CLI only
# - Server management tools
# - Network services
# - Security hardening
```

### Development Installation

For development work:

```bash
# Install development environment
multios-installer --development

# Includes:
# - Full source tree
# - Build tools
# - Debug symbols
# - Development documentation
```

## Performance Tips

1. **For better performance:**
   - Use SSD storage for better I/O
   - Allocate sufficient RAM (2GB+ recommended)
   - Enable hardware acceleration in VMs

2. **For resource-constrained systems:**
   - Boot in safe mode
   - Disable unnecessary services
   - Use CLI instead of GUI

## Conclusion

Welcome to the future of operating systems! MultiOS combines the safety of Rust with the power of a modern OS, all while supporting multiple hardware platforms.

Start with the basic installation and gradually explore the advanced features. The comprehensive documentation and active community will help you make the most of your MultiOS experience.

**Happy computing with MultiOS! 🚀**

---

## Quick Reference Commands

| Action | Command |
|--------|---------|
| System Info | `systeminfo` |
| Package Manager | `multios-pkg <command>` |
| System Monitor | `multios-top` |
| File Manager | `multios-fm` |
| Network Config | `netcfg` |
| System Update | `multios-update` |
| Help | `help` or `man multios` |

For more detailed information, see the [Installation Guide](installation.md) and [System Requirements](requirements.md).