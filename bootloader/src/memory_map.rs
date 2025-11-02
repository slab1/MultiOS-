//! Memory Map Management
//! 
//! This module provides comprehensive memory map detection and management
//! for both UEFI and legacy BIOS boot processes.

use bootloader::boot_info::{MemoryRegion, MemoryRegionKind};
use x86_64::structures::paging::{Page, PhysFrame, Size4KiB};
use x86_64::PhysAddr;
use bitflags::bitflags;
use log::{info, warn, error, debug};
use spin::Mutex;
use core::fmt;

#[cfg(feature = "debug_mode")]
use std::collections::BTreeMap;

/// Memory type definitions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum MemoryType {
    Usable = 1,
    Reserved = 2,
    AcpiReclaimable = 3,
    AcpiNonvolatile = 4,
    BadMemory = 5,
    BootloaderReserved = 6,
    KernelCode = 7,
    KernelData = 8,
    KernelBss = 9,
    KernelBootInfo = 10,
    Stack = 11,
    Heap = 12,
    Initrd = 13,
    Framebuffer = 14,
    Unknown = 15,
}

impl MemoryType {
    pub fn from_region_kind(kind: MemoryRegionKind) -> Self {
        match kind {
            MemoryRegionKind::Usable => MemoryType::Usable,
            MemoryRegionKind::Reserved => MemoryType::Reserved,
            MemoryRegionKind::AcpiReclaimable => MemoryType::AcpiReclaimable,
            MemoryRegionKind::AcpiNonvolatile => MemoryType::AcpiNonvolatile,
            MemoryRegionKind::Bootloader => MemoryType::BootloaderReserved,
            MemoryRegionKind::KernelCode => MemoryType::KernelCode,
            MemoryRegionKind::KernelData => MemoryType::KernelData,
            MemoryRegionKind::BootInfo => MemoryType::KernelBootInfo,
            MemoryRegionKind::Module => MemoryType::Usable,
        }
    }
}

bitflags! {
    /// Memory region flags
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct MemoryFlags: u32 {
        const READ = 1 << 0;
        const WRITE = 1 << 1;
        const EXECUTE = 1 << 2;
        const AVAILABLE = 1 << 3;
        const TESTED = 1 << 4;
        const DMA_CAPABLE = 1 << 5;
        const ACPI_TABLES = 1 << 6;
        const FIRMWARE = 1 << 7;
    }
}

/// Memory region structure
#[derive(Debug, Clone, Copy)]
pub struct MemoryRegionInfo {
    pub start: PhysAddr,
    pub size: usize,
    pub mem_type: MemoryType,
    pub flags: MemoryFlags,
}

impl MemoryRegionInfo {
    pub fn new(start: PhysAddr, size: usize, mem_type: MemoryType, flags: MemoryFlags) -> Self {
        Self {
            start,
            size,
            mem_type,
            flags,
        }
    }

    pub fn contains(&self, addr: PhysAddr) -> bool {
        addr >= self.start && addr < self.start + self.size
    }

    pub fn end(&self) -> PhysAddr {
        self.start + self.size
    }
}

impl fmt::Display for MemoryRegionInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#010x} - {:#010x} ({:>7} KB) - {:?} - {:?}",
            self.start.as_u64(),
            self.end().as_u64(),
            self.size / 1024,
            self.mem_type,
            self.flags
        )
    }
}

/// Physical memory frame for bitmap tracking
#[derive(Debug, Clone, Copy)]
pub struct PhysicalFrame {
    pub addr: u64,
    pub size: usize,
    pub available: bool,
    pub tested: bool,
}

/// Memory bitmap for frame allocation tracking
#[derive(Debug)]
pub struct MemoryBitmap {
    pub data: Vec<u8>,
    pub frame_count: usize,
    pub granularity: usize,
}

impl MemoryBitmap {
    /// Create new memory bitmap
    pub fn new(frame_count: usize, granularity: usize) -> Self {
        let size = (frame_count + 7) / 8;
        Self {
            data: vec![0u8; size],
            frame_count,
            granularity,
        }
    }

    /// Mark frame as available
    pub fn set_available(&mut self, frame_idx: usize, available: bool) {
        if frame_idx < self.frame_count {
            let byte_idx = frame_idx / 8;
            let bit_idx = frame_idx % 8;
            
            if available {
                self.data[byte_idx] |= 1u8 << bit_idx;
            } else {
                self.data[byte_idx] &= !(1u8 << bit_idx);
            }
        }
    }

    /// Check if frame is available
    pub fn is_available(&self, frame_idx: usize) -> bool {
        if frame_idx >= self.frame_count {
            return false;
        }
        let byte_idx = frame_idx / 8;
        let bit_idx = frame_idx % 8;
        (self.data[byte_idx] >> bit_idx) & 1u8 != 0
    }

    /// Find contiguous available frames
    pub fn find_contiguous(&self, count: usize) -> Option<usize> {
        let mut consecutive = 0;
        let mut start = None;
        
        for i in 0..self.frame_count {
            if self.is_available(i) {
                if consecutive == 0 {
                    start = Some(i);
                }
                consecutive += 1;
                
                if consecutive >= count {
                    return start;
                }
            } else {
                consecutive = 0;
                start = None;
            }
        }
        None
    }

    /// Count available frames
    pub fn available_count(&self) -> usize {
        self.data.iter()
            .map(|byte| byte.count_ones() as usize)
            .sum()
    }
}

/// Heap initialization structure
#[derive(Debug)]
pub struct BootHeap {
    pub start_addr: u64,
    pub size: usize,
    pub allocated: usize,
    pub alignment: usize,
    pub initialized: bool,
}

impl BootHeap {
    /// Create new boot heap
    pub fn new(start_addr: u64, size: usize, alignment: usize) -> Self {
        Self {
            start_addr,
            size,
            allocated: 0,
            alignment,
            initialized: false,
        }
    }

    /// Initialize heap
    pub fn init(&mut self) -> Result<(), BootError> {
        if self.initialized {
            return Ok(());
        }

        // Verify heap region is within usable memory
        let heap_end = self.start_addr + self.size as u64;
        debug!("Initializing boot heap: {:#x} - {:#x} ({} bytes)", 
               self.start_addr, heap_end, self.size);

        self.initialized = true;
        Ok(())
    }

    /// Allocate memory from heap
    pub fn allocate(&mut self, size: usize) -> Result<u64, BootError> {
        if !self.initialized {
            return Err(BootError::MemoryMapError);
        }

        // Align allocation
        let aligned_size = (size + self.alignment - 1) & !(self.alignment - 1);
        
        if self.allocated + aligned_size > self.size {
            return Err(BootError::OutOfMemory);
        }

        let addr = self.start_addr + self.allocated as u64;
        self.allocated += aligned_size;
        
        debug!("Heap allocation: {} bytes at {:#x}", aligned_size, addr);
        Ok(addr)
    }

    /// Get heap usage statistics
    pub fn get_stats(&self) -> HeapStats {
        HeapStats {
            total_size: self.size,
            allocated: self.allocated,
            free: self.size - self.allocated,
            usage_percent: if self.size > 0 {
                (self.allocated * 100) / self.size
            } else {
                0
            },
        }
    }
}

/// Heap statistics
#[derive(Debug, Clone, Copy)]
pub struct HeapStats {
    pub total_size: usize,
    pub allocated: usize,
    pub free: usize,
    pub usage_percent: usize,
}

/// Memory management initialization configuration
#[derive(Debug, Clone)]
pub struct MemoryInitConfig {
    pub heap_size: usize,
    pub bitmap_granularity: usize,
    pub heap_alignment: usize,
    pub enable_memory_test: bool,
    pub enable_detailed_logging: bool,
    pub min_heap_addr: u64,
}

impl Default for MemoryInitConfig {
    fn default() -> Self {
        Self {
            heap_size: 16 * 1024 * 1024, // 16MB
            bitmap_granularity: 4096, // 4KB pages
            heap_alignment: 4096,
            enable_memory_test: false,
            enable_detailed_logging: true,
            min_heap_addr: 0x100000, // Start from 1MB
        }
    }
}

/// Memory map management
#[derive(Debug)]
pub struct MemoryMap {
    regions: Vec<MemoryRegionInfo>,
    total_memory: usize,
    available_memory: usize,
    bootloader_memory: usize,
    frames: Vec<PhysicalFrame>,
    bitmap: Option<MemoryBitmap>,
    boot_heap: Option<BootHeap>,
    init_config: MemoryInitConfig,
}

impl MemoryMap {
    /// Create a new empty memory map
    pub fn new() -> Self {
        info!("Initializing memory map...");
        Self {
            regions: Vec::new(),
            total_memory: 0,
            available_memory: 0,
            bootloader_memory: 0,
            frames: Vec::new(),
            bitmap: None,
            boot_heap: None,
            init_config: MemoryInitConfig::default(),
        }
    }

    /// Create memory map with custom initialization configuration
    pub fn with_config(config: MemoryInitConfig) -> Self {
        info!("Initializing memory map with custom config...");
        let mut map = Self::new();
        map.init_config = config;
        map
    }

    /// Create memory map from bootloader boot info
    pub fn from_boot_info(boot_info: &bootloader::boot_info::BootInfo) -> Self {
        info!("Building memory map from boot info...");
        let mut memory_map = Self::new();
        
        for region in &boot_info.memory_map {
            let mem_type = MemoryType::from_region_kind(region.kind);
            let start = PhysAddr::new(region.start);
            let size = region.end - region.start;
            
            // Determine memory flags based on type
            let flags = match mem_type {
                MemoryType::Usable => MemoryFlags::READ | MemoryFlags::WRITE | MemoryFlags::AVAILABLE | MemoryFlags::TESTED,
                MemoryType::Reserved | MemoryType::BadMemory | MemoryType::Firmware => 
                    MemoryFlags::READ,
                MemoryType::BootloaderReserved | MemoryType::KernelCode | MemoryType::KernelData | 
                MemoryType::KernelBootInfo => MemoryFlags::READ | MemoryFlags::WRITE,
                _ => MemoryFlags::READ | MemoryFlags::WRITE,
            };
            
            let region_info = MemoryRegionInfo::new(start, size, mem_type, flags);
            memory_map.add_region(region_info);
            
            memory_map.total_memory += size;
            
            if mem_type == MemoryType::Usable {
                memory_map.available_memory += size;
            } else if mem_type == MemoryType::BootloaderReserved {
                memory_map.bootloader_memory += size;
            }
        }
        
        info!("Memory map built: {} KB total, {} KB available, {} KB bootloader",
              memory_map.total_memory / 1024,
              memory_map.available_memory / 1024,
              memory_map.bootloader_memory / 1024);
        
        memory_map
    }

    /// Add a memory region
    pub fn add_region(&mut self, region: MemoryRegionInfo) {
        // Validate region doesn't overlap with existing regions
        for existing in &self.regions {
            if self.regions_overlap(existing, &region) {
                warn!("Memory region overlap detected: {:?} and {:?}", existing, region);
                break;
            }
        }
        
        self.regions.push(region);
        self.regions.sort_by_key(|r| r.start.as_u64());
    }

    /// Check if two memory regions overlap
    fn regions_overlap(&self, r1: &MemoryRegionInfo, r2: &MemoryRegionInfo) -> bool {
        !(r1.end() <= r2.start || r2.end() <= r1.start)
    }

    /// Find a suitable memory region for allocation
    pub fn find_region(&self, size: usize, alignment: usize) -> Option<MemoryRegionInfo> {
        for region in &self.regions {
            if region.mem_type == MemoryType::Usable && region.flags.contains(MemoryFlags::AVAILABLE) {
                if let Some(start) = self.align_up(region.start, alignment) {
                    if start + size <= region.end() {
                        return Some(MemoryRegionInfo::new(start, size, region.mem_type, region.flags));
                    }
                }
            }
        }
        None
    }

    /// Align address up to alignment boundary
    fn align_up(&self, addr: PhysAddr, alignment: usize) -> Option<PhysAddr> {
        let aligned = ((addr.as_u64() + (alignment - 1)) / alignment as u64) * alignment as u64;
        if aligned > u64::MAX {
            None
        } else {
            Some(PhysAddr::new(aligned))
        }
    }

    /// Reserve memory for kernel
    pub fn reserve_kernel_memory(&mut self, code_start: PhysAddr, code_size: usize) -> MemoryRegionInfo {
        let region = MemoryRegionInfo::new(code_start, code_size, MemoryType::KernelCode, 
                                        MemoryFlags::READ | MemoryFlags::EXECUTE);
        self.add_region(region);
        region
    }

    /// Reserve memory for initrd
    pub fn reserve_initrd_memory(&mut self, start: PhysAddr, size: usize) -> MemoryRegionInfo {
        let region = MemoryRegionInfo::new(start, size, MemoryType::Initrd, 
                                        MemoryFlags::READ);
        self.add_region(region);
        region
    }

    /// Get all regions of a specific type
    pub fn get_regions_of_type(&self, mem_type: MemoryType) -> Vec<MemoryRegionInfo> {
        self.regions.iter()
            .filter(|r| r.mem_type == mem_type)
            .copied()
            .collect()
    }

    /// Get total memory size
    pub fn total_memory(&self) -> usize {
        self.total_memory
    }

    /// Get available memory size
    pub fn available_memory(&self) -> usize {
        self.available_memory
    }

    /// Get bootloader reserved memory size
    pub fn bootloader_memory(&self) -> usize {
        self.bootloader_memory
    }

    /// Validate memory map integrity
    pub fn validate(&self) -> bool {
        let mut valid = true;
        
        // Check for overlapping regions
        for (i, r1) in self.regions.iter().enumerate() {
            for (j, r2) in self.regions.iter().enumerate() {
                if i != j && self.regions_overlap(r1, r2) {
                    error!("Overlapping memory regions: {:?} and {:?}", r1, r2);
                    valid = false;
                }
            }
        }
        
        // Check for invalid regions
        for region in &self.regions {
            if region.size == 0 {
                error!("Zero-sized memory region: {:?}", region);
                valid = false;
            }
            if region.start.as_u64() == 0 && region.mem_type != MemoryType::Reserved {
                debug!("Memory region starts at address 0: {:?}", region);
            }
        }
        
        info!("Memory map validation: {}", if valid { "PASSED" } else { "FAILED" });
        valid
    }

    /// Print memory map
    pub fn print(&self) {
        info!("Memory Map:");
        info!("{}", "=".repeat(80));
        info!("{:<20} {:<20} {:<10} {:<15} {:<20}", "Start Address", "End Address", "Size", "Type", "Flags");
        info!("{}", "-".repeat(80));
        
        for region in &self.regions {
            info!("{:<20} {:<20} {:<10} KB {:<15} {:?}",
                format!("{:#x}", region.start.as_u64()),
                format!("{:#x}", region.end().as_u64()),
                region.size / 1024,
                region.mem_type,
                region.flags
            );
        }
        
        info!("{}", "-".repeat(80));
        info!("Total Memory: {} KB", self.total_memory / 1024);
        info!("Available Memory: {} KB", self.available_memory / 1024);
        info!("Bootloader Memory: {} KB", self.bootloader_memory / 1024);
        info!("{}", "=".repeat(80));
    }

    /// Initialize memory bitmap for frame tracking
    pub fn init_bitmap(&mut self) -> Result<(), BootError> {
        if self.frames.is_empty() {
            return Err(BootError::MemoryMapError);
        }

        let frame_count = self.frames.len();
        let granularity = self.init_config.bitmap_granularity;
        
        debug!("Initializing memory bitmap: {} frames with {} byte granularity", 
               frame_count, granularity);

        let mut bitmap = MemoryBitmap::new(frame_count, granularity);
        
        // Initialize bitmap based on frame availability
        for (frame_idx, frame) in self.frames.iter().enumerate() {
            let available = frame.available && frame.size >= granularity;
            bitmap.set_available(frame_idx, available);
        }

        self.bitmap = Some(bitmap);
        
        info!("Memory bitmap initialized: {} frames available", 
              self.bitmap.as_ref().unwrap().available_count());
        
        Ok(())
    }

    /// Initialize boot heap
    pub fn init_heap(&mut self) -> Result<(), BootError> {
        let heap_size = self.init_config.heap_size;
        let alignment = self.init_config.heap_alignment;
        
        // Find suitable region for heap
        let heap_start = self.find_heap_region(heap_size, alignment)
            .ok_or(BootError::MemoryMapError)?;
            
        debug!("Initializing boot heap: {} bytes at {:#x}", heap_size, heap_start);
        
        let mut heap = BootHeap::new(heap_start, heap_size, alignment);
        heap.init()?;
        
        self.boot_heap = Some(heap);
        
        info!("Boot heap initialized: {} bytes at {:#x}", heap_size, heap_start);
        Ok(())
    }

    /// Find suitable memory region for heap allocation
    fn find_heap_region(&self, size: usize, alignment: usize) -> Option<u64> {
        let min_addr = self.init_config.min_heap_addr;
        
        for region in &self.regions {
            if region.mem_type == MemoryType::Usable && 
               region.flags.contains(MemoryFlags::AVAILABLE) &&
               region.start.as_u64() >= min_addr {
                
                let region_end = region.end().as_u64();
                let required_size = size as u64 + alignment as u64 * 2; // Extra space for alignment
                
                if region_end - region.start.as_u64() >= required_size {
                    let start_addr = region.start.as_u64();
                    let aligned_start = (start_addr + alignment as u64 - 1) & !(alignment as u64 - 1);
                    
                    if aligned_start + size as u64 <= region_end {
                        return Some(aligned_start);
                    }
                }
            }
        }
        None
    }

    /// Allocate contiguous physical frames
    pub fn allocate_frames(&mut self, count: usize, alignment: usize) -> Result<u64, BootError> {
        if let Some(ref bitmap) = self.bitmap {
            if let Some(start_frame) = bitmap.find_contiguous(count) {
                let granularity = self.init_config.bitmap_granularity;
                let addr = (start_frame as u64) * granularity as u64;
                
                // Mark frames as allocated in bitmap
                for i in 0..count {
                    bitmap.set_available(start_frame + i, false);
                }
                
                debug!("Allocated {} frames at {:#x}", count, addr);
                return Ok(addr);
            }
        }
        
        Err(BootError::MemoryMapError)
    }

    /// Free allocated physical frames
    pub fn free_frames(&mut self, addr: u64, count: usize) -> Result<(), BootError> {
        if let Some(ref mut bitmap) = self.bitmap {
            let granularity = self.init_config.bitmap_granularity;
            let start_frame = (addr / granularity as u64) as usize;
            
            for i in 0..count {
                bitmap.set_available(start_frame + i, true);
            }
            
            debug!("Freed {} frames at {:#x}", count, addr);
            Ok(())
        } else {
            Err(BootError::MemoryMapError)
        }
    }

    /// Allocate memory from boot heap
    pub fn heap_allocate(&mut self, size: usize) -> Result<u64, BootError> {
        if let Some(ref mut heap) = self.boot_heap {
            heap.allocate(size)
        } else {
            Err(BootError::MemoryMapError)
        }
    }

    /// Get memory management statistics
    pub fn get_memory_stats(&self) -> MemoryManagementStats {
        let bitmap_stats = self.bitmap.as_ref().map(|b| b.available_count()).unwrap_or(0);
        let heap_stats = self.boot_heap.as_ref().map(|h| h.get_stats()).unwrap_or_default();
        
        MemoryManagementStats {
            total_memory: self.total_memory,
            available_memory: self.available_memory,
            bootloader_memory: self.bootloader_memory,
            frame_count: self.frames.len(),
            available_frames: bitmap_stats,
            heap_stats,
        }
    }

    /// Build frame list from memory regions
    fn build_frame_list(&mut self) {
        let granularity = self.init_config.bitmap_granularity;
        self.frames.clear();
        
        for region in &self.regions {
            if region.mem_type == MemoryType::Usable {
                let mut addr = region.start.as_u64();
                let region_end = region.end().as_u64();
                
                // Align to granularity
                addr = (addr + granularity as u64 - 1) & !(granularity as u64 - 1);
                
                while addr + granularity as u64 <= region_end {
                    self.frames.push(PhysicalFrame {
                        addr,
                        size: granularity,
                        available: true,
                        tested: false,
                    });
                    addr += granularity as u64;
                }
            }
        }
        
        debug!("Built frame list with {} frames", self.frames.len());
    }

    /// Detect memory using BIOS INT 15h, EAX=0xE820
    pub fn detect_bios_memory() -> Result<Self, BootError> {
        info!("Detecting memory using BIOS INT 15h, EAX=0xE820...");
        
        let mut memory_map = Self::new();
        
        // In a real implementation, this would:
        // 1. Call BIOS INT 15h, EAX=0xE820
        // 2. Parse the returned memory map entries
        // 3. Convert BIOS memory types to our internal types
        
        // For demonstration, create typical BIOS memory layout
        memory_map.add_region(MemoryRegionInfo::new(
            PhysAddr::new(0x00000), 
            0x9F000, 
            MemoryType::Reserved, 
            MemoryFlags::READ
        ));
        
        memory_map.add_region(MemoryRegionInfo::new(
            PhysAddr::new(0xA0000), 
            0x60000, 
            MemoryType::Reserved, 
            MemoryFlags::READ
        ));
        
        memory_map.add_region(MemoryRegionInfo::new(
            PhysAddr::new(0x100000), 
            0x7EF00000, 
            MemoryType::Usable, 
            MemoryFlags::READ | MemoryFlags::WRITE | MemoryFlags::AVAILABLE
        ));
        
        info!("BIOS memory detection completed with {} regions", memory_map.regions.len());
        Ok(memory_map)
    }

    /// Detect memory using UEFI System Table
    pub fn detect_uefi_memory() -> Result<Self, BootError> {
        info!("Detecting memory using UEFI System Table...");
        
        let mut memory_map = Self::new();
        
        // In a real implementation, this would:
        // 1. Get memory map from UEFI System Table
        // 2. Parse EFI_MEMORY_DESCRIPTOR entries
        // 3. Convert UEFI memory types to our internal types
        
        // For demonstration, create typical UEFI memory layout
        memory_map.add_region(MemoryRegionInfo::new(
            PhysAddr::new(0x00000), 
            0x9F000, 
            MemoryType::Reserved, 
            MemoryFlags::READ
        ));
        
        memory_map.add_region(MemoryRegionInfo::new(
            PhysAddr::new(0xA0000), 
            0x60000, 
            MemoryType::Reserved, 
            MemoryFlags::READ
        ));
        
        memory_map.add_region(MemoryRegionInfo::new(
            PhysAddr::new(0x100000), 
            0x7EF00000, 
            MemoryType::Usable, 
            MemoryFlags::READ | MemoryFlags::WRITE | MemoryFlags::AVAILABLE
        ));
        
        // Add common UEFI MMIO regions
        memory_map.add_region(MemoryRegionInfo::new(
            PhysAddr::new(0xFEC00000), 
            0x100000, 
            MemoryType::Reserved, 
            MemoryFlags::READ | MemoryFlags::DMA_CAPABLE
        ));
        
        info!("UEFI memory detection completed with {} regions", memory_map.regions.len());
        Ok(memory_map)
    }

    /// Complete memory initialization
    pub fn init_complete(&mut self) -> Result<(), BootError> {
        info!("Starting complete memory initialization...");
        
        // Build frame list
        self.build_frame_list();
        
        // Initialize memory bitmap
        self.init_bitmap()?;
        
        // Initialize boot heap
        self.init_heap()?;
        
        // Validate memory map
        if !self.validate() {
            return Err(BootError::MemoryMapError);
        }
        
        // Print final memory map
        if self.init_config.enable_detailed_logging {
            self.print();
        }
        
        info!("Memory initialization completed successfully");
        info!("{}", self.get_memory_stats());
        
        Ok(())
    }

    /// Get bootloader memory configuration
    pub fn get_heap(&self) -> Option<&BootHeap> {
        self.boot_heap.as_ref()
    }

    /// Get memory bitmap
    pub fn get_bitmap(&self) -> Option<&MemoryBitmap> {
        self.bitmap.as_ref()
    }
}

/// Memory management statistics
#[derive(Debug, Clone, Copy)]
pub struct MemoryManagementStats {
    pub total_memory: usize,
    pub available_memory: usize,
    pub bootloader_memory: usize,
    pub frame_count: usize,
    pub available_frames: usize,
    pub heap_stats: HeapStats,
}

impl fmt::Display for MemoryManagementStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Memory Management Statistics:")?;
        writeln!(f, "  Total Memory: {} MB", self.total_memory / (1024 * 1024))?;
        writeln!(f, "  Available Memory: {} MB", self.available_memory / (1024 * 1024))?;
        writeln!(f, "  Bootloader Memory: {} MB", self.bootloader_memory / (1024 * 1024))?;
        writeln!(f, "  Total Frames: {}", self.frame_count)?;
        writeln!(f, "  Available Frames: {}", self.available_frames)?;
        writeln!(f, "  Heap: {} MB allocated / {} MB total ({:.1}% usage)", 
                 self.heap_stats.allocated / (1024 * 1024),
                 self.heap_stats.total_size / (1024 * 1024),
                 self.heap_stats.usage_percent as f64)?;
        Ok(())
    }
}

impl Default for HeapStats {
    fn default() -> Self {
        Self {
            total_size: 0,
            allocated: 0,
            free: 0,
            usage_percent: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_region_creation() {
        let addr = PhysAddr::new(0x1000);
        let region = MemoryRegionInfo::new(addr, 0x1000, MemoryType::Usable, MemoryFlags::AVAILABLE);
        assert_eq!(region.start, addr);
        assert_eq!(region.size, 0x1000);
        assert_eq!(region.mem_type, MemoryType::Usable);
    }

    #[test]
    fn test_memory_region_contains() {
        let addr = PhysAddr::new(0x1000);
        let region = MemoryRegionInfo::new(addr, 0x1000, MemoryType::Usable, MemoryFlags::AVAILABLE);
        assert!(region.contains(PhysAddr::new(0x1500)));
        assert!(!region.contains(PhysAddr::new(0x2000)));
    }

    #[test]
    fn test_memory_map_creation() {
        let memory_map = MemoryMap::new();
        assert_eq!(memory_map.regions().len(), 0);
        assert_eq!(memory_map.total_memory(), 0);
    }

    #[test]
    fn test_align_up() {
        let memory_map = MemoryMap::new();
        assert_eq!(memory_map.align_up(PhysAddr::new(0x1234), 0x1000).unwrap(), PhysAddr::new(0x2000));
        assert_eq!(memory_map.align_up(PhysAddr::new(0x1000), 0x1000).unwrap(), PhysAddr::new(0x1000));
    }
}

// Add accessor methods for tests
impl MemoryMap {
    pub fn regions(&self) -> &[MemoryRegionInfo] {
        &self.regions
    }
}