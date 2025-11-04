# MultiOS Bootloader Memory Management

## Overview

The MultiOS bootloader implements comprehensive memory management initialization that supports both BIOS and UEFI environments. This system provides memory detection, bitmap allocation, heap initialization, and safe Rust interfaces for memory operations.

## Architecture

### Core Components

1. **Memory Detection**
   - BIOS memory detection via INT 15h, EAX=0xE820
   - UEFI memory detection via System Table
   - Fallback memory detection for unknown boot modes

2. **Memory Bitmap**
   - Efficient frame tracking using bitmaps
   - Contiguous frame allocation
   - Support for variable granularity (4KB default)

3. **Boot Heap**
   - Early-stage heap allocation
   - Alignment support
   - Safe Rust interfaces

4. **Memory Types**
   - Usable, Reserved, ACPI regions
   - Kernel code/data regions
   - Framebuffer and MMIO regions

## Memory Layout

### BIOS Memory Layout

```
0x00000 - 0x9F000   : Conventional memory (BIOS, Conventional RAM)
0x9F000 - 0xA0000   : BIOS data area (ACPI reclaimable)
0xA0000 - 0x100000  : Video memory and reserved
0x100000+           : System memory (usable)
```

### UEFI Memory Layout

```
0x00000 - 0x9F000   : Conventional memory
0x9F000 - 0xA0000   : BIOS data area
0xA0000 - 0x100000  : Video memory
0x100000+           : System memory (usable)
0xFEC00000+         : MMIO regions (APIC, etc.)
```

### Boot Heap Allocation

- **Default Size**: 16MB
- **Alignment**: 4KB
- **Location**: First available usable memory region
- **Granularity**: 4KB pages

## Initialization Sequence

### 1. Boot Mode Detection
```rust
let boot_mode = detect_boot_mode();
```

### 2. Memory Detection
```rust
let memory_map = match boot_mode {
    BootMode::UEFI => MemoryMap::detect_uefi_memory()?,
    BootMode::LegacyBIOS => MemoryMap::detect_bios_memory()?,
    BootMode::Unknown => fallback_memory_map(),
};
```

### 3. Bitmap Initialization
```rust
memory_map.init_bitmap()?;
// Builds frame list and initializes bitmap
```

### 4. Heap Initialization
```rust
memory_map.init_heap()?;
// Finds suitable region and initializes heap
```

### 5. Global Allocator Setup
```rust
boot_heap::init_boot_memory(boot_mode, config)?;
```

## Safe Rust Interfaces

### Memory Allocation
```rust
use boot_heap::safe_alloc;

// Allocate memory
let ptr = safe_alloc::alloc(1024, 8)?;

// Allocate zeroed memory
let zeroed_ptr = safe_alloc::alloc_zeroed(2048, 16)?;

// Check availability
if safe_alloc::is_available() {
    // Safe to allocate
}
```

### Memory Map Operations
```rust
use bootloader::memory_map;

// Get memory statistics
let stats = memory_map.get_memory_stats();

// Allocate physical frames
let frame_addr = memory_map.allocate_frames(8, 4096)?;

// Free frames
memory_map.free_frames(frame_addr, 8)?;
```

### Heap Operations
```rust
let mut memory_map = boot_heap::get_memory_map().unwrap();

// Allocate from heap
let heap_addr = memory_map.heap_allocate(4096)?;

// Get heap statistics
let heap_stats = memory_map.get_heap().unwrap().get_stats();
```

## Memory Bitmap Implementation

### Frame Tracking
```rust
struct PhysicalFrame {
    pub addr: u64,
    pub size: usize,
    pub available: bool,
    pub tested: bool,
}
```

### Bitmap Management
```rust
struct MemoryBitmap {
    data: Vec<u8>,      // Bit array
    frame_count: usize, // Total frames
    granularity: usize, // Frame size
}
```

### Allocation Algorithm
1. Search for contiguous available frames
2. Verify alignment requirements
3. Mark frames as allocated in bitmap
4. Return base address

## Error Handling

### Error Types
```rust
pub enum BootError {
    UefiNotSupported,
    LegacyNotSupported,
    KernelNotFound,
    MemoryMapError,
    InvalidKernelFormat,
    BootProcessError,
    OutOfMemory,
    HeapInitializationError,
    BitmapInitializationError,
}
```

### Recovery Mechanisms
```rust
// Attempt recovery from allocation failure
boot_heap::recovery::recover_from_allocation_failure()?;

// Reset heap allocator
let mut allocator = BOOT_HEAP_ALLOCATOR.lock();
allocator.reset()?;
```

## Configuration

### MemoryInitConfig
```rust
pub struct MemoryInitConfig {
    pub heap_size: usize,              // Default: 16MB
    pub bitmap_granularity: usize,     // Default: 4KB
    pub heap_alignment: usize,         // Default: 4KB
    pub enable_memory_test: bool,      // Default: false
    pub enable_detailed_logging: bool, // Default: true
    pub min_heap_addr: u64,            // Default: 1MB
}
```

### Usage Example
```rust
let config = MemoryInitConfig {
    heap_size: 32 * 1024 * 1024, // 32MB
    bitmap_granularity: 4096,
    heap_alignment: 4096,
    enable_detailed_logging: true,
    ..Default::default()
};

let memory_map = boot_heap::init_boot_memory(boot_mode, Some(config))?;
```

## Testing and Validation

### Memory Map Validation
```rust
// Validate memory map integrity
assert!(memory_map.validate());

// Check for overlapping regions
let regions = memory_map.regions();
for (i, r1) in regions.iter().enumerate() {
    for (j, r2) in regions.iter().enumerate() {
        if i != j {
            assert!(!memory_map.regions_overlap(r1, r2));
        }
    }
}
```

### Allocation Testing
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_allocation() {
        let mut memory_map = MemoryMap::new();
        memory_map.add_region(0x100000, 0x1000000, MemoryType::Usable);
        memory_map.init_bitmap().unwrap();

        let addr = memory_map.allocate_frames(4, 0x1000).unwrap();
        assert_eq!(addr, 0x100000);
    }
}
```

## Memory Pool Support

### Fixed-Size Block Allocation
```rust
// Create memory pool
let mut pool = MemoryPool::new(0x100000, 0x10000, 0x1000).unwrap();

// Allocate block
let addr = pool.allocate_block()?;

// Get statistics
let stats = pool.get_stats();
println!("{}", stats);
```

## Performance Considerations

### Bitmap Efficiency
- **Space**: 1 bit per frame (vs 1 byte per frame)
- **Lookup**: O(1) for availability check
- **Allocation**: O(n) for contiguous search

### Memory Alignment
- Minimum alignment: 4KB (page size)
- Automatic alignment in allocation functions
- Configurable granularity

### Optimization Features
- Contiguous frame allocation reduces fragmentation
- Bitmap compression for large memory systems
- Early heap allocation for reduced overhead

## Security Considerations

### Memory Protection
- Separate kernel and user memory regions
- NX bit support for executable pages
- Validation of all memory addresses

### Safe Interfaces
- All allocations return `NonNull<u8>`
- Bounds checking on all operations
- Panic-free error handling

## Integration Points

### Bootloader Integration
```rust
// In bootloader initialization
pub fn boot_start() -> ! {
    // ... previous initialization
    let memory_map = boot_heap::init_boot_memory(boot_mode, None)
        .expect("Memory initialization failed");
    
    // Continue with kernel loading
}
```

### Kernel Handoff
```rust
// Memory information passed to kernel
let boot_info = kernel_loader::create_kernel_boot_info(
    config.kernel_path,
    config.command_line,
    memory_map, // Include memory map
);
```

## Debug Features

### Detailed Logging
```rust
let config = MemoryInitConfig {
    enable_detailed_logging: true,
    ..Default::default()
};
```

### Memory Profiling
```rust
#[cfg(feature = "debug_mode")]
use boot_heap::profiling;

// Track allocations
profiling::track_allocation(addr, size, align);

// Get statistics
let stats = profiling::get_allocation_stats();
```

## Future Enhancements

1. **NUMA Support**
   - Node-aware allocation
   - NUMA topology detection

2. **Advanced Algorithms**
   - Buddy system allocator
   - Slab allocation for small objects

3. **Persistence**
   - Memory map serialization
   - Checkpoint/restore support

4. **Hardware Features**
   - ECC memory support
   - Memory scrubbing

## Usage Examples

### Complete Bootloader Setup
```rust
use bootloader::{boot_start, memory_map, boot_heap};

#[no_mangle]
pub extern "C" fn _start() -> ! {
    // Set up panic handler
    std::panic::set_hook(Box::new(|info| {
        eprintln!("PANIC: {}", info);
    }));

    // Start bootloader
    boot_start()
}
```

### Custom Memory Configuration
```rust
fn custom_memory_setup() {
    let config = MemoryInitConfig {
        heap_size: 64 * 1024 * 1024, // 64MB heap
        bitmap_granularity: 8192,    // 8KB frames
        enable_memory_test: true,
        ..Default::default()
    };

    let memory_map = boot_heap::init_boot_memory(boot_mode, Some(config))
        .expect("Memory setup failed");
}
```

This comprehensive memory management system provides a solid foundation for the MultiOS bootloader, ensuring reliable memory initialization across different hardware configurations and boot environments.
