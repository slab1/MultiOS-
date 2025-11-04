# QEMU Multi-Architecture Testing Environment

[![QEMU](https://img.shields.io/badge/QEMU-Multi--Arch-green.svg)](https://www.qemu.org/)
[![x86_64](https://img.shields.io/badge/Architecture-x86__64-blue.svg)](https://www.qemu.org/)
[![ARM64](https://img.shields.io/badge/Architecture-ARM64-blue.svg)](https://www.qemu.org/)
[![RISC-V](https://img.shields.io/badge/Architecture-RISC--V-blue.svg)](https://www.qemu.org/)

A comprehensive testing environment for QEMU virtualization supporting multiple architectures.

## 🚀 Quick Start

```bash
# 1. Setup the environment
./scripts/setup.sh

# 2. Install QEMU (if needed)
make install-deps

# 3. Run all tests
make test-all
```

## 📋 Supported Architectures

- **x86_64**: Standard PC architecture with KVM acceleration
- **ARM64**: AArch64 with UEFI support and virt machine type
- **RISC-V**: RV64GC with OpenSBI firmware

## 🛠️ Key Features

- ✅ Multi-architecture support
- ✅ Automated test runners
- ✅ Configurable VM settings
- ✅ Disk image management
- ✅ Network configuration options
- ✅ Logging and monitoring
- ✅ Makefile integration
- ✅ Comprehensive documentation

## 📁 Directory Structure

```
qemu_testing/
├── 📄 README.md              # This file
├── 📄 Makefile               # Build automation
├── 📁 configs/               # VM configurations
│   ├── x86_64_basic.conf
│   ├── arm64_basic.conf
│   └── riscv_basic.conf
├── 📁 scripts/               # Test runners
│   ├── run_x86_64.sh         # x86_64 test runner
│   ├── run_arm64.sh          # ARM64 test runner
│   ├── run_riscv.sh          # RISC-V test runner
│   ├── run_all.sh            # Unified test runner
│   └── setup.sh              # Environment setup
├── 📁 images/                # ISO files (download here)
├── 📁 disks/                 # Disk images
├── 📁 logs/                  # Test logs
└── 📁 templates/             # VM templates
```

## 🎯 Usage Examples

### Individual Architecture Testing

```bash
# x86_64
./scripts/run_x86_64.sh -m 1G -c 4

# ARM64
./scripts/run_arm64.sh -m 2G -M vexpress

# RISC-V
./scripts/run_riscv.sh -i rv64imafdc
```

### Make Commands

```bash
make setup       # Initialize environment
make test-x86    # Test x86_64
make test-arm    # Test ARM64
make test-riscv  # Test RISC-V
make test-all    # Test all architectures
make disks       # Create disk images
make clean       # Clean generated files
make logs        # Show recent logs
```

## 📊 Requirements

### System Requirements

- Linux OS (Ubuntu 20.04+, Fedora 35+, CentOS 8+)
- Minimum 8GB RAM (16GB recommended)
- KVM support for better performance
- 20GB free disk space

### QEMU Packages

```bash
# Ubuntu/Debian
sudo apt install qemu-system qemu-system-arm qemu-system-riscv qemu-utils

# Fedora
sudo dnf install qemu-kvm qemu-system-x86 qemu-system-arm qemu-system-riscv

# CentOS/RHEL
sudo yum install qemu-kvm qemu-system-x86 qemu-system-arm qemu-system-riscv
```

## 💡 Configuration Options

### Memory Settings

- **x86_64**: 512MB - 8GB (default: 512MB)
- **ARM64**: 1GB - 8GB (default: 1GB)
- **RISC-V**: 1GB - 4GB (default: 1GB)

### CPU Settings

- **x86_64**: 1-16 cores (default: 2)
- **ARM64**: 1-8 cores (default: 2)
- **RISC-V**: 1-4 cores (default: 2)

### Network Modes

- **user**: NAT-based networking (default)
- **none**: No networking
- **bridge**: Bridge networking (requires setup)

## 🔧 Advanced Usage

### Custom Configurations

```bash
# Create custom VM configuration
cp configs/x86_64_basic.conf configs/x86_64_dev.conf
# Edit the configuration file
./scripts/run_x86_64.sh -c configs/x86_64_dev.conf
```

### Snapshot Testing

```bash
# Create snapshot
qemu-img create -f qcow2 -b disks/x86_64_disk.qcow2 snapshot.qcow2

# Run from snapshot
qemu-system-x86_64 -m 1G -hda snapshot.qcow2
```

### Serial Console

```bash
# Add serial console for headless testing
./scripts/run_x86_64.sh -append "console=ttyS0"
```

## 📖 Documentation

For detailed documentation, see: [`docs/setup/qemu_testing.md`](../setup/qemu_testing.md)

## 🐛 Troubleshooting

### QEMU Not Found

```bash
# Check installation
make check-deps

# Install missing packages
make install-deps
```

### Permission Denied

```bash
# Make scripts executable
chmod +x scripts/*.sh

# Add user to kvm group
sudo usermod -a -G kvm $USER
```

### Boot Issues

```bash
# Verify ISO exists
ls -la images/

# Check ISO integrity
file images/ubuntu.iso
```

## 🎓 Learning Resources

- [QEMU Documentation](https://www.qemu.org/documentation/)
- [Virtualization Guide](https://www.qemu.org/docs/master/system/index.html)
- [KVM Documentation](https://www.linux-kvm.org/page/Documents)
- [RISC-V QEMU](https://wiki.qemu.org/Features/RISC-V)

## 🤝 Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Test across all architectures
5. Submit a pull request

## 📝 License

This project is provided as-is for educational and development purposes.

## ⭐ Features Roadmap

- [ ] Docker container support
- [ ] GUI management interface
- [ ] Automated CI/CD integration
- [ ] Performance benchmarking
- [ ] More architecture support (PowerPC, MIPS, s390x)
- [ ] Network topology simulation
- [ ] Storage pool management

## 📞 Support

For issues and questions:

- Check the troubleshooting section
- Review logs: `make logs`
- Consult QEMU documentation
- Open an issue in the repository

---

**Happy Testing! 🎉**