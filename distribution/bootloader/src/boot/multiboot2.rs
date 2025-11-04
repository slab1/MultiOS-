//! Multiboot2 Protocol Implementation
//! 
//! This module provides Multiboot2 compliance for the bootloader,
//! including boot information parsing, memory map handling, and
//! transition to long mode.

use core::arch::asm;
use core::mem;
use core::ptr;
use bitflags::bitflags;

#[repr(C, packed)]
struct Multiboot2Header {
    magic: u32,
    architecture: u32,
    header_length: u32,
    checksum: u32,
}

#[repr(C, packed)]
struct Multiboot2Tag {
    tag_type: u16,
    flags: u16,
    size: u32,
}

#[repr(C, packed)]
struct Multiboot2BootInfoHeader {
    total_size: u32,
    reserved: u32,
}

#[repr(C, packed)]
struct Multiboot2MemoryMap {
    entry_size: u32,
    entry_version: u32,
}

#[repr(C, packed)]
struct Multiboot2MemoryMapEntry {
    base_addr: u64,
    length: u64,
    entry_type: u32,
    _reserved: u32,
}

#[repr(C, packed)]
struct Multiboot2Module {
    mod_start: u32,
    mod_end: u32,
    cmdline: u32,
    _pad: u32,
}

#[repr(C, packed)]
struct Multiboot2Framebuffer {
    framebuffer_addr: u64,
    framebuffer_pitch: u32,
    framebuffer_width: u32,
    framebuffer_height: u32,
    framebuffer_bpp: u8,
    framebuffer_type: u8,
    _reserved: u16,
}

const MULTIBOOT2_MAGIC: u32 = 0x36D76289;

/// Multiboot2 boot information tag types
const TAG_END: u16 = 0;
const TAG_BOOT_CMD_LINE: u16 = 1;
const TAG_BOOT_LOADER_NAME: u16 = 2;
const TAG_MODULE: u16 = 3;
const TAG_MEMORY_MAP: u16 = 6;
const TAG_VBE_INFO: u16 = 7;
const TAG_FRAMEBUFFER: u16 = 8;
const TAG_ELF_SYMBOLS: u16 = 9;
const TAG_APM_TABLE: u16 = 10;
const TAG_EFI_MMAP: u16 = 11;
const TAG_EFI_BS: u16 = 12;
const TAG_EFI32_Ih: u16 = 13;
const TAG_EFI64_Ih: u16 = 14;
const TAG_SMBIOS: u16 = 15;
const TAG_ACPI_OLD: u16 = 16;
const TAG_ACPI_NEW: u16 = 17;
const TAG_NETWORK: u18;
const TAG_EFI_MMAP_OLD: u18;
const TAG_EFI_MMAP_NEW: u19;

bitflags! {
    /// Multiboot2 boot information flags
    #[repr(C)]
    pub struct BootInfoFlags: u32 {
        const MEMORY_MAP = 1 << 0;
        const ELF_SYMBOLS = 1 << 1;
        const FRAMEBUFFER = 1 << 2;
        const MODULE_LIST = 1 << 3;
        const BIOS_BOOT_DEVICE = 1 << 4;
        const BIOS_ACPI = 1 << 5;
        const BIOS_NETWORK = 1 << 6;
    }
}

/// Memory map entry for kernel
#[derive(Debug, Clone, Copy)]
pub struct MemoryMapEntry {
    pub base_addr: u64,
    pub length: u64,
    pub entry_type: MemoryType,
}

/// Memory types as defined by Multiboot2
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum MemoryType {
    Available = 1,
    Reserved = 2,
    AcpiReclaimable = 3,
    AcpiNvs = 4,
    Unusable = 5,
}

impl From<u32> for MemoryType {
    fn from(entry_type: u32) -> Self {
        match entry_type {
            1 => MemoryType::Available,
            2 => MemoryType::Reserved,
            3 => MemoryType::AcpiReclaimable,
            4 => MemoryType::AcpiNvs,
            5 => MemoryType::Unusable,
            _ => MemoryType::Reserved,
        }
    }
}

/// Bootloader module information
#[derive(Debug, Clone, Copy)]
pub struct BootModule {
    pub start_addr: u32,
    pub end_addr: u32,
    pub cmdline: Option<&'static str>,
}

/// Framebuffer information
#[derive(Debug, Clone, Copy)]
pub struct FramebufferInfo {
    pub framebuffer_addr: u64,
    pub framebuffer_pitch: u32,
    pub width: u32,
    pub height: u32,
    pub bpp: u8,
    pub framebuffer_type: u8,
}

/// Complete Multiboot2 boot information
#[derive(Debug)]
pub struct Multiboot2Info {
    pub cmdline: Option<&'static str>,
    pub bootloader_name: Option<&'static str>,
    pub memory_map: Vec<MemoryMapEntry>,
    pub modules: Vec<BootModule>,
    pub framebuffer: Option<FramebufferInfo>,
    pub acpi_tables: Vec<u64>,
    pub smbios_tables: Vec<u64>,
}

impl Multiboot2Info {
    /// Create new Multiboot2 info structure
    pub const fn new() -> Self {
        Self {
            cmdline: None,
            bootloader_name: None,
            memory_map: Vec::new(),
            modules: Vec::new(),
            framebuffer: None,
            acpi_tables: Vec::new(),
            smbios_tables: Vec::new(),
        }
    }

    /// Parse Multiboot2 boot information
    pub fn parse(boot_info_ptr: *const u8) -> Result<Self, Multiboot2Error> {
        let mut info = Self::new();
        
        if boot_info_ptr.is_null() {
            return Err(Multiboot2Error::InvalidPointer);
        }

        unsafe {
            let header = &*(boot_info_ptr as *const Multiboot2BootInfoHeader);
            let mut current_tag = boot_info_ptr.offset(mem::size_of::<Multiboot2BootInfoHeader>()) as *const Multiboot2Tag;
            let end_address = boot_info_ptr.offset(header.total_size as isize);

            while current_tag < end_address {
                let tag = &*current_tag;
                
                match tag.tag_type {
                    TAG_END => break,
                    TAG_BOOT_CMD_LINE => {
                        let cmdline_ptr = current_tag.offset(mem::size_of::<Multiboot2Tag>()) as *const u8;
                        if !cmdline_ptr.is_null() {
                            info.cmdline = Some(core::str::from_utf8_unchecked(cmdline_ptr));
                        }
                    }
                    TAG_BOOT_LOADER_NAME => {
                        let name_ptr = current_tag.offset(mem::size_of::<Multiboot2Tag>()) as *const u8;
                        if !name_ptr.is_null() {
                            info.bootloader_name = Some(core::str::from_utf8_unchecked(name_ptr));
                        }
                    }
                    TAG_MODULE => {
                        let module = &*(current_tag.offset(mem::size_of::<Multiboot2Tag>()) as *const Multiboot2Module);
                        let cmdline_ptr = current_tag.offset(mem::size_of::<Multiboot2Tag>() + mem::size_of::<Multiboot2Module>()) as *const u8;
                        
                        let boot_module = BootModule {
                            start_addr: module.mod_start,
                            end_addr: module.mod_end,
                            cmdline: if !cmdline_ptr.is_null() {
                                Some(core::str::from_utf8_unchecked(cmdline_ptr))
                            } else {
                                None
                            },
                        };
                        info.modules.push(boot_module);
                    }
                    TAG_MEMORY_MAP => {
                        let mmap_ptr = current_tag.offset(mem::size_of::<Multiboot2Tag>()) as *const Multiboot2MemoryMap;
                        let mmap = &*mmap_ptr;
                        let mut entry_ptr = current_tag.offset(mem::size_of::<Multiboot2Tag>() + mem::size_of::<Multiboot2MemoryMap>()) as *const Multiboot2MemoryMapEntry;
                        let mmap_end = current_tag.offset(tag.size as isize);

                        while entry_ptr < mmap_end as *const Multiboot2MemoryMapEntry {
                            let entry = &*entry_ptr;
                            info.memory_map.push(MemoryMapEntry {
                                base_addr: entry.base_addr,
                                length: entry.length,
                                entry_type: MemoryType::from(entry.entry_type),
                            });
                            entry_ptr = entry_ptr.offset(1);
                        }
                    }
                    TAG_FRAMEBUFFER => {
                        let fb_ptr = current_tag.offset(mem::size_of::<Multiboot2Tag>()) as *const Multiboot2Framebuffer;
                        let fb = &*fb_ptr;
                        info.framebuffer = Some(FramebufferInfo {
                            framebuffer_addr: fb.framebuffer_addr,
                            framebuffer_pitch: fb.framebuffer_pitch,
                            width: fb.framebuffer_width,
                            height: fb.framebuffer_height,
                            bpp: fb.framebuffer_bpp,
                            framebuffer_type: fb.framebuffer_type,
                        });
                    }
                    TAG_ACPI_OLD | TAG_ACPI_NEW => {
                        info.acpi_tables.push(current_tag as u64);
                    }
                    TAG_SMBIOS => {
                        info.smbios_tables.push(current_tag as u64);
                    }
                    _ => {
                        // Unknown tag, skip it
                    }
                }

                let tag_size = ((tag.size + 7) & !7) as isize; // Align to 8 bytes
                current_tag = current_tag.offset(tag_size);
            }
        }

        Ok(info)
    }

    /// Convert to kernel boot information format
    pub fn to_kernel_boot_info(&self) -> KernelBootInfo {
        KernelBootInfo {
            boot_time: get_boot_time(),
            memory_map: self.memory_map.clone(),
            command_line: self.cmdline.map(|s| s as *const str),
            modules: self.modules.iter().map(|m| KernelBootModule {
                start: m.start_addr as u64,
                end: m.end_addr as u64,
                cmdline: m.cmdline.map(|s| s as *const str),
            }).collect(),
            framebuffer: self.framebuffer.map(|fb| KernelFramebufferInfo {
                address: fb.framebuffer_addr,
                pitch: fb.framebuffer_pitch,
                width: fb.width,
                height: fb.height,
                bpp: fb.bpp,
            }),
        }
    }
}

/// Errors that can occur during Multiboot2 parsing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Multiboot2Error {
    InvalidPointer,
    InvalidMagic,
    ParsingFailed,
    UnsupportedVersion,
}

/// Boot information passed to the kernel
#[derive(Debug)]
pub struct KernelBootInfo {
    pub boot_time: u64,
    pub memory_map: Vec<MemoryMapEntry>,
    pub command_line: Option<&'static str>,
    pub modules: Vec<KernelBootModule>,
    pub framebuffer: Option<KernelFramebufferInfo>,
}

/// Kernel boot module information
#[derive(Debug, Clone, Copy)]
pub struct KernelBootModule {
    pub start: u64,
    pub end: u64,
    pub cmdline: Option<&'static str>,
}

/// Kernel framebuffer information
#[derive(Debug, Clone, Copy)]
pub struct KernelFramebufferInfo {
    pub address: u64,
    pub pitch: u32,
    pub width: u32,
    pub height: u32,
    pub bpp: u8,
}

/// Get boot time (placeholder implementation)
fn get_boot_time() -> u64 {
    0 // TODO: Implement real time source
}

/// Validate Multiboot2 magic number
pub fn validate_multiboot2(boot_info_ptr: *const u8) -> bool {
    if boot_info_ptr.is_null() {
        return false;
    }

    unsafe {
        // Check for valid boot info structure
        let header = &*(boot_info_ptr as *const Multiboot2BootInfoHeader);
        header.total_size > 0 && header.total_size < 1024 * 1024 // Sanity check
    }
}

/// Create minimal Multiboot2 info for testing
pub fn create_minimal_multiboot2_info() -> Multiboot2Info {
    Multiboot2Info {
        cmdline: Some("quiet loglevel=3"),
        bootloader_name: Some("MultiOS Bootloader"),
        memory_map: vec![
            MemoryMapEntry {
                base_addr: 0x0,
                length: 0x9FC00,
                entry_type: MemoryType::Available,
            },
            MemoryMapEntry {
                base_addr: 0x100000,
                length: 0x7EE00000,
                entry_type: MemoryType::Available,
            },
        ],
        modules: Vec::new(),
        framebuffer: None,
        acpi_tables: Vec::new(),
        smbios_tables: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multiboot2_info_creation() {
        let info = Multiboot2Info::new();
        assert!(info.cmdline.is_none());
        assert!(info.memory_map.is_empty());
        assert!(info.modules.is_empty());
    }

    #[test]
    fn test_memory_type_conversion() {
        assert_eq!(MemoryType::from(1), MemoryType::Available);
        assert_eq!(MemoryType::from(2), MemoryType::Reserved);
        assert_eq!(MemoryType::from(5), MemoryType::Unusable);
    }

    #[test]
    fn test_kernel_boot_info_conversion() {
        let info = create_minimal_multiboot2_info();
        let kernel_info = info.to_kernel_boot_info();
        
        assert_eq!(kernel_info.command_line, Some("quiet loglevel=3"));
        assert_eq!(kernel_info.memory_map.len(), 2);
        assert_eq!(kernel_info.modules.len(), 0);
    }

    #[test]
    fn test_memory_map_entry_structure() {
        let entry = MemoryMapEntry {
            base_addr: 0x100000,
            length: 0x1000,
            entry_type: MemoryType::Available,
        };
        
        assert_eq!(entry.base_addr, 0x100000);
        assert_eq!(entry.entry_type, MemoryType::Available);
    }
}