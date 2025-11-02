//! Integration Bridge Example
//!
//! This example demonstrates how to use the integration bridge to combine
//! the comprehensive driver testing framework with the existing device-drivers
//! crate testing infrastructure.

use driver_testing_framework::{
    integration::{
        DeviceDriversTestBridge, DeviceDriversIntegrationConfig,
        ComprehensiveTestResults
    },
    core::{TestCategory, TestStatus},
    simulation::SimulationEnvironment,
    stress_testing::StressTestConfig,
    performance::PerformanceBenchmarkConfig,
    validation::ValidationConfig,
    debugging::DebugConfig
};

use crate::AdvancedDriverId;
use alloc::string::ToString;

/// Example demonstrating legacy test integration
pub async fn example_legacy_test_integration() -> Result<(), Box<dyn core::error::Error>> {
    println!("Starting legacy test integration example...");

    // Create integration configuration
    let config = DeviceDriversIntegrationConfig {
        enable_legacy_compatibility: true,
        auto_generate_tests: true,
        bridge_timeout_ms: 10000,
        enable_advanced_features: true,
    };

    // Create test bridge
    let mut bridge = DeviceDriversTestBridge::new(config);

    // Create advanced test suite builder with comprehensive configurations
    let simulation_env = SimulationEnvironment::default();
    let stress_config = StressTestConfig::default()
        .with_max_concurrent_operations(10)
        .with_resource_limit_tests_enabled(true);
    
    let performance_config = PerformanceBenchmarkConfig::default()
        .with_latency_measurement_enabled(true)
        .with_throughput_measurement_enabled(true);
    
    let validation_config = ValidationConfig::default()
        .with_api_compliance_checks_enabled(true);
    
    let debug_config = DebugConfig::default()
        .with_state_inspection_enabled(true);

    // Example driver ID (would normally come from the device-drivers crate)
    let driver_id = AdvancedDriverId(1);

    println!("Running comprehensive tests with legacy compatibility...");

    // Run comprehensive tests combining legacy and advanced capabilities
    let results = bridge.run_comprehensive_tests(
        driver_id,
        Some("basic_functionality") // Example legacy suite name
    ).await?;

    println!("Test execution completed!");
    println!("Total tests executed: {}", results.total_tests());
    println!("Tests passed: {}", results.passed_tests());
    println!("Tests failed: {}", results.failed_tests());

    // Display results summary
    display_results_summary(&results);

    Ok(())
}

/// Example demonstrating advanced simulation with legacy compatibility
pub async fn example_advanced_simulation() -> Result<(), Box<dyn core::error::Error>> {
    println!("\nStarting advanced simulation example...");

    let mut bridge = DeviceDriversTestBridge::new(DeviceDriversIntegrationConfig::default());
    let driver_id = AdvancedDriverId(1);

    // Create advanced simulations for different device types
    let keyboard_sim = bridge.create_advanced_simulation(
        driver_id,
        "keyboard"
    ).await?;
    
    let serial_sim = bridge.create_advanced_simulation(
        driver_id,
        "serial"
    ).await?;
    
    let timer_sim = bridge.create_advanced_simulation(
        driver_id,
        "timer"
    ).await?;

    println!("Advanced simulations created:");
    println!("- Keyboard simulation: {}", keyboard_sim.details);
    println!("- Serial simulation: {}", serial_sim.details);
    println!("- Timer simulation: {}", timer_sim.details);

    Ok(())
}

/// Example demonstrating test migration from legacy to advanced framework
pub async fn example_test_migration() -> Result<(), Box<dyn core::error::Error>> {
    println!("\nStarting test migration example...");

    let bridge = DeviceDriversTestBridge::new(DeviceDriversIntegrationConfig::default());

    // Example legacy test definitions (would normally come from device-drivers crate)
    let legacy_tests = vec![
        crate::Test {
            name: "driver_init_test",
            test_type: crate::TestType::Unit,
            category: crate::TestCategory::Initialization,
            timeout_ms: 1000,
            critical: true,
            enabled: true,
            test_func: |context| {
                context.custom_data.insert("init_test".to_string(), "passed".to_string());
                crate::TestResult::Pass
            },
        },
        crate::Test {
            name: "driver_operations_test",
            test_type: crate::TestType::Integration,
            category: crate::TestCategory::Operations,
            timeout_ms: 2000,
            critical: false,
            enabled: true,
            test_func: |context| {
                context.custom_data.insert("operations_test".to_string(), "passed".to_string());
                crate::TestResult::Pass
            },
        },
        crate::Test {
            name: "driver_performance_test",
            test_type: crate::TestType::Performance,
            category: crate::TestCategory::Performance,
            timeout_ms: 5000,
            critical: false,
            enabled: true,
            test_func: |context| {
                context.custom_data.insert("performance_test".to_string(), "passed".to_string());
                crate::TestResult::Pass
            },
        },
    ];

    // Convert legacy tests to advanced framework tests
    let advanced_tests = bridge.generate_advanced_tests_from_legacy(&legacy_tests)?;

    println!("Successfully migrated {} legacy tests to advanced framework:", advanced_tests.len());
    for (i, test) in advanced_tests.iter().enumerate() {
        println!("  {}. {} (Category: {:?})", i + 1, test.get_name(), test.get_category());
    }

    // Run the migrated tests
    let driver_id = AdvancedDriverId(2);
    for test in advanced_tests {
        let result = test.run_test(driver_id).await?;
        println!("Test '{}' completed: {:?}", result.name, result.status);
    }

    Ok(())
}

/// Display comprehensive results summary
fn display_results_summary(results: &ComprehensiveTestResults) {
    println!("\n=== Comprehensive Test Results Summary ===");
    
    // Legacy results
    if !results.legacy_results.is_empty() {
        println!("\nLegacy Test Results:");
        for result in &results.legacy_results {
            let status_symbol = match result.status {
                TestStatus::Passed => "✓",
                TestStatus::Failed => "✗",
                TestStatus::Skipped => "○",
                _ => "?",
            };
            println!("  {} {} ({}ms): {}", 
                status_symbol, 
                result.name, 
                result.duration.as_millis(),
                result.message
            );
        }
    }

    // Simulation results
    if !results.simulation_results.is_empty() {
        println!("\nSimulation Results:");
        for sim_result in &results.simulation_results {
            let status_symbol = if sim_result.success { "✓" } else { "✗" };
            println!("  {} Simulation for driver {:?}: {}", 
                status_symbol,
                sim_result.driver_id,
                sim_result.details
            );
        }
    }

    // Performance results summary
    if !results.performance_results.is_empty() {
        println!("\nPerformance Benchmark Results:");
        for perf_result in &results.performance_results {
            println!("  Latency: {}ns, Throughput: {} ops/sec", 
                perf_result.latency_ns,
                perf_result.throughput_ops_per_sec
            );
        }
    }

    // Validation results summary
    if !results.validation_results.is_empty() {
        println!("\nValidation Results:");
        let passed_validations = results.validation_results.iter()
            .filter(|v| v.is_valid)
            .count();
        println!("  Validations passed: {}/{}", 
            passed_validations,
            results.validation_results.len()
        );
    }
}

/// Example of complete integration workflow
pub async fn example_complete_integration_workflow() -> Result<(), Box<dyn core::error::Error>> {
    println!("\n=== Complete Integration Workflow Example ===");

    // Step 1: Initialize integration bridge with full configuration
    let mut bridge = DeviceDriversTestBridge::new(DeviceDriversIntegrationConfig::default());
    
    // Step 2: Set up comprehensive testing environment
    let simulation_env = SimulationEnvironment::default();
    let stress_config = StressTestConfig::default()
        .with_memory_stress_enabled(true)
        .with_cpu_stress_enabled(true)
        .with_concurrent_operation_tests_enabled(true);
    
    let performance_config = PerformanceBenchmarkConfig::default()
        .with_memory_usage_profiling_enabled(true)
        .with_latency_profiling_enabled(true);
    
    let validation_config = ValidationConfig::default()
        .with_driver_api_compliance_enabled(true)
        .with_resource_management_validation_enabled(true);

    // Step 3: Test multiple drivers
    let test_drivers = vec![
        AdvancedDriverId(1), // Example: Serial driver
        AdvancedDriverId(2), // Example: Keyboard driver  
        AdvancedDriverId(3), // Example: Timer driver
    ];

    for (index, driver_id) in test_drivers.iter().enumerate() {
        println!("\nTesting driver {}: {:?}", index + 1, driver_id);

        // Run comprehensive tests
        let results = bridge.run_comprehensive_tests(*driver_id, None).await?;
        
        // Display results
        display_results_summary(&results);
        
        // Create specific simulations for each driver type
        let simulation_type = match index {
            0 => "serial",
            1 => "keyboard", 
            2 => "timer",
            _ => "serial",
        };
        
        let simulation_result = bridge.create_advanced_simulation(*driver_id, simulation_type).await?;
        println!("Driver {} simulation: {}", index + 1, simulation_result.details);
    }

    // Step 4: Display integration statistics
    let stats = bridge.get_integration_statistics();
    println!("\n=== Integration Statistics ===");
    println!("Legacy compatibility: {}", stats.legacy_compatibility_enabled);
    println!("Advanced features: {}", stats.advanced_features_enabled);
    println!("Simulation environment: {}", stats.has_simulation_env);
    println!("Stress testing: {}", stats.has_stress_config);
    println!("Performance benchmarking: {}", stats.has_performance_config);
    println!("Validation: {}", stats.has_validation_config);
    println!("Debugging: {}", stats.has_debug_config);

    println!("\nComplete integration workflow finished successfully!");
    Ok(())
}

/// Main function demonstrating all integration examples
#[cfg(feature = "std")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Driver Testing Framework Integration Bridge Examples ===\n");

    // Run all integration examples
    try {
        example_legacy_test_integration().await?;
        example_advanced_simulation().await?;
        example_test_migration().await?;
        example_complete_integration_workflow().await?;
        
        println!("\n🎉 All integration examples completed successfully!");
    } catch (error) {
        println!("❌ Error during integration examples: {}", error);
        return Err(error);
    }

    Ok(())
}

/// Test function for non-async environments
#[cfg(not(feature = "std"))]
pub fn main() -> Result<(), Box<dyn core::error::Error>> {
    // Non-async version would use simpler test patterns
    println!("Running integration bridge tests in no_std environment...");
    
    let config = DeviceDriversIntegrationConfig::default();
    let bridge = DeviceDriversTestBridge::new(config);
    
    let stats = bridge.get_integration_statistics();
    println!("Bridge created with {} features enabled", 
        stats.legacy_compatibility_enabled as u32 + stats.advanced_features_enabled as u32
    );
    
    Ok(())
}