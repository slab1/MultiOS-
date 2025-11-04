//! Permission Delegation System
//! 
//! This module provides comprehensive permission delegation functionality including:
//! - User-to-user permission delegation
//! - Delegation scope and limitation management
//! - Delegation revocation mechanisms
//! - Audit trail for delegation operations

use alloc::vec::Vec;
use alloc::string::{String, ToString};
use alloc::collections::{HashMap, BTreeSet, BTreeMap};
use crate::admin::user_manager::UserId;
use crate::security::rbac::{RbacPermission, SecurityLevel};
use super::RbacError;

/// Delegation scope defining the boundaries of delegated permissions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum DelegationScope {
    None = 0,          // No delegation allowed
    User = 1,          // Delegate to specific user
    Group = 2,         // Delegate to group members
    Role = 3,          // Delegate to role members
    Organization = 4,  // Delegate within organization
    Resource = 5,      // Delegate specific resource access
    TimeLimited = 6,   // Time-limited delegation
    Conditional = 7,   // Conditional delegation based on context
}

/// Delegation limitations and constraints
#[derive(Debug, Clone)]
pub struct DelegationConstraints {
    pub max_delegations: Option<u32>,          // Maximum number of delegations
    pub time_limit: Option<u64>,               // Time limit in seconds
    pub resource_restrictions: Vec<String>,    // Restricted resource paths
    pub permission_restrictions: Vec<RbacPermission>, // Restricted permissions
    pub requires_approval: bool,               // Requires approval before delegation
    pub can_revoke: bool,                      // Can revoke own delegations
    pub chain_delegation: bool,                // Can delegate received permissions
}

/// Permission delegation record
#[derive(Debug, Clone)]
pub struct PermissionDelegation {
    pub delegation_id: String,
    pub delegator_user_id: UserId,
    pub delegatee_user_id: UserId,
    pub resource_id: String,
    pub permissions: Vec<RbacPermission>,
    pub scope: DelegationScope,
    pub constraints: DelegationConstraints,
    pub granted_at: u64,
    pub expires_at: Option<u64>,
    pub approved_by: Option<UserId>,
    pub revoked_by: Option<UserId>,
    pub revoked_at: Option<u64>,
    pub is_active: bool,
    pub audit_trail: Vec<DelegationAuditEntry>,
}

/// Audit entry for delegation operations
#[derive(Debug, Clone)]
pub struct DelegationAuditEntry {
    pub timestamp: u64,
    pub operation: DelegationOperation,
    pub actor_user_id: Option<UserId>,
    pub target_user_id: Option<UserId>,
    pub resource_id: Option<String>,
    pub details: String,
    pub success: bool,
    pub ip_address: Option<String>,
}

/// Delegation operations for audit trail
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum DelegationOperation {
    Grant = 0,
    Revoke = 1,
    Approve = 2,
    Reject = 3,
    Expire = 4,
    Modify = 5,
    Transfer = 6,
    Query = 7,
}

/// Delegation chain representing transitive permission delegation
#[derive(Debug, Clone)]
pub struct DelegationChain {
    pub original_grantor: UserId,
    pub final_grantee: UserId,
    pub chain: Vec<PermissionDelegation>,
    pub total_permissions: BTreeSet<RbacPermission>,
    pub conflicts: Vec<DelegationConflict>,
}

/// Delegation conflict when multiple delegators grant conflicting permissions
#[derive(Debug, Clone)]
pub struct DelegationConflict {
    pub resource_id: String,
    pub conflicting_permissions: Vec<RbacPermission>,
    pub delegators: Vec<UserId>,
    pub resolution_strategy: DelegationResolution,
}

/// Conflict resolution strategies for delegation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum DelegationResolution {
    FirstComeFirstServed = 0,    // First delegation takes precedence
    LastComeFirstServed = 1,     // Last delegation takes precedence
    MostTrustedDelegator = 2,    // Most trusted delegator's permissions apply
    MergePermissions = 3,        // Merge all permissions
    ExplicitDeny = 4,            // Explicit deny overrides all
    ManualResolution = 5,        // Requires manual resolution
}

/// Delegation manager for handling permission delegation operations
pub struct DelegationManager {
    delegations: HashMap<String, PermissionDelegation>,
    delegation_chains: HashMap<(UserId, String), DelegationChain>,
    delegation_audit: Vec<DelegationAuditEntry>,
    max_delegation_depth: u32,
    default_constraints: DelegationConstraints,
}

impl DelegationManager {
    /// Create a new delegation manager
    pub fn new() -> Self {
        Self {
            delegations: HashMap::new(),
            delegation_chains: HashMap::new(),
            delegation_audit: Vec::new(),
            max_delegation_depth: 5, // Limit delegation depth to prevent abuse
            default_constraints: DelegationConstraints {
                max_delegations: Some(100),
                time_limit: Some(86400), // 24 hours
                resource_restrictions: Vec::new(),
                permission_restrictions: vec![RbacPermission::Admin, RbacPermission::System],
                requires_approval: false,
                can_revoke: true,
                chain_delegation: false,
            },
        }
    }

    /// Create a new permission delegation
    pub fn create_delegation(&mut self, mut delegation: PermissionDelegation) -> RbacResult<String> {
        // Validate delegation
        self.validate_delegation(&delegation)?;

        // Generate delegation ID
        let delegation_id = format!("delegation_{}_{}_{}", 
                                   delegation.delegator_user_id, 
                                   delegation.delegatee_user_id, 
                                   crate::hal::get_current_time());
        delegation.delegation_id = delegation_id.clone();

        // Check delegation limits
        if let Some(max_delegations) = self.default_constraints.max_delegations {
            let existing_delegations = self.get_delegator_delegations(delegation.delegator_user_id).len();
            if existing_delegations >= max_delegations {
                return Err(RbacError::ResourceExhausted);
            }
        }

        // Check for circular delegation
        if self.has_circular_delegation(delegation.delegator_user_id, delegation.delegatee_user_id) {
            return Err(RbacError::CircularDependency);
        }

        // Store delegation
        self.delegations.insert(delegation_id.clone(), delegation.clone());

        // Build delegation chain
        self.build_delegation_chain(&delegation)?;

        // Add audit entry
        self.add_audit_entry(DelegationOperation::Grant, Some(delegation.delegator_user_id), 
                            Some(delegation.delegatee_user_id), Some(&delegation.resource_id), 
                            "Delegation created", true, None);

        info!("Created permission delegation: {} from user {} to user {}", 
              delegation_id, delegation.delegator_user_id, delegation.delegatee_user_id);

        Ok(delegation_id)
    }

    /// Revoke a permission delegation
    pub fn revoke_delegation(&mut self, delegation_id: &str, revoker_id: UserId) -> RbacResult<()> {
        let delegation = self.delegations.get_mut(delegation_id)
            .ok_or(RbacError::ResourceNotFound)?;

        // Check revocation permissions
        if !self.can_revoke_delegation(delegation, revoker_id) {
            return Err(RbacError::PermissionDenied);
        }

        // Mark as revoked
        delegation.is_active = false;
        delegation.revoked_by = Some(revoker_id);
        delegation.revoked_at = Some(crate::hal::get_current_time());

        // Remove from active delegations
        self.delegations.remove(delegation_id);

        // Update delegation chains
        self.remove_from_delegation_chains(delegation_id);

        // Add audit entry
        self.add_audit_entry(DelegationOperation::Revoke, Some(revoker_id), 
                            Some(delegation.delegatee_user_id), Some(&delegation.resource_id), 
                            "Delegation revoked", true, None);

        info!("Revoked permission delegation: {}", delegation_id);
        Ok(())
    }

    /// Get active delegations for a user
    pub fn get_active_delegations(&self, user_id: UserId) -> Vec<&PermissionDelegation> {
        let current_time = crate::hal::get_current_time();
        
        self.delegations.values()
            .filter(|delegation| {
                delegation.is_active && 
                (delegation.expires_at.is_none() || current_time <= delegation.expires_at.unwrap()) &&
                (delegation.delegator_user_id == user_id || delegation.delegatee_user_id == user_id)
            })
            .collect()
    }

    /// Get delegated permissions for user on resource
    pub fn get_delegated_permissions(&self, user_id: UserId, resource_id: &str) -> BTreeSet<RbacPermission> {
        let current_time = crate::hal::get_current_time();
        let mut permissions = BTreeSet::new();

        for delegation in self.delegations.values() {
            if delegation.is_active && 
               delegation.delegatee_user_id == user_id &&
               delegation.resource_id == resource_id &&
               (delegation.expires_at.is_none() || current_time <= delegation.expires_at.unwrap()) {
                
                for &permission in &delegation.permissions {
                    permissions.insert(permission);
                }
            }
        }

        permissions
    }

    /// Check if user can delegate permission
    pub fn can_delegate_permission(&self, delegator_id: UserId, permission: RbacPermission, 
                                  resource_id: &str) -> bool {
        // Check if user has the permission to delegate
        // This would typically involve checking RBAC permissions
        // For now, simplified implementation
        
        if delegation::permission_is_restricted(permission) {
            return false;
        }

        // Check user constraints
        let delegations = self.get_delegator_delegations(delegator_id);
        
        if let Some(max_delegations) = self.default_constraints.max_delegations {
            if delegations.len() >= max_delegations as usize {
                return false;
            }
        }

        true
    }

    /// Modify an existing delegation
    pub fn modify_delegation(&mut self, delegation_id: &str, modifier_id: UserId,
                           new_permissions: Vec<RbacPermission>, new_expires_at: Option<u64>) -> RbacResult<()> {
        let delegation = self.delegations.get_mut(delegation_id)
            .ok_or(RbacError::ResourceNotFound)?;

        // Check modification permissions
        if !self.can_modify_delegation(delegation, modifier_id) {
            return Err(RbacError::PermissionDenied);
        }

        // Update permissions
        delegation.permissions = new_permissions;
        delegation.expires_at = new_expires_at;

        // Update delegation chain
        self.update_delegation_chain(delegation)?;

        // Add audit entry
        self.add_audit_entry(DelegationOperation::Modify, Some(modifier_id), 
                            Some(delegation.delegatee_user_id), Some(&delegation.resource_id), 
                            "Delegation modified", true, None);

        info!("Modified permission delegation: {}", delegation_id);
        Ok(())
    }

    /// Get delegation chains for user
    pub fn get_delegation_chains(&self, user_id: UserId) -> Vec<&DelegationChain> {
        self.delegation_chains.values()
            .filter(|chain| {
                chain.original_grantor == user_id || chain.final_grantee == user_id
            })
            .collect()
    }

    /// Clean up expired delegations
    pub fn cleanup_expired_delegations(&mut self) -> u32 {
        let current_time = crate::hal::get_current_time();
        let mut expired_count = 0;

        let expired_delegations: Vec<String> = self.delegations.values()
            .filter(|delegation| {
                delegation.is_active && 
                delegation.expires_at.is_some() && 
                current_time > delegation.expires_at.unwrap()
            })
            .map(|delegation| delegation.delegation_id.clone())
            .collect();

        for delegation_id in expired_delegations {
            if let Some(delegation) = self.delegations.remove(&delegation_id) {
                self.add_audit_entry(DelegationOperation::Expire, None, 
                                    Some(delegation.delegatee_user_id), Some(&delegation.resource_id), 
                                    "Delegation expired", true, None);
                expired_count += 1;
            }
        }

        info!("Cleaned up {} expired delegations", expired_count);
        expired_count
    }

    /// Get delegation audit trail
    pub fn get_audit_trail(&self, user_id: Option<UserId>, resource_id: Option<&str>, 
                          operation: Option<DelegationOperation>) -> Vec<&DelegationAuditEntry> {
        self.delegation_audit.iter()
            .filter(|entry| {
                if let Some(filter_user) = user_id {
                    if entry.actor_user_id != Some(filter_user) && entry.target_user_id != Some(filter_user) {
                        return false;
                    }
                }

                if let Some(filter_resource) = resource_id {
                    if entry.resource_id.as_ref() != Some(&filter_resource.to_string()) {
                        return false;
                    }
                }

                if let Some(filter_operation) = operation {
                    if entry.operation != filter_operation {
                        return false;
                    }
                }

                true
            })
            .collect()
    }

    // ==================== Internal Helper Methods ====================

    /// Validate delegation before creation
    fn validate_delegation(&self, delegation: &PermissionDelegation) -> RbacResult<()> {
        // Check resource ID
        if delegation.resource_id.is_empty() {
            return Err(RbacError::InvalidParameter);
        }

        // Check permissions
        if delegation.permissions.is_empty() {
            return Err(RbacError::InvalidParameter);
        }

        // Check user IDs
        if delegation.delegator_user_id == delegation.delegatee_user_id {
            return Err(RbacError::InvalidParameter);
        }

        // Check time constraints
        if let Some(expires_at) = delegation.expires_at {
            if expires_at <= delegation.granted_at {
                return Err(RbacError::InvalidParameter);
            }
        }

        // Check permission restrictions
        for &permission in &delegation.permissions {
            if self.default_constraints.permission_restrictions.contains(&permission) {
                return Err(RbacError::DelegationNotAllowed);
            }
        }

        Ok(())
    }

    /// Check if user can revoke a delegation
    fn can_revoke_delegation(&self, delegation: &PermissionDelegation, revoker_id: UserId) -> bool {
        // Delegator can always revoke
        if delegation.delegator_user_id == revoker_id {
            return true;
        }

        // Admin users can revoke
        if revoker_id == 0 { // Assuming 0 is root/admin
            return true;
        }

        // Check if revocation is allowed by constraints
        delegation.constraints.can_revoke && delegation.delegatee_user_id == revoker_id
    }

    /// Check if user can modify a delegation
    fn can_modify_delegation(&self, delegation: &PermissionDelegation, modifier_id: UserId) -> bool {
        // Only delegator and admin can modify
        delegation.delegator_user_id == modifier_id || modifier_id == 0
    }

    /// Check for circular delegation
    fn has_circular_delegation(&self, delegator_id: UserId, delegatee_id: UserId) -> bool {
        if delegator_id == delegatee_id {
            return true;
        }

        // Check if delegatee already has a path back to delegator
        for chain in self.delegation_chains.values() {
            if chain.final_grantee == delegatee_id {
                for delegation in &chain.chain {
                    if delegation.delegatee_user_id == delegator_id {
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Build delegation chain for a new delegation
    fn build_delegation_chain(&mut self, delegation: &PermissionDelegation) -> RbacResult<()> {
        let chain_key = (delegation.delegatee_user_id, delegation.resource_id.clone());

        // Check if chain already exists
        if let Some(existing_chain) = self.delegation_chains.get_mut(&chain_key) {
            // Extend existing chain
            existing_chain.chain.push(delegation.clone());
        } else {
            // Create new chain
            let mut chain = DelegationChain {
                original_grantor: delegation.delegator_user_id,
                final_grantee: delegation.delegatee_user_id,
                chain: vec![delegation.clone()],
                total_permissions: BTreeSet::new(),
                conflicts: Vec::new(),
            };

            // Calculate total permissions
            for permission in &delegation.permissions {
                chain.total_permissions.insert(*permission);
            }

            self.delegation_chains.insert(chain_key, chain);
        }

        Ok(())
    }

    /// Remove delegation from chains
    fn remove_from_delegation_chains(&mut self, delegation_id: &str) {
        self.delegation_chains.retain(|_, chain| {
            chain.chain.retain(|delegation| delegation.delegation_id != delegation_id);
            !chain.chain.is_empty()
        });
    }

    /// Update delegation chain after modification
    fn update_delegation_chain(&self, delegation: &PermissionDelegation) -> RbacResult<()> {
        let chain_key = (delegation.delegatee_user_id, delegation.resource_id.clone());
        
        if let Some(chain) = self.delegation_chains.get_mut(&chain_key) {
            // Recalculate total permissions
            chain.total_permissions.clear();
            for d in &chain.chain {
                for &permission in &d.permissions {
                    chain.total_permissions.insert(permission);
                }
            }
        }

        Ok(())
    }

    /// Get delegations made by a user
    fn get_delegator_delegations(&self, delegator_id: UserId) -> Vec<&PermissionDelegation> {
        self.delegations.values()
            .filter(|delegation| delegation.delegator_user_id == delegator_id && delegation.is_active)
            .collect()
    }

    /// Add audit entry
    fn add_audit_entry(&mut self, operation: DelegationOperation, actor_user_id: Option<UserId>,
                      target_user_id: Option<UserId>, resource_id: Option<&str>,
                      details: &str, success: bool, ip_address: Option<&str>) {
        let entry = DelegationAuditEntry {
            timestamp: crate::hal::get_current_time(),
            operation,
            actor_user_id,
            target_user_id,
            resource_id: resource_id.map(|s| s.to_string()),
            details: details.to_string(),
            success,
            ip_address: ip_address.map(|s| s.to_string()),
        };

        self.delegation_audit.push(entry);

        // Keep audit trail manageable
        if self.delegation_audit.len() > 10000 {
            self.delegation_audit.drain(0..1000);
        }
    }
}

/// Helper functions for delegation
pub mod delegation {
    use super::*;

    /// Check if permission is restricted from delegation
    pub fn permission_is_restricted(permission: RbacPermission) -> bool {
        matches!(permission, RbacPermission::Admin | RbacPermission::System)
    }

    /// Create standard delegation constraints
    pub fn create_standard_constraints() -> DelegationConstraints {
        DelegationConstraints {
            max_delegations: Some(50),
            time_limit: Some(86400), // 24 hours
            resource_restrictions: vec![],
            permission_restrictions: vec![RbacPermission::Admin, RbacPermission::System],
            requires_approval: false,
            can_revoke: true,
            chain_delegation: false,
        }
    }

    /// Create time-limited delegation constraints
    pub fn create_time_limited_constraints(hours: u64) -> DelegationConstraints {
        DelegationConstraints {
            max_delegations: Some(10),
            time_limit: Some(hours * 3600),
            resource_restrictions: vec![],
            permission_restrictions: vec![RbacPermission::Admin, RbacPermission::System],
            requires_approval: true,
            can_revoke: true,
            chain_delegation: false,
        }
    }

    /// Create resource-specific delegation constraints
    pub fn create_resource_specific_constraints(resources: Vec<&str>) -> DelegationConstraints {
        DelegationConstraints {
            max_delegations: Some(20),
            time_limit: Some(43200), // 12 hours
            resource_restrictions: resources.iter().map(|r| r.to_string()).collect(),
            permission_restrictions: vec![RbacPermission::Admin, RbacPermission::System, RbacPermission::Delete],
            requires_approval: false,
            can_revoke: true,
            chain_delegation: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delegation_creation() {
        let mut manager = DelegationManager::new();
        
        let delegation = PermissionDelegation {
            delegation_id: "".to_string(),
            delegator_user_id: 1000,
            delegatee_user_id: 1001,
            resource_id: "/test/resource".to_string(),
            permissions: vec![RbacPermission::Read, RbacPermission::Write],
            scope: DelegationScope::User,
            constraints: delegation::create_standard_constraints(),
            granted_at: 0,
            expires_at: None,
            approved_by: None,
            revoked_by: None,
            revoked_at: None,
            is_active: true,
            audit_trail: Vec::new(),
        };

        let result = manager.create_delegation(delegation);
        assert!(result.is_ok());
    }

    #[test]
    fn test_delegation_revocation() {
        let mut manager = DelegationManager::new();
        
        let mut delegation = PermissionDelegation {
            delegation_id: "".to_string(),
            delegator_user_id: 1000,
            delegatee_user_id: 1001,
            resource_id: "/test/resource".to_string(),
            permissions: vec![RbacPermission::Read],
            scope: DelegationScope::User,
            constraints: delegation::create_standard_constraints(),
            granted_at: 0,
            expires_at: None,
            approved_by: None,
            revoked_by: None,
            revoked_at: None,
            is_active: true,
            audit_trail: Vec::new(),
        };

        let delegation_id = manager.create_delegation(delegation).unwrap();
        
        let result = manager.revoke_delegation(&delegation_id, 1000);
        assert!(result.is_ok());
    }

    #[test]
    fn test_permission_check() {
        let manager = DelegationManager::new();
        
        // Admin and System permissions should be restricted
        assert!(delegation::permission_is_restricted(RbacPermission::Admin));
        assert!(delegation::permission_is_restricted(RbacPermission::System));
        
        // Regular permissions should not be restricted
        assert!(!delegation::permission_is_restricted(RbacPermission::Read));
        assert!(!delegation::permission_is_restricted(RbacPermission::Write));
    }
}
