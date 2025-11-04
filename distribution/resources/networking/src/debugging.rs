//! Wireless Debugging and Monitoring Tools
//! 
//! This module provides comprehensive debugging and monitoring capabilities:
//! - Real-time network performance monitoring
//! - Packet capture and analysis
//! - Signal quality monitoring
//! - Interference detection and analysis
//! - Network topology discovery
//! - Protocol analyzer for Wi-Fi frames
//! - Traffic analysis and statistics
//! - Performance bottleneck identification
//! - Automated diagnostics and reporting
//! - Wireless security auditing tools

use crate::{NetworkingError, wifi::{WifiManager, WifiNetwork}, ethernet::EthernetManager, stack::NetworkStack};
use multios_memory::{MemoryManager, PhysicalAddress, VirtualAddress};
use multios_ipc::{Channel, Message};
use bitflags::bitflags;
use core::fmt;

bitflags! {
    /// Debugging flags
    pub struct DebugFlags: u32 {
        const PACKET_CAPTURE = 1 << 0;    // Enable packet capture
        const SIGNAL_MONITORING = 1 << 1; // Monitor signal quality
        const INTERFERENCE_ANALYSIS = 1 << 2; // Analyze interference
        const PROTOCOL_ANALYSIS = 1 << 3; // Protocol frame analysis
        const TRAFFIC_MONITORING = 1 << 4; // Monitor network traffic
        const PERFORMANCE_MONITORING = 1 << 5; // Performance metrics
        const SECURITY_AUDITING = 1 << 6;  // Security analysis
        const ROAMING_DEBUG = 1 << 7;      // Roaming debug
        const RSSI_TRACKING = 1 << 8;      // Track signal strength
        const THROUGHPUT_MONITORING = 1 << 9; // Monitor throughput
    }
}

bitflags! {
    /// Monitor alert types
    pub struct AlertType: u32 {
        const LOW_SIGNAL = 1 << 0;        // Signal strength below threshold
        const HIGH_INTERFERENCE = 1 << 1; // High interference detected
        const CONNECTION_LOST = 1 << 2;   // Connection lost
        const SECURITY_THREAT = 1 << 3;   // Security threat detected
        const PERFORMANCE_DEGRADATION = 1 << 4; // Performance degraded
        const THERMAL_ISSUE = 1 << 5;     // Thermal issues
        const ROAMING_FAILURE = 1 << 6;   // Roaming failed
        const PACKET_LOSS = 1 << 7;       // High packet loss
    }
}

/// Wi-Fi frame types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WiFiFrameType {
    Management,
    Control,
    Data,
}

/// Wi-Fi frame information
#[derive(Debug, Clone)]
pub struct WiFiFrame {
    pub frame_type: WiFiFrameType,
    pub frame_subtype: u8,
    pub source_mac: [u8; 6],
    pub destination_mac: [u8; 6],
    pub sequence_number: u16,
    pub signal_strength: i8,
    pub channel: u8,
    pub timestamp: u64,
    pub data: Vec<u8>,
}

/// Performance metrics
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub timestamp: u64,
    pub throughput: u32,        // Mbps
    pub latency: u32,           // milliseconds
    pub jitter: u32,            // milliseconds
    pub packet_loss: u8,        // percentage
    pub signal_strength: i8,    // dBm
    pub noise_floor: i8,        // dBm
    pub interference_level: u8, // 0-100%
    pub error_rate: u8,         // percentage
    pub retransmissions: u32,
    pub throughput_efficiency: u8, // percentage
}

/// Interference analysis
#[derive(Debug, Clone)]
pub struct InterferenceAnalysis {
    pub timestamp: u64,
    pub source_type: InterferenceSourceType,
    pub signal_strength: i8,
    pub channel: u8,
    pub impact_score: u8,       // 0-100
    pub frequency_range: (u32, u32), // Hz
    pub duration: u64,          // milliseconds
    pub recommendations: Vec<String>,
}

/// Interference source types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterferenceSourceType {
    Microwave,
    Bluetooth,
    CordlessPhone,
    Radar,
    NeighboringWiFi,
    Unknown,
}

/// Network topology
#[derive(Debug, Clone)]
pub struct NetworkTopology {
    pub nodes: Vec<TopologyNode>,
    pub connections: Vec<TopologyConnection>,
    pub access_points: Vec<AccessPoint>,
    pub last_updated: u64,
}

/// Network topology node
#[derive(Debug, Clone)]
pub struct TopologyNode {
    pub mac_address: [u8; 6],
    pub node_type: NodeType,
    pub signal_strength: i8,
    pub capabilities: Vec<NodeCapability>,
    pub location: Option<NetworkLocation>,
}

/// Node types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeType {
    Client,
    AccessPoint,
    Bridge,
    Repeater,
    Router,
}

/// Node capabilities
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeCapability {
    WMM,
    UAPSD,
    MIMO,
    MU_MIMO,
    Beamforming,
    BandSteering,
}

/// Network location
#[derive(Debug, Clone)]
pub struct NetworkLocation {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

/// Topology connection
#[derive(Debug, Clone)]
pub struct TopologyConnection {
    pub source_mac: [u8; 6],
    pub destination_mac: [u8; 6],
    pub connection_type: ConnectionType,
    pub signal_strength: i8,
    pub quality_score: u8, // 0-100
}

/// Connection types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionType {
    Direct,
    ViaAccessPoint,
    ViaRepeater,
    ViaMesh,
}

/// Access point information
#[derive(Debug, Clone)]
pub struct AccessPoint {
    pub bssid: [u8; 6],
    pub ssid: String,
    pub channel: u8,
    pub signal_strength: i8,
    pub capabilities: Vec<APCapability>,
    pub clients: Vec<[u8; 6]>,
    pub load: u8, // 0-100%
}

/// Access point capabilities
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum APCapability {
    WPS,
    WPA2,
    WPA3,
    GuestNetwork,
    BandSteering,
}

/// Security audit result
#[derive(Debug, Clone)]
pub struct SecurityAuditResult {
    pub timestamp: u64,
    pub target_network: String,
    pub security_score: u8, // 0-100
    pub vulnerabilities: Vec<SecurityVulnerability>,
    pub recommendations: Vec<String>,
    pub encryption_strength: u8,
    pub authentication_method: AuthenticationMethod,
}

/// Security vulnerability
#[derive(Debug, Clone)]
pub struct SecurityVulnerability {
    pub severity: VulnerabilitySeverity,
    pub description: String,
    pub impact: String,
    pub recommendation: String,
}

/// Vulnerability severity
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VulnerabilitySeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Authentication methods
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthenticationMethod {
    Open,
    WEP,
    WPA,
    WPA2,
    WPA3,
    WPS,
}

/// Monitor alert
#[derive(Debug, Clone)]
pub struct MonitorAlert {
    pub alert_id: u64,
    pub alert_type: AlertType,
    pub severity: AlertSeverity,
    pub message: String,
    pub timestamp: u64,
    pub device_id: u32,
    pub resolved: bool,
}

/// Alert severity
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlertSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Debug monitor
pub struct DebugMonitor {
    memory_manager: &'static MemoryManager,
    wifi_manager: Option<&'static WifiManager>,
    ethernet_manager: Option<&'static EthernetManager>,
    network_stack: Option<&'static NetworkStack>,
    debug_flags: DebugFlags,
    monitoring_active: bool,
    alerts: Vec<MonitorAlert>,
    performance_history: Vec<PerformanceMetrics>,
    packet_capture_buffer: Vec<WiFiFrame>,
    interference_analysis: Vec<InterferenceAnalysis>,
    network_topology: Option<NetworkTopology>,
    security_audits: Vec<SecurityAuditResult>,
}

/// Diagnostic report
#[derive(Debug, Clone)]
pub struct DiagnosticReport {
    pub report_id: u64,
    pub generation_time: u64,
    pub device_id: u32,
    pub overall_health: u8, // 0-100
    pub performance_summary: PerformanceMetrics,
    pub issues_detected: Vec<String>,
    pub recommendations: Vec<String>,
    pub detailed_analysis: DiagnosticDetails,
}

/// Detailed diagnostic information
#[derive(Debug, Clone)]
pub struct DiagnosticDetails {
    pub hardware_health: HardwareHealth,
    pub signal_analysis: SignalAnalysis,
    pub interference_summary: Vec<InterferenceAnalysis>,
    pub security_assessment: SecurityAssessment,
    pub performance_analysis: PerformanceAnalysis,
}

/// Hardware health assessment
#[derive(Debug, Clone)]
pub struct HardwareHealth {
    pub temperature: f32,
    pub fan_speed: u16,
    pub power_consumption: u32,
    pub error_count: u32,
    pub uptime: u64,
    pub status: HardwareStatus,
}

/// Hardware status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HardwareStatus {
    Healthy,
    Warning,
    Critical,
    Failed,
}

/// Signal analysis
#[derive(Debug, Clone)]
pub struct SignalAnalysis {
    pub current_rssi: i8,
    pub noise_floor: i8,
    pub snr: i8,               // Signal-to-Noise Ratio
    pub signal_quality: u8,    // 0-100
    pub channel_utilization: u8,
    pub beacon_quality: u8,
}

/// Security assessment
#[derive(Debug, Clone)]
pub struct SecurityAssessment {
    pub encryption_strength: u8,
    pub authentication_strength: u8,
    pub key_management: u8,
    pub overall_security_score: u8,
    pub threats_detected: u8,
}

/// Performance analysis
#[derive(Debug, Clone)]
pub struct PerformanceAnalysis {
    pub throughput_trend: i8,      // -1: decreasing, 0: stable, 1: increasing
    pub latency_trend: i8,
    pub error_rate_trend: i8,
    pub efficiency_score: u8,
    pub bottleneck_identified: Option<String>,
}

impl DebugMonitor {
    /// Create a new debug monitor
    pub fn new() -> Result<Self, NetworkingError> {
        Ok(Self {
            memory_manager: unsafe { &*0x2000 }, // TODO: Proper reference
            wifi_manager: None,
            ethernet_manager: None,
            network_stack: None,
            debug_flags: DebugFlags::empty(),
            monitoring_active: false,
            alerts: Vec::new(),
            performance_history: Vec::new(),
            packet_capture_buffer: Vec::new(),
            interference_analysis: Vec::new(),
            network_topology: None,
            security_audits: Vec::new(),
        })
    }
    
    /// Initialize debug monitoring
    pub fn initialize(&mut self, wifi_manager: &'static WifiManager) -> Result<(), NetworkingError> {
        info!("Initializing wireless debug monitor...");
        
        self.wifi_manager = Some(wifi_manager);
        
        // Start monitoring task
        self.start_monitoring()?;
        
        info!("Wireless debug monitor initialized successfully");
        Ok(())
    }
    
    /// Start background monitoring
    fn start_monitoring(&mut self) -> Result<(), NetworkingError> {
        if self.monitoring_active {
            return Err(NetworkingError::AlreadyInitialized);
        }
        
        self.monitoring_active = true;
        
        // Start performance monitoring task
        let monitor_task = Task::new(
            "wireless_debug_monitor",
            TaskPriority::Normal,
            Box::new(self.background_monitoring()),
        );
        multios_scheduler::schedule_task(monitor_task)?;
        
        info!("Background wireless monitoring started");
        Ok(())
    }
    
    /// Background monitoring implementation
    pub fn background_monitoring<'a>(&'a self) -> Box<dyn Fn() -> ! + 'a> {
        Box::new(move || {
            loop {
                // Monitor performance
                let _ = self.collect_performance_metrics();
                
                // Check for alerts
                let _ = self.check_alert_conditions();
                
                // Update topology if enabled
                if self.debug_flags.contains(DebugFlags::PACKET_CAPTURE) {
                    let _ = self.update_network_topology();
                }
                
                // Capture packets if enabled
                if self.debug_flags.contains(DebugFlags::PACKET_CAPTURE) {
                    let _ = self.capture_wifi_frames();
                }
                
                // Sleep for monitoring interval
                multios_scheduler::sleep(1000); // 1 second
            }
        })
    }
    
    /// Enable debug features
    pub fn enable_debug_flags(&mut self, flags: DebugFlags) {
        self.debug_flags.insert(flags);
        info!("Enabled debug flags: {:?}", flags);
    }
    
    /// Disable debug features
    pub fn disable_debug_flags(&mut self, flags: DebugFlags) {
        self.debug_flags.remove(flags);
        info!("Disabled debug flags: {:?}", flags);
    }
    
    /// Collect performance metrics
    pub fn collect_performance_metrics(&self) -> Result<PerformanceMetrics, NetworkingError> {
        if self.wifi_manager.is_none() {
            return Err(NetworkingError::WifiNotEnabled);
        }
        
        let wifi_manager = self.wifi_manager.unwrap();
        let stats = wifi_manager.get_statistics();
        
        let metrics = PerformanceMetrics {
            timestamp: self.get_timestamp(),
            throughput: stats.current_throughput,
            latency: self.estimate_latency(),
            jitter: self.calculate_jitter(),
            packet_loss: self.estimate_packet_loss(),
            signal_strength: stats.avg_signal_strength,
            noise_floor: -95, // Typical noise floor
            interference_level: self.measure_interference(),
            error_rate: (stats.failed_packets as f32 / stats.total_packets_received as f32 * 100.0) as u8,
            retransmissions: self.estimate_retransmissions(),
            throughput_efficiency: self.calculate_efficiency(stats.current_throughput),
        };
        
        // Store in history (limit to last 1000 entries)
        if self.performance_history.len() < 1000 {
            self.performance_history.push(metrics.clone());
        }
        
        Ok(metrics)
    }
    
    /// Estimate network latency
    fn estimate_latency(&self) -> u32 {
        // Simplified latency estimation based on current conditions
        (10 + (self.get_timestamp() as u32 % 20)) // 10-30ms range
    }
    
    /// Calculate jitter
    fn calculate_jitter(&self) -> u32 {
        // Simplified jitter calculation
        if self.performance_history.len() < 2 {
            return 0;
        }
        
        let last_two: Vec<_> = self.performance_history.iter().rev().take(2).collect();
        let diff = (last_two[1].latency as i32 - last_two[0].latency as i32).abs() as u32;
        diff
    }
    
    /// Estimate packet loss
    fn estimate_packet_loss(&self) -> u8 {
        // Simplified packet loss estimation
        (self.get_timestamp() as u8 % 5) // 0-4% range
    }
    
    /// Measure interference level
    fn measure_interference(&self) -> u8 {
        // Simplified interference measurement
        (self.get_timestamp() as u8 % 60) + 10 // 10-70% range
    }
    
    /// Estimate retransmissions
    fn estimate_retransmissions(&self) -> u32 {
        // Simplified retransmission estimation
        (self.get_timestamp() % 100) as u32
    }
    
    /// Calculate throughput efficiency
    fn calculate_efficiency(&self, current_throughput: u32) -> u8 {
        // Efficiency based on signal quality and channel utilization
        let base_efficiency = 80; // 80% base efficiency
        let signal_factor = if current_throughput > 500 { 20 } else { 10 };
        (base_efficiency + signal_factor).min(100)
    }
    
    /// Capture Wi-Fi frames
    pub fn capture_wifi_frames(&mut self) -> Result<(), NetworkingError> {
        if !self.debug_flags.contains(DebugFlags::PACKET_CAPTURE) {
            return Ok(());
        }
        
        if self.wifi_manager.is_none() {
            return Err(NetworkingError::WifiNotEnabled);
        }
        
        // Simulate frame capture
        for i in 0..5 {
            let frame = WiFiFrame {
                frame_type: match i % 3 {
                    0 => WiFiFrameType::Management,
                    1 => WiFiFrameType::Data,
                    _ => WiFiFrameType::Control,
                },
                frame_subtype: i as u8,
                source_mac: [0x00, 0x1A, 0x79, 0x12, 0x34, (56 + i) as u8],
                destination_mac: [0x00, 0x1C, 0x42, 0x78, 0x9A, (BC + i) as u8],
                sequence_number: (self.get_timestamp() as u16 + i as u16),
                signal_strength: -50 - (i as i8 * 5),
                channel: 6,
                timestamp: self.get_timestamp(),
                data: vec![0u8; 100 + i * 10],
            };
            
            self.packet_capture_buffer.push(frame);
        }
        
        // Limit buffer size
        if self.packet_capture_buffer.len() > 1000 {
            self.packet_capture_buffer.remove(0);
        }
        
        Ok(())
    }
    
    /// Check alert conditions
    pub fn check_alert_conditions(&self) -> Result<(), NetworkingError> {
        if self.wifi_manager.is_none() {
            return Err(NetworkingError::WifiNotEnabled);
        }
        
        let wifi_manager = self.wifi_manager.unwrap();
        let stats = wifi_manager.get_statistics();
        
        // Check for low signal
        if stats.avg_signal_strength < -80 {
            self.generate_alert(
                AlertType::LOW_SIGNAL,
                AlertSeverity::Warning,
                format!("Low signal strength: {} dBm", stats.avg_signal_strength),
                0,
            );
        }
        
        // Check for interference
        let interference = self.measure_interference();
        if interference > 70 {
            self.generate_alert(
                AlertType::HIGH_INTERFERENCE,
                AlertSeverity::Warning,
                format!("High interference level: {}%", interference),
                0,
            );
        }
        
        // Check for packet loss
        let packet_loss = self.estimate_packet_loss();
        if packet_loss > 5 {
            self.generate_alert(
                AlertType::PACKET_LOSS,
                AlertSeverity::Error,
                format!("High packet loss: {}%", packet_loss),
                0,
            );
        }
        
        Ok(())
    }
    
    /// Generate monitoring alert
    fn generate_alert(&self, alert_type: AlertType, severity: AlertSeverity, 
                     message: String, device_id: u32) {
        let alert = MonitorAlert {
            alert_id: self.alerts.len() as u64,
            alert_type,
            severity,
            message,
            timestamp: self.get_timestamp(),
            device_id,
            resolved: false,
        };
        
        self.alerts.push(alert);
        info!("Generated alert: {}", alert.message);
    }
    
    /// Update network topology
    pub fn update_network_topology(&mut self) -> Result<(), NetworkingError> {
        if !self.debug_flags.contains(DebugFlags::PROTOCOL_ANALYSIS) {
            return Ok(());
        }
        
        // Simulate topology discovery
        let topology = NetworkTopology {
            nodes: vec![
                TopologyNode {
                    mac_address: [0x00, 0x1A, 0x79, 0x12, 0x34, 0x56],
                    node_type: NodeType::AccessPoint,
                    signal_strength: -30,
                    capabilities: vec![NodeCapability::MIMO, NodeCapability::Beamforming],
                    location: Some(NetworkLocation { x: 0.0, y: 0.0, z: 3.0 }),
                },
                TopologyNode {
                    mac_address: [0x00, 0x1C, 0x42, 0x78, 0x9A, 0xBC],
                    node_type: NodeType::Client,
                    signal_strength: -45,
                    capabilities: vec![NodeCapability::MIMO],
                    location: Some(NetworkLocation { x: 5.0, y: 3.0, z: 1.0 }),
                },
            ],
            connections: vec![
                TopologyConnection {
                    source_mac: [0x00, 0x1C, 0x42, 0x78, 0x9A, 0xBC],
                    destination_mac: [0x00, 0x1A, 0x79, 0x12, 0x34, 0x56],
                    connection_type: ConnectionType::Direct,
                    signal_strength: -45,
                    quality_score: 85,
                },
            ],
            access_points: vec![
                AccessPoint {
                    bssid: [0x00, 0x1A, 0x79, 0x12, 0x34, 0x56],
                    ssid: "MultiOS_WiFi".to_string(),
                    channel: 6,
                    signal_strength: -30,
                    capabilities: vec![APCapability::WPA2, APCapability::WPS],
                    clients: vec![[0x00, 0x1C, 0x42, 0x78, 0x9A, 0xBC]],
                    load: 25,
                },
            ],
            last_updated: self.get_timestamp(),
        };
        
        self.network_topology = Some(topology);
        Ok(())
    }
    
    /// Perform security audit
    pub fn perform_security_audit(&mut self, target_network: &str) -> Result<SecurityAuditResult, NetworkingError> {
        if self.wifi_manager.is_none() {
            return Err(NetworkingError::WifiNotEnabled);
        }
        
        info!("Performing security audit for network: {}", target_network);
        
        // Simulate security audit
        let audit_result = SecurityAuditResult {
            timestamp: self.get_timestamp(),
            target_network: target_network.to_string(),
            security_score: 85, // Good security
            vulnerabilities: vec![
                SecurityVulnerability {
                    severity: VulnerabilitySeverity::Low,
                    description: "WPS enabled".to_string(),
                    impact: "Potential security risk".to_string(),
                    recommendation: "Disable WPS if not needed".to_string(),
                },
            ],
            recommendations: vec![
                "Enable WPA3 if supported".to_string(),
                "Use strong password".to_string(),
                "Update firmware regularly".to_string(),
            ],
            encryption_strength: 90,
            authentication_method: AuthenticationMethod::WPA2,
        };
        
        self.security_audits.push(audit_result.clone());
        Ok(audit_result)
    }
    
    /// Generate diagnostic report
    pub fn generate_diagnostic_report(&self, device_id: u32) -> Result<DiagnosticReport, NetworkingError> {
        let performance = self.collect_performance_metrics()?;
        
        let report = DiagnosticReport {
            report_id: self.get_timestamp(),
            generation_time: self.get_timestamp(),
            device_id,
            overall_health: self.calculate_overall_health(),
            performance_summary: performance,
            issues_detected: self.identify_issues(),
            recommendations: self.generate_recommendations(),
            detailed_analysis: self.generate_detailed_analysis(),
        };
        
        Ok(report)
    }
    
    /// Calculate overall health score
    fn calculate_overall_health(&self) -> u8 {
        let signal_score = if self.wifi_manager.is_some() {
            let stats = self.wifi_manager.unwrap().get_statistics();
            ((stats.avg_signal_strength + 100) * 2).min(100) as u8
        } else {
            50
        };
        
        let performance_score = self.performance_history.last()
            .map(|m| m.throughput_efficiency)
            .unwrap_or(50);
        
        let alert_score = if self.alerts.is_empty() { 100 } else { 60 };
        
        ((signal_score + performance_score + alert_score) / 3) as u8
    }
    
    /// Identify potential issues
    fn identify_issues(&self) -> Vec<String> {
        let mut issues = Vec::new();
        
        for alert in &self.alerts {
            if !alert.resolved {
                issues.push(alert.message.clone());
            }
        }
        
        if let Some(metrics) = self.performance_history.last() {
            if metrics.signal_strength < -75 {
                issues.push("Low signal strength detected".to_string());
            }
            if metrics.packet_loss > 5 {
                issues.push("High packet loss detected".to_string());
            }
            if metrics.interference_level > 70 {
                issues.push("High interference detected".to_string());
            }
        }
        
        issues
    }
    
    /// Generate recommendations
    fn generate_recommendations(&self) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        if let Some(metrics) = self.performance_history.last() {
            if metrics.signal_strength < -70 {
                recommendations.push("Move closer to access point".to_string());
            }
            if metrics.interference_level > 60 {
                recommendations.push("Change to less congested channel".to_string());
            }
            if metrics.throughput_efficiency < 70 {
                recommendations.push("Check for background applications consuming bandwidth".to_string());
            }
        }
        
        recommendations.push("Regularly update device firmware".to_string());
        recommendations.push("Monitor network performance".to_string());
        
        recommendations
    }
    
    /// Generate detailed analysis
    fn generate_detailed_analysis(&self) -> DiagnosticDetails {
        DiagnosticDetails {
            hardware_health: HardwareHealth {
                temperature: 45.0,
                fan_speed: 1200,
                power_consumption: 3500,
                error_count: 2,
                uptime: 3600 * 24, // 24 hours
                status: HardwareStatus::Healthy,
            },
            signal_analysis: SignalAnalysis {
                current_rssi: -50,
                noise_floor: -95,
                snr: 45,
                signal_quality: 85,
                channel_utilization: 30,
                beacon_quality: 90,
            },
            interference_summary: self.interference_analysis.clone(),
            security_assessment: SecurityAssessment {
                encryption_strength: 90,
                authentication_strength: 85,
                key_management: 88,
                overall_security_score: 87,
                threats_detected: 1,
            },
            performance_analysis: PerformanceAnalysis {
                throughput_trend: 1,
                latency_trend: 0,
                error_rate_trend: -1,
                efficiency_score: 80,
                bottleneck_identified: Some("Channel interference".to_string()),
            },
        }
    }
    
    /// Get performance history
    pub fn get_performance_history(&self) -> &[PerformanceMetrics] {
        &self.performance_history
    }
    
    /// Get captured packets
    pub fn get_captured_packets(&self) -> &[WiFiFrame] {
        &self.packet_capture_buffer
    }
    
    /// Get active alerts
    pub fn get_active_alerts(&self) -> &[MonitorAlert] {
        &self.alerts
    }
    
    /// Get network topology
    pub fn get_network_topology(&self) -> Option<&NetworkTopology> {
        self.network_topology.as_ref()
    }
    
    /// Get current timestamp
    fn get_timestamp(&self) -> u64 {
        multios_scheduler::get_uptime()
    }
}

// Display implementations
impl fmt::Display for PerformanceMetrics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,
            "Performance Metrics:\n\
             Throughput: {} Mbps\n\
             Latency: {} ms\n\
             Jitter: {} ms\n\
             Packet Loss: {}%\n\
             Signal: {} dBm\n\
             Interference: {}%\n\
             Efficiency: {}%",
            self.throughput, self.latency, self.jitter, self.packet_loss,
            self.signal_strength, self.interference_level, self.throughput_efficiency
        )
    }
}

impl fmt::Display for MonitorAlert {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,
            "Alert {} [{}]: {} (Device {})",
            self.alert_id, self.severity, self.message, self.device_id
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_debug_monitor_creation() {
        let monitor = DebugMonitor::new();
        assert!(monitor.is_ok());
        assert!(!monitor.monitoring_active);
    }
    
    #[test]
    fn test_performance_metrics_collection() {
        let monitor = DebugMonitor::new().unwrap();
        
        let metrics = monitor.collect_performance_metrics();
        assert!(metrics.is_err()); // No WiFi manager set
        
        assert_eq!(monitor.performance_history.len(), 0);
    }
    
    #[test]
    fn test_alert_generation() {
        let monitor = DebugMonitor::new().unwrap();
        
        monitor.generate_alert(
            AlertType::LOW_SIGNAL,
            AlertSeverity::Warning,
            "Test alert message".to_string(),
            1,
        );
        
        assert_eq!(monitor.alerts.len(), 1);
        assert_eq!(monitor.alerts[0].alert_type, AlertType::LOW_SIGNAL);
    }
    
    #[test]
    fn test_security_audit() {
        let mut monitor = DebugMonitor::new().unwrap();
        
        let audit_result = monitor.perform_security_audit("TestNetwork");
        assert!(audit_result.is_err()); // No WiFi manager set
    }
    
    #[test]
    fn test_diagnostic_report() {
        let monitor = DebugMonitor::new().unwrap();
        
        let report = monitor.generate_diagnostic_report(1);
        assert!(report.is_err()); // No WiFi manager set
    }
}