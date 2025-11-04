//! Test Automation and CI/CD Integration
//! 
//! This module provides automation features for integration testing:
//! - CI/CD pipeline integration
//! - Automated test execution
//! - Test result reporting and visualization
//! - Continuous monitoring and alerting
//! - Test environment provisioning

use super::*;
use crate::*;
use crate::Result;
use log::{info, warn, error};

/// Test automation configuration
#[derive(Debug, Clone)]
pub struct AutomationConfig {
    pub ci_pipeline_enabled: bool,
    pub continuous_monitoring: bool,
    pub automated_reporting: bool,
    pub test_provisioning: bool,
    pub alert_thresholds: AlertThresholds,
    pub reporting_config: ReportingConfig,
}

/// Alert thresholds for automation
#[derive(Debug, Clone)]
pub struct AlertThresholds {
    pub test_failure_rate: f64,        // Percentage
    pub performance_regression: f64,   // Percentage
    pub resource_utilization: f64,     // Percentage
    pub response_time_threshold: u64,  // Milliseconds
}

/// Reporting configuration
#[derive(Debug, Clone)]
pub struct ReportingConfig {
    pub output_format: ReportFormat,
    pub include_performance_data: bool,
    pub include_coverage_data: bool,
    pub generate_html_reports: bool,
    pub send_email_notifications: bool,
    pub webhook_urls: Vec<String>,
}

/// Report output formats
#[derive(Debug, Clone)]
pub enum ReportFormat {
    Json,
    Xml,
    Html,
    Markdown,
    JunitXml,
}

/// Test automation coordinator
pub struct TestAutomationCoordinator {
    config: AutomationConfig,
    test_runner: AutomatedTestRunner,
    monitor: ContinuousMonitor,
    reporter: AutomatedReporter,
    provisioner: TestEnvironmentProvisioner,
}

/// Automated test runner
pub struct AutomatedTestRunner {
    pub parallel_execution: bool,
    pub retry_failed_tests: bool,
    pub max_retries: usize,
    pub timeout_per_test: u64,
}

/// Continuous monitoring system
pub struct ContinuousMonitor {
    pub monitoring_enabled: bool,
    pub check_interval_ms: u64,
    pub metrics_collection: MetricsCollector,
}

/// Automated reporting system
pub struct AutomatedReporter {
    pub report_generation_enabled: bool,
    pub output_directory: String,
    pub historical_data_retention_days: usize,
}

/// Test environment provisioner
pub struct TestEnvironmentProvisioner {
    pub auto_provisioning: bool,
    pub cleanup_after_tests: bool,
    pub resource_limits: ResourceLimits,
}

/// Metrics collector for continuous monitoring
#[derive(Debug, Clone)]
pub struct MetricsCollector {
    pub memory_usage_tracking: bool,
    pub cpu_usage_tracking: bool,
    pub latency_tracking: bool,
    pub throughput_tracking: bool,
    pub error_rate_tracking: bool,
}

/// Resource limits for test environments
#[derive(Debug, Clone)]
pub struct ResourceLimits {
    pub max_memory_mb: usize,
    pub max_cpu_cores: usize,
    pub max_disk_space_gb: usize,
    pub max_network_bandwidth_mbps: usize,
}

/// Test execution result for automation
#[derive(Debug, Clone)]
pub struct AutomatedTestResult {
    pub test_suite_name: String,
    pub execution_timestamp: u64,
    pub total_tests: usize,
    pub passed_tests: usize,
    pub failed_tests: usize,
    pub skipped_tests: usize,
    pub execution_time_ms: u64,
    pub performance_metrics: Option<PerformanceMetrics>,
    pub error_details: Vec<String>,
    pub build_info: BuildInfo,
}

/// Build information for CI/CD integration
#[derive(Debug, Clone)]
pub struct BuildInfo {
    pub build_number: String,
    pub commit_sha: String,
    pub branch_name: String,
    pub build_timestamp: u64,
    pub triggered_by: String,
}

/// Continuous monitoring data
#[derive(Debug, Clone)]
pub struct MonitoringData {
    pub timestamp: u64,
    pub test_environment_status: EnvironmentStatus,
    pub system_metrics: SystemMetrics,
    pub integration_metrics: IntegrationMetrics,
}

/// System performance metrics
#[derive(Debug, Clone)]
pub struct SystemMetrics {
    pub cpu_usage_percent: f64,
    pub memory_usage_mb: usize,
    pub disk_usage_percent: f64,
    pub network_latency_ms: f64,
}

/// Integration-specific metrics
#[derive(Debug, Clone)]
pub struct IntegrationMetrics {
    pub component_interaction_latency_ms: f64,
    pub cross_component_success_rate: f64,
    pub system_integration_score: f64,
}

/// Test environment status
#[derive(Debug, Clone)]
pub enum EnvironmentStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

/// Automation events for CI/CD integration
#[derive(Debug, Clone)]
pub enum AutomationEvent {
    TestSuiteStarted(String),
    TestSuiteCompleted(AutomatedTestResult),
    PerformanceRegression(String, f64),
    EnvironmentIssue(String),
    BuildFailed(String),
    BuildSuccessful(String),
}

impl Default for AutomationConfig {
    fn default() -> Self {
        Self {
            ci_pipeline_enabled: true,
            continuous_monitoring: true,
            automated_reporting: true,
            test_provisioning: true,
            alert_thresholds: AlertThresholds {
                test_failure_rate: 5.0,
                performance_regression: 10.0,
                resource_utilization: 80.0,
                response_time_threshold: 5000,
            },
            reporting_config: ReportingConfig {
                output_format: ReportFormat::Json,
                include_performance_data: true,
                include_coverage_data: true,
                generate_html_reports: true,
                send_email_notifications: false,
                webhook_urls: Vec::new(),
            },
        }
    }
}

impl TestAutomationCoordinator {
    /// Create a new test automation coordinator
    pub fn new(config: AutomationConfig) -> Self {
        Self {
            config: config.clone(),
            test_runner: AutomatedTestRunner {
                parallel_execution: true,
                retry_failed_tests: true,
                max_retries: 3,
                timeout_per_test: 30000,
            },
            monitor: ContinuousMonitor {
                monitoring_enabled: config.continuous_monitoring,
                check_interval_ms: 30000,
                metrics_collection: MetricsCollector {
                    memory_usage_tracking: true,
                    cpu_usage_tracking: true,
                    latency_tracking: true,
                    throughput_tracking: true,
                    error_rate_tracking: true,
                },
            },
            reporter: AutomatedReporter {
                report_generation_enabled: config.automated_reporting,
                output_directory: "/tmp/multios_test_reports".to_string(),
                historical_data_retention_days: 30,
            },
            provisioner: TestEnvironmentProvisioner {
                auto_provisioning: config.test_provisioning,
                cleanup_after_tests: true,
                resource_limits: ResourceLimits {
                    max_memory_mb: 8192,
                    max_cpu_cores: 8,
                    max_disk_space_gb: 50,
                    max_network_bandwidth_mbps: 1000,
                },
            },
        }
    }

    /// Run automated integration test suite
    pub fn run_automated_tests(&mut self, build_info: BuildInfo) -> Result<AutomatedTestResult> {
        info!("Starting automated integration test suite...");
        
        // Send automation event
        self.send_automation_event(AutomationEvent::TestSuiteStarted("integration_tests".to_string()));
        
        // Provision test environment if needed
        if self.provisioner.auto_provisioning {
            self.provision_test_environment()?;
        }
        
        // Start continuous monitoring
        if self.monitor.monitoring_enabled {
            self.start_continuous_monitoring();
        }
        
        // Run the integration test suite
        let test_config = IntegrationTestConfig {
            test_timeout_ms: self.test_runner.timeout_per_test,
            cleanup_enabled: true,
            parallel_tests: self.test_runner.parallel_execution,
            verbose_logging: true,
            performance_baselines: true,
            mock_hardware: false,
            test_environment: TestEnvironment::Emulated,
        };
        
        let start_time = crate::hal::get_current_time_ms();
        let test_results = run_integration_test_suite(test_config)?;
        let end_time = crate::hal::get_current_time_ms();
        
        // Analyze results
        let total_tests = test_results.len();
        let passed_tests = test_results.iter().filter(|r| r.passed).count();
        let failed_tests = total_tests - passed_tests;
        let skipped_tests = 0; // No skipped tests in current implementation
        
        // Calculate overall performance metrics
        let avg_execution_time = test_results.iter()
            .map(|r| r.execution_time_ms)
            .sum::<u64>() as f64 / total_tests as f64;
        
        let avg_memory_usage = test_results.iter()
            .filter_map(|r| r.performance_metrics.as_ref().map(|m| m.memory_usage_kb as f64))
            .sum::<f64>() / test_results.iter().filter_map(|r| r.performance_metrics.as_ref()).count() as f64;
        
        let avg_throughput = test_results.iter()
            .filter_map(|r| r.performance_metrics.as_ref().map(|m| m.throughput_ops_per_sec))
            .sum::<f64>() / test_results.iter().filter_map(|r| r.performance_metrics.as_ref()).count() as f64;
        
        let performance_metrics = Some(PerformanceMetrics {
            memory_usage_kb: avg_memory_usage as usize,
            cpu_time_ms: avg_execution_time as u64,
            throughput_ops_per_sec: avg_throughput,
            latency_p95_ms: 0.0, // Would need to calculate from individual tests
            latency_p99_ms: 0.0,
        });
        
        // Collect error details
        let error_details: Vec<String> = test_results.iter()
            .filter(|r| !r.passed)
            .flat_map(|r| {
                if let Some(error) = &r.error_message {
                    vec![format!("{}: {}", r.test_name, error)]
                } else {
                    vec![format!("{}: Unknown error", r.test_name)]
                }
            })
            .collect();
        
        // Check for performance regressions
        self.check_performance_regressions(&test_results)?;
        
        // Generate automated report
        if self.reporter.report_generation_enabled {
            self.generate_automated_report(&test_results, &build_info)?;
        }
        
        // Stop continuous monitoring
        if self.monitor.monitoring_enabled {
            self.stop_continuous_monitoring();
        }
        
        // Cleanup test environment
        if self.provisioner.cleanup_after_tests {
            self.cleanup_test_environment()?;
        }
        
        let automated_result = AutomatedTestResult {
            test_suite_name: "integration_tests".to_string(),
            execution_timestamp: crate::hal::get_current_time_ms(),
            total_tests,
            passed_tests,
            failed_tests,
            skipped_tests,
            execution_time_ms: end_time - start_time,
            performance_metrics,
            error_details,
            build_info,
        };
        
        // Send completion event
        self.send_automation_event(AutomationEvent::TestSuiteCompleted(automated_result.clone()));
        
        // Check if build should be marked as failed
        if failed_tests > 0 || !error_details.is_empty() {
            self.send_automation_event(AutomationEvent::BuildFailed(
                format!("{} tests failed out of {}", failed_tests, total_tests)
            ));
        } else {
            self.send_automation_event(AutomationEvent::BuildSuccessful(
                format!("All {} tests passed", total_tests)
            ));
        }
        
        info!("Automated integration test suite completed: {}/{} passed", 
              passed_tests, total_tests);
        
        Ok(automated_result)
    }

    /// Provision test environment
    fn provision_test_environment(&mut self) -> Result<()> {
        info!("Provisioning test environment...");
        
        // Check resource availability
        let current_memory = crate::memory::get_memory_stats();
        if current_memory.total_pages * 4096 / 1024 / 1024 < self.provisioner.resource_limits.max_memory_mb {
            return Err(KernelError::ResourceExhausted);
        }
        
        // Create test directories
        let _ = crate::filesystem::create_directory("/tmp/multios_test_environment");
        let _ = crate::filesystem::create_directory("/tmp/multios_test_data");
        let _ = crate::filesystem::create_directory("/tmp/multios_test_reports");
        
        info!("Test environment provisioned successfully");
        Ok(())
    }

    /// Start continuous monitoring
    fn start_continuous_monitoring(&mut self) {
        info!("Starting continuous monitoring...");
        // In a real implementation, this would start background monitoring tasks
    }

    /// Stop continuous monitoring
    fn stop_continuous_monitoring(&mut self) {
        info!("Stopping continuous monitoring...");
        // In a real implementation, this would stop background monitoring tasks
    }

    /// Check for performance regressions
    fn check_performance_regressions(&self, test_results: &[IntegrationTestResult]) -> Result<()> {
        let regression_threshold = self.config.alert_thresholds.performance_regression;
        
        for result in test_results {
            if let Some(metrics) = &result.performance_metrics {
                // Check latency regression
                if metrics.latency_p95_ms > regression_threshold {
                    self.send_automation_event(AutomationEvent::PerformanceRegression(
                        result.test_name.clone(),
                        metrics.latency_p95_ms
                    ));
                }
                
                // Check throughput regression
                if metrics.throughput_ops_per_sec < 50.0 { // Baseline threshold
                    self.send_automation_event(AutomationEvent::PerformanceRegression(
                        result.test_name.clone(),
                        100.0 - metrics.throughput_ops_per_sec
                    ));
                }
            }
        }
        
        Ok(())
    }

    /// Generate automated report
    fn generate_automated_report(&self, test_results: &[IntegrationTestResult], 
                                build_info: &BuildInfo) -> Result<()> {
        info!("Generating automated test report...");
        
        let report_data = self.create_report_data(test_results, build_info);
        
        match self.config.reporting_config.output_format {
            ReportFormat::Json => {
                let json_report = serde_json::to_string_pretty(&report_data)?;
                let _ = crate::filesystem::write_file(
                    &format!("{}/test_report_{}.json", self.reporter.output_directory, build_info.build_number),
                    json_report.as_bytes()
                );
            }
            ReportFormat::Html => {
                let html_report = self.generate_html_report(&report_data)?;
                let _ = crate::filesystem::write_file(
                    &format!("{}/test_report_{}.html", self.reporter.output_directory, build_info.build_number),
                    html_report.as_bytes()
                );
            }
            _ => {
                warn!("Report format {:?} not yet implemented", self.config.reporting_config.output_format);
            }
        }
        
        // Send webhooks if configured
        for webhook_url in &self.config.reporting_config.webhook_urls {
            self.send_webhook(webhook_url, &report_data)?;
        }
        
        info!("Automated test report generated");
        Ok(())
    }

    /// Create report data structure
    fn create_report_data(&self, test_results: &[IntegrationTestResult], 
                         build_info: &BuildInfo) -> TestReportData {
        TestReportData {
            build_info: build_info.clone(),
            execution_timestamp: crate::hal::get_current_time_ms(),
            summary: TestSummary {
                total_tests: test_results.len(),
                passed: test_results.iter().filter(|r| r.passed).count(),
                failed: test_results.iter().filter(|r| !r.passed).count(),
                skipped: 0,
                execution_time_ms: test_results.iter().map(|r| r.execution_time_ms).sum(),
            },
            test_categories: self.group_tests_by_category(test_results),
            performance_metrics: self.aggregate_performance_metrics(test_results),
            error_analysis: self.analyze_errors(test_results),
            recommendations: self.generate_recommendations(test_results),
        }
    }

    /// Generate HTML report
    fn generate_html_report(&self, report_data: &TestReportData) -> Result<String> {
        let html = format!(r#"
<!DOCTYPE html>
<html>
<head>
    <title>MultiOS Integration Test Report</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 40px; }}
        .header {{ background-color: #f4f4f4; padding: 20px; border-radius: 5px; }}
        .summary {{ background-color: #e8f5e8; padding: 15px; margin: 20px 0; border-radius: 5px; }}
        .test-result {{ margin: 10px 0; padding: 10px; border-left: 4px solid #ccc; }}
        .passed {{ border-left-color: #4caf50; }}
        .failed {{ border-left-color: #f44336; }}
        .metrics {{ background-color: #fff3cd; padding: 15px; margin: 20px 0; border-radius: 5px; }}
    </style>
</head>
<body>
    <div class="header">
        <h1>MultiOS Integration Test Report</h1>
        <p>Build: {} | Branch: {} | Commit: {}</p>
        <p>Generated: {}</p>
    </div>
    
    <div class="summary">
        <h2>Test Summary</h2>
        <p>Total Tests: {} | Passed: {} | Failed: {} | Skipped: {}</p>
        <p>Execution Time: {}ms</p>
        <p>Success Rate: {:.1}%</p>
    </div>
    
    <div class="metrics">
        <h2>Performance Metrics</h2>
        <p>Average Memory Usage: {}KB</p>
        <p>Average Throughput: {:.2} ops/sec</p>
        <p>Average Latency P95: {:.2}ms</p>
    </div>
    
    <h2>Test Results</h2>
    {}
</body>
</html>
        "#,
            report_data.build_info.build_number,
            report_data.build_info.branch_name,
            report_data.build_info.commit_sha,
            report_data.execution_timestamp,
            report_data.summary.total_tests,
            report_data.summary.passed,
            report_data.summary.failed,
            report_data.summary.skipped,
            report_data.summary.execution_time_ms,
            (report_data.summary.passed as f64 / report_data.summary.total_tests as f64) * 100.0,
            report_data.performance_metrics.avg_memory_usage_kb,
            report_data.performance_metrics.avg_throughput_ops_per_sec,
            report_data.performance_metrics.avg_latency_p95_ms,
            "" // Test results would be populated here
        );
        
        Ok(html)
    }

    /// Group tests by category
    fn group_tests_by_category(&self, test_results: &[IntegrationTestResult]) 
                              -> alloc::collections::BTreeMap<TestCategory, CategorySummary> {
        let mut categories = alloc::collections::BTreeMap::new();
        
        for result in test_results {
            let category_summary = categories.entry(result.category.clone())
                .or_insert(CategorySummary {
                    total: 0,
                    passed: 0,
                    failed: 0,
                    avg_execution_time: 0,
                });
            
            category_summary.total += 1;
            if result.passed {
                category_summary.passed += 1;
            } else {
                category_summary.failed += 1;
            }
        }
        
        categories
    }

    /// Aggregate performance metrics
    fn aggregate_performance_metrics(&self, test_results: &[IntegrationTestResult]) 
                                   -> AggregatedPerformanceMetrics {
        let performance_tests: Vec<_> = test_results.iter()
            .filter_map(|r| r.performance_metrics.as_ref())
            .collect();
        
        let count = performance_tests.len();
        if count == 0 {
            return AggregatedPerformanceMetrics {
                avg_memory_usage_kb: 0,
                avg_throughput_ops_per_sec: 0.0,
                avg_latency_p95_ms: 0.0,
            };
        }
        
        AggregatedPerformanceMetrics {
            avg_memory_usage_kb: performance_tests.iter()
                .map(|m| m.memory_usage_kb as f64)
                .sum::<f64>() / count as f64,
            avg_throughput_ops_per_sec: performance_tests.iter()
                .map(|m| m.throughput_ops_per_sec)
                .sum::<f64>() / count as f64,
            avg_latency_p95_ms: performance_tests.iter()
                .map(|m| m.latency_p95_ms)
                .sum::<f64>() / count as f64,
        }
    }

    /// Analyze errors
    fn analyze_errors(&self, test_results: &[IntegrationTestResult]) -> ErrorAnalysis {
        let failed_tests: Vec<_> = test_results.iter()
            .filter(|r| !r.passed)
            .collect();
        
        let mut error_patterns = alloc::collections::BTreeMap::new();
        for test in &failed_tests {
            if let Some(error) = &test.error_message {
                *error_patterns.entry(error.clone()).or_insert(0) += 1;
            }
        }
        
        ErrorAnalysis {
            total_failures: failed_tests.len(),
            unique_error_patterns: error_patterns.len(),
            most_common_errors: error_patterns.into_iter()
                .take(5)
                .collect(),
        }
    }

    /// Generate recommendations
    fn generate_recommendations(&self, test_results: &[IntegrationTestResult]) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        let failure_rate = test_results.iter().filter(|r| !r.passed).count() as f64 / test_results.len() as f64;
        
        if failure_rate > 0.1 {
            recommendations.push("High test failure rate detected. Review test stability and fix flaky tests.".to_string());
        }
        
        let slow_tests: Vec<_> = test_results.iter()
            .filter(|r| r.execution_time_ms > 10000)
            .collect();
        
        if !slow_tests.is_empty() {
            recommendations.push(format!("Found {} slow tests (>10s). Consider optimization.", slow_tests.len()));
        }
        
        recommendations.push("Review performance metrics for potential regressions.".to_string());
        recommendations.push("Ensure test environment is properly isolated and reproducible.".to_string());
        
        recommendations
    }

    /// Send webhook notification
    fn send_webhook(&self, webhook_url: &str, report_data: &TestReportData) -> Result<()> {
        info!("Sending webhook notification to: {}", webhook_url);
        // In a real implementation, this would send HTTP POST request
        Ok(())
    }

    /// Send automation event
    fn send_automation_event(&self, event: AutomationEvent) {
        match event {
            AutomationEvent::TestSuiteStarted(name) => {
                info!("[CI/CD] Test suite started: {}", name);
            }
            AutomationEvent::TestSuiteCompleted(result) => {
                info!("[CI/CD] Test suite completed: {}/{} passed", 
                      result.passed_tests, result.total_tests);
            }
            AutomationEvent::PerformanceRegression(test_name, regression) => {
                warn!("[CI/CD] Performance regression detected in {}: {:.2}%", test_name, regression);
            }
            AutomationEvent::EnvironmentIssue(issue) => {
                warn!("[CI/CD] Test environment issue: {}", issue);
            }
            AutomationEvent::BuildFailed(reason) => {
                error!("[CI/CD] Build failed: {}", reason);
            }
            AutomationEvent::BuildSuccessful(message) => {
                info!("[CI/CD] Build successful: {}", message);
            }
        }
    }

    /// Cleanup test environment
    fn cleanup_test_environment(&mut self) -> Result<()> {
        info!("Cleaning up test environment...");
        
        // Remove test directories
        let _ = crate::filesystem::delete_directory("/tmp/multios_test_environment");
        let _ = crate::filesystem::delete_directory("/tmp/multios_test_data");
        
        info!("Test environment cleanup completed");
        Ok(())
    }

    /// Get current monitoring data
    pub fn get_monitoring_data(&self) -> Result<MonitoringData> {
        let memory_stats = crate::memory::get_memory_stats();
        
        Ok(MonitoringData {
            timestamp: crate::hal::get_current_time_ms(),
            test_environment_status: EnvironmentStatus::Healthy,
            system_metrics: SystemMetrics {
                cpu_usage_percent: crate::hal::get_cpu_usage(),
                memory_usage_mb: (memory_stats.used_pages * 4096) / 1024 / 1024,
                disk_usage_percent: 50.0, // Mock data
                network_latency_ms: 1.0,  // Mock data
            },
            integration_metrics: IntegrationMetrics {
                component_interaction_latency_ms: 25.0,
                cross_component_success_rate: 95.0,
                system_integration_score: 90.0,
            },
        })
    }
}

/// Supporting data structures for reporting
#[derive(Debug, Clone)]
pub struct TestReportData {
    pub build_info: BuildInfo,
    pub execution_timestamp: u64,
    pub summary: TestSummary,
    pub test_categories: alloc::collections::BTreeMap<TestCategory, CategorySummary>,
    pub performance_metrics: AggregatedPerformanceMetrics,
    pub error_analysis: ErrorAnalysis,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct TestSummary {
    pub total_tests: usize,
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
    pub execution_time_ms: u64,
}

#[derive(Debug, Clone)]
pub struct CategorySummary {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub avg_execution_time: u64,
}

#[derive(Debug, Clone)]
pub struct AggregatedPerformanceMetrics {
    pub avg_memory_usage_kb: f64,
    pub avg_throughput_ops_per_sec: f64,
    pub avg_latency_p95_ms: f64,
}

#[derive(Debug, Clone)]
pub struct ErrorAnalysis {
    pub total_failures: usize,
    pub unique_error_patterns: usize,
    pub most_common_errors: Vec<(String, usize)>,
}

/// CI/CD pipeline integration functions

/// Run integration tests as part of CI/CD pipeline
pub fn run_ci_integration_tests(build_info: BuildInfo) -> Result<AutomatedTestResult> {
    let automation_config = AutomationConfig::default();
    let mut coordinator = TestAutomationCoordinator::new(automation_config);
    
    coordinator.run_automated_tests(build_info)
}

/// Quick integration test for development
pub fn run_quick_integration_tests() -> Result<Vec<IntegrationTestResult>> {
    let quick_config = IntegrationTestConfig {
        test_timeout_ms: 10_000,
        cleanup_enabled: true,
        parallel_tests: true,
        verbose_logging: true,
        performance_baselines: false,
        mock_hardware: true,
        test_environment: TestEnvironment::Emulated,
    };
    
    run_integration_test_suite(quick_config)
}

/// Full integration test suite for release builds
pub fn run_full_integration_tests(build_info: BuildInfo) -> Result<AutomatedTestResult> {
    let mut full_config = AutomationConfig::default();
    full_config.continuous_monitoring = true;
    full_config.automated_reporting = true;
    
    let mut coordinator = TestAutomationCoordinator::new(full_config);
    coordinator.run_automated_tests(build_info)
}
