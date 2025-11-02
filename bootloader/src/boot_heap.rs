//! Boot Heap Management for MultiOS Bootloader
//! 
//! This module provides safe Rust interfaces for heap allocation during
//! the bootloader phase, supporting both UEFI and BIOS environments.

use crate::memory_map::{MemoryMap, BootHeap, MemoryInitConfig, MemoryManagementStats};
use crate::{BootError, BootResult};
use log::{info, warn, error, debug};
use spin::Mutex;
use core::alloc::{GlobalAlloc, Layout};
use core::ptr::NonNull;

/// Safe heap allocator wrapper
#[derive(Debug)]
pub struct BootHeapAllocator {
    memory_map: Option<MemoryMap>,
    initialized: bool,
}

impl BootHeapAllocator {
    /// Create new boot heap allocator
    pub fn new() -> Self {
        Self {
            memory_map: None,
            initialized: false,
        }
    }

    /// Initialize the heap allocator with memory map
    pub fn init(&mut self, memory_map: MemoryMap) -> BootResult<()> {
        if self.initialized {
            warn!("Heap allocator already initialized");
            return Ok(());
        }

        self.memory_map = Some(memory_map);
        self.initialized = true;
        
        info!("Boot heap allocator initialized successfully");
        Ok(())
    }

    /// Check if allocator is initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized && self.memory_map.is_some()
    }

    /// Allocate memory from boot heap
    pub fn allocate(&mut self, size: usize, align: usize) -> BootResult<NonNull<u8>> {
        if !self.is_initialized() {
            return Err(BootError::HeapInitializationError);
        }

        if let Some(ref mut memory_map) = self.memory_map {
            let addr = memory_map.heap_allocate(size)?;
            let ptr = addr as *mut u8;
            
            if ptr.is_null() {
                return Err(BootError::OutOfMemory);
            }

            debug!("Heap allocated: {} bytes at {:#x}", size, addr);
            unsafe {
                Ok(NonNull::new_unchecked(ptr))
            }
        } else {
            Err(BootError::HeapInitializationError)
        }
    }

    /// Deallocate memory (not implemented for boot heap)
    pub fn deallocate(&mut self, _ptr: NonNull<u8>, _layout: Layout) -> BootResult<()> {
        // Boot heap doesn't support deallocation
        debug!("Deallocation requested but not supported in boot heap");
        Ok(())
    }

    /// Get heap statistics
    pub fn get_stats(&self) -> BootResult<HeapStats> {
        if let Some(ref memory_map) = self.memory_map {
            let stats = memory_map.get_memory_stats();
            Ok(stats.heap_stats)
        } else {
            Err(BootError::HeapInitializationError)
        }
    }

    /// Get memory management statistics
    pub fn get_memory_stats(&self) -> BootResult<MemoryManagementStats> {
        if let Some(ref memory_map) = self.memory_map {
            Ok(memory_map.get_memory_stats())
        } else {
            Err(BootError::HeapInitializationError)
        }
    }

    /// Reset heap allocator
    pub fn reset(&mut self) -> BootResult<()> {
        if let Some(ref memory_map) = self.memory_map {
            // Reinitialize memory map
            let mut new_memory_map = memory_map.clone();
            new_memory_map.init_complete()?;
            self.memory_map = Some(new_memory_map);
            
            info!("Heap allocator reset successfully");
            Ok(())
        } else {
            Err(BootError::HeapInitializationError)
        }
    }
}

impl Default for BootHeapAllocator {
    fn default() -> Self {
        Self::new()
    }
}

/// Global boot heap allocator instance
static BOOT_HEAP_ALLOCATOR: Mutex<BootHeapAllocator> = Mutex::new(BootHeapAllocator::new());

/// Safe memory allocation interface
pub mod safe_alloc {
    use super::*;
    use core::ptr::NonNull;

    /// Safely allocate memory with specified size and alignment
    pub fn alloc(size: usize, align: usize) -> Result<NonNull<u8>, BootError> {
        let mut allocator = BOOT_HEAP_ALLOCATOR.lock();
        allocator.allocate(size, align)
    }

    /// Allocate zeroed memory
    pub fn alloc_zeroed(size: usize, align: usize) -> Result<NonNull<u8>, BootError> {
        let ptr = alloc(size, align)?;
        unsafe {
            core::ptr::write_bytes(ptr.as_ptr(), 0, size);
        }
        Ok(ptr)
    }

    /// Get allocation statistics
    pub fn get_stats() -> Result<HeapStats, BootError> {
        let allocator = BOOT_HEAP_ALLOCATOR.lock();
        allocator.get_stats()
    }

    /// Check if allocator is available
    pub fn is_available() -> bool {
        let allocator = BOOT_HEAP_ALLOCATOR.lock();
        allocator.is_initialized()
    }

    /// Deallocate memory (not supported in boot heap)
    pub fn dealloc(_ptr: NonNull<u8>, _layout: Layout) -> Result<(), BootError> {
        // Boot heap doesn't support deallocation
        Ok(())
    }
}

/// Memory pool for frequently used allocations
#[derive(Debug)]
pub struct MemoryPool {
    start_addr: u64,
    size: usize,
    used: usize,
    block_size: usize,
    block_count: usize,
    free_blocks: Vec<usize>,
}

impl MemoryPool {
    /// Create new memory pool
    pub fn new(start_addr: u64, size: usize, block_size: usize) -> BootResult<Self> {
        if block_size == 0 || size < block_size {
            return Err(BootError::InvalidKernelFormat);
        }

        let block_count = size / block_size;
        let mut free_blocks = Vec::with_capacity(block_count);
        
        for i in 0..block_count {
            free_blocks.push(i);
        }

        Ok(Self {
            start_addr,
            size,
            used: 0,
            block_size,
            block_count,
            free_blocks,
        })
    }

    /// Allocate block from pool
    pub fn allocate_block(&mut self) -> BootResult<u64> {
        if let Some(block_idx) = self.free_blocks.pop() {
            let addr = self.start_addr + (block_idx as u64) * self.block_size as u64;
            self.used += self.block_size;
            Ok(addr)
        } else {
            Err(BootError::OutOfMemory)
        }
    }

    /// Free block back to pool
    pub fn free_block(&mut self, addr: u64) -> BootResult<()> {
        if addr < self.start_addr || addr >= self.start_addr + self.size as u64 {
            return Err(BootError::InvalidKernelFormat);
        }

        let block_idx = ((addr - self.start_addr) / self.block_size as u64) as usize;
        if block_idx < self.block_count {
            self.free_blocks.push(block_idx);
            self.used -= self.block_size;
            Ok(())
        } else {
            Err(BootError::InvalidKernelFormat)
        }
    }

    /// Get pool statistics
    pub fn get_stats(&self) -> PoolStats {
        PoolStats {
            total_size: self.size,
            used_size: self.used,
            free_blocks: self.free_blocks.len(),
            total_blocks: self.block_count,
            utilization_percent: if self.size > 0 {
                (self.used * 100) / self.size
            } else {
                0
            },
        }
    }
}

/// Memory pool statistics
#[derive(Debug, Clone, Copy)]
pub struct PoolStats {
    pub total_size: usize,
    pub used_size: usize,
    pub free_blocks: usize,
    pub total_blocks: usize,
    pub utilization_percent: usize,
}

impl fmt::Display for PoolStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Memory Pool Statistics:")?;
        writeln!(f, "  Total Size: {} KB", self.total_size / 1024)?;
        writeln!(f, "  Used Size: {} KB", self.used_size / 1024)?;
        writeln!(f, "  Free Blocks: {}", self.free_blocks)?;
        writeln!(f, "  Total Blocks: {}", self.total_blocks)?;
        writeln!(f, "  Utilization: {:.1}%", self.utilization_percent as f64)?;
        Ok(())
    }
}

/// Initialize boot heap memory management
pub fn init_boot_memory(boot_mode: crate::BootMode, config: Option<MemoryInitConfig>) -> BootResult<MemoryMap> {
    let init_config = config.unwrap_or_default();
    
    info!("Initializing boot memory management...");
    info!("Configuration: heap_size={}MB, granularity={}B, alignment={}B",
          init_config.heap_size / (1024 * 1024),
          init_config.bitmap_granularity,
          init_config.heap_alignment);

    // Create memory map based on boot mode
    let mut memory_map = match boot_mode {
        crate::BootMode::UEFI => {
            info!("Creating UEFI memory map...");
            MemoryMap::detect_uefi_memory()?
        }
        crate::BootMode::LegacyBIOS => {
            info!("Creating BIOS memory map...");
            MemoryMap::detect_bios_memory()?
        }
        crate::BootMode::Unknown => {
            warn!("Unknown boot mode, using fallback memory detection");
            let mut map = MemoryMap::with_config(init_config);
            // Add basic memory regions for unknown boot mode
            map.add_region(MemoryMap::default_region(0x100000, 0x3FF00000));
            map
        }
    };

    // Apply configuration
    memory_map.init_config = init_config;

    // Complete memory initialization
    memory_map.init_complete()?;

    // Initialize global heap allocator
    {
        let mut allocator = BOOT_HEAP_ALLOCATOR.lock();
        allocator.init(memory_map)?;
    }

    info!("Boot memory management initialized successfully");
    
    Ok(get_memory_map().unwrap())
}

/// Get current memory map
pub fn get_memory_map() -> Option<MemoryMap> {
    let allocator = BOOT_HEAP_ALLOCATOR.lock();
    allocator.memory_map.clone()
}

/// Update memory map
pub fn update_memory_map(new_map: MemoryMap) {
    let mut allocator = BOOT_HEAP_ALLOCATOR.lock();
    allocator.memory_map = Some(new_map);
}

/// Memory region factory for creating standard regions
impl MemoryMap {
    /// Create default memory region
    pub fn default_region(start: usize, size: usize) -> MemoryRegionInfo {
        use x86_64::PhysAddr;
        use crate::memory_map::{MemoryType, MemoryFlags};
        
        MemoryRegionInfo::new(
            PhysAddr::new(start as u64),
            size,
            MemoryType::Usable,
            MemoryFlags::READ | MemoryFlags::WRITE | MemoryFlags::AVAILABLE,
        )
    }
}

/// Allocation profiling and debugging
#[cfg(feature = "debug_mode")]
pub mod profiling {
    use super::*;
    use std::collections::HashMap;

    static ALLOCATION_TRACKER: Mutex<HashMap<u64, AllocationInfo>> = Mutex::new(HashMap::new());
    static ALLOCATION_STATS: Mutex<AllocationStats> = Mutex::new(AllocationStats::default());

    #[derive(Debug, Clone)]
    pub struct AllocationInfo {
        pub size: usize,
        pub align: usize,
        pub timestamp: u64,
    }

    #[derive(Debug, Default, Clone)]
    pub struct AllocationStats {
        pub total_allocations: usize,
        pub total_freed: usize,
        pub current_allocations: usize,
        pub total_bytes_allocated: usize,
        pub peak_bytes_allocated: usize,
    }

    /// Track allocation
    pub fn track_allocation(addr: u64, size: usize, align: usize) {
        let mut tracker = ALLOCATION_TRACKER.lock();
        let mut stats = ALLOCATION_STATS.lock();
        
        let info = AllocationInfo {
            size,
            align,
            timestamp: get_timestamp(),
        };
        
        tracker.insert(addr, info);
        stats.total_allocations += 1;
        stats.current_allocations += 1;
        stats.total_bytes_allocated += size;
        
        if stats.total_bytes_allocated > stats.peak_bytes_allocated {
            stats.peak_bytes_allocated = stats.total_bytes_allocated;
        }
    }

    /// Track deallocation
    pub fn track_deallocation(addr: u64) {
        let mut tracker = ALLOCATION_TRACKER.lock();
        let mut stats = ALLOCATION_STATS.lock();
        
        if let Some(info) = tracker.remove(&addr) {
            stats.total_freed += 1;
            stats.current_allocations -= 1;
            stats.total_bytes_allocated -= info.size;
        }
    }

    /// Get allocation statistics
    pub fn get_allocation_stats() -> AllocationStats {
        ALLOCATION_STATS.lock().clone()
    }

    /// Get current timestamp (simplified)
    fn get_timestamp() -> u64 {
        // In real implementation, would get from system timer
        0
    }
}

/// Error handling and recovery
pub mod recovery {
    use super::*;
    
    /// Attempt to recover from memory allocation failure
    pub fn recover_from_allocation_failure() -> BootResult<()> {
        warn!("Attempting memory allocation recovery...");
        
        // Try to reset heap allocator
        let mut allocator = BOOT_HEAP_ALLOCATOR.lock();
        if allocator.is_initialized() {
            allocator.reset()?;
            info!("Heap allocator reset successfully");
            Ok(())
        } else {
            error!("Cannot recover: heap allocator not initialized");
            Err(BootError::HeapInitializationError)
        }
    }

    /// Force garbage collection (not implemented for boot heap)
    pub fn force_gc() -> BootResult<usize> {
        warn!("Garbage collection not supported in boot heap");
        Ok(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boot_heap_allocator_creation() {
        let allocator = BootHeapAllocator::new();
        assert!(!allocator.is_initialized());
    }

    #[test]
    fn test_memory_pool_creation() {
        let pool = MemoryPool::new(0x100000, 0x10000, 0x1000).unwrap();
        let stats = pool.get_stats();
        
        assert_eq!(stats.total_size, 0x10000);
        assert_eq!(stats.free_blocks, 16); // 0x10000 / 0x1000 = 16
    }

    #[test]
    fn test_memory_pool_allocation() {
        let mut pool = MemoryPool::new(0x100000, 0x10000, 0x1000).unwrap();
        
        let addr = pool.allocate_block().unwrap();
        assert_eq!(addr, 0x100000);
        
        let stats = pool.get_stats();
        assert_eq!(stats.used_size, 0x1000);
        assert_eq!(stats.free_blocks, 15);
    }

    #[test]
    fn test_allocation_interface() {
        assert!(!safe_alloc::is_available());
        
        // This should fail since allocator is not initialized
        let result = safe_alloc::alloc(1024, 8);
        assert!(result.is_err());
    }
}
