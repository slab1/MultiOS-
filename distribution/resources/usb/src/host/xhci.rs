//! xHCI (eXtensible Host Controller Interface) Driver
//! 
//! Supports USB 3.0+ hosts with modern features like:
//! - SuperSpeed support
//! - Stream mode
//! - Multiple transfer rings
//! - USB 3.0 features

use core::mem;
use crate::*;

#[cfg(feature = "std")]
use std::collections::BTreeMap;

/// xHCI Registers (Memory Mapped I/O)
const XHCI_BASE_OFFSET: usize = 0x0;
const XHCI_CAP_LENGTH: usize = 0x0;
const XHCI_HCSPARAMS1: usize = 0x4;
const XHCI_HCSPARAMS2: usize = 0x8;
const XHCI_HCSPARAMS3: usize = 0xC;
const XHCI_HCCPARAMS1: usize = 0x10;
const XHCI_HCCPARAMS2: usize = 0x14;
const XHCI_DBOFF: usize = 0x18;
const XHCI_RTORS_BASE: usize = 0x1C;

const XHCI_USBCMD_OFFSET: usize = 0x20;
const XHCI_USBSTS_OFFSET: usize = 0x24;
const XHCI_PAGESIZE_OFFSET: usize = 0x28;
const XHCI_DNCTRL_OFFSET: usize = 0x34;
const XHCI_DCR_OFFSET: usize = 0x40;
const XHCI_IMOD_OFFSET: usize = 0x44;

/// xHCI Operational Register Offsets
const XHCI_MFINDEX_OFFSET: usize = 0x0;
const XHCI_CRCR_OFFSET: usize = 0x4;
const XHCI_DCBAAP_OFFSET: usize = 0x8;
const XHCI_CONFIG_OFFSET: usize = 0x24;

/// xHCI Extended Capability Register Offsets
const XHCI_USBLEGSUP_OFFSET: usize = 0x0;
const XHCI_USBLEGCTLSTS_OFFSET: usize = 0x4;
const XHCI_DOE_OFFSET: usize = 0x8;
const XHCI_SP_OFFSET: usize = 0xC;

/// xHCI Command Register (USBCMD) bit fields
const XHCI_CMD_RS: u32 = 1 << 0;     // Run/Stop
const XHCI_CMD_HCRST: u32 = 1 << 1;  // Host Controller Reset
const XHCI_CMD_INTE: u32 = 1 << 2;   // Interrupter Enable
const XHCI_CMD_HSEE: u32 = 1 << 3;   // Hardware SRE Enable
const XHCI_CMD_LHCRT: u32 = 1 << 4;  // Light HC Reset
const XHCI_CMD_CSS: u32 = 1 << 8;    // Controller Save State
const XHCI_CMD_CRS: u32 = 1 << 9;    // Controller Restore State
const XHCI_CMD_EWE: u32 = 1 << 10;   // Enable Wrap Event
const XHCI_CMD_EU3S: u32 = 1 << 11;  // Enable U3 MFINDEX Stop

/// xHCI Status Register (USBSTS) bit fields
const XHCI_STS_HCH: u32 = 1 << 0;    // Host Controller Halted
const XHCI_STS_HSE: u32 = 1 << 2;    // Host System Error
const XHCI_STS_EINT: u32 = 1 << 0;   // Event Interrupt
const XHCI_STS_PCD: u32 = 1 << 4;    // Port Change Detect
const XHCI_STS_SSS: u32 = 1 << 8;    // Save State Status
const XHCI_STS_RSS: u32 = 1 << 9;    // Restore State Status
const XHCI_STS_SRE: u32 = 1 << 10;   // Save Restore Error
const XHCI_STS_CNR: u32 = 1 << 11;   // Controller Not Ready
const XHCI_STS_HCE: u32 = 1 << 12;   // Host Controller Error

/// xHCI TRB Types
const XHCI_TRB_RESERVED: u32 = 0;
const XHCI_TRB_NORMAL: u32 = 1;
const XHCI_TRB_SETUP: u32 = 2;
const XHCI_TRB_DATA: u32 = 3;
const XHCI_TRB_STATUS: u32 = 4;
const XHCI_TRB_ISOCH: u32 = 5;
const XHCI_TRB_LINK: u32 = 6;
const XHCI_TRB_EVENT_DATA: u7 << 1 = 7;
const XHCI_TRB_NO_OP: u32 = 8;

/// xHCI TRB Completion Codes
const XHCI_CC_SUCCESS: u32 = 1;
const XHCI_CC_DATA_BUFFER_ERROR: u32 = 4;
const XHCI_CC_BABBLE_DETECTED: u32 = 5;
const XHCI_CC_USB_TRANSACTION_ERROR: u32 = 6;
const XHCI_CC_TRB_ERROR: u32 = 7;
const XHCI_CC_STALL_ERROR: u32 = 14;
const XHCI_CC_INVALID_STREAM_TYPE: u32 = 21;
const XHCI_CC_INVALID_EP_STATE: u32 = 22;
const XHCI_CC_RING_OVERRUN: u32 = 27;
const XHCI_CC_RING_STOPPED: u32 = 28;
const XHCI_CC_LINK_LOST: u32 = 29;
const XHCI_CC_ISOCH_BUFFER_OVERRUN: u32 = 31;
const XHCI_CC_ISOCH_BUFFER_UNDERRUN: u32 = 32;

/// xHCI Endpoint Types
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XhciEndpointType {
    NotConfigured = 0,
    IsochronousOut = 1,
    BulkOut = 2,
    InterruptOut = 3,
    Control = 4,
    IsochronousIn = 5,
    BulkIn = 6,
    InterruptIn = 7,
}

/// xHCI Slot Context
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct XhciSlotContext {
    pub route_string: u32,
    pub speed: u8,
    pub mtt: u8,
    pub hub: u8,
    pub context_entries: u8,
    pub max_exit_latency: u16,
    pub root_hub_port_number: u8,
    pub number_of_ports: u8,
    pub parent_hub_slot_id: u8,
    pub parent_hub_port_number: u8,
    pub ttt: u8,
    pub u1_device_exit_latency: u8,
    pub reserved: u8,
    pub u2_device_exit_latency: u16,
    pub slot_state: u8,
    pub reserved2: [u8; 3],
}

/// xHCI Device Context Array Entry
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct XhciDeviceContextArray {
    pub input_control_context: u32,
    pub slot_context: u32,
    pub ep_context_1: u32,
    pub ep_context_2: u32,
    pub ep_context_3: u32,
    pub ep_context_4: u32,
    pub ep_context_5: u32,
    pub ep_context_6: u32,
    pub ep_context_7: u32,
    pub ep_context_8: u32,
    pub ep_context_9: u32,
    pub ep_context_10: u32,
    pub ep_context_11: u32,
    pub ep_context_12: u32,
    pub ep_context_13: u32,
    pub ep_context_14: u32,
    pub ep_context_15: u32,
    pub ep_context_16: u32,
    pub ep_context_17: u32,
    pub ep_context_18: u32,
    pub ep_context_19: u32,
    pub ep_context_20: u32,
    pub ep_context_21: u32,
    pub ep_context_22: u32,
    pub ep_context_23: u32,
    pub ep_context_24: u32,
    pub ep_context_25: u32,
    pub ep_context_26: u32,
    pub ep_context_27: u32,
    pub ep_context_28: u32,
    pub ep_context_29: u32,
    pub ep_context_30: u32,
    pub ep_context_31: u32,
}

/// xHCI Transfer Ring
#[derive(Debug)]
pub struct XhciTransferRing {
    pub base: *mut u8,
    pub enqueue_ptr: *mut u32,
    pub dequeue_ptr: *mut u32,
    pub ring_size: usize,
    pub consumer_cycle_state: bool,
    pub pcs_supported: bool,
}

/// xHCI Event Ring
#[derive(Debug)]
pub struct XhciEventRing {
    pub base: *mut u32,
    pub enqueue_ptr: *mut u32,
    pub dequeue_ptr: *mut u32,
    pub ring_size: usize,
    pub segment_size: usize,
    pub producer_cycle_state: bool,
    pub pcs_supported: bool,
}

/// xHCI Capability Parameters
#[derive(Debug, Clone, Copy)]
pub struct XhciCapabilityParams {
    pub usb_cap_length: u8,
    pub hci_version: u16,
    pub hcs_params1: u32,
    pub hcs_params2: u32,
    pub hcs_params3: u32,
    pub hcc_params1: u32,
    pub hcc_params2: u32,
    pub doorbell_offset: u32,
    pub rtors_base: u32,
    pub hcc_params_ext: u32,
}

/// xHCI Controller State
#[derive(Debug, Clone)]
pub enum XhciControllerState {
    Uninitialized,
    Initialized,
    Running,
    Suspended,
    Error,
}

/// xHCI Doorbell Register
#[derive(Debug)]
pub struct XhciDoorbell {
    pub address: *mut u32,
}

/// xHCI Port Status and Control
#[derive(Debug, Clone, Copy)]
pub struct XhciPortStatus {
    pub portsc: u32,
    pub portpmsc: u32,
    pub portli: u32,
    pub porthlpmc: u32,
}

/// xHCI Command Ring
#[derive(Debug)]
pub struct XhciCommandRing {
    pub base: *mut u32,
    pub enqueue_ptr: *mut u32,
    pub dequeue_ptr: *mut u32,
    pub ring_size: usize,
    pub cycle_state: bool,
}

/// xHCI Port Information
#[derive(Debug)]
pub struct XhciPort {
    pub port_number: u8,
    pub speed: UsbSpeed,
    pub status: XhciPortStatus,
    pub connection_status: bool,
    pub device_attached: bool,
    pub power_state: UsbPowerState,
}

/// xHCI Controller Implementation
pub struct XhciController {
    pub base_address: u64,
    pub capability_params: XhciCapabilityParams,
    pub state: XhciControllerState,
    pub device_context_array: *mut u32,
    pub command_ring: Option<XhciCommandRing>,
    pub event_ring: Option<XhciEventRing>,
    pub doorbell_array: *mut u32,
    pub slots: BTreeMap<u8, XhciSlotContext>,
    pub transfer_rings: BTreeMap<u32, XhciTransferRing>,
    pub ports: Vec<XhciPort>,
    pub interrupt_vector: u8,
    pub max_slots: u8,
    pub max_ports: u8,
    pub max_streams: u8,
    pub max_intr_interrupts: u8,
    pub extended_capabilities_offset: u32,
}

impl XhciController {
    /// Create a new xHCI controller instance
    pub fn new(base_address: u64) -> Self {
        let mut controller = Self {
            base_address,
            capability_params: XhciCapabilityParams {
                usb_cap_length: 0,
                hci_version: 0,
                hcs_params1: 0,
                hcs_params2: 0,
                hcs_params3: 0,
                hcc_params1: 0,
                hcc_params2: 0,
                doorbell_offset: 0,
                rtors_base: 0,
                hcc_params_ext: 0,
            },
            state: XhciControllerState::Uninitialized,
            device_context_array: core::ptr::null_mut(),
            command_ring: None,
            event_ring: None,
            doorbell_array: core::ptr::null_mut(),
            slots: BTreeMap::new(),
            transfer_rings: BTreeMap::new(),
            ports: Vec::new(),
            interrupt_vector: 0,
            max_slots: 0,
            max_ports: 0,
            max_streams: 0,
            max_intr_interrupts: 0,
            extended_capabilities_offset: 0,
        };

        // Read capability parameters
        controller.read_capability_params();

        controller
    }

    /// Read xHCI capability parameters from registers
    pub fn read_capability_params(&mut self) {
        unsafe {
            let cap_base = self.base_address as *const u32;
            
            self.capability_params.usb_cap_length = (*cap_base.offset(XHCI_CAP_LENGTH as isize)) as u8;
            self.capability_params.hci_version = ((*cap_base.offset(XHCI_CAP_LENGTH as isize)) >> 16) as u16;
            self.capability_params.hcs_params1 = *cap_base.offset(XHCI_HCSPARAMS1 as isize);
            self.capability_params.hcs_params2 = *cap_base.offset(XHCI_HCSPARAMS2 as isize);
            self.capability_params.hcs_params3 = *cap_base.offset(XHCI_HCSPARAMS3 as isize);
            self.capability_params.hcc_params1 = *cap_base.offset(XHCI_HCCPARAMS1 as isize);
            self.capability_params.hcc_params2 = *cap_base.offset(XHCI_HCCPARAMS2 as isize);
            self.capability_params.doorbell_offset = *cap_base.offset(XHCI_DBOFF as isize);
            self.capability_params.rtors_base = *cap_base.offset(XHCI_RTORS_BASE as isize);
            self.capability_params.hcc_params_ext = *cap_base.offset(0x20); // Extended capability offset

            // Extract important parameters
            self.max_ports = (self.capability_params.hcs_params1 & 0xFF) as u8;
            self.max_slots = ((self.capability_params.hcs_params1 >> 8) & 0xFF) as u8;
            self.max_streams = ((self.capability_params.hcs_params1 >> 16) & 0x1F) as u8;
            self.max_intr_interrupts = ((self.capability_params.hcs_params1 >> 8) & 0x1FF) as u8;

            // Get extended capabilities offset
            let hcc_params1 = self.capability_params.hcc_params1;
            self.extended_capabilities_offset = ((hcc_params1 & 0xFF) << 2) & 0xFFFFFC;
        }

        log::info!("xHCI Controller initialized:");
        log::info!("  Max Ports: {}", self.max_ports);
        log::info!("  Max Slots: {}", self.max_slots);
        log::info!("  Max Streams: {}", self.max_streams);
        log::info!("  HCI Version: {:#06x}", self.capability_params.hci_version);
    }

    /// Initialize xHCI controller
    pub fn initialize(&mut self) -> UsbResult<()> {
        if self.state == XhciControllerState::Initialized {
            return Ok(());
        }

        // Reset the controller
        self.reset()?;

        // Enable controller
        self.enable()?;

        // Initialize device context array
        self.initialize_device_context_array()?;

        // Initialize command ring
        self.initialize_command_ring()?;

        // Initialize event ring
        self.initialize_event_ring()?;

        // Initialize doorbell array
        self.initialize_doorbell_array()?;

        // Discover ports
        self.discover_ports()?;

        self.state = XhciControllerState::Initialized;
        log::info!("xHCI controller initialized successfully");

        Ok(())
    }

    /// Reset the xHCI controller
    pub fn reset(&mut self) -> UsbResult<()> {
        unsafe {
            let cmd_reg = (self.base_address + 0x20) as *mut u32;
            let status_reg = (self.base_address + 0x24) as *mut u32;

            // Reset the controller
            *cmd_reg = XHCI_CMD_HCRST;
            
            // Wait for reset to complete
            for _ in 0..10000 {
                let status = *status_reg;
                if status & XHCI_STS_CNR == 0 {
                    break;
                }
            }

            // Check if reset was successful
            let status = *status_reg;
            if status & XHCI_STS_CNR != 0 {
                return Err(UsbDriverError::ControllerNotInitialized);
            }

            self.state = XhciControllerState::Uninitialized;
        }

        log::info!("xHCI controller reset completed");
        Ok(())
    }

    /// Enable the xHCI controller
    pub fn enable(&mut self) -> UsbResult<()> {
        unsafe {
            let cmd_reg = (self.base_address + 0x20) as *mut u32;
            let mut cmd = *cmd_reg;
            
            // Set Run/Stop bit
            cmd |= XHCI_CMD_RS;
            *cmd_reg = cmd;

            // Enable interrupts
            cmd |= XHCI_CMD_INTE;
            *cmd_reg = cmd;

            // Wait for controller to start
            for _ in 0..10000 {
                let status = *(self.base_address + 0x24) as *const u32;
                if status & XHCI_STS_HCH == 0 {
                    break;
                }
            }

            let status = *(self.base_address + 0x24) as *const u32;
            if status & XHCI_STS_HCH != 0 {
                return Err(UsbDriverError::ControllerNotInitialized);
            }

            self.state = XhciControllerState::Running;
        }

        log::info!("xHCI controller enabled and running");
        Ok(())
    }

    /// Initialize device context array
    pub fn initialize_device_context_array(&mut self) -> UsbResult<()> {
        let size = (self.max_slots as usize + 1) * mem::size_of::<u32>();
        
        unsafe {
            self.device_context_array = {
                let alloc = alloc::alloc::alloc(alloc::alloc::Layout::from_size_align(size, 64)?);
                if alloc.is_null() {
                    return Err(UsbDriverError::UnsupportedFeature);
                }
                alloc as *mut u32
            };

            // Zero out the device context array
            core::ptr::write_bytes(self.device_context_array, 0, size / mem::size_of::<u32>());
        }

        // Write DCBAAP register
        unsafe {
            let dcbaa_reg = (self.base_address + 0x8) as *mut u64;
            *dcbaa_reg = self.device_context_array as u64;
        }

        log::info!("Device context array initialized at {:#x}", self.device_context_array as u64);
        Ok(())
    }

    /// Initialize command ring
    pub fn initialize_command_ring(&mut self) -> UsbResult<()> {
        let ring_size = 256; // 256 TRBs
        let trb_size = mem::size_of::<u32>();
        let total_size = ring_size * trb_size;

        unsafe {
            let ring_base = alloc::alloc::alloc(alloc::alloc::Layout::from_size_align(total_size, 64)?);
            if ring_base.is_null() {
                return Err(UsbDriverError::UnsupportedFeature);
            }

            let command_ring = XhciCommandRing {
                base: ring_base as *mut u32,
                enqueue_ptr: ring_base as *mut u32,
                dequeue_ptr: ring_base as *mut u32,
                ring_size,
                cycle_state: true,
            };

            self.command_ring = Some(command_ring);

            // Write CRCR register
            let crcr_reg = (self.base_address + 0x4) as *mut u64;
            let mut crcr_value = self.command_ring.as_ref().unwrap().base as u64;
            crcr_value |= 1; // Ring cycle state bit
            *crcr_reg = crcr_value;
        }

        log::info!("Command ring initialized");
        Ok(())
    }

    /// Initialize event ring
    pub fn initialize_event_ring(&mut self) -> UsbResult<()> {
        let ring_size = 256; // 256 event TRBs
        let trb_size = mem::size_of::<u32>();
        let total_size = ring_size * trb_size;

        unsafe {
            let ring_base = alloc::alloc::alloc(alloc::alloc::Layout::from_size_align(total_size, 64)?);
            if ring_base.is_null() {
                return Err(UsbDriverError::UnsupportedFeature);
            }

            let event_ring = XhciEventRing {
                base: ring_base as *mut u32,
                enqueue_ptr: ring_base as *mut u32,
                dequeue_ptr: ring_base as *mut u32,
                ring_size,
                segment_size: 256,
                producer_cycle_state: true,
                pcs_supported: true,
            };

            self.event_ring = Some(event_ring);
        }

        log::info!("Event ring initialized");
        Ok(())
    }

    /// Initialize doorbell array
    pub fn initialize_doorbell_array(&mut self) -> UsbResult<()> {
        self.doorbell_array = unsafe {
            let size = (self.max_slots as usize + 1) * mem::size_of::<u32>();
            let alloc = alloc::alloc::alloc(alloc::alloc::Layout::from_size_align(size, 64)?);
            if alloc.is_null() {
                return Err(UsbDriverError::UnsupportedFeature);
            }
            core::ptr::write_bytes(alloc, 0, size / mem::size_of::<u32>());
            alloc as *mut u32
        };

        log::info!("Doorbell array initialized");
        Ok(())
    }

    /// Discover and initialize ports
    pub fn discover_ports(&mut self) -> UsbResult<()> {
        self.ports.clear();

        for port_num in 1..=self.max_ports {
            let port = XhciPort {
                port_number: port_num,
                speed: UsbSpeed::High,
                status: XhciPortStatus {
                    portsc: 0,
                    portpmsc: 0,
                    portli: 0,
                    porthlpmc: 0,
                },
                connection_status: false,
                device_attached: false,
                power_state: UsbPowerState::Active,
            };

            self.ports.push(port);
        }

        log::info!("Discovered {} xHCI ports", self.ports.len());
        Ok(())
    }

    /// Get port status
    pub fn get_port_status(&mut self, port_number: u8) -> UsbResult<XhciPortStatus> {
        if port_number == 0 || port_number > self.max_ports {
            return Err(UsbDriverError::DeviceNotFound { address: port_number });
        }

        unsafe {
            let portsc_reg = (self.base_address + 0x400 + ((port_number as usize - 1) * 0x10)) as *const u32;
            let portpmsc_reg = (self.base_address + 0x404 + ((port_number as usize - 1) * 0x10)) as *const u32;
            let portli_reg = (self.base_address + 0x408 + ((port_number as usize - 1) * 0x10)) as *const u32;
            let porthlpmc_reg = (self.base_address + 0x40C + ((port_number as usize - 1) * 0x10)) as *const u32;

            let status = XhciPortStatus {
                portsc: *portsc_reg,
                portpmsc: *portpmsc_reg,
                portli: *portli_reg,
                porthlpmc: *porthlpmc_reg,
            };

            // Update port information
            if port_number as usize <= self.ports.len() {
                let port = &mut self.ports[port_number as usize - 1];
                port.status = status;
                port.connection_status = (status.portsc & 1) != 0;
                port.device_attached = (status.portsc & (1 << 9)) != 0;
            }

            Ok(status)
        }
    }

    /// Ring the doorbell for a device slot
    pub fn ring_doorbell(&mut self, slot_id: u8, target: u32) -> UsbResult<()> {
        unsafe {
            if slot_id as usize > self.max_slots as usize {
                return Err(UsbDriverError::DeviceNotFound { address: slot_id });
            }

            let doorbell_offset = self.capability_params.doorbell_offset as usize;
            let doorbell_addr = self.base_address as *mut u32;
            let doorbell_ptr = doorbell_addr.add(doorbell_offset / mem::size_of::<u32>() + slot_id as usize);
            
            *doorbell_ptr = target;
        }

        Ok(())
    }

    /// Process event ring
    pub fn process_events(&mut self) -> UsbResult<Vec<UsbEvent>> {
        let mut events = Vec::new();

        // Implementation would process event ring and generate USB events
        // For now, return empty vector

        Ok(events)
    }

    /// Enable slot (initialize device context)
    pub fn enable_slot(&mut self, slot_id: u8) -> UsbResult<()> {
        if slot_id as usize > self.max_slots as usize {
            return Err(UsbDriverError::DeviceNotFound { address: slot_id });
        }

        self.ring_doorbell(slot_id, 1)?; // Target 1 = Enable Slot
        Ok(())
    }

    /// Disable slot
    pub fn disable_slot(&mut self, slot_id: u8) -> UsbResult<()> {
        if slot_id as usize > self.max_slots as usize {
            return Err(UsbDriverError::DeviceNotFound { address: slot_id });
        }

        self.ring_doorbell(slot_id, 2)?; // Target 2 = Disable Slot
        Ok(())
    }

    /// Configure endpoint
    pub fn configure_endpoint(&mut self, slot_id: u8, endpoint_id: u8) -> UsbResult<()> {
        if slot_id as usize > self.max_slots as usize {
            return Err(UsbDriverError::DeviceNotFound { address: slot_id });
        }

        self.ring_doorbell(slot_id, endpoint_id + 1)?;
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
}

impl Drop for XhciController {
    fn drop(&mut self) {
        unsafe {
            // Clean up allocated memory
            if !self.device_context_array.is_null() {
                let size = (self.max_slots as usize + 1) * mem::size_of::<u32>();
                alloc::alloc::dealloc(
                    self.device_context_array as *mut u8,
                    alloc::alloc::Layout::from_size_align(size, 64).unwrap(),
                );
            }

            if let Some(command_ring) = &self.command_ring {
                let size = command_ring.ring_size * mem::size_of::<u32>();
                alloc::alloc::dealloc(
                    command_ring.base as *mut u8,
                    alloc::alloc::Layout::from_size_align(size, 64).unwrap(),
                );
            }

            if let Some(event_ring) = &self.event_ring {
                let size = event_ring.ring_size * mem::size_of::<u32>();
                alloc::alloc::dealloc(
                    event_ring.base as *mut u8,
                    alloc::alloc::Layout::from_size_align(size, 64).unwrap(),
                );
            }

            if !self.doorbell_array.is_null() {
                let size = (self.max_slots as usize + 1) * mem::size_of::<u32>();
                alloc::alloc::dealloc(
                    self.doorbell_array as *mut u8,
                    alloc::alloc::Layout::from_size_align(size, 64).unwrap(),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xhci_controller_creation() {
        let controller = XhciController::new(0x12345678);
        assert_eq!(controller.base_address, 0x12345678);
        assert_eq!(controller.state, XhciControllerState::Uninitialized);
    }

    #[test]
    fn test_xhci_port_creation() {
        let port = XhciPort {
            port_number: 1,
            speed: UsbSpeed::Super,
            status: XhciPortStatus {
                portsc: 0,
                portpmsc: 0,
                portli: 0,
                porthlpmc: 0,
            },
            connection_status: false,
            device_attached: false,
            power_state: UsbPowerState::Active,
        };

        assert_eq!(port.port_number, 1);
        assert_eq!(port.speed, UsbSpeed::Super);
        assert!(!port.connection_status);
    }
}