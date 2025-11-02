//! VM Configuration and Types
//! 
//! Defines the configuration structure for virtual machines and error types
//! used throughout the hypervisor system.

use alloc::string::String;
use bitflags::bitflags;

/// Virtual Machine ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct VmId(pub u32);

impl VmId {
    /// Create a new VM ID
    pub fn new(id: u32) -> Self {
        VmId(id)
    }
    
    /// Get the raw ID value
    pub fn as_u32(&self) -> u32 {
        self.0
    }
}

/// Virtual Machine Configuration
#[derive(Debug, Clone)]
pub struct VmConfig {
    /// VM name for identification
    pub name: String,
    /// Number of virtual CPUs
    pub vcpu_count: usize,
    /// Memory allocation in MB
    pub memory_mb: u64,
    /// CPU architecture type
    pub arch: VmArchitecture,
    /// Boot configuration
    pub boot: BootConfig,
    /// Device configuration
    pub devices: DeviceConfig,
    /// Feature flags
    pub features: VmFeatures,
    /// Network configuration
    pub network: NetworkConfig,
    /// Storage configuration
    pub storage: StorageConfig,
    /// Security configuration
    pub security: SecurityConfig,
}

impl VmConfig {
    /// Create a minimal VM configuration
    pub fn minimal(name: String, vcpu_count: usize, memory_mb: u64) -> Self {
        VmConfig {
            name,
            vcpu_count,
            memory_mb,
            arch: VmArchitecture::X86_64,
            boot: BootConfig::default(),
            devices: DeviceConfig::default(),
            features: VmFeatures::empty(),
            network: NetworkConfig::default(),
            storage: StorageConfig::default(),
            security: SecurityConfig::default(),
        }
    }
    
    /// Create an educational VM configuration
    pub fn educational(name: String) -> Self {
        VmConfig {
            name,
            vcpu_count: 1,
            memory_mb: 512,
            arch: VmArchitecture::X86_64,
            boot: BootConfig::default(),
            devices: DeviceConfig::educational(),
            features: VmFeatures::EDUCATIONAL | VmFeatures::DEBUG,
            network: NetworkConfig::disabled(),
            storage: StorageConfig::minimal(),
            security: SecurityConfig::default(),
        }
    }
    
    /// Create a nested virtualization configuration
    pub fn nested(name: String, host_vcpu_count: usize) -> Self {
        VmConfig {
            name,
            vcpu_count: host_vcpu_count,
            memory_mb: 4096,
            arch: VmArchitecture::X86_64,
            boot: BootConfig::default(),
            devices: DeviceConfig::nested(),
            features: VmFeatures::NESTED | VmFeatures::RESOURCE_MONITORING,
            network: NetworkConfig::default(),
            storage: StorageConfig::nested(),
            security: SecurityConfig::default(),
        }
    }
}

/// CPU Architecture for VMs
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VmArchitecture {
    /// x86 64-bit architecture
    X86_64,
    /// AMD64 architecture (alias for x86_64)
    AMD64,
    /// ARM 64-bit architecture
    AArch64,
    /// ARM 32-bit architecture
    ARMv7,
}

/// Boot Configuration
#[derive(Debug, Clone)]
pub struct BootConfig {
    /// Boot device order
    pub boot_order: BootOrder,
    /// Boot kernel path (if using direct kernel boot)
    pub kernel_path: Option<String>,
    /// Boot initrd path (if using direct kernel boot)
    pub initrd_path: Option<String>,
    /// Boot parameters
    pub kernel_args: String,
    /// Boot timeout in seconds
    pub timeout_sec: u32,
}

impl Default for BootConfig {
    fn default() -> Self {
        BootConfig {
            boot_order: BootOrder::default(),
            kernel_path: None,
            initrd_path: None,
            kernel_args: String::new(),
            timeout_sec: 10,
        }
    }
}

/// Boot device order
#[derive(Debug, Clone, Copy)]
pub enum BootOrder {
    /// Boot from disk first
    DiskFirst,
    /// Boot from network first
    NetworkFirst,
    /// Boot from CD-ROM first
    CdromFirst,
    /// Custom boot order
    Custom([BootDevice; 4]),
}

impl Default for BootOrder {
    fn default() -> Self {
        BootOrder::DiskFirst
    }
}

/// Boot device types
#[derive(Debug, Clone, Copy)]
pub enum BootDevice {
    HardDisk,
    CDROM,
    Network,
    USB,
}

/// Device Configuration
#[derive(Debug, Clone)]
pub struct DeviceConfig {
    /// Graphics device configuration
    pub graphics: GraphicsConfig,
    /// Network adapter configuration
    pub network_adapters: Vec<NetworkAdapterConfig>,
    /// Storage device configuration
    pub storage_devices: Vec<StorageDeviceConfig>,
    /// Serial console configuration
    pub serial_console: SerialConfig,
    /// Audio device configuration
    pub audio: AudioConfig,
    /// USB controller configuration
    pub usb: UsbConfig,
}

impl Default for DeviceConfig {
    fn default() -> Self {
        DeviceConfig {
            graphics: GraphicsConfig::default(),
            network_adapters: Vec::new(),
            storage_devices: Vec::new(),
            serial_console: SerialConfig::default(),
            audio: AudioConfig::disabled(),
            usb: UsbConfig::disabled(),
        }
    }
}

impl DeviceConfig {
    /// Create educational device configuration
    pub fn educational() -> Self {
        DeviceConfig {
            graphics: GraphicsConfig::vga(),
            network_adapters: Vec::new(),
            storage_devices: vec![StorageDeviceConfig::minimal()],
            serial_console: SerialConfig::enabled(),
            audio: AudioConfig::disabled(),
            usb: UsbConfig::disabled(),
        }
    }
    
    /// Create nested virtualization device configuration
    pub fn nested() -> Self {
        DeviceConfig {
            graphics: GraphicsConfig::default(),
            network_adapters: vec![NetworkAdapterConfig::default()],
            storage_devices: vec![StorageDeviceConfig::large()],
            serial_console: SerialConfig::default(),
            audio: AudioConfig::disabled(),
            usb: UsbConfig::default(),
        }
    }
}

/// Graphics Configuration
#[derive(Debug, Clone)]
pub struct GraphicsConfig {
    /// Graphics card type
    pub card_type: GraphicsCardType,
    /// Screen resolution
    pub resolution: (u32, u32),
    /// Display count
    pub display_count: u32,
}

impl Default for GraphicsConfig {
    fn default() -> Self {
        GraphicsConfig {
            card_type: GraphicsCardType::VGA,
            resolution: (1024, 768),
            display_count: 1,
        }
    }
}

impl GraphicsConfig {
    /// Create VGA graphics configuration
    pub fn vga() -> Self {
        GraphicsConfig {
            card_type: GraphicsCardType::VGA,
            resolution: (800, 600),
            display_count: 1,
        }
    }
}

/// Graphics Card Types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GraphicsCardType {
    /// Standard VGA graphics
    VGA,
    /// Virtualized GPU
    VGPU,
    /// No graphics (headless)
    Headless,
}

/// Network Adapter Configuration
#[derive(Debug, Clone)]
pub struct NetworkAdapterConfig {
    /// Network interface name
    pub interface_name: String,
    /// Network mode
    pub mode: NetworkMode,
    /// MAC address
    pub mac_address: [u8; 6],
    /// VLAN ID (if applicable)
    pub vlan_id: Option<u16>,
}

impl Default for NetworkAdapterConfig {
    fn default() -> Self {
        NetworkAdapterConfig {
            interface_name: String::from("eth0"),
            mode: NetworkMode::NAT,
            mac_address: [0x52, 0x54, 0x00, 0x12, 0x34, 0x56],
            vlan_id: None,
        }
    }
}

/// Network Modes
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NetworkMode {
    /// NAT mode (default)
    NAT,
    /// Bridged mode
    Bridge,
    /// Host-only mode
    HostOnly,
    /// No network
    None,
}

/// Storage Device Configuration
#[derive(Debug, Clone)]
pub struct StorageDeviceConfig {
    /// Device type
    pub device_type: StorageDeviceType,
    /// File path (for file-based storage)
    pub file_path: Option<String>,
    /// Device size in bytes
    pub size_bytes: u64,
    /// Read-only flag
    pub read_only: bool,
    /// Cache mode
    pub cache_mode: CacheMode,
}

impl StorageDeviceConfig {
    /// Create minimal storage configuration
    pub fn minimal() -> Self {
        StorageDeviceConfig {
            device_type: StorageDeviceType::Qcow2,
            file_path: Some(String::from("disk.qcow2")),
            size_bytes: 10 * 1024 * 1024 * 1024, // 10GB
            read_only: false,
            cache_mode: CacheMode::WriteBack,
        }
    }
    
    /// Create large storage configuration
    pub fn large() -> Self {
        StorageDeviceConfig {
            device_type: StorageDeviceType::Qcow2,
            file_path: Some(String::from("large_disk.qcow2")),
            size_bytes: 100 * 1024 * 1024 * 1024, // 100GB
            read_only: false,
            cache_mode: CacheMode::WriteBack,
        }
    }
}

/// Storage Device Types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StorageDeviceType {
    /// QEMU Copy-on-Write format
    Qcow2,
    /// Raw disk image
    Raw,
    /// VMDK format
    VMDK,
    /// Physical disk
    Physical,
}

/// Cache Modes
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CacheMode {
    /// No caching
    None,
    /// Write-through cache
    WriteThrough,
    /// Write-back cache
    WriteBack,
}

/// Serial Console Configuration
#[derive(Debug, Clone)]
pub struct SerialConfig {
    /// Enable serial console
    pub enabled: bool,
    /// Serial port number
    pub port: SerialPort,
    /// Baud rate
    pub baud_rate: u32,
}

impl Default for SerialConfig {
    fn default() -> Self {
        SerialConfig {
            enabled: true,
            port: SerialPort::COM1,
            baud_rate: 115200,
        }
    }
}

impl SerialConfig {
    /// Enable serial console
    pub fn enabled() -> Self {
        SerialConfig {
            enabled: true,
            port: SerialPort::COM1,
            baud_rate: 115200,
        }
    }
    
    /// Disable serial console
    pub fn disabled() -> Self {
        SerialConfig {
            enabled: false,
            port: SerialPort::COM1,
            baud_rate: 115200,
        }
    }
}

/// Serial Port types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SerialPort {
    COM1,
    COM2,
    COM3,
    COM4,
}

/// Audio Configuration
#[derive(Debug, Clone)]
pub struct AudioConfig {
    /// Enable audio support
    pub enabled: bool,
    /// Audio driver type
    pub driver: AudioDriver,
}

impl AudioConfig {
    /// Disabled audio configuration
    pub fn disabled() -> Self {
        AudioConfig {
            enabled: false,
            driver: AudioDriver::AC97,
        }
    }
}

/// Audio Driver types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AudioDriver {
    AC97,
    HDA,
    None,
}

/// USB Configuration
#[derive(Debug, Clone)]
pub struct UsbConfig {
    /// Enable USB support
    pub enabled: bool,
    /// USB version
    pub version: UsbVersion,
}

impl Default for UsbConfig {
    fn default() -> Self {
        UsbConfig {
            enabled: false,
            version: UsbVersion::USB2,
        }
    }
}

impl UsbConfig {
    /// Disabled USB configuration
    pub fn disabled() -> Self {
        UsbConfig {
            enabled: false,
            version: UsbVersion::USB2,
        }
    }
}

/// USB Version types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UsbVersion {
    USB1,
    USB2,
    USB3,
}

/// Network Configuration
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    /// Enable networking
    pub enabled: bool,
    /// Network bridge name (for bridged mode)
    pub bridge_name: String,
    /// DNS servers
    pub dns_servers: Vec<String>,
    /// Default gateway
    pub default_gateway: Option<String>,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        NetworkConfig {
            enabled: true,
            bridge_name: String::from("virbr0"),
            dns_servers: vec![String::from("8.8.8.8")],
            default_gateway: None,
        }
    }
}

impl NetworkConfig {
    /// Disabled network configuration
    pub fn disabled() -> Self {
        NetworkConfig {
            enabled: false,
            bridge_name: String::new(),
            dns_servers: Vec::new(),
            default_gateway: None,
        }
    }
}

/// Storage Configuration
#[derive(Debug, Clone)]
pub struct StorageConfig {
    /// Default storage pool
    pub default_pool: String,
    /// Auto-create storage pools
    pub auto_create_pools: bool,
    /// Storage format
    pub default_format: StorageDeviceType,
}

impl Default for StorageConfig {
    fn default() -> Self {
        StorageConfig {
            default_pool: String::from("default"),
            auto_create_pools: true,
            default_format: StorageDeviceType::Qcow2,
        }
    }
}

impl StorageConfig {
    /// Minimal storage configuration
    pub fn minimal() -> Self {
        StorageConfig {
            default_pool: String::from("default"),
            auto_create_pools: false,
            default_format: StorageDeviceType::Qcow2,
        }
    }
    
    /// Nested virtualization storage configuration
    pub fn nested() -> Self {
        StorageConfig {
            default_pool: String::from("nested_pool"),
            auto_create_pools: true,
            default_format: StorageDeviceType::Qcow2,
        }
    }
}

/// Security Configuration
#[derive(Debug, Clone)]
pub struct SecurityConfig {
    /// Enable SELinux integration
    pub selinux_enabled: bool,
    /// Enable AppArmor integration
    pub apparmor_enabled: bool,
    /// Security model
    pub security_model: SecurityModel,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        SecurityConfig {
            selinux_enabled: false,
            apparmor_enabled: false,
            security_model: SecurityModel::None,
        }
    }
}

/// Security Models
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SecurityModel {
    None,
    SELinux,
    AppArmor,
}

/// VM Feature flags
bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct VmFeatures: u32 {
        const DEBUG = 1 << 0;
        const EDUCATIONAL = 1 << 1;
        const NESTED = 1 << 2;
        const REAL_TIME = 1 << 3;
        const HIGH_PERFORMANCE = 1 << 4;
        const RESOURCE_MONITORING = 1 << 5;
        const SNAPSHOT_SUPPORT = 1 << 6;
        const MIGRATION_SUPPORT = 1 << 7;
        const LIVE_MIGRATION = 1 << 8;
        const KERNEL_DEBUG = 1 << 9;
    }
}

/// Hypervisor Error types
#[derive(Debug, Clone, PartialEq)]
pub enum HypervisorError {
    /// Insufficient hardware support
    InsufficientHardwareSupport,
    /// Too many VMs created
    TooManyVms,
    /// Too many VCPUs requested
    TooManyVcpus,
    /// VM not found
    VmNotFound,
    /// VCPU not found
    VcpuNotFound,
    /// Invalid VM state
    InvalidVmState,
    /// Invalid VCPU state
    InvalidVcpuState,
    /// Cannot delete running VM
    CannotDeleteRunningVm,
    /// Feature not supported
    FeatureNotSupported,
    /// Configuration error
    ConfigurationError(String),
    /// Hardware virtualization not available
    HardwareVirtNotAvailable,
    /// Memory allocation failed
    MemoryAllocationFailed,
    /// I/O error
    IoError(String),
    /// Invalid parameter
    InvalidParameter,
}

/// Convert errors to debug strings
impl core::fmt::Display for HypervisorError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            HypervisorError::InsufficientHardwareSupport => {
                write!(f, "Insufficient hardware virtualization support")
            },
            HypervisorError::TooManyVms => write!(f, "Too many virtual machines"),
            HypervisorError::TooManyVcpus => write!(f, "Too many VCPUs requested"),
            HypervisorError::VmNotFound => write!(f, "Virtual machine not found"),
            HypervisorError::VcpuNotFound => write!(f, "VCPU not found"),
            HypervisorError::InvalidVmState => write!(f, "Invalid virtual machine state"),
            HypervisorError::InvalidVcpuState => write!(f, "Invalid VCPU state"),
            HypervisorError::CannotDeleteRunningVm => write!(f, "Cannot delete running virtual machine"),
            HypervisorError::FeatureNotSupported => write!(f, "Feature not supported"),
            HypervisorError::ConfigurationError(msg) => write!(f, "Configuration error: {}", msg),
            HypervisorError::HardwareVirtNotAvailable => write!(f, "Hardware virtualization not available"),
            HypervisorError::MemoryAllocationFailed => write!(f, "Memory allocation failed"),
            HypervisorError::IoError(msg) => write!(f, "I/O error: {}", msg),
            HypervisorError::InvalidParameter => write!(f, "Invalid parameter"),
        }
    }
}