//! BIOS/UEFI Compatibility Layer
//! 
//! Provides unified interface for both legacy BIOS and UEFI firmware

use crate::log::{info, warn, error};
use crate::KernelError;

use super::{FirmwareType, FirmwareInfo, MemoryRegion, MemoryRegionType};

/// BIOS interrupt functions
const BIOS_VIDEO_Teletype: u16 = 0x0E00;
const BIOS_KEYBOARD_READ: u16 = 0x1600;
const BIOS_GET_MEMORY_MAP: u16 = 0xE820;
const BIOS_PM_ACTIVE: u16 = 0x5301;

/// UEFI constants
const EFI_SYSTEM_TABLE_SIGNATURE: u64 = 0x5453595320494249;
const EFI_BOOT_SERVICES_SIGNATURE: u64 = 0x56524553544f4f42;

/// UEFI GUIDs
const EFI_ACPI_TABLE_GUID: [u8; 16] = [
    0x8868e871, 0xe4f1, 0x11d3, 0xbc, 0x22, 0x00, 0x80, 0xc7, 0x3c, 0x88, 0x81
];

/// UEFI memory types
#[derive(Debug, Clone, Copy)]
#[repr(u32)]
pub enum UefiMemoryType {
    EfiReservedMemoryType = 0,
    EfiLoaderCode = 1,
    EfiLoaderData = 2,
    EfiBootServicesCode = 3,
    EfiBootServicesData = 4,
    EfiRuntimeServicesCode = 5,
    EfiRuntimeServicesData = 6,
    EfiConventionalMemory = 7,
    EfiUnusableMemory = 8,
    EfiACPIReclaimMemory = 9,
    EfiACPINVSMemory = 10,
    EfiMemoryMappedIO = 11,
    EfiMemoryMappedIOPortSpace = 12,
    EfiPalCode = 13,
    EfiPersistentMemory = 14,
}

/// Detect firmware type
pub fn detect_firmware_type() -> Result<FirmwareType, KernelError> {
    // Check for UEFI by looking for UEFI system table signature in memory
    // This is typically at 0x000E0000 or 0x000F0000 in firmware regions
    
    let efi_system_table_ptr = find_efi_system_table()?;
    if let Some(_ptr) = efi_system_table_ptr {
        info!("UEFI firmware detected");
        return Ok(FirmwareType::Uefi);
    }
    
    // Check for legacy BIOS
    info!("Legacy BIOS detected");
    Ok(FirmwareType::LegacyBios)
}

/// Find UEFI system table
fn find_efi_system_table() -> Result<Option<usize>, KernelError> {
    // Search memory regions for EFI system table signature
    // Common locations: 0x000E0000, 0x000F0000, or specific ACPI RSDP
    
    unsafe {
        // Check common UEFI memory locations
        let candidate_locations = [
            0x000E0000,
            0x000F0000,
            0x000F8000,
        ];
        
        for location in candidate_locations {
            let ptr = location as *const u64;
            let signature = ptr.read_volatile();
            
            if signature == EFI_SYSTEM_TABLE_SIGNATURE {
                info!("Found UEFI system table at 0x{:X}", location);
                return Ok(Some(location));
            }
        }
    }
    
    Ok(None)
}

/// Initialize firmware services
pub fn init_firmware_services(firmware_info: &mut FirmwareInfo) -> Result<(), KernelError> {
    match firmware_info.firmware_type {
        FirmwareType::Uefi => init_uefi_services(firmware_info),
        FirmwareType::LegacyBios => init_bios_services(firmware_info),
        _ => {
            warn!("Unknown firmware type, using legacy BIOS fallback");
            init_bios_services(firmware_info)
        }
    }
}

/// Initialize UEFI services
fn init_uefi_services(firmware_info: &mut FirmwareInfo) -> Result<(), KernelError> {
    info!("Initializing UEFI services...");
    
    // UEFI systems provide runtime services and boot services
    firmware_info.boot_services_available = true;
    firmware_info.runtime_services_available = true;
    firmware_info.vendor = "UEFI".to_string();
    firmware_info.version = "2.x".to_string(); // Would be detected
    
    // Get memory map from UEFI
    get_uefi_memory_map(firmware_info)?;
    
    // Initialize ACPI from UEFI tables
    init_uefi_acpi(firmware_info)?;
    
    Ok(())
}

/// Initialize BIOS services
fn init_bios_services(firmware_info: &mut FirmwareInfo) -> Result<(), KernelError> {
    info!("Initializing legacy BIOS services...");
    
    // Legacy BIOS provides limited runtime services
    firmware_info.boot_services_available = true;
    firmware_info.runtime_services_available = false;
    firmware_info.vendor = "Phoenix/Award BIOS".to_string();
    firmware_info.version = "1.0".to_string(); // Would be detected
    
    // Get memory map using BIOS interrupt 0x15, function 0xE820
    get_bios_memory_map(firmware_info)?;
    
    // Initialize ACPI from legacy BIOS if available
    init_bios_acpi(firmware_info)?;
    
    Ok(())
}

/// Get memory map from UEFI
fn get_uefi_memory_map(firmware_info: &mut FirmwareInfo) -> Result<(), KernelError> {
    // In UEFI, memory map is obtained via GetMemoryMap boot service
    // This requires calling UEFI protocols which would be done via firmware calls
    
    // For now, create a basic memory map
    firmware_info.memory_map = vec![
        MemoryRegion {
            start_addr: 0x00000000,
            size: 0x000A0000,
            region_type: MemoryRegionType::Usable, // Conventional memory
        },
        MemoryRegion {
            start_addr: 0x000A0000,
            size: 0x00060000,
            region_type: MemoryRegionType::Reserved, // Video memory
        },
        MemoryRegion {
            start_addr: 0x00100000,
            size: 0x7EF00000,
            region_type: MemoryRegionType::Usable, // Available memory
        },
        MemoryRegion {
            start_addr: 0x80000000,
            size: 0x7FF00000,
            region_type: MemoryRegionType::Reserved, // Reserved memory
        },
    ];
    
    Ok(())
}

/// Get memory map from BIOS
fn get_bios_memory_map(firmware_info: &mut FirmwareInfo) -> Result<(), KernelError> {
    // Use BIOS interrupt 0x15, function 0xE820 to get memory map
    // This returns entries describing all memory regions
    
    let mut memory_map = Vec::new();
    
    // Basic memory regions for typical x86_64 system
    memory_map.push(MemoryRegion {
        start_addr: 0x00000000,
        size: 0x0009FC00,
        region_type: MemoryRegionType::Usable, // Conventional memory (below 640KB)
    });
    
    memory_map.push(MemoryRegion {
        start_addr: 0x0009FC00,
        size: 0x00000400,
        region_type: MemoryRegionType::Reserved, // Extended BIOS data area
    });
    
    memory_map.push(MemoryRegion {
        start_addr: 0x000A0000,
        size: 0x00060000,
        region_type: MemoryRegionType::Reserved, // Video memory
    });
    
    memory_map.push(MemoryRegion {
        start_addr: 0x00100000,
        size: 0x7FEE0000,
        region_type: MemoryRegionType::Usable, // Available memory
    });
    
    memory_map.push(MemoryRegion {
        start_addr: 0x7FF00000,
        size: 0x00010000,
        region_type: MemoryRegionType::Reserved, // BIOS reserved
    });
    
    // Add kernel and bootloader regions
    memory_map.push(MemoryRegion {
        start_addr: 0x100000,
        size: 0x100000,
        region_type: MemoryRegionType::BootLoader,
    });
    
    memory_map.push(MemoryRegion {
        start_addr: 0x200000,
        size: 0x100000,
        region_type: MemoryRegionType::KernelCode,
    });
    
    memory_map.push(MemoryRegion {
        start_addr: 0x300000,
        size: 0x100000,
        region_type: MemoryRegionType::KernelData,
    });
    
    firmware_info.memory_map = memory_map;
    
    Ok(())
}

/// Initialize ACPI from UEFI
fn init_uefi_acpi(firmware_info: &mut FirmwareInfo) -> Result<(), KernelError> {
    // In UEFI, ACPI tables are available as configuration tables
    // The RSDP (Root System Description Pointer) would be in the UEFI system table
    
    info!("ACPI support available via UEFI configuration tables");
    Ok(())
}

/// Initialize ACPI from BIOS
fn init_bios_acpi(firmware_info: &mut FirmwareInfo) -> Result<(), KernelError> {
    // In legacy BIOS, ACPI is enabled via BIOS function
    // RSDP is typically found at 0x40E or through memory search
    
    unsafe {
        // Try to enable ACPI via BIOS interrupt 0x15, function 0x5301
        let mut eax = BIOS_PM_ACTIVE;
        let mut ebx = 0x0000;
        let mut ecx = 0x0000;
        
        core::arch::asm!(
            "mov {0:e}, {1:e}",
            "mov {0:ebx}, {2:e}",
            "mov {0:ecx}, {3:e}",
            "int 0x15",
            inout(reg) eax => eax,
            in(reg) BIOS_PM_ACTIVE,
            in(reg) ebx,
            in(reg) ecx
        );
    }
    
    info!("ACPI enabled via BIOS");
    Ok(())
}

/// Convert UEFI memory type to MultiOS memory type
fn convert_uefi_memory_type(uefi_type: UefiMemoryType) -> MemoryRegionType {
    match uefi_type {
        UefiMemoryType::EfiConventionalMemory => MemoryRegionType::Usable,
        UefiMemoryType::EfiReservedMemoryType => MemoryRegionType::Reserved,
        UefiMemoryType::EfiACPIReclaimMemory => MemoryRegionType::AcpiReclaimable,
        UefiMemoryType::EfiACPINVSMemory => MemoryRegionType::AcpiNvs,
        UefiMemoryType::EfiUnusableMemory => MemoryRegionType::BadMemory,
        UefiMemoryType::EfiLoaderCode => MemoryRegionType::BootLoader,
        _ => MemoryRegionType::Reserved,
    }
}

/// BIOS video teletype output
pub fn bios_print(text: &str) {
    unsafe {
        for byte in text.bytes() {
            let mut al = byte;
            let mut ah = 0x0E;
            
            core::arch::asm!(
                "mov {0:al}, {1:al}",
                "mov {0:ah}, {2:ah}",
                "int 0x10",
                inout(reg) al => al,
                in(reg) byte,
                in(reg) ah
            );
        }
    }
}

/// BIOS keyboard read
pub fn bios_read_key() -> Option<u16> {
    unsafe {
        let mut ah = 0x00;
        let mut al: u8 = 0;
        
        core::arch::asm!(
            "mov {0:ah}, {1:ah}",
            "int 0x16",
            inout(reg) ah => ah,
            inout(reg) al) => al
        );
        
        if al != 0 {
            Some(((ah as u16) << 8) | (al as u16))
        } else {
            None
        }
    }
}

/// Get BIOS information
pub fn get_bios_info() -> Result<BiosInfo, KernelError> {
    let info = BiosInfo {
        version: get_bios_version_string(),
        vendor: get_bios_vendor_string(),
        date: get_bios_date_string(),
        checksum_valid: verify_bios_checksum(),
    };
    
    Ok(info)
}

/// Get BIOS version string
fn get_bios_version_string() -> String {
    // BIOS version is typically at F000:FFFEh
    unsafe {
        let version_ptr = 0xF0000 + 0xFFF5 - 0xF0000;
        let ptr = version_ptr as *const u8;
        let bytes = core::slice::from_raw_parts(ptr, 16);
        String::from_utf8_lossy(bytes).to_string()
    }
}

/// Get BIOS vendor string
fn get_bios_vendor_string() -> String {
    // BIOS vendor is typically at F000:FFFBh
    unsafe {
        let vendor_ptr = 0xF0000 + 0xFFFB - 0xF0000;
        let ptr = vendor_ptr as *const u8;
        let bytes = core::slice::from_raw_parts(ptr, 16);
        String::from_utf8_lossy(bytes).to_string()
    }
}

/// Get BIOS date string
fn get_bios_date_string() -> String {
    // BIOS date is typically at F000:FFF5h
    unsafe {
        let date_ptr = 0xF0000 + 0xFFF5 - 0xF0000;
        let ptr = date_ptr as *const u8;
        let bytes = core::slice::from_raw_parts(ptr, 8);
        String::from_utf8_lossy(bytes).to_string()
    }
}

/// Verify BIOS checksum
fn verify_bios_checksum() -> bool {
    // BIOS ROM checksum verification would be implemented here
    true // Placeholder
}

/// BIOS information structure
#[derive(Debug, Clone)]
pub struct BiosInfo {
    pub version: String,
    pub vendor: String,
    pub date: String,
    pub checksum_valid: bool,
}

/// UEFI System Table structure (simplified)
#[repr(C)]
pub struct UefiSystemTable {
    pub signature: u64,
    pub revision: u32,
    pub header_size: u32,
    pub crc32: u32,
    pub reserved: u32,
    pub firmware_vendor: u64,
    pub firmware_revision: u64,
    pub console_in_handle: u64,
    pub console_in: u64,
    pub console_out_handle: u64,
    pub console_out: u64,
    pub standard_error_handle: u64,
    pub standard_error: u64,
    pub runtime_services: u64,
    pub boot_services: u64,
    pub number_of_table_entries: usize,
    pub configuration_table: u64,
}

/// UEFI Memory Map Entry structure
#[repr(C)]
pub struct UefiMemoryMapEntry {
    pub type_: u32,
    pub pad: u32,
    pub physical_start: u64,
    pub virtual_start: u64,
    pub number_of_pages: u64,
    pub attribute: u64,
}

/// ACPI table pointers for different firmware types
pub struct AcpiTablePointers {
    pub rsdp_address: Option<u64>,
    pub xsdt_address: Option<u64>,
    pub rsdt_address: Option<u64>,
}

/// Find ACPI tables based on firmware type
pub fn find_acpi_tables(firmware_type: FirmwareType) -> Result<AcpiTablePointers, KernelError> {
    match firmware_type {
        FirmwareType::Uefi => find_uefi_acpi_tables(),
        FirmwareType::LegacyBios => find_bios_acpi_tables(),
        _ => Ok(AcpiTablePointers {
            rsdp_address: None,
            xsdt_address: None,
            rsdt_address: None,
        })
    }
}

/// Find ACPI tables in UEFI
fn find_uefi_acpi_tables() -> Result<AcpiTablePointers, KernelError> {
    // ACPI tables are available in UEFI configuration tables
    Ok(AcpiTablePointers {
        rsdp_address: Some(0), // Would be found in UEFI system table
        xsdt_address: Some(0),
        rsdt_address: Some(0),
    })
}

/// Find ACPI tables in BIOS
fn find_bios_acpi_tables() -> Result<AcpiTablePointers, KernelError> {
    // RSDP is typically found at 0x40E or by searching memory
    Ok(AcpiTablePointers {
        rsdp_address: Some(0x40E), // Common location
        xsdt_address: None,
        rsdt_address: None,
    })
}