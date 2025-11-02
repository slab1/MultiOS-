//! Device Virtualization Framework
//! 
//! Provides a framework for virtualizing devices in virtual machines,
//! including educational VMs with simplified device models.

use crate::{HypervisorError, VmId};
use crate::core::VmExitReason;

use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use bitflags::bitflags;
use spin::RwLock;

/// Device types enumeration
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DeviceType {
    /// PCI bridge device
    PciBridge,
    /// ISA bus device
    IsaBus,
    /// VGA graphics controller
    VgaController,
    /// Network interface card
    NetworkCard,
    /// Disk controller
    DiskController,
    /// Serial port
    SerialPort,
    /// Parallel port
    ParallelPort,
    /// USB controller
    UsbController,
    /// Audio device
    AudioDevice,
    /// Keyboard controller
    KeyboardController,
    /// Mouse controller
    MouseController,
    /// Timer device
    TimerDevice,
    /// RTC device
    RtcDevice,
    /// GPIO device
    GpioDevice,
    /// Educational demo device
    EducationalDemo,
}

/// Device state enumeration
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DeviceState {
    /// Device is not initialized
    Uninitialized,
    /// Device is initialized but not configured
    Initialized,
    /// Device is configured and ready
    Ready,
    /// Device is running
    Running,
    /// Device is paused
    Paused,
    /// Device is in error state
    Error,
}

/// Device access permissions
bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct DeviceAccess: u32 {
        const READ = 1 << 0;
        const WRITE = 1 << 1;
        const EXECUTE = 1 << 2;
        const CONFIG = 1 << 3;
        const INTERRUPT = 1 << 4;
        const DMA = 1 << 5;
    }
}

/// Device interrupt information
#[derive(Debug, Clone, Copy)]
pub struct InterruptInfo {
    pub interrupt_line: u8,
    pub level_triggered: bool,
    pub edge_triggered: bool,
    pub active: bool,
}

/// Device register information
#[derive(Debug, Clone)]
pub struct DeviceRegister {
    pub offset: u64,
    pub size: u8, // 1, 2, 4, or 8 bytes
    pub access: DeviceAccess,
    pub reset_value: u64,
    pub volatile: bool,
}

/// Device capability
#[derive(Debug, Clone)]
pub struct DeviceCapability {
    pub name: String,
    pub description: String,
    pub value: String,
}

/// Virtual device structure
#[derive(Debug)]
pub struct VirtualDevice {
    /// Device type
    pub device_type: DeviceType,
    /// Device ID
    pub device_id: String,
    /// Device name
    pub name: String,
    /// Current state
    pub state: DeviceState,
    /// Device configuration
    pub config: DeviceConfig,
    /// Memory-mapped I/O regions
    pub mmio_regions: Vec<MmioRegion>,
    /// I/O port ranges
    pub io_ports: Vec<IoPortRange>,
    /// Interrupt information
    pub interrupt: Option<InterruptInfo>,
    /// Device registers
    pub registers: Vec<DeviceRegister>,
    /// Device capabilities
    pub capabilities: Vec<DeviceCapability>,
    /// Device statistics
    pub stats: DeviceStats,
}

/// Device configuration
#[derive(Debug, Clone)]
pub struct DeviceConfig {
    /// Enable device
    pub enabled: bool,
    /// Device address (PCI/ISA)
    pub address: u32,
    /// Interrupt line
    pub interrupt_line: Option<u8>,
    /// DMA channels
    pub dma_channels: Vec<u8>,
    /// Custom configuration
    pub custom_config: BTreeMap<String, String>,
}

/// MMIO (Memory-Mapped I/O) region
#[derive(Debug, Clone, Copy)]
pub struct MmioRegion {
    pub base_address: u64,
    pub size: u64,
    pub access: DeviceAccess,
}

/// I/O port range
#[derive(Debug, Clone, Copy)]
pub struct IoPortRange {
    pub base_port: u16,
    pub size: u16,
    pub access: DeviceAccess,
}

/// Device statistics
#[derive(Debug, Clone)]
pub struct DeviceStats {
    pub read_count: u64,
    pub write_count: u64,
    pub interrupt_count: u64,
    pub error_count: u64,
    pub last_access_time: u64,
}

/// Device framework manager
pub struct DeviceFramework {
    /// VM ID this framework belongs to
    pub vm_id: VmId,
    /// Registered virtual devices
    pub devices: BTreeMap<String, Arc<RwLock<VirtualDevice>>>,
    /// Device count
    pub device_count: usize,
    /// Framework initialization time
    pub init_time: u64,
}

impl DeviceFramework {
    /// Create a new device framework
    pub fn new(vm_id: VmId) -> Self {
        DeviceFramework {
            vm_id,
            devices: BTreeMap::new(),
            device_count: 0,
            init_time: 0, // Would use actual timestamp
        }
    }
    
    /// Register a virtual device
    pub fn register_device(&mut self, device: VirtualDevice) -> Result<String, HypervisorError> {
        let device_id = format!("dev_{}_{}", device.device_type as u32, self.device_count);
        
        self.devices.insert(device_id.clone(), Arc::new(RwLock::new(device)));
        self.device_count += 1;
        
        info!("Registered device {} of type {:?}", device_id, self.devices[&device_id].read().device_type);
        Ok(device_id)
    }
    
    /// Create and register educational demo device
    pub fn create_educational_demo_device(&mut self) -> Result<String, HypervisorError> {
        let device = self.build_educational_demo_device()?;
        self.register_device(device)
    }
    
    /// Build educational demo device
    fn build_educational_demo_device(&self) -> Result<VirtualDevice, HypervisorError> {
        let device = VirtualDevice {
            device_type: DeviceType::EducationalDemo,
            device_id: String::new(),
            name: String::from("Educational Demo Device"),
            state: DeviceState::Uninitialized,
            config: DeviceConfig {
                enabled: true,
                address: 0x100,
                interrupt_line: Some(5),
                dma_channels: Vec::new(),
                custom_config: BTreeMap::new(),
            },
            mmio_regions: vec![
                MmioRegion {
                    base_address: 0xFE000000,
                    size: 0x1000,
                    access: DeviceAccess::READ | DeviceAccess::WRITE,
                }
            ],
            io_ports: Vec::new(),
            interrupt: Some(InterruptInfo {
                interrupt_line: 5,
                level_triggered: true,
                edge_triggered: false,
                active: false,
            }),
            registers: vec![
                DeviceRegister {
                    offset: 0x00,
                    size: 4,
                    access: DeviceAccess::READ | DeviceAccess::WRITE,
                    reset_value: 0x00000000,
                    volatile: false,
                },
                DeviceRegister {
                    offset: 0x04,
                    size: 2,
                    access: DeviceAccess::READ | DeviceAccess::WRITE,
                    reset_value: 0x0000,
                    volatile: false,
                },
                DeviceRegister {
                    offset: 0x08,
                    size: 1,
                    access: DeviceAccess::READ | DeviceAccess::WRITE,
                    reset_value: 0x00,
                    volatile: false,
                },
            ],
            capabilities: vec![
                DeviceCapability {
                    name: String::from("demo_mode"),
                    description: String::from("Enable demo mode"),
                    value: String::from("enabled"),
                },
                DeviceCapability {
                    name: String::from("educational_features"),
                    description: String::from("Educational features enabled"),
                    value: String::from("true"),
                },
            ],
            stats: DeviceStats {
                read_count: 0,
                write_count: 0,
                interrupt_count: 0,
                error_count: 0,
                last_access_time: 0,
            },
        };
        
        Ok(device)
    }
    
    /// Create and register standard educational VM devices
    pub fn create_educational_devices(&mut self) -> Result<(), HypervisorError> {
        // VGA controller
        let vga_device = self.build_vga_controller()?;
        self.register_device(vga_device)?;
        
        // Serial port
        let serial_device = self.build_serial_port()?;
        self.register_device(serial_device)?;
        
        // Keyboard controller
        let keyboard_device = self.build_keyboard_controller()?;
        self.register_device(keyboard_device)?;
        
        // Educational demo device
        let demo_device = self.build_educational_demo_device()?;
        self.register_device(demo_device)?;
        
        info!("Created educational device set with {} devices", self.device_count);
        Ok(())
    }
    
    /// Build VGA controller device
    fn build_vga_controller(&self) -> Result<VirtualDevice, HypervisorError> {
        let mut custom_config = BTreeMap::new();
        custom_config.insert(String::from("vram_size"), String::from("8MB"));
        custom_config.insert(String::from("resolution"), String::from("1024x768"));
        
        Ok(VirtualDevice {
            device_type: DeviceType::VgaController,
            device_id: String::new(),
            name: String::from("VGA Graphics Controller"),
            state: DeviceState::Uninitialized,
            config: DeviceConfig {
                enabled: true,
                address: 0x100,
                interrupt_line: None,
                dma_channels: Vec::new(),
                custom_config,
            },
            mmio_regions: vec![
                MmioRegion {
                    base_address: 0xA0000,
                    size: 128 * 1024,
                    access: DeviceAccess::READ | DeviceAccess::WRITE,
                },
                MmioRegion {
                    base_address: 0xC0000,
                    size: 256 * 1024,
                    access: DeviceAccess::READ | DeviceAccess::WRITE,
                }
            ],
            io_ports: Vec::new(),
            interrupt: None,
            registers: Vec::new(),
            capabilities: Vec::new(),
            stats: DeviceStats {
                read_count: 0,
                write_count: 0,
                interrupt_count: 0,
                error_count: 0,
                last_access_time: 0,
            },
        })
    }
    
    /// Build serial port device
    fn build_serial_port(&self) -> Result<VirtualDevice, HypervisorError> {
        Ok(VirtualDevice {
            device_type: DeviceType::SerialPort,
            device_id: String::new(),
            name: String::from("COM1 Serial Port"),
            state: DeviceState::Uninitialized,
            config: DeviceConfig {
                enabled: true,
                address: 0x3F8,
                interrupt_line: Some(4),
                dma_channels: Vec::new(),
                custom_config: BTreeMap::new(),
            },
            mmio_regions: Vec::new(),
            io_ports: vec![
                IoPortRange {
                    base_port: 0x3F8,
                    size: 8,
                    access: DeviceAccess::READ | DeviceAccess::WRITE,
                }
            ],
            interrupt: Some(InterruptInfo {
                interrupt_line: 4,
                level_triggered: true,
                edge_triggered: false,
                active: false,
            }),
            registers: vec![
                DeviceRegister {
                    offset: 0,
                    size: 1,
                    access: DeviceAccess::READ | DeviceAccess::WRITE,
                    reset_value: 0x00,
                    volatile: true,
                },
            ],
            capabilities: Vec::new(),
            stats: DeviceStats {
                read_count: 0,
                write_count: 0,
                interrupt_count: 0,
                error_count: 0,
                last_access_time: 0,
            },
        })
    }
    
    /// Build keyboard controller device
    fn build_keyboard_controller(&self) -> Result<VirtualDevice, HypervisorError> {
        Ok(VirtualDevice {
            device_type: DeviceType::KeyboardController,
            device_id: String::new(),
            name: String::from("PS/2 Keyboard Controller"),
            state: DeviceState::Uninitialized,
            config: DeviceConfig {
                enabled: true,
                address: 0x60,
                interrupt_line: Some(1),
                dma_channels: Vec::new(),
                custom_config: BTreeMap::new(),
            },
            mmio_regions: Vec::new(),
            io_ports: vec![
                IoPortRange {
                    base_port: 0x60,
                    size: 2,
                    access: DeviceAccess::READ | DeviceAccess::WRITE,
                }
            ],
            interrupt: Some(InterruptInfo {
                interrupt_line: 1,
                level_triggered: true,
                edge_triggered: false,
                active: false,
            }),
            registers: vec![
                DeviceRegister {
                    offset: 0,
                    size: 1,
                    access: DeviceAccess::READ | DeviceAccess::WRITE,
                    reset_value: 0xFF,
                    volatile: true,
                },
            ],
            capabilities: Vec::new(),
            stats: DeviceStats {
                read_count: 0,
                write_count: 0,
                interrupt_count: 0,
                error_count: 0,
                last_access_time: 0,
            },
        })
    }
    
    /// Handle device read operation
    pub fn handle_device_read(&mut self, device_id: &str, offset: u64, size: usize) -> Result<u64, HypervisorError> {
        if let Some(device) = self.devices.get(device_id) {
            let mut device = device.write();
            device.stats.read_count += 1;
            
            match device.device_type {
                DeviceType::EducationalDemo => {
                    // Simulate educational demo device read
                    Ok(self.read_educational_demo(&device, offset, size))
                },
                DeviceType::SerialPort => {
                    // Simulate serial port read
                    Ok(0) // No data available
                },
                DeviceType::KeyboardController => {
                    // Simulate keyboard controller read
                    Ok(0x00) // No key pressed
                },
                _ => {
                    device.stats.error_count += 1;
                    Err(HypervisorError::IoError(String::from("Unsupported device read")))
                },
            }
        } else {
            Err(HypervisorError::IoError(format!("Device {} not found", device_id)))
        }
    }
    
    /// Handle device write operation
    pub fn handle_device_write(&mut self, device_id: &str, offset: u64, value: u64, size: usize) -> Result<(), HypervisorError> {
        if let Some(device) = self.devices.get(device_id) {
            let mut device = device.write();
            device.stats.write_count += 1;
            
            match device.device_type {
                DeviceType::EducationalDemo => {
                    self.write_educational_demo(&device, offset, value, size);
                },
                DeviceType::SerialPort => {
                    // Handle serial port write
                    info!("Serial write: 0x{:02x} to offset 0x{:x}", value, offset);
                },
                DeviceType::KeyboardController => {
                    // Handle keyboard controller write
                    info!("Keyboard write: 0x{:02x} to offset 0x{:x}", value, offset);
                },
                _ => {
                    device.stats.error_count += 1;
                    return Err(HypervisorError::IoError(String::from("Unsupported device write")));
                },
            }
            
            Ok(())
        } else {
            Err(HypervisorError::IoError(format!("Device {} not found", device_id)))
        }
    }
    
    /// Handle educational demo device read
    fn read_educational_demo(&self, device: &VirtualDevice, offset: u64, size: usize) -> u64 {
        match offset {
            0x00 => {
                // Demo status register
                0x01 // Device ready
            },
            0x04 => {
                // Demo data register
                0x42 // Sample data
            },
            _ => {
                0x00
            }
        }
    }
    
    /// Handle educational demo device write
    fn write_educational_demo(&self, device: &VirtualDevice, offset: u64, value: u64, size: usize) {
        match offset {
            0x00 => {
                // Demo control register
                info!("Demo device control: 0x{:02x}", value);
            },
            0x04 => {
                // Demo data register
                info!("Demo device data: 0x{:02x}", value);
            },
            0x08 => {
                // Demo LED register
                info!("Demo device LED: 0x{:02x}", value);
            },
            _ => {
                // Unknown register
                warn!("Demo device write to unknown offset: 0x{:x} = 0x{:02x}", offset, value);
            },
        }
    }
    
    /// Initialize all devices
    pub fn initialize_devices(&mut self) -> Result<(), HypervisorError> {
        for (device_id, device) in &self.devices {
            let mut device = device.write();
            
            match device.device_type {
                DeviceType::EducationalDemo => {
                    device.state = DeviceState::Ready;
                    info!("Initialized educational demo device");
                },
                DeviceType::SerialPort => {
                    device.state = DeviceState::Ready;
                    info!("Initialized serial port");
                },
                DeviceType::KeyboardController => {
                    device.state = DeviceState::Ready;
                    info!("Initialized keyboard controller");
                },
                _ => {
                    device.state = DeviceState::Initialized;
                    info!("Initialized device {}", device_id);
                },
            }
        }
        
        info!("Initialized {} devices", self.devices.len());
        Ok(())
    }
    
    /// Generate device report
    pub fn generate_device_report(&self) -> String {
        let mut report = String::new();
        report.push_str(&format!("Device Framework Report for VM {}\n", self.vm_id.0));
        report.push_str(&format!("Total devices: {}\n", self.devices.len()));
        report.push_str("==================\n\n");
        
        for (device_id, device) in &self.devices {
            let device = device.read();
            report.push_str(&format!("Device: {}\n", device_id));
            report.push_str(&format!("  Name: {}\n", device.name));
            report.push_str(&format!("  Type: {:?}\n", device.device_type));
            report.push_str(&format!("  State: {:?}\n", device.state));
            report.push_str(&format!("  Reads: {}\n", device.stats.read_count));
            report.push_str(&format!("  Writes: {}\n", device.stats.write_count));
            report.push_str(&format!("  Interrupts: {}\n", device.stats.interrupt_count));
            report.push_str(&format!("  Errors: {}\n\n", device.stats.error_count));
        }
        
        report
    }
    
    /// Get device list
    pub fn get_device_list(&self) -> Vec<String> {
        self.devices.keys().cloned().collect()
    }
    
    /// Find device by type
    pub fn find_device_by_type(&self, device_type: DeviceType) -> Option<String> {
        for (device_id, device) in &self.devices {
            if device.read().device_type == device_type {
                return Some(device_id.clone());
            }
        }
        None
    }
}