# Multi-Stage Boot Implementation Documentation

## Overview

This document describes the comprehensive multi-stage boot support implementation for the MultiOS bootloader. The system provides a flexible, configurable boot environment supporting multiple architectures (x86_64, ARM64, RISC-V) and various boot configurations.

## Architecture

The multi-stage boot system is built around several core components:

```
┌─────────────────────────────────────────────────────────────────┐
│                        Firmware Layer                           │
│                  (UEFI/BIOS/OpenSBI)                           │
└─────────────────────────────┬───────────────────────────────────┘
                              │
┌─────────────────────────────▼───────────────────────────────────┐
│                    Bootloader Entry                            │
│                  (boot_start function)                         │
└─────────────────────────────┬───────────────────────────────────┘
                              │
┌─────────────────────────────▼───────────────────────────────────┐
│                   Multi-Stage Boot Manager                     │
│                 (multi_stage_boot module)                      │
└─────┬─────────────────────┬─────────────────────┬───────────────┘
      │                     │                     │
┌─────▼─────┐        ┌──────▼──────┐       ┌─────▼─────┐
│   Stage   │        │    Stage    │       │   Stage   │
│Device     │        │Config       │       │Boot Menu  │
│Detection  │        │Parsing      │       │Selection  │
└─────┬─────┘        └──────┬──────┘       └─────┬─────┘
      │                     │                     │
┌─────▼─────┐        ┌──────▼──────┐       ┌─────▼─────┐
│   Stage   │        │    Stage    │       │   Stage   │
│Kernel     │        │  Boot Menu  │       │Kernel     │
│Loading    │        │  Display    │       │Handoff    │
└───────────┘        └─────────────┘       └───────────┘
```

## Boot Stages

### Stage 1: Firmware/BIOS/UEFI
- **Responsibility**: Platform initialization and boot media discovery
- **Actions**:
  - POST (Power-On Self-Test)
  - Hardware initialization
  - Boot device enumeration
  - Memory detection and configuration
- **UEFI specific**:
  - System Table initialization
  - Boot Services available
  - Memory map acquisition
- **Legacy BIOS specific**:
  - Real mode operation
  - INT 13h services for disk access

### Stage 2: Bootloader Initialization
- **File**: `multi_stage_boot.rs`
- **Function**: `initialize_bootloader()`
- **Actions**:
  - Initialize global boot state
  - Set up serial console logging
  - Initialize memory management subsystems
  - Prepare error handling infrastructure

### Stage 3: Device Detection & Configuration Parsing
- **Files**: 
  - `device_detection.rs`
  - `config_parser.rs`
- **Actions**:
  - **Device Detection**:
    - Detect bootable devices based on architecture
    - Enumerate USB, SATA, SD card, network boot devices
    - Sort devices by boot priority
    - Verify device accessibility
  - **Configuration Parsing**:
    - Parse boot configuration files (GRUB2, systemd-boot, custom formats)
    - Extract boot entries with parameters
    - Validate configuration syntax
    - Handle fallback configurations

### Stage 4: Boot Menu Display & Selection
- **File**: `boot_menu.rs`
- **Actions**:
  - Initialize boot menu with detected entries
  - Display interactive boot menu (if enabled)
  - Handle timeout-based auto-selection
  - Support multiple boot modes:
    - Normal boot
    - Debug mode (verbose logging)
    - Recovery mode (system repair)
    - Safe mode (minimal drivers)
    - Memory testing

### Stage 5: Kernel Loading
- **File**: `kernel_loader.rs`
- **Actions**:
  - Load kernel binary from selected device
  - Load initrd/initramfs if specified
  - Verify kernel image integrity
  - Set up kernel command line
  - Prepare boot information structure

### Stage 6: Kernel Handoff
- **Actions**:
  - Exit boot services (UEFI)
  - Jump to kernel entry point
  - Pass boot information
  - Transfer control to kernel

## Boot Configuration

### Configuration File Formats

The system supports multiple boot configuration formats:

#### GRUB2 Format
```bash
timeout=10
default=multios-normal

title MultiOS Normal
  linux /boot/multios/kernel
  options quiet loglevel=3 console=ttyS0

title MultiOS Debug
  linux /boot/multios/kernel
  options debug loglevel=8 console=ttyS0

title MultiOS Recovery
  linux /boot/multios/recovery
  initrd /boot/multios/recovery/initrd
  options init=/bin/bash single
  recovery_mode
```

#### systemd-boot Format
```bash
timeout 10
default multios-normal

title MultiOS Normal
  linux /boot/multios/kernel
  options quiet loglevel=3

title MultiOS Debug
  linux /boot/multios/kernel
  options debug loglevel=8

title MultiOS Recovery
  linux /boot/multios/recovery
  initrd /boot/multios/recovery/initrd
  options init=/bin/bash single
  recovery_mode
```

#### Custom JSON Format
```json
{
  "timeout": 10,
  "default_entry": "multios-normal",
  "entries": [
    {
      "title": "MultiOS Normal",
      "linux": "/boot/multios/kernel",
      "options": ["quiet", "loglevel=3"],
      "serial_console": true
    },
    {
      "title": "MultiOS Debug",
      "linux": "/boot/multios/kernel", 
      "options": ["debug", "loglevel=8"],
      "debug_mode": true
    },
    {
      "title": "MultiOS Recovery",
      "linux": "/boot/multios/recovery",
      "initrd": "/boot/multios/recovery/initrd",
      "options": ["init=/bin/bash", "single"],
      "recovery_mode": true
    }
  ]
}
```

### Boot Parameters

Supported boot parameters include:

#### Console Parameters
- `console=<device>` - Set console device (e.g., `ttyS0`, `ttyAMA0`)
- `quiet` - Reduce boot messages
- `debug` - Enable debug output
- `loglevel=<0-9>` - Set console log level

#### System Parameters
- `init=<path>` - Specify init program
- `root=<device>` - Root filesystem device
- `single` - Single-user mode
- `rescue` - Rescue mode

#### Device Parameters
- `memtest` - Run memory test
- `safe_mode` - Safe mode with minimal drivers
- `no_drivers` - Disable driver loading
- `no_services` - Disable service startup

## Boot Modes

### Normal Mode
- Standard system boot with default configuration
- All drivers and services loaded
- Verbosity level 3 (informational messages)

### Debug Mode
- Enhanced logging and debugging information
- Verbosity level 8 (debug messages)
- Serial console output enabled
- Detailed boot process logging

### Recovery Mode
- Minimal boot for system repair
- Single-user mode
- Manual init process (`/bin/bash`)
- Memory testing enabled

### Safe Mode
- Minimal driver loading
- No unnecessary services
- Fallback to basic functionality
- Used when normal boot fails

## Architecture-Specific Implementation

### x86_64 Support

#### Device Detection
- **UEFI**: Uses System Table and Boot Services
- **Legacy BIOS**: Uses INT 13h services
- **Devices**: SATA, USB, CD-ROM, Network (PXE)

#### Boot Process
1. Firmware loads bootloader from boot device
2. Bootloader detects devices using UEFI protocols or BIOS INT calls
3. Configuration parsed from `/boot/grub/grub.cfg` or `/boot/multios/boot.cfg`
4. Boot menu displayed with timeout
5. Kernel loaded via UEFI LoadImage/StartImage or BIOS disk I/O
6. Handoff via UEFI ExitBootServices or direct jump

### ARM64 Support

#### Device Detection
- **Firmware**: EDK2/UEFI or U-Boot
- **Devices**: SD Card, eMMC, USB
- **Device Tree**: Parsed from DTB passed by firmware

#### Boot Process
1. Firmware (U-Boot/EDK2) loads bootloader
2. Device Tree (DTB) parsed for hardware configuration
3. Bootable devices enumerated (SD, eMMC, USB)
4. Configuration from `/boot/multios/boot.cfg`
5. Kernel loaded with DTB pointer
6. Handoff to kernel with DTB address in register

### RISC-V Support

#### Device Detection
- **Firmware**: OpenSBI with FW_DYNAMIC
- **Devices**: SPI Flash, eMMC, SD Card
- **Device Tree**: Passed in register a1
- **SBI**: Supervisor Binary Interface for firmware calls

#### Boot Process
1. ZSBL (Zero Stage Bootloader) starts OpenSBI
2. OpenSBI initializes M-mode environment
3. Device detection via firmware interfaces
4. DTB passed in register a1, hart ID in a0
5. Configuration parsed by bootloader
6. Kernel loaded and executed in S-mode

## Boot Menu System

### Features
- **Timeout-based selection**: Auto-boot after configured timeout
- **Keyboard navigation**: Arrow keys and numeric selection
- **Visual indicators**: Default entry marking, recovery mode indication
- **Multiple formats**: Support for different configuration syntaxes
- **Fallback handling**: Graceful degradation when menu fails

### Entry Types
- **Normal entries**: Standard boot options
- **Recovery entries**: Marked for system recovery
- **Debug entries**: Enhanced logging and debugging
- **Special entries**: Memory test, firmware setup, safe mode

### Configuration Options
```rust
pub struct BootMenuConfig {
    pub timeout_seconds: u8,          // Auto-select timeout
    pub enable_recovery_mode: bool,   // Enable recovery options
    pub enable_debug_mode: bool,      // Enable debug options  
    pub enable_normal_mode: bool,     // Enable normal options
    pub default_boot_mode: BootMenuSelection, // Default selection
}
```

## Error Handling and Recovery

### Error Detection
- Each boot stage reports success/failure status
- Comprehensive error logging at each stage
- Graceful degradation when non-critical components fail

### Recovery Mechanisms
1. **Device fallback**: Try alternative boot devices if primary fails
2. **Config fallback**: Use default configuration if parsing fails
3. **Menu fallback**: Default boot entry if menu interaction fails
4. **Kernel fallback**: Try recovery kernel if normal kernel fails

### Error Types
```rust
pub enum BootStageError {
    StageFailed(BootStage),      // Specific stage failure
    ConfigurationError,          // Configuration parsing error
    DeviceDetectionFailed,       // Device enumeration error
    KernelNotFound,              // Kernel image not found
    BootMenuFailed,              // Menu interaction error
    InvalidBootConfig,           // Invalid boot configuration
}
```

## Performance Considerations

### Boot Speed Optimizations
- **Parallel device detection**: Scan multiple devices concurrently
- **Cache configurations**: Parse and cache configuration files
- **Skip menu when possible**: Direct boot for configured defaults
- **Efficient memory management**: Minimize memory allocations during boot

### Memory Usage
- **Stack-based parsing**: Avoid heap allocation during config parsing
- **Lazy initialization**: Initialize components only when needed
- **Memory mapping reuse**: Leverage firmware-provided memory maps
- **Boot heap management**: Dedicated memory allocation for boot process

## Testing and Debugging

### Boot Testing
- **Emulation testing**: QEMU for x86_64, ARM64, RISC-V
- **Hardware testing**: Real hardware validation
- **Configuration testing**: Various config file formats and syntax
- **Failure injection**: Test recovery mechanisms

### Debug Features
- **Serial console logging**: Detailed boot process logging
- **Boot stage logging**: Each stage reports status
- **Device enumeration logging**: List of detected devices
- **Configuration parsing logging**: Configuration file analysis
- **Error context preservation**: Detailed error information

### Debug Commands
- Boot device listing
- Configuration file display
- Boot parameter parsing
- Boot stage log display
- Manual boot entry selection

## Future Enhancements

### Planned Features
1. **Network boot**: PXE and HTTP boot support
2. **Secure boot**: UEFI Secure Boot integration
3. **Encrypted boot**: Support for encrypted boot images
4. **A/B updates**: Boot image versioning and rollback
5. **Remote management**: Network-based boot configuration
6. **Performance monitoring**: Boot time analysis and optimization

### Configuration Extensions
1. **Theme support**: Customizable boot menu appearance
2. **Internationalization**: Multi-language boot menu support
3. **Network configuration**: Dynamic boot configuration via network
4. **User profiles**: Personalized boot configurations
5. **Hardware profiles**: Architecture-specific configurations

## API Reference

### Main Entry Points

#### `boot_start_multi_stage()`
Enhanced multi-stage boot entry point with full functionality.

#### `boot_start_educational_lab()`
Educational lab configuration with extended timeout and debug features.

#### `boot_start_embedded()`
Embedded system configuration with minimal features and fast boot.

### Utility Functions

#### `get_best_boot_device()`
Get the highest priority available boot device.

#### `display_boot_devices()`
Detect and display all available boot devices.

#### `parse_boot_config_from_file(path)`
Parse boot configuration from file.

#### `get_boot_menu_entries_from_config(config)`
Convert configuration to boot menu entries.

### Configuration Types

#### `MultiStageBootConfig`
Complete multi-stage boot configuration structure.

#### `BootMenuConfig`
Boot menu specific configuration.

#### `ParsedBootConfig`
Parsed boot configuration with entries.

This documentation provides a comprehensive guide to the multi-stage boot implementation, enabling developers and educators to understand, modify, and extend the boot system as needed.