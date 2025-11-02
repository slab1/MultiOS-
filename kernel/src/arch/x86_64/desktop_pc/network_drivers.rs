//! Network Interface Support
//! 
//! Provides support for Ethernet, wireless, and other network interfaces
//! including TCP/IP offload capabilities

use crate::log::{info, warn, error};
const PCI_VENDOR_INTEL: u16 = 0x8086;
use crate::KernelError;

use super::{PciManager, NetworkInterfaceInfo, NetworkInterfaceType, NetworkDuplex};

/// Ethernet frame types
const ETHERTYPE_IPV4: u16 = 0x0800;
const ETHERTYPE_ARP: u16 = 0x0806;
const ETHERTYPE_IPV6: u16 = 0x86DD;

/// TCP/IP header constants
const TCP_HEADER_LENGTH: u16 = 20;
const UDP_HEADER_LENGTH: u16 = 8;
const IP_HEADER_LENGTH: u16 = 20;

/// Common network device vendor IDs
const VENDOR_REALTEK: u16 = 0x10EC;
const VENDOR_INTEL: u16 = 0x8086;
const VENDOR_AMD: u16 = 0x1022;
const VENDOR_BROADCOM: u16 = 0x14E4;
const VENDOR_MARVELL: u16 = 0x11AB;

/// Network buffer sizes
const MAX_ETHERNET_FRAME_SIZE: usize = 1522;
const MIN_ETHERNET_FRAME_SIZE: usize = 64;
const ETHERNET_HEADER_SIZE: usize = 14;
const MAX_TRANSMISSION_UNIT: usize = 1500;

/// Network interface capabilities
const IFF_UP: u32 = 0x1;             // Interface is up
const IFF_BROADCAST: u32 = 0x2;      // Broadcast address valid
const IFF_DEBUG: u32 = 0x4;          // Turn on debugging
const IFF_LOOPBACK: u32 = 0x8;       // Is a loopback net
const IFF_POINTOPOINT: u32 = 0x10;   // Interface is point-to-point link
const IFF_NOTRAILERS: u32 = 0x20;    // Avoid use of trailers
const IFF_RUNNING: u32 = 0x40;       // Resources allocated
const IFF_NOARP: u32 = 0x80;         // No ARP protocol
const IFF_PROMISC: u32 = 0x100;      // Receive all packets
const IFF_ALLMULTI: u32 = 0x200;     // Receive all multicast packets
const IFF_MASTER: u32 = 0x400;       // Master of a load balancer
const IFF_SLAVE: u32 = 0x800;        // Slave of a load balancer
const IFF_PORTSEL: u32 = 0x1000;     // Can select media type
const IFF_AUTOMEDIA: u32 = 0x2000;   // Auto media select active
const IFF_DYNAMIC: u32 = 0x4000;     // Dialup device with changing addresses
const IFF_MULTICAST: u32 = 0x8000;   // Supports multicast

/// Ethernet device capabilities
const ETH_CAP_10HD: u32 = 0x0001;    // 10 Mbps Half Duplex
const ETH_CAP_10FD: u32 = 0x0002;    // 10 Mbps Full Duplex
const ETH_CAP_100HD: u32 = 0x0004;   // 100 Mbps Half Duplex
const ETH_CAP_100FD: u32 = 0x0008;   // 100 Mbps Full Duplex
const ETH_CAP_1000HD: u32 = 0x0010;  // 1000 Mbps Half Duplex
const ETH_CAP_1000FD: u32 = 0x0020;  // 1000 Mbps Full Duplex
const ETH_CAP_2500FD: u32 = 0x0040;  // 2.5 Gbps Full Duplex
const ETH_CAP_5000FD: u32 = 0x0080;  // 5 Gbps Full Duplex
const ETH_CAP_10G: u32 = 0x0100;     // 10 Gbps
const ETH_CAP_25G: u32 = 0x0200;     // 25 Gbps
const ETH_CAP_40G: u32 = 0x0400;     // 40 Gbps
const ETH_CAP_100G: u32 = 0x0800;    // 100 Gbps

/// Ethernet device information
#[derive(Debug, Clone)]
pub struct EthernetDevice {
    pub pci_address: (u8, u8, u8),
    pub device_id: u16,
    pub subsystem_vendor_id: u16,
    pub subsystem_device_id: u16,
    pub mac_address: [u8; 6],
    pub current_speed: u32,
    pub current_duplex: NetworkDuplex,
    pub auto_negotiation_enabled: bool,
    pub link_status: LinkStatus,
    pub capabilities: u32,
    pub flow_control_support: bool,
    pub vlan_support: bool,
    pub wol_support: bool,
    pub energy_efficient_ethernet: bool,
}

/// Link status
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LinkStatus {
    Unknown,
    Down,
    Up,
}

/// Wireless device information
#[derive(Debug, Clone)]
pub struct WirelessDevice {
    pub pci_address: (u8, u8, u8),
    pub device_id: u16,
    pub mac_address: [u8; 6],
    pub supported_standards: Vec<WirelessStandard>,
    pub supported_channels: Vec<u16>,
    pub current_channel: u16,
    pub current_mode: WirelessMode,
    pub encryption_support: Vec<EncryptionType>,
    pub current_encryption: Option<EncryptionType>,
    pub signal_strength: i32, // dBm
    pub noise_level: i32,     // dBm
    pub bitrate: u32,         // Mbps
}

/// Wireless standards
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WirelessStandard {
    B,
    G,
    N,
    Ac,
    Ax,
    Ay,
}

/// Wireless modes
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WirelessMode {
    Infrastructure,
    AdHoc,
    Monitor,
    Master,
}

/// Encryption types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EncryptionType {
    None,
    Wep,
    Wpa,
    Wpa2,
    Wpa3,
}

/// Network statistics
#[derive(Debug, Clone, Default)]
pub struct NetworkStats {
    pub rx_packets: u64,
    pub tx_packets: u64,
    pub rx_bytes: u64,
    pub tx_bytes: u64,
    pub rx_errors: u64,
    pub tx_errors: u64,
    pub rx_dropped: u64,
    pub tx_dropped: u64,
    pub multicast: u64,
    pub collisions: u64,
    pub rx_length_errors: u64,
    pub rx_over_errors: u64,
    pub rx_crc_errors: u64,
    pub rx_frame_errors: u64,
    pub rx_fifo_errors: u64,
    pub rx_missed_errors: u64,
    pub tx_aborted_errors: u64,
    pub tx_carrier_errors: u64,
    pub tx_fifo_errors: u64,
    pub tx_heartbeat_errors: u64,
    pub tx_window_errors: u64,
    pub rx_compressed: u64,
    pub tx_compressed: u64,
}

/// Network buffer
#[derive(Debug, Clone)]
pub struct NetworkBuffer {
    pub data: Vec<u8>,
    pub length: usize,
    pub offset: usize,
    pub flags: u32,
}

/// Network interface flags
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InterfaceFlags {
    Up = IFF_UP as isize,
    Running = IFF_RUNNING as isize,
    Promiscuous = IFF_PROMISC as isize,
    AllMulti = IFF_ALLMULTI as isize,
    Multicast = IFF_MULTICAST as isize,
}

/// IP address structure
#[derive(Debug, Clone, Copy)]
pub struct IpAddress {
    pub octets: [u8; 4],
}

impl IpAddress {
    pub fn new(a: u8, b: u8, c: u8, d: u8) -> Self {
        Self {
            octets: [a, b, c, d],
        }
    }
    
    pub fn from_string(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() != 4 {
            return None;
        }
        
        let octets = parts.iter().filter_map(|&part| part.parse().ok()).collect::<Vec<u8>>();
        if octets.len() != 4 {
            return None;
        }
        
        Some(Self { octets: [octets[0], octets[1], octets[2], octets[3]] })
    }
}

/// MAC address structure
#[derive(Debug, Clone, Copy)]
pub struct MacAddress {
    pub bytes: [u8; 6],
}

impl MacAddress {
    pub fn new(a: u8, b: u8, c: u8, d: u8, e: u8, f: u8) -> Self {
        Self {
            bytes: [a, b, c, d, e, f],
        }
    }
    
    pub fn from_bytes(bytes: &[u8; 6]) -> Self {
        Self { bytes: *bytes }
    }
    
    pub fn to_string(&self) -> String {
        format!("{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
                self.bytes[0], self.bytes[1], self.bytes[2],
                self.bytes[3], self.bytes[4], self.bytes[5])
    }
}

/// Network configuration
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    pub ip_address: Option<IpAddress>,
    pub subnet_mask: Option<IpAddress>,
    pub gateway: Option<IpAddress>,
    pub primary_dns: Option<IpAddress>,
    pub secondary_dns: Option<IpAddress>,
    pub mac_address: MacAddress,
}

/// Network Manager
pub struct NetworkManager {
    pub initialized: bool,
    pub ethernet_devices: Vec<EthernetDevice>,
    pub wireless_devices: Vec<WirelessDevice>,
    pub network_interfaces: Vec<NetworkInterfaceInfo>,
    pub stats: Vec<NetworkStats>,
    pub buffers: Vec<NetworkBuffer>,
    pub packet_queue: Vec<NetworkBuffer>,
    pub configurations: Vec<NetworkConfig>,
    pub tcp_ip_stack_enabled: bool,
    pub is_receiving: bool,
    pub is_transmitting: bool,
}

impl NetworkManager {
    /// Create new network manager
    pub fn new() -> Self {
        Self {
            initialized: false,
            ethernet_devices: Vec::new(),
            wireless_devices: Vec::new(),
            network_interfaces: Vec::new(),
            stats: Vec::new(),
            buffers: Vec::new(),
            packet_queue: Vec::new(),
            configurations: Vec::new(),
            tcp_ip_stack_enabled: false,
            is_receiving: false,
            is_transmitting: false,
        }
    }
    
    /// Initialize network subsystem
    pub fn initialize(&mut self, pci_manager: &PciManager) -> Result<(), KernelError> {
        info!("Initializing network subsystem...");
        
        // Step 1: Detect Ethernet devices
        self.detect_ethernet_devices(pci_manager)?;
        
        // Step 2: Detect Wireless devices
        self.detect_wireless_devices(pci_manager)?;
        
        // Step 3: Initialize network interfaces
        self.init_network_interfaces()?;
        
        // Step 4: Setup packet processing
        self.setup_packet_processing()?;
        
        // Step 5: Enable TCP/IP stack
        self.enable_tcp_ip_stack()?;
        
        self.initialized = true;
        info!("Network subsystem initialized: {} Ethernet, {} Wireless devices",
              self.ethernet_devices.len(), self.wireless_devices.len());
        
        Ok(())
    }
    
    /// Detect Ethernet devices
    fn detect_ethernet_devices(&mut self, pci_manager: &PciManager) -> Result<(), KernelError> {
        info!("Detecting Ethernet devices...");
        
        // Find network controller PCI devices (class 0x02)
        let network_devices = pci_manager.find_devices_by_class(0x02); // Network controller
        
        for device in network_devices {
            if device.subclass == 0x00 { // Ethernet controller
                if self.is_ethernet_device(device.vendor_id, device.device_id) {
                    let ethernet_device = self.init_ethernet_device(
                        device.bus, device.device, device.function,
                        device.vendor_id, device.device_id
                    )?;
                    
                    if ethernet_device.is_some() {
                        self.ethernet_devices.push(ethernet_device.unwrap());
                        info!("Found Ethernet device at {}:{}:{} (Vendor: 0x{:04X}, Device: 0x{:04X})",
                              device.bus, device.device, device.function,
                              device.vendor_id, device.device_id);
                    }
                }
            }
        }
        
        info!("Detected {} Ethernet devices", self.ethernet_devices.len());
        Ok(())
    }
    
    /// Detect Wireless devices
    fn detect_wireless_devices(&mut self, pci_manager: &PciManager) -> Result<(), KernelError> {
        info!("Detecting Wireless devices...");
        
        // Wireless controllers have subclass 0x80 (Wireless controller)
        let wireless_devices = pci_manager.devices.iter()
            .filter(|d| d.class_code == 0x0D) // Wireless controller class
            .collect::<Vec<_>>();
        
        for device in wireless_devices {
            if self.is_wireless_device(device.vendor_id, device.device_id) {
                let wireless_device = self.init_wireless_device(
                    device.bus, device.device, device.function,
                    device.vendor_id, device.device_id
                )?;
                
                if wireless_device.is_some() {
                    self.wireless_devices.push(wireless_device.unwrap());
                    info!("Found Wireless device at {}:{}:{} (Vendor: 0x{:04X}, Device: 0x{:04X})",
                          device.bus, device.device, device.function,
                          device.vendor_id, device.device_id);
                }
            }
        }
        
        info!("Detected {} Wireless devices", self.wireless_devices.len());
        Ok(())
    }
    
    /// Check if device is Ethernet device
    fn is_ethernet_device(&self, vendor_id: u16, device_id: u16) -> bool {
        // Check known Ethernet device combinations
        match vendor_id {
            VENDOR_INTEL => match device_id {
                0x100E | 0x10D3 | 0x15A2 | 0x15F9 | 0x1F40 | 0x1F41 | 0x1F42 | 0x1F43 => true, // Intel
                _ => false,
            },
            VENDOR_REALTEK => match device_id {
                0x8139 | 0x8168 | 0x8111 | 0x8129 | 0x8100 | 0x8101 | 0x8102 => true, // Realtek
                _ => false,
            },
            VENDOR_BROADCOM => match device_id {
                0x1692 | 0x1693 | 0x16A4 | 0x16A6 | 0x16AA | 0x16AE => true, // Broadcom
                _ => false,
            },
            VENDOR_MARVELL => true,
            _ => false,
        }
    }
    
    /// Check if device is Wireless device
    fn is_wireless_device(&self, vendor_id: u16, device_id: u16) -> bool {
        // Check known Wireless device combinations
        match vendor_id {
            VENDOR_INTEL => match device_id {
                0x0090 | 0x0091 | 0x0084 | 0x0085 | 0x24FB | 0x24FD => true, // Intel
                _ => false,
            },
            VENDOR_REALTEK => match device_id {
                0x8179 | 0x8723 | 0x8821 | 0x8852 => true, // Realtek
                _ => false,
            },
            VENDOR_AMD => match device_id {
                0x157E | 0x1590 | 0x15C7 => true, // AMD
                _ => false,
            },
            _ => false,
        }
    }
    
    /// Initialize Ethernet device
    fn init_ethernet_device(&self, bus: u8, device: u8, function: u8, 
                          vendor_id: u16, device_id: u16) -> Result<Option<EthernetDevice>, KernelError> {
        // This would initialize the specific Ethernet controller
        // For now, create a basic device info
        
        let ethernet_device = EthernetDevice {
            pci_address: (bus, device, function),
            device_id,
            subsystem_vendor_id: 0, // Would be read from PCI config
            subsystem_device_id: 0,
            mac_address: [0x02, 0x00, 0x00, 0x00, 0x00, 0x01], // Locally administered MAC
            current_speed: 1000, // Mbps
            current_duplex: NetworkDuplex::Full,
            auto_negotiation_enabled: true,
            link_status: LinkStatus::Down,
            capabilities: ETH_CAP_10FD | ETH_CAP_100FD | ETH_CAP_1000FD,
            flow_control_support: true,
            vlan_support: true,
            wol_support: true,
            energy_efficient_ethernet: true,
        };
        
        Ok(Some(ethernet_device))
    }
    
    /// Initialize Wireless device
    fn init_wireless_device(&self, bus: u8, device: u8, function: u8,
                          vendor_id: u16, device_id: u16) -> Result<Option<WirelessDevice>, KernelError> {
        // This would initialize the specific Wireless controller
        let wireless_device = WirelessDevice {
            pci_address: (bus, device, function),
            device_id,
            mac_address: [0x02, 0x00, 0x00, 0x00, 0x00, 0x02], // Locally administered MAC
            supported_standards: vec![
                WirelessStandard::B,
                WirelessStandard::G,
                WirelessStandard::N,
                WirelessStandard::Ac,
            ],
            supported_channels: (1..=11).collect(), // 2.4 GHz channels
            current_channel: 6,
            current_mode: WirelessMode::Infrastructure,
            encryption_support: vec![
                EncryptionType::Wpa2,
                EncryptionType::Wpa3,
            ],
            current_encryption: None,
            signal_strength: -50, // dBm
            noise_level: -90,     // dBm
            bitrate: 150,         // Mbps
        };
        
        Ok(Some(wireless_device))
    }
    
    /// Initialize network interfaces
    fn init_network_interfaces(&mut self) -> Result<(), KernelError> {
        info!("Initializing network interfaces...");
        
        // Create network interfaces for each device
        for (i, eth_device) in self.ethernet_devices.iter().enumerate() {
            let interface = NetworkInterfaceInfo {
                interface_type: NetworkInterfaceType::Ethernet,
                mac_address: eth_device.mac_address,
                pci_location: Some(eth_device.pci_address),
                speed_mbps: eth_device.current_speed,
                duplex: eth_device.current_duplex,
                driver_attached: false,
            };
            
            self.network_interfaces.push(interface);
            
            // Initialize statistics
            self.stats.push(NetworkStats::default());
        }
        
        for (i, wireless_device) in self.wireless_devices.iter().enumerate() {
            let interface = NetworkInterfaceInfo {
                interface_type: NetworkInterfaceType::Wireless,
                mac_address: wireless_device.mac_address,
                pci_location: Some(wireless_device.pci_address),
                speed_mbps: wireless_device.bitrate,
                duplex: NetworkDuplex::Full, // Wireless is full duplex
                driver_attached: false,
            };
            
            self.network_interfaces.push(interface);
            
            // Initialize statistics
            self.stats.push(NetworkStats::default());
        }
        
        info!("Created {} network interfaces", self.network_interfaces.len());
        Ok(())
    }
    
    /// Setup packet processing
    fn setup_packet_processing(&mut self) -> Result<(), KernelError> {
        info!("Setting up packet processing...");
        
        // Setup interrupt handlers for network devices
        // Configure DMA rings for reception and transmission
        // Setup packet filtering
        
        Ok(())
    }
    
    /// Enable TCP/IP stack
    fn enable_tcp_ip_stack(&mut self) -> Result<(), KernelError> {
        info!("Enabling TCP/IP stack...");
        
        // Initialize IP routing table
        // Setup ARP cache
        // Initialize TCP/UDP ports
        // Enable network protocols
        
        self.tcp_ip_stack_enabled = true;
        
        Ok(())
    }
    
    /// Send packet
    pub fn send_packet(&mut self, interface_id: usize, packet: &[u8]) -> Result<(), KernelError> {
        if !self.initialized {
            return Err(KernelError::NotInitialized);
        }
        
        if interface_id >= self.network_interfaces.len() {
            return Err(KernelError::NotFound);
        }
        
        // Create network buffer
        let mut buffer = NetworkBuffer {
            data: Vec::with_capacity(packet.len() + ETHERNET_HEADER_SIZE),
            length: packet.len(),
            offset: 0,
            flags: 0,
        };
        
        buffer.data.extend_from_slice(packet);
        
        // Add to transmission queue
        self.packet_queue.push(buffer);
        
        // Update statistics
        if interface_id < self.stats.len() {
            self.stats[interface_id].tx_packets += 1;
            self.stats[interface_id].tx_bytes += packet.len() as u64;
        }
        
        // Start transmission if not already running
        if !self.is_transmitting {
            self.is_transmitting = true;
            self.process_transmit_queue()?;
        }
        
        Ok(())
    }
    
    /// Process transmission queue
    fn process_transmit_queue(&mut self) -> Result<(), KernelError> {
        while let Some(buffer) = self.packet_queue.pop() {
            // This would transmit the packet through the appropriate device
            info!("Transmitting packet of size {} bytes", buffer.length);
            
            // Simulate transmission delay
            // In real implementation, this would be asynchronous
        }
        
        self.is_transmitting = false;
        Ok(())
    }
    
    /// Receive packet
    pub fn receive_packet(&mut self, interface_id: usize, packet: &[u8]) -> Result<(), KernelError> {
        if !self.initialized {
            return Err(KernelError::NotInitialized);
        }
        
        if interface_id >= self.network_interfaces.len() {
            return Err(KernelError::NotFound);
        }
        
        // Update statistics
        if interface_id < self.stats.len() {
            self.stats[interface_id].rx_packets += 1;
            self.stats[interface_id].rx_bytes += packet.len() as u64;
        }
        
        // Process packet (parse Ethernet header, determine protocol, etc.)
        self.process_received_packet(interface_id, packet)?;
        
        Ok(())
    }
    
    /// Process received packet
    fn process_received_packet(&self, interface_id: usize, packet: &[u8]) -> Result<(), KernelError> {
        if packet.len() < ETHERNET_HEADER_SIZE {
            return Err(KernelError::InvalidData);
        }
        
        // Parse Ethernet header
        let dest_mac = &packet[0..6];
        let src_mac = &packet[6..12];
        let ethertype = u16::from_be_bytes([packet[12], packet[13]]);
        
        // Check if packet is for this interface
        let interface_mac = self.network_interfaces[interface_id].mac_address;
        
        if dest_mac == interface_mac.bytes || dest_mac == [0xFF; 6] {
            // Process based on ethertype
            match ethertype {
                ETHERTYPE_IPV4 => {
                    info!("Received IPv4 packet from {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
                          src_mac[0], src_mac[1], src_mac[2], src_mac[3], src_mac[4], src_mac[5]);
                },
                ETHERTYPE_ARP => {
                    info!("Received ARP packet from {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
                          src_mac[0], src_mac[1], src_mac[2], src_mac[3], src_mac[4], src_mac[5]);
                },
                ETHERTYPE_IPV6 => {
                    info!("Received IPv6 packet from {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
                          src_mac[0], src_mac[1], src_mac[2], src_mac[3], src_mac[4], src_mac[5]);
                },
                _ => {
                    info!("Received unknown packet type 0x{:04X} from {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
                          ethertype, src_mac[0], src_mac[1], src_mac[2], src_mac[3], src_mac[4], src_mac[5]);
                }
            }
        }
        
        Ok(())
    }
    
    /// Configure network interface
    pub fn configure_interface(&mut self, interface_id: usize, config: &NetworkConfig) -> Result<(), KernelError> {
        if interface_id >= self.network_interfaces.len() {
            return Err(KernelError::NotFound);
        }
        
        // Store configuration
        if interface_id < self.configurations.len() {
            self.configurations[interface_id] = config.clone();
        } else {
            self.configurations.push(config.clone());
        }
        
        info!("Configured interface {} with IP {}", interface_id, 
              if let Some(ip) = config.ip_address {
                  format!("{}.{}.{}.{}", ip.octets[0], ip.octets[1], ip.octets[2], ip.octets[3])
              } else {
                  "DHCP".to_string()
              });
        
        Ok(())
    }
    
    /// Get network statistics
    pub fn get_statistics(&self, interface_id: usize) -> Result<NetworkStats, KernelError> {
        if interface_id >= self.stats.len() {
            return Err(KernelError::NotFound);
        }
        
        Ok(self.stats[interface_id].clone())
    }
    
    /// Get all network interfaces
    pub fn get_network_interfaces(&self) -> &[NetworkInterfaceInfo] {
        &self.network_interfaces
    }
    
    /// Get Ethernet devices
    pub fn get_ethernet_devices(&self) -> &[EthernetDevice] {
        &self.ethernet_devices
    }
    
    /// Get Wireless devices
    pub fn get_wireless_devices(&self) -> &[WirelessDevice] {
        &self.wireless_devices
    }
    
    /// Enable interface
    pub fn enable_interface(&self, interface_id: usize) -> Result<(), KernelError> {
        if interface_id >= self.network_interfaces.len() {
            return Err(KernelError::NotFound);
        }
        
        info!("Enabling network interface {}", interface_id);
        
        // This would enable the device in hardware
        Ok(())
    }
    
    /// Disable interface
    pub fn disable_interface(&self, interface_id: usize) -> Result<(), KernelError> {
        if interface_id >= self.network_interfaces.len() {
            return Err(KernelError::NotFound);
        }
        
        info!("Disabling network interface {}", interface_id);
        
        // This would disable the device in hardware
        Ok(())
    }
    
    /// Get interface MAC address
    pub fn get_mac_address(&self, interface_id: usize) -> Result<MacAddress, KernelError> {
        if interface_id >= self.network_interfaces.len() {
            return Err(KernelError::NotFound);
        }
        
        Ok(MacAddress::from_bytes(&self.network_interfaces[interface_id].mac_address))
    }
    
    /// Get link status
    pub fn get_link_status(&self, interface_id: usize) -> Result<LinkStatus, KernelError> {
        if interface_id < self.ethernet_devices.len() {
            Ok(self.ethernet_devices[interface_id].link_status)
        } else if interface_id < self.network_interfaces.len() {
            Ok(LinkStatus::Up) // Assume wireless is up
        } else {
            Err(KernelError::NotFound)
        }
    }
    
    /// Get device capabilities
    pub fn get_device_capabilities(&self, interface_id: usize) -> Result<u32, KernelError> {
        if interface_id < self.ethernet_devices.len() {
            Ok(self.ethernet_devices[interface_id].capabilities)
        } else {
            Ok(0)
        }
    }
}