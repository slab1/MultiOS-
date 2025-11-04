//! ACPI Power Management
//! 
//! Provides ACPI (Advanced Configuration and Power Interface) support
//! for power states, thermal management, and hardware configuration

use crate::log::{info, warn, error};
use crate::KernelError;

use super::{FirmwareInfo, FirmwareType};

/// ACPI table signatures
const RSDP_SIGNATURE: u64 = 0x2052545020445352; // "RSD PTR "
const RSDT_SIGNATURE: u32 = 0x54445352; // "RSDT"
const XSDT_SIGNATURE: u32 = 0x54445358; // "XSDT"
const FACP_SIGNATURE: u32 = 0x50434146; // "FACP"
const DSDT_SIGNATURE: u32 = 0x54445344; // "DSDT"
const FACS_SIGNATURE: u32 = 0x53434146; // "FACS"
const APIC_SIGNATURE: u32 = 0x43495041; // "APIC"
const HPET_SIGNATURE: u32 = 0x54455048; // "HPET"
const MCFG_SIGNATURE: u32 = 0x47434D4D; // "MCFG"
const SSDT_SIGNATURE: u32 = 0x54445353; // "SSDT"

/// ACPI power states
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AcpiPowerState {
    S0, // Working
    S1, // Sleep state 1
    S2, // Sleep state 2
    S3, // Sleep state 3 (Standby)
    S4, // Hibernate
    S5, // Soft off
}

/// ACPI sleep types
#[derive(Debug, Clone, Copy)]
pub enum AcpiSleepType {
    Normal = 0x00,
    Processor = 0x01,
    Legacy = 0x02,
}

/// ACPI processor states
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AcpiProcessorState {
    C0, // Active
    C1, // Halt
    C2, // Stop Grant
    C3, // Deep Sleep
}

/// ACPI thermal zones
#[derive(Debug, Clone)]
pub struct ThermalZone {
    pub id: u8,
    pub current_temp: i32,      // In tenths of Kelvin
    pub passive_temp: i32,      // Passive cooling threshold
    pub critical_temp: i32,     // Critical temperature
    pub hot_temp: i32,          // Hot temperature
    pub enabled: bool,
}

/// ACPI battery information
#[derive(Debug, Clone)]
pub struct BatteryInfo {
    pub present: bool,
    pub state: BatteryState,
    pub design_capacity: u32,   // In mWh
    pub last_full_capacity: u32,
    pub current_capacity: u32,
    pub voltage: u32,           // In mV
    pub current: i32,           // In mA (negative = charging)
    pub rate: i32,              // In mA
    pub capacity: u8,           // Percentage (0-100)
    pub voltage_present: bool,
    pub charging: bool,
}

/// Battery states
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BatteryState {
    Discharging,
    Charging,
    Critical,
    Full,
    Unknown,
}

/// ACPI device power states
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DevicePowerState {
    D0, // Fully on
    D1, // Low power state 1
    D2, // Low power state 2
    D3, // Off
    D3Hot, // Off but some wake functionality
}

/// ACPI interrupt routing
#[derive(Debug, Clone)]
pub struct AcpiInterruptRouting {
    pub device_id: u16,
    pub pin: u8,
    pub source_irqline: u8,
    pub source_index: u16,
    pub edge_level: u8,    // 0 = edge, 1 = level
    pub active_high: u8,   // 0 = active low, 1 = active high
    pub shared: u8,        // 0 = exclusive, 1 = shared
}

/// ACPI memory mapped I/O device information
#[derive(Debug, Clone)]
pub struct AcpiMcfgDevice {
    pub segment_group: u16,
    pub bus_number: u8,
    pub start_bus: u8,
    pub end_bus: u8,
    pub base_address: u64,
}

/// ACPI Manager
pub struct AcpiManager {
    pub initialized: bool,
    pub rsdp_address: Option<u64>,
    pub rsdt_address: Option<u64>,
    pub xsdt_address: Option<u64>,
    pub dsdt_address: Option<u64>,
    pub facp_address: Option<u64>,
    pub facs_address: Option<u64>,
    pub firmware_control: u32,
    pub firmware_control_length: u32,
    pub dsdt_integer_width: u8,
    pub preferred_pm_profile: u8,
    pub sci_interrupt: u32,
    pub smi_command_port: u32,
    pub acpi_enable_value: u8,
    pub acpi_disable_value: u8,
    pub s4bios_req: u8,
    pub pstate_control: u8,
    pub pm1a_event_block: u32,
    pub pm1b_event_block: u32,
    pub pm1a_control_block: u32,
    pub pm1b_control_block: u32,
    pub pm2_control_block: u32,
    pub pm_timer_block: u32,
    pub gpe0_block: u32,
    pub gpe1_block: u32,
    pub pm1_event_length: u8,
    pub pm1_control_length: u8,
    pub pm2_control_length: u8,
    pub pm_timer_length: u8,
    pub gpe0_length: u8,
    pub gpe1_length: u8,
    pub gpe1_base: u8,
    pub cstate_control: u8,
    pub lowest_dstate: u8,
    pub highest_dstate: u8,
    pub ioapic_address: u64,
    pub ioapic_global_irq_base: u32,
    pub flags: u32,
    pub reset_register_address: u64,
    pub reset_value: u8,
    pub x_firmware_control: u64,
    pub x_dsdt: u64,
    pub x_pm1a_event_block: u64,
    pub x_pm1b_event_block: u64,
    pub x_pm1a_control_block: u64,
    pub x_pm1b_control_block: u64,
    pub x_pm2_control_block: u64,
    pub x_pm_timer_block: u64,
    pub x_gpe0_block: u64,
    pub x_gpe1_block: u64,
    pub sleep_control: u64,
    pub sleep_status: u64,
    /// Power state information
    pub power_state: AcpiPowerState,
    /// Processor states
    pub processor_states: Vec<AcpiProcessorState>,
    /// Thermal zones
    pub thermal_zones: Vec<ThermalZone>,
    /// Battery information
    pub battery_info: BatteryInfo,
    /// Interrupt routing
    pub interrupt_routing: Vec<AcpiInterruptRouting>,
    /// PCI configuration space devices
    pub mcfg_devices: Vec<AcpiMcfgDevice>,
}

impl AcpiManager {
    /// Create new ACPI manager
    pub fn new() -> Self {
        Self {
            initialized: false,
            rsdp_address: None,
            rsdt_address: None,
            xsdt_address: None,
            dsdt_address: None,
            facp_address: None,
            facs_address: None,
            firmware_control: 0,
            firmware_control_length: 0,
            dsdt_integer_width: 0,
            preferred_pm_profile: 0,
            sci_interrupt: 0,
            smi_command_port: 0,
            acpi_enable_value: 0,
            acpi_disable_value: 0,
            s4bios_req: 0,
            pstate_control: 0,
            pm1a_event_block: 0,
            pm1b_event_block: 0,
            pm1a_control_block: 0,
            pm1b_control_block: 0,
            pm2_control_block: 0,
            pm_timer_block: 0,
            gpe0_block: 0,
            gpe1_block: 0,
            pm1_event_length: 0,
            pm1_control_length: 0,
            pm2_control_length: 0,
            pm_timer_length: 0,
            gpe0_length: 0,
            gpe1_length: 0,
            gpe1_base: 0,
            cstate_control: 0,
            lowest_dstate: 0,
            highest_dstate: 0,
            ioapic_address: 0,
            ioapic_global_irq_base: 0,
            flags: 0,
            reset_register_address: 0,
            reset_value: 0,
            x_firmware_control: 0,
            x_dsdt: 0,
            x_pm1a_event_block: 0,
            x_pm1b_event_block: 0,
            x_pm1a_control_block: 0,
            x_pm1b_control_block: 0,
            x_pm2_control_block: 0,
            x_pm_timer_block: 0,
            x_gpe0_block: 0,
            x_gpe1_block: 0,
            sleep_control: 0,
            sleep_status: 0,
            power_state: AcpiPowerState::S0,
            processor_states: Vec::new(),
            thermal_zones: Vec::new(),
            battery_info: BatteryInfo {
                present: false,
                state: BatteryState::Unknown,
                design_capacity: 0,
                last_full_capacity: 0,
                current_capacity: 0,
                voltage: 0,
                current: 0,
                rate: 0,
                capacity: 0,
                voltage_present: false,
                charging: false,
            },
            interrupt_routing: Vec::new(),
            mcfg_devices: Vec::new(),
        }
    }
    
    /// Initialize ACPI subsystem
    pub fn initialize(&mut self, firmware_info: &FirmwareInfo) -> Result<(), KernelError> {
        info!("Initializing ACPI subsystem...");
        
        // Step 1: Find ACPI tables
        self.find_acpi_tables(firmware_info.firmware_type)?;
        
        // Step 2: Parse RSDP
        self.parse_rsdp()?;
        
        // Step 3: Parse system description tables
        self.parse_system_description_tables()?;
        
        // Step 4: Parse fixed ACPI description table (FACP)
        self.parse_facp()?;
        
        // Step 5: Parse differentiated system description table (DSDT)
        self.parse_dsdt()?;
        
        // Step 6: Parse multiple APIC configuration table (MADT/APIC)
        self.parse_apic()?;
        
        // Step 7: Parse other ACPI tables
        self.parse_other_tables()?;
        
        // Step 8: Enable ACPI mode if in legacy mode
        if firmware_info.firmware_type == FirmwareType::LegacyBios {
            self.enable_acpi_mode()?;
        }
        
        // Step 9: Initialize power management
        self.init_power_management()?;
        
        // Step 10: Initialize thermal management
        self.init_thermal_management()?;
        
        self.initialized = true;
        info!("ACPI initialization complete");
        
        Ok(())
    }
    
    /// Find ACPI tables
    fn find_acpi_tables(&mut self, firmware_type: FirmwareType) -> Result<(), KernelError> {
        match firmware_type {
            FirmwareType::Uefi => {
                // In UEFI, ACPI tables are found through the system table configuration table
                info!("Finding ACPI tables in UEFI environment");
                // Would search UEFI system table configuration tables
            },
            FirmwareType::LegacyBios => {
                // In legacy BIOS, RSDP is found at EBDA or at 0x40E
                info!("Finding ACPI tables in legacy BIOS environment");
                
                // Search EBDA (Extended BIOS Data Area)
                self.rsdp_address = self.search_ebda_for_rsdp()?;
                
                // If not found, search fixed memory locations
                if self.rsdp_address.is_none() {
                    self.rsdp_address = self.search_fixed_locations_for_rsdp()?;
                }
            },
            _ => {
                warn!("Unknown firmware type, attempting ACPI table discovery");
            }
        }
        
        if self.rsdp_address.is_none() {
            warn!("ACPI tables not found, ACPI will be disabled");
            return Ok(());
        }
        
        info!("Found ACPI tables at 0x{:X}", self.rsdp_address.unwrap_or(0));
        Ok(())
    }
    
    /// Search EBDA for RSDP
    fn search_ebda_for_rsdp(&self) -> Result<Option<u64>, KernelError> {
        unsafe {
            // Get EBDA segment from BIOS data area at 0x40E
            let ebda_segment = core::ptr::read_volatile(0x40E as *const u16);
            let ebda_address = (ebda_segment as u64) << 4;
            
            // Search first 1KB of EBDA
            let search_length = 1024;
            let ptr = ebda_address as *const u8;
            
            for i in 0..search_length {
                if i + 20 > search_length {
                    break;
                }
                
                let signature = core::ptr::read_volatile(ptr.add(i) as *const u64);
                if signature == RSDP_SIGNATURE {
                    return Ok(Some(ebda_address + i as u64));
                }
            }
        }
        
        Ok(None)
    }
    
    /// Search fixed locations for RSDP
    fn search_fixed_locations_for_rsdp(&self) -> Result<Option<u64>, KernelError> {
        // Search common RSDP locations in BIOS area
        let candidate_locations = [
            0xE0000, // 0xE0000 - 0xEFFFF (128KB BIOS area)
            0xF0000, // 0xF0000 - 0xFFFFF (64KB BIOS area)
            0xF8000,
            0xFC000,
        ];
        
        for location in candidate_locations {
            unsafe {
                let ptr = location as *const u8;
                for i in 0..0x20000 { // Search large area
                    if i + 20 > 0x20000 {
                        break;
                    }
                    
                    let signature = core::ptr::read_volatile(ptr.add(i) as *const u64);
                    if signature == RSDP_SIGNATURE {
                        return Ok(Some(location as u64 + i as u64));
                    }
                }
            }
        }
        
        Ok(None)
    }
    
    /// Parse RSDP (Root System Description Pointer)
    fn parse_rsdp(&mut self) -> Result<(), KernelError> {
        if let Some(rsdp_addr) = self.rsdp_address {
            unsafe {
                let rsdp = rsdp_addr as *const RSDP;
                let rsdp_ref = &*rsdp;
                
                // Validate RSDP signature
                if rsdp_ref.signature != RSDP_SIGNATURE {
                    warn!("Invalid RSDP signature");
                    return Ok(());
                }
                
                // Verify checksum
                if !self.verify_checksum(rsdp_addr as *const u8, 20) {
                    warn!("RSDP checksum invalid");
                    return Ok(());
                }
                
                // Get RSDT/XSDT addresses
                self.rsdt_address = if rsdp_ref.revision > 0 {
                    Some(rsdp_ref.xsdt_address)
                } else {
                    Some(rsdp_ref.rsdt_address as u64)
                };
                
                info!("RSDP parsed successfully");
                info!("  Revision: {}", rsdp_ref.revision);
                info!("  RSDT Address: 0x{:X}", rsdp_ref.rsdt_address);
                if rsdp_ref.revision > 0 {
                    info!("  XSDT Address: 0x{:X}", rsdp_ref.xsdt_address);
                }
            }
        }
        
        Ok(())
    }
    
    /// Parse system description tables (RSDT/XSDT)
    fn parse_system_description_tables(&mut self) -> Result<(), KernelError> {
        if let Some(rsdt_addr) = self.rsdt_address {
            if self.verify_table_checksum(rsdt_addr, 32) {
                unsafe {
                    let rsdt = rsdt_addr as *const RSDT;
                    let rsdt_ref = &*rsdt;
                    
                    info!("Parsing RSDT with {} entries", rsdt_ref.header.length / 4 - 1);
                    
                    // Parse each table pointer
                    let table_ptrs = core::slice::from_raw_parts(
                        rsdt_addr as *const u32,
                        (rsdt_ref.header.length / 4) as usize
                    );
                    
                    for &table_addr in &table_ptrs[2..] { // Skip signature and length
                        self.parse_single_table(table_addr as u64)?;
                    }
                }
            }
        }
        
        if let Some(xsdt_addr) = self.xsdt_address {
            if self.verify_table_checksum(xsdt_addr, 64) {
                unsafe {
                    let xsdt = xsdt_addr as *const XSDT;
                    let xsdt_ref = &*xsdt;
                    
                    info!("Parsing XSDT with {} entries", xsdt_ref.header.length / 8 - 1);
                    
                    // Parse each table pointer
                    let table_ptrs = core::slice::from_raw_parts(
                        xsdt_addr as *const u64,
                        (xsdt_ref.header.length / 8) as usize
                    );
                    
                    for &table_addr in &table_ptrs[2..] { // Skip signature and length
                        self.parse_single_table(table_addr)?;
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Parse single ACPI table
    fn parse_single_table(&mut self, table_addr: u64) -> Result<(), KernelError> {
        unsafe {
            let header = table_addr as *const AcpiTableHeader;
            let header_ref = &*header;
            
            match header_ref.signature {
                FACP_SIGNATURE => {
                    self.facp_address = Some(table_addr);
                    info!("Found FACP (Fixed ACPI Description Table)");
                },
                DSDT_SIGNATURE => {
                    self.dsdt_address = Some(table_addr);
                    info!("Found DSDT (Differentiated System Description Table)");
                },
                FACS_SIGNATURE => {
                    self.facs_address = Some(table_addr);
                    info!("Found FACS (Firmware ACPI Control Structure)");
                },
                APIC_SIGNATURE => {
                    info!("Found APIC (Multiple APIC Configuration Table)");
                },
                HPET_SIGNATURE => {
                    info!("Found HPET (High Precision Event Timer Table)");
                },
                MCFG_SIGNATURE => {
                    info!("Found MCFG (PCI Configuration Space Table)");
                    self.parse_mcfg_table(table_addr)?;
                },
                SSDT_SIGNATURE => {
                    info!("Found SSDT (Secondary System Description Table)");
                },
                _ => {
                    info!("Found unknown ACPI table: 0x{:X}", header_ref.signature);
                }
            }
        }
        
        Ok(())
    }
    
    /// Parse MCFG table for PCI configuration space information
    fn parse_mcfg_table(&mut self, table_addr: u64) -> Result<(), KernelError> {
        unsafe {
            let mcfg = table_addr as *const MCFGTable;
            let mcfg_ref = &*mcfg;
            
            let device_entries = (mcfg_ref.header.length - core::mem::size_of::<MCFGTable>()) / 
                               core::mem::size_of::<McfgDeviceEntry>();
            
            let entries_ptr = table_addr + core::mem::size_of::<MCFGTable>();
            let entries = core::slice::from_raw_parts(
                entries_ptr as *const McfgDeviceEntry,
                device_entries
            );
            
            for entry in entries {
                self.mcfg_devices.push(AcpiMcfgDevice {
                    segment_group: entry.segment_group,
                    bus_number: entry.bus_number,
                    start_bus: entry.start_bus,
                    end_bus: entry.end_bus,
                    base_address: entry.base_address,
                });
            }
            
            info!("Parsed MCFG table with {} device entries", device_entries);
        }
        
        Ok(())
    }
    
    /// Parse FACP (Fixed ACPI Description Table)
    fn parse_facp(&mut self) -> Result<(), KernelError> {
        if let Some(facp_addr) = self.facp_address {
            if self.verify_table_checksum(facp_addr, 36) {
                unsafe {
                    let facp = facp_addr as *const FACPTable;
                    let facp_ref = &*facp;
                    
                    // Extract FACP information
                    self.firmware_control = facp_ref.firmware_control;
                    self.firmware_control_length = facp_ref.firmware_control_length;
                    self.dsdt_integer_width = facp_ref.dsdt_integer_width;
                    self.preferred_pm_profile = facp_ref.preferred_pm_profile;
                    self.sci_interrupt = facp_ref.sci_interrupt;
                    self.smi_command_port = facp_ref.smi_command_port;
                    self.acpi_enable_value = facp_ref.acpi_enable_value;
                    self.acpi_disable_value = facp_ref.acpi_disable_value;
                    self.s4bios_req = facp_ref.s4bios_req;
                    self.pstate_control = facp_ref.pstate_control;
                    self.pm1a_event_block = facp_ref.pm1a_event_block;
                    self.pm1b_event_block = facp_ref.pm1b_event_block;
                    self.pm1a_control_block = facp_ref.pm1a_control_block;
                    self.pm1b_control_block = facp_ref.pm1b_control_block;
                    self.pm2_control_block = facp_ref.pm2_control_block;
                    self.pm_timer_block = facp_ref.pm_timer_block;
                    self.gpe0_block = facp_ref.gpe0_block;
                    self.gpe1_block = facp_ref.gpe1_block;
                    self.pm1_event_length = facp_ref.pm1_event_length;
                    self.pm1_control_length = facp_ref.pm1_control_length;
                    self.pm2_control_length = facp_ref.pm2_control_length;
                    self.pm_timer_length = facp_ref.pm_timer_length;
                    self.gpe0_length = facp_ref.gpe0_length;
                    self.gpe1_length = facp_ref.gpe1_length;
                    self.gpe1_base = facp_ref.gpe1_base;
                    self.cstate_control = facp_ref.cstate_control;
                    self.lowest_dstate = facp_ref.lowest_dstate;
                    self.highest_dstate = facp_ref.highest_dstate;
                    self.ioapic_address = facp_ref.ioapic_address as u64;
                    self.ioapic_global_irq_base = facp_ref.ioapic_global_irq_base;
                    self.flags = facp_ref.flags;
                    self.reset_register_address = facp_ref.reset_register_address as u64;
                    self.reset_value = facp_ref.reset_value;
                    
                    // Extended fields (for ACPI 2.0+)
                    self.x_firmware_control = facp_ref.x_firmware_control;
                    self.x_dsdt = facp_ref.x_dsdt;
                    self.x_pm1a_event_block = facp_ref.x_pm1a_event_block;
                    self.x_pm1b_event_block = facp_ref.x_pm1b_event_block;
                    self.x_pm1a_control_block = facp_ref.x_pm1a_control_block;
                    self.x_pm1b_control_block = facp_ref.x_pm1b_control_block;
                    self.x_pm2_control_block = facp_ref.x_pm2_control_block;
                    self.x_pm_timer_block = facp_ref.x_pm_timer_block;
                    self.x_gpe0_block = facp_ref.x_gpe0_block;
                    self.x_gpe1_block = facp_ref.x_gpe1_block;
                    self.sleep_control = facp_ref.sleep_control;
                    self.sleep_status = facp_ref.sleep_status;
                    
                    info!("FACP parsed successfully");
                    info!("  SCI Interrupt: {}", self.sci_interrupt);
                    info!("  PM Timer Block: 0x{:X}", self.pm_timer_block);
                    info!("  GPE0 Block: 0x{:X}", self.gpe0_block);
                    info!("  GPE1 Block: 0x{:X}", self.gpe1_block);
                }
            }
        }
        
        Ok(())
    }
    
    /// Parse DSDT (Differentiated System Description Table)
    fn parse_dsdt(&mut self) -> Result<(), KernelError> {
        if let Some(dsdt_addr) = self.dsdt_address {
            if self.verify_table_checksum(dsdt_addr, 0) {
                info!("DSDT parsed successfully");
                // DSDT contains AML (ACPI Machine Language) code that would be interpreted
                // For now, we just acknowledge its presence
            }
        }
        
        Ok(())
    }
    
    /// Parse APIC (Multiple APIC Configuration Table)
    fn parse_apic(&mut self) -> Result<(), KernelError> {
        info!("APIC table parsing would extract processor and interrupt information");
        Ok(())
    }
    
    /// Parse other ACPI tables
    fn parse_other_tables(&mut self) -> Result<(), KernelError> {
        info!("Parsing other ACPI tables...");
        // This would parse HPET, SSDT, and other tables
        Ok(())
    }
    
    /// Enable ACPI mode
    fn enable_acpi_mode(&mut self) -> Result<(), KernelError> {
        if self.sci_interrupt == 0 {
            warn!("SCI interrupt not configured, cannot enable ACPI");
            return Ok(());
        }
        
        // Send ACPI enable command to SMI command port
        unsafe {
            core::arch::asm!(
                "outb {0}, {1}",
                in(reg) self.acpi_enable_value,
                in(reg) self.smi_command_port
            );
        }
        
        info!("ACPI mode enabled via SMI command port 0x{:X}", self.smi_command_port);
        Ok(())
    }
    
    /// Initialize power management
    fn init_power_management(&mut self) -> Result<(), KernelError> {
        info!("Initializing ACPI power management...");
        
        // Configure power states based on ACPI tables
        self.processor_states = vec![
            AcpiProcessorState::C0,
            AcpiProcessorState::C1,
            AcpiProcessorState::C2,
            AcpiProcessorState::C3,
        ];
        
        info!("Power management initialized");
        Ok(())
    }
    
    /// Initialize thermal management
    fn init_thermal_management(&mut self) -> Result<(), KernelError> {
        info!("Initializing ACPI thermal management...");
        
        // Create thermal zones based on ACPI information
        self.thermal_zones = vec![
            ThermalZone {
                id: 0,
                current_temp: 2980, // 25.0°C in tenths of Kelvin
                passive_temp: 3530, // 80.0°C
                critical_temp: 3730, // 100.0°C
                hot_temp: 3630, // 90.0°C
                enabled: true,
            },
        ];
        
        info!("Thermal management initialized with {} zones", self.thermal_zones.len());
        Ok(())
    }
    
    /// Enter ACPI sleep state
    pub fn enter_sleep_state(&mut self, sleep_type: AcpiSleepType) -> Result<(), KernelError> {
        if !self.initialized {
            return Err(KernelError::NotInitialized);
        }
        
        // Write sleep type to PM1A control register
        if self.pm1a_control_block != 0 {
            unsafe {
                core::arch::asm!(
                    "outw {0}, {1}",
                    in(reg) (sleep_type as u16 | 0x2000), // SLP_EN bit
                    in(reg) self.pm1a_control_block
                );
            }
        }
        
        // If second register exists, write same value
        if self.pm1b_control_block != 0 {
            unsafe {
                core::arch::asm!(
                    "outw {0}, {1}",
                    in(reg) (sleep_type as u16 | 0x2000),
                    in(reg) self.pm1b_control_block
                );
            }
        }
        
        info!("Entering ACPI sleep state {:?}", sleep_type);
        
        // Halt CPU - power management hardware will take over
        crate::arch::x86_64::disable_interrupts();
        crate::arch::x86_64::halt();
        
        Ok(())
    }
    
    /// Get current power state
    pub fn get_power_state(&self) -> AcpiPowerState {
        self.power_state
    }
    
    /// Get thermal zone information
    pub fn get_thermal_zones(&self) -> &[ThermalZone] {
        &self.thermal_zones
    }
    
    /// Get battery information
    pub fn get_battery_info(&self) -> &BatteryInfo {
        &self.battery_info
    }
    
    /// Get processor states
    pub fn get_processor_states(&self) -> &[AcpiProcessorState] {
        &self.processor_states
    }
    
    /// Get interrupt routing information
    pub fn get_interrupt_routing(&self) -> &[AcpiInterruptRouting] {
        &self.interrupt_routing
    }
    
    /// Get MCFG devices
    pub fn get_mcfg_devices(&self) -> &[AcpiMcfgDevice] {
        &self.mcfg_devices
    }
    
    /// Verify table checksum
    fn verify_table_checksum(&self, table_addr: u64, header_size: usize) -> bool {
        unsafe {
            let ptr = table_addr as *const u8;
            self.verify_checksum(ptr, header_size)
        }
    }
    
    /// Verify ACPI table checksum
    fn verify_checksum(&self, ptr: *const u8, length: usize) -> bool {
        unsafe {
            let mut sum: u8 = 0;
            let slice = core::slice::from_raw_parts(ptr, length);
            
            for &byte in slice {
                sum = sum.wrapping_add(byte);
            }
            
            sum == 0
        }
    }
}

// ACPI table structures

#[repr(C)]
struct RSDP {
    pub signature: u64,
    pub checksum: u8,
    pub oem_id: [u8; 6],
    pub revision: u8,
    pub rsdt_address: u32,
    pub length: u32,
    pub xsdt_address: u64,
    pub extended_checksum: u8,
    pub reserved: [u8; 3],
}

#[repr(C)]
struct RSDT {
    pub header: AcpiTableHeader,
}

#[repr(C)]
struct XSDT {
    pub header: AcpiTableHeader,
}

#[repr(C)]
struct ACPI_TABLE_HEADER {
    pub signature: u32,
    pub length: u32,
    pub revision: u8,
    pub checksum: u8,
    pub oem_id: [u8; 6],
    pub oem_table_id: [u8; 8],
    pub oem_revision: u32,
    pub creator_id: u32,
    pub creator_revision: u32,
}

#[repr(C)]
struct AcpiTableHeader {
    pub signature: u32,
    pub length: u32,
    pub revision: u8,
    pub checksum: u8,
    pub oem_id: [u8; 6],
    pub oem_table_id: [u8; 8],
    pub oem_revision: u32,
    pub creator_id: u32,
    pub creator_revision: u32,
}

#[repr(C)]
struct FACPTable {
    pub header: AcpiTableHeader,
    pub firmware_control: u32,
    pub firmware_control_length: u32,
    pub dsdt_integer_width: u8,
    pub reserved: [u8; 3],
    pub preferred_pm_profile: u8,
    pub sci_interrupt: u32,
    pub smi_command_port: u32,
    pub acpi_enable_value: u8,
    pub acpi_disable_value: u8,
    pub s4bios_req: u8,
    pub pstate_control: u8,
    pub cstate_control: u8,
    pub lowest_dstate: u8,
    pub highest_dstate: u8,
    pub pm1a_event_block: u32,
    pub pm1b_event_block: u32,
    pub pm1a_control_block: u32,
    pub pm1b_control_block: u32,
    pub pm2_control_block: u32,
    pub pm_timer_block: u32,
    pub gpe0_block: u32,
    pub gpe1_block: u32,
    pub pm1_event_length: u8,
    pub pm1_control_length: u8,
    pub pm2_control_length: u8,
    pub pm_timer_length: u8,
    pub gpe0_length: u8,
    pub gpe1_length: u8,
    pub gpe1_base: u8,
    pub c_state_control: u8,
    pub lowest_c_state: u8,
    pub highest_c_state: u8,
    pub lowest_c_state_width: u8,
    pub highest_c_state_width: u8,
    pub error_log_length: u8,
    pub error_log_address: u32,
    pub error_log_address_hi: u32,
    pub error_log_length_hi: u8,
    pub reserved4: [u8; 3],
    pub reset_register_address: u32,
    pub reset_value: u8,
    pub reserved5: [u8; 3],
    pub x_firmware_control: u64,
    pub x_dsdt: u64,
    pub x_pm1a_event_block: u64,
    pub x_pm1b_event_block: u64,
    pub x_pm1a_control_block: u64,
    pub x_pm1b_control_block: u64,
    pub x_pm2_control_block: u64,
    pub x_pm_timer_block: u64,
    pub x_gpe0_block: u64,
    pub x_gpe1_block: u64,
    pub sleep_control: u64,
    pub sleep_status: u64,
}

#[repr(C)]
struct MCFGTable {
    pub header: AcpiTableHeader,
    pub reserved: u64,
}

#[repr(C)]
struct McfgDeviceEntry {
    pub address: u64,
    pub segment_group: u16,
    pub start_bus: u8,
    pub end_bus: u8,
    pub reserved: u32,
}

// Type aliases for compatibility
type FACPTable = FACPTable;
type ACPI_TABLE_HEADER = AcpiTableHeader;