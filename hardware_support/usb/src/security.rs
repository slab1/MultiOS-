//! USB Security Isolation Module
//! 
//! This module provides comprehensive security isolation for USB devices,
//! including device fingerprinting, permission management, attack detection,
//! and security policy enforcement.

use core::fmt;
use alloc::vec::Vec;
use alloc::string::String;
use crate::UsbResult;

/// Security isolation levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecurityLevel {
    /// No security restrictions - full access
    None,
    /// Basic device verification required
    Basic,
    /// Device fingerprinting and permission checks
    Medium,
    /// Strict isolation with comprehensive monitoring
    High,
    /// Maximum security - only explicitly trusted devices
    Maximum,
}

impl fmt::Display for SecurityLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SecurityLevel::None => write!(f, "None"),
            SecurityLevel::Basic => write!(f, "Basic"),
            SecurityLevel::Medium => write!(f, "Medium"),
            SecurityLevel::High => write!(f, "High"),
            SecurityLevel::Maximum => write!(f, "Maximum"),
        }
    }
}

/// Device trust state
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrustState {
    /// Device is explicitly trusted
    Trusted,
    /// Device has been verified but not explicitly trusted
    Verified,
    /// Device has unknown status
    Unknown,
    /// Device is blocked
    Blocked,
    /// Device failed security checks
    Failed,
}

/// USB device fingerprint for security purposes
#[derive(Debug, Clone)]
pub struct DeviceFingerprint {
    /// USB vendor ID
    pub vendor_id: u16,
    /// USB product ID
    pub product_id: u16,
    /// Device serial number (if available)
    pub serial: Option<String>,
    /// Device manufacturer string
    pub manufacturer: Option<String>,
    /// Device product string
    pub product: Option<String>,
    /// Device class, subclass, protocol
    pub class: (u8, u8, u8),
    /// Device configuration hash
    pub config_hash: u64,
    /// Additional device-specific data
    pub custom_data: Vec<u8>,
}

impl DeviceFingerprint {
    /// Create a new device fingerprint
    pub fn new(
        vendor_id: u16,
        product_id: u16,
        class: (u8, u8, u8),
    ) -> Self {
        Self {
            vendor_id,
            product_id,
            serial: None,
            manufacturer: None,
            product: None,
            class,
            config_hash: 0,
            custom_data: Vec::new(),
        }
    }

    /// Calculate a unique hash for this device fingerprint
    pub fn hash(&self) -> u64 {
        let mut hash = 0x5bd1e995u64;
        
        hash = hash.wrapping_mul(0x85ebca6b);
        hash ^= (self.vendor_id as u64) & 0xFFFF;
        hash = hash.wrapping_mul(0x85ebca6b);
        
        hash = hash.wrapping_mul(0x85ebca6b);
        hash ^= (self.product_id as u64) & 0xFFFF;
        hash = hash.wrapping_mul(0x85ebca6b);
        
        hash = hash.wrapping_mul(0x85ebca6b);
        hash ^= self.class.0 as u64;
        hash = hash.wrapping_mul(0x85ebca6b);
        
        hash = hash.wrapping_mul(0x85ebca6b);
        hash ^= self.class.1 as u64;
        hash = hash.wrapping_mul(0x85ebca6b);
        
        hash = hash.wrapping_mul(0x85ebca6b);
        hash ^= self.class.2 as u64;
        hash = hash.wrapping_mul(0x85ebca6b);
        
        // Add serial number if present
        if let Some(ref serial) = self.serial {
            for &byte in serial.as_bytes() {
                hash = hash.wrapping_mul(0x85ebca6b);
                hash ^= byte as u64;
            }
        }
        
        // Add custom data
        for &byte in &self.custom_data {
            hash = hash.wrapping_mul(0x85ebca6b);
            hash ^= byte as u64;
        }
        
        hash
    }
}

/// Security policy for a device or device class
#[derive(Debug, Clone)]
pub struct SecurityPolicy {
    /// Policy name
    pub name: String,
    /// Allowed devices (empty = all)
    pub allowed_devices: Vec<DeviceFingerprint>,
    /// Blocked devices (empty = none)
    pub blocked_devices: Vec<DeviceFingerprint>,
    /// Allowed device classes (empty = all)
    pub allowed_classes: Vec<(u8, u8, u8)>,
    /// Blocked device classes (empty = none)
    pub blocked_classes: Vec<(u8, u8, u8)>,
    /// Minimum security level required
    pub min_security_level: SecurityLevel,
    /// Allow USB mass storage
    pub allow_storage: bool,
    /// Allow USB network devices
    pub allow_network: bool,
    /// Allow USB audio devices
    pub allow_audio: bool,
    /// Maximum power consumption (mA)
    pub max_power_ma: u16,
    /// Enable device monitoring
    pub enable_monitoring: bool,
    /// Require device verification
    pub require_verification: bool,
}

impl SecurityPolicy {
    /// Create a default security policy
    pub fn default() -> Self {
        Self {
            name: "Default Policy".to_string(),
            allowed_devices: Vec::new(),
            blocked_devices: Vec::new(),
            allowed_classes: Vec::new(),
            blocked_classes: Vec::new(),
            min_security_level: SecurityLevel::Basic,
            allow_storage: true,
            allow_network: true,
            allow_audio: true,
            max_power_ma: 500, // Standard USB 2.0 power limit
            enable_monitoring: true,
            require_verification: true,
        }
    }

    /// Check if a device matches this policy
    pub fn allows_device(&self, fingerprint: &DeviceFingerprint) -> bool {
        // Check blocked devices first
        for blocked in &self.blocked_devices {
            if self.devices_match(fingerprint, blocked) {
                return false;
            }
        }

        // Check if we have explicit allowed devices
        if !self.allowed_devices.is_empty() {
            for allowed in &self.allowed_devices {
                if self.devices_match(fingerprint, allowed) {
                    return true;
                }
            }
            return false;
        }

        // Check blocked classes
        for blocked_class in &self.blocked_classes {
            if fingerprint.class == *blocked_class {
                return false;
            }
        }

        // Check allowed classes
        if !self.allowed_classes.is_empty() {
            for allowed_class in &self.allowed_classes {
                if fingerprint.class == *allowed_class {
                    break;
                }
            }
            return false;
        }

        true
    }

    /// Check if two device fingerprints match
    fn devices_match(&self, device1: &DeviceFingerprint, device2: &DeviceFingerprint) -> bool {
        if device1.vendor_id != device2.vendor_id {
            return false;
        }

        if device1.product_id != device2.product_id {
            return false;
        }

        // If both have serial numbers, they must match
        match (device1.serial.as_ref(), device2.serial.as_ref()) {
            (Some(s1), Some(s2)) => s1 == s2,
            (None, None) => true,  // Both don't have serials
            _ => false,            // One has serial, other doesn't
        }
    }
}

/// Security event types
#[derive(Debug, Clone)]
pub enum SecurityEvent {
    /// Device connected
    DeviceConnected { fingerprint: DeviceFingerprint },
    /// Device disconnected
    DeviceDisconnected { fingerprint: DeviceFingerprint },
    /// Device access denied
    AccessDenied { fingerprint: DeviceFingerprint, reason: String },
    /// Device verification failed
    VerificationFailed { fingerprint: DeviceFingerprint, reason: String },
    /// Security policy violation
    PolicyViolation { fingerprint: DeviceFingerprint, violation: String },
    /// Device monitoring alert
    MonitoringAlert { fingerprint: DeviceFingerprint, alert: String },
    /// Unknown device type
    UnknownDevice { fingerprint: DeviceFingerprint },
    /// Suspicious device behavior
    SuspiciousBehavior { fingerprint: DeviceFingerprint, behavior: String },
}

/// Security event handler
pub trait SecurityEventHandler: Send + Sync {
    /// Handle a security event
    fn handle_event(&self, event: &SecurityEvent);
}

/// USB security manager
pub struct SecurityManager {
    /// Current security level
    level: SecurityLevel,
    /// Security policies
    policies: Vec<SecurityPolicy>,
    /// Device trust cache
    device_cache: Vec<(DeviceFingerprint, TrustState)>,
    /// Security event handlers
    event_handlers: Vec<Box<dyn SecurityEventHandler>>,
    /// Device monitoring enabled
    monitoring_enabled: bool,
    /// Security audit log
    audit_log: Vec<SecurityEvent>,
    /// Maximum audit log entries
    max_audit_entries: usize,
}

impl SecurityManager {
    /// Create a new security manager
    pub fn new(level: SecurityLevel) -> Self {
        let mut manager = Self {
            level,
            policies: Vec::new(),
            device_cache: Vec::new(),
            event_handlers: Vec::new(),
            monitoring_enabled: true,
            audit_log: Vec::new(),
            max_audit_entries: 1000,
        };

        // Add default policy
        manager.add_policy(SecurityPolicy::default());
        
        manager
    }

    /// Set the security level
    pub fn set_security_level(&mut self, level: SecurityLevel) {
        self.level = level;
    }

    /// Get the current security level
    pub fn get_security_level(&self) -> SecurityLevel {
        self.level
    }

    /// Add a security policy
    pub fn add_policy(&mut self, policy: SecurityPolicy) {
        self.policies.push(policy);
    }

    /// Remove a security policy by name
    pub fn remove_policy(&mut self, name: &str) {
        self.policies.retain(|p| p.name != name);
    }

    /// Register a security event handler
    pub fn register_event_handler(&mut self, handler: Box<dyn SecurityEventHandler>) {
        self.event_handlers.push(handler);
    }

    /// Set device monitoring enabled/disabled
    pub fn set_monitoring_enabled(&mut self, enabled: bool) {
        self.monitoring_enabled = enabled;
    }

    /// Check if a device is allowed to connect
    pub fn check_device_access(&mut self, fingerprint: &DeviceFingerprint) -> UsbResult<TrustState> {
        // Check cache first
        for (cached_fp, trust_state) in &self.device_cache {
            if self.devices_match(fingerprint, cached_fp) {
                self.log_event(SecurityEvent::DeviceConnected { 
                    fingerprint: fingerprint.clone() 
                });
                return Ok(*trust_state);
            }
        }

        // Apply security policies
        let mut policy_match = false;
        let mut access_allowed = false;

        for policy in &self.policies {
            if policy.allows_device(fingerprint) {
                policy_match = true;
                access_allowed = true;
                break;
            }
        }

        if !policy_match {
            // Check default policy
            for policy in &self.policies {
                if policy.name == "Default Policy" {
                    if policy.allows_device(fingerprint) {
                        access_allowed = true;
                        break;
                    }
                }
            }
        }

        let trust_state = if access_allowed {
            if self.level >= SecurityLevel::High {
                TrustState::Verified
            } else {
                TrustState::Trusted
            }
        } else {
            TrustState::Blocked
        };

        // Add to cache
        self.device_cache.push((fingerprint.clone(), trust_state));

        if !access_allowed {
            self.log_event(SecurityEvent::AccessDenied { 
                fingerprint: fingerprint.clone(),
                reason: "Device not allowed by security policy".to_string()
            });
        } else {
            self.log_event(SecurityEvent::DeviceConnected { 
                fingerprint: fingerprint.clone() 
            });
        }

        Ok(trust_state)
    }

    /// Mark a device as trusted
    pub fn trust_device(&mut self, fingerprint: &DeviceFingerprint) {
        // Update cache
        for (cached_fp, trust_state) in &mut self.device_cache {
            if self.devices_match(fingerprint, cached_fp) {
                *trust_state = TrustState::Trusted;
                break;
            }
        }

        // Add to cache if not present
        self.device_cache.push((fingerprint.clone(), TrustState::Trusted));

        self.log_event(SecurityEvent::DeviceConnected { 
            fingerprint: fingerprint.clone() 
        });
    }

    /// Block a device
    pub fn block_device(&mut self, fingerprint: &DeviceFingerprint) {
        // Update cache
        for (cached_fp, trust_state) in &mut self.device_cache {
            if self.devices_match(fingerprint, cached_fp) {
                *trust_state = TrustState::Blocked;
                break;
            }
        }

        // Add to cache if not present
        self.device_cache.push((fingerprint.clone(), TrustState::Blocked));

        self.log_event(SecurityEvent::AccessDenied {
            fingerprint: fingerprint.clone(),
            reason: "Device manually blocked".to_string()
        });
    }

    /// Verify a device fingerprint
    pub fn verify_device(&mut self, fingerprint: &DeviceFingerprint) -> bool {
        if self.level == SecurityLevel::None {
            return true;
        }

        // Check for basic verification requirements
        if fingerprint.vendor_id == 0 || fingerprint.product_id == 0 {
            self.log_event(SecurityEvent::VerificationFailed {
                fingerprint: fingerprint.clone(),
                reason: "Invalid vendor/product ID".to_string()
            });
            return false;
        }

        // Check for suspicious characteristics
        if self.level >= SecurityLevel::High {
            // Check for uncommon class codes
            if fingerprint.class.0 != 0 && 
               fingerprint.class.0 != 0x03 &&  // HID
               fingerprint.class.0 != 0x08 &&  // Mass storage
               fingerprint.class.0 != 0x0E {   // Video
                
                if !self.is_allowed_vendor(fingerprint.vendor_id) {
                    self.log_event(SecurityEvent::VerificationFailed {
                        fingerprint: fingerprint.clone(),
                        reason: "Unrecognized vendor/class combination".to_string()
                    });
                    return false;
                }
            }
        }

        self.log_event(SecurityEvent::DeviceConnected { 
            fingerprint: fingerprint.clone() 
        });
        true
    }

    /// Monitor device behavior
    pub fn monitor_device(&mut self, fingerprint: &DeviceFingerprint, behavior: &str) {
        if !self.monitoring_enabled {
            return;
        }

        // Check for suspicious behaviors
        let suspicious_keywords = ["malware", "virus", "rootkit", "backdoor", "exploit"];
        
        for keyword in suspicious_keywords {
            if behavior.to_lowercase().contains(keyword) {
                self.log_event(SecurityEvent::SuspiciousBehavior {
                    fingerprint: fingerprint.clone(),
                    behavior: behavior.to_string()
                });
                break;
            }
        }

        self.log_event(SecurityEvent::MonitoringAlert {
            fingerprint: fingerprint.clone(),
            alert: behavior.to_string()
        });
    }

    /// Check if vendor ID is allowed
    fn is_allowed_vendor(&self, vendor_id: u16) -> bool {
        // List of common, trusted USB vendors
        let trusted_vendors = [
            0x046D,  // Logitech
            0x045E,  // Microsoft
            0x04B4,  // Cypress
            0x0B27,  // SanDisk
            0x058F,  // Alcor
            0x0951,  // Kingston
            0x1E3D,  // Generic vendor for testing
        ];

        trusted_vendors.contains(&vendor_id)
    }

    /// Check if two device fingerprints match
    fn devices_match(&self, device1: &DeviceFingerprint, device2: &DeviceFingerprint) -> bool {
        if device1.vendor_id != device2.vendor_id {
            return false;
        }

        if device1.product_id != device2.product_id {
            return false;
        }

        // If both have serial numbers, they must match
        match (device1.serial.as_ref(), device2.serial.as_ref()) {
            (Some(s1), Some(s2)) => s1 == s2,
            (None, None) => true,  // Both don't have serials
            _ => false,            // One has serial, other doesn't
        }
    }

    /// Log a security event
    fn log_event(&mut self, event: SecurityEvent) {
        // Add event to audit log
        self.audit_log.push(event.clone());

        // Limit audit log size
        if self.audit_log.len() > self.max_audit_entries {
            self.audit_log.remove(0);
        }

        // Notify event handlers
        for handler in &self.event_handlers {
            handler.handle_event(&event);
        }
    }

    /// Get audit log
    pub fn get_audit_log(&self) -> &[SecurityEvent] {
        &self.audit_log
    }

    /// Clear audit log
    pub fn clear_audit_log(&mut self) {
        self.audit_log.clear();
    }

    /// Get trusted devices
    pub fn get_trusted_devices(&self) -> Vec<&DeviceFingerprint> {
        self.device_cache
            .iter()
            .filter(|(_, trust_state)| **trust_state == TrustState::Trusted)
            .map(|(fingerprint, _)| fingerprint)
            .collect()
    }

    /// Get blocked devices
    pub fn get_blocked_devices(&self) -> Vec<&DeviceFingerprint> {
        self.device_cache
            .iter()
            .filter(|(_, trust_state)| **trust_state == TrustState::Blocked)
            .map(|(fingerprint, _)| fingerprint)
            .collect()
    }

    /// Generate security report
    pub fn generate_security_report(&self) -> String {
        let mut report = String::new();
        report.push_str("USB Security Report\n");
        report.push_str("===================\n\n");
        
        report.push_str(&format!("Security Level: {}\n", self.level));
        report.push_str(&format!("Device Cache Size: {}\n", self.device_cache.len()));
        report.push_str(&format!("Security Policies: {}\n", self.policies.len()));
        report.push_str(&format!("Monitoring Enabled: {}\n\n", self.monitoring_enabled));
        
        report.push_str("Device Statistics:\n");
        report.push_str("------------------\n");
        
        let trusted = self.device_cache.iter().filter(|(_, state)| **state == TrustState::Trusted).count();
        let verified = self.device_cache.iter().filter(|(_, state)| **state == TrustState::Verified).count();
        let unknown = self.device_cache.iter().filter(|(_, state)| **state == TrustState::Unknown).count();
        let blocked = self.device_cache.iter().filter(|(|_, state)| **state == TrustState::Blocked).count();
        let failed = self.device_cache.iter().filter(|(_, state)| **state == TrustState::Failed).count();
        
        report.push_str(&format!("Trusted: {}\n", trusted));
        report.push_str(&format!("Verified: {}\n", verified));
        report.push_str(&format!("Unknown: {}\n", unknown));
        report.push_str(&format!("Blocked: {}\n", blocked));
        report.push_str(&format!("Failed: {}\n\n", failed));
        
        if !self.audit_log.is_empty() {
            report.push_str(&format!("Recent Events: {} entries\n", self.audit_log.len()));
            
            let recent_events = &self.audit_log[self.audit_log.len().saturating_sub(10)..];
            for event in recent_events {
                report.push_str(&format!("- {:?}\n", event));
            }
        }
        
        report
    }

    /// Export security configuration
    pub fn export_config(&self) -> String {
        let mut config = String::new();
        config.push_str(&format!("security_level = {}\n", self.level));
        config.push_str(&format!("monitoring_enabled = {}\n", self.monitoring_enabled));
        config.push_str(&format!("max_audit_entries = {}\n", self.max_audit_entries));
        config.push_str(&format!("num_policies = {}\n", self.policies.len()));
        config.push_str(&format!("num_device_cache = {}\n", self.device_cache.len()));
        config
    }

    /// Import security configuration
    pub fn import_config(&mut self, config: &str) -> UsbResult<()> {
        for line in config.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim();

                match key {
                    "security_level" => {
                        self.level = match value {
                            "None" => SecurityLevel::None,
                            "Basic" => SecurityLevel::Basic,
                            "Medium" => SecurityLevel::Medium,
                            "High" => SecurityLevel::High,
                            "Maximum" => SecurityLevel::Maximum,
                            _ => return Err(UsbResult::InvalidArgument),
                        };
                    }
                    "monitoring_enabled" => {
                        self.monitoring_enabled = value == "true";
                    }
                    "max_audit_entries" => {
                        self.max_audit_entries = value.parse()
                            .unwrap_or(1000);
                    }
                    _ => {
                        // Ignore unknown config keys
                    }
                }
            }
        }
        
        Ok(())
    }
}

/// Default security event handler for printing events
pub struct DefaultSecurityHandler;

impl SecurityEventHandler for DefaultSecurityHandler {
    fn handle_event(&self, event: &SecurityEvent) {
        match event {
            SecurityEvent::DeviceConnected { fingerprint } => {
                println!("USB Security: Device connected - VID:{:04X}, PID:{:04X}", 
                    fingerprint.vendor_id, fingerprint.product_id);
            }
            SecurityEvent::DeviceDisconnected { fingerprint } => {
                println!("USB Security: Device disconnected - VID:{:04X}, PID:{:04X}", 
                    fingerprint.vendor_id, fingerprint.product_id);
            }
            SecurityEvent::AccessDenied { fingerprint, reason } => {
                println!("USB Security: Access denied - VID:{:04X}, PID:{:04X} - {}", 
                    fingerprint.vendor_id, fingerprint.product_id, reason);
            }
            SecurityEvent::VerificationFailed { fingerprint, reason } => {
                println!("USB Security: Verification failed - VID:{:04X}, PID:{:04X} - {}", 
                    fingerprint.vendor_id, fingerprint.product_id, reason);
            }
            SecurityEvent::PolicyViolation { fingerprint, violation } => {
                println!("USB Security: Policy violation - VID:{:04X}, PID:{:04X} - {}", 
                    fingerprint.vendor_id, fingerprint.product_id, violation);
            }
            SecurityEvent::MonitoringAlert { fingerprint, alert } => {
                println!("USB Security: Monitoring alert - VID:{:04X}, PID:{:04X} - {}", 
                    fingerprint.vendor_id, fingerprint.product_id, alert);
            }
            SecurityEvent::UnknownDevice { fingerprint } => {
                println!("USB Security: Unknown device type - VID:{:04X}, PID:{:04X}", 
                    fingerprint.vendor_id, fingerprint.product_id);
            }
            SecurityEvent::SuspiciousBehavior { fingerprint, behavior } => {
                println!("USB Security: Suspicious behavior - VID:{:04X}, PID:{:04X} - {}", 
                    fingerprint.vendor_id, fingerprint.product_id, behavior);
            }
        }
    }
}

/// Simple in-memory security event handler
pub struct MemorySecurityHandler {
    events: Vec<SecurityEvent>,
}

impl MemorySecurityHandler {
    /// Create a new memory security handler
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
        }
    }

    /// Get all captured events
    pub fn get_events(&self) -> &[SecurityEvent] {
        &self.events
    }

    /// Clear all events
    pub fn clear(&mut self) {
        self.events.clear();
    }
}

impl SecurityEventHandler for MemorySecurityHandler {
    fn handle_event(&self, event: &SecurityEvent) {
        // Store event (this would need &mut self in real implementation)
        // For now, just print the event
        println!("Security Event: {:?}", event);
    }
}

impl Default for MemorySecurityHandler {
    fn default() -> Self {
        Self::new()
    }
}