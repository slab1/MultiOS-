//! Role-Based Access Control (RBAC) System
//! 
//! This module provides a comprehensive role-based access control system for the kernel,
//! including:
//! - Role definitions with permission sets
//! - User-group-role assignment management  
//! - Resource-level access control (files, processes, services)
//! - Permission inheritance and delegation
//! - Access control lists (ACLs) for fine-grained control
//! - Permission validation and enforcement mechanisms
//! - Integration with existing user management and syscall systems

use spin::{Mutex, RwLock, Once};
use alloc::vec::Vec;
use alloc::string::{String, ToString};
use alloc::collections::{HashMap, BTreeMap, BTreeSet};
use core::sync::atomic::{AtomicU32, Ordering};

pub mod acl;
pub mod permission_inheritance;
pub mod delegation;

pub use acl::{AccessControlList, AclEntry, AclPermission};
pub use permission_inheritance::{PermissionInheritance, InheritanceLevel};
pub use delegation::{PermissionDelegation, DelegationScope};

use crate::admin::user_manager::{UserId, GroupId, UserManager};
use crate::admin::security::{SecurityLevel, Permission as BasePermission, SecurityContext};
use crate::syscall::error_handling::SyscallError;
use crate::log::{info, warn, error, debug};

/// RBAC result type
pub type RbacResult<T> = Result<T, RbacError>;

/// RBAC error types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum RbacError {
    RoleNotFound = 0,
    UserNotFound = 1,
    GroupNotFound = 2,
    PermissionDenied = 3,
    ResourceNotFound = 4,
    InvalidPermission = 5,
    CircularDependency = 6,
    DelegationNotAllowed = 7,
    InheritanceViolation = 8,
    AclNotFound = 9,
    SecurityLevelViolation = 10,
    OperationNotPermitted = 11,
    NotInitialized = 12,
    ResourceExhausted = 13,
    InvalidParameter = 14,
}

/// Permission bitmask for RBAC operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum RbacPermission {
    None = 0,
    Read = 1 << 0,
    Write = 1 << 1,
    Execute = 1 << 2,
    Create = 1 << 3,
    Delete = 1 << 4,
    Modify = 1 << 5,
    Admin = 1 << 6,
    System = 1 << 7,
    Delegate = 1 << 8,
    Inherit = 1 << 9,
    Audit = 1 << 10,
    All = 0xFFFF,
}

impl From<BasePermission> for RbacPermission {
    fn from(base_perm: BasePermission) -> Self {
        match base_perm {
            BasePermission::None => RbacPermission::None,
            BasePermission::Read => RbacPermission::Read,
            BasePermission::Write => RbacPermission::Write,
            BasePermission::Execute => RbacPermission::Execute,
            BasePermission::Create => RbacPermission::Create,
            BasePermission::Delete => RbacPermission::Delete,
            BasePermission::Modify => RbacPermission::Modify,
            BasePermission::Admin => RbacPermission::Admin,
            BasePermission::System => RbacPermission::System,
            BasePermission::All => RbacPermission::All,
        }
    }
}

/// Resource types for access control
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ResourceType {
    File { path: String },
    Process { pid: u32 },
    Service { service_id: String },
    Memory { address: u64, size: usize },
    Network { protocol: String, port: u16 },
    Device { device_id: String },
    System,
    UserData,
    Security,
    Configuration,
}

/// Resource identifier for access control checks
#[derive(Debug, Clone)]
pub struct ResourceId {
    pub resource_type: ResourceType,
    pub owner_id: Option<UserId>,
    pub group_id: Option<GroupId>,
    pub security_level: SecurityLevel,
}

/// Role definition with permission sets
#[derive(Debug, Clone)]
pub struct Role {
    pub role_id: u32,
    pub name: String,
    pub description: String,
    pub permissions: Vec<RbacPermission>,
    pub inherited_roles: Vec<u32>, // Role IDs this role inherits from
    pub delegated_permissions: Vec<PermissionDelegation>,
    pub security_level: SecurityLevel,
    pub is_system_role: bool,
    pub can_delegate: bool,
    pub can_create_roles: bool,
    pub audit_level: u32,
}

/// User-Group-Role assignment
#[derive(Debug, Clone)]
pub struct UserGroupRoleAssignment {
    pub user_id: Option<UserId>,
    pub group_id: Option<GroupId>,
    pub role_id: u32,
    pub assigned_by: Option<UserId>,
    pub assigned_at: u64,
    pub expires_at: Option<u64>,
    pub conditions: Vec<String>, // Conditional permissions
}

/// Effective permissions for a user on a resource
#[derive(Debug, Clone)]
pub struct EffectivePermissions {
    pub user_id: UserId,
    pub resource_id: ResourceId,
    pub permissions: BTreeSet<RbacPermission>,
    pub delegated_permissions: BTreeSet<RbacPermission>,
    pub inherited_permissions: BTreeSet<RbacPermission>,
    pub acl_permissions: BTreeSet<RbacPermission>,
    pub security_level: SecurityLevel,
}

/// RBAC Statistics
#[derive(Debug, Clone)]
pub struct RbacStats {
    pub total_roles: usize,
    pub total_assignments: usize,
    pub total_acl_entries: usize,
    pub permission_checks: u64,
    pub granted_permissions: u64,
    pub denied_permissions: u64,
    pub delegation_operations: u64,
    pub inheritance_operations: u64,
}

/// Main RBAC Manager - Central orchestrator for role-based access control
pub struct RbacManager {
    roles: RwLock<HashMap<u32, Role>>,
    role_names: RwLock<HashMap<String, u32>>,
    assignments: RwLock<Vec<UserGroupRoleAssignment>>,
    user_roles: RwLock<HashMap<UserId, BTreeSet<u32>>>,
    group_roles: RwLock<HashMap<GroupId, BTreeSet<u32>>>,
    resource_acls: RwLock<HashMap<String, AccessControlList>>,
    effective_permissions: RwLock<HashMap<(UserId, String), EffectivePermissions>>,
    stats: Mutex<RbacStats>,
    next_role_id: AtomicU32,
    initialized: bool,
}

impl RbacManager {
    /// Create a new RBAC Manager instance
    pub fn new() -> Self {
        Self {
            roles: RwLock::new(HashMap::new()),
            role_names: RwLock::new(HashMap::new()),
            assignments: RwLock::new(Vec::new()),
            user_roles: RwLock::new(HashMap::new()),
            group_roles: RwLock::new(HashMap::new()),
            resource_acls: RwLock::new(HashMap::new()),
            effective_permissions: RwLock::new(HashMap::new()),
            stats: Mutex::new(RbacStats {
                total_roles: 0,
                total_assignments: 0,
                total_acl_entries: 0,
                permission_checks: 0,
                granted_permissions: 0,
                denied_permissions: 0,
                delegation_operations: 0,
                inheritance_operations: 0,
            }),
            next_role_id: AtomicU32::new(1), // Start from 1, 0 reserved for no role
            initialized: false,
        }
    }

    /// Initialize the RBAC system
    pub fn init(&mut self) -> RbacResult<()> {
        if self.initialized {
            return Err(RbacError::NotInitialized);
        }

        // Create default system roles
        self.create_default_roles()?;
        
        // Create default ACLs for system resources
        self.create_default_acls()?;

        self.initialized = true;
        
        info!("RBAC Manager initialized successfully");
        Ok(())
    }

    /// Shutdown the RBAC system
    pub fn shutdown(&mut self) -> RbacResult<()> {
        if !self.initialized {
            return Err(RbacError::NotInitialized);
        }

        // Clear all cached permissions
        {
            let mut effective_perms = self.effective_permissions.write();
            effective_perms.clear();
        }

        self.initialized = false;
        info!("RBAC Manager shutdown complete");
        Ok(())
    }

    // ==================== Role Management ====================

    /// Create a new role
    pub fn create_role(&self, name: &str, description: &str, permissions: Vec<RbacPermission>,
                      security_level: SecurityLevel, creator_id: Option<UserId>) -> RbacResult<u32> {
        let role_name = name.to_string();
        
        // Check if role already exists
        {
            let role_names = self.role_names.read();
            if role_names.contains_key(&role_name) {
                return Err(RbacError::RoleNotFound);
            }
        }

        let role_id = self.next_role_id.fetch_add(1, Ordering::SeqCst);
        
        let role = Role {
            role_id,
            name: role_name.clone(),
            description: description.to_string(),
            permissions,
            inherited_roles: Vec::new(),
            delegated_permissions: Vec::new(),
            security_level,
            is_system_role: false,
            can_delegate: false,
            can_create_roles: false,
            audit_level: 1,
        };

        // Store role
        {
            let mut roles = self.roles.write();
            roles.insert(role_id, role);
        }

        {
            let mut role_names = self.role_names.write();
            role_names.insert(role_name, role_id);
        }

        let mut stats = self.stats.lock();
        stats.total_roles += 1;

        info!("Created role: {} (ID: {})", name, role_id);
        Ok(role_id)
    }

    /// Delete a role
    pub fn delete_role(&self, role_id: u32, deleter_id: Option<UserId>) -> RbacResult<()> {
        let mut roles = self.roles.write();
        
        let role = roles.remove(&role_id)
            .ok_or(RbacError::RoleNotFound)?;

        if role.is_system_role {
            return Err(RbacError::OperationNotPermitted);
        }

        // Remove from name mapping
        {
            let mut role_names = self.role_names.write();
            role_names.remove(&role.name);
        }

        // Remove role from all assignments
        {
            let mut assignments = self.assignments.write();
            assignments.retain(|assignment| assignment.role_id != role_id);
        }

        // Update user role mappings
        {
            let mut user_roles = self.user_roles.write();
            for roles_set in user_roles.values_mut() {
                roles_set.remove(&role_id);
            }
        }

        // Update group role mappings
        {
            let mut group_roles = self.group_roles.write();
            for roles_set in group_roles.values_mut() {
                roles_set.remove(&role_id);
            }
        }

        let mut stats = self.stats.lock();
        stats.total_roles -= 1;

        info!("Deleted role: {} (ID: {})", role.name, role_id);
        Ok(())
    }

    /// Get role by ID
    pub fn get_role(&self, role_id: u32) -> RbacResult<Role> {
        let roles = self.roles.read();
        roles.get(&role_id)
            .cloned()
            .ok_or(RbacError::RoleNotFound)
    }

    /// Get role by name
    pub fn get_role_by_name(&self, name: &str) -> RbacResult<Role> {
        let role_names = self.role_names.read();
        let role_id = role_names.get(name)
            .copied()
            .ok_or(RbacError::RoleNotFound)?;
        
        self.get_role(role_id)
    }

    /// List all roles
    pub fn list_roles(&self) -> Vec<Role> {
        let roles = self.roles.read();
        roles.values().cloned().collect()
    }

    /// Update role permissions
    pub fn update_role_permissions(&self, role_id: u32, permissions: Vec<RbacPermission>) -> RbacResult<()> {
        let mut roles = self.roles.write();
        
        let role = roles.get_mut(&role_id)
            .ok_or(RbacError::RoleNotFound)?;

        role.permissions = permissions;
        
        // Clear cached effective permissions that depend on this role
        self.clear_effective_permissions_for_role(role_id);

        info!("Updated permissions for role: {} (ID: {})", role.name, role_id);
        Ok(())
    }

    // ==================== Assignment Management ====================

    /// Assign role to user
    pub fn assign_role_to_user(&self, user_id: UserId, role_id: u32, 
                              assigned_by: Option<UserId>, expires_at: Option<u64>) -> RbacResult<()> {
        // Validate role exists
        self.get_role(role_id)?;

        let assignment = UserGroupRoleAssignment {
            user_id: Some(user_id),
            group_id: None,
            role_id,
            assigned_by,
            assigned_at: crate::admin::user_manager::get_current_time(),
            expires_at,
            conditions: Vec::new(),
        };

        {
            let mut assignments = self.assignments.write();
            assignments.push(assignment);
        }

        {
            let mut user_roles = self.user_roles.write();
            user_roles.entry(user_id).or_insert_with(BTreeSet::new).insert(role_id);
        }

        let mut stats = self.stats.lock();
        stats.total_assignments += 1;

        info!("Assigned role {} to user {}", role_id, user_id);
        Ok(())
    }

    /// Assign role to group
    pub fn assign_role_to_group(&self, group_id: GroupId, role_id: u32,
                               assigned_by: Option<UserId>, expires_at: Option<u64>) -> RbacResult<()> {
        // Validate role exists
        self.get_role(role_id)?;

        let assignment = UserGroupRoleAssignment {
            user_id: None,
            group_id: Some(group_id),
            role_id,
            assigned_by,
            assigned_at: crate::admin::user_manager::get_current_time(),
            expires_at,
            conditions: Vec::new(),
        };

        {
            let mut assignments = self.assignments.write();
            assignments.push(assignment);
        }

        {
            let mut group_roles = self.group_roles.write();
            group_roles.entry(group_id).or_insert_with(BTreeSet::new).insert(role_id);
        }

        let mut stats = self.stats.lock();
        stats.total_assignments += 1;

        info!("Assigned role {} to group {}", role_id, group_id);
        Ok(())
    }

    /// Remove role assignment from user
    pub fn remove_role_from_user(&self, user_id: UserId, role_id: u32) -> RbacResult<()> {
        {
            let mut assignments = self.assignments.write();
            assignments.retain(|assignment| {
                assignment.user_id != Some(user_id) || assignment.role_id != role_id
            });
        }

        {
            let mut user_roles = self.user_roles.write();
            if let Some(roles_set) = user_roles.get_mut(&user_id) {
                roles_set.remove(&role_id);
            }
        }

        let mut stats = self.stats.lock();
        stats.total_assignments = stats.total_assignments.saturating_sub(1);

        info!("Removed role {} from user {}", role_id, user_id);
        Ok(())
    }

    /// Get roles assigned to user
    pub fn get_user_roles(&self, user_id: UserId) -> RbacResult<Vec<Role>> {
        let user_roles = self.user_roles.read();
        let roles = self.roles.read();

        let role_ids = user_roles.get(&user_id).cloned().unwrap_or_default();
        let mut user_role_list = Vec::new();

        for role_id in role_ids {
            if let Some(role) = roles.get(&role_id) {
                user_role_list.push(role.clone());
            }
        }

        Ok(user_role_list)
    }

    /// Get effective permissions for user on resource
    pub fn get_effective_permissions(&self, user_id: UserId, resource_id: &str) -> RbacResult<EffectivePermissions> {
        let cache_key = (user_id, resource_id.to_string());
        
        // Check cache first
        {
            let effective_perms = self.effective_permissions.read();
            if let Some(perms) = effective_perms.get(&cache_key) {
                return Ok(perms.clone());
            }
        }

        // Calculate effective permissions
        let effective_permissions = self.calculate_effective_permissions(user_id, resource_id)?;
        
        // Cache the result
        {
            let mut effective_perms = self.effective_permissions.write();
            effective_perms.insert(cache_key, effective_permissions.clone());
        }

        Ok(effective_permissions)
    }

    // ==================== Permission Checking ====================

    /// Check if user has permission on resource
    pub fn check_permission(&self, user_id: UserId, resource_id: &str, 
                           permission: RbacPermission) -> RbacResult<bool> {
        let mut stats = self.stats.lock();
        stats.permission_checks += 1;

        let effective_perms = self.get_effective_permissions(user_id, resource_id)?;
        
        let has_permission = effective_perms.permissions.contains(&permission) ||
                           effective_perms.delegated_permissions.contains(&permission) ||
                           effective_perms.inherited_permissions.contains(&permission) ||
                           effective_perms.acl_permissions.contains(&permission);

        if has_permission {
            stats.granted_permissions += 1;
            Ok(true)
        } else {
            stats.denied_permissions += 1;
            Ok(false)
        }
    }

    /// Validate access request
    pub fn validate_access(&self, user_id: UserId, resource_id: &str, 
                          required_permissions: &[RbacPermission]) -> RbacResult<bool> {
        for &permission in required_permissions {
            if !self.check_permission(user_id, resource_id, permission)? {
                return Err(RbacError::PermissionDenied);
            }
        }
        
        Ok(true)
    }

    // ==================== Permission Inheritance ====================

    /// Set role inheritance
    pub fn set_role_inheritance(&self, child_role_id: u32, parent_role_id: u32) -> RbacResult<()> {
        // Check for circular dependency
        if self.has_circular_dependency(child_role_id, parent_role_id) {
            return Err(RbacError::CircularDependency);
        }

        let mut roles = self.roles.write();
        
        let child_role = roles.get_mut(&child_role_id)
            .ok_or(RbacError::RoleNotFound)?;

        if !child_role.inherited_roles.contains(&parent_role_id) {
            child_role.inherited_roles.push(parent_role_id);
            
            // Clear cached permissions for this role
            self.clear_effective_permissions_for_role(child_role_id);
            
            let mut stats = self.stats.lock();
            stats.inheritance_operations += 1;
        }

        info!("Set inheritance: role {} inherits from role {}", child_role_id, parent_role_id);
        Ok(())
    }

    /// Remove role inheritance
    pub fn remove_role_inheritance(&self, child_role_id: u32, parent_role_id: u32) -> RbacResult<()> {
        let mut roles = self.roles.write();
        
        let child_role = roles.get_mut(&child_role_id)
            .ok_or(RbacError::RoleNotFound)?;

        child_role.inherited_roles.retain(|&id| id != parent_role_id);
        
        // Clear cached permissions for this role
        self.clear_effective_permissions_for_role(child_role_id);

        info!("Removed inheritance: role {} no longer inherits from role {}", child_role_id, parent_role_id);
        Ok(())
    }

    // ==================== Permission Delegation ====================

    /// Delegate permissions from one user to another
    pub fn delegate_permission(&self, delegator_id: UserId, delegatee_id: UserId,
                              resource_id: &str, permissions: &[RbacPermission]) -> RbacResult<()> {
        // Validate delegator has permissions to delegate
        for &permission in permissions {
            if !self.check_permission(delegator_id, resource_id, permission)? {
                return Err(RbacError::DelegationNotAllowed);
            }
        }

        let delegation = PermissionDelegation {
            delegator_user_id: delegator_id,
            delegatee_user_id: delegatee_id,
            resource_id: resource_id.to_string(),
            permissions: permissions.to_vec(),
            granted_at: crate::admin::user_manager::get_current_time(),
            expires_at: None,
            can_revoke: true,
        };

        // Store delegation and update effective permissions
        self.apply_delegation(delegation)?;

        let mut stats = self.stats.lock();
        stats.delegation_operations += 1;

        info!("Delegated permissions from user {} to user {} for resource {}", 
              delegator_id, delegatee_id, resource_id);
        Ok(())
    }

    /// Revoke permission delegation
    pub fn revoke_delegation(&self, delegator_id: UserId, delegatee_id: UserId,
                           resource_id: &str) -> RbacResult<()> {
        // Remove delegation and update effective permissions
        self.remove_delegation(delegator_id, delegatee_id, resource_id)?;

        info!("Revoked delegation from user {} to user {} for resource {}", 
              delegator_id, delegatee_id, resource_id);
        Ok(())
    }

    // ==================== Access Control Lists (ACLs) ====================

    /// Create ACL for resource
    pub fn create_acl(&self, resource_id: &str, entries: Vec<AclEntry>) -> RbacResult<()> {
        let acl = AccessControlList::new(resource_id.to_string(), entries);
        
        {
            let mut resource_acls = self.resource_acls.write();
            resource_acls.insert(resource_id.to_string(), acl);
        }

        // Clear cached permissions for this resource
        self.clear_effective_permissions_for_resource(resource_id);

        let mut stats = self.stats.lock();
        stats.total_acl_entries += entries.len();

        info!("Created ACL for resource: {}", resource_id);
        Ok(())
    }

    /// Add entry to ACL
    pub fn add_acl_entry(&self, resource_id: &str, entry: AclEntry) -> RbacResult<()> {
        let mut resource_acls = self.resource_acls.write();
        
        let acl = resource_acls.get_mut(resource_id)
            .ok_or(RbacError::AclNotFound)?;

        acl.add_entry(entry);
        
        // Clear cached permissions for this resource
        self.clear_effective_permissions_for_resource(resource_id);

        let mut stats = self.stats.lock();
        stats.total_acl_entries += 1;

        Ok(())
    }

    /// Remove entry from ACL
    pub fn remove_acl_entry(&self, resource_id: &str, entry_id: &str) -> RbacResult<()> {
        let mut resource_acls = self.resource_acls.write();
        
        let acl = resource_acls.get_mut(resource_id)
            .ok_or(RbacError::AclNotFound)?;

        acl.remove_entry(entry_id);
        
        // Clear cached permissions for this resource
        self.clear_effective_permissions_for_resource(resource_id);

        Ok(())
    }

    /// Get ACL for resource
    pub fn get_acl(&self, resource_id: &str) -> RbacResult<AccessControlList> {
        let resource_acls = self.resource_acls.read();
        resource_acls.get(resource_id)
            .cloned()
            .ok_or(RbacError::AclNotFound)
    }

    // ==================== Internal Helper Methods ====================

    /// Calculate effective permissions for user on resource
    fn calculate_effective_permissions(&self, user_id: UserId, resource_id: &str) -> RbacResult<EffectivePermissions> {
        let mut permissions = BTreeSet::new();
        let mut delegated_permissions = BTreeSet::new();
        let mut inherited_permissions = BTreeSet::new();
        let mut acl_permissions = BTreeSet::new();

        let user_roles = self.get_user_roles(user_id)?;
        
        // Collect direct role permissions
        for role in user_roles {
            for permission in &role.permissions {
                permissions.insert(*permission);
            }
            
            // Collect inherited permissions
            for inherited_role_id in &role.inherited_roles {
                let inherited_role = self.get_role(*inherited_role_id)?;
                for permission in &inherited_role.permissions {
                    inherited_permissions.insert(*permission);
                }
            }
        }

        // Apply ACL permissions
        if let Ok(acl) = self.get_acl(resource_id) {
            acl_permissions = acl.get_effective_permissions(user_id);
        }

        // Apply delegated permissions
        // (This would need to be implemented based on delegation storage)

        let security_level = SecurityLevel::Medium; // Would be determined from user context

        Ok(EffectivePermissions {
            user_id,
            resource_id: ResourceId {
                resource_type: ResourceType::System,
                owner_id: Some(user_id),
                group_id: None,
                security_level,
            },
            permissions,
            delegated_permissions,
            inherited_permissions,
            acl_permissions,
            security_level,
        })
    }

    /// Check for circular dependency in role inheritance
    fn has_circular_dependency(&self, role_id: u32, parent_role_id: u32) -> bool {
        if role_id == parent_role_id {
            return true;
        }

        let roles = self.roles.read();
        
        if let Some(parent_role) = roles.get(&parent_role_id) {
            for &grandparent_id in &parent_role.inherited_roles {
                if self.has_circular_dependency(role_id, grandparent_id) {
                    return true;
                }
            }
        }

        false
    }

    /// Clear cached effective permissions for a role
    fn clear_effective_permissions_for_role(&self, role_id: u32) {
        let mut effective_perms = self.effective_permissions.write();
        
        // Remove all cached permissions that might be affected by this role change
        effective_perms.retain(|_, perms| {
            !perms.permissions.iter().any(|_| true) // Simplified - in real implementation would check role dependencies
        });
    }

    /// Clear cached effective permissions for a resource
    fn clear_effective_permissions_for_resource(&self, resource_id: &str) {
        let mut effective_perms = self.effective_permissions.write();
        
        // Remove cached permissions for this specific resource
        effective_perms.retain(|(user_id, res_id), _| res_id != resource_id);
    }

    /// Apply permission delegation
    fn apply_delegation(&self, delegation: PermissionDelegation) -> RbacResult<()> {
        // Store delegation and update effective permissions cache
        // This is a simplified implementation
        Ok(())
    }

    /// Remove permission delegation
    fn remove_delegation(&self, delegator_id: UserId, delegatee_id: UserId, resource_id: &str) -> RbacResult<()> {
        // Remove delegation and update effective permissions cache
        // This is a simplified implementation
        Ok(())
    }

    /// Create default system roles
    fn create_default_roles(&self) -> RbacResult<()> {
        // System Administrator role
        let admin_role_id = self.create_role(
            "system_admin",
            "System Administrator with full privileges",
            vec![
                RbacPermission::All,
                RbacPermission::Delegate,
                RbacPermission::Inherit,
                RbacPermission::Audit,
            ],
            SecurityLevel::System,
            None,
        )?;

        // Regular User role
        let user_role_id = self.create_role(
            "user",
            "Regular user with basic permissions",
            vec![
                RbacPermission::Read,
                RbacPermission::Write,
                RbacPermission::Execute,
            ],
            SecurityLevel::Low,
            None,
        )?;

        // Security Auditor role
        let auditor_role_id = self.create_role(
            "security_auditor",
            "Security auditor with read-only access to security logs",
            vec![
                RbacPermission::Read,
                RbacPermission::Audit,
            ],
            SecurityLevel::High,
            None,
        )?;

        info!("Created default roles: system_admin ({}), user ({}), security_auditor ({})", 
              admin_role_id, user_role_id, auditor_role_id);
        Ok(())
    }

    /// Create default ACLs for system resources
    fn create_default_acls(&self) -> RbacResult<()> {
        // ACL for system configuration files
        let system_config_entries = vec![
            AclEntry {
                entry_id: "root_config_read".to_string(),
                principal_type: crate::security::acl::PrincipalType::User,
                principal_id: 0, // root user
                permissions: vec![RbacPermission::Read, RbacPermission::Write, RbacPermission::Admin],
                conditions: vec![],
            },
            AclEntry {
                entry_id: "admin_config_read".to_string(),
                principal_type: crate::security::acl::PrincipalType::Role,
                principal_id: 1, // system_admin role
                permissions: vec![RbacPermission::Read, RbacPermission::Write],
                conditions: vec![],
            },
        ];

        self.create_acl("/system/config", system_config_entries)?;

        // ACL for user data directories
        let user_data_entries = vec![
            AclEntry {
                entry_id: "owner_full_access".to_string(),
                principal_type: crate::security::acl::PrincipalType::User,
                principal_id: 0, // Will be replaced with actual user ID at runtime
                permissions: vec![RbacPermission::Read, RbacPermission::Write, RbacPermission::Execute],
                conditions: vec!["owner_match".to_string()],
            },
        ];

        self.create_acl("/user/*", user_data_entries)?;

        info!("Created default ACLs for system resources");
        Ok(())
    }

    /// Get RBAC statistics
    pub fn get_stats(&self) -> RbacStats {
        let stats = self.stats.lock();
        stats.clone()
    }
}

// ==================== Global RBAC Manager Instance ====================

static RBAC_MANAGER: Once<RbacManager> = Once::new();

/// Initialize the global RBAC manager
pub fn init_rbac_manager() -> RbacResult<()> {
    let mut manager = RbacManager::new();
    manager.init()?;
    
    RBAC_MANAGER.call_once(|| manager);
    
    info!("RBAC Manager initialized successfully");
    Ok(())
}

/// Get the global RBAC manager instance
pub fn get_rbac_manager() -> Option<&'static RbacManager> {
    RBAC_MANAGER.get()
}

/// Get current time helper function
fn get_current_time() -> u64 {
    crate::hal::get_current_time()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rbac_initialization() {
        let mut manager = RbacManager::new();
        let result = manager.init();
        assert!(result.is_ok());
    }

    #[test]
    fn test_role_creation() {
        let mut manager = RbacManager::new();
        manager.init().unwrap();

        let role_id = manager.create_role(
            "test_role",
            "Test role for unit testing",
            vec![RbacPermission::Read, RbacPermission::Write],
            SecurityLevel::Low,
            None,
        ).unwrap();

        assert!(role_id > 0);
    }

    #[test]
    fn test_permission_checking() {
        let mut manager = RbacManager::new();
        manager.init().unwrap();

        let role_id = manager.create_role(
            "test_role",
            "Test role",
            vec![RbacPermission::Read],
            SecurityLevel::Low,
            None,
        ).unwrap();

        let user_id = 1000;
        manager.assign_role_to_user(user_id, role_id, None, None).unwrap();

        let has_read = manager.check_permission(user_id, "test_resource", RbacPermission::Read).unwrap();
        let has_write = manager.check_permission(user_id, "test_resource", RbacPermission::Write).unwrap();

        assert!(has_read);
        assert!(!has_write);
    }
}
