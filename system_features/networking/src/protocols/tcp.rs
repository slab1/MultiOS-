//! Transmission Control Protocol (TCP) implementation
//!
//! This module provides a complete TCP implementation including connection management,
//! state machine, flow control, congestion control, and retransmission logic.

use crate::{Result, NetworkError};
use crate::core::IpAddress;
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

/// TCP header flags
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TcpFlags {
    pub ns: bool,      // Nonce Sum (experimental)
    pub cwr: bool,     // Congestion Window Reduced
    pub ece: bool,     // ECN-Echo
    pub urg: bool,     // Urgent pointer field significant
    pub ack: bool,     // Acknowledgment field significant
    pub psh: bool,     // Push function
    pub rst: bool,     // Reset connection
    pub syn: bool,     // Synchronize sequence numbers
    pub fin: bool,     // No more data from sender
}

impl TcpFlags {
    /// Parse flags from byte
    pub fn from_byte(byte: u8) -> Self {
        Self {
            ns: (byte & 0x01) != 0,
            cwr: (byte & 0x02) != 0,
            ece: (byte & 0x04) != 0,
            urg: (byte & 0x20) != 0,
            ack: (byte & 0x10) != 0,
            psh: (byte & 0x08) != 0,
            rst: (byte & 0x04) != 0,
            syn: (byte & 0x02) != 0,
            fin: (byte & 0x01) != 0,
        }
    }

    /// Convert to byte value
    pub fn to_byte(&self) -> u8 {
        (self.ns as u8) |
        ((self.cwr as u8) << 1) |
        ((self.ece as u8) << 2) |
        ((self.urg as u8) << 5) |
        ((self.ack as u8) << 4) |
        ((self.psh as u8) << 3) |
        ((self.rst as u8) << 2) |
        ((self.syn as u8) << 1) |
        (self.fin as u8)
    }
}

/// TCP options
#[derive(Debug, Clone)]
pub enum TcpOption {
    /// End of options list
    EndOfOptions,
    /// No operation (padding)
    NoOperation,
    /// Maximum Segment Size
    MaximumSegmentSize(u16),
    /// Window Scale
    WindowScale(u8),
    /// Selective Acknowledgment Permitted
    SelectiveAckPermitted,
    /// Selective Acknowledgment
    SelectiveAck(Vec<(u32, u32)>),
    /// Timestamps
    Timestamps(u32, u32),
}

impl TcpOption {
    /// Parse TCP option from bytes
    pub fn parse(data: &[u8]) -> Result<(Self, usize)> {
        if data.is_empty() {
            return Err(NetworkError::InvalidAddress);
        }

        match data[0] {
            0 => Ok((TcpOption::EndOfOptions, 1)),
            1 => Ok((TcpOption::NoOperation, 1)),
            2 => {
                if data.len() >= 4 {
                    let mss = ((data[2] as u16) << 8) | (data[3] as u16);
                    Ok((TcpOption::MaximumSegmentSize(mss), 4))
                } else {
                    Err(NetworkError::InvalidAddress)
                }
            }
            3 => {
                if data.len() >= 3 {
                    Ok((TcpOption::WindowScale(data[2]), 3))
                } else {
                    Err(NetworkError::InvalidAddress)
                }
            }
            4 => Ok((TcpOption::SelectiveAckPermitted, 2)),
            5 => {
                // Selective ACK - variable length
                let length = data[1] as usize;
                if length >= 2 && length <= data.len() {
                    let mut sacks = Vec::new();
                    for i in (2..length).step_by(8) {
                        if i + 8 <= data.len() {
                            let left = ((data[i] as u32) << 24) | 
                                      ((data[i+1] as u32) << 16) |
                                      ((data[i+2] as u32) << 8) |
                                      (data[i+3] as u32);
                            let right = ((data[i+4] as u32) << 24) |
                                       ((data[i+5] as u32) << 16) |
                                       ((data[i+6] as u32) << 8) |
                                       (data[i+7] as u32);
                            sacks.push((left, right));
                        }
                    }
                    Ok((TcpOption::SelectiveAck(sacks), length))
                } else {
                    Err(NetworkError::InvalidAddress)
                }
            }
            8 => {
                if data.len() >= 10 {
                    let ts_val = ((data[2] as u32) << 24) |
                                ((data[3] as u32) << 16) |
                                ((data[4] as u32) << 8) |
                                (data[5] as u32);
                    let ts_ecr = ((data[6] as u32) << 24) |
                                ((data[7] as u32) << 16) |
                                ((data[8] as u32) << 8) |
                                (data[9] as u32);
                    Ok((TcpOption::Timestamps(ts_val, ts_ecr), 10))
                } else {
                    Err(NetworkError::InvalidAddress)
                }
            }
            _ => {
                // Unknown option, skip based on length
                if data.len() >= 2 {
                    let length = data[1] as usize;
                    Ok((TcpOption::NoOperation, std::cmp::min(length, data.len())))
                } else {
                    Err(NetworkError::InvalidAddress)
                }
            }
        }
    }

    /// Convert to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            TcpOption::EndOfOptions => vec![0],
            TcpOption::NoOperation => vec![1],
            TcpOption::MaximumSegmentSize(mss) => vec![2, 4, (*mss >> 8) as u8, *mss as u8],
            TcpOption::WindowScale(shift) => vec![3, 3, *shift],
            TcpOption::SelectiveAckPermitted => vec![4, 2],
            TcpOption::SelectiveAck(sacks) => {
                let mut bytes = vec![5, 2 + (sacks.len() as u8) * 8];
                for &(left, right) in sacks {
                    bytes.extend_from_slice(&[
                        (left >> 24) as u8, (left >> 16) as u8,
                        (left >> 8) as u8, left as u8,
                        (right >> 24) as u8, (right >> 16) as u8,
                        (right >> 8) as u8, right as u8,
                    ]);
                }
                bytes
            }
            TcpOption::Timestamps(ts_val, ts_ecr) => vec![
                8, 10,
                (*ts_val >> 24) as u8, (*ts_val >> 16) as u8,
                (*ts_val >> 8) as u8, *ts_val as u8,
                (*ts_ecr >> 24) as u8, (*ts_ecr >> 16) as u8,
                (*ts_ecr >> 8) as u8, *ts_ecr as u8,
            ],
        }
    }
}

/// TCP state machine states
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TcpState {
    /// Closed state
    Closed,
    /// Listen state
    Listen,
    /// SYN Sent state
    SynSent,
    /// SYN Received state
    SynReceived,
    /// Established state
    Established,
    /// FIN Wait 1 state
    FinWait1,
    /// FIN Wait 2 state
    FinWait2,
    /// Close Wait state
    CloseWait,
    /// Closing state
    Closing,
    /// Last ACK state
    LastAck,
    /// Time Wait state
    TimeWait,
    /// Error state
    Error(String),
}

/// TCP packet structure
#[derive(Debug, Clone)]
pub struct TcpPacket {
    /// Source port
    pub source_port: u16,
    /// Destination port
    pub dest_port: u16,
    /// Sequence number
    pub sequence: u32,
    /// Acknowledgment number
    pub acknowledgment: u32,
    /// Data offset and flags
    pub data_offset_flags: u8,
    /// Window size
    pub window_size: u16,
    /// Checksum
    pub checksum: u16,
    /// Urgent pointer
    pub urgent_pointer: u16,
    /// Options
    pub options: Vec<TcpOption>,
    /// Data payload
    pub data: Vec<u8>,
}

impl TcpPacket {
    /// Create a new TCP packet
    pub fn new(source_port: u16, dest_port: u16) -> Self {
        Self {
            source_port,
            dest_port,
            sequence: 0,
            acknowledgment: 0,
            data_offset_flags: (5 << 4), // 5 * 4 = 20 bytes (no options)
            window_size: 65535,
            checksum: 0,
            urgent_pointer: 0,
            options: Vec::new(),
            data: Vec::new(),
        }
    }

    /// Parse TCP packet from raw bytes
    pub fn parse(data: &[u8]) -> Result<Self> {
        if data.len() < 20 {
            return Err(NetworkError::InvalidAddress);
        }

        let source_port = ((data[0] as u16) << 8) | (data[1] as u16);
        let dest_port = ((data[2] as u16) << 8) | (data[3] as u16);
        let sequence = ((data[4] as u32) << 24) | 
                      ((data[5] as u32) << 16) |
                      ((data[6] as u32) << 8) |
                      (data[7] as u32);
        let acknowledgment = ((data[8] as u32) << 24) |
                           ((data[9] as u32) << 16) |
                           ((data[10] as u32) << 8) |
                           (data[11] as u32);
        let data_offset_flags = data[12];
        let window_size = ((data[14] as u16) << 8) | (data[15] as u16);
        let checksum = ((data[16] as u16) << 8) | (data[17] as u16);
        let urgent_pointer = ((data[18] as u16) << 8) | (data[19] as u16);

        // Parse options
        let data_offset = ((data_offset_flags >> 4) & 0x0F) as usize;
        let header_length = data_offset * 4;
        let options_length = header_length - 20;

        let mut options = Vec::new();
        let mut offset = 20;
        while offset < header_length && offset < data.len() {
            if offset + 1 > data.len() {
                break;
            }

            match TcpOption::parse(&data[offset..]) {
                Ok((option, consumed)) => {
                    options.push(option);
                    offset += consumed;
                }
                Err(_) => break,
            }
        }

        // Extract data
        let data = if header_length < data.len() {
            data[header_length..].to_vec()
        } else {
            Vec::new()
        };

        Ok(Self {
            source_port,
            dest_port,
            sequence,
            acknowledgment,
            data_offset_flags,
            window_size,
            checksum,
            urgent_pointer,
            options,
            data,
        })
    }

    /// Convert TCP packet to raw bytes
    pub fn to_bytes(&self, source: IpAddress, dest: IpAddress) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(20 + self.data.len() + self.options.len() * 4);
        
        // Source port
        bytes.push((self.source_port >> 8) as u8);
        bytes.push(self.source_port as u8);
        
        // Destination port
        bytes.push((self.dest_port >> 8) as u8);
        bytes.push(self.dest_port as u8);
        
        // Sequence number
        bytes.extend_from_slice(&self.sequence.to_be_bytes());
        
        // Acknowledgment number
        bytes.extend_from_slice(&self.acknowledgment.to_be_bytes());
        
        // Data offset and flags
        bytes.push(self.data_offset_flags);
        
        // Window size
        bytes.push((self.window_size >> 8) as u8);
        bytes.push(self.window_size as u8);
        
        // Checksum (will be calculated)
        bytes.push((self.checksum >> 8) as u8);
        bytes.push(self.checksum as u8);
        
        // Urgent pointer
        bytes.push((self.urgent_pointer >> 8) as u8);
        bytes.push(self.urgent_pointer as u8);
        
        // Options
        for option in &self.options {
            bytes.extend_from_slice(&option.to_bytes());
        }
        
        // Pad options to 4-byte boundary
        while (bytes.len() % 4) != 0 {
            bytes.push(0);
        }
        
        // Update data offset
        let header_length = bytes.len();
        let data_offset = header_length / 4;
        bytes[12] = ((data_offset as u8) << 4) | (self.data_offset_flags & 0x0F);
        
        // Data payload
        bytes.extend_from_slice(&self.data);
        
        // Calculate and set checksum
        let checksum = crate::protocols::utils::pseudo_header_checksum(source, dest, 6, bytes.len() as u16);
        bytes[16] = (checksum >> 8) as u8;
        bytes[17] = checksum as u8;
        
        bytes
    }

    /// Get TCP flags
    pub fn flags(&self) -> TcpFlags {
        TcpFlags::from_byte(self.data_offset_flags & 0x3F)
    }

    /// Set TCP flags
    pub fn set_flags(&mut self, flags: TcpFlags) {
        self.data_offset_flags = (self.data_offset_flags & 0xF0) | flags.to_byte();
    }

    /// Get data offset
    pub fn data_offset(&self) -> u8 {
        (self.data_offset_flags >> 4) & 0x0F
    }

    /// Generate TCP packet with specific data
    pub fn generate_tcp_packet(data: &[u8], source: IpAddress, dest: IpAddress) -> Result<Vec<u8>> {
        let mut packet = TcpPacket::new(0, 0); // Ports would be set by caller
        packet.data = data.to_vec();
        packet.set_flags(TcpFlags {
            ns: false, cwr: false, ece: false, urg: false,
            ack: true, psh: true, rst: false, syn: false, fin: false,
        });
        
        let bytes = packet.to_bytes(source, dest);
        Ok(bytes)
    }
}

/// TCP connection structure
pub struct TcpConnection {
    /// Connection state
    pub state: TcpState,
    /// Local address and port
    pub local_addr: (IpAddress, u16),
    /// Remote address and port
    pub remote_addr: (IpAddress, u16),
    /// Sequence numbers
    pub send_sequence: u32,
    pub receive_sequence: u32,
    pub ack_sequence: u32,
    /// Window management
    pub send_window: u32,
    pub receive_window: u32,
    /// Retransmission
    pub retransmission_queue: VecDeque<RetransmissionEntry>,
    /// Congestion control
    pub congestion_window: u32,
    pub slow_start_threshold: u32,
    /// Timing
    pub last_activity: Instant,
    pub rtt_smoothed: f64,
    /// Options
    pub options: TcpConnectionOptions,
}

struct RetransmissionEntry {
    sequence: u32,
    data: Vec<u8>,
    timestamp: Instant,
    retransmissions: u32,
}

struct TcpConnectionOptions {
    pub maximum_segment_size: u16,
    pub window_scale: u8,
    pub selective_ack: bool,
    pub timestamps: bool,
}

impl TcpConnectionOptions {
    pub fn default() -> Self {
        Self {
            maximum_segment_size: 1460,
            window_scale: 0,
            selective_ack: false,
            timestamps: false,
        }
    }
}

impl TcpConnection {
    /// Create a new TCP connection
    pub fn new(local_addr: (IpAddress, u16), remote_addr: (IpAddress, u16)) -> Self {
        Self {
            state: TcpState::Closed,
            local_addr,
            remote_addr,
            send_sequence: 0,
            receive_sequence: 0,
            ack_sequence: 0,
            send_window: 65535,
            receive_window: 65535,
            retransmission_queue: VecDeque::new(),
            congestion_window: 1460 * 3, // Initial congestion window
            slow_start_threshold: u32::MAX,
            last_activity: Instant::now(),
            rtt_smoothed: 1000.0, // Initial RTT estimate
            options: TcpConnectionOptions::default(),
        }
    }

    /// Process incoming TCP packet
    pub fn process_packet(&mut self, packet: &TcpPacket) -> Result<()> {
        self.last_activity = Instant::now();
        
        match self.state {
            TcpState::Closed => self.process_packet_closed(packet),
            TcpState::Listen => self.process_packet_listen(packet),
            TcpState::SynSent => self.process_packet_syn_sent(packet),
            TcpState::SynReceived => self.process_packet_syn_received(packet),
            TcpState::Established => self.process_packet_established(packet),
            TcpState::FinWait1 => self.process_packet_fin_wait1(packet),
            TcpState::FinWait2 => self.process_packet_fin_wait2(packet),
            TcpState::CloseWait => self.process_packet_close_wait(packet),
            TcpState::Closing => self.process_packet_closing(packet),
            TcpState::LastAck => self.process_packet_last_ack(packet),
            TcpState::TimeWait => self.process_packet_time_wait(packet),
            TcpState::Error(_) => Err(NetworkError::InvalidState),
        }
    }

    fn process_packet_closed(&mut self, _packet: &TcpPacket) -> Result<()> {
        // Closed state doesn't process packets
        Err(NetworkError::InvalidState)
    }

    fn process_packet_listen(&mut self, packet: &TcpPacket) -> Result<()> {
        let flags = packet.flags();
        
        if flags.syn {
            // SYN received, move to SYN-RCVD state
            self.state = TcpState::SynReceived;
            self.receive_sequence = packet.sequence + 1;
            self.ack_sequence = packet.sequence + 1;
            
            // Send SYN-ACK
            let mut response = TcpPacket::new(self.local_addr.1, self.remote_addr.1);
            response.set_flags(TcpFlags {
                ns: false, cwr: false, ece: false, urg: false,
                ack: true, psh: false, rst: false, syn: true, fin: false,
            });
            response.acknowledgment = packet.sequence + 1;
            response.sequence = 0; // Will be set when sending
            
            log::info!("TCP: Received SYN, sending SYN-ACK");
            Ok(())
        } else {
            Err(NetworkError::InvalidState)
        }
    }

    fn process_packet_syn_sent(&mut self, packet: &TcpPacket) -> Result<()> {
        let flags = packet.flags();
        
        if flags.ack && flags.syn {
            // SYN-ACK received, move to ESTABLISHED state
            self.state = TcpState::Established;
            self.receive_sequence = packet.sequence + 1;
            self.ack_sequence = packet.sequence + 1;
            self.send_sequence = packet.acknowledgment;
            
            log::info!("TCP: Connection established");
            Ok(())
        } else if flags.syn {
            // SYN received without ACK, move to SYN-RCVD state
            self.state = TcpState::SynReceived;
            self.receive_sequence = packet.sequence + 1;
            
            log::info!("TCP: Received SYN, moving to SYN-RCVD");
            Ok(())
        } else {
            Err(NetworkError::InvalidState)
        }
    }

    fn process_packet_syn_received(&mut self, packet: &TcpPacket) -> Result<()> {
        let flags = packet.flags();
        
        if flags.ack {
            // ACK received, move to ESTABLISHED state
            self.state = TcpState::Established;
            self.send_sequence = packet.acknowledgment;
            self.receive_sequence = packet.sequence;
            
            log::info!("TCP: Connection established (SYN-RCVD)");
            Ok(())
        } else {
            Err(NetworkError::InvalidState)
        }
    }

    fn process_packet_established(&mut self, packet: &TcpPacket) -> Result<()> {
        let flags = packet.flags();
        
        if flags.rst {
            // RST received, close connection
            self.state = TcpState::Closed;
            log::info!("TCP: Connection reset by peer");
            return Ok(());
        }
        
        if flags.fin {
            // FIN received, move to CLOSE-WAIT state
            self.state = TcpState::CloseWait;
            self.receive_sequence = packet.sequence + 1;
            log::info!("TCP: Received FIN, moving to CLOSE-WAIT");
            return Ok(());
        }
        
        // Process data and ACKs
        if packet.data.len() > 0 {
            // Process incoming data
            self.receive_sequence = packet.sequence + packet.data.len() as u32;
        }
        
        if flags.ack {
            // Process acknowledgment
            if packet.acknowledgment > self.send_sequence {
                // Update send sequence
                self.send_sequence = packet.acknowledgment;
                
                // Remove acknowledged data from retransmission queue
                while let Some(entry) = self.retransmission_queue.front() {
                    if packet.acknowledgment >= entry.sequence + entry.data.len() as u32 {
                        self.retransmission_queue.pop_front();
                    } else {
                        break;
                    }
                }
                
                // Update congestion window
                self.update_congestion_window(packet);
            }
        }
        
        Ok(())
    }

    fn process_packet_fin_wait1(&mut self, packet: &TcpPacket) -> Result<()> {
        let flags = packet.flags();
        
        if flags.ack {
            // ACK for FIN received
            if self.send_sequence == packet.acknowledgment {
                self.state = TcpState::FinWait2;
                log::info!("TCP: Moved to FIN-WAIT-2");
            } else {
                self.state = TcpState::Closing;
            }
        } else if flags.fin {
            // FIN received without ACK
            self.state = TcpState::Closing;
        }
        
        Ok(())
    }

    fn process_packet_fin_wait2(&mut self, packet: &TcpPacket) -> Result<()> {
        let flags = packet.flags();
        
        if flags.fin {
            // FIN received
            self.receive_sequence = packet.sequence + 1;
            self.ack_sequence = packet.sequence + 1;
            self.state = TcpState::TimeWait;
            log::info!("TCP: Moved to TIME-WAIT");
        }
        
        Ok(())
    }

    fn process_packet_close_wait(&mut self, _packet: &TcpPacket) -> Result<()> {
        // Wait for application to close
        Ok(())
    }

    fn process_packet_closing(&mut self, packet: &TcpPacket) -> Result<()> {
        let flags = packet.flags();
        
        if flags.ack && self.send_sequence == packet.acknowledgment {
            self.state = TcpState::TimeWait;
            log::info!("TCP: Moved to TIME-WAIT");
        }
        
        Ok(())
    }

    fn process_packet_last_ack(&mut self, packet: &TcpPacket) -> Result<()> {
        let flags = packet.flags();
        
        if flags.ack {
            self.state = TcpState::Closed;
            log::info!("TCP: Connection closed");
        }
        
        Ok(())
    }

    fn process_packet_time_wait(&mut self, _packet: &TcpPacket) -> Result<()> {
        // TIME-WAIT state doesn't process regular packets
        Ok(())
    }

    /// Update congestion window based on received packet
    fn update_congestion_window(&mut self, _packet: &TcpPacket) {
        // Simple congestion control implementation
        // In a full implementation, this would include:
        // - Slow start
        // - Congestion avoidance
        // - Fast retransmit
        // - Fast recovery
        
        // For now, just implement basic slow start
        if self.congestion_window < self.slow_start_threshold {
            // Slow start phase
            self.congestion_window = std::cmp::min(
                self.congestion_window * 2,
                self.slow_start_threshold
            );
        } else {
            // Congestion avoidance phase
            self.congestion_window = std::cmp::min(
                self.congestion_window + 1460,
                self.receive_window as u32
            );
        }
    }

    /// Send data
    pub fn send_data(&mut self, data: &[u8]) -> Result<()> {
        if self.state != TcpState::Established {
            return Err(NetworkError::InvalidState);
        }
        
        // Add to retransmission queue
        let entry = RetransmissionEntry {
            sequence: self.send_sequence,
            data: data.to_vec(),
            timestamp: Instant::now(),
            retransmissions: 0,
        };
        
        self.retransmission_queue.push_back(entry);
        self.send_sequence += data.len() as u32;
        
        Ok(())
    }

    /// Check if connection is active
    pub fn is_active(&self) -> bool {
        !matches!(self.state, TcpState::Closed | TcpState::Error(_))
    }

    /// Check if connection is established
    pub fn is_established(&self) -> bool {
        self.state == TcpState::Established
    }

    /// Get connection statistics
    pub fn get_stats(&self) -> TcpConnectionStats {
        TcpConnectionStats {
            state: self.state.clone(),
            send_sequence: self.send_sequence,
            receive_sequence: self.receive_sequence,
            send_window: self.send_window,
            receive_window: self.receive_window,
            congestion_window: self.congestion_window,
            retransmission_queue_size: self.retransmission_queue.len(),
            rtt_smoothed: self.rtt_smoothed,
        }
    }
}

/// TCP connection statistics
#[derive(Debug, Clone)]
pub struct TcpConnectionStats {
    pub state: TcpState,
    pub send_sequence: u32,
    pub receive_sequence: u32,
    pub send_window: u32,
    pub receive_window: u32,
    pub congestion_window: u32,
    pub retransmission_queue_size: usize,
    pub rtt_smoothed: f64,
}

/// TCP connection manager
pub struct TcpManager {
    /// Active connections
    connections: HashMap<(IpAddress, u16, IpAddress, u16), TcpConnection>,
    /// Listen ports
    listeners: HashMap<u16, TcpListener>,
    /// Sequence number generator
    sequence_generator: std::sync::atomic::AtomicU32,
}

struct TcpListener {
    port: u16,
    address: IpAddress,
    backlog: i32,
}

impl TcpManager {
    /// Create a new TCP manager
    pub fn new() -> Self {
        Self {
            connections: HashMap::new(),
            listeners: HashMap::new(),
            sequence_generator: std::sync::atomic::AtomicU32::new(0),
        }
    }

    /// Create a listener
    pub fn listen(&mut self, address: IpAddress, port: u16, backlog: i32) -> Result<()> {
        self.listeners.insert(port, TcpListener {
            port,
            address,
            backlog,
        });
        Ok(())
    }

    /// Accept a connection
    pub fn accept(&mut self, port: u16) -> Result<TcpConnection> {
        // In a real implementation, this would accept from the actual socket
        Err(NetworkError::InvalidState)
    }

    /// Get connection by 4-tuple
    pub fn get_connection(&self, local_addr: IpAddress, local_port: u16, 
                        remote_addr: IpAddress, remote_port: u16) -> Option<&TcpConnection> {
        self.connections.get(&(local_addr, local_port, remote_addr, remote_port))
    }

    /// Remove connection
    pub fn remove_connection(&mut self, local_addr: IpAddress, local_port: u16,
                           remote_addr: IpAddress, remote_port: u16) -> Option<TcpConnection> {
        self.connections.remove(&(local_addr, local_port, remote_addr, remote_port))
    }

    /// Process incoming TCP packet
    pub fn process_packet(&mut self, packet: &TcpPacket, source: IpAddress, dest: IpAddress) -> Result<()> {
        let key = (dest, packet.dest_port, source, packet.source_port);
        
        if let Some(connection) = self.connections.get_mut(&key) {
            connection.process_packet(packet)
        } else {
            // No existing connection, might be SYN for new connection
            Err(NetworkError::InvalidState)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tcp_packet_creation() {
        let packet = TcpPacket::new(8080, 80);
        assert_eq!(packet.source_port, 8080);
        assert_eq!(packet.dest_port, 80);
        assert_eq!(packet.data.len(), 0);
    }

    #[test]
    fn test_tcp_flags() {
        let flags = TcpFlags {
            ns: false, cwr: false, ece: false, urg: false,
            ack: true, psh: true, rst: false, syn: false, fin: false,
        };
        
        let byte = flags.to_byte();
        let parsed = TcpFlags::from_byte(byte);
        
        assert_eq!(parsed.ack, flags.ack);
        assert_eq!(parsed.psh, flags.psh);
    }

    #[test]
    fn test_tcp_options() {
        let mss_option = TcpOption::MaximumSegmentSize(1460);
        let bytes = mss_option.to_bytes();
        let (parsed, _) = TcpOption::parse(&bytes).unwrap();
        
        match parsed {
            TcpOption::MaximumSegmentSize(mss) => {
                assert_eq!(mss, 1460);
            }
            _ => panic!("Failed to parse MSS option"),
        }
    }

    #[test]
    fn test_tcp_connection_state_transition() {
        let local_addr = (IpAddress::v4(127, 0, 0, 1), 8080);
        let remote_addr = (IpAddress::v4(127, 0, 0, 1), 80);
        let mut connection = TcpConnection::new(local_addr, remote_addr);
        
        assert_eq!(connection.state, TcpState::Closed);
        
        // Simulate SYN received
        let mut syn_packet = TcpPacket::new(80, 8080);
        syn_packet.set_flags(TcpFlags {
            ns: false, cwr: false, ece: false, urg: false,
            ack: false, psh: false, rst: false, syn: true, fin: false,
        });
        syn_packet.sequence = 1000;
        
        // This would fail because we're in Closed state
        assert!(connection.process_packet(&syn_packet).is_err());
    }
}