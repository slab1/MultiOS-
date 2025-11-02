//! Network driver framework
//!
//! This module provides a comprehensive framework for implementing network interface
//! drivers across different hardware platforms and operating systems.

use crate::{Result, NetworkError};
use crate::core::{NetworkInterface, NetworkDriver, InterfaceStatus, InterfaceStats, IpAddress};
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Network driver types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriverType {
    /// Ethernet driver
    Ethernet,
    /// WiFi driver
    WiFi,
    /// Virtual network driver
    Virtual,
    /// Loopback driver
    Loopback,
    /// USB network adapter driver
    UsbNet,
    /// PCI network card driver
    PciNet,
    /// Bluetooth network driver
    Bluetooth,
    /// Serial line interface driver
    Slip,
    /// PPP driver
    Ppp,
    /// Generic driver
    Generic,
}

/// Driver capabilities
#[derive(Debug, Clone, Copy)]
pub struct DriverCapabilities {
    /// Can send and receive packets
    pub can_transmit: bool,
    /// Supports hardware checksum offload
    pub checksum_offload: bool,
    /// Supports hardware encryption
    pub encryption_offload: bool,
    /// Supports VLAN tagging
    pub vlan_tagging: bool,
    /// Supports multicast
    pub multicast_support: bool,
    /// Supports promiscuous mode
    pub promiscuous_mode: bool,
    /// Supports link state detection
    pub link_state_detection: bool,
    /// Maximum transmission unit
    pub max_mtu: u16,
    /// Current MTU
    pub current_mtu: u16,
    /// Hardware address (MAC) length
    pub hw_addr_len: usize,
}

impl Default for DriverCapabilities {
    fn default() -> Self {
        Self {
            can_transmit: true,
            checksum_offload: false,
            encryption_offload: false,
            vlan_tagging: false,
            multicast_support: true,
            promiscuous_mode: false,
            link_state_detection: true,
            max_mtu: 1500,
            current_mtu: 1500,
            hw_addr_len: 6,
        }
    }
}

/// Network driver configuration
#[derive(Debug, Clone)]
pub struct DriverConfig {
    /// Driver name
    pub name: String,
    /// Driver type
    pub driver_type: DriverType,
    /// Interface name
    pub interface_name: String,
    /// Hardware address
    pub hw_address: Option<[u8; 6]>,
    /// Initial MTU
    pub initial_mtu: u16,
    /// Enable checksum offload
    pub checksum_offload: bool,
    /// Enable promiscuous mode
    pub promiscuous_mode: bool,
    /// DMA buffer size
    pub dma_buffer_size: usize,
    /// Interrupt handling settings
    pub interrupt_settings: InterruptSettings,
    /// Power management settings
    pub power_management: PowerManagementSettings,
}

#[derive(Debug, Clone)]
pub struct InterruptSettings {
    /// Interrupt number
    pub irq: Option<u8>,
    /// Interrupt type
    pub interrupt_type: InterruptType,
    /// Polling interval for non-interrupt drivers
    pub polling_interval: Duration,
}

#[derive(Debug, Clone, Copy)]
pub enum InterruptType {
    /// Hardware interrupts
    Hardware,
    /// Polling mode
    Polling,
    /// MSI (Message Signaled Interrupts)
    Msi,
    /// MSI-X (Extended)
    MsiX,
}

impl Default for InterruptSettings {
    fn default() -> Self {
        Self {
            irq: None,
            interrupt_type: InterruptType::Polling,
            polling_interval: Duration::from_millis(1),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PowerManagementSettings {
    /// Enable power management
    pub enabled: bool,
    /// Wake on LAN support
    pub wake_on_lan: bool,
    /// Power saving timeout
    pub power_save_timeout: Duration,
}

impl Default for PowerManagementSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            wake_on_lan: false,
            power_save_timeout: Duration::from_secs(300),
        }
    }
}

impl Default for DriverConfig {
    fn default() -> Self {
        Self {
            name: "generic".to_string(),
            driver_type: DriverType::Generic,
            interface_name: "eth0".to_string(),
            hw_address: None,
            initial_mtu: 1500,
            checksum_offload: false,
            promiscuous_mode: false,
            dma_buffer_size: 8192,
            interrupt_settings: InterruptSettings::default(),
            power_management: PowerManagementSettings::default(),
        }
    }
}

/// Packet buffer for network transmission
#[derive(Debug, Clone)]
pub struct PacketBuffer {
    /// Packet data
    pub data: Vec<u8>,
    /// Packet length
    pub length: usize,
    /// Source address
    pub source: Option<IpAddress>,
    /// Destination address
    pub dest: Option<IpAddress>,
    /// Timestamp when packet was created
    pub timestamp: Instant,
    /// User data pointer
    pub user_data: Option<Box<dyn std::any::Any + Send + Sync>>,
}

impl PacketBuffer {
    /// Create a new packet buffer
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            data: data.clone(),
            length: data.len(),
            source: None,
            dest: None,
            timestamp: Instant::now(),
            user_data: None,
        }
    }

    /// Create packet buffer with metadata
    pub fn with_metadata(data: Vec<u8>, source: IpAddress, dest: IpAddress) -> Self {
        Self {
            data: data.clone(),
            length: data.len(),
            source: Some(source),
            dest: Some(dest),
            timestamp: Instant::now(),
            user_data: None,
        }
    }

    /// Set user data
    pub fn set_user_data<T: 'static + Send + Sync>(&mut self, data: T) {
        self.user_data = Some(Box::new(data));
    }

    /// Get user data
    pub fn get_user_data<T: 'static>(&self) -> Option<&T> {
        if let Some(user_data) = &self.user_data {
            user_data.downcast_ref::<T>()
        } else {
            None
        }
    }

    /// Get age of packet
    pub fn age(&self) -> Duration {
        Instant::now().duration_since(self.timestamp)
    }

    /// Check if packet is too old
    pub fn is_expired(&self, max_age: Duration) -> bool {
        self.age() > max_age
    }
}

/// Network driver trait
pub trait NetworkDriver: Send + Sync {
    /// Initialize the driver
    fn init(&self, config: &DriverConfig) -> Result<NetworkInterface>;
    
    /// Start the driver
    fn start(&self) -> Result<()>;
    
    /// Stop the driver
    fn stop(&self) -> Result<()>;
    
    /// Send a packet
    fn send_packet(&self, packet: &PacketBuffer) -> Result<()>;
    
    /// Receive a packet
    fn receive_packet(&self) -> Result<Option<PacketBuffer>>;
    
    /// Get driver capabilities
    fn get_capabilities(&self) -> &DriverCapabilities;
    
    /// Get current status
    fn get_status(&self) -> InterfaceStatus;
    
    /// Get hardware address
    fn get_hw_address(&self) -> Result<[u8; 6]>;
    
    /// Set hardware address
    fn set_hw_address(&self, addr: &[u8]) -> Result<()>;
    
    /// Get MTU
    fn get_mtu(&self) -> u16;
    
    /// Set MTU
    fn set_mtu(&self, mtu: u16) -> Result<()>;
    
    /// Get statistics
    fn get_stats(&self) -> Result<InterfaceStats>;
    
    /// Reset statistics
    fn reset_stats(&self) -> Result<()>;
    
    /// Configure interface
    fn configure(&self, ip_addr: IpAddress, netmask: IpAddress, gateway: Option<IpAddress>) -> Result<()>;
    
    /// Enable/disable interface
    fn set_enabled(&self, enabled: bool) -> Result<()>;
    
    /// Get driver information
    fn get_info(&self) -> DriverInfo;
    
    /// Set promiscuous mode
    fn set_promiscuous(&self, enabled: bool) -> Result<()>;
    
    /// Join multicast group
    fn join_multicast_group(&self, addr: IpAddress) -> Result<()>;
    
    /// Leave multicast group
    fn leave_multicast_group(&self, addr: IpAddress) -> Result<()>;
    
    /// Get driver name
    fn get_name(&self) -> &str;
    
    /// Get interface name
    fn get_interface_name(&self) -> &str;
    
    /// Handle interrupt (if applicable)
    fn handle_interrupt(&self) -> Result<()>;
    
    /// Poll for activity (for polling mode drivers)
    fn poll(&self) -> Result<()>;
    
    /// Set power management
    fn set_power_management(&self, settings: &PowerManagementSettings) -> Result<()>;
}

/// Driver information structure
#[derive(Debug, Clone)]
pub struct DriverInfo {
    /// Driver name
    pub name: String,
    /// Driver version
    pub version: String,
    /// Driver type
    pub driver_type: DriverType,
    /// Device vendor
    pub vendor: Option<String>,
    /// Device model
    pub model: Option<String>,
    /// Device revision
    pub revision: Option<String>,
    /// Serial number
    pub serial_number: Option<String>,
    /// Firmware version
    pub firmware_version: Option<String>,
    /// Driver capabilities
    pub capabilities: DriverCapabilities,
    /// Supported features
    pub supported_features: Vec<String>,
}

impl DriverInfo {
    /// Create basic driver info
    pub fn new(name: String, driver_type: DriverType) -> Self {
        Self {
            name,
            version: "1.0.0".to_string(),
            driver_type,
            vendor: None,
            model: None,
            revision: None,
            serial_number: None,
            firmware_version: None,
            capabilities: DriverCapabilities::default(),
            supported_features: Vec::new(),
        }
    }
}

/// Driver manager for managing multiple network drivers
pub struct DriverManager {
    /// Registered drivers
    drivers: HashMap<String, Arc<dyn NetworkDriver>>,
    /// Driver registry
    registry: DriverRegistry,
    /// Configuration manager
    config_manager: DriverConfigManager,
    /// Statistics aggregator
    stats_aggregator: StatsAggregator,
}

impl DriverManager {
    /// Create a new driver manager
    pub fn new() -> Self {
        Self {
            drivers: HashMap::new(),
            registry: DriverRegistry::new(),
            config_manager: DriverConfigManager::new(),
            stats_aggregator: StatsAggregator::new(),
        }
    }

    /// Register a driver
    pub fn register_driver(&mut self, driver: Arc<dyn NetworkDriver>) -> Result<()> {
        let name = driver.get_name().to_string();
        self.drivers.insert(name, driver);
        Ok(())
    }

    /// Unregister a driver
    pub fn unregister_driver(&mut self, name: &str) -> Result<()> {
        self.drivers.remove(name);
        Ok(())
    }

    /// Get driver by name
    pub fn get_driver(&self, name: &str) -> Option<&Arc<dyn NetworkDriver>> {
        self.drivers.get(name)
    }

    /// Get all drivers
    pub fn get_all_drivers(&self) -> Vec<&Arc<dyn NetworkDriver>> {
        self.drivers.values().collect()
    }

    /// Initialize all drivers
    pub fn initialize_all_drivers(&self) -> Result<()> {
        for driver in self.drivers.values() {
            let config = self.config_manager.get_config(driver.get_name())?;
            driver.init(&config)?;
        }
        Ok(())
    }

    /// Start all drivers
    pub fn start_all_drivers(&self) -> Result<()> {
        for driver in self.drivers.values() {
            driver.start()?;
        }
        Ok(())
    }

    /// Stop all drivers
    pub fn stop_all_drivers(&self) -> Result<()> {
        for driver in self.drivers.values() {
            driver.stop()?;
        }
        Ok(())
    }

    /// Get aggregated statistics
    pub fn get_aggregated_stats(&self) -> AggregatedNetworkStats {
        self.stats_aggregator.get_stats(&self.drivers)
    }

    /// Detect and auto-configure drivers
    pub fn auto_configure_drivers(&mut self) -> Result<()> {
        let detected_devices = self.registry.detect_devices();
        
        for device in detected_devices {
            let config = DriverConfig {
                name: format!("{}_{}", device.driver_type.name(), device.index),
                driver_type: device.driver_type,
                interface_name: format!("{}{}", device.interface_prefix, device.index),
                hw_address: device.hw_address,
                ..Default::default()
            };
            
            self.config_manager.register_config(&config.name, config);
        }
        
        Ok(())
    }
}

/// Driver registry for discovering and registering drivers
pub struct DriverRegistry {
    /// Available driver types
    available_drivers: HashMap<DriverType, Box<dyn DriverFactory>>,
    /// Detected devices
    detected_devices: Vec<DetectedDevice>,
}

#[derive(Debug, Clone)]
struct DetectedDevice {
    /// Device type
    pub driver_type: DriverType,
    /// Device index
    pub index: u32,
    /// Hardware address
    pub hw_address: Option<[u8; 6]>,
    /// Interface prefix
    pub interface_prefix: String,
}

impl DriverRegistry {
    /// Create a new driver registry
    pub fn new() -> Self {
        let mut registry = Self {
            available_drivers: HashMap::new(),
            detected_devices: Vec::new(),
        };
        
        registry.register_built_in_drivers();
        registry
    }

    /// Register built-in drivers
    fn register_built_in_drivers(&mut self) {
        // Register loopback driver
        self.available_drivers.insert(DriverType::Loopback, Box::new(LoopbackDriverFactory));
        
        // Register virtual driver
        self.available_drivers.insert(DriverType::Virtual, Box::new(VirtualDriverFactory));
        
        // Register Ethernet driver
        self.available_drivers.insert(DriverType::Ethernet, Box::new(EthernetDriverFactory));
    }

    /// Create driver instance
    pub fn create_driver(&self, driver_type: DriverType) -> Result<Arc<dyn NetworkDriver>> {
        if let Some(factory) = self.available_drivers.get(&driver_type) {
            factory.create_driver()
        } else {
            Err(NetworkError::Other(format!("Driver type {:?} not supported", driver_type).into()))
        }
    }

    /// Detect available devices
    pub fn detect_devices(&mut self) -> &Vec<DetectedDevice> {
        self.detected_devices.clear();
        
        // Simulate device detection
        // In a real implementation, this would scan for actual hardware
        
        // Detect loopback interface
        self.detected_devices.push(DetectedDevice {
            driver_type: DriverType::Loopback,
            index: 0,
            hw_address: Some([0x00, 0x00, 0x00, 0x00, 0x00, 0x00]),
            interface_prefix: "lo".to_string(),
        });
        
        // Detect virtual interfaces
        for i in 0..4 {
            self.detected_devices.push(DetectedDevice {
                driver_type: DriverType::Virtual,
                index: i,
                hw_address: Some([0x52, 0x54, 0x00, 0x12, 0x34, 0x56 + i as u8]),
                interface_prefix: "veth".to_string(),
            });
        }
        
        &self.detected_devices
    }

    /// Get available driver types
    pub fn get_available_driver_types(&self) -> Vec<DriverType> {
        self.available_drivers.keys().cloned().collect()
    }

    /// Check if driver type is supported
    pub fn is_driver_supported(&self, driver_type: DriverType) -> bool {
        self.available_drivers.contains_key(&driver_type)
    }
}

/// Driver factory trait
pub trait DriverFactory {
    fn create_driver(&self) -> Result<Arc<dyn NetworkDriver>>;
}

/// Loopback driver implementation
struct LoopbackDriver;

impl NetworkDriver for LoopbackDriver {
    fn init(&self, config: &DriverConfig) -> Result<NetworkInterface> {
        log::info!("Initializing loopback driver: {}", config.interface_name);
        
        let hw_address = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        
        Ok(NetworkInterface {
            name: config.interface_name.clone(),
            index: 1,
            ip_address: Some(IpAddress::localhost()),
            netmask: Some(IpAddress::v4(255, 0, 0, 0)),
            gateway: None,
            mtu: 65536,
            speed: crate::core::NetworkSpeed::TenGigabit,
            duplex: crate::core::DuplexMode::Full,
            status: InterfaceStatus::Up,
            driver: Arc::new(LoopbackDriver),
        })
    }

    fn start(&self) -> Result<()> {
        log::info!("Starting loopback driver");
        Ok(())
    }

    fn stop(&self) -> Result<()> {
        log::info!("Stopping loopback driver");
        Ok(())
    }

    fn send_packet(&self, packet: &PacketBuffer) -> Result<()> {
        log::debug!("Loopback: Sending {} bytes", packet.length);
        // Loopback driver just drops packets as they're for local testing
        Ok(())
    }

    fn receive_packet(&self) -> Result<Option<PacketBuffer>> {
        // Loopback doesn't receive external packets
        Ok(None)
    }

    fn get_capabilities(&self) -> &DriverCapabilities {
        &DriverCapabilities {
            can_transmit: true,
            checksum_offload: true,
            encryption_offload: false,
            vlan_tagging: false,
            multicast_support: true,
            promiscuous_mode: false,
            link_state_detection: false,
            max_mtu: 65536,
            current_mtu: 65536,
            hw_addr_len: 6,
        }
    }

    fn get_status(&self) -> InterfaceStatus {
        InterfaceStatus::Up
    }

    fn get_hw_address(&self) -> Result<[u8; 6]> {
        Ok([0x00, 0x00, 0x00, 0x00, 0x00, 0x00])
    }

    fn set_hw_address(&self, _addr: &[u8]) -> Result<()> {
        // Loopback doesn't support changing MAC address
        Err(NetworkError::PermissionDenied)
    }

    fn get_mtu(&self) -> u16 {
        65536
    }

    fn set_mtu(&self, _mtu: u16) -> Result<()> {
        // Loopback doesn't support changing MTU
        Err(NetworkError::PermissionDenied)
    }

    fn get_stats(&self) -> Result<InterfaceStats> {
        Ok(InterfaceStats {
            bytes_sent: 0,
            bytes_received: 0,
            packets_sent: 0,
            packets_received: 0,
            errors_sent: 0,
            errors_received: 0,
            dropped_packets: 0,
            collisions: 0,
        })
    }

    fn reset_stats(&self) -> Result<()> {
        Ok(())
    }

    fn configure(&self, _ip_addr: IpAddress, _netmask: IpAddress, _gateway: Option<IpAddress>) -> Result<()> {
        Ok(())
    }

    fn set_enabled(&self, _enabled: bool) -> Result<()> {
        Ok(())
    }

    fn get_info(&self) -> DriverInfo {
        DriverInfo::new("loopback".to_string(), DriverType::Loopback)
    }

    fn set_promiscuous(&self, _enabled: bool) -> Result<()> {
        Ok(())
    }

    fn join_multicast_group(&self, _addr: IpAddress) -> Result<()> {
        Ok(())
    }

    fn leave_multicast_group(&self, _addr: IpAddress) -> Result<()> {
        Ok(())
    }

    fn get_name(&self) -> &str {
        "loopback"
    }

    fn get_interface_name(&self) -> &str {
        "lo"
    }

    fn handle_interrupt(&self) -> Result<()> {
        Ok(())
    }

    fn poll(&self) -> Result<()> {
        Ok(())
    }

    fn set_power_management(&self, _settings: &PowerManagementSettings) -> Result<()> {
        Ok(())
    }
}

struct LoopbackDriverFactory;

impl DriverFactory for LoopbackDriverFactory {
    fn create_driver(&self) -> Result<Arc<dyn NetworkDriver>> {
        Ok(Arc::new(LoopbackDriver))
    }
}

/// Virtual driver implementation
struct VirtualDriver {
    config: DriverConfig,
    stats: Arc<Mutex<InterfaceStats>>,
}

impl VirtualDriver {
    fn new(config: DriverConfig) -> Self {
        Self {
            config,
            stats: Arc::new(Mutex::new(InterfaceStats::default())),
        }
    }
}

impl NetworkDriver for VirtualDriver {
    fn init(&self, config: &DriverConfig) -> Result<NetworkInterface> {
        log::info!("Initializing virtual driver: {}", config.interface_name);
        
        let hw_address = config.hw_address.unwrap_or([0x52, 0x54, 0x00, 0x12, 0x34, 0x56]);
        
        Ok(NetworkInterface {
            name: config.interface_name.clone(),
            index: 0,
            ip_address: None,
            netmask: None,
            gateway: None,
            mtu: config.initial_mtu,
            speed: crate::core::NetworkSpeed::Gigabit,
            duplex: crate::core::DuplexMode::Full,
            status: InterfaceStatus::Down,
            driver: Arc::new(VirtualDriver::new(config.clone())),
        })
    }

    fn start(&self) -> Result<()> {
        log::info!("Starting virtual driver: {}", self.config.interface_name);
        Ok(())
    }

    fn stop(&self) -> Result<()> {
        log::info!("Stopping virtual driver: {}", self.config.interface_name);
        Ok(())
    }

    fn send_packet(&self, packet: &PacketBuffer) -> Result<()> {
        let mut stats = self.stats.lock().unwrap();
        stats.packets_sent += 1;
        stats.bytes_sent += packet.length as u64;
        log::debug!("Virtual driver: Sent {} bytes", packet.length);
        Ok(())
    }

    fn receive_packet(&self) -> Result<Option<PacketBuffer>> {
        // Virtual drivers don't receive packets
        Ok(None)
    }

    fn get_capabilities(&self) -> &DriverCapabilities {
        &DriverCapabilities {
            can_transmit: true,
            checksum_offload: true,
            encryption_offload: false,
            vlan_tagging: true,
            multicast_support: true,
            promiscuous_mode: true,
            link_state_detection: true,
            max_mtu: 9000,
            current_mtu: self.config.initial_mtu,
            hw_addr_len: 6,
        }
    }

    fn get_status(&self) -> InterfaceStatus {
        InterfaceStatus::Up
    }

    fn get_hw_address(&self) -> Result<[u8; 6]> {
        Ok(self.config.hw_address.unwrap_or([0x52, 0x54, 0x00, 0x12, 0x34, 0x56]))
    }

    fn set_hw_address(&self, addr: &[u8]) -> Result<()> {
        if addr.len() != 6 {
            return Err(NetworkError::InvalidAddress);
        }
        log::info!("Virtual driver: MAC address set to {:?}", addr);
        Ok(())
    }

    fn get_mtu(&self) -> u16 {
        self.config.initial_mtu
    }

    fn set_mtu(&self, mtu: u16) -> Result<()> {
        if mtu > 9000 {
            return Err(NetworkError::InvalidAddress);
        }
        log::info!("Virtual driver: MTU set to {}", mtu);
        Ok(())
    }

    fn get_stats(&self) -> Result<InterfaceStats> {
        Ok(self.stats.lock().unwrap().clone())
    }

    fn reset_stats(&self) -> Result<()> {
        *self.stats.lock().unwrap() = InterfaceStats::default();
        Ok(())
    }

    fn configure(&self, _ip_addr: IpAddress, _netmask: IpAddress, _gateway: Option<IpAddress>) -> Result<()> {
        Ok(())
    }

    fn set_enabled(&self, _enabled: bool) -> Result<()> {
        Ok(())
    }

    fn get_info(&self) -> DriverInfo {
        DriverInfo::new("virtual".to_string(), DriverType::Virtual)
    }

    fn set_promiscuous(&self, enabled: bool) -> Result<()> {
        log::info!("Virtual driver: Promiscuous mode {}", if enabled { "enabled" } else { "disabled" });
        Ok(())
    }

    fn join_multicast_group(&self, _addr: IpAddress) -> Result<()> {
        Ok(())
    }

    fn leave_multicast_group(&self, _addr: IpAddress) -> Result<()> {
        Ok(())
    }

    fn get_name(&self) -> &str {
        &self.config.name
    }

    fn get_interface_name(&self) -> &str {
        &self.config.interface_name
    }

    fn handle_interrupt(&self) -> Result<()> {
        Ok(())
    }

    fn poll(&self) -> Result<()> {
        Ok(())
    }

    fn set_power_management(&self, _settings: &PowerManagementSettings) -> Result<()> {
        Ok(())
    }
}

struct VirtualDriverFactory;

impl DriverFactory for VirtualDriverFactory {
    fn create_driver(&self) -> Result<Arc<dyn NetworkDriver>> {
        let config = DriverConfig::default();
        Ok(Arc::new(VirtualDriver::new(config)))
    }
}

/// Ethernet driver implementation (simplified)
struct EthernetDriver {
    config: DriverConfig,
    stats: Arc<Mutex<InterfaceStats>>,
    packet_queue: Arc<Mutex<VecDeque<PacketBuffer>>>,
}

impl EthernetDriver {
    fn new(config: DriverConfig) -> Self {
        Self {
            config,
            stats: Arc::new(Mutex::new(InterfaceStats::default())),
            packet_queue: Arc::new(Mutex::new(VecDeque::new())),
        }
    }
}

impl NetworkDriver for EthernetDriver {
    fn init(&self, config: &DriverConfig) -> Result<NetworkInterface> {
        log::info!("Initializing Ethernet driver: {}", config.interface_name);
        
        let hw_address = config.hw_address.unwrap_or([0x52, 0x54, 0x00, 0x12, 0x34, 0x56]);
        
        Ok(NetworkInterface {
            name: config.interface_name.clone(),
            index: 0,
            ip_address: None,
            netmask: None,
            gateway: None,
            mtu: config.initial_mtu,
            speed: crate::core::NetworkSpeed::Gigabit,
            duplex: crate::core::DuplexMode::Full,
            status: InterfaceStatus::Down,
            driver: Arc::new(EthernetDriver::new(config.clone())),
        })
    }

    fn start(&self) -> Result<()> {
        log::info!("Starting Ethernet driver: {}", self.config.interface_name);
        Ok(())
    }

    fn stop(&self) -> Result<()> {
        log::info!("Stopping Ethernet driver: {}", self.config.interface_name);
        Ok(())
    }

    fn send_packet(&self, packet: &PacketBuffer) -> Result<()> {
        let mut stats = self.stats.lock().unwrap();
        stats.packets_sent += 1;
        stats.bytes_sent += packet.length as u64;
        
        // Simulate packet transmission
        self.p_queue(packet)?;
        log::debug!("Ethernet driver: Sent {} bytes", packet.length);
        Ok(())
    }

    fn p_queue(&self, packet: &PacketBuffer) -> Result<()> {
        let mut queue = self.packet_queue.lock().unwrap();
        queue.push_back(packet.clone());
        Ok(())
    }

    fn receive_packet(&self) -> Result<Option<PacketBuffer>> {
        let mut queue = self.packet_queue.lock().unwrap();
        if let Some(packet) = queue.pop_front() {
            let mut stats = self.stats.lock().unwrap();
            stats.packets_received += 1;
            stats.bytes_received += packet.length as u64;
            Ok(Some(packet))
        } else {
            Ok(None)
        }
    }

    fn get_capabilities(&self) -> &DriverCapabilities {
        &DriverCapabilities {
            can_transmit: true,
            checksum_offload: self.config.checksum_offload,
            encryption_offload: false,
            vlan_tagging: true,
            multicast_support: true,
            promiscuous_mode: self.config.promiscuous_mode,
            link_state_detection: true,
            max_mtu: 9000,
            current_mtu: self.config.initial_mtu,
            hw_addr_len: 6,
        }
    }

    fn get_status(&self) -> InterfaceStatus {
        InterfaceStatus::Up
    }

    fn get_hw_address(&self) -> Result<[u8; 6]> {
        Ok(self.config.hw_address.unwrap_or([0x52, 0x54, 0x00, 0x12, 0x34, 0x56]))
    }

    fn set_hw_address(&self, addr: &[u8]) -> Result<()> {
        if addr.len() != 6 {
            return Err(NetworkError::InvalidAddress);
        }
        log::info!("Ethernet driver: MAC address set to {:?}", addr);
        Ok(())
    }

    fn get_mtu(&self) -> u16 {
        self.config.initial_mtu
    }

    fn set_mtu(&self, mtu: u16) -> Result<()> {
        if mtu > 9000 || mtu < 68 {
            return Err(NetworkError::InvalidAddress);
        }
        log::info!("Ethernet driver: MTU set to {}", mtu);
        Ok(())
    }

    fn get_stats(&self) -> Result<InterfaceStats> {
        Ok(self.stats.lock().unwrap().clone())
    }

    fn reset_stats(&self) -> Result<()> {
        *self.stats.lock().unwrap() = InterfaceStats::default();
        Ok(())
    }

    fn configure(&self, _ip_addr: IpAddress, _netmask: IpAddress, _gateway: Option<IpAddress>) -> Result<()> {
        Ok(())
    }

    fn set_enabled(&self, _enabled: bool) -> Result<()> {
        Ok(())
    }

    fn get_info(&self) -> DriverInfo {
        DriverInfo::new("ethernet".to_string(), DriverType::Ethernet)
    }

    fn set_promiscuous(&self, enabled: bool) -> Result<()> {
        log::info!("Ethernet driver: Promiscuous mode {}", if enabled { "enabled" } else { "disabled" });
        Ok(())
    }

    fn join_multicast_group(&self, _addr: IpAddress) -> Result<()> {
        Ok(())
    }

    fn leave_multicast_group(&self, _addr: IpAddress) -> Result<()> {
        Ok(())
    }

    fn get_name(&self) -> &str {
        &self.config.name
    }

    fn get_interface_name(&self) -> &str {
        &self.config.interface_name
    }

    fn handle_interrupt(&self) -> Result<()> {
        Ok(())
    }

    fn poll(&self) -> Result<()> {
        Ok(())
    }

    fn set_power_management(&self, _settings: &PowerManagementSettings) -> Result<()> {
        Ok(())
    }
}

struct EthernetDriverFactory;

impl DriverFactory for EthernetDriverFactory {
    fn create_driver(&self) -> Result<Arc<dyn NetworkDriver>> {
        let config = DriverConfig::default();
        Ok(Arc::new(EthernetDriver::new(config)))
    }
}

/// Driver configuration manager
struct DriverConfigManager {
    configs: HashMap<String, DriverConfig>,
}

impl DriverConfigManager {
    /// Create a new configuration manager
    fn new() -> Self {
        Self {
            configs: HashMap::new(),
        }
    }

    /// Register driver configuration
    fn register_config(&mut self, name: &str, config: DriverConfig) {
        self.configs.insert(name.to_string(), config);
    }

    /// Get driver configuration
    fn get_config(&self, name: &str) -> Result<DriverConfig> {
        self.configs.get(name)
            .cloned()
            .ok_or_else(|| NetworkError::Other(format!("Configuration not found for driver: {}", name).into()))
    }

    /// Update driver configuration
    fn update_config(&mut self, name: &str, config: DriverConfig) -> Result<()> {
        self.configs.insert(name.to_string(), config);
        Ok(())
    }

    /// Remove driver configuration
    fn remove_config(&mut self, name: &str) -> Result<()> {
        self.configs.remove(name);
        Ok(())
    }
}

/// Statistics aggregator
struct StatsAggregator;

impl StatsAggregator {
    /// Create a new statistics aggregator
    fn new() -> Self {
        Self
    }

    /// Get aggregated statistics from all drivers
    fn get_stats(&self, drivers: &HashMap<String, Arc<dyn NetworkDriver>>) -> AggregatedNetworkStats {
        let mut total_stats = AggregatedNetworkStats::default();
        
        for driver in drivers.values() {
            if let Ok(stats) = driver.get_stats() {
                total_stats.total_bytes_sent += stats.bytes_sent;
                total_stats.total_bytes_received += stats.bytes_received;
                total_stats.total_packets_sent += stats.packets_sent;
                total_stats.total_packets_received += stats.packets_received;
                total_stats.total_errors_sent += stats.errors_sent;
                total_stats.total_errors_received += stats.errors_received;
                total_stats.total_dropped_packets += stats.dropped_packets;
                total_stats.total_collisions += stats.collisions;
                total_stats.active_interfaces += 1;
            }
        }
        
        total_stats
    }
}

/// Aggregated network statistics
#[derive(Debug, Clone, Default)]
pub struct AggregatedNetworkStats {
    pub total_bytes_sent: u64,
    pub total_bytes_received: u64,
    pub total_packets_sent: u64,
    pub total_packets_received: u64,
    pub total_errors_sent: u64,
    pub total_errors_received: u64,
    pub total_dropped_packets: u64,
    pub total_collisions: u64,
    pub active_interfaces: usize,
}

impl AggregatedNetworkStats {
    /// Calculate total throughput
    pub fn total_throughput(&self) -> u64 {
        self.total_bytes_sent + self.total_bytes_received
    }

    /// Calculate packet loss percentage
    pub fn packet_loss_percentage(&self) -> f64 {
        let total_sent = self.total_packets_sent + self.total_dropped_packets;
        if total_sent > 0 {
            (self.total_dropped_packets as f64 / total_sent as f64) * 100.0
        } else {
            0.0
        }
    }

    /// Calculate error rate
    pub fn error_rate(&self) -> f64 {
        let total_packets = self.total_packets_sent + self.total_packets_received;
        let total_errors = self.total_errors_sent + self.total_errors_received;
        
        if total_packets > 0 {
            (total_errors as f64 / total_packets as f64) * 100.0
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_packet_buffer_creation() {
        let data = vec![0x01, 0x02, 0x03, 0x04];
        let packet = PacketBuffer::new(data.clone());
        
        assert_eq!(packet.length, 4);
        assert_eq!(packet.data, data);
        assert!(packet.source.is_none());
        assert!(packet.dest.is_none());
    }

    #[test]
    fn test_driver_capabilities() {
        let mut caps = DriverCapabilities::default();
        assert!(caps.can_transmit);
        assert_eq!(caps.current_mtu, 1500);
        assert_eq!(caps.hw_addr_len, 6);
        
        caps.current_mtu = 9000;
        assert_eq!(caps.current_mtu, 9000);
    }

    #[test]
    fn test_driver_config() {
        let config = DriverConfig {
            name: "test_driver".to_string(),
            driver_type: DriverType::Ethernet,
            interface_name: "eth0".to_string(),
            initial_mtu: 1500,
            ..Default::default()
        };
        
        assert_eq!(config.name, "test_driver");
        assert_eq!(config.driver_type, DriverType::Ethernet);
        assert_eq!(config.interface_name, "eth0");
    }

    #[test]
    fn test_network_driver_trait() {
        let loopback = LoopbackDriver;
        let info = loopback.get_info();
        
        assert_eq!(info.name, "loopback");
        assert_eq!(info.driver_type, DriverType::Loopback);
        assert!(loopback.get_capabilities().can_transmit);
    }

    #[test]
    fn test_driver_registry() {
        let mut registry = DriverRegistry::new();
        let available_types = registry.get_available_driver_types();
        
        assert!(available_types.contains(&DriverType::Loopback));
        assert!(available_types.contains(&DriverType::Virtual));
        assert!(available_types.contains(&DriverType::Ethernet));
        
        assert!(registry.is_driver_supported(DriverType::Loopback));
        assert!(!registry.is_driver_supported(DriverType::WiFi));
    }

    #[test]
    fn test_aggregated_stats() {
        let mut stats = AggregatedNetworkStats::default();
        
        stats.total_packets_sent = 100;
        stats.total_packets_received = 80;
        stats.total_dropped_packets = 10;
        stats.total_errors_sent = 5;
        stats.total_errors_received = 3;
        
        assert_eq!(stats.total_throughput(), 0);
        assert!((stats.packet_loss_percentage() - 9.09).abs() < 0.01);
        assert!((stats.error_rate() - 4.44).abs() < 0.01);
    }

    #[test]
    fn test_ip_address_range_matching() {
        let range = IpAddressRange::new(
            IpAddress::v4(192, 168, 1, 0),
            IpAddress::v4(192, 168, 1, 255)
        );
        
        assert!(range.contains(IpAddress::v4(192, 168, 1, 1)));
        assert!(range.contains(IpAddress::v4(192, 168, 1, 255)));
        assert!(!range.contains(IpAddress::v4(10, 0, 0, 1)));
    }
}