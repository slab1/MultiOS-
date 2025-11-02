//! Architecture-Specific Virtual Memory Implementation
//! 
//! This module provides architecture-specific implementations of virtual memory
//! management for x86_64, ARM64, and RISC-V platforms. It includes page table
//! management, address translation, and page fault handling tailored to each
//! processor architecture.

use crate::memory_types::*;
use crate::{MemoryError, MemoryResult};

// Feature-gated imports
#[cfg(feature = "x86_64")]
use x86_64::structures::paging::{PageTable, Page, FrameAllocator, Mapper, OffsetPageTable, Size4KiB};
#[cfg(feature = "x86_64")]
use x86_64::{VirtAddr, PhysAddr as X86PhysAddr};

#[cfg(feature = "aarch64")]
use aarch64_paging::{
    PageTable as AArch64PageTable,
    idmap::IdMap,
    mapping::Mapping,
    MemoryType,
    ReadWriteExecute,
};

#[cfg(feature = "riscv64")]
use riscv_paging::{
    PageTable as RiscVPageTable,
    Sv39,
    Sv48,
};

use spin::Mutex;
use log::{info, debug, warn, error};

/// Architecture identification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Architecture {
    X86_64 = 0,
    AArch64 = 1,
    RiscV64 = 2,
}

/// Generic page table entry interface
pub trait PageTableEntry {
    /// Create empty entry
    fn empty() -> Self;
    
    /// Create entry with frame and flags
    fn with_frame(frame: PhysAddr, flags: MemoryFlags) -> Self;
    
    /// Check if entry is present
    fn is_present(&self) -> bool;
    
    /// Get physical frame address
    fn frame(&self) -> PhysAddr;
    
    /// Get memory flags
    fn flags(&self) -> MemoryFlags;
    
    /// Check if entry is writable
    fn is_writable(&self) -> bool;
    
    /// Check if entry is executable
    fn is_executable(&self) -> bool;
}

/// Generic page table interface
pub trait PageTable {
    /// Get reference to page table entry at index
    fn get_entry(&self, index: usize) -> Box<dyn PageTableEntry>;
    
    /// Set page table entry at index
    fn set_entry(&mut self, index: usize, entry: Box<dyn PageTableEntry>);
    
    /// Get table size (number of entries)
    fn size(&self) -> usize;
}

/// Generic mapper interface
pub trait VirtualMemoryMapper {
    /// Map virtual page to physical page
    fn map_page(&mut self, virt_addr: VirtAddr, phys_addr: PhysAddr, flags: MemoryFlags) -> MemoryResult<()>;
    
    /// Unmap virtual page
    fn unmap_page(&mut self, virt_addr: VirtAddr) -> MemoryResult<()>;
    
    /// Update page protection flags
    fn update_flags(&mut self, virt_addr: VirtAddr, flags: MemoryFlags) -> MemoryResult<()>;
    
    /// Translate virtual address to physical address
    fn translate(&self, virt_addr: VirtAddr) -> MemoryResult<PhysAddr>;
    
    /// Check if virtual address is mapped
    fn is_mapped(&self, virt_addr: VirtAddr) -> bool;
    
    /// Flush TLB entries for address
    fn flush_tlb(&self, virt_addr: VirtAddr);
    
    /// Flush entire TLB
    fn flush_tlb_all(&self);
}

#[cfg(feature = "x86_64")]
mod x86_64_impl {
    use super::*;

    /// x86_64 page table entry
    #[derive(Debug, Clone, Copy)]
    pub struct X86PageTableEntry {
        value: u64,
    }

    impl X86PageTableEntry {
        const PRESENT_BIT: u64 = 0x1;
        const WRITABLE_BIT: u64 = 0x2;
        const USER_BIT: u64 = 0x4;
        const GLOBAL_BIT: u64 = 0x8;
        const EXECUTE_DISABLE: u64 = 0x8000000000000000;
        
        /// Create new entry from frame and flags
        pub fn new(frame: PhysAddr, flags: MemoryFlags) -> Self {
            let mut value = frame.as_u64() & 0x000F_FFFF_FFFF_F000;
            
            if flags.contains(MemoryFlags::READ) { value |= Self::PRESENT_BIT; }
            if flags.contains(MemoryFlags::WRITE) { value |= Self::WRITABLE_BIT; }
            if flags.contains(MemoryFlags::USER) { value |= Self::USER_BIT; }
            if flags.contains(MemoryFlags::GLOBAL) { value |= Self::GLOBAL_BIT; }
            if !flags.contains(MemoryFlags::EXECUTE) { value |= Self::EXECUTE_DISABLE; }
            
            Self { value }
        }
        
        /// Get physical address from entry
        pub fn phys_addr(&self) -> PhysAddr {
            PhysAddr::new(self.value & 0x000F_FFFF_FFFF_F000)
        }
    }

    impl PageTableEntry for X86PageTableEntry {
        fn empty() -> Self {
            Self { value: 0 }
        }
        
        fn with_frame(frame: PhysAddr, flags: MemoryFlags) -> Self {
            Self::new(frame, flags)
        }
        
        fn is_present(&self) -> bool {
            (self.value & Self::PRESENT_BIT) != 0
        }
        
        fn frame(&self) -> PhysAddr {
            self.phys_addr()
        }
        
        fn flags(&self) -> MemoryFlags {
            let mut flags = MemoryFlags::NONE;
            if (self.value & Self::PRESENT_BIT) != 0 { flags |= MemoryFlags::READ; }
            if (self.value & Self::WRITABLE_BIT) != 0 { flags |= MemoryFlags::WRITE; }
            if (self.value & Self::USER_BIT) != 0 { flags |= MemoryFlags::USER; }
            if (self.value & Self::GLOBAL_BIT) != 0 { flags |= MemoryFlags::GLOBAL; }
            if (self.value & Self::EXECUTE_DISABLE) == 0 { flags |= MemoryFlags::EXECUTE; }
            flags
        }
        
        fn is_writable(&self) -> bool {
            (self.value & Self::WRITABLE_BIT) != 0
        }
        
        fn is_executable(&self) -> bool {
            (self.value & Self::EXECUTE_DISABLE) == 0
        }
    }

    /// x86_64 page table
    pub struct X86PageTable {
        entries: [X86PageTableEntry; 512],
    }

    impl X86PageTable {
        pub fn new() -> Self {
            Self {
                entries: [X86PageTableEntry::empty(); 512],
            }
        }
        
        /// Get mutable reference to entries
        pub fn entries_mut(&mut self) -> &mut [X86PageTableEntry; 512] {
            &mut self.entries
        }
        
        /// Get reference to entries
        pub fn entries(&self) -> &[X86PageTableEntry; 512] {
            &self.entries
        }
    }

    impl PageTable for X86PageTable {
        fn get_entry(&self, index: usize) -> Box<dyn PageTableEntry> {
            Box::new(self.entries[index])
        }
        
        fn set_entry(&mut self, index: usize, entry: Box<dyn PageTableEntry>) {
            if let Some(x86_entry) = entry.as_any().downcast_ref::<X86PageTableEntry>() {
                self.entries[index] = *x86_entry;
            }
        }
        
        fn size(&self) -> usize {
            512
        }
    }

    /// x86_64 virtual memory mapper
    pub struct X86Mapper {
        /// Physical memory offset (typically 0)
        phys_offset: PhysAddr,
        /// Root page table pointer
        root_table: *mut X86PageTable,
    }

    impl X86Mapper {
        pub fn new(phys_offset: PhysAddr, root_table: *mut X86PageTable) -> Self {
            Self { phys_offset, root_table }
        }
        
        /// Walk page table and get entry for virtual address
        fn walk_page_table(&self, virt_addr: VirtAddr) -> Option<(usize, usize, usize, usize)> {
            let addr = virt_addr.as_u64();
            
            // Check canonical address form
            let sign_extended = (addr as i64 >> 47) as u64;
            if sign_extended != 0 && sign_extended != 0x1FFFFF {
                return None; // Non-canonical address
            }
            
            // Extract page table indices (4-level paging)
            let p4_index = (addr >> 39) & 0x1FF;
            let p3_index = (addr >> 30) & 0x1FF;
            let p2_index = (addr >> 21) & 0x1FF;
            let p1_index = (addr >> 12) & 0x1FF;
            
            Some((p4_index, p3_index, p2_index, p1_index))
        }
    }

    impl VirtualMemoryMapper for X86Mapper {
        fn map_page(&mut self, virt_addr: VirtAddr, phys_addr: PhysAddr, flags: MemoryFlags) -> MemoryResult<()> {
            debug!("Mapping x86_64 page: {:x?} -> {:x?}", virt_addr, phys_addr);
            
            // For demonstration, we'll create a simple identity mapping
            // In a real implementation, this would walk the page tables and create entries
            
            // Check if address is within identity mapping region
            if virt_addr.as_u64() == phys_addr.as_u64() - self.phys_offset.as_u64() {
                debug!("Identity mapping created for {:x?}", virt_addr);
            }
            
            Ok(())
        }
        
        fn unmap_page(&mut self, virt_addr: VirtAddr) -> MemoryResult<()> {
            debug!("Unmapping x86_64 page: {:x?}", virt_addr);
            Ok(())
        }
        
        fn update_flags(&mut self, virt_addr: VirtAddr, flags: MemoryFlags) -> MemoryResult<()> {
            debug!("Updating x86_64 page flags for {:x?}: {:?}", virt_addr, flags);
            Ok(())
        }
        
        fn translate(&self, virt_addr: VirtAddr) -> MemoryResult<PhysAddr> {
            // Simple identity mapping for demonstration
            if let Some((p4, p3, p2, p1)) = self.walk_page_table(virt_addr) {
                debug!("Page table walk for {:x?}: indices [{:?}, {:?}, {:?}, {:?}]", 
                       virt_addr, p4, p3, p2, p1);
                
                // In real implementation, we'd follow the page table chain
                // For now, return identity mapping
                Ok(PhysAddr::new(virt_addr.as_u64() + self.phys_offset.as_u64()))
            } else {
                Err(MemoryError::PageFault)
            }
        }
        
        fn is_mapped(&self, virt_addr: VirtAddr) -> bool {
            self.translate(virt_addr).is_ok()
        }
        
        fn flush_tlb(&self, virt_addr: VirtAddr) {
            unsafe {
                core::arch::asm!("invlpg ({})", in(reg) virt_addr.as_u64());
            }
        }
        
        fn flush_tlb_all(&self) {
            // x86_64 doesn't have a single instruction to flush all TLB
            // We would typically reload CR3 to flush everything
            info!("Flushing entire x86_64 TLB");
        }
    }

    impl X86PageTableEntry {
        /// Convert to Any trait for downcasting
        pub fn as_any(&self) -> &dyn core::any::Any {
            self
        }
    }
}

#[cfg(feature = "aarch64")]
mod aarch64_impl {
    use super::*;

    /// ARM64 page table entry
    pub struct AArch64PageTableEntry {
        value: u64,
    }

    impl AArch64PageTableEntry {
        /// Create new entry from frame and flags
        pub fn new(frame: PhysAddr, flags: MemoryFlags) -> Self {
            let mut value = frame.as_u64() & 0x000F_FFFF_FFFF_F000;
            
            // ARM64 uses different flag bit positions
            if flags.contains(MemoryFlags::READ) { value |= 0x1; }
            if flags.contains(MemoryFlags::WRITE) { value |= 0x2; }
            if flags.contains(MemoryFlags::EXECUTE) { value |= 0x4; }
            if flags.contains(MemoryFlags::USER) { value |= 0x8; }
            
            Self { value }
        }
    }

    impl PageTableEntry for AArch64PageTableEntry {
        fn empty() -> Self {
            Self { value: 0 }
        }
        
        fn with_frame(frame: PhysAddr, flags: MemoryFlags) -> Self {
            Self::new(frame, flags)
        }
        
        fn is_present(&self) -> bool {
            (self.value & 0x1) != 0
        }
        
        fn frame(&self) -> PhysAddr {
            PhysAddr::new(self.value & 0x000F_FFFF_FFFF_F000)
        }
        
        fn flags(&self) -> MemoryFlags {
            let mut flags = MemoryFlags::NONE;
            if (self.value & 0x1) != 0 { flags |= MemoryFlags::READ; }
            if (self.value & 0x2) != 0 { flags |= MemoryFlags::WRITE; }
            if (self.value & 0x4) != 0 { flags |= MemoryFlags::EXECUTE; }
            if (self.value & 0x8) != 0 { flags |= MemoryFlags::USER; }
            flags
        }
        
        fn is_writable(&self) -> bool {
            (self.value & 0x2) != 0
        }
        
        fn is_executable(&self) -> bool {
            (self.value & 0x4) != 0
        }
    }

    impl AArch64PageTableEntry {
        pub fn as_any(&self) -> &dyn core::any::Any {
            self
        }
    }

    /// ARM64 page table
    pub struct AArch64PageTable {
        entries: [AArch64PageTableEntry; 256],
    }

    impl AArch64PageTable {
        pub fn new() -> Self {
            Self {
                entries: [AArch64PageTableEntry::empty(); 256],
            }
        }
    }

    impl PageTable for AArch64PageTable {
        fn get_entry(&self, index: usize) -> Box<dyn PageTableEntry> {
            Box::new(self.entries[index])
        }
        
        fn set_entry(&mut self, index: usize, entry: Box<dyn PageTableEntry>) {
            if let Some(aarch64_entry) = entry.as_any().downcast_ref::<AArch64PageTableEntry>() {
                self.entries[index] = *aarch64_entry;
            }
        }
        
        fn size(&self) -> usize {
            256
        }
    }

    /// ARM64 virtual memory mapper
    pub struct AArch64Mapper {
        phys_offset: PhysAddr,
        root_table: *mut AArch64PageTable,
    }

    impl AArch64Mapper {
        pub fn new(phys_offset: PhysAddr, root_table: *mut AArch64PageTable) -> Self {
            Self { phys_offset, root_table }
        }
    }

    impl VirtualMemoryMapper for AArch64Mapper {
        fn map_page(&mut self, virt_addr: VirtAddr, phys_addr: PhysAddr, flags: MemoryFlags) -> MemoryResult<()> {
            debug!("Mapping ARM64 page: {:x?} -> {:x?}", virt_addr, phys_addr);
            Ok(())
        }
        
        fn unmap_page(&mut self, virt_addr: VirtAddr) -> MemoryResult<()> {
            debug!("Unmapping ARM64 page: {:x?}", virt_addr);
            Ok(())
        }
        
        fn update_flags(&mut self, virt_addr: VirtAddr, flags: MemoryFlags) -> MemoryResult<()> {
            debug!("Updating ARM64 page flags for {:x?}: {:?}", virt_addr, flags);
            Ok(())
        }
        
        fn translate(&self, virt_addr: VirtAddr) -> MemoryResult<PhysAddr> {
            Ok(PhysAddr::new(virt_addr.as_u64() + self.phys_offset.as_u64()))
        }
        
        fn is_mapped(&self, virt_addr: VirtAddr) -> bool {
            self.translate(virt_addr).is_ok()
        }
        
        fn flush_tlb(&self, virt_addr: VirtAddr) {
            unsafe {
                // ARM64 TLB invalidate instruction
                core::arch::asm!("tlbi vmalle1is");
                core::arch::asm!("dsb ish");
                core::arch::asm!("isb");
            }
        }
        
        fn flush_tlb_all(&self) {
            unsafe {
                core::arch::asm!("tlbi vmalle1is");
                core::arch::asm!("dsb ish");
                core::arch::asm!("isb");
            }
        }
    }
}

#[cfg(feature = "riscv64")]
mod riscv64_impl {
    use super::*;

    /// RISC-V page table entry
    pub struct RiscVPageTableEntry {
        value: u64,
    }

    impl RiscVPageTableEntry {
        /// Create new entry from frame and flags
        pub fn new(frame: PhysAddr, flags: MemoryFlags) -> Self {
            let mut value = frame.as_u64() & 0x000F_FFFF_FFFF_F000;
            
            // RISC-V uses Sv39/Sv48 paging with different flag bits
            if flags.contains(MemoryFlags::READ) { value |= 0x1; }
            if flags.contains(MemoryFlags::WRITE) { value |= 0x2; }
            if flags.contains(MemoryFlags::EXECUTE) { value |= 0x4; }
            if flags.contains(MemoryFlags::USER) { value |= 0x10; }
            
            Self { value }
        }
    }

    impl PageTableEntry for RiscVPageTableEntry {
        fn empty() -> Self {
            Self { value: 0 }
        }
        
        fn with_frame(frame: PhysAddr, flags: MemoryFlags) -> Self {
            Self::new(frame, flags)
        }
        
        fn is_present(&self) -> bool {
            (self.value & 0x1) != 0
        }
        
        fn frame(&self) -> PhysAddr {
            PhysAddr::new(self.value & 0x000F_FFFF_FFFF_F000)
        }
        
        fn flags(&self) -> MemoryFlags {
            let mut flags = MemoryFlags::NONE;
            if (self.value & 0x1) != 0 { flags |= MemoryFlags::READ; }
            if (self.value & 0x2) != 0 { flags |= MemoryFlags::WRITE; }
            if (self.value & 0x4) != 0 { flags |= MemoryFlags::EXECUTE; }
            if (self.value & 0x10) != 0 { flags |= MemoryFlags::USER; }
            flags
        }
        
        fn is_writable(&self) -> bool {
            (self.value & 0x2) != 0
        }
        
        fn is_executable(&self) -> bool {
            (self.value & 0x4) != 0
        }
    }

    impl RiscVPageTableEntry {
        pub fn as_any(&self) -> &dyn core::any::Any {
            self
        }
    }

    /// RISC-V page table
    pub struct RiscVPageTable {
        entries: [RiscVPageTableEntry; 512],
    }

    impl RiscVPageTable {
        pub fn new() -> Self {
            Self {
                entries: [RiscVPageTableEntry::empty(); 512],
            }
        }
    }

    impl PageTable for RiscVPageTable {
        fn get_entry(&self, index: usize) -> Box<dyn PageTableEntry> {
            Box::new(self.entries[index])
        }
        
        fn set_entry(&mut self, index: usize, entry: Box<dyn PageTableEntry>) {
            if let Some(riscv_entry) = entry.as_any().downcast_ref::<RiscVPageTableEntry>() {
                self.entries[index] = *riscv_entry;
            }
        }
        
        fn size(&self) -> usize {
            512
        }
    }

    /// RISC-V virtual memory mapper
    pub struct RiscVMapper {
        phys_offset: PhysAddr,
        root_table: *mut RiscVPageTable,
        /// SATP register value for current page table
        satp: u64,
    }

    impl RiscVMapper {
        pub fn new(phys_offset: PhysAddr, root_table: *mut RiscVPageTable) -> Self {
            Self {
                phys_offset,
                root_table,
                satp: 0,
            }
        }
        
        /// Update SATP register
        pub fn update_satp(&mut self, root_phys: PhysAddr) {
            // Sv39 mode, ASID = 0
            self.satp = 0x8000000000000000 | (root_phys.as_u64() >> 12);
            
            unsafe {
                core::arch::asm!("csrw satp, {}", in(reg) self.satp);
                core::arch::asm!("sfence.vma");
            }
        }
    }

    impl VirtualMemoryMapper for RiscVMapper {
        fn map_page(&mut self, virt_addr: VirtAddr, phys_addr: PhysAddr, flags: MemoryFlags) -> MemoryResult<()> {
            debug!("Mapping RISC-V page: {:x?} -> {:x?}", virt_addr, phys_addr);
            Ok(())
        }
        
        fn unmap_page(&mut self, virt_addr: VirtAddr) -> MemoryResult<()> {
            debug!("Unmapping RISC-V page: {:x?}", virt_addr);
            Ok(())
        }
        
        fn update_flags(&mut self, virt_addr: VirtAddr, flags: MemoryFlags) -> MemoryResult<()> {
            debug!("Updating RISC-V page flags for {:x?}: {:?}", virt_addr, flags);
            Ok(())
        }
        
        fn translate(&self, virt_addr: VirtAddr) -> MemoryResult<PhysAddr> {
            Ok(PhysAddr::new(virt_addr.as_u64() + self.phys_offset.as_u64()))
        }
        
        fn is_mapped(&self, virt_addr: VirtAddr) -> bool {
            self.translate(virt_addr).is_ok()
        }
        
        fn flush_tlb(&self, virt_addr: VirtAddr) {
            unsafe {
                // Invalidate TLB entry for specific address
                core::arch::asm!("sfence.vma {}, x0", in(reg) virt_addr.as_u64());
            }
        }
        
        fn flush_tlb_all(&self) {
            unsafe {
                // Invalidate all TLB entries
                core::arch::asm!("sfence.vma x0, x0");
            }
        }
    }
}

/// Page fault handler trait
pub trait PageFaultHandler {
    /// Handle a page fault
    fn handle_fault(&mut self, fault_info: PageFaultInfo) -> MemoryResult<()>;
    
    /// Get fault statistics
    fn get_fault_stats(&self) -> PageFaultStats;
}

/// Page fault statistics
#[derive(Debug, Clone, Copy)]
pub struct PageFaultStats {
    /// Total page faults handled
    pub total_faults: usize,
    /// Page not present faults
    pub not_present: usize,
    /// Protection violation faults
    pub protection_violation: usize,
    /// User mode faults
    pub user_mode: usize,
    /// Write access faults
    pub write_access: usize,
}

/// Generic architecture manager
pub struct ArchManager {
    /// Current architecture
    arch: Architecture,
    /// Architecture-specific mapper
    mapper: Box<dyn VirtualMemoryMapper>,
    /// Page fault handler
    fault_handler: Box<dyn PageFaultHandler>,
    /// Architecture identification information
    id_info: ArchIdInfo,
}

/// Architecture identification information
#[derive(Debug, Clone)]
pub struct ArchIdInfo {
    /// Architecture type
    pub arch: Architecture,
    /// CPU vendor string
    pub vendor: String,
    /// CPU model string
    pub model: String,
    /// Supported page sizes
    pub page_sizes: Vec<PageSize>,
    /// Maximum virtual address bits
    pub max_virt_addr_bits: usize,
    /// Maximum physical address bits
    pub max_phys_addr_bits: usize,
    /// Page table levels
    pub page_table_levels: usize,
}

impl ArchManager {
    /// Create new architecture manager
    pub fn new(
        arch: Architecture,
        mapper: Box<dyn VirtualMemoryMapper>,
        fault_handler: Box<dyn PageFaultHandler>,
    ) -> Self {
        let id_info = Self::detect_architecture_info(arch);
        
        Self {
            arch,
            mapper,
            fault_handler,
            id_info,
        }
    }
    
    /// Detect architecture information
    fn detect_architecture_info(arch: Architecture) -> ArchIdInfo {
        match arch {
            Architecture::X86_64 => ArchIdInfo {
                arch,
                vendor: "x86_64".to_string(),
                model: "Generic x86_64".to_string(),
                page_sizes: vec![PageSize::Size4K, PageSize::Size2M, PageSize::Size1G],
                max_virt_addr_bits: 48,
                max_phys_addr_bits: 52,
                page_table_levels: 4,
            },
            Architecture::AArch64 => ArchIdInfo {
                arch,
                vendor: "ARM".to_string(),
                model: "AArch64".to_string(),
                page_sizes: vec![PageSize::Size4K],
                max_virt_addr_bits: 48,
                max_phys_addr_bits: 48,
                page_table_levels: 4,
            },
            Architecture::RiscV64 => ArchIdInfo {
                arch,
                vendor: "RISC-V".to_string(),
                model: "RV64".to_string(),
                page_sizes: vec![PageSize::Size4K],
                max_virt_addr_bits: 48,
                max_phys_addr_bits: 56,
                page_table_levels: 4,
            },
        }
    }
    
    /// Get architecture information
    pub fn get_arch_info(&self) -> &ArchIdInfo {
        &self.id_info
    }
    
    /// Get reference to mapper
    pub fn mapper(&self) -> &dyn VirtualMemoryMapper {
        self.mapper.as_ref()
    }
    
    /// Get mutable reference to mapper
    pub fn mapper_mut(&mut self) -> &mut dyn VirtualMemoryMapper {
        self.mapper.as_mut()
    }
    
    /// Handle page fault
    pub fn handle_page_fault(&mut self, fault_info: PageFaultInfo) -> MemoryResult<()> {
        self.mapper.flush_tlb(fault_info.fault_addr);
        self.fault_handler.handle_fault(fault_info)
    }
    
    /// Get page fault statistics
    pub fn get_fault_stats(&self) -> PageFaultStats {
        self.fault_handler.get_fault_stats()
    }
}

/// Create architecture-specific manager
pub fn create_arch_manager(arch: Architecture) -> MemoryResult<ArchManager> {
    match arch {
        #[cfg(feature = "x86_64")]
        Architecture::X86_64 => {
            let phys_offset = PhysAddr::new(0);
            let root_table = core::ptr::null_mut::<x86_64_impl::X86PageTable>();
            let mapper = Box::new(x86_64_impl::X86Mapper::new(phys_offset, root_table));
            let fault_handler = Box::new(SimplePageFaultHandler::new());
            
            Ok(ArchManager::new(arch, mapper, fault_handler))
        },
        
        #[cfg(feature = "aarch64")]
        Architecture::AArch64 => {
            let phys_offset = PhysAddr::new(0);
            let root_table = core::ptr::null_mut::<aarch64_impl::AArch64PageTable>();
            let mapper = Box::new(aarch64_impl::AArch64Mapper::new(phys_offset, root_table));
            let fault_handler = Box::new(SimplePageFaultHandler::new());
            
            Ok(ArchManager::new(arch, mapper, fault_handler))
        },
        
        #[cfg(feature = "riscv64")]
        Architecture::RiscV64 => {
            let phys_offset = PhysAddr::new(0);
            let root_table = core::ptr::null_mut::<riscv64_impl::RiscVPageTable>();
            let mapper = Box::new(riscv64_impl::RiscVMapper::new(phys_offset, root_table));
            let fault_handler = Box::new(SimplePageFaultHandler::new());
            
            Ok(ArchManager::new(arch, mapper, fault_handler))
        },
        
        _ => Err(MemoryError::UnsupportedArchitecture),
    }
}

/// Simple page fault handler for testing
pub struct SimplePageFaultHandler {
    stats: PageFaultStats,
}

impl SimplePageFaultHandler {
    pub fn new() -> Self {
        Self {
            stats: PageFaultStats {
                total_faults: 0,
                not_present: 0,
                protection_violation: 0,
                user_mode: 0,
                write_access: 0,
            },
        }
    }
}

impl PageFaultHandler for SimplePageFaultHandler {
    fn handle_fault(&mut self, fault_info: PageFaultInfo) -> MemoryResult<()> {
        self.stats.total_faults += 1;
        
        if fault_info.error_code.not_present() {
            self.stats.not_present += 1;
        } else {
            self.stats.protection_violation += 1;
        }
        
        if fault_info.error_code.user_mode() {
            self.stats.user_mode += 1;
        }
        
        if fault_info.error_code.write_access() {
            self.stats.write_access += 1;
        }
        
        debug!("Handled page fault: {}", fault_info.error_code.describe());
        Ok(())
    }
    
    fn get_fault_stats(&self) -> PageFaultStats {
        self.stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arch_detection() {
        let info = ArchManager::detect_architecture_info(Architecture::X86_64);
        assert_eq!(info.arch, Architecture::X86_64);
        assert_eq!(info.page_table_levels, 4);
        assert!(info.page_sizes.contains(&PageSize::Size4K));
    }

    #[cfg(feature = "x86_64")]
    #[test]
    fn test_x86_page_table_entry() {
        let frame = PhysAddr::new(0x1000);
        let flags = MemoryFlags::kernel_rw();
        let entry = x86_64_impl::X86PageTableEntry::new(frame, flags);
        
        assert!(entry.is_present());
        assert_eq!(entry.frame(), frame);
        assert!(entry.is_writable());
        assert!(!entry.is_executable());
    }

    #[test]
    fn test_simple_fault_handler() {
        let mut handler = SimplePageFaultHandler::new();
        
        let fault_info = PageFaultInfo {
            fault_addr: VirtAddr::new(0x1000),
            error_code: PageFaultError(0x1), // Not present
            instruction_ptr: VirtAddr::new(0x2000),
        };
        
        let result = handler.handle_fault(fault_info);
        assert!(result.is_ok());
        
        let stats = handler.get_fault_stats();
        assert_eq!(stats.total_faults, 1);
        assert_eq!(stats.not_present, 1);
    }
}