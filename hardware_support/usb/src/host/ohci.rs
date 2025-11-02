//! OHCI (Open Host Controller Interface) Driver
//! 
//! Supports USB 1.0/1.1 legacy hosts with features like:
//! - Low and Full speed USB support
//! - Simple interrupt-driven architecture
//! - ED (Endpoint Descriptor) and TD (Transfer Descriptor) based
//! - Legacy system support for older hardware

use crate::*;

#[cfg(feature = "std")]
use std::collections::BTreeMap;

/// OHCI Host Controller Registers
const OHCI_HCREVISION: u32 = 0x00;
const OHCI_HCCONTROL: u32 = 0x04;
const OHCI_HCCOMMANDSTATUS: u32 = 0x08;
const OHCI_HCINTERRUPTSTATUS: u32 = 0x0C;
const OHCI_HCINTERRUPTENABLE: u32 = 0x10;
const OHCI_HCINTERRUPTDISABLE: u32 = 0x14;
const OHCI_HCHCCA: u32 = 0x18;
const OHCI_HCPERIODCURRENTED: u32 = 0x1C;
const OHCI_HCCTRLHEADED: u32 = 0x20;
const OHCI_HCCTRLCURRENTED: u32 = 0x24;
const OHCI_HCBULKHEADED: u32 = 0x28;
const OHCI_HCBULKCURRENTED: u32 = 0x2C;
const OHCI_HCDONEHEAD: u32 = 0x30;
const OHCI_HCFMINTERVAL: u32 = 0x34;
const OHCI_HCFMREMAINING: u32 = 0x38;
const OHCI_HCFMRNUMBER: u32 = 0x3C;
const OHCI_HCFSMDYNAMICLOAD: u32 = 0x40;
const OHCI_HCCTRLRDCOMPLETION: u32 = 0x44;
const OHCI_HCPORTSTATUS1: u32 = 0x64;
const OHCI_HCPORTSTATUS2: u32 = 0x68;

/// OHCI Frame Interval
const OHCI_FR_INTERVAL: u32 = 0x2EDF; // Frame interval for 12MHz clock
const OHCI_FR_NUMBER_MASK: u32 = 0x3FFF;
const OHCI_FR_ADJUST_MASK: u32 = 0x1F;

/// OHCI Control Register bit fields
const OHCI_CTRL_CBSR_MASK: u32 = 0x00000003;
const OHCI_CTRL_CBSR_SHIFT: u32 = 0;
const OHCI_CTRL_HCFS_MASK: u32 = 0x0000001C;
const OHCI_CTRL_HCFS_SHIFT: u32 = 2;
const OHCI_CTRL_BLE: u32 = 0x00000020;
const OHCI_CTRL_CLE: u32 = 0x00000040;
const OHCI_CTRL_IE: u32 = 0x00000080;
const OHCI_CTRL_RWE: u32 = 0x00000100;
const OHCI_CTRL_RWC: u32 = 0x00000200;
const OHCI_CTRL_IR: u32 = 0x00000400;

/// OHCI Host Controller Functional States
const OHCI_HCFS_USBRESET: u32 = 0x00000000;
const OHCI_HCFS_USBOPERATIONAL: u32 = 0x00000008;
const OHCI_HCFS_USBSUSPEND: u32 = 0x0000000C;

/// OHCI Command Status Register bit fields
const OHCI_CMD_HCR: u32 = 0x00000001;
const OHCI_CMD_CLF: u32 = 0x00000002;
const OHCI_CMD_BLF: u32 = 0x00000004;
const OHCI_CMD_OCR: u32 = 0x00000008;

/// OHCI Interrupt Status/Enable/Disable bit fields
const OHCI_INT_SO: u32 = 0x00000001;
const OHCI_INT_WDH: u32 = 0x00000002;
const OHCI_INT_SF: u32 = 0x00000004;
const OHCI_INT_RD: u32 = 0x00000008;
const OHCI_INT_UE: u32 = 0x00000010;
const OHCI_INT_FNO: u32 = 0x00000020;
const OHCI_INT_RHSC: u32 = 0x00000040;
const OHCI_INT_OC: u32 = 0x40000000;
const OHCI_INT_MIE: u32 = 0x80000000;

/// OHCI Port Status Register bit fields
const OHCI_PORT_CCS: u32 = 0x00000001;
const OHCI_PORT_PES: u32 = 0x00000002;
const OHCI_PORT_PSS: u32 = 0x00000004;
const OHCI_PORT_POCI: u32 = 0x00000008;
const OHCI_PORT_PRSC: u32 = 0x00000010;
const OHCI_PORT_LSDA: u32 = 0x00000100;
const OHCI_PORT_CSC: u32 = 0x00000001;
const OHCI_PORT_PESC: u32 = 0x00000002;
const OHCI_PORT_PSSC: u32 = 0x00000004;
const OHCI_PORT_OCIC: u32 = 0x00000008;

/// OHCI Transfer Descriptor (TD) structure
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct OhciTD {
    pub next_td: u32,            // Link to next TD
    pub alt_next_td: u32,        // Alternate next TD
    pub cbp: u32,                // Current buffer pointer
    pub td_control: u32,         // TD control fields
}

/// OHCI Transfer Descriptor Control bit fields
const OHCI_TD_CC_MASK: u32 = 0xF0000000;
const OHCI_TD_CC_SHIFT: u32 = 28;
const OHCI_TD_EC_MASK: u32 = 0x0C000000;
const OHCI_TD_EC_SHIFT: u32 = 26;
const OHCI_TD_T_MASK: u32 = 0x03000000;
const OHCI_TD_T_SHIFT: u32 = 24;
const OHCI_TD_DI_MASK: u32 = 0x00E00000;
const OHCI_TD_DI_SHIFT: u32 = 21;
const OHCI_TD_DP_MASK: u32 = 0x00180000;
const OHCI_TD_DP_SHIFT: u32 = 19;
const OHCI_TD_CBP_MASK: u32 = 0x001FFFFF;

/// OHCI Completion Codes
const OHCI_CC_NO_ERROR: u32 = 0x00000000;
const OHCI_CC_CRC: u32 = 0x00000001;
const OHCI_CC_BIT_STUFFING: u32 = 0x00000002;
const OHCI_CC_DATA_TOGGLE_MISMATCH: u32 = 0x00000003;
const OHCI_CC_STALL: u32 = 0x00000004;
const OHCI_CC_DEVICE_NOT_RESPONDING: u32 = 0x00000005;
const OHCI_CC_PID_CHECK_FAILURE: u32 = 0x00000006;
const OHCI_CC_PID_UNEXPECTED: u32 = 0x00000007;
const OHCI_CC_DATA_UNDERUN: u32 = 0x00000008;
const OHCI_CC_DATA_OVERRUN: u32 = 0x00000009;
const OHCI_CC_BUFFER_OVERRUN: u32 = 0x0000000A;
const OHCI_CC_BUFFER_UNDERRUN: u32 = 0x0000000B;
const OHCI_CC_NOT_ACCESSED: u32 = 0x0000000C;

/// OHCI Endpoint Descriptor (ED) structure
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct OhciED {
    pub next_ed: u32,            // Link to next ED
    pub td_control: u32,         // ED control fields
    pub current_td: u32,         // Current TD pointer
    pub tail_td: u32,            // Tail TD pointer
}

/// OHCI Endpoint Descriptor Control bit fields
const OHCI_ED_FA_MASK: u32 = 0x0000007F;
const OHCI_ED_FA_SHIFT: u32 = 0;
const OHCI_ED_EN_MASK: u32 = 0x00000780;
const OHCI_ED_EN_SHIFT: u32 = 7;
const OHCI_ED_D_MASK: u32 = 0x00001800;
const OHCI_ED_D_SHIFT: u32 = 11;
const OHCI_ED_S_MASK: u32 = 0x00006000;
const OHCI_ED_S_SHIFT: u32 = 13;
const OHCI_ED_K_MASK: u32 = 0x00008000;
const OHCI_ED_F_MASK: u32 = 0x00010000;
const OHCI_ED_MPS_MASK: u32 = 0x07FF0000;
const OHCI_ED_MPS_SHIFT: u32 = 16;

/// OHCI Endpoint Direction codes
const OHCI_ED_DIR_OUT: u32 = 0x00000000;
const OHCI_ED_DIR_IN: u32 = 0x00000800;
const OHCI_ED_DIR_SETUP: u32 = 0x00001000;

/// OHCI Endpoint Speed codes
const OHCI_ED_SPEED_FULL: u32 = 0x00000000;
const OHCI_ED_SPEED_LOW: u32 = 0x00002000;

/// OHCI Host Controller capability parameters
#[derive(Debug, Clone, Copy)]
pub struct OhciCapabilityParams {
    pub hc_revision: u8,
    pub hc_control_hc_bulk_ep_ratios: u8,
    pub hc_hardware_support: u8,
    pub hc_reset_recovery_time: u8,
}

/// OHCI Controller State
#[derive(Debug, Clone)]
pub enum OhciControllerState {
    Uninitialized,
    Initialized,
    Operational,
    Suspended,
    Reset,
    Error,
}

/// OHCI Port Information
#[derive(Debug)]
pub struct OhciPort {
    pub port_number: u8,
    pub speed: UsbSpeed,
    pub status: u32,
    pub change_status: u32,
    pub connection_status: bool,
    pub device_attached: bool,
    pub power_state: UsbPowerState,
    pub port_enabled: bool,
    pub low_speed_device: bool,
}

/// OHCI Transfer Pool
#[derive(Debug)]
pub struct OhciTransferPool {
    pub ed_base: *mut u8,
    pub td_base: *mut u8,
    pub eds: Vec<*mut OhciED>,
    pub tds: Vec<*mut OhciTD>,
    pub ed_count: usize,
    pub td_count: usize,
}

/// OHCI Controller Implementation
pub struct OhciController {
    pub base_address: u64,
    pub capability_params: OhciCapabilityParams,
    pub state: OhciControllerState,
    pub hcca: *mut u8,
    pub control_head: *mut OhciED,
    pub control_current: *mut OhciED,
    pub bulk_head: *mut OhciED,
    pub bulk_current: *mut OhciED,
    pub done_head: *mut u32,
    pub transfer_pool: Option<OhciTransferPool>,
    pub ports: Vec<OhciPort>,
    pub max_ports: u8,
    pub frame_interval: u32,
    pub interrupt_threshold: u32,
    pub periodic_schedule_enabled: bool,
    pub control_schedule_enabled: bool,
    pub bulk_schedule_enabled: bool,
}

impl OhciController {
    /// Create a new OHCI controller instance
    pub fn new(base_address: u64) -> Self {
        let mut controller = Self {
            base_address,
            capability_params: OhciCapabilityParams {
                hc_revision: 0,
                hc_control_hc_bulk_ep_ratios: 0,
                hc_hardware_support: 0,
                hc_reset_recovery_time: 0,
            },
            state: OhciControllerState::Uninitialized,
            hcca: core::ptr::null_mut(),
            control_head: core::ptr::null_mut(),
            control_current: core::ptr::null_mut(),
            bulk_head: core::ptr::null_mut(),
            bulk_current: core::ptr::null_mut(),
            done_head: core::ptr::null_mut(),
            transfer_pool: None,
            ports: Vec::new(),
            max_ports: 0,
            frame_interval: 0,
            interrupt_threshold: 0,
            periodic_schedule_enabled: false,
            control_schedule_enabled: false,
            bulk_schedule_enabled: false,
        };

        // Read capability parameters
        controller.read_capability_params();

        controller
    }

    /// Read OHCI capability parameters from registers
    pub fn read_capability_params(&mut self) {
        unsafe {
            let cap_base = self.base_address as *const u8;
            
            self.capability_params.hc_revision = *cap_base;
            self.capability_params.hc_control_hc_bulk_ep_ratios = *(cap_base.add(1));
            self.capability_params.hc_hardware_support = *(cap_base.add(2));
            self.capability_params.hc_reset_recovery_time = *(cap_base.add(3));

            // Read max ports from frame length register
            let fminterval_reg = (self.base_address + OHCI_HCFMINTERVAL as u64) as *const u32;
            self.frame_interval = *fminterval_reg & 0x3FFF;

            // Determine max ports (typically 2 for UHCI compatible, 15 for others)
            self.max_ports = 2; // Conservative default for legacy systems
        }

        log::info!("OHCI Controller initialized:");
        log::info!("  HC Revision: {}", self.capability_params.hc_revision);
        log::info!("  Max Ports: {}", self.max_ports);
        log::info!("  Frame Interval: {}", self.frame_interval);
    }

    /// Initialize OHCI controller
    pub fn initialize(&mut self) -> UsbResult<()> {
        if self.state == OhciControllerState::Initialized {
            return Ok(());
        }

        // Reset the controller
        self.reset()?;

        // Initialize memory structures
        self.initialize_memory()?;

        // Configure the controller
        self.configure_controller()?;

        // Discover ports
        self.discover_ports()?;

        self.state = OhciControllerState::Initialized;
        log::info!("OHCI controller initialized successfully");

        Ok(())
    }

    /// Reset the OHCI controller
    pub fn reset(&mut self) -> UsbResult<()> {
        unsafe {
            let cmd_reg = (self.base_address + OHCI_HCCOMMANDSTATUS as u64) as *mut u32;
            let control_reg = (self.base_address + OHCI_HCCONTROL as u64) as *mut u32;

            // Reset the controller
            *cmd_reg = OHCI_CMD_HCR;
            
            // Wait for reset to complete
            for _ in 0..10000 {
                let cmd_status = *cmd_reg;
                if cmd_status & OHCI_CMD_HCR == 0 {
                    break;
                }
            }

            // Set controller to reset state
            let mut control = *control_reg;
            control &= !OHCI_CTRL_HCFS_MASK;
            control |= OHCI_HCFS_USBRESET;
            *control_reg = control;

            self.state = OhciControllerState::Reset;
        }

        log::info!("OHCI controller reset completed");
        Ok(())
    }

    /// Initialize OHCI memory structures (HCCA, ED/TD pools)
    pub fn initialize_memory(&mut self) -> UsbResult<()> {
        unsafe {
            // Allocate HCCA (Host Controller Communications Area)
            let hcca_size = 256; // 256 bytes as per OHCI spec
            let hcca_base = alloc::alloc::alloc(alloc::alloc::Layout::from_size_align(hcca_size, 256)?);
            if hcca_base.is_null() {
                return Err(UsbDriverError::UnsupportedFeature);
            }
            self.hcca = hcca_base;

            // Zero out HCCA
            core::ptr::write_bytes(self.hcca, 0, hcca_size);

            // Initialize ED and TD pools
            let num_eds = 32;
            let num_tds = 128;
            let ed_size = num_eds * mem::size_of::<OhciED>();
            let td_size = num_tds * mem::size_of::<OhciTD>();

            let ed_base = alloc::alloc::alloc(alloc::alloc::Layout::from_size_align(ed_size, 32)?);
            let td_base = alloc::alloc::alloc(alloc::alloc::Layout::from_size_align(td_size, 32)?);

            if ed_base.is_null() || td_base.is_null() {
                return Err(UsbDriverError::UnsupportedFeature);
            }

            let mut eds = Vec::new();
            for i in 0..num_eds {
                let ed_ptr = ed_base.add(i * mem::size_of::<OhciED>()) as *mut OhciED;
                eds.push(ed_ptr);
            }

            let mut tds = Vec::new();
            for i in 0..num_tds {
                let td_ptr = td_base.add(i * mem::size_of::<OhciTD>()) as *mut OhciTD;
                tds.push(td_ptr);
            }

            let pool = OhciTransferPool {
                ed_base,
                td_base,
                eds,
                tds,
                ed_count: num_eds,
                td_count: num_tds,
            };

            self.transfer_pool = Some(pool);

            // Initialize done head
            self.done_head = core::ptr::null_mut();

            // Write HCCA base address to controller
            let hcca_reg = (self.base_address + OHCI_HCHCCA as u64) as *mut u32;
            *hcca_reg = self.hcca as u32;
        }

        log::info!("OHCI memory structures initialized:");
        log::info!("  HCCA: {:#x}", self.hcca as u64);
        if let Some(pool) = &self.transfer_pool {
            log::info!("  EDs: {}", pool.ed_count);
            log::info!("  TDs: {}", pool.td_count);
        }
        Ok(())
    }

    /// Configure OHCI controller operational parameters
    pub fn configure_controller(&mut self) -> UsbResult<()> {
        unsafe {
            let control_reg = (self.base_address + OHCI_HCCONTROL as u64) as *mut u32;
            let mut control = *control_reg;

            // Set frame interval
            let fminterval_reg = (self.base_address + OHCI_HCFMINTERVAL as u64) as *mut u32;
            *fminterval_reg = OHCI_FR_INTERVAL | (31 << 16); // 12MHz clock, 31 frame adjust

            // Configure control register
            control &= !OHCI_CTRL_HCFS_MASK;
            control |= OHCI_HCFS_USBOPERATIONAL; // Set to operational state
            control |= OHCI_CTRL_IE; // Enable interrupts
            control |= OHCI_CTRL_RWE; // Enable remote wakeup

            *control_reg = control;

            // Initialize head pointers for control and bulk lists
            if let Some(pool) = &self.transfer_pool {
                if !pool.eds.is_empty() {
                    self.control_head = pool.eds[0];
                    self.bulk_head = pool.eds[1 % pool.ed_count];
                }
            }

            // Clear interrupt status
            let intr_stat_reg = (self.base_address + OHCI_HCINTERRUPTSTATUS as u64) as *mut u32;
            *intr_stat_reg = 0xFFFFFFFF;
        }

        self.state = OhciControllerState::Operational;
        log::info!("OHCI controller configured for operational state");
        Ok(())
    }

    /// Discover and initialize ports
    pub fn discover_ports(&mut self) -> UsbResult<()> {
        self.ports.clear();

        // Start with 2 ports (UHCI compatible), will be discovered dynamically
        for port_num in 1..=self.max_ports {
            let port = OhciPort {
                port_number: port_num,
                speed: UsbSpeed::Full,
                status: 0,
                change_status: 0,
                connection_status: false,
                device_attached: false,
                power_state: UsbPowerState::Active,
                port_enabled: false,
                low_speed_device: false,
            };

            self.ports.push(port);
        }

        log::info!("Discovered {} OHCI ports", self.ports.len());
        Ok(())
    }

    /// Get port status
    pub fn get_port_status(&mut self, port_number: u8) -> UsbResult<u32> {
        if port_number == 0 || port_number > self.max_ports {
            return Err(UsbDriverError::DeviceNotFound { address: port_number });
        }

        unsafe {
            let port_offset = match port_number {
                1 => OHCI_HCPORTSTATUS1,
                2 => OHCI_HCPORTSTATUS2,
                _ => OHCI_HCPORTSTATUS1 + ((port_number - 1) * 0x04),
            };

            let portsc_reg = (self.base_address + port_offset as u64) as *const u32;
            let status = *portsc_reg;

            // Update port information
            if port_number as usize <= self.ports.len() {
                let port = &mut self.ports[port_number as usize - 1];
                port.status = status;
                port.connection_status = (status & OHCI_PORT_CCS) != 0;
                port.port_enabled = (status & OHCI_PORT_PES) != 0;
                port.low_speed_device = (status & OHCI_PORT_LSDA) != 0;
                port.device_attached = port.connection_status;
                port.speed = if port.low_speed_device { 
                    UsbSpeed::Low 
                } else { 
                    UsbSpeed::Full 
                };
            }

            Ok(status)
        }
    }

    /// Set port feature
    pub fn set_port_feature(&mut self, port_number: u8, feature: u32) -> UsbResult<()> {
        if port_number == 0 || port_number > self.max_ports {
            return Err(UsbDriverError::DeviceNotFound { address: port_number });
        }

        unsafe {
            let port_offset = match port_number {
                1 => OHCI_HCPORTSTATUS1,
                2 => OHCI_HCPORTSTATUS2,
                _ => OHCI_HCPORTSTATUS1 + ((port_number - 1) * 0x04),
            };

            let portsc_reg = (self.base_address + port_offset as u64) as *mut u32;
            let mut status = *portsc_reg;

            match feature {
                // USB Port Enable
                0x0004 => {
                    status |= OHCI_PORT_PES;
                }
                // USB Port Suspend
                0x0005 => {
                    status |= OHCI_PORT_PSS;
                }
                // USB Port Reset
                0x0004 => {
                    status |= OHCI_PORT_PRSC;
                }
                // USB Port Power
                0x0008 => {
                    status |= OHCI_PORT_PPOWER;
                }
                _ => {
                    log::warn!("Unsupported OHCI port feature: {:#x}", feature);
                    return Err(UsbDriverError::UnsupportedFeature);
                }
            }

            *portsc_reg = status;
        }

        Ok(())
    }

    /// Clear port feature
    pub fn clear_port_feature(&mut self, port_number: u8, feature: u32) -> UsbResult<()> {
        if port_number == 0 || port_number > self.max_ports {
            return Err(UsbDriverError::DeviceNotFound { address: port_number });
        }

        unsafe {
            let port_offset = match port_number {
                1 => OHCI_HCPORTSTATUS1,
                2 => OHCI_HCPORTSTATUS2,
                _ => OHCI_HCPORTSTATUS1 + ((port_number - 1) * 0x04),
            };

            let portsc_reg = (self.base_address + port_offset as u64) as *mut u32;
            let mut status = *portsc_reg;

            match feature {
                // USB Port Enable Disable Change
                0x0004 => {
                    status &= !OHCI_PORT_PESC;
                }
                // USB Port Suspend Status Change
                0x0008 => {
                    status &= !OHCI_PORT_PSSC;
                }
                // USB Port Reset Status Change
                0x000A => {
                    status &= !OHCI_PORT_PRSC;
                }
                // USB Port Connection Status Change
                0x0008 => {
                    status &= !OHCI_PORT_CSC;
                }
                // USB Port Over Current Indicator Change
                0x000C => {
                    status &= !OHCI_PORT_OCIC;
                }
                _ => {
                    log::warn!("Unsupported OHCI port feature: {:#x}", feature);
                    return Err(UsbDriverError::UnsupportedFeature);
                }
            }

            *portsc_reg = status;
        }

        Ok(())
    }

    /// Enable periodic schedule
    pub fn enable_periodic_schedule(&mut self) -> UsbResult<()> {
        unsafe {
            let control_reg = (self.base_address + OHCI_HCCONTROL as u64) as *mut u32;
            let mut control = *control_reg;
            
            control |= OHCI_CTRL_PLE;
            *control_reg = control;
            self.periodic_schedule_enabled = true;
        }

        log::info!("OHCI periodic schedule enabled");
        Ok(())
    }

    /// Enable control schedule
    pub fn enable_control_schedule(&mut self) -> UsbResult<()> {
        unsafe {
            let control_reg = (self.base_address + OHCI_HCCONTROL as u64) as *mut u32;
            let mut control = *control_reg;
            
            control |= OHCI_CTRL_CLE;
            *control_reg = control;
            self.control_schedule_enabled = true;
        }

        log::info!("OHCI control schedule enabled");
        Ok(())
    }

    /// Enable bulk schedule
    pub fn enable_bulk_schedule(&mut self) -> UsbResult<()> {
        unsafe {
            let control_reg = (self.base_address + OHCI_HCCONTROL as u64) as *mut u32;
            let mut control = *control_reg;
            
            control |= OHCI_CTRL_BLE;
            *control_reg = control;
            self.bulk_schedule_enabled = true;
        }

        log::info!("OHCI bulk schedule enabled");
        Ok(())
    }

    /// Get current frame number
    pub fn get_frame_number(&self) -> u32 {
        unsafe {
            let fmnumber_reg = (self.base_address + OHCI_HCFMRNUMBER as u64) as *const u32;
            *fmnumber_reg & OHCI_FR_NUMBER_MASK
        }
    }

    /// Create endpoint descriptor
    pub fn create_endpoint_descriptor(&mut self, device_address: u8, endpoint_number: u8, 
                                    direction: UsbDirection, speed: UsbSpeed, 
                                    packet_size: u16) -> UsbResult<*mut OhciED> {
        let pool = self.transfer_pool.as_mut().ok_or(UsbDriverError::UnsupportedFeature)?;
        
        // Find free ED
        for &ed_ptr in &pool.eds {
            unsafe {
                let ed = &mut *ed_ptr;
                if ed.td_control == 0 { // Unused ED
                    // Initialize ED
                    ed.next_ed = 0x00000001; // Terminate bit
                    ed.td_control = (device_address as u32) |
                                   (endpoint_number as u32) << OHCI_ED_EN_SHIFT |
                                   (match direction {
                                       UsbDirection::Out => OHCI_ED_DIR_OUT,
                                       UsbDirection::In => OHCI_ED_DIR_IN,
                                   }) |
                                   (match speed {
                                       UsbSpeed::Low => OHCI_ED_SPEED_LOW,
                                       UsbSpeed::Full => OHCI_ED_SPEED_FULL,
                                       _ => OHCI_ED_SPEED_FULL, // Default
                                   }) |
                                   (packet_size as u32) << OHCI_ED_MPS_SHIFT;
                    
                    ed.current_td = 0x00000001; // Terminate
                    ed.tail_td = 0x00000001; // Terminate
                    
                    return Ok(ed_ptr);
                }
            }
        }
        
        Err(UsbDriverError::UnsupportedFeature)
    }

    /// Create transfer descriptor
    pub fn create_transfer_descriptor(&mut self, data: &[u8], toggle_bit: bool) -> UsbResult<*mut OhciTD> {
        let pool = self.transfer_pool.as_mut().ok_or(UsbDriverError::UnsupportedFeature)?;
        
        // Find free TD
        for &td_ptr in &pool.tds {
            unsafe {
                let td = &mut *td_ptr;
                if td.td_control == 0 { // Unused TD
                    // Initialize TD
                    td.next_td = 0x00000001; // Terminate bit
                    td.alt_next_td = 0x00000001; // Terminate bit
                    td.cbp = data.as_ptr() as u32;
                    td.td_control = OHCI_CC_NOT_ACCESSED << OHCI_TD_CC_SHIFT |
                                   (if toggle_bit { 1 } else { 0 }) << OHCI_TD_T_SHIFT |
                                   (data.len() as u32 & OHCI_TD_CBP_MASK);
                    
                    return Ok(td_ptr);
                }
            }
        }
        
        Err(UsbDriverError::UnsupportedFeature)
    }

    /// Get controller statistics
    pub fn get_stats(&self) -> UsbControllerStats {
        UsbControllerStats {
            total_transactions: 0,
            successful_transactions: 0,
            failed_transactions: 0,
            bytes_transferred: 0,
            error_count: 0,
            last_error: None,
        }
    }

    /// Check if controller is operational
    pub fn is_operational(&self) -> bool {
        self.state == OhciControllerState::Operational
    }
}

impl Drop for OhciController {
    fn drop(&mut self) {
        unsafe {
            // Clean up allocated memory
            if !self.hcca.is_null() {
                let hcca_size = 256;
                alloc::alloc::dealloc(
                    self.hcca as *mut u8,
                    alloc::alloc::Layout::from_size_align(hcca_size, 256).unwrap(),
                );
            }

            if let Some(pool) = &self.transfer_pool {
                alloc::alloc::dealloc(
                    pool.ed_base as *mut u8,
                    alloc::alloc::Layout::from_size_align(pool.ed_count * mem::size_of::<OhciED>(), 32).unwrap(),
                );
                alloc::alloc::dealloc(
                    pool.td_base as *mut u8,
                    alloc::alloc::Layout::from_size_align(pool.td_count * mem::size_of::<OhciTD>(), 32).unwrap(),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ohci_controller_creation() {
        let controller = OhciController::new(0x12345678);
        assert_eq!(controller.base_address, 0x12345678);
        assert_eq!(controller.state, OhciControllerState::Uninitialized);
    }

    #[test]
    fn test_ohci_port_creation() {
        let port = OhciPort {
            port_number: 1,
            speed: UsbSpeed::Full,
            status: 0,
            change_status: 0,
            connection_status: false,
            device_attached: false,
            power_state: UsbPowerState::Active,
            port_enabled: false,
            low_speed_device: false,
        };

        assert_eq!(port.port_number, 1);
        assert_eq!(port.speed, UsbSpeed::Full);
        assert!(!port.connection_status);
    }
}