//! Kernel Loading and Boot Information
//! 
//! This module provides comprehensive kernel loading functionality for both
//! UEFI and legacy BIOS boot processes, including boot information structures
//! and kernel entry point management.

use x86_64::PhysAddr;
use x86_64::structures::paging::{Page, Size4KiB};
use bitflags::bitflags;
use log::{info, warn, error, debug};
use bootloader::boot_info::{MemoryRegion, MemoryRegionKind, FramebufferInfo, ModuleInfo};
use uefi::table::Boot as UefiBoot;

use crate::BootError;
use crate::memory_map::{MemoryMap, MemoryRegionInfo, MemoryType};
use crate::uefi::FramebufferFormat;
use crate::legacy::VideoInfo;

/// Kernel boot information structure
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct KernelBootInfo {
    pub magic: u64,
    pub version: u32,
    pub kernel_entry: PhysAddr,
    pub kernel_size: usize,
    pub kernel_load_addr: PhysAddr,
    pub memory_map: PhysAddr,
    pub memory_map_size: usize,
    pub memory_map_entry_size: usize,
    pub framebuffer: Option<KFramebufferInfo>,
    pub rsdp: PhysAddr,
    pub cmdline: PhysAddr,
    pub cmdline_size: usize,
    pub bootloader_name: [u8; 64],
    pub bootloader_version: [u8; 64],
    pub acpi_rsdp: PhysAddr,
    pub smbios_tables: PhysAddr,
    pub module_list: PhysAddr,
    pub module_count: usize,
    pub tsc_frequency: u64,
    pub boot_time: u64,
}

/// Kernel framebuffer information
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct KFramebufferInfo {
    pub addr: PhysAddr,
    pub size: usize,
    pub width: u32,
    pub height: u32,
    pub pitch: u32,
    pub format: u32,
    pub red_mask_size: u8,
    pub red_mask_shift: u8,
    pub green_mask_size: u8,
    pub green_mask_shift: u8,
    pub blue_mask_size: u8,
    pub blue_mask_shift: u8,
    pub reserved_mask_size: u8,
    pub reserved_mask_shift: u8,
}

/// Kernel memory map entry
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct KMemoryMapEntry {
    pub base_addr: PhysAddr,
    pub size: usize,
    pub mem_type: u32,
    pub flags: u32,
}

/// Loaded kernel module information
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct KModuleInfo {
    pub start: PhysAddr,
    pub end: PhysAddr,
    pub name: [u8; 256],
    pub name_len: usize,
}

bitflags! {
    /// Kernel boot flags
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct BootFlags: u64 {
        const UEFI_BOOT = 1 << 0;
        const LEGACY_BOOT = 1 << 1;
        const MEMORY_TESTED = 1 << 2;
        const SERIAL_CONSOLE = 1 << 3;
        const FRAMEBUFFER_AVAILABLE = 1 << 4;
        const ACPI_AVAILABLE = 1 << 5;
        const SECURE_BOOT = 1 << 6;
        const DEBUG_MODE = 1 << 7;
    }
}

/// Kernel boot configuration
#[derive(Debug, Clone)]
pub struct KernelBootConfig {
    pub kernel_path: &'static str,
    pub command_line: Option<&'static str>,
    pub initrd_path: Option<&'static str>,
    pub enable_logging: bool,
    pub enable_debug: bool,
    pub test_memory: bool,
    pub boot_flags: BootFlags,
}

/// Boot information buffer
#[derive(Debug)]
pub struct BootInfoBuffer {
    pub buffer: Vec<u8>,
    pub boot_info: KernelBootInfo,
    pub memory_map: Vec<KMemoryMapEntry>,
    pub modules: Vec<KModuleInfo>,
}

impl BootInfoBuffer {
    /// Create a new boot info buffer
    pub fn new(size: usize) -> Self {
        let mut buffer = vec![0u8; size];
        let boot_info_ptr = buffer.as_mut_ptr() as *mut KernelBootInfo;
        
        // Initialize boot info header
        unsafe {
            *boot_info_ptr = KernelBootInfo {
                magic: 0x2022_4D55_4B4E_494F, // "MINIKERNEL" magic
                version: 1,
                kernel_entry: PhysAddr::new(0),
                kernel_size: 0,
                kernel_load_addr: PhysAddr::new(0),
                memory_map: PhysAddr::new(0),
                memory_map_size: 0,
                memory_map_entry_size: core::mem::size_of::<KMemoryMapEntry>(),
                framebuffer: None,
                rsdp: PhysAddr::new(0),
                cmdline: PhysAddr::new(0),
                cmdline_size: 0,
                bootloader_name: *b"MultiOS Bootloader v0.1.0                          \0",
                bootloader_version: *b"0.1.0                                             \0",
                acpi_rsdp: PhysAddr::new(0),
                smbios_tables: PhysAddr::new(0),
                module_list: PhysAddr::new(0),
                module_count: 0,
                tsc_frequency: 0,
                boot_time: 0,
            };
        }
        
        Self {
            buffer,
            boot_info: unsafe { *boot_info_ptr },
            memory_map: Vec::new(),
            modules: Vec::new(),
        }
    }

    /// Set kernel information
    pub fn set_kernel_info(&mut self, entry: PhysAddr, size: usize, load_addr: PhysAddr) {
        self.boot_info.kernel_entry = entry;
        self.boot_info.kernel_size = size;
        self.boot_info.kernel_load_addr = load_addr;
        debug!("Set kernel info: entry={:?}, size={}, load_addr={:?}", 
               entry, size, load_addr);
    }

    /// Set framebuffer information
    pub fn set_framebuffer(&mut self, fb_info: &FramebufferInfo) {
        self.boot_info.framebuffer = Some(KFramebufferInfo {
            addr: PhysAddr::new(fb_info.addr.as_u64()),
            size: fb_info.size,
            width: fb_info.width,
            height: fb_info.height,
            pitch: fb_info.pitch,
            format: fb_info.format.bits(),
            red_mask_size: 8,
            red_mask_shift: 16,
            green_mask_size: 8,
            green_mask_shift: 8,
            blue_mask_size: 8,
            blue_mask_shift: 0,
            reserved_mask_size: 8,
            reserved_mask_shift: 24,
        });
        debug!("Set framebuffer info: {}x{} format={:?}", 
               fb_info.width, fb_info.height, fb_info.format);
    }

    /// Set memory map
    pub fn set_memory_map(&mut self, memory_map: &MemoryMap) {
        self.memory_map.clear();
        
        for region in memory_map.regions() {
            let entry = KMemoryMapEntry {
                base_addr: region.start,
                size: region.size,
                mem_type: region.mem_type as u32,
                flags: region.flags.bits(),
            };
            self.memory_map.push(entry);
        }
        
        // Update boot info pointers
        let map_start = self.buffer.as_mut_ptr().add(self.buffer.len() - self.memory_map.len() * core::mem::size_of::<KMemoryMapEntry>());
        self.boot_info.memory_map = PhysAddr::new(map_start as u64);
        self.boot_info.memory_map_size = self.memory_map.len() * core::mem::size_of::<KMemoryMapEntry>();
        
        debug!("Set memory map: {} entries", self.memory_map.len());
    }

    /// Set command line
    pub fn set_command_line(&mut self, cmdline: Option<&'static str>) {
        if let Some(cmd) = cmdline {
            let cmd_bytes = cmd.as_bytes();
            let cmd_len = cmd_bytes.len().min(2048);
            
            // Copy command line to end of buffer
            let cmd_start = self.buffer.as_mut_ptr().add(self.buffer.len() - cmd_len - 1);
            unsafe {
                core::ptr::copy_nonoverlapping(cmd_bytes.as_ptr(), cmd_start, cmd_len);
                *cmd_start.add(cmd_len) = 0; // Null terminator
            }
            
            self.boot_info.cmdline = PhysAddr::new(cmd_start as u64);
            self.boot_info.cmdline_size = cmd_len;
            
            debug!("Set command line: {}", cmd);
        }
    }

    /// Add module information
    pub fn add_module(&mut self, start: PhysAddr, end: PhysAddr, name: &str) {
        let mut module = KModuleInfo {
            start,
            end,
            name: [0; 256],
            name_len: 0,
        };
        
        let name_bytes = name.as_bytes();
        let name_len = name_bytes.len().min(255);
        module.name[..name_len].copy_from_slice(&name_bytes[..name_len]);
        module.name_len = name_len;
        
        self.modules.push(module);
        self.boot_info.module_count = self.modules.len();
        
        debug!("Added module: {} at {:?}-{:?}", name, start, end);
    }

    /// Finalize boot info buffer
    pub fn finalize(&mut self) -> PhysAddr {
        // Copy memory map entries to buffer
        let map_size = self.memory_map.len() * core::mem::size_of::<KMemoryMapEntry>();
        let map_start = self.buffer.as_mut_ptr().add(self.buffer.len() - map_size);
        unsafe {
            core::ptr::copy_nonoverlapping(
                self.memory_map.as_ptr(),
                map_start as *mut KMemoryMapEntry,
                self.memory_map.len()
            );
        }
        
        // Update boot info pointer
        self.boot_info.memory_map = PhysAddr::new(map_start as u64);
        
        debug!("Boot info buffer finalized");
        PhysAddr::new(self.buffer.as_ptr() as u64)
    }
}

/// Create kernel boot information
pub fn create_kernel_boot_info(
    kernel_path: &str,
    command_line: Option<&'static str>,
) -> KernelBootConfig {
    info!("Creating kernel boot information for: {}", kernel_path);
    
    let boot_flags = BootFlags::empty();
    
    // Set flags based on current boot mode
    #[cfg(target_arch = "x86_64")]
    {
        // Determine boot mode and set appropriate flags
        if cfg!(feature = "uefi") {
            boot_flags.insert(BootFlags::UEFI_BOOT);
        } else {
            boot_flags.insert(BootFlags::LEGACY_BOOT);
        }
    }
    
    if cfg!(feature = "debug_mode") {
        boot_flags.insert(BootFlags::DEBUG_MODE);
    }
    
    if cfg!(feature = "logging") {
        boot_flags.insert(BootFlags::SERIAL_CONSOLE);
    }
    
    let config = KernelBootConfig {
        kernel_path,
        command_line,
        initrd_path: None,
        enable_logging: cfg!(feature = "logging"),
        enable_debug: cfg!(feature = "debug_mode"),
        test_memory: cfg!(feature = "memory_test"),
        boot_flags,
    };
    
    info!("Kernel boot configuration: {:?}", config);
    config
}

/// Validate kernel boot information
pub fn validate_boot_info(boot_info: &KernelBootInfo) -> bool {
    let mut valid = true;
    
    // Check magic number
    if boot_info.magic != 0x2022_4D55_4B4E_494F {
        error!("Invalid boot info magic: {:#x}", boot_info.magic);
        valid = false;
    }
    
    // Check version
    if boot_info.version != 1 {
        error!("Unsupported boot info version: {}", boot_info.version);
        valid = false;
    }
    
    // Check kernel entry point
    if boot_info.kernel_entry.as_u64() == 0 {
        error!("Invalid kernel entry point: {:#x}", boot_info.kernel_entry.as_u64());
        valid = false;
    }
    
    // Check memory map
    if boot_info.memory_map_size == 0 {
        warn!("No memory map provided");
    }
    
    // Check command line
    if boot_info.cmdline_size > 0 && boot_info.cmdline.as_u64() == 0 {
        error!("Invalid command line pointer");
        valid = false;
    }
    
    info!("Boot info validation: {}", if valid { "PASSED" } else { "FAILED" });
    valid
}

/// Enter kernel with boot information
pub fn enter_kernel(boot_info: KernelBootConfig) -> ! {
    info!("Entering kernel with boot information...");
    
    // Validate boot info
    // Note: This would validate the actual boot info buffer in real implementation
    
    // Set up registers for kernel entry
    // The exact register state depends on the kernel's entry point expectations
    
    info!("Transitioning to kernel at entry point...");
    
    // This is a simplified transition - real implementation would:
    // 1. Set up kernel stack
    // 2. Load kernel page tables
    // 3. Jump to kernel entry point with boot info pointer
    
    loop {
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}

/// Load kernel from memory buffer
pub fn load_kernel_from_buffer(kernel_buffer: &[u8], load_addr: PhysAddr) -> Result<KernelLoadInfo, BootError> {
    info!("Loading kernel from memory buffer: {} bytes", kernel_buffer.len());
    
    // Validate kernel format
    validate_elf_format(kernel_buffer)?;
    
    // Get entry point from ELF header
    let entry_point = extract_elf_entry_point(kernel_buffer)?;
    
    // Copy kernel to load address
    // In real implementation, this would involve:
    // 1. Allocating physical memory
    // 2. Setting up page tables
    // 3. Copying kernel sections to their proper locations
    
    debug!("Kernel loaded: entry={:?}, size={}, load_addr={:?}", 
           entry_point, kernel_buffer.len(), load_addr);
    
    Ok(KernelLoadInfo {
        buffer: kernel_buffer.to_vec(),
        size: kernel_buffer.len(),
        entry_point,
        load_addr,
    })
}

/// Validate ELF kernel format
fn validate_elf_format(kernel_data: &[u8]) -> Result<(), BootError> {
    if kernel_data.len() < 64 {
        return Err(BootError::InvalidKernelFormat);
    }
    
    // Check ELF magic number
    if kernel_data[0..4] != [0x7F, b'E', b'L', b'F'] {
        return Err(BootError::InvalidKernelFormat);
    }
    
    // Check ELF class (64-bit)
    if kernel_data[4] != 2 {
        return Err(BootError::InvalidKernelFormat);
    }
    
    // Check ELF data (little-endian)
    if kernel_data[5] != 1 {
        return Err(BootError::InvalidKernelFormat);
    }
    
    // Check ELF type (executable)
    let elf_type = u16::from_le_bytes(kernel_data[16..18].try_into().unwrap());
    if elf_type != 2 {
        return Err(BootError::InvalidKernelFormat);
    }
    
    info!("Kernel ELF format validation passed");
    Ok(())
}

/// Extract entry point from ELF header
fn extract_elf_entry_point(kernel_data: &[u8]) -> Result<PhysAddr, BootError> {
    // ELF entry point is at offset 0x18 in the ELF header
    let entry_bytes = &kernel_data[0x18..0x20];
    let entry_addr = u64::from_le_bytes(entry_bytes.try_into().unwrap());
    
    if entry_addr == 0 {
        return Err(BootError::InvalidKernelFormat);
    }
    
    Ok(PhysAddr::new(entry_addr))
}

/// Kernel load information
#[derive(Debug)]
pub struct KernelLoadInfo {
    pub buffer: Vec<u8>,
    pub size: usize,
    pub entry_point: PhysAddr,
    pub load_addr: PhysAddr,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kernel_boot_info_creation() {
        let boot_info = KernelBootInfo {
            magic: 0x2022_4D55_4B4E_494F,
            version: 1,
            kernel_entry: PhysAddr::new(0x100000),
            kernel_size: 1024 * 1024,
            kernel_load_addr: PhysAddr::new(0x100000),
            memory_map: PhysAddr::new(0),
            memory_map_size: 0,
            memory_map_entry_size: 32,
            framebuffer: None,
            rsdp: PhysAddr::new(0),
            cmdline: PhysAddr::new(0),
            cmdline_size: 0,
            bootloader_name: [0; 64],
            bootloader_version: [0; 64],
            acpi_rsdp: PhysAddr::new(0),
            smbios_tables: PhysAddr::new(0),
            module_list: PhysAddr::new(0),
            module_count: 0,
            tsc_frequency: 0,
            boot_time: 0,
        };
        
        assert_eq!(boot_info.magic, 0x2022_4D55_4B4E_494F);
        assert_eq!(boot_info.version, 1);
        assert_eq!(boot_info.kernel_entry.as_u64(), 0x100000);
    }

    #[test]
    fn test_kframebuffer_info_creation() {
        let fb_info = KFramebufferInfo {
            addr: PhysAddr::new(0x80000000),
            size: 1920 * 1080 * 4,
            width: 1920,
            height: 1080,
            pitch: 1920 * 4,
            format: 0,
            red_mask_size: 8,
            red_mask_shift: 16,
            green_mask_size: 8,
            green_mask_shift: 8,
            blue_mask_size: 8,
            blue_mask_shift: 0,
            reserved_mask_size: 8,
            reserved_mask_shift: 24,
        };
        
        assert_eq!(fb_info.size, 1920 * 1080 * 4);
        assert_eq!(fb_info.width, 1920);
        assert_eq!(fb_info.height, 1080);
    }

    #[test]
    fn test_kmemory_map_entry_creation() {
        let entry = KMemoryMapEntry {
            base_addr: PhysAddr::new(0x0),
            size: 640 * 1024,
            mem_type: MemoryType::Usable as u32,
            flags: 0xF,
        };
        
        assert_eq!(entry.size, 640 * 1024);
        assert_eq!(entry.mem_type, MemoryType::Usable as u32);
    }

    #[test]
    fn test_kernel_boot_config_creation() {
        let config = create_kernel_boot_info("/boot/kernel", Some("quiet loglevel=3"));
        assert_eq!(config.kernel_path, "/boot/kernel");
        assert!(config.command_line.is_some());
        assert_eq!(config.command_line.unwrap(), "quiet loglevel=3");
    }

    #[test]
    fn test_elf_validation() {
        let valid_elf = vec![
            0x7F, b'E', b'L', b'F',  // Magic
            2,                       // 64-bit
            1,                       // Little-endian
            1,                       // ELF version 1
            0,                       // System V ABI
            0, 0, 0, 0, 0, 0, 0, 0, // Padding
            2, 0,                    // Executable type
            0x3E, 0,                 // x86-64 architecture
            1, 0, 0, 0,             // ELF version
            0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // Entry point (simplified)
        ];
        
        assert!(validate_elf_format(&valid_elf).is_ok());
    }
}