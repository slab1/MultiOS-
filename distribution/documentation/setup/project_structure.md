# MultiOS Project Structure

## Overview

MultiOS is a hybrid microkernel operating system written in Rust, designed for portability across multiple architectures and device types. This document describes the project organization and architecture.

## Directory Structure

```
multios/
├── Cargo.toml              # Workspace configuration
├── kernel/                 # Core kernel components
│   ├── Cargo.toml         # Kernel crate configuration
│   └── src/
│       └── lib.rs         # Main kernel library
├── bootloader/             # Bootloader components
│   ├── Cargo.toml         # Bootloader crate configuration
│   └── src/
│       └── lib.rs         # Bootloader library
├── libraries/              # Shared libraries and services
│   ├── memory-manager/     # Memory management system
│   ├── scheduler/          # Process and thread scheduling
│   ├── device-drivers/     # Device driver framework
│   ├── ipc/               # Inter-process communication
│   └── filesystem/        # File system framework
├── tools/                  # Development and build tools
│   ├── build/             # Build automation scripts
│   ├── cross-compile/     # Cross-compilation utilities
│   ├── debug/             # Debugging and testing tools
│   └── qemu/              # QEMU configuration and scripts
├── tests/                  # Test suites and integration tests
├── docs/                   # Documentation
│   ├── setup/             # Setup and installation guides
│   ├── architecture/      # Architecture documentation
│   ├── development/       # Development guidelines
│   └── api/               # API documentation
```

## Architecture Overview

### Hybrid Microkernel Design

MultiOS follows a hybrid microkernel architecture that combines the best aspects of monolithic and microkernel designs:

- **Kernel**: Minimal core providing essential services
- **Libraries**: Reusable components for various OS functions
- **Drivers**: Device-specific implementations
- **Services**: User-space system services

### Core Components

#### 1. Kernel (`kernel/`)
The kernel provides the foundational services for MultiOS:

- **Memory Management**: Virtual and physical memory allocation
- **Process Scheduling**: Multi-threaded process scheduling
- **IPC**: Inter-process communication primitives
- **Device Interface**: Abstract device access layer
- **File System Interface**: VFS (Virtual File System) layer

#### 2. Bootloader (`bootloader/`)
The bootloader handles system initialization:

- **UEFI Support**: Modern UEFI boot protocol
- **Legacy BIOS**: Traditional BIOS boot support
- **Memory Map**: Physical memory layout detection
- **Kernel Loading**: Boot-time kernel image loading

#### 3. Memory Manager (`libraries/memory-manager/`)
Dedicated memory management system:

- **Physical Memory**: Page-level memory allocation
- **Virtual Memory**: Address space management
- **Memory Protection**: Hardware-enforced memory protection
- **Memory Optimization**: Efficient allocation algorithms

#### 4. Scheduler (`libraries/scheduler/`)
Process and thread scheduling system:

- **Multi-threading**: Thread creation and management
- **Priority Scheduling**: Priority-based scheduling
- **Real-time Support**: Optional real-time scheduling
- **Multi-core Support**: SMP (Symmetric Multi-Processing)

#### 5. Device Drivers (`libraries/device-drivers/`)
Unified device driver framework:

- **Driver Registry**: Centralized driver management
- **Hardware Abstraction**: Common device interfaces
- **Hot-pluggable**: Dynamic driver loading/unloading
- **Multiple Architectures**: Cross-platform support

#### 6. IPC System (`libraries/ipc/`)
Inter-process communication mechanisms:

- **Message Passing**: Channel-based communication
- **Shared Memory**: Shared memory regions
- **Synchronization**: Semaphores and mutexes
- **Named Resources**: Named channels and shared memory

#### 7. File System (`libraries/filesystem/`)
File system framework and implementations:

- **VFS Layer**: Virtual File System abstraction
- **Multiple FS Types**: Support for various file systems
- **Mount Points**: Unified mount interface
- **File Operations**: Standard file and directory operations

## Workspace Configuration

The `Cargo.toml` file in the root directory defines the workspace configuration:

### Workspace Members
- `kernel/` - Core kernel
- `bootloader/` - Bootloader
- `libraries/*/` - All library crates
- `tools/*/` - Development tools

### Shared Dependencies
Common dependencies are centralized in the workspace configuration:
- `spin` - Lock-free primitives
- `bitflags` - Flag management
- `log` - Logging framework
- `x86_64` - x86-64 architecture support
- `bootloader` - Bootloader framework

### Build Profiles
- **dev**: Development profile with optimization level 1
- **release**: Release profile with size optimization
- **panic behavior**: Abort on panic for smaller binaries

## Development Guidelines

### Code Organization
- Each component has its own `src/lib.rs` file
- Public interfaces are clearly defined
- Error handling uses Result types
- No external dependencies without justification

### Testing Strategy
- Unit tests in each crate
- Integration tests in `tests/` directory
- Architecture-specific tests
- Bootloader and kernel tests

### Documentation Requirements
- API documentation in code comments
- Architecture documentation in `docs/architecture/`
- Development guides in `docs/development/`
- Setup instructions in `docs/setup/`

## Build System

### Prerequisites
- Rust 1.70+ toolchain
- Cross-compilation target support
- QEMU for testing (x86_64-unknown-none target)
- BOOTSYS or similar for bootloader development

### Building
```bash
# Build all components
cargo build

# Build specific component
cargo build -p multios-kernel
cargo build -p multios-bootloader

# Build for release
cargo build --release
```

### Testing
```bash
# Run all tests
cargo test

# Run integration tests
cargo test --test integration

# Run specific component tests
cargo test -p multios-memory-manager
```

### Cross-Compilation
Cross-compilation targets are configured in `tools/cross-compile/`:
- `x86_64-unknown-none` - Bare metal x86-64
- `aarch64-unknown-none` - Bare metal ARM64
- `riscv64gc-unknown-none` - Bare metal RISC-V

## Architecture Support

### Currently Supported
- **x86_64**: Full support including UEFI and legacy boot
- **AArch64**: ARM64 support (planned)
- **RISC-V64**: RISC-V 64-bit support (planned)

### Cross-Platform Design
The architecture is designed for portability:
- Architecture-specific code is isolated
- Common abstractions are provided
- Hardware details are abstracted
- Driver interfaces are platform-agnostic

## Future Extensions

### Planned Features
- **Virtualization**: Hypervisor support
- **Containers**: Lightweight process isolation
- **Networking**: Network protocol stack
- **Graphics**: Graphics subsystem and windowing
- **USB**: USB device support
- **Real-time**: Real-time scheduling extensions

### Architecture Expansion
- **ARM**: Full ARM architecture support
- **PowerPC**: PowerPC architecture support
- **MIPS**: MIPS architecture support
- **RISC-V**: Complete RISC-V ecosystem support

## Getting Started

### Development Environment Setup
1. Install Rust toolchain
2. Clone the repository
3. Install cross-compilation targets
4. Set up QEMU for testing
5. Build the project
6. Run initial tests

### Contributing
1. Follow the coding standards
2. Add tests for new features
3. Update documentation
4. Submit pull requests
5. Review architecture changes

## Conclusion

The MultiOS project structure supports a modular, maintainable approach to operating system development. The hybrid microkernel architecture provides a balance between performance and modularity, while the Rust implementation ensures memory safety and performance.

The workspace configuration enables efficient builds and dependency management, while the comprehensive documentation supports both new developers and system architects.