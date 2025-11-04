//! Network Security Module
//!
//! This module provides comprehensive network security including:
//! - Firewall with rule management
//! - Secure communication protocols
//! - VPN support
//! - Network intrusion detection and prevention
//! - Traffic monitoring and analysis

use spin::Mutex;
use alloc::vec::Vec;
use alloc::string::String;
use alloc::collections::BTreeMap;
use core::fmt;
use log::{info, warn, error, debug};

/// Network security result type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkSecurityResult {
    /// Operation succeeded
    Success,
    /// Operation failed
    Failed,
    /// Access denied
    Denied,
    /// Connection blocked
    Blocked,
    /// Intrusion detected
    IntrusionDetected,
    /// Invalid packet
    InvalidPacket,
    /// Rule not found
    RuleNotFound,
    /// Memory error
    OutOfMemory,
}

/// Firewall rule types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FirewallRuleType {
    /// Allow traffic
    Allow,
    /// Deny traffic
    Deny,
    /// Log traffic
    Log,
    /// Rate limit
    RateLimit,
}

/// Network protocol types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkProtocol {
    /// TCP protocol
    Tcp,
    /// UDP protocol
    Udp,
    /// ICMP protocol
    Icmp,
    /// Any protocol
    Any,
}

/// Network packet structure
#[derive(Debug, Clone)]
pub struct NetworkPacket {
    /// Source IP address
    pub src_ip: [u8; 4],
    /// Destination IP address
    pub dst_ip: [u8; 4],
    /// Source port (for TCP/UDP)
    pub src_port: u16,
    /// Destination port (for TCP/UDP)
    pub dst_port: u16,
    /// Protocol
    pub protocol: NetworkProtocol,
    /// Packet data
    pub data: Vec<u8>,
    /// Packet size
    pub size: usize,
    /// Arrival timestamp
    pub timestamp: u64,
    /// Interface index
    pub interface_idx: u32,
}

/// Firewall rule
#[derive(Debug, Clone)]
pub struct FirewallRule {
    /// Rule ID
    pub id: u32,
    /// Rule name
    pub name: String,
    /// Rule type
    pub rule_type: FirewallRuleType,
    /// Source IP range
    pub src_ip_range: Option<(u32, u32)>,
    /// Destination IP range
    pub dst_ip_range: Option<(u32, u32)>,
    /// Source port range
    pub src_port_range: Option<(u16, u16)>,
    /// Destination port range
    pub dst_port_range: Option<(u16, u16)>,
    /// Protocol
    pub protocol: NetworkProtocol,
    /// Rate limit (packets per second)
    pub rate_limit: Option<u32>,
    /// Priority (lower number = higher priority)
    pub priority: u32,
    /// Whether rule is active
    pub active: bool,
    /// Rule statistics
    pub stats: RuleStats,
}

/// Firewall rule statistics
#[derive(Debug, Clone, Default)]
pub struct RuleStats {
    /// Total packets matched
    pub packets_matched: u64,
    /// Total bytes processed
    pub bytes_processed: u64,
    /// Last match timestamp
    pub last_match: u64,
    /// Rule hit count
    pub hit_count: u32,
}

/// VPN tunnel configuration
#[derive(Debug, Clone)]
pub struct VpnTunnel {
    /// Tunnel ID
    pub tunnel_id: u32,
    /// Local endpoint
    pub local_endpoint: [u8; 4],
    /// Remote endpoint
    pub remote_endpoint: [u8; 4],
    /// Encryption algorithm
    pub encryption: VpnEncryption,
    /// Authentication algorithm
    pub authentication: VpnAuth,
    /// Tunnel status
    pub status: VpnStatus,
    /// Encrypted data
    pub encrypted_data: Vec<u8>,
    /// Session key
    pub session_key: Vec<u8>,
}

/// VPN encryption types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VpnEncryption {
    /// AES-128 encryption
    Aes128,
    /// AES-256 encryption
    Aes256,
    /// ChaCha20 encryption
    ChaCha20,
    /// No encryption
    None,
}

/// VPN authentication types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VpnAuth {
    /// SHA-256 authentication
    Sha256,
    /// SHA-384 authentication
    Sha384,
    /// HMAC-SHA256
    HmacSha256,
    /// No authentication
    None,
}

/// VPN tunnel status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VpnStatus {
    /// Tunnel is active
    Active,
    /// Tunnel is inactive
    Inactive,
    /// Tunnel is connecting
    Connecting,
    /// Tunnel has an error
    Error,
}

/// Intrusion detection signature
#[derive(Debug, Clone)]
pub struct IntrusionSignature {
    /// Signature ID
    pub id: u32,
    /// Signature name
    pub name: String,
    /// Protocol to monitor
    pub protocol: NetworkProtocol,
    /// Source port pattern
    pub src_port_pattern: Option<u16>,
    /// Destination port pattern
    pub dst_port_pattern: Option<u16>,
    /// Payload pattern to match
    pub payload_pattern: Vec<u8>,
    /// Severity level
    pub severity: IntrusionSeverity,
    /// Whether signature is active
    pub active: bool,
}

/// Intrusion severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntrusionSeverity {
    /// Low severity
    Low,
    /// Medium severity
    Medium,
    /// High severity
    High,
    /// Critical severity
    Critical,
}

/// Network intrusion event
#[derive(Debug, Clone)]
pub struct IntrusionEvent {
    /// Event ID
    pub event_id: u64,
    /// Source IP
    pub src_ip: [u8; 4],
    /// Destination IP
    pub dst_ip: [u8; 4],
    /// Source port
    pub src_port: u16,
    /// Destination port
    pub dst_port: u16,
    /// Protocol
    pub protocol: NetworkProtocol,
    /// Detected signature
    pub signature: IntrusionSignature,
    /// Event timestamp
    pub timestamp: u64,
    /// Response action taken
    pub response: IntrusionResponse,
}

/// Intrusion response types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntrusionResponse {
    /// Log only
    LogOnly,
    /// Block source IP
    BlockSource,
    /// Block destination IP
    BlockDestination,
    /// Block port
    BlockPort,
    /// Alert administrators
    Alert,
}

/// Network interface security configuration
#[derive(Debug, Clone)]
pub struct InterfaceSecurity {
    /// Interface index
    pub interface_idx: u32,
    /// Interface name
    pub name: String,
    /// Whether firewall is enabled
    pub firewall_enabled: bool,
    /// Whether intrusion detection is enabled
    pub ids_enabled: bool,
    /// Whether VPN is enabled
    pub vpn_enabled: bool,
    /// Default action for untracked traffic
    pub default_action: FirewallRuleType,
    /// Interface-specific rules
    pub rules: Vec<u32>,
}

/// Network security statistics
#[derive(Debug, Clone, Default)]
pub struct NetworkSecurityStats {
    /// Total packets processed
    pub total_packets: u64,
    /// Total bytes processed
    pub total_bytes: u64,
    /// Packets blocked
    pub packets_blocked: u64,
    /// Packets allowed
    pub packets_allowed: u64,
    /// Intrusions detected
    pub intrusions_detected: u64,
    /// VPN tunnels active
    pub vpn_tunnels_active: u32,
    /// Firewall rules active
    pub firewall_rules_active: u32,
    /// Last update timestamp
    pub last_update: u64,
}

/// Main network security manager
pub struct NetworkSecurity {
    /// Firewall rules
    firewall_rules: BTreeMap<u32, FirewallRule>,
    /// VPN tunnels
    vpn_tunnels: BTreeMap<u32, VpnTunnel>,
    /// Intrusion signatures
    intrusion_signatures: BTreeMap<u32, IntrusionSignature>,
    /// Interface security configurations
    interface_security: BTreeMap<u32, InterfaceSecurity>,
    /// Security statistics
    stats: NetworkSecurityStats,
    /// Intrusion events buffer
    intrusion_events: Vec<IntrusionEvent>,
    /// Maximum intrusion events to store
    max_intrusion_events: usize,
}

impl NetworkSecurity {
    /// Create new network security manager
    pub fn new() -> Self {
        Self {
            firewall_rules: BTreeMap::new(),
            vpn_tunnels: BTreeMap::new(),
            intrusion_signatures: BTreeMap::new(),
            interface_security: BTreeMap::new(),
            stats: NetworkSecurityStats::default(),
            intrusion_events: Vec::new(),
            max_intrusion_events: 1000,
        }
    }
    
    /// Add firewall rule
    pub fn add_firewall_rule(&mut self, rule: FirewallRule) -> NetworkSecurityResult {
        self.firewall_rules.insert(rule.id, rule);
        info!("Added firewall rule: ID {}", rule.id);
        NetworkSecurityResult::Success
    }
    
    /// Remove firewall rule
    pub fn remove_firewall_rule(&mut self, rule_id: u32) -> NetworkSecurityResult {
        if self.firewall_rules.remove(&rule_id).is_some() {
            info!("Removed firewall rule: ID {}", rule_id);
            NetworkSecurityResult::Success
        } else {
            NetworkSecurityResult::RuleNotFound
        }
    }
    
    /// Process network packet through firewall
    pub fn process_packet(&mut self, packet: &NetworkPacket) -> NetworkSecurityResult {
        self.stats.total_packets += 1;
        self.stats.total_bytes += packet.size as u64;
        
        // Get interface security configuration
        if let Some(iface_security) = self.interface_security.get(&packet.interface_idx) {
            if !iface_security.firewall_enabled {
                return NetworkSecurityResult::Success;
            }
        }
        
        // Check firewall rules
        for rule in self.firewall_rules.values() {
            if !rule.active {
                continue;
            }
            
            if self.packet_matches_rule(packet, rule) {
                rule.stats.packets_matched += 1;
                rule.stats.bytes_processed += packet.size as u64;
                rule.stats.hit_count += 1;
                rule.stats.last_match = packet.timestamp;
                
                match rule.rule_type {
                    FirewallRuleType::Allow => {
                        debug!("Packet allowed by rule: {}", rule.name);
                        self.stats.packets_allowed += 1;
                        return NetworkSecurityResult::Success;
                    }
                    FirewallRuleType::Deny => {
                        warn!("Packet denied by rule: {}", rule.name);
                        self.stats.packets_blocked += 1;
                        return NetworkSecurityResult::Denied;
                    }
                    FirewallRuleType::Log => {
                        info!("Packet logged by rule: {}", rule.name);
                        self.stats.packets_allowed += 1;
                        return NetworkSecurityResult::Success;
                    }
                    FirewallRuleType::RateLimit => {
                        // Rate limiting implementation
                        return NetworkSecurityResult::Success;
                    }
                }
            }
        }
        
        // No matching rule found, use default action
        if let Some(iface_security) = self.interface_security.get(&packet.interface_idx) {
            match iface_security.default_action {
                FirewallRuleType::Deny => {
                    warn!("Packet denied by default rule");
                    self.stats.packets_blocked += 1;
                    return NetworkSecurityResult::Denied;
                }
                _ => {
                    self.stats.packets_allowed += 1;
                    return NetworkSecurityResult::Success;
                }
            }
        }
        
        self.stats.packets_allowed += 1;
        NetworkSecurityResult::Success
    }
    
    /// Create VPN tunnel
    pub fn create_vpn_tunnel(&mut self, tunnel: VpnTunnel) -> NetworkSecurityResult {
        self.vpn_tunnels.insert(tunnel.tunnel_id, tunnel);
        info!("Created VPN tunnel: ID {}", tunnel.tunnel_id);
        NetworkSecurityResult::Success
    }
    
    /// Encrypt data through VPN tunnel
    pub fn encrypt_vpn_data(&mut self, tunnel_id: u32, data: &[u8]) -> Result<Vec<u8>, NetworkSecurityResult> {
        if let Some(tunnel) = self.vpn_tunnels.get_mut(&tunnel_id) {
            if tunnel.status != VpnStatus::Active {
                return Err(NetworkSecurityResult::Failed);
            }
            
            // Encrypt data using tunnel's encryption algorithm
            let encrypted = self.encrypt_data(data, tunnel.encryption);
            tunnel.encrypted_data = encrypted.clone();
            
            Ok(encrypted)
        } else {
            Err(NetworkSecurityResult::Failed)
        }
    }
    
    /// Decrypt data through VPN tunnel
    pub fn decrypt_vpn_data(&mut self, tunnel_id: u32, encrypted_data: &[u8]) -> Result<Vec<u8>, NetworkSecurityResult> {
        if let Some(tunnel) = self.vpn_tunnels.get_mut(&tunnel_id) {
            if tunnel.status != VpnStatus::Active {
                return Err(NetworkSecurityResult::Failed);
            }
            
            // Decrypt data using tunnel's encryption algorithm
            let decrypted = self.decrypt_data(encrypted_data, tunnel.encryption);
            
            Ok(decrypted)
        } else {
            Err(NetworkSecurityResult::Failed)
        }
    }
    
    /// Add intrusion detection signature
    pub fn add_intrusion_signature(&mut self, signature: IntrusionSignature) -> NetworkSecurityResult {
        self.intrusion_signatures.insert(signature.id, signature);
        info!("Added intrusion signature: {}", signature.name);
        NetworkSecurityResult::Success
    }
    
    /// Detect network intrusions
    pub fn detect_intrusions(&mut self, packet: &NetworkPacket) -> Vec<IntrusionEvent> {
        let mut events = Vec::new();
        
        // Get interface security configuration
        if let Some(iface_security) = self.interface_security.get(&packet.interface_idx) {
            if !iface_security.ids_enabled {
                return events;
            }
        }
        
        // Check against all active signatures
        for signature in self.intrusion_signatures.values() {
            if !signature.active {
                continue;
            }
            
            if self.packet_matches_signature(packet, signature) {
                let event = IntrusionEvent {
                    event_id: self.stats.total_packets,
                    src_ip: packet.src_ip,
                    dst_ip: packet.dst_ip,
                    src_port: packet.src_port,
                    dst_port: packet.dst_port,
                    protocol: packet.protocol,
                    signature: signature.clone(),
                    timestamp: packet.timestamp,
                    response: IntrusionResponse::LogOnly,
                };
                
                events.push(event);
                self.stats.intrusions_detected += 1;
                
                // Apply response action
                self.apply_intrusion_response(&event);
            }
        }
        
        // Store events in buffer
        for event in &events {
            self.intrusion_events.push(event.clone());
            if self.intrusion_events.len() > self.max_intrusion_events {
                self.intrusion_events.remove(0);
            }
        }
        
        events
    }
    
    /// Configure network interface security
    pub fn configure_interface(&mut self, config: InterfaceSecurity) -> NetworkSecurityResult {
        self.interface_security.insert(config.interface_idx, config);
        info!("Configured security for interface: {}", config.name);
        NetworkSecurityResult::Success
    }
    
    /// Get security statistics
    pub fn get_stats(&self) -> &NetworkSecurityStats {
        &self.stats
    }
    
    /// Get recent intrusion events
    pub fn get_intrusion_events(&self) -> &[IntrusionEvent] {
        &self.intrusion_events
    }
    
    /// Check if packet matches firewall rule
    fn packet_matches_rule(&self, packet: &NetworkPacket, rule: &FirewallRule) -> bool {
        // Check protocol
        if rule.protocol != NetworkProtocol::Any && rule.protocol != packet.protocol {
            return false;
        }
        
        // Check source IP range
        if let Some((start, end)) = rule.src_ip_range {
            let src_ip_u32 = u32::from_be_bytes(packet.src_ip);
            if src_ip_u32 < start || src_ip_u32 > end {
                return false;
            }
        }
        
        // Check destination IP range
        if let Some((start, end)) = rule.dst_ip_range {
            let dst_ip_u32 = u32::from_be_bytes(packet.dst_ip);
            if dst_ip_u32 < start || dst_ip_u32 > end {
                return false;
            }
        }
        
        // Check source port range
        if let Some((start, end)) = rule.src_port_range {
            if packet.src_port < start || packet.src_port > end {
                return false;
            }
        }
        
        // Check destination port range
        if let Some((start, end)) = rule.dst_port_range {
            if packet.dst_port < start || packet.dst_port > end {
                return false;
            }
        }
        
        true
    }
    
    /// Check if packet matches intrusion signature
    fn packet_matches_signature(&self, packet: &NetworkPacket, signature: &IntrusionSignature) -> bool {
        // Check protocol
        if signature.protocol != NetworkProtocol::Any && signature.protocol != packet.protocol {
            return false;
        }
        
        // Check source port
        if let Some(port) = signature.src_port_pattern {
            if packet.src_port != port {
                return false;
            }
        }
        
        // Check destination port
        if let Some(port) = signature.dst_port_pattern {
            if packet.dst_port != port {
                return false;
            }
        }
        
        // Check payload pattern
        if !signature.payload_pattern.is_empty() {
            if !packet.data.windows(signature.payload_pattern.len())
                .any(|window| window == signature.payload_pattern) {
                return false;
            }
        }
        
        true
    }
    
    /// Apply intrusion response
    fn apply_intrusion_response(&mut self, event: &IntrusionEvent) {
        match event.response {
            IntrusionResponse::BlockSource => {
                warn!("Blocking source IP: {}.{}.{}.{}", 
                    event.src_ip[0], event.src_ip[1], event.src_ip[2], event.src_ip[3]);
                // Implementation would add block rule
            }
            IntrusionResponse::BlockDestination => {
                warn!("Blocking destination IP: {}.{}.{}.{}", 
                    event.dst_ip[0], event.dst_ip[1], event.dst_ip[2], event.dst_ip[3]);
            }
            IntrusionResponse::BlockPort => {
                warn!("Blocking port: {}", event.dst_port);
            }
            IntrusionResponse::Alert => {
                warn!("ALERT: Intrusion detected: {}", event.signature.name);
            }
            IntrusionResponse::LogOnly => {
                info!("Intrusion logged: {}", event.signature.name);
            }
        }
    }
    
    /// Encrypt data using specified algorithm
    fn encrypt_data(&self, data: &[u8], encryption: VpnEncryption) -> Vec<u8> {
        match encryption {
            VpnEncryption::None => data.to_vec(),
            VpnEncryption::Aes128 => {
                // Simplified encryption (in real implementation, use proper AES)
                data.iter().map(|&b| b ^ 0x5A).collect()
            }
            VpnEncryption::Aes256 => {
                // Simplified encryption (in real implementation, use proper AES)
                data.iter().map(|&b| b ^ 0x6B).collect()
            }
            VpnEncryption::ChaCha20 => {
                // Simplified encryption (in real implementation, use proper ChaCha20)
                data.iter().map(|&b| b ^ 0x7C).collect()
            }
        }
    }
    
    /// Decrypt data using specified algorithm
    fn decrypt_data(&self, encrypted_data: &[u8], encryption: VpnEncryption) -> Vec<u8> {
        match encryption {
            VpnEncryption::None => encrypted_data.to_vec(),
            VpnEncryption::Aes128 => {
                // Simplified decryption (reverse of encrypt)
                encrypted_data.iter().map(|&b| b ^ 0x5A).collect()
            }
            VpnEncryption::Aes256 => {
                // Simplified decryption (reverse of encrypt)
                encrypted_data.iter().map(|&b| b ^ 0x6B).collect()
            }
            VpnEncryption::ChaCha20 => {
                // Simplified decryption (reverse of encrypt)
                encrypted_data.iter().map(|&b| b ^ 0x7C).collect()
            }
        }
    }
    
    /// Get firewall rule by ID
    pub fn get_firewall_rule(&self, rule_id: u32) -> Option<&FirewallRule> {
        self.firewall_rules.get(&rule_id)
    }
    
    /// Get VPN tunnel by ID
    pub fn get_vpn_tunnel(&self, tunnel_id: u32) -> Option<&VpnTunnel> {
        self.vpn_tunnels.get(&tunnel_id)
    }
    
    /// Get intrusion signature by ID
    pub fn get_intrusion_signature(&self, sig_id: u32) -> Option<&IntrusionSignature> {
        self.intrusion_signatures.get(&sig_id)
    }
    
    /// Update statistics
    pub fn update_stats(&mut self) {
        self.stats.last_update = self.get_timestamp();
        self.stats.vpn_tunnels_active = self.vpn_tunnels.values()
            .filter(|t| t.status == VpnStatus::Active).count() as u32;
        self.stats.firewall_rules_active = self.firewall_rules.values()
            .filter(|r| r.active).count() as u32;
    }
    
    /// Get current timestamp (simplified)
    fn get_timestamp(&self) -> u64 {
        0 // Placeholder
    }
}

/// Network security manager instance
static NETWORK_SECURITY: Mutex<Option<NetworkSecurity>> = Mutex::new(None);

/// Initialize network security
pub fn init() {
    let mut net_security = NetworkSecurity::new();
    
    // Load default security rules
    load_default_rules(&mut net_security);
    
    *net_security.lock() = Some(net_security);
    info!("Network security initialized");
}

/// Load default security rules
fn load_default_rules(net_security: &mut NetworkSecurity) {
    // Add default deny rule
    let default_rule = FirewallRule {
        id: 0,
        name: "Default Deny".to_string(),
        rule_type: FirewallRuleType::Deny,
        src_ip_range: None,
        dst_ip_range: None,
        src_port_range: None,
        dst_port_range: None,
        protocol: NetworkProtocol::Any,
        rate_limit: None,
        priority: u32::MAX,
        active: true,
        stats: RuleStats::default(),
    };
    
    let _ = net_security.add_firewall_rule(default_rule);
    
    // Add some common allow rules
    let allow_dns = FirewallRule {
        id: 1,
        name: "Allow DNS".to_string(),
        rule_type: FirewallRuleType::Allow,
        src_ip_range: None,
        dst_ip_range: Some((u32::from_be_bytes([8, 8, 8, 8]), u32::from_be_bytes([8, 8, 8, 8]))),
        src_port_range: None,
        dst_port_range: Some((53, 53)),
        protocol: NetworkProtocol::Udp,
        rate_limit: None,
        priority: 1,
        active: true,
        stats: RuleStats::default(),
    };
    
    let _ = net_security.add_firewall_rule(allow_dns);
    
    info!("Loaded default security rules");
}

/// Get network security instance
pub fn instance() -> Option<NetworkSecurity> {
    NETWORK_SECURITY.lock().as_ref().cloned()
}

/// Process network packet
pub fn process_packet(packet: &NetworkPacket) -> NetworkSecurityResult {
    if let Some(ref mut net_security) = *NETWORK_SECURITY.lock() {
        net_security.process_packet(packet)
    } else {
        NetworkSecurityResult::Failed
    }
}

/// Detect network intrusions
pub fn detect_intrusions(packet: &NetworkPacket) -> Vec<IntrusionEvent> {
    if let Some(ref mut net_security) = *NETWORK_SECURITY.lock() {
        net_security.detect_intrusions(packet)
    } else {
        Vec::new()
    }
}

/// Add firewall rule
pub fn add_firewall_rule(rule: FirewallRule) -> NetworkSecurityResult {
    if let Some(ref mut net_security) = *NETWORK_SECURITY.lock() {
        net_security.add_firewall_rule(rule)
    } else {
        NetworkSecurityResult::Failed
    }
}

/// Create VPN tunnel
pub fn create_vpn_tunnel(tunnel: VpnTunnel) -> NetworkSecurityResult {
    if let Some(ref mut net_security) = *NETWORK_SECURITY.lock() {
        net_security.create_vpn_tunnel(tunnel)
    } else {
        NetworkSecurityResult::Failed
    }
}

/// Get security statistics
pub fn get_stats() -> Option<NetworkSecurityStats> {
    if let Some(ref net_security) = *NETWORK_SECURITY.lock() {
        Some(net_security.get_stats().clone())
    } else {
        None
    }
}

impl fmt::Display for NetworkSecurityResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            NetworkSecurityResult::Success => write!(f, "Success"),
            NetworkSecurityResult::Failed => write!(f, "Failed"),
            NetworkSecurityResult::Denied => write!(f, "Denied"),
            NetworkSecurityResult::Blocked => write!(f, "Blocked"),
            NetworkSecurityResult::IntrusionDetected => write!(f, "Intrusion Detected"),
            NetworkSecurityResult::InvalidPacket => write!(f, "Invalid Packet"),
            NetworkSecurityResult::RuleNotFound => write!(f, "Rule Not Found"),
            NetworkSecurityResult::OutOfMemory => write!(f, "Out of Memory"),
        }
    }
}

impl Default for NetworkSecurity {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_security_creation() {
        let net_security = NetworkSecurity::new();
        assert_eq!(net_security.get_stats().total_packets, 0);
    }

    #[test]
    fn test_firewall_rule_creation() {
        let rule = FirewallRule {
            id: 1,
            name: "Test Rule".to_string(),
            rule_type: FirewallRuleType::Allow,
            src_ip_range: None,
            dst_ip_range: None,
            src_port_range: Some((80, 80)),
            dst_port_range: None,
            protocol: NetworkProtocol::Tcp,
            rate_limit: None,
            priority: 1,
            active: true,
            stats: RuleStats::default(),
        };
        
        assert_eq!(rule.name, "Test Rule");
        assert_eq!(rule.rule_type, FirewallRuleType::Allow);
    }

    #[test]
    fn test_network_packet_creation() {
        let packet = NetworkPacket {
            src_ip: [192, 168, 1, 1],
            dst_ip: [192, 168, 1, 100],
            src_port: 12345,
            dst_port: 80,
            protocol: NetworkProtocol::Tcp,
            data: vec![1, 2, 3, 4],
            size: 4,
            timestamp: 1234567890,
            interface_idx: 0,
        };
        
        assert_eq!(packet.src_ip, [192, 168, 1, 1]);
        assert_eq!(packet.dst_port, 80);
    }
}
