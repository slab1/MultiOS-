//! Regression Detection Module
//!
//! Provides algorithms for detecting both performance and functional regressions
//! using statistical analysis, machine learning, and heuristic approaches.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use simple_statistics::standard_deviation;
use std::collections::HashMap;

use crate::{
    DetectedRegression, PerformanceBaseline, PerformanceMeasurement, RegressionSeverity, 
    RegressionType, TestResult, TestStatus, TestType, Uuid,
};

/// Performance regression detector
#[derive(Debug, Clone)]
pub struct PerformanceDetector {
    thresholds: crate::PerformanceThresholds,
}

impl PerformanceDetector {
    /// Create new performance detector
    pub fn new(thresholds: crate::PerformanceThresholds) -> Self {
        Self { thresholds }
    }

    /// Detect performance regressions from measurements
    pub async fn detect_regressions(
        &self,
        measurements: &[PerformanceMeasurement],
        baselines: &[PerformanceBaseline],
    ) -> Result<Vec<DetectedRegression>> {
        info!("Analyzing {} measurements for performance regressions", measurements.len());
        
        let mut regressions = Vec::new();
        
        // Group measurements by component and metric type
        let grouped_measurements = self.group_measurements_by_component_metric(measurements);
        
        for (component_metric, measurement_group) in grouped_measurements {
            if let Some(regressions_for_group) = self.analyze_measurements_for_regressions(
                &component_metric.0,
                &component_metric.1,
                &measurement_group,
                baselines,
            ).await? {
                regressions.extend(regressions_for_group);
            }
        }
        
        info!("Detected {} performance regressions", regressions.len());
        Ok(regressions)
    }

    /// Group measurements by component and metric type
    fn group_measurements_by_component_metric(
        &self,
        measurements: &[PerformanceMeasurement],
    ) -> HashMap<(String, String), Vec<&PerformanceMeasurement>> {
        let mut grouped = HashMap::new();
        
        for measurement in measurements {
            let key = (measurement.component.clone(), measurement.metric_type.clone());
            grouped.entry(key).or_insert_with(Vec::new).push(measurement);
        }
        
        grouped
    }

    /// Analyze measurement group for regressions
    async fn analyze_measurements_for_regressions(
        &self,
        component: &str,
        metric_type: &str,
        measurements: &[&PerformanceMeasurement],
        baselines: &[PerformanceBaseline],
    ) -> Result<Option<Vec<DetectedRegression>>> {
        // Find matching baseline
        let baseline = self.find_matching_baseline(baselines, component, metric_type)?;
        
        if let Some(baseline) = baseline {
            let recent_measurements = self.get_recent_measurements(measurements);
            
            if recent_measurements.len() >= self.thresholds.sample_size_minimum {
                let regressions = self.detect_regressions_vs_baseline(
                    component,
                    metric_type,
                    &recent_measurements,
                    &baseline,
                ).await?;
                
                Ok(Some(regressions))
            } else {
                debug!("Insufficient sample size for {}/{}: {} samples", component, metric_type, recent_measurements.len());
                Ok(None)
            }
        } else {
            debug!("No baseline found for {}/{}", component, metric_type);
            Ok(None)
        }
    }

    /// Find matching baseline for component and metric
    fn find_matching_baseline(
        &self,
        baselines: &[PerformanceBaseline],
        component: &str,
        metric_type: &str,
    ) -> Result<Option<PerformanceBaseline>> {
        // Look for exact match first
        for baseline in baselines {
            if baseline.component == component && baseline.metric_type == metric_type && baseline.is_active {
                return Ok(Some(baseline.clone()));
            }
        }
        
        // If no exact match, look for component-specific baseline
        for baseline in baselines {
            if baseline.component == component && baseline.is_active {
                warn!("Using component baseline for different metric: {} vs {}", baseline.metric_type, metric_type);
                return Ok(Some(baseline.clone()));
            }
        }
        
        Ok(None)
    }

    /// Get recent measurements (last 24 hours)
    fn get_recent_measurements<'a>(
        &self,
        measurements: &'a [&'a PerformanceMeasurement],
    ) -> Vec<&'a PerformanceMeasurement> {
        let cutoff_time = Utc::now() - chrono::Duration::hours(24);
        
        measurements
            .iter()
            .filter(|m| m.timestamp >= cutoff_time)
            .copied()
            .collect()
    }

    /// Detect regressions against baseline
    async fn detect_regressions_vs_baseline(
        &self,
        component: &str,
        metric_type: &str,
        recent_measurements: &[&PerformanceMeasurement],
        baseline: &PerformanceBaseline,
    ) -> Result<Vec<DetectedRegression>> {
        let mut regressions = Vec::new();
        
        // Calculate statistics for recent measurements
        let recent_values: Vec<f64> = recent_measurements.iter().map(|m| m.value).collect();
        let mean_recent = recent_values.iter().sum::<f64>() / recent_values.len() as f64;
        let stddev_recent = if recent_values.len() > 1 {
            standard_deviation(&recent_values) as f64
        } else {
            0.0
        };
        
        // Determine regression type based on metric type
        let regression_type = self.get_regression_type_for_metric(metric_type);
        
        // Check for different types of regressions
        let latency_regressions = self.detect_latency_regression(
            component,
            metric_type,
            mean_recent,
            baseline.baseline_value,
            &recent_values,
        )?;
        
        let throughput_regressions = self.detect_throughput_regression(
            component,
            metric_type,
            mean_recent,
            baseline.baseline_value,
            &recent_values,
        )?;
        
        let resource_regressions = self.detect_resource_regression(
            component,
            metric_type,
            mean_recent,
            baseline.baseline_value,
            &recent_values,
        )?;
        
        regressions.extend(latency_regressions);
        regressions.extend(throughput_regressions);
        regressions.extend(resource_regressions);
        
        Ok(regressions)
    }

    /// Get regression type for metric
    fn get_regression_type_for_metric(&self, metric_type: &str) -> RegressionType {
        match metric_type.to_lowercase().as_str() {
            "latency" | "response_time" | "execution_time" => RegressionType::PerformanceLatency,
            "throughput" | "requests_per_second" | "ops_per_sec" => RegressionType::PerformanceThroughput,
            "memory_usage" | "memory" | "ram" => RegressionType::PerformanceMemory,
            "cpu_usage" | "cpu" | "processor" => RegressionType::PerformanceCpu,
            "memory_leak" => RegressionType::MemoryLeak,
            "resource_exhaustion" => RegressionType::ResourceExhaustion,
            _ => RegressionType::PerformanceLatency, // Default
        }
    }

    /// Detect latency regressions
    fn detect_latency_regression(
        &self,
        component: &str,
        metric_type: &str,
        recent_mean: f64,
        baseline_value: f64,
        recent_values: &[f64],
    ) -> Result<Vec<DetectedRegression>> {
        let mut regressions = Vec::new();
        
        // Calculate regression percentage
        if baseline_value > 0.0 {
            let regression_percentage = ((recent_mean - baseline_value) / baseline_value) * 100.0;
            
            // Check if regression exceeds threshold
            if regression_percentage > self.thresholds.latency_regression_pct {
                let severity = if regression_percentage > self.thresholds.latency_regression_pct * 2.0 {
                    RegressionSeverity::Critical
                } else if regression_percentage > self.thresholds.latency_regression_pct * 1.5 {
                    RegressionSeverity::Major
                } else {
                    RegressionSeverity::Minor
                };
                
                // Calculate confidence score based on sample consistency
                let confidence_score = self.calculate_confidence_score(recent_values, baseline_value);
                
                if confidence_score >= self.thresholds.confidence_threshold {
                    regressions.push(DetectedRegression {
                        id: Uuid::new_v4(),
                        regression_type: RegressionType::PerformanceLatency,
                        severity,
                        component: component.to_string(),
                        test_name: format!("{}_latency_test", component),
                        current_value: recent_mean,
                        baseline_value,
                        regression_percentage,
                        detection_algorithm: "statistical_analysis".to_string(),
                        confidence_score,
                        test_run_id: Uuid::new_v4().to_string(),
                        timestamp: Utc::now(),
                        metadata: HashMap::new(),
                    });
                }
            }
        }
        
        Ok(regressions)
    }

    /// Detect throughput regressions
    fn detect_throughput_regression(
        &self,
        component: &str,
        metric_type: &str,
        recent_mean: f64,
        baseline_value: f64,
        recent_values: &[f64],
    ) -> Result<Vec<DetectedRegression>> {
        let mut regressions = Vec::new();
        
        // For throughput, we expect higher is better, so a regression is when it decreases
        if baseline_value > 0.0 {
            let regression_percentage = ((baseline_value - recent_mean) / baseline_value) * 100.0;
            
            if regression_percentage > self.thresholds.throughput_regression_pct {
                let severity = if regression_percentage > self.thresholds.throughput_regression_pct * 2.0 {
                    RegressionSeverity::Critical
                } else if regression_percentage > self.thresholds.throughput_regression_pct * 1.5 {
                    RegressionSeverity::Major
                } else {
                    RegressionSeverity::Minor
                };
                
                let confidence_score = self.calculate_confidence_score(recent_values, baseline_value);
                
                if confidence_score >= self.thresholds.confidence_threshold {
                    regressions.push(DetectedRegression {
                        id: Uuid::new_v4(),
                        regression_type: RegressionType::PerformanceThroughput,
                        severity,
                        component: component.to_string(),
                        test_name: format!("{}_throughput_test", component),
                        current_value: recent_mean,
                        baseline_value,
                        regression_percentage,
                        detection_algorithm: "statistical_analysis".to_string(),
                        confidence_score,
                        test_run_id: Uuid::new_v4().to_string(),
                        timestamp: Utc::now(),
                        metadata: HashMap::new(),
                    });
                }
            }
        }
        
        Ok(regressions)
    }

    /// Detect resource usage regressions
    fn detect_resource_regression(
        &self,
        component: &str,
        metric_type: &str,
        recent_mean: f64,
        baseline_value: f64,
        recent_values: &[f64],
    ) -> Result<Vec<DetectedRegression>> {
        let mut regressions = Vec::new();
        
        if baseline_value > 0.0 {
            let regression_percentage = ((recent_mean - baseline_value) / baseline_value) * 100.0;
            
            // Determine threshold based on metric type
            let threshold = match metric_type.to_lowercase().as_str() {
                "memory_usage" | "memory" => self.thresholds.memory_regression_pct,
                "cpu_usage" | "cpu" => self.thresholds.cpu_regression_pct,
                _ => self.thresholds.cpu_regression_pct, // Default
            };
            
            if regression_percentage > threshold {
                let severity = if regression_percentage > threshold * 2.0 {
                    RegressionSeverity::Critical
                } else if regression_percentage > threshold * 1.5 {
                    RegressionSeverity::Major
                } else {
                    RegressionSeverity::Minor
                };
                
                let confidence_score = self.calculate_confidence_score(recent_values, baseline_value);
                
                if confidence_score >= self.thresholds.confidence_threshold {
                    let regression_type = match metric_type.to_lowercase().as_str() {
                        "memory_usage" | "memory" => RegressionType::PerformanceMemory,
                        "cpu_usage" | "cpu" => RegressionType::PerformanceCpu,
                        _ => RegressionType::PerformanceCpu,
                    };
                    
                    regressions.push(DetectedRegression {
                        id: Uuid::new_v4(),
                        regression_type,
                        severity,
                        component: component.to_string(),
                        test_name: format!("{}_{}_test", component, metric_type),
                        current_value: recent_mean,
                        baseline_value,
                        regression_percentage,
                        detection_algorithm: "statistical_analysis".to_string(),
                        confidence_score,
                        test_run_id: Uuid::new_v4().to_string(),
                        timestamp: Utc::now(),
                        metadata: HashMap::new(),
                    });
                }
            }
        }
        
        Ok(regressions)
    }

    /// Calculate confidence score based on measurement consistency
    fn calculate_confidence_score(&self, recent_values: &[f64], baseline_value: f64) -> f64 {
        if recent_values.len() <= 1 {
            return 0.0;
        }
        
        let mean = recent_values.iter().sum::<f64>() / recent_values.len() as f64;
        let variance = recent_values
            .iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / recent_values.len() as f64;
        let stddev = variance.sqrt();
        
        // Calculate coefficient of variation (normalized standard deviation)
        let coefficient_of_variation = if mean != 0.0 {
            stddev / mean
        } else {
            1.0
        };
        
        // Higher confidence for lower coefficient of variation (more consistent measurements)
        let consistency_score = 1.0 / (1.0 + coefficient_of_variation);
        
        // Also consider how far from baseline (more deviation = lower confidence)
        let deviation_factor = if baseline_value != 0.0 {
            ((mean - baseline_value).abs() / baseline_value).min(2.0) / 2.0
        } else {
            1.0
        };
        
        (consistency_score * (1.0 - deviation_factor)) * 100.0
    }

    /// Detect outliers using statistical methods
    pub fn detect_outliers(&self, values: &[f64]) -> Vec<usize> {
        if values.len() < 3 {
            return Vec::new();
        }
        
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance = values
            .iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / values.len() as f64;
        let stddev = variance.sqrt();
        
        if stddev == 0.0 {
            return Vec::new();
        }
        
        let lower_bound = mean - self.thresholds.outlier_detection_sigma * stddev;
        let upper_bound = mean + self.thresholds.outlier_detection_sigma * stddev;
        
        values
            .iter()
            .enumerate()
            .filter(|(_, &value)| value < lower_bound || value > upper_bound)
            .map(|(index, _)| index)
            .collect()
    }

    /// Analyze trend direction using linear regression
    pub fn analyze_trend(&self, values: &[f64]) -> (f64, f64) {
        if values.len() < 2 {
            return (0.0, 0.0); // No trend
        }
        
        // Simple linear regression: y = mx + b
        let n = values.len() as f64;
        let sum_x = (0..values.len()).sum::<usize>() as f64;
        let sum_y = values.iter().sum::<f64>();
        let sum_xy = values
            .iter()
            .enumerate()
            .map(|(i, &y)| i as f64 * y)
            .sum::<f64>();
        let sum_x2 = (0..values.len())
            .map(|i| (i as f64).powi(2))
            .sum::<f64>();
        
        let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x * sum_x);
        let intercept = (sum_y - slope * sum_x) / n;
        
        (slope, intercept)
    }
}

/// Functional regression detector
#[derive(Debug, Clone)]
pub struct FunctionalDetector {
    // Configuration for functional testing
}

impl FunctionalDetector {
    /// Create new functional detector
    pub fn new() -> Self {
        Self {}
    }

    /// Run functional tests
    pub async fn run_functional_tests(
        &self,
        config: &crate::TestSuiteConfig,
    ) -> Result<Vec<TestResult>> {
        info!("Running functional tests");
        
        let mut results = Vec::new();
        
        // This is a placeholder - in real implementation, this would:
        // 1. Discover and load functional test definitions
        // 2. Execute tests in appropriate test framework
        // 3. Collect results and metadata
        
        // For now, return mock results
        for test_suite in &config.functional_test_suites {
            let mock_result = self.run_mock_functional_test(test_suite).await?;
            results.push(mock_result);
        }
        
        Ok(results)
    }

    /// Run single functional test
    pub async fn run_single_test(&self, test_config: &crate::TestSuiteConfig) -> Result<TestResult> {
        // Mock implementation - would run actual test
        self.run_mock_functional_test("single_test").await
    }

    /// Mock functional test execution
    async fn run_mock_functional_test(&self, test_name: &str) -> Result<TestResult> {
        // Simulate test execution
        let execution_time = std::time::Duration::from_millis(100 + rand::random::<u64>() % 1000);
        tokio::time::sleep(execution_time).await;
        
        // Randomly determine test outcome (80% pass rate)
        let pass_rate = 0.8;
        let is_passed = rand::random::<f64>() < pass_rate;
        
        let status = if is_passed {
            TestStatus::Passed
        } else {
            TestStatus::Failed
        };
        
        Ok(TestResult {
            id: Uuid::new_v4(),
            test_name: test_name.to_string(),
            component: "mock_component".to_string(),
            test_type: TestType::Functional,
            status,
            execution_time_ms: execution_time.as_millis() as u64,
            timestamp: Utc::now(),
            environment: crate::TestEnvironment {
                name: "mock_env".to_string(),
                hardware_config: HashMap::new(),
                software_config: HashMap::new(),
                environment_hash: "mock_hash".to_string(),
            },
            metrics: HashMap::new(),
            metadata: HashMap::new(),
        })
    }

    /// Detect functional regressions from test results
    pub async fn detect_functional_regressions(
        &self,
        current_results: &[TestResult],
        historical_results: &[TestResult],
    ) -> Result<Vec<DetectedRegression>> {
        let mut regressions = Vec::new();
        
        // Group results by test name
        let current_grouped = self.group_test_results(current_results);
        let historical_grouped = self.group_test_results(historical_results);
        
        for (test_name, current_group) in current_grouped {
            if let Some(historical_group) = historical_grouped.get(&test_name) {
                if let Some(regression) = self.compare_test_results(
                    test_name,
                    current_group,
                    historical_group,
                )? {
                    regressions.push(regression);
                }
            }
        }
        
        Ok(regressions)
    }

    /// Group test results by test name
    fn group_test_results(&self, results: &[TestResult]) -> HashMap<String, Vec<&TestResult>> {
        let mut grouped = HashMap::new();
        
        for result in results {
            grouped
                .entry(result.test_name.clone())
                .or_insert_with(Vec::new)
                .push(result);
        }
        
        grouped
    }

    /// Compare current vs historical test results
    fn compare_test_results(
        &self,
        test_name: String,
        current_results: &[&TestResult],
        historical_results: &[&TestResult],
    ) -> Result<Option<DetectedRegression>> {
        // Calculate success rates
        let current_pass_rate = self.calculate_pass_rate(current_results);
        let historical_pass_rate = self.calculate_pass_rate(historical_results);
        
        // Detect significant regression in success rate
        if current_pass_rate < historical_pass_rate {
            let regression_percentage = ((historical_pass_rate - current_pass_rate) / historical_pass_rate) * 100.0;
            
            // Threshold for functional regression detection
            if regression_percentage > 5.0 { // 5% regression threshold
                let severity = if regression_percentage > 20.0 {
                    RegressionSeverity::Critical
                } else if regression_percentage > 10.0 {
                    RegressionSeverity::Major
                } else {
                    RegressionSeverity::Minor
                };
                
                return Ok(Some(DetectedRegression {
                    id: Uuid::new_v4(),
                    regression_type: RegressionType::Functional,
                    severity,
                    component: current_results[0].component.clone(),
                    test_name,
                    current_value: current_pass_rate,
                    baseline_value: historical_pass_rate,
                    regression_percentage,
                    detection_algorithm: "functional_success_rate_comparison".to_string(),
                    confidence_score: 90.0, // High confidence for functional regressions
                    test_run_id: Uuid::new_v4().to_string(),
                    timestamp: Utc::now(),
                    metadata: HashMap::new(),
                }));
            }
        }
        
        Ok(None)
    }

    /// Calculate pass rate from test results
    fn calculate_pass_rate(&self, results: &[&TestResult]) -> f64 {
        let total = results.len() as f64;
        if total == 0.0 {
            return 0.0;
        }
        
        let passed = results
            .iter()
            .filter(|r| r.status == TestStatus::Passed)
            .count() as f64;
        
        (passed / total) * 100.0
    }
}