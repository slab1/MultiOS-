//! MultiOS Memory Manager
//! 
//! This module provides comprehensive memory management functionality for the MultiOS kernel.
//! It includes virtual memory management with 4-level paging, physical memory allocation,
//! heap management, and memory safety using Rust's ownership model.
//! 
//! # Features
//! 
//! - **Virtual Memory Management**: 4-level paging support for x86_64, ARM64, and RISC-V
//! - **Physical Memory Allocation**: Efficient page-based physical memory management
//! - **Safe Heap Allocation**: Rust-compatible allocator with memory tracking
//! - **Memory Protection**: Fine-grained memory permission control
//! - **Page Fault Handling**: Architecture-specific page fault handling
//! - **Cross-Platform**: Unified API across multiple architectures
//! 
//! # Usage
//! 
//! ```rust
//! use multios_memory_manager::*;
//! 
//! // Initialize memory management
//! let result = init();
//! if let Err(e) = result {
//!     panic!("Memory init failed: {:?}", e);
//! }
//! 
//! // Allocate physical memory
//! let phys_addr = allocate_physical_page()?;
//! 
//! // Map virtual memory
//! let virt_addr = VirtAddr::new(0x1000);
//! map_memory(virt_addr, phys_addr, 4096, MemoryFlags::kernel_rw())?;
//! ```
//! 
//! # Architecture Support
//! 
//! - **x86_64**: Full 4-level paging support with huge pages
//! - **ARM64**: 4-level paging with ARM-specific features
//! - **RISC-V**: Sv39/Sv48 paging support
//! 
//! This memory manager provides a safe, high-performance foundation for
//! the MultiOS kernel's memory management needs.

#![no_std]
#![feature(allocator_api)]
#![feature(const_option_ext)]
#![feature(core_intrinsics)]
#![feature(ptr_as_ref)]
#![allow(clippy::missing_safety_doc)]

extern crate alloc;
extern crate spin;
extern crate bitflags;

// Feature-gated imports
#[cfg(feature = "x86_64")]
extern crate x86_64;

// Public modules
pub mod memory_types;
pub mod physical_memory;
pub mod virtual_memory;
pub mod allocator;
pub mod arch_specific;
pub mod numa;
pub mod cache_coherency;
pub mod large_scale_vm;

#[cfg(test)]
pub mod tests;

// Re-export public types
pub use memory_types::*;
pub use physical_memory::*;
pub use virtual_memory::*;
pub use allocator::*;
pub use arch_specific::*;
pub use numa::*;
pub use cache_coherency::*;
pub use large_scale_vm::*;

use log::{info, debug, warn, error};

/// Result type for all memory operations
pub type MemoryResult<T> = Result<T, MemoryError>;

/// Memory manager initialization context
#[derive(Debug, Clone)]
pub struct MemoryInitContext {
    /// Physical memory map from bootloader
    pub memory_map: Vec<MemoryMapEntry>,
    /// Kernel start address
    pub kernel_start: PhysAddr,
    /// Kernel end address
    pub kernel_end: PhysAddr,
    /// Physical memory offset (where kernel is loaded)
    pub physical_offset: PhysAddr,
    /// Target architecture
    pub target_arch: arch_specific::Architecture,
}

/// Global memory manager state
static MEMORY_MANAGER: Mutex<Option<MemoryManager>> = Mutex::new(None);

/// Main memory management structure
pub struct MemoryManager {
    /// Physical memory manager
    physical_manager: PhysicalMemoryManager,
    /// Virtual memory manager
    virtual_manager: arch_specific::ArchManager,
    /// Heap allocator
    heap_allocator: allocator::SafeAllocator,
    /// Memory statistics
    stats: MemoryStats,
    /// Initialized flag
    initialized: bool,
}

impl MemoryManager {
    /// Create a new memory manager
    pub fn new(context: MemoryInitContext) -> Self {
        info!("Creating memory manager for {:?}", context.target_arch);
        
        // Create architecture-specific components
        let virtual_manager = arch_specific::create_arch_manager(context.target_arch)
            .expect("Failed to create architecture manager");
        
        Self {
            physical_manager: PhysicalMemoryManager::new(),
            virtual_manager,
            heap_allocator: allocator::SafeAllocator::new(),
            stats: MemoryStats::default(),
            initialized: false,
        }
    }

    /// Initialize the memory manager
    pub fn init(&mut self, context: &MemoryInitContext) -> MemoryResult<()> {
        if self.initialized {
            return Err(MemoryError::AllocationFailed);
        }

        info!("Initializing MultiOS memory manager...");
        info!("Architecture: {:?}", context.target_arch);
        info!("Physical memory offset: {:?}", context.physical_offset);
        info!("Kernel range: {:?} - {:?}", context.kernel_start, context.kernel_end);

        // Initialize physical memory manager
        info!("Initializing physical memory manager...");
        self.physical_manager.init(&context.memory_map);
        
        // Initialize virtual memory manager
        info!("Initializing virtual memory manager...");
        self.initialize_virtual_memory(context)?;
        
        // Initialize heap allocator
        info!("Initializing heap allocator...");
        self.initialize_heap(context)?;
        
        // Create kernel mappings
        info!("Creating kernel memory mappings...");
        self.create_kernel_mappings(context)?;
        
        // Update statistics
        self.update_statistics();
        
        self.initialized = true;
        info!("Memory manager initialized successfully!");
        self.print_statistics();
        
        Ok(())
    }

    /// Initialize virtual memory management
    fn initialize_virtual_memory(&mut self, context: &MemoryInitContext) -> MemoryResult<()> {
        // Map kernel code and data
        let kernel_size = (context.kernel_end.as_u64() - context.kernel_start.as_u64()) as usize;
        let kernel_flags = MemoryFlags::kernel_ro();
        
        self.virtual_manager.mapper_mut().map_page(
            VirtAddr::new(context.kernel_start.as_u64()),
            context.kernel_start,
            kernel_size,
            kernel_flags,
        )?;

        // Map physical memory
        self.map_physical_memory(context)?;
        
        Ok(())
    }

    /// Map physical memory regions
    fn map_physical_memory(&mut self, context: &MemoryInitContext) -> MemoryResult<()> {
        // Map first few GB of physical memory for kernel use
        let phys_mem_size = self.physical_manager.get_stats().total_memory as usize;
        let map_size = core::cmp::min(phys_mem_size, 0x40000000); // 1GB max
        
        let virt_start = VirtAddr::new(context.physical_offset.as_u64());
        let phys_start = PhysAddr::new(0);
        
        self.virtual_manager.mapper_mut().map_page(
            virt_start,
            phys_start,
            map_size,
            MemoryFlags::kernel_rw(),
        )?;

        debug!("Mapped physical memory: {:?} -> {:?}", phys_start, virt_start);
        Ok(())
    }

    /// Initialize heap allocator
    fn initialize_heap(&mut self, context: &MemoryInitContext) -> MemoryResult<()> {
        // Place heap after kernel
        let heap_start = context.kernel_end.align_up(PageSize::Size4K);
        let heap_size = 16 * 1024 * 1024; // 16MB initial heap
        
        debug!("Initializing heap: {:?} ({} bytes)", heap_start, heap_size);
        
        self.heap_allocator.init(heap_start, heap_size)?;
        
        // Also initialize global allocator
        allocator::init_global_allocator(heap_start, heap_size)?;
        
        Ok(())
    }

    /// Create essential kernel memory mappings
    fn create_kernel_mappings(&mut self, context: &MemoryInitContext) -> MemoryResult<()> {
        // Map kernel stack
        let stack_size = 8 * 1024 * 1024; // 8MB stack
        let stack_start = VirtAddr::new(0xFFFF_FF00_0000_0000); // High kernel addresses
        
        // Allocate physical pages for stack
        let stack_phys = self.physical_manager.allocate_pages(stack_size / PageSize::Size4K.as_usize())?;
        
        // Map stack pages
        self.virtual_manager.mapper_mut().map_page(
            stack_start,
            stack_phys.to_phys_addr(PageSize::Size4K),
            stack_size,
            MemoryFlags::kernel_rw(),
        )?;

        debug!("Mapped kernel stack: {:?} -> {:?}", stack_start, stack_phys);
        Ok(())
    }

    /// Update memory statistics
    fn update_statistics(&mut self) {
        self.stats = MemoryStats {
            total_memory: self.physical_manager.get_stats().total_memory,
            used_memory: self.physical_manager.get_stats().used_memory,
            available_memory: self.physical_manager.get_stats().available_memory,
            total_pages: self.physical_manager.get_stats().total_pages,
            used_pages: self.physical_manager.get_stats().used_pages,
            free_pages: self.physical_manager.get_stats().free_pages,
            reserved_pages: self.physical_manager.get_stats().reserved_pages,
        };
    }

    /// Print memory statistics
    fn print_statistics(&self) {
        let heap_stats = self.heap_allocator.get_stats();
        
        info!("Memory Manager Statistics:");
        info!("  Physical Memory: {} MB total, {} MB used, {} MB free", 
              self.stats.total_memory / 1024 / 1024,
              self.stats.used_memory / 1024 / 1024,
              self.stats.available_memory / 1024 / 1024);
        info!("  Pages: {} total, {} used, {} free, {} reserved", 
              self.stats.total_pages,
              self.stats.used_pages,
              self.stats.free_pages,
              self.stats.reserved_pages);
        info!("  Heap: {} MB total, {} MB allocated ({}%), {} allocations", 
              heap_stats.total_size / 1024 / 1024,
              heap_stats.allocated_bytes / 1024 / 1024,
              heap_stats.utilization_percent,
              heap_stats.allocation_count);
    }

    /// Allocate physical memory
    pub fn allocate_physical(&mut self) -> MemoryResult<PhysAddr> {
        let frame = self.physical_manager.allocate_page()?;
        Ok(self.physical_manager.frame_to_addr(frame))
    }

    /// Map virtual memory
    pub fn map_virtual(&mut self, virt_addr: VirtAddr, phys_addr: PhysAddr, size: usize, flags: MemoryFlags) -> MemoryResult<()> {
        self.virtual_manager.mapper_mut().map_page(virt_addr, phys_addr, size, flags)
    }

    /// Translate virtual to physical address
    pub fn translate(&self, virt_addr: VirtAddr) -> MemoryResult<PhysAddr> {
        self.virtual_manager.mapper().translate(virt_addr)
    }

    /// Handle page fault
    pub fn handle_page_fault(&mut self, fault_info: PageFaultInfo) -> MemoryResult<()> {
        self.virtual_manager.handle_page_fault(fault_info)
    }

    /// Get memory statistics
    pub fn get_stats(&self) -> MemoryStats {
        self.stats
    }

    /// Check if initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Get architecture information
    pub fn get_arch_info(&self) -> &arch_specific::ArchIdInfo {
        self.virtual_manager.get_arch_info()
    }

    /// Get heap allocator
    pub fn heap_allocator(&self) -> &allocator::SafeAllocator {
        &self.heap_allocator
    }

    /// Get physical memory manager
    pub fn physical_manager(&self) -> &PhysicalMemoryManager {
        &self.physical_manager
    }

    /// Get virtual memory manager
    pub fn virtual_manager(&self) -> &arch_specific::ArchManager {
        &self.virtual_manager
    }
}

/// Initialize the global memory management system
/// 
/// This function sets up the entire memory management subsystem for the kernel.
/// It must be called early in the kernel startup process.
/// 
/// # Arguments
/// 
/// * `context` - Initialization context with memory map and configuration
/// 
/// # Returns
/// 
/// Returns `MemoryResult<()>` indicating success or failure.
pub fn init(context: MemoryInitContext) -> MemoryResult<()> {
    info!("Initializing MultiOS Memory Manager...");
    
    let mut manager = MemoryManager::new(context.clone());
    manager.init(&context)?;
    
    *MEMORY_MANAGER.lock() = Some(manager);
    
    info!("Global memory manager initialized successfully!");
    Ok(())
}

/// Get the global memory manager
/// 
/// # Safety
/// 
/// This function is unsafe because it returns a reference to global state
/// that could be accessed from multiple threads. The memory manager must
/// be initialized before calling this function.
pub unsafe fn get_manager() -> MemoryResult<spin::MutexGuard<'static, Option<MemoryManager>>> {
    MEMORY_MANAGER.lock().as_ref()
        .ok_or(MemoryError::AllocationFailed)
        .map(|_| MEMORY_MANAGER.lock())
        .and_then(|guard| {
            guard.as_ref().ok_or(MemoryError::AllocationFailed)
                .map(|_| guard)
        })
}

/// High-level memory allocation interface
pub mod alloc_helpers {
    use super::*;

    /// Allocate and zero-initialize a type
    pub fn allocate_zeroed<T>() -> MemoryResult<Box<T>> {
        unsafe {
            let manager = get_manager()?;
            Ok(Box::new_zeroed().assume_init())
        }
    }

    /// Allocate a slice of a type
    pub fn allocate_slice<T>(len: usize) -> MemoryResult<Box<[T]>> {
        unsafe {
            let manager = get_manager()?;
            let layout = core::alloc::Layout::array::<T>(len)
                .map_err(|_| MemoryError::InvalidAddress)?;
            let ptr = manager.heap_allocator().allocator.lock().allocate(layout)?;
            Ok(Box::from_raw(core::slice::from_raw_parts_mut(ptr as *mut T, len)))
        }
    }

    /// Allocate a string with specified capacity
    pub fn allocate_string_with_capacity(capacity: usize) -> MemoryResult<alloc::string::String> {
        unsafe {
            let manager = get_manager()?;
            let mut string = alloc::string::String::with_capacity(capacity);
            // Pre-allocate internal buffer if needed
            if capacity > string.capacity() {
                string.reserve(capacity - string.capacity());
            }
            Ok(string)
        }
    }
}

/// Memory safety utilities
pub mod safety {
    use super::*;

    /// Validate memory address range
    pub fn validate_address_range(start: VirtAddr, size: usize) -> MemoryResult<()> {
        if size == 0 {
            return Ok(());
        }

        let end = start.offset(size as u64);
        
        // Check for overflow
        if end.as_u64() < start.as_u64() {
            return Err(MemoryError::InvalidAddress);
        }

        Ok(())
    }

    /// Check if memory region is properly aligned
    pub fn check_alignment(addr: VirtAddr, page_size: PageSize) -> bool {
        addr.is_aligned(page_size)
    }

    /// Validate memory flags
    pub fn validate_flags(flags: MemoryFlags) -> MemoryResult<()> {
        // Basic validation - at least read permission required
        if !flags.is_readable() {
            return Err(MemoryError::InvalidAddress);
        }

        Ok(())
    }
}

/// Performance monitoring
pub mod perf {
    use super::*;
    use core::time::Duration;
    use spin::Mutex;

    static ALLOCATION_COUNTER: Mutex<usize> = Mutex::new(0);
    static FAULT_COUNTER: Mutex<usize> = Mutex::new(0);
    static LAST_ALLOCATION_TIME: Mutex<Option<Duration>> = Mutex::new(None);

    /// Increment allocation counter
    pub fn record_allocation() {
        let mut counter = ALLOCATION_COUNTER.lock();
        *counter += 1;
    }

    /// Increment fault counter
    pub fn record_fault() {
        let mut counter = FAULT_COUNTER.lock();
        *counter += 1;
    }

    /// Get allocation count
    pub fn get_allocation_count() -> usize {
        *ALLOCATION_COUNTER.lock()
    }

    /// Get fault count
    pub fn get_fault_count() -> usize {
        *FAULT_COUNTER.lock()
    }
}

/// Test utilities
#[cfg(test)]
pub mod test_utils {
    use super::*;

    /// Create a test memory manager
    pub fn create_test_manager() -> MemoryResult<MemoryManager> {
        let context = MemoryInitContext {
            memory_map: vec![
                super::super::kernel::MemoryMapEntry {
                    base: 0,
                    size: 0x1000,
                    entry_type: super::super::kernel::MemoryType::Usable,
                },
            ],
            kernel_start: PhysAddr::new(0x100000),
            kernel_end: PhysAddr::new(0x200000),
            physical_offset: PhysAddr::new(0),
            target_arch: arch_specific::Architecture::X86_64,
        };

        let mut manager = MemoryManager::new(context);
        manager.init(&context)?;
        Ok(manager)
    }

    /// Create test memory mapping
    pub fn create_test_mapping() -> (VirtAddr, PhysAddr, MemoryFlags) {
        (
            VirtAddr::new(0x1000),
            PhysAddr::new(0x100000),
            MemoryFlags::kernel_rw(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_init_context() {
        let context = MemoryInitContext {
            memory_map: Vec::new(),
            kernel_start: PhysAddr::new(0x100000),
            kernel_end: PhysAddr::new(0x200000),
            physical_offset: PhysAddr::new(0),
            target_arch: arch_specific::Architecture::X86_64,
        };

        assert_eq!(context.kernel_start.as_u64(), 0x100000);
        assert_eq!(context.kernel_end.as_u64(), 0x200000);
        assert_eq!(context.target_arch, arch_specific::Architecture::X86_64);
    }

    #[test]
    fn test_memory_result_handling() {
        let result: MemoryResult<i32> = Ok(42);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);

        let error_result: MemoryResult<i32> = Err(MemoryError::OutOfMemory);
        assert!(error_result.is_err());
        assert_eq!(error_result.unwrap_err(), MemoryError::OutOfMemory);
    }

    #[test]
    fn test_safety_validation() {
        let valid_addr = VirtAddr::new(0x1000);
        let result = safety::validate_address_range(valid_addr, 4096);
        assert!(result.is_ok());

        let overflow_result = safety::validate_address_range(VirtAddr::new(u64::MAX), 1);
        assert!(overflow_result.is_err());
    }
}