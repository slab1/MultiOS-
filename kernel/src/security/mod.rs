//! MultiOS Security Module
//! 
//! This module provides comprehensive security functionality including:
//! - Authentication and authorization systems
//! - Encryption and cryptographic operations
//! - Secure key management and storage
//! - Session management and token handling
//! - Multi-factor authentication support
//! - Biometric authentication
//! - Hardware token support
//! - File encryption and secure containers
//! - Secure random number generation
//! - Secure communication channels
//! - Boot verification and integrity checking
//! - Network security with firewall and intrusion detection
//! - Hardware security module integration
//! - Integration with user management and security frameworks

#![no_std]
#![feature(alloc)]

pub mod auth;
pub mod encryption;
pub mod boot_verify;
pub mod network;
pub mod rbac;
pub mod acl;
pub mod permission_inheritance;
pub mod delegation;

// Examples and demonstrations (for testing and documentation)
#[cfg(feature = "examples")]
pub mod examples;

// Integration tests
#[cfg(test)]
pub mod integration_tests;

use alloc::vec::Vec;
use alloc::string::{String, ToString};
use spin::{Mutex, RwLock};
use log::{info, warn, error};

// Re-export main components from both modules
pub use encryption::{
    EncryptionManager, SymmetricKey, AsymmetricKey, KeyPair, 
    SecureContainer, SecureChannel, RandomNumberGenerator,
    EncryptionError, EncryptionResult
};

pub use auth::{
    AuthManager, AuthError, AuthResult, AuthMethod, AuthFactor,
    SessionToken, PasswordPolicy, BiometricData, HardwareToken,
    TOTPConfig, SMSConfig, AuthStats, AuthConfig, LockoutInfo,
    RateLimitInfo, AuthMiddlewareConfig, AuthMiddleware,
    init_auth_manager, shutdown_auth_manager, get_auth_manager,
    get_auth_manager_mut,
};

pub use boot_verify::{
    BootVerify, BootVerifyResult, BootImageInfo, BootChainElement,
    BootComponentType, BootVerifyConfig, BootAttestation, BootEvent,
    BootStatusSummary, HsmInterface, HsmStatus, HsmError,
    init as init_boot_verify, verify_image, verify_chain, measured_boot,
    instance as boot_verify_instance,
};

pub use network::{
    NetworkSecurity, NetworkSecurityResult, FirewallRule, FirewallRuleType,
    NetworkProtocol, NetworkPacket, VpnTunnel, VpnEncryption, VpnAuth,
    VpnStatus, IntrusionSignature, IntrusionSeverity, IntrusionEvent,
    IntrusionResponse, InterfaceSecurity, NetworkSecurityStats, RuleStats,
    init as init_network_security, process_packet, detect_intrusions,
    add_firewall_rule, create_vpn_tunnel, get_stats as get_network_stats,
    instance as network_security_instance,
};

// RBAC and Access Control exports
pub use rbac::{
    RbacManager, RbacResult, RbacError, RbacPermission,
    Role, ResourceType, ResourceId, UserGroupRoleAssignment, EffectivePermissions,
    init_rbac_manager, get_rbac_manager,
};

pub use acl::{
    AccessControlList, AclEntry, AclPermission, PrincipalType, AclContext, AclManager,
};

pub use permission_inheritance::{
    PermissionInheritance, InheritanceRule, InheritanceChain, InheritanceConflict,
    InheritanceLevel, InheritanceContext, InheritancePolicy, ConflictResolution,
    InheritanceHelpers,
};

pub use delegation::{
    DelegationManager, PermissionDelegation, DelegationConstraints, DelegationChain,
    DelegationConflict, DelegationScope, DelegationOperation, DelegationResolution,
    DelegationAuditEntry, delegation,
};

/// Security subsystem initialization
pub fn init_security() -> Result<(), Box<dyn core::fmt::Display>> {
    info!("Initializing security subsystem...");
    
    // Initialize encryption manager
    if let Err(e) = EncryptionManager::init() {
        error!("Failed to initialize encryption manager: {:?}", e);
        return Err(Box::new(e));
    }
    
    // Initialize secure random number generator
    if let Err(e) = RandomNumberGenerator::init() {
        error!("Failed to initialize random number generator: {:?}", e);
        return Err(Box::new(e));
    }
    
    info!("Security subsystem initialized successfully");
    Ok(())
}

/// Initialize authentication subsystem
pub fn init_authentication(config: AuthConfig) -> AuthResult<()> {
    info!("Initializing authentication subsystem...");
    init_auth_manager(config)
}

/// Shutdown security subsystem
pub fn shutdown_security() -> Result<(), Box<dyn core::fmt::Display>> {
    info!("Shutting down security subsystem...");
    
    // Cleanup encryption resources
    if let Err(e) = EncryptionManager::shutdown() {
        error!("Failed to shutdown encryption manager: {:?}", e);
        return Err(Box::new(e));
    }
    
    info!("Security subsystem shutdown complete");
    Ok(())
}

/// Shutdown authentication subsystem
pub fn shutdown_authentication() -> AuthResult<()> {
    info!("Shutting down authentication subsystem...");
    shutdown_auth_manager()
}

/// Initialize comprehensive security system
pub fn init_comprehensive_security() -> Result<(), Box<dyn core::fmt::Display>> {
    info!("Initializing comprehensive security system...");
    
    // Initialize core security
    init_security()?;
    
    // Initialize RBAC system
    if let Err(e) = rbac::init_rbac_manager() {
        error!("Failed to initialize RBAC manager: {:?}", e);
        return Err(Box::new(e));
    }
    info!("RBAC system initialized");
    
    // Initialize boot verification
    let boot_config = boot_verify::BootVerifyConfig {
        verify_images: true,
        verify_chain: true,
        measured_boot: true,
        use_tpm: true,
        use_hsm: false,
        strict_mode: false,
        trust_anchor: vec![0; 32],
    };
    boot_verify::init(boot_config);
    info!("Boot verification initialized");
    
    // Initialize network security
    network::init();
    info!("Network security initialized");
    
    info!("Comprehensive security system initialized successfully");
    Ok(())
}

/// Get comprehensive security statistics
pub fn get_security_stats() -> (AuthStats, EncryptionManager, NetworkSecurityStats) {
    let auth_stats = get_auth_stats();
    let encryption_stats = EncryptionManager::get_stats().unwrap_or_default();
    let network_stats = get_network_stats().unwrap_or_default();
    (auth_stats, encryption_stats, network_stats)
}

/// Get authentication statistics only
pub fn get_auth_stats() -> AuthStats {
    if let Some(auth_manager) = get_auth_manager() {
        if let Some(manager) = auth_manager.lock().as_ref() {
            return manager.get_stats();
        }
    }
    
    AuthStats {
        total_login_attempts: 0,
        successful_logins: 0,
        failed_logins: 0,
        locked_accounts: 0,
        active_sessions: 0,
        expired_sessions: 0,
        multi_factor_successes: 0,
        multi_factor_failures: 0,
        biometric_attempts: 0,
        biometric_successes: 0,
        hardware_token_attempts: 0,
        totp_verifications: 0,
        rate_limit_triggers: 0,
    }
}
// Tests and integration verification
#[cfg(test)]
pub mod tests;