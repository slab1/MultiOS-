# QEMU Testing Environment

A comprehensive multi-architecture QEMU testing environment supporting x86_64, ARM64 (AArch64), and RISC-V targets.

## Overview

This QEMU testing environment provides:

- **Multi-architecture support**: x86_64, ARM64, and RISC-V
- **Automated test runners**: Scripts for each architecture
- **Pre-configured VM settings**: Development and testing optimized
- **Unified management**: Single command to run all tests
- **Logging and monitoring**: Detailed test execution logs
- **Disk image management**: Automated disk creation and management

## Quick Start

### 1. Setup the Environment

```bash
# Run the setup script
cd qemu_testing
./scripts/setup.sh

# Or use make
make setup
```

### 2. Install QEMU (if not already installed)

#### Ubuntu/Debian
```bash
sudo apt update
sudo apt install qemu-system qemu-system-arm qemu-system-riscv qemu-utils
```

#### Fedora
```bash
sudo dnf install qemu-kvm qemu-system-x86 qemu-system-arm qemu-system-riscv
```

#### CentOS/RHEL
```bash
sudo yum install qemu-kvm qemu-system-x86 qemu-system-arm qemu-system-riscv
```

### 3. Download ISO Images

Create the images directory and download OS images:

```bash
# Create images directory
mkdir -p qemu_testing/images

# Download ISO images
# x86_64
wget https://releases.ubuntu.com/22.04/ubuntu-22.04.3-live-server-amd64.iso -O qemu_testing/images/ubuntu.iso

# ARM64
wget https://releases.ubuntu.com/22.04/ubuntu-22.04.3-live-server-arm64.iso -O qemu_testing/images/ubuntu-arm64.iso

# RISC-V (Fedora)
wget https://dl.fedoraproject.org/pub/alt/cloud/riscv64/images/40-20231114.0/Fedora-RISCV-40-20231114.0-noble-RAWHIDE-20231114-1148-sda.raw.xz -O qemu_testing/images/fedora-riscv.raw.xz
```

### 4. Run Tests

```bash
# Run all architectures
./scripts/run_all.sh

# Run specific architecture
./scripts/run_x86_64.sh
./scripts/run_arm64.sh
./scripts/run_riscv.sh

# Or use make
make test-all
make test-x86
make test-arm
make test-riscv
```

## Directory Structure

```
qemu_testing/
├── configs/              # VM configuration files
│   ├── x86_64_basic.conf
│   ├── arm64_basic.conf
│   └── riscv_basic.conf
├── scripts/              # Test runner scripts
│   ├── run_x86_64.sh
│   ├── run_arm64.sh
│   ├── run_riscv.sh
│   ├── run_all.sh
│   └── setup.sh
├── images/               # ISO and disk images
├── disks/                # Created disk images
├── logs/                 # Test execution logs
├── templates/            # VM templates
└── Makefile             # Build automation
```

## Usage Examples

### x86_64 Testing

```bash
# Basic run with defaults (512MB RAM, 2 CPUs)
./scripts/run_x86_64.sh

# Run with custom settings
./scripts/run_x86_64.sh -m 1G -c 4 -d 20G

# Boot from hard disk instead of CD-ROM
./scripts/run_x86_64.sh -b hd

# Run with specific network mode
./scripts/run_x86_64.sh -n none  # No network
./scripts/run_x86_64.sh -n bridge  # Bridge mode

# View all options
./scripts/run_x86_64.sh --help
```

### ARM64 Testing

```bash
# Basic run (1GB RAM, 2 CPUs, virt machine)
./scripts/run_arm64.sh

# Run with 2GB RAM and 4 CPUs
./scripts/run_arm64.sh -m 2G -c 4

# Use Versatile Express machine instead of virt
./scripts/run_arm64.sh -M vexpress

# View all options
./scripts/run_arm64.sh --help
```

### RISC-V Testing

```bash
# Basic run (1GB RAM, 2 CPUs, rv64gc ISA)
./scripts/run_riscv.sh

# Run with specific ISA
./scripts/run_riscv.sh -i rv64imafdc

# Use Spike machine instead of virt
./scripts/run_riscv.sh -M spike

# View all options
./scripts/run_riscv.sh --help
```

### Running All Architectures

```bash
# Run all architectures sequentially
./scripts/run_all.sh

# Run specific architecture with make
make test-all
```

## Configuration Files

Configuration files are located in `configs/` directory:

### x86_64 Basic Configuration

```ini
[machine]
type = "pc"
accel = "kvm"

[memory]
size = "512M"

[cpu]
smp = "2"

[boot]
mode = "cdrom"

[network]
type = "user"
model = "e1000"

[display]
type = "gtk"

[devices]
vga = "VGA"
sound = "ac97"
rtc_base = "utc"
```

### ARM64 Basic Configuration

```ini
[machine]
type = "virt"

[memory]
size = "1G"

[cpu]
model = "cortex-a57"
smp = "2"

[boot]
firmware = "uefi"

[network]
type = "user"
model = "virtio-net"

[display]
type = "gtk"

[devices]
xhci = "enabled"
usb_tablet = "enabled"
usb_keyboard = "enabled"
rtc_base = "utc"
```

### RISC-V Basic Configuration

```ini
[machine]
type = "virt"

[memory]
size = "1G"

[cpu]
model = "rv64gc"
smp = "2"

[firmware]
opensbi = "enabled"

[network]
type = "user"
model = "virtio-net"

[display]
type = "gtk"

[devices]
xhci = "enabled"
virtio_rng = "enabled"
virtio_gpu = "enabled"
virtio_tablet = "enabled"
virtio_keyboard = "enabled"
rtc_base = "utc"
```

## Makefile Targets

The Makefile provides convenient commands:

```bash
make help              # Show help
make setup             # Run setup script
make test-x86          # Run x86_64 test
make test-arm          # Run ARM64 test
make test-riscv        # Run RISC-V test
make test-all          # Run all tests
make disks             # Create disk images
make clean             # Clean generated files
make logs              # Show recent logs
make check-deps        # Check dependencies
make install-deps      # Install dependencies
make iso-dir           # Show ISO download info
make watch             # Watch logs in real-time
make show-config       # Show current configuration
```

## Creating Disk Images

The environment can automatically create disk images:

```bash
# Using make
make disks

# Manual disk creation
qemu-img create -f qcow2 qemu_testing/disks/x86_64_disk.qcow2 10G
qemu-img create -f qcow2 qemu_testing/disks/arm64_disk.qcow2 10G
qemu-img create -f qcow2 qemu_testing/disks/riscv_disk.qcow2 10G
```

## Network Configuration

### User Mode Networking (Default)

- Simple NAT-based networking
- VM can access internet
- No external access to VM
- Good for testing

```bash
./scripts/run_x86_64.sh -n user
```

### No Networking

- No network interface
- Maximum isolation
- Good for security testing

```bash
./scripts/run_x86_64.sh -n none
```

### Bridge Mode

- VM gets own IP on local network
- External access to VM
- Requires bridge setup

```bash
./scripts/run_x86_64.sh -n bridge
```

## Performance Tuning

### Memory Settings

```bash
# Increase memory for heavy workloads
./scripts/run_x86_64.sh -m 4G
./scripts/run_arm64.sh -m 4G
./scripts/run_riscv.sh -m 2G
```

### CPU Settings

```bash
# Increase CPU count for parallel tasks
./scripts/run_x86_64.sh -c 8
./scripts/run_arm64.sh -c 4
./scripts/run_riscv.sh -c 4
```

### KVM Acceleration

All scripts automatically enable KVM when available for better performance.

### Disk I/O Tuning

For I/O intensive workloads, create custom disk images with:

```bash
qemu-img create -f qcow2 -o preallocation=metadata,cluster_size=2M disk.qcow2 10G
```

## Troubleshooting

### QEMU Not Found

```bash
# Check if QEMU is installed
make check-deps

# Install if missing
make install-deps  # Ubuntu/Debian
```

### Permission Denied

```bash
# Make scripts executable
chmod +x scripts/*.sh

# Check user is in kvm group (for KVM acceleration)
sudo usermod -a -G kvm $USER
# Log out and back in for group changes to take effect
```

### No Bootable Device

```bash
# Verify ISO image exists
ls -la images/

# Download fresh ISO if needed
wget https://releases.ubuntu.com/22.04/ubuntu-22.04.3-live-server-amd64.iso -O images/ubuntu.iso
```

### Network Issues

```bash
# Disable networking for isolation
./scripts/run_x86_64.sh -n none

# Check firewall settings
sudo ufw status
```

### Performance Issues

```bash
# Increase memory and CPU
./scripts/run_x86_64.sh -m 2G -c 4

# Check KVM is working
ls -la /dev/kvm

# Check system resources
htop
```

## Advanced Usage

### Custom Boot Sequences

Edit the scripts to add custom boot options:

```bash
# Add kernel boot parameters
QEMU_ARGS="$QEMU_ARGS -append 'console=ttyS0'"
```

### Serial Console Access

```bash
# Add serial console to any VM
QEMU_ARGS="$QEMU_ARGS -serial stdio -append 'console=ttyS0'"
```

### Monitor Interface

Access QEMU monitor for runtime configuration:

```bash
# Press Ctrl+C to exit
# Monitor commands:
info status       # VM status
info network      # Network info
system_powerdown  # Graceful shutdown
```

### Snapshot Management

```bash
# Create snapshot
qemu-img create -f qcow2 -b disk.qcow2 snapshot.qcow2

# Run from snapshot
./scripts/run_x86_64.sh -hda snapshot.qcow2
```

## Development and Testing

### Automated Testing

Create test scripts in `scripts/test_*.sh`:

```bash
#!/bin/bash
# Example: test_installation.sh
./run_x86_64.sh -m 1G -c 2 <<EOF
# VM commands go here
echo "Testing installation"
# Add automated tests
EOF
```

### CI/CD Integration

Add to your CI pipeline:

```yaml
# Example GitHub Actions
- name: Test QEMU Environments
  run: |
    cd qemu_testing
    make setup
    make test-x86
    make test-arm
    make test-riscv
```

### Continuous Monitoring

```bash
# Watch logs in real-time
make watch

# Check recent logs
make logs
```

## Best Practices

1. **Always check dependencies** before running tests
2. **Use KVM acceleration** when available for better performance
3. **Allocate sufficient resources** for your use case
4. **Monitor system resources** during intensive testing
5. **Keep ISO images updated** for security patches
6. **Use snapshots** for safe experimentation
7. **Log test executions** for debugging
8. **Test all architectures** before production deployment

## Support and Resources

- **QEMU Documentation**: https://www.qemu.org/documentation/
- **QEMU Git Repository**: https://git.qemu.org/
- **Community Forums**: https://forum.qemu.org/
- **Bug Reports**: https://gitlab.com/qemu-project/qemu/-/issues

## License

This QEMU testing environment is provided as-is for educational and development purposes.

## Changelog

### Version 1.0.0
- Initial release
- Multi-architecture support (x86_64, ARM64, RISC-V)
- Automated test runners
- Configuration management
- Documentation and examples