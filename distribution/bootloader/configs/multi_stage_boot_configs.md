# Multi-Stage Boot Configuration Files

This directory contains comprehensive examples of boot configuration files for the MultiOS bootloader, demonstrating all supported formats and boot modes.

## Configuration File Formats

### 1. GRUB2 Format (`/boot/multios/grub2_style.cfg`)

```bash
# MultiOS GRUB2-Style Boot Configuration
# This configuration demonstrates standard GRUB2 syntax

# Global timeout (in seconds)
timeout=10

# Default boot entry (can be entry number or title)
default=0

# Font and theme (optional)
# font /boot/grub/themes/starfield/theme.txt

# Boot menu colors
# set menu_color_normal=white/black
# set menu_color_highlight=black/white

# Menuentry for normal MultiOS boot
title MultiOS - Normal Boot
    linux /boot/multios/kernel
    initrd /boot/multios/initrd.img
    options quiet loglevel=3 console=ttyS0

# Menuentry for debug mode
title MultiOS - Debug Mode  
    linux /boot/multios/kernel
    initrd /boot/multios/initrd.img
    options debug loglevel=8 console=ttyS0 maxcpus=1

# Menuentry for recovery mode
title MultiOS - Recovery Mode
    linux /boot/multios/recovery-kernel
    initrd /boot/multios/recovery-initrd.img
    options init=/bin/bash single rescue ro

# Menuentry for safe mode
title MultiOS - Safe Mode
    linux /boot/multios/kernel
    options safe_mode no_drivers no_services console=ttyS0

# Menuentry for memory testing
title Memory Test (MemTest86+)
    linux /boot/multios/memtest86plus
    options memtest verbose

# Menuentry for network boot
title MultiOS - Network Boot
    linux /boot/multios/kernel
    initrd /boot/multios/initrd.img
    options netboot console=ttyS0

# Menuentry for educational demo
title MultiOS - Educational Demo
    linux /boot/multios/demo-kernel
    initrd /boot/multios/demo-initrd.img
    options demo_mode interactive console=ttyAMA0 loglevel=6

# Menuentry with fallback options
title MultiOS - With Fallback
    linux /boot/multios/kernel
    initrd /boot/multios/initrd.img
    options quiet loglevel=3
    fallback /boot/multios/kernel.fallback

# Menuentry for firmware setup
title Firmware Setup (UEFI/BIOS)
    linux /boot/multios/firmware-setup
    options setup_mode

# Boot to UEFI Shell
title UEFI Shell
    linux /boot/multios/uefi-shell

# Shutdown option
title Shutdown
    linux /boot/multios/shutdown

# Reboot option  
title Reboot
    linux /boot/multios/reboot
```

### 2. systemd-boot Format (`/boot/multios/systemd_boot_style.conf`)

```bash
# MultiOS systemd-boot Configuration
# This configuration follows systemd-boot/bootctl conventions

# Global timeout in seconds
timeout 10

# Default entry (title name)
default multios-normal

# Editor mode (allow kernel parameter editing)
# editor 1

# Console mode settings
# console-mode keep

# Auto-entries for installed kernels
auto-entry auto

# MultiOS Normal Boot
title MultiOS - Normal Boot
    linux /boot/multios/vmlinuz
    initrd /boot/multios/initrd.img
    options quiet loglevel=3 console=ttyS0

# MultiOS Debug Mode
title MultiOS - Debug Mode
    linux /boot/multios/vmlinuz
    initrd /boot/multios/initrd.img
    options debug loglevel=8 console=ttyS0 maxcpus=1

# MultiOS Recovery
title MultiOS - Recovery Mode
    linux /boot/multios/recovery-vmlinuz
    initrd /boot/multios/recovery-initrd.img
    options init=/bin/bash single rescue ro

# MultiOS Safe Mode
title MultiOS - Safe Mode
    linux /boot/multios/vmlinuz
    options safe_mode no_drivers no_services

# Memory Test
title Memory Test
    linux /boot/multios/memtest86plus

# Educational Demo
title MultiOS - Educational Demo
    linux /boot/multios/demo-vmlinuz
    initrd /boot/multios/demo-initrd.img
    options demo_mode interactive console=ttyAMA0

# Update Entry
title Update MultiOS
    linux /boot/multios/update-kernel
    initrd /boot/multios/update-initrd.img
    options update_mode

# UEFI Firmware Settings
title Firmware Settings
    efi /boot/efi/EFI/BOOT/firmware.efi

# UEFI Shell
title UEFI Shell
    efi /boot/efi/EFI/shellx64.efi

# Reboot to Firmware
title Reboot to Firmware
    efi /boot/efi/EFI/BOOT/reboot.efi
```

### 3. Custom JSON Format (`/boot/multios/json_style.cfg`)

```json
{
  "boot_config": {
    "version": "1.0",
    "format": "json",
    "timeout": 10,
    "default_entry": "multios-normal",
    "auto_boot": true,
    "serial_console": true,
    "graphic_mode": true
  },
  "global_options": {
    "loglevel": "info",
    "timezone": "UTC",
    "keyboard_layout": "us"
  },
  "boot_entries": [
    {
      "id": "multios-normal",
      "title": "MultiOS - Normal Boot",
      "description": "Standard MultiOS installation with default settings",
      "type": "linux",
      "kernel": "/boot/multios/kernel",
      "initrd": "/boot/multios/initrd.img",
      "options": [
        "quiet",
        "loglevel=3",
        "console=ttyS0"
      ],
      "fallback_options": [
        "emergency",
        "single"
      ],
      "timeout": null,
      "serial_console": true,
      "debug_mode": false,
      "recovery_mode": false,
      "memory_test": false,
      "network_boot": false,
      "secure_boot": false,
      "priority": 1,
      "is_default": true
    },
    {
      "id": "multios-debug",
      "title": "MultiOS - Debug Mode",
      "description": "MultiOS with detailed debug output and logging",
      "type": "linux",
      "kernel": "/boot/multios/kernel",
      "initrd": "/boot/multios/debug-initrd.img",
      "options": [
        "debug",
        "loglevel=8",
        "console=ttyS0",
        "maxcpus=1",
        "earlyprintk=serial"
      ],
      "fallback_options": [
        "debug",
        "console=ttyAMA0"
      ],
      "timeout": null,
      "serial_console": true,
      "debug_mode": true,
      "recovery_mode": false,
      "memory_test": false,
      "network_boot": false,
      "secure_boot": false,
      "priority": 2,
      "is_default": false
    },
    {
      "id": "multios-recovery",
      "title": "MultiOS - Recovery Mode",
      "description": "Recovery mode for system repair and maintenance",
      "type": "linux",
      "kernel": "/boot/multios/recovery-kernel",
      "initrd": "/boot/multios/recovery-initrd.img",
      "options": [
        "init=/bin/bash",
        "single",
        "rescue",
        "ro"
      ],
      "fallback_options": [
        "init=/system/bin/sh",
        "rescue"
      ],
      "timeout": null,
      "serial_console": true,
      "debug_mode": false,
      "recovery_mode": true,
      "memory_test": true,
      "network_boot": false,
      "secure_boot": false,
      "priority": 3,
      "is_default": false
    },
    {
      "id": "multios-safe",
      "title": "MultiOS - Safe Mode",
      "description": "Safe mode with minimal drivers and services",
      "type": "linux",
      "kernel": "/boot/multios/kernel",
      "initrd": null,
      "options": [
        "safe_mode",
        "no_drivers",
        "no_services",
        "console=ttyS0"
      ],
      "fallback_options": [
        "single",
        "no_network"
      ],
      "timeout": null,
      "serial_console": true,
      "debug_mode": false,
      "recovery_mode": false,
      "memory_test": false,
      "network_boot": false,
      "secure_boot": false,
      "priority": 4,
      "is_default": false
    },
    {
      "id": "memory-test",
      "title": "Memory Test Suite",
      "description": "Comprehensive memory testing (MemTest86+ compatible)",
      "type": "linux",
      "kernel": "/boot/multios/memtest86plus",
      "initrd": null,
      "options": [
        "memtest",
        "verbose",
        "all_cpus"
      ],
      "fallback_options": [
        "memtest",
        "standard"
      ],
      "timeout": 0,
      "serial_console": true,
      "debug_mode": false,
      "recovery_mode": false,
      "memory_test": true,
      "network_boot": false,
      "secure_boot": false,
      "priority": 10,
      "is_default": false
    },
    {
      "id": "educational-demo",
      "title": "MultiOS - Educational Demo",
      "description": "Interactive educational demonstration",
      "type": "linux",
      "kernel": "/boot/multios/demo-kernel",
      "initrd": "/boot/multios/demo-initrd.img",
      "options": [
        "demo_mode",
        "interactive",
        "console=ttyAMA0",
        "loglevel=6"
      ],
      "fallback_options": [
        "demo_mode",
        "console=ttyS0"
      ],
      "timeout": null,
      "serial_console": true,
      "debug_mode": true,
      "recovery_mode": false,
      "memory_test": false,
      "network_boot": false,
      "secure_boot": false,
      "priority": 5,
      "is_default": false
    },
    {
      "id": "network-boot",
      "title": "MultiOS - Network Boot",
      "description": "Network-based boot via PXE/HTTP",
      "type": "linux",
      "kernel": "/boot/multios/kernel",
      "initrd": "/boot/multios/initrd.img",
      "options": [
        "netboot",
        "console=ttyS0",
        "ip=dhcp"
      ],
      "fallback_options": [
        "netboot",
        "ip=static"
      ],
      "timeout": null,
      "serial_console": true,
      "debug_mode": false,
      "recovery_mode": false,
      "memory_test": false,
      "network_boot": true,
      "secure_boot": false,
      "priority": 20,
      "is_default": false
    },
    {
      "id": "firmware-setup",
      "title": "Firmware Setup",
      "description": "Enter UEFI/BIOS firmware configuration",
      "type": "firmware",
      "kernel": null,
      "initrd": null,
      "options": [
        "firmware_setup"
      ],
      "fallback_options": [],
      "timeout": 0,
      "serial_console": false,
      "debug_mode": false,
      "recovery_mode": false,
      "memory_test": false,
      "network_boot": false,
      "secure_boot": false,
      "priority": 100,
      "is_default": false
    }
  ],
  "environment": {
    "architecture": "x86_64",
    "boot_mode": "uefi",
    "secure_boot": false,
    "tpm_support": true
  },
  "hardware_detection": {
    "cpu": {
      "vendor": "Intel",
      "model": "Core i7",
      "cores": 8,
      "features": ["sse4", "avx2"]
    },
    "memory": {
      "total_gb": 16,
      "type": "DDR4",
      "speed": "3200"
    },
    "storage": [
      {
        "type": "nvme",
        "size_gb": 512,
        "bootable": true
      },
      {
        "type": "sata",
        "size_gb": 1024,
        "bootable": false
      }
    ],
    "network": [
      {
        "type": "ethernet",
        "speed_gbps": 1,
        "bootable": true
      },
      {
        "type": "wifi",
        "speed_mbps": 300,
        "bootable": false
      }
    ]
  }
}
```

### 4. Educational Lab Configuration (`/boot/multios/edu-lab.cfg`)

```bash
# MultiOS Educational Lab Configuration
# This configuration is designed for classroom and learning environments

# Extended timeout for learning and experimentation
timeout=60

# Default to debug mode for educational purposes
default=multios-debug

# Title and theme settings
title MultiOS Educational Laboratory
subtitle Explore Operating System Concepts

# Educational kernel with enhanced features
title MultiOS - Debug Mode (Recommended for Labs)
    linux /boot/multios/kernel
    initrd /boot/multios/initrd.img
    options debug loglevel=8 console=ttyS0 maxcpus=1 earlyprintk=serial
    options interactive_demo=True
    serial_console

# Standard installation for comparison
title MultiOS - Normal Mode
    linux /boot/multios/kernel
    initrd /boot/multios/initrd.img
    options quiet loglevel=3 console=ttyS0

# Single-user mode for system administration
title MultiOS - Single User Mode
    linux /boot/multios/kernel
    initrd /boot/multios/initrd.img
    options single console=ttyS0

# Recovery mode for troubleshooting
title MultiOS - Recovery Mode
    linux /boot/multios/recovery-kernel
    initrd /boot/multios/recovery-initrd.img
    options init=/bin/bash single rescue ro

# Kernel development mode
title MultiOS - Kernel Development
    linux /boot/multios/dev-kernel
    initrd /boot/multios/dev-initrd.img
    options development_mode debug loglevel=9 console=ttyS0

# Driver development mode
title MultiOS - Driver Development
    linux /boot/multios/kernel
    initrd /boot/multios/dev-initrd.img
    options driver_dev debug loglevel=8 console=ttyS0

# Network programming lab
title MultiOS - Network Programming Lab
    linux /boot/multios/kernel
    initrd /boot/multios/network-initrd.img
    options network_lab debug console=ttyS0 ip=dhcp

# File system lab
title MultiOS - File System Lab
    linux /boot/multios/kernel
    initrd /boot/multios/fs-initrd.img
    options fs_lab debug console=ttyS0

# Memory management lab
title MultiOS - Memory Management Lab
    linux /boot/multios/kernel
    initrd /boot/multios/mm-initrd.img
    options mm_lab debug console=ttyS0

# Process scheduling lab
title MultiOS - Process Scheduling Lab
    linux /boot/multios/kernel
    initrd /boot/multios/sched-initrd.img
    options sched_lab debug console=ttyS0

# Memory test before any lab
title Memory Test (Required for Labs)
    linux /boot/multios/memtest86plus
    options memtest verbose all_cpus

# Shell access for custom experiments
title Custom Shell
    linux /boot/multios/shell
    initrd /boot/multios/shell-initrd.img
    options shell debug

# Boot from different architectures (if available)
title MultiOS - ARM64 Mode (if available)
    linux /boot/multios-arm64/kernel
    initrd /boot/multios-arm64/initrd.img
    options debug console=ttyAMA0

title MultiOS - RISC-V Mode (if available)
    linux /boot/multios-riscv/kernel
    initrd /boot/multios-riscv/initrd.img
    options debug console=ttytyS0
```

### 5. Embedded System Configuration (`/boot/multios/embedded.cfg`)

```bash
# MultiOS Embedded System Configuration
# Minimal configuration for embedded/IoT devices

# Fast boot with minimal timeout
timeout=3

# Single boot option for embedded systems
default=0

# Primary boot (only option)
title MultiOS - Embedded System
    linux /boot/multios/kernel
    initrd /boot/multios/initrd.img
    options quiet console=ttyAMA0
    serial_console

# Factory reset option (if needed)
title MultiOS - Factory Reset
    linux /boot/multios/kernel
    initrd /boot/multios/reset-initrd.img
    options factory_reset console=ttyAMA0

# Boot to update mode
title MultiOS - Update Mode
    linux /boot/multios/kernel
    initrd /boot/multios/update-initrd.img
    options update_mode console=ttyAMA0
```

## Boot Parameters Reference

### Console Parameters
- `console=<device>` - Console device (ttyS0, ttyAMA0, etc.)
- `quiet` - Reduce boot messages
- `debug` - Enable debug output
- `loglevel=<0-9>` - Set console log level (0=emergency, 7=debug)
- `earlyprintk=<device>` - Early kernel messages

### System Parameters
- `init=<path>` - Init process path
- `root=<device>` - Root filesystem device
- `rootfstype=<type>` - Root filesystem type
- `ro` - Mount root filesystem read-only
- `rw` - Mount root filesystem read-write

### Debug Parameters
- `debug` - Enable kernel debug mode
- `ignore_loglevel` - Ignore loglevel settings
- `no_console_suspend` - Keep console active during suspend
- `no_sleep` - Prevent CPU sleep states

### Recovery Parameters
- `single` - Single-user mode
- `rescue` - Rescue mode
- `emergency` - Emergency mode
- `init=/bin/bash` - Custom init shell

### Safe Mode Parameters
- `safe_mode` - Safe boot mode
- `no_drivers` - Disable driver loading
- `no_services` - Disable service startup
- `single` - Single user mode

### Memory Parameters
- `memtest` - Run memory test
- `no_console_suspend` - Console never suspends
- `disable_mtrr_cleanup` - Disable MTRR cleanup
- `nopat` - Disable PAT (Page Attribute Table)

### Network Parameters
- `netboot` - Enable network boot
- `ip=dhcp` - Use DHCP for IP configuration
- `ip=<address>` - Static IP address
- `netdev=<interface>` - Network interface

### Hardware Parameters
- `maxcpus=<number>` - Maximum number of CPUs
- `nosmp` - Disable SMP
- `noapic` - Disable APIC
- `acpi=off` - Disable ACPI

### Educational Parameters
- `demo_mode` - Enable demo features
- `interactive_demo` - Interactive demonstration
- `development_mode` - Development environment
- `driver_dev` - Driver development mode
- `network_lab` - Network programming lab
- `fs_lab` - File system lab
- `mm_lab` - Memory management lab
- `sched_lab` - Process scheduling lab

### Architecture-Specific Parameters

#### ARM64
- `arm64.nobt` - Disable ARM64 BTI (Branch Target Identification)
- `arm64.nopac` - Disable ARM64 PAC (Pointer Authentication)

#### RISC-V
- `riscv.firmware=<type>` - Firmware type (opensbi, bbl)
- `riscv.nommu` - Disable MMU

## Configuration File Locations

The bootloader searches for configuration files in the following order:

1. `/boot/multios/<architecture>-specific.cfg`
2. `/boot/multios/boot.cfg`
3. `/boot/multios/grub.cfg`
4. `/boot/multios/grub2.cfg`
5. `/boot/grub/grub.cfg`
6. `/boot/loader/entries/multios.conf`
7. `/boot/efi/EFI/multios/boot.cfg`

## Boot Priority and Device Discovery

### Device Priority Order (by architecture)

#### x86_64 (UEFI/BIOS)
1. Primary SATA/NVMe drive
2. Secondary SATA/NVMe drive  
3. USB flash drive
4. CD/DVD drive (legacy BIOS only)
5. Network boot (PXE)

#### ARM64
1. SD Card (mmcblk0)
2. eMMC storage (mmcblk1)
3. USB storage
4. Network boot

#### RISC-V
1. SPI Flash (mtd0)
2. eMMC storage (mmcblk0)
3. SD Card (mmcblk1)
4. USB storage
5. Network boot

## Security Considerations

### Secure Boot
- Secure boot requires signed boot images
- Configuration files must be signed
- Boot menu may be disabled in secure boot mode
- Only trusted boot entries are shown

### Boot Integrity
- Boot images should include integrity checksums
- Configuration files should be protected from modification
- Boot logs should be tamper-evident

### Network Security
- Network boot should use authenticated protocols
- DHCP should provide secure configurations
- HTTP boot should use HTTPS when possible

This comprehensive collection of configuration examples demonstrates all the features and flexibility of the MultiOS multi-stage boot system.