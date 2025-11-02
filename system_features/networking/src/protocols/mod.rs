//! Network protocols implementation
//!
//! This module contains the implementation of core network protocols:
//! - IP (Internet Protocol)
//! - TCP (Transmission Control Protocol)
//! - UDP (User Datagram Protocol)
//! - ICMP (Internet Control Message Protocol)

pub mod ip;
pub mod tcp;
pub mod udp;
pub mod icmp;

// Re-export protocol types for convenience
pub use ip::{IpPacket, IpProtocol, IpFlags};
pub use tcp::{TcpPacket, TcpConnection, TcpState, TcpFlags, TcpOptions};
pub use udp::{UdpPacket, UdpHeader};
pub use icmp::{IcmpPacket, IcmpType, IcmpCode, IcmpEcho};

use crate::{Result, NetworkError};
use crate::core::IpAddress;

/// Protocol handler trait for processing network packets
pub trait ProtocolHandler {
    /// Process an incoming packet
    fn process_incoming(&self, packet: &[u8], source: IpAddress, dest: IpAddress) -> Result<()>;
    
    /// Generate a packet for outgoing data
    fn generate_packet(&self, data: &[u8], source: IpAddress, dest: IpAddress) -> Result<Vec<u8>>;
    
    /// Get the protocol number this handler supports
    fn protocol_number(&self) -> u8;
}

/// Protocol registry for managing protocol handlers
struct ProtocolRegistry {
    handlers: std::collections::HashMap<u8, Box<dyn ProtocolHandler>>,
}

impl ProtocolRegistry {
    fn new() -> Self {
        let mut registry = Self {
            handlers: std::collections::HashMap::new(),
        };
        
        // Register default protocols
        registry.register_default_protocols();
        registry
    }
    
    fn register_default_protocols(&mut self) {
        // Register TCP handler
        let tcp_handler = ip::TcpProtocolHandler;
        self.handlers.insert(IpProtocol::Tcp as u8, Box::new(tcp_handler));
        
        // Register UDP handler
        let udp_handler = ip::UdpProtocolHandler;
        self.handlers.insert(IpProtocol::Udp as u8, Box::new(udp_handler));
        
        // Register ICMP handler
        let icmp_handler = ip::IcmpProtocolHandler;
        self.handlers.insert(IpProtocol::Icmp as u8, Box::new(icmp_handler));
    }
    
    fn register_handler(&mut self, protocol: IpProtocol, handler: Box<dyn ProtocolHandler>) {
        self.handlers.insert(protocol as u8, handler);
    }
    
    fn get_handler(&self, protocol: IpProtocol) -> Option<&Box<dyn ProtocolHandler>> {
        self.handlers.get(&(protocol as u8))
    }
}

use parking_lot::RwLock;
use std::sync::Arc;

/// Global protocol registry
static PROTOCOL_REGISTRY: RwLock<ProtocolRegistry> = 
    RwLock::new(ProtocolRegistry::new());

/// Register a protocol handler
pub fn register_protocol_handler(protocol: IpProtocol, handler: Box<dyn ProtocolHandler>) {
    let mut registry = PROTOCOL_REGISTRY.write();
    registry.register_handler(protocol, handler);
}

/// Get a protocol handler
pub fn get_protocol_handler(protocol: IpProtocol) -> Option<Arc<dyn ProtocolHandler>> {
    let registry = PROTOCOL_REGISTRY.read();
    registry.get_handler(protocol).map(|h| Arc::new(**h) as Arc<dyn ProtocolHandler>)
}

/// Protocol utilities
pub mod utils {
    use crate::core::IpAddress;
    
    /// Calculate IP checksum
    pub fn calculate_checksum(data: &[u8]) -> u16 {
        let mut sum: u32 = 0;
        
        // Sum all 16-bit words
        for chunk in data.chunks(2) {
            if chunk.len() == 2 {
                sum += ((chunk[0] as u16) << 8) | (chunk[1] as u16);
            } else if chunk.len() == 1 {
                sum += (chunk[0] as u16) << 8;
            }
        }
        
        // Add carry bits
        while (sum >> 16) != 0 {
            sum = (sum & 0xFFFF) + (sum >> 16);
        }
        
        // One's complement
        (!sum) as u16
    }
    
    /// Verify IP checksum
    pub fn verify_checksum(data: &[u8]) -> bool {
        calculate_checksum(data) == 0
    }
    
    /// Calculate TCP/UDP pseudo-header checksum
    pub fn pseudo_header_checksum(source: IpAddress, dest: IpAddress, protocol: u8, length: u16) -> u16 {
        let mut sum: u32 = 0;
        
        // Add source address
        for &octet in &source.octets {
            sum += octet as u32;
        }
        
        // Add destination address
        for &octet in &dest.octets {
            sum += octet as u32;
        }
        
        // Add protocol and length
        sum += (protocol as u32) << 8;
        sum += length as u32;
        
        // Add carry bits
        while (sum >> 16) != 0 {
            sum = (sum & 0xFFFF) + (sum >> 16);
        }
        
        (!sum) as u16
    }
    
    /// Fragment IP packet if necessary
    pub fn fragment_ip_packet(packet: &[u8], mtu: u16) -> Vec<Vec<u8>> {
        let mut fragments = Vec::new();
        
        // IP header is 20 bytes minimum
        if packet.len() <= 20 {
            fragments.push(packet.to_vec());
            return fragments;
        }
        
        let ip_header_len = (packet[0] & 0x0F) * 4;
        let data_start = ip_header_len;
        let payload_len = packet.len() - ip_header_len;
        
        // Calculate fragment data size (must be multiple of 8 bytes, except last fragment)
        let max_data_per_fragment = ((mtu - ip_header_len) / 8) * 8;
        
        if max_data_per_fragment < 8 {
            fragments.push(packet.to_vec());
            return fragments;
        }
        
        let mut offset = 0;
        let mut fragment_id = ((packet[4] as u16) << 8) | (packet[5] as u16);
        
        while offset < payload_len {
            let fragment_data_len = std::cmp::min(max_data_per_fragment, payload_len - offset);
            let is_last_fragment = offset + fragment_data_len >= payload_len;
            
            // Create fragment
            let mut fragment = packet[..ip_header_len].to_vec();
            
            // Set fragment flags
            if offset == 0 {
                // First fragment
                let flags = packet[6] & 0xE0 | ((packet[6] & 0x1F) & 0x1F);
                fragment[6] = flags;
            } else if !is_last_fragment {
                // Middle fragment
                let flags = packet[6] & 0xE0 | 0x01; // Set MF bit
                fragment[6] = flags;
            } else {
                // Last fragment
                let flags = packet[6] & 0xFE; // Clear MF bit
                fragment[6] = flags;
            }
            
            // Set fragment offset
            let fragment_offset = (offset / 8) as u16;
            fragment[6] |= ((fragment_offset >> 8) & 0x1F) as u8;
            fragment[7] = (fragment_offset & 0xFF) as u8;
            
            // Add fragment data
            let data_end = data_start + fragment_data_len;
            fragment.extend_from_slice(&packet[data_start..data_end]);
            
            // Update total length
            fragment[2] = ((fragment.len() >> 8) & 0xFF) as u8;
            fragment[3] = (fragment.len() & 0xFF) as u8;
            
            fragments.push(fragment);
            
            offset += fragment_data_len;
            fragment_id = fragment_id.wrapping_add(1);
        }
        
        fragments
    }
    
    /// Reassemble IP fragments
    pub fn reassemble_ip_fragments(fragments: &[Vec<u8>]) -> Result<Vec<u8>> {
        if fragments.is_empty() {
            return Err(NetworkError::InvalidAddress);
        }
        
        // Get fragment information from first fragment
        let first_fragment = &fragments[0];
        let ip_header_len = (first_fragment[0] & 0x0F) * 4;
        let fragment_id = ((first_fragment[4] as u16) << 8) | (first_fragment[5] as u16);
        
        // Extract fragment offset from first fragment
        let mut first_frag_offset = ((first_fragment[6] & 0x1F) as u16) << 8 | (first_fragment[7] as u16);
        first_frag_offset *= 8;
        
        // Check if first fragment has MF bit set
        let has_mf = (first_fragment[6] & 0x01) != 0;
        
        // If it's the only fragment and no more fragments expected, return it
        if !has_mf && fragments.len() == 1 && first_frag_offset == 0 {
            return Ok(first_fragment.clone());
        }
        
        // Collect all fragments
        let mut fragments_map = std::collections::HashMap::new();
        let mut total_data_len = 0;
        let mut got_first_frag = false;
        
        for fragment in fragments {
            let frag_header_len = (fragment[0] & 0x0F) * 4;
            let frag_data_start = frag_header_len;
            
            let frag_offset = ((fragment[6] & 0x1F) as u16) << 8 | (fragment[7] as u16);
            let frag_offset_bytes = (frag_offset as usize) * 8;
            let is_first_frag = frag_offset_bytes == 0;
            
            if is_first_frag {
                got_first_frag = true;
                if fragment[6] & 0x01 == 0 {
                    // No more fragments
                    total_data_len = frag_offset_bytes + (fragment.len() - frag_data_start);
                } else {
                    // More fragments follow, set initial estimate
                    total_data_len = frag_offset_bytes + (fragment.len() - frag_data_start) + 8 * 1024; // Estimate
                }
            } else {
                let fragment_data_len = fragment.len() - frag_data_start;
                if frag_offset_bytes + fragment_data_len > total_data_len {
                    total_data_len = frag_offset_bytes + fragment_data_len;
                }
            }
            
            fragments_map.insert(frag_offset_bytes, fragment.clone());
        }
        
        // Verify we have the first fragment
        if !got_first_frag {
            return Err(NetworkError::InvalidAddress);
        }
        
        // Reassemble the packet
        let mut reassembled = Vec::with_capacity(ip_header_len + total_data_len);
        
        // Add IP header from first fragment
        reassembled.extend_from_slice(&first_fragment[..ip_header_len]);
        
        // Add data from all fragments in order
        let mut offset = 0;
        while offset < total_data_len {
            if let Some(fragment) = fragments_map.get(&offset) {
                let frag_header_len = (fragment[0] & 0x0F) * 4;
                let frag_data = &fragment[frag_header_len..];
                
                // Pad with zeros if this fragment is shorter than expected
                let expected_len = std::cmp::min(8 * 1024, total_data_len - offset);
                let actual_len = frag_data.len();
                
                if actual_len < expected_len && offset + expected_len < total_data_len {
                    // This is not the last fragment but it's shorter than expected
                    // This might be an error, but we'll continue
                }
                
                reassembled.extend_from_slice(frag_data);
                offset += actual_len;
            } else {
                // Missing fragment, pad with zeros
                let pad_len = std::cmp::min(8 * 1024, total_data_len - offset);
                reassembled.extend(std::iter::repeat(0u8).take(pad_len));
                offset += pad_len;
            }
        }
        
        // Update total length in IP header
        reassembled[2] = ((reassembled.len() >> 8) & 0xFF) as u8;
        reassembled[3] = (reassembled.len() & 0xFF) as u8;
        
        // Clear fragment-related fields
        reassembled[6] &= 0xE0; // Clear fragment offset and MF bit
        reassembled[7] = 0;
        
        Ok(reassembled)
    }
}