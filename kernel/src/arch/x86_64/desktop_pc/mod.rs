//! x86_64 Desktop PC Support Module
//! 
//! This module provides comprehensive support for x86_64 desktop PCs including
//! BIOS/UEFI compatibility, multi-core CPU support, device enumeration, and
//! desktop-specific features.

use crate::log::{info, warn, error};
use crate::KernelError;

pub mod bios_uefi;
pub mod cpu_manager;
pub mod instruction_sets;
pub mod acpi;
pub mod pci;
pub mod storage_drivers;
pub mod network_drivers;
pub mod desktop_features;
pub mod optimization;

pub use bios_uefi::{FirmwareInfo, MemoryRegion, MemoryRegionType, BiosInfo};
pub use cpu_manager::{CpuManager, CpuInfo, CpuVendor, CpuTopology, CpuState, ApicInfo, PmcInfo, CpuPowerInfo};
pub use instruction_sets::{SimdMathOps, InstructionSetCapabilities, SimdOperationResult, SimdPerformanceMetrics, F32x4, F64x2, I32x4, I64x2, F32x8, F64x4};
pub use acpi::{AcpiManager, AcpiPowerState, AcpiSleepType, AcpiProcessorState, ThermalZone, BatteryInfo, BatteryState};
pub use pci::{PciManager, PciBusInfo, PciCapabilityInfo, PciExpressInfo, PciInterruptInfo, PciResourceInfo, PciBarInfo};
pub use storage_drivers::{StorageManager, SataDevice, SataDeviceType, AhciController, AhciPort, NvmeDevice, DeviceHealthInfo};
pub use network_drivers::{NetworkManager, EthernetDevice, LinkStatus, WirelessDevice, WirelessStandard, WirelessMode, NetworkStats, NetworkBuffer, IpAddress, MacAddress};
pub use desktop_features::{DesktopManager, UsbManager, UsbHubInfo, UsbDevice, UsbDeviceType, GraphicsManager, GraphicsInfo, GraphicsMode, PixelFormat, MonitorLayout, MultiMonitorConfig};
pub use optimization::{OptimizationManager, OptimizationProfile, PerformanceImprovement, CpuTuningParams, MemoryTuningParams, IoTuningParams, AccessPattern};

/// Desktop PC system information
#[derive(Debug, Clone)]
pub struct DesktopPcInfo {
    pub firmware_type: FirmwareType,
    pub cpu_vendor: String,
    pub cpu_brand: String,
    pub cpu_cores: u32,
    pub cpu_threads: u32,
    pub cpu_frequency_mhz: u32,
    pub memory_size_mb: u32,
    pub pci_devices: Vec<PciDeviceInfo>,
    pub storage_devices: Vec<StorageDeviceInfo>,
    pub network_interfaces: Vec<NetworkInterfaceInfo>,
    pub usb_controllers: Vec<UsbControllerInfo>,
    pub display_devices: Vec<DisplayDeviceInfo>,
    pub acpi_tables: Vec<String>,
    pub supported_features: SupportedFeatures,
}

/// Firmware types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FirmwareType {
    LegacyBios,
    Uefi,
    Coreboot,
    OpenFirmware,
}

/// PCI device information
#[derive(Debug, Clone)]
pub struct PciDeviceInfo {
    pub bus: u8,
    pub device: u8,
    pub function: u8,
    pub vendor_id: u16,
    pub device_id: u16,
    pub class_code: u8,
    pub subclass: u8,
    pub prog_if: u8,
    pub device_name: String,
    pub driver_attached: bool,
}

/// Storage device information
#[derive(Debug, Clone)]
pub struct StorageDeviceInfo {
    pub device_type: StorageDeviceType,
    pub controller_type: ControllerType,
    pub capacity_bytes: u64,
    pub sector_size: u32,
    pub max_transfer_size: usize,
    pub device_name: String,
    pub driver_attached: bool,
}

/// Storage device types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StorageDeviceType {
    SataHdd,
    SataSsd,
    NvmeSsd,
    M2Sata,
    M2Nvme,
    UsbStorage,
    MemoryCard,
}

/// Controller types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ControllerType {
    Ahci,
    Nvme,
    Pata,
    UsbMassStorage,
    SdController,
}

/// Network interface information
#[derive(Debug, Clone)]
pub struct NetworkInterfaceInfo {
    pub interface_type: NetworkInterfaceType,
    pub mac_address: [u8; 6],
    pub pci_location: Option<(u8, u8, u8)>,
    pub speed_mbps: u32,
    pub duplex: NetworkDuplex,
    pub driver_attached: bool,
}

/// Network interface types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NetworkInterfaceType {
    Ethernet,
    Wireless,
    Bluetooth,
}

/// Network duplex modes
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NetworkDuplex {
    Half,
    Full,
}

/// USB controller information
#[derive(Debug, Clone)]
pub struct UsbControllerInfo {
    pub controller_type: UsbControllerType,
    pub pci_location: Option<(u8, u8, u8)>,
    pub usb_version: UsbVersion,
    pub ports: u8,
    pub driver_attached: bool,
}

/// USB controller types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UsbControllerType {
    Ehci,
    Xhci,
    Ohci,
    Uhci,
}

/// USB versions
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UsbVersion {
    Usb1_1,
    Usb2_0,
    Usb3_0,
    Usb3_1,
    Usb3_2,
    Usb4_0,
}

/// Display device information
#[derive(Debug, Clone)]
pub struct DisplayDeviceInfo {
    pub display_type: DisplayType,
    pub resolution: (u32, u32),
    pub refresh_rate_hz: u32,
    pub color_depth: u8,
    pub connection_type: ConnectionType,
    pub edid_info: Option<Vec<u8>>,
}

/// Display types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DisplayType {
    Monitor,
    LaptopScreen,
    Projector,
    Tv,
}

/// Connection types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConnectionType {
    Hdmi,
    Dvi,
    Vga,
    DisplayPort,
    UsbC,
}

/// Supported features
#[derive(Debug, Clone)]
pub struct SupportedFeatures {
    pub sse: bool,
    pub sse2: bool,
    pub sse3: bool,
    pub sse4_1: bool,
    pub sse4_2: bool,
    pub avx: bool,
    pub avx2: bool,
    pub avx512: bool,
    pub fma: bool,
    pub bmi1: bool,
    pub bmi2: bool,
    pub sha: bool,
    pub aes_ni: bool,
    pub rdrand: bool,
    pub rdseed: bool,
    pub clmul: bool,
    pub movbe: bool,
    pub popcnt: bool,
    pub lzcnt: bool,
    pub cmov: bool,
    pub fcmov: bool,
}

/// Desktop PC system state
pub struct DesktopPcSystem {
    pub initialized: bool,
    pub firmware_info: FirmwareInfo,
    pub cpu_info: CpuManager,
    pub acpi_manager: acpi::AcpiManager,
    pub pci_manager: pci::PciManager,
    pub storage_manager: storage_drivers::StorageManager,
    pub network_manager: network_drivers::NetworkManager,
    pub usb_manager: desktop_features::UsbManager,
    pub graphics_manager: desktop_features::GraphicsManager,
}

/// Firmware information
#[derive(Debug, Clone)]
pub struct FirmwareInfo {
    pub firmware_type: FirmwareType,
    pub version: String,
    pub vendor: String,
    pub boot_services_available: bool,
    pub runtime_services_available: bool,
    pub memory_map: Vec<MemoryRegion>,
}

/// Memory region information
#[derive(Debug, Clone)]
pub struct MemoryRegion {
    pub start_addr: u64,
    pub size: usize,
    pub region_type: MemoryRegionType,
}

/// Memory region types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MemoryRegionType {
    Usable,
    Reserved,
    AcpiReclaimable,
    AcpiNvs,
    BadMemory,
    BootLoader,
    KernelCode,
    KernelData,
}

/// Initialize desktop PC support
pub fn init_desktop_pc() -> Result<(), KernelError> {
    info!("Initializing x86_64 Desktop PC Support...");
    
    let mut system = DesktopPcSystem {
        initialized: false,
        firmware_info: FirmwareInfo {
            firmware_type: FirmwareType::LegacyBios,
            version: String::new(),
            vendor: String::new(),
            boot_services_available: false,
            runtime_services_available: false,
            memory_map: Vec::new(),
        },
        cpu_info: cpu_manager::CpuManager::new(),
        acpi_manager: acpi::AcpiManager::new(),
        pci_manager: pci::PciManager::new(),
        storage_manager: storage_drivers::StorageManager::new(),
        network_manager: network_drivers::NetworkManager::new(),
        usb_manager: desktop_features::UsbManager::new(),
        graphics_manager: desktop_features::GraphicsManager::new(),
    };
    
    // Step 1: Detect firmware type (BIOS/UEFI)
    info!("Detecting firmware type...");
    system.firmware_info.firmware_type = bios_uefi::detect_firmware_type()?;
    
    // Step 2: Initialize firmware services
    info!("Initializing firmware services...");
    bios_uefi::init_firmware_services(&mut system.firmware_info)?;
    
    // Step 3: Initialize CPU management
    info!("Initializing CPU management...");
    system.cpu_info.initialize()?;
    
    // Step 4: Initialize ACPI
    info!("Initializing ACPI...");
    system.acpi_manager.initialize(&system.firmware_info)?;
    
    // Step 5: Initialize PCI/PCIe
    info!("Initializing PCI/PCIe subsystem...");
    system.pci_manager.initialize()?;
    
    // Step 6: Initialize storage drivers
    info!("Initializing storage drivers...");
    system.storage_manager.initialize(&system.pci_manager)?;
    
    // Step 7: Initialize network drivers
    info!("Initializing network drivers...");
    system.network_manager.initialize(&system.pci_manager)?;
    
    // Step 8: Initialize USB support
    info!("Initializing USB support...");
    system.usb_manager.initialize(&system.pci_manager)?;
    
    // Step 9: Initialize graphics support
    info!("Initializing graphics support...");
    system.graphics_manager.initialize(&system.pci_manager)?;
    
    // Step 10: Optimize for desktop workloads
    info!("Applying desktop-specific optimizations...");
    optimization::apply_desktop_optimizations(&system)?;
    
    info!("Desktop PC initialization complete!");
    
    Ok(())
}

/// Get desktop PC system information
pub fn get_desktop_pc_info() -> DesktopPcInfo {
    // This would collect information from all subsystems
    DesktopPcInfo {
        firmware_type: FirmwareType::LegacyBios, // Would be detected
        cpu_vendor: "GenuineIntel".to_string(), // Would be detected via CPUID
        cpu_brand: "Unknown CPU".to_string(),
        cpu_cores: 4,
        cpu_threads: 8,
        cpu_frequency_mhz: 3600,
        memory_size_mb: 16384,
        pci_devices: Vec::new(),
        storage_devices: Vec::new(),
        network_interfaces: Vec::new(),
        usb_controllers: Vec::new(),
        display_devices: Vec::new(),
        acpi_tables: Vec::new(),
        supported_features: SupportedFeatures {
            sse: true,
            sse2: true,
            sse3: true,
            sse4_1: true,
            sse4_2: true,
            avx: true,
            avx2: true,
            avx512: false, // Would be detected
            fma: true,
            bmi1: true,
            bmi2: true,
            sha: true,
            aes_ni: true,
            rdrand: true,
            rdseed: true,
            clmul: true,
            movbe: true,
            popcnt: true,
            lzcnt: true,
            cmov: true,
            fcmov: false,
        },
    }
}