//! PCI/PCIe Device Enumeration
//! 
//! Provides comprehensive PCI and PCIe device detection, enumeration,
//! and configuration space access

use crate::log::{info, warn, error};
use crate::KernelError;

use super::{PciDeviceInfo, FirmwareType};

/// PCI Configuration Space Access Methods
const PCI_CONFIG_ACCESS_MECHANISM1: u8 = 1;
const PCI_CONFIG_ACCESS_MECHANISM2: u8 = 2;

/// PCI Configuration Space Addresses
const PCI_CONFIG_SPACE_SIZE: usize = 256;
const PCI_EXTENDED_CONFIG_SPACE_SIZE: usize = 4096;

/// PCI Command Register
const PCI_COMMAND_IO_SPACE: u16 = 0x0001;
const PCI_COMMAND_MEMORY_SPACE: u16 = 0x0002;
const PCI_COMMAND_BUS_MASTER: u16 = 0x0004;
const PCI_COMMAND_SPECIAL_CYCLES: u16 = 0x0008;
const PCI_COMMAND_WRITE_INVALIDATE: u16 = 0x0010;
const PCI_COMMAND_VGA_PALETTE: u16 = 0x0020;
const PCI_COMMAND_PARITY_ERROR: u16 = 0x0040;
const PCI_COMMAND_SERR: u16 = 0x0100;
const PCI_COMMAND_FAST_BACK_TO_BACK: u16 = 0x0200;
const PCI_COMMAND_INTERRUPT_DISABLE: u16 = 0x0400;

/// PCI Status Register
const PCI_STATUS_INTERRUPT_STATUS: u16 = 0x0008;
const PCI_STATUS_CAPABILITY_LIST: u16 = 0x0010;
const PCI_STATUS_66MHZ: u16 = 0x0020;
const PCI_STATUS_RESERVED: u16 = 0x0040;
const PCI_STATUS_FAST_BACK_TO_BACK: u16 = 0x0080;
const PCI_STATUS_DATA_PARITY_ERROR: u16 = 0x0100;
const PCI_STATUS_DEVSEL_TIMING_MASK: u16 = 0x0600;
const PCI_STATUS_SIGNALED_TARGET_ABORT: u16 = 0x0800;
const PCI_STATUS_RECEIVED_TARGET_ABORT: u16 = 0x1000;
const PCI_STATUS_RECEIVED_MASTER_ABORT: u16 = 0x2000;
const PCI_STATUS_SIGNALED_SYSTEM_ERROR: u16 = 0x4000;
const PCI_STATUS_DETECTED_PARITY_ERROR: u16 = 0x8000;

/// PCI Class Code Definitions
const PCI_CLASS_UNCLASSIFIED: u8 = 0x00;
const PCI_CLASS_MASS_STORAGE: u8 = 0x01;
const PCI_CLASS_NETWORK: u8 = 0x02;
const PCI_CLASS_DISPLAY: u8 = 0x03;
const PCI_CLASS_MULTIMEDIA: u8 = 0x04;
const PCI_CLASS_MEMORY: u8 = 0x05;
const PCI_CLASS_BRIDGE: u8 = 0x06;
const PCI_CLASS_COMMUNICATION: u8 = 0x07;
const PCI_CLASS_GENERAL_PERIPHERALS: u8 = 0x08;
const PCI_CLASS_INPUT_DEVICE: u8 = 0x09;
const PCI_CLASS_DOCKING_STATION: u8 = 0x0A;
const PCI_CLASS_PROCESSOR: u8 = 0x0B;
const PCI_CLASS_SERIAL_BUS: u8 = 0x0C;
const PCI_CLASS_WIRELESS_CONTROLLER: u8 = 0x0D;
const PCI_CLASS_INTELLIGENT_IO: u8 = 0x0E;
const PCI_CLASS_SATELLITE_COMMUNICATION: u8 = 0x0F;
const PCI_CLASS_ENCRYPTION_DECRYPTION: u8 = 0x10;
const PCI_CLASS_DATA_ACQUISITION: u8 = 0x11;

/// Common PCI Vendor IDs
const PCI_VENDOR_INTEL: u16 = 0x8086;
const PCI_VENDOR_AMD: u16 = 0x1002;
const PCI_VENDOR_NVIDIA: u16 = 0x10DE;
const PCI_VENDOR_VMWARE: u16 = 0x15AD;

/// PCI Configuration Space Register Offsets
const PCI_VENDOR_ID: u16 = 0x00;
const PCI_DEVICE_ID: u16 = 0x02;
const PCI_COMMAND: u16 = 0x04;
const PCI_STATUS: u16 = 0x06;
const PCI_REVISION_ID: u16 = 0x08;
const PCI_PROG_IF: u16 = 0x09;
const PCI_SUBCLASS_CODE: u16 = 0x0A;
const PCI_CLASS_CODE: u16 = 0x0B;
const PCI_CACHE_LINE_SIZE: u16 = 0x0C;
const PCI_LATENCY_TIMER: u16 = 0x0D;
const PCI_HEADER_TYPE: u16 = 0x0E;
const PCI_BIST: u16 = 0x0F;
const PCI_BASE_ADDRESS_0: u16 = 0x10;
const PCI_BASE_ADDRESS_1: u16 = 0x14;
const PCI_BASE_ADDRESS_2: u16 = 0x18;
const PCI_BASE_ADDRESS_3: u16 = 0x1C;
const PCI_BASE_ADDRESS_4: u16 = 0x20;
const PCI_BASE_ADDRESS_5: u16 = 0x24;
const PCI_CARDBUS_CIS: u16 = 0x28;
const PCI_SUBSYSTEM_VENDOR_ID: u16 = 0x2C;
const PCI_SUBSYSTEM_ID: u16 = 0x2E;
const PCI_ROM_ADDRESS: u16 = 0x30;
const PCI_CAPABILITIES_POINTER: u16 = 0x34;
const PCI_INTERRUPT_LINE: u16 = 0x3C;
const PCI_INTERRUPT_PIN: u16 = 0x3D;
const PCI_MIN_GRANT: u16 = 0x3E;
const PCI_MAX_LATENCY: u16 = 0x3F;

/// PCI Express Configuration Space Extended Registers
const PCI_EXPRESS_CAPABILITY: u16 = 0x100;
const PCI_EXPRESS_DEVICE_CAPABILITIES: u16 = 0x104;
const PCI_EXPRESS_DEVICE_CONTROL: u16 = 0x108;
const PCI_EXPRESS_DEVICE_STATUS: u16 = 0x10A;
const PCI_EXPRESS_LINK_CAPABILITIES: u16 = 0x10C;
const PCI_EXPRESS_LINK_CONTROL: u16 = 0x110;
const PCI_EXPRESS_LINK_STATUS: u16 = 0x112;

/// PCI Capability IDs
const PCI_CAPABILITY_ID_PM: u8 = 0x01;
const PCI_CAPABILITY_ID_MSI: u8 = 0x05;
const PCI_CAPABILITY_ID_VPD: u8 = 0x03;
const PCI_CAPABILITY_ID_VENDOR: u8 = 0x09;
const PCI_CAPABILITY_ID_PCIX: u8 = 0x07;
const PCI_CAPABILITY_ID_MSIX: u8 = 0x11;
const PCI_CAPABILITY_ID_SATA: u8 = 0x12;

/// PCI Bus Information
#[derive(Debug, Clone)]
pub struct PciBusInfo {
    pub bus_number: u8,
    pub secondary_bus: u8,
    pub subordinate_bus: u8,
    pub bridge_control: u16,
    pub primary_bus: u8,
    pub io_base: u16,
    pub io_limit: u16,
    pub memory_base: u32,
    pub memory_limit: u32,
    pub prefetchable_memory_base: u32,
    pub prefetchable_memory_limit: u32,
    pub io_base_upper16: u16,
    pub io_limit_upper16: u16,
}

/// PCI Device Capability Information
#[derive(Debug, Clone)]
pub struct PciCapabilityInfo {
    pub capability_id: u8,
    pub capability_offset: u16,
    pub size: u16,
    pub data: Vec<u8>,
}

/// PCI Express Device Information
#[derive(Debug, Clone)]
pub struct PciExpressInfo {
    pub supported_link_speeds: Vec<u32>, // In GT/s
    pub max_link_width: u32,
    pub current_link_speed: u32,
    pub current_link_width: u32,
    pub max_payload_size: u32,
    pub max_read_request_size: u32,
    pub device_port_type: u32,
    pub slot_implemented: bool,
    pub attention_indicator_present: bool,
    pub power_indicator_present: bool,
    pub attention_button_present: bool,
    pub command_completed_support: bool,
    pub device_serial_number_supported: bool,
}

/// PCI Interrupt Information
#[derive(Debug, Clone)]
pub struct PciInterruptInfo {
    pub interrupt_line: u8,
    pub interrupt_pin: u8,
    pub supports_msi: bool,
    pub supports_msix: bool,
    pub msi_count: u32,
    pub msix_count: u32,
}

/// PCI Device Resources
#[derive(Debug, Clone)]
pub struct PciResourceInfo {
    pub base_addresses: Vec<PciBarInfo>,
    pub expansion_rom_base: Option<u64>,
    pub expansion_rom_size: u32,
    pub io_space_used: u32,
    pub memory_space_used: u32,
    pub prefetchable_memory_used: u32,
}

/// PCI Base Address Register Information
#[derive(Debug, Clone)]
pub struct PciBarInfo {
    pub index: u8,
    pub address: u64,
    pub size: u64,
    pub is_io: bool,
    pub is_prefetchable: bool,
    pub is_64bit: bool,
    pub is_enabled: bool,
}

/// PCI Manager
pub struct PciManager {
    pub initialized: bool,
    pub config_access_method: u8,
    pub config_address_port: u16,
    pub config_data_port: u16,
    pub buses: Vec<PciBusInfo>,
    pub devices: Vec<PciDeviceInfo>,
    pub device_capabilities: Vec<PciCapabilityInfo>,
    pub pci_express_devices: Vec<PciExpressInfo>,
    pub interrupt_info: Vec<PciInterruptInfo>,
    pub resource_info: Vec<PciResourceInfo>,
    pub scan_complete: bool,
}

impl PciManager {
    /// Create new PCI manager
    pub fn new() -> Self {
        Self {
            initialized: false,
            config_access_method: PCI_CONFIG_ACCESS_MECHANISM1,
            config_address_port: 0xCF8,
            config_data_port: 0xCFC,
            buses: Vec::new(),
            devices: Vec::new(),
            device_capabilities: Vec::new(),
            pci_express_devices: Vec::new(),
            interrupt_info: Vec::new(),
            resource_info: Vec::new(),
            scan_complete: false,
        }
    }
    
    /// Initialize PCI subsystem
    pub fn initialize(&mut self) -> Result<(), KernelError> {
        info!("Initializing PCI/PCIe subsystem...");
        
        // Step 1: Detect configuration access method
        self.detect_config_access_method()?;
        
        // Step 2: Scan PCI buses
        self.scan_pci_buses()?;
        
        // Step 3: Enumerate all devices
        self.enumerate_devices()?;
        
        // Step 4: Parse device capabilities
        self.parse_device_capabilities()?;
        
        // Step 5: Check for PCI Express support
        self.detect_pci_express()?;
        
        // Step 6: Configure resources
        self.configure_resources()?;
        
        self.initialized = true;
        self.scan_complete = true;
        
        info!("PCI/PCIe initialization complete. Found {} devices", self.devices.len());
        
        Ok(())
    }
    
    /// Detect PCI configuration access method
    fn detect_config_access_method(&mut self) -> Result<(), KernelError> {
        // Try mechanism 1 (most common)
        let test_device = self.make_config_address(0, 0, 0, PCI_VENDOR_ID);
        let original_value = self.read_config_u32(test_device);
        
        // Write test value
        self.write_config_u32(test_device, 0x12345678);
        let read_value = self.read_config_u32(test_device);
        
        // Restore original value
        self.write_config_u32(test_device, original_value);
        
        if read_value == 0x12345678 {
            self.config_access_method = PCI_CONFIG_ACCESS_MECHANISM1;
            info!("Using PCI configuration access mechanism 1");
        } else {
            self.config_access_method = PCI_CONFIG_ACCESS_MECHANISM2;
            info!("Using PCI configuration access mechanism 2");
        }
        
        Ok(())
    }
    
    /// Scan PCI buses
    fn scan_pci_buses(&mut self) -> Result<(), KernelError> {
        info!("Scanning PCI buses...");
        
        // Start with bus 0
        self.scan_bus(0)?;
        
        // Check for additional buses
        self.check_for_additional_buses()?;
        
        info!("Found {} PCI buses", self.buses.len() + 1); // +1 for bus 0
        Ok(())
    }
    
    /// Scan specific PCI bus
    fn scan_bus(&mut self, bus_number: u8) -> Result<(), KernelError> {
        info!("Scanning PCI bus {}", bus_number);
        
        // Scan all device slots (0-31) and functions (0-7)
        for device_number in 0..32 {
            for function_number in 0..8 {
                if function_number == 0 || (function_number > 0 && self.is_multi_function_device(bus_number, device_number)?) {
                    if self.device_exists(bus_number, device_number, function_number)? {
                        // Device found, record it
                        self.record_device(bus_number, device_number, function_number)?;
                        
                        // Check if this is a PCI-to-PCI bridge
                        if self.is_pci_bridge(bus_number, device_number, function_number)? {
                            self.handle_pci_bridge(bus_number, device_number, function_number)?;
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Check for additional buses
    fn check_for_additional_buses(&mut self) -> Result<(), KernelError> {
        // This would recursively scan buses discovered through bridges
        info!("Additional bus scanning not implemented");
        Ok(())
    }
    
    /// Enumerate all PCI devices
    fn enumerate_devices(&mut self) -> Result<(), KernelError> {
        info!("Enumerating PCI devices...");
        
        for bus in 0..=255 {
            for device in 0..32 {
                for function in 0..8 {
                    if function == 0 || (function > 0 && self.is_multi_function_device(bus, device)?) {
                        if self.device_exists(bus, device, function)? {
                            let device_info = self.get_device_info(bus, device, function)?;
                            self.devices.push(device_info);
                            
                            info!("Found PCI device: {}:{}:{} - {} (Vendor: 0x{:04X}, Device: 0x{:04X})",
                                  bus, device, function, self.get_device_class_name(device_info.class_code, device_info.subclass),
                                  device_info.vendor_id, device_info.device_id);
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Check if device exists
    fn device_exists(&self, bus: u8, device: u8, function: u8) -> Result<bool, KernelError> {
        let address = self.make_config_address(bus, device, function, PCI_VENDOR_ID);
        let vendor_id = self.read_config_u16(address);
        
        Ok(vendor_id != 0xFFFF && vendor_id != 0x0000)
    }
    
    /// Check if device is multi-function
    fn is_multi_function_device(&self, bus: u8, device: u8) -> Result<bool, KernelError> {
        let address = self.make_config_address(bus, device, 0, PCI_HEADER_TYPE);
        let header_type = self.read_config_u8(address);
        
        Ok((header_type & 0x80) != 0)
    }
    
    /// Check if device is a PCI-to-PCI bridge
    fn is_pci_bridge(&self, bus: u8, device: u8, function: u8) -> Result<bool, KernelError> {
        let address = self.make_config_address(bus, device, function, PCI_CLASS_CODE);
        let class_code = self.read_config_u8(address);
        let subclass = self.read_config_u8(address + 1);
        
        Ok(class_code == PCI_CLASS_BRIDGE && subclass == 0x04) // PCI-to-PCI bridge
    }
    
    /// Handle PCI bridge device
    fn handle_pci_bridge(&mut self, bus: u8, device: u8, function: u8) -> Result<(), KernelError> {
        let address = self.make_config_address(bus, device, function, 0x18); // Primary/Secondary bus registers
        let secondary_bus = self.read_config_u8(address + 1);
        
        info!("Found PCI bridge: {}:{}:{} - Secondary bus: {}", bus, device, function, secondary_bus);
        
        // Recursively scan the secondary bus
        self.scan_bus(secondary_bus)?;
        
        Ok(())
    }
    
    /// Record device information
    fn record_device(&mut self, bus: u8, device: u8, function: u8) -> Result<(), KernelError> {
        // This is handled by get_device_info during enumeration
        Ok(())
    }
    
    /// Get detailed device information
    fn get_device_info(&self, bus: u8, device: u8, function: u8) -> Result<PciDeviceInfo, KernelError> {
        let vendor_addr = self.make_config_address(bus, device, function, PCI_VENDOR_ID);
        let device_addr = self.make_config_address(bus, device, function, PCI_DEVICE_ID);
        let class_addr = self.make_config_address(bus, device, function, PCI_CLASS_CODE);
        
        let vendor_id = self.read_config_u16(vendor_addr);
        let device_id = self.read_config_u16(device_addr);
        let class_code = self.read_config_u8(class_addr);
        let subclass = self.read_config_u8(class_addr + 1);
        let prog_if = self.read_config_u8(class_addr + 2);
        
        let device_name = self.get_device_name(vendor_id, device_id, class_code, subclass);
        let driver_attached = false; // Would be determined by driver manager
        
        Ok(PciDeviceInfo {
            bus,
            device,
            function,
            vendor_id,
            device_id,
            class_code,
            subclass,
            prog_if,
            device_name,
            driver_attached,
        })
    }
    
    /// Get device name
    fn get_device_name(&self, vendor_id: u16, device_id: u16, class_code: u8, subclass: u8) -> String {
        match (vendor_id, device_id) {
            (PCI_VENDOR_INTEL, 0x1237) => "Intel 440FX (Triton II)".to_string(),
            (PCI_VENDOR_INTEL, 0x7000) => "Intel PIIX3 ISA Bridge".to_string(),
            (PCI_VENDOR_INTEL, 0x7010) => "Intel PIIX3 IDE Controller".to_string(),
            (PCI_VENDOR_INTEL, 0x7113) => "Intel PIIX4 ACPI".to_string(),
            (PCI_VENDOR_INTEL, 0x100E) => "Intel PRO/1000 MT Desktop Adapter".to_string(),
            (PCI_VENDOR_AMD, 0x4378) => "AMD/ATI Rage 128".to_string(),
            (PCI_VENDOR_NVIDIA, 0x0020) => "NVIDIA RIVA 128".to_string(),
            _ => {
                let class_name = self.get_device_class_name(class_code, subclass);
                format!("{} Device (0x{:04X}:0x{:04X})", class_name, vendor_id, device_id)
            }
        }
    }
    
    /// Get device class name
    fn get_device_class_name(&self, class_code: u8, subclass: u8) -> String {
        match class_code {
            PCI_CLASS_UNCLASSIFIED => "Unclassified Device".to_string(),
            PCI_CLASS_MASS_STORAGE => match subclass {
                0x00 => "SCSI Bus Controller".to_string(),
                0x01 => "IDE Controller".to_string(),
                0x02 => "Floppy Disk Controller".to_string(),
                0x03 => "IPI Bus Controller".to_string(),
                0x04 => "RAID Controller".to_string(),
                0x05 => "ATA Controller".to_string(),
                0x06 => "Serial ATA".to_string(),
                0x07 => "Serial Attached SCSI".to_string(),
                0x08 => "Non-Volatile Memory".to_string(),
                _ => "Mass Storage Controller".to_string(),
            },
            PCI_CLASS_NETWORK => match subclass {
                0x00 => "Ethernet Controller".to_string(),
                0x01 => "Token Ring Controller".to_string(),
                0x02 => "FDDI Controller".to_string(),
                0x03 => "ATM Controller".to_string(),
                0x04 => "ISDN Controller".to_string(),
                0x05 => "WorldFip Controller".to_string(),
                0x06 => "PICMG Controller".to_string(),
                0x07 => "Infiniband Controller".to_string(),
                0x08 => "Fabric Controller".to_string(),
                _ => "Network Controller".to_string(),
            },
            PCI_CLASS_DISPLAY => match subclass {
                0x00 => "VGA Compatible Controller".to_string(),
                0x01 => "8514 Compatible Controller".to_string(),
                _ => "Display Controller".to_string(),
            },
            PCI_CLASS_MULTIMEDIA => "Multimedia Controller".to_string(),
            PCI_CLASS_MEMORY => "Memory Controller".to_string(),
            PCI_CLASS_BRIDGE => match subclass {
                0x00 => "Host Bridge".to_string(),
                0x01 => "ISA Bridge".to_string(),
                0x02 => "EISA Bridge".to_string(),
                0x03 => "MCA Bridge".to_string(),
                0x04 => "PCI-to-PCI Bridge".to_string(),
                0x05 => "PCMCIA Bridge".to_string(),
                0x06 => "NuBus Bridge".to_string(),
                0x07 => "CardBus Bridge".to_string(),
                0x08 => "RACEway Bridge".to_string(),
                0x09 => "PCI-to-PCI Bridge".to_string(),
                0x0A => "InfiniBand to PCI Host Bridge".to_string(),
                _ => "Bridge Device".to_string(),
            },
            PCI_CLASS_COMMUNICATION => "Communication Controller".to_string(),
            PCI_CLASS_GENERAL_PERIPHERALS => "General Peripheral".to_string(),
            PCI_CLASS_INPUT_DEVICE => "Input Device".to_string(),
            PCI_CLASS_DOCKING_STATION => "Docking Station".to_string(),
            PCI_CLASS_PROCESSOR => "Processor".to_string(),
            PCI_CLASS_SERIAL_BUS => match subclass {
                0x00 => "FireWire (IEEE 1394)".to_string(),
                0x01 => "ACCESS.bus".to_string(),
                0x02 => "SSA".to_string(),
                0x03 => "USB Controller".to_string(),
                0x04 => "Fibre Channel".to_string(),
                0x05 => "SMBus".to_string(),
                0x06 => "InfiniBand".to_string(),
                0x07 => "IPMI Interface".to_string(),
                0x08 => "SERCOS Interface".to_string(),
                0x09 => "CANbus".to_string(),
                _ => "Serial Bus Controller".to_string(),
            },
            PCI_CLASS_WIRELESS_CONTROLLER => "Wireless Controller".to_string(),
            _ => "Unknown Device".to_string(),
        }
    }
    
    /// Parse device capabilities
    fn parse_device_capabilities(&mut self) -> Result<(), KernelError> {
        info!("Parsing PCI device capabilities...");
        
        for device in &self.devices {
            let capabilities = self.parse_device_capability_list(
                device.bus, device.device, device.function
            )?;
            
            if !capabilities.is_empty() {
                self.device_capabilities.extend(capabilities);
                info!("Device {}:{}:{} has {} capabilities", 
                      device.bus, device.device, device.function, capabilities.len());
            }
        }
        
        Ok(())
    }
    
    /// Parse capability list for a device
    fn parse_device_capability_list(&self, bus: u8, device: u8, function: u8) 
        -> Result<Vec<PciCapabilityInfo>, KernelError> {
        let mut capabilities = Vec::new();
        
        // Check if device has capability list
        let status_addr = self.make_config_address(bus, device, function, PCI_STATUS);
        let status = self.read_config_u16(status_addr);
        
        if (status & PCI_STATUS_CAPABILITY_LIST) == 0 {
            return Ok(capabilities);
        }
        
        // Get capability list pointer
        let cap_ptr_addr = self.make_config_address(bus, device, function, PCI_CAPABILITIES_POINTER);
        let mut cap_ptr = self.read_config_u8(cap_ptr_addr) as u16;
        
        while cap_ptr != 0 {
            // Read capability ID
            let cap_id = self.read_config_u8(self.make_config_address(bus, device, function, cap_ptr));
            
            // Read capability data (simplified - would read actual data)
            let cap_info = PciCapabilityInfo {
                capability_id: cap_id,
                capability_offset: cap_ptr,
                size: 4, // Would calculate actual size
                data: Vec::new(), // Would read actual data
            };
            
            capabilities.push(cap_info);
            
            // Get next capability pointer
            cap_ptr = self.read_config_u8(self.make_config_address(bus, device, function, cap_ptr + 1)) as u16;
            
            // Safety check to prevent infinite loop
            if cap_ptr < 0x40 || cap_ptr > 0x1000 {
                break;
            }
        }
        
        Ok(capabilities)
    }
    
    /// Detect PCI Express support
    fn detect_pci_express(&mut self) -> Result<(), KernelError> {
        info!("Detecting PCI Express devices...");
        
        for device in &self.devices {
            if device.class_code == PCI_CLASS_BRIDGE && device.subclass == 0x04 {
                // This is a PCI-to-PCI bridge, might be PCIe
                let exp_cap = self.read_pci_express_capability(device.bus, device.device, device.function);
                
                if exp_cap.is_some() {
                    let (speed, width) = self.get_pci_express_link_info(device.bus, device.device, device.function)?;
                    
                    let pci_express_info = PciExpressInfo {
                        supported_link_speeds: vec![speed],
                        max_link_width: width,
                        current_link_speed: speed,
                        current_link_width: width,
                        max_payload_size: 128, // Default
                        max_read_request_size: 512, // Default
                        device_port_type: 0,
                        slot_implemented: false,
                        attention_indicator_present: false,
                        power_indicator_present: false,
                        attention_button_present: false,
                        command_completed_support: false,
                        device_serial_number_supported: false,
                    };
                    
                    self.pci_express_devices.push(pci_express_info);
                }
            }
        }
        
        Ok(())
    }
    
    /// Read PCI Express capability register
    fn read_pci_express_capability(&self, bus: u8, device: u8, function: u8) -> Option<u32> {
        let mut cap_ptr = self.read_config_u8(self.make_config_address(bus, device, function, PCI_CAPABILITIES_POINTER)) as u16;
        
        while cap_ptr != 0 {
            let cap_id = self.read_config_u8(self.make_config_address(bus, device, function, cap_ptr));
            
            if cap_id == 0x10 { // PCI Express capability ID
                let exp_cap_addr = self.make_config_address(bus, device, function, cap_ptr + PCI_EXPRESS_CAPABILITY);
                return Some(self.read_config_u32(exp_cap_addr));
            }
            
            cap_ptr = self.read_config_u8(self.make_config_address(bus, device, function, cap_ptr + 1)) as u16;
        }
        
        None
    }
    
    /// Get PCI Express link information
    fn get_pci_express_link_info(&self, bus: u8, device: u8, function: u8) -> Result<(u32, u32), KernelError> {
        // Read link capabilities and status
        let link_cap_addr = self.make_config_address(bus, device, function, PCI_EXPRESS_LINK_CAPABILITIES);
        let link_status_addr = self.make_config_address(bus, device, function, PCI_EXPRESS_LINK_STATUS);
        
        let link_cap = self.read_config_u32(link_cap_addr);
        let link_status = self.read_config_u32(link_status_addr);
        
        let max_speed = (link_cap & 0xF) as u32; // Link Speed Vector
        let max_width = ((link_cap >> 4) & 0x3F) as u32; // Link Width Vector
        
        let current_speed = (link_status & 0xF) as u32;
        let current_width = ((link_status >> 4) & 0x3F) as u32;
        
        Ok((current_speed, current_width))
    }
    
    /// Configure device resources
    fn configure_resources(&mut self) -> Result<(), KernelError> {
        info!("Configuring PCI device resources...");
        
        for device in &self.devices {
            let resources = self.get_device_resources(device.bus, device.device, device.function)?;
            self.resource_info.push(resources);
        }
        
        Ok(())
    }
    
    /// Get device resource information
    fn get_device_resources(&self, bus: u8, device: u8, function: u8) -> Result<PciResourceInfo, KernelError> {
        let mut base_addresses = Vec::new();
        let mut expansion_rom_base = None;
        
        // Parse base address registers (BARs 0-5)
        for i in 0..6 {
            let bar_addr = self.make_config_address(bus, device, function, PCI_BASE_ADDRESS_0 + (i as u16 * 4));
            let bar_value = self.read_config_u32(bar_addr);
            
            let bar_info = self.parse_base_address_register(bar_value, i)?;
            base_addresses.push(bar_info);
        }
        
        // Parse expansion ROM (if present)
        let rom_addr = self.make_config_address(bus, device, function, PCI_ROM_ADDRESS);
        let rom_value = self.read_config_u32(rom_addr);
        
        if (rom_value & 0x00000001) != 0 {
            expansion_rom_base = Some((rom_value & 0xFFFFF800) as u64);
        }
        
        Ok(PciResourceInfo {
            base_addresses,
            expansion_rom_base,
            expansion_rom_size: 0, // Would calculate
            io_space_used: 0,
            memory_space_used: 0,
            prefetchable_memory_used: 0,
        })
    }
    
    /// Parse base address register
    fn parse_base_address_register(&self, value: u32, index: u8) -> Result<PciBarInfo, KernelError> {
        if (value & 0x00000001) != 0 {
            // I/O Space
            Ok(PciBarInfo {
                index,
                address: (value & 0xFFFFFFF0) as u64,
                size: 0, // Would calculate size
                is_io: true,
                is_prefetchable: false,
                is_64bit: false,
                is_enabled: (value & 0x00000002) != 0,
            })
        } else {
            // Memory Space
            let is_prefetchable = (value & 0x00000008) != 0;
            let bar_type = (value & 0x00000006) >> 1;
            let is_64bit = bar_type == 0b10;
            
            Ok(PciBarInfo {
                index,
                address: (value & 0xFFFFFFF0) as u64,
                size: 0, // Would calculate size
                is_io: false,
                is_prefetchable,
                is_64bit,
                is_enabled: true,
            })
        }
    }
    
    /// Make configuration address
    fn make_config_address(&self, bus: u8, device: u8, function: u8, offset: u16) -> u32 {
        match self.config_access_method {
            PCI_CONFIG_ACCESS_MECHANISM1 => {
                ((bus as u32) << 16) | ((device as u32) << 11) | ((function as u32) << 8) | (offset as u32)
            },
            _ => 0, // Mechanism 2 would be different
        }
    }
    
    /// Read configuration space 8-bit value
    fn read_config_u8(&self, address: u32) -> u8 {
        unsafe {
            core::arch::asm!(
                "outl {0:e}, {1:w}",
                in(reg) address,
                in(reg) self.config_address_port
            );
            
            let mut result: u8 = 0;
            core::arch::asm!(
                "inb {0:w}, {1:b}",
                in(reg) self.config_data_port,
                out(reg) result
            );
            
            result
        }
    }
    
    /// Read configuration space 16-bit value
    fn read_config_u16(&self, address: u32) -> u16 {
        unsafe {
            core::arch::asm!(
                "outl {0:e}, {1:w}",
                in(reg) address,
                in(reg) self.config_address_port
            );
            
            let mut result: u16 = 0;
            core::arch::asm!(
                "inw {0:w}, {1:w}",
                in(reg) self.config_data_port,
                out(reg) result
            );
            
            result
        }
    }
    
    /// Read configuration space 32-bit value
    fn read_config_u32(&self, address: u32) -> u32 {
        unsafe {
            core::arch::asm!(
                "outl {0:e}, {1:w}",
                in(reg) address,
                in(reg) self.config_address_port
            );
            
            let mut result: u32 = 0;
            core::arch::asm!(
                "inl {0:w}, {1:e}",
                in(reg) self.config_data_port,
                out(reg) result
            );
            
            result
        }
    }
    
    /// Write configuration space 8-bit value
    fn write_config_u8(&self, address: u32, value: u8) {
        unsafe {
            core::arch::asm!(
                "outl {0:e}, {1:w}",
                in(reg) address,
                in(reg) self.config_address_port
            );
            
            core::arch::asm!(
                "outb {0:b}, {1:w}",
                in(reg) value,
                in(reg) self.config_data_port
            );
        }
    }
    
    /// Write configuration space 16-bit value
    fn write_config_u16(&self, address: u32, value: u16) {
        unsafe {
            core::arch::asm!(
                "outl {0:e}, {1:w}",
                in(reg) address,
                in(reg) self.config_address_port
            );
            
            core::arch::asm!(
                "outw {0:w}, {1:w}",
                in(reg) value,
                in(reg) self.config_data_port
            );
        }
    }
    
    /// Write configuration space 32-bit value
    fn write_config_u32(&self, address: u32, value: u32) {
        unsafe {
            core::arch::asm!(
                "outl {0:e}, {1:w}",
                in(reg) address,
                in(reg) self.config_address_port
            );
            
            core::arch::asm!(
                "outl {0:e}, {1:w}",
                in(reg) value,
                in(reg) self.config_data_port
            );
        }
    }
    
    /// Get all devices
    pub fn get_devices(&self) -> &[PciDeviceInfo] {
        &self.devices
    }
    
    /// Find device by vendor/device ID
    pub fn find_device(&self, vendor_id: u16, device_id: u16) -> Option<&PciDeviceInfo> {
        self.devices.iter().find(|&d| d.vendor_id == vendor_id && d.device_id == device_id)
    }
    
    /// Find devices by class code
    pub fn find_devices_by_class(&self, class_code: u8) -> Vec<&PciDeviceInfo> {
        self.devices.iter().filter(|&d| d.class_code == class_code).collect()
    }
    
    /// Enable device
    pub fn enable_device(&self, bus: u8, device: u8, function: u8) -> Result<(), KernelError> {
        let command_addr = self.make_config_address(bus, device, function, PCI_COMMAND);
        let mut command = self.read_config_u16(command_addr);
        
        // Enable I/O space, memory space, and bus mastering
        command |= PCI_COMMAND_IO_SPACE | PCI_COMMAND_MEMORY_SPACE | PCI_COMMAND_BUS_MASTER;
        
        self.write_config_u16(command_addr, command);
        
        info!("Enabled device {}:{}:{}", bus, device, function);
        Ok(())
    }
    
    /// Get device by location
    pub fn get_device_by_location(&self, bus: u8, device: u8, function: u8) -> Option<&PciDeviceInfo> {
        self.devices.iter().find(|&d| d.bus == bus && d.device == device && d.function == function)
    }
}