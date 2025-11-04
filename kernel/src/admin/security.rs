//! MultiOS Security Management Module
//! 
//! This module provides comprehensive security management functionality including:
//! - Access control and permission management
//! - Security policy enforcement
//! - Role-based access control (RBAC)
//! - Security level management
//! - Integration with user management for secure authentication

#![no_std]
#![feature(alloc)]

use spin::{Mutex, RwLock};
use alloc::vec::Vec;
use alloc::string::{String, ToString};
use alloc::collections::HashMap;

/// Security management result
pub type SecurityResult<T> = Result<T, SecurityError>;

/// Security error types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SecurityError {
    PermissionDenied = 0,
    AccessDenied = 1,
    InvalidSecurityLevel = 2,
    PolicyViolation = 3,
    CapabilityNotHeld = 4,
    OperationNotPermitted = 5,
    SecurityViolation = 6,
    PolicyNotFound = 7,
    InvalidParameter = 8,
    NotInitialized = 9,
    ResourceExhausted = 10,
}

/// Security levels for different operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum SecurityLevel {
    Untrusted = 0,
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
    System = 5,
}

/// Access control permissions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum Permission {
    None = 0,
    Read = 1 << 0,
    Write = 1 << 1,
    Execute = 1 << 2,
    Create = 1 << 3,
    Delete = 1 << 4,
    Modify = 1 << 5,
    Admin = 1 << 6,
    System = 1 << 7,
    All = 0xFFFF,
}

/// Capability-based access control
#[derive(Debug, Clone)]
pub struct Capability {
    pub name: String,
    pub level: SecurityLevel,
    pub permissions: u32,
    pub resource_types: Vec<String>,
}

/// Security context for processes and users
#[derive(Debug, Clone)]
pub struct SecurityContext {
    pub user_id: crate::admin::user_manager::UserId,
    pub session_id: u64,
    pub security_level: SecurityLevel,
    pub capabilities: Vec<Capability>,
    pub policies: Vec<String>,
    pub audit_enabled: bool,
    pub isolation_level: u32,
}

/// Role-based access control
#[derive(Debug, Clone)]
pub struct Role {
    pub name: String,
    pub description: String,
    pub security_level: SecurityLevel,
    pub permissions: u32,
    pub capabilities: Vec<String>,
    pub is_system_role: bool,
}

/// Security policy definition
#[derive(Debug, Clone)]
pub struct SecurityPolicy {
    pub name: String,
    pub description: String,
    pub enabled: bool,
    pub rules: Vec<PolicyRule>,
    pub enforcement_level: SecurityLevel,
}

/// Policy rule for access control
#[derive(Debug, Clone)]
pub struct PolicyRule {
    pub name: String,
    pub action: PolicyAction,
    pub resource_type: String,
    pub operation: String,
    pub allowed_levels: Vec<SecurityLevel>,
    pub required_permissions: u32,
    pub required_capabilities: Vec<String>,
    pub audit_required: bool,
}

/// Policy action types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PolicyAction {
    Allow = 0,
    Deny = 1,
    Audit = 2,
    Challenge = 3,
}

/// Security statistics
#[derive(Debug, Clone)]
pub struct SecurityStats {
    pub total_policies: usize,
    pub active_policies: usize,
    pub total_roles: usize,
    pub security_violations: u64,
    pub denied_accesses: u64,
    pub security_audits: u64,
    pub capability_checks: u64,
}

/// Global security manager instance
static SECURITY_MANAGER: Mutex<Option<SecurityManager>> = Mutex::new(None);

/// Security Manager - Main orchestrator for security operations
pub struct SecurityManager {
    policies: RwLock<HashMap<String, SecurityPolicy>>,
    roles: RwLock<HashMap<String, Role>>,
    capabilities: RwLock<HashMap<String, Capability>>,
    security_contexts: RwLock<HashMap<u64, SecurityContext>>,
    active_contexts: RwLock<Vec<u64>>,
    initialized: bool,
    stats: Mutex<SecurityStats>,
}

impl SecurityManager {
    /// Create a new Security Manager instance
    pub fn new() -> Self {
        Self {
            policies: RwLock::new(HashMap::new()),
            roles: RwLock::new(HashMap::new()),
            capabilities: RwLock::new(HashMap::new()),
            security_contexts: RwLock::new(HashMap::new()),
            active_contexts: RwLock::new(Vec::new()),
            initialized: false,
            stats: Mutex::new(SecurityStats {
                total_policies: 0,
                active_policies: 0,
                total_roles: 0,
                security_violations: 0,
                denied_accesses: 0,
                security_audits: 0,
                capability_checks: 0,
            }),
        }
    }

    /// Initialize the security manager
    pub fn init(&mut self) -> SecurityResult<()> {
        if self.initialized {
            return Err(SecurityError::PolicyNotFound); // Using existing error code
        }

        // Create default security policies
        self.create_default_policies()?;
        
        // Create default roles
        self.create_default_roles()?;
        
        // Create default capabilities
        self.create_default_capabilities()?;

        self.initialized = true;
        
        info!("Security Manager initialized successfully");
        Ok(())
    }

    /// Shutdown the security manager
    pub fn shutdown(&mut self) -> SecurityResult<()> {
        if !self.initialized {
            return Err(SecurityError::NotInitialized);
        }

        // Clear all active security contexts
        {
            let mut active_contexts = self.active_contexts.write();
            active_contexts.clear();
        }

        self.initialized = false;
        info!("Security Manager shutdown complete");
        Ok(())
    }

    // ==================== Security Context Management ====================

    /// Create a new security context for a user session
    pub fn create_security_context(&self, user_id: crate::admin::user_manager::UserId, 
                                 session_id: u64, security_level: SecurityLevel) -> SecurityResult<u64> {
        let context_id = session_id; // Use session ID as context ID
        
        // Get user information to determine base capabilities
        let user_manager = crate::admin::user_manager::get_user_manager()
            .lock().as_ref().and_then(|mgr| mgr.as_ref().ok());
        
        let mut capabilities = Vec::new();
        if let Some(manager) = user_manager {
            if let Ok(user) = manager.get_user(user_id) {
                // Add capabilities based on user privileges
                for privilege in &user.privileges {
                    let cap = self.get_capability_for_privilege(privilege, security_level)?;
                    capabilities.push(cap);
                }
            }
        }

        // Add default system capabilities
        if security_level >= SecurityLevel::High {
            capabilities.extend(self.get_system_capabilities(security_level)?);
        }

        let context = SecurityContext {
            user_id,
            session_id,
            security_level,
            capabilities,
            policies: Vec::new(), // Would be populated based on security level
            audit_enabled: security_level >= SecurityLevel::Medium,
            isolation_level: self.get_isolation_level(security_level)?,
        };

        {
            let mut contexts = self.security_contexts.write();
            contexts.insert(context_id, context);
        }

        {
            let mut active_contexts = self.active_contexts.write();
            if !active_contexts.contains(&context_id) {
                active_contexts.push(context_id);
            }
        }

        info!("Created security context {} for user {} at level {:?}", 
              context_id, user_id, security_level);
        
        Ok(context_id)
    }

    /// Destroy a security context
    pub fn destroy_security_context(&self, context_id: u64) -> SecurityResult<()> {
        {
            let mut contexts = self.security_contexts.write();
            contexts.remove(&context_id);
        }

        {
            let mut active_contexts = self.active_contexts.write();
            active_contexts.retain(|&id| id != context_id);
        }

        info!("Destroyed security context {}", context_id);
        Ok(())
    }

    /// Get security context
    pub fn get_security_context(&self, context_id: u64) -> SecurityResult<SecurityContext> {
        let contexts = self.security_contexts.read();
        contexts.get(&context_id)
            .cloned()
            .ok_or(SecurityError::PolicyNotFound)
    }

    // ==================== Access Control Operations ====================

    /// Check if an operation is permitted
    pub fn check_permission(&self, context_id: u64, resource_type: &str, 
                          operation: &str, required_level: SecurityLevel) -> SecurityResult<bool> {
        let context = self.get_security_context(context_id)?;
        
        // Check security level
        if context.security_level < required_level {
            let mut stats = self.stats.lock();
            stats.denied_accesses += 1;
            return Ok(false);
        }

        // Check policy rules
        let policies = self.policies.read();
        let mut allowed = false;

        for policy in policies.values() {
            if !policy.enabled {
                continue;
            }

            for rule in &policy.rules {
                if self.rule_matches(rule, resource_type, operation) {
                    match rule.action {
                        PolicyAction::Allow => {
                            if context.security_level >= rule.allowed_levels.iter().min().copied().unwrap_or(SecurityLevel::Low) {
                                allowed = true;
                            }
                        }
                        PolicyAction::Deny => {
                            return Ok(false);
                        }
                        PolicyAction::Audit => {
                            // Log for audit but don't deny
                            self.audit_security_event("audit_access", context_id, resource_type, operation)?;
                        }
                        PolicyAction::Challenge => {
                            // Additional authentication required
                            self.audit_security_event("challenge_access", context_id, resource_type, operation)?;
                        }
                    }
                }
            }
        }

        if !allowed {
            let mut stats = self.stats.lock();
            stats.denied_accesses += 1;
        }

        Ok(allowed)
    }

    /// Check if a capability is held
    pub fn check_capability(&self, context_id: u64, capability_name: &str) -> SecurityResult<bool> {
        let context = self.get_security_context(context_id)?;
        
        let mut stats = self.stats.lock();
        stats.capability_checks += 1;

        for capability in &context.capabilities {
            if capability.name == capability_name && context.security_level >= capability.level {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Grant a capability to a security context
    pub fn grant_capability(&self, context_id: u64, capability: Capability) -> SecurityResult<()> {
        let mut contexts = self.security_contexts.write();
        
        let context = contexts.get_mut(&context_id)
            .ok_or(SecurityError::PolicyNotFound)?;

        // Only allow granting capabilities at or above current security level
        if context.security_level < capability.level {
            return Err(SecurityError::PermissionDenied);
        }

        // Remove existing capability with same name
        context.capabilities.retain(|cap| cap.name != capability.name);
        context.capabilities.push(capability);

        info!("Granted capability '{}' to context {}", capability.name, context_id);
        Ok(())
    }

    /// Revoke a capability from a security context
    pub fn revoke_capability(&self, context_id: u64, capability_name: &str) -> SecurityResult<()> {
        let mut contexts = self.security_contexts.write();
        
        let context = contexts.get_mut(&context_id)
            .ok_or(SecurityError::PolicyNotFound)?;

        let before_count = context.capabilities.len();
        context.capabilities.retain(|cap| cap.name != capability_name);
        let after_count = context.capabilities.len();

        if after_count < before_count {
            info!("Revoked capability '{}' from context {}", capability_name, context_id);
        }

        Ok(())
    }

    // ==================== Role-Based Access Control ====================

    /// Assign a role to a user
    pub fn assign_role(&self, user_id: crate::admin::user_manager::UserId, role_name: &str) -> SecurityResult<()> {
        let roles = self.roles.read();
        
        let role = roles.get(role_name)
            .ok_or(SecurityError::PolicyNotFound)?;

        // Get user manager and assign role
        let user_manager = crate::admin::user_manager::get_user_manager()
            .lock().as_ref().and_then(|mgr| mgr.as_ref().ok());
        
        if let Some(manager) = user_manager {
            // Update user's privileges to include role permissions
            let _ = manager.get_user(user_id).map(|mut user| {
                if !user.privileges.contains(&role.name) {
                    user.privileges.push(role.name.clone());
                }
                user
            });
        }

        info!("Assigned role '{}' to user {}", role_name, user_id);
        Ok(())
    }

    /// Remove a role from a user
    pub fn remove_role(&self, user_id: crate::admin::user_manager::UserId, role_name: &str) -> SecurityResult<()> {
        // Get user manager and remove role
        let user_manager = crate::admin::user_manager::get_user_manager()
            .lock().as_ref().and_then(|mgr| mgr.as_ref().ok());
        
        if let Some(manager) = user_manager {
            let _ = manager.get_user(user_id).map(|mut user| {
                user.privileges.retain(|priv| priv != role_name);
                user
            });
        }

        info!("Removed role '{}' from user {}", role_name, user_id);
        Ok(())
    }

    /// Create a new role
    pub fn create_role(&self, role: Role) -> SecurityResult<()> {
        let mut roles = self.roles.write();
        
        if roles.contains_key(&role.name) {
            return Err(SecurityError::PolicyNotFound);
        }

        roles.insert(role.name.clone(), role);
        
        info!("Created role: {}", role.name);
        Ok(())
    }

    // ==================== Policy Management ====================

    /// Create a security policy
    pub fn create_policy(&self, policy: SecurityPolicy) -> SecurityResult<()> {
        let mut policies = self.policies.write();
        
        if policies.contains_key(&policy.name) {
            return Err(SecurityError::PolicyNotFound);
        }

        policies.insert(policy.name.clone(), policy);
        
        let mut stats = self.stats.lock();
        stats.total_policies += 1;
        stats.active_policies += 1;

        info!("Created security policy: {}", policy.name);
        Ok(())
    }

    /// Enable or disable a policy
    pub fn set_policy_enabled(&self, policy_name: &str, enabled: bool) -> SecurityResult<()> {
        let mut policies = self.policies.write();
        
        let policy = policies.get_mut(policy_name)
            .ok_or(SecurityError::PolicyNotFound)?;

        let was_enabled = policy.enabled;
        policy.enabled = enabled;

        let mut stats = self.stats.lock();
        if was_enabled && !enabled {
            stats.active_policies -= 1;
        } else if !was_enabled && enabled {
            stats.active_policies += 1;
        }

        info!("{}d policy: {}", if enabled { "Enable" } else { "Disable" }, policy_name);
        Ok(())
    }

    // ==================== Internal Helper Methods ====================

    /// Create default security policies
    fn create_default_policies(&self) -> SecurityResult<()> {
        let default_policy = SecurityPolicy {
            name: "default_access".to_string(),
            description: "Default access control policy".to_string(),
            enabled: true,
            rules: vec![
                PolicyRule {
                    name: "file_read".to_string(),
                    action: PolicyAction::Allow,
                    resource_type: "file".to_string(),
                    operation: "read".to_string(),
                    allowed_levels: vec![SecurityLevel::Low, SecurityLevel::Medium, SecurityLevel::High, SecurityLevel::Critical, SecurityLevel::System],
                    required_permissions: Permission::Read as u32,
                    required_capabilities: vec!["file_read".to_string()],
                    audit_required: true,
                },
                PolicyRule {
                    name: "system_admin".to_string(),
                    action: PolicyAction::Allow,
                    resource_type: "system".to_string(),
                    operation: "admin".to_string(),
                    allowed_levels: vec![SecurityLevel::System],
                    required_permissions: Permission::Admin as u32 | Permission::System as u32,
                    required_capabilities: vec!["system_admin".to_string()],
                    audit_required: true,
                },
            ],
            enforcement_level: SecurityLevel::Medium,
        };

        self.create_policy(default_policy)?;
        Ok(())
    }

    /// Create default roles
    fn create_default_roles(&self) -> SecurityResult<()> {
        let admin_role = Role {
            name: "administrator".to_string(),
            description: "System Administrator".to_string(),
            security_level: SecurityLevel::System,
            permissions: Permission::All as u32,
            capabilities: vec!["system_admin".to_string(), "user_admin".to_string()],
            is_system_role: true,
        };

        let user_role = Role {
            name: "user".to_string(),
            description: "Regular User".to_string(),
            security_level: SecurityLevel::Low,
            permissions: Permission::Read as u32 | Permission::Write as u32 | Permission::Execute as u32,
            capabilities: vec!["file_read".to_string(), "file_write".to_string()],
            is_system_role: false,
        };

        let auditor_role = Role {
            name: "auditor".to_string(),
            description: "Security Auditor".to_string(),
            security_level: SecurityLevel::High,
            permissions: Permission::Read as u32,
            capabilities: vec!["audit_read".to_string(), "security_read".to_string()],
            is_system_role: false,
        };

        self.create_role(admin_role)?;
        self.create_role(user_role)?;
        self.create_role(auditor_role)?;
        Ok(())
    }

    /// Create default capabilities
    fn create_default_capabilities(&self) -> SecurityResult<()> {
        let mut capabilities = self.capabilities.write();

        capabilities.insert("file_read".to_string(), Capability {
            name: "file_read".to_string(),
            level: SecurityLevel::Low,
            permissions: Permission::Read as u32,
            resource_types: vec!["file".to_string()],
        });

        capabilities.insert("file_write".to_string(), Capability {
            name: "file_write".to_string(),
            level: SecurityLevel::Medium,
            permissions: Permission::Write as u32,
            resource_types: vec!["file".to_string()],
        });

        capabilities.insert("system_admin".to_string(), Capability {
            name: "system_admin".to_string(),
            level: SecurityLevel::System,
            permissions: Permission::Admin as u32 | Permission::System as u32,
            resource_types: vec!["system".to_string()],
        });

        capabilities.insert("audit_read".to_string(), Capability {
            name: "audit_read".to_string(),
            level: SecurityLevel::High,
            permissions: Permission::Read as u32,
            resource_types: vec!["audit".to_string()],
        });

        Ok(())
    }

    /// Get capability for a privilege
    fn get_capability_for_privilege(&self, privilege: &str, level: SecurityLevel) -> SecurityResult<Capability> {
        let capabilities = self.capabilities.read();
        
        let cap_name = match privilege.as_str() {
            "system" => "system_admin".to_string(),
            "user" => "file_read".to_string(),
            _ => format!("{}_capability", privilege),
        };

        if let Some(cap) = capabilities.get(&cap_name) {
            Ok(cap.clone())
        } else {
            // Create a basic capability if not found
            Ok(Capability {
                name: cap_name,
                level,
                permissions: Permission::Read as u32,
                resource_types: vec!["default".to_string()],
            })
        }
    }

    /// Get system capabilities for a security level
    fn get_system_capabilities(&self, level: SecurityLevel) -> SecurityResult<Vec<Capability>> {
        let capabilities = self.capabilities.read();
        let mut result = Vec::new();

        for cap in capabilities.values() {
            if cap.level <= level {
                result.push(cap.clone());
            }
        }

        Ok(result)
    }

    /// Get isolation level for security level
    fn get_isolation_level(&self, security_level: SecurityLevel) -> SecurityResult<u32> {
        Ok(match security_level {
            SecurityLevel::Untrusted => 1,
            SecurityLevel::Low => 2,
            SecurityLevel::Medium => 3,
            SecurityLevel::High => 4,
            SecurityLevel::Critical => 5,
            SecurityLevel::System => 6,
        })
    }

    /// Check if a policy rule matches the resource and operation
    fn rule_matches(&self, rule: &PolicyRule, resource_type: &str, operation: &str) -> bool {
        rule.resource_type == resource_type && rule.operation == operation
    }

    /// Audit security event
    fn audit_security_event(&self, event_type: &str, context_id: u64, resource: &str, operation: &str) -> SecurityResult<()> {
        let context = self.get_security_context(context_id)?;
        
        if context.audit_enabled {
            let mut stats = self.stats.lock();
            stats.security_audits += 1;
            
            info!("SECURITY AUDIT: {} - Context: {} - Resource: {} - Operation: {} - User: {}", 
                  event_type, context_id, resource, operation, context.user_id);
        }

        Ok(())
    }

    /// Get security statistics
    pub fn get_stats(&self) -> SecurityStats {
        let stats = self.stats.lock();
        stats.clone()
    }
}

/// Initialize the global security manager
pub fn init_security_manager() -> SecurityResult<()> {
    let mut manager_guard = SECURITY_MANAGER.lock();
    
    if manager_guard.is_some() {
        return Err(SecurityError::PolicyNotFound);
    }

    let mut manager = SecurityManager::new();
    manager.init()?;
    
    *manager_guard = Some(manager);
    
    info!("Security Manager initialized successfully");
    Ok(())
}

/// Shutdown the global security manager
pub fn shutdown_security_manager() -> SecurityResult<()> {
    let mut manager_guard = SECURITY_MANAGER.lock();
    
    if let Some(mut manager) = manager_guard.take() {
        manager.shutdown()?;
    }
    
    info!("Security Manager shutdown complete");
    Ok(())
}

/// Get the global security manager instance
pub fn get_security_manager() -> Option<&'static Mutex<Option<SecurityManager>>> {
    Some(&SECURITY_MANAGER)
}