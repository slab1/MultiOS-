# Rust Coding Standards

This document defines the coding standards and style guidelines for all Rust code in the MultiOS project. These standards ensure code quality, maintainability, and consistency across the entire codebase.

## 📋 Table of Contents

- [Code Formatting](#code-formatting)
- [Naming Conventions](#naming-conventions)
- [Error Handling](#error-handling)
- [Memory Safety](#memory-safety)
- [Performance Guidelines](#performance-guidelines)
- [Documentation Standards](#documentation-standards)
- [Testing Requirements](#testing-requirements)
- [Dependencies](#dependencies)
- [Linting and CI](#linting-and-ci)

## 🎨 Code Formatting

### General Formatting Rules

All Rust code in MultiOS must be formatted using `rustfmt`. The project uses a specific configuration that extends the standard rustfmt settings.

**Required Configuration (`rustfmt.toml`):**
```toml
version = "Two"
edition = "2021"
use_try_syntax = true
use_field_init_shorthand = true
force_explicit_abi = true
empty_item_single_line = true
struct_lit_single_line = true
fn_single_line = false
where_single_line = true
imports_layout = "Mixed"
imports_granularity = "Crate"
reorder_imports = true
reorder_modules = true
remove_nested_parens = true
merge_derives = true
use_try_shorthand = true
use_field_init_shorthand = true
force_explicit_abi = true
```

### Line Length and Layout

- **Maximum line length**: 100 characters
- **Indentation**: 4 spaces (no tabs)
- **Trailing whitespace**: Must be removed
- **Line breaks**: Use LF (`\n`) line endings

### Code Structure

**Module Structure:**
```rust
// Core modules first
use core::prelude::*;

// Standard library
use std::collections::HashMap;

// Third-party dependencies
use some_crate::{SomeStruct, some_function};

// Local modules
mod inner_module;

// Public interface
pub struct PublicStruct {
    pub field: Type,
    internal: InternalType,
}

impl PublicStruct {
    pub fn new() -> Self {
        Self {
            field: Default::default(),
            internal: InternalType::new(),
        }
    }
}

// Private implementation details
struct InternalStruct {
    data: Vec<u8>,
}
```

## 🏷️ Naming Conventions

### Variables and Functions

```rust
// Variables: snake_case
let max_buffer_size = 4096;
let mut current_pointer = 0x1000;

// Functions: snake_case
fn calculate_checksum(data: &[u8]) -> u32 {
    // implementation
}

fn initialize_kernel() -> Result<(), KernelError> {
    // implementation
}
```

### Types and Constants

```rust
// Structs, Enums, Traits: PascalCase
pub struct MemoryManager;
pub enum TaskState;
pub trait DeviceDriver;
pub struct NetworkPacket {
    pub header: PacketHeader,
    pub payload: Vec<u8>,
}

// Constants: SCREAMING_SNAKE_CASE
const MAX_MEMORY_SIZE: usize = 0xFFFF_FFFF;
const DEFAULT_PAGE_SIZE: u32 = 4096;

// Associated constants
impl NetworkPacket {
    const HEADER_SIZE: usize = 20;
    const MAX_PACKET_SIZE: usize = 1500;
}
```

### Modules and Files

```rust
// Module names: snake_case
mod memory_manager;
mod network_stack;
mod device_drivers;

// File names: snake_case (same as module names)
// memory_manager.rs
// network_stack.rs
// device_drivers.rs
```

### Private vs Public

```rust
// Public API - clearly documented
/// Represents a file system node in the virtual file system
pub struct VfsNode {
    /// Inode number for identification
    pub inode: u64,
    /// Node type and permissions
    pub mode: FileMode,
    // Internal fields (private)
    children: HashMap<String, Arc<VfsNode>>,
}

// Private implementation
struct VfsNodeInternal {
    reference_count: AtomicU64,
    last_accessed: AtomicU64,
}
```

## ⚠️ Error Handling

### Result and Option Usage

```rust
// Use Result for operations that can fail
fn read_file(path: &str) -> Result<Vec<u8>, IoError> {
    // Implementation that returns Ok or Err
}

// Use Option for optional values
fn get_cached_data(key: &str) -> Option<CacheEntry> {
    // Implementation that returns Some or None
}

// Avoid unwrap() in production code
fn safe_division(a: f64, b: f64) -> Result<f64, DivisionError> {
    if b == 0.0 {
        return Err(DivisionError::DivisionByZero);
    }
    Ok(a / b)
}

// Good error propagation
fn process_config_file() -> Result<Config, ConfigError> {
    let data = read_file("config.toml")
        .map_err(ConfigError::IoError)?;
    
    let config = toml::from_str(&data)
        .map_err(ConfigError::ParseError)?;
    
    Ok(config)
}
```

### Custom Error Types

```rust
/// Custom error type for MultiOS kernel operations
#[derive(Debug, Clone)]
pub enum KernelError {
    /// Out of memory error
    OutOfMemory,
    /// Invalid argument provided
    InvalidArgument(String),
    /// Permission denied
    PermissionDenied,
    /// Device not found
    DeviceNotFound,
    /// System call failed
    SyscallFailed {
        call: SyscallNumber,
        error: Errno,
    },
}

impl std::fmt::Display for KernelError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            KernelError::OutOfMemory => write!(f, "Out of memory"),
            KernelError::InvalidArgument(msg) => write!(f, "Invalid argument: {}", msg),
            KernelError::PermissionDenied => write!(f, "Permission denied"),
            KernelError::DeviceNotFound => write!(f, "Device not found"),
            KernelError::SyscallFailed { call, error } => {
                write!(f, "Syscall {} failed with error: {}", call, error)
            }
        }
    }
}
```

## 🛡️ Memory Safety

### Ownership and Borrowing

```rust
// Use references for read-only access
fn validate_buffer(buffer: &[u8]) -> bool {
    buffer.len() > 0 && buffer.len() <= MAX_BUFFER_SIZE
}

// Use mutable references for read-write access
fn update_counter(counter: &mut u32) {
    *counter += 1;
}

// Use Arc for shared ownership across threads
use std::sync::Arc;
use std::sync::Mutex;

pub struct SharedResource {
    data: Arc<Mutex<Vec<u8>>>,
    readers: std::sync::atomic::AtomicU32,
}

// Clone Arc when passing to multiple owners
fn share_resource(resource: Arc<Mutex<Vec<u8>>>) {
    let cloned = Arc::new(Mutex::new(Vec::new()));
    // Work with both Arc pointers safely
}
```

### Lifetimes

```rust
// Explicit lifetimes for complex references
fn longest<'a>(s1: &'a str, s2: &'a str) -> &'a str {
    if s1.len() > s2.len() {
        s1
    } else {
        s2
    }
}

// Lifetime bounds on trait objects
trait EventHandler<'a> {
    fn handle_event(&self, event: &Event) -> Result<(), Box<dyn std::error::Error + 'a>>;
}
```

### Unsafe Code Guidelines

```rust
// When using unsafe, document why and what guarantees are maintained
unsafe fn read_memory_at(address: usize) -> u32 {
    // SAFETY: This function is only used for debugging and memory inspection.
    // The caller guarantees that the address is valid and aligned.
    // No undefined behavior is introduced.
    let ptr = address as *const u32;
    ptr.read_volatile()
}

// Zero-cost abstractions preferred
pub fn optimized_copy<T: Copy>(src: &[T], dst: &mut [T]) -> Result<(), CopyError> {
    if src.len() != dst.len() {
        return Err(CopyError::SizeMismatch);
    }
    
    // Copy using memcpy for optimal performance
    unsafe {
        std::ptr::copy_nonoverlapping(
            src.as_ptr(),
            dst.as_mut_ptr(),
            src.len()
        );
    }
    Ok(())
}
```

## ⚡ Performance Guidelines

### General Principles

1. **Profile Before Optimizing**: Always measure before making performance claims
2. **Prefer Zero-Cost Abstractions**: Use Rust's powerful type system and iterators
3. **Minimize Allocations**: Reuse buffers and avoid unnecessary heap allocations
4. **Use Appropriate Data Structures**: HashMap for lookups, BTreeMap for ordered data

### Performance-Sensitive Code

```rust
// Pre-allocate buffers for known sizes
fn create_response_buffer(size: usize) -> Vec<u8> {
    let mut buffer = Vec::with_capacity(size);
    buffer.extend_from_slice(b"HTTP/1.1 200 OK\r\n");
    buffer.extend_from_slice(b"Content-Length: ");
    buffer.extend_from_slice(size.to_string().as_bytes());
    buffer.extend_from_slice(b"\r\n\r\n");
    buffer
}

// Use iterators for efficient operations
fn filter_active_connections(connections: &[Connection]) -> Vec<&Connection> {
    connections.iter()
        .filter(|conn| conn.is_active())
        .collect()
}

// Inline hot paths for performance
#[inline]
fn fast_hash(key: &[u8]) -> u64 {
    // Fast non-cryptographic hash for hash maps
    key.iter().fold(0, |acc, &b| acc.wrapping_mul(31).wrapping_add(b as u64))
}
```

## 📚 Documentation Standards

### Public API Documentation

```rust
/// Initializes the kernel memory management subsystem.
///
/// This function sets up the page allocator, heap allocator, and virtual memory
/// management structures required for kernel operation. It must be called
/// before any other memory management functions.
///
/// # Arguments
///
/// * `memory_info` - Information about available physical memory regions
///
/// # Returns
///
/// Returns a `Result` containing the initialized `MemoryManager` on success,
/// or a `KernelError::OutOfMemory` if allocation fails.
///
/// # Examples
///
/// ```rust
/// use multios::memory::MemoryManager;
/// use multios::arch::MemoryInfo;
///
/// let memory_info = MemoryInfo::from_efi();
/// let manager = MemoryManager::new(memory_info)
///     .expect("Failed to initialize memory manager");
/// ```
///
/// # Safety
///
/// This function is unsafe because it assumes the given memory regions
/// are valid and not already in use by other subsystems.
///
/// # Panics
///
/// This function will panic if the memory layout is invalid or if
/// required memory structures cannot be allocated.
pub unsafe fn new(memory_info: MemoryInfo) -> Result<Self, KernelError> {
    // Implementation
}
```

### Internal Documentation

```rust
// Internal helper functions don't need extensive documentation
// but should have brief comments explaining their purpose
fn align_up(value: usize, alignment: usize) -> usize {
    // Round value up to the next alignment boundary
    (value + alignment - 1) & !(alignment - 1)
}

// Complex algorithms should have algorithmic comments
fn quick_sort<T: PartialOrd>(slice: &mut [T]) {
    fn partition<T: PartialOrd>(slice: &mut [T]) -> usize {
        let pivot_idx = slice.len() - 1;
        let mut i = 0;
        
        for j in 0..pivot_idx {
            // Lomuto partition scheme
            if slice[j] <= slice[pivot_idx] {
                slice.swap(i, j);
                i += 1;
            }
        }
        
        slice.swap(i, pivot_idx);
        i
    }
    
    fn quick_sort_recursive<T: PartialOrd>(slice: &mut [T]) {
        if slice.len() <= 1 {
            return;
        }
        
        let pivot = partition(slice);
        
        // Recursively sort both partitions
        quick_sort_recursive(&mut slice[..pivot]);
        quick_sort_recursive(&mut slice[pivot + 1..]);
    }
    
    quick_sort_recursive(slice);
}
```

## 🧪 Testing Requirements

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_allocation() {
        let mut manager = MemoryManager::new_test();
        let addr = manager.allocate(4096);
        assert!(addr.is_some());
    }

    #[test]
    fn test_invalid_allocation() {
        let mut manager = MemoryManager::new_test();
        let result = manager.allocate(usize::MAX);
        assert!(result.is_none());
    }

    #[test]
    fn test_multiple_allocations() {
        let mut manager = MemoryManager::new_test();
        let addr1 = manager.allocate(1024).unwrap();
        let addr2 = manager.allocate(1024).unwrap();
        let addr3 = manager.allocate(1024).unwrap();
        
        // Verify allocations don't overlap
        assert!(addr1 + 1024 <= addr2);
        assert!(addr2 + 1024 <= addr3);
    }
}
```

### Integration Tests

```rust
// tests/integration/test_kernel_boot.rs
use multios::kernel::Kernel;
use multios::arch::ArchInfo;

#[test]
fn test_full_kernel_boot() {
    let arch_info = ArchInfo::test_x86_64();
    let mut kernel = Kernel::new(arch_info);
    
    // Test boot sequence
    assert!(kernel.initialize().is_ok());
    assert!(kernel.is_running());
    
    // Test basic services are available
    assert!(kernel.memory_manager().is_some());
    assert!(kernel.scheduler().is_some());
}
```

### Benchmark Tests

```rust
// tests/benchmarks/memory_bench.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_allocation(c: &mut Criterion) {
    c.bench_function("allocation_1kb", |b| {
        let mut manager = MemoryManager::new_test();
        b.iter(|| {
            let addr = manager.allocate(black_box(1024));
            if let Some(addr) = addr {
                manager.deallocate(addr);
            }
        });
    });
}

criterion_group!(benches, benchmark_allocation);
criterion_main!(benches);
```

## 📦 Dependencies

### Dependency Management

```toml
# Cargo.toml
[dependencies]
# Standard library extensions
tokio = { version = "1.0", features = ["full"], optional = true }
serde = { version = "1.0", features = ["derive"] }

# Optional features
log = "0.4"
env_logger = { version = "0.9", optional = true }

# Conditional compilation
[target.'cfg(target_os = "multios")'.dependencies]
multios-specific = { path = "../libraries/multios-specific" }

# Dev dependencies (testing only)
[dev-dependencies]
tempfile = "3.0"
mockall = "0.11"

# Build dependencies
[build-dependencies]
cc = "1.0"
```

### Crate Features

```rust
// lib.rs
#[cfg(feature = "logging")]
pub use log::{debug, info, warn, error};

// Conditional compilation for optional features
#[cfg(feature = "performance_monitoring")]
pub mod performance;

#[cfg(not(feature = "performance_monitoring"))]
pub mod performance {
    pub fn start_profiler() {}
    pub fn stop_profiler() -> ProfilingResult { ProfilingResult::default() }
}
```

## 🔧 Linting and CI

### Required Linting Tools

1. **rustfmt**: Code formatting
2. **clippy**: Linting and style checking
3. **cargo-audit**: Security vulnerability scanning
4. **cargo-outdated**: Dependency version checking

### CI Configuration

```yaml
# .github/workflows/rust.yml
name: Rust CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3
    
    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        components: rustfmt, clippy
        
    - name: Check formatting
      run: cargo fmt --all -- --check
      
    - name: Run clippy
      run: cargo clippy --all-targets --all-features -- -D warnings
      
    - name: Run tests
      run: cargo test --all-features
      
    - name: Run benchmarks
      if: github.event_name == 'push'
      run: cargo bench --all-features
      
    - name: Audit dependencies
      run: cargo audit
```

### Recommended clippy configuration

```toml
# .clippy.toml
warn = [
    "clippy::all",
    "clippy::pedantic",
    "clippy::cargo",
]

allow = [
    "clippy::must_use_candidate",
    "clippy::missing_errors_doc",
    "clippy::missing_panics_doc",
]

# MultiOS-specific allows
allow-features = [
    "test",
]
```

---

## 📋 Checklist for Code Reviews

When reviewing Rust code, ensure:

- [ ] Code follows formatting standards (run rustfmt)
- [ ] No clippy warnings or lints
- [ ] All public functions are documented
- [ ] Error handling is appropriate (Result/Option used correctly)
- [ ] Memory safety is maintained (no unsafe code without justification)
- [ ] Tests are included and passing
- [ ] Dependencies are minimal and justified
- [ ] Performance implications are considered
- [ ] Architecture guidelines are followed

*Last Updated: November 3, 2025*