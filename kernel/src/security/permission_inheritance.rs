//! Permission Inheritance System
//! 
//! This module provides comprehensive permission inheritance functionality including:
//! - Multi-level permission inheritance
//! - Inheritance policy management
//! - Permission propagation rules
//! - Conflict resolution mechanisms

use alloc::vec::Vec;
use alloc::string::{String, ToString};
use alloc::collections::{HashMap, BTreeSet, BTreeMap};
use crate::security::rbac::{RbacPermission, SecurityLevel};
use crate::admin::user_manager::{UserId, GroupId};
use super::RbacError;

/// Inheritance levels for permission propagation
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum InheritanceLevel {
    None = 0,        // No inheritance
    Parent = 1,      // Inherit from direct parent only
    Hierarchy = 2,   // Inherit from entire hierarchy
    Conditional = 3, // Conditional inheritance based on context
    Dynamic = 4,     // Dynamic inheritance based on runtime conditions
}

/// Inheritance rule for permission propagation
#[derive(Debug, Clone)]
pub struct InheritanceRule {
    pub rule_id: String,
    pub source_resource: String,
    pub target_resource: String,
    pub permissions: Vec<RbacPermission>,
    pub inheritance_level: InheritanceLevel,
    pub conditions: Vec<String>,
    pub priority: u32,
    pub enabled: bool,
    pub created_by: Option<UserId>,
    pub created_at: u64,
    pub expires_at: Option<u64>,
}

/// Inheritance chain representing the path of permission inheritance
#[derive(Debug, Clone)]
pub struct InheritanceChain {
    pub resource_id: String,
    pub chain: Vec<String>, // Ordered list of resources in inheritance chain
    pub inherited_permissions: BTreeMap<String, BTreeSet<RbacPermission>>,
    pub conflicts: Vec<InheritanceConflict>,
    pub resolved_permissions: BTreeSet<RbacPermission>,
}

/// Inheritance conflict when multiple sources grant conflicting permissions
#[derive(Debug, Clone)]
pub struct InheritanceConflict {
    pub resource_id: String,
    pub conflicting_permissions: Vec<RbacPermission>,
    pub sources: Vec<String>,
    pub resolution_strategy: ConflictResolution,
}

/// Conflict resolution strategies
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ConflictResolution {
    DenyTakesPrecedence = 0,     // Deny permissions override allow permissions
    AllowTakesPrecedence = 1,    // Allow permissions override deny permissions
    MostRestrictive = 2,         // Apply most restrictive interpretation
    MostPermissive = 3,          // Apply most permissive interpretation
    MergeWithMask = 4,           // Merge permissions with conflict mask
    ManualResolution = 5,        // Requires manual resolution
}

/// Inheritance context for permission evaluation
#[derive(Debug, Clone)]
pub struct InheritanceContext {
    pub user_id: Option<UserId>,
    pub group_ids: Vec<GroupId>,
    pub current_security_level: SecurityLevel,
    pub source_resource: String,
    pub target_resource: String,
    pub evaluation_time: u64,
    pub inherited_from: Option<String>, // The resource this permission was inherited from
}

/// Permission inheritance policy
#[derive(Debug, Clone)]
pub struct InheritancePolicy {
    pub policy_id: String,
    pub name: String,
    pub description: String,
    pub default_inheritance_level: InheritanceLevel,
    pub conflict_resolution: ConflictResolution,
    pub enabled: bool,
    pub resource_types: Vec<String>,
    pub permission_mappings: HashMap<RbacPermission, Vec<RbacPermission>>,
}

/// Permission inheritance calculator
#[derive(Debug, Clone)]
pub struct PermissionInheritance {
    pub inheritance_rules: HashMap<String, Vec<InheritanceRule>>,
    pub inheritance_policies: HashMap<String, InheritancePolicy>,
    pub inheritance_chains: HashMap<String, InheritanceChain>,
    pub inheritance_cache: HashMap<(String, String, UserId), BTreeSet<RbacPermission>>,
}

impl PermissionInheritance {
    /// Create a new permission inheritance instance
    pub fn new() -> Self {
        Self {
            inheritance_rules: HashMap::new(),
            inheritance_policies: HashMap::new(),
            inheritance_chains: HashMap::new(),
            inheritance_cache: HashMap::new(),
        }
    }

    /// Add inheritance rule
    pub fn add_inheritance_rule(&mut self, rule: InheritanceRule) -> Result<(), RbacError> {
        // Validate rule
        if rule.source_resource.is_empty() || rule.target_resource.is_empty() {
            return Err(RbacError::InvalidParameter);
        }

        if rule.permissions.is_empty() {
            return Err(RbacError::InvalidParameter);
        }

        // Check for circular dependencies
        if self.has_circular_dependency(&rule.source_resource, &rule.target_resource) {
            return Err(RbacError::CircularDependency);
        }

        // Add rule to source resource's rules
        self.inheritance_rules
            .entry(rule.source_resource.clone())
            .or_insert_with(Vec::new)
            .push(rule);

        // Sort rules by priority (highest first)
        if let Some(rules) = self.inheritance_rules.get_mut(&rule.source_resource) {
            rules.sort_by(|a, b| b.priority.cmp(&a.priority));
        }

        Ok(())
    }

    /// Remove inheritance rule
    pub fn remove_inheritance_rule(&mut self, rule_id: &str) -> Result<(), RbacError> {
        let mut found = false;
        
        for rules in self.inheritance_rules.values_mut() {
            rules.retain(|rule| {
                if rule.rule_id == rule_id {
                    found = true;
                    false
                } else {
                    true
                }
            });
        }

        if !found {
            return Err(RbacError::ResourceNotFound);
        }

        // Clear related cache entries
        self.clear_cache_for_rule(rule_id);

        Ok(())
    }

    /// Get inheritance rules for a resource
    pub fn get_inheritance_rules(&self, resource_id: &str) -> Vec<&InheritanceRule> {
        if let Some(rules) = self.inheritance_rules.get(resource_id) {
            rules.iter().filter(|rule| rule.enabled).collect()
        } else {
            Vec::new()
        }
    }

    /// Calculate inherited permissions for a resource
    pub fn calculate_inherited_permissions(&mut self, resource_id: &str, 
                                          user_id: UserId, context: &InheritanceContext) -> RbacResult<BTreeSet<RbacPermission>> {
        let cache_key = (resource_id.to_string(), context.source_resource.clone(), user_id);
        
        // Check cache first
        if let Some(cached_permissions) = self.inheritance_cache.get(&cache_key) {
            return Ok(cached_permissions.clone());
        }

        // Build inheritance chain
        let chain = self.build_inheritance_chain(resource_id, &context.source_resource)?;
        
        // Calculate inherited permissions
        let mut inherited_permissions = BTreeSet::new();
        let mut conflicts = Vec::new();

        for resource in &chain.chain {
            let resource_rules = self.get_inheritance_rules(resource);
            
            for rule in resource_rules {
                // Check if rule applies to target resource
                if self.rule_applies_to_resource(rule, resource_id, context) {
                    // Check conditions
                    if self.conditions_satisfied(rule, context) {
                        // Check inheritance level
                        if self.inheritance_level_allows(rule.inheritance_level, &chain.chain, resource)? {
                            for &permission in &rule.permissions {
                                inherited_permissions.insert(permission);
                            }
                        }
                    }
                }
            }
        }

        // Resolve conflicts
        let resolved_permissions = self.resolve_conflicts(&mut inherited_permissions, &mut conflicts, context)?;

        // Cache the result
        self.inheritance_cache.insert(cache_key, resolved_permissions.clone());

        // Update inheritance chain record
        self.update_inheritance_chain(resource_id, chain, conflicts, &resolved_permissions);

        Ok(resolved_permissions)
    }

    /// Apply inheritance policy
    pub fn apply_inheritance_policy(&mut self, policy_id: &str, resource_id: &str, user_id: UserId) -> RbacResult<BTreeSet<RbacPermission>> {
        let policy = self.inheritance_policies.get(policy_id)
            .ok_or(RbacError::ResourceNotFound)?;

        if !policy.enabled {
            return Err(RbacError::OperationNotPermitted);
        }

        let mut inherited_permissions = BTreeSet::new();

        // Apply permission mappings
        for (&source_perm, &ref target_perms) in &policy.permission_mappings {
            // Check if source permission exists (would need to be passed as parameter)
            // For now, include all mapped permissions
            for &target_perm in target_perms {
                inherited_permissions.insert(target_perm);
            }
        }

        Ok(inherited_permissions)
    }

    /// Add inheritance policy
    pub fn add_inheritance_policy(&mut self, policy: InheritancePolicy) -> Result<(), RbacError> {
        // Validate policy
        if policy.name.is_empty() {
            return Err(RbacError::InvalidParameter);
        }

        self.inheritance_policies.insert(policy.policy_id.clone(), policy);
        Ok(())
    }

    /// Remove inheritance policy
    pub fn remove_inheritance_policy(&mut self, policy_id: &str) -> Result<(), RbacError> {
        if self.inheritance_policies.remove(policy_id).is_none() {
            return Err(RbacError::ResourceNotFound);
        }
        Ok(())
    }

    /// Clear inheritance cache
    pub fn clear_cache(&mut self) {
        self.inheritance_cache.clear();
    }

    /// Clear cache for specific user/resource combination
    pub fn clear_user_cache(&mut self, user_id: UserId) {
        self.inheritance_cache.retain(|(.., cached_user_id), _| *cached_user_id != user_id);
    }

    // ==================== Internal Helper Methods ====================

    /// Build inheritance chain for resource
    fn build_inheritance_chain(&self, target_resource: &str, source_resource: &str) -> RbacResult<InheritanceChain> {
        let mut chain = Vec::new();
        let mut visited = BTreeSet::new();

        // Build chain from source to target
        let mut current = source_resource.to_string();
        while current != target_resource {
            if visited.contains(&current) {
                return Err(RbacError::CircularDependency);
            }
            visited.insert(current.clone());
            chain.push(current.clone());

            // Find next resource in inheritance path
            current = self.find_next_inheritance_source(&current, target_resource)?
        }

        // Add target resource
        chain.push(target_resource.to_string());

        Ok(InheritanceChain {
            resource_id: target_resource.to_string(),
            chain,
            inherited_permissions: BTreeMap::new(),
            conflicts: Vec::new(),
            resolved_permissions: BTreeSet::new(),
        })
    }

    /// Find next inheritance source in chain
    fn find_next_inheritance_source(&self, current: &str, target: &str) -> RbacResult<String> {
        let rules = self.get_inheritance_rules(current);
        
        for rule in rules {
            if rule.enabled && 
               rule.target_resource == target && 
               self.inheritance_level_allows(rule.inheritance_level, &[current.to_string()], current)? {
                return Ok(rule.source_resource.clone());
            }
        }

        // If no direct rule found, return current (indicates end of chain)
        Ok(current.to_string())
    }

    /// Check if inheritance rule applies to resource
    fn rule_applies_to_resource(&self, rule: &InheritanceRule, target_resource: &str, context: &InheritanceContext) -> bool {
        // Check if rule's target matches
        if rule.target_resource != target_resource {
            return false;
        }

        // Check security level
        if context.current_security_level < SecurityLevel::Medium {
            // Additional security checks would go here
        }

        true
    }

    /// Check if conditions are satisfied
    fn conditions_satisfied(&self, rule: &InheritanceRule, context: &InheritanceContext) -> bool {
        for condition in &rule.conditions {
            match condition.as_str() {
                "security_level_check" => {
                    if context.current_security_level < SecurityLevel::Low {
                        return false;
                    }
                },
                "user_match" => {
                    if Some(0) != context.user_id { // Example: only root can inherit
                        return false;
                    }
                },
                "resource_type_match" => {
                    // Would need resource type information
                    // Simplified for now
                },
                _ => {
                    // Unknown condition, treat as not satisfied
                    return false;
                }
            }
        }
        true
    }

    /// Check if inheritance level allows inheritance
    fn inheritance_level_allows(&self, level: InheritanceLevel, chain: &[String], current: &str) -> RbacResult<bool> {
        match level {
            InheritanceLevel::None => Ok(false),
            InheritanceLevel::Parent => Ok(chain.len() <= 2),
            InheritanceLevel::Hierarchy => Ok(true),
            InheritanceLevel::Conditional => {
                // Would evaluate conditional logic here
                Ok(true)
            },
            InheritanceLevel::Dynamic => {
                // Would evaluate dynamic conditions here
                Ok(true)
            },
        }
    }

    /// Resolve permission conflicts
    fn resolve_conflicts(&self, permissions: &mut BTreeSet<RbacPermission>, 
                        conflicts: &mut Vec<InheritanceConflict>, 
                        context: &InheritanceContext) -> RbacResult<BTreeSet<RbacPermission>> {
        // Simple conflict resolution: use most permissive approach
        // More sophisticated conflict resolution would implement proper algorithms
        
        // Check for conflicting permissions (simplified)
        if permissions.contains(&RbacPermission::Read) && permissions.contains(&RbacPermission::None) {
            // Conflict detected - use most permissive (Read takes precedence)
            permissions.remove(&RbacPermission::None);
        }

        Ok(permissions.clone())
    }

    /// Update inheritance chain record
    fn update_inheritance_chain(&mut self, resource_id: &str, chain: InheritanceChain, 
                               conflicts: Vec<InheritanceConflict>, resolved_permissions: &BTreeSet<RbacPermission>) {
        let chain_key = resource_id.to_string();
        
        let mut updated_chain = chain;
        updated_chain.conflicts = conflicts;
        updated_chain.resolved_permissions = resolved_permissions.clone();
        
        self.inheritance_chains.insert(chain_key, updated_chain);
    }

    /// Check for circular dependency
    fn has_circular_dependency(&self, source: &str, target: &str) -> bool {
        if source == target {
            return true;
        }

        let mut visited = BTreeSet::new();
        let mut stack = vec![source.to_string()];

        while let Some(current) = stack.pop() {
            if visited.contains(&current) {
                continue;
            }
            visited.insert(current.clone());

            if current == target {
                return true;
            }

            // Check all resources that inherit from current
            for (resource, rules) in &self.inheritance_rules {
                for rule in rules {
                    if rule.enabled && rule.source_resource == current {
                        stack.push(resource.clone());
                    }
                }
            }
        }

        false
    }

    /// Clear cache entries related to a rule
    fn clear_cache_for_rule(&mut self, rule_id: &str) {
        self.inheritance_cache.clear(); // Clear entire cache for simplicity
    }
}

/// Helper functions for permission inheritance
pub struct InheritanceHelpers;

impl InheritanceHelpers {
    /// Create standard inheritance rule
    pub fn create_parent_inheritance_rule(parent_resource: &str, child_resource: &str, 
                                        permissions: Vec<RbacPermission>) -> InheritanceRule {
        InheritanceRule {
            rule_id: format!("{}_to_{}_parent", parent_resource, child_resource),
            source_resource: parent_resource.to_string(),
            target_resource: child_resource.to_string(),
            permissions,
            inheritance_level: InheritanceLevel::Parent,
            conditions: vec![],
            priority: 100,
            enabled: true,
            created_by: None,
            created_at: crate::hal::get_current_time(),
            expires_at: None,
        }
    }

    /// Create hierarchy inheritance rule
    pub fn create_hierarchy_inheritance_rule(source_resource: &str, target_resource: &str,
                                           permissions: Vec<RbacPermission>) -> InheritanceRule {
        InheritanceRule {
            rule_id: format!("{}_to_{}_hierarchy", source_resource, target_resource),
            source_resource: source_resource.to_string(),
            target_resource: target_resource.to_string(),
            permissions,
            inheritance_level: InheritanceLevel::Hierarchy,
            conditions: vec![],
            priority: 50,
            enabled: true,
            created_by: None,
            created_at: crate::hal::get_current_time(),
            expires_at: None,
        }
    }

    /// Create conditional inheritance rule
    pub fn create_conditional_inheritance_rule(source_resource: &str, target_resource: &str,
                                             permissions: Vec<RbacPermission>, conditions: Vec<String>) -> InheritanceRule {
        InheritanceRule {
            rule_id: format!("{}_to_{}_conditional", source_resource, target_resource),
            source_resource: source_resource.to_string(),
            target_resource: target_resource.to_string(),
            permissions,
            inheritance_level: InheritanceLevel::Conditional,
            conditions,
            priority: 75,
            enabled: true,
            created_by: None,
            created_at: crate::hal::get_current_time(),
            expires_at: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inheritance_rule_creation() {
        let rule = InheritanceRule {
            rule_id: "test_rule".to_string(),
            source_resource: "/parent".to_string(),
            target_resource: "/child".to_string(),
            permissions: vec![RbacPermission::Read],
            inheritance_level: InheritanceLevel::Parent,
            conditions: vec![],
            priority: 100,
            enabled: true,
            created_by: None,
            created_at: 0,
            expires_at: None,
        };

        assert_eq!(rule.source_resource, "/parent");
        assert_eq!(rule.target_resource, "/child");
        assert_eq!(rule.permissions.len(), 1);
    }

    #[test]
    fn test_inheritance_calculation() {
        let mut inheritance = PermissionInheritance::new();
        
        let rule = InheritanceHelpers::create_parent_inheritance_rule(
            "/parent",
            "/child",
            vec![RbacPermission::Read, RbacPermission::Write]
        );
        
        let result = inheritance.add_inheritance_rule(rule);
        assert!(result.is_ok());

        let context = InheritanceContext {
            user_id: Some(1000),
            group_ids: vec![],
            current_security_level: SecurityLevel::Low,
            source_resource: "/parent".to_string(),
            target_resource: "/child".to_string(),
            evaluation_time: 0,
            inherited_from: None,
        };

        let permissions = inheritance.calculate_inherited_permissions("/child", 1000, &context);
        assert!(permissions.is_ok());
    }

    #[test]
    fn test_circular_dependency_detection() {
        let mut inheritance = PermissionInheritance::new();
        
        let rule1 = InheritanceHelpers::create_parent_inheritance_rule("/a", "/b", vec![RbacPermission::Read]);
        let rule2 = InheritanceHelpers::create_parent_inheritance_rule("/b", "/a", vec![RbacPermission::Read]);
        
        assert!(inheritance.add_inheritance_rule(rule1).is_ok());
        assert!(inheritance.add_inheritance_rule(rule2).is_err()); // Should detect circular dependency
    }
}
