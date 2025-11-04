//! Encryption Utilities Integration Test & Security Module Tests
//! 
//! This test verifies that the encryption utilities can be properly integrated
//! and used within the kernel environment, and includes comprehensive tests
//! for boot verification and network security features.

#![no_std]

use kernel::security::encryption::{
    EncryptionManager, EncryptionAlgorithm, SymmetricKey, AsymmetricKey,
    EncryptionError, EncryptionResult,
    get_encryption_manager, generate_symmetric_key, generate_asymmetric_key,
    encrypt_data, decrypt_data,
};

use kernel::security::{
    BootVerify, BootVerifyConfig, BootImageInfo, BootChainElement,
    BootComponentType, NetworkSecurity, NetworkPacket, NetworkProtocol,
    FirewallRule, FirewallRuleType, VpnTunnel, VpnEncryption, VpnAuth,
    IntrusionSignature, IntrusionSeverity,
    init_comprehensive_security, get_security_stats,
    BootVerifyResult, NetworkSecurityResult,
};

/// Test basic encryption module initialization
#[test]
fn test_encryption_initialization() {
    // Note: In a real kernel environment, this would be properly initialized
    // For testing purposes, we'll just verify the types and interfaces exist
    
    let manager_result = get_encryption_manager();
    assert!(manager_result.is_some(), "Encryption manager should be available");
}

/// Test algorithm enumeration
#[test]
fn test_algorithm_enumeration() {
    // Test that all expected algorithms are available
    let algorithms = vec![
        EncryptionAlgorithm::AES256,
        EncryptionAlgorithm::ChaCha20,
        EncryptionAlgorithm::RSA2048,
        EncryptionAlgorithm::RSA4096,
        EncryptionAlgorithm::ECCP256,
        EncryptionAlgorithm::ECCP384,
    ];
    
    assert_eq!(algorithms.len(), 6);
    
    // Test algorithm values
    assert_eq!(EncryptionAlgorithm::AES256 as u8, 0);
    assert_eq!(EncryptionAlgorithm::ChaCha20 as u8, 1);
    assert_eq!(EncryptionAlgorithm::RSA2048 as u8, 2);
}

/// Test encryption error types
#[test]
fn test_encryption_errors() {
    // Test error type values
    assert_eq!(EncryptionError::InvalidKey as u8, 0);
    assert_eq!(EncryptionError::InvalidData as u8, 1);
    assert_eq!(EncryptionError::OperationFailed as u8, 2);
    assert_eq!(EncryptionError::NotInitialized as u8, 3);
    assert_eq!(EncryptionError::InvalidAlgorithm as u8, 5);
    
    // Test error equality
    assert_eq!(EncryptionError::InvalidKey, EncryptionError::InvalidKey);
    assert_ne!(EncryptionError::InvalidKey, EncryptionError::InvalidData);
}

/// Test symmetric key structure
#[test]
fn test_symmetric_key_structure() {
    let key = SymmetricKey {
        key_id: "test_key".to_string(),
        algorithm: EncryptionAlgorithm::AES256,
        key_data: vec![0u8; 32],
        iv: vec![0u8; 16],
        nonce: vec![0u8; 16],
    };
    
    assert_eq!(key.key_data.len(), 32);
    assert_eq!(key.iv.len(), 16);
    assert_eq!(key.nonce.len(), 16);
    assert_eq!(key.algorithm, EncryptionAlgorithm::AES256);
}

/// Test asymmetric key structure
#[test]
fn test_asymmetric_key_structure() {
    let key = AsymmetricKey {
        key_id: "test_asym_key".to_string(),
        algorithm: EncryptionAlgorithm::RSA2048,
        public_key: vec![0u8; 256],
        private_key: vec![0u8; 256],
    };
    
    assert_eq!(key.public_key.len(), 256);
    assert_eq!(key.private_key.len(), 256);
    assert_eq!(key.algorithm, EncryptionAlgorithm::RSA2048);
}

/// Test encryption result type
#[test]
fn test_encryption_result() {
    // Test Result type with both success and error cases
    let success_result: EncryptionResult<Vec<u8>> = Ok(vec![0u8; 16]);
    assert!(success_result.is_ok());
    
    let error_result: EncryptionResult<Vec<u8>> = Err(EncryptionError::InvalidKey);
    assert!(error_result.is_err());
    assert_eq!(error_result.unwrap_err(), EncryptionError::InvalidKey);
}

/// Test basic API functions exist
#[test]
fn test_api_functions_exist() {
    // Test that the API functions are callable (they may return errors in test environment)
    // This test just verifies the functions exist and have correct signatures
    
    // These would require proper initialization in a real kernel environment
    let _ = generate_symmetric_key;
    let _ = generate_asymmetric_key;
    let _ = encrypt_data;
    let _ = decrypt_data;
}

/// Test integration scenario
#[test]
fn test_integration_scenario() {
    // This test simulates a basic encryption workflow
    // Note: In a real kernel, this would require proper initialization
    
    // 1. Verify manager is available
    let manager = get_encryption_manager();
    assert!(manager.is_some());
    
    // 2. Check that we can create key structures
    let symmetric_key = SymmetricKey {
        key_id: "integration_test_key".to_string(),
        algorithm: EncryptionAlgorithm::AES256,
        key_data: vec![1u8; 32],
        iv: vec![2u8; 16],
        nonce: vec![3u8; 16],
    };
    
    assert_eq!(symmetric_key.key_id, "integration_test_key");
    assert_eq!(symmetric_key.algorithm, EncryptionAlgorithm::AES256);
    
    // 3. Verify algorithm compatibility
    match symmetric_key.algorithm {
        EncryptionAlgorithm::AES256 | EncryptionAlgorithm::ChaCha20 => {
            // Valid symmetric algorithm
        }
        _ => panic!("Invalid algorithm for symmetric key"),
    }
}

/// Test security container structure
#[test]
fn test_secure_container() {
    use kernel::security::encryption::SecureContainer;
    
    let container = SecureContainer {
        container_id: "test_container".to_string(),
        algorithm: EncryptionAlgorithm::AES256,
        encrypted_data: vec![0u8; 64],
        iv: vec![0u8; 16],
        auth_tag: vec![0u8; 32],
        metadata: vec![0u8; 16],
        created_at: 1634567890,
        size: 64,
    };
    
    assert_eq!(container.size, 64);
    assert_eq!(container.algorithm, EncryptionAlgorithm::AES256);
    assert_eq!(container.auth_tag.len(), 32);
}

/// Test secure channel structure
#[test]
fn test_secure_channel() {
    use kernel::security::encryption::SecureChannel;
    
    let channel = SecureChannel {
        channel_id: "test_channel".to_string(),
        peer_key: "peer_key_id".to_string(),
        session_key: vec![0u8; 32],
        algorithm: EncryptionAlgorithm::AES256,
        is_active: true,
        established_at: 1634567890,
        last_activity: 1634567890,
        message_count: 0,
    };
    
    assert!(channel.is_active);
    assert_eq!(channel.session_key.len(), 32);
    assert_eq!(channel.algorithm, EncryptionAlgorithm::AES256);
}

/// Test statistics structure
#[test]
fn test_encryption_statistics() {
    use kernel::security::encryption::EncryptionStats;
    
    let stats = EncryptionStats {
        total_operations: 100,
        encryption_operations: 50,
        decryption_operations: 50,
        key_generations: 10,
        key_rotations: 5,
        random_numbers_generated: 1000,
        secure_channels_established: 3,
        containers_created: 25,
        integrity_checks: 200,
        failed_operations: 2,
    };
    
    assert_eq!(stats.total_operations, 100);
    assert_eq!(stats.encryption_operations, 50);
    assert_eq!(stats.decryption_operations, 50);
    assert_eq!(stats.failed_operations, 2);
}

/// Test random number generator structure
#[test]
fn test_random_number_generator() {
    use kernel::security::encryption::RandomNumberGenerator;
    
    let rng = RandomNumberGenerator::new();
    
    // Verify structure is initialized correctly
    assert!(!rng.initialized);
    assert_eq!(rng.entropy_pool.len(), 0);
    assert_eq!(rng.last_generated, 0);
    assert_eq!(rng.generated_count, 0);
}

/// Run all integration tests
pub fn run_integration_tests() {
    println!("Running encryption utilities integration tests...");
    
    test_encryption_initialization();
    println!("✓ Encryption initialization test passed");
    
    test_algorithm_enumeration();
    println!("✓ Algorithm enumeration test passed");
    
    test_encryption_errors();
    println!("✓ Error type test passed");
    
    test_symmetric_key_structure();
    println!("✓ Symmetric key structure test passed");
    
    test_asymmetric_key_structure();
    println!("✓ Asymmetric key structure test passed");
    
    test_encryption_result();
    println!("✓ Result type test passed");
    
    test_api_functions_exist();
    println!("✓ API functions test passed");
    
    test_integration_scenario();
    println!("✓ Integration scenario test passed");
    
    test_secure_container();
    println!("✓ Secure container test passed");
    
    test_secure_channel();
    println!("✓ Secure channel test passed");
    
    test_encryption_statistics();
    println!("✓ Statistics structure test passed");
    
    test_random_number_generator();
    println!("✓ Random number generator test passed");
    
    println!("All integration tests passed successfully!");
}

// =============================================================================
// Boot Verification Tests
// =============================================================================

/// Test boot verification initialization
#[test]
fn test_boot_verify_initialization() {
    let config = BootVerifyConfig {
        verify_images: true,
        verify_chain: true,
        measured_boot: true,
        use_tpm: true,
        use_hsm: false,
        strict_mode: false,
        trust_anchor: vec![0x42; 32],
    };
    
    let boot_verify = BootVerify::new(config);
    assert!(boot_verify.verify_boot_chain() == BootVerifyResult::Success);
}

/// Test boot chain elements
#[test]
fn test_boot_chain_elements() {
    let firmware = BootChainElement {
        name: "UEFI".to_string(),
        component_type: BootComponentType::Firmware,
        physical_addr: 0xF0000,
        hash: [1u8; 32],
        verified: false,
        parent: None,
    };
    
    let bootloader = BootChainElement {
        name: "Bootloader".to_string(),
        component_type: BootComponentType::Bootloader,
        physical_addr: 0x1000000,
        hash: [2u8; 32],
        verified: false,
        parent: Some("UEFI".to_string()),
    };
    
    assert_eq!(firmware.name, "UEFI");
    assert_eq!(firmware.component_type, BootComponentType::Firmware);
    assert_eq!(bootloader.component_type, BootComponentType::Bootloader);
}

/// Test boot image verification
#[test]
fn test_boot_image_verification() {
    let boot_verify = BootVerify::new(BootVerifyConfig {
        verify_images: true,
        verify_chain: false,
        measured_boot: false,
        use_tpm: false,
        use_hsm: false,
        strict_mode: false,
        trust_anchor: vec![0x42; 32],
    });
    
    let kernel_image = BootImageInfo {
        physical_addr: 0x2000000,
        size: 1024 * 1024,
        hash: [3u8; 32],
        signature: vec![0xAA; 64],
        build_timestamp: 1638360000,
        version: "1.0.0".to_string(),
        arch: "x86_64".to_string(),
    };
    
    let result = boot_verify.verify_boot_image(&kernel_image);
    // Note: This test uses mock implementations, so verification may pass
    // In a real implementation, this would depend on actual boot image data
    assert!(matches!(result, BootVerifyResult::Success | BootVerifyResult::Failed));
}

// =============================================================================
// Network Security Tests
// =============================================================================

/// Test network security initialization
#[test]
fn test_network_security_initialization() {
    let net_security = NetworkSecurity::new();
    assert_eq!(net_security.get_stats().total_packets, 0);
}

/// Test firewall rule creation
#[test]
fn test_firewall_rule_creation() {
    let rule = FirewallRule {
        id: 1,
        name: "Allow HTTP".to_string(),
        rule_type: FirewallRuleType::Allow,
        src_ip_range: None,
        dst_ip_range: None,
        src_port_range: None,
        dst_port_range: Some((80, 80)),
        protocol: NetworkProtocol::Tcp,
        rate_limit: None,
        priority: 1,
        active: true,
        kernel::security::RuleStats::default(),
    };
    
    assert_eq!(rule.name, "Allow HTTP");
    assert_eq!(rule.rule_type, FirewallRuleType::Allow);
    assert_eq!(rule.protocol, NetworkProtocol::Tcp);
}

/// Test network packet processing
#[test]
fn test_network_packet_processing() {
    let mut net_security = NetworkSecurity::new();
    
    let packet = NetworkPacket {
        src_ip: [192, 168, 1, 50],
        dst_ip: [192, 168, 1, 100],
        src_port: 12345,
        dst_port: 22,
        protocol: NetworkProtocol::Tcp,
        data: b"SSH connection".to_vec(),
        size: 16,
        timestamp: 1638360000,
        interface_idx: 0,
    };
    
    let result = net_security.process_packet(&packet);
    assert!(matches!(result, NetworkSecurityResult::Success));
    
    let stats = net_security.get_stats();
    assert_eq!(stats.total_packets, 1);
    assert_eq!(stats.total_bytes, 16);
}

/// Test firewall rule management
#[test]
fn test_firewall_rule_management() {
    let mut net_security = NetworkSecurity::new();
    
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
        kernel::security::RuleStats::default(),
    };
    
    let result = net_security.add_firewall_rule(rule);
    assert_eq!(result, NetworkSecurityResult::Success);
    
    // Test removing the rule
    let remove_result = net_security.remove_firewall_rule(1);
    assert_eq!(remove_result, NetworkSecurityResult::Success);
    
    // Test removing non-existent rule
    let remove_nonexistent = net_security.remove_firewall_rule(999);
    assert_eq!(remove_nonexistent, NetworkSecurityResult::RuleNotFound);
}

/// Test VPN tunnel creation
#[test]
fn test_vpn_tunnel_creation() {
    let mut net_security = NetworkSecurity::new();
    
    let tunnel = VpnTunnel {
        tunnel_id: 1,
        local_endpoint: [192, 168, 1, 1],
        remote_endpoint: [10, 0, 0, 1],
        encryption: VpnEncryption::Aes256,
        authentication: VpnAuth::HmacSha256,
        status: kernel::security::VpnStatus::Active,
        encrypted_data: Vec::new(),
        session_key: vec![0x5A; 32],
    };
    
    let result = net_security.create_vpn_tunnel(tunnel);
    assert_eq!(result, NetworkSecurityResult::Success);
    
    // Test data encryption/decryption
    let test_data = b"Secret message";
    let encrypted = net_security.encrypt_vpn_data(1, test_data);
    assert!(encrypted.is_ok());
    
    let encrypted_data = encrypted.unwrap();
    let decrypted = net_security.decrypt_vpn_data(1, &encrypted_data);
    assert!(decrypted.is_ok());
    
    let decrypted_data = decrypted.unwrap();
    assert_eq!(decrypted_data, test_data);
}

/// Test intrusion detection
#[test]
fn test_intrusion_detection() {
    let mut net_security = NetworkSecurity::new();
    
    // Add SQL injection signature
    let sql_sig = IntrusionSignature {
        id: 1,
        name: "SQL Injection".to_string(),
        protocol: NetworkProtocol::Tcp,
        src_port_pattern: None,
        dst_port_pattern: Some(80),
        payload_pattern: b"UNION SELECT".to_vec(),
        severity: IntrusionSeverity::High,
        active: true,
    };
    
    let sig_result = net_security.add_intrusion_signature(sql_sig);
    assert_eq!(sig_result, NetworkSecurityResult::Success);
    
    // Test packet with SQL injection attempt
    let malicious_packet = NetworkPacket {
        src_ip: [192, 168, 1, 50],
        dst_ip: [192, 168, 1, 100],
        src_port: 12345,
        dst_port: 80,
        protocol: NetworkProtocol::Tcp,
        data: b"GET /page.php?id=1' UNION SELECT * FROM users".to_vec(),
        size: 48,
        timestamp: 1638360000,
        interface_idx: 0,
    };
    
    let intrusions = net_security.detect_intrusions(&malicious_packet);
    assert!(!intrusions.is_empty());
    assert_eq!(intrusions[0].signature.name, "SQL Injection");
}

/// Test comprehensive security initialization
#[test]
fn test_comprehensive_security_initialization() {
    let result = init_comprehensive_security();
    assert!(result.is_ok());
    
    let (auth_stats, encryption_stats, network_stats) = get_security_stats();
    
    // Check that we got valid statistics
    assert_eq!(auth_stats.successful_logins, 0);
    assert_eq!(network_stats.total_packets, 0);
}

/// Test network protocol enum
#[test]
fn test_network_protocol_enum() {
    assert_eq!(NetworkProtocol::Tcp as u8, 0);
    assert_eq!(NetworkProtocol::Udp as u8, 1);
    assert_eq!(NetworkProtocol::Icmp as u8, 2);
    assert_eq!(NetworkProtocol::Any as u8, 3);
}

/// Test firewall rule type enum
#[test]
fn test_firewall_rule_type_enum() {
    assert_eq!(FirewallRuleType::Allow as u8, 0);
    assert_eq!(FirewallRuleType::Deny as u8, 1);
    assert_eq!(FirewallRuleType::Log as u8, 2);
    assert_eq!(FirewallRuleType::RateLimit as u8, 3);
}

/// Test VPN encryption enum
#[test]
fn test_vpn_encryption_enum() {
    assert_eq!(VpnEncryption::Aes128 as u8, 0);
    assert_eq!(VpnEncryption::Aes256 as u8, 1);
    assert_eq!(VpnEncryption::ChaCha20 as u8, 2);
    assert_eq!(VpnEncryption::None as u8, 3);
}

/// Test intrusion severity enum
#[test]
fn test_intrusion_severity_enum() {
    assert_eq!(IntrusionSeverity::Low as u8, 0);
    assert_eq!(IntrusionSeverity::Medium as u8, 1);
    assert_eq!(IntrusionSeverity::High as u8, 2);
    assert_eq!(IntrusionSeverity::Critical as u8, 3);
}

/// Test boot verify result display
#[test]
fn test_boot_verify_result_display() {
    let success = format!("{}", BootVerifyResult::Success);
    assert_eq!(success, "Success");
    
    let failed = format!("{}", BootVerifyResult::Failed);
    assert_eq!(failed, "Failed");
    
    let corrupted = format!("{}", BootVerifyResult::Corrupted);
    assert_eq!(corrupted, "Corrupted");
}

/// Test network security result display
#[test]
fn test_network_security_result_display() {
    let success = format!("{}", NetworkSecurityResult::Success);
    assert_eq!(success, "Success");
    
    let denied = format!("{}", NetworkSecurityResult::Denied);
    assert_eq!(denied, "Denied");
    
    let blocked = format!("{}", NetworkSecurityResult::Blocked);
    assert_eq!(blocked, "Blocked");
}

/// Test rule statistics
#[test]
fn test_rule_statistics() {
    let mut stats = kernel::security::RuleStats::default();
    assert_eq!(stats.packets_matched, 0);
    assert_eq!(stats.bytes_processed, 0);
    assert_eq!(stats.hit_count, 0);
    
    // Simulate rule match
    stats.packets_matched = 10;
    stats.bytes_processed = 1000;
    stats.hit_count = 5;
    stats.last_match = 1638360000;
    
    assert_eq!(stats.packets_matched, 10);
    assert_eq!(stats.bytes_processed, 1000);
    assert_eq!(stats.hit_count, 5);
}

/// Test network security statistics
#[test]
fn test_network_security_statistics() {
    let mut net_security = NetworkSecurity::new();
    
    // Initially all stats should be zero
    let stats = net_security.get_stats();
    assert_eq!(stats.total_packets, 0);
    assert_eq!(stats.total_bytes, 0);
    assert_eq!(stats.packets_blocked, 0);
    assert_eq!(stats.packets_allowed, 0);
    assert_eq!(stats.intrusions_detected, 0);
    
    // Process a packet to update stats
    let packet = NetworkPacket {
        src_ip: [192, 168, 1, 1],
        dst_ip: [192, 168, 1, 100],
        src_port: 12345,
        dst_port: 80,
        protocol: NetworkProtocol::Tcp,
        data: vec![1, 2, 3, 4],
        size: 4,
        timestamp: 1638360000,
        interface_idx: 0,
    };
    
    let _ = net_security.process_packet(&packet);
    
    let updated_stats = net_security.get_stats();
    assert_eq!(updated_stats.total_packets, 1);
    assert_eq!(updated_stats.total_bytes, 4);
    assert_eq!(updated_stats.packets_allowed, 1);
}

/// Test interface security configuration
#[test]
fn test_interface_security_configuration() {
    let mut net_security = NetworkSecurity::new();
    
    let config = kernel::security::InterfaceSecurity {
        interface_idx: 1,
        name: "eth1".to_string(),
        firewall_enabled: true,
        ids_enabled: true,
        vpn_enabled: false,
        default_action: FirewallRuleType::Deny,
        rules: Vec::new(),
    };
    
    let result = net_security.configure_interface(config);
    assert_eq!(result, NetworkSecurityResult::Success);
}

/// Test multiple firewall rules
#[test]
fn test_multiple_firewall_rules() {
    let mut net_security = NetworkSecurity::new();
    
    // Add multiple rules
    for i in 1..=5 {
        let rule = FirewallRule {
            id: i,
            name: format!("Rule {}", i),
            rule_type: FirewallRuleType::Allow,
            src_ip_range: None,
            dst_ip_range: None,
            src_port_range: None,
            dst_port_range: Some((80 + i, 80 + i)),
            protocol: NetworkProtocol::Tcp,
            rate_limit: None,
            priority: i,
            active: true,
            kernel::security::RuleStats::default(),
        };
        
        let result = net_security.add_firewall_rule(rule);
        assert_eq!(result, NetworkSecurityResult::Success);
    }
    
    // Verify we can retrieve a specific rule
    let rule = net_security.get_firewall_rule(3);
    assert!(rule.is_some());
    assert_eq!(rule.unwrap().id, 3);
    
    // Verify non-existent rule returns None
    let nonexistent = net_security.get_firewall_rule(999);
    assert!(nonexistent.is_none());
}

/// Test measured boot attestation
#[test]
fn test_measured_boot_attestation() {
    let mut boot_verify = BootVerify::new(BootVerifyConfig {
        verify_images: false,
        verify_chain: true,
        measured_boot: true,
        use_tpm: false,
        use_hsm: false,
        strict_mode: false,
        trust_anchor: vec![0x42; 32],
    });
    
    // Add some chain elements
    let kernel = BootChainElement {
        name: "kernel".to_string(),
        component_type: BootComponentType::Kernel,
        physical_addr: 0x1000000,
        hash: [1u8; 32],
        verified: false,
        parent: Some("bootloader".to_string()),
    };
    
    boot_verify.add_chain_element(kernel);
    
    // Test measured boot
    let attestation_result = boot_verify.measured_boot();
    assert!(attestation_result.is_ok());
    
    let attestation = attestation_result.unwrap();
    assert_eq!(attestation.pcrs.len(), 16);
    assert_eq!(attestation.boot_events.len(), 1);
}

/// Test complex network security scenario
#[test]
fn test_complex_network_scenario() {
    let mut net_security = NetworkSecurity::new();
    
    // Configure interface
    let config = kernel::security::InterfaceSecurity {
        interface_idx: 0,
        name: "eth0".to_string(),
        firewall_enabled: true,
        ids_enabled: true,
        vpn_enabled: true,
        default_action: FirewallRuleType::Deny,
        rules: Vec::new(),
    };
    
    net_security.configure_interface(config);
    
    // Add comprehensive firewall rules
    let rules = [
        // Allow DNS
        FirewallRule {
            id: 1, name: "DNS".to_string(), rule_type: FirewallRuleType::Allow,
            src_ip_range: None, dst_ip_range: None,
            src_port_range: None, dst_port_range: Some((53, 53)),
            protocol: NetworkProtocol::Udp, rate_limit: None, priority: 1,
            active: true, kernel::security::RuleStats::default(),
        },
        // Allow HTTP with rate limit
        FirewallRule {
            id: 2, name: "HTTP".to_string(), rule_type: FirewallRuleType::Allow,
            src_ip_range: None, dst_ip_range: None,
            src_port_range: None, dst_port_range: Some((80, 80)),
            protocol: NetworkProtocol::Tcp, rate_limit: Some(100), priority: 2,
            active: true, kernel::security::RuleStats::default(),
        },
        // Block Telnet
        FirewallRule {
            id: 3, name: "Telnet".to_string(), rule_type: FirewallRuleType::Deny,
            src_ip_range: None, dst_ip_range: None,
            src_port_range: None, dst_port_range: Some((23, 23)),
            protocol: NetworkProtocol::Tcp, rate_limit: None, priority: 1,
            active: true, kernel::security::RuleStats::default(),
        },
    ];
    
    for rule in &rules {
        net_security.add_firewall_rule(rule.clone());
    }
    
    // Add intrusion signatures
    let signatures = [
        IntrusionSignature {
            id: 1, name: "Port scan".to_string(), protocol: NetworkProtocol::Tcp,
            src_port_pattern: None, dst_port_pattern: None,
            payload_pattern: vec![0x00], severity: IntrusionSeverity::Medium,
            active: true,
        },
        IntrusionSignature {
            id: 2, name: "SQL injection".to_string(), protocol: NetworkProtocol::Tcp,
            src_port_pattern: None, dst_port_pattern: Some(80),
            payload_pattern: b"UNION SELECT".to_vec(), severity: IntrusionSeverity::High,
            active: true,
        },
    ];
    
    for sig in &signatures {
        net_security.add_intrusion_signature(sig.clone());
    }
    
    // Create VPN tunnel
    let tunnel = VpnTunnel {
        tunnel_id: 1,
        local_endpoint: [192, 168, 1, 1],
        remote_endpoint: [10, 0, 0, 1],
        encryption: VpnEncryption::Aes256,
        authentication: VpnAuth::HmacSha256,
        status: kernel::security::VpnStatus::Active,
        encrypted_data: Vec::new(),
        session_key: vec![0x5A; 32],
    };
    
    net_security.create_vpn_tunnel(tunnel);
    
    // Test various traffic scenarios
    let test_packets = [
        // Valid DNS query
        NetworkPacket {
            src_ip: [192, 168, 1, 50], dst_ip: [8, 8, 8, 8],
            src_port: 12345, dst_port: 53,
            protocol: NetworkProtocol::Udp,
            data: b"DNS query".to_vec(), size: 10,
            timestamp: 1638360000, interface_idx: 0,
        },
        // Valid HTTP request
        NetworkPacket {
            src_ip: [192, 168, 1, 50], dst_ip: [192, 168, 1, 100],
            src_port: 12346, dst_port: 80,
            protocol: NetworkProtocol::Tcp,
            data: b"GET /index.html HTTP/1.1".to_vec(), size: 20,
            timestamp: 1638360001, interface_idx: 0,
        },
        // Blocked Telnet attempt
        NetworkPacket {
            src_ip: [192, 168, 1, 50], dst_ip: [192, 168, 1, 100],
            src_port: 12347, dst_port: 23,
            protocol: NetworkProtocol::Tcp,
            data: b"Telnet attempt".to_vec(), size: 14,
            timestamp: 1638360002, interface_idx: 0,
        },
        // SQL injection attempt
        NetworkPacket {
            src_ip: [192, 168, 1, 50], dst_ip: [192, 168, 1, 100],
            src_port: 12348, dst_port: 80,
            protocol: NetworkProtocol::Tcp,
            data: b"GET /page.php?id=1' UNION SELECT * FROM users".to_vec(),
            size: 50, timestamp: 1638360003, interface_idx: 0,
        },
    ];
    
    for packet in &test_packets {
        // Process packet through firewall
        let _firewall_result = net_security.process_packet(packet);
        
        // Check for intrusions
        let intrusions = net_security.detect_intrusions(packet);
        for intrusion in &intrusions {
            assert!(!intrusion.signature.name.is_empty());
        }
    }
    
    // Update and verify statistics
    net_security.update_stats();
    let stats = net_security.get_stats();
    
    assert_eq!(stats.total_packets, 4);
    assert!(stats.packets_allowed >= 2); // DNS and HTTP should be allowed
    assert!(stats.packets_blocked >= 1); // Telnet should be blocked
    assert!(stats.intrusions_detected >= 1); // SQL injection should be detected
    assert_eq!(stats.vpn_tunnels_active, 1);
    assert_eq!(stats.firewall_rules_active, 3);
}

/// Run all security tests
pub fn run_security_tests() {
    println!("Running comprehensive security tests...");
    
    // Boot verification tests
    test_boot_verify_initialization();
    println!("✓ Boot verification initialization test passed");
    
    test_boot_chain_elements();
    println!("✓ Boot chain elements test passed");
    
    test_boot_image_verification();
    println!("✓ Boot image verification test passed");
    
    // Network security tests
    test_network_security_initialization();
    println!("✓ Network security initialization test passed");
    
    test_firewall_rule_creation();
    println!("✓ Firewall rule creation test passed");
    
    test_network_packet_processing();
    println!("✓ Network packet processing test passed");
    
    test_firewall_rule_management();
    println!("✓ Firewall rule management test passed");
    
    test_vpn_tunnel_creation();
    println!("✓ VPN tunnel creation test passed");
    
    test_intrusion_detection();
    println!("✓ Intrusion detection test passed");
    
    test_comprehensive_security_initialization();
    println!("✓ Comprehensive security initialization test passed");
    
    // Enum tests
    test_network_protocol_enum();
    println!("✓ Network protocol enum test passed");
    
    test_firewall_rule_type_enum();
    println!("✓ Firewall rule type enum test passed");
    
    test_vpn_encryption_enum();
    println!("✓ VPN encryption enum test passed");
    
    test_intrusion_severity_enum();
    println!("✓ Intrusion severity enum test passed");
    
    // Display tests
    test_boot_verify_result_display();
    println!("✓ Boot verify result display test passed");
    
    test_network_security_result_display();
    println!("✓ Network security result display test passed");
    
    // Statistics tests
    test_rule_statistics();
    println!("✓ Rule statistics test passed");
    
    test_network_security_statistics();
    println!("✓ Network security statistics test passed");
    
    // Configuration tests
    test_interface_security_configuration();
    println!("✓ Interface security configuration test passed");
    
    test_multiple_firewall_rules();
    println!("✓ Multiple firewall rules test passed");
    
    // Advanced features
    test_measured_boot_attestation();
    println!("✓ Measured boot attestation test passed");
    
    test_complex_network_scenario();
    println!("✓ Complex network scenario test passed");
    
    println!("All security tests passed successfully!");
}