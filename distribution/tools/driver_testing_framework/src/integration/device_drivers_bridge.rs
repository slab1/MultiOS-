//! Integration Bridge for Device Drivers Testing Framework
//!
//! This module provides seamless integration between the comprehensive driver testing framework
//! and the existing device-drivers crate testing infrastructure. It enables leveraging
//! the advanced testing capabilities while maintaining compatibility with existing driver code.

use crate::core::*;
use crate::simulation::*;
use crate::stress_testing::*;
use crate::performance::*;
use crate::validation::*;
use crate::debugging::*;
use crate::troubleshooting::*;
use crate::reporting::*;

use crate::AdvancedDriverId;
use crate::AdvancedDriverError;
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use core::time::Duration;

/// Bridge configuration for integrating with device-drivers crate
#[derive(Debug, Clone)]
pub struct DeviceDriversIntegrationConfig {
    /// Enable legacy test compatibility
    pub enable_legacy_compatibility: bool,
    /// Auto-generate tests from existing test suites
    pub auto_generate_tests: bool,
    /// Bridge timeout in milliseconds
    pub bridge_timeout_ms: u64,
    /// Enable advanced testing features
    pub enable_advanced_features: bool,
}

impl Default for DeviceDriversIntegrationConfig {
    fn default() -> Self {
        Self {
            enable_legacy_compatibility: true,
            auto_generate_tests: true,
            bridge_timeout_ms: 5000,
            enable_advanced_features: true,
        }
    }
}

/// Test result converter between frameworks
#[derive(Debug, Clone)]
pub struct TestResultConverter {
    /// Mapping from legacy test categories to new framework categories
    category_mapping: BTreeMap<String, TestCategory>,
    /// Mapping from legacy test types to new framework types
    type_mapping: BTreeMap<String, crate::core::TestType>,
}

impl TestResultConverter {
    /// Create a new test result converter
    pub fn new() -> Self {
        let mut category_mapping = BTreeMap::new();
        category_mapping.insert("Unit".to_string(), TestCategory::Unit);
        category_mapping.insert("Integration".to_string(), TestCategory::Integration);
        category_mapping.insert("Performance".to_string(), TestCategory::Performance);
        category_mapping.insert("Stress".to_string(), TestCategory::Stress);
        category_mapping.insert("Compatibility".to_string(), TestCategory::Compatibility);
        category_mapping.insert("Reliability".to_string(), TestCategory::Reliability);
        category_mapping.insert("Security".to_string(), TestCategory::Security);
        category_mapping.insert("Regression".to_string(), TestCategory::Regression);

        let mut type_mapping = BTreeMap::new();
        type_mapping.insert("Unit".to_string(), crate::core::TestType::Unit);
        type_mapping.insert("Integration".to_string(), crate::core::TestType::Integration);
        type_mapping.insert("Performance".to_string(), crate::core::TestType::Performance);
        type_mapping.insert("Stress".to_string(), crate::core::TestType::Stress);
        type_mapping.insert("Compatibility".to_string(), crate::core::TestType::Compatibility);

        Self {
            category_mapping,
            type_mapping,
        }
    }

    /// Convert legacy test result to new framework result
    pub fn convert_test_result(&self, legacy_result: TestResult) -> crate::core::TestStatus {
        match legacy_result {
            TestResult::Pass => crate::core::TestStatus::Passed,
            TestResult::Fail => crate::core::TestStatus::Failed,
            TestResult::Skip => crate::core::TestStatus::Skipped,
            TestResult::Timeout => crate::core::TestStatus::Timeout,
            TestResult::Error => crate::core::TestStatus::Error,
            TestResult::NotImplemented => crate::core::TestStatus::Skipped,
        }
    }

    /// Convert legacy test category to new framework category
    pub fn convert_category(&self, category: &str) -> TestCategory {
        self.category_mapping.get(category).cloned().unwrap_or(TestCategory::Custom)
    }

    /// Convert legacy test type to new framework type
    pub fn convert_test_type(&self, test_type: &str) -> crate::core::TestType {
        self.type_mapping.get(test_type).cloned().unwrap_or(crate::core::TestType::Custom)
    }
}

/// Advanced test suite builder using the comprehensive framework
pub struct AdvancedTestSuiteBuilder {
    config: DeviceDriversIntegrationConfig,
    simulation_env: Option<SimulationEnvironment>,
    stress_config: Option<StressTestConfig>,
    performance_config: Option<PerformanceBenchmarkConfig>,
    validation_config: Option<ValidationConfig>,
    debug_config: Option<DebugConfig>,
}

impl AdvancedTestSuiteBuilder {
    /// Create a new advanced test suite builder
    pub fn new(config: DeviceDriversIntegrationConfig) -> Self {
        Self {
            config,
            simulation_env: None,
            stress_config: None,
            performance_config: None,
            validation_config: None,
            debug_config: None,
        }
    }

    /// Add simulation environment
    pub fn with_simulation_env(mut self, env: SimulationEnvironment) -> Self {
        self.simulation_env = Some(env);
        self
    }

    /// Add stress testing configuration
    pub fn with_stress_config(mut self, config: StressTestConfig) -> Self {
        self.stress_config = Some(config);
        self
    }

    /// Add performance benchmarking configuration
    pub fn with_performance_config(mut self, config: PerformanceBenchmarkConfig) -> Self {
        self.performance_config = Some(config);
        self
    }

    /// Add validation configuration
    pub fn with_validation_config(mut self, config: ValidationConfig) -> Self {
        self.validation_config = Some(config);
        self
    }

    /// Add debugging configuration
    pub fn with_debug_config(mut self, config: DebugConfig) -> Self {
        self.debug_config = Some(config);
        self
    }

    /// Build and run advanced tests for the given driver
    pub async fn build_and_run(&self, driver_id: AdvancedDriverId) -> Result<ComprehensiveTestResults, DriverTestError> {
        let mut results = ComprehensiveTestResults::new();

        // Initialize simulation environment if configured
        if let Some(sim_env) = &self.simulation_env {
            let sim_result = sim_env.create_simulation_for_driver(driver_id).await?;
            results.add_simulation_result(sim_result);
        }

        // Run stress tests if configured
        if let Some(stress_config) = &self.stress_config {
            let stress_runner = StressTestRunner::new(stress_config.clone());
            let stress_results = stress_runner.run_driver_stress_tests(driver_id).await?;
            results.add_stress_test_results(stress_results);
        }

        // Run performance tests if configured
        if let Some(perf_config) = &self.performance_config {
            let perf_runner = PerformanceBenchmarkRunner::new(perf_config.clone());
            let perf_results = perf_runner.benchmark_driver_performance(driver_id).await?;
            results.add_performance_results(perf_results);
        }

        // Run validation tests if configured
        if let Some(validation_config) = &self.validation_config {
            let validator = DriverValidator::new(validation_config.clone());
            let validation_results = validator.validate_driver_compliance(driver_id).await?;
            results.add_validation_results(validation_results);
        }

        Ok(results)
    }
}

/// Bridge between legacy and advanced testing frameworks
pub struct DeviceDriversTestBridge {
    config: DeviceDriversIntegrationConfig,
    converter: TestResultConverter,
    advanced_suite_builder: Option<AdvancedTestSuiteBuilder>,
    legacy_test_manager: Option<crate::TestManager>,
}

impl DeviceDriversTestBridge {
    /// Create a new test bridge
    pub fn new(config: DeviceDriversIntegrationConfig) -> Self {
        Self {
            config: config.clone(),
            converter: TestResultConverter::new(),
            advanced_suite_builder: Some(AdvancedTestSuiteBuilder::new(config)),
            legacy_test_manager: None,
        }
    }

    /// Set up legacy test manager compatibility
    pub fn setup_legacy_compatibility(&mut self, test_manager: crate::TestManager) {
        if self.config.enable_legacy_compatibility {
            self.legacy_test_manager = Some(test_manager);
        }
    }

    /// Run comprehensive tests combining legacy and advanced capabilities
    pub async fn run_comprehensive_tests(
        &mut self,
        driver_id: AdvancedDriverId,
        legacy_suite_name: Option<&str>,
    ) -> Result<ComprehensiveTestResults, DriverTestError> {
        let mut results = ComprehensiveTestResults::new();

        // Run legacy tests if available
        if let (Some(test_manager), Some(suite_name)) = (&mut self.legacy_test_manager, legacy_suite_name) {
            let legacy_results = test_manager.run_test_suite(driver_id, suite_name)
                .map_err(|e| DriverTestError::TestExecutionError(format!("Legacy test failed: {:?}", e)))?;
            
            // Convert legacy results to new format
            for test_result in &legacy_results.individual_results {
                let converted_status = self.converter.convert_test_result(test_result.result);
                let new_result = TestResult {
                    name: test_result.test_name.to_string(),
                    status: converted_status,
                    duration: Duration::from_millis(test_result.duration_ms),
                    message: test_result.error_message.clone().unwrap_or_default(),
                    category: TestCategory::Unit, // Default category for legacy tests
                };
                results.add_result(new_result);
            }
        }

        // Run advanced tests if suite builder is available
        if let Some(suite_builder) = &self.advanced_suite_builder {
            let advanced_results = suite_builder.build_and_run(driver_id).await?;
            results.merge(advanced_results);
        }

        Ok(results)
    }

    /// Create advanced simulation for legacy tests
    pub async fn create_advanced_simulation(
        &mut self,
        driver_id: AdvancedDriverId,
        simulation_type: &str,
    ) -> Result<SimulationResult, DriverTestError> {
        let simulation_env = SimulationEnvironment::default();
        
        let mock_device = match simulation_type {
            "keyboard" => simulation_env.create_mock_keyboard().await?,
            "serial" => simulation_env.create_mock_serial_port().await?,
            "timer" => simulation_env.create_mock_timer().await?,
            _ => return Err(DriverTestError::HardwareSimulationError(
                format!("Unsupported simulation type: {}", simulation_type)
            )),
        };
        
        Ok(SimulationResult {
            driver_id,
            device: mock_device,
            simulation_environment: "advanced".to_string(),
            success: true,
            details: format!("Advanced simulation created for {} driver", simulation_type),
        })
    }

    /// Generate advanced tests from legacy test definitions
    pub fn generate_advanced_tests_from_legacy(
        &self,
        legacy_tests: &[crate::Test],
    ) -> Result<Vec<DriverTest>, DriverTestError> {
        let mut advanced_tests = Vec::new();
        
        for legacy_test in legacy_tests {
            let category = self.converter.convert_category(match legacy_test.category {
                crate::TestCategory::Unit => "Unit",
                crate::TestCategory::Operations => "Integration",
                crate::TestCategory::ErrorHandling => "Unit",
                crate::TestCategory::Performance => "Performance",
                crate::TestCategory::Stress => "Stress",
                crate::TestCategory::Compatibility => "Compatibility",
                crate::TestCategory::Compliance => "Integration",
                crate::TestCategory::Custom => "Unit",
            });
            
            let test_type = self.converter.convert_test_type(match legacy_test.test_type {
                crate::TestType::Unit => "Unit",
                crate::TestType::Integration => "Integration",
                crate::TestType::Performance => "Performance",
                crate::TestType::Stress => "Stress",
                crate::TestType::Compatibility => "Compatibility",
                crate::TestType::Reliability => "Unit",
                crate::TestType::Security => "Integration",
                crate::TestType::Regression => "Unit",
            });

            let test_config = TestConfig::new(
                legacy_test.name.to_string(),
                category
            )
            .with_timeout(Duration::from_millis(legacy_test.timeout_ms))
            .with_critical(legacy_test.critical);

            let advanced_test = DriverTest::new(test_config, Box::new(move |driver_id| {
                // Convert legacy test function to new format
                let legacy_test = legacy_test; // Capture for closure
                Box::pin(async move {
                    let mut context = crate::TestContext::new(driver_id, legacy_test.name);
                    let result = (legacy_test.test_func)(&mut context);
                    
                    Ok(TestResult {
                        name: legacy_test.name.to_string(),
                        status: self.converter.convert_test_result(result),
                        duration: Duration::from_millis(context.get_duration().unwrap_or(0)),
                        message: "Converted from legacy test".to_string(),
                        category,
                    })
                })
            }));

            advanced_tests.push(advanced_test);
        }

        Ok(advanced_tests)
    }

    /// Get integration statistics
    pub fn get_integration_statistics(&self) -> IntegrationStatistics {
        IntegrationStatistics {
            legacy_compatibility_enabled: self.config.enable_legacy_compatibility,
            advanced_features_enabled: self.config.enable_advanced_features,
            has_simulation_env: self.simulation_env.is_some(),
            has_stress_config: self.stress_config.is_some(),
            has_performance_config: self.performance_config.is_some(),
            has_validation_config: self.validation_config.is_some(),
            has_debug_config: self.debug_config.is_some(),
        }
    }
}

impl Default for DeviceDriversTestBridge {
    fn default() -> Self {
        Self::new(DeviceDriversIntegrationConfig::default())
    }
}

/// Integration statistics
#[derive(Debug, Clone)]
pub struct IntegrationStatistics {
    pub legacy_compatibility_enabled: bool,
    pub advanced_features_enabled: bool,
    pub has_simulation_env: bool,
    pub has_stress_config: bool,
    pub has_performance_config: bool,
    pub has_validation_config: bool,
    pub has_debug_config: bool,
}

/// Comprehensive test results combining legacy and advanced testing
#[derive(Debug, Clone)]
pub struct ComprehensiveTestResults {
    pub legacy_results: Vec<TestResult>,
    pub simulation_results: Vec<SimulationResult>,
    pub stress_test_results: Vec<StressTestResult>,
    pub performance_results: Vec<PerformanceBenchmarkResult>,
    pub validation_results: Vec<ValidationResult>,
    pub debug_results: Vec<DebugResult>,
    pub troubleshooting_results: Vec<TroubleshootingResult>,
}

impl ComprehensiveTestResults {
    /// Create new comprehensive results
    pub fn new() -> Self {
        Self {
            legacy_results: Vec::new(),
            simulation_results: Vec::new(),
            stress_test_results: Vec::new(),
            performance_results: Vec::new(),
            validation_results: Vec::new(),
            debug_results: Vec::new(),
            troubleshooting_results: Vec::new(),
        }
    }

    /// Add a legacy test result
    pub fn add_result(&mut self, result: TestResult) {
        self.legacy_results.push(result);
    }

    /// Add simulation results
    pub fn add_simulation_result(&mut self, result: SimulationResult) {
        self.simulation_results.push(result);
    }

    /// Add stress test results
    pub fn add_stress_test_results(&mut self, results: Vec<StressTestResult>) {
        self.stress_test_results.extend(results);
    }

    /// Add performance results
    pub fn add_performance_results(&mut self, results: Vec<PerformanceBenchmarkResult>) {
        self.performance_results.extend(results);
    }

    /// Add validation results
    pub fn add_validation_results(&mut self, results: Vec<ValidationResult>) {
        self.validation_results.extend(results);
    }

    /// Merge with other comprehensive results
    pub fn merge(&mut self, other: ComprehensiveTestResults) {
        self.legacy_results.extend(other.legacy_results);
        self.simulation_results.extend(other.simulation_results);
        self.stress_test_results.extend(other.stress_test_results);
        self.performance_results.extend(other.performance_results);
        self.validation_results.extend(other.validation_results);
        self.debug_results.extend(other.debug_results);
        self.troubleshooting_results.extend(other.troubleshooting_results);
    }

    /// Get total number of tests executed
    pub fn total_tests(&self) -> usize {
        self.legacy_results.len()
            + self.simulation_results.len()
            + self.stress_test_results.len()
            + self.performance_results.len()
            + self.validation_results.len()
            + self.debug_results.len()
            + self.troubleshooting_results.len()
    }

    /// Get number of passed tests
    pub fn passed_tests(&self) -> usize {
        self.legacy_results.iter()
            .filter(|r| r.status == TestStatus::Passed)
            .count()
    }

    /// Get number of failed tests
    pub fn failed_tests(&self) -> usize {
        self.legacy_results.iter()
            .filter(|r| r.status == TestStatus::Failed)
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_test_result_converter() {
        let converter = TestResultConverter::new();
        
        assert_eq!(
            converter.convert_test_result(TestResult::Pass),
            crate::core::TestStatus::Passed
        );
        assert_eq!(
            converter.convert_test_result(TestResult::Fail),
            crate::core::TestStatus::Failed
        );
        assert_eq!(
            converter.convert_test_result(TestResult::Skip),
            crate::core::TestStatus::Skipped
        );
    }

    #[test]
    fn test_category_conversion() {
        let converter = TestResultConverter::new();
        
        assert_eq!(
            converter.convert_category("Unit"),
            TestCategory::Unit
        );
        assert_eq!(
            converter.convert_category("Performance"),
            TestCategory::Performance
        );
    }

    #[test]
    fn test_comprehensive_results() {
        let mut results = ComprehensiveTestResults::new();
        
        results.add_result(TestResult {
            name: "test_1".to_string(),
            status: TestStatus::Passed,
            duration: Duration::from_millis(100),
            message: "Test passed".to_string(),
            category: TestCategory::Unit,
        });
        
        assert_eq!(results.total_tests(), 1);
        assert_eq!(results.passed_tests(), 1);
        assert_eq!(results.failed_tests(), 0);
    }

    #[test]
    fn test_integration_bridge_creation() {
        let config = DeviceDriversIntegrationConfig::default();
        let bridge = DeviceDriversTestBridge::new(config);
        
        let stats = bridge.get_integration_statistics();
        assert!(stats.legacy_compatibility_enabled);
        assert!(stats.advanced_features_enabled);
    }
}