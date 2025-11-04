//! Access Control List (ACL) Implementation
//! 
//! This module provides comprehensive ACL functionality including:
//! - Fine-grained access control entries
//! - Principal-based permission management
//! - Conditional access control
//! - Inheritance and propagation rules

use alloc::vec::Vec;
use alloc::string::{String, ToString};
use alloc::collections::{BTreeSet, HashMap};
use crate::admin::user_manager::{UserId, GroupId};
use crate::security::rbac::{RbacPermission, SecurityLevel};
use super::RbacError;

/// Principal types for ACL entries
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum PrincipalType {
    User = 0,
    Group = 1,
    Role = 2,
    Everyone = 3,
    System = 4,
}

/// ACL entry representing a permission grant or denial
#[derive(Debug, Clone)]
pub struct AclEntry {
    pub entry_id: String,
    pub principal_type: PrincipalType,
    pub principal_id: u32, // User ID, Group ID, or Role ID depending on principal_type
    pub permissions: Vec<RbacPermission>,
    pub conditions: Vec<String>, // Conditions that must be satisfied for this entry to apply
    pub effective: bool, // Whether this entry is currently effective
    pub inherited: bool, // Whether this entry is inherited from parent
    pub priority: u32, // Higher priority entries take precedence
    pub expires_at: Option<u64>, // When this entry expires
}

/// Access Control List for a resource
#[derive(Debug, Clone)]
pub struct AccessControlList {
    pub resource_id: String,
    pub entries: Vec<AclEntry>,
    pub default_entry: Option<AclEntry>,
    pub mask_entry: Option<AclEntry>, // Used for permission masking
    pub inherit_acls: bool, // Whether to inherit ACLs from parent
    pub owner_user_id: Option<UserId>,
    pub owner_group_id: Option<GroupId>,
    pub default_security_level: SecurityLevel,
}

impl AccessControlList {
    /// Create a new ACL
    pub fn new(resource_id: String, entries: Vec<AclEntry>) -> Self {
        Self {
            resource_id,
            entries,
            default_entry: None,
            mask_entry: None,
            inherit_acls: true,
            owner_user_id: None,
            owner_group_id: None,
            default_security_level: SecurityLevel::Low,
        }
    }

    /// Add an entry to the ACL
    pub fn add_entry(&mut self, entry: AclEntry) {
        self.entries.push(entry);
        self.entries.sort_by(|a, b| b.priority.cmp(&a.priority)); // Sort by priority descending
    }

    /// Remove an entry from the ACL
    pub fn remove_entry(&mut self, entry_id: &str) {
        self.entries.retain(|entry| entry.entry_id != entry_id);
    }

    /// Get entry by ID
    pub fn get_entry(&self, entry_id: &str) -> Option<&AclEntry> {
        self.entries.iter().find(|entry| entry.entry_id == entry_id)
    }

    /// Update an existing entry
    pub fn update_entry(&mut self, entry_id: &str, updated_entry: AclEntry) -> Result<(), RbacError> {
        for entry in &mut self.entries {
            if entry.entry_id == entry_id {
                *entry = updated_entry;
                self.entries.sort_by(|a, b| b.priority.cmp(&a.priority));
                return Ok(());
            }
        }
        Err(RbacError::AclNotFound)
    }

    /// Check if a principal has a specific permission
    pub fn has_permission(&self, principal_type: PrincipalType, principal_id: u32, 
                         permission: RbacPermission, context: &AclContext) -> bool {
        let current_time = context.current_time;
        
        for entry in &self.entries {
            // Skip expired entries
            if let Some(expires_at) = entry.expires_at {
                if current_time > expires_at {
                    continue;
                }
            }

            // Skip ineffective entries
            if !entry.effective {
                continue;
            }

            // Check if principal matches
            if !self.principal_matches(principal_type, principal_id, entry) {
                continue;
            }

            // Check conditions
            if !self.conditions_satisfied(entry, context) {
                continue;
            }

            // Check if permission is granted
            if entry.permissions.contains(&permission) {
                return true;
            }
        }

        false
    }

    /// Get all effective permissions for a principal
    pub fn get_effective_permissions(&self, user_id: UserId) -> BTreeSet<RbacPermission> {
        let mut permissions = BTreeSet::new();
        let current_time = crate::hal::get_current_time();
        
        let context = AclContext {
            current_time,
            user_id: Some(user_id),
            group_ids: vec![], // Would be populated with user's group memberships
            security_level: SecurityLevel::Low,
            ip_address: None,
            session_id: None,
        };

        for entry in &self.entries {
            // Skip expired entries
            if let Some(expires_at) = entry.expires_at {
                if current_time > expires_at {
                    continue;
                }
            }

            // Skip ineffective entries
            if !entry.effective {
                continue;
            }

            // Check if principal matches
            let principal_type = PrincipalType::User;
            if !self.principal_matches(principal_type, user_id, entry) {
                continue;
            }

            // Add permissions
            for permission in &entry.permissions {
                permissions.insert(*permission);
            }
        }

        permissions
    }

    /// Apply permission mask
    pub fn apply_mask(&self, permissions: BTreeSet<RbacPermission>) -> BTreeSet<RbacPermission> {
        if let Some(mask) = &self.mask_entry {
            permissions.into_iter()
                .filter(|perm| mask.permissions.contains(perm))
                .collect()
        } else {
            permissions
        }
    }

    /// Check inheritance from parent ACL
    pub fn check_inheritance(&self, parent_acl: &AccessControlList, user_id: UserId) -> bool {
        if !self.inherit_acls || !parent_acl.inherit_acls {
            return false;
        }

        // Check if user has read permission on parent to inherit ACL
        parent_acl.has_permission(PrincipalType::User, user_id, RbacPermission::Read, &AclContext {
            current_time: crate::hal::get_current_time(),
            user_id: Some(user_id),
            group_ids: vec![],
            security_level: SecurityLevel::Low,
            ip_address: None,
            session_id: None,
        })
    }

    /// Validate ACL integrity
    pub fn validate(&self) -> Result<(), RbacError> {
        // Check for duplicate entry IDs
        let mut seen_ids = HashMap::new();
        for entry in &self.entries {
            if let Some(&count) = seen_ids.get(&entry.entry_id) {
                if count > 0 {
                    return Err(RbacError::InvalidParameter);
                }
            }
            seen_ids.insert(entry.entry_id.clone(), count + 1);
        }

        // Check mask permissions are subset of other permissions
        if let Some(mask) = &self.mask_entry {
            for mask_perm in &mask.permissions {
                let mut has_perm = false;
                for entry in &self.entries {
                    if entry.permissions.contains(mask_perm) {
                        has_perm = true;
                        break;
                    }
                }
                if !has_perm {
                    return Err(RbacError::InvalidParameter);
                }
            }
        }

        Ok(())
    }

    /// Get all entries for a principal type
    pub fn get_entries_for_principal(&self, principal_type: PrincipalType, principal_id: u32) -> Vec<&AclEntry> {
        self.entries.iter()
            .filter(|entry| entry.principal_type == principal_type && entry.principal_id == principal_id)
            .collect()
    }

    /// Get all effective entries
    pub fn get_effective_entries(&self, current_time: u64) -> Vec<&AclEntry> {
        self.entries.iter()
            .filter(|entry| {
                entry.effective && 
                (entry.expires_at.is_none() || current_time <= entry.expires_at.unwrap())
            })
            .collect()
    }

    /// Clear all entries
    pub fn clear(&mut self) {
        self.entries.clear();
        self.default_entry = None;
        self.mask_entry = None;
    }

    /// Get entry count
    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    /// Check if ACL is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Internal helper methods

    /// Check if a principal matches an ACL entry
    fn principal_matches(&self, principal_type: PrincipalType, principal_id: u32, entry: &AclEntry) -> bool {
        match entry.principal_type {
            PrincipalType::User => principal_type == PrincipalType::User && entry.principal_id == principal_id,
            PrincipalType::Group => {
                // For group matching, we would need access to user's group memberships
                // This is simplified for the kernel context
                principal_type == PrincipalType::Group && entry.principal_id == principal_id
            },
            PrincipalType::Role => {
                // For role matching, we would need access to user's roles
                // This is simplified for the kernel context
                principal_type == PrincipalType::Role && entry.principal_id == principal_id
            },
            PrincipalType::Everyone => true,
            PrincipalType::System => principal_type == PrincipalType::System,
        }
    }

    /// Check if conditions are satisfied
    fn conditions_satisfied(&self, entry: &AclEntry, context: &AclContext) -> bool {
        // Simplified condition checking - in real implementation would parse and evaluate conditions
        for condition in &entry.conditions {
            match condition.as_str() {
                "owner_match" => {
                    if let Some(owner_id) = self.owner_user_id {
                        if context.user_id != Some(owner_id) {
                            return false;
                        }
                    }
                },
                "group_match" => {
                    if let Some(group_id) = self.owner_group_id {
                        if !context.group_ids.contains(&group_id) {
                            return false;
                        }
                    }
                },
                "security_level_check" => {
                    if context.security_level < self.default_security_level {
                        return false;
                    }
                },
                "ip_restriction" => {
                    if let Some(ip) = &context.ip_address {
                        // Simplified IP check - in real implementation would use proper IP filtering
                        if !ip.starts_with("192.168.") && !ip.starts_with("10.") {
                            return false;
                        }
                    }
                },
                "session_check" => {
                    if context.session_id.is_none() {
                        return false;
                    }
                },
                _ => {
                    // Unknown condition - treat as not satisfied
                    return false;
                }
            }
        }
        true
    }
}

/// ACL evaluation context
#[derive(Debug, Clone)]
pub struct AclContext {
    pub current_time: u64,
    pub user_id: Option<UserId>,
    pub group_ids: Vec<GroupId>,
    pub security_level: SecurityLevel,
    pub ip_address: Option<String>,
    pub session_id: Option<u64>,
}

impl AclContext {
    /// Create a new ACL context
    pub fn new(user_id: Option<UserId>) -> Self {
        Self {
            current_time: crate::hal::get_current_time(),
            user_id,
            group_ids: Vec::new(),
            security_level: SecurityLevel::Low,
            ip_address: None,
            session_id: None,
        }
    }

    /// Create context with security level
    pub fn with_security_level(user_id: Option<UserId>, security_level: SecurityLevel) -> Self {
        Self {
            current_time: crate::hal::get_current_time(),
            user_id,
            group_ids: Vec::new(),
            security_level,
            ip_address: None,
            session_id: None,
        }
    }

    /// Create context with IP address
    pub fn with_ip(user_id: Option<UserId>, ip_address: &str) -> Self {
        Self {
            current_time: crate::hal::get_current_time(),
            user_id,
            group_ids: Vec::new(),
            security_level: SecurityLevel::Low,
            ip_address: Some(ip_address.to_string()),
            session_id: None,
        }
    }
}

/// ACL manager for resource-specific ACL operations
pub struct AclManager {
    acls: Vec<AccessControlList>,
}

impl AclManager {
    /// Create a new ACL manager
    pub fn new() -> Self {
        Self {
            acls: Vec::new(),
        }
    }

    /// Add ACL for a resource
    pub fn add_acl(&mut self, acl: AccessControlList) -> Result<(), RbacError> {
        // Check for duplicate resource IDs
        if self.acls.iter().any(|existing_acl| existing_acl.resource_id == acl.resource_id) {
            return Err(RbacError::AclNotFound);
        }

        // Validate ACL before adding
        acl.validate()?;

        self.acls.push(acl);
        Ok(())
    }

    /// Remove ACL for a resource
    pub fn remove_acl(&mut self, resource_id: &str) -> Result<(), RbacError> {
        let original_len = self.acls.len();
        self.acls.retain(|acl| acl.resource_id != resource_id);
        
        if self.acls.len() == original_len {
            Err(RbacError::AclNotFound)
        } else {
            Ok(())
        }
    }

    /// Get ACL for a resource
    pub fn get_acl(&self, resource_id: &str) -> Option<&AccessControlList> {
        self.acls.iter().find(|acl| acl.resource_id == resource_id)
    }

    /// Check permission using ACL
    pub fn check_permission(&self, resource_id: &str, principal_type: PrincipalType,
                           principal_id: u32, permission: RbacPermission, context: &AclContext) -> bool {
        if let Some(acl) = self.get_acl(resource_id) {
            acl.has_permission(principal_type, principal_id, permission, context)
        } else {
            // If no ACL exists, apply default permissions
            self.check_default_permission(principal_type, principal_id, permission)
        }
    }

    /// Check default permission when no ACL exists
    fn check_default_permission(&self, principal_type: PrincipalType, principal_id: u32, permission: RbacPermission) -> bool {
        match principal_type {
            PrincipalType::User => {
                // Default: users have read/write/execute on their own resources
                match permission {
                    RbacPermission::Read | RbacPermission::Write | RbacPermission::Execute => true,
                    _ => false,
                }
            },
            PrincipalType::Everyone => {
                // Default: everyone has read permission on public resources
                matches!(permission, RbacPermission::Read)
            },
            _ => false,
        }
    }

    /// Get effective permissions for user on resource
    pub fn get_effective_permissions(&self, resource_id: &str, user_id: UserId) -> BTreeSet<RbacPermission> {
        if let Some(acl) = self.get_acl(resource_id) {
            acl.get_effective_permissions(user_id)
        } else {
            // Return default permissions
            let mut permissions = BTreeSet::new();
            permissions.insert(RbacPermission::Read);
            permissions
        }
    }

    /// List all ACLs
    pub fn list_acls(&self) -> Vec<&AccessControlList> {
        self.acls.iter().collect()
    }

    /// Get ACL count
    pub fn acl_count(&self) -> usize {
        self.acls.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_acl_creation() {
        let entries = vec![
            AclEntry {
                entry_id: "user_read".to_string(),
                principal_type: PrincipalType::User,
                principal_id: 1000,
                permissions: vec![RbacPermission::Read],
                conditions: vec![],
                effective: true,
                inherited: false,
                priority: 100,
                expires_at: None,
            }
        ];

        let acl = AccessControlList::new("test_resource".to_string(), entries);
        assert_eq!(acl.resource_id, "test_resource");
        assert_eq!(acl.entry_count(), 1);
    }

    #[test]
    fn test_permission_checking() {
        let entries = vec![
            AclEntry {
                entry_id: "user_full_access".to_string(),
                principal_type: PrincipalType::User,
                principal_id: 1000,
                permissions: vec![RbacPermission::Read, RbacPermission::Write],
                conditions: vec![],
                effective: true,
                inherited: false,
                priority: 100,
                expires_at: None,
            }
        ];

        let acl = AccessControlList::new("test_resource".to_string(), entries);
        let context = AclContext::new(Some(1000));

        let has_read = acl.has_permission(PrincipalType::User, 1000, RbacPermission::Read, &context);
        let has_write = acl.has_permission(PrincipalType::User, 1000, RbacPermission::Write, &context);
        let has_execute = acl.has_permission(PrincipalType::User, 1000, RbacPermission::Execute, &context);

        assert!(has_read);
        assert!(has_write);
        assert!(!has_execute);
    }

    #[test]
    fn test_acl_manager() {
        let mut manager = AclManager::new();
        
        let entries = vec![
            AclEntry {
                entry_id: "admin_all".to_string(),
                principal_type: PrincipalType::User,
                principal_id: 0,
                permissions: vec![RbacPermission::All],
                conditions: vec![],
                effective: true,
                inherited: false,
                priority: 200,
                expires_at: None,
            }
        ];

        let acl = AccessControlList::new("/system/config".to_string(), entries);
        let result = manager.add_acl(acl);
        assert!(result.is_ok());
        assert_eq!(manager.acl_count(), 1);
    }
}
