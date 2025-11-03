# MultiOS Installation Wizard

A comprehensive installation wizard for MultiOS with automated hardware detection capabilities.

## Overview

The MultiOS Installation Wizard is a sophisticated installation framework built in Rust that provides:

- **Automated Hardware Detection**: Detects CPU, memory, storage, network, graphics, and input devices
- **User-Friendly GUI**: Step-by-step graphical installation interface
- **Flexible Partitioning**: Automatic and manual partition management
- **Driver Installation**: Automated driver installation and configuration
- **Multi-boot Support**: Detection and configuration of existing boot loaders
- **User Management**: Account creation and system configuration
- **Progress Tracking**: Detailed installation progress with logging
- **Error Recovery**: Rollback capabilities and error recovery
- **UEFI/Legacy BIOS Support**: Comprehensive boot system support
- **Network Configuration**: DHCP and static IP configuration during installation

## Architecture

```
src/
├── core/               # Core installation wizard logic
│   ├── mod.rs         # Main wizard orchestration
│   ├── config.rs      # Installation configuration
│   ├── state.rs       # Installation state management
│   ├── progress.rs    # Progress tracking and events
│   └── installation_manager.rs  # File copy and system setup
├── hardware/           # Hardware detection
│   ├── mod.rs         # Hardware detection logic
│   └── error.rs       # Hardware error types
├── partitioning/       # Disk partitioning
│   ├── mod.rs         # Partition management
│   └── error.rs       # Partitioning error types
├── drivers/            # Driver management
│   ├── mod.rs         # Driver installation
│   └── error.rs       # Driver error types
├── network/            # Network configuration
│   ├── mod.rs         # Network setup
│   └── error.rs       # Network error types
├── user/               # User account management
│   ├── mod.rs         # User creation and config
│   └── error.rs       # User error types
├── recovery/           # Installation rollback
│   ├── mod.rs         # Recovery and rollback
│   └── error.rs       # Recovery error types
├── gui/                # Graphical user interface
│   ├── mod.rs         # GUI manager and main app
│   ├── components.rs  # Reusable GUI components
│   └── pages.rs       # Installation wizard pages
└── main.rs            # Application entry point
```

## Key Components

### 1. Hardware Detection (`hardware/`)

The hardware detection system automatically identifies:

- **CPU**: Architecture, vendor, model, cores, threads, frequency, features
- **Memory**: Total, available, modules, type, speed, ECC support
- **Storage**: Devices, capacity, interface, model, type (SSD/HDD)
- **Network**: Interfaces, type, MAC address, speed, driver
- **Graphics**: GPUs, vendor, model, driver, memory, resolution
- **Audio**: Sound cards, driver, formats, sample rates
- **Input**: Keyboards, mice, touchpads, touchscreens
- **Boot**: Boot type, loader, firmware, security features

### 2. Partition Management (`partitioning/`)

Supports multiple partitioning modes:

- **Guided**: Automatic full-disk partitioning
- **Free Space**: Use existing free space
- **Manual**: Custom partition configuration
- **LVM**: Logical Volume Management support
- **Encryption**: LUKS encryption support
- **Multi-boot**: Detection of existing OS installations

### 3. Driver Management (`drivers/`)

Automated driver installation for:

- **Graphics**: NVIDIA, AMD, Intel, open-source alternatives
- **Network**: Ethernet and WiFi drivers
- **Storage**: AHCI, NVMe, SAS/SATA drivers
- **Audio**: ALSA/PulseAudio drivers
- **Custom**: User-provided driver packages

### 4. Installation Process

The installation follows these steps:

1. **Hardware Detection**: Scan and catalog system hardware
2. **Network Configuration**: Set up network connectivity
3. **Partitioning**: Configure disk partitions
4. **Driver Installation**: Install required drivers
5. **System Files**: Copy MultiOS system files
6. **Boot Configuration**: Set up bootloader
7. **User Setup**: Create user accounts
8. **Finalization**: Complete system configuration

### 5. Recovery System (`recovery/`)

Comprehensive rollback capabilities:

- **Recovery Points**: Create checkpoints before major changes
- **System Backup**: Backup critical configuration files
- **Partition Backup**: Save partition table information
- **Bootloader Backup**: Preserve existing boot configurations
- **Validation**: Verify recovery point integrity
- **Rollback**: Restore from previous state if needed

## Installation Types

### Text Mode Installation

Command-line based installation for servers or headless systems:

```bash
multios-installer --no-gui --config installation.json
```

### GUI Installation

Graphical installation with step-by-step wizard:

```bash
multios-installer
```

### Automated Installation

Scripted installation with configuration file:

```bash
multios-installer --config automated_install.json --dry-run
```

## Configuration

### Basic Configuration

```json
{
  "target": {
    "type": "disk",
    "path": "/dev/sda"
  },
  "boot_type": "auto",
  "username": "user",
  "hostname": "multios",
  "partition_config": {
    "root_size": 20971520,
    "home_size": 52428800,
    "swap_size": 4194304,
    "boot_size": 524288,
    "use_lvm": false,
    "encryption": false
  },
  "driver_selection": {
    "graphics_driver": "auto",
    "auto_install_recommended": true
  }
}
```

### Advanced Configuration

```json
{
  "target": {
    "type": "partition",
    "path": "/dev/sda1"
  },
  "boot_type": "uefi",
  "locale": "en_US.UTF-8",
  "timezone": "America/New_York",
  "keyboard_layout": "us",
  "username": "admin",
  "full_name": "System Administrator",
  "password": "secure_password",
  "auto_login": false,
  "hostname": "multios-server",
  "partition_config": {
    "root_size": 31457280,
    "home_size": 104857600,
    "swap_size": 8388608,
    "boot_size": 1048576,
    "use_lvm": true,
    "encryption": true,
    "encryption_password": "encryption_password"
  },
  "network_config": {
    "dhcp": false,
    "static_ip": "192.168.1.100",
    "netmask": "255.255.255.0",
    "gateway": "192.168.1.1",
    "dns_servers": ["8.8.8.8", "8.8.4.4"],
    "hostname": "multios-server"
  },
  "driver_selection": {
    "graphics_driver": "nvidia",
    "network_drivers": ["e1000", "iwlwifi"],
    "audio_driver": "hda_intel",
    "auto_install_recommended": true
  },
  "recovery_config": {
    "enable_recovery": true,
    "recovery_points": 5,
    "max_recovery_size": 5368709120,
    "enable_rollback": true
  }
}
```

## Examples

### Basic Installation

```rust
use multios_installation_wizard::{
    core::{InstallationConfig, InstallationWizard},
    hardware::HardwareDetector,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Detect hardware
    let hardware_info = HardwareDetector::detect_all().await?;
    
    // Create configuration
    let config = InstallationConfig::minimal();
    
    // Run installer
    let mut wizard = InstallationWizard::new(config, hardware_info);
    wizard.run_text_mode().await?;
    
    Ok(())
}
```

### Hardware Detection Only

```rust
use multios_installation_wizard::hardware::HardwareDetector;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let hardware_info = HardwareDetector::detect_all().await?;
    
    println!("CPU: {} cores", hardware_info.cpu.core_count);
    println!("Memory: {:.1} GB", 
        hardware_info.memory.total_bytes as f64 / 1e9);
    println!("Graphics: {}", hardware_info.graphics.gpu_vendor);
    
    Ok(())
}
```

## Command Line Options

```
MultiOS Installation Wizard v0.1.0

USAGE:
    multios-installer [OPTIONS]

OPTIONS:
    -c, --config <FILE>          Path to installation configuration file
    --dry-run                    Perform a dry run without actual installation
    --no-gui                     Run in text mode without GUI
    --output-log <FILE>          Path to save installation log
    -h, --help                   Print help information
    -V, --version                Print version information
```

## Hardware Requirements

### Minimum Requirements
- **CPU**: x86_64, ARM64, or RISC-V
- **Memory**: 2 GB RAM
- **Storage**: 10 GB available space
- **Graphics**: Any graphics card

### Recommended Requirements
- **CPU**: 4+ cores, 64-bit architecture
- **Memory**: 8 GB RAM
- **Storage**: 50 GB+ available space, SSD preferred
- **Graphics**: Dedicated GPU with drivers available

## Supported Platforms

### Architectures
- x86_64 (Intel/AMD)
- ARM64 (AArch64)
- RISC-V (64-bit)

### Boot Systems
- UEFI
- Legacy BIOS
- Hybrid boot

### File Systems
- ext4 (root, home)
- FAT32 (boot/EFI)
- btrfs (optional)
- XFS (optional)

## Error Recovery

The installation wizard includes comprehensive error recovery:

1. **Pre-installation Validation**: Hardware and configuration checks
2. **Recovery Points**: Automatic backups before major changes
3. **Error Detection**: Real-time monitoring of installation process
4. **Rollback Capability**: Restore to previous state on failure
5. **Detailed Logging**: Complete audit trail for troubleshooting

## Development

### Building

```bash
cd deployment/installation_wizard
cargo build
```

### Testing

```bash
cargo test
```

### Examples

```bash
cargo run --example basic_installation
cargo run --example hardware_detection
```

### GUI Development

To enable GUI features:

```bash
cargo build --features gui
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

## License

MIT License - see LICENSE file for details

## Support

For support and bug reports:
- GitHub Issues: [MultiOS Installation Wizard Issues](https://github.com/multios/installation-wizard/issues)
- Documentation: [MultiOS Documentation](https://docs.multios.org)
- Community: [MultiOS Community](https://community.multios.org)