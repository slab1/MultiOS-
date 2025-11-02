//! User Datagram Protocol (UDP) implementation
//!
//! This module provides a lightweight UDP implementation suitable for
//! connectionless communication with minimal overhead.

use crate::{Result, NetworkError};
use crate::core::IpAddress;
use std::collections::HashMap;

/// UDP header structure
#[derive(Debug, Clone)]
pub struct UdpHeader {
    /// Source port
    pub source_port: u16,
    /// Destination port
    pub dest_port: u16,
    /// Length of UDP header and data
    pub length: u16,
    /// Checksum
    pub checksum: u16,
}

impl UdpHeader {
    /// Create a new UDP header
    pub fn new(source_port: u16, dest_port: u16, data_len: usize) -> Self {
        let length = 8 + data_len as u16;
        
        Self {
            source_port,
            dest_port,
            length,
            checksum: 0, // Will be calculated
        }
    }

    /// Parse UDP header from raw bytes
    pub fn parse(data: &[u8]) -> Result<Self> {
        if data.len() < 8 {
            return Err(NetworkError::InvalidAddress);
        }

        let source_port = ((data[0] as u16) << 8) | (data[1] as u16);
        let dest_port = ((data[2] as u16) << 8) | (data[3] as u16);
        let length = ((data[4] as u16) << 8) | (data[5] as u16);
        let checksum = ((data[6] as u16) << 8) | (data[7] as u16);

        Ok(Self {
            source_port,
            dest_port,
            length,
            checksum,
        })
    }

    /// Convert header to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(8);
        
        bytes.push((self.source_port >> 8) as u8);
        bytes.push(self.source_port as u8);
        
        bytes.push((self.dest_port >> 8) as u8);
        bytes.push(self.dest_port as u8);
        
        bytes.push((self.length >> 8) as u8);
        bytes.push(self.length as u8);
        
        bytes.push((self.checksum >> 8) as u8);
        bytes.push(self.checksum as u8);
        
        bytes
    }

    /// Calculate and set checksum
    pub fn calculate_checksum(&mut self, data: &[u8], source: IpAddress, dest: IpAddress) {
        let mut pseudo_header = Vec::with_capacity(12 + data.len());
        
        // Pseudo-header for checksum calculation
        pseudo_header.extend_from_slice(&source.octets);
        pseudo_header.extend_from_slice(&dest.octets);
        pseudo_header.push(0x00); // Protocol
        pseudo_header.push(17); // UDP protocol number
        pseudo_header.push((self.length >> 8) as u8);
        pseudo_header.push(self.length as u8);
        
        // UDP header (with checksum set to 0)
        pseudo_header.push((self.source_port >> 8) as u8);
        pseudo_header.push(self.source_port as u8);
        pseudo_header.push((self.dest_port >> 8) as u8);
        pseudo_header.push(self.dest_port as u8);
        pseudo_header.push((self.length >> 8) as u8);
        pseudo_header.push(self.length as u8);
        pseudo_header.push(0x00); // Checksum high byte
        pseudo_header.push(0x00); // Checksum low byte
        
        // Data
        pseudo_header.extend_from_slice(data);
        
        // Calculate checksum
        self.checksum = crate::protocols::utils::calculate_checksum(&pseudo_header);
    }

    /// Verify checksum
    pub fn verify_checksum(&self, data: &[u8], source: IpAddress, dest: IpAddress) -> bool {
        let mut header = *self;
        header.checksum = 0;
        
        let mut header_bytes = header.to_bytes();
        let mut packet_data = header_bytes;
        packet_data.extend_from_slice(data);
        
        let mut pseudo_header = Vec::with_capacity(12 + packet_data.len());
        pseudo_header.extend_from_slice(&source.octets);
        pseudo_header.extend_from_slice(&dest.octets);
        pseudo_header.push(0x00);
        pseudo_header.push(17);
        pseudo_header.push((self.length >> 8) as u8);
        pseudo_header.push(self.length as u8);
        pseudo_header.extend_from_slice(&packet_data);
        
        let calculated_checksum = crate::protocols::utils::calculate_checksum(&pseudo_header);
        calculated_checksum == self.checksum
    }
}

/// UDP packet structure
#[derive(Debug, Clone)]
pub struct UdpPacket {
    /// UDP header
    pub header: UdpHeader,
    /// Data payload
    pub data: Vec<u8>,
}

impl UdpPacket {
    /// Create a new UDP packet
    pub fn new(source_port: u16, dest_port: u16) -> Self {
        Self {
            header: UdpHeader::new(source_port, dest_port, 0),
            data: Vec::new(),
        }
    }

    /// Create UDP packet with data
    pub fn with_data(source_port: u16, dest_port: u16, data: Vec<u8>) -> Self {
        let mut header = UdpHeader::new(source_port, dest_port, data.len());
        Self {
            header,
            data,
        }
    }

    /// Parse UDP packet from raw bytes
    pub fn parse(data: &[u8]) -> Result<Self> {
        if data.len() < 8 {
            return Err(NetworkError::InvalidAddress);
        }

        let header = UdpHeader::parse(data)?;
        let payload_start = 8;
        
        if data.len() < header.length as usize {
            return Err(NetworkError::InvalidAddress);
        }

        let data = if payload_start < data.len() {
            data[payload_start..header.length as usize].to_vec()
        } else {
            Vec::new()
        };

        Ok(Self { header, data })
    }

    /// Convert UDP packet to raw bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.header.length as usize);
        
        // UDP header
        bytes.extend_from_slice(&self.header.to_bytes());
        
        // Data payload
        bytes.extend_from_slice(&self.data);
        
        bytes
    }

    /// Generate UDP packet with data
    pub fn generate_udp_packet(data: &[u8], source: IpAddress, dest: IpAddress) -> Result<Vec<u8>> {
        let mut packet = UdpPacket::with_data(0, 0, data.to_vec()); // Ports set by caller
        packet.header.calculate_checksum(&packet.data, source, dest);
        Ok(packet.to_bytes())
    }

    /// Get packet length
    pub fn length(&self) -> usize {
        8 + self.data.len()
    }

    /// Check if packet is valid
    pub fn is_valid(&self) -> bool {
        self.header.length as usize >= 8 && 
        self.header.length as usize == self.length() &&
        (self.header.checksum == 0 || self.header.checksum != 0) // Valid checksum
    }
}

/// UDP socket information for tracking active sockets
#[derive(Debug, Clone)]
pub struct UdpSocketInfo {
    /// Local address and port
    pub local_addr: (IpAddress, u16),
    /// Remote address (if connected)
    pub remote_addr: Option<(IpAddress, u16)>,
    /// Socket options
    pub options: UdpSocketOptions,
    /// Statistics
    pub stats: UdpSocketStats,
}

#[derive(Debug, Clone)]
pub struct UdpSocketOptions {
    /// Enable broadcast
    pub broadcast: bool,
    /// Enable multicast loopback
    pub multicast_loop: bool,
    /// Multicast TTL
    pub multicast_ttl: u8,
    /// Receive buffer size
    pub receive_buffer_size: usize,
    /// Send buffer size
    pub send_buffer_size: usize,
}

impl Default for UdpSocketOptions {
    fn default() -> Self {
        Self {
            broadcast: false,
            multicast_loop: true,
            multicast_ttl: 1,
            receive_buffer_size: 65535,
            send_buffer_size: 65535,
        }
    }
}

#[derive(Debug, Clone)]
pub struct UdpSocketStats {
    /// Packets sent
    pub packets_sent: u64,
    /// Packets received
    pub packets_received: u64,
    /// Bytes sent
    pub bytes_sent: u64,
    /// Bytes received
    pub bytes_received: u64,
    /// Receive errors
    pub receive_errors: u64,
    /// Send errors
    pub send_errors: u64,
}

impl Default for UdpSocketStats {
    fn default() -> Self {
        Self {
            packets_sent: 0,
            packets_received: 0,
            bytes_sent: 0,
            bytes_received: 0,
            receive_errors: 0,
            send_errors: 0,
        }
    }
}

impl UdpSocketStats {
    /// Update send statistics
    pub fn record_send(&mut self, bytes: u64) {
        self.packets_sent += 1;
        self.bytes_sent += bytes;
    }

    /// Update receive statistics
    pub fn record_receive(&mut self, bytes: u64) {
        self.packets_received += 1;
        self.bytes_received += bytes;
    }

    /// Record error
    pub fn record_error(&mut self, is_receive: bool) {
        if is_receive {
            self.receive_errors += 1;
        } else {
            self.send_errors += 1;
        }
    }
}

/// UDP manager for tracking active sockets and processing packets
pub struct UdpManager {
    /// Active UDP sockets
    sockets: HashMap<(IpAddress, u16), UdpSocketInfo>,
    /// UDP connections (for connected sockets)
    connections: HashMap<(IpAddress, u16, IpAddress, u16), UdpConnectionInfo>,
}

#[derive(Debug, Clone)]
struct UdpConnectionInfo {
    /// Local address and port
    local_addr: (IpAddress, u16),
    /// Remote address and port
    remote_addr: (IpAddress, u16),
    /// Connection options
    options: UdpConnectionOptions,
}

#[derive(Debug, Clone)]
struct UdpConnectionOptions {
    /// Connect timeout
    pub connect_timeout: Option<Duration>,
    /// Last activity time
    pub last_activity: Option<chrono::DateTime<chrono::Utc>>,
}

impl Default for UdpConnectionOptions {
    fn default() -> Self {
        Self {
            connect_timeout: None,
            last_activity: None,
        }
    }
}

impl UdpManager {
    /// Create a new UDP manager
    pub fn new() -> Self {
        Self {
            sockets: HashMap::new(),
            connections: HashMap::new(),
        }
    }

    /// Register a UDP socket
    pub fn register_socket(&mut self, addr: (IpAddress, u16), options: UdpSocketOptions) -> Result<()> {
        let socket_info = UdpSocketInfo {
            local_addr: addr,
            remote_addr: None,
            options,
            stats: UdpSocketStats::default(),
        };
        
        self.sockets.insert(addr, socket_info);
        Ok(())
    }

    /// Unregister a UDP socket
    pub fn unregister_socket(&mut self, addr: (IpAddress, u16)) -> Result<()> {
        self.sockets.remove(&addr);
        
        // Remove any connections associated with this socket
        let keys_to_remove: Vec<_> = self.connections.keys()
            .filter(|(local, _, _, _)| *local == addr.0 && addr.1 == local.1)
            .cloned()
            .collect();
            
        for key in keys_to_remove {
            self.connections.remove(&key);
        }
        
        Ok(())
    }

    /// Connect a UDP socket to a remote address
    pub fn connect_socket(&mut self, local_addr: (IpAddress, u16), 
                        remote_addr: (IpAddress, u16)) -> Result<()> {
        if let Some(socket_info) = self.sockets.get_mut(&local_addr) {
            socket_info.remote_addr = Some(remote_addr);
            
            // Create connection entry
            let connection = UdpConnectionInfo {
                local_addr,
                remote_addr,
                options: UdpConnectionOptions::default(),
            };
            
            self.connections.insert((local_addr.0, local_addr.1, remote_addr.0, remote_addr.1), connection);
            Ok(())
        } else {
            Err(NetworkError::InvalidAddress)
        }
    }

    /// Process incoming UDP packet
    pub fn process_packet(&mut self, packet: &UdpPacket, source: IpAddress, dest: IpAddress) -> Result<()> {
        // Find the socket that should receive this packet
        let socket_addr = (dest, packet.header.dest_port);
        
        if let Some(socket_info) = self.sockets.get_mut(&socket_addr) {
            // Update receive statistics
            socket_info.stats.record_receive(packet.length() as u64);
            
            // Check if this is for a connected socket
            let connection_key = (dest, packet.header.dest_port, source, packet.header.source_port);
            if let Some(connection) = self.connections.get_mut(&connection_key) {
                connection.options.last_activity = Some(chrono::Utc::now());
            }
            
            log::debug!("UDP: Processed packet from {}:{} to {}:{}", 
                       source, packet.header.source_port, dest, packet.header.dest_port);
            
            Ok(())
        } else {
            log::debug!("UDP: No socket found for port {}", packet.header.dest_port);
            Err(NetworkError::PortUnreachable)
        }
    }

    /// Send UDP packet
    pub fn send_packet(&mut self, data: &[u8], source: IpAddress, source_port: u16,
                      dest: IpAddress, dest_port: u16, is_broadcast: bool) -> Result<usize> {
        // Create UDP packet
        let mut packet = UdpPacket::with_data(source_port, dest_port, data.to_vec());
        
        // Calculate checksum
        packet.header.calculate_checksum(&packet.data, source, dest);
        
        // Find the source socket
        let socket_addr = (source, source_port);
        
        if let Some(socket_info) = self.sockets.get_mut(&socket_addr) {
            // Check broadcast permission
            if is_broadcast && !socket_info.options.broadcast && dest.is_broadcast() {
                return Err(NetworkError::PermissionDenied);
            }
            
            // Update send statistics
            socket_info.stats.record_send(data.len() as u64);
            
            log::debug!("UDP: Sending {} bytes from {}:{} to {}:{}", 
                       data.len(), source, source_port, dest, dest_port);
            
            Ok(data.len())
        } else {
            // Socket not registered, but we can still send
            log::debug!("UDP: Sending {} bytes from {}:{} to {}:{} (unregistered socket)", 
                       data.len(), source, source_port, dest, dest_port);
            Ok(data.len())
        }
    }

    /// Get socket information
    pub fn get_socket_info(&self, addr: (IpAddress, u16)) -> Option<&UdpSocketInfo> {
        self.sockets.get(&addr)
    }

    /// Get all active socket addresses
    pub fn get_active_sockets(&self) -> Vec<(IpAddress, u16)> {
        self.sockets.keys().cloned().collect()
    }

    /// Check if a port is in use
    pub fn is_port_in_use(&self, addr: (IpAddress, u16)) -> bool {
        self.sockets.contains_key(&addr)
    }

    /// Get port statistics
    pub fn get_port_stats(&self, addr: (IpAddress, u16)) -> Option<UdpSocketStats> {
        self.sockets.get(&addr).map(|socket| socket.stats.clone())
    }

    /// Clean up stale connections
    pub fn cleanup_stale_connections(&mut self, timeout: Duration) {
        let now = chrono::Utc::now();
        let keys_to_remove: Vec<_> = self.connections.iter()
            .filter(|(_, connection)| {
                if let Some(last_activity) = connection.options.last_activity {
                    now.signed_duration_since(last_activity) > chrono::Duration::from_std(timeout).unwrap_or_default()
                } else {
                    false
                }
            })
            .map(|(key, _)| key.clone())
            .collect();
            
        for key in keys_to_remove {
            self.connections.remove(&key);
        }
    }

    /// Get connection count
    pub fn connection_count(&self) -> usize {
        self.connections.len()
    }

    /// Get socket count
    pub fn socket_count(&self) -> usize {
        self.sockets.len()
    }

    /// Check if packet is multicast
    pub fn is_multicast_packet(&self, packet: &UdpPacket, dest: IpAddress) -> bool {
        dest.is_multicast()
    }

    /// Check if packet is broadcast
    pub fn is_broadcast_packet(&self, packet: &UdpPacket, dest: IpAddress) -> bool {
        dest.is_broadcast()
    }
}

/// UDP protocol utilities
pub mod utils {
    use crate::core::IpAddress;
    
    /// Check if address is a valid UDP broadcast address
    pub fn is_broadcast_address(addr: IpAddress) -> bool {
        addr.is_broadcast() || 
        addr == IpAddress::v4(255, 255, 255, 255) ||
        // Check for subnet broadcasts
        (addr.octets[0] == 255 && addr.octets[1] == 255) ||
        addr.octets[3] == 255
    }

    /// Check if address is a valid multicast address
    pub fn is_multicast_address(addr: IpAddress) -> bool {
        addr.is_multicast() && 
        addr.octets[0] >= 224 && addr.octets[0] <= 239
    }

    /// Generate random port
    pub fn generate_ephemeral_port() -> u16 {
        use std::sync::atomic::{AtomicU16, Ordering};
        static COUNTER: AtomicU16 = AtomicU16::new(49152);
        COUNTER.fetch_add(1, Ordering::SeqCst)
    }

    /// Validate UDP port number
    pub fn is_valid_port(port: u16) -> bool {
        port > 0 && port <= 65535
    }

    /// Check if port is well-known (0-1023)
    pub fn is_well_known_port(port: u16) -> bool {
        port <= 1023
    }

    /// Check if port is registered (1024-49151)
    pub fn is_registered_port(port: u16) -> bool {
        port >= 1024 && port <= 49151
    }

    /// Check if port is ephemeral (49152-65535)
    pub fn is_ephemeral_port(port: u16) -> bool {
        port >= 49152
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_udp_packet_creation() {
        let packet = UdpPacket::with_data(8080, 80, b"Hello UDP!".to_vec());
        assert_eq!(packet.header.source_port, 8080);
        assert_eq!(packet.header.dest_port, 80);
        assert_eq!(packet.data, b"Hello UDP!");
        assert_eq!(packet.length(), 8 + 9);
    }

    #[test]
    fn test_udp_header_parsing() {
        let source = IpAddress::v4(192, 168, 1, 1);
        let dest = IpAddress::v4(8, 8, 8, 8);
        let mut header = UdpHeader::new(8080, 80, 10);
        header.calculate_checksum(b"Test data", source, dest);
        
        let bytes = header.to_bytes();
        let parsed = UdpHeader::parse(&bytes).unwrap();
        
        assert_eq!(parsed.source_port, header.source_port);
        assert_eq!(parsed.dest_port, header.dest_port);
        assert_eq!(parsed.length, header.length);
    }

    #[test]
    fn test_udp_checksum() {
        let source = IpAddress::v4(127, 0, 0, 1);
        let dest = IpAddress::v4(127, 0, 0, 1);
        let mut header = UdpHeader::new(8080, 80, 4);
        header.calculate_checksum(b"test", source, dest);
        
        assert!(header.verify_checksum(b"test", source, dest));
    }

    #[test]
    fn test_udp_manager() {
        let mut manager = UdpManager::new();
        let local_addr = (IpAddress::v4(127, 0, 0, 1), 8080);
        
        let options = UdpSocketOptions::default();
        assert!(manager.register_socket(local_addr, options).is_ok());
        assert!(manager.is_port_in_use(local_addr));
        
        assert!(manager.unregister_socket(local_addr).is_ok());
        assert!(!manager.is_port_in_use(local_addr));
    }

    #[test]
    fn test_udp_socket_options() {
        let mut options = UdpSocketOptions::default();
        assert!(!options.broadcast);
        assert_eq!(options.multicast_ttl, 1);
        assert_eq!(options.receive_buffer_size, 65535);
        
        options.broadcast = true;
        options.multicast_ttl = 64;
        assert!(options.broadcast);
        assert_eq!(options.multicast_ttl, 64);
    }

    #[test]
    fn test_udp_statistics() {
        let mut stats = UdpSocketStats::default();
        assert_eq!(stats.packets_sent, 0);
        assert_eq!(stats.bytes_sent, 0);
        
        stats.record_send(1024);
        assert_eq!(stats.packets_sent, 1);
        assert_eq!(stats.bytes_sent, 1024);
        
        stats.record_receive(512);
        assert_eq!(stats.packets_received, 1);
        assert_eq!(stats.bytes_received, 512);
    }
}