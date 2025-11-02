//! EHCI (Enhanced Host Controller Interface) Driver
//! 
//! Supports USB 2.0 high-speed hosts with features like:
//! - High-speed USB 2.0 support
//! - Asynchronous scheduling
//! - Isochronous scheduling
//! - Support for companion controllers

use crate::*;

#[cfg(feature = "std")]
use std::collections::BTreeMap;

/// EHCI Capability Registers
const EHCI_CAPLENGTH: u32 = 0x00;
const EHCI_HCIVERSION: u32 = 0x02;
const EHCI_HCSPARAMS: u32 = 0x04;
const EHCI_HCCPARAMS: u32 = 0x08;
const EHCI_DBOFF: u32 = 0x0C;
const EHCI_DTI: u32 = 0x10;

/// EHCI Operational Registers
const EHCI_USBCMD: u32 = 0x20;
const EHCI_USBSTS: u32 = 0x24;
const EHCI_USBINTR: u32 = 0x28;
const EHCI_FRINDEX: u32 = 0x2C;
const EHCI_CTRLDSSYNPAGE: u32 = 0x30;
const EHCI_PERIODICLISTBASE: u32 = 0x34;
const EHCI_ASYNCLISTADDR: u32 = 0x38;
const EHCI_ASYNCTHHRESHOLD: u32 = 0x3C;

/// EHCI Port Status and Control Register
const EHCI_PORTSC1: u32 = 0x44;

/// EHCI Command Register (USBCMD) bit fields
const EHCI_CMD_ITC_MASKS: u32 = 0x000000FF;
const EHCI_CMD_ITC_SHIFT: u32 = 16;
const EHCI_CMD_IAADB: u32 = 0x00000080;
const EHCI_CMD_ASE: u32 = 0x00000040;
const EHCI_CMD_PSE: u32 = 0x00000020;
const EHCI_CMD_FLS: u32 = 0x0000000C;
const EHCI_CMD_LHCRT: u32 = 0x00000008;
const EHCI_CMD_C_SSS: u32 = 0x00000004;
const EHCI_CMD_C_RSS: u32 = 0x00000002;
const EHCI_CMD_RS: u32 = 0x00000001;
const EHCI_CMD_HCRESET: u32 = 0x00000002;
const EHCI_CMD_HLE: u32 = 0x00000100;
const EHCI_CMD_PPCEE: u32 = 0x00000200;
const EHCI_CMD_FS1: u32 = 0x00000400;
const EHCI_CMD_ATDTW: u32 = 0x00000800;
const EHCI_CMD_ASYPCE: u32 = 0x00001000;
const EHCI_CMD_LPM: u32 = 0x00002000;

/// EHCI Status Register (USBSTS) bit fields
const EHCI_STS_ASS: u32 = 0x00008000;
const EHCI_STS_PSS: u32 = 0x00004000;
const EHCI_STS_RECL: u32 = 0x00002000;
const EHCI_STS_HCH: u32 = 0x00001000;
const EHCI_STS_AAI: u32 = 0x00000800;
const EHCI_STS_NAK: u32 = 0x00000400;
const EHCI_STS_PCD: u32 = 0x00000200;
const EHCI_STS_FLR: u32 = 0x00000100;
const EHCI_STS_PCI: u32 = 0x00000080;
const EHCI_STS_ERRINT: u32 = 0x00000040;
const EHCI_STS_INT: u32 = 0x00000020;
const EHCI_STS_HSE: u32 = 0x00000010;
const EHCI_STS_ASR: u32 = 0x00000008;
const EHCI_STS_PSSR: u32 = 0x00000004;
const EHCI_STS_RHSC: u32 = 0x00000002;
const EHCI_STS_HCINTERRUPTS: u32 = 0x00000001;

/// EHCI Port Status Register (PORTSC1) bit fields
const EHCI_PORT_W1C_BITS: u32 = 0xFF007FFF;
const EHCI_PORT_CONNECT: u32 = 0x00000001;
const EHCI_PORT_PEC: u32 = 0x00000002;
const EHCI_PORT_PEDC: u32 = 0x00000004;
const EHCI_PORT_OCA: u32 = 0x00000008;
const EHCI_PORT_OCC: u32 = 0x00000010;
const EHCI_PORT_FPR: u32 = 0x00000020;
const EHCI_PORT_SUSP: u32 = 0x00000040;
const EHCI_PORT_RPR: u32 = 0x00000080;
const EHCI_PORT_LS: u32 = 0x00000100;
const EHCI_PORT_PP: u32 = 0x00001000;
const EHCI_PORT_PIC: u32 = 0x00006000;
const EHCI_PORT_POWNER: u32 = 0x20000000;
const EHCI_PORT_PPOWER: u32 = 0x40000000;
const EHCI_PORT_READONLY: u32 = 0x80000000;

/// EHCI Transfer Descriptor (qTD) structure
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct EhciQTD {
    pub next_qtd: u32,           // Next qTD pointer
    pub alternate_next_qtd: u32, // Alternate next qTD pointer
    pub token: u32,              // Status and token fields
    pub buffer_page0: u32,       // Buffer page 0
    pub buffer_page1: u32,       // Buffer page 1
    pub buffer_page2: u32,       // Buffer page 2
    pub buffer_page3: u32,       // Buffer page 3
    pub buffer_page4: u32,       // Buffer page 4
    pub extended_buffer: u32,    // Extended buffer (for large transfers)
}

/// EHCI Transfer Descriptor Token bit fields
const EHCI_QTD_TOKENS_ACTIVE: u32 = 0x00000080;
const EHCI_QTD_TOKENS_HALT: u32 = 0x00000040;
const EHCI_QTD_TOKENS_DATAGERR: u32 = 0x00000020;
const EHCI_QTD_TOKENS_BABBLED: u32 = 0x00000010;
const EHCI_QTD_TOKENS_XACT_ERROR: u32 = 0x00000008;
const EHCI_QTD_TOKENS_MISSED_FRAME: u32 = 0x00000004;
const EHCI_QTD_TOKENS_SPLIT: u32 = 0x00000002;
const EHCI_QTD_TOKENS_PING_STATE: u32 = 0x00000001;
const EHCI_QTD_TOKENS_IOC: u32 = 0x00008000;
const EHCI_QTD_TOKENS_CPAGE_SHIFT: u32 = 12;
const EHCI_QTD_TOKENS_CPAGE_MASK: u32 = 0x00007000;

/// EHCI Transfer Queue Head (qH) structure
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct EhciQH {
    pub link: u32,                   // Link to next QH or itd
    pub ep_capabilities: u32,        // Endpoint characteristics
    pub current_qtd: u32,            // Current qTD pointer
    pub next_qtd: u32,               // Next qTD pointer
    pub altnext_qtd: u32,            // Alternate next qTD pointer
    pub token: u32,                  // Status and token fields
    pub buffer_page0: u32,           // Buffer page 0
    pub buffer_page1: u32,           // Buffer page 1
    pub buffer_page2: u32,           // Buffer page 2
    pub buffer_page3: u32,           // Buffer page 3
    pub buffer_page4: u32,           // Buffer page 4
}

/// EHCI Host Controller capability parameters
#[derive(Debug, Clone, Copy)]
pub struct EhciCapabilityParams {
    pub cap_length: u8,
    pub hc_interface_version: u16,
    pub hcs_params: u32,
    pub hcc_params: u32,
    pub doorbell_offset: u32,
    pub usb_dccparams: u32,
}

/// EHCI Controller State
#[derive(Debug, Clone)]
pub enum EhciControllerState {
    Uninitialized,
    Initialized,
    Running,
    Suspended,
    Error,
}

/// EHCI Port Information
#[derive(Debug)]
pub struct EhciPort {
    pub port_number: u8,
    pub speed: UsbSpeed,
    pub status: u32,
    pub connection_status: bool,
    pub device_attached: bool,
    pub power_state: UsbPowerState,
    pub is_oc_enabled: bool,
    pub is_connected_to_companion: bool,
}

/// EHCI Queue Head Pool
#[derive(Debug)]
pub struct EhciQueueHeadPool {
    pub base: *mut u8,
    pub size: usize,
    pub queue_heads: BTreeMap<u8, *mut EhciQH>,
}

/// EHCI Transfer Descriptor Pool
#[derive(Debug)]
pub struct EhciQtdPool {
    pub base: *mut u8,
    pub size: usize,
    pub qtds: Vec<*mut EhciQTD>,
}

/// EHCI Controller Implementation
pub struct EhciController {
    pub base_address: u64,
    pub capability_params: EhciCapabilityParams,
    pub state: EhciControllerState,
    pub doorbell_array: *mut u32,
    pub periodic_frame_list: *mut u32,
    pub async_list_head: *mut EhciQH,
    pub queue_head_pool: Option<EhciQueueHeadPool>,
    pub qtd_pool: Option<EhciQtdPool>,
    pub ports: Vec<EhciPort>,
    pub max_ports: u8,
    pub frame_number: u32,
    pub frame_list_size: usize,
    pub interrupt_threshold: u32,
    pub companion_required: bool,
    pub port_routing_enabled: bool,
}

impl EhciController {
    /// Create a new EHCI controller instance
    pub fn new(base_address: u64) -> Self {
        let mut controller = Self {
            base_address,
            capability_params: EhciCapabilityParams {
                cap_length: 0,
                hc_interface_version: 0,
                hcs_params: 0,
                hcc_params: 0,
                doorbell_offset: 0,
                usb_dccparams: 0,
            },
            state: EhciControllerState::Uninitialized,
            doorbell_array: core::ptr::null_mut(),
            periodic_frame_list: core::ptr::null_mut(),
            async_list_head: core::ptr::null_mut(),
            queue_head_pool: None,
            qtd_pool: None,
            ports: Vec::new(),
            max_ports: 0,
            frame_number: 0,
            frame_list_size: 0,
            interrupt_threshold: 0x08,
            companion_required: false,
            port_routing_enabled: false,
        };

        // Read capability parameters
        controller.read_capability_params();

        controller
    }

    /// Read EHCI capability parameters from registers
    pub fn read_capability_params(&mut self) {
        unsafe {
            let cap_base = self.base_address as *const u8;
            
            self.capability_params.cap_length = *cap_base;
            self.capability_params.hc_interface_version = core::ptr::read_unaligned(
                cap_base.add(2) as *const u16
            ) as u16;
            self.capability_params.hcs_params = core::ptr::read_unaligned(
                self.base_address.add(EHCI_HCSPARAMS as usize) as *const u32
            );
            self.capability_params.hcc_params = core::ptr::read_unaligned(
                self.base_address.add(EHCI_HCCPARAMS as usize) as *const u32
            );
            self.capability_params.doorbell_offset = core::ptr::read_unaligned(
                self.base_address.add(EHCI_DBOFF as usize) as *const u32
            );
            self.capability_params.usb_dccparams = core::ptr::read_unaligned(
                self.base_address.add(EHCI_DTI as usize) as *const u32
            );

            // Extract important parameters
            self.max_ports = (self.capability_params.hcs_params & 0x0F) as u8;
            self.companion_required = (self.capability_params.hcs_params & (1 << 4)) != 0;
            self.port_routing_enabled = (self.capability_params.hcs_params & (1 << 5)) != 0;

            // Determine frame list size
            let fls = (self.capability_params.hcc_params >> 2) & 0x3;
            self.frame_list_size = match fls {
                0 => 1024,
                1 => 512,
                2 => 256,
                _ => 1024,
            };
        }

        log::info!("EHCI Controller initialized:");
        log::info!("  Max Ports: {}", self.max_ports);
        log::info!("  Companion Required: {}", self.companion_required);
        log::info!("  Frame List Size: {}", self.frame_list_size);
        log::info!("  HC Version: {:#06x}", self.capability_params.hc_interface_version);
    }

    /// Initialize EHCI controller
    pub fn initialize(&mut self) -> UsbResult<()> {
        if self.state == EhciControllerState::Initialized {
            return Ok(());
        }

        // Reset the controller
        self.reset()?;

        // Enable controller
        self.enable()?;

        // Initialize frame lists
        self.initialize_frame_lists()?;

        // Initialize queue head and qTD pools
        self.initialize_pools()?;

        // Discover ports
        self.discover_ports()?;

        self.state = EhciControllerState::Initialized;
        log::info!("EHCI controller initialized successfully");

        Ok(())
    }

    /// Reset the EHCI controller
    pub fn reset(&mut self) -> UsbResult<()> {
        unsafe {
            let cmd_reg = (self.base_address + EHCI_USBCMD as u64) as *mut u32;
            let status_reg = (self.base_address + EHCI_USBSTS as u64) as *mut u32;

            // Reset the controller
            *cmd_reg = EHCI_CMD_HCRESET;
            
            // Wait for reset to complete
            for _ in 0..10000 {
                let status = *status_reg;
                if status & EHCI_STS_HCH == 0 {
                    break;
                }
            }

            // Check if reset was successful
            let status = *status_reg;
            if status & EHCI_STS_HCH == 0 {
                return Err(UsbDriverError::ControllerNotInitialized);
            }

            self.state = EhciControllerState::Uninitialized;
        }

        log::info!("EHCI controller reset completed");
        Ok(())
    }

    /// Enable the EHCI controller
    pub fn enable(&mut self) -> UsbResult<()> {
        unsafe {
            let cmd_reg = (self.base_address + EHCI_USBCMD as u64) as *mut u32;
            let mut cmd = *cmd_reg;
            
            // Set Run/Stop bit
            cmd |= EHCI_CMD_RS;
            *cmd_reg = cmd;

            // Enable interrupts
            cmd |= EHCI_CMD_PPCEE; // Port Power Control Enable
            *cmd_reg = cmd;

            // Wait for controller to start
            for _ in 0..10000 {
                let status = *(self.base_address + EHCI_USBSTS as u64) as *const u32;
                if status & EHCI_STS_HCH == 0 {
                    break;
                }
            }

            let status = *(self.base_address + EHCI_USBSTS as u64) as *const u32;
            if status & EHCI_STS_HCH != 0 {
                return Err(UsbDriverError::ControllerNotInitialized);
            }

            self.state = EhciControllerState::Running;
        }

        log::info!("EHCI controller enabled and running");
        Ok(())
    }

    /// Initialize frame lists (periodic and asynchronous)
    pub fn initialize_frame_lists(&mut self) -> UsbResult<()> {
        unsafe {
            // Allocate periodic frame list
            let frame_list_size = self.frame_list_size * mem::size_of::<u32>();
            let frame_list = alloc::alloc::alloc(alloc::alloc::Layout::from_size_align(frame_list_size, 4096)?);
            if frame_list.is_null() {
                return Err(UsbDriverError::UnsupportedFeature);
            }

            self.periodic_frame_list = frame_list as *mut u32;

            // Zero out frame list
            core::ptr::write_bytes(self.periodic_frame_list, 0, self.frame_list_size);

            // Allocate async list head
            let async_head_size = mem::size_of::<EhciQH>();
            let async_head = alloc::alloc::alloc(alloc::alloc::Layout::from_size_align(async_head_size, 64)?);
            if async_head.is_null() {
                return Err(UsbDriverError::UnsupportedFeature);
            }

            self.async_list_head = async_head as *mut EhciQH;

            // Initialize async list head
            let qh = &mut *self.async_list_head;
            qh.link = (self.async_list_head as u32) | 0x00000001; // Terminate bit
            qh.ep_capabilities = 0x80000001; // Head of reclamation list
            qh.current_qtd = 0;
            qh.next_qtd = 0x00000001; // Terminate bit
            qh.altnext_qtd = 0x00000001; // Terminate bit
            qh.token = 0x40000000; // Device address 0
            qh.buffer_page0 = 0;
            qh.buffer_page1 = 0;
            qh.buffer_page2 = 0;
            qh.buffer_page3 = 0;
            qh.buffer_page4 = 0;

            // Write frame list base addresses to registers
            let periodic_reg = (self.base_address + EHCI_PERIODICLISTBASE as u64) as *mut u32;
            let async_reg = (self.base_address + EHCI_ASYNCLISTADDR as u64) as *mut u32;

            *periodic_reg = self.periodic_frame_list as u32;
            *async_reg = self.async_list_head as u32;
        }

        log::info!("Frame lists initialized:");
        log::info!("  Periodic frame list: {:#x}", self.periodic_frame_list as u64);
        log::info!("  Async list head: {:#x}", self.async_list_head as u64);
        Ok(())
    }

    /// Initialize queue head and qTD pools
    pub fn initialize_pools(&mut self) -> UsbResult<()> {
        let num_queue_heads = self.max_ports as usize * 32; // 32 endpoints per port
        let num_qtds = num_queue_heads * 4; // 4 qTDs per endpoint

        // Initialize queue head pool
        unsafe {
            let qh_pool_size = num_queue_heads * mem::size_of::<EhciQH>();
            let qh_pool_base = alloc::alloc::alloc(alloc::alloc::Layout::from_size_align(qh_pool_size, 64)?);
            if qh_pool_base.is_null() {
                return Err(UsbDriverError::UnsupportedFeature);
            }

            let qh_pool = EhciQueueHeadPool {
                base: qh_pool_base,
                size: qh_pool_size,
                queue_heads: BTreeMap::new(),
            };

            self.queue_head_pool = Some(qh_pool);
        }

        // Initialize qTD pool
        unsafe {
            let qtd_pool_size = num_qtds * mem::size_of::<EhciQTD>();
            let qtd_pool_base = alloc::alloc::alloc(alloc::alloc::Layout::from_size_align(qtd_pool_size, 64)?);
            if qtd_pool_base.is_null() {
                return Err(UsbDriverError::UnsupportedFeature);
            }

            let mut qtds = Vec::new();
            for i in 0..num_qtds {
                let qtd_ptr = qtd_pool_base.add(i * mem::size_of::<EhciQTD>()) as *mut EhciQTD;
                qtds.push(qtd_ptr);
            }

            let qtd_pool = EhciQtdPool {
                base: qtd_pool_base,
                size: qtd_pool_size,
                qtds,
            };

            self.qtd_pool = Some(qtd_pool);
        }

        log::info!("Queue pools initialized:");
        log::info!("  Queue heads: {}", num_queue_heads);
        log::info!("  qTDs: {}", num_qtds);
        Ok(())
    }

    /// Discover and initialize ports
    pub fn discover_ports(&mut self) -> UsbResult<()> {
        self.ports.clear();

        for port_num in 1..=self.max_ports {
            let port = EhciPort {
                port_number: port_num,
                speed: UsbSpeed::High,
                status: 0,
                connection_status: false,
                device_attached: false,
                power_state: UsbPowerState::Active,
                is_oc_enabled: false,
                is_connected_to_companion: false,
            };

            self.ports.push(port);
        }

        log::info!("Discovered {} EHCI ports", self.ports.len());
        Ok(())
    }

    /// Get port status
    pub fn get_port_status(&mut self, port_number: u8) -> UsbResult<u32> {
        if port_number == 0 || port_number > self.max_ports {
            return Err(UsbDriverError::DeviceNotFound { address: port_number });
        }

        unsafe {
            let portsc_offset = EHCI_PORTSC1 as u64 + ((port_number as u64 - 1) * 0x04);
            let portsc_reg = (self.base_address + portsc_offset) as *const u32;
            let status = *portsc_reg;

            // Update port information
            if port_number as usize <= self.ports.len() {
                let port = &mut self.ports[port_number as usize - 1];
                port.status = status;
                port.connection_status = (status & EHCI_PORT_CONNECT) != 0;
                port.device_attached = (status & EHCI_PORT_CONNECT) != 0;
                port.is_connected_to_companion = (status & EHCI_PORT_POWNER) != 0;
                
                // Determine port speed from status
                let line_status = (status & EHCI_PORT_LS) >> 10;
                port.speed = match line_status {
                    0 => UsbSpeed::Full,  // Full speed
                    1 => UsbSpeed::Low,   // Low speed
                    2 => UsbSpeed::High,  // High speed
                    _ => UsbSpeed::Full,  // Default
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
            let portsc_offset = EHCI_PORTSC1 as u64 + ((port_number as u64 - 1) * 0x04);
            let portsc_reg = (self.base_address + portsc_offset) as *mut u32;
            let mut status = *portsc_reg;

            match feature {
                // USB Port Power Control
                0x0008 => {
                    status |= EHCI_PORT_PPOWER;
                }
                // USB Port Reset
                0x0004 => {
                    status |= EHCI_PORT_FPR;
                }
                // USB Port Suspend
                0x0005 => {
                    status |= EHCI_PORT_SUSP;
                }
                _ => {
                    log::warn!("Unsupported port feature: {:#x}", feature);
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
            let portsc_offset = EHCI_PORTSC1 as u64 + ((port_number as u64 - 1) * 0x04);
            let portsc_reg = (self.base_address + portsc_offset) as *mut u32;
            let mut status = *portsc_reg;

            match feature {
                // USB Port Enable/Disable Change
                0x0004 => {
                    status &= !EHCI_PORT_PEC;
                    status &= !EHCI_PORT_PEDC;
                }
                // USB Port Connection Status Change
                0x0008 => {
                    status &= !EHCI_PORT_OCC;
                }
                // USB Port Power
                0x0008 => {
                    status &= !EHCI_PORT_PPOWER;
                }
                // USB Port Reset Change
                0x0007 => {
                    status &= !EHCI_PORT_RPR;
                }
                _ => {
                    log::warn!("Unsupported port feature: {:#x}", feature);
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
            let cmd_reg = (self.base_address + EHCI_USBCMD as u64) as *mut u32;
            let mut cmd = *cmd_reg;
            
            cmd |= EHCI_CMD_PSE;
            *cmd_reg = cmd;
        }

        log::info!("Periodic schedule enabled");
        Ok(())
    }

    /// Enable asynchronous schedule
    pub fn enable_async_schedule(&mut self) -> UsbResult<()> {
        unsafe {
            let cmd_reg = (self.base_address + EHCI_USBCMD as u64) as *mut u32;
            let mut cmd = *cmd_reg;
            
            cmd |= EHCI_CMD_ASE;
            *cmd_reg = cmd;
        }

        log::info!("Asynchronous schedule enabled");
        Ok(())
    }

    /// Disable periodic schedule
    pub fn disable_periodic_schedule(&mut self) -> UsbResult<()> {
        unsafe {
            let cmd_reg = (self.base_address + EHCI_USBCMD as u64) as *mut u32;
            let mut cmd = *cmd_reg;
            
            cmd &= !EHCI_CMD_PSE;
            *cmd_reg = cmd;
        }

        log::info!("Periodic schedule disabled");
        Ok(())
    }

    /// Disable asynchronous schedule
    pub fn disable_async_schedule(&mut self) -> UsbResult<()> {
        unsafe {
            let cmd_reg = (self.base_address + EHCI_USBCMD as u64) as *mut u32;
            let mut cmd = *cmd_reg;
            
            cmd &= !EHCI_CMD_ASE;
            *cmd_reg = cmd;
        }

        log::info!("Asynchronous schedule disabled");
        Ok(())
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

    /// Get current frame number
    pub fn get_frame_number(&self) -> u32 {
        unsafe {
            let frindex_reg = (self.base_address + EHCI_FRINDEX as u64) as *const u32;
            *frindex_reg
        }
    }

    /// Create a queue head for an endpoint
    pub fn create_queue_head(&mut self, device_address: u8, endpoint_number: u8, 
                           direction: UsbDirection, speed: UsbSpeed) -> UsbResult<*mut EhciQH> {
        let queue_heads = self.queue_head_pool.as_mut().ok_or(UsbDriverError::UnsupportedFeature)?;
        
        unsafe {
            // Find free queue head
            for qh_base in core::iter::Step::new(queue_heads.base, queue_heads.size, mem::size_of::<EhciQH>()) {
                let qh_ptr = qh_base as *mut EhciQH;
                let qh = &mut *qh_ptr;
                
                // Check if queue head is unused (token == 0)
                if qh.token == 0 {
                    // Initialize queue head
                    qh.link = 0x00000001; // Terminate bit
                    qh.ep_capabilities = (endpoint_number as u32) << 8 | (device_address as u32);
                    
                    // Set endpoint characteristics based on speed
                    match speed {
                        UsbSpeed::Low | UsbSpeed::Full => {
                            qh.ep_capabilities |= 1 << 12; // Device is a low/full speed device
                        }
                        UsbSpeed::High => {
                            qh.ep_capabilities |= 2 << 12; // Device is a high speed device
                        }
                        _ => {}
                    }
                    
                    qh.current_qtd = 0;
                    qh.next_qtd = 0x00000001; // Terminate bit
                    qh.altnext_qtd = 0x00000001; // Terminate bit
                    qh.token = 0x40000000; // Device address 0
                    qh.buffer_page0 = 0;
                    qh.buffer_page1 = 0;
                    qh.buffer_page2 = 0;
                    qh.buffer_page3 = 0;
                    qh.buffer_page4 = 0;
                    
                    // Store queue head reference
                    queue_heads.queue_heads.insert(device_address, qh_ptr);
                    
                    return Ok(qh_ptr);
                }
            }
            
            Err(UsbDriverError::UnsupportedFeature)
        }
    }

    /// Get queue head for device
    pub fn get_queue_head(&self, device_address: u8) -> UsbResult<*mut EhciQH> {
        let queue_heads = self.queue_head_pool.as_ref().ok_or(UsbDriverError::UnsupportedFeature)?;
        queue_heads.queue_heads.get(&device_address)
            .copied()
            .ok_or(UsbDriverError::DeviceNotFound { address: device_address })
    }
}

impl Drop for EhciController {
    fn drop(&mut self) {
        unsafe {
            // Clean up allocated memory
            if !self.periodic_frame_list.is_null() {
                let frame_list_size = self.frame_list_size * mem::size_of::<u32>();
                alloc::alloc::dealloc(
                    self.periodic_frame_list as *mut u8,
                    alloc::alloc::Layout::from_size_align(frame_list_size, 4096).unwrap(),
                );
            }

            if !self.async_list_head.is_null() {
                let async_head_size = mem::size_of::<EhciQH>();
                alloc::alloc::dealloc(
                    self.async_list_head as *mut u8,
                    alloc::alloc::Layout::from_size_align(async_head_size, 64).unwrap(),
                );
            }

            if let Some(qh_pool) = &self.queue_head_pool {
                alloc::alloc::dealloc(
                    qh_pool.base as *mut u8,
                    alloc::alloc::Layout::from_size_align(qh_pool.size, 64).unwrap(),
                );
            }

            if let Some(qtd_pool) = &self.qtd_pool {
                alloc::alloc::dealloc(
                    qtd_pool.base as *mut u8,
                    alloc::alloc::Layout::from_size_align(qtd_pool.size, 64).unwrap(),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ehci_controller_creation() {
        let controller = EhciController::new(0x12345678);
        assert_eq!(controller.base_address, 0x12345678);
        assert_eq!(controller.state, EhciControllerState::Uninitialized);
    }

    #[test]
    fn test_ehci_port_creation() {
        let port = EhciPort {
            port_number: 1,
            speed: UsbSpeed::High,
            status: 0,
            connection_status: false,
            device_attached: false,
            power_state: UsbPowerState::Active,
            is_oc_enabled: false,
            is_connected_to_companion: false,
        };

        assert_eq!(port.port_number, 1);
        assert_eq!(port.speed, UsbSpeed::High);
        assert!(!port.connection_status);
    }
}