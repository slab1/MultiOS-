//! Memory Types and Constants
//! 
//! This module defines fundamental memory types, page sizes, and architectural constants
//! used throughout the memory management system.

use bitflags::bitflags;

#[cfg(feature = "x86_64")]
use x86_64::structures::paging::PageSize as X86PageSize;

#[cfg(feature = "x86_64")]
use x86_64::PhysAddr;

/// Page size constants for different architectures
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageSize {
    /// 4KB standard page size
    Size4K = 0x1000,
    /// 2MB large page size (x86_64)
    Size2M = 0x200000,
    /// 1GB huge page size (x86_64)
    Size1G = 0x40000000,
}

impl PageSize {
    /// Convert page size to bytes
    pub const fn as_usize(&self) -> usize {
        match self {
            PageSize::Size4K => 0x1000,
            PageSize::Size2M => 0x200000,
            PageSize::Size1G => 0x40000000,
        }
    }

    /// Check if this is a huge page
    pub const fn is_huge(&self) -> bool {
        matches!(self, PageSize::Size2M | PageSize::Size1G)
    }

    /// Get the page size for a given architecture
    pub const fn default_size() -> Self {
        #[cfg(feature = "x86_64")]
        return PageSize::Size4K;
        
        #[cfg(feature = "aarch64")]
        return PageSize::Size4K;
        
        #[cfg(feature = "riscv64")]
        return PageSize::Size4K;
        
        #[cfg(not(any(feature = "x86_64", feature = "aarch64", feature = "riscv64")))]
        return PageSize::Size4K;
    }
}

/// Physical memory address type
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysAddr(pub u64);

impl PhysAddr {
    /// Create a new physical address
    pub const fn new(addr: u64) -> Self {
        PhysAddr(addr)
    }

    /// Get the address value
    pub const fn as_u64(&self) -> u64 {
        self.0
    }

    /// Align address to page boundary
    pub const fn align_up(&self, page_size: PageSize) -> Self {
        PhysAddr((self.0 + page_size.as_usize() - 1) & !(page_size.as_usize() - 1))
    }

    /// Align address down to page boundary
    pub const fn align_down(&self, page_size: PageSize) -> Self {
        PhysAddr(self.0 & !(page_size.as_usize() - 1))
    }

    /// Check if address is page-aligned
    pub const fn is_aligned(&self, page_size: PageSize) -> bool {
        (self.0 & (page_size.as_usize() - 1)) == 0
    }

    /// Add offset to address
    pub const fn offset(&self, offset: u64) -> Self {
        PhysAddr(self.0 + offset)
    }

    #[cfg(feature = "x86_64")]
    /// Convert to x86_64 PhysAddr
    pub const fn as_x86_64(&self) -> x86_64::PhysAddr {
        x86_64::PhysAddr::new(self.0)
    }

    #[cfg(feature = "x86_64")]
    /// Create from x86_64 PhysAddr
    pub const fn from_x86_64(addr: x86_64::PhysAddr) -> Self {
        PhysAddr::new(addr.as_u64())
    }
}

/// Virtual memory address type
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct VirtAddr(pub u64);

impl VirtAddr {
    /// Create a new virtual address
    pub const fn new(addr: u64) -> Self {
        VirtAddr(addr)
    }

    /// Get the address value
    pub const fn as_u64(&self) -> u64 {
        self.0
    }

    /// Align address to page boundary
    pub const fn align_up(&self, page_size: PageSize) -> Self {
        VirtAddr((self.0 + page_size.as_usize() - 1) & !(page_size.as_usize() - 1))
    }

    /// Align address down to page boundary
    pub const fn align_down(&self, page_size: PageSize) -> Self {
        VirtAddr(self.0 & !(page_size.as_usize() - 1))
    }

    /// Check if address is page-aligned
    pub const fn is_aligned(&self, page_size: PageSize) -> bool {
        (self.0 & (page_size.as_usize() - 1)) == 0
    }

    /// Get page number for a given page size
    pub const fn page_number(&self, page_size: PageSize) -> usize {
        (self.0 / page_size.as_usize()) as usize
    }

    /// Get offset within page
    pub const fn page_offset(&self, page_size: PageSize) -> u64 {
        self.0 & (page_size.as_usize() - 1)
    }

    /// Check if this is a canonical address (x86_64)
    #[cfg(feature = "x86_64")]
    pub const fn is_canonical(&self) -> bool {
        let sign_extended = (self.0 as i64 >> 47) as u64;
        sign_extended == 0 || sign_extended == 0x1FFFFF
    }
}

/// Page frame number
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PageFrame(pub usize);

impl PageFrame {
    /// Create a new page frame
    pub const fn new(frame: usize) -> Self {
        PageFrame(frame)
    }

    /// Get frame number
    pub const fn as_usize(&self) -> usize {
        self.0
    }

    /// Convert to physical address
    pub const fn to_phys_addr(&self, page_size: PageSize) -> PhysAddr {
        PhysAddr::new((self.0 * page_size.as_usize()) as u64)
    }

    /// Create from physical address
    pub const fn from_phys_addr(addr: PhysAddr, page_size: PageSize) -> Self {
        PageFrame::new((addr.as_u64() / page_size.as_usize()) as usize)
    }
}

/// Memory permissions flags
bitflags! {
    /// Memory protection and access flags
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct MemoryFlags: u8 {
        /// No access allowed
        const NONE = 0;
        /// Read permission
        const READ = 1 << 0;
        /// Write permission
        const WRITE = 1 << 1;
        /// Execute permission
        const EXECUTE = 1 << 2;
        /// User space access allowed
        const USER = 1 << 3;
        /// Memory is globally mapped (not flushed on context switch)
        const GLOBAL = 1 << 4;
        /// Memory caching disabled
        const UNCACHED = 1 << 5;
        /// Write-through caching
        const WRITE_THROUGH = 1 << 6;
        /// Copy-on-write
        const COPY_ON_WRITE = 1 << 7;
    }
}

impl MemoryFlags {
    /// Standard kernel read-write flags
    pub const fn kernel_rw() -> Self {
        Self::READ | Self::WRITE
    }

    /// Standard kernel read-only flags
    pub const fn kernel_ro() -> Self {
        Self::READ
    }

    /// Standard user read-write flags
    pub const fn user_rw() -> Self {
        Self::READ | Self::WRITE | Self::USER
    }

    /// Standard user read-only flags
    pub const fn user_ro() -> Self {
        Self::READ | Self::USER
    }

    /// Check if memory is readable
    pub const fn is_readable(&self) -> bool {
        self.contains(Self::READ)
    }

    /// Check if memory is writable
    pub const fn is_writable(&self) -> bool {
        self.contains(Self::WRITE)
    }

    /// Check if memory is executable
    pub const fn is_executable(&self) -> bool {
        self.contains(Self::EXECUTE)
    }

    /// Check if memory is user accessible
    pub const fn is_user(&self) -> bool {
        self.contains(Self::USER)
    }
}

/// Memory region types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryRegion {
    /// Available for allocation
    Usable,
    /// Reserved by firmware/bootloader
    Reserved,
    /// ACPI reclaimable memory
    AcpiReclaimable,
    /// ACPI NVS memory
    AcpiNvs,
    /// Bad memory regions
    BadMemory,
    /// Bootloader reclaimable
    BootloaderReclaimable,
    /// Kernel and module memory
    KernelAndModules,
    /// Framebuffer memory
    Framebuffer,
    /// DMA buffer region
    DmaBuffer,
    /// Device memory
    DeviceMemory,
}

/// Page fault information
#[derive(Debug, Clone, Copy)]
pub struct PageFaultInfo {
    /// Faulting virtual address
    pub fault_addr: VirtAddr,
    /// Page fault error code
    pub error_code: PageFaultError,
    /// Instruction pointer at fault
    pub instruction_ptr: VirtAddr,
}

/// Page fault error codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PageFaultError(pub u64);

impl PageFaultError {
    /// Check if fault was caused by non-present page
    pub const fn not_present(&self) -> bool {
        (self.0 & 0x1) == 0
    }

    /// Check if fault was caused by write access
    pub const fn write_access(&self) -> bool {
        (self.0 & 0x2) != 0
    }

    /// Check if fault was in user mode
    pub const fn user_mode(&self) -> bool {
        (self.0 & 0x4) != 0
    }

    /// Check if fault was caused by reserved bit violation
    pub const fn reserved_bit_violation(&self) -> bool {
        (self.0 & 0x8) != 0
    }

    /// Check if fault was caused by instruction fetch
    pub const fn instruction_fetch(&self) -> bool {
        (self.0 & 0x10) != 0
    }

    /// Get human-readable description of fault
    pub fn describe(&self) -> &'static str {
        if self.not_present() {
            "Page not present"
        } else if self.write_access() {
            "Write protection violation"
        } else if self.instruction_fetch() {
            "Execute protection violation"
        } else {
            "Unknown page fault"
        }
    }
}

/// Memory mapping options
#[derive(Debug, Clone, Copy)]
pub struct MappingOptions {
    /// Target physical address
    pub phys_addr: PhysAddr,
    /// Memory flags and permissions
    pub flags: MemoryFlags,
    /// Page size to use
    pub page_size: PageSize,
    /// Allocate contiguous physical pages
    pub contiguous: bool,
    /// Pre-fault pages (map immediately)
    pub prefault: bool,
}

impl MappingOptions {
    /// Create default mapping options
    pub const fn default() -> Self {
        Self {
            phys_addr: PhysAddr::new(0),
            flags: MemoryFlags::kernel_rw(),
            page_size: PageSize::default_size(),
            contiguous: false,
            prefault: false,
        }
    }

    /// Create kernel mapping options
    pub const fn kernel() -> Self {
        Self::default()
    }

    /// Create user mapping options
    pub const fn user() -> Self {
        Self {
            flags: MemoryFlags::user_rw(),
            ..Self::default()
        }
    }
}

/// Memory statistics
#[derive(Debug, Clone, Copy)]
pub struct MemoryStats {
    /// Total physical memory in bytes
    pub total_memory: u64,
    /// Used physical memory in bytes
    pub used_memory: u64,
    /// Available physical memory in bytes
    pub available_memory: u64,
    /// Total number of pages
    pub total_pages: usize,
    /// Number of used pages
    pub used_pages: usize,
    /// Number of free pages
    pub free_pages: usize,
    /// Number of reserved pages
    pub reserved_pages: usize,
}

impl Default for MemoryStats {
    fn default() -> Self {
        Self {
            total_memory: 0,
            used_memory: 0,
            available_memory: 0,
            total_pages: 0,
            used_pages: 0,
            free_pages: 0,
            reserved_pages: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phys_addr_alignment() {
        let addr = PhysAddr::new(0x1234);
        let aligned = addr.align_up(PageSize::Size4K);
        
        assert_eq!(aligned.as_u64(), 0x2000);
        
        let aligned_down = addr.align_down(PageSize::Size4K);
        assert_eq!(aligned_down.as_u64(), 0x1000);
    }

    #[test]
    fn test_virt_addr_page_number() {
        let addr = VirtAddr::new(0x12345);
        let page_num = addr.page_number(PageSize::Size4K);
        let offset = addr.page_offset(PageSize::Size4K);
        
        assert_eq!(page_num, 0x12);
        assert_eq!(offset, 0x345);
    }

    #[test]
    fn test_memory_flags() {
        let flags = MemoryFlags::kernel_rw();
        assert!(flags.is_readable());
        assert!(flags.is_writable());
        assert!(!flags.is_executable());
        assert!(!flags.is_user());
        
        let user_flags = MemoryFlags::user_ro();
        assert!(user_flags.is_readable());
        assert!(user_flags.is_user());
        assert!(!user_flags.is_writable());
    }

    #[test]
    fn test_page_fault_error() {
        let error = PageFaultError(0x5); // Present + User mode
        assert!(!error.not_present());
        assert!(!error.write_access());
        assert!(error.user_mode());
    }

    #[test]
    fn test_memory_stats() {
        let stats = MemoryStats {
            total_memory: 16_777_216, // 16MB
            used_memory: 4_194_304,   // 4MB
            available_memory: 12_582_912, // 12MB
            total_pages: 4096,
            used_pages: 1024,
            free_pages: 3072,
            reserved_pages: 0,
        };
        
        assert_eq!(stats.total_pages * 4096, stats.total_memory as usize);
        assert_eq!(stats.free_pages + stats.used_pages, stats.total_pages);
    }
}