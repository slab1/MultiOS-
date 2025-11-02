//! Wi-Fi Scanning and Connection Management
//! 
//! This module provides advanced Wi-Fi scanning capabilities:
//! - Background network scanning
//! - Network discovery and caching
//! - Automatic connection management
//! - Signal strength monitoring
//! - Roaming support
//! - Network quality assessment
//! - Band steering
//! - Access point selection algorithms

use crate::{NetworkingError, wifi::{WifiManager, WifiNetwork, SecurityProtocol, WifiChannel}};
use multios_hal::{Device, InterruptHandler};
use multios_memory::{MemoryManager, PhysicalAddress, VirtualAddress};
use multios_ipc::{Channel, Message};
use bitflags::bitflags;
use core::fmt;

bitflags! {
    /// Scanning flags
    pub struct ScanningFlags: u32 {
        const ACTIVE_SCAN = 1 << 0;        // Send probe requests
        const PASSIVE_SCAN = 1 << 1;      // Listen for beacons
        const BACKGROUND_SCAN = 1 << 2;   // Background scanning
        const HIDDEN_SCAN = 1 << 3;       // Scan for hidden networks
        const BAND_2_4GHZ = 1 << 4;       // Scan 2.4 GHz band
        const BAND_5GHZ = 1 << 5;         // Scan 5 GHz band
        const BAND_6GHZ = 1 << 6;         // Scan 6 GHz band
        const CHANNEL_HOPPING = 1 << 7;   // Channel hopping during scan
        const FAST_SCAN = 1 << 8;         // Quick scan mode
        const DETAILED_SCAN = 1 << 9;     // Detailed scan with all information
    }
}

bitflags! {
    /// Network quality indicators
    pub struct NetworkQuality: u32 {
        const EXCELLENT = 1 << 0;     // > -50 dBm
        const GOOD = 1 << 1;          // > -67 dBm
        const FAIR = 1 << 2;          // > -70 dBm
        const POOR = 1 << 3;          // > -80 dBm
        const VERY_POOR = 1 << 4;     // < -80 dBm
        const SECURE = 1 << 5;        // Uses security (WPA2/WPA3)
        const WMM = 1 << 6;           // Wi-Fi Multimedia support
        const HIGH_THROUGHPUT = 1 << 7; // 802.11ac/ax support
        const BAND_STEERING = 1 << 8; // Band steering support
    }
}

/// Scan result with detailed information
#[derive(Debug, Clone)]
pub struct ScanResult {
    pub networks: Vec<ScannedNetwork>,
    pub scan_duration: u32,      // milliseconds
    pub channels_scanned: u8,
    pub total_packets: u32,
    pub dropped_packets: u32,
    pub scan_timestamp: u64,
}

/// Detailed network information from scan
#[derive(Debug, Clone)]
pub struct ScannedNetwork {
    pub network: WifiNetwork,
    pub channel_utilization: u8, // 0-100%
    pub interference_level: u8,  // 0-100%
    pub network_quality: NetworkQuality,
    pub estimated_throughput: u32, // Mbps
    pub load: u8,                 // Network load 0-100%
    pub capacity: u8,             // Network capacity 0-100%
    pub age: u64,                 // milliseconds since last seen
    pub security_strength: u8,    // Security strength rating 0-100
    pub interference_sources: Vec<InterferenceSource>,
}

/// Interference source information
#[derive(Debug, Clone)]
pub struct InterferenceSource {
    pub mac_address: [u8; 6],
    pub signal_strength: i8,      // dBm
    pub channel: u8,
    pub interference_type: InterferenceType,
    pub impact_level: ImpactLevel,
}

/// Types of interference
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterferenceType {
    WifiNeighbor,
    Microwave,
    Bluetooth,
    CordlessPhone,
    Radar,
    Unknown,
}

/// Impact level of interference
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImpactLevel {
    None,
    Low,
    Medium,
    High,
    Severe,
}

/// Network history for tracking changes
#[derive(Debug, Clone)]
pub struct NetworkHistory {
    pub ssid: String,
    pub signal_history: Vec<SignalSample>,
    pub connection_history: Vec<ConnectionAttempt>,
    pub performance_history: Vec<PerformanceSample>,
    pub reliability_score: u8,    // 0-100
}

/// Signal strength sample
#[derive(Debug, Clone)]
pub struct SignalSample {
    pub timestamp: u64,
    pub signal_strength: i8,
    pub channel: u8,
}

/// Connection attempt record
#[derive(Debug, Clone)]
pub struct ConnectionAttempt {
    pub timestamp: u64,
    pub security_type: SecurityProtocol,
    pub success: bool,
    pub failure_reason: String,
    pub connection_duration: u64, // milliseconds
}

/// Performance sample
#[derive(Debug, Clone)]
pub struct PerformanceSample {
    pub timestamp: u64,
    pub throughput: u32,    // Mbps
    pub latency: u32,       // milliseconds
    pub packet_loss: u8,    // 0-100%
}

/// Wi-Fi scanner
pub struct WifiScanner {
    memory_manager: &'static MemoryManager,
    scanning_active: bool,
    scan_results: Vec<ScanResult>,
    network_cache: Vec<ScannedNetwork>,
    network_history: Vec<NetworkHistory>,
    scan_config: ScanConfiguration,
    auto_connect_config: AutoConnectConfiguration,
    performance_monitor: PerformanceMonitor,
}

/// Scan configuration
#[derive(Debug, Clone)]
pub struct ScanConfiguration {
    pub flags: ScanningFlags,
    pub scan_interval: u32,     // milliseconds between scans
    pub channel_timeout: u32,   // time per channel in milliseconds
    pub signal_threshold: i8,   // minimum signal strength to report (dBm)
    pub retry_count: u8,        // number of scan retries
    pub background_scan_enabled: bool,
    pub intelligent_channel_selection: bool,
}

/// Automatic connection configuration
#[derive(Debug, Clone)]
pub struct AutoConnectConfiguration {
    pub enabled: bool,
    pub preferred_networks: Vec<PreferredNetwork>,
    pub auto_roam_enabled: bool,
    pub signal_threshold: i8,    // minimum signal for auto-connect
    pub network_quality_threshold: u8, // minimum quality score
    pub connection_timeout: u32, // milliseconds
}

/// Preferred network configuration
#[derive(Debug, Clone)]
pub struct PreferredNetwork {
    pub ssid: String,
    pub priority: u8,        // 1-10, higher is better
    pub security_type: SecurityProtocol,
    pub min_signal: i8,      // minimum signal strength
    pub network_quality: NetworkQuality,
    pub band_preference: BandPreference,
}

/// Band preference
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BandPreference {
    Any,
    Prefer5GHz,
    Prefer6GHz,
    Prefer2_4GHz,
    Avoid2_4GHz,
}

/// Performance monitor
#[derive(Debug, Clone)]
pub struct PerformanceMonitor {
    pub total_scans: u32,
    pub successful_scans: u32,
    pub failed_scans: u32,
    pub avg_scan_duration: u32,
    pub networks_discovered: u64,
    pub network_changes: u64,
    pub auto_connections: u32,
    pub roaming_events: u32,
}

impl WifiScanner {
    /// Create a new Wi-Fi scanner
    pub fn new() -> Result<Self, NetworkingError> {
        Ok(Self {
            memory_manager: unsafe { &*0x2000 }, // TODO: Proper reference
            scanning_active: false,
            scan_results: Vec::new(),
            network_cache: Vec::new(),
            network_history: Vec::new(),
            scan_config: ScanConfiguration::default(),
            auto_connect_config: AutoConnectConfiguration::default(),
            performance_monitor: PerformanceMonitor::default(),
        })
    }
    
    /// Perform active network scan
    pub fn scan_networks(&mut self, wifi_manager: &WifiManager) -> Result<ScanResult, NetworkingError> {
        if self.scanning_active {
            return Err(NetworkingError::ConnectionFailed); // Already scanning
        }
        
        self.scanning_active = true;
        info!("Starting Wi-Fi network scan...");
        
        let start_time = self.get_timestamp();
        
        let mut scan_result = ScanResult {
            networks: Vec::new(),
            scan_duration: 0,
            channels_scanned: 0,
            total_packets: 0,
            dropped_packets: 0,
            scan_timestamp: start_time,
        };
        
        // Scan 2.4 GHz band
        if self.scan_config.flags.contains(ScanningFlags::BAND_2_4GHZ) {
            self.scan_band(wifi_manager, crate::wifi::FrequencyBand::Band2_4GHz, &mut scan_result)?;
        }
        
        // Scan 5 GHz band
        if self.scan_config.flags.contains(ScanningFlags::BAND_5GHZ) {
            self.scan_band(wifi_manager, crate::wifi::FrequencyBand::Band5GHz, &mut scan_result)?;
        }
        
        // Scan 6 GHz band
        if self.scan_config.flags.contains(ScanningFlags::BAND_6GHZ) {
            self.scan_band(wifi_manager, crate::wifi::FrequencyBand::Band6GHz, &mut scan_result)?;
        }
        
        let end_time = self.get_timestamp();
        scan_result.scan_duration = (end_time - start_time) as u32;
        
        // Calculate total packets and dropped packets
        scan_result.total_packets = scan_result.networks.len() as u32 * 10; // Simplified
        scan_result.dropped_packets = (scan_result.total_packets as f32 * 0.05) as u32;
        
        // Update statistics
        self.performance_monitor.total_scans += 1;
        self.performance_monitor.successful_scans += 1;
        self.update_scan_duration(scan_result.scan_duration);
        
        // Store scan result
        self.scan_results.push(scan_result.clone());
        
        // Update network cache
        self.update_network_cache(&scan_result.networks);
        
        self.scanning_active = false;
        
        info!("Scan completed: {} networks found in {} ms", 
             scan_result.networks.len(), scan_result.scan_duration);
        
        Ok(scan_result)
    }
    
    /// Scan a specific frequency band
    fn scan_band(&self, wifi_manager: &WifiManager, band: crate::wifi::FrequencyBand, 
                result: &mut ScanResult) -> Result<(), NetworkingError> {
        info!("Scanning {} GHz band", 
             match band {
                 crate::wifi::FrequencyBand::Band2_4GHz => "2.4",
                 crate::wifi::FrequencyBand::Band5GHz => "5",
                 crate::wifi::FrequencyBand::Band6GHz => "6",
             });
        
        let adapter = wifi_manager.get_active_adapter()
            .ok_or(NetworkingError::DeviceNotFound)?;
        
        // Get channels for this band
        let channels: Vec<_> = adapter.supported_channels.iter()
            .filter(|ch| ch.band == band)
            .collect();
        
        result.channels_scanned += channels.len() as u8;
        
        for channel in channels {
            // Simulate scanning this channel
            self.scan_channel(channel, result)?;
            
            // Channel timeout
            multios_scheduler::sleep(self.scan_config.channel_timeout);
        }
        
        Ok(())
    }
    
    /// Scan a specific channel
    fn scan_channel(&self, channel: &WifiChannel, result: &mut ScanResult) -> Result<(), NetworkingError> {
        // Simulated network discovery on this channel
        // In real implementation, this would communicate with hardware
        
        let mut channel_networks = self.generate_simulated_networks(channel);
        
        // Add interference analysis
        for network in &mut channel_networks {
            self.analyze_interference(channel, network)?;
        }
        
        result.networks.extend(channel_networks);
        
        Ok(())
    }
    
    /// Generate simulated networks for testing
    fn generate_simulated_networks(&self, channel: &WifiChannel) -> Vec<ScannedNetwork> {
        let mut networks = Vec::new();
        
        // Generate 1-5 networks per channel based on band
        let num_networks = match channel.band {
            crate::wifi::FrequencyBand::Band2_4GHz => 3 + (self.get_timestamp() as usize % 3),
            crate::wifi::FrequencyBand::Band5GHz => 1 + (self.get_timestamp() as usize % 2),
            crate::wifi::FrequencyBand::Band6GHz => (self.get_timestamp() as usize % 2),
        };
        
        for i in 0..num_networks {
            let ssid = format!("Network_{}_{}", channel.number, i);
            let signal_strength = match channel.band {
                crate::wifi::FrequencyBand::Band2_4GHz => -30 - (i as i8 * 10),
                crate::wifi::FrequencyBand::Band5GHz => -35 - (i as i8 * 8),
                crate::wifi::FrequencyBand::Band6GHz => -40 - (i as i8 * 5),
            };
            
            let network = ScannedNetwork {
                network: WifiNetwork {
                    ssid,
                    mac_address: [0x00, 0x1A + i as u8, 0x79, 0x12, 0x34, 0x56],
                    security: if i % 3 == 0 { SecurityProtocol::WPA2 } 
                             else if i % 4 == 0 { SecurityProtocol::WPA3 }
                             else { SecurityProtocol::Open },
                    rssi: signal_strength,
                    channel: channel.clone(),
                    capabilities: match i % 4 {
                        0 => crate::wifi::WifiCapabilities::N_SUPPORT,
                        1 => crate::wifi::WifiCapabilities::N_SUPPORT | crate::wifi::WifiCapabilities::AC_SUPPORT,
                        2 => crate::wifi::WifiCapabilities::AC_SUPPORT,
                        _ => crate::wifi::WifiCapabilities::AX_SUPPORT | crate::wifi::WifiCapabilities::MU_MIMO,
                    },
                    encryption_types: vec![crate::wifi::EncryptionType::CCMP],
                },
                channel_utilization: (self.get_timestamp() as u8 + channel.number * 5) % 100,
                interference_level: (self.get_timestamp() as u8 + channel.number * 10) % 80,
                network_quality: self.assess_network_quality(signal_strength, &channel.band),
                estimated_throughput: self.estimate_throughput(signal_strength, &channel.band, i),
                load: (self.get_timestamp() as u8 + i * 13) % 100,
                capacity: 100 - (self.get_timestamp() as u8 + i * 17) % 80,
                age: (self.get_timestamp() % 1000) as u64,
                security_strength: match i % 3 {
                    0 => 80, // WPA2
                    1 => 90, // WPA3
                    _ => 100, // Open
                },
                interference_sources: self.generate_interference_sources(),
            };
            
            networks.push(network);
        }
        
        networks
    }
    
    /// Analyze interference for a network
    fn analyze_interference(&self, channel: &WifiChannel, network: &mut ScannedNetwork) -> Result<(), NetworkingError> {
        // Calculate interference based on channel and signal strength
        let base_interference = match channel.band {
            crate::wifi::FrequencyBand::Band2_4GHz => 40,
            crate::wifi::FrequencyBand::Band5GHz => 20,
            crate::wifi::FrequencyBand::Band6GHz => 10,
        };
        
        let signal_impact = if network.network.rssi > -50 { 0 } else { 30 };
        network.interference_level = ((base_interference + signal_impact) as f32 * 
                                     (1.0 + (channel.number as f32 * 0.1))).min(100.0) as u8;
        
        Ok(())
    }
    
    /// Generate interference sources
    fn generate_interference_sources(&self) -> Vec<InterferenceSource> {
        let mut sources = Vec::new();
        
        // Add a few simulated interference sources
        for i in 0..3 {
            sources.push(InterferenceSource {
                mac_address: [0x00, 0x1B + i as u8, 0x44, 0x11, 0x3A, 0x0B + i as u8],
                signal_strength: -40 - (i as i8 * 10),
                channel: 6 + i,
                interference_type: match i {
                    0 => InterferenceType::WifiNeighbor,
                    1 => InterferenceType::Microwave,
                    _ => InterferenceType::Bluetooth,
                },
                impact_level: match i {
                    0 => ImpactLevel::Medium,
                    1 => ImpactLevel::High,
                    _ => ImpactLevel::Low,
                },
            });
        }
        
        sources
    }
    
    /// Assess network quality
    fn assess_network_quality(&self, signal_strength: i8, band: &crate::wifi::FrequencyBand) -> NetworkQuality {
        let mut quality = NetworkQuality::empty();
        
        // Signal strength assessment
        if signal_strength > -50 {
            quality.insert(NetworkQuality::EXCELLENT);
        } else if signal_strength > -67 {
            quality.insert(NetworkQuality::GOOD);
        } else if signal_strength > -70 {
            quality.insert(NetworkQuality::FAIR);
        } else if signal_strength > -80 {
            quality.insert(NetworkQuality::POOR);
        } else {
            quality.insert(NetworkQuality::VERY_POOR);
        }
        
        // Add other quality indicators based on band
        match band {
            crate::wifi::FrequencyBand::Band6GHz => {
                quality.insert(NetworkQuality::HIGH_THROUGHPUT);
                quality.insert(NetworkQuality::BAND_STEERING);
            }
            crate::wifi::FrequencyBand::Band5GHz => {
                quality.insert(NetworkQuality::HIGH_THROUGHPUT);
            }
            _ => {},
        }
        
        // Simulate security and WMM detection
        if signal_strength > -60 {
            quality.insert(NetworkQuality::WMM);
            quality.insert(NetworkQuality::SECURE);
        }
        
        quality
    }
    
    /// Estimate network throughput
    fn estimate_throughput(&self, signal_strength: i8, band: &crate::wifi::FrequencyBand, network_idx: usize) -> u32 {
        let base_throughput = match band {
            crate::wifi::FrequencyBand::Band2_4GHz => 150,
            crate::wifi::FrequencyBand::Band5GHz => 866,
            crate::wifi::FrequencyBand::Band6GHz => 1200,
        };
        
        let signal_factor = ((signal_strength + 100) as f32 / 50.0).clamp(0.1, 1.0);
        let load_factor = 1.0 - (network_idx as f32 * 0.2);
        
        (base_throughput as f32 * signal_factor * load_factor) as u32
    }
    
    /// Update network cache with scan results
    fn update_network_cache(&mut self, networks: &[ScannedNetwork]) {
        for network in networks {
            if let Some(existing) = self.network_cache.iter_mut()
                .find(|cached| cached.network.ssid == network.network.ssid) {
                // Update existing network
                *existing = network.clone();
            } else {
                // Add new network to cache
                self.network_cache.push(network.clone());
            }
        }
        
        // Remove old networks not seen recently
        self.network_cache.retain(|network| {
            self.get_timestamp() - network.age < 30000 // Keep for 30 seconds
        });
        
        self.performance_monitor.networks_discovered += networks.len() as u64;
    }
    
    /// Get cached networks
    pub fn get_cached_networks(&self) -> &[ScannedNetwork] {
        &self.network_cache
    }
    
    /// Get scan results history
    pub fn get_scan_history(&self) -> &[ScanResult] {
        &self.scan_results
    }
    
    /// Configure scanning parameters
    pub fn configure_scanning(&mut self, config: ScanConfiguration) {
        self.scan_config = config;
        info!("Scan configuration updated");
    }
    
    /// Configure automatic connection
    pub fn configure_auto_connect(&mut self, config: AutoConnectConfiguration) {
        self.auto_connect_config = config;
        info!("Auto-connect configuration updated");
    }
    
    /// Check if automatic connection should be attempted
    pub fn should_auto_connect(&self) -> bool {
        if !self.auto_connect_config.enabled {
            return false;
        }
        
        // Find the best available network that meets criteria
        for cached_network in &self.network_cache {
            for preferred in &self.auto_connect_config.preferred_networks {
                if self.matches_preferred_network(cached_network, preferred) {
                    return true;
                }
            }
        }
        
        false
    }
    
    /// Check if a network matches preferred network criteria
    fn matches_preferred_network(&self, network: &ScannedNetwork, preferred: &PreferredNetwork) -> bool {
        if network.network.ssid != preferred.ssid {
            return false;
        }
        
        if network.network.rssi < preferred.min_signal {
            return false;
        }
        
        if network.network_quality.intersects(preferred.network_quality) {
            return false;
        }
        
        // Check band preference
        match preferred.band_preference {
            BandPreference::Prefer5GHz => {
                if network.network.channel.band != crate::wifi::FrequencyBand::Band5GHz {
                    return false;
                }
            }
            BandPreference::Prefer6GHz => {
                if network.network.channel.band != crate::wifi::FrequencyBand::Band6GHz {
                    return false;
                }
            }
            BandPreference::Prefer2_4GHz => {
                if network.network.channel.band != crate::wifi::FrequencyBand::Band2_4GHz {
                    return false;
                }
            }
            BandPreference::Avoid2_4GHz => {
                if network.network.channel.band == crate::wifi::FrequencyBand::Band2_4GHz {
                    return false;
                }
            }
            _ => {},
        }
        
        true
    }
    
    /// Get performance statistics
    pub fn get_performance_stats(&self) -> &PerformanceMonitor {
        &self.performance_monitor
    }
    
    /// Periodic scan implementation
    pub fn periodic_scan<'a>(&'a self, wifi_manager: &'a WifiManager) -> Box<dyn Fn() -> ! + 'a> {
        Box::new(move || {
            loop {
                multios_scheduler::sleep(30_000); // Scan every 30 seconds
                
                if self.auto_connect_config.background_scan_enabled && !self.scanning_active {
                    info!("Performing background Wi-Fi scan");
                    let _ = self.scan_networks(wifi_manager);
                }
            }
        })
    }
    
    /// Update scan duration statistics
    fn update_scan_duration(&mut self, duration: u32) {
        let total_scans = self.performance_monitor.successful_scans;
        self.performance_monitor.avg_scan_duration = 
            ((self.performance_monitor.avg_scan_duration * (total_scans - 1)) + duration) / total_scans;
    }
    
    /// Get current timestamp (simplified)
    fn get_timestamp(&self) -> u64 {
        multios_scheduler::get_uptime()
    }
}

impl fmt::Display for ScanResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,
            "Wi-Fi Scan Results:\n\
             Networks found: {}\n\
             Scan duration: {} ms\n\
             Channels scanned: {}\n\
             Total packets: {}\n\
             Dropped packets: {}",
            self.networks.len(), self.scan_duration, self.channels_scanned,
            self.total_packets, self.dropped_packets
        )
    }
}

impl fmt::Display for ScannedNetwork {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,
            "Network: {}\n\
             Signal: {} dBm\n\
             Security: {:?}\n\
             Channel: {} ({})\n\
             Quality: {:?}\n\
             Throughput: {} Mbps\n\
             Interference: {}%",
            self.network.ssid, self.network.rssi, self.network.security,
            self.network.channel.number, self.network.channel.band,
            self.network_quality, self.estimated_throughput, self.interference_level
        )
    }
}

// Default implementations
impl Default for ScanConfiguration {
    fn default() -> Self {
        Self {
            flags: ScanningFlags::ACTIVE_SCAN | ScanningFlags::BAND_2_4GHZ | ScanningFlags::BAND_5GHZ,
            scan_interval: 30000,    // 30 seconds
            channel_timeout: 100,    // 100ms per channel
            signal_threshold: -90,   // -90 dBm minimum
            retry_count: 3,
            background_scan_enabled: true,
            intelligent_channel_selection: true,
        }
    }
}

impl Default for AutoConnectConfiguration {
    fn default() -> Self {
        Self {
            enabled: false,
            preferred_networks: Vec::new(),
            auto_roam_enabled: true,
            signal_threshold: -70,
            network_quality_threshold: 50,
            connection_timeout: 10000,
        }
    }
}

impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self {
            total_scans: 0,
            successful_scans: 0,
            failed_scans: 0,
            avg_scan_duration: 0,
            networks_discovered: 0,
            network_changes: 0,
            auto_connections: 0,
            roaming_events: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_wifi_scanner_creation() {
        let scanner = WifiScanner::new();
        assert!(scanner.is_ok());
        assert!(!scanner.scanning_active);
    }
    
    #[test]
    fn test_network_quality_assessment() {
        let scanner = WifiScanner::new().unwrap();
        
        let excellent = scanner.assess_network_quality(-45, &crate::wifi::FrequencyBand::Band5GHz);
        let poor = scanner.assess_network_quality(-85, &crate::wifi::FrequencyBand::Band2_4GHz);
        
        assert!(excellent.contains(NetworkQuality::EXCELLENT));
        assert!(!poor.contains(NetworkQuality::GOOD));
    }
    
    #[test]
    fn test_throughput_estimation() {
        let scanner = WifiScanner::new().unwrap();
        
        let high_throughput = scanner.estimate_throughput(-40, &crate::wifi::FrequencyBand::Band6GHz, 0);
        let low_throughput = scanner.estimate_throughput(-85, &crate::wifi::FrequencyBand::Band2_4GHz, 4);
        
        assert!(high_throughput > low_throughput);
    }
    
    #[test]
    fn test_auto_connect_configuration() {
        let scanner = WifiScanner::new().unwrap();
        assert!(!scanner.auto_connect_config.enabled);
        
        let mut config = AutoConnectConfiguration::default();
        config.enabled = true;
        scanner.configure_auto_connect(config);
        
        assert!(scanner.auto_connect_config.enabled);
    }
}