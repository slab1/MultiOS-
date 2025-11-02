//! Network security implementation
//!
//! This module provides comprehensive network security features including:
//! - Firewall with configurable rules
//! - Packet filtering and inspection
//! - Network intrusion detection
//! - Traffic analysis and monitoring
//! - Security policies and access control

use crate::{Result, NetworkError};
use crate::core::{IpAddress, SocketAddr};
use crate::protocols::{ip::IpPacket, tcp::TcpPacket, udp::UdpPacket, icmp::IcmpPacket};
use std::collections::{HashMap, BTreeMap};
use std::time::{Duration, Instant};
use std::net::SocketAddr as NetSocketAddr;

/// Firewall rule action
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FirewallAction {
    /// Accept the packet
    Accept,
    /// Drop the packet silently
    Drop,
    /// Reject the packet (send error response)
    Reject,
    /// Log the packet
    Log,
    /// Count the packet
    Count,
    /// Continue to next rule
    Continue,
}

/// Firewall rule direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleDirection {
    /// Incoming packets
    In,
    /// Outgoing packets
    Out,
    /// Both directions
    Both,
}

/// Network protocol
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NetworkProtocol {
    Tcp,
    Udp,
    Icmp,
    Any,
}

impl NetworkProtocol {
    /// Convert to u8 value
    pub fn to_u8(self) -> u8 {
        match self {
            NetworkProtocol::Tcp => 6,
            NetworkProtocol::Udp => 17,
            NetworkProtocol::Icmp => 1,
            NetworkProtocol::Any => 0,
        }
    }

    /// Convert from u8 value
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            1 => Some(NetworkProtocol::Icmp),
            6 => Some(NetworkProtocol::Tcp),
            17 => Some(NetworkProtocol::Udp),
            _ => Some(NetworkProtocol::Any),
        }
    }
}

/// IP address range
#[derive(Debug, Clone)]
pub struct IpAddressRange {
    /// Start address
    pub start: IpAddress,
    /// End address
    pub end: IpAddress,
}

impl IpAddressRange {
    /// Create a range from start to end
    pub fn new(start: IpAddress, end: IpAddress) -> Self {
        Self { start, end }
    }

    /// Create a single address range
    pub fn single(address: IpAddress) -> Self {
        Self { start: address, end: address }
    }

    /// Create a CIDR network range
    pub fn from_cidr(network: IpAddress, prefix_length: u32) -> Result<Self> {
        if prefix_length > 32 {
            return Err(NetworkError::InvalidAddress);
        }

        let mask = if prefix_length == 0 {
            0u32
        } else {
            (!0u32) << (32 - prefix_length)
        };

        let start = IpAddress::from_u32(network.to_u32() & mask);
        let end = IpAddress::from_u32(network.to_u32() | !mask);

        Ok(Self { start, end })
    }

    /// Check if address is in range
    pub fn contains(&self, address: IpAddress) -> bool {
        let addr = address.to_u32();
        let start = self.start.to_u32();
        let end = self.end.to_u32();
        
        if start <= end {
            addr >= start && addr <= end
        } else {
            // Handle wraparound case
            addr >= start || addr <= end
        }
    }
}

/// Port range
#[derive(Debug, Clone)]
pub struct PortRange {
    /// Start port
    pub start: u16,
    /// End port
    pub end: u16,
}

impl PortRange {
    /// Create a range from start to end
    pub fn new(start: u16, end: u16) -> Self {
        Self { start, end }
    }

    /// Create a single port range
    pub fn single(port: u16) -> Self {
        Self { start: port, end: port }
    }

    /// Create well-known ports range
    pub fn well_known() -> Self {
        Self::new(0, 1023)
    }

    /// Create registered ports range
    pub fn registered() -> Self {
        Self::new(1024, 49151)
    }

    /// Create ephemeral ports range
    pub fn ephemeral() -> Self {
        Self::new(49152, 65535)
    }

    /// Check if port is in range
    pub fn contains(&self, port: u16) -> bool {
        port >= self.start && port <= self.end
    }
}

/// Time window for rule application
#[derive(Debug, Clone)]
pub struct TimeWindow {
    /// Start time (0-23 hours)
    pub start_hour: u8,
    /// End time (0-23 hours)
    pub end_hour: u8,
    /// Days of week (0=Sunday, 6=Saturday)
    pub days_of_week: u8, // Bitmask: bit 0 = Sunday, bit 6 = Saturday
}

impl TimeWindow {
    /// Create a time window
    pub fn new(start_hour: u8, end_hour: u8, days_of_week: u8) -> Self {
        Self {
            start_hour,
            end_hour,
            days_of_week,
        }
    }

    /// Create all day, every day window
    pub fn always() -> Self {
        Self {
            start_hour: 0,
            end_hour: 23,
            days_of_week: 0x7F, // All 7 days
        }
    }

    /// Check if current time is within window
    pub fn is_within_window(&self) -> bool {
        let now = chrono::Utc::now();
        let hour = now.hour() as u8;
        let day = now.weekday() as u8; // Sunday = 0, Saturday = 6
        
        // Check day of week
        let day_bit = 1 << day;
        if (self.days_of_week & day_bit) == 0 {
            return false;
        }
        
        // Check hour range (handle wraparound)
        if self.start_hour <= self.end_hour {
            hour >= self.start_hour && hour <= self.end_hour
        } else {
            hour >= self.start_hour || hour <= self.end_hour
        }
    }
}

/// Filter rule structure
#[derive(Debug, Clone)]
pub struct FilterRule {
    /// Rule ID
    pub id: u32,
    /// Rule name
    pub name: String,
    /// Rule description
    pub description: Option<String>,
    /// Action to take
    pub action: FirewallAction,
    /// Direction of traffic
    pub direction: RuleDirection,
    /// Source IP range
    pub source_ip: Option<IpAddressRange>,
    /// Destination IP range
    pub dest_ip: Option<IpAddressRange>,
    /// Source port range
    pub source_port: Option<PortRange>,
    /// Destination port range
    pub dest_port: Option<PortRange>,
    /// Network protocol
    pub protocol: NetworkProtocol,
    /// TCP flags (for TCP packets)
    pub tcp_flags: Option<u8>,
    /// Time window for rule application
    pub time_window: Option<TimeWindow>,
    /// Rule priority (lower number = higher priority)
    pub priority: u32,
    /// Rule statistics
    pub stats: RuleStatistics,
    /// Rule status (enabled/disabled)
    pub enabled: bool,
    /// Rule creation time
    pub created_at: Instant,
    /// Rule modification time
    pub modified_at: Instant,
}

#[derive(Debug, Clone, Default)]
pub struct RuleStatistics {
    /// Number of packets matched
    pub packets_matched: u64,
    /// Number of bytes matched
    pub bytes_matched: u64,
    /// Last matched time
    pub last_matched: Option<Instant>,
    /// First matched time
    pub first_matched: Option<Instant>,
}

impl FilterRule {
    /// Create a new filter rule
    pub fn new(id: u32, name: String, action: FirewallAction, protocol: NetworkProtocol) -> Self {
        Self {
            id,
            name,
            description: None,
            action,
            direction: RuleDirection::Both,
            source_ip: None,
            dest_ip: None,
            source_port: None,
            dest_port: None,
            protocol,
            tcp_flags: None,
            time_window: None,
            priority: 100,
            stats: RuleStatistics::default(),
            enabled: true,
            created_at: Instant::now(),
            modified_at: Instant::now(),
        }
    }

    /// Create a basic allow rule
    pub fn allow_all() -> Self {
        Self::new(0, "Allow All".to_string(), FirewallAction::Accept, NetworkProtocol::Any)
    }

    /// Create a basic deny rule
    pub fn deny_all() -> Self {
        Self::new(0, "Deny All".to_string(), FirewallAction::Drop, NetworkProtocol::Any)
    }

    /// Check if rule matches a packet
    pub fn matches_packet(&self, packet: &FilteredPacket) -> bool {
        // Check if rule is enabled
        if !self.enabled {
            return false;
        }

        // Check time window if specified
        if let Some(window) = &self.time_window {
            if !window.is_within_window() {
                return false;
            }
        }

        // Check direction
        match self.direction {
            RuleDirection::Both => {},
            RuleDirection::In => {
                if !packet.is_incoming {
                    return false;
                }
            },
            RuleDirection::Out => {
                if packet.is_incoming {
                    return false;
                }
            },
        }

        // Check protocol
        if self.protocol != NetworkProtocol::Any {
            match packet.protocol {
                crate::protocols::ip::IpProtocol::Tcp => {
                    if self.protocol != NetworkProtocol::Tcp {
                        return false;
                    }
                },
                crate::protocols::ip::IpProtocol::Udp => {
                    if self.protocol != NetworkProtocol::Udp {
                        return false;
                    }
                },
                crate::protocols::ip::IpProtocol::Icmp => {
                    if self.protocol != NetworkProtocol::Icmp {
                        return false;
                    }
                },
                _ => {
                    if self.protocol != NetworkProtocol::Any {
                        return false;
                    }
                },
            }
        }

        // Check source IP
        if let Some(source_range) = &self.source_ip {
            if !source_range.contains(packet.source_ip) {
                return false;
            }
        }

        // Check destination IP
        if let Some(dest_range) = &self.dest_ip {
            if !dest_range.contains(packet.dest_ip) {
                return false;
            }
        }

        // Check ports if available
        if let (Some(src_port), Some(src_range)) = (packet.source_port, &self.source_port) {
            if !src_range.contains(src_port) {
                return false;
            }
        }

        if let (Some(dst_port), Some(dst_range)) = (packet.dest_port, &self.dest_port) {
            if !dst_range.contains(dst_port) {
                return false;
            }
        }

        // Check TCP flags if specified
        if let Some(required_flags) = self.tcp_flags {
            if let Some(actual_flags) = packet.tcp_flags {
                // Check if required flags are set (simplified logic)
                if (actual_flags & required_flags) != required_flags {
                    return false;
                }
            }
        }

        true
    }

    /// Update rule statistics
    pub fn record_match(&mut self, bytes: u64) {
        self.stats.packets_matched += 1;
        self.stats.bytes_matched += bytes;
        
        let now = Instant::now();
        if self.stats.first_matched.is_none() {
            self.stats.first_matched = Some(now);
        }
        self.stats.last_matched = Some(now);
    }

    /// Check if rule should be logged
    pub fn should_log(&self) -> bool {
        matches!(self.action, FirewallAction::Log) || matches!(self.action, FirewallAction::Drop)
    }

    /// Check if rule can terminate processing
    pub fn is_terminating(&self) -> bool {
        !matches!(self.action, FirewallAction::Continue)
    }

    /// Get rule priority for sorting
    pub fn get_priority(&self) -> u32 {
        self.priority
    }

    /// Update rule priority
    pub fn set_priority(&mut self, priority: u32) {
        self.priority = priority;
        self.modified_at = Instant::now();
    }

    /// Enable rule
    pub fn enable(&mut self) {
        self.enabled = true;
        self.modified_at = Instant::now();
    }

    /// Disable rule
    pub fn disable(&mut self) {
        self.enabled = false;
        self.modified_at = Instant::now();
    }

    /// Reset statistics
    pub fn reset_stats(&mut self) {
        self.stats = RuleStatistics::default();
    }
}

/// Packet filtering information
#[derive(Debug, Clone)]
pub struct FilteredPacket {
    /// Source IP address
    pub source_ip: IpAddress,
    /// Destination IP address
    pub dest_ip: IpAddress,
    /// Source port (if applicable)
    pub source_port: Option<u16>,
    /// Destination port (if applicable)
    pub dest_port: Option<u16>,
    /// Protocol type
    pub protocol: crate::protocols::ip::IpProtocol,
    /// TCP flags (if TCP)
    pub tcp_flags: Option<u8>,
    /// ICMP type (if ICMP)
    pub icmp_type: Option<u8>,
    /// Packet size in bytes
    pub packet_size: usize,
    /// Is incoming packet
    pub is_incoming: bool,
    /// Arrival timestamp
    pub arrival_time: Instant,
    /// Interface name
    pub interface: String,
}

impl FilteredPacket {
    /// Create from IP packet
    pub fn from_ip_packet(packet: &crate::protocols::ip::IpPacket, is_incoming: bool, interface: String) -> Self {
        let source_port = None;
        let dest_port = None;
        let tcp_flags = None;
        let icmp_type = None;

        Self {
            source_ip: packet.source,
            dest_ip: packet.destination,
            source_port,
            dest_port,
            protocol: packet.protocol,
            tcp_flags,
            icmp_type,
            packet_size: packet.payload.len(),
            is_incoming,
            arrival_time: Instant::now(),
            interface,
        }
    }

    /// Create from TCP packet
    pub fn from_tcp_packet(tcp_packet: &TcpPacket, source: IpAddress, dest: IpAddress, 
                          is_incoming: bool, interface: String) -> Self {
        Self {
            source_ip: source,
            dest_ip: dest,
            source_port: Some(tcp_packet.source_port),
            dest_port: Some(tcp_packet.dest_port),
            protocol: crate::protocols::ip::IpProtocol::Tcp,
            tcp_flags: Some(tcp_packet.data_offset_flags & 0x3F),
            icmp_type: None,
            packet_size: tcp_packet.data.len(),
            is_incoming,
            arrival_time: Instant::now(),
            interface,
        }
    }

    /// Create from UDP packet
    pub fn from_udp_packet(udp_packet: &UdpPacket, source: IpAddress, dest: IpAddress,
                          is_incoming: bool, interface: String) -> Self {
        Self {
            source_ip: source,
            dest_ip: dest,
            source_port: Some(udp_packet.header.source_port),
            dest_port: Some(udp_packet.header.dest_port),
            protocol: crate::protocols::ip::IpProtocol::Udp,
            tcp_flags: None,
            icmp_type: None,
            packet_size: udp_packet.data.len(),
            is_incoming,
            arrival_time: Instant::now(),
            interface,
        }
    }

    /// Create from ICMP packet
    pub fn from_icmp_packet(icmp_packet: &IcmpPacket, source: IpAddress, dest: IpAddress,
                           is_incoming: bool, interface: String) -> Self {
        Self {
            source_ip: source,
            dest_ip: dest,
            source_port: None,
            dest_port: None,
            protocol: crate::protocols::ip::IpProtocol::Icmp,
            tcp_flags: None,
            icmp_type: Some(icmp_packet.icmp_type as u8),
            packet_size: icmp_packet.data.len(),
            is_incoming,
            arrival_time: Instant::now(),
            interface,
        }
    }
}

/// Firewall configuration
#[derive(Debug, Clone)]
pub struct FirewallConfig {
    /// Default action for unmatched packets
    pub default_action: FirewallAction,
    /// Enable stateful filtering
    pub stateful_filtering: bool,
    /// Enable logging
    pub logging_enabled: bool,
    /// Log level
    pub log_level: LogLevel,
    /// Maximum rules
    pub max_rules: usize,
    /// Rule processing timeout
    pub rule_timeout: Duration,
    /// Enable NAT traversal
    pub nat_traversal: bool,
    /// Connection tracking timeout
    pub connection_timeout: Duration,
}

#[derive(Debug, Clone, Copy)]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
    Critical,
}

impl Default for FirewallConfig {
    fn default() -> Self {
        Self {
            default_action: FirewallAction::Drop,
            stateful_filtering: true,
            logging_enabled: true,
            log_level: LogLevel::Info,
            max_rules: 10000,
            rule_timeout: Duration::from_millis(100),
            nat_traversal: false,
            connection_timeout: Duration::from_secs(300),
        }
    }
}

/// Firewall statistics
#[derive(Debug, Clone, Default)]
pub struct FirewallStats {
    /// Total packets processed
    pub total_packets: u64,
    /// Packets accepted
    pub packets_accepted: u64,
    /// Packets dropped
    pub packets_dropped: u64,
    /// Packets rejected
    pub packets_rejected: u64,
    /// Packets logged
    pub packets_logged: u64,
    /// Bytes processed
    pub bytes_processed: u64,
    /// Rule evaluations
    pub rule_evaluations: u64,
    /// Average processing time
    pub avg_processing_time: Duration,
    /// Total processing time
    pub total_processing_time: Duration,
    /// Active connections
    pub active_connections: usize,
    /// Blocked IP addresses
    pub blocked_ips: usize,
    /// Port scan detections
    pub port_scan_detections: u64,
}

/// Firewall implementation
pub struct Firewall {
    /// Filter rules
    rules: BTreeMap<u32, FilterRule>,
    /// Configuration
    config: FirewallConfig,
    /// Statistics
    stats: FirewallStats,
    /// Connection tracking
    connection_tracker: ConnectionTracker,
    /// IP blacklist
    ip_blacklist: HashMap<IpAddress, Instant>,
    /// Port scan detector
    port_scan_detector: PortScanDetector,
    /// Log handler
    log_handler: LogHandler,
}

impl Firewall {
    /// Create a new firewall
    pub fn new(config: FirewallConfig) -> Self {
        Self {
            rules: BTreeMap::new(),
            config,
            stats: FirewallStats::default(),
            connection_tracker: ConnectionTracker::new(config.connection_timeout),
            ip_blacklist: HashMap::new(),
            port_scan_detector: PortScanDetector::new(),
            log_handler: LogHandler::new(config.logging_enabled, config.log_level),
        }
    }

    /// Create firewall with default configuration
    pub fn with_default_config() -> Self {
        Self::new(FirewallConfig::default())
    }

    /// Add a filter rule
    pub fn add_rule(&mut self, rule: FilterRule) -> Result<()> {
        if self.rules.len() >= self.config.max_rules {
            return Err(NetworkError::SecurityError("Maximum rules limit reached".to_string()));
        }

        self.rules.insert(rule.id, rule);
        self.log_handler.log_rule_added(&rule);
        
        Ok(())
    }

    /// Remove a filter rule
    pub fn remove_rule(&mut self, rule_id: u32) -> Result<FilterRule> {
        if let Some(rule) = self.rules.remove(&rule_id) {
            self.log_handler.log_rule_removed(&rule);
            Ok(rule)
        } else {
            Err(NetworkError::SecurityError("Rule not found".to_string()))
        }
    }

    /// Get a rule by ID
    pub fn get_rule(&self, rule_id: u32) -> Option<&FilterRule> {
        self.rules.get(&rule_id)
    }

    /// Get all rules
    pub fn get_all_rules(&self) -> Vec<&FilterRule> {
        self.rules.values().collect()
    }

    /// Enable a rule
    pub fn enable_rule(&mut self, rule_id: u32) -> Result<()> {
        if let Some(rule) = self.rules.get_mut(&rule_id) {
            rule.enable();
            Ok(())
        } else {
            Err(NetworkError::SecurityError("Rule not found".to_string()))
        }
    }

    /// Disable a rule
    pub fn disable_rule(&mut self, rule_id: u32) -> Result<()> {
        if let Some(rule) = self.rules.get_mut(&rule_id) {
            rule.disable();
            Ok(())
        } else {
            Err(NetworkError::SecurityError("Rule not found".to_string()))
        }
    }

    /// Process a packet through the firewall
    pub fn process_packet(&mut self, filtered_packet: &FilteredPacket) -> Result<FirewallAction> {
        let start_time = Instant::now();
        self.stats.total_packets += 1;
        self.stats.bytes_processed += filtered_packet.packet_size as u64;

        // Check if source IP is blacklisted
        if self.ip_blacklist.contains_key(&filtered_packet.source_ip) {
            self.stats.packets_dropped += 1;
            self.log_handler.log_packet_blocked(filtered_packet, "IP Blacklisted");
            return Ok(FirewallAction::Drop);
        }

        // Check for port scanning
        if self.port_scan_detector.detect_port_scan(filtered_packet) {
            self.stats.port_scan_detections += 1;
            self.add_to_blacklist(filtered_packet.source_ip, Duration::from_secs(3600));
            self.stats.packets_dropped += 1;
            self.log_handler.log_security_event("Port scan detected", &filtered_packet.source_ip);
            return Ok(FirewallAction::Drop);
        }

        // Evaluate rules in priority order
        let mut action_taken = false;
        let mut final_action = self.config.default_action;

        for rule in self.rules.values() {
            if rule.matches_packet(filtered_packet) {
                self.stats.rule_evaluations += 1;
                rule.record_match(filtered_packet.packet_size as u64);

                match rule.action {
                    FirewallAction::Accept => {
                        self.stats.packets_accepted += 1;
                        if rule.should_log() {
                            self.stats.packets_logged += 1;
                            self.log_handler.log_packet_accepted(filtered_packet, &rule.name);
                        }
                        final_action = FirewallAction::Accept;
                        action_taken = true;
                        break;
                    },
                    FirewallAction::Drop => {
                        self.stats.packets_dropped += 1;
                        if rule.should_log() {
                            self.stats.packets_logged += 1;
                            self.log_handler.log_packet_dropped(filtered_packet, &rule.name);
                        }
                        final_action = FirewallAction::Drop;
                        action_taken = true;
                        break;
                    },
                    FirewallAction::Reject => {
                        self.stats.packets_rejected += 1;
                        if rule.should_log() {
                            self.stats.packets_logged += 1;
                            self.log_handler.log_packet_rejected(filtered_packet, &rule.name);
                        }
                        final_action = FirewallAction::Reject;
                        action_taken = true;
                        break;
                    },
                    FirewallAction::Log => {
                        self.stats.packets_logged += 1;
                        self.log_handler.log_packet_logged(filtered_packet, &rule.name);
                        // Continue to next rule
                    },
                    FirewallAction::Count => {
                        // Just count, continue to next rule
                    },
                    FirewallAction::Continue => {
                        // Continue to next rule
                    },
                }
            }
        }

        // If no rule matched, apply default action
        if !action_taken {
            match final_action {
                FirewallAction::Accept => self.stats.packets_accepted += 1,
                FirewallAction::Drop => self.stats.packets_dropped += 1,
                FirewallAction::Reject => self.stats.packets_rejected += 1,
                _ => {},
            }
        }

        // Update processing time statistics
        let processing_time = start_time.elapsed();
        self.stats.total_processing_time += processing_time;
        let total_evaluations = self.stats.rule_evaluations.max(1);
        self.stats.avg_processing_time = Duration::from_nanos(
            self.stats.total_processing_time.as_nanos() as u64 / total_evaluations
        );

        // Update connection tracking
        if self.config.stateful_filtering {
            self.connection_tracker.track_packet(filtered_packet);
            self.stats.active_connections = self.connection_tracker.active_connections();
        }

        Ok(final_action)
    }

    /// Add IP to blacklist
    pub fn add_to_blacklist(&mut self, ip: IpAddress, duration: Duration) {
        self.ip_blacklist.insert(ip, Instant::now() + duration);
        self.log_handler.log_security_event(&format!("IP blacklisted: {}", ip), &ip);
    }

    /// Remove IP from blacklist
    pub fn remove_from_blacklist(&mut self, ip: &IpAddress) {
        self.ip_blacklist.remove(ip);
    }

    /// Clean up expired blacklist entries
    pub fn cleanup_blacklist(&mut self) {
        let now = Instant::now();
        let before = self.ip_blacklist.len();
        
        self.ip_blacklist.retain(|_, &mut expiry| now < expiry);
        
        let removed = before - self.ip_blacklist.len();
        if removed > 0 {
            self.stats.blocked_ips = self.ip_blacklist.len();
            self.log_handler.log_info(&format!("Removed {} expired IP blacklist entries", removed));
        }
    }

    /// Update firewall configuration
    pub fn update_config(&mut self, config: FirewallConfig) {
        self.config = config;
        self.connection_tracker.set_timeout(config.connection_timeout);
        self.log_handler.set_logging(config.logging_enabled, config.log_level);
    }

    /// Get firewall statistics
    pub fn get_stats(&self) -> &FirewallStats {
        &self.stats
    }

    /// Get connection tracker statistics
    pub fn get_connection_stats(&self) -> &ConnectionTrackerStats {
        self.connection_tracker.get_stats()
    }

    /// Reset statistics
    pub fn reset_stats(&mut self) {
        self.stats = FirewallStats::default();
        self.port_scan_detector.reset_stats();
        for rule in self.rules.values_mut() {
            rule.reset_stats();
        }
        self.log_handler.log_info("Firewall statistics reset");
    }

    /// Get blocked IP addresses
    pub fn get_blocked_ips(&self) -> Vec<IpAddress> {
        self.ip_blacklist.keys().cloned().collect()
    }

    /// Check if IP is blocked
    pub fn is_ip_blocked(&self, ip: &IpAddress) -> bool {
        self.ip_blacklist.contains_key(ip)
    }

    /// Get port scan detector statistics
    pub fn get_port_scan_stats(&self) -> &PortScanStats {
        self.port_scan_detector.get_stats()
    }

    /// Load default security rules
    pub fn load_default_rules(&mut self) -> Result<()> {
        // Allow localhost traffic
        let localhost_rule = FilterRule::new(1, "Allow Localhost".to_string(), 
                                           FirewallAction::Accept, NetworkProtocol::Any);
        let mut localhost_rule = localhost_rule;
        localhost_rule.source_ip = Some(IpAddressRange::single(IpAddress::localhost()));
        localhost_rule.dest_ip = Some(IpAddressRange::single(IpAddress::localhost()));
        localhost_rule.priority = 10;
        self.add_rule(localhost_rule)?;

        // Allow established connections
        let established_rule = FilterRule::new(2, "Allow Established".to_string(),
                                             FirewallAction::Accept, NetworkProtocol::Tcp);
        let mut established_rule = established_rule;
        established_rule.tcp_flags = Some(0x10); // ACK flag
        established_rule.priority = 20;
        self.add_rule(established_rule)?;

        // Allow HTTP traffic
        let http_rule = FilterRule::new(3, "Allow HTTP".to_string(),
                                      FirewallAction::Accept, NetworkProtocol::Tcp);
        let mut http_rule = http_rule;
        http_rule.dest_port = Some(PortRange::new(80, 80));
        http_rule.priority = 30;
        self.add_rule(http_rule)?;

        // Allow HTTPS traffic
        let https_rule = FilterRule::new(4, "Allow HTTPS".to_string(),
                                       FirewallAction::Accept, NetworkProtocol::Tcp);
        let mut https_rule = https_rule;
        https_rule.dest_port = Some(PortRange::new(443, 443));
        https_rule.priority = 30;
        self.add_rule(https_rule)?;

        // Block all other traffic
        let default_rule = FilterRule::new(9999, "Default Deny".to_string(),
                                         FirewallAction::Drop, NetworkProtocol::Any);
        let mut default_rule = default_rule;
        default_rule.priority = 1000;
        self.add_rule(default_rule)?;

        Ok(())
    }
}

/// Connection tracking for stateful filtering
struct ConnectionTracker {
    /// Active connections
    connections: HashMap<(IpAddress, u16, IpAddress, u16), ConnectionState>,
    /// Connection timeout
    timeout: Duration,
}

#[derive(Debug, Clone)]
struct ConnectionState {
    /// Source address and port
    source: (IpAddress, u16),
    /// Destination address and port
    dest: (IpAddress, u16),
    /// Protocol
    protocol: NetworkProtocol,
    /// Connection state
    state: ConnectionStateType,
    /// Last activity time
    last_activity: Instant,
    /// Connection start time
    start_time: Instant,
    /// Packet counters
    packets_in: u64,
    packets_out: u64,
    bytes_in: u64,
    bytes_out: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConnectionStateType {
    New,
    Established,
    Closing,
    Closed,
}

struct ConnectionTrackerStats {
    pub total_connections: u64,
    pub active_connections: usize,
    pub completed_connections: u64,
    pub failed_connections: u64,
    pub avg_connection_duration: Duration,
    pub total_duration: Duration,
    pub measurement_count: u32,
}

impl Default for ConnectionTrackerStats {
    fn default() -> Self {
        Self {
            total_connections: 0,
            active_connections: 0,
            completed_connections: 0,
            failed_connections: 0,
            avg_connection_duration: Duration::from_secs(0),
            total_duration: Duration::from_secs(0),
            measurement_count: 0,
        }
    }
}

impl ConnectionTracker {
    /// Create a new connection tracker
    fn new(timeout: Duration) -> Self {
        Self {
            connections: HashMap::new(),
            timeout,
        }
    }

    /// Set connection timeout
    fn set_timeout(&mut self, timeout: Duration) {
        self.timeout = timeout;
    }

    /// Track a packet
    fn track_packet(&mut self, packet: &FilteredPacket) {
        if packet.source_port.is_none() || packet.dest_port.is_none() {
            return; // Only track packets with ports
        }

        let src_port = packet.source_port.unwrap();
        let dst_port = packet.dest_port.unwrap();
        let key = (packet.source_ip, src_port, packet.dest_ip, dst_port);

        // Determine protocol
        let protocol = match packet.protocol {
            crate::protocols::ip::IpProtocol::Tcp => NetworkProtocol::Tcp,
            crate::protocols::ip::IpProtocol::Udp => NetworkProtocol::Udp,
            _ => return, // Only track TCP and UDP
        };

        match self.connections.get_mut(&key) {
            Some(connection) => {
                connection.last_activity = Instant::now();
                if packet.is_incoming {
                    connection.packets_in += 1;
                    connection.bytes_in += packet.packet_size as u64;
                } else {
                    connection.packets_out += 1;
                    connection.bytes_out += packet.packet_size as u64;
                }

                // Update connection state based on TCP flags
                if protocol == NetworkProtocol::Tcp {
                    if let Some(flags) = packet.tcp_flags {
                        if flags & 0x01 != 0 { // FIN flag
                            connection.state = ConnectionStateType::Closing;
                        }
                    }
                }
            },
            None => {
                // Create new connection
                let connection = ConnectionState {
                    source: (packet.source_ip, src_port),
                    dest: (packet.dest_ip, dst_port),
                    protocol,
                    state: ConnectionStateType::New,
                    last_activity: Instant::now(),
                    start_time: Instant::now(),
                    packets_in: if packet.is_incoming { 1 } else { 0 },
                    packets_out: if !packet.is_incoming { 1 } else { 0 },
                    bytes_in: if packet.is_incoming { packet.packet_size as u64 } else { 0 },
                    bytes_out: if !packet.is_incoming { packet.packet_size as u64 } else { 0 },
                };

                self.connections.insert(key, connection);
            }
        }
    }

    /// Get active connection count
    fn active_connections(&self) -> usize {
        self.connections.len()
    }

    /// Clean up expired connections
    fn cleanup_expired(&mut self) {
        let now = Instant::now();
        let before = self.connections.len();
        
        self.connections.retain(|_, connection| {
            now.duration_since(connection.last_activity) < self.timeout
        });
        
        let removed = before - self.connections.len();
        if removed > 0 {
            log::debug!("Removed {} expired connections", removed);
        }
    }

    /// Get connection statistics
    fn get_stats(&self) -> &ConnectionTrackerStats {
        // This would need to be calculated differently in a real implementation
        todo!()
    }
}

/// Port scan detection
struct PortScanDetector {
    /// Port scan attempts
    scan_attempts: HashMap<IpAddress, PortScanState>,
    /// Detection statistics
    stats: PortScanStats,
}

#[derive(Debug, Clone)]
struct PortScanState {
    /// Scanned ports
    scanned_ports: HashMap<u16, Instant>,
    /// First scan time
    first_scan: Instant,
    /// Scan count
    scan_count: u32,
    /// Blocked status
    blocked: bool,
}

#[derive(Debug, Clone, Default)]
struct PortScanStats {
    pub total_scans_detected: u64,
    pub scans_blocked: u64,
    pub unique_attackers: usize,
    pub common_scan_ports: HashMap<u16, u64>,
}

impl PortScanDetector {
    /// Create a new port scan detector
    fn new() -> Self {
        Self {
            scan_attempts: HashMap::new(),
            stats: PortScanStats::default(),
        }
    }

    /// Detect port scanning
    fn detect_port_scan(&mut self, packet: &FilteredPacket) -> bool {
        // Only check inbound packets with destination ports
        if !packet.is_incoming || packet.dest_port.is_none() {
            return false;
        }

        let source_ip = packet.source_ip;
        let dest_port = packet.dest_port.unwrap();
        let now = Instant::now();

        let scan_state = self.scan_attempts.entry(source_ip).or_insert_with(|| PortScanState {
            scanned_ports: HashMap::new(),
            first_scan: now,
            scan_count: 0,
            blocked: false,
        });

        // Clean old port entries
        scan_state.scanned_ports.retain(|_, &mut timestamp| {
            now.duration_since(timestamp) < Duration::from_secs(300) // 5 minutes
        });

        // Record this port scan
        scan_state.scanned_ports.insert(dest_port, now);
        scan_state.scan_count += 1;

        // Check for scanning patterns
        let time_window = Duration::from_secs(60); // 1 minute window
        let recent_scans = scan_state.scanned_ports.values()
            .filter(|&&timestamp| now.duration_since(timestamp) < time_window)
            .count();

        // Detect different types of scans
        let is_scan = if scan_state.scanned_ports.len() >= 10 {
            // Port scan: many different ports in short time
            true
        } else if scan_state.scan_count >= 50 {
            // Fast scan: many attempts
            true
        } else if recent_scans >= 20 {
            // Rapid scan: many ports in 1 minute
            true
        } else {
            false
        };

        if is_scan && !scan_state.blocked {
            scan_state.blocked = true;
            self.stats.total_scans_detected += 1;
            self.stats.scans_blocked += 1;
            
            // Track common scanned ports
            *self.stats.common_scan_ports.entry(dest_port).or_insert(0) += 1;
            
            log::warn!("Port scan detected from {}: {} ports scanned", source_ip, scan_state.scanned_ports.len());
            true
        } else {
            false
        }
    }

    /// Reset statistics
    fn reset_stats(&mut self) {
        self.stats = PortScanStats::default();
    }

    /// Get detection statistics
    fn get_stats(&self) -> &PortScanStats {
        &self.stats
    }
}

/// Log handler for security events
struct LogHandler {
    /// Enable logging
    logging_enabled: bool,
    /// Log level
    log_level: LogLevel,
}

impl LogHandler {
    /// Create a new log handler
    fn new(logging_enabled: bool, log_level: LogLevel) -> Self {
        Self {
            logging_enabled,
            log_level,
        }
    }

    /// Set logging parameters
    fn set_logging(&mut self, enabled: bool, level: LogLevel) {
        self.logging_enabled = enabled;
        self.log_level = level;
    }

    /// Log packet accepted
    fn log_packet_accepted(&self, packet: &FilteredPacket, rule_name: &str) {
        if self.logging_enabled {
            log::info!("ACCEPT: {}:{} -> {}:{} ({}) [Rule: {}]", 
                      packet.source_ip, packet.source_port.map(|p| p.to_string()).unwrap_or_else(|| "*".to_string()),
                      packet.dest_ip, packet.dest_port.map(|p| p.to_string()).unwrap_or_else(|| "*".to_string()),
                      match packet.protocol {
                          crate::protocols::ip::IpProtocol::Tcp => "TCP",
                          crate::protocols::ip::IpProtocol::Udp => "UDP",
                          crate::protocols::ip::IpProtocol::Icmp => "ICMP",
                          _ => "OTHER",
                      },
                      rule_name);
        }
    }

    /// Log packet dropped
    fn log_packet_dropped(&self, packet: &FilteredPacket, rule_name: &str) {
        if self.logging_enabled {
            log::warn!("DROP: {}:{} -> {}:{} ({}) [Rule: {}]", 
                      packet.source_ip, packet.source_port.map(|p| p.to_string()).unwrap_or_else(|| "*".to_string()),
                      packet.dest_ip, packet.dest_port.map(|p| p.to_string()).unwrap_or_else(|| "*".to_string()),
                      match packet.protocol {
                          crate::protocols::ip::IpProtocol::Tcp => "TCP",
                          crate::protocols::ip::IpProtocol::Udp => "UDP",
                          crate::protocols::ip::IpProtocol::Icmp => "ICMP",
                          _ => "OTHER",
                      },
                      rule_name);
        }
    }

    /// Log packet rejected
    fn log_packet_rejected(&self, packet: &FilteredPacket, rule_name: &str) {
        if self.logging_enabled {
            log::warn!("REJECT: {}:{} -> {}:{} ({}) [Rule: {}]", 
                      packet.source_ip, packet.source_port.map(|p| p.to_string()).unwrap_or_else(|| "*".to_string()),
                      packet.dest_ip, packet.dest_port.map(|p| p.to_string()).unwrap_or_else(|| "*".to_string()),
                      match packet.protocol {
                          crate::protocols::ip::IpProtocol::Tcp => "TCP",
                          crate::protocols::ip::IpProtocol::Udp => "UDP",
                          crate::protocols::ip::IpProtocol::Icmp => "ICMP",
                          _ => "OTHER",
                      },
                      rule_name);
        }
    }

    /// Log packet blocked
    fn log_packet_blocked(&self, packet: &FilteredPacket, reason: &str) {
        if self.logging_enabled {
            log::warn!("BLOCKED: {} -> {} [{}]", packet.source_ip, packet.dest_ip, reason);
        }
    }

    /// Log packet logged
    fn log_packet_logged(&self, packet: &FilteredPacket, rule_name: &str) {
        if self.logging_enabled {
            log::info!("LOG: {}:{} -> {}:{} [Rule: {}]", 
                      packet.source_ip, packet.source_port.map(|p| p.to_string()).unwrap_or_else(|| "*".to_string()),
                      packet.dest_ip, packet.dest_port.map(|p| p.to_string()).unwrap_or_else(|| "*".to_string()),
                      rule_name);
        }
    }

    /// Log security event
    fn log_security_event(&self, event: &str, source: &IpAddress) {
        if self.logging_enabled {
            log::warn!("SECURITY: {} from {}", event, source);
        }
    }

    /// Log rule added
    fn log_rule_added(&self, rule: &FilterRule) {
        if self.logging_enabled {
            log::info!("RULE ADDED: {} (ID: {}, Priority: {})", rule.name, rule.id, rule.priority);
        }
    }

    /// Log rule removed
    fn log_rule_removed(&self, rule: &FilterRule) {
        if self.logging_enabled {
            log::info!("RULE REMOVED: {} (ID: {})", rule.name, rule.id);
        }
    }

    /// Log info message
    fn log_info(&self, message: &str) {
        if self.logging_enabled {
            log::info!("FIREWALL: {}", message);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_rule_creation() {
        let rule = FilterRule::allow_all();
        assert_eq!(rule.action, FirewallAction::Accept);
        assert_eq!(rule.protocol, NetworkProtocol::Any);
        assert!(rule.enabled);
    }

    #[test]
    fn test_ip_address_range() {
        let range = IpAddressRange::new(
            IpAddress::v4(192, 168, 1, 0),
            IpAddress::v4(192, 168, 1, 255)
        );
        
        assert!(range.contains(IpAddress::v4(192, 168, 1, 100)));
        assert!(!range.contains(IpAddress::v4(10, 0, 0, 1)));
    }

    #[test]
    fn test_port_range() {
        let web_ports = PortRange::new(80, 443);
        assert!(web_ports.contains(80));
        assert!(web_ports.contains(443));
        assert!(!web_ports.contains(22));
    }

    #[test]
    fn test_time_window() {
        let always_window = TimeWindow::always();
        assert!(always_window.is_within_window());
    }

    #[test]
    fn test_filtered_packet() {
        let source = IpAddress::v4(192, 168, 1, 100);
        let dest = IpAddress::v4(192, 168, 1, 1);
        
        let packet = FilteredPacket {
            source_ip: source,
            dest_ip: dest,
            source_port: Some(12345),
            dest_port: Some(80),
            protocol: crate::protocols::ip::IpProtocol::Tcp,
            tcp_flags: Some(0x02), // SYN
            icmp_type: None,
            packet_size: 40,
            is_incoming: true,
            arrival_time: Instant::now(),
            interface: "eth0".to_string(),
        };
        
        assert!(packet.source_port.is_some());
        assert_eq!(packet.dest_port, Some(80));
    }

    #[test]
    fn test_firewall_creation() {
        let config = FirewallConfig::default();
        let firewall = Firewall::new(config);
        assert_eq!(firewall.stats.total_packets, 0);
    }

    #[test]
    fn test_network_protocol_conversion() {
        assert_eq!(NetworkProtocol::Tcp.to_u8(), 6);
        assert_eq!(NetworkProtocol::from_u8(17), Some(NetworkProtocol::Udp));
    }
}