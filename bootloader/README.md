# MultiOS Bootloader

The MultiOS Bootloader is a comprehensive Rust-based bootloader supporting both UEFI and legacy BIOS boot methods for the MultiOS operating system. It provides a robust foundation for educational operating system development with proper error handling, logging, and memory management.

## Features

### Core Functionality
- **Dual Boot Support**: Both UEFI and legacy BIOS boot methods
- **Memory Management**: Comprehensive memory map detection and management
- **Error Handling**: Robust error handling with detailed logging
- **Serial Console**: Built-in serial console support for debugging
- **Modular Design**: Clean separation of concerns with module-based architecture

### Boot Methods
- **UEFI Boot**: Complete UEFI system table interaction and boot services management
- **Legacy BIOS**: Traditional BIOS interrupts and memory detection via INT 15h
- **Boot Mode Detection**: Automatic detection of boot method from firmware

### Memory Management
- **Memory Map Detection**: Automatic memory region detection and classification
- **Memory Validation**: Integrity checking for memory regions
- **Boot Information**: Structured boot info for kernel handoff
- **Memory Regions**: Support for various memory types (usable, reserved, kernel, etc.)

## Architecture

### Module Structure

```
bootloader/
├── src/
│   ├── lib.rs           # Main bootloader entry point and orchestration
│   ├── uefi.rs          # UEFI boot support
│   ├── legacy.rs        # Legacy BIOS boot support
│   ├── memory_map.rs    # Memory map detection and management
│   └── kernel_loader.rs # Kernel loading and boot information
```

### Core Components

#### 1. UEFI Support (`uefi.rs`)
- UEFI system table interaction
- Boot services management
- Memory map extraction from UEFI
- Framebuffer information extraction
- ACPI table detection
- Kernel loading via UEFI file system
- Boot services exit and kernel transition

#### 2. Legacy BIOS Support (`legacy.rs`)
- BIOS information detection via INT calls
- Memory map detection via INT 15h
- Boot device detection
- Video mode detection
- Disk device access (INT 13h)
- Kernel loading from boot devices

#### 3. Memory Map Management (`memory_map.rs`)
- Memory region structure and classification
- Boot info conversion to memory map
- Memory allocation and alignment
- Memory validation and overlap detection
- Memory statistics and reporting

#### 4. Kernel Loading (`kernel_loader.rs`)
- Kernel boot information structures
- ELF format validation
- Boot configuration management
- Kernel entry point determination
- Boot info buffer management

## Building

### Prerequisites
- Rust nightly toolchain
- x86_64 target support
- QEMU for testing (optional)

### Building for x86_64

```bash
# Build with UEFI support (default)
cargo build --release

# Build with legacy BIOS support
cargo build --release --features legacy

# Build with debug features
cargo build --release --features debug_mode

# Build with memory testing
cargo build --release --features memory_test
```

### Feature Flags

- `uefi` (default): Enable UEFI boot support
- `legacy`: Enable legacy BIOS boot support
- `logging` (default): Enable serial console logging
- `debug_mode`: Enable debug features and verbose logging
- `memory_test`: Enable memory testing during boot

### Cross-Compilation

The bootloader supports cross-compilation for different architectures:

```bash
# Install target
rustup target add x86_64-unknown-none

# Cross-compile
cargo build --target x86_64-unknown-none --release
```

## Usage

### Basic Boot Process

The bootloader automatically detects the boot method and proceeds:

1. **Initialization**: Set up serial console and logging
2. **Boot Detection**: Determine UEFI vs legacy BIOS boot
3. **Memory Map**: Detect and validate memory regions
4. **Kernel Load**: Load kernel from boot device
5. **Handoff**: Transfer control to kernel with boot info

### Boot Configuration

```rust
use multios_bootloader::{BootConfig, BootMode};

let config = BootConfig {
    mode: BootMode::UEFI,
    kernel_path: "/boot/multios/kernel",
    initrd_path: Some("/boot/initrd"),
    command_line: Some("quiet loglevel=3"),
    memory_test: true,
    serial_console: true,
    debug_mode: true,
    log_level: Level::Debug,
};
```

### Memory Map Usage

```rust
use multios_bootloader::memory_map::{MemoryMap, MemoryType};

let memory_map = MemoryMap::new();

// Add memory regions
memory_map.add_region(MemoryRegionInfo::new(
    PhysAddr::new(0x100000),
    1024 * 1024,
    MemoryType::Usable,
    MemoryFlags::READ | MemoryFlags::WRITE | MemoryFlags::AVAILABLE,
));

// Find allocation region
let region = memory_map.find_region(size, alignment);
```

## Boot Information

The bootloader creates a structured boot information block for the kernel containing:

- **Magic Number**: `0x2022_4D55_4B4E_494F` ("MINIKERNEL")
- **Kernel Entry Point**: Address to jump to for kernel startup
- **Memory Map**: Complete memory region information
- **Framebuffer**: Graphics mode information (if available)
- **ACPI Tables**: ACPI/UEFI table addresses
- **Command Line**: Kernel command line parameters
- **Boot Configuration**: Bootloader and system information

## Error Handling

The bootloader implements comprehensive error handling:

```rust
pub enum BootError {
    UefiNotSupported,
    LegacyNotSupported,
    KernelNotFound,
    MemoryMapError,
    InvalidKernelFormat,
    BootProcessError,
    SerialConsoleError,
    // ... additional error types
}
```

All errors are logged with detailed context, and the system halts with informative panic messages.

## Testing

### Unit Tests

```bash
cargo test
```

### Integration Testing

The bootloader can be tested using QEMU:

```bash
# Build test kernel
cargo build --release

# Test in QEMU
qemu-system-x86_64 -kernel target/release/multios-bootloader
```

### Boot Testing

The bootloader supports multiple testing scenarios:

1. **UEFI Testing**: Test with UEFI firmware
2. **Legacy BIOS Testing**: Test with traditional BIOS
3. **Memory Testing**: Validate memory detection
4. **Error Simulation**: Test error handling paths

## Debugging

### Serial Console

The bootloader outputs detailed logs via serial console (COM1/0x3F8):

```
MultiOS Bootloader v0.1.0 starting...
Bootloader features: UEFI=true, Legacy=true, Logging=true, Debug=true
UEFI boot detected
Initializing memory map...
Memory map: 16384 KB total, 8192 KB available, 1024 KB bootloader
Kernel loaded: 1048576 bytes
UEFI boot process completed, handing control to kernel
```

### Debug Mode

Enable debug mode for verbose output:

```toml
[features]
debug_mode = []
```

## Educational Value

This bootloader is designed for educational purposes and demonstrates:

1. **Low-Level Systems Programming**: Direct hardware interaction
2. **Boot Process Understanding**: Complete boot sequence implementation
3. **Memory Management**: Operating system memory management concepts
4. **Error Handling**: Robust system error handling
5. **Rust Safety**: Safe systems programming in Rust
6. **Cross-Platform Design**: Multi-architecture boot support

## Technical Specifications

### Supported Platforms
- **x86_64**: Full UEFI and legacy BIOS support
- **ARM64**: Future support (UEFI)
- **RISC-V**: Future support (OpenSBI)

### Memory Requirements
- **Minimum**: 640KB conventional memory
- **Recommended**: 1MB+ total memory
- **Optimal**: 4GB+ for full functionality

### Boot Devices
- **UEFI**: FAT32, FAT16 file systems
- **Legacy BIOS**: MBR, GPT partitioned disks
- **Network**: PXE boot (future)

### Security Features
- **Boot Validation**: Kernel format validation
- **Memory Protection**: Memory region isolation
- **Error Containment**: Fault isolation

## Contributing

The bootloader welcomes contributions for:

1. **Additional Architecture Support**: ARM64, RISC-V
2. **Boot Protocol Extensions**: PXE, network boot
3. **Security Enhancements**: Secure boot support
4. **Performance Improvements**: Boot speed optimization
5. **Educational Content**: Documentation and examples

## References

- [UEFI Specification](https://uefi.org/specs/UEFI/2.10/01_Introduction.html)
- [Rust Bootloader Crate](https://github.com/rust-osdev/bootloader)
- [x86_64 Architecture Manual](https://software.intel.com/content/www/us/en/develop/articles/intel-sdm.html)
- [MultiOS Technical Specifications](../multios_technical_specifications.md)

## License

This bootloader is licensed under MIT OR Apache-2.0.

---

For questions or contributions, please refer to the MultiOS project documentation and contribution guidelines.