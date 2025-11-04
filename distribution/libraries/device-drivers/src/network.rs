//! Network Interface Drivers
//! 
//! Support for Ethernet and WiFi network interfaces with packet handling operations.

use crate::{DeviceType, DriverResult, DriverError, Device, DeviceHandle, DeviceInfo, HardwareAddress, BusHandle, BusType, DeviceCapabilities, DeviceState, DeviceDriver};
use crate::device::DeviceCapability;
use spin::{Mutex, RwLock};
use alloc::{vec::Vec, collections::BTreeMap};
use log::{info, warn, error};

/// Network device types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum NetworkType {
    Unknown = 0,
    Ethernet = 1,
    WiFi = 2,
    WirelessG = 3,
    WirelessN = 4,
    WirelessAC = 5,
    WirelessAX = 6,
}

/// Network interface capabilities
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum NetworkCapability {
    None = 0,
    Promiscuous = 1 << 0,
    Multicast = 1 << 1,
    Broadcast = 1 << 2,
    VLAN = 1 << 3,
    HardwareChecksum = 1 << 4,
    LargeFrame = 1 << 5,
    HardwareEncryption = 1 << 6,
}

bitflags! {
    /// Network capability flags
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct NetworkCapabilities: u32 {
        const NONE = NetworkCapability::None as u32;
        const PROMISCUOUS = NetworkCapability::Promiscuous as u32;
        const MULTICAST = NetworkCapability::Multicast as u32;
        const BROADCAST = NetworkCapability::Broadcast as u32;
        const VLAN = NetworkCapability::Vlan as u32;
        const HARDWARE_CHECKSUM = NetworkCapability::HardwareChecksum as u32;
        const LARGE_FRAME = NetworkCapability::LargeFrame as u32;
        const HARDWARE_ENCRYPTION = NetworkCapability::HardwareEncryption as u32;
    }
}

/// MAC address
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MacAddress {
    pub bytes: [u8; 6],
}

impl MacAddress {
    /// Create new MAC address
    pub fn new(bytes: [u8; 6]) -> Self {
        Self { bytes }
    }
    
    /// Create MAC address from string
    pub fn from_string(s: &str) -> Result<Self, &'static str> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 6 {
            return Err("Invalid MAC address format");
        }
        
        let mut bytes = [0u8; 6];
        for (i, part) in parts.iter().enumerate() {
            bytes[i] = u8::from_str_radix(part, 16)
                .map_err(|_| "Invalid hex digit")?;
        }
        
        Ok(Self { bytes })
    }
    
    /// Convert to string format
    pub fn to_string(&self) -> String {
        format!("{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
                self.bytes[0], self.bytes[1], self.bytes[2],
                self.bytes[3], self.bytes[4], self.bytes[5])
    }
    
    /// Check if MAC address is multicast
    pub fn is_multicast(&self) -> bool {
        self.bytes[0] & 0x01 != 0
    }
    
    /// Check if MAC address is broadcast
    pub fn is_broadcast(&self) -> bool {
        self.bytes.iter().all(|&b| b == 0xFF)
    }
}

/// Network packet buffer
#[derive(Debug, Clone)]
pub struct NetworkPacket {
    pub data: Vec<u8>,
    pub length: usize,
    pub packet_type: NetworkPacketType,
    pub timestamp: u64,
}

/// Network packet types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum NetworkPacketType {
    Unknown = 0,
    Ethernet = 1,
    IPv4 = 2,
    IPv6 = 3,
    Arp = 4,
    Rarp = 5,
    Custom = 6,
}

/// Network interface information
#[derive(Debug, Clone)]
pub struct NetworkInterfaceInfo {
    pub interface_name: &'static str,
    pub mac_address: MacAddress,
    pub network_type: NetworkType,
    pub capabilities: NetworkCapabilities,
    pub max_frame_size: u32,
    pub speed_mbps: u32,
    pub link_up: bool,
    pub duplex_full: bool,
    pub mtu: u32,
}

/// Network operations
pub trait NetworkInterface: Send + Sync {
    /// Send packet over network
    fn send_packet(&self, packet: &NetworkPacket) -> DriverResult<usize>;
    
    /// Receive packet from network
    fn receive_packet(&self) -> DriverResult<Option<NetworkPacket>>;
    
    /// Get interface information
    fn get_interface_info(&self) -> DriverResult<NetworkInterfaceInfo>;
    
    /// Set MAC address
    fn set_mac_address(&self, mac: &MacAddress) -> DriverResult<()>;
    
    /// Enable/disable interface
    fn set_enabled(&self, enabled: bool) -> DriverResult<()>;
    
    /// Get link status
    fn is_link_up(&self) -> bool;
    
    /// Set MTU
    fn set_mtu(&self, mtu: u32) -> DriverResult<()>;
}

/// Ethernet Driver
pub struct EthernetDriver {
    io_base: u64,
    mac_address: MacAddress,
    interface_info: NetworkInterfaceInfo,
    receive_buffer: Vec<u8>,
    transmit_buffer: Vec<u8>,
    interrupt_enabled: bool,
    link_up: bool,
}

impl EthernetDriver {
    /// Create new Ethernet driver
    pub fn new(io_base: u64, mac_address: MacAddress) -> Self {
        Self {
            io_base,
            mac_address,
            interface_info: NetworkInterfaceInfo {
                interface_name: "eth0",
                mac_address,
                network_type: NetworkType::Ethernet,
                capabilities: NetworkCapabilities::PROMISCUOUS | 
                             NetworkCapabilities::MULTICAST | 
                             NetworkCapabilities::BROADCAST |
                             NetworkCapabilities::HARDWARE_CHECKSUM,
                max_frame_size: 1518,
                speed_mbps: 1000,
                link_up: false,
                duplex_full: true,
                mtu: 1500,
            },
            receive_buffer: vec![0u8; 8192],
            transmit_buffer: vec![0u8; 8192],
            interrupt_enabled: false,
            link_up: false,
        }
    }
    
    /// Initialize Ethernet controller
    pub fn init(&mut self) -> DriverResult<()> {
        info!("Initializing Ethernet interface");
        
        // Reset controller
        self.reset_controller()?;
        
        // Configure MAC address
        self.configure_mac()?;
        
        // Set up receive buffers
        self.setup_receive_buffers()?;
        
        // Enable interrupts
        self.enable_interrupts()?;
        
        // Start transmission and reception
        self.start()?;
        
        info!("Ethernet interface initialized: {} ({} Mbps)", 
              self.mac_address.to_string(), self.interface_info.speed_mbps);
        
        Ok(())
    }
    
    /// Reset Ethernet controller
    fn reset_controller(&self) -> DriverResult<()> {
        info!("Resetting Ethernet controller");
        
        // Send reset command to controller
        // Clear all registers
        // Wait for reset completion
        
        Ok(())
    }
    
    /// Configure MAC address
    fn configure_mac(&self) -> DriverResult<()> {
        info!("Configuring MAC address: {}", self.mac_address.to_string());
        
        // Write MAC address to controller registers
        // Enable MAC address filtering
        
        Ok(())
    }
    
    /// Set up receive buffers
    fn setup_receive_buffers(&self) -> DriverResult<()> {
        info!("Setting up Ethernet receive buffers");
        
        // Initialize DMA descriptors for receive
        // Set up receive buffer ring
        // Configure buffer sizes
        
        Ok(())
    }
    
    /// Enable interrupts
    fn enable_interrupts(&mut self) -> DriverResult<()> {
        self.interrupt_enabled = true;
        info!("Ethernet interrupts enabled");
        Ok(())
    }
    
    /// Start reception and transmission
    fn start(&mut self) -> DriverResult<()> {
        self.link_up = true;
        self.interface_info.link_up = true;
        info!("Ethernet interface started");
        Ok(())
    }
    
    /// Transmit Ethernet frame
    fn transmit_frame(&self, data: &[u8]) -> DriverResult<usize> {
        if !self.link_up {
            return Err(DriverError::DeviceNotFound);
        }
        
        if data.len() > self.interface_info.max_frame_size as usize {
            return Err(DriverError::HardwareError);
        }
        
        info!("Transmitting Ethernet frame: {} bytes", data.len());
        
        // Copy data to transmit buffer
        let frame_size = data.len().min(self.transmit_buffer.len());
        self.transmit_buffer[..frame_size].copy_from_slice(&data[..frame_size]);
        
        // Set up DMA descriptor
        // Start transmission
        
        Ok(frame_size)
    }
    
    /// Receive Ethernet frame
    fn receive_frame(&self) -> DriverResult<Option<Vec<u8>>> {
        if !self.link_up {
            return Ok(None);
        }
        
        // Check if receive buffer has data
        // Read frame from controller
        // Strip off Ethernet header
        // Return payload data
        
        // For simulation, return empty frame occasionally
        if self.receive_buffer[0] % 4 == 0 {
            Ok(None)
        } else {
            let frame_size = ((self.receive_buffer[1] as usize) << 8) | (self.receive_buffer[0] as usize);
            if frame_size > 0 && frame_size <= self.receive_buffer.len() {
                Ok(Some(self.receive_buffer[..frame_size].to_vec()))
            } else {
                Ok(None)
            }
        }
    }
    
    /// Check link status
    fn check_link_status(&self) -> DriverResult<bool> {
        // Read PHY status register
        // Return link status
        
        info!("Checking Ethernet link status");
        Ok(self.link_up)
    }
}

impl DeviceDriver for EthernetDriver {
    fn name(&self) -> &'static str {
        "Ethernet Driver"
    }
    
    fn supported_devices(&self) -> &[DeviceType] {
        &[DeviceType::Network]
    }
    
    fn init(&self, _device: &Device) -> DriverResult<()> {
        info!("Initializing Ethernet driver");
        Ok(())
    }
    
    fn remove(&self, _device: &Device) -> DriverResult<()> {
        info!("Removing Ethernet driver");
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

impl NetworkInterface for EthernetDriver {
    fn send_packet(&self, packet: &NetworkPacket) -> DriverResult<usize> {
        match packet.packet_type {
            NetworkPacketType::Ethernet | NetworkPacketType::IPv4 | NetworkPacketType::IPv6 | 
            NetworkPacketType::Arp | NetworkPacketType::Rarp => {
                self.transmit_frame(&packet.data)
            }
            _ => Err(DriverError::DeviceNotFound),
        }
    }
    
    fn receive_packet(&self) -> DriverResult<Option<NetworkPacket>> {
        if let Ok(Some(frame_data)) = self.receive_frame() {
            let packet_type = if frame_data.len() >= 14 {
                // Parse Ethernet type field
                let ethertype = ((frame_data[12] as u16) << 8) | (frame_data[13] as u16);
                match ethertype {
                    0x0800 => NetworkPacketType::IPv4,
                    0x86DD => NetworkPacketType::IPv6,
                    0x0806 => NetworkPacketType::Arp,
                    0x8035 => NetworkPacketType::Rarp,
                    _ => NetworkPacketType::Ethernet,
                }
            } else {
                NetworkPacketType::Ethernet
            };
            
            Ok(Some(NetworkPacket {
                data: frame_data,
                length: frame_data.len(),
                packet_type,
                timestamp: 0, // Would be set to actual timestamp
            }))
        } else {
            Ok(None)
        }
    }
    
    fn get_interface_info(&self) -> DriverResult<NetworkInterfaceInfo> {
        Ok(self.interface_info.clone())
    }
    
    fn set_mac_address(&self, mac: &MacAddress) -> DriverResult<()> {
        info!("Setting MAC address to: {}", mac.to_string());
        self.mac_address = *mac;
        self.interface_info.mac_address = *mac;
        Ok(())
    }
    
    fn set_enabled(&self, enabled: bool) -> DriverResult<()> {
        if enabled {
            self.start()?;
        } else {
            self.link_up = false;
            self.interface_info.link_up = false;
            info!("Ethernet interface disabled");
        }
        Ok(())
    }
    
    fn is_link_up(&self) -> bool {
        self.link_up
    }
    
    fn set_mtu(&self, mtu: u32) -> DriverResult<()> {
        if mtu > self.interface_info.max_frame_size - 18 { // Account for Ethernet header/trailer
            return Err(DriverError::HardwareError);
        }
        
        self.interface_info.mtu = mtu;
        info!("Ethernet MTU set to: {}", mtu);
        Ok(())
    }
}

/// WiFi Driver
pub struct WifiDriver {
    radio_type: WifiRadioType,
    ssid: String,
    mac_address: MacAddress,
    interface_info: NetworkInterfaceInfo,
    connected: bool,
    channel: u8,
    encryption: WifiEncryption,
    signal_strength: i8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum WifiRadioType {
    Unknown = 0,
    A = 1,
    B = 2,
    G = 3,
    N = 4,
    AC = 5,
    AX = 6,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum WifiEncryption {
    None = 0,
    Wep = 1,
    Wpa = 2,
    Wpa2 = 3,
    Wpa3 = 4,
}

impl WifiDriver {
    /// Create new WiFi driver
    pub fn new(mac_address: MacAddress, radio_type: WifiRadioType) -> Self {
        Self {
            radio_type,
            ssid: String::new(),
            mac_address,
            interface_info: NetworkInterfaceInfo {
                interface_name: "wlan0",
                mac_address,
                network_type: NetworkType::WiFi,
                capabilities: NetworkCapabilities::PROMISCUOUS | 
                             NetworkCapabilities::MULTICAST | 
                             NetworkCapabilities::HARDWARE_ENCRYPTION,
                max_frame_size: 2304,
                speed_mbps: match radio_type {
                    WifiRadioType::N => 300,
                    WifiRadioType::AC => 1300,
                    WifiRadioType::AX => 6000,
                    _ => 54,
                },
                link_up: false,
                duplex_full: true, // WiFi is always full duplex
                mtu: 1500,
            },
            connected: false,
            channel: 6,
            encryption: WifiEncryption::Wpa2,
            signal_strength: -50, // Good signal strength
        }
    }
    
    /// Initialize WiFi radio
    pub fn init(&mut self) -> DriverResult<()> {
        info!("Initializing WiFi interface");
        
        // Reset WiFi chip
        self.reset_radio()?;
        
        // Configure MAC address
        self.configure_mac()?;
        
        // Set initial channel
        self.set_channel(self.channel)?;
        
        info!("WiFi interface initialized: {} ({} Mbps)", 
              self.mac_address.to_string(), self.interface_info.speed_mbps);
        
        Ok(())
    }
    
    /// Reset WiFi radio
    fn reset_radio(&self) -> DriverResult<()> {
        info!("Resetting WiFi radio");
        
        // Send reset command to WiFi chip
        // Clear all registers
        // Reinitialize RF settings
        
        Ok(())
    }
    
    /// Configure MAC address
    fn configure_mac(&self) -> DriverResult<()> {
        info!("Configuring WiFi MAC address: {}", self.mac_address.to_string());
        
        // Write MAC address to WiFi chip
        // Configure MAC filtering
        
        Ok(())
    }
    
    /// Scan for access points
    pub fn scan(&self) -> DriverResult<Vec<AccessPoint>> {
        info!("Scanning for WiFi access points");
        
        // Send scan command
        // Collect scan results
        // Return list of access points
        
        let mut access_points = Vec::new();
        
        // Add some example access points
        access_points.push(AccessPoint {
            ssid: "HomeNetwork".to_string(),
            mac_address: MacAddress::new([0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]),
            signal_strength: -40,
            channel: 6,
            encryption: WifiEncryption::Wpa2,
            radio_type: self.radio_type,
        });
        
        Ok(access_points)
    }
    
    /// Connect to access point
    pub fn connect(&mut self, ssid: &str, password: Option<&str>) -> DriverResult<()> {
        info!("Connecting to WiFi network: {}", ssid);
        
        // Find access point
        let access_points = self.scan()?;
        if let Some(ap) = access_points.iter().find(|ap| ap.ssid == ssid) {
            // Set authentication
            if ap.encryption != WifiEncryption::None {
                if let Some(pass) = password {
                    info!("Using password for {}", ap.ssid);
                } else {
                    return Err(DriverError::PermissionDenied);
                }
            }
            
            // Set channel
            self.set_channel(ap.channel)?;
            
            // Send association request
            self.send_association_request(ap)?;
            
            // Start data connection
            self.connected = true;
            self.ssid = ssid.to_string();
            self.channel = ap.channel;
            self.interface_info.link_up = true;
            
            info!("Successfully connected to WiFi: {}", ssid);
            Ok(())
        } else {
            Err(DriverError::DeviceNotFound)
        }
    }
    
    /// Disconnect from access point
    pub fn disconnect(&mut self) -> DriverResult<()> {
        info!("Disconnecting from WiFi network");
        
        self.connected = false;
        self.ssid.clear();
        self.interface_info.link_up = false;
        
        Ok(())
    }
    
    /// Set channel
    fn set_channel(&self, channel: u8) -> DriverResult<()> {
        if channel < 1 || channel > 14 {
            return Err(DriverError::HardwareError);
        }
        
        info!("Setting WiFi channel to: {}", channel);
        
        // Tune RF to channel frequency
        Ok(())
    }
    
    /// Send association request
    fn send_association_request(&self, ap: &AccessPoint) -> DriverResult<()> {
        info!("Sending association request to {}", ap.ssid);
        
        // Build association request frame
        // Send to access point
        // Wait for association response
        
        Ok(())
    }
    
    /// Transmit WiFi frame
    fn transmit_frame(&self, data: &[u8]) -> DriverResult<usize> {
        if !self.connected {
            return Err(DriverError::DeviceNotFound);
        }
        
        info!("Transmitting WiFi frame: {} bytes", data.len());
        
        // Build 802.11 frame
        // Add encryption if enabled
        // Transmit via RF
        
        Ok(data.len())
    }
    
    /// Receive WiFi frame
    fn receive_frame(&self) -> DriverResult<Option<Vec<u8>>> {
        if !self.connected {
            return Ok(None);
        }
        
        // Check if receive buffer has data
        // Read 802.11 frame from WiFi chip
        // Decrypt if encrypted
        // Return payload data
        
        Ok(None)
    }
}

/// Access point information
#[derive(Debug, Clone)]
pub struct AccessPoint {
    pub ssid: String,
    pub mac_address: MacAddress,
    pub signal_strength: i8,
    pub channel: u8,
    pub encryption: WifiEncryption,
    pub radio_type: WifiRadioType,
}

impl DeviceDriver for WifiDriver {
    fn name(&self) -> &'static str {
        "WiFi Driver"
    }
    
    fn supported_devices(&self) -> &[DeviceType] {
        &[DeviceType::Network]
    }
    
    fn init(&self, _device: &Device) -> DriverResult<()> {
        info!("Initializing WiFi driver");
        Ok(())
    }
    
    fn remove(&self, _device: &Device) -> DriverResult<()> {
        info!("Removing WiFi driver");
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

impl NetworkInterface for WifiDriver {
    fn send_packet(&self, packet: &NetworkPacket) -> DriverResult<usize> {
        match packet.packet_type {
            NetworkPacketType::Ethernet | NetworkPacketType::IPv4 | NetworkPacketType::IPv6 => {
                self.transmit_frame(&packet.data)
            }
            _ => Err(DriverError::DeviceNotFound),
        }
    }
    
    fn receive_packet(&self) -> DriverResult<Option<NetworkPacket>> {
        if let Ok(Some(frame_data)) = self.receive_frame() {
            // Strip 802.11 header and return payload
            let packet_type = if frame_data.len() >= 14 {
                let ethertype = ((frame_data[12] as u16) << 8) | (frame_data[13] as u16);
                match ethertype {
                    0x0800 => NetworkPacketType::IPv4,
                    0x86DD => NetworkPacketType::IPv6,
                    _ => NetworkPacketType::Ethernet,
                }
            } else {
                NetworkPacketType::Ethernet
            };
            
            Ok(Some(NetworkPacket {
                data: frame_data,
                length: frame_data.len(),
                packet_type,
                timestamp: 0,
            }))
        } else {
            Ok(None)
        }
    }
    
    fn get_interface_info(&self) -> DriverResult<NetworkInterfaceInfo> {
        Ok(self.interface_info.clone())
    }
    
    fn set_mac_address(&self, mac: &MacAddress) -> DriverResult<()> {
        info!("Setting WiFi MAC address to: {}", mac.to_string());
        self.mac_address = *mac;
        self.interface_info.mac_address = *mac;
        Ok(())
    }
    
    fn set_enabled(&self, enabled: bool) -> DriverResult<()> {
        if enabled {
            self.reset_radio()?;
        } else {
            self.connected = false;
            self.interface_info.link_up = false;
        }
        info!("WiFi interface {}", if enabled { "enabled" } else { "disabled" });
        Ok(())
    }
    
    fn is_link_up(&self) -> bool {
        self.connected && self.interface_info.link_up
    }
    
    fn set_mtu(&self, mtu: u32) -> DriverResult<()> {
        self.interface_info.mtu = mtu;
        info!("WiFi MTU set to: {}", mtu);
        Ok(())
    }
}

/// Network driver manager
pub struct NetworkDriverManager {
    ethernet_interfaces: Vec<EthernetDriver>,
    wifi_interfaces: Vec<WifiDriver>,
    primary_interface: Option<&'static dyn NetworkInterface>,
    packet_queue: Vec<NetworkPacket>,
}

impl NetworkDriverManager {
    /// Create new network driver manager
    pub fn new() -> Self {
        Self {
            ethernet_interfaces: Vec::new(),
            wifi_interfaces: Vec::new(),
            primary_interface: None,
            packet_queue: Vec::new(),
        }
    }
    
    /// Register Ethernet interface
    pub fn register_ethernet(&mut self, io_base: u64, mac_address: MacAddress) -> DriverResult<()> {
        let mut driver = EthernetDriver::new(io_base, mac_address);
        driver.init()?;
        
        self.ethernet_interfaces.push(driver);
        
        // Set as primary if no interface exists
        if self.primary_interface.is_none() {
            self.primary_interface = Some(self.ethernet_interfaces.last().unwrap());
        }
        
        info!("Ethernet interface registered");
        Ok(())
    }
    
    /// Register WiFi interface
    pub fn register_wifi(&mut self, mac_address: MacAddress, radio_type: WifiRadioType) -> DriverResult<()> {
        let mut driver = WifiDriver::new(mac_address, radio_type);
        driver.init()?;
        
        self.wifi_interfaces.push(driver);
        
        info!("WiFi interface registered");
        Ok(())
    }
    
    /// Get primary network interface
    pub fn get_primary_interface(&self) -> Option<&dyn NetworkInterface> {
        self.primary_interface.map(|interface| *interface)
    }
    
    /// Send packet on primary interface
    pub fn send_packet(&self, packet: &NetworkPacket) -> DriverResult<usize> {
        if let Some(interface) = self.get_primary_interface() {
            interface.send_packet(packet)
        } else {
            Err(DriverError::DeviceNotFound)
        }
    }
    
    /// Receive packet from any interface
    pub fn receive_packet(&self) -> DriverResult<Option<NetworkPacket>> {
        // Check all interfaces for packets
        let mut packets = Vec::new();
        
        // Check Ethernet interfaces
        for eth in &self.ethernet_interfaces {
            if let Ok(Some(packet)) = eth.receive_packet() {
                packets.push(packet);
            }
        }
        
        // Check WiFi interfaces
        for wifi in &self.wifi_interfaces {
            if let Ok(Some(packet)) = wifi.receive_packet() {
                packets.push(packet);
            }
        }
        
        // Return first available packet
        if !packets.is_empty() {
            Ok(Some(packets[0].clone()))
        } else {
            Ok(None)
        }
    }
    
    /// Get all network interfaces
    pub fn list_interfaces(&self) -> Vec<NetworkInterfaceInfo> {
        let mut interfaces = Vec::new();
        
        for eth in &self.ethernet_interfaces {
            if let Ok(info) = eth.get_interface_info() {
                interfaces.push(info);
            }
        }
        
        for wifi in &self.wifi_interfaces {
            if let Ok(info) = wifi.get_interface_info() {
                interfaces.push(info);
            }
        }
        
        interfaces
    }
    
    /// Check if any interface is connected
    pub fn is_connected(&self) -> bool {
        self.ethernet_interfaces.iter().any(|eth| eth.is_link_up()) ||
        self.wifi_interfaces.iter().any(|wifi| wifi.is_link_up())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mac_address_creation() {
        let mac = MacAddress::new([0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC]);
        assert_eq!(mac.to_string(), "12:34:56:78:9A:BC");
    }

    #[test]
    fn test_mac_address_from_string() {
        let mac = MacAddress::from_string("AA:BB:CC:DD:EE:FF").unwrap();
        assert_eq!(mac.bytes, [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]);
    }

    #[test]
    fn test_mac_address_multicast() {
        let multicast_mac = MacAddress::new([0x01, 0x00, 0x00, 0x00, 0x00, 0x00]);
        assert!(multicast_mac.is_multicast());
        
        let unicast_mac = MacAddress::new([0x02, 0x00, 0x00, 0x00, 0x00, 0x00]);
        assert!(!unicast_mac.is_multicast());
    }

    #[test]
    fn test_mac_address_broadcast() {
        let broadcast_mac = MacAddress::new([0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]);
        assert!(broadcast_mac.is_broadcast());
    }

    #[test]
    fn test_network_packet() {
        let packet = NetworkPacket {
            data: vec![0x12, 0x34, 0x56, 0x78],
            length: 4,
            packet_type: NetworkPacketType::IPv4,
            timestamp: 1000,
        };
        
        assert_eq!(packet.length, 4);
        assert_eq!(packet.packet_type, NetworkPacketType::IPv4);
    }

    #[test]
    fn test_network_interface_info() {
        let mac = MacAddress::new([0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC]);
        let info = NetworkInterfaceInfo {
            interface_name: "eth0",
            mac_address: mac,
            network_type: NetworkType::Ethernet,
            capabilities: NetworkCapabilities::PROMISCUOUS | NetworkCapabilities::MULTICAST,
            max_frame_size: 1518,
            speed_mbps: 1000,
            link_up: true,
            duplex_full: true,
            mtu: 1500,
        };
        
        assert_eq!(info.speed_mbps, 1000);
        assert!(info.link_up);
    }

    #[test]
    fn test_network_driver_manager() {
        let mut manager = NetworkDriverManager::new();
        
        // Register Ethernet interface
        let mac = MacAddress::new([0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC]);
        assert!(manager.register_ethernet(0x1C00, mac).is_ok());
        assert!(manager.is_connected());
        
        // Get interface info
        let interfaces = manager.list_interfaces();
        assert!(!interfaces.is_empty());
    }
}
