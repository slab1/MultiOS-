# MultiOS - Universal Operating System Distribution

[![Version](https://img.shields.io/badge/version-1.0.0-blue.svg)](https://github.com/multios/multios)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-green.svg)](LICENSE)
[![Documentation](https://img.shields.io/badge/docs-latest-blue.svg)](documentation/)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)]()

## Overview

MultiOS is a universal, educational operating system written in Rust that supports multiple architectures and deployment scenarios. This distribution package contains everything needed to install, build, and develop MultiOS.

## Quick Start

### Automatic Installation (Recommended)

```bash
# Extract the distribution
tar -xzf multios-distribution-*.tar.gz
cd multios-distribution-*

# Run the master installer
./install.sh

# Follow the interactive menu to select your installation type
```

### Manual Installation

Choose the installation script for your use case:

```bash
# Desktop/Laptop
sudo ./installation/desktop/install_multios_desktop.sh

# Server
sudo ./installation/server/install_multios_server.sh

# Embedded/IoT
sudo ./installation/embedded/install_multios_embedded.sh

# Development Environment
./installation/development/install_multios_dev.sh
```

## What's Included

This distribution contains:

### Core Components
- **Kernel** - Hybrid microkernel with multi-architecture support
- **Bootloader** - Multi-stage bootloader with UEFI and legacy support
- **Libraries** - Core system libraries and drivers
- **HAL** - Hardware Abstraction Layer for portability

### Installation Options
- **Desktop** - Full desktop environment with GUI support
- **Server** - Optimized for server workloads with monitoring
- **Embedded** - Minimal footprint for IoT and embedded devices
- **Development** - Complete development environment with tools

### Tools and Utilities
- Build scripts and automation
- Testing frameworks
- Debugging utilities
- Documentation generators
- Performance monitoring

### Documentation
- [Installation Guide](documentation/installation/)
- [User Manual](documentation/user-guide/)
- [Developer Guide](documentation/developer/)
- [API Documentation](documentation/api/)
- [Architecture Overview](documentation/architecture/)

## System Requirements

### Minimum Requirements

| Component | Desktop | Server | Embedded | Development |
|-----------|---------|--------|----------|-------------|
| CPU | x86_64/ARM64/RISC-V | x86_64/ARM64 | ARM/RISC-V | x86_64 |
| Memory | 2GB | 4GB | 512MB | 8GB |
| Storage | 10GB | 20GB | 2GB | 20GB |

### Recommended Requirements

| Component | Desktop | Server | Embedded | Development |
|-----------|---------|--------|----------|-------------|
| CPU | Multi-core | Multi-core | ARM Cortex-A | Multi-core |
| Memory | 4GB | 8GB | 1GB | 16GB |
| Storage | 50GB | 100GB | 8GB | 50GB |

## Supported Platforms

### Architectures
- **x86_64** - Desktop PCs and servers
- **ARM64 (AArch64)** - Modern ARM devices, servers
- **RISC-V** - Educational and embedded systems

### Device Types
- Desktop computers and laptops
- Servers and data centers
- Raspberry Pi and similar SBCs
- Embedded IoT devices
- Virtual machines
- Containers

## Installation Types

### 1. Desktop Installation
Full desktop environment with:
- Graphical user interface
- Window manager
- File manager
- Development tools
- Multimedia support

**Use Case:** Daily computing, development, education

### 2. Server Installation
Optimized for server workloads with:
- Enhanced security
- Monitoring dashboard
- Load balancing
- Backup systems
- Network services

**Use Case:** Web servers, databases, file servers

### 3. Embedded Installation
Minimal footprint with:
- IoT sensor support
- GPIO control
- Edge computing
- Web interface
- Cloud synchronization

**Use Case:** IoT gateways, smart devices, sensors

### 4. Development Installation
Complete development environment with:
- Rust toolchain
- IDE integration
- Build automation
- Testing frameworks
- Debugging tools

**Use Case:** Kernel development, driver writing, educational use

## Architecture

```
┌─────────────────────────────────────┐
│           MultiOS Kernel            │
├─────────────────────────────────────┤
│  HAL (Hardware Abstraction Layer)   │
├─────────────────────────────────────┤
│  Drivers │ Services │ IPC │ Memory │
├─────────────────────────────────────┤
│     Bootloader (Multi-stage)        │
└─────────────────────────────────────┘
```

### Key Features
- **Hybrid Microkernel** - Combines performance of monolithic kernels with modularity of microkernels
- **Rust Implementation** - Memory safety, concurrency, and performance
- **Multi-architecture** - Single codebase supporting x86_64, ARM64, and RISC-V
- **Service Architecture** - Modular services for easy extension
- **Hardware Abstraction** - Clean separation between hardware and software

## Building from Source

### Prerequisites

Install build dependencies:

```bash
# Ubuntu/Debian
sudo apt-get install build-essential rustc cargo qemu-system-x86

# Fedora
sudo dnf install @development-tools rust cargo qemu-system-x86

# Arch Linux
sudo pacman -S base-devel rust cargo qemu
```

### Build Commands

```bash
# Build for x86_64
cargo build --release --target x86_64-unknown-linux-gnu

# Build for ARM64
cargo build --release --target aarch64-unknown-none

# Build for RISC-V
cargo build --release --target riscv64gc-unknown-none-elf

# Run tests
cargo test

# Build documentation
cargo doc --no-deps
```

## Testing

### Verification

Verify the distribution integrity:

```bash
./verify.sh
```

### Kernel Testing

```bash
# Run in QEMU
qemu-system-x86_64 -kernel target/x86_64-unknown-linux-gnu/release/multios-kernel

# Run with debugging
qemu-system-x86_64 -kernel target/x86_64-unknown-linux-gnu/release/multios-kernel -s -S &
gdb target/x86_64-unknown-linux-gnu/release/multios-kernel
```

### Automated Tests

```bash
# Run full test suite
./scripts/test_all.sh

# Run architecture-specific tests
./scripts/test_x86_64.sh
./scripts/test_aarch64.sh
./scripts/test_riscv64.sh
```

## Development

### Setup Development Environment

```bash
# Install development dependencies
./installation/development/install_multios_dev.sh

# Configure IDE (VS Code, Vim, etc.)
# See installation/development/README.md for details
```

### Development Workflow

1. **Fork and Clone**
   ```bash
   git clone https://github.com/your-username/multios.git
   cd multios
   ```

2. **Build and Test**
   ```bash
   cargo build
   cargo test
   ```

3. **Debug**
   ```bash
   multios-debug.sh x86_64
   ```

4. **Contribute**
   - Follow coding standards
   - Write tests
   - Submit pull request

### IDE Support

- **VS Code** - Extensions: rust-analyzer, CodeLLDB
- **Vim** - Plugins: rust.vim, vim-racer
- **Emacs** - Modes: rust-mode, cargo-mode

## Configuration

### Kernel Configuration

Edit `/etc/multios/kernel.conf`:

```ini
[Memory]
max_heap=128M
enable_paging=true

[Processes]
max_processes=256

[Features]
enable_networking=true
enable_graphics=true
enable_security=true
```

### Service Configuration

Services are configured in `/etc/multios/services/`:

- `network.conf` - Network settings
- `graphics.conf` - Display settings
- `security.conf` - Security policies

## Troubleshooting

### Common Issues

**Installation fails with permission error**
```bash
# Ensure you have sudo privileges
sudo visudo
# Add: username ALL=(ALL) NOPASSWD: ALL
```

**Kernel won't boot**
```bash
# Check logs
sudo journalctl -u multios-kernel

# Verify bootloader installation
sudo multios-bootloader --verify
```

**Build errors**
```bash
# Update Rust toolchain
rustup update

# Clean build
cargo clean
cargo build
```

**QEMU won't start**
```bash
# Check available KVM
ls /dev/kvm

# Install KVM if needed
sudo apt-get install qemu-kvm libvirt-daemon-system
```

### Debug Mode

Boot MultiOS in debug mode:

1. Select "MultiOS (Debug)" from bootloader
2. Enable verbose logging
3. Connect GDB: `gdb -x .gdb/multios-kernel`

### Log Locations

- Kernel logs: `/var/log/multios/kernel.log`
- System logs: `/var/log/multios/system.log`
- Service logs: `/var/log/multios/services/*.log`

## Security

### Features
- Memory-safe implementation (Rust)
- Secure boot support
- Hardware security module integration
- Cryptographic libraries
- Access control and RBAC

### Security Checklist

- [ ] Enable secure boot (if supported)
- [ ] Configure firewall rules
- [ ] Set up automatic security updates
- [ ] Review audit logs
- [ ] Configure user permissions

## Performance

### Benchmarks

Typical performance characteristics:

| Metric | Desktop | Server | Embedded |
|--------|---------|--------|----------|
| Boot Time | 5-10s | 3-8s | 2-5s |
| Memory Footprint | 100-200MB | 200-500MB | 50-100MB |
| Context Switch | 1-2μs | <1μs | 2-5μs |
| System Calls | 500ns-1μs | 200-500ns | 1-2μs |

### Optimization Tips

- Enable KVM for virtualization
- Use appropriate kernel features
- Configure memory management
- Enable hardware acceleration
- Use optimized build flags

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Ways to Contribute
- Report bugs
- Suggest features
- Write documentation
- Fix issues
- Add new features
- Improve tests

### Development Process

1. Check existing issues
2. Create a new issue
3. Fork the repository
4. Create a feature branch
5. Make changes
6. Write tests
7. Submit pull request

## License

This project is dual-licensed under either:

- MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)

at your option.

## Support

### Getting Help

- **Documentation:** [docs.multios.org](https://docs.multios.org)
- **Issues:** [GitHub Issues](https://github.com/multios/multios/issues)
- **Discussions:** [GitHub Discussions](https://github.com/multios/multios/discussions)
- **Discord:** [multiosdiscord.com](https://discord.gg/multios)
- **Email:** support@multios.org

### Community

- **GitHub:** https://github.com/multios/multios
- **Website:** https://multios.org
- **Blog:** https://blog.multios.org
- **YouTube:** [MultiOS Channel](https://youtube.com/c/multios)

### Professional Support

Commercial support and consulting available. Contact: business@multios.org

## Changelog

### Version 1.0.0 (Current)
- Initial release
- Multi-architecture support (x86_64, ARM64, RISC-V)
- Desktop, Server, Embedded, and Development installations
- Complete documentation
- Comprehensive testing suite

### Roadmap

- [ ] Hardware acceleration support
- [ ] Additional architectures (MIPS, PowerPC)
- [ ] Cloud-native features
- [ ] Enhanced security features
- [ ] Performance optimizations
- [ ] Mobile device support

## Acknowledgments

Special thanks to:
- Rust community for the amazing language and ecosystem
- OSDev community for resources and knowledge
- Contributors to the MultiOS project
- Educational institutions using MultiOS for teaching

## Version Information

- **Distribution Version:** 1.0.0
- **Kernel Version:** 0.1.0
- **Build Date:** $(date -u +%Y-%m-%d)
- **Git Commit:** $(git rev-parse --short HEAD 2>/dev/null || echo "unknown")
- **Rust Version:** $(rustc --version 2>/dev/null || echo "unknown")

---

**Happy Computing with MultiOS! 🚀**