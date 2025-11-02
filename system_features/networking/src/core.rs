//! Core networking functionality
//!
//! This module provides the foundational components of the network stack including
//! initialization, configuration, statistics, and core data structures.

use crate::{Result, NetworkError};
use parking_lot::RwLock;
use std::sync::Arc;
use std::collections::HashMap;

/// Network configuration structure
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    /// Default MTU (Maximum Transmission Unit)
    pub default_mtu: u16,
    /// Default TTL (Time To Live) for IP packets
    pub default_ttl: u8,
    /// Enable/disable TCP
    pub tcp_enabled: bool,
    /// Enable/disable UDP
    pub udp_enabled: bool,
    /// Enable/disable ICMP
    pub icmp_enabled: bool,
    /// Enable network forwarding
    pub ip_forwarding: bool,
    /// DNS server addresses
    pub dns_servers: Vec<IpAddress>,
    /// Network interface configurations
    pub interfaces: HashMap<String, InterfaceConfig>,
    /// Security settings
    pub security: SecurityConfig,
    /// Logging configuration
    pub logging: LogConfig,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            default_mtu: 1500,
            default_ttl: 64,
            tcp_enabled: true,
            udp_enabled: true,
            icmp_enabled: true,
            ip_forwarding: false,
            dns_servers: vec![
                IpAddress::v4(8, 8, 8, 8),
                IpAddress::v4(8, 8, 4, 4),
            ],
            interfaces: HashMap::new(),
            security: SecurityConfig::default(),
            logging: LogConfig::default(),
        }
    }
}

/// Interface configuration
#[derive(Debug, Clone)]
pub struct InterfaceConfig {
    pub name: String,
    pub ip_address: Option<IpAddress>,
    pub netmask: Option<IpAddress>,
    pub gateway: Option<IpAddress>,
    pub mtu: Option<u16>,
    pub enabled: bool,
    pub driver_type: String,
    pub duplex: DuplexMode,
    pub speed: NetworkSpeed,
}

impl Default for InterfaceConfig {
    fn default() -> Self {
        Self {
            name: String::new(),
            ip_address: None,
            netmask: None,
            gateway: None,
            mtu: Some(1500),
            enabled: true,
            driver_type: "generic".to_string(),
            duplex: DuplexMode::Full,
            speed: NetworkSpeed::FastEthernet,
        }
    }
}

/// Network speed configuration
#[derive(Debug, Clone, PartialEq)]
pub enum NetworkSpeed {
    /// 10 Mbps
    Ethernet10,
    /// 100 Mbps
    FastEthernet,
    /// 1 Gbps
    Gigabit,
    /// 10 Gbps
    TenGigabit,
    /// 40 Gbps
    FortyGigabit,
    /// 100 Gbps
    HundredGigabit,
}

impl NetworkSpeed {
    /// Get the speed in Mbps
    pub fn to_mbps(&self) -> u32 {
        match self {
            NetworkSpeed::Ethernet10 => 10,
            NetworkSpeed::FastEthernet => 100,
            NetworkSpeed::Gigabit => 1000,
            NetworkSpeed::TenGigabit => 10000,
            NetworkSpeed::FortyGigabit => 40000,
            NetworkSpeed::HundredGigabit => 100000,
        }
    }
}

/// Duplex mode configuration
#[derive(Debug, Clone, PartialEq)]
pub enum DuplexMode {
    /// Half duplex
    Half,
    /// Full duplex
    Full,
    /// Auto-negotiation
    Auto,
}

/// Security configuration
#[derive(Debug, Clone)]
pub struct SecurityConfig {
    pub enable_firewall: bool,
    pub default_policy: FirewallPolicy,
    pub port_scanning_protection: bool,
    pub connection_rate_limiting: bool,
    pub max_connections_per_ip: usize,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            enable_firewall: true,
            default_policy: FirewallPolicy::Drop,
            port_scanning_protection: true,
            connection_rate_limiting: true,
            max_connections_per_ip: 100,
        }
    }
}

/// Firewall policy
#[derive(Debug, Clone, PartialEq)]
pub enum FirewallPolicy {
    /// Accept all packets (no filtering)
    Accept,
    /// Drop all packets (maximum security)
    Drop,
    /// Reject packets (respond with error)
    Reject,
}

/// Logging configuration
#[derive(Debug, Clone)]
pub struct LogConfig {
    pub enable_logging: bool,
    pub log_level: LogLevel,
    pub log_connections: bool,
    pub log_errors: bool,
    pub log_performance: bool,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            enable_logging: true,
            log_level: LogLevel::Info,
            log_connections: true,
            log_errors: true,
            log_performance: false,
        }
    }
}

/// Logging levels
#[derive(Debug, Clone, PartialEq)]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
    Critical,
}

/// Network interface structure
#[derive(Debug, Clone)]
pub struct NetworkInterface {
    pub name: String,
    pub index: usize,
    pub ip_address: Option<IpAddress>,
    pub netmask: Option<IpAddress>,
    pub gateway: Option<IpAddress>,
    pub mtu: u16,
    pub speed: NetworkSpeed,
    pub duplex: DuplexMode,
    pub status: InterfaceStatus,
    pub driver: Arc<dyn NetworkDriver>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InterfaceStatus {
    /// Interface is up and running
    Up,
    /// Interface is down
    Down,
    /// Interface is in error state
    Error,
    /// Interface is disabled
    Disabled,
}

/// IPv4 address structure
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IpAddress {
    pub octets: [u8; 4],
}

impl IpAddress {
    /// Create IPv4 address from octets
    pub const fn new(a: u8, b: u8, c: u8, d: u8) -> Self {
        Self { octets: [a, b, c, d] }
    }

    /// Create IPv4 address from bytes
    pub const fn from_bytes(bytes: [u8; 4]) -> Self {
        Self { octets: bytes }
    }

    /// Create IPv4 address from u32
    pub fn from_u32(addr: u32) -> Self {
        Self {
            octets: [
                (addr >> 24) as u8,
                (addr >> 16) as u8,
                (addr >> 8) as u8,
                addr as u8,
            ]
        }
    }

    /// Convert to u32
    pub fn to_u32(&self) -> u32 {
        (self.octets[0] as u32) << 24 |
        (self.octets[1] as u32) << 16 |
        (self.octets[2] as u32) << 8 |
        self.octets[3] as u32
    }

    /// Create IPv4 address from dotted decimal string
    pub fn parse(s: &str) -> Result<Self> {
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() != 4 {
            return Err(NetworkError::InvalidAddress);
        }

        let octets = parts.iter().try_fold([0u8; 4], |mut acc, part| {
            let octet = part.parse::<u8>().map_err(|_| NetworkError::InvalidAddress)?;
            if !acc.is_empty() || !acc.iter().any(|&x| x != 0) {
                acc.rotate_left(1);
            }
            acc[3] = octet;
            Ok(acc)
        })?;

        Ok(Self { octets })
    }

    /// Convert to string representation
    pub fn to_string(&self) -> String {
        format!("{}.{}.{}.{}", 
                self.octets[0], self.octets[1], 
                self.octets[2], self.octets[3])
    }

    /// Convenience method to create common IPv4 addresses
    pub fn v4(a: u8, b: u8, c: u8, d: u8) -> Self {
        Self::new(a, b, c, d)
    }

    /// Create localhost address (127.0.0.1)
    pub fn localhost() -> Self {
        Self::v4(127, 0, 0, 1)
    }

    /// Create broadcast address
    pub fn broadcast() -> Self {
        Self::v4(255, 255, 255, 255)
    }

    /// Create private network addresses
    pub fn private_a() -> Self {
        Self::v4(10, 0, 0, 0)
    }

    pub fn private_b() -> Self {
        Self::v4(172, 16, 0, 0)
    }

    pub fn private_c() -> Self {
        Self::v4(192, 168, 0, 0)
    }

    /// Check if address is in private range
    pub fn is_private(&self) -> bool {
        let addr = self.to_u32();
        
        // 10.0.0.0/8
        if (addr >> 24) == 10 {
            return true;
        }
        
        // 172.16.0.0/12
        if (addr >> 20) == 0xAC1 {
            return true;
        }
        
        // 192.168.0.0/16
        if (addr >> 16) == 0xC0A8 {
            return true;
        }
        
        false
    }

    /// Check if address is loopback
    pub fn is_loopback(&self) -> bool {
        self.octets[0] == 127
    }

    /// Check if address is link-local (169.254.0.0/16)
    pub fn is_link_local(&self) -> bool {
        self.octets[0] == 169 && self.octets[1] == 254
    }

    /// Check if address is multicast
    pub fn is_multicast(&self) -> bool {
        self.octets[0] >= 224 && self.octets[0] <= 239
    }

    /// Check if address is broadcast
    pub fn is_broadcast(&self) -> bool {
        self == &Self::broadcast()
    }
}

impl std::fmt::Display for IpAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}.{}", 
               self.octets[0], self.octets[1], 
               self.octets[2], self.octets[3])
    }
}

impl std::str::FromStr for IpAddress {
    type Err = NetworkError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Self::parse(s)
    }
}

/// Network statistics
#[derive(Debug, Clone)]
pub struct NetworkStats {
    pub packets_sent: u64,
    pub packets_received: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub errors_sent: u64,
    pub errors_received: u64,
    pub active_connections: usize,
    pub routing_table_size: usize,
    pub arp_cache_size: usize,
    pub interface_count: usize,
}

/// Internal network state
struct NetworkState {
    interfaces: Vec<NetworkInterface>,
    config: NetworkConfig,
    stats: NetworkStats,
}

impl Default for NetworkState {
    fn default() -> Self {
        Self {
            interfaces: Vec::new(),
            config: NetworkConfig::default(),
            stats: NetworkStats::default(),
        }
    }
}

impl NetworkStats {
    /// Create empty network statistics
    pub fn new() -> Self {
        Self::default()
    }

    /// Increment packet counter
    pub fn increment_packets_sent(&mut self) {
        self.packets_sent += 1;
    }

    pub fn increment_packets_received(&mut self) {
        self.packets_received += 1;
    }

    /// Increment byte counter
    pub fn increment_bytes_sent(&mut self, bytes: u64) {
        self.bytes_sent += bytes;
    }

    pub fn increment_bytes_received(&mut self, bytes: u64) {
        self.bytes_received += bytes;
    }

    /// Increment error counter
    pub fn increment_errors_sent(&mut self) {
        self.errors_sent += 1;
    }

    pub fn increment_errors_received(&mut self) {
        self.errors_received += 1;
    }

    /// Update connection count
    pub fn set_active_connections(&mut self, count: usize) {
        self.active_connections = count;
    }

    /// Update routing table size
    pub fn set_routing_table_size(&mut self, size: usize) {
        self.routing_table_size = size;
    }

    /// Update ARP cache size
    pub fn set_arp_cache_size(&mut self, size: usize) {
        self.arp_cache_size = size;
    }

    /// Update interface count
    pub fn set_interface_count(&mut self, count: usize) {
        self.interface_count = count;
    }
}

impl Default for NetworkStats {
    fn default() -> Self {
        Self {
            packets_sent: 0,
            packets_received: 0,
            bytes_sent: 0,
            bytes_received: 0,
            errors_sent: 0,
            errors_received: 0,
            active_connections: 0,
            routing_table_size: 0,
            arp_cache_size: 0,
            interface_count: 0,
        }
    }
}

/// Global network state
static NETWORK_STATE: RwLock<NetworkState> = RwLock::new(NetworkState::default());

/// Network driver trait
pub trait NetworkDriver: Send + Sync {
    /// Initialize the network driver
    fn init(&self) -> Result<()>;
    
    /// Send a packet through this interface
    fn send_packet(&self, packet: &[u8], dest: &IpAddress) -> Result<()>;
    
    /// Receive a packet from this interface
    fn receive_packet(&self, buffer: &mut [u8]) -> Result<usize>;
    
    /// Get interface statistics
    fn get_stats(&self) -> Result<InterfaceStats>;
    
    /// Get interface status
    fn get_status(&self) -> InterfaceStatus;
    
    /// Configure interface
    fn configure(&self, config: &InterfaceConfig) -> Result<()>;
    
    /// Get interface name
    fn get_name(&self) -> &str;
    
    /// Get hardware address
    fn get_mac_address(&self) -> Result<[u8; 6]>;
}

/// Interface statistics
#[derive(Debug, Clone)]
pub struct InterfaceStats {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub packets_sent: u64,
    pub packets_received: u64,
    pub errors_sent: u64,
    pub errors_received: u64,
    pub dropped_packets: u64,
    pub collisions: u64,
}

/// Initialize the core network stack
pub fn init_core(interfaces: Vec<String>, config: &NetworkConfig) -> Result<()> {
    let mut state = NETWORK_STATE.write();
    
    // Initialize configuration
    state.config = config.clone();
    
    // Initialize interfaces if provided
    for interface_name in interfaces {
        let interface_config = InterfaceConfig {
            name: interface_name.clone(),
            enabled: true,
            ..Default::default()
        };
        
        // Initialize each interface
        // Note: This would be implemented with actual driver initialization
        log::info!("Initializing interface: {}", interface_name);
    }
    
    // Update interface count in statistics
    state.stats.set_interface_count(state.interfaces.len());
    
    log::info!("Network stack initialized successfully");
    Ok(())
}

/// Shutdown the core network stack
pub fn shutdown_core() -> Result<()> {
    let mut state = NETWORK_STATE.write();
    
    // Close all interfaces
    for interface in &state.interfaces {
        log::info!("Shutting down interface: {}", interface.name);
    }
    
    // Clear state
    state.interfaces.clear();
    
    log::info!("Network stack shutdown completed");
    Ok(())
}

/// Get current network statistics
pub fn get_stats() -> Result<NetworkStats> {
    let state = NETWORK_STATE.read();
    Ok(state.stats.clone())
}

/// Add a network interface
pub fn add_interface(interface: NetworkInterface) -> Result<()> {
    let mut state = NETWORK_STATE.write();
    state.interfaces.push(interface);
    state.stats.set_interface_count(state.interfaces.len());
    Ok(())
}

/// Get all network interfaces
pub fn get_interfaces() -> Result<Vec<NetworkInterface>> {
    let state = NETWORK_STATE.read();
    Ok(state.interfaces.clone())
}

/// Update network configuration
pub fn update_config(config: &NetworkConfig) -> Result<()> {
    let mut state = NETWORK_STATE.write();
    state.config = config.clone();
    Ok(())
}

/// Get current configuration
pub fn get_config() -> Result<NetworkConfig> {
    let state = NETWORK_STATE.read();
    Ok(state.config.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ip_address_creation() {
        let addr = IpAddress::v4(192, 168, 1, 1);
        assert_eq!(addr.octets, [192, 168, 1, 1]);
        assert_eq!(addr.to_string(), "192.168.1.1");
    }

    #[test]
    fn test_ip_address_parsing() {
        let addr = IpAddress::parse("10.0.0.1").unwrap();
        assert_eq!(addr.octets, [10, 0, 0, 1]);
    }

    #[test]
    fn test_ip_address_conversion() {
        let addr = IpAddress::v4(192, 168, 1, 1);
        let u32_val = addr.to_u32();
        assert_eq!(u32_val, 0xC0A80101);
        
        let addr_from_u32 = IpAddress::from_u32(u32_val);
        assert_eq!(addr, addr_from_u32);
    }

    #[test]
    fn test_ip_address_properties() {
        let localhost = IpAddress::localhost();
        assert!(localhost.is_loopback());
        
        let private = IpAddress::private_a();
        assert!(private.is_private());
        
        let link_local = IpAddress::v4(169, 254, 1, 1);
        assert!(link_local.is_link_local());
    }

    #[test]
    fn test_network_config() {
        let config = NetworkConfig::default();
        assert!(config.tcp_enabled);
        assert!(config.udp_enabled);
        assert_eq!(config.default_mtu, 1500);
        assert_eq!(config.default_ttl, 64);
    }

    #[test]
    fn test_network_speed() {
        assert_eq!(NetworkSpeed::FastEthernet.to_mbps(), 100);
        assert_eq!(NetworkSpeed::Gigabit.to_mbps(), 1000);
        assert_eq!(NetworkSpeed::TenGigabit.to_mbps(), 10000);
    }
}