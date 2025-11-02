//! Wi-Fi Scanning Example
//! 
//! This example demonstrates how to use the MultiOS Wi-Fi driver
//! to scan for networks and connect to access points.

use multios_networking::prelude::*;
use multios_hal::{DeviceManager, MemoryManager};
use multios_scheduler::{Scheduler, Task, TaskPriority};
use std::io::{self, Write};

fn main() {
    println!("MultiOS Wi-Fi Scanning Example");
    println!("==============================");
    
    // Initialize the networking system
    println!("1. Initializing networking system...");
    let memory_manager = unsafe { &*0x2000 as &MemoryManager };
    let device_manager = unsafe { &*0x1000 as &DeviceManager };
    
    match NetworkingManager::init(memory_manager, device_manager) {
        Ok(()) => println!("   ✓ Networking system initialized"),
        Err(e) => {
            println!("   ✗ Failed to initialize networking: {}", e);
            return;
        }
    }
    
    // Get the networking manager
    let manager = get_manager().expect("Failed to get networking manager");
    
    // Initialize Wi-Fi
    println!("2. Initializing Wi-Fi subsystem...");
    let mut wifi_manager = match manager.wifi_manager() {
        Ok(manager) => {
            println!("   ✓ Wi-Fi manager retrieved");
            manager
        }
        Err(e) => {
            println!("   ✗ Failed to get Wi-Fi manager: {}", e);
            return;
        }
    };
    
    // Initialize adapters
    println!("3. Initializing Wi-Fi adapters...");
    match wifi_manager.initialize_adapters() {
        Ok(()) => println!("   ✓ Wi-Fi adapters initialized"),
        Err(e) => {
            println!("   ✗ Failed to initialize adapters: {}", e);
            return;
        }
    }
    
    // Display adapter information
    println!("4. Available Wi-Fi adapters:");
    let adapters = wifi_manager.get_adapters();
    for adapter in adapters {
        println!("   - {} (MAC: {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X})",
                adapter.name,
                adapter.mac_address[0], adapter.mac_address[1], adapter.mac_address[2],
                adapter.mac_address[3], adapter.mac_address[4], adapter.mac_address[5]);
        println!("     Capabilities: {:?}", adapter.capabilities);
        println!("     Supported bands: {:?}", adapter.supported_bands);
    }
    
    // Get active adapter
    if let Some(adapter) = wifi_manager.get_active_adapter() {
        println!("\n5. Active adapter: {}", adapter.name);
    }
    
    // Scan for networks
    println!("\n6. Scanning for Wi-Fi networks...");
    match wifi_manager.scan_networks(10000) { // 10 second timeout
        Ok(networks) => {
            println!("   ✓ Found {} networks:", networks.len());
            println!("   {}", "=".repeat(80));
            
            for (i, network) in networks.iter().enumerate() {
                println!("   Network {}: {}", i + 1, network.ssid);
                println!("     BSSID: {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
                        network.mac_address[0], network.mac_address[1], network.mac_address[2],
                        network.mac_address[3], network.mac_address[4], network.mac_address[5]);
                println!("     Signal Strength: {} dBm", network.rssi);
                println!("     Channel: {} ({})", network.channel.number, network.channel.band);
                println!("     Security: {:?}", network.security);
                println!("     Encryption: {:?}", network.encryption_types);
                println!("     Capabilities: {:?}", network.capabilities);
                println!("   {}", "-".repeat(80));
            }
        }
        Err(e) => {
            println!("   ✗ Scan failed: {}", e);
            return;
        }
    }
    
    // Interactive connection
    println!("\n7. Interactive connection test");
    println!("   Do you want to connect to a network? (y/n):");
    
    if should_connect() {
        connect_to_network(&mut wifi_manager);
    }
    
    // Display statistics
    println!("\n8. Wi-Fi Statistics:");
    let stats = wifi_manager.get_statistics();
    println!("{}", stats);
    
    println!("\n9. Cleanup and shutdown...");
    
    println!("\n✓ Wi-Fi scanning example completed successfully!");
}

fn should_connect() -> bool {
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap_or_default();
    input.trim().eq_ignore_ascii_case("y") || input.trim().eq_ignore_ascii_case("yes")
}

fn connect_to_network(wifi_manager: &mut WifiManager) {
    println!("\n   Available networks:");
    match wifi_manager.scan_networks(5000) {
        Ok(networks) => {
            for (i, network) in networks.iter().enumerate() {
                println!("   {}. {} ({} dBm, {:?})", 
                        i + 1, network.ssid, network.rssi, network.security);
            }
            
            println!("   Select network number to connect:");
            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap_or_default();
            
            if let Ok(selection) = input.trim().parse::<usize>() {
                if selection > 0 && selection <= networks.len() {
                    let selected_network = &networks[selection - 1];
                    
                    println!("   Connecting to: {}", selected_network.ssid);
                    
                    // Create Wi-Fi configuration
                    let mut config = WifiConfig {
                        ssid: selected_network.ssid.clone(),
                        security: selected_network.security.clone(),
                        password: None,
                        auto_connect: false,
                        prioritize_saved: true,
                        hidden_network: false,
                    };
                    
                    // Ask for password if needed
                    if selected_network.security != SecurityProtocol::Open {
                        println!("   Enter password for {}:", selected_network.ssid);
                        let mut password = String::new();
                        io::stdin().read_line(&mut password).unwrap_or_default();
                        config.password = Some(password.trim().to_string());
                    }
                    
                    // Attempt connection
                    match wifi_manager.connect_to_network(config) {
                        Ok(connection) => {
                            println!("   ✓ Connection initiated to: {}", connection.ssid);
                            println!("   Connection ID: {}", connection.id);
                            println!("   State: {:?}", connection.state);
                        }
                        Err(e) => {
                            println!("   ✗ Connection failed: {}", e);
                        }
                    }
                } else {
                    println!("   ✗ Invalid selection");
                }
            }
        }
        Err(e) => {
            println!("   ✗ Failed to get network list: {}", e);
        }
    }
}

//
// Example of using the Wi-Fi scanning module directly
//
fn demonstrate_advanced_scanning() {
    println!("\nAdvanced Wi-Fi Scanning Demo");
    println!("============================");
    
    // Create a dedicated Wi-Fi scanner
    let scanner = WifiScanner::new().expect("Failed to create Wi-Fi scanner");
    
    // Configure scanning parameters
    let scan_config = ScanConfiguration {
        flags: ScanningFlags::ACTIVE_SCAN | 
               ScanningFlags::BAND_2_4GHZ | 
               ScanningFlags::BAND_5GHZ | 
               ScanningFlags::BAND_6GHZ,
        scan_interval: 30000,
        channel_timeout: 200,
        signal_threshold: -80,
        retry_count: 3,
        background_scan_enabled: true,
        intelligent_channel_selection: true,
    };
    
    // Configure auto-connect
    let auto_config = AutoConnectConfiguration {
        enabled: true,
        preferred_networks: vec![
            PreferredNetwork {
                ssid: "MultiOS_Main".to_string(),
                priority: 10,
                security_type: SecurityProtocol::WPA3,
                min_signal: -65,
                network_quality: NetworkQuality::GOOD | NetworkQuality::SECURE,
                band_preference: BandPreference::Prefer6GHz,
            },
            PreferredNetwork {
                ssid: "MultiOS_Guest".to_string(),
                priority: 5,
                security_type: SecurityProtocol::WPA2,
                min_signal: -70,
                network_quality: NetworkQuality::FAIR | NetworkQuality::SECURE,
                band_preference: BandPreference::Any,
            },
        ],
        auto_roam_enabled: true,
        signal_threshold: -75,
        network_quality_threshold: 60,
        connection_timeout: 15000,
    };
    
    // Apply configurations
    scanner.configure_scanning(scan_config);
    scanner.configure_auto_connect(auto_config);
    
    println!("✓ Advanced scanning configured");
    println!("  - Active scan across 2.4/5/6 GHz bands");
    println!("  - Intelligent channel selection enabled");
    println!("  - Auto-connect configured for 2 networks");
    
    // Get performance statistics
    let perf_stats = scanner.get_performance_stats();
    println!("\nPerformance Statistics:");
    println!("  Total scans: {}", perf_stats.total_scans);
    println!("  Successful scans: {}", perf_stats.successful_scans);
    println!("  Networks discovered: {}", perf_stats.networks_discovered);
    
    // Show cached networks
    let cached_networks = scanner.get_cached_networks();
    println!("\nCached networks ({} total):", cached_networks.len());
    for network in cached_networks {
        println!("  - {} ({} dBm, {:?})", 
                network.network.ssid, 
                network.network.rssi, 
                network.network_quality);
    }
}

//
// Example of Wi-Fi security management
//
fn demonstrate_security_management() {
    println!("\nWi-Fi Security Management Demo");
    println!("===============================");
    
    let mut security_manager = SecurityManager::new().expect("Failed to create security manager");
    
    // Configure security for WPA2-PSK
    let wpa2_config = WPA2PSKConfig {
        ssid: "MultiOS_Main".to_string(),
        password: "MySecurePassword123!".to_string(),
        pmk: None,
    };
    
    match security_manager.initialize_wpa2_psk(wpa2_config) {
        Ok(session) => {
            println!("✓ WPA2-PSK session created");
            println!("  SSID: {}", session.ssid);
            println!("  Protocol: {:?}", session.protocol);
            println!("  Key Management: {:?}", session.key_mgmt_algorithm);
            println!("  Encryption: {:?}", session.encryption_algorithm);
        }
        Err(e) => {
            println!("✗ Failed to create WPA2-PSK session: {}", e);
        }
    }
    
    // Configure security for WPA3-SAE
    let wpa3_config = WPA3SAEConfig {
        ssid: "MultiOS_Secure".to_string(),
        password: "WPA3Password123!".to_string(),
        group: SAEGroup::Group19,
    };
    
    match security_manager.initialize_wpa3_sae(wpa3_config) {
        Ok(session) => {
            println!("✓ WPA3-SAE session created");
            println!("  SSID: {}", session.ssid);
            println!("  Protocol: {:?}", session.protocol);
            println!("  SAE Group: {:?}", SAEGroup::Group19);
        }
        Err(e) => {
            println!("✗ Failed to create WPA3-SAE session: {}", e);
        }
    }
    
    // Configure security for Enterprise (EAP-TLS)
    let eap_config = EAPTLSConfig {
        identity: "user@company.com".to_string(),
        client_cert: vec![0u8; 1024], // Placeholder
        client_key: vec![0u8; 2048], // Placeholder
        ca_cert: vec![0u8; 512],     // Placeholder
        server_name: "eap.company.com".to_string(),
    };
    
    match security_manager.initialize_eap_tls(eap_config) {
        Ok(session) => {
            println!("✓ EAP-TLS session created");
            println!("  Identity: user@company.com");
            println!("  Protocol: {:?}", session.protocol);
            println!("  Key Management: {:?}", session.key_mgmt_algorithm);
        }
        Err(e) => {
            println!("✗ Failed to create EAP-TLS session: {}", e);
        }
    }
    
    // Security protocol analysis
    println!("\nSecurity Protocol Analysis:");
    let protocols = vec![
        SecurityProtocol::WPA3,
        SecurityProtocol::WPA2,
        SecurityProtocol::WPA,
        SecurityProtocol::WEP,
        SecurityProtocol::Open,
    ];
    
    for protocol in protocols {
        let recommended_encryption = SecurityProtocols::get_recommended_encryption(protocol);
        let is_secure = SecurityProtocols::validate_password_strength("password123", protocol);
        
        println!("  {:?}: {} encryption, Password validation: {}",
                protocol, recommended_encryption, if is_secure { "valid" } else { "invalid" });
    }
    
    // Show all active sessions
    let sessions = security_manager.get_all_sessions();
    println!("\nActive security sessions ({} total):", sessions.len());
    for session in sessions {
        println!("  - {} ({:?}, {}, {} bytes PMK)",
                session.ssid, session.protocol, 
                session.key_mgmt_algorithm, session.pmk.len());
    }
}

//
// Example of Wi-Fi performance monitoring
//
fn demonstrate_performance_monitoring() {
    println!("\nWi-Fi Performance Monitoring Demo");
    println!("==================================");
    
    // Get current statistics
    let manager = get_manager().expect("Failed to get networking manager");
    let wifi_manager = manager.wifi_manager().expect("Failed to get Wi-Fi manager");
    
    let stats = wifi_manager.get_statistics();
    println!("Current Performance Metrics:");
    println!("  Throughput: {} Mbps", stats.current_throughput);
    println!("  Signal Strength: {} dBm", stats.avg_signal_strength);
    println!("  Packets Sent: {}", stats.total_packets_sent);
    println!("  Packets Received: {}", stats.total_packets_received);
    println!("  Failed Packets: {}", stats.failed_packets);
    println!("  Connection Uptime: {} seconds", stats.connection_uptime);
    
    // Performance analysis
    let quality_score = calculate_quality_score(&stats);
    println!("\nQuality Analysis:");
    println!("  Overall Quality Score: {}/100", quality_score);
    
    if stats.avg_signal_strength > -50 {
        println!("  Signal Quality: EXCELLENT");
    } else if stats.avg_signal_strength > -67 {
        println!("  Signal Quality: GOOD");
    } else if stats.avg_signal_strength > -70 {
        println!("  Signal Quality: FAIR");
    } else {
        println!("  Signal Quality: POOR");
    }
    
    // Calculate efficiency
    let theoretical_max = match stats.current_throughput {
        x if x > 1000 => 1200, // Wi-Fi 6
        x if x > 500 => 867,  // Wi-Fi 5
        x if x > 150 => 150,  // Wi-Fi 4
        _ => 54,              // Wi-Fi b/g
    };
    
    let efficiency = (stats.current_throughput as f32 / theoretical_max as f32 * 100.0) as u8;
    println!("  Throughput Efficiency: {}%", efficiency);
    
    // Recommendations
    println!("\nRecommendations:");
    if stats.avg_signal_strength < -70 {
        println!("  • Move closer to access point or remove obstacles");
    }
    if stats.failed_packets > stats.total_packets_received / 10 {
        println!("  • Check for interference or switch to less congested channel");
    }
    if efficiency < 70 {
        println!("  • Consider upgrading to faster Wi-Fi standard if hardware supports");
    }
    if stats.connection_uptime < 3600 {
        println!("  • Check connection stability and power management settings");
    }
}

fn calculate_quality_score(stats: &WifiStatistics) -> u8 {
    let mut score = 0;
    
    // Signal strength contribution (40 points max)
    if stats.avg_signal_strength > -40 {
        score += 40;
    } else if stats.avg_signal_strength > -50 {
        score += 35;
    } else if stats.avg_signal_strength > -60 {
        score += 30;
    } else if stats.avg_signal_strength > -70 {
        score += 20;
    } else {
        score += 10;
    }
    
    // Throughput contribution (30 points max)
    if stats.current_throughput > 500 {
        score += 30;
    } else if stats.current_throughput > 200 {
        score += 25;
    } else if stats.current_throughput > 100 {
        score += 20;
    } else if stats.current_throughput > 50 {
        score += 15;
    } else {
        score += 5;
    }
    
    // Reliability contribution (30 points max)
    if stats.failed_packets == 0 {
        score += 30;
    } else if stats.failed_packets < 100 {
        score += 25;
    } else if stats.failed_packets < 500 {
        score += 20;
    } else if stats.failed_packets < 1000 {
        score += 15;
    } else {
        score += 5;
    }
    
    score
}

//
// Main execution with comprehensive demo
//
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_quality_score_calculation() {
        let stats = WifiStatistics {
            total_packets_sent: 1000,
            total_packets_received: 950,
            failed_packets: 50,
            avg_signal_strength: -45,
            current_throughput: 650,
            connection_uptime: 7200,
        };
        
        let score = calculate_quality_score(&stats);
        assert!(score >= 80); // Should be high quality score
    }
    
    #[test]
    fn test_security_protocol_validation() {
        let test_passwords = vec![
            ("password123", SecurityProtocol::WPA2, true),
            ("short", SecurityProtocol::WPA2, false),
            ("hello", SecurityProtocol::WEP, true),
            ("invalid_len", SecurityProtocol::WEP, false),
            ("12345678", SecurityProtocol::WPS, true),
        ];
        
        for (password, protocol, expected) in test_passwords {
            let is_valid = SecurityProtocols::validate_password_strength(password, protocol);
            assert_eq!(is_valid, expected, "Password validation failed for {:?}", protocol);
        }
    }
}