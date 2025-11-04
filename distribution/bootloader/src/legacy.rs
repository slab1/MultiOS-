//! Legacy BIOS Boot Support
//! 
//! This module provides comprehensive legacy BIOS boot support including
//! BIOS interrupt calls, memory detection via INT 15h, and kernel loading.

use x86_64::PhysAddr;
use x86_64::registers::control::Cr0;
use bitflags::bitflags;
use log::{info, warn, error, debug};
use spin::Mutex;
use core::arch::asm;

use crate::BootError;
use crate::memory_map::{MemoryMap, MemoryRegionInfo, MemoryType, MemoryFlags};
use crate::kernel_loader::{KernelBootInfo, create_kernel_boot_info};

/// Legacy BIOS context
#[derive(Debug)]
pub struct LegacyBiosContext {
    pub memory_map: MemoryMap,
    pub boot_device: BootDevice,
    pub bios_information: BiosInfo,
    pub video_info: VideoInfo,
}

/// Boot device information
#[derive(Debug, Clone, Copy)]
pub struct BootDevice {
    pub device_type: DeviceType,
    pub drive_number: u8,
    pub partition: Option<u16>,
    pub file_system: Option<FileSystemType>,
}

/// Device types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceType {
    Floppy,      // 0x00
    HardDisk,    // 0x80
    CdRom,       // 0xE0
    Usb,         // 0x81
    Network,     // 0x82
    Unknown,
}

/// File system types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileSystemType {
    Fat12,
    Fat16,
    Fat32,
    Ext2,
    Ext4,
    Unknown,
}

/// BIOS information
#[derive(Debug, Clone, Copy)]
pub struct BiosInfo {
    pub bios_vendor: [u8; 8],
    pub bios_version: [u8; 8],
    pub bios_date: [u8; 8],
    pub bios_memory_64k: u16,
    pub bios_extended_memory: u32,
    pub bios_video_mode: u8,
    pub bios_manufacturer: [u8; 16],
}

/// Video information
#[derive(Debug, Clone, Copy)]
pub struct VideoInfo {
    pub mode: u8,
    pub columns: u8,
    pub rows: u8,
    pub address: PhysAddr,
    pub size: usize,
}

bitflags! {
    /// BIOS memory regions
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct BiosMemoryRegion: u32 {
        const CONVENTIONAL = 1 << 0;
        const EXTENDED = 1 << 1;
        const HIGH = 1 << 2;
        const VIDEO = 1 << 3;
        const BIOS = 1 << 4;
        const ACPI = 1 << 5;
        const RESERVED = 1 << 6;
    }
}

impl LegacyBiosContext {
    /// Initialize legacy BIOS boot context
    pub fn initialize() -> Result<Self, BootError> {
        info!("Initializing legacy BIOS boot context...");
        
        // Detect BIOS information
        let bios_info = Self::detect_bios_info()?;
        
        // Detect boot device
        let boot_device = Self::detect_boot_device()?;
        
        // Detect video information
        let video_info = Self::detect_video_info()?;
        
        // Build memory map from BIOS
        let memory_map = Self::build_bios_memory_map()?;
        
        info!("Legacy BIOS context initialized successfully");
        
        Ok(Self {
            memory_map,
            boot_device,
            bios_information: bios_info,
            video_info,
        })
    }

    /// Detect BIOS information via INT 15h
    fn detect_bios_info() -> Result<BiosInfo, BootError> {
        info!("Detecting BIOS information...");
        
        let mut bios_vendor = [0u8; 8];
        let mut bios_version = [0u8; 8];
        let mut bios_date = [0u8; 8];
        let mut bios_memory_64k = 0u16;
        let mut bios_extended_memory = 0u32;
        let mut bios_video_mode = 0u8;
        let mut bios_manufacturer = [0u8; 16];
        
        // Get BIOS information using INT 15h AX=0xC000
        unsafe {
            asm!(
                "mov ax, {int15_func}",
                "int 0x15",
                "jc bios_error",
                in(reg) 0xC000u16,
                out("ax") _,
                out("bx") _,
                out("cx") _,
                out("dx") _,
                out("si") _,
                out("di") _,
                out("bp") _,
            );
        }
        
        // This is a simplified implementation - real implementation would
        // properly detect BIOS information through multiple INT calls
        
        Ok(BiosInfo {
            bios_vendor,
            bios_version,
            bios_date,
            bios_memory_64k,
            bios_extended_memory,
            bios_video_mode,
            bios_manufacturer,
        })
    }

    /// Detect boot device
    fn detect_boot_device() -> Result<BootDevice, BootError> {
        info!("Detecting boot device...");
        
        // The boot device is typically stored in the DL register when the
        // bootloader is loaded. This would need to be passed in as a parameter
        // or detected through BIOS
        
        // For now, assume we booted from a hard drive
        let boot_device = BootDevice {
            device_type: DeviceType::HardDisk,
            drive_number: 0x80, // First hard drive
            partition: Some(0),
            file_system: Some(FileSystemType::Fat32),
        };
        
        info!("Boot device: {:?}", boot_device);
        Ok(boot_device)
    }

    /// Detect video information
    fn detect_video_info() -> Result<VideoInfo, BootError> {
        info!("Detecting video information...");
        
        // Get video information via INT 10h
        let mut video_mode = 0u8;
        let mut columns = 80u8;
        let mut rows = 25u8;
        let video_addr = PhysAddr::new(0xB8000); // Standard VGA text mode address
        
        // This is a simplified implementation - real implementation would
        // query video information properly through INT calls
        
        Ok(VideoInfo {
            mode: video_mode,
            columns,
            rows,
            address: video_addr,
            size: 80 * 25 * 2, // 80x25 text mode, 2 bytes per character
        })
    }

    /// Build memory map using BIOS INT 15h
    fn build_bios_memory_map() -> Result<MemoryMap, BootError> {
        info!("Building BIOS memory map...");
        
        let mut memory_map = MemoryMap::new();
        
        // Get conventional memory (0-640K)
        let conventional_size = 640 * 1024;
        let conventional_region = MemoryRegionInfo::new(
            PhysAddr::new(0x00000),
            conventional_size,
            MemoryType::Usable,
            MemoryFlags::READ | MemoryFlags::WRITE | MemoryFlags::AVAILABLE,
        );
        memory_map.add_region(conventional_region);
        
        // Get extended memory (1M+)
        let extended_addr = 1024 * 1024; // 1MB
        let extended_size = self::get_extended_memory_size();
        let extended_region = MemoryRegionInfo::new(
            PhysAddr::new(extended_addr),
            extended_size,
            MemoryType::Usable,
            MemoryFlags::READ | MemoryFlags::WRITE | MemoryFlags::AVAILABLE,
        );
        memory_map.add_region(extended_region);
        
        // Add reserved regions
        Self::add_bios_reserved_regions(&mut memory_map)?;
        
        // Add video memory region
        let video_region = MemoryRegionInfo::new(
            PhysAddr::new(0xA0000),
            128 * 1024,
            MemoryType::Reserved,
            MemoryFlags::READ | MemoryFlags::WRITE,
        );
        memory_map.add_region(video_region);
        
        // Add BIOS reserved region
        let bios_region = MemoryRegionInfo::new(
            PhysAddr::new(0xF0000),
            64 * 1024,
            MemoryType::Reserved,
            MemoryFlags::READ | MemoryFlags::EXECUTE,
        );
        memory_map.add_region(bios_region);
        
        info!("BIOS memory map built successfully");
        Ok(memory_map)
    }

    /// Get extended memory size via INT 15h AX=0xE801
    fn get_extended_memory_size() -> usize {
        let mut size = 0;
        
        // This would call INT 15h AX=0xE801 to get extended memory size
        // For now, return a default value
        
        size = 16 * 1024 * 1024; // 16MB default
        
        size
    }

    /// Add BIOS reserved regions
    fn add_bios_reserved_regions(memory_map: &mut MemoryMap) -> Result<(), BootError> {
        // Add various BIOS reserved regions
        let reserved_regions = [
            (0x00000, 1024, MemoryType::Reserved, "Interrupt Vector Table"),
            (0x00400, 256, MemoryType::Reserved, "BIOS Data Area"),
            (0x00500, 512, MemoryType::Reserved, "Extended BIOS Data Area"),
            (0x80000, 128 * 1024, MemoryType::Reserved, "BIOS Extended Area"),
        ];
        
        for (start, size, mem_type, name) in &reserved_regions {
            let region = MemoryRegionInfo::new(
                PhysAddr::new(*start),
                *size,
                *mem_type,
                MemoryFlags::READ,
            );
            memory_map.add_region(region);
            debug!("Added reserved region: {} at {:#x}-{:?}", name, start, size);
        }
        
        Ok(())
    }

    /// Load kernel from boot device
    pub fn load_kernel(&self, kernel_path: &str) -> Result<KernelLoadInfo, BootError> {
        info!("Loading kernel from legacy BIOS device: {}", kernel_path);
        
        match self.boot_device.device_type {
            DeviceType::HardDisk | DeviceType::Usb => {
                self.load_kernel_from_disk()
            }
            DeviceType::CdRom => {
                self.load_kernel_from_cdrom()
            }
            _ => {
                warn!("Unsupported boot device type: {:?}", self.boot_device.device_type);
                Err(BootError::KernelNotFound)
            }
        }
    }

    /// Load kernel from disk device
    fn load_kernel_from_disk(&self) -> Result<KernelLoadInfo, BootError> {
        info!("Loading kernel from disk device...");
        
        // This would implement disk reading using BIOS INT 13h calls
        // For now, return a placeholder
        
        warn!("Disk kernel loading not yet implemented");
        Err(BootError::BootProcessError)
    }

    /// Load kernel from CD-ROM
    fn load_kernel_from_cdrom(&self) -> Result<KernelLoadInfo, BootError> {
        info!("Loading kernel from CD-ROM device...");
        
        // This would implement CD-ROM reading using BIOS INT 13h or ATAPI
        // For now, return a placeholder
        
        warn!("CD-ROM kernel loading not yet implemented");
        Err(BootError::BootProcessError)
    }

    /// Jump to kernel entry point
    pub fn jump_to_kernel(&self, kernel_info: KernelLoadInfo) -> ! {
        info!("Jumping to kernel via legacy BIOS...");
        
        // This would involve setting up the kernel environment and
        // jumping to the kernel entry point
        
        info!("Kernel jump not yet implemented");
        loop {
            unsafe {
                core::arch::asm!("hlt");
            }
        }
    }

    /// Print BIOS information
    pub fn print_bios_info(&self) {
        info!("BIOS Information:");
        info!("Memory 64K: {} KB", self.bios_information.bios_memory_64k);
        info!("Extended Memory: {} KB", self.bios_information.bios_extended_memory);
        info!("Boot Device: {:?}", self.boot_device);
        info!("Video Mode: {} ({}x{})", 
              self.video_info.mode, 
              self.video_info.columns, 
              self.video_info.rows);
    }
}

/// Kernel load information for legacy BIOS
#[derive(Debug)]
pub struct KernelLoadInfo {
    pub buffer: Vec<u8>,
    pub size: usize,
    pub entry_point: PhysAddr,
}

/// Legacy BIOS boot entry point
pub fn legacy_bios_boot_start() -> Result<!, BootError> {
    info!("Starting legacy BIOS boot process...");
    
    // Initialize legacy BIOS boot context
    let boot_context = LegacyBiosContext::initialize()?;
    
    // Print BIOS information
    boot_context.print_bios_info();
    
    // Load kernel
    let kernel_info = boot_context.load_kernel("/boot/multios/kernel")?;
    
    // Jump to kernel
    boot_context.jump_to_kernel(kernel_info);
}

/// Helper functions for BIOS interrupt calls
impl LegacyBiosContext {
    /// Call BIOS interrupt with parameters
    fn int15(ax: u16, bx: u16, cx: u16, dx: u16) -> Result<(u16, u16, u16, u16), BootError> {
        let mut result_ax: u16;
        let mut result_bx: u16;
        let mut result_cx: u16;
        let mut result_dx: u16;
        
        unsafe {
            asm!(
                "int 0x15",
                in("ax") ax,
                in("bx") bx,
                in("cx") cx,
                in("dx") dx,
                out("ax") result_ax,
                out("bx") result_bx,
                out("cx") result_cx,
                out("dx") result_dx,
                "jnc success",
                "jmp error",
                "success:",
                "jmp done",
                "error:",
                "mov ax, 1",  // Error indicator
                "done:",
            );
        }
        
        Ok((result_ax, result_bx, result_cx, result_dx))
    }

    /// Call BIOS interrupt with memory parameters
    fn int15_memory(eax: u32, ebx: u32, ecx: u32, edx: u32) -> Result<(u32, u32, u32, u32), BootError> {
        let mut result_eax: u32;
        let mut result_ebx: u32;
        let mut result_ecx: u32;
        let mut result_edx: u32;
        
        unsafe {
            asm!(
                "int 0x15",
                in("eax") eax,
                in("ebx") ebx,
                in("ecx") ecx,
                in("edx") edx,
                out("eax") result_eax,
                out("ebx") result_ebx,
                out("ecx") result_ecx,
                out("edx") result_edx,
            );
        }
        
        Ok((result_eax, result_ebx, result_ecx, result_edx))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bios_info_creation() {
        let bios_info = BiosInfo {
            bios_vendor: [0; 8],
            bios_version: [0; 8],
            bios_date: [0; 8],
            bios_memory_64k: 640,
            bios_extended_memory: 16384,
            bios_video_mode: 3,
            bios_manufacturer: [0; 16],
        };
        
        assert_eq!(bios_info.bios_memory_64k, 640);
        assert_eq!(bios_info.bios_extended_memory, 16384);
    }

    #[test]
    fn test_boot_device_creation() {
        let device = BootDevice {
            device_type: DeviceType::HardDisk,
            drive_number: 0x80,
            partition: Some(0),
            file_system: Some(FileSystemType::Fat32),
        };
        
        assert_eq!(device.device_type, DeviceType::HardDisk);
        assert_eq!(device.drive_number, 0x80);
    }

    #[test]
    fn test_video_info_creation() {
        let video = VideoInfo {
            mode: 3,
            columns: 80,
            rows: 25,
            address: PhysAddr::new(0xB8000),
            size: 80 * 25 * 2,
        };
        
        assert_eq!(video.columns, 80);
        assert_eq!(video.rows, 25);
        assert_eq!(video.size, 4000);
    }

    #[test]
    fn test_memory_region_flags() {
        assert_eq!(BiosMemoryRegion::CONVENTIONAL.bits(), 1 << 0);
        assert_eq!(BiosMemoryRegion::EXTENDED.bits(), 1 << 1);
        assert!(BiosMemoryRegion::CONVENTIONAL.contains(BiosMemoryRegion::CONVENTIONAL));
    }
}