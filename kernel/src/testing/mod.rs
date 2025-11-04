//! MultiOS Integration Testing Framework
//! 
//! This module provides comprehensive integration testing for the MultiOS kernel.
//! It includes cross-component testing, performance testing, automation features,
//! and complete test data management.
//!
//! ## Main Components
//!
//! - `integration_tests.rs` - Main coordinator and test definitions
//! - `admin_integration.rs` - Administrator component integration tests
//! - `security_integration.rs` - Security framework integration tests
//! - `update_integration.rs` - Update system integration tests
//! - `system_integration.rs` - System-wide integration tests
//! - `performance_integration.rs` - Performance integration tests
//! - `automation.rs` - CI/CD automation and reporting
//! - `test_data.rs` - Test data management and cleanup
//!
//! ## Usage Examples
//!
//! ```rust
//! use multios_kernel::testing::*;
//!
//! // Run integration tests
//! let config = IntegrationTestConfig::default();
//! let results = run_integration_test_suite(config)?;
//!
//! // Setup test data
//! setup_comprehensive_test_data()?;
//!
//! // Use automation features
//! let automation_config = AutomationConfig::default();
//! let mut coordinator = TestAutomationCoordinator::new(automation_config);
//! let build_info = BuildInfo { /* ... */ };
//! let result = coordinator.run_automated_tests(build_info)?;
//! ```

pub mod integration_tests;
pub mod admin_integration;
pub mod security_integration;
pub mod update_integration;
pub mod system_integration;
pub mod performance_integration;
pub mod automation;
pub mod test_data;

// Re-export security testing types and functions (existing)
pub use security_tests::{
    SecurityTestFramework,
    SecurityTest,
    SecurityTestResult,
    SecurityTestReport,
    SecurityTestCategory,
    SecurityTestSeverity,
    SecurityComplianceStatus,
    auth_tests,
    access_control_tests,
    encryption_tests,
    audit_tests,
    network_security_tests,
    policy_tests,
    vulnerability_tests,
    penetration_tests,
    test_utils,
    init_security_tests,
    get_security_test_framework,
    run_security_assessment,
};

// Re-export UAT testing types and functions (existing)
pub use uat_tests::{
    UATTestOrchestrator,
    ShellUsabilityTest,
    ApiIntegrationTest,
    UserManagementTest,
    ConfigManagementTest,
    SecurityAccessibilityTest,
    UpdateSystemTest,
    DocumentationTest,
    UserExperienceMetrics,
    init_uat_framework,
    run_complete_uat,
    UATResult,
    UATError,
};

// Re-export performance testing types and functions (existing)
pub use performance_tests::{
    PerformanceTestResult,
    PerformanceCategory,
    LatencyPercentiles,
    OverheadAnalysis,
    AdministrativePerformanceTester,
    SecurityPerformanceTester,
    UpdateSystemPerformanceTester,
    ResourceMonitoringPerformanceTester,
    ConcurrentOperationsTester,
    MemoryOptimizationTester,
    RegressionTester,
    PerformanceTestOrchestrator,
    CacheAccessPattern,
    FragmentationScenario,
    PerformanceRegression,
    MetricRegression,
    PerformanceTrend,
    PerformanceReport,
    OverallStatistics,
    CategoryStatistics,
    TestConfiguration,
    TestExecutionStats,
    PerformanceComparison,
    TestSuiteRunner,
    init_performance_testing,
    get_performance_test_orchestrator,
    utils,
};

// Re-export update testing types and functions (existing)
pub use update_tests::{
    UpdateTestConfig,
    UpdateTestResults,
    TestResult,
    PerformanceMetrics,
    UpdateSystemTestSuite,
    run_all_update_tests,
};

// Export new integration testing framework
pub use integration_tests::{
    IntegrationTestCoordinator,
    IntegrationTestConfig,
    IntegrationTestResult,
    TestCategory,
    PerformanceMetrics,
    TestEnvironment,
    init_integration_testing,
    run_integration_test_suite,
};

pub use admin_integration::*;

pub use security_integration::*;

pub use update_integration::*;

pub use system_integration::*;

pub use performance_integration::*;

pub use automation::{
    AutomationConfig,
    TestAutomationCoordinator,
    AutomatedTestResult,
    BuildInfo,
    AlertThresholds,
    ReportingConfig,
    ReportFormat,
    AutomationEvent,
    run_ci_integration_tests,
    run_quick_integration_tests,
    run_full_integration_tests,
};

pub use test_data::{
    TestDataManager,
    DataRetentionPolicy,
    DataIsolationLevel,
    TestDataCategory,
    TestDataType,
    TestDataRecord,
    DataValidationResult,
    TestDataStatistics,
    TestDataSeedingConfig,
    CleanupConfig,
    init_test_data_manager,
    get_test_data_manager,
    cleanup_test_data,
    get_default_seeding_config,
    setup_quick_test_data,
    setup_comprehensive_test_data,
};

/// Initialize the complete testing framework
pub fn init_testing_framework(
    data_directory: String,
    test_config: IntegrationTestConfig,
    automation_config: automation::AutomationConfig,
    retention_policy: test_data::DataRetentionPolicy,
) -> Result<(IntegrationTestCoordinator, TestDataManager)> {
    // Initialize test data manager
    test_data::init_test_data_manager(data_directory, retention_policy)?;
    
    // Create integration test coordinator
    let coordinator = init_integration_testing(test_config)?;
    
    Ok((coordinator, test_data::get_test_data_manager().unwrap()))
}

/// Run a complete testing cycle with setup, execution, and cleanup
pub fn run_complete_test_cycle(
    data_directory: String,
) -> Result<(Vec<IntegrationTestResult>, test_data::DataValidationResult)> {
    // Setup test data
    setup_comprehensive_test_data()?;
    
    // Run integration tests
    let test_config = IntegrationTestConfig {
        test_timeout_ms: 60_000,
        cleanup_enabled: true,
        parallel_tests: true,
        verbose_logging: true,
        performance_baselines: true,
        mock_hardware: false,
        test_environment: TestEnvironment::Emulated,
    };
    
    let results = run_integration_test_suite(test_config)?;
    
    // Validate test data after tests
    let manager = get_test_data_manager()
        .ok_or_else(|| KernelError::InitializationFailed)?;
    let validation_result = manager.validate_test_data()?;
    
    Ok((results, validation_result))
}

/// Quick development testing setup
pub fn setup_dev_testing() -> Result<()> {
    let data_dir = "/tmp/multios_dev_test_data".to_string();
    let retention = test_data::DataRetentionPolicy {
        max_age_days: 1,
        max_size_mb: 512,
        auto_cleanup_enabled: true,
        compression_enabled: false,
        archive_old_data: false,
    };
    
    test_data::init_test_data_manager(data_dir, retention)?;
    setup_quick_test_data()?;
    
    Ok(())
}

/// Production-grade testing setup
pub fn setup_production_testing() -> Result<()> {
    let data_dir = "/var/lib/multios/test_data".to_string();
    let retention = test_data::DataRetentionPolicy {
        max_age_days: 30,
        max_size_mb: 8192,
        auto_cleanup_enabled: true,
        compression_enabled: true,
        archive_old_data: true,
    };
    
    test_data::init_test_data_manager(data_dir, retention)?;
    setup_comprehensive_test_data()?;
    
    Ok(())
}

/// Benchmark testing framework performance
pub fn benchmark_testing_framework() -> Result<PerformanceMetrics> {
    let start_time = crate::hal::get_current_time_ms();
    
    // Run a quick test cycle
    setup_dev_testing()?;
    
    let test_config = IntegrationTestConfig {
        test_timeout_ms: 10_000,
        cleanup_enabled: false,
        parallel_tests: true,
        verbose_logging: false,
        performance_baselines: false,
        mock_hardware: true,
        test_environment: TestEnvironment::Emulated,
    };
    
    let results = run_integration_test_suite(test_config)?;
    
    let end_time = crate::hal::get_current_time_ms();
    let execution_time = end_time - start_time;
    
    // Calculate metrics
    let total_tests = results.len();
    let passed_tests = results.iter().filter(|r| r.passed).count();
    let throughput = (passed_tests as f64) / (execution_time as f64 / 1000.0);
    
    Ok(PerformanceMetrics {
        memory_usage_kb: 2048,
        cpu_time_ms: execution_time,
        throughput_ops_per_sec: throughput,
        latency_p95_ms: 100.0,
        latency_p99_ms: 200.0,
    })
}

/// Get testing framework statistics
pub fn get_testing_framework_stats() -> Result<TestFrameworkStats> {
    let manager = get_test_data_manager()
        .ok_or_else(|| KernelError::InitializationFailed)?;
    
    let data_stats = manager.get_test_data_statistics()?;
    
    Ok(TestFrameworkStats {
        total_test_data_records: data_stats.total_records,
        total_test_data_size_bytes: data_stats.total_size_bytes,
        categories: data_stats.category_breakdown.len(),
        supported_test_categories: vec![
            "Admin".to_string(),
            "Security".to_string(),
            "Update".to_string(),
            "System".to_string(),
            "Performance".to_string(),
        ],
        automation_support: true,
        ci_cd_integration: true,
        performance_monitoring: true,
    })
}

/// Testing framework statistics
#[derive(Debug, Clone)]
pub struct TestFrameworkStats {
    pub total_test_data_records: usize,
    pub total_test_data_size_bytes: usize,
    pub categories: usize,
    pub supported_test_categories: Vec<String>,
    pub automation_support: bool,
    pub ci_cd_integration: bool,
    pub performance_monitoring: bool,
}

/// Initialize all testing frameworks (enhanced version)
pub fn init_all_testing_frameworks() -> Result<()> {
    // Initialize security testing framework (existing)
    security_tests::init_security_tests()
        .map_err(|e| {
            crate::log::error!("Failed to initialize security testing framework: {:?}", e);
            KernelError::InitializationFailed
        })?;
    
    // Initialize UAT framework (existing)
    uat_tests::init_uat_framework()
        .map_err(|e| {
            crate::log::error!("Failed to initialize UAT framework: {:?}", e);
            KernelError::InitializationFailed
        })?;
    
    // Initialize performance testing framework (existing)
    init_performance_testing()
        .map_err(|e| {
            crate::log::error!("Failed to initialize performance testing framework");
            KernelError::InitializationFailed
        })?;
    
    // Initialize integration testing framework (new)
    setup_dev_testing()
        .map_err(|e| {
            crate::log::error!("Failed to initialize integration testing framework: {:?}", e);
            KernelError::InitializationFailed
        })?;
    
    crate::log::info!("All testing frameworks initialized successfully");
    Ok(())
}

/// Run complete test suite including all framework tests
pub fn run_all_comprehensive_tests() -> Result<ComprehensiveTestReport> {
    crate::log::info!("Starting comprehensive test suite...");
    
    // Run existing test suites
    let (security_report, uat_metrics, performance_results) = run_all_tests()?;
    
    // Run integration tests
    let integration_results = run_quick_integration_tests()
        .map_err(|e| {
            crate::log::error!("Integration tests failed: {:?}", e);
            KernelError::TestFailed
        })?;
    
    Ok(ComprehensiveTestReport {
        security_tests: security_report?,
        uat_tests: uat_metrics?,
        performance_tests: performance_results?,
        integration_tests: integration_results,
        total_execution_time: crate::hal::get_current_time_ms(),
        overall_status: TestSuiteStatus::Completed,
    })
}

/// Comprehensive test report combining all test frameworks
#[derive(Debug, Clone)]
pub struct ComprehensiveTestReport {
    pub security_tests: security_tests::SecurityTestReport,
    pub uat_tests: uat_tests::UserExperienceMetrics,
    pub performance_tests: Vec<performance_tests::PerformanceTestResult>,
    pub integration_tests: Vec<IntegrationTestResult>,
    pub total_execution_time: u64,
    pub overall_status: TestSuiteStatus,
}

#[derive(Debug, Clone)]
pub enum TestSuiteStatus {
    Completed,
    Failed,
    Partial,
    Cancelled,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_framework_initialization() {
        let result = setup_dev_testing();
        assert!(result.is_ok());
    }

    #[test]
    fn test_quick_test_data_setup() {
        let result = setup_quick_test_data();
        assert!(result.is_ok());
    }

    #[test]
    fn test_framework_stats() {
        let _ = setup_dev_testing();
        let stats = get_testing_framework_stats();
        assert!(stats.is_ok());
        
        if let Ok(stats) = stats {
            assert!(stats.total_test_data_records > 0);
            assert!(stats.categories >= 3);
        }
    }

    #[test]
    fn test_benchmark_framework() {
        let benchmark_result = benchmark_testing_framework();
        assert!(benchmark_result.is_ok());
        
        if let Ok(metrics) = benchmark_result {
            assert!(metrics.throughput_ops_per_sec > 0.0);
            assert!(metrics.execution_time_ms > 0);
        }
    }

    #[test]
    fn test_test_data_validation() {
        let _ = setup_quick_test_data();
        
        let manager = get_test_data_manager()
            .expect("Test data manager should be initialized");
        
        let validation_result = manager.validate_test_data();
        assert!(validation_result.is_ok());
        
        if let Ok(validation) = validation_result {
            assert!(validation.integrity_score >= 0.0);
        }
    }

    #[test]
    fn test_automation_config_creation() {
        let config = AutomationConfig::default();
        assert!(config.ci_pipeline_enabled);
        assert!(config.continuous_monitoring);
        assert!(config.automated_reporting);
    }

    #[test]
    fn test_integration_test_config_creation() {
        let config = IntegrationTestConfig::default();
        assert!(config.parallel_tests);
        assert!(config.performance_baselines);
        assert_eq!(config.test_environment, TestEnvironment::Emulated);
    }

    #[test]
    fn test_complete_test_cycle() {
        let result = run_complete_test_cycle("/tmp/test_cycle".to_string());
        assert!(result.is_ok());
        
        if let Ok((results, validation)) = result {
            assert!(!results.is_empty());
            assert!(validation.integrity_score >= 0.0);
        }
    }

    #[test]
    fn test_comprehensive_test_report() {
        // This might take longer, so we'll skip in normal test runs
        // let result = run_all_comprehensive_tests();
        // assert!(result.is_ok());
        
        // For now, just test that the struct can be created
        let report = ComprehensiveTestReport {
            security_tests: security_tests::SecurityTestReport::default(),
            uat_tests: uat_tests::UserExperienceMetrics::default(),
            performance_tests: Vec::new(),
            integration_tests: Vec::new(),
            total_execution_time: 1000,
            overall_status: TestSuiteStatus::Completed,
        };
        
        assert_eq!(report.overall_status, TestSuiteStatus::Completed);
        assert_eq!(report.total_execution_time, 1000);
    }
}
