//! Physical Memory Management
//! 
//! This module provides physical memory allocation and management functionality.
//! It handles physical page allocation, memory map initialization, and physical
//! memory tracking for the entire system.

use crate::memory_types::*;
use crate::{MemoryError, MemoryResult};
use spin::Mutex;
use log::{info, warn, error, debug};

/// Global physical memory manager instance
static PHYSICAL_MEMORY_MANAGER: Mutex<Option<PhysicalMemoryManager>> = Mutex::new(None);

/// Physical memory manager
pub struct PhysicalMemoryManager {
    /// Total number of physical pages
    total_pages: usize,
    /// Free page allocator
    free_pages: PageFrameAllocator,
    /// Reserved memory regions
    reserved_regions: Vec<MemoryRegionDescriptor>,
    /// Memory statistics
    stats: MemoryStats,
}

/// Memory region descriptor
#[derive(Debug, Clone)]
struct MemoryRegionDescriptor {
    /// Physical address of region start
    start: PhysAddr,
    /// Physical address of region end
    end: PhysAddr,
    /// Region type
    region_type: MemoryRegion,
}

/// Simple page frame allocator using a bitmap
pub struct PageFrameAllocator {
    /// Bitmap tracking free/used pages
    free_bitmap: Vec<u8>,
    /// Total number of pages managed
    total_pages: usize,
    /// Index of first free page
    first_free: usize,
}

impl PageFrameAllocator {
    /// Create a new page frame allocator
    pub const fn new() -> Self {
        Self {
            free_bitmap: Vec::new(),
            total_pages: 0,
            first_free: 0,
        }
    }

    /// Initialize the allocator with a memory map
    pub fn init(&mut self, memory_map: &[(PhysAddr, usize, MemoryRegion)]) {
        self.total_pages = memory_map.iter()
            .map(|(_, size, _)| *size / PageSize::Size4K.as_usize())
            .sum();
        
        let bitmap_size = (self.total_pages + 7) / 8;
        self.free_bitmap = vec![0xFF; bitmap_size];
        
        // Mark usable pages as free
        for (phys_addr, size, region_type) in memory_map {
            if *region_type == MemoryRegion::Usable {
                let start_page = (*phys_addr / PageSize::Size4K.as_usize()) as usize;
                let num_pages = size / PageSize::Size4K.as_usize();
                
                for i in 0..num_pages {
                    self.clear_bit(start_page + i);
                }
            }
        }
        
        debug!("Physical memory allocator initialized: {} pages total", self.total_pages);
    }

    /// Allocate a single page frame
    pub fn allocate_frame(&mut self) -> Option<PageFrame> {
        // Simple first-fit allocation
        let mut page_index = self.first_free;
        
        while page_index < self.total_pages {
            if self.is_bit_set(page_index) {
                self.clear_bit(page_index);
                self.first_free = page_index + 1;
                return Some(PageFrame::new(page_index));
            }
            page_index += 1;
        }
        
        // If we reach here, no free pages found
        // Try to find free pages from the beginning
        page_index = 0;
        while page_index < self.total_pages {
            if self.is_bit_set(page_index) {
                self.clear_bit(page_index);
                self.first_free = page_index + 1;
                return Some(PageFrame::new(page_index));
            }
            page_index += 1;
        }
        
        None
    }

    /// Allocate multiple contiguous page frames
    pub fn allocate_contiguous_frames(&mut self, count: usize) -> Option<PageFrame> {
        if count == 0 {
            return None;
        }
        
        let mut page_index = 0;
        
        while page_index <= self.total_pages - count {
            // Check if we have 'count' consecutive free pages
            let mut found = true;
            for i in 0..count {
                if !self.is_bit_set(page_index + i) {
                    found = false;
                    page_index += i + 1;
                    break;
                }
            }
            
            if found {
                // Allocate all pages
                for i in 0..count {
                    self.clear_bit(page_index + i);
                }
                self.first_free = page_index + count;
                return Some(PageFrame::new(page_index));
            }
        }
        
        None
    }

    /// Free a page frame
    pub fn free_frame(&mut self, frame: PageFrame) {
        let index = frame.as_usize();
        if index < self.total_pages {
            self.set_bit(index);
            
            // Update first_free if we freed an earlier page
            if index < self.first_free {
                self.first_free = index;
            }
        }
    }

    /// Get number of free pages
    pub fn free_page_count(&self) -> usize {
        self.free_bitmap.iter()
            .map(|byte| byte.count_ones() as usize)
            .sum()
    }

    /// Get number of used pages
    pub fn used_page_count(&self) -> usize {
        self.total_pages - self.free_page_count()
    }

    /// Check if a page is free
    fn is_bit_set(&self, index: usize) -> bool {
        let byte_index = index / 8;
        let bit_index = index % 8;
        (self.free_bitmap[byte_index] & (1 << bit_index)) != 0
    }

    /// Mark a page as free
    fn clear_bit(&mut self, index: usize) {
        let byte_index = index / 8;
        let bit_index = index % 8;
        self.free_bitmap[byte_index] &= !(1 << bit_index);
    }

    /// Mark a page as used
    fn set_bit(&mut self, index: usize) {
        let byte_index = index / 8;
        let bit_index = index % 8;
        self.free_bitmap[byte_index] |= (1 << bit_index);
    }
}

impl Default for PageFrameAllocator {
    fn default() -> Self {
        Self::new()
    }
}

impl PhysicalMemoryManager {
    /// Create a new physical memory manager
    pub const fn new() -> Self {
        Self {
            total_pages: 0,
            free_pages: PageFrameAllocator::new(),
            reserved_regions: Vec::new(),
            stats: MemoryStats::default(),
        }
    }

    /// Initialize physical memory management
    pub fn init(&mut self, memory_map_entries: &[MemoryMapEntry]) {
        info!("Initializing physical memory manager...");
        
        // Build physical memory map
        let mut physical_map = Vec::new();
        
        for entry in memory_map_entries {
            let region_type = match entry.entry_type {
                super::super::kernel::MemoryType::Usable => MemoryRegion::Usable,
                super::super::kernel::MemoryType::Reserved => MemoryRegion::Reserved,
                super::super::kernel::MemoryType::AcpiReclaimable => MemoryRegion::AcpiReclaimable,
                super::super::kernel::MemoryType::AcpiNvs => MemoryRegion::AcpiNvs,
                super::super::kernel::MemoryType::BadMemory => MemoryRegion::BadMemory,
                super::super::kernel::MemoryType::BootloaderReclaimable => MemoryRegion::BootloaderReclaimable,
                super::super::kernel::MemoryType::KernelAndModules => MemoryRegion::KernelAndModules,
                _ => MemoryRegion::Reserved,
            };
            
            let phys_addr = PhysAddr::new(entry.base);
            let size = entry.size as usize;
            
            // Add to physical memory map
            physical_map.push((phys_addr, size, region_type.clone()));
            
            // Add to reserved regions if not usable
            if region_type != MemoryRegion::Usable {
                self.reserved_regions.push(MemoryRegionDescriptor {
                    start: phys_addr,
                    end: phys_addr.offset(size as u64),
                    region_type,
                });
            }
        }
        
        // Initialize free page allocator
        self.free_pages.init(&physical_map);
        
        // Calculate total pages from usable memory
        self.total_pages = physical_map.iter()
            .filter_map(|(addr, size, region)| {
                if *region == MemoryRegion::Usable {
                    Some(size / PageSize::Size4K.as_usize())
                } else {
                    None
                }
            })
            .sum();
        
        // Update statistics
        self.update_stats();
        
        info!("Physical memory manager initialized:");
        info!("  Total pages: {}", self.total_pages);
        info!("  Free pages: {}", self.free_pages.free_page_count());
        info!("  Used pages: {}", self.free_pages.used_page_count());
        info!("  Reserved regions: {}", self.reserved_regions.len());
    }

    /// Allocate a physical page frame
    pub fn allocate_page(&mut self) -> MemoryResult<PageFrame> {
        self.free_pages.allocate_frame()
            .ok_or(MemoryError::OutOfMemory)
    }

    /// Allocate contiguous physical page frames
    pub fn allocate_pages(&mut self, count: usize) -> MemoryResult<PageFrame> {
        self.free_pages.allocate_contiguous_frames(count)
            .ok_or(MemoryError::OutOfMemory)
    }

    /// Free a physical page frame
    pub fn free_page(&mut self, frame: PageFrame) {
        self.free_pages.free_frame(frame);
        self.update_stats();
    }

    /// Get physical address for a page frame
    pub fn frame_to_addr(&self, frame: PageFrame) -> PhysAddr {
        frame.to_phys_addr(PageSize::Size4K)
    }

    /// Get page frame for a physical address
    pub fn addr_to_frame(&self, addr: PhysAddr) -> PageFrame {
        PageFrame::from_phys_addr(addr, PageSize::Size4K)
    }

    /// Check if a physical address range is valid and available
    pub fn is_range_available(&self, start: PhysAddr, size: usize) -> bool {
        let end = start.offset(size as u64);
        
        // Check against reserved regions
        for region in &self.reserved_regions {
            if ranges_overlap(start, end, region.start, region.end) {
                return false;
            }
        }
        
        // Additional checks for physical memory bounds
        let max_addr = PhysAddr::new((self.total_pages * PageSize::Size4K.as_usize()) as u64);
        if start.as_u64() >= max_addr.as_u64() || end.as_u64() > max_addr.as_u64() {
            return false;
        }
        
        true
    }

    /// Reserve a physical memory region
    pub fn reserve_region(&mut self, start: PhysAddr, size: usize, region_type: MemoryRegion) {
        let descriptor = MemoryRegionDescriptor {
            start,
            end: start.offset(size as u64),
            region_type,
        };
        
        self.reserved_regions.push(descriptor);
    }

    /// Get current memory statistics
    pub fn get_stats(&self) -> MemoryStats {
        self.stats
    }

    /// Update memory statistics
    fn update_stats(&mut self) {
        let free_pages = self.free_pages.free_page_count();
        let used_pages = self.free_pages.used_page_count();
        
        self.stats = MemoryStats {
            total_memory: (self.total_pages * PageSize::Size4K.as_usize()) as u64,
            used_memory: (used_pages * PageSize::Size4K.as_usize()) as u64,
            available_memory: (free_pages * PageSize::Size4K.as_usize()) as u64,
            total_pages: self.total_pages,
            used_pages,
            free_pages,
            reserved_pages: self.reserved_regions.len(),
        };
    }
}

/// Check if two address ranges overlap
fn ranges_overlap(start1: PhysAddr, end1: PhysAddr, start2: PhysAddr, end2: PhysAddr) -> bool {
    start1.as_u64() < end2.as_u64() && start2.as_u64() < end1.as_u64()
}

/// Initialize the global physical memory manager
pub fn init() -> MemoryResult<()> {
    info!("Initializing physical memory management...");
    
    // This function would be called with actual memory map from bootloader
    // For now, we'll initialize with empty memory map
    let empty_memory_map = Vec::new();
    
    let mut manager = PhysicalMemoryManager::new();
    manager.init(&empty_memory_map);
    
    *PHYSICAL_MEMORY_MANAGER.lock() = Some(manager);
    
    Ok(())
}

/// Get the global physical memory manager
fn get_manager() -> MemoryResult<spin::MutexGuard<'static, Option<PhysicalMemoryManager>>> {
    PHYSICAL_MEMORY_MANAGER.lock().as_ref()
        .ok_or(MemoryError::AllocationFailed)
        .map(|_| PHYSICAL_MEMORY_MANAGER.lock())
        .and_then(|guard| {
            guard.as_ref().ok_or(MemoryError::AllocationFailed)
                .map(|_| guard)
        })
}

/// Allocate a single physical page
pub fn allocate_physical_page() -> MemoryResult<PhysAddr> {
    let mut manager = PHYSICAL_MEMORY_MANAGER.lock();
    let manager_ref = manager.as_mut().ok_or(MemoryError::AllocationFailed)?;
    
    let frame = manager_ref.allocate_page()?;
    Ok(manager_ref.frame_to_addr(frame))
}

/// Allocate multiple contiguous physical pages
pub fn allocate_physical_pages(count: usize) -> MemoryResult<PhysAddr> {
    let mut manager = PHYSICAL_MEMORY_MANAGER.lock();
    let manager_ref = manager.as_mut().ok_or(MemoryError::AllocationFailed)?;
    
    let frame = manager_ref.allocate_pages(count)?;
    Ok(manager_ref.frame_to_addr(frame))
}

/// Free a physical page
pub fn free_physical_page(addr: PhysAddr) -> MemoryResult<()> {
    let mut manager = PHYSICAL_MEMORY_MANAGER.lock();
    let manager_ref = manager.as_mut().ok_or(MemoryError::AllocationFailed)?;
    
    let frame = manager_ref.addr_to_frame(addr);
    manager_ref.free_page(frame);
    
    Ok(())
}

/// Get total number of physical pages
pub fn get_total_pages() -> usize {
    let manager = PHYSICAL_MEMORY_MANAGER.lock();
    manager.as_ref()
        .map(|m| m.total_pages)
        .unwrap_or(0)
}

/// Get number of used physical pages
pub fn get_used_pages() -> usize {
    let manager = PHYSICAL_MEMORY_MANAGER.lock();
    manager.as_ref()
        .map(|m| m.free_pages.used_page_count())
        .unwrap_or(0)
}

/// Get number of available physical pages
pub fn get_available_pages() -> usize {
    let manager = PHYSICAL_MEMORY_MANAGER.lock();
    manager.as_ref()
        .map(|m| m.free_pages.free_page_count())
        .unwrap_or(0)
}

/// Get physical memory statistics
pub fn get_memory_stats() -> MemoryStats {
    let manager = PHYSICAL_MEMORY_MANAGER.lock();
    manager.as_ref()
        .map(|m| m.get_stats())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_page_frame_allocator() {
        let mut allocator = PageFrameAllocator::new();
        
        // Simulate 4 pages of usable memory
        let memory_map = vec![
            (PhysAddr::new(0), 0x4000, MemoryRegion::Usable), // 16KB = 4 pages
        ];
        allocator.init(&memory_map);
        
        assert_eq!(allocator.total_pages, 4);
        assert_eq!(allocator.free_page_count(), 4);
        assert_eq!(allocator.used_page_count(), 0);
        
        // Allocate a page
        let frame1 = allocator.allocate_frame().unwrap();
        assert_eq!(frame1.as_usize(), 0);
        assert_eq!(allocator.free_page_count(), 3);
        assert_eq!(allocator.used_page_count(), 1);
        
        // Allocate another page
        let frame2 = allocator.allocate_frame().unwrap();
        assert_eq!(frame2.as_usize(), 1);
        
        // Free first frame
        allocator.free_frame(frame1);
        assert_eq!(allocator.free_page_count(), 4);
        assert_eq!(allocator.used_page_count(), 0);
    }

    #[test]
    fn test_contiguous_allocation() {
        let mut allocator = PageFrameAllocator::new();
        
        // Simulate 8 pages of usable memory
        let memory_map = vec![
            (PhysAddr::new(0), 0x8000, MemoryRegion::Usable), // 32KB = 8 pages
        ];
        allocator.init(&memory_map);
        
        assert_eq!(allocator.total_pages, 8);
        
        // Allocate 3 contiguous pages
        let frame = allocator.allocate_contiguous_frames(3).unwrap();
        assert_eq!(frame.as_usize(), 0);
        
        // Free one page in the middle
        allocator.free_frame(PageFrame::new(1));
        
        // Should not be able to allocate 3 contiguous pages
        assert!(allocator.allocate_contiguous_frames(3).is_none());
        
        // Should be able to allocate 1 page
        assert!(allocator.allocate_frame().is_some());
    }

    #[test]
    fn test_memory_regions() {
        let mut manager = PhysicalMemoryManager::new();
        
        let memory_map = vec![
            super::super::kernel::MemoryMapEntry {
                base: 0,
                size: 0x1000,
                entry_type: super::super::kernel::MemoryType::Usable,
            },
            super::super::kernel::MemoryMapEntry {
                base: 0x1000,
                size: 0x1000,
                entry_type: super::super::kernel::MemoryType::Reserved,
            },
        ];
        
        manager.init(&memory_map);
        
        assert_eq!(manager.total_pages, 1);
        assert_eq!(manager.reserved_regions.len(), 1);
        assert_eq!(manager.reserved_regions[0].region_type, MemoryRegion::Reserved);
    }
}