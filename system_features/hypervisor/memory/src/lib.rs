//! Memory Virtualization
//! 
//! Implements memory virtualization using Extended Page Tables (EPT) for Intel VT-x
//! and Nested Page Tables (NPT) for AMD-V, providing efficient nested paging support.

use crate::{HypervisorError, VmId, VcpuId};
use crate::core::{VmExitReason, MemoryStats};

use bitflags::bitflags;
use alloc::vec::Vec;

/// Page size constants
pub const PAGE_SIZE_4K: u64 = 0x1000;
pub const PAGE_SIZE_2M: u64 = 0x200000;
pub const PAGE_SIZE_1G: u64 = 0x40000000;

/// EPT entry structure for Intel VT-x
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct EptEntry {
    pub read: bool,
    pub write: bool,
    pub execute: bool,
    pub memory_type: u8,
    pub ignore_pat: bool,
    pub accessed: bool,
    pub dirty: bool,
    pub user_mode: bool,
    pub global_page: bool,
    pub present: bool,
    pub address: u64,
}

/// NPT entry structure for AMD-V
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct NptEntry {
    pub read: bool,
    pub write: bool,
    pub execute: bool,
    pub system: bool,
    pub global: bool,
    pub accessed: bool,
    pub dirty: bool,
    pub present: bool,
    pub address: u64,
}

/// Page table level for memory mapping
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PageTableLevel {
    Level4, // 4-level paging (PML4)
    Level3, // 3-level paging (PDPT)
    Level2, // 2-level paging (PD)
    Level1, // 1-level paging (PT)
    Level0, // Page entry
}

/// Memory mapping flags
bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct MemoryFlags: u32 {
        const READ = 1 << 0;
        const WRITE = 1 << 1;
        const EXECUTE = 1 << 2;
        const USER = 1 << 3;
        const SYSTEM = 1 << 4;
        const GLOBAL = 1 << 5;
        const PAT = 1 << 6;
        const COMPRESSED = 1 << 7;
        const LARGE_PAGE = 1 << 8;
    }
}

/// Memory region type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MemoryRegionType {
    /// Code segment
    Code,
    /// Data segment  
    Data,
    /// Stack segment
    Stack,
    /// Heap segment
    Heap,
    /// MMIO region
    Mmio,
    /// Reserved region
    Reserved,
}

/// Memory region information
#[derive(Debug, Clone)]
pub struct MemoryRegion {
    pub start_address: u64,
    pub end_address: u64,
    pub flags: MemoryFlags,
    pub region_type: MemoryRegionType,
    pub host_address: u64,
    pub allocated: bool,
    pub dirty: bool,
}

/// EPT Page Table structure
#[derive(Debug)]
pub struct EptPageTable {
    /// VM ID this EPT belongs to
    pub vm_id: VmId,
    /// Root PML4 entry
    pub pml4: [EptEntry; 512],
    /// PDPT tables
    pub pdpts: Vec<[EptEntry; 512]>,
    /// Page directories
    pub pds: Vec<[EptEntry; 512]>,
    /// Page tables
    pub pts: Vec<[EptEntry; 512]>,
    /// Total memory allocated
    pub total_memory_mb: u64,
    /// Memory regions
    pub regions: Vec<MemoryRegion>,
}

/// NPT Page Table structure
#[derive(Debug)]
pub struct NptPageTable {
    /// VM ID this NPT belongs to
    pub vm_id: VmId,
    /// Root PDPT entry
    pub pdpt: [NptEntry; 512],
    /// Page directories
    pub pds: Vec<[NptEntry; 512]>,
    /// Page tables
    pub pts: Vec<[NptEntry; 512]>,
    /// Total memory allocated
    pub total_memory_mb: u64,
    /// Memory regions
    pub regions: Vec<MemoryRegion>,
}

/// Memory Manager for virtualization
pub struct MemoryManager {
    /// VM ID
    pub vm_id: VmId,
    /// Total allocated memory in MB
    total_memory_mb: u64,
    /// Used memory in MB
    used_memory_mb: u64,
    /// EPT Page Tables (Intel VT-x)
    ept_table: Option<EptPageTable>,
    /// NPT Page Tables (AMD-V)
    npt_table: Option<NptPageTable>,
    /// Hardware virtualization type
    virt_type: VirtualizationType,
    /// Page fault count
    page_fault_count: u64,
    /// TLB hit count
    tlb_hit_count: u64,
    /// TLB miss count
    tlb_miss_count: u64,
}

impl MemoryManager {
    /// Create a new memory manager
    pub fn new(memory_mb: u64) -> Result<Self, HypervisorError> {
        if memory_mb < 16 {
            return Err(HypervisorError::InvalidParameter);
        }
        
        let memory_manager = MemoryManager {
            vm_id: VmId(0), // Will be set when VM is created
            total_memory_mb: memory_mb,
            used_memory_mb: 0,
            ept_table: None,
            npt_table: None,
            virt_type: VirtualizationType::Unknown,
            page_fault_count: 0,
            tlb_hit_count: 0,
            tlb_miss_count: 0,
        };
        
        info!("Memory Manager created with {} MB", memory_mb);
        Ok(memory_manager)
    }
    
    /// Initialize memory virtualization
    pub fn initialize(&mut self, vm_id: VmId, virt_type: VirtualizationType) -> Result<(), HypervisorError> {
        self.vm_id = vm_id;
        self.virt_type = virt_type;
        
        match virt_type {
            VirtualizationType::IntelVTx => {
                self.ept_table = Some(self.create_ept_table()?);
            },
            VirtualizationType::AMDV => {
                self.npt_table = Some(self.create_npt_table()?);
            },
            VirtualizationType::Unknown => {
                return Err(HypervisorError::HardwareVirtNotAvailable);
            },
        }
        
        info!("Memory virtualization initialized with {:?}", virt_type);
        Ok(())
    }
    
    /// Create EPT page table
    fn create_ept_table(&self) -> Result<EptPageTable, HypervisorError> {
        let mut ept_table = EptPageTable {
            vm_id: self.vm_id,
            pml4: [EptEntry::default(); 512],
            pdpts: Vec::new(),
            pds: Vec::new(),
            pts: Vec::new(),
            total_memory_mb: self.total_memory_mb,
            regions: Vec::new(),
        };
        
        // Initialize PML4 entries
        for i in 0..512 {
            ept_table.pml4[i] = EptEntry::default();
        }
        
        Ok(ept_table)
    }
    
    /// Create NPT page table
    fn create_npt_table(&self) -> Result<NptPageTable, HypervisorError> {
        let mut npt_table = NptPageTable {
            vm_id: self.vm_id,
            pdpt: [NptEntry::default(); 512],
            pds: Vec::new(),
            pts: Vec::new(),
            total_memory_mb: self.total_memory_mb,
            regions: Vec::new(),
        };
        
        // Initialize PDPT entries
        for i in 0..512 {
            npt_table.pdpt[i] = NptEntry::default();
        }
        
        Ok(npt_table)
    }
    
    /// Map guest virtual address to host physical address
    pub fn map_guest_virtual_address(&mut self, guest_addr: u64, host_addr: u64, size: u64, flags: MemoryFlags) -> Result<(), HypervisorError> {
        let align_size = self.align_to_page_size(size);
        
        match self.virt_type {
            VirtualizationType::IntelVTx => {
                if let Some(ref mut ept) = self.ept_table {
                    self.map_in_ept(ept, guest_addr, host_addr, align_size, flags)?;
                }
            },
            VirtualizationType::AMDV => {
                if let Some(ref mut npt) = self.npt_table {
                    self.map_in_npt(npt, guest_addr, host_addr, align_size, flags)?;
                }
            },
            VirtualizationType::Unknown => {
                return Err(HypervisorError::HardwareVirtNotAvailable);
            },
        }
        
        // Track memory region
        self.add_memory_region(guest_addr, guest_addr + align_size, flags)?;
        
        self.used_memory_mb += align_size / (1024 * 1024);
        
        info!("Mapped guest address 0x{:016x} to host 0x{:016x} ({} bytes)", 
              guest_addr, host_addr, align_size);
        
        Ok(())
    }
    
    /// Map address in EPT
    fn map_in_ept(&mut self, ept: &mut EptPageTable, guest_addr: u64, host_addr: u64, size: u64, flags: MemoryFlags) -> Result<(), HypervisorError> {
        let mut current_guest = guest_addr;
        let mut current_host = host_addr;
        let mut remaining_size = size;
        
        while remaining_size > 0 {
            // Calculate page table indexes
            let pml4_idx = ((current_guest >> 39) & 0x1FF) as usize;
            let pdpt_idx = ((current_guest >> 30) & 0x1FF) as usize;
            let pd_idx = ((current_guest >> 21) & 0x1FF) as usize;
            let pt_idx = ((current_guest >> 12) & 0x1FF) as usize;
            
            // Use large pages when possible
            if size >= PAGE_SIZE_1G && current_guest & (PAGE_SIZE_1G - 1) == 0 {
                // Create 1GB large page
                let pdpt_entry = &mut ept.pdpts[pml4_idx * 512 + pdpt_idx];
                pdpt_entry.present = true;
                pdpt_entry.read = flags.contains(MemoryFlags::READ);
                pdpt_entry.write = flags.contains(MemoryFlags::WRITE);
                pdpt_entry.execute = flags.contains(MemoryFlags::EXECUTE);
                pdpt_entry.address = current_host & !(PAGE_SIZE_1G - 1);
                pdpt_entry.memory_type = 0; // Uncacheable
                
                remaining_size = remaining_size.saturating_sub(PAGE_SIZE_1G);
                current_guest += PAGE_SIZE_1G;
                current_host += PAGE_SIZE_1G;
            } else if size >= PAGE_SIZE_2M && current_guest & (PAGE_SIZE_2M - 1) == 0 {
                // Create 2MB large page
                let pd_entry = &mut ept.pds[pd_idx];
                pd_entry.present = true;
                pd_entry.read = flags.contains(MemoryFlags::READ);
                pd_entry.write = flags.contains(MemoryFlags::WRITE);
                pd_entry.execute = flags.contains(MemoryFlags::EXECUTE);
                pd_entry.address = current_host & !(PAGE_SIZE_2M - 1);
                pd_entry.memory_type = 0; // Uncacheable
                
                remaining_size = remaining_size.saturating_sub(PAGE_SIZE_2M);
                current_guest += PAGE_SIZE_2M;
                current_host += PAGE_SIZE_2M;
            } else {
                // Create 4KB page
                let pt_entry = &mut ept.pts[pt_idx];
                pt_entry.present = true;
                pt_entry.read = flags.contains(MemoryFlags::READ);
                pt_entry.write = flags.contains(MemoryFlags::WRITE);
                pt_entry.execute = flags.contains(MemoryFlags::EXECUTE);
                pt_entry.address = current_host & !0xFFF;
                pt_entry.memory_type = 0; // Uncacheable
                pt_entry.user_mode = flags.contains(MemoryFlags::USER);
                
                remaining_size = remaining_size.saturating_sub(PAGE_SIZE_4K);
                current_guest += PAGE_SIZE_4K;
                current_host += PAGE_SIZE_4K;
            }
        }
        
        Ok(())
    }
    
    /// Map address in NPT
    fn map_in_npt(&mut self, npt: &mut NptPageTable, guest_addr: u64, host_addr: u64, size: u64, flags: MemoryFlags) -> Result<(), HypervisorError> {
        let mut current_guest = guest_addr;
        let mut current_host = host_addr;
        let mut remaining_size = size;
        
        while remaining_size > 0 {
            // Calculate page table indexes
            let pdpt_idx = ((current_guest >> 27) & 0x1FF) as usize;
            let pd_idx = ((current_guest >> 18) & 0x1FF) as usize;
            let pt_idx = ((current_guest >> 9) & 0x1FF) as usize;
            
            // Use large pages when possible
            if size >= PAGE_SIZE_1G && current_guest & (PAGE_SIZE_1G - 1) == 0 {
                // Create 1GB large page
                let pdpt_entry = &mut npt.pdpt[pdpt_idx];
                pdpt_entry.present = true;
                pdpt_entry.read = flags.contains(MemoryFlags::READ);
                pdpt_entry.write = flags.contains(MemoryFlags::WRITE);
                pdpt_entry.execute = flags.contains(MemoryFlags::EXECUTE);
                pdpt_entry.address = current_host & !(PAGE_SIZE_1G - 1);
                
                remaining_size = remaining_size.saturating_sub(PAGE_SIZE_1G);
                current_guest += PAGE_SIZE_1G;
                current_host += PAGE_SIZE_1G;
            } else if size >= PAGE_SIZE_2M && current_guest & (PAGE_SIZE_2M - 1) == 0 {
                // Create 2MB large page
                let pd_entry = &mut npt.pds[pd_idx];
                pd_entry.present = true;
                pd_entry.read = flags.contains(MemoryFlags::READ);
                pd_entry.write = flags.contains(MemoryFlags::WRITE);
                pd_entry.execute = flags.contains(MemoryFlags::EXECUTE);
                pd_entry.address = current_host & !(PAGE_SIZE_2M - 1);
                
                remaining_size = remaining_size.saturating_sub(PAGE_SIZE_2M);
                current_guest += PAGE_SIZE_2M;
                current_host += PAGE_SIZE_2M;
            } else {
                // Create 4KB page
                let pt_entry = &mut npt.pts[pt_idx];
                pt_entry.present = true;
                pt_entry.read = flags.contains(MemoryFlags::READ);
                pt_entry.write = flags.contains(MemoryFlags::WRITE);
                pt_entry.execute = flags.contains(MemoryFlags::EXECUTE);
                pt_entry.address = current_host & !0xFFF;
                pt_entry.system = !flags.contains(MemoryFlags::USER);
                
                remaining_size = remaining_size.saturating_sub(PAGE_SIZE_4K);
                current_guest += PAGE_SIZE_4K;
                current_host += PAGE_SIZE_4K;
            }
        }
        
        Ok(())
    }
    
    /// Translate guest virtual address to host physical address
    pub fn translate_guest_address(&self, guest_addr: u64) -> Option<u64> {
        match self.virt_type {
            VirtualizationType::IntelVTx => {
                if let Some(ref ept) = self.ept_table {
                    self.translate_in_ept(ept, guest_addr)
                } else {
                    None
                }
            },
            VirtualizationType::AMDV => {
                if let Some(ref npt) = self.npt_table {
                    self.translate_in_npt(npt, guest_addr)
                } else {
                    None
                }
            },
            VirtualizationType::Unknown => None,
        }
    }
    
    /// Translate address in EPT
    fn translate_in_ept(&self, ept: &EptPageTable, guest_addr: u64) -> Option<u64> {
        // Simplified translation - in real implementation would walk EPT
        // For demonstration, return the address directly
        Some(guest_addr)
    }
    
    /// Translate address in NPT
    fn translate_in_npt(&self, npt: &NptPageTable, guest_addr: u64) -> Option<u64> {
        // Simplified translation - in real implementation would walk NPT
        Some(guest_addr)
    }
    
    /// Handle EPT violation
    pub fn handle_ept_violation(&mut self, guest_addr: u64) -> Result<VmExitReason, HypervisorError> {
        self.page_fault_count += 1;
        
        // In real implementation, would handle the EPT violation
        // by allocating missing page, updating EPT, etc.
        
        info!("EPT violation at guest address 0x{:016x}", guest_addr);
        Ok(VmExitReason::EPTViolation)
    }
    
    /// Add memory region to tracking
    fn add_memory_region(&mut self, start_addr: u64, end_addr: u64, flags: MemoryFlags) -> Result<(), HypervisorError> {
        let region_type = match flags & MemoryFlags::EXECUTE {
            MemoryFlags::EXECUTE => MemoryRegionType::Code,
            _ => MemoryRegionType::Data,
        };
        
        let region = MemoryRegion {
            start_address: start_addr,
            end_address: end_addr,
            flags,
            region_type,
            host_address: start_addr, // Simplified
            allocated: true,
            dirty: false,
        };
        
        match self.virt_type {
            VirtualizationType::IntelVTx => {
                if let Some(ref mut ept) = self.ept_table {
                    ept.regions.push(region);
                }
            },
            VirtualizationType::AMDV => {
                if let Some(ref mut npt) = self.npt_table {
                    npt.regions.push(region);
                }
            },
            VirtualizationType::Unknown => {
                return Err(HypervisorError::HardwareVirtNotAvailable);
            },
        }
        
        Ok(())
    }
    
    /// Align size to next page boundary
    fn align_to_page_size(&self, size: u64) -> u64 {
        (size + PAGE_SIZE_4K - 1) & !(PAGE_SIZE_4K - 1)
    }
    
    /// Get memory statistics
    pub fn get_stats(&self) -> MemoryStats {
        MemoryStats {
            allocated_mb: self.total_memory_mb,
            used_mb: self.used_memory_mb,
            page_faults: self.page_fault_count,
        }
    }
    
    /// Invalidate TLB entry
    pub fn invalidate_tlb(&mut self, guest_addr: u64) {
        // In real implementation, would invalidate TLB entry
        self.tlb_miss_count += 1;
        info!("Invalidated TLB entry for guest address 0x{:016x}", guest_addr);
    }
    
    /// Flush all TLB entries
    pub fn flush_tlb(&mut self) {
        // In real implementation, would flush all TLB entries
        self.tlb_miss_count += 1000; // Simulate many misses
        info!("Flushed all TLB entries");
    }
    
    /// Get root page table address
    pub fn get_root_page_table_address(&self) -> Option<u64> {
        match self.virt_type {
            VirtualizationType::IntelVTx => {
                if let Some(ref ept) = self.ept_table {
                    // Return address of PML4 table
                    Some(&ept.pml4 as *const _ as u64)
                } else {
                    None
                }
            },
            VirtualizationType::AMDV => {
                if let Some(ref npt) = self.npt_table {
                    // Return address of PDPT table
                    Some(&npt.pdpt as *const _ as u64)
                } else {
                    None
                }
            },
            VirtualizationType::Unknown => None,
        }
    }
}

/// Virtualization type for memory management
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VirtualizationType {
    IntelVTx,
    AMDV,
    Unknown,
}

impl Default for EptEntry {
    fn default() -> Self {
        EptEntry {
            read: false,
            write: false,
            execute: false,
            memory_type: 0,
            ignore_pat: false,
            accessed: false,
            dirty: false,
            user_mode: false,
            global_page: false,
            present: false,
            address: 0,
        }
    }
}

impl Default for NptEntry {
    fn default() -> Self {
        NptEntry {
            read: false,
            write: false,
            execute: false,
            system: false,
            global: false,
            accessed: false,
            dirty: false,
            present: false,
            address: 0,
        }
    }
}

/// Memory Statistics structure
#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub allocated_mb: u64,
    pub used_mb: u64,
    pub page_faults: u64,
}