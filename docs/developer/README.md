# MultiOS Developer Guide

Welcome to the MultiOS Developer Guide! This comprehensive guide will help you set up a development environment, understand the codebase, and start contributing to MultiOS.

## Table of Contents

1. [Development Environment Setup](#development-environment-setup)
2. [Codebase Overview](#codebase-overview)
3. [Building MultiOS](#building-multios)
4. [Project Structure](#project-structure)
5. [Coding Standards](#coding-standards)
6. [Testing Framework](#testing-framework)
7. [Debugging Guide](#debugging-guide)
8. [Contributing Guidelines](#contributing-guidelines)
9. [Release Process](#release-process)
10. [Documentation](#documentation)

## Development Environment Setup

### Prerequisites

Before you start developing MultiOS, ensure you have:

#### Required Software

**Rust Toolchain (1.70+):**
```bash
# Install rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install additional targets
rustup target add x86_64-unknown-none-elf
rustup target add aarch64-unknown-none-elf
rustup target add riscv64gc-unknown-none-elf

# Install additional tools
cargo install cargo-audit cargo-tarpaulin cross
```

**Build Tools:**
```bash
# Ubuntu/Debian
sudo apt-get install -y \
    build-essential \
    qemu-system-x86 qemu-system-aarch64 qemu-system-riscv64 \
    gcc-aarch64-linux-gnu gcc-riscv64-linux-gnu \
    doxygen graphviz git cmake ninja-build

# macOS
brew install qemu cmake ninja git

# Additional Rust tools
cargo install cargo-edit cargo-outdated cargo-watch
```

**Optional but Recommended:**
- **Visual Studio Code** with Rust extensions
- **Git hooks** for code quality checks
- **Docker** for consistent development environment

### IDE Configuration

#### Visual Studio Code

**Required Extensions:**
```json
{
  "recommendations": [
    "rust-lang.rust-analyzer",
    "ms-vscode.vscode-json",
    "redhat.vscode-yaml",
    "ms-vscode.cpptools",
    "github.vscode-pull-request-github",
    "eamodio.gitlens"
  ]
}
```

**Recommended Settings** (`.vscode/settings.json`):
```json
{
  "rust-analyzer.checkOnSave.command": "clippy",
  "rust-analyzer.cargo.features": "all",
  "rust-analyzer.procMacro.ignored": {},
  "editor.formatOnSave": true,
  "editor.codeActionsOnSave": {
    "source.fixAll": true
  },
  "files.exclude": {
    "**/target": true,
    "**/.git": false
  }
}
```

**Launch Configuration** (`.vscode/launch.json`):
```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "name": "Debug x86_64",
      "type": "cppdbg",
      "request": "launch",
      "program": "${workspaceFolder}/target/x86_64-unknown-none-elf/debug/multios",
      "args": [],
      "stopAtEntry": false,
      "cwd": "${workspaceFolder}",
      "environment": [],
      "externalConsole": false,
      "MIMode": "gdb",
      "miDebuggerPath": "gdb-multiarch",
      "miDebuggerArgs": "-x .gdb/x86_64.gdb",
      "setupCommands": [
        {
          "description": "Enable pretty printing",
          "text": "-enable-pretty-printing",
          "ignoreFailures": true
        }
      ]
    }
  ]
}
```

**Tasks Configuration** (`.vscode/tasks.json`):
```json
{
  "version": "2.0.0",
  "tasks": [
    {
      "label": "Build x86_64",
      "type": "shell",
      "command": "cargo",
      "args": ["build", "--target", "x86_64-unknown-none-elf"],
      "group": "build",
      "presentation": {
        "echo": true,
        "reveal": "always",
        "focus": false,
        "panel": "shared"
      }
    },
    {
      "label": "Build All",
      "type": "shell",
      "command": "make",
      "args": ["build-all"],
      "group": "build"
    },
    {
      "label": "Run x86_64",
      "type": "shell",
      "command": "make",
      "args": ["run-x86_64"],
      "group": "build"
    },
    {
      "label": "Test All",
      "type": "shell",
      "command": "make",
      "args": ["test-all"],
      "group": "test"
    }
  ]
}
```

### Environment Configuration

**Shell Configuration** (add to `~/.bashrc` or `~/.zshrc`):
```bash
# MultiOS Development Environment
export MULTIOS_HOME="$HOME/multios"
export PATH="$MULTIOS_HOME/scripts:$PATH"

# Rust development
export RUST_BACKTRACE=1
export RUST_LOG=debug

# QEMU settings
export QEMU_AUDIO_DRV=none
export QEMU_LOG_LEVEL=info

# Build settings
export CARGO_INCREMENTAL=0
export RUSTFLAGS="-C debuginfo=2"
```

**Git Configuration:**
```bash
git config --global user.name "Your Name"
git config --global user.email "your.email@example.com"

# Useful aliases
git config --global alias.st status
git config --global alias.co checkout
git config --global alias.br branch
git config --global alias.ci commit
git config --global alias.lg "log --oneline --graph --decorate --all"

# Git hooks
git config core.hooksPath .githooks
```

### Docker Development Environment

Create a consistent development environment using Docker:

**Dockerfile:**
```dockerfile
FROM ubuntu:22.04

# Install system dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    qemu-system-x86 qemu-system-aarch64 qemu-system-riscv64 \
    gcc-aarch64-linux-gnu gcc-riscv64-linux-gnu \
    git curl vim htop \
    && rm -rf /var/lib/apt/lists/*

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Install Rust targets
RUN rustup target add x86_64-unknown-none-elf
RUN rustup target add aarch64-unknown-none-elf
RUN rustup target add riscv64gc-unknown-none-elf

# Install Rust tools
RUN cargo install cargo-audit cargo-tarpaulin cross cargo-edit cargo-watch

# Create workspace directory
WORKDIR /workspace

# Copy project files
COPY . .

# Build project
RUN make build-all

CMD ["make", "shell"]
```

**docker-compose.yml:**
```yaml
version: '3.8'
services:
  multios-dev:
    build: .
    container_name: multios-dev
    volumes:
      - .:/workspace
      - multios-cache:/root/.cargo
    working_dir: /workspace
    stdin_open: true
    tty: true

volumes:
  multios-cache:
```

## Codebase Overview

### Architecture Overview

MultiOS follows a hybrid microkernel architecture with the following components:

```
┌─────────────────────────────────────────────────────────────┐
│                    User Space Applications                  │
├─────────────────────────────────────────────────────────────┤
│                   System Call Interface                     │
├─────────────────────────────────────────────────────────────┤
│   ┌─────────────┐ ┌─────────────┐ ┌─────────────┐           │
│   │   Kernel    │ │  Services   │ │   Drivers   │           │
│   │   Core      │ │   Layer     │ │   Layer     │           │
│   └─────────────┘ └─────────────┘ └─────────────┘           │
├─────────────────────────────────────────────────────────────┤
│                 Hardware Abstraction Layer                  │
├─────────────────────────────────────────────────────────────┤
│                    Hardware Platforms                       │
│              (x86_64, ARM64, RISC-V)                        │
└─────────────────────────────────────────────────────────────┘
```

### Core Components

#### 1. Bootloader
- **Location**: `bootloader/`
- **Purpose**: System boot and initialization
- **Features**: Multi-stage boot, architecture detection, boot menu

#### 2. Kernel Core
- **Location**: `kernel/`
- **Purpose**: Core kernel functionality
- **Subsystems**:
  - Memory management
  - Process and thread management
  - Interrupt handling
  - System calls

#### 3. Device Drivers
- **Location**: `libraries/device-drivers/`
- **Purpose**: Hardware device abstraction
- **Features**: Unified device interface, hot-plug support

#### 4. Services Layer
- **Location**: `kernel/services/`
- **Purpose**: System services implementation
- **Features**: File system, network stack, GUI system

#### 5. User Space
- **Location**: `userland/`
- **Purpose**: User applications and utilities
- **Features**: CLI shell, GUI applications, development tools

#### 6. Cross-Platform Layer
- **Location**: `cross_platform_compat_layer/`
- **Purpose**: Architecture abstraction
- **Features**: Unified APIs across architectures

### Key Design Principles

1. **Memory Safety**: Leveraging Rust's guarantees
2. **Modularity**: Clear separation of concerns
3. **Performance**: Optimized for educational use
4. **Portability**: Multi-architecture support
5. **Testability**: Comprehensive testing framework

## Building MultiOS

### Build System

MultiOS uses Cargo for Rust code and Make for build orchestration:

#### Basic Build Commands

```bash
# Build for specific architecture
make build-x86_64          # x86_64 architecture
make build-arm64           # ARM64 architecture
make build-riscv64         # RISC-V architecture

# Build all architectures
make build-all

# Clean build artifacts
make clean
cargo clean
```

#### Build Targets

**Debug Builds:**
```bash
# Quick development builds
make build-x86_64
cargo build --target x86_64-unknown-none-elf
```

**Release Builds:**
```bash
# Optimized builds
RELEASE=1 make build-x86_64
cargo build --release --target x86_64-unknown-none-elf
```

**Custom Builds:**
```bash
# With specific features
cargo build --features debug,trace

# With custom target directory
CARGO_TARGET_DIR=target/custom make build-x86_64

# Parallel builds
make build-x86_64 -j$(nproc)
```

### Cross-Compilation

MultiOS supports building for multiple architectures:

#### Target Configurations

**Rust Targets:**
```bash
# x86_64 (Intel/AMD)
rustup target add x86_64-unknown-none-elf

# ARM64 (AArch64)
rustup target add aarch64-unknown-none-elf

# RISC-V 64-bit
rustup target add riscv64gc-unknown-none-elf
```

**Cross-Compilation:**
```bash
# Build for ARM64 from x86_64 host
cross build --target aarch64-unknown-none-elf

# Build for RISC-V from x86_64 host
cross build --target riscv64gc-unknown-none-elf
```

### Build Verification

```bash
# Run tests after build
make test-x86_64

# Check code formatting
cargo fmt --check

# Run linting
cargo clippy

# Security audit
cargo audit

# Check documentation
cargo doc --no-deps --document-private-items
```

## Project Structure

### Root Directory Structure

```
multios/
├── bootloader/               # Bootloader implementation
├── kernel/                   # Kernel core
├── userland/                 # User space applications
├── libraries/                # Reusable libraries
│   ├── device-drivers/      # Device driver framework
│   ├── filesystem/          # File system implementations
│   ├── memory-manager/      # Memory management
│   ├── scheduler/           # Process/thread scheduling
│   └── ipc/                 # Inter-process communication
├── cross_platform_compat_layer/  # Architecture abstraction
├── tests/                   # Test suites
├── tools/                   # Build and development tools
├── scripts/                 # Build and utility scripts
├── docs/                    # Documentation
├── examples/                # Code examples
├── ci/                      # CI/CD configuration
├── qemu_testing/           # QEMU testing setup
├── driver_testing_framework/  # Driver testing
└── filesystem_testing/      # File system testing
```

### Kernel Structure

```
kernel/
├── src/
│   ├── arch/               # Architecture-specific code
│   │   ├── x86_64/         # x86_64 implementation
│   │   ├── aarch64/        # ARM64 implementation
│   │   └── riscv64/        # RISC-V implementation
│   ├── bootstrap/          # Bootstrapping code
│   ├── drivers/            # Core drivers
│   ├── filesystem/         # File system implementation
│   ├── fonts/              # Font rendering
│   ├── hal/                # Hardware abstraction layer
│   ├── interrupt/          # Interrupt handling
│   ├── ipc/                # Inter-process communication
│   ├── memory/             # Memory management
│   ├── scheduler/          # Task scheduling
│   ├── service_manager/    # Service management
│   ├── services/           # System services
│   ├── syscall/            # System call implementation
│   └── lib.rs              # Kernel crate root
├── Cargo.toml              # Kernel crate configuration
├── build_bootstrap.sh       # Bootstrap build script
└── BOOTSTRAP_DOCUMENTATION.md
```

### Code Organization Principles

1. **Separation of Concerns**: Each module has a clear responsibility
2. **Layered Architecture**: Higher layers depend on lower layers
3. **Interface Definitions**: Clear APIs between components
4. **Feature Gates**: Optional functionality controlled by features
5. **Architecture Abstraction**: Common interfaces across platforms

## Coding Standards

### Rust Standards

MultiOS follows Rust community standards with some additions:

#### Code Style

**Formatting:**
```rust
// Use rustfmt for consistent formatting
cargo fmt

// Maximum line length: 100 characters
// Use 4 spaces for indentation
// Use snake_case for variables and functions
// Use PascalCase for types and traits
```

**Naming Conventions:**
```rust
// Variables and functions: snake_case
let user_name = "John";
fn process_data(data: &Data) -> Result<(), Error>;

// Types and traits: PascalCase
struct ProcessControlBlock;
trait DeviceDriver;

// Constants: SCREAMING_SNAKE_CASE
const MAX_PROCESSES: usize = 1024;

// Modules: snake_case
mod memory_manager;
pub mod syscall_interface;
```

#### Error Handling

```rust
// Use Result for functions that can fail
fn initialize_device() -> Result<(), DriverError> {
    // Implementation
    Ok(())
}

// Use ? operator for early return on error
fn setup_system() -> Result<(), SystemError> {
    initialize_memory()?;
    initialize_devices()?;
    initialize_scheduler()?;
    Ok(())
}

// Provide meaningful error messages
fn divide(a: f64, b: f64) -> Result<f64, MathError> {
    if b == 0.0 {
        return Err(MathError::DivisionByZero);
    }
    Ok(a / b)
}
```

#### Memory Safety

```rust
// Use references instead of raw pointers when possible
fn process_data(data: &[u8]) -> Result<(), ProcessingError> {
    // Safe processing
}

// Use unsafe blocks only when necessary and well-documented
unsafe fn read_physical_memory(address: usize) -> u32 {
    // Only in kernel code for hardware access
}

// Prefer owned types with clear ownership
struct KernelHeap {
    allocator: GlobalAlloc,
}
```

#### Documentation

```rust
/// Initialize the kernel memory subsystem.
///
/// This function sets up the physical memory manager and creates
/// the initial kernel heap. It must be called before any other
/// memory allocation functions.
///
/// # Arguments
///
/// * `memory_map` - Physical memory layout provided by bootloader
///
/// # Returns
///
/// Returns `Ok(())` on success, or `MemoryError` if initialization
/// fails.
///
/// # Examples
///
/// ```
/// use multios_kernel::memory::init_kernel_memory;
/// 
/// let memory_map = get_bootloader_memory_map();
/// init_kernel_memory(&memory_map)?;
/// ```
pub fn init_kernel_memory(memory_map: &MemoryMap) -> Result<(), MemoryError> {
    // Implementation
}
```

### Architecture-Specific Code

#### Code Organization

```rust
// Architecture-specific modules
mod arch {
    pub mod x86_64;
    pub mod aarch64;
    pub mod riscv64;
}

// Use feature gates for architecture-specific code
#[cfg(target_arch = "x86_64")]
mod x86_64_specific;

#[cfg(target_arch = "aarch64")]
mod aarch64_specific;

#[cfg(target_arch = "riscv64")]
mod riscv64_specific;
```

#### Hardware Abstraction

```rust
// Abstract hardware interface
pub trait HardwareInterface {
    fn read_register(&self, addr: usize) -> u32;
    fn write_register(&self, addr: usize, value: u32);
    fn enable_interrupts(&self);
    fn disable_interrupts(&self);
}

// Architecture-specific implementations
#[cfg(target_arch = "x86_64")]
pub struct X86_64Hardware;

#[cfg(target_arch = "aarch64")]
pub struct AArch64Hardware;
```

### Testing Standards

#### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_creation() {
        let mut process_manager = ProcessManager::new();
        let params = ProcessCreateParams {
            name: b"test_process".to_vec(),
            priority: ProcessPriority::Normal,
            stack_size: 4096,
            entry_point: Some(test_entry),
        };

        let result = process_manager.create_process(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_memory_allocation() {
        let mut memory_manager = MemoryManager::new();
        
        let buffer = memory_manager.allocate(1024);
        assert!(buffer.is_ok());
        
        let allocated = buffer.unwrap();
        assert!(!allocated.is_empty());
    }
}
```

#### Integration Tests

```rust
#[cfg(test)]
mod integration_tests {
    use multios_kernel::{init, shutdown};
    
    #[test]
    fn test_kernel_initialization() {
        let result = init();
        assert!(result.is_ok());
        
        // Perform integration tests
        shutdown();
    }
}
```

## Testing Framework

### Test Organization

MultiOS uses multiple levels of testing:

#### 1. Unit Tests
- Test individual functions and modules
- Fast execution, no hardware dependencies
- Located in `src/.../mod.rs` files

#### 2. Integration Tests
- Test component interactions
- Located in `tests/` directory
- May use QEMU for hardware emulation

#### 3. System Tests
- Test complete system functionality
- Located in `tests/` with `system_` prefix
- Run on QEMU or real hardware

#### 4. Performance Tests
- Benchmark system performance
- Located in `tests/benchmarks/`
- Automated performance regression detection

### Running Tests

#### Basic Test Commands

```bash
# Run all tests
cargo test

# Run tests for specific crate
cargo test -p kernel

# Run tests for specific architecture
cargo test --target x86_64-unknown-none-elf

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_process_creation
```

#### Test Categories

```bash
# Unit tests only
cargo test --lib

# Integration tests only
cargo test --test integration

# Doc tests
cargo test --doc

# Benchmarks
cargo bench
```

#### QEMU Testing

```bash
# Run tests in QEMU
make test-qemu-x86_64

# Run all architecture tests
make test-qemu-all

# Custom QEMU test
qemu-system-x86_64 \
    -kernel target/x86_64-unknown-none-elf/debug/multios \
    -m 256M \
    -serial stdio \
    -nographic \
    -smp 1 \
    -S -gdb tcp::1234
```

### Test Coverage

```bash
# Generate coverage report
make coverage

# View coverage in browser
open target/coverage/index.html

# Upload coverage to service (if configured)
make coverage-upload
```

### Continuous Integration

The project uses GitHub Actions for CI:

```yaml
# .github/workflows/ci.yml
name: CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        target: [x86_64-unknown-none-elf, aarch64-unknown-none-elf, riscv64gc-unknown-none-elf]
    
    steps:
    - uses: actions/checkout@v2
    - uses: actions-rs/toolchain@v1
      with:
        targets: ${{ matrix.target }}
        toolchain: stable
    - name: Build
      run: cargo build --target ${{ matrix.target }}
    - name: Test
      run: cargo test --target ${{ matrix.target }}
    - name: Check formatting
      run: cargo fmt -- --check
    - name: Clippy
      run: cargo clippy --all-targets --all-features -- -D warnings
```

## Debugging Guide

### Debug Build Configuration

```bash
# Build with debug information
cargo build --target x86_64-unknown-none-elf

# Build with additional debug features
cargo build --target x86_64-unknown-none-elf --features debug,trace

# Disable optimizations for debugging
cargo build --target x86_64-unknown-none-elf --profile dev
```

### GDB Debugging

#### Setting Up GDB

```bash
# Install GDB with multi-architecture support
sudo apt-get install gdb-multiarch

# Configure GDB for MultiOS
echo "source .gdb/x86_64.gdb" > ~/.gdbinit

# Start GDB with MultiOS binary
gdb-multiarch target/x86_64-unknown-none-elf/debug/multios
```

#### Common GDB Commands

```gdb
# Connection to QEMU
target remote localhost:1234

# Breakpoints
break kernel_main
break memory::init
break syscall_handler

# Execution control
continue
step
stepi
next
finish

# Information commands
info registers
info stack
info threads
backtrace
list

# Memory examination
x/10x $rsp
x/s 0x80000000
info variables
```

#### GDB Scripts

**`.gdb/x86_64.gdb`:**
```gdb
# x86_64 specific setup
set architecture i386:x86-64
set disassembly-flavor intel
set pagination off

# MultiOS specific commands
define multios-help
    echo MultiOS Debug Commands:\n
    echo   setup-x86_64      - Setup x86_64 debugging\n
    echo   info-memory       - Show memory layout\n
    echo   info-processes    - Show running processes\n
    echo   backtrace-full    - Full backtrace with locals\n
end

define setup-x86_64
    set architecture i386:x86-64
    set disassembly-flavor intel
    info registers
end

define info-memory
    info proc mappings
    x/32x 0x80000000
end

# Auto-load script
define onload
    setup-x86_64
end
document onload
Setup x86_64 debugging environment
end
```

### QEMU Debugging

#### Starting QEMU for Debugging

```bash
# Start QEMU with GDB server
qemu-system-x86_64 \
    -kernel target/x86_64-unknown-none-elf/debug/multios \
    -S -gdb tcp::1234 \
    -m 256M \
    -nographic \
    -smp 1

# Start with more debugging options
qemu-system-x86_64 \
    -kernel target/x86_64-unknown-none-elf/debug/multios \
    -S -gdb tcp::1234 \
    -m 256M \
    -nographic \
    -smp 1 \
    -d guest_errors \
    -D qemu.log \
    -singlestep
```

#### QEMU Monitor

```bash
# Access QEMU monitor (Ctrl+A then C)
# Or use separate terminal
tmux new-session -d -s qemu 'qemu-system-x86_64 -kernel ... -monitor stdio'
tmux send-keys -t qemu C-a c 'info registers' Enter
```

### Logging and Tracing

#### Kernel Logging

```rust
// Enable kernel logging
use multios_kernel::log::{info, warn, error, debug};

fn kernel_function() {
    info!("Starting kernel function");
    debug!("Processing data: {:?}", data);
    
    if error_condition {
        error!("Failed to process data");
        return Err(Error::Failed);
    }
    
    info!("Kernel function completed successfully");
}
```

#### Conditional Compilation

```rust
#[cfg(feature = "trace")]
fn trace_function() {
    println!("Function called: {}", function_name!());
}

#[cfg(feature = "debug")]
fn debug_check(value: usize) -> usize {
    assert!(value < MAX_VALUE, "Value too large");
    value
}
```

### Memory Debugging

#### Memory Leak Detection

```rust
// Enable memory debugging
#[cfg(feature = "memory-debug")]
static ALLOCATIONS: Spinlock<HashMap<usize, AllocationInfo>> = Spinlock::new(HashMap::new());

fn allocate_memory(size: usize) -> Result<usize, AllocError> {
    let ptr = kmalloc(size)?;
    
    #[cfg(feature = "memory-debug")]
    {
        let mut allocations = ALLOCATIONS.lock();
        allocations.insert(ptr as usize, AllocationInfo {
            size,
            timestamp: get_time(),
            backtrace: get_backtrace(),
        });
    }
    
    Ok(ptr)
}
```

#### Stack Overflow Detection

```rust
// Stack canary check
fn check_stack_overflow() {
    let stack_ptr = get_stack_pointer();
    let stack_bottom = get_stack_bottom();
    
    if stack_ptr < stack_bottom + STACK_GUARD_SIZE {
        panic!("Stack overflow detected!");
    }
}
```

## Contributing Guidelines

### How to Contribute

We welcome contributions to MultiOS! Here's how to get started:

#### 1. Fork and Clone

```bash
# Fork the repository on GitHub, then clone
git clone https://github.com/your-username/multios.git
cd multios
git remote add upstream https://github.com/multios/multios.git
```

#### 2. Create a Branch

```bash
# Create a feature branch
git checkout -b feature/your-feature-name

# Or for bugfix
git checkout -b bugfix/issue-number-description
```

#### 3. Make Changes

- Follow the coding standards
- Add tests for new functionality
- Update documentation
- Ensure all tests pass

#### 4. Commit Changes

```bash
# Stage changes
git add .

# Commit with descriptive message
git commit -m "feat: add memory leak detection feature

- Implement allocation tracking for memory debugging
- Add stack overflow detection
- Include comprehensive tests

Closes #123"

# Push to your fork
git push origin feature/your-feature-name
```

#### 5. Create Pull Request

1. Open GitHub and navigate to your fork
2. Click "New Pull Request"
3. Choose your branch and create PR
4. Fill out the PR template
5. Submit for review

### Pull Request Guidelines

#### PR Title Format

- `feat:` - New features
- `fix:` - Bug fixes
- `docs:` - Documentation changes
- `style:` - Code style changes
- `refactor:` - Code refactoring
- `test:` - Test additions/modifications
- `chore:` - Build/process changes

#### PR Description Template

```markdown
## Description
Brief description of changes made.

## Type of Change
- [ ] Bug fix (non-breaking change which fixes an issue)
- [ ] New feature (non-breaking change which adds functionality)
- [ ] Breaking change (fix or feature that would cause existing functionality to not work as expected)
- [ ] This change requires a documentation update

## Testing
- [ ] Unit tests pass
- [ ] Integration tests pass
- [ ] Manual testing completed
- [ ] Cross-platform testing completed

## Checklist
- [ ] My code follows the style guidelines
- [ ] I have performed a self-review of my code
- [ ] I have commented my code, particularly in hard-to-understand areas
- [ ] I have made corresponding changes to the documentation
- [ ] My changes generate no new warnings
- [ ] I have added tests that prove my fix is effective or that my feature works
- [ ] New and existing unit tests pass locally with my changes
```

### Code Review Process

#### For Contributors

1. **Keep PRs Focused**: One feature or fix per PR
2. **Write Good Commit Messages**: Clear and descriptive
3. **Add Tests**: Include comprehensive test coverage
4. **Update Documentation**: Keep docs synchronized with code
5. **Respond to Feedback**: Address reviewer comments promptly

#### For Reviewers

1. **Be Respectful**: Provide constructive feedback
2. **Be Thorough**: Check all aspects of the change
3. **Test Changes**: Run locally when possible
4. **Provide Examples**: Suggest improvements with examples
5. **Be Timely**: Review PRs promptly

### Issue Guidelines

#### Bug Reports

Use the bug report template:

```markdown
**Describe the bug**
A clear description of the bug.

**To Reproduce**
Steps to reproduce the behavior:
1. Run command '...'
2. See error

**Expected behavior**
What you expected to happen.

**Environment:**
- OS: [e.g. Ubuntu 20.04]
- Architecture: [e.g. x86_64]
- Rust version: [e.g. 1.70.0]
- MultiOS version: [e.g. v1.0.0]

**Additional context**
Any other relevant information.
```

#### Feature Requests

Use the feature request template:

```markdown
**Is your feature request related to a problem?**
A clear description of what the problem is.

**Describe the solution you'd like**
A clear description of what you want to happen.

**Describe alternatives you've considered**
A clear description of any alternative solutions.

**Additional context**
Any other relevant information or examples.
```

## Release Process

### Version Numbering

MultiOS follows Semantic Versioning (SemVer):

- **MAJOR.MINOR.PATCH** (e.g., 1.2.3)
- **MAJOR**: Breaking changes
- **MINOR**: New features (backward compatible)
- **PATCH**: Bug fixes (backward compatible)

### Release Workflow

#### 1. Prepare Release

```bash
# Update version numbers
# Edit Cargo.toml files
# Update CHANGELOG.md

# Test release candidate
make test-all
make test-qemu-all

# Generate release notes
make release-notes
```

#### 2. Create Release Branch

```bash
git checkout -b release/v1.2.0
# Make version bumps
git commit -m "chore: prepare release v1.2.0"
git tag -a v1.2.0 -m "Release v1.2.0"
```

#### 3. Release

```bash
# Merge to main
git checkout main
git merge release/v1.2.0

# Push release
git push origin main
git push origin v1.2.0

# Create GitHub release with notes
make create-github-release
```

### Automated Release

GitHub Actions handles releases:

```yaml
# .github/workflows/release.yml
name: Release

on:
  push:
    tags:
      - 'v*'

jobs:
  release:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v2
    - uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        targets: x86_64-unknown-none-elf,aarch64-unknown-none-elf,riscv64gc-unknown-none-elf
    
    - name: Build
      run: make build-all
      
    - name: Test
      run: make test-all
      
    - name: Create Release
      uses: actions/create-release@v1
      with:
        tag_name: ${{ github.ref }}
        release_name: Release ${{ github.ref }}
        body: |
          See [CHANGELOG.md](CHANGELOG.md) for details.
        draft: false
        prerelease: false
        
    - name: Upload Artifacts
      uses: actions/upload-release-asset@v1
      with:
        upload_url: ${{ steps.create_release.outputs.upload_url }}
        asset_path: ./target/release/multios
        asset_name: multios
        asset_content_type: application/octet-stream
```

## Documentation

### Documentation Guidelines

#### API Documentation

```rust
/// Brief description of function or type.
///
/// ## Detailed Description
/// Longer explanation of what this does and why.
///
/// ## Arguments
///
/// * `param1` - Description of first parameter
/// * `param2` - Description of second parameter
///
/// ## Returns
///
/// Description of return value or error types.
///
/// ## Examples
///
/// ```
/// use multios_kernel::module;
///
/// let result = my_function(42);
/// assert_eq!(result.is_ok(), true);
/// ```
///
/// ## Panics
///
/// This function panics if:
///
/// - Parameter is zero
/// - Memory allocation fails
///
/// ## Errors
///
/// Possible error conditions:
///
/// - [`Error::InvalidParameter`](enum.Error.html#variant.InvalidParameter)
/// - [`Error::OutOfMemory`](enum.Error.html#variant.OutOfMemory)
pub fn my_function(param1: usize, param2: &str) -> Result<(), Error> {
    // Implementation
}
```

#### Guide Documentation

- Use Markdown format
- Include code examples
- Add diagrams where helpful
- Keep sections focused and concise
- Include troubleshooting sections

### Building Documentation

```bash
# Generate Rust API documentation
cargo doc --no-deps --document-private-items

# Build documentation website
make docs

# Serve documentation locally
make serve-docs

# Check documentation links
make doc-check-links
```

### Documentation Structure

```
docs/
├── README.md              # Documentation index
├── getting_started/       # Getting started guides
├── user_guide/           # User documentation
├── developer/            # Developer documentation
├── api/                  # API reference
├── architecture/         # Architecture guides
├── tutorials/            # Step-by-step tutorials
├── examples/             # Code examples
├── troubleshooting/      # Problem solving
└── research/             # Design documents
```

---

**Up**: [Documentation Index](../README.md)  
**Next**: [Building MultiOS](building.md)  
**Related**: [Contributing Guidelines](contributing.md) | [Testing Guide](testing.md)