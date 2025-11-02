//! Historical Trending and Analytics Module
//!
//! Provides comprehensive historical trend analysis, predictive analytics,
//! and reporting capabilities for regression testing data over time.

use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Utc};
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use simple_statistics::linear_regression;
use std::collections::{HashMap, VecDeque};

use crate::{
    database::DatabaseManager, DetectedRegression, PerformanceMeasurement, TestResult,
    TrendData, TrendDirection, TrendPrediction, TrendStatistics, Uuid,
};

/// Trend analyzer for historical data
#[derive(Debug, Clone)]
pub struct TrendAnalyzer {
    /// Configuration for trend analysis
    pub config: TrendConfig,
    /// Cache for recent trend data to improve performance
    cache: TrendCache,
}

/// Configuration for trend analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendConfig {
    pub analysis_window_days: u32,
    pub prediction_horizon_days: u32,
    pub trend_sensitivity: f64,
    pub seasonal_analysis: bool,
    pub correlation_threshold: f64,
    pub anomaly_detection_window: usize,
}

/// Trend data cache for performance optimization
#[derive(Debug, Default)]
struct TrendCache {
    /// Recent trend data cached for quick access
    recent_trends: HashMap<String, VecDeque<TrendDataPoint>>,
    /// Cache expiration time
    last_update: DateTime<Utc>,
    /// Cache TTL in minutes
    ttl_minutes: u32,
}

impl TrendCache {
    fn new() -> Self {
        Self {
            recent_trends: HashMap::new(),
            last_update: Utc::now(),
            ttl_minutes: 15, // Cache expires after 15 minutes
        }
    }

    /// Check if cache is still valid
    fn is_valid(&self) -> bool {
        (Utc::now() - self.last_update).num_minutes() < self.ttl_minutes as i64
    }

    /// Update cache timestamp
    fn update_timestamp(&mut self) {
        self.last_update = Utc::now();
    }

    /// Get cached trend data for component and metric
    fn get_cached_data(&self, key: &str) -> Option<&VecDeque<TrendDataPoint>> {
        self.recent_trends.get(key)
    }

    /// Cache trend data for component and metric
    fn cache_data(&mut self, key: String, data: VecDeque<TrendDataPoint>) {
        self.recent_trends.insert(key, data);
        self.update_timestamp();
    }

    /// Clear expired cache
    fn clear_expired(&mut self) {
        if !self.is_valid() {
            self.recent_trends.clear();
        }
    }
}

/// Individual trend data point
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TrendDataPoint {
    timestamp: DateTime<Utc>,
    value: f64,
    source: String, // test_run_id, measurement_id, etc.
    metadata: HashMap<String, serde_json::Value>,
}

impl TrendAnalyzer {
    /// Create new trend analyzer
    pub fn new() -> Self {
        Self {
            config: TrendConfig {
                analysis_window_days: 30,
                prediction_horizon_days: 7,
                trend_sensitivity: 0.1,
                seasonal_analysis: true,
                correlation_threshold: 0.7,
                anomaly_detection_window: 24,
            },
            cache: TrendCache::new(),
        }
    }

    /// Create trend analyzer with custom configuration
    pub fn with_config(config: TrendConfig) -> Self {
        Self {
            config,
            cache: TrendCache::new(),
        }
    }

    /// Analyze component trends over time
    pub async fn analyze_component_trends(
        &self,
        db: &DatabaseManager,
        component: &str,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<crate::TrendAnalysisResult> {
        info!("Analyzing trends for component: {} from {} to {}", component, start_time, end_time);
        
        // Get performance measurements for the component
        let measurements = db.get_performance_measurements(
            component,
            "", // Get all metric types - we'll filter in the analysis
            start_time,
            end_time,
        ).await?;

        // Get functional test results
        let test_results = db.get_test_results(
            component,
            start_time,
            end_time,
        ).await?;

        // Get detected regressions
        let regressions = db.get_unresolved_regressions().await?;

        // Analyze trends by metric type
        let trend_data = self.analyze_measurement_trends(&measurements).await?;
        
        // Analyze functional test trends
        let functional_trends = self.analyze_functional_trends(&test_results).await?;

        // Combine trend data
        let mut all_trend_data = trend_data;
        all_trend_data.extend(functional_trends);

        // Generate summary
        let summary = self.generate_trend_summary(&all_trend_data, &regressions);

        // Generate recommendations
        let recommendations = self.generate_trend_recommendations(&all_trend_data, &regressions);

        Ok(crate::TrendAnalysisResult {
            component: component.to_string(),
            analysis_period: (start_time, end_time),
            trend_data: all_trend_data,
            summary,
            recommendations,
        })
    }

    /// Analyze trends in performance measurements
    async fn analyze_measurement_trends(
        &self,
        measurements: &[PerformanceMeasurement],
    ) -> Result<HashMap<String, TrendData>> {
        debug!("Analyzing trends for {} measurements", measurements.len());
        
        let mut trend_data = HashMap::new();
        
        // Group measurements by metric type
        let grouped_measurements = self.group_measurements_by_metric(measurements);
        
        for (metric_name, metric_measurements) in grouped_measurements {
            if metric_measurements.len() >= self.config.analysis_window_days as usize {
                let trend = self.create_trend_data(&metric_name, &metric_measurements).await?;
                trend_data.insert(metric_name, trend);
            } else {
                debug!("Insufficient data for trend analysis of metric: {} ({} points)", 
                      metric_name, metric_measurements.len());
            }
        }
        
        Ok(trend_data)
    }

    /// Analyze trends in functional test results
    async fn analyze_functional_trends(
        &self,
        test_results: &[TestResult],
    ) -> Result<HashMap<String, TrendData>> {
        debug!("Analyzing functional test trends for {} results", test_results.length());
        
        let mut trend_data = HashMap::new();
        
        // Group by test name
        let grouped_results = self.group_test_results_by_name(test_results);
        
        for (test_name, test_group) in grouped_results {
            if test_group.len() >= self.config.analysis_window_days as usize {
                // Create trend data based on pass rates over time
                let trend = self.create_functional_trend_data(&test_name, &test_group).await?;
                trend_data.insert(format!("functional_{}", test_name), trend);
            }
        }
        
        Ok(trend_data)
    }

    /// Create trend data from measurements
    async fn create_trend_data(
        &self,
        metric_name: &str,
        measurements: &[&PerformanceMeasurement],
    ) -> Result<TrendData> {
        debug!("Creating trend data for metric: {} with {} measurements", 
               metric_name, measurements.len());
        
        // Create time series data
        let mut time_series = Vec::new();
        for measurement in measurements {
            time_series.push((measurement.timestamp, measurement.value));
        }
        
        // Sort by timestamp
        time_series.sort_by(|a, b| a.0.cmp(&b.0));
        
        // Calculate statistics
        let values: Vec<f64> = time_series.iter().map(|(_, value)| *value).collect();
        let statistics = self.calculate_trend_statistics(&values)?;
        
        // Determine trend direction
        let trend_direction = self.determine_trend_direction(&values);
        
        // Generate predictions
        let predictions = self.generate_trend_predictions(&time_series)?;
        
        Ok(TrendData {
            metric_name: metric_name.to_string(),
            component: measurements[0].component.clone(),
            time_series,
            statistics,
            trend_direction,
            predictions,
        })
    }

    /// Create trend data from functional test results
    async fn create_functional_trend_data(
        &self,
        test_name: &str,
        test_results: &[&TestResult],
    ) -> Result<TrendData> {
        debug!("Creating functional trend data for test: {} with {} results", 
               test_name, test_results.len());
        
        // Create time series based on pass rates (daily aggregation)
        let mut daily_pass_rates = self.calculate_daily_pass_rates(test_results);
        
        let mut time_series = Vec::new();
        for (date, pass_rate) in daily_pass_rates {
            time_series.push((date, pass_rate));
        }
        
        // Sort by timestamp
        time_series.sort_by(|a, b| a.0.cmp(&b.0));
        
        // Calculate statistics
        let values: Vec<f64> = time_series.iter().map(|(_, value)| *value).collect();
        let statistics = self.calculate_trend_statistics(&values)?;
        
        // Determine trend direction
        let trend_direction = self.determine_trend_direction(&values);
        
        // Generate predictions
        let predictions = self.generate_trend_predictions(&time_series)?;
        
        Ok(TrendData {
            metric_name: format!("pass_rate_{}", test_name),
            component: test_results[0].component.clone(),
            time_series,
            statistics,
            trend_direction,
            predictions,
        })
    }

    /// Calculate daily pass rates from test results
    fn calculate_daily_pass_rates(&self, test_results: &[&TestResult]) -> HashMap<DateTime<Utc>, f64> {
        let mut daily_results = HashMap::new();
        
        // Group results by day
        for test_result in test_results {
            let date_key = test_result.timestamp.date_naive().and_hms_opt(0, 0, 0)
                .unwrap_or(test_result.timestamp.date_naive().and_hms(0, 0, 0, 0).unwrap())
                .with_timezone(&Utc);
            
            let daily_stats = daily_results.entry(date_key).or_insert((0, 0)); // (passed, total)
            daily_stats.1 += 1;
            
            if matches!(test_result.status, crate::TestStatus::Passed) {
                daily_stats.0 += 1;
            }
        }
        
        // Calculate pass rates
        daily_results
            .into_iter()
            .map(|(date, (passed, total))| {
                let pass_rate = if total > 0 { (passed as f64 / total as f64) * 100.0 } else { 0.0 };
                (date, pass_rate)
            })
            .collect()
    }

    /// Calculate trend statistics
    fn calculate_trend_statistics(&self, values: &[f64]) -> Result<TrendStatistics> {
        if values.is_empty() {
            return Err(anyhow::anyhow!("No values for statistical analysis"));
        }
        
        // Use simple_statistics crate functions
        let mean_val = simple_statistics::mean(values).unwrap_or(0.0);
        let stddev = simple_statistics::standard_deviation(values).unwrap_or(0.0);
        let median_val = simple_statistics::median(values).unwrap_or(0.0);
        let p95 = simple_statistics::percentile(values, 95.0).unwrap_or(0.0);
        let p99 = simple_statistics::percentile(values, 99.0).unwrap_or(0.0);
        let min_val = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max_val = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        
        Ok(TrendStatistics {
            mean: mean_val,
            standard_deviation: stddev,
            median: median_val,
            percentile_95: p95,
            percentile_99: p99,
            min_value: min_val,
            max_value: max_val,
        })
    }

    /// Determine trend direction using linear regression
    fn determine_trend_direction(&self, values: &[f64]) -> TrendDirection {
        if values.len() < 3 {
            return TrendDirection::Unknown;
        }
        
        // Use simple linear regression
        let x_values: Vec<f64> = (0..values.len()).map(|i| i as f64).collect();
        
        let regression = linear_regression(&x_values, values).unwrap_or((0.0, 0.0));
        let slope = regression.0;
        
        // Determine direction based on slope
        match slope.abs() {
            s if s < self.config.trend_sensitivity => TrendDirection::Stable,
            s if s > 0.0 => {
                // For most metrics, increasing is bad (degrading)
                if self.is_higher_worse() {
                    TrendDirection::Degrading
                } else {
                    TrendDirection::Improving
                }
            },
            _ => {
                if self.is_higher_worse() {
                    TrendDirection::Improving
                } else {
                    TrendDirection::Degrading
                }
            }
        }
    }

    /// Determine if higher values are worse (e.g., latency, memory usage)
    fn is_higher_worse(&self) -> bool {
        true // Default assumption - most performance metrics get worse with higher values
    }

    /// Generate trend predictions
    fn generate_trend_predictions(&self, time_series: &[(DateTime<Utc>, f64)]) -> Result<Vec<TrendPrediction>> {
        if time_series.len() < 3 {
            return Ok(Vec::new());
        }
        
        let values: Vec<f64> = time_series.iter().map(|(_, v)| *v).collect();
        let timestamps: Vec<f64> = time_series.iter().map(|(t, _)| t.timestamp() as f64).collect();
        
        // Use simple linear regression for prediction
        let x_values: Vec<f64> = (0..time_series.len()).map(|i| i as f64).collect();
        let regression = linear_regression(&x_values, &values).unwrap_or((0.0, values[0]));
        
        let mut predictions = Vec::new();
        let last_timestamp = time_series.last().unwrap().0.timestamp() as f64;
        let time_step = self.estimate_time_step(time_series);
        
        // Generate predictions for the specified horizon
        for i in 1..=self.config.prediction_horizon_days {
            let future_timestamp = last_timestamp + (time_step * i as f64);
            let predicted_value = regression.0 * (time_series.len() as f64 + i as f64) + regression.1;
            
            predictions.push(TrendPrediction {
                timestamp: DateTime::from_timestamp(future_timestamp as i64, 0)
                    .unwrap_or(Utc::now()),
                predicted_value,
                confidence_interval: (predicted_value * 0.8, predicted_value * 1.2),
                confidence_level: 0.85,
            });
        }
        
        Ok(predictions)
    }

    /// Estimate time step from time series
    fn estimate_time_step(&self, time_series: &[(DateTime<Utc>, f64)]) -> f64 {
        if time_series.len() < 2 {
            return 86400.0; // Default to 1 day in seconds
        }
        
        let mut time_diffs = Vec::new();
        for i in 1..time_series.len() {
            time_diffs.push(
                (time_series[i].0 - time_series[i-1].0).num_seconds() as f64
            );
        }
        
        let avg_diff: f64 = time_diffs.iter().sum::<f64>() / time_diffs.len() as f64;
        if avg_diff.is_finite() && avg_diff > 0.0 {
            avg_diff
        } else {
            86400.0 // Default to 1 day
        }
    }

    /// Analyze correlation between different metrics
    pub async fn analyze_metric_correlations(
        &self,
        db: &DatabaseManager,
        component: &str,
        metric_names: &[String],
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<HashMap<String, CorrelationAnalysis>> {
        info!("Analyzing correlations between {} metrics for component: {}", 
              metric_names.len(), component);
        
        // Get measurements for all metrics
        let mut metric_data = HashMap::new();
        for metric_name in metric_names {
            let measurements = db.get_performance_measurements(
                component,
                metric_name,
                start_time,
                end_time,
            ).await?;
            
            if !measurements.is_empty() {
                metric_data.insert(metric_name.clone(), measurements);
            }
        }
        
        // Calculate correlations
        let mut correlations = HashMap::new();
        let metric_names_vec = metric_data.keys().collect::<Vec<_>>();
        
        for i in 0..metric_names_vec.len() {
            for j in i+1..metric_names_vec.len() {
                let name1 = metric_names_vec[i];
                let name2 = metric_names_vec[j];
                
                let correlation = self.calculate_correlation(
                    &metric_data[name1],
                    &metric_data[name2],
                )?;
                
                if correlation.strength.abs() >= self.config.correlation_threshold {
                    let correlation_key = format!("{}_vs_{}", name1, name2);
                    correlations.insert(correlation_key, correlation);
                }
            }
        }
        
        Ok(correlations)
    }

    /// Calculate correlation between two measurement series
    fn calculate_correlation(
        &self,
        measurements1: &[PerformanceMeasurement],
        measurements2: &[PerformanceMeasurement],
    ) -> Result<CorrelationAnalysis> {
        // Align measurements by timestamp (simplified - in practice, use interpolation)
        let mut aligned_pairs = Vec::new();
        
        let mut i = 0;
        let mut j = 0;
        
        while i < measurements1.len() && j < measurements2.len() {
            let time1 = measurements1[i].timestamp;
            let time2 = measurements2[j].timestamp;
            
            match time1.cmp(&time2) {
                std::cmp::Ordering::Equal => {
                    aligned_pairs.push((measurements1[i].value, measurements2[j].value));
                    i += 1;
                    j += 1;
                }
                std::cmp::Ordering::Less => {
                    i += 1;
                }
                std::cmp::Ordering::Greater => {
                    j += 1;
                }
            }
        }
        
        if aligned_pairs.len() < 3 {
            return Ok(CorrelationAnalysis {
                metric1: measurements1[0].metric_type.clone(),
                metric2: measurements2[0].metric_type.clone(),
                correlation_coefficient: 0.0,
                strength: 0.0,
                significance: "insignificant".to_string(),
                data_points: aligned_pairs.len(),
            });
        }
        
        let values1: Vec<f64> = aligned_pairs.iter().map(|(v1, _)| *v1).collect();
        let values2: Vec<f64> = aligned_pairs.iter().map(|(_, v2)| *v2).collect();
        
        let correlation_coefficient = self.calculate_pearson_correlation(&values1, &values2);
        let strength = correlation_coefficient.abs();
        
        let significance = if aligned_pairs.len() >= 10 && strength >= 0.3 {
            "significant".to_string()
        } else {
            "insignificant".to_string()
        };
        
        Ok(CorrelationAnalysis {
            metric1: measurements1[0].metric_type.clone(),
            metric2: measurements2[0].metric_type.clone(),
            correlation_coefficient,
            strength,
            significance,
            data_points: aligned_pairs.len(),
        })
    }

    /// Calculate Pearson correlation coefficient
    fn calculate_pearson_correlation(&self, x: &[f64], y: &[f64]) -> f64 {
        if x.len() != y.len() || x.is_empty() {
            return 0.0;
        }
        
        let n = x.len() as f64;
        let sum_x = x.iter().sum::<f64>();
        let sum_y = y.iter().sum::<f64>();
        let sum_xy = x.iter().zip(y.iter()).map(|(xi, yi)| xi * yi).sum::<f64>();
        let sum_x2 = x.iter().map(|xi| xi * xi).sum::<f64>();
        let sum_y2 = y.iter().map(|yi| yi * yi).sum::<f64>();
        
        let numerator = n * sum_xy - sum_x * sum_y;
        let denominator = ((n * sum_x2 - sum_x * sum_x) * (n * sum_y2 - sum_y * sum_y)).sqrt();
        
        if denominator == 0.0 {
            0.0
        } else {
            numerator / denominator
        }
    }

    /// Generate trend summary
    fn generate_trend_summary(
        &self,
        trend_data: &HashMap<String, TrendData>,
        regressions: &[DetectedRegression],
    ) -> crate::TrendSummary {
        let mut improving_trends = 0;
        let mut degrading_trends = 0;
        let mut stable_trends = 0;
        let mut unknown_trends = 0;
        
        for trend in trend_data.values() {
            match trend.trend_direction {
                TrendDirection::Improving => improving_trends += 1,
                TrendDirection::Degrading => degrading_trends += 1,
                TrendDirection::Stable => stable_trends += 1,
                TrendDirection::Unknown => unknown_trends += 1,
            }
        }
        
        let total_regressions = regressions.len();
        let avg_severity = if !regressions.is_empty() {
            regressions.iter()
                .map(|r| match r.severity {
                    crate::RegressionSeverity::Minor => 1.0,
                    crate::RegressionSeverity::Major => 2.0,
                    crate::RegressionSeverity::Critical => 3.0,
                    crate::RegressionSeverity::Blocker => 4.0,
                })
                .sum::<f64>() / regressions.len() as f64
        } else {
            0.0
        };
        
        let component_regression_counts: HashMap<String, usize> = regressions
            .iter()
            .map(|r| (r.component.clone(), 1))
            .fold(HashMap::new(), |mut acc, (comp, count)| {
                *acc.entry(comp).or_insert(0) += count;
                acc
            });
        
        let most_affected_components: Vec<String> = component_regression_counts
            .into_iter()
            .sorted_by(|a, b| b.1.cmp(&a.1))
            .take(5)
            .map(|(comp, _)| comp)
            .collect();
        
        crate::TrendSummary {
            total_regressions,
            improving_trends,
            degrading_trends,
            stable_trends,
            avg_regression_severity: avg_severity,
            most_affected_components,
        }
    }

    /// Generate recommendations based on trend analysis
    fn generate_trend_recommendations(
        &self,
        trend_data: &HashMap<String, TrendData>,
        regressions: &[DetectedRegression],
    ) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        // Analyze degrading trends
        let degrading_metrics: Vec<&String> = trend_data
            .iter()
            .filter(|(_, trend)| trend.trend_direction == TrendDirection::Degrading)
            .map(|(name, _)| name)
            .collect();
        
        if !degrading_metrics.is_empty() {
            recommendations.push(format!(
                "Critical: {} metrics showing degrading trends: {}. Immediate investigation required.",
                degrading_metrics.len(),
                degrading_metrics.join(", ")
            ));
        }
        
        // Analyze high-variance metrics
        let high_variance_metrics: Vec<&String> = trend_data
            .iter()
            .filter(|(_, trend)| trend.statistics.standard_deviation > trend.statistics.mean * 0.5)
            .map(|(name, _)| name)
            .collect();
        
        if !high_variance_metrics.is_empty() {
            recommendations.push(format!(
                "Performance instability detected in {} metrics. Review measurement consistency.",
                high_variance_metrics.len()
            ));
        }
        
        // Analyze correlation insights
        if trend_data.len() > 1 {
            let metrics: Vec<String> = trend_data.keys().cloned().collect();
            recommendations.push(format!(
                "Monitor correlations between metrics: {}. Use insights for root cause analysis.",
                metrics.join(", ")
            ));
        }
        
        // Regression-specific recommendations
        if !regressions.is_empty() {
            let critical_regressions: Vec<_> = regressions
                .iter()
                .filter(|r| matches!(r.severity, crate::RegressionSeverity::Critical | crate::RegressionSeverity::Blocker))
                .collect();
            
            if !critical_regressions.is_empty() {
                recommendations.push(format!(
                    "URGENT: {} critical regressions detected. Priority fix required.",
                    critical_regressions.len()
                ));
            }
        }
        
        recommendations
    }

    // Helper methods

    /// Group measurements by metric type
    fn group_measurements_by_metric(
        &self,
        measurements: &[PerformanceMeasurement],
    ) -> HashMap<String, Vec<&PerformanceMeasurement>> {
        let mut grouped = HashMap::new();
        
        for measurement in measurements {
            grouped
                .entry(measurement.metric_type.clone())
                .or_insert_with(Vec::new)
                .push(measurement);
        }
        
        grouped
    }

    /// Group test results by name
    fn group_test_results_by_name(&self, results: &[TestResult]) -> HashMap<String, Vec<&TestResult>> {
        let mut grouped = HashMap::new();
        
        for result in results {
            grouped
                .entry(result.test_name.clone())
                .or_insert_with(Vec::new)
                .push(result);
        }
        
        grouped
    }

    /// Check cache validity and get data if available
    pub fn get_cached_trend_data(&self, key: &str) -> Option<&VecDeque<TrendDataPoint>> {
        if self.cache.is_valid() {
            self.cache.get_cached_data(key)
        } else {
            None
        }
    }

    /// Cache trend data for performance
    pub fn cache_trend_data(&mut self, key: String, data: VecDeque<TrendDataPoint>) {
        self.cache.cache_data(key, data);
        debug!("Cached trend data for performance optimization");
    }

    /// Clear expired cache
    pub fn clear_cache(&mut self) {
        self.cache.clear_expired();
    }
}

/// Correlation analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationAnalysis {
    pub metric1: String,
    pub metric2: String,
    pub correlation_coefficient: f64,
    pub strength: f64,
    pub significance: String,
    pub data_points: usize,
}

impl TrendAnalyzer {
    /// Create trend data from historical regression patterns
    pub async fn analyze_regression_patterns(
        &self,
        db: &DatabaseManager,
        component: &str,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<RegressionPatternAnalysis> {
        info!("Analyzing regression patterns for component: {}", component);
        
        // Get all regressions for the component in the time range
        // Note: This would require a query to filter by component and time range
        // For now, we'll work with the general unresolved regressions
        
        let all_regressions = db.get_unresolved_regressions().await?;
        let component_regressions: Vec<_> = all_regressions
            .iter()
            .filter(|r| r.component == component)
            .cloned()
            .collect();
        
        // Analyze patterns
        let pattern_analysis = self.analyze_regression_frequency(&component_regressions)?;
        let severity_pattern = self.analyze_severity_patterns(&component_regressions)?;
        let time_pattern = self.analyze_time_patterns(&component_regressions)?;
        
        Ok(RegressionPatternAnalysis {
            component: component.to_string(),
            analysis_period: (start_time, end_time),
            total_regressions: component_regressions.len(),
            pattern_analysis,
            severity_pattern,
            time_pattern,
            risk_score: self.calculate_risk_score(&component_regressions),
            recommendations: self.generate_pattern_recommendations(&component_regressions),
        })
    }

    /// Analyze frequency patterns of regressions
    fn analyze_regression_frequency(&self, regressions: &[DetectedRegression]) -> Result<PatternAnalysis> {
        if regressions.is_empty() {
            return Ok(PatternAnalysis {
                frequency_distribution: HashMap::new(),
                dominant_pattern: "none".to_string(),
                variability_score: 0.0,
            });
        }
        
        // Group regressions by type
        let mut frequency_distribution = HashMap::new();
        for regression in regressions {
            let pattern_key = format!("{:?}", regression.regression_type);
            *frequency_distribution.entry(pattern_key).or_insert(0) += 1;
        }
        
        // Find dominant pattern
        let dominant_pattern = frequency_distribution
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(pattern, _)| pattern.clone())
            .unwrap_or_else(|| "none".to_string());
        
        // Calculate variability score
        let frequencies: Vec<usize> = frequency_distribution.values().copied().collect();
        let mean_freq = frequencies.iter().sum::<usize>() as f64 / frequencies.len() as f64;
        let variance = frequencies
            .iter()
            .map(|&f| (f as f64 - mean_freq).powi(2))
            .sum::<f64>() / frequencies.len() as f64;
        let variability_score = variance.sqrt() / mean_freq if mean_freq > 0.0 else 0.0;
        
        Ok(PatternAnalysis {
            frequency_distribution,
            dominant_pattern,
            variability_score,
        })
    }

    /// Analyze severity patterns
    fn analyze_severity_patterns(&self, regressions: &[DetectedRegression]) -> Result<SeverityPattern> {
        if regressions.is_empty() {
            return Ok(SeverityPattern {
                severity_distribution: HashMap::new(),
                severity_trend: "stable".to_string(),
                escalation_risk: 0.0,
            });
        }
        
        let mut severity_distribution = HashMap::new();
        let mut severity_scores = Vec::new();
        
        for regression in regressions {
            let severity_key = format!("{:?}", regression.severity);
            *severity_distribution.entry(severity_key).or_insert(0) += 1;
            
            let severity_score = match regression.severity {
                crate::RegressionSeverity::Minor => 1.0,
                crate::RegressionSeverity::Major => 2.0,
                crate::RegressionSeverity::Critical => 3.0,
                crate::RegressionSeverity::Blocker => 4.0,
            };
            severity_scores.push(severity_score);
        }
        
        // Analyze severity trend (simplified - would need time series analysis)
        let avg_severity = severity_scores.iter().sum::<f64>() / severity_scores.len() as f64;
        let severity_trend = if avg_severity >= 2.5 {
            "escalating".to_string()
        } else if avg_severity >= 1.5 {
            "moderate".to_string()
        } else {
            "stable".to_string()
        };
        
        // Calculate escalation risk based on recent high-severity regressions
        let recent_critical = regressions
            .iter()
            .filter(|r| matches!(r.severity, crate::RegressionSeverity::Critical | crate::RegressionSeverity::Blocker))
            .count();
        let escalation_risk = (recent_critical as f64 / regressions.len() as f64).min(1.0);
        
        Ok(SeverityPattern {
            severity_distribution,
            severity_trend,
            escalation_risk,
        })
    }

    /// Analyze temporal patterns
    fn analyze_time_patterns(&self, regressions: &[DetectedRegression]) -> Result<TimePattern> {
        if regressions.is_empty() {
            return Ok(TimePattern {
                hourly_distribution: HashMap::new(),
                daily_distribution: HashMap::new(),
                monthly_distribution: HashMap::new(),
                peak_detection_times: Vec::new(),
            });
        }
        
        let mut hourly_distribution = HashMap::new();
        let mut daily_distribution = HashMap::new();
        let mut monthly_distribution = HashMap::new();
        
        for regression in regressions {
            let hour = regression.timestamp.hour();
            let day = regression.timestamp.weekday().to_string();
            let month = regression.timestamp.format("%Y-%m").to_string();
            
            *hourly_distribution.entry(hour).or_insert(0) += 1;
            *daily_distribution.entry(day).or_insert(0) += 1;
            *monthly_distribution.entry(month).or_insert(0) += 1;
        }
        
        // Find peak detection times
        let mut peak_times = hourly_distribution
            .iter()
            .max_by_key(|(_, &count)| count)
            .map(|(&hour, &count)| vec![(hour, count)])
            .unwrap_or_default();
        
        // Add secondary peaks
        if peak_times.len() == 1 {
            let peak_count = peak_times[0].1;
            let secondary_peaks: Vec<_> = hourly_distribution
                .iter()
                .filter(|(&hour, &count)| {
                    hour != peak_times[0].0 && count >= peak_count as f64 * 0.8
                })
                .map(|(&hour, &count)| (hour, count))
                .collect();
            peak_times.extend(secondary_peaks);
        }
        
        Ok(TimePattern {
            hourly_distribution,
            daily_distribution,
            monthly_distribution,
            peak_detection_times: peak_times,
        })
    }

    /// Calculate overall risk score
    fn calculate_risk_score(&self, regressions: &[DetectedRegression]) -> f64 {
        if regressions.is_empty() {
            return 0.0;
        }
        
        let severity_weights = HashMap::from([
            ("Minor", 1.0),
            ("Major", 2.0),
            ("Critical", 3.0),
            ("Blocker", 4.0),
        ]);
        
        let total_weighted_severity: f64 = regressions
            .iter()
            .map(|r| severity_weights.get(&format!("{:?}", r.severity))
                .copied().unwrap_or(1.0))
            .sum();
        
        let avg_severity = total_weighted_severity / regressions.len() as f64;
        let frequency_factor = (regressions.len() as f64 / 30.0).min(1.0); // Normalize by 30 days
        let recency_factor = self.calculate_recency_factor(regressions);
        
        // Weighted risk score
        (avg_severity * 0.4 + frequency_factor * 0.4 + recency_factor * 0.2) * 25.0 // Scale to 0-100
    }

    /// Calculate recency factor (how recent the regressions are)
    fn calculate_recency_factor(&self, regressions: &[DetectedRegression]) -> f64 {
        let now = Utc::now();
        
        let recency_scores: Vec<f64> = regressions
            .iter()
            .map(|r| {
                let hours_old = (now - r.timestamp).num_hours();
                let score = if hours_old < 24 {
                    1.0
                } else if hours_old < 168 { // 1 week
                    0.8
                } else if hours_old < 720 { // 1 month
                    0.5
                } else {
                    0.2
                };
                score
            })
            .collect();
        
        recency_scores.iter().sum::<f64>() / recency_scores.len() as f64
    }

    /// Generate recommendations based on pattern analysis
    fn generate_pattern_recommendations(&self, regressions: &[DetectedRegression]) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        if regressions.is_empty() {
            recommendations.push("No regressions detected in the analysis period. Continue monitoring.".to_string());
            return recommendations;
        }
        
        // High frequency recommendations
        if regressions.len() > 20 {
            recommendations.push(
                "High frequency of regressions detected. Review development processes and testing coverage.".to_string()
            );
        }
        
        // Severity escalation recommendations
        let critical_count = regressions
            .iter()
            .filter(|r| matches!(r.severity, crate::RegressionSeverity::Critical | crate::RegressionSeverity::Blocker))
            .count();
        
        if critical_count > regressions.len() / 4 {
            recommendations.push(
                "High proportion of critical regressions. Immediate action required to prevent system instability.".to_string()
            );
        }
        
        // Pattern-specific recommendations
        let pattern_analysis = self.analyze_regression_frequency(regressions).unwrap_or_default();
        if pattern_analysis.variability_score > 0.5 {
            recommendations.push(
                "High variability in regression types suggests diverse issues. Consider comprehensive system review.".to_string()
            );
        }
        
        // Time pattern recommendations
        let time_pattern = self.analyze_time_patterns(regressions).unwrap_or_default();
        if !time_pattern.peak_detection_times.is_empty() {
            let peak_hours: Vec<u32> = time_pattern.peak_detection_times
                .iter()
                .map(|(hour, _)| *hour)
                .collect();
            recommendations.push(format!(
                "Regressions peak during hours: {}. Consider increased monitoring during these periods.",
                peak_hours.iter().map(|h| h.to_string()).collect::<Vec<_>>().join(", ")
            ));
        }
        
        recommendations
    }
}

/// Regression pattern analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionPatternAnalysis {
    pub component: String,
    pub analysis_period: (DateTime<Utc>, DateTime<Utc>),
    pub total_regressions: usize,
    pub pattern_analysis: PatternAnalysis,
    pub severity_pattern: SeverityPattern,
    pub time_pattern: TimePattern,
    pub risk_score: f64,
    pub recommendations: Vec<String>,
}

/// Pattern analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternAnalysis {
    pub frequency_distribution: HashMap<String, usize>,
    pub dominant_pattern: String,
    pub variability_score: f64,
}

/// Severity pattern analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeverityPattern {
    pub severity_distribution: HashMap<String, usize>,
    pub severity_trend: String,
    pub escalation_risk: f64,
}

/// Time pattern analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimePattern {
    pub hourly_distribution: HashMap<u32, usize>,
    pub daily_distribution: HashMap<String, usize>,
    pub monthly_distribution: HashMap<String, usize>,
    pub peak_detection_times: Vec<(u32, usize)>,
}