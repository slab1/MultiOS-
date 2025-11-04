//! Kernel Allocator
//! 
//! This module provides safe heap allocation for the kernel using Rust's GlobalAlloc trait.
//! It integrates with the physical memory manager to provide page-based allocation
//! and uses a linked list allocator for small allocations.

use crate::memory_types::*;
use crate::{MemoryError, MemoryResult};
use linked_list_allocator::LockedHeap;
use spin::Mutex;
use core::alloc::{GlobalAlloc, Layout};
use log::{info, debug, error};

/// Global kernel heap allocator
#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap = LockedHeap::new();

/// Kernel allocator with memory tracking
pub struct KernelAllocator {
    /// Heap start address
    heap_start: VirtAddr,
    /// Heap size
    heap_size: usize,
    /// Memory tracking
    allocated_bytes: usize,
    /// Allocation count tracking
    allocation_count: usize,
}

/// Heap statistics
#[derive(Debug, Clone, Copy)]
pub struct HeapStats {
    /// Total heap size in bytes
    pub total_size: usize,
    /// Currently allocated bytes
    pub allocated_bytes: usize,
    /// Number of active allocations
    pub allocation_count: usize,
    /// Available bytes for allocation
    pub available_bytes: usize,
    /// Memory utilization percentage
    pub utilization_percent: f32,
}

impl KernelAllocator {
    /// Create a new kernel allocator
    pub const fn new() -> Self {
        Self {
            heap_start: VirtAddr::new(0),
            heap_size: 0,
            allocated_bytes: 0,
            allocation_count: 0,
        }
    }

    /// Initialize the kernel allocator
    pub fn init(&mut self, heap_start: VirtAddr, heap_size: usize) -> MemoryResult<()> {
        info!("Initializing kernel allocator: {:x?} - {} bytes", heap_start, heap_size);
        
        self.heap_start = heap_start;
        self.heap_size = heap_size;
        
        // Safety: We need to initialize the global allocator
        unsafe {
            HEAP_ALLOCATOR.lock().init(heap_start.as_u64() as *mut u8, heap_size);
        }
        
        debug!("Kernel allocator initialized successfully");
        Ok(())
    }

    /// Allocate memory with specific layout
    pub fn allocate(&mut self, layout: Layout) -> MemoryResult<*mut u8> {
        let result = unsafe {
            HEAP_ALLOCATOR.alloc(layout)
        };
        
        if result.is_null() {
            return Err(MemoryError::AllocationFailed);
        }
        
        // Update tracking
        self.allocated_bytes += layout.size();
        self.allocation_count += 1;
        
        debug!("Allocated {} bytes at {:x?}", layout.size(), result as usize);
        Ok(result)
    }

    /// Allocate memory with automatic layout
    pub fn allocate_bytes(&mut self, size: usize) -> MemoryResult<*mut u8> {
        let layout = Layout::from_size_align(size, core::mem::align_of::<usize>())
            .map_err(|_| MemoryError::InvalidAddress)?;
        self.allocate(layout)
    }

    /// Deallocate memory
    pub fn deallocate(&mut self, ptr: *mut u8, layout: Layout) {
        unsafe {
            HEAP_ALLOCATOR.dealloc(ptr, layout);
        }
        
        // Update tracking
        if self.allocated_bytes >= layout.size() {
            self.allocated_bytes -= layout.size();
        }
        if self.allocation_count > 0 {
            self.allocation_count -= 1;
        }
        
        debug!("Deallocated {} bytes at {:x?}", layout.size(), ptr as usize);
    }

    /// Reallocate memory
    pub fn reallocate(&mut self, ptr: *mut u8, old_layout: Layout, new_size: usize) -> MemoryResult<*mut u8> {
        let new_layout = Layout::from_size_align(new_size, old_layout.align())
            .map_err(|_| MemoryError::InvalidAddress)?;
        
        let new_ptr = unsafe {
            HEAP_ALLOCATOR.alloc(new_layout)
        };
        
        if new_ptr.is_null() {
            return Err(MemoryError::AllocationFailed);
        }
        
        // Copy old data if both pointers are valid
        if !ptr.is_null() {
            unsafe {
                core::ptr::copy_nonoverlapping(
                    ptr,
                    new_ptr,
                    core::cmp::min(old_layout.size(), new_size)
                );
            }
            self.deallocate(ptr, old_layout);
        }
        
        // Update tracking
        let size_diff = new_size as isize - old_layout.size() as isize;
        if size_diff > 0 {
            self.allocated_bytes += size_diff as usize;
        } else if size_diff < 0 {
            let decrease = (-size_diff) as usize;
            if self.allocated_bytes >= decrease {
                self.allocated_bytes -= decrease;
            }
        }
        
        debug!("Reallocated {} -> {} bytes at {:x?}", old_layout.size(), new_size, new_ptr as usize);
        Ok(new_ptr)
    }

    /// Get current heap statistics
    pub fn get_stats(&self) -> HeapStats {
        let available_bytes = if self.heap_size > self.allocated_bytes {
            self.heap_size - self.allocated_bytes
        } else {
            0
        };
        
        let utilization_percent = if self.heap_size > 0 {
            (self.allocated_bytes as f32 / self.heap_size as f32) * 100.0
        } else {
            0.0
        };
        
        HeapStats {
            total_size: self.heap_size,
            allocated_bytes: self.allocated_bytes,
            allocation_count: self.allocation_count,
            available_bytes,
            utilization_percent,
        }
    }

    /// Check if allocator is initialized
    pub fn is_initialized(&self) -> bool {
        self.heap_size > 0
    }

    /// Reset allocator (for testing)
    pub fn reset(&mut self) {
        self.allocated_bytes = 0;
        self.allocation_count = 0;
    }
}

impl Default for KernelAllocator {
    fn default() -> Self {
        Self::new()
    }
}

/// Safe allocator wrapper that provides Rust-friendly interfaces
pub struct SafeAllocator {
    /// Internal kernel allocator
    allocator: Mutex<KernelAllocator>,
}

impl SafeAllocator {
    /// Create a new safe allocator
    pub const fn new() -> Self {
        Self {
            allocator: Mutex::new(KernelAllocator::new()),
        }
    }

    /// Initialize the allocator
    pub fn init(&self, heap_start: VirtAddr, heap_size: usize) -> MemoryResult<()> {
        let mut allocator = self.allocator.lock();
        allocator.init(heap_start, heap_size)
    }

    /// Allocate memory and return a boxed value
    pub fn allocate_box<T>(&self) -> MemoryResult<Box<T>> {
        let mut allocator = self.allocator.lock();
        let layout = Layout::new::<T>();
        let ptr = allocator.allocate(layout)?;
        
        Ok(unsafe { Box::from_raw(ptr as *mut T) })
    }

    /// Allocate memory and return a boxed slice
    pub fn allocate_box_slice<T>(&self, len: usize) -> MemoryResult<Box<[T]>> {
        let mut allocator = self.allocator.lock();
        let layout = Layout::array::<T>(len)
            .map_err(|_| MemoryError::InvalidAddress)?;
        let ptr = allocator.allocate(layout)?;
        
        let slice = unsafe { Box::from_raw(core::slice::from_raw_parts_mut(ptr as *mut T, len)) };
        Ok(slice)
    }

    /// Allocate zeroed memory for a type
    pub fn allocate_zeroed<T>(&self) -> MemoryResult<Box<T>> {
        let mut allocator = self.allocator.lock();
        let layout = Layout::new::<T>();
        let ptr = allocator.allocate(layout)?;
        
        unsafe {
            core::ptr::write_bytes(ptr, 0, layout.size());
            Ok(Box::from_raw(ptr as *mut T))
        }
    }

    /// Get allocator statistics
    pub fn get_stats(&self) -> HeapStats {
        let allocator = self.allocator.lock();
        allocator.get_stats()
    }

    /// Check if allocator is initialized
    pub fn is_initialized(&self) -> bool {
        let allocator = self.allocator.lock();
        allocator.is_initialized()
    }
}

impl Default for SafeAllocator {
    fn default() -> Self {
        Self::new()
    }
}

/// Pool allocator for frequently allocated objects of the same size
pub struct PoolAllocator<T> {
    /// Fixed-size memory pool
    pool: Vec<Option<Box<T>>>,
    /// Free object indices
    free_indices: Vec<usize>,
    /// Pool statistics
    total_objects: usize,
}

impl<T> PoolAllocator<T> {
    /// Create a new pool allocator
    pub fn with_capacity(capacity: usize) -> Self {
        let mut pool = Vec::with_capacity(capacity);
        let mut free_indices = Vec::with_capacity(capacity);
        
        for i in 0..capacity {
            pool.push(None);
            free_indices.push(i);
        }
        
        Self {
            pool,
            free_indices,
            total_objects: capacity,
        }
    }

    /// Allocate an object from the pool
    pub fn allocate(&mut self) -> MemoryResult<PoolObject<T>> {
        if let Some(index) = self.free_indices.pop() {
            Ok(PoolObject {
                pool: self,
                index,
                inner: None,
            })
        } else {
            Err(MemoryError::OutOfMemory)
        }
    }

    /// Get pool statistics
    pub fn get_stats(&self) -> PoolStats {
        PoolStats {
            total_objects: self.total_objects,
            allocated_objects: self.total_objects - self.free_indices.len(),
            free_objects: self.free_indices.len(),
            utilization_percent: if self.total_objects > 0 {
                ((self.total_objects - self.free_indices.len()) as f32 / self.total_objects as f32) * 100.0
            } else {
                0.0
            },
        }
    }
}

/// Pool object wrapper that automatically returns to pool on drop
pub struct PoolObject<'a, T> {
    /// Reference to pool allocator
    pool: &'a mut PoolAllocator<T>,
    /// Object index in pool
    index: usize,
    /// Inner object (None when allocated, Some when in use)
    inner: Option<Box<T>>,
}

impl<T> PoolObject<'_, T> {
    /// Get mutable reference to the object
    pub fn as_mut(&mut self) -> &mut T {
        if let Some(ref mut inner) = self.inner {
            inner
        } else {
            panic!("PoolObject is not initialized");
        }
    }

    /// Get immutable reference to the object
    pub fn as_ref(&self) -> &T {
        if let Some(ref inner) = self.inner {
            inner
        } else {
            panic!("PoolObject is not initialized");
        }
    }

    /// Take ownership of the object (removes it from pool)
    pub fn into_inner(self) -> Box<T> {
        if let Some(inner) = self.inner {
            // Note: This breaks the lifetime constraint but is safe in this context
            let index = self.index;
            drop(self);
            
            // Manually remove from pool
            self.pool.pool[index] = None;
            self.pool.free_indices.push(index);
            
            inner
        } else {
            panic!("PoolObject is not initialized");
        }
    }
}

impl<T> core::ops::Deref for PoolObject<'_, T> {
    type Target = T;
    
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<T> core::ops::DerefMut for PoolObject<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::T {
        self.as_mut()
    }
}

impl<T> Drop for PoolObject<'_, T> {
    fn drop(&mut self) {
        // Automatically return object to pool
        self.inner = None;
        self.pool.free_indices.push(self.index);
    }
}

/// Pool allocator statistics
#[derive(Debug, Clone, Copy)]
pub struct PoolStats {
    /// Total objects in pool
    pub total_objects: usize,
    /// Currently allocated objects
    pub allocated_objects: usize,
    /// Free objects available
    pub free_objects: usize,
    /// Pool utilization percentage
    pub utilization_percent: f32,
}

/// Simple bump allocator for temporary allocations
pub struct BumpAllocator {
    /// Next allocation address
    next: VirtAddr,
    /// Last allocation address
    last: VirtAddr,
    /// Total allocated bytes
    allocated: usize,
}

impl BumpAllocator {
    /// Create a new bump allocator
    pub fn new(start: VirtAddr, size: usize) -> Self {
        Self {
            next: start,
            last: start.offset(size as u64),
            allocated: 0,
        }
    }

    /// Allocate memory from the bump allocator
    pub fn allocate(&mut self, layout: Layout) -> MemoryResult<*mut u8> {
        let align = layout.align().saturating_sub(1);
        let current = self.next.as_u64() as usize;
        let aligned = (current + align) & !align;
        
        if (aligned + layout.size()) > self.last.as_u64() as usize {
            return Err(MemoryError::OutOfMemory);
        }
        
        let ptr = aligned as *mut u8;
        self.next = VirtAddr::new((aligned + layout.size()) as u64);
        self.allocated += layout.size();
        
        Ok(ptr)
    }

    /// Reset the bump allocator (only safe if all allocations are freed)
    pub fn reset(&mut self) {
        self.next = VirtAddr::new(self.last.as_u64() - self.allocated as u64);
        self.allocated = 0;
    }

    /// Get number of allocated bytes
    pub fn allocated_bytes(&self) -> usize {
        self.allocated
    }

    /// Check if allocator is exhausted
    pub fn is_exhausted(&self) -> bool {
        self.next.as_u64() >= self.last.as_u64()
    }
}

/// Initialize the global kernel allocator
pub fn init_global_allocator(heap_start: VirtAddr, heap_size: usize) -> MemoryResult<()> {
    info!("Initializing global kernel allocator...");
    
    // Initialize the locked heap allocator
    unsafe {
        HEAP_ALLOCATOR.lock().init(heap_start.as_u64() as *mut u8, heap_size);
    }
    
    info!("Global kernel allocator initialized: {} bytes at {:x?}", heap_size, heap_start);
    Ok(())
}

/// Create a new kernel allocator instance
pub fn create_kernel_allocator() -> KernelAllocator {
    KernelAllocator::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kernel_allocator_creation() {
        let allocator = KernelAllocator::new();
        assert_eq!(allocator.heap_size, 0);
        assert_eq!(allocator.allocated_bytes, 0);
        assert_eq!(allocator.allocation_count, 0);
        assert!(!allocator.is_initialized());
    }

    #[test]
    fn test_pool_allocator() {
        let mut pool = PoolAllocator::with_capacity(3);
        
        assert_eq!(pool.get_stats().total_objects, 3);
        assert_eq!(pool.get_stats().allocated_objects, 0);
        assert_eq!(pool.get_stats().free_objects, 3);
        
        // Allocate first object
        let mut obj1 = pool.allocate().unwrap();
        *obj1.as_mut() = 42i32;
        
        let stats1 = pool.get_stats();
        assert_eq!(stats1.allocated_objects, 1);
        assert_eq!(stats1.free_objects, 2);
        
        // Allocate second object
        let mut obj2 = pool.allocate().unwrap();
        *obj2.as_mut() = 24i32;
        
        let stats2 = pool.get_stats();
        assert_eq!(stats2.allocated_objects, 2);
        assert_eq!(stats2.free_objects, 1);
        
        // Drop first object (returns to pool)
        drop(obj1);
        
        let stats3 = pool.get_stats();
        assert_eq!(stats3.allocated_objects, 1);
        assert_eq!(stats3.free_objects, 2);
    }

    #[test]
    fn test_bump_allocator() {
        let mut bump = BumpAllocator::new(VirtAddr::new(0x1000), 0x1000);
        
        assert_eq!(bump.allocated_bytes(), 0);
        assert!(!bump.is_exhausted());
        
        let layout = Layout::from_size_align(16, 8).unwrap();
        let ptr1 = bump.allocate(layout).unwrap();
        assert_eq!(bump.allocated_bytes(), 16);
        
        let ptr2 = bump.allocate(layout).unwrap();
        assert_eq!(bump.allocated_bytes(), 32);
        
        assert_ne!(ptr1 as usize, ptr2 as usize);
    }

    #[test]
    fn test_heap_stats() {
        let stats = HeapStats {
            total_size: 1024,
            allocated_bytes: 256,
            allocation_count: 5,
            available_bytes: 768,
            utilization_percent: 25.0,
        };
        
        assert_eq!(stats.total_size, 1024);
        assert_eq!(stats.allocated_bytes, 256);
        assert_eq!(stats.utilization_percent, 25.0);
    }
}