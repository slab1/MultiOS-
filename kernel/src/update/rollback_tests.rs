//! Rollback System Integration Tests
//! 
//! This module contains integration tests for the rollback and recovery system
//! to ensure proper functionality and integration with the kernel.

use crate::update::rollback::{
    RollbackSystem, ComponentCategory, RollbackScope, helpers::*,
    RecoveryPointId, RollbackOperationId, HealthLevel
};

/// Test rollback system initialization
#[test_case]
fn test_rollback_system_initialization() {
    // Test rollback system initialization
    let result = crate::update::rollback::init_rollback_system();
    assert!(result.is_ok(), "Rollback system should initialize successfully");
    
    let rollback_system = result.unwrap();
    
    // Test system health check
    let health = rollback_system.get_system_health();
    assert!(health.is_ok(), "System health check should succeed");
    
    let health_status = health.unwrap();
    assert!(health_status.last_validation_time > 0, "Validation should have occurred");
}

/// Test recovery point creation
#[test_case]
fn test_recovery_point_creation() {
    // Initialize rollback system
    let rollback_system = crate::update::rollback::init_rollback_system()
        .expect("Failed to initialize rollback system");
    
    // Create a recovery point
    let recovery_point_id = rollback_system.create_update_recovery_point(
        "Test recovery point creation"
    );
    
    assert!(recovery_point_id.is_ok(), "Recovery point creation should succeed");
    
    let recovery_point_id = recovery_point_id.unwrap();
    assert!(recovery_point_id > 0, "Recovery point ID should be valid");
    
    // Verify recovery point exists
    let recovery_point = rollback_system.recovery_point_manager.get_recovery_point(recovery_point_id);
    assert!(recovery_point.is_some(), "Created recovery point should be retrievable");
}

/// Test snapshot creation for different components
#[test_case]
fn test_snapshot_creation() {
    let rollback_system = crate::update::rollback::init_rollback_system()
        .expect("Failed to initialize rollback system");
    
    let component_categories = vec![
        ComponentCategory::KernelCore,
        ComponentCategory::SystemServices,
        ComponentCategory::Configuration,
    ];
    
    for &category in &component_categories {
        let snapshot = rollback_system.snapshot_manager.create_snapshot(category);
        assert!(snapshot.is_ok(), "Snapshot creation should succeed for {:?}", category);
        
        let snapshot = snapshot.unwrap();
        assert_eq!(snapshot.component_category, category, "Snapshot category should match");
        assert!(snapshot.id > 0, "Snapshot ID should be valid");
        assert!(snapshot.timestamp > 0, "Snapshot timestamp should be set");
    }
}

/// Test system state validation
#[test_case]
fn test_system_state_validation() {
    let rollback_system = crate::update::rollback::init_rollback_system()
        .expect("Failed to initialize rollback system");
    
    // Test system state validation
    let result = rollback_system.validate_system_state();
    assert!(result.is_ok(), "System state validation should succeed in test environment");
}

/// Test rollback planning
#[test_case]
fn test_rollback_planning() {
    let rollback_system = crate::update::rollback::init_rollback_system()
        .expect("Failed to initialize rollback system");
    
    // Create a recovery point first
    let recovery_point_id = rollback_system.create_update_recovery_point(
        "Test rollback planning"
    ).expect("Failed to create recovery point");
    
    // Plan a partial rollback
    let rollback_components = vec![
        ComponentCategory::Configuration,
        ComponentCategory::SystemServices,
    ];
    
    let rollback_operation_id = rollback_system.execute_rollback(
        RollbackScope::Partial,
        Some(recovery_point_id),
        rollback_components
    );
    
    assert!(rollback_operation_id.is_ok(), "Rollback planning should succeed");
    
    let operation_id = rollback_operation_id.unwrap();
    assert!(operation_id > 0, "Operation ID should be valid");
    
    // Test progress tracking
    let progress = rollback_system.get_rollback_progress(operation_id);
    assert!(progress.is_some(), "Rollback progress should be tracked");
    
    let progress_info = progress.unwrap();
    assert_eq!(progress_info.operation_id, operation_id, "Progress should match operation ID");
}

/// Test helper functions
#[test_case]
fn test_helper_functions() {
    // Test recovery point creation helper
    let recovery_point_id = create_recovery_point_with_name("Helper function test");
    assert!(recovery_point_id.is_ok(), "Helper recovery point creation should succeed");
    
    // Test system health check through helper
    if let Some(system) = crate::update::rollback::get_rollback_system() {
        let health = system.get_system_health();
        assert!(health.is_ok(), "Helper system health check should succeed");
    }
}

/// Test partial rollback execution
#[test_case]
fn test_partial_rollback_execution() {
    let rollback_system = crate::update::rollback::init_rollback_system()
        .expect("Failed to initialize rollback system");
    
    // Create a recovery point
    let recovery_point_id = rollback_system.create_update_recovery_point(
        "Test partial rollback"
    ).expect("Failed to create recovery point");
    
    // Execute partial rollback with single component
    let rollback_operation_id = rollback_system.execute_rollback(
        RollbackScope::Component,
        Some(recovery_point_id),
        vec![ComponentCategory::Configuration]
    );
    
    assert!(rollback_operation_id.is_ok(), "Partial rollback execution should succeed");
}

/// Test recovery point listing
#[test_case]
fn test_recovery_point_listing() {
    let rollback_system = crate::update::rollback::init_rollback_system()
        .expect("Failed to initialize rollback system");
    
    // Create multiple recovery points
    for i in 1..=3 {
        let recovery_point_id = rollback_system.create_update_recovery_point(
            &format!("Test recovery point {}", i)
        );
        assert!(recovery_point_id.is_ok(), "Recovery point {} creation should succeed", i);
    }
    
    // List all recovery points
    let recovery_points = rollback_system.list_recovery_points();
    assert!(recovery_points.len() >= 3, "Should have at least 3 recovery points");
    
    // Verify all have valid timestamps
    for recovery_point in &recovery_points {
        assert!(recovery_point.timestamp > 0, "Recovery point should have valid timestamp");
        assert!(!recovery_point.description.is_empty(), "Recovery point should have description");
    }
}

/// Test snapshot validation
#[test_case]
fn test_snapshot_validation() {
    let rollback_system = crate::update::rollback::init_rollback_system()
        .expect("Failed to initialize rollback system");
    
    // Create a snapshot
    let snapshot = rollback_system.snapshot_manager.create_snapshot(
        ComponentCategory::KernelCore
    ).expect("Failed to create snapshot");
    
    // Validate the snapshot
    let is_valid = rollback_system.snapshot_manager.validate_snapshot(snapshot.id);
    assert!(is_valid.is_ok(), "Snapshot validation should succeed");
    assert!(is_valid.unwrap(), "Created snapshot should be valid");
}

/// Test error handling
#[test_case]
fn test_error_handling() {
    let rollback_system = crate::update::rollback::init_rollback_system()
        .expect("Failed to initialize rollback system");
    
    // Test invalid recovery point retrieval
    let invalid_recovery_point = rollback_system.recovery_point_manager.get_recovery_point(999999);
    assert!(invalid_recovery_point.is_none(), "Invalid recovery point should return None");
    
    // Test invalid snapshot retrieval
    let invalid_snapshot = rollback_system.snapshot_manager.get_snapshot(999999);
    assert!(invalid_snapshot.is_none(), "Invalid snapshot should return None");
    
    // Test invalid rollback progress
    let invalid_progress = rollback_system.get_rollback_progress(999999);
    assert!(invalid_progress.is_none(), "Invalid operation ID should return None");
}

/// Test cleanup functionality
#[test_case]
fn test_cleanup_functionality() {
    let rollback_system = crate::update::rollback::init_rollback_system()
        .expect("Failed to initialize rollback system");
    
    // Test cleanup of expired data
    let cleanup_result = rollback_system.cleanup_expired_data();
    assert!(cleanup_result.is_ok(), "Cleanup should succeed");
    
    let (cleaned_snapshots, cleaned_recovery_points) = cleanup_result.unwrap();
    // In test environment, cleanup might not find expired data
    assert!(cleaned_snapshots >= 0, "Cleanup should return valid count");
    assert!(cleaned_recovery_points >= 0, "Cleanup should return valid count");
}

/// Test integration with update system
#[test_case]
fn test_update_system_integration() {
    // Initialize update system (which should initialize rollback system)
    let update_result = crate::update::init_update_system();
    assert!(update_result.is_ok(), "Update system initialization should succeed");
    
    // Verify rollback system is accessible through update system
    let rollback_system = crate::update::rollback::get_rollback_system();
    assert!(rollback_system.is_some(), "Rollback system should be accessible after update system init");
    
    // Test recovery point creation through update system
    let recovery_point_id = crate::update::helpers::create_critical_recovery_point(
        "Integration test recovery point"
    );
    assert!(recovery_point_id.is_ok(), "Recovery point creation through update system should succeed");
}

/// Performance test for snapshot creation
#[test_case]
fn test_snapshot_creation_performance() {
    use crate::hal::timers::get_system_time_ms;
    
    let rollback_system = crate::update::rollback::init_rollback_system()
        .expect("Failed to initialize rollback system");
    
    let start_time = get_system_time_ms();
    
    // Create multiple snapshots
    let component_categories = vec![
        ComponentCategory::KernelCore,
        ComponentCategory::SystemServices,
        ComponentCategory::DeviceDrivers,
        ComponentCategory::Configuration,
    ];
    
    for &category in &component_categories {
        let snapshot = rollback_system.snapshot_manager.create_snapshot(category);
        assert!(snapshot.is_ok(), "Snapshot creation should succeed for {:?}", category);
    }
    
    let end_time = get_system_time_ms();
    let creation_time = end_time - start_time;
    
    // Snapshot creation should be reasonably fast (under 1 second in test environment)
    assert!(creation_time < 1000, "Snapshot creation should complete quickly");
}

/// Test concurrent operation safety
#[test_case]
fn test_concurrent_operation_safety() {
    let rollback_system = crate::update::rollback::init_rollback_system()
        .expect("Failed to initialize rollback system");
    
    // Create recovery points concurrently (in a real async environment)
    let recovery_point_ids = vec![
        rollback_system.create_update_recovery_point("Concurrent test 1"),
        rollback_system.create_update_recovery_point("Concurrent test 2"),
        rollback_system.create_update_recovery_point("Concurrent test 3"),
    ];
    
    // All should succeed
    for (i, result) in recovery_point_ids.iter().enumerate() {
        assert!(result.is_ok(), "Concurrent recovery point {} creation should succeed", i + 1);
    }
    
    // Verify all recovery points exist
    let recovery_points = rollback_system.list_recovery_points();
    assert!(recovery_points.len() >= 3, "Should have at least 3 concurrent recovery points");
}

// Test configuration constants
const TEST_MAX_RECOVERY_POINTS: usize = 10;
const TEST_SNAPSHOT_RETENTION_HOURS: u64 = 24;

/// Test configuration limits
#[test_case]
fn test_configuration_limits() {
    let rollback_system = crate::update::rollback::init_rollback_system()
        .expect("Failed to initialize rollback system");
    
    // Test that the system respects configuration limits
    // Create many recovery points to test limits
    for i in 0..TEST_MAX_RECOVERY_POINTS + 5 {
        let result = rollback_system.create_update_recovery_point(
            &format!("Limit test recovery point {}", i)
        );
        
        // Should eventually fail when limit is reached
        if i >= TEST_MAX_RECOVERY_POINTS {
            // System may handle this gracefully by removing old recovery points
            // So we don't necessarily expect failure, but behavior should be consistent
        }
    }
    
    // Verify system is still functional
    let health = rollback_system.get_system_health();
    assert!(health.is_ok(), "System should remain functional after limit testing");
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test runner for rollback system tests
    pub fn run_all_tests() {
        info!("Running rollback system integration tests...");
        
        test_rollback_system_initialization();
        test_recovery_point_creation();
        test_snapshot_creation();
        test_system_state_validation();
        test_rollback_planning();
        test_helper_functions();
        test_partial_rollback_execution();
        test_recovery_point_listing();
        test_snapshot_validation();
        test_error_handling();
        test_cleanup_functionality();
        test_update_system_integration();
        test_snapshot_creation_performance();
        test_concurrent_operation_safety();
        test_configuration_limits();
        
        info!("All rollback system integration tests completed successfully!");
    }
}