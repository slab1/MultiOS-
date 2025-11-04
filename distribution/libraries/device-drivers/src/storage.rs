//! Storage Controller Drivers
//! 
//! Support for SATA, NVMe, and USB mass storage devices with block device operations.

use crate::{DeviceType, DriverResult, DriverError, Device, DeviceHandle, DeviceInfo, HardwareAddress, BusHandle, BusType, DeviceCapabilities, DeviceState, DeviceDriver};
use crate::device::DeviceCapability;
use spin::{Mutex, RwLock};
use alloc::{vec::Vec, collections::BTreeMap};
use log::{info, warn, error};

/// Storage device types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum StorageType {
    Unknown = 0,
    Sata = 1,
    Nvme = 2,
    UsbMass = 3,
    Scsi = 4,
    Ide = 5,
}

/// Block device information
#[derive(Debug, Clone)]
pub struct BlockDeviceInfo {
    pub sector_size: u32,
    pub total_sectors: u64,
    pub max_transfer_size: u32,
    pub queue_depth: u32,
    pub is_removable: bool,
    pub is_read_only: bool,
    pub device_name: &'static str,
}

/// Block I/O request
#[derive(Debug, Clone)]
pub struct BlockIoRequest {
    pub request_id: u64,
    pub operation: BlockOperation,
    pub sector: u64,
    pub sector_count: u32,
    pub buffer: Vec<u8>,
    pub callback: Option<fn(BlockIoResult)>,
}

/// Block device operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum BlockOperation {
    Read = 0,
    Write = 1,
    Flush = 2,
    Trim = 3,
    Synchronize = 4,
}

/// Block I/O result
#[derive(Debug, Clone)]
pub struct BlockIoResult {
    pub request_id: u64,
    pub success: bool,
    pub bytes_transferred: usize,
    pub error: Option<DriverError>,
}

/// Block device traits
pub trait BlockDevice: Send + Sync {
    /// Read sectors from device
    fn read_sectors(&self, sector: u64, count: u32, buffer: &mut [u8]) -> DriverResult<usize>;
    
    /// Write sectors to device
    fn write_sectors(&self, sector: u64, count: u32, buffer: &[u8]) -> DriverResult<usize>;
    
    /// Flush write cache
    fn flush(&self) -> DriverResult<()>;
    
    /// Get device information
    fn get_info(&self) -> DriverResult<BlockDeviceInfo>;
    
    /// Trim (deallocate) sectors
    fn trim_sectors(&self, sector: u64, count: u32) -> DriverResult<()>;
    
    /// Check if device is ready
    fn is_ready(&self) -> bool;
}

/// SATA Controller Driver
pub struct SataController {
    io_base: u64,
    control_base: u64,
    devices: BTreeMap<u8, SataDevice>,
    interrupt_enabled: bool,
}

#[derive(Debug, Clone)]
struct SataDevice {
    port: u8,
    device_type: SataDeviceType,
    sector_size: u32,
    total_sectors: u64,
    is_present: bool,
    is_ready: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum SataDeviceType {
    None = 0,
    Sata = 1,
    Sata2 = 2,
    Sata3 = 3,
}

impl SataController {
    /// Create new SATA controller driver
    pub fn new(io_base: u64, control_base: u64) -> Self {
        Self {
            io_base,
            control_base,
            devices: BTreeMap::new(),
            interrupt_enabled: false,
        }
    }
    
    /// Initialize SATA controller
    pub fn init(&mut self) -> DriverResult<()> {
        info!("Initializing SATA controller at I/O base: 0x{:X}", self.io_base);
        
        // Reset controller
        self.reset_controller()?;
        
        // Enable interrupts
        self.enable_interrupts()?;
        
        // Detect devices
        self.detect_devices()?;
        
        info!("SATA controller initialized with {} devices", self.devices.len());
        Ok(())
    }
    
    /// Reset SATA controller
    fn reset_controller(&self) -> DriverResult<()> {
        info!("Resetting SATA controller");
        
        // Reset via control register
        // In real implementation, write to specific control registers
        
        // Wait for reset completion
        // This would involve polling status registers
        
        info!("SATA controller reset completed");
        Ok(())
    }
    
    /// Enable interrupts
    fn enable_interrupts(&mut self) -> DriverResult<()> {
        self.interrupt_enabled = true;
        info!("SATA interrupts enabled");
        Ok(())
    }
    
    /// Detect SATA devices
    fn detect_devices(&mut self) -> DriverResult<()> {
        info!("Detecting SATA devices");
        
        // Check each SATA port for devices
        for port in 0..6 {
            if let Ok(device_info) = self.detect_device_on_port(port) {
                self.devices.insert(port, device_info);
                info!("Found SATA device on port {}", port);
            }
        }
        
        Ok(())
    }
    
    /// Detect device on specific port
    fn detect_device_on_port(&self, port: u8) -> DriverResult<SataDevice> {
        // Perform device identification
        // Send identify command and parse response
        
        let device_type = if port % 3 == 0 {
            SataDeviceType::Sata
        } else if port % 3 == 1 {
            SataDeviceType::Sata2
        } else {
            SataDeviceType::Sata3
        };
        
        Ok(SataDevice {
            port,
            device_type,
            sector_size: 512,
            total_sectors: 250_059_350_016 / 512, // ~250GB example
            is_present: true,
            is_ready: true,
        })
    }
    
    /// Read sectors from SATA device
    fn read_sectors(&self, port: u8, sector: u64, count: u32, buffer: &mut [u8]) -> DriverResult<usize> {
        if let Some(device) = self.devices.get(&port) {
            if !device.is_ready {
                return Err(DriverError::DeviceNotFound);
            }
            
            // Calculate total bytes to read
            let bytes_to_read = (count as usize) * (device.sector_size as usize);
            
            if buffer.len() < bytes_to_read {
                return Err(DriverError::HardwareError);
            }
            
            info!("Reading {} sectors from SATA device on port {} starting at sector {}", count, port, sector);
            
            // In real implementation, this would:
            // 1. Set up DMA descriptors
            // 2. Build ATA command
            // 3. Submit command to controller
            // 4. Wait for completion
            // 5. Copy data from DMA buffer
            
            // For now, simulate successful read
            for i in 0..bytes_to_read {
                buffer[i] = ((sector + (i as u64 / 512)) & 0xFF) as u8;
            }
            
            Ok(bytes_to_read)
        } else {
            Err(DriverError::DeviceNotFound)
        }
    }
    
    /// Write sectors to SATA device
    fn write_sectors(&self, port: u8, sector: u64, count: u32, buffer: &[u8]) -> DriverResult<usize> {
        if let Some(device) = self.devices.get(&port) {
            if !device.is_ready {
                return Err(DriverError::DeviceNotFound);
            }
            
            // Calculate total bytes to write
            let bytes_to_write = (count as usize) * (device.sector_size as usize);
            
            if buffer.len() < bytes_to_write {
                return Err(DriverError::HardwareError);
            }
            
            info!("Writing {} sectors to SATA device on port {} starting at sector {}", count, port, sector);
            
            // In real implementation, this would:
            // 1. Set up DMA descriptors
            // 2. Copy data to DMA buffer
            // 3. Build ATA write command
            // 4. Submit command to controller
            // 5. Wait for completion
            
            Ok(bytes_to_write)
        } else {
            Err(DriverError::DeviceNotFound)
        }
    }
    
    /// Flush SATA device cache
    fn flush_device(&self, port: u8) -> DriverResult<()> {
        if let Some(_device) = self.devices.get(&port) {
            info!("Flushing SATA device cache on port {}", port);
            // In real implementation, send flush command
            Ok(())
        } else {
            Err(DriverError::DeviceNotFound)
        }
    }
}

impl DeviceDriver for SataController {
    fn name(&self) -> &'static str {
        "SATA Controller Driver"
    }
    
    fn supported_devices(&self) -> &[DeviceType] {
        &[DeviceType::Storage]
    }
    
    fn init(&self, _device: &Device) -> DriverResult<()> {
        info!("Initializing SATA controller driver");
        Ok(())
    }
    
    fn remove(&self, _device: &Device) -> DriverResult<()> {
        info!("Removing SATA controller driver");
        Ok(())
    }
    
    fn read(&self, _device: &Device, _buffer: &mut [u8]) -> DriverResult<usize> {
        Ok(0)
    }
    
    fn write(&self, _device: &Device, _buffer: &[u8]) -> DriverResult<usize> {
        Ok(0)
    }
    
    fn ioctl(&self, _device: &Device, _command: u32, _data: usize) -> DriverResult<usize> {
        Ok(0)
    }
    
    fn capabilities(&self) -> DeviceCapabilities {
        DeviceCapabilities::READ | DeviceCapabilities::WRITE | DeviceCapabilities::INTERRUPT | DeviceCapabilities::DMA
    }
}

impl BlockDevice for SataController {
    fn read_sectors(&self, sector: u64, count: u32, buffer: &mut [u8]) -> DriverResult<usize> {
        // Default to port 0 for simplicity
        self.read_sectors(0, sector, count, buffer)
    }
    
    fn write_sectors(&self, sector: u64, count: u32, buffer: &[u8]) -> DriverResult<usize> {
        // Default to port 0 for simplicity
        self.write_sectors(0, sector, count, buffer)
    }
    
    fn flush(&self) -> DriverResult<()> {
        self.flush_device(0)
    }
    
    fn get_info(&self) -> DriverResult<BlockDeviceInfo> {
        if let Some(device) = self.devices.get(&0) {
            Ok(BlockDeviceInfo {
                sector_size: device.sector_size,
                total_sectors: device.total_sectors,
                max_transfer_size: 8 * 1024 * 1024, // 8MB max transfer
                queue_depth: 32,
                is_removable: false,
                is_read_only: false,
                device_name: "SATA Device 0",
            })
        } else {
            Err(DriverError::DeviceNotFound)
        }
    }
    
    fn trim_sectors(&self, _sector: u64, _count: u32) -> DriverResult<()> {
        // TRIM support would be implemented for SSDs
        info!("TRIM operation requested");
        Ok(())
    }
    
    fn is_ready(&self) -> bool {
        self.devices.contains_key(&0) && self.devices.get(&0).unwrap().is_ready
    }
}

/// NVMe Controller Driver
pub struct NvmeController {
    admin_base: u64,
    io_base: u64,
    queue_size: u16,
    namespace_count: u32,
    devices: BTreeMap<u32, NvmeNamespace>,
    interrupt_enabled: bool,
}

#[derive(Debug, Clone)]
struct NvmeNamespace {
    namespace_id: u32,
    size: u64,
    capacity: u64,
    lba_size: u32,
    max_io_size: u32,
    is_active: bool,
}

impl NvmeController {
    /// Create new NVMe controller driver
    pub fn new(admin_base: u64, io_base: u64) -> Self {
        Self {
            admin_base,
            io_base,
            queue_size: 1024,
            namespace_count: 1,
            devices: BTreeMap::new(),
            interrupt_enabled: false,
        }
    }
    
    /// Initialize NVMe controller
    pub fn init(&mut self) -> DriverResult<()> {
        info!("Initializing NVMe controller");
        
        // Reset controller
        self.reset_controller()?;
        
        // Set up admin queue
        self.setup_admin_queue()?;
        
        // Enable interrupts
        self.enable_interrupts()?;
        
        // Enumerate namespaces
        self.enumerate_namespaces()?;
        
        info!("NVMe controller initialized with {} namespaces", self.devices.len());
        Ok(())
    }
    
    /// Reset NVMe controller
    fn reset_controller(&self) -> DriverResult<()> {
        info!("Resetting NVMe controller");
        
        // In real implementation:
        // 1. Disable controller
        // 2. Reset all queues
        // 3. Clear configuration
        // 4. Re-enable controller
        
        Ok(())
    }
    
    /// Set up admin queue
    fn setup_admin_queue(&self) -> DriverResult<()> {
        info!("Setting up NVMe admin queue");
        // Create and configure admin submission/completion queues
        Ok(())
    }
    
    /// Enable interrupts
    fn enable_interrupts(&mut self) -> DriverResult<()> {
        self.interrupt_enabled = true;
        info!("NVMe interrupts enabled");
        Ok(())
    }
    
    /// Enumerate namespaces
    fn enumerate_namespaces(&mut self) -> DriverResult<()> {
        info!("Enumerating NVMe namespaces");
        
        // In real implementation, this would query the controller for all namespaces
        
        // Add example namespace
        self.devices.insert(1, NvmeNamespace {
            namespace_id: 1,
            size: 500_118_046_720, // ~500GB
            capacity: 500_118_046_720,
            lba_size: 512,
            max_io_size: 16 * 1024 * 1024, // 16MB max I/O
            is_active: true,
        });
        
        Ok(())
    }
    
    /// Read LBA from NVMe namespace
    fn read_lba(&self, namespace_id: u32, lba: u64, count: u32, buffer: &mut [u8]) -> DriverResult<usize> {
        if let Some(namespace) = self.devices.get(&namespace_id) {
            if !namespace.is_active {
                return Err(DriverError::DeviceNotFound);
            }
            
            let bytes_to_read = (count as usize) * (namespace.lba_size as usize);
            
            if buffer.len() < bytes_to_read {
                return Err(DriverError::HardwareError);
            }
            
            info!("Reading {} LBAs from NVMe namespace {} starting at LBA {}", count, namespace_id, lba);
            
            // Simulate NVMe read operation
            for i in 0..bytes_to_read {
                buffer[i] = ((lba + (i as u64 / 512)) & 0xFF) as u8;
            }
            
            Ok(bytes_to_read)
        } else {
            Err(DriverError::DeviceNotFound)
        }
    }
    
    /// Write LBA to NVMe namespace
    fn write_lba(&self, namespace_id: u32, lba: u64, count: u32, buffer: &[u8]) -> DriverResult<usize> {
        if let Some(namespace) = self.devices.get(&namespace_id) {
            if !namespace.is_active {
                return Err(DriverError::DeviceNotFound);
            }
            
            let bytes_to_write = (count as usize) * (namespace.lba_size as usize);
            
            if buffer.len() < bytes_to_write {
                return Err(DriverError::HardwareError);
            }
            
            info!("Writing {} LBAs to NVMe namespace {} starting at LBA {}", count, namespace_id, lba);
            
            Ok(bytes_to_write)
        } else {
            Err(DriverError::DeviceNotFound)
        }
    }
}

impl DeviceDriver for NvmeController {
    fn name(&self) -> &'static str {
        "NVMe Controller Driver"
    }
    
    fn supported_devices(&self) -> &[DeviceType] {
        &[DeviceType::Storage]
    }
    
    fn init(&self, _device: &Device) -> DriverResult<()> {
        info!("Initializing NVMe controller driver");
        Ok(())
    }
    
    fn remove(&self, _device: &Device) -> DriverResult<()> {
        info!("Removing NVMe controller driver");
        Ok(())
    }
    
    fn read(&self, _device: &Device, _buffer: &mut [u8]) -> DriverResult<usize> {
        Ok(0)
    }
    
    fn write(&self, _device: &Device, _buffer: &[u8]) -> DriverResult<usize> {
        Ok(0)
    }
    
    fn ioctl(&self, _device: &Device, _command: u32, _data: usize) -> DriverResult<usize> {
        Ok(0)
    }
    
    fn capabilities(&self) -> DeviceCapabilities {
        DeviceCapabilities::READ | DeviceCapabilities::WRITE | DeviceCapabilities::INTERRUPT | DeviceCapabilities::DMA
    }
}

impl BlockDevice for NvmeController {
    fn read_sectors(&self, sector: u64, count: u32, buffer: &mut [u8]) -> DriverResult<usize> {
        // Default to namespace 1 for simplicity
        self.read_lba(1, sector, count, buffer)
    }
    
    fn write_sectors(&self, sector: u64, count: u32, buffer: &[u8]) -> DriverResult<usize> {
        // Default to namespace 1 for simplicity
        self.write_lba(1, sector, count, buffer)
    }
    
    fn flush(&self) -> DriverResult<()> {
        info!("Flushing NVMe controller cache");
        Ok(())
    }
    
    fn get_info(&self) -> DriverResult<BlockDeviceInfo> {
        if let Some(namespace) = self.devices.get(&1) {
            Ok(BlockDeviceInfo {
                sector_size: namespace.lba_size,
                total_sectors: namespace.capacity / namespace.lba_size as u64,
                max_transfer_size: namespace.max_io_size,
                queue_depth: 64, // NVMe typically has deeper queues
                is_removable: false,
                is_read_only: false,
                device_name: "NVMe Namespace 1",
            })
        } else {
            Err(DriverError::DeviceNotFound)
        }
    }
    
    fn trim_sectors(&self, _sector: u64, _count: u32) -> DriverResult<()> {
        info!("TRIM operation requested for NVMe");
        Ok(())
    }
    
    fn is_ready(&self) -> bool {
        self.devices.contains_key(&1) && self.devices.get(&1).unwrap().is_active
    }
}

/// USB Mass Storage Driver
pub struct UsbMassStorage {
    usb_controller: u8,
    device_address: u8,
    endpoint_in: u8,
    endpoint_out: u8,
    max_lun: u8,
    current_lun: u8,
    removable: bool,
}

impl UsbMassStorage {
    /// Create new USB mass storage driver
    pub fn new(usb_controller: u8, device_address: u8, endpoint_in: u8, endpoint_out: u8) -> Self {
        Self {
            usb_controller,
            device_address,
            endpoint_in,
            endpoint_out,
            max_lun: 0,
            current_lun: 0,
            removable: true,
        }
    }
    
    /// Initialize USB mass storage device
    pub fn init(&mut self) -> DriverResult<()> {
        info!("Initializing USB Mass Storage device");
        
        // Get device information
        self.get_device_info()?;
        
        // Reset device
        self.reset_device()?;
        
        // Initialize LUNs
        self.initialize_luns()?;
        
        info!("USB Mass Storage initialized");
        Ok(())
    }
    
    /// Get device information
    fn get_device_info(&mut self) -> DriverResult<()> {
        info!("Getting USB Mass Storage device information");
        
        // Send SCSI INQUIRY command
        // Parse response to get device type and capabilities
        
        self.max_lun = 0; // Single LUN device
        
        Ok(())
    }
    
    /// Reset USB device
    fn reset_device(&self) -> DriverResult<()> {
        info!("Resetting USB Mass Storage device");
        
        // Send USB reset request
        // Clear endpoint halt conditions
        
        Ok(())
    }
    
    /// Initialize logical units
    fn initialize_luns(&mut self) -> DriverResult<()> {
        info!("Initializing USB Mass Storage LUNs");
        
        // For each LUN, send SCSI TEST UNIT READY and READ CAPACITY
        
        Ok(())
    }
    
    /// Read sectors via SCSI
    fn read_sectors_scsi(&self, lun: u8, sector: u64, count: u32, buffer: &mut [u8]) -> DriverResult<usize> {
        info!("Reading {} sectors from USB Mass Storage LUN {} starting at sector {}", count, lun, sector);
        
        // Build SCSI READ(10) command
        // Send via bulk-out endpoint
        // Receive data via bulk-in endpoint
        
        let bytes_to_read = (count as usize) * 512;
        if buffer.len() < bytes_to_read {
            return Err(DriverError::HardwareError);
        }
        
        // Simulate read
        for i in 0..bytes_to_read {
            buffer[i] = ((sector + (i as u64 / 512)) & 0xFF) as u8;
        }
        
        Ok(bytes_to_read)
    }
    
    /// Write sectors via SCSI
    fn write_sectors_scsi(&self, lun: u8, sector: u64, count: u32, buffer: &[u8]) -> DriverResult<usize> {
        info!("Writing {} sectors to USB Mass Storage LUN {} starting at sector {}", count, lun, sector);
        
        let bytes_to_write = (count as usize) * 512;
        if buffer.len() < bytes_to_write {
            return Err(DriverError::HardwareError);
        }
        
        // Build SCSI WRITE(10) command
        // Send command and data via bulk-out endpoint
        
        Ok(bytes_to_write)
    }
}

impl DeviceDriver for UsbMassStorage {
    fn name(&self) -> &'static str {
        "USB Mass Storage Driver"
    }
    
    fn supported_devices(&self) -> &[DeviceType] {
        &[DeviceType::Storage]
    }
    
    fn init(&self, _device: &Device) -> DriverResult<()> {
        info!("Initializing USB Mass Storage driver");
        Ok(())
    }
    
    fn remove(&self, _device: &Device) -> DriverResult<()> {
        info!("Removing USB Mass Storage driver");
        Ok(())
    }
    
    fn read(&self, _device: &Device, _buffer: &mut [u8]) -> DriverResult<usize> {
        Ok(0)
    }
    
    fn write(&self, _device: &Device, _buffer: &[u8]) -> DriverResult<usize> {
        Ok(0)
    }
    
    fn ioctl(&self, _device: &Device, _command: u32, _data: usize) -> DriverResult<usize> {
        Ok(0)
    }
    
    fn capabilities(&self) -> DeviceCapabilities {
        DeviceCapabilities::READ | DeviceCapabilities::WRITE | DeviceCapabilities::INTERRUPT | DeviceCapabilities::HOT_PLUG
    }
}

impl BlockDevice for UsbMassStorage {
    fn read_sectors(&self, sector: u64, count: u32, buffer: &mut [u8]) -> DriverResult<usize> {
        self.read_sectors_scsi(self.current_lun, sector, count, buffer)
    }
    
    fn write_sectors(&self, sector: u64, count: u32, buffer: &[u8]) -> DriverResult<usize> {
        self.write_sectors_scsi(self.current_lun, sector, count, buffer)
    }
    
    fn flush(&self) -> DriverResult<()> {
        info!("Flushing USB Mass Storage device");
        // Send SYNCHRONIZE CACHE SCSI command
        Ok(())
    }
    
    fn get_info(&self) -> DriverResult<BlockDeviceInfo> {
        Ok(BlockDeviceInfo {
            sector_size: 512,
            total_sectors: 1_000_000_000, // Example size
            max_transfer_size: 64 * 1024, // 64KB max transfer
            queue_depth: 1, // USB mass storage typically single-threaded
            is_removable: self.removable,
            is_read_only: false,
            device_name: "USB Mass Storage",
        })
    }
    
    fn trim_sectors(&self, _sector: u64, _count: u32) -> DriverResult<()> {
        // Some USB SSDs support TRIM via SCSI UNMAP
        info!("TRIM operation requested for USB Mass Storage");
        Ok(())
    }
    
    fn is_ready(&self) -> bool {
        // Check if device is ready for I/O
        true
    }
}

/// Storage driver manager
pub struct StorageDriverManager {
    sata_controller: Option<SataController>,
    nvme_controller: Option<NvmeController>,
    usb_mass_storage: Vec<UsbMassStorage>,
    current_device: Option<&'static dyn BlockDevice>,
}

impl StorageDriverManager {
    /// Create new storage driver manager
    pub fn new() -> Self {
        Self {
            sata_controller: None,
            nvme_controller: None,
            usb_mass_storage: Vec::new(),
            current_device: None,
        }
    }
    
    /// Register SATA controller
    pub fn register_sata(&mut self, io_base: u64, control_base: u64) -> DriverResult<()> {
        let mut controller = SataController::new(io_base, control_base);
        controller.init()?;
        self.sata_controller = Some(controller);
        info!("SATA controller registered and initialized");
        Ok(())
    }
    
    /// Register NVMe controller
    pub fn register_nvme(&mut self, admin_base: u64, io_base: u64) -> DriverResult<()> {
        let mut controller = NvmeController::new(admin_base, io_base);
        controller.init()?;
        self.nvme_controller = Some(controller);
        info!("NVMe controller registered and initialized");
        Ok(())
    }
    
    /// Register USB mass storage device
    pub fn register_usb_mass_storage(&mut self, usb_controller: u8, device_address: u8, 
                                    endpoint_in: u8, endpoint_out: u8) -> DriverResult<()> {
        let mut device = UsbMassStorage::new(usb_controller, device_address, endpoint_in, endpoint_out);
        device.init()?;
        self.usb_mass_storage.push(device);
        info!("USB Mass Storage device registered and initialized");
        Ok(())
    }
    
    /// Get primary storage device
    pub fn get_primary_device(&self) -> Option<&dyn BlockDevice> {
        // Prefer NVMe, then SATA, then USB
        if let Some(ref nvme) = self.nvme_controller {
            Some(nvme)
        } else if let Some(ref sata) = self.sata_controller {
            Some(sata)
        } else if let Some(usb) = self.usb_mass_storage.first() {
            Some(usb)
        } else {
            None
        }
    }
    
    /// Read sectors from primary device
    pub fn read_sectors(&self, sector: u64, count: u32, buffer: &mut [u8]) -> DriverResult<usize> {
        if let Some(device) = self.get_primary_device() {
            device.read_sectors(sector, count, buffer)
        } else {
            Err(DriverError::DeviceNotFound)
        }
    }
    
    /// Write sectors to primary device
    pub fn write_sectors(&self, sector: u64, count: u32, buffer: &[u8]) -> DriverResult<usize> {
        if let Some(device) = self.get_primary_device() {
            device.write_sectors(sector, count, buffer)
        } else {
            Err(DriverError::DeviceNotFound)
        }
    }
    
    /// Get device information
    pub fn get_device_info(&self) -> DriverResult<BlockDeviceInfo> {
        if let Some(device) = self.get_primary_device() {
            device.get_info()
        } else {
            Err(DriverError::DeviceNotFound)
        }
    }
    
    /// Check if primary device is ready
    pub fn is_ready(&self) -> bool {
        self.get_primary_device().map_or(false, |d| d.is_ready())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_device_info() {
        let info = BlockDeviceInfo {
            sector_size: 512,
            total_sectors: 1_000_000_000,
            max_transfer_size: 8 * 1024 * 1024,
            queue_depth: 32,
            is_removable: false,
            is_read_only: false,
            device_name: "Test Device",
        };
        
        assert_eq!(info.sector_size, 512);
        assert_eq!(info.total_sectors, 1_000_000_000);
    }

    #[test]
    fn test_block_io_request() {
        let request = BlockIoRequest {
            request_id: 1,
            operation: BlockOperation::Read,
            sector: 0,
            sector_count: 8,
            buffer: vec![0u8; 4096],
            callback: None,
        };
        
        assert_eq!(request.request_id, 1);
        assert_eq!(request.operation, BlockOperation::Read);
    }

    #[test]
    fn test_sata_controller_creation() {
        let controller = SataController::new(0x1F0, 0x3F6);
        assert!(!controller.is_ready());
    }

    #[test]
    fn test_storage_driver_manager() {
        let mut manager = StorageDriverManager::new();
        
        // Register SATA controller
        assert!(manager.register_sata(0x1F0, 0x3F6).is_ok());
        assert!(manager.is_ready());
        
        // Get device info
        assert!(manager.get_device_info().is_ok());
    }
}
