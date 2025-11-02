//! Internet Control Message Protocol (ICMP) implementation
//!
//! This module provides ICMP packet handling for network diagnostics,
//! error reporting, and ping functionality.

use crate::{Result, NetworkError};
use crate::core::IpAddress;
use std::collections::HashMap;

/// ICMP message types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IcmpType {
    /// Echo Reply (0)
    EchoReply = 0,
    /// Destination Unreachable (3)
    DestinationUnreachable = 3,
    /// Source Quench (4)
    SourceQuench = 4,
    /// Redirect (5)
    Redirect = 5,
    /// Echo Request (8)
    EchoRequest = 8,
    /// Time Exceeded (11)
    TimeExceeded = 11,
    /// Parameter Problem (12)
    ParameterProblem = 12,
    /// Timestamp Request (13)
    TimestampRequest = 13,
    /// Timestamp Reply (14)
    TimestampReply = 14,
    /// Information Request (15)
    InformationRequest = 15,
    /// Information Reply (16)
    InformationReply = 16,
}

/// ICMP message codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IcmpCode {
    /// Network unreachable
    NetworkUnreachable = 0,
    /// Host unreachable
    HostUnreachable = 1,
    /// Protocol unreachable
    ProtocolUnreachable = 2,
    /// Port unreachable
    PortUnreachable = 3,
    /// Fragmentation needed and DF set
    FragmentationNeeded = 4,
    /// Source route failed
    SourceRouteFailed = 5,
    /// Destination network unknown
    DestinationNetworkUnknown = 6,
    /// Destination host unknown
    DestinationHostUnknown = 7,
    /// Source host isolated
    SourceHostIsolated = 8,
    /// Network administratively prohibited
    NetworkProhibited = 9,
    /// Host administratively prohibited
    HostProhibited = 10,
    /// Network unreachable for type of service
    NetworkUnreachableForTos = 11,
    /// Host unreachable for type of service
    HostUnreachableForTos = 12,
    /// Communication administratively prohibited
    CommunicationProhibited = 13,
    /// Host precedence violation
    HostPrecedenceViolation = 14,
    /// Precedence cutoff in effect
    PrecedenceCutoff = 15,
    /// Redirect for network
    RedirectForNetwork = 0,
    /// Redirect for host
    RedirectForHost = 1,
    /// Redirect for type of service and network
    RedirectForTosAndNetwork = 2,
    /// Redirect for type of service and host
    RedirectForTosAndHost = 3,
    /// Time to live exceeded in transit
    TtlExceededInTransit = 0,
    /// Fragment reassembly time exceeded
    FragmentReassemblyTimeExceeded = 1,
    /// Pointer indicates the error
    PointerIndicatesError = 0,
    /// Missing a required option
    MissingRequiredOption = 1,
    /// Bad length
    BadLength = 2,
}

/// ICMP Echo structure for ping requests/replies
#[derive(Debug, Clone)]
pub struct IcmpEcho {
    /// Identifier
    pub identifier: u16,
    /// Sequence number
    pub sequence_number: u16,
    /// Data payload
    pub data: Vec<u8>,
}

impl IcmpEcho {
    /// Create a new ICMP Echo
    pub fn new(identifier: u16, sequence_number: u16, data: Vec<u8>) -> Self {
        Self {
            identifier,
            sequence_number,
            data,
        }
    }

    /// Parse ICMP Echo from bytes
    pub fn parse(data: &[u8]) -> Result<Self> {
        if data.len() < 4 {
            return Err(NetworkError::InvalidAddress);
        }

        let identifier = ((data[0] as u16) << 8) | (data[1] as u16);
        let sequence_number = ((data[2] as u16) << 8) | (data[3] as u16);
        let payload = if data.len() > 4 {
            data[4..].to_vec()
        } else {
            Vec::new()
        };

        Ok(Self::new(identifier, sequence_number, payload))
    }

    /// Convert to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(4 + self.data.len());
        
        bytes.push((self.identifier >> 8) as u8);
        bytes.push(self.identifier as u8);
        
        bytes.push((self.sequence_number >> 8) as u8);
        bytes.push(self.sequence_number as u8);
        
        bytes.extend_from_slice(&self.data);
        
        bytes
    }
}

/// ICMP Destination Unreachable structure
#[derive(Debug, Clone)]
pub struct IcmpDestinationUnreachable {
    /// Code describing why destination is unreachable
    pub code: IcmpCode,
    /// Unused field (must be zero)
    pub unused: u32,
    /// IP header and first 8 bytes of original datagram
    pub original_datagram: Vec<u8>,
}

impl IcmpDestinationUnreachable {
    /// Create a new Destination Unreachable message
    pub fn new(code: IcmpCode, original_datagram: Vec<u8>) -> Self {
        Self {
            code,
            unused: 0,
            original_datagram,
        }
    }

    /// Parse from bytes
    pub fn parse(data: &[u8]) -> Result<Self> {
        if data.len() < 8 {
            return Err(NetworkError::InvalidAddress);
        }

        let code_byte = data[0];
        let unused = ((data[4] as u32) << 24) |
                     ((data[5] as u32) << 16) |
                     ((data[6] as u32) << 8) |
                     (data[7] as u32);

        let code = match code_byte {
            0 => IcmpCode::NetworkUnreachable,
            1 => IcmpCode::HostUnreachable,
            2 => IcmpCode::ProtocolUnreachable,
            3 => IcmpCode::PortUnreachable,
            4 => IcmpCode::FragmentationNeeded,
            5 => IcmpCode::SourceRouteFailed,
            6 => IcmpCode::DestinationNetworkUnknown,
            7 => IcmpCode::DestinationHostUnknown,
            8 => IcmpCode::SourceHostIsolated,
            9 => IcmpCode::NetworkProhibited,
            10 => IcmpCode::HostProhibited,
            11 => IcmpCode::NetworkUnreachableForTos,
            12 => IcmpCode::HostUnreachableForTos,
            13 => IcmpCode::CommunicationProhibited,
            14 => IcmpCode::HostPrecedenceViolation,
            15 => IcmpCode::PrecedenceCutoff,
            _ => IcmpCode::NetworkUnreachable,
        };

        let original_datagram = data[8..].to_vec();

        Ok(Self::new(code, original_datagram))
    }

    /// Convert to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(8 + self.original_datagram.len());
        
        bytes.push(self.code as u8);
        bytes.push(0); // Unused
        bytes.push(0);
        bytes.push(0);
        
        bytes.extend_from_slice(&self.unused.to_be_bytes());
        bytes.extend_from_slice(&self.original_datagram);
        
        bytes
    }
}

/// ICMP Time Exceeded structure
#[derive(Debug, Clone)]
pub struct IcmpTimeExceeded {
    /// Code describing why time was exceeded
    pub code: IcmpCode,
    /// Unused field
    pub unused: u32,
    /// IP header and first 8 bytes of original datagram
    pub original_datagram: Vec<u8>,
}

impl IcmpTimeExceeded {
    /// Create a new Time Exceeded message
    pub fn new(code: IcmpCode, original_datagram: Vec<u8>) -> Self {
        Self {
            code,
            unused: 0,
            original_datagram,
        }
    }

    /// Parse from bytes
    pub fn parse(data: &[u8]) -> Result<Self> {
        if data.len() < 8 {
            return Err(NetworkError::InvalidAddress);
        }

        let code_byte = data[0];
        let unused = ((data[4] as u32) << 24) |
                     ((data[5] as u32) << 16) |
                     ((data[6] as u32) << 8) |
                     (data[7] as u32);

        let code = match code_byte {
            0 => IcmpCode::TtlExceededInTransit,
            1 => IcmpCode::FragmentReassemblyTimeExceeded,
            _ => IcmpCode::TtlExceededInTransit,
        };

        let original_datagram = data[8..].to_vec();

        Ok(Self::new(code, original_datagram))
    }

    /// Convert to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(8 + self.original_datagram.len());
        
        bytes.push(self.code as u8);
        bytes.push(0); // Unused
        bytes.push(0);
        bytes.push(0);
        
        bytes.extend_from_slice(&self.unused.to_be_bytes());
        bytes.extend_from_slice(&self.original_datagram);
        
        bytes
    }
}

/// ICMP Parameter Problem structure
#[derive(Debug, Clone)]
pub struct IcmpParameterProblem {
    /// Pointer to the octet where the error was detected
    pub pointer: u8,
    /// Unused field
    pub unused: [u8; 3],
    /// IP header and first 8 bytes of original datagram
    pub original_datagram: Vec<u8>,
}

impl IcmpParameterProblem {
    /// Create a new Parameter Problem message
    pub fn new(pointer: u8, original_datagram: Vec<u8>) -> Self {
        Self {
            pointer,
            unused: [0, 0, 0],
            original_datagram,
        }
    }

    /// Parse from bytes
    pub fn parse(data: &[u8]) -> Result<Self> {
        if data.len() < 8 {
            return Err(NetworkError::InvalidAddress);
        }

        let pointer = data[0];
        let unused = [data[1], data[2], data[3]];
        let original_datagram = data[8..].to_vec();

        Ok(Self::new(pointer, original_datagram))
    }

    /// Convert to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(8 + self.original_datagram.len());
        
        bytes.push(self.pointer);
        bytes.extend_from_slice(&self.unused);
        
        bytes.push(0); // Unused
        bytes.push(0);
        bytes.push(0);
        bytes.push(0);
        
        bytes.extend_from_slice(&self.original_datagram);
        
        bytes
    }
}

/// ICMP packet structure
#[derive(Debug, Clone)]
pub struct IcmpPacket {
    /// ICMP type
    pub icmp_type: IcmpType,
    /// ICMP code
    pub code: u8,
    /// Checksum
    pub checksum: u16,
    /// Type-specific data
    pub data: Vec<u8>,
}

impl IcmpPacket {
    /// Create a new ICMP packet
    pub fn new(icmp_type: IcmpType, code: u8, data: Vec<u8>) -> Self {
        Self {
            icmp_type,
            code,
            checksum: 0, // Will be calculated
            data,
        }
    }

    /// Parse ICMP packet from raw bytes
    pub fn parse(data: &[u8]) -> Result<Self> {
        if data.len() < 4 {
            return Err(NetworkError::InvalidAddress);
        }

        let icmp_type = data[0];
        let code = data[1];
        let checksum = ((data[2] as u16) << 8) | (data[3] as u16);
        let payload = if data.len() > 4 {
            data[4..].to_vec()
        } else {
            Vec::new()
        };

        let icmp_type_enum = match icmp_type {
            0 => IcmpType::EchoReply,
            3 => IcmpType::DestinationUnreachable,
            4 => IcmpType::SourceQuench,
            5 => IcmpType::Redirect,
            8 => IcmpType::EchoRequest,
            11 => IcmpType::TimeExceeded,
            12 => IcmpType::ParameterProblem,
            13 => IcmpType::TimestampRequest,
            14 => IcmpType::TimestampReply,
            15 => IcmpType::InformationRequest,
            16 => IcmpType::InformationReply,
            _ => IcmpType::EchoRequest, // Default
        };

        Ok(Self {
            icmp_type: icmp_type_enum,
            code,
            checksum,
            data: payload,
        })
    }

    /// Convert ICMP packet to raw bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(4 + self.data.len());
        
        bytes.push(self.icmp_type as u8);
        bytes.push(self.code);
        
        // Checksum will be calculated later
        bytes.push((self.checksum >> 8) as u8);
        bytes.push(self.checksum as u8);
        
        bytes.extend_from_slice(&self.data);
        
        bytes
    }

    /// Calculate and set checksum
    pub fn update_checksum(&mut self) {
        let mut bytes = self.to_bytes();
        bytes[2] = 0; // Reset checksum
        bytes[3] = 0;
        self.checksum = crate::protocols::utils::calculate_checksum(&bytes);
    }

    /// Verify checksum
    pub fn verify_checksum(&self) -> bool {
        let mut bytes = self.to_bytes();
        bytes[2] = 0; // Reset checksum
        bytes[3] = 0;
        crate::protocols::utils::verify_checksum(&bytes)
    }

    /// Generate ICMP Echo Request packet
    pub fn generate_echo_request(identifier: u16, sequence_number: u16, data: Vec<u8>) -> Self {
        let echo = IcmpEcho::new(identifier, sequence_number, data);
        Self::new(IcmpType::EchoRequest, 0, echo.to_bytes())
    }

    /// Generate ICMP Echo Reply packet
    pub fn generate_echo_reply(request: &IcmpPacket, source: IpAddress, dest: IpAddress) -> Result<Self> {
        let echo = IcmpEcho::parse(&request.data)?;
        let mut reply = Self::new(IcmpType::EchoReply, 0, echo.to_bytes());
        reply.update_checksum();
        Ok(reply)
    }

    /// Generate ICMP Destination Unreachable packet
    pub fn generate_destination_unreachable(code: IcmpCode, original_datagram: Vec<u8>) -> Self {
        let unreachable = IcmpDestinationUnreachable::new(code, original_datagram);
        Self::new(IcmpType::DestinationUnreachable, code as u8, unreachable.to_bytes())
    }

    /// Generate ICMP Time Exceeded packet
    pub fn generate_time_exceeded(code: IcmpCode, original_datagram: Vec<u8>) -> Self {
        let time_exceeded = IcmpTimeExceeded::new(code, original_datagram);
        Self::new(IcmpType::TimeExceeded, code as u8, time_exceeded.to_bytes())
    }

    /// Generate ICMP Parameter Problem packet
    pub fn generate_parameter_problem(pointer: u8, original_datagram: Vec<u8>) -> Self {
        let param_problem = IcmpParameterProblem::new(pointer, original_datagram);
        Self::new(IcmpType::ParameterProblem, 0, param_problem.to_bytes())
    }

    /// Create ICMP packet with data
    pub fn generate_icmp_packet(data: &[u8], source: IpAddress, dest: IpAddress) -> Result<Vec<u8>> {
        let mut packet = Self::new(IcmpType::EchoRequest, 0, data.to_vec());
        packet.update_checksum();
        Ok(packet.to_bytes())
    }
}

/// ICMP session for tracking ping requests
#[derive(Debug, Clone)]
pub struct IcmpSession {
    /// Session identifier
    pub id: u16,
    /// Remote address
    pub remote_addr: IpAddress,
    /// Sequence number counter
    pub sequence_counter: u16,
    /// Pending requests
    pub pending_requests: HashMap<u16, IcmpEcho>,
    /// Session statistics
    pub stats: IcmpSessionStats,
    /// Session start time
    pub start_time: std::time::Instant,
}

#[derive(Debug, Clone)]
pub struct IcmpSessionStats {
    /// Packets sent
    pub packets_sent: u64,
    /// Packets received
    pub packets_received: u64,
    /// Packets lost
    pub packets_lost: u64,
    /// Minimum RTT
    pub min_rtt: Option<f64>,
    /// Maximum RTT
    pub max_rtt: Option<f64>,
    /// Average RTT
    pub avg_rtt: Option<f64>,
    /// Total RTT for calculation
    pub total_rtt: f64,
    /// RTT samples count
    pub rtt_samples: u32,
}

impl IcmpSessionStats {
    /// Create new session statistics
    pub fn new() -> Self {
        Self {
            packets_sent: 0,
            packets_received: 0,
            packets_lost: 0,
            min_rtt: None,
            max_rtt: None,
            avg_rtt: None,
            total_rtt: 0.0,
            rtt_samples: 0,
        }
    }

    /// Record packet sent
    pub fn record_sent(&mut self) {
        self.packets_sent += 1;
    }

    /// Record packet received
    pub fn record_received(&mut self) {
        self.packets_received += 1;
    }

    /// Record packet loss
    pub fn record_loss(&mut self) {
        self.packets_lost += 1;
    }

    /// Record RTT measurement
    pub fn record_rtt(&mut self, rtt: f64) {
        self.total_rtt += rtt;
        self.rtt_samples += 1;
        
        if let Some(min_rtt) = self.min_rtt {
            self.min_rtt = Some(min_rtt.min(rtt));
        } else {
            self.min_rtt = Some(rtt);
        }
        
        if let Some(max_rtt) = self.max_rtt {
            self.max_rtt = Some(max_rtt.max(rtt));
        } else {
            self.max_rtt = Some(rtt);
        }
        
        self.avg_rtt = Some(self.total_rtt / self.rtt_samples as f64);
    }

    /// Calculate loss percentage
    pub fn loss_percentage(&self) -> f64 {
        if self.packets_sent == 0 {
            0.0
        } else {
            (self.packets_lost as f64 / self.packets_sent as f64) * 100.0
        }
    }
}

impl IcmpSession {
    /// Create a new ICMP session
    pub fn new(remote_addr: IpAddress) -> Self {
        Self {
            id: rand::random(),
            remote_addr,
            sequence_counter: 0,
            pending_requests: HashMap::new(),
            stats: IcmpSessionStats::new(),
            start_time: std::time::Instant::now(),
        }
    }

    /// Send an echo request
    pub fn send_echo_request(&mut self, data: Vec<u8>) -> Result<IcmpPacket> {
        let sequence = self.sequence_counter;
        self.sequence_counter = self.sequence_counter.wrapping_add(1);
        
        let echo = IcmpEcho::new(self.id, sequence, data);
        self.pending_requests.insert(sequence, echo.clone());
        self.stats.record_sent();
        
        let mut packet = IcmpPacket::generate_echo_request(self.id, sequence, echo.data);
        packet.update_checksum();
        
        Ok(packet)
    }

    /// Process echo reply
    pub fn process_echo_reply(&mut self, packet: &IcmpPacket) -> Result<IcmpEcho> {
        if packet.icmp_type != IcmpType::EchoReply {
            return Err(NetworkError::InvalidAddress);
        }
        
        let echo = IcmpEcho::parse(&packet.data)?;
        
        // Remove from pending requests
        if self.pending_requests.remove(&echo.sequence_number).is_some() {
            self.stats.record_received();
            Ok(echo)
        } else {
            Err(NetworkError::InvalidAddress)
        }
    }

    /// Get session statistics
    pub fn get_stats(&self) -> &IcmpSessionStats {
        &self.stats
    }

    /// Check if session has timed out
    pub fn is_timed_out(&self, timeout: std::time::Duration) -> bool {
        let elapsed = std::time::Instant::now().duration_since(self.start_time);
        elapsed > timeout
    }

    /// Get session duration
    pub fn duration(&self) -> std::time::Duration {
        std::time::Instant::now().duration_since(self.start_time)
    }
}

/// ICMP manager for handling ICMP sessions and packets
pub struct IcmpManager {
    /// Active sessions
    sessions: HashMap<IpAddress, IcmpSession>,
    /// Processing statistics
    stats: IcmpManagerStats,
}

#[derive(Debug, Clone)]
pub struct IcmpManagerStats {
    /// Total ICMP packets processed
    pub packets_processed: u64,
    /// ICMP packets sent
    pub packets_sent: u64,
    /// ICMP packets received
    pub packets_received: u64,
    /// Echo requests sent
    pub echo_requests_sent: u64,
    /// Echo replies received
    pub echo_replies_received: u64,
    /// Destination unreachables received
    pub dest_unreachables: u64,
    /// Time exceeded messages received
    pub time_exceeded: u64,
    /// Parameter problems received
    pub parameter_problems: u64,
}

impl IcmpManagerStats {
    /// Create new manager statistics
    pub fn new() -> Self {
        Self {
            packets_processed: 0,
            packets_sent: 0,
            packets_received: 0,
            echo_requests_sent: 0,
            echo_replies_received: 0,
            dest_unreachables: 0,
            time_exceeded: 0,
            parameter_problems: 0,
        }
    }

    /// Record packet processing
    pub fn record_processed(&mut self) {
        self.packets_processed += 1;
    }

    /// Record packet sent
    pub fn record_sent(&mut self) {
        self.packets_sent += 1;
    }

    /// Record packet received
    pub fn record_received(&mut self) {
        self.packets_received += 1;
    }
}

impl IcmpManager {
    /// Create a new ICMP manager
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            stats: IcmpManagerStats::new(),
        }
    }

    /// Start a new ping session
    pub fn start_ping_session(&mut self, remote_addr: IpAddress) -> Result<IcmpSession> {
        if self.sessions.contains_key(&remote_addr) {
            return Err(NetworkError::InvalidAddress);
        }
        
        let session = IcmpSession::new(remote_addr);
        self.sessions.insert(remote_addr, session.clone());
        Ok(session)
    }

    /// End a ping session
    pub fn end_ping_session(&mut self, remote_addr: &IpAddress) -> Option<IcmpSession> {
        self.sessions.remove(remote_addr)
    }

    /// Process incoming ICMP packet
    pub fn process_packet(&mut self, packet: &IcmpPacket, source: IpAddress, _dest: IpAddress) -> Result<()> {
        self.stats.record_processed();
        self.stats.record_received();
        
        match packet.icmp_type {
            IcmpType::EchoReply => {
                self.stats.echo_replies_received += 1;
                self.handle_echo_reply(packet, source)
            }
            IcmpType::DestinationUnreachable => {
                self.stats.dest_unreachables += 1;
                self.handle_destination_unreachable(packet, source)
            }
            IcmpType::TimeExceeded => {
                self.stats.time_exceeded += 1;
                self.handle_time_exceeded(packet, source)
            }
            IcmpType::ParameterProblem => {
                self.stats.parameter_problems += 1;
                self.handle_parameter_problem(packet, source)
            }
            IcmpType::EchoRequest => {
                self.handle_echo_request(packet, source)
            }
            _ => {
                log::debug!("ICMP: Received unhandled ICMP type: {:?}", packet.icmp_type);
                Ok(())
            }
        }
    }

    fn handle_echo_reply(&mut self, packet: &IcmpPacket, source: IpAddress) -> Result<()> {
        if let Some(session) = self.sessions.get_mut(&source) {
            match session.process_echo_reply(packet) {
                Ok(_echo) => {
                    log::debug!("ICMP: Received echo reply from {}", source);
                    Ok(())
                }
                Err(e) => {
                    log::debug!("ICMP: Failed to process echo reply: {:?}", e);
                    Ok(())
                }
            }
        } else {
            log::debug!("ICMP: Received echo reply for unknown session: {}", source);
            Ok(())
        }
    }

    fn handle_destination_unreachable(&mut self, packet: &IcmpPacket, source: IpAddress) -> Result<()> {
        log::debug!("ICMP: Destination unreachable from {}: code {}", source, packet.code);
        Ok(())
    }

    fn handle_time_exceeded(&mut self, packet: &IcmpPacket, source: IpAddress) -> Result<()> {
        log::debug!("ICMP: Time exceeded from {}: code {}", source, packet.code);
        Ok(())
    }

    fn handle_parameter_problem(&mut self, packet: &IcmpPacket, source: IpAddress) -> Result<()> {
        log::debug!("ICMP: Parameter problem from {}: pointer {}", source, packet.data[0]);
        Ok(())
    }

    fn handle_echo_request(&mut self, packet: &IcmpPacket, source: IpAddress) -> Result<()> {
        log::debug!("ICMP: Echo request from {}", source);
        Ok(())
    }

    /// Generate ping packet
    pub fn generate_ping_packet(&mut self, remote_addr: IpAddress, data: Vec<u8>) -> Result<IcmpPacket> {
        let session = self.sessions.get_mut(&remote_addr)
            .ok_or(NetworkError::InvalidAddress)?;
        
        let packet = session.send_echo_request(data)?;
        self.stats.echo_requests_sent += 1;
        Ok(packet)
    }

    /// Get all active sessions
    pub fn get_active_sessions(&self) -> Vec<&IcmpSession> {
        self.sessions.values().collect()
    }

    /// Get session by address
    pub fn get_session(&self, addr: &IpAddress) -> Option<&IcmpSession> {
        self.sessions.get(addr)
    }

    /// Get manager statistics
    pub fn get_stats(&self) -> &IcmpManagerStats {
        &self.stats
    }

    /// Clean up timed out sessions
    pub fn cleanup_sessions(&mut self, timeout: std::time::Duration) {
        let timed_out: Vec<IpAddress> = self.sessions.iter()
            .filter(|(_, session)| session.is_timed_out(timeout))
            .map(|(addr, _)| *addr)
            .collect();
            
        for addr in timed_out {
            self.sessions.remove(&addr);
            log::debug!("ICMP: Cleaned up timed out session for {}", addr);
        }
    }

    /// Get active session count
    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_icmp_echo_creation() {
        let echo = IcmpEcho::new(1234, 5, b"ping test".to_vec());
        assert_eq!(echo.identifier, 1234);
        assert_eq!(echo.sequence_number, 5);
        assert_eq!(echo.data, b"ping test");
        
        let bytes = echo.to_bytes();
        let parsed = IcmpEcho::parse(&bytes).unwrap();
        assert_eq!(parsed.identifier, echo.identifier);
        assert_eq!(parsed.sequence_number, echo.sequence_number);
        assert_eq!(parsed.data, echo.data);
    }

    #[test]
    fn test_icmp_packet_creation() {
        let data = b"ICMP test data".to_vec();
        let packet = IcmpPacket::new(IcmpType::EchoRequest, 0, data);
        
        assert_eq!(packet.icmp_type, IcmpType::EchoRequest);
        assert_eq!(packet.code, 0);
        assert_eq!(packet.data, b"ICMP test data");
        
        packet.update_checksum();
        assert!(packet.verify_checksum());
    }

    #[test]
    fn test_icmp_echo_request_reply() {
        let request_data = b"hello".to_vec();
        let mut request = IcmpPacket::generate_echo_request(1234, 1, request_data.clone());
        request.update_checksum();
        
        let reply = IcmpPacket::generate_echo_reply(&request, IpAddress::v4(192, 168, 1, 1), IpAddress::v4(8, 8, 8, 8)).unwrap();
        
        assert_eq!(reply.icmp_type, IcmpType::EchoReply);
        assert_eq!(reply.code, 0);
    }

    #[test]
    fn test_icmp_session() {
        let remote_addr = IpAddress::v4(8, 8, 8, 8);
        let mut session = IcmpSession::new(remote_addr);
        
        let packet = session.send_echo_request(b"ping".to_vec()).unwrap();
        assert_eq!(session.pending_requests.len(), 1);
        assert_eq!(session.stats.packets_sent, 1);
        
        let echo = IcmpEcho::parse(&packet.data).unwrap();
        let reply_packet = IcmpPacket::new(IcmpType::EchoReply, 0, echo.to_bytes());
        reply_packet.update_checksum();
        
        let received_echo = session.process_echo_reply(&reply_packet).unwrap();
        assert_eq!(session.pending_requests.len(), 0);
        assert_eq!(session.stats.packets_received, 1);
    }

    #[test]
    fn test_icmp_manager() {
        let mut manager = IcmpManager::new();
        let remote_addr = IpAddress::v4(8, 8, 8, 8);
        
        let session = manager.start_ping_session(remote_addr).unwrap();
        assert_eq!(manager.session_count(), 1);
        assert!(manager.get_session(&remote_addr).is_some());
        
        let packet = manager.generate_ping_packet(remote_addr, b"ping".to_vec()).unwrap();
        assert_eq!(manager.get_stats().echo_requests_sent, 1);
        
        let ended = manager.end_ping_session(&remote_addr).unwrap();
        assert_eq!(manager.session_count(), 0);
    }
}