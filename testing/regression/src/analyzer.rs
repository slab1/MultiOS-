//! Performance Analysis Module
//!
//! Provides advanced statistical analysis capabilities for performance data,
//! trend analysis, anomaly detection, and predictive modeling for regression testing.

use anyhow::{Result};
use chrono::{DateTime, Utc};
use log::{debug, info, warn};
use nalgebra::{vector, Matrix2, Vector2};
use serde::{Deserialize, Serialize};
use simple_statistics::{
    mean, median, percentile, standard_deviation, linear_regression,
    mean_and_standard_deviation, rolling_average
};
use std::collections::HashMap;

use crate::{
    PerformanceMeasurement, TrendData, TrendDirection, TrendPrediction,
    TrendStatistics, DatabaseManager, Uuid,
};

/// Advanced performance analyzer
#[derive(Debug, Clone)]
pub struct PerformanceAnalyzer {
    /// Configuration for statistical analysis
    pub config: AnalysisConfig,
}

/// Configuration for performance analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisConfig {
    pub outlier_detection: OutlierConfig,
    pub trend_analysis: TrendAnalysisConfig,
    pub anomaly_detection: AnomalyConfig,
    pub prediction_model: PredictionConfig,
}

/// Outlier detection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutlierConfig {
    pub method: OutlierMethod,
    pub z_score_threshold: f64,
    pub iqr_multiplier: f64,
}

/// Outlier detection methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutlierMethod {
    ZScore,
    IQR,
    ModifiedZScore,
    IsolationForest,
}

/// Trend analysis configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendAnalysisConfig {
    pub min_data_points: usize,
    pub regression_window_size: usize,
    pub significance_threshold: f64,
}

/// Anomaly detection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyConfig {
    pub method: AnomalyMethod,
    pub threshold: f64,
    pub sensitivity: f64,
}

/// Anomaly detection methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnomalyMethod {
    Statistical,
    MachineLearning,
    RuleBased,
    Hybrid,
}

/// Prediction model configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionConfig {
    pub model_type: PredictionModelType,
    pub confidence_level: f64,
    pub forecast_horizon: usize,
}

/// Prediction model types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PredictionModelType {
    Linear,
    Polynomial,
    Exponential,
    Seasonal,
    LSTM,
}

impl Default for AnalysisConfig {
    fn default() -> Self {
        Self {
            outlier_detection: OutlierConfig {
                method: OutlierMethod::ZScore,
                z_score_threshold: 2.0,
                iqr_multiplier: 1.5,
            },
            trend_analysis: TrendAnalysisConfig {
                min_data_points: 10,
                regression_window_size: 30,
                significance_threshold: 0.05,
            },
            anomaly_detection: AnomalyConfig {
                method: AnomalyMethod::Statistical,
                threshold: 2.0,
                sensitivity: 0.1,
            },
            prediction_model: PredictionModelConfig {
                model_type: PredictionModelType::Linear,
                confidence_level: 0.95,
                forecast_horizon: 7, // 7 days
            },
        }
    }
}

impl PerformanceAnalyzer {
    /// Create new performance analyzer
    pub fn new() -> Self {
        Self {
            config: AnalysisConfig::default(),
        }
    }

    /// Create analyzer with custom configuration
    pub fn with_config(config: AnalysisConfig) -> Self {
        Self { config }
    }

    /// Analyze performance measurement data
    pub async fn analyze_performance_data(
        &self,
        measurements: &[PerformanceMeasurement],
        component: &str,
    ) -> Result<PerformanceAnalysis> {
        info!("Analyzing performance data for component: {}", component);
        
        if measurements.is_empty() {
            return Err(anyhow::anyhow!("No measurements provided for analysis"));
        }

        let analysis = PerformanceAnalysis {
            component: component.to_string(),
            data_points: measurements.len(),
            statistical_summary: self.calculate_statistical_summary(measurements)?,
            trend_analysis: self.analyze_trends(measurements)?,
            outliers: self.detect_outliers(measurements),
            anomalies: self.detect_anomalies(measurements)?,
            predictions: self.generate_predictions(measurements)?,
            correlations: self.analyze_correlations(measurements)?,
            recommendations: self.generate_recommendations(measurements)?,
            timestamp: Utc::now(),
        };

        debug!("Performance analysis completed for {} data points", measurements.len());
        Ok(analysis)
    }

    /// Calculate statistical summary of measurements
    fn calculate_statistical_summary(&self, measurements: &[PerformanceMeasurement]) -> Result<TrendStatistics> {
        if measurements.is_empty() {
            return Err(anyhow::anyhow!("No measurements for statistical analysis"));
        }

        let values: Vec<f64> = measurements.iter().map(|m| m.value).collect();
        
        if values.is_empty() {
            return Err(anyhow::anyhow!("No values found in measurements"));
        }

        let mean_val = mean(&values).unwrap_or(0.0);
        let (stddev, _) = mean_and_standard_deviation(&values);
        let median_val = median(&values).unwrap_or(0.0);
        let percentile_95 = percentile(&values, 95.0).unwrap_or(0.0);
        let percentile_99 = percentile(&values, 99.0).unwrap_or(0.0);
        let min_val = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max_val = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        Ok(TrendStatistics {
            mean: mean_val,
            standard_deviation: stddev,
            median: median_val,
            percentile_95,
            percentile_99,
            min_value: min_val,
            max_value: max_val,
        })
    }

    /// Analyze trends in measurement data
    fn analyze_trends(&self, measurements: &[PerformanceMeasurement]) -> Result<TrendAnalysis> {
        if measurements.len() < self.config.trend_analysis.min_data_points {
            warn!("Insufficient data points for trend analysis: {} < {}", 
                  measurements.len(), self.config.trend_analysis.min_data_points);
            return Ok(TrendAnalysis::default());
        }

        let values: Vec<f64> = measurements.iter().map(|m| m.value).collect();
        let timestamps: Vec<f64> = measurements
            .iter()
            .map(|m| m.timestamp.timestamp() as f64)
            .collect();

        // Linear regression analysis
        let regression_result = self.calculate_linear_regression(&timestamps, &values);
        
        // Determine trend direction
        let trend_direction = match regression_result.slope {
            slope if slope.abs() < 0.001 => TrendDirection::Stable,
            slope if slope > 0.0 => TrendDirection::Degrading, // For most metrics, increasing is bad
            _ => TrendDirection::Improving,
        };

        // Calculate trend strength (R-squared)
        let r_squared = self.calculate_r_squared(&timestamps, &values, &regression_result);

        // Calculate trend statistics
        let change_rate = self.calculate_change_rate(&values);
        let volatility = self.calculate_volatility(&values);

        Ok(TrendAnalysis {
            direction: trend_direction,
            slope: regression_result.slope,
            intercept: regression_result.intercept,
            r_squared,
            change_rate,
            volatility,
            significance: self.calculate_significance(&regression_result, values.len()),
        })
    }

    /// Detect outliers using various methods
    fn detect_outliers(&self, measurements: &[PerformanceMeasurement]) -> Vec<OutlierDetection> {
        let mut outliers = Vec::new();
        let values: Vec<f64> = measurements.iter().map(|m| m.value).collect();

        match self.config.outlier_detection.method {
            OutlierMethod::ZScore => {
                outliers.extend(self.detect_zscore_outliers(&values));
            }
            OutlierMethod::IQR => {
                outliers.extend(self.detect_iqr_outliers(&values));
            }
            OutlierMethod::ModifiedZScore => {
                outliers.extend(self.detect_modified_zscore_outliers(&values));
            }
            OutlierMethod::IsolationForest => {
                // TODO: Implement isolation forest method
                warn!("IsolationForest outlier detection not yet implemented");
            }
        }

        // Add measurement context to outliers
        for outlier in &mut outliers {
            if let Some(measurement) = measurements.get(outlier.index) {
                outlier.timestamp = Some(measurement.timestamp);
                outlier.value = measurement.value;
                outlier.context = Some(measurement.test_name.clone());
            }
        }

        outliers
    }

    /// Detect outliers using Z-score method
    fn detect_zscore_outliers(&self, values: &[f64]) -> Vec<OutlierDetection> {
        let (mean_val, stddev) = mean_and_standard_deviation(values);
        let threshold = self.config.outlier_detection.z_score_threshold;

        if stddev == 0.0 {
            return Vec::new();
        }

        values
            .iter()
            .enumerate()
            .map(|(i, &value)| {
                let z_score = (value - mean_val).abs() / stddev;
                (i, z_score, value)
            })
            .filter(|(_, z_score, _)| *z_score > threshold)
            .map(|(index, z_score, value)| OutlierDetection {
                index,
                value,
                method: "ZScore".to_string(),
                score: z_score,
                timestamp: None,
                context: None,
            })
            .collect()
    }

    /// Detect outliers using Interquartile Range (IQR) method
    fn detect_iqr_outliers(&self, values: &[f64]) -> Vec<OutlierDetection> {
        if values.len() < 4 {
            return Vec::new();
        }

        let mut sorted_values = values.to_vec();
        sorted_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let q1 = percentile(&sorted_values, 25.0).unwrap_or(0.0);
        let q3 = percentile(&sorted_values, 75.0).unwrap_or(0.0);
        let iqr = q3 - q1;
        let lower_bound = q1 - self.config.outlier_detection.iqr_multiplier * iqr;
        let upper_bound = q3 + self.config.outlier_detection.iqr_multiplier * iqr;

        values
            .iter()
            .enumerate()
            .filter(|(_, &value)| value < lower_bound || value > upper_bound)
            .map(|(index, value)| {
                let score = if value < lower_bound {
                    (lower_bound - value).abs()
                } else {
                    value - upper_bound
                };
                OutlierDetection {
                    index,
                    value,
                    method: "IQR".to_string(),
                    score,
                    timestamp: None,
                    context: None,
                }
            })
            .collect()
    }

    /// Detect outliers using Modified Z-score method (more robust)
    fn detect_modified_zscore_outliers(&self, values: &[f64]) -> Vec<OutlierDetection> {
        if values.len() < 2 {
            return Vec::new();
        }

        let median_val = median(values).unwrap_or(0.0);
        
        // Calculate Median Absolute Deviation (MAD)
        let abs_deviations: Vec<f64> = values
            .iter()
            .map(|&value| (value - median_val).abs())
            .collect();
        
        let mad = median(&abs_deviations).unwrap_or(0.0);
        
        if mad == 0.0 {
            return Vec::new();
        }

        let threshold = self.config.outlier_detection.z_score_threshold;
        
        values
            .iter()
            .enumerate()
            .map(|(i, &value)| {
                let modified_z_score = 0.6745 * (value - median_val) / mad;
                (i, modified_z_score.abs(), value)
            })
            .filter(|(_, score, _)| *score > threshold)
            .map(|(index, score, value)| OutlierDetection {
                index,
                value,
                method: "ModifiedZScore".to_string(),
                score,
                timestamp: None,
                context: None,
            })
            .collect()
    }

    /// Detect anomalies using various methods
    fn detect_anomalies(&self, measurements: &[PerformanceMeasurement]) -> Result<Vec<AnomalyDetection>> {
        let mut anomalies = Vec::new();
        let values: Vec<f64> = measurements.iter().map(|m| m.value).collect();

        match self.config.anomaly_detection.method {
            AnomalyMethod::Statistical => {
                anomalies.extend(self.detect_statistical_anomalies(&values));
            }
            AnomalyMethod::RuleBased => {
                anomalies.extend(self.detect_rule_based_anomalies(measurements)?;
            }
            AnomalyMethod::MachineLearning => {
                // TODO: Implement ML-based anomaly detection
                warn!("MachineLearning anomaly detection not yet implemented");
            }
            AnomalyMethod::Hybrid => {
                let stat_anomalies = self.detect_statistical_anomalies(&values);
                let rule_anomalies = self.detect_rule_based_anomalies(measurements)?;
                
                // Combine and deduplicate
                anomalies.extend(stat_anomalies);
                anomalies.extend(rule_anomalies);
            }
        }

        // Add measurement context
        for anomaly in &mut anomalies {
            if let Some(measurement) = measurements.get(anomaly.index) {
                anomaly.timestamp = measurement.timestamp;
                anomaly.value = measurement.value;
                anomaly.test_name = measurement.test_name.clone();
                anomaly.component = measurement.component.clone();
            }
        }

        Ok(anomalies)
    }

    /// Detect statistical anomalies
    fn detect_statistical_anomalies(&self, values: &[f64]) -> Vec<AnomalyDetection> {
        if values.len() < 3 {
            return Vec::new();
        }

        let (mean_val, stddev) = mean_and_standard_deviation(values);
        let threshold = self.config.anomaly_detection.threshold;

        if stddev == 0.0 {
            return Vec::new();
        }

        let anomaly_indices: Vec<usize> = values
            .iter()
            .enumerate()
            .filter(|(_, &value)| {
                let z_score = (value - mean_val).abs() / stddev;
                z_score > threshold
            })
            .map(|(index, _)| index)
            .collect();

        // Check for sudden spikes or drops
        let sudden_changes = self.detect_sudden_changes(values);

        let mut all_indices = anomaly_indices;
        all_indices.extend(sudden_changes);
        all_indices.sort();
        all_indices.dedup();

        all_indices
            .into_iter()
            .map(|index| AnomalyDetection {
                index,
                value: values[index],
                method: "Statistical".to_string(),
                confidence: self.calculate_anomaly_confidence(values, index),
                timestamp: None,
                test_name: String::new(),
                component: String::new(),
                severity: self.calculate_anomaly_severity(values, index),
            })
            .collect()
    }

    /// Detect sudden changes in values
    fn detect_sudden_changes(&self, values: &[f64]) -> Vec<usize> {
        if values.len() < 2 {
            return Vec::new();
        }

        let threshold = self.config.anomaly_detection.sensitivity;
        let mut changes = Vec::new();

        for i in 1..values.len() {
            let change = (values[i] - values[i-1]).abs();
            let relative_change = if values[i-1] != 0.0 {
                change / values[i-1]
            } else {
                0.0
            };

            if relative_change > threshold {
                changes.push(i);
            }
        }

        changes
    }

    /// Detect rule-based anomalies
    fn detect_rule_based_anomalies(&self, measurements: &[PerformanceMeasurement]) -> Result<Vec<AnomalyDetection>> {
        let mut anomalies = Vec::new();

        for (i, measurement) in measurements.iter().enumerate() {
            // Rule 1: Extremely high values (e.g., > 99th percentile of baseline)
            if measurement.value > 1000.0 { // Arbitrary threshold for demo
                anomalies.push(AnomalyDetection {
                    index: i,
                    value: measurement.value,
                    method: "RuleBased".to_string(),
                    confidence: 0.8,
                    timestamp: measurement.timestamp,
                    test_name: measurement.test_name.clone(),
                    component: measurement.component.clone(),
                    severity: "High".to_string(),
                });
            }

            // Rule 2: Negative values (often indicate measurement errors)
            if measurement.value < 0.0 {
                anomalies.push(AnomalyDetection {
                    index: i,
                    value: measurement.value,
                    method: "RuleBased".to_string(),
                    confidence: 0.9,
                    timestamp: measurement.timestamp,
                    test_name: measurement.test_name.clone(),
                    component: measurement.component.clone(),
                    severity: "Medium".to_string(),
                });
            }
        }

        Ok(anomalies)
    }

    /// Generate predictions using selected model
    fn generate_predictions(&self, measurements: &[PerformanceMeasurement]) -> Result<Vec<TrendPrediction>> {
        if measurements.len() < self.config.trend_analysis.min_data_points {
            return Ok(Vec::new());
        }

        match self.config.prediction_model.model_type {
            PredictionModelType::Linear => self.generate_linear_predictions(measurements),
            PredictionModelType::Polynomial => self.generate_polynomial_predictions(measurements),
            PredictionModelType::Exponential => self.generate_exponential_predictions(measurements),
            PredictionModelType::Seasonal => self.generate_seasonal_predictions(measurements),
            PredictionModelType::LSTM => {
                warn!("LSTM prediction model not yet implemented, falling back to linear");
                self.generate_linear_predictions(measurements)
            }
        }
    }

    /// Generate linear predictions
    fn generate_linear_predictions(&self, measurements: &[PerformanceMeasurement]) -> Result<Vec<TrendPrediction>> {
        let values: Vec<f64> = measurements.iter().map(|m| m.value).collect();
        let timestamps: Vec<f64> = measurements
            .iter()
            .map(|m| m.timestamp.timestamp() as f64)
            .collect();

        let regression = self.calculate_linear_regression(&timestamps, &values);
        
        let mut predictions = Vec::new();
        let last_timestamp = timestamps.last().unwrap_or(&0.0);
        let time_step = self.estimate_time_step(&timestamps);

        for i in 1..=self.config.prediction_model.forecast_horizon {
            let future_timestamp = last_timestamp + (time_step * i as f64);
            let predicted_value = regression.slope * future_timestamp + regression.intercept;
            
            // Calculate confidence interval (simplified)
            let confidence_interval = self.calculate_prediction_confidence(&values, i as f64);

            predictions.push(TrendPrediction {
                timestamp: DateTime::from_timestamp(future_timestamp as i64, 0)
                    .unwrap_or(Utc::now()),
                predicted_value,
                confidence_interval,
                confidence_level: self.config.prediction_model.confidence_level,
            });
        }

        Ok(predictions)
    }

    /// Generate polynomial predictions (simplified quadratic)
    fn generate_polynomial_predictions(&self, measurements: &[PerformanceMeasurement]) -> Result<Vec<TrendPrediction>> {
        // For simplicity, use quadratic fitting (can be extended to higher degrees)
        let values: Vec<f64> = measurements.iter().map(|m| m.value).collect();
        let timestamps: Vec<f64> = measurements
            .iter()
            .map(|m| m.timestamp.timestamp() as f64)
            .collect();

        if values.len() < 3 {
            return self.generate_linear_predictions(measurements);
        }

        // Simplified polynomial regression (quadratic)
        let n = values.len() as f64;
        let sum_x = timestamps.iter().sum::<f64>();
        let sum_x2 = timestamps.iter().map(|x| x * x).sum::<f64>();
        let sum_x3 = timestamps.iter().map(|x| x * x * x).sum::<f64>();
        let sum_x4 = timestamps.iter().map(|x| x * x * x * x).sum::<f64>();
        let sum_y = values.iter().sum::<f64>();
        let sum_xy = timestamps.iter().zip(values.iter()).map(|(x, y)| x * y).sum::<f64>();
        let sum_x2y = timestamps.iter().zip(values.iter()).map(|(x, y)| x * x * y).sum::<f64>();

        // Solve for quadratic coefficients using least squares (simplified)
        // This is a basic implementation - in practice, use a proper linear algebra library
        let denom = n * sum_x2 * sum_x4 + 2.0 * sum_x * sum_x2 * sum_x3 - sum_x2.powi(2) * sum_x2 - n * sum_x3.powi(2) - sum_x.powi(2) * sum_x4;
        
        if denom.abs() < 1e-10 {
            return self.generate_linear_predictions(measurements);
        }

        let a = (sum_y * sum_x2 * sum_x4 + sum_xy * sum_x * sum_x4 + sum_x2y * sum_x * sum_x3 
                - sum_x2 * sum_x2y * sum_x3 - sum_y * sum_x.powi(2) * sum_x4 - sum_xy * sum_x2 * sum_x2) / denom;
        
        let b = (n * sum_xy * sum_x4 + sum_x * sum_x2y * sum_x3 + sum_x * sum_xy * sum_x2 
                - sum_x2 * sum_x2y * sum_x - n * sum_xy * sum_x3 - sum_x * sum_x * sum_x4) / denom;
        
        let c = (n * sum_x2 * sum_x2y + sum_x * sum_xy * sum_x3 + sum_x * sum_x * sum_x2y 
                - sum_x2 * sum_xy * sum_x - n * sum_x2y * sum_x3 - sum_x * sum_x * sum_x2y) / denom;

        let last_timestamp = timestamps.last().unwrap_or(&0.0);
        let time_step = self.estimate_time_step(&timestamps);

        let mut predictions = Vec::new();
        for i in 1..=self.config.prediction_model.forecast_horizon {
            let future_timestamp = last_timestamp + (time_step * i as f64);
            let predicted_value = a * future_timestamp.powi(2) + b * future_timestamp + c;
            
            predictions.push(TrendPrediction {
                timestamp: DateTime::from_timestamp(future_timestamp as i64, 0)
                    .unwrap_or(Utc::now()),
                predicted_value,
                confidence_interval: (0.0, 0.0), // Would need proper calculation
                confidence_level: self.config.prediction_model.confidence_level,
            });
        }

        Ok(predictions)
    }

    /// Generate exponential predictions
    fn generate_exponential_predictions(&self, measurements: &[PerformanceMeasurement]) -> Result<Vec<TrendPrediction>> {
        // For exponential prediction, take logarithm of values
        let values: Vec<f64> = measurements.iter().map(|m| m.value).filter(|&&x| x > 0.0).collect();
        
        if values.len() < self.config.trend_analysis.min_data_points || values.iter().any(|&x| x <= 0.0) {
            return self.generate_linear_predictions(measurements);
        }

        let log_values: Vec<f64> = values.iter().map(|x| x.ln()).collect();
        let timestamps: Vec<f64> = measurements.iter()
            .filter(|m| m.value > 0.0)
            .map(|m| m.timestamp.timestamp() as f64)
            .collect();

        let regression = self.calculate_linear_regression(&timestamps, &log_values);
        
        let last_timestamp = timestamps.last().unwrap_or(&0.0);
        let time_step = self.estimate_time_step(&timestamps);

        let mut predictions = Vec::new();
        for i in 1..=self.config.prediction_model.forecast_horizon {
            let future_timestamp = last_timestamp + (time_step * i as f64);
            let log_predicted_value = regression.slope * future_timestamp + regression.intercept;
            let predicted_value = log_predicted_value.exp();
            
            predictions.push(TrendPrediction {
                timestamp: DateTime::from_timestamp(future_timestamp as i64, 0)
                    .unwrap_or(Utc::now()),
                predicted_value,
                confidence_interval: (0.0, 0.0),
                confidence_level: self.config.prediction_model.confidence_level,
            });
        }

        Ok(predictions)
    }

    /// Generate seasonal predictions (simplified)
    fn generate_seasonal_predictions(&self, measurements: &[PerformanceMeasurement]) -> Result<Vec<TrendPrediction>> {
        // Simplified seasonal pattern detection
        // In a real implementation, this would use more sophisticated seasonal decomposition
        
        if measurements.len() < 24 { // Need at least 24 data points for daily seasonality
            return self.generate_linear_predictions(measurements);
        }

        // Detect daily patterns
        let hourly_averages = self.calculate_hourly_averages(measurements);
        
        let last_timestamp = measurements.last().unwrap_or(&measurements[0]).timestamp;
        let time_step = chrono::Duration::hours(1);
        
        let mut predictions = Vec::new();
        for i in 1..=self.config.prediction_model.forecast_horizon {
            let future_timestamp = last_timestamp + time_step * i as i32;
            let hour = future_timestamp.hour() as usize;
            
            let predicted_value = hourly_averages.get(&hour).copied().unwrap_or(0.0);
            
            predictions.push(TrendPrediction {
                timestamp: future_timestamp,
                predicted_value,
                confidence_interval: (predicted_value * 0.8, predicted_value * 1.2),
                confidence_level: self.config.prediction_model.confidence_level,
            });
        }

        Ok(predictions)
    }

    /// Analyze correlations between metrics
    fn analyze_correlations(&self, measurements: &[PerformanceMeasurement]) -> Result<HashMap<String, CorrelationAnalysis>> {
        // This would be more useful with multiple measurement series
        // For now, provide self-correlation (lag analysis)
        
        let mut correlations = HashMap::new();
        let values: Vec<f64> = measurements.iter().map(|m| m.value).collect();
        
        if values.len() < 2 {
            return Ok(correlations);
        }

        // Calculate autocorrelation
        let max_lag = (values.len() / 4).min(10); // Use up to 1/4 of data length or 10
        let mut autocorrelations = Vec::new();
        
        for lag in 1..=max_lag {
            if lag < values.len() {
                let correlation = self.calculate_autocorrelation(&values, lag);
                autocorrelations.push((lag, correlation));
            }
        }

        correlations.insert("autocorrelation".to_string(), CorrelationAnalysis {
            metric_name: "self".to_string(),
            correlation_type: "autocorrelation".to_string(),
            correlations: autocorrelations,
            significance: if autocorrelations.iter().any(|(_, corr)| corr.abs() > 0.3) {
                "significant".to_string()
            } else {
                "insignificant".to_string()
            },
        });

        Ok(correlations)
    }

    /// Calculate autocorrelation at given lag
    fn calculate_autocorrelation(&self, values: &[f64], lag: usize) -> f64 {
        if values.len() <= lag {
            return 0.0;
        }

        let n = values.len() - lag;
        let mean_val = mean(values).unwrap_or(0.0);
        
        let numerator: f64 = (0..n)
            .map(|i| (values[i] - mean_val) * (values[i + lag] - mean_val))
            .sum();
        
        let denominator: f64 = values
            .iter()
            .map(|&value| (value - mean_val).powi(2))
            .sum();

        if denominator == 0.0 {
            0.0
        } else {
            numerator / denominator
        }
    }

    /// Generate recommendations based on analysis
    fn generate_recommendations(&self, measurements: &[PerformanceMeasurement]) -> Result<Vec<String>> {
        let mut recommendations = Vec::new();

        if measurements.is_empty() {
            return Ok(recommendations);
        }

        let values: Vec<f64> = measurements.iter().map(|m| m.value).collect();
        let mean_val = mean(&values).unwrap_or(0.0);
        let stddev = standard_deviation(&values).unwrap_or(0.0);

        // High variability recommendation
        if stddev > mean_val * 0.5 {
            recommendations.push(
                "High performance variability detected. Consider investigating root causes.".to_string()
            );
        }

        // Trend recommendations
        let trend_analysis = self.analyze_trends(measurements)?;
        match trend_analysis.direction {
            TrendDirection::Degrading => {
                recommendations.push(
                    "Performance is degrading over time. Review recent changes and optimize bottlenecks.".to_string()
                );
            }
            TrendDirection::Improving => {
                recommendations.push(
                    "Performance is improving. Document current optimizations for future reference.".to_string()
                );
            }
            TrendDirection::Stable => {
                recommendations.push(
                    "Performance is stable. Continue current monitoring practices.".to_string()
                );
            }
            _ => {}
        }

        // Outlier recommendations
        let outliers = self.detect_outliers(measurements);
        if outliers.len() > measurements.len() * 0.1 {
            recommendations.push(
                "Frequent outliers detected. Consider improving measurement consistency.".to_string()
            );
        }

        // Insufficient data recommendation
        if measurements.len() < 100 {
            recommendations.push(
                "Insufficient data for robust analysis. Collect more measurements.".to_string()
            );
        }

        Ok(recommendations)
    }

    // Helper methods

    /// Calculate linear regression
    fn calculate_linear_regression(&self, x: &[f64], y: &[f64]) -> LinearRegression {
        let n = x.len();
        if n != y.len() || n == 0 {
            return LinearRegression {
                slope: 0.0,
                intercept: 0.0,
            };
        }

        let sum_x = x.iter().sum::<f64>();
        let sum_y = y.iter().sum::<f64>();
        let sum_xy = x.iter().zip(y.iter()).map(|(xi, yi)| xi * yi).sum::<f64>();
        let sum_x2 = x.iter().map(|xi| xi * xi).sum::<f64>();

        let denominator = n as f64 * sum_x2 - sum_x * sum_x;
        
        if denominator.abs() < 1e-10 {
            return LinearRegression {
                slope: 0.0,
                intercept: sum_y / n as f64,
            };
        }

        let slope = (n as f64 * sum_xy - sum_x * sum_y) / denominator;
        let intercept = (sum_y - slope * sum_x) / n as f64;

        LinearRegression { slope, intercept }
    }

    /// Calculate R-squared value
    fn calculate_r_squared(&self, x: &[f64], y: &[f64], regression: &LinearRegression) -> f64 {
        if x.len() != y.len() || x.is_empty() {
            return 0.0;
        }

        let mean_y = mean(y).unwrap_or(0.0);
        let total_sum_squares: f64 = y.iter().map(|yi| (yi - mean_y).powi(2)).sum();
        
        if total_sum_squares == 0.0 {
            return 1.0;
        }

        let residual_sum_squares: f64 = x.iter().zip(y.iter())
            .map(|(xi, yi)| {
                let predicted = regression.slope * xi + regression.intercept;
                (yi - predicted).powi(2)
            })
            .sum();

        1.0 - (residual_sum_squares / total_sum_squares)
    }

    /// Calculate change rate
    fn calculate_change_rate(&self, values: &[f64]) -> f64 {
        if values.len() < 2 {
            return 0.0;
        }

        let first_value = values[0];
        let last_value = values[values.len() - 1];
        
        if first_value == 0.0 {
            0.0
        } else {
            ((last_value - first_value) / first_value) * 100.0
        }
    }

    /// Calculate volatility (coefficient of variation)
    fn calculate_volatility(&self, values: &[f64]) -> f64 {
        if values.is_empty() {
            return 0.0;
        }

        let mean_val = mean(values).unwrap_or(0.0);
        let stddev = standard_deviation(values).unwrap_or(0.0);
        
        if mean_val == 0.0 {
            0.0
        } else {
            (stddev / mean_val) * 100.0
        }
    }

    /// Calculate statistical significance
    fn calculate_significance(&self, regression: &LinearRegression, sample_size: usize) -> f64 {
        // Simplified significance calculation
        // In a full implementation, this would use proper statistical tests
        let abs_slope = regression.slope.abs();
        
        // Higher significance for larger slopes and larger sample sizes
        let sample_factor = (sample_size as f64).sqrt();
        (abs_slope * sample_factor).min(1.0)
    }

    /// Calculate anomaly confidence
    fn calculate_anomaly_confidence(&self, values: &[f64], index: usize) -> f64 {
        if values.is_empty() || index >= values.len() {
            return 0.0;
        }

        let (mean_val, stddev) = mean_and_standard_deviation(values);
        if stddev == 0.0 {
            return 0.0;
        }

        let z_score = (values[index] - mean_val).abs() / stddev;
        (z_score / 3.0).min(1.0) // Normalize to [0, 1]
    }

    /// Calculate anomaly severity
    fn calculate_anomaly_severity(&self, values: &[f64], index: usize) -> String {
        if values.is_empty() || index >= values.len() {
            return "Low".to_string();
        }

        let (mean_val, stddev) = mean_and_standard_deviation(values);
        if stddev == 0.0 {
            return "Low".to_string();
        }

        let z_score = (values[index] - mean_val).abs() / stddev;
        
        match z_score {
            z if z > 3.0 => "High",
            z if z > 2.0 => "Medium",
            _ => "Low",
        }.to_string()
    }

    /// Calculate prediction confidence interval
    fn calculate_prediction_confidence(&self, values: &[f64], future_steps: f64) -> (f64, f64) {
        let stddev = standard_deviation(values).unwrap_or(0.0);
        let margin = stddev * future_steps.sqrt(); // Uncertainty grows with sqrt of steps
        
        // This is a simplified calculation - real implementation would use proper prediction intervals
        (0.0, margin * 2.0)
    }

    /// Estimate time step from timestamps
    fn estimate_time_step(&self, timestamps: &[f64]) -> f64 {
        if timestamps.len() < 2 {
            return 3600.0; // Default to 1 hour in seconds
        }

        let differences: Vec<f64> = timestamps.windows(2)
            .map(|window| window[1] - window[0])
            .collect();

        mean(&differences).unwrap_or(3600.0)
    }

    /// Calculate hourly averages for seasonal patterns
    fn calculate_hourly_averages(&self, measurements: &[PerformanceMeasurement]) -> HashMap<usize, f64> {
        let mut hourly_sums = HashMap::new();
        let mut hourly_counts = HashMap::new();

        for measurement in measurements {
            let hour = measurement.timestamp.hour() as usize;
            *hourly_sums.entry(hour).or_insert(0.0) += measurement.value;
            *hourly_counts.entry(hour).or_insert(0) += 1;
        }

        hourly_sums
            .into_iter()
            .map(|(hour, sum)| {
                let count = hourly_counts.get(&hour).copied().unwrap_or(1);
                (hour, sum / count as f64)
            })
            .collect()
    }
}

/// Performance analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAnalysis {
    pub component: String,
    pub data_points: usize,
    pub statistical_summary: TrendStatistics,
    pub trend_analysis: TrendAnalysis,
    pub outliers: Vec<OutlierDetection>,
    pub anomalies: Vec<AnomalyDetection>,
    pub predictions: Vec<TrendPrediction>,
    pub correlations: HashMap<String, CorrelationAnalysis>,
    pub recommendations: Vec<String>,
    pub timestamp: DateTime<Utc>,
}

/// Trend analysis result
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TrendAnalysis {
    pub direction: TrendDirection,
    pub slope: f64,
    pub intercept: f64,
    pub r_squared: f64,
    pub change_rate: f64,
    pub volatility: f64,
    pub significance: f64,
}

/// Outlier detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutlierDetection {
    pub index: usize,
    pub value: f64,
    pub method: String,
    pub score: f64,
    pub timestamp: Option<DateTime<Utc>>,
    pub context: Option<String>,
}

/// Anomaly detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyDetection {
    pub index: usize,
    pub value: f64,
    pub method: String,
    pub confidence: f64,
    pub timestamp: Option<DateTime<Utc>>,
    pub test_name: String,
    pub component: String,
    pub severity: String,
}

/// Linear regression result
#[derive(Debug, Clone)]
pub struct LinearRegression {
    pub slope: f64,
    pub intercept: f64,
}

/// Correlation analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationAnalysis {
    pub metric_name: String,
    pub correlation_type: String,
    pub correlations: Vec<(usize, f64)>,
    pub significance: String,
}