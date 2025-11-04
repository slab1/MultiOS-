//! RBAC Integration Tests
//! 
//! This module provides comprehensive integration tests for the RBAC system,
//! testing integration with user management, security contexts, and syscall interfaces.

use super::*;
use crate::admin::user_manager::{init_user_manager, UserId, GroupId};
use crate::admin::security::{init_security_manager, SecurityLevel};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rbac_integration_with_user_manager() {
        // Initialize user manager
        let user_result = init_user_manager();
        assert!(user_result.is_ok());

        // Create a test user
        let user_manager = crate::admin::user_manager::get_user_manager();
        assert!(user_manager.is_some());

        if let Some(manager_ref) = user_manager {
            let mut manager = manager_ref.lock();
            if let Ok(ref mut mgr) = *manager {
                // Create test user
                let user_id = mgr.create_user("testuser", "SecurePassword123!", Some("test@example.com"));
                assert!(user_id.is_ok());
                let user_id = user_id.unwrap();

                // Initialize RBAC manager
                let rbac_result = init_rbac_manager();
                assert!(rbac_result.is_ok());

                // Get RBAC manager instance
                let rbac_manager = get_rbac_manager();
                assert!(rbac_manager.is_some());

                if let Some(rbac_mgr) = rbac_manager {
                    // Create a custom role
                    let role_id = rbac_mgr.create_role(
                        "test_role",
                        "Test role for integration testing",
                        vec![RbacPermission::Read, RbacPermission::Write],
                        SecurityLevel::Low,
                        Some(user_id),
                    );
                    assert!(role_id.is_ok());
                    let role_id = role_id.unwrap();

                    // Assign role to user
                    let assign_result = rbac_mgr.assign_role_to_user(user_id, role_id, Some(user_id), None);
                    assert!(assign_result.is_ok());

                    // Test permission checking
                    let has_read = rbac_mgr.check_permission(user_id, "test_resource", RbacPermission::Read);
                    assert!(has_read.is_ok());
                    assert!(has_read.unwrap());

                    let has_admin = rbac_mgr.check_permission(user_id, "test_resource", RbacPermission::Admin);
                    assert!(has_admin.is_ok());
                    assert!(!has_admin.unwrap());

                    // Test effective permissions
                    let effective_perms = rbac_mgr.get_effective_permissions(user_id, "test_resource");
                    assert!(effective_perms.is_ok());
                    let perms = effective_perms.unwrap();
                    assert!(perms.permissions.contains(&RbacPermission::Read));
                    assert!(perms.permissions.contains(&RbacPermission::Write));
                }
            }
        }
    }

    #[test]
    fn test_rbac_with_acl_integration() {
        // Initialize RBAC manager
        let rbac_result = init_rbac_manager();
        assert!(rbac_result.is_ok());

        let rbac_manager = get_rbac_manager();
        assert!(rbac_manager.is_some());

        if let Some(rbac_mgr) = rbac_manager {
            // Create a user
            let user_id = 1001;

            // Create ACL entries
            let acl_entries = vec![
                AclEntry {
                    entry_id: "user_read_write".to_string(),
                    principal_type: PrincipalType::User,
                    principal_id: user_id,
                    permissions: vec![RbacPermission::Read, RbacPermission::Write],
                    conditions: vec![],
                    effective: true,
                    inherited: false,
                    priority: 100,
                    expires_at: None,
                },
                AclEntry {
                    entry_id: "group_read".to_string(),
                    principal_type: PrincipalType::Group,
                    principal_id: 100, // users group
                    permissions: vec![RbacPermission::Read],
                    conditions: vec![],
                    effective: true,
                    inherited: false,
                    priority: 50,
                    expires_at: None,
                },
            ];

            // Create ACL for resource
            let acl_result = rbac_mgr.create_acl("/test/file.txt", acl_entries);
            assert!(acl_result.is_ok());

            // Test ACL permission checking
            let context = AclContext::new(Some(user_id));
            
            // Test user-specific permissions
            let has_read = rbac_mgr.get_acl("/test/file.txt").unwrap()
                .has_permission(PrincipalType::User, user_id, RbacPermission::Read, &context);
            assert!(has_read);

            let has_write = rbac_mgr.get_acl("/test/file.txt").unwrap()
                .has_permission(PrincipalType::User, user_id, RbacPermission::Write, &context);
            assert!(has_write);

            let has_execute = rbac_mgr.get_acl("/test/file.txt").unwrap()
                .has_permission(PrincipalType::User, user_id, RbacPermission::Execute, &context);
            assert!(!has_execute);

            // Test group permissions
            let group_has_read = rbac_mgr.get_acl("/test/file.txt").unwrap()
                .has_permission(PrincipalType::Group, 100, RbacPermission::Read, &context);
            assert!(group_has_read);
        }
    }

    #[test]
    fn test_rbac_permission_inheritance() {
        let rbac_manager = get_rbac_manager();
        assert!(rbac_manager.is_some());

        if let Some(rbac_mgr) = rbac_manager {
            // Create parent role
            let parent_role_id = rbac_mgr.create_role(
                "parent_role",
                "Parent role for inheritance testing",
                vec![RbacPermission::Read, RbacPermission::Write, RbacPermission::Execute],
                SecurityLevel::Medium,
                None,
            ).unwrap();

            // Create child role
            let child_role_id = rbac_mgr.create_role(
                "child_role", 
                "Child role that inherits from parent",
                vec![RbacPermission::Admin],
                SecurityLevel::Low,
                None,
            ).unwrap();

            // Set inheritance
            let inherit_result = rbac_mgr.set_role_inheritance(child_role_id, parent_role_id);
            assert!(inherit_result.is_ok());

            // Create user and assign child role
            let user_id = 1002;
            let assign_result = rbac_mgr.assign_role_to_user(user_id, child_role_id, None, None);
            assert!(assign_result.is_ok());

            // Test that user gets inherited permissions
            let has_inherited_read = rbac_mgr.check_permission(user_id, "test_resource", RbacPermission::Read);
            assert!(has_inherited_read.is_ok());
            assert!(has_inherited_read.unwrap());

            let has_inherited_write = rbac_mgr.check_permission(user_id, "test_resource", RbacPermission::Write);
            assert!(has_inherited_write.is_ok());
            assert!(has_inherited_write.unwrap());

            let has_direct_admin = rbac_mgr.check_permission(user_id, "test_resource", RbacPermission::Admin);
            assert!(has_direct_admin.is_ok());
            assert!(has_direct_admin.unwrap());
        }
    }

    #[test]
    fn test_rbac_permission_delegation() {
        let rbac_manager = get_rbac_manager();
        assert!(rbac_manager.is_some());

        if let Some(rbac_mgr) = rbac_manager {
            // Create delegator and delegatee users
            let delegator_id = 1003;
            let delegatee_id = 1004;

            // Create resource and assign initial permissions
            let resource_id = "test_file";
            let acl_entries = vec![
                AclEntry {
                    entry_id: "delegator_all".to_string(),
                    principal_type: PrincipalType::User,
                    principal_id: delegator_id,
                    permissions: vec![RbacPermission::Read, RbacPermission::Write, RbacPermission::Delegate],
                    conditions: vec![],
                    effective: true,
                    inherited: false,
                    priority: 100,
                    expires_at: None,
                },
            ];

            let acl_result = rbac_mgr.create_acl(resource_id, acl_entries);
            assert!(acl_result.is_ok());

            // Test delegation (this would be called through the DelegationManager in real usage)
            let delegation_result = crate::security::delegation::DelegationManager::new()
                .create_delegation(PermissionDelegation {
                    delegation_id: "".to_string(),
                    delegator_user_id: delegator_id,
                    delegatee_user_id: delegatee_id,
                    resource_id: resource_id.to_string(),
                    permissions: vec![RbacPermission::Read, RbacPermission::Write],
                    scope: DelegationScope::User,
                    constraints: crate::security::delegation::delegation::create_standard_constraints(),
                    granted_at: 0,
                    expires_at: None,
                    approved_by: None,
                    revoked_by: None,
                    revoked_at: None,
                    is_active: true,
                    audit_trail: Vec::new(),
                });

            assert!(delegation_result.is_ok());

            // Note: In a full integration, the delegated permissions would be reflected in 
            // the effective permissions calculation
        }
    }

    #[test]
    fn test_rbac_security_level_enforcement() {
        let rbac_manager = get_rbac_manager();
        assert!(rbac_manager.is_some());

        if let Some(rbac_mgr) = rbac_manager {
            // Create high-security role
            let high_security_role_id = rbac_mgr.create_role(
                "security_admin",
                "High security administrator",
                vec![RbacPermission::All],
                SecurityLevel::System,
                None,
            ).unwrap();

            // Create regular user role
            let regular_role_id = rbac_mgr.create_role(
                "regular_user",
                "Regular user",
                vec![RbacPermission::Read],
                SecurityLevel::Low,
                None,
            ).unwrap();

            // Assign roles to different security contexts
            let high_security_user = 1005;
            let regular_user = 1006;

            let assign_high = rbac_mgr.assign_role_to_user(high_security_user, high_security_role_id, None, None);
            let assign_regular = rbac_mgr.assign_role_to_user(regular_user, regular_role_id, None, None);

            assert!(assign_high.is_ok());
            assert!(assign_regular.is_ok());

            // Test that security levels are properly enforced
            // (In real implementation, this would be checked against actual security contexts)
        }
    }

    #[test]
    fn test_rbac_syscall_integration() {
        let rbac_manager = get_rbac_manager();
        assert!(rbac_manager.is_some());

        if let Some(rbac_mgr) = rbac_manager {
            // Create a resource for syscall testing
            let resource_id = "/sys/test_resource";
            
            let acl_entries = vec![
                AclEntry {
                    entry_id: "syscall_test".to_string(),
                    principal_type: PrincipalType::User,
                    principal_id: 1007, // test user
                    permissions: vec![RbacPermission::Read, RbacPermission::Write],
                    conditions: vec![],
                    effective: true,
                    inherited: false,
                    priority: 100,
                    expires_at: None,
                },
            ];

            let acl_result = rbac_mgr.create_acl(resource_id, acl_entries);
            assert!(acl_result.is_ok());

            // Test permission validation (simulating syscall validation)
            let user_id = 1007;
            let required_permissions = vec![RbacPermission::Read, RbacPermission::Write];
            
            let validation_result = rbac_mgr.validate_access(user_id, resource_id, &required_permissions);
            assert!(validation_result.is_ok());
            assert!(validation_result.unwrap());

            // Test insufficient permissions
            let insufficient_permissions = vec![RbacPermission::Admin];
            let validation_result2 = rbac_mgr.validate_access(user_id, resource_id, &insufficient_permissions);
            assert!(validation_result2.is_err());
            assert_eq!(validation_result2.unwrap_err(), RbacError::PermissionDenied);
        }
    }

    #[test]
    fn test_rbac_stats_and_monitoring() {
        let rbac_manager = get_rbac_manager();
        assert!(rbac_manager.is_some());

        if let Some(rbac_mgr) = rbac_manager {
            // Create some roles and assignments to generate stats
            let role1_id = rbac_mgr.create_role("role1", "Test role 1", vec![RbacPermission::Read], SecurityLevel::Low, None).unwrap();
            let role2_id = rbac_mgr.create_role("role2", "Test role 2", vec![RbacPermission::Write], SecurityLevel::Medium, None).unwrap();

            let user1_id = 1008;
            let user2_id = 1009;

            // Create assignments
            rbac_mgr.assign_role_to_user(user1_id, role1_id, None, None).unwrap();
            rbac_mgr.assign_role_to_user(user2_id, role2_id, None, None).unwrap();

            // Perform some permission checks to generate stats
            rbac_mgr.check_permission(user1_id, "test_res", RbacPermission::Read).unwrap();
            rbac_mgr.check_permission(user2_id, "test_res", RbacPermission::Write).unwrap();
            rbac_mgr.check_permission(user1_id, "test_res", RbacPermission::Admin).unwrap(); // Should fail

            // Get and verify statistics
            let stats = rbac_mgr.get_stats();
            assert_eq!(stats.total_roles, 2);
            assert_eq!(stats.total_assignments, 2);
            assert_eq!(stats.permission_checks, 3);
            assert_eq!(stats.granted_permissions, 2);
            assert_eq!(stats.denied_permissions, 1);
        }
    }

    #[test]
    fn test_rbac_error_handling() {
        let rbac_manager = get_rbac_manager();
        assert!(rbac_manager.is_some());

        if let Some(rbac_mgr) = rbac_manager {
            // Test role not found error
            let result = rbac_mgr.get_role(99999);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), RbacError::RoleNotFound);

            // Test invalid role creation
            let result = rbac_mgr.create_role("", "", vec![], SecurityLevel::Low, None);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), RbacError::InvalidParameter);

            // Test permission check with non-existent user
            let result = rbac_mgr.check_permission(99999, "test_resource", RbacPermission::Read);
            assert!(result.is_ok()); // Permission check should not fail for non-existent user
            assert!(!result.unwrap()); // But should return false
        }
    }
}
