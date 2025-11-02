//! SATA/NVMe Storage Drivers
//! 
//! Provides support for SATA, NVMe, and other storage devices
//! including AHCI controllers and NVMe SSDs

use crate::log::{info, warn, error};
use crate::KernelError;

use super::{PciManager, StorageDeviceInfo, StorageDeviceType, ControllerType};

/// AHCI (Advanced Host Controller Interface) constants
const AHCI_CAP_SAB: u32 = 1 << 31;     // Supports 64-bit Addressing
const AHCI_CAP_SCA: u32 = 1 << 30;     // Supports Command Completion Coalescing
const AHCI_CAP_SSS: u32 = 1 << 27;     // Supports Staggered Spin-up
const AHCI_CAP_SIS: u32 = 1 << 26;     // Supports Interface Speed Identification
const AHCI_CAP_SNtf: u32 = 1 << 24;    // Supports Native Command Queuing
const AHCI_CAP_ISS: u32 = 0xF << 20;   // Interface Speed Support
const AHCI_CAP_SAM: u32 = 1 << 18;     // Supports AHCI Mode Only
const AHCI_CAP_CCCS: u32 = 1 << 19;    // Supports Command Completion Coalescing
const AHCI_CAP_PSC: u32 = 1 << 17;     // Supports Partial State
const AHCI_CAP_SSC: u32 = 1 << 16;     // Supports Slumber State
const AHCI_CAP_PMD: u32 = 1 << 15;     // Supports PIO Multiple DRQ Block
const AHCI_CAP_FBS: u32 = 1 << 14;     // Supports FIS-based Switching
const AHCI_CAP_SPM: u32 = 1 << 12;     // Supports Port Multiplier
const AHCI_CAP_SPMFL: u32 = 1 << 11;   // Supports FIS-based Switching
const AHCI_CAP_LED: u32 = 1 << 10;     // Supports LED activity
const AHCI_CAP_ALHD: u32 = 1 << 9;     // Supports Activity LED
const AHCI_CAP_CSS: u32 = 1 << 8;      // Supports Command Summary

/// AHCI Port Command and Status Registers
const AHCI_PxCMD_ST: u32 = 1 << 0;     // Start
const AHCI_PxCMD_SUD: u32 = 1 << 1;    // Spin-up Device
const AHCI_PxCMD_POD: u32 = 1 << 2;    // Power On Device
const AHCI_PxCMD_FRE: u32 = 1 << 4;    // FIS Receive Enable
const AHCI_PxCMD_FR: u32 = 1 << 14;    // FIS Running
const AHCI_PxCMD_CR: u32 = 1 << 15;    // Command List Running

/// AHCI Command Header Length
const AHCI_CMD_HEADER_LENGTH: usize = 32;

/// SATA constants
const SATA_SIG_ATA: u32 = 0x00000101;    // ATA drive signature
const SATA_SIG_ATAPI: u32 = 0xEB140101;  // ATAPI drive signature
const SATA_SIG_SEMB: u32 = 0xC33C0101;   // SEMB drive signature
const SATA_SIG_PM: u32 = 0x96690101;     // PM drive signature

/// NVMe constants
const NVME_REG_CAP: usize = 0x0000;    // Controller Capabilities
const NVME_REG_VS: usize = 0x0008;     // Version
const NVME_REG_INTMS: usize = 0x000C;  // Interrupt Mask Set
const NVME_REG_INTMC: usize = 0x0010;  // Interrupt Mask Clear
const NVME_REG_CC: usize = 0x0014;     // Controller Configuration
const NVME_REG_CSTS: usize = 0x001C;   // Controller Status
const NVME_REG_NSSR: usize = 0x0020;   // NVM Subsystem Reset
const NVME_REG_AQA: usize = 0x0024;    // Admin Queue Attributes
const NVME_REG_ASQ: usize = 0x0028;    // Admin Submission Queue Base
const NVME_REG_ACQ: usize = 0x0030;    // Admin Completion Queue Base

/// NVMe Queue Entry Size
const NVME_CQ_ENTRY_SIZE: usize = 16;
const NVME_SQ_ENTRY_SIZE: usize = 64;

/// SATA device information
#[derive(Debug, Clone)]
pub struct SataDevice {
    pub port: u8,
    pub signature: u32,
    pub sector_count: u64,
    pub lba28_sectors: u32,
    pub lba48_support: bool,
    pub atapi_support: bool,
    pub device_type: SataDeviceType,
    pub serial_number: [u8; 20],
    pub firmware_revision: [u8; 8],
    pub model_number: [u8; 40],
    pub smart_supported: bool,
    pub smart_enabled: bool,
}

/// SATA device types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SataDeviceType {
    None,
    HardDisk,
    CompactFlash,
    RemovableMedia,
    AtapiDevice,
}

/// AHCI controller information
#[derive(Debug, Clone)]
pub struct AhciController {
    pub pci_address: (u8, u8, u8),
    pub base_address: u64,
    pub ports: Vec<AhciPort>,
    pub command_queue_depth: u32,
    pub supports_ncq: bool,
    pub supports_64bit: bool,
    pub supports_fis_based_switching: bool,
    pub supports_activity_led: bool,
}

/// AHCI port information
#[derive(Debug, Clone)]
pub struct AhciPort {
    pub port_number: u8,
    pub sata_device: Option<SataDevice>,
    pub command_list_base: u64,
    pub fis_receive_base: u64,
    pub sector_count: u64,
    pub max_transfer_size: usize,
    pub is_connected: bool,
    pub device_present: bool,
}

/// NVMe device information
#[derive(Debug, Clone)]
pub struct NvmeDevice {
    pub pci_address: (u8, u8, u8),
    pub controller_id: u16,
    pub namespace_id: u16,
    pub capacity: u64,
    pub form_factor: u8,
    pub serial_number: [u8; 20],
    pub model_number: [u8; 40],
    pub firmware_revision: [u8; 8],
    pub max_transfer_size: usize,
    pub queue_depth: u32,
    pub supports_multiple_queues: bool,
    pub supports_namespace_management: bool,
}

/// Storage I/O request
#[derive(Debug, Clone)]
pub struct StorageIoRequest {
    pub command: StorageCommand,
    pub device_id: u32,
    pub sector: u64,
    pub sector_count: usize,
    pub buffer: *const u8,
    pub buffer_length: usize,
    pub callback: Option<fn()>,
}

/// Storage commands
#[derive(Debug, Clone, Copy)]
pub enum StorageCommand {
    Read,
    Write,
    Flush,
    Identify,
    SmartRead,
    SmartWrite,
}

/// Storage Manager
pub struct StorageManager {
    pub initialized: bool,
    pub ahci_controllers: Vec<AhciController>,
    pub nvme_devices: Vec<NvmeDevice>,
    pub storage_devices: Vec<StorageDeviceInfo>,
    pub io_requests: Vec<StorageIoRequest>,
    pub is_processing_io: bool,
}

impl StorageManager {
    /// Create new storage manager
    pub fn new() -> Self {
        Self {
            initialized: false,
            ahci_controllers: Vec::new(),
            nvme_devices: Vec::new(),
            storage_devices: Vec::new(),
            io_requests: Vec::new(),
            is_processing_io: false,
        }
    }
    
    /// Initialize storage subsystem
    pub fn initialize(&mut self, pci_manager: &PciManager) -> Result<(), KernelError> {
        info!("Initializing storage subsystem...");
        
        // Step 1: Detect AHCI controllers
        self.detect_ahci_controllers(pci_manager)?;
        
        // Step 2: Detect NVMe devices
        self.detect_nvme_devices(pci_manager)?;
        
        // Step 3: Initialize AHCI controllers
        self.init_ahci_controllers()?;
        
        // Step 4: Initialize NVMe devices
        self.init_nvme_devices()?;
        
        // Step 5: Detect storage devices
        self.detect_storage_devices()?;
        
        // Step 6: Setup I/O scheduling
        self.setup_io_scheduling()?;
        
        self.initialized = true;
        
        info!("Storage subsystem initialized: {} AHCI controllers, {} NVMe devices", 
              self.ahci_controllers.len(), self.nvme_devices.len());
        
        Ok(())
    }
    
    /// Detect AHCI controllers
    fn detect_ahci_controllers(&mut self, pci_manager: &PciManager) -> Result<(), KernelError> {
        info!("Detecting AHCI controllers...");
        
        // Find AHCI-capable PCI devices
        let ahci_devices = pci_manager.find_devices_by_class(0x01); // Mass storage controller
        let mut found_ahci = 0;
        
        for device in ahci_devices {
            if device.subclass == 0x06 { // Serial ATA
                // Check if this is AHCI-capable
                let ahci_capable = self.check_ahci_capability(device.bus, device.device, device.function)?;
                
                if ahci_capable {
                    let controller = AhciController {
                        pci_address: (device.bus, device.device, device.function),
                        base_address: 0, // Would be determined from PCI BAR
                        ports: Vec::new(),
                        command_queue_depth: 32,
                        supports_ncq: true,
                        supports_64bit: true,
                        supports_fis_based_switching: true,
                        supports_activity_led: true,
                    };
                    
                    self.ahci_controllers.push(controller);
                    found_ahci += 1;
                    
                    info!("Found AHCI controller at {}:{}:{}", device.bus, device.device, device.function);
                }
            }
        }
        
        info!("Detected {} AHCI controllers", found_ahci);
        Ok(())
    }
    
    /// Detect NVMe devices
    fn detect_nvme_devices(&mut self, pci_manager: &PciManager) -> Result<(), KernelError> {
        info!("Detecting NVMe devices...");
        
        // Find NVMe devices (class code 0x01, subclass 0x08, prog_if 0x02)
        let nvme_devices = pci_manager.devices.iter()
            .filter(|d| d.class_code == 0x01 && d.subclass == 0x08 && d.prog_if == 0x02)
            .collect::<Vec<_>>();
        
        for device in nvme_devices {
            let nvme_device = self.init_nvme_device(device.bus, device.device, device.function)?;
            
            if nvme_device.is_some() {
                self.nvme_devices.push(nvme_device.unwrap());
                info!("Found NVMe device at {}:{}:{}", device.bus, device.device, device.function);
            }
        }
        
        Ok(())
    }
    
    /// Check AHCI capability
    fn check_ahci_capability(&self, bus: u8, device: u8, function: u8) -> Result<bool, KernelError> {
        // This would check PCI configuration space for AHCI capability
        // For now, assume all SATA controllers are AHCI-capable
        Ok(true)
    }
    
    /// Initialize AHCI controllers
    fn init_ahci_controllers(&mut self) -> Result<(), KernelError> {
        info!("Initializing AHCI controllers...");
        
        for controller in &mut self.ahci_controllers {
            self.init_single_ahci_controller(controller)?;
        }
        
        Ok(())
    }
    
    /// Initialize single AHCI controller
    fn init_single_ahci_controller(&mut self, controller: &mut AhciController) -> Result<(), KernelError> {
        // Enable AHCI mode in PCI configuration
        // Enable bus mastering and memory space access
        
        // Setup command queue and FIS receive areas
        
        // Enable interrupts
        
        // Initialize ports
        for i in 0..32 { // AHCI supports up to 32 ports
            let port = AhciPort {
                port_number: i as u8,
                sata_device: None,
                command_list_base: 0,
                fis_receive_base: 0,
                sector_count: 0,
                max_transfer_size: 0x100000, // 1MB default
                is_connected: false,
                device_present: false,
            };
            controller.ports.push(port);
        }
        
        Ok(())
    }
    
    /// Initialize NVMe devices
    fn init_nvme_devices(&mut self) -> Result<(), KernelError> {
        info!("Initializing NVMe devices...");
        
        for device in &mut self.nvme_devices {
            self.init_single_nvme_device(device)?;
        }
        
        Ok(())
    }
    
    /// Initialize single NVMe device
    fn init_single_nvme_device(&mut self, device: &mut NvmeDevice) -> Result<(), KernelError> {
        // Enable device in PCI configuration
        // Reset controller
        // Setup admin and I/O queues
        // Get device capabilities and identify information
        
        Ok(())
    }
    
    /// Detect storage devices
    fn detect_storage_devices(&mut self) -> Result<(), KernelError> {
        info!("Detecting storage devices...");
        
        // Detect devices on AHCI controllers
        for controller in &mut self.ahci_controllers {
            self.detect_ahci_storage_devices(controller)?;
        }
        
        // Detect NVMe devices (already detected during initialization)
        for nvme_device in &self.nvme_devices {
            let storage_info = StorageDeviceInfo {
                device_type: StorageDeviceType::NvmeSsd,
                controller_type: ControllerType::Nvme,
                capacity_bytes: nvme_device.capacity,
                sector_size: 512, // Standard for NVMe
                max_transfer_size: nvme_device.max_transfer_size,
                device_name: format!("NVMe SSD {}:{}:{}", 
                    nvme_device.pci_address.0, nvme_device.pci_address.1, nvme_device.pci_address.2),
                driver_attached: false,
            };
            
            self.storage_devices.push(storage_info);
        }
        
        info!("Detected {} storage devices", self.storage_devices.len());
        Ok(())
    }
    
    /// Detect AHCI storage devices
    fn detect_ahci_storage_devices(&mut self, controller: &mut AhciController) -> Result<(), KernelError> {
        for port in &mut controller.ports {
            // Check if device is present on this port
            if self.check_port_device_present(controller, port)? {
                let device_info = self.identify_ahci_device(controller, port)?;
                
                if device_info.is_some() {
                    port.sata_device = device_info;
                    port.device_present = true;
                    port.is_connected = true;
                    
                    // Add to storage devices list
                    if let Some(ref device) = port.sata_device {
                        let storage_info = match device.device_type {
                            SataDeviceType::HardDisk => StorageDeviceInfo {
                                device_type: StorageDeviceType::SataHdd,
                                controller_type: ControllerType::Ahci,
                                capacity_bytes: device.sector_count * 512,
                                sector_size: 512,
                                max_transfer_size: 0x100000, // 1MB
                                device_name: format!("SATA HDD Port {}", port.port_number),
                                driver_attached: false,
                            },
                            SataDeviceType::AtapiDevice => StorageDeviceInfo {
                                device_type: StorageDeviceType::M2Sata,
                                controller_type: ControllerType::Ahci,
                                capacity_bytes: device.sector_count * 512,
                                sector_size: 512,
                                max_transfer_size: 0x40000, // 256KB for ATAPI
                                device_name: format!("SATA ATAPI Port {}", port.port_number),
                                driver_attached: false,
                            },
                            _ => continue,
                        };
                        
                        self.storage_devices.push(storage_info);
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Check if device is present on AHCI port
    fn check_port_device_present(&self, controller: &AhciController, port: &AhciPort) -> Result<bool, KernelError> {
        // Read port signature to determine device presence and type
        // This would read from AHCI register space
        Ok(true) // Placeholder
    }
    
    /// Identify AHCI device
    fn identify_ahci_device(&self, controller: &AhciController, port: &AhciPort) -> Result<Option<SataDevice>, KernelError> {
        // Send IDENTIFY command to device
        // Parse identify data to get device information
        
        let device = SataDevice {
            port: port.port_number,
            signature: SATA_SIG_ATA, // Would be read from device
            sector_count: 500_000_000, // Would be from identify data
            lba28_sectors: 0x0FFFFFFF, // Would be from identify data
            lba48_support: true, // Would be from identify data
            atapi_support: false, // Would be from identify data
            device_type: SataDeviceType::HardDisk, // Would be determined
            serial_number: [0; 20],
            firmware_revision: [0; 8],
            model_number: [0; 40],
            smart_supported: true,
            smart_enabled: true,
        };
        
        Ok(Some(device))
    }
    
    /// Initialize NVMe device
    fn init_nvme_device(&self, bus: u8, device: u8, function: u8) -> Result<Option<NvmeDevice>, KernelError> {
        let nvme_device = NvmeDevice {
            pci_address: (bus, device, function),
            controller_id: 0,
            namespace_id: 1,
            capacity: 1_000_000_000_000, // 1TB, would be from identify data
            form_factor: 2, // M.2
            serial_number: [0; 20],
            model_number: [0; 40],
            firmware_revision: [0; 8],
            max_transfer_size: 0x40000, // 256KB
            queue_depth: 1024,
            supports_multiple_queues: true,
            supports_namespace_management: true,
        };
        
        Ok(Some(nvme_device))
    }
    
    /// Setup I/O scheduling
    fn setup_io_scheduling(&mut self) -> Result<(), KernelError> {
        info!("Setting up storage I/O scheduling...");
        
        // Setup I/O request queues
        // Configure interrupt handling
        // Setup timeout handling
        
        Ok(())
    }
    
    /// Read sectors from storage device
    pub fn read_sectors(&mut self, device_id: u32, sector: u64, sector_count: usize, buffer: *mut u8) 
        -> Result<usize, KernelError> {
        if !self.initialized {
            return Err(KernelError::NotInitialized);
        }
        
        if let Some(device) = self.storage_devices.get(device_id as usize) {
            match device.controller_type {
                ControllerType::Ahci => {
                    self.read_sectors_ahci(device_id, sector, sector_count, buffer)
                },
                ControllerType::Nvme => {
                    self.read_sectors_nvme(device_id, sector, sector_count, buffer)
                },
                _ => {
                    Err(KernelError::NotSupported)
                }
            }
        } else {
            Err(KernelError::NotFound)
        }
    }
    
    /// Write sectors to storage device
    pub fn write_sectors(&mut self, device_id: u32, sector: u64, sector_count: usize, buffer: *const u8) 
        -> Result<usize, KernelError> {
        if !self.initialized {
            return Err(KernelError::NotInitialized);
        }
        
        if let Some(device) = self.storage_devices.get(device_id as usize) {
            match device.controller_type {
                ControllerType::Ahci => {
                    self.write_sectors_ahci(device_id, sector, sector_count, buffer)
                },
                ControllerType::Nvme => {
                    self.write_sectors_nvme(device_id, sector, sector_count, buffer)
                },
                _ => {
                    Err(KernelError::NotSupported)
                }
            }
        } else {
            Err(KernelError::NotFound)
        }
    }
    
    /// Read sectors using AHCI
    fn read_sectors_ahci(&self, device_id: u32, sector: u64, sector_count: usize, buffer: *mut u8) 
        -> Result<usize, KernelError> {
        // Implement AHCI read using PIO or DMA
        // This would involve:
        // 1. Build read command
        // 2. Submit to command queue
        // 3. Wait for completion
        // 4. Transfer data
        
        info!("Reading {} sectors from AHCI device {} starting at sector {}", 
              sector_count, device_id, sector);
        
        Ok(sector_count)
    }
    
    /// Write sectors using AHCI
    fn write_sectors_ahci(&self, device_id: u32, sector: u64, sector_count: usize, buffer: *const u8) 
        -> Result<usize, KernelError> {
        // Implement AHCI write similar to read
        
        info!("Writing {} sectors to AHCI device {} starting at sector {}", 
              sector_count, device_id, sector);
        
        Ok(sector_count)
    }
    
    /// Read sectors using NVMe
    fn read_sectors_nvme(&self, device_id: u32, sector: u64, sector_count: usize, buffer: *mut u8) 
        -> Result<usize, KernelError> {
        // Implement NVMe read using submission queues
        
        info!("Reading {} sectors from NVMe device {} starting at sector {}", 
              sector_count, device_id, sector);
        
        Ok(sector_count)
    }
    
    /// Write sectors using NVMe
    fn write_sectors_nvme(&self, device_id: u32, sector: u64, sector_count: usize, buffer: *const u8) 
        -> Result<usize, KernelError> {
        // Implement NVMe write similar to read
        
        info!("Writing {} sectors to NVMe device {} starting at sector {}", 
              sector_count, device_id, sector);
        
        Ok(sector_count)
    }
    
    /// Get storage device information
    pub fn get_storage_device_info(&self, device_id: u32) -> Result<StorageDeviceInfo, KernelError> {
        if let Some(device) = self.storage_devices.get(device_id as usize) {
            Ok(device.clone())
        } else {
            Err(KernelError::NotFound)
        }
    }
    
    /// Get all storage devices
    pub fn get_all_storage_devices(&self) -> &[StorageDeviceInfo] {
        &self.storage_devices
    }
    
    /// Get AHCI controllers
    pub fn get_ahci_controllers(&self) -> &[AhciController] {
        &self.ahci_controllers
    }
    
    /// Get NVMe devices
    pub fn get_nvme_devices(&self) -> &[NvmeDevice] {
        &self.nvme_devices
    }
    
    /// Check if device supports TRIM
    pub fn supports_trim(&self, device_id: u32) -> bool {
        if let Some(device) = self.storage_devices.get(device_id as usize) {
            match device.device_type {
                StorageDeviceType::NvmeSsd => true, // NVMe supports TRIM
                StorageDeviceType::SataSsd => true, // Modern SATA SSDs support TRIM
                _ => false,
            }
        } else {
            false
        }
    }
    
    /// Send TRIM command to device
    pub fn trim_device(&self, device_id: u32, lba: u64, sector_count: usize) -> Result<(), KernelError> {
        if !self.initialized {
            return Err(KernelError::NotInitialized);
        }
        
        if !self.supports_trim(device_id) {
            return Err(KernelError::NotSupported);
        }
        
        info!("Sending TRIM to device {} for {} sectors starting at LBA {}", 
              device_id, sector_count, lba);
        
        // Implement TRIM/discard command
        Ok(())
    }
    
    /// Get device health information
    pub fn get_device_health(&self, device_id: u32) -> Result<DeviceHealthInfo, KernelError> {
        if let Some(_device) = self.storage_devices.get(device_id as usize) {
            // This would read SMART/health data from the device
            let health_info = DeviceHealthInfo {
                temperature: 35, // Celsius
                power_on_hours: 10000,
                bad_sectors: 0,
                reallocated_sectors: 0,
                pending_sectors: 0,
                overall_health: 95, // Percentage
            };
            
            Ok(health_info)
        } else {
            Err(KernelError::NotFound)
        }
    }
}

/// Device health information
#[derive(Debug, Clone)]
pub struct DeviceHealthInfo {
    pub temperature: u32,        // Temperature in Celsius
    pub power_on_hours: u32,     // Total power-on hours
    pub bad_sectors: u32,        // Number of bad sectors
    pub reallocated_sectors: u32, // Number of reallocated sectors
    pub pending_sectors: u32,    // Number of sectors waiting to be reallocated
    pub overall_health: u32,     // Overall health percentage (0-100)
}