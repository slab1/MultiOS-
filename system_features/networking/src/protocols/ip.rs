//! Internet Protocol (IP) implementation
//!
//! This module provides IPv4 packet handling, fragmentation, and reassembly.

use crate::{Result, NetworkError};
use crate::core::IpAddress;
use std::collections::HashMap;

/// IP protocol numbers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpProtocol {
    /// Dummy protocol for TCP
    HopByHop = 0,
    /// Internet Control Message Protocol
    Icmp = 1,
    /// Internet Group Management Protocol
    Igmp = 2,
    /// Gateway-to-Gateway Protocol
    Ggp = 3,
    /// IP in IP encapsulation
    IpInIp = 4,
    /// Stream Protocol
    St = 5,
    /// Transmission Control Protocol
    Tcp = 6,
    /// Exterior Gateway Protocol
    Egp = 8,
    /// User Datagram Protocol
    Udp = 17,
    /// ISO Transport Protocol Class 4
    IsoTp4 = 29,
    /// Broadband Radio Service
    Brs = 32,
    /// VMTP
    Vmtp = 51,
    /// OSPF routing protocol
    Ospf = 89,
    /// IPX encapsulation
    IpPip = 94,
    /// ICMPv6
    Icmpv6 = 58,
    /// No Next Header for IPv6
    NoNext = 59,
    /// Destination Options for IPv6
    DestOpts = 60,
}

/// IP packet flags
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IpFlags {
    /// Reserved bit
    pub reserved: bool,
    /// Don't Fragment flag
    pub dont_fragment: bool,
    /// More Fragments flag
    pub more_fragments: bool,
}

impl IpFlags {
    /// Parse flags from byte
    pub fn from_byte(byte: u8) -> Self {
        Self {
            reserved: (byte & 0x80) != 0,
            dont_fragment: (byte & 0x40) != 0,
            more_fragments: (byte & 0x20) != 0,
        }
    }

    /// Convert to byte value
    pub fn to_byte(&self) -> u8 {
        (self.reserved as u8) << 7 |
        (self.dont_fragment as u8) << 6 |
        (self.more_fragments as u8) << 5
    }
}

/// IPv4 packet structure
#[derive(Debug, Clone)]
pub struct IpPacket {
    /// Version and header length (4 bits each)
    pub version_ihl: u8,
    /// Type of service
    pub tos: u8,
    /// Total length (header + data)
    pub total_length: u16,
    /// Identification
    pub identification: u16,
    /// Flags and fragment offset
    pub flags_frag_offset: u16,
    /// Time to live
    pub ttl: u8,
    /// Protocol
    pub protocol: IpProtocol,
    /// Header checksum
    pub header_checksum: u16,
    /// Source IP address
    pub source: IpAddress,
    /// Destination IP address
    pub destination: IpAddress,
    /// Options (if any)
    pub options: Vec<u8>,
    /// Payload data
    pub payload: Vec<u8>,
}

impl IpPacket {
    /// Create a new IP packet
    pub fn new(source: IpAddress, destination: IpAddress, protocol: IpProtocol, payload: Vec<u8>) -> Self {
        let header_length = 20; // Minimum IPv4 header length
        let total_length = header_length + payload.len();

        Self {
            version_ihl: (4 << 4) | (header_length / 4), // Version 4, IHL
            tos: 0,
            total_length: total_length as u16,
            identification: 0, // Will be set when sending
            flags_frag_offset: 0,
            ttl: 64,
            protocol,
            header_checksum: 0,
            source,
            destination,
            options: Vec::new(),
            payload,
        }
    }

    /// Parse IP packet from raw bytes
    pub fn parse(data: &[u8]) -> Result<Self> {
        if data.len() < 20 {
            return Err(NetworkError::InvalidAddress);
        }

        let version = (data[0] >> 4) & 0x0F;
        if version != 4 {
            return Err(NetworkError::InvalidAddress);
        }

        let ihl = data[0] & 0x0F;
        let header_length = (ihl as usize) * 4;

        if data.len() < header_length {
            return Err(NetworkError::InvalidAddress);
        }

        let total_length = ((data[2] as u16) << 8) | (data[3] as u16) as u16;
        if data.len() < total_length as usize {
            return Err(NetworkError::InvalidAddress);
        }

        let identification = ((data[4] as u16) << 8) | (data[5] as u16);
        let flags_frag_offset = ((data[6] as u16) << 8) | (data[7] as u16);
        let ttl = data[8];
        let protocol_num = data[9];
        let header_checksum = ((data[10] as u16) << 8) | (data[11] as u16);
        
        let source = IpAddress::from_bytes([
            data[12], data[13], data[14], data[15]
        ]);
        let destination = IpAddress::from_bytes([
            data[16], data[17], data[18], data[19]
        ]);

        // Parse options if present
        let options = if header_length > 20 {
            data[20..header_length].to_vec()
        } else {
            Vec::new()
        };

        // Extract payload
        let payload = if header_length < total_length as usize {
            data[header_length..total_length as usize].to_vec()
        } else {
            Vec::new()
        };

        let protocol = match protocol_num {
            1 => IpProtocol::Icmp,
            6 => IpProtocol::Tcp,
            17 => IpProtocol::Udp,
            _ => IpProtocol::Tcp, // Default
        };

        Ok(Self {
            version_ihl: data[0],
            tos: data[1],
            total_length,
            identification,
            flags_frag_offset,
            ttl,
            protocol,
            header_checksum,
            source,
            destination,
            options,
            payload,
        })
    }

    /// Convert IP packet to raw bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.total_length as usize);
        
        // Version and IHL
        bytes.push(self.version_ihl);
        
        // Type of Service
        bytes.push(self.tos);
        
        // Total Length
        bytes.push((self.total_length >> 8) as u8);
        bytes.push(self.total_length as u8);
        
        // Identification
        bytes.push((self.identification >> 8) as u8);
        bytes.push(self.identification as u8);
        
        // Flags and Fragment Offset
        bytes.push((self.flags_frag_offset >> 8) as u8);
        bytes.push(self.flags_frag_offset as u8);
        
        // TTL
        bytes.push(self.ttl);
        
        // Protocol
        bytes.push(self.protocol as u8);
        
        // Header Checksum (will be calculated)
        bytes.push((self.header_checksum >> 8) as u8);
        bytes.push(self.header_checksum as u8);
        
        // Source Address
        bytes.extend_from_slice(&self.source.octets);
        
        // Destination Address
        bytes.extend_from_slice(&self.destination.octets);
        
        // Options
        bytes.extend_from_slice(&self.options);
        
        // Pad options to 4-byte boundary
        while (bytes.len() % 4) != 0 {
            bytes.push(0);
        }
        
        // Calculate and set checksum
        let checksum = crate::protocols::utils::calculate_checksum(&bytes);
        bytes[10] = (checksum >> 8) as u8;
        bytes[11] = checksum as u8;
        
        // Payload
        bytes.extend_from_slice(&self.payload);
        
        bytes
    }

    /// Get the header length in bytes
    pub fn header_length(&self) -> usize {
        ((self.version_ihl & 0x0F) as usize) * 4
    }

    /// Get the payload length
    pub fn payload_length(&self) -> usize {
        self.payload.len()
    }

    /// Get the flags
    pub fn flags(&self) -> IpFlags {
        IpFlags::from_byte((self.flags_frag_offset >> 8) as u8)
    }

    /// Get the fragment offset
    pub fn fragment_offset(&self) -> u16 {
        self.flags_frag_offset & 0x1FFF
    }

    /// Set the flags
    pub fn set_flags(&mut self, flags: IpFlags) {
        self.flags_frag_offset = (self.flags_frag_offset & 0xE000) | 
                                 ((flags.to_byte() as u16) << 8) | 
                                 (self.fragment_offset());
    }

    /// Set the fragment offset
    pub fn set_fragment_offset(&mut self, offset: u16) {
        self.flags_frag_offset = (self.flags_frag_offset & 0xE000) | offset;
    }

    /// Decrement TTL
    pub fn decrement_ttl(&mut self) {
        self.ttl = self.ttl.saturating_sub(1);
    }

    /// Check if packet can be fragmented
    pub fn can_fragment(&self) -> bool {
        !self.flags().dont_fragment
    }

    /// Check if this is a fragment
    pub fn is_fragment(&self) -> bool {
        self.fragment_offset() != 0 || self.flags().more_fragments
    }

    /// Check if this is the last fragment
    pub fn is_last_fragment(&self) -> bool {
        !self.flags().more_fragments
    }

    /// Verify packet integrity
    pub fn verify_checksum(&self) -> bool {
        let mut bytes = self.to_bytes();
        // Reset checksum to zero for verification
        bytes[10] = 0;
        bytes[11] = 0;
        crate::protocols::utils::verify_checksum(&bytes)
    }

    /// Calculate and set checksum
    pub fn update_checksum(&mut self) {
        let mut bytes = self.to_bytes();
        bytes[10] = 0;
        bytes[11] = 0;
        self.header_checksum = crate::protocols::utils::calculate_checksum(&bytes);
    }
}

/// IP packet processing
pub struct IpProcessor {
    /// Fragment reassembly buffer
    fragments: HashMap<(u16, IpAddress, IpAddress), FragmentBuffer>,
    /// Next packet identification
    next_id: u16,
}

struct FragmentBuffer {
    /// Fragment identification
    id: u16,
    /// Source and destination addresses
    source: IpAddress,
    destination: IpAddress,
    /// Received fragments
    fragments: Vec<Fragment>,
    /// Timeout timer
    timeout: std::time::Instant,
}

struct Fragment {
    /// Fragment offset
    offset: u16,
    /// More fragments flag
    more_fragments: bool,
    /// Fragment data
    data: Vec<u8>,
}

impl IpProcessor {
    /// Create a new IP processor
    pub fn new() -> Self {
        Self {
            fragments: HashMap::new(),
            next_id: 1,
        }
    }

    /// Process incoming IP packet
    pub fn process_packet(&mut self, packet: &[u8]) -> Result<Vec<IpPacket>> {
        let ip_packet = IpPacket::parse(packet)?;
        
        // Check TTL
        if ip_packet.ttl == 0 {
            return Err(NetworkError::HostUnreachable);
        }
        
        // Check if it's a fragment
        if ip_packet.is_fragment() {
            self.handle_fragment(ip_packet)
        } else {
            // Complete packet, return it
            Ok(vec![ip_packet])
        }
    }

    /// Handle IP fragment reassembly
    fn handle_fragment(&mut self, fragment: IpPacket) -> Result<Vec<IpPacket>> {
        let key = (fragment.identification, fragment.source, fragment.destination);
        
        // Check if we have a fragment buffer for this packet
        let fragment_info = Fragment {
            offset: fragment.fragment_offset(),
            more_fragments: fragment.flags().more_fragments,
            data: fragment.payload,
        };
        
        if let Some(buffer) = self.fragments.get_mut(&key) {
            // Add fragment to existing buffer
            buffer.fragments.push(fragment_info);
            
            // Check if we have all fragments
            if !fragment_info.more_fragments {
                self.reassemble_packet(key)
            } else {
                Ok(Vec::new()) // More fragments expected
            }
        } else {
            // Create new fragment buffer
            let mut fragments = Vec::new();
            fragments.push(fragment_info);
            
            let buffer = FragmentBuffer {
                id: fragment.identification,
                source: fragment.source,
                destination: fragment.destination,
                fragments,
                timeout: std::time::Instant::now(),
            };
            
            self.fragments.insert(key, buffer);
            Ok(Vec::new()) // More fragments expected
        }
    }

    /// Reassemble IP fragments into complete packet
    fn reassemble_packet(&mut self, key: (u16, IpAddress, IpAddress)) -> Result<Vec<IpPacket>> {
        if let Some(buffer) = self.fragments.remove(&key) {
            let mut fragments_data = Vec::new();
            for fragment in buffer.fragments {
                fragments_data.push(fragment.data);
            }
            
            let packet = crate::protocols::utils::reassemble_ip_fragments(&fragments_data)?;
            let ip_packet = IpPacket::parse(&packet)?;
            Ok(vec![ip_packet])
        } else {
            Err(NetworkError::InvalidAddress)
        }
    }

    /// Send IP packet
    pub fn send_packet(&mut self, packet: &mut IpPacket) -> Result<Vec<Vec<u8>>> {
        // Set identification
        packet.identification = self.next_id;
        self.next_id = self.next_id.wrapping_add(1);
        
        // Update checksum
        packet.update_checksum();
        
        // Fragment if necessary (MTU handling)
        let max_packet_size = 1500; // Default MTU
        if packet.total_length > max_packet_size as u16 && packet.can_fragment() {
            self.fragment_packet(packet, max_packet_size)
        } else {
            Ok(vec![packet.to_bytes()])
        }
    }

    /// Fragment IP packet for transmission
    fn fragment_packet(&self, packet: &IpPacket, mtu: u16) -> Result<Vec<Vec<u8>>> {
        let raw_packet = packet.to_bytes();
        let fragments = crate::protocols::utils::fragment_ip_packet(&raw_packet, mtu);
        
        // Parse fragments back to IPPacket structs if needed
        let mut fragment_packets = Vec::new();
        for fragment_data in fragments {
            if let Ok(ip_fragment) = IpPacket::parse(&fragment_data) {
                fragment_packets.push(fragment_data);
            }
        }
        
        Ok(fragment_packets)
    }

    /// Clean up old fragments
    pub fn cleanup_fragments(&mut self) {
        let timeout_duration = std::time::Duration::from_secs(60);
        let now = std::time::Instant::now();
        
        self.fragments.retain(|_, buffer| {
            now.duration_since(buffer.timeout) < timeout_duration
        });
    }
}

/// Protocol handlers for IP packets
pub struct TcpProtocolHandler;

impl ProtocolHandler for TcpProtocolHandler {
    fn process_incoming(&self, packet: &[u8], source: IpAddress, dest: IpAddress) -> Result<()> {
        log::debug!("Processing TCP packet from {} to {}", source, dest);
        // TCP processing would be handled by the TCP module
        Ok(())
    }

    fn generate_packet(&self, data: &[u8], source: IpAddress, dest: IpAddress) -> Result<Vec<u8>> {
        // This would generate an IP packet for TCP data
        let tcp_packet = crate::protocols::tcp::TcpPacket::generate_tcp_packet(data, source, dest)?;
        let mut ip_packet = IpPacket::new(source, dest, IpProtocol::Tcp, tcp_packet.to_bytes());
        Ok(ip_packet.to_bytes())
    }

    fn protocol_number(&self) -> u8 {
        IpProtocol::Tcp as u8
    }
}

pub struct UdpProtocolHandler;

impl ProtocolHandler for UdpProtocolHandler {
    fn process_incoming(&self, packet: &[u8], source: IpAddress, dest: IpAddress) -> Result<()> {
        log::debug!("Processing UDP packet from {} to {}", source, dest);
        // UDP processing would be handled by the UDP module
        Ok(())
    }

    fn generate_packet(&self, data: &[u8], source: IpAddress, dest: IpAddress) -> Result<Vec<u8>> {
        // This would generate an IP packet for UDP data
        let udp_packet = crate::protocols::udp::UdpPacket::generate_udp_packet(data, source, dest)?;
        let mut ip_packet = IpPacket::new(source, dest, IpProtocol::Udp, udp_packet.to_bytes());
        Ok(ip_packet.to_bytes())
    }

    fn protocol_number(&self) -> u8 {
        IpProtocol::Udp as u8
    }
}

pub struct IcmpProtocolHandler;

impl ProtocolHandler for IcmpProtocolHandler {
    fn process_incoming(&self, packet: &[u8], source: IpAddress, dest: IpAddress) -> Result<()> {
        log::debug!("Processing ICMP packet from {} to {}", source, dest);
        // ICMP processing would be handled by the ICMP module
        Ok(())
    }

    fn generate_packet(&self, data: &[u8], source: IpAddress, dest: IpAddress) -> Result<Vec<u8>> {
        // This would generate an IP packet for ICMP data
        let icmp_packet = crate::protocols::icmp::IcmpPacket::generate_icmp_packet(data, source, dest)?;
        let mut ip_packet = IpPacket::new(source, dest, IpProtocol::Icmp, icmp_packet.to_bytes());
        Ok(ip_packet.to_bytes())
    }

    fn protocol_number(&self) -> u8 {
        IpProtocol::Icmp as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ip_packet_creation() {
        let source = IpAddress::v4(192, 168, 1, 1);
        let dest = IpAddress::v4(8, 8, 8, 8);
        let payload = b"Hello, IP!".to_vec();
        
        let packet = IpPacket::new(source, dest, IpProtocol::Tcp, payload);
        assert_eq!(packet.source, source);
        assert_eq!(packet.destination, dest);
        assert_eq!(packet.protocol, IpProtocol::Tcp);
        assert_eq!(packet.payload_length(), 10);
    }

    #[test]
    fn test_ip_packet_parsing() {
        let source = IpAddress::v4(192, 168, 1, 1);
        let dest = IpAddress::v4(8, 8, 8, 8);
        let payload = b"Hello, IP!".to_vec();
        
        let mut packet = IpPacket::new(source, dest, IpProtocol::Tcp, payload);
        packet.update_checksum();
        
        let bytes = packet.to_bytes();
        let parsed = IpPacket::parse(&bytes).unwrap();
        
        assert_eq!(parsed.source, source);
        assert_eq!(parsed.destination, dest);
        assert_eq!(parsed.protocol, IpProtocol::Tcp);
        assert_eq!(parsed.payload, b"Hello, IP!");
        assert!(parsed.verify_checksum());
    }

    #[test]
    fn test_ip_flags() {
        let flags = IpFlags {
            reserved: false,
            dont_fragment: true,
            more_fragments: false,
        };
        
        let byte = flags.to_byte();
        let parsed = IpFlags::from_byte(byte);
        
        assert_eq!(parsed.reserved, flags.reserved);
        assert_eq!(parsed.dont_fragment, flags.dont_fragment);
        assert_eq!(parsed.more_fragments, flags.more_fragments);
    }

    #[test]
    fn test_ip_fragmentation() {
        let source = IpAddress::v4(192, 168, 1, 1);
        let dest = IpAddress::v4(8, 8, 8, 8);
        let payload = vec![0u8; 3000]; // Large payload
        
        let packet = IpPacket::new(source, dest, IpProtocol::Tcp, payload);
        let fragments = crate::protocols::utils::fragment_ip_packet(&packet.to_bytes(), 1500);
        
        assert!(fragments.len() > 1);
    }
}