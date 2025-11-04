//! Performance Analysis Module
//! 
//! Advanced analysis engine for scheduler performance data including:
//! - Trend analysis and pattern recognition
//! - Performance regression detection
//! - Optimization recommendation generation
//! - Workload characterization
//! - Predictive modeling

use crate::*;
use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

/// Advanced analysis engine for scheduler performance
pub struct AnalysisEngine {
    config: ProfilerConfig,
    /// Historical performance baselines
    baselines: HashMap<String, PerformanceBaseline>,
    /// Trend analysis data
    trend_data: TrendAnalysis,
    /// Regression detector
    regression_detector: RegressionDetector,
    /// Optimization recommender
    optimization_recommender: OptimizationRecommender,
    /// Workload classifier
    workload_classifier: WorkloadClassifier,
}

/// Performance baseline for comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceBaseline {
    pub name: String,
    pub algorithm: SchedulerAlgorithm,
    pub core_count: usize,
    pub metrics: BaselineMetrics,
    pub created_at: Instant,
    pub valid_until: Instant,
    pub confidence_level: f32,
}

/// Baseline performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineMetrics {
    pub avg_scheduling_latency_ns: u64,
    pub avg_context_switch_overhead_us: f32,
    pub avg_cpu_utilization: f32,
    pub avg_fairness_index: f32,
    pub avg_responsiveness_score: f32,
    pub avg_throughput: f64,
    pub load_balancing_efficiency: f32,
    pub priority_inversions_per_hour: u32,
}

/// Trend analysis data
#[derive(Debug)]
pub struct TrendAnalysis {
    /// Performance trends over time
    latency_trend: TrendLine,
    throughput_trend: TrendLine,
    fairness_trend: TrendLine,
    utilization_trend: TrendLine,
    /// Seasonal patterns (if any)
    seasonal_patterns: HashMap<String, Vec<f32>>,
}

/// Performance trend line
#[derive(Debug)]
pub struct TrendLine {
    pub data_points: Vec<(Instant, f32)>,
    pub slope: f32,
    pub r_squared: f32,
    pub trend_direction: TrendDirection,
}

/// Trend direction
#[derive(Debug, Clone, PartialEq)]
pub enum TrendDirection {
    Improving,
    Stable,
    Degrading,
    Unknown,
}

/// Performance regression detector
pub struct RegressionDetector {
    /// Regression thresholds
    latency_regression_threshold: f32,      // 20% increase
    throughput_regression_threshold: f32,   // 15% decrease
    fairness_regression_threshold: f32,     // 10% decrease
    /// Detected regressions
    detected_regressions: Vec<PerformanceRegression>,
}

/// Performance regression information
#[derive(Debug, Clone)]
pub struct PerformanceRegression {
    pub regression_type: RegressionType,
    pub severity: RegressionSeverity,
    pub affected_metrics: Vec<String>,
    pub detected_at: Instant,
    pub baseline_name: String,
    pub current_value: f32,
    pub baseline_value: f32,
    pub deviation_percent: f32,
}

/// Types of performance regressions
#[derive(Debug, Clone, PartialEq)]
pub enum RegressionType {
    LatencyIncrease,
    ThroughputDecrease,
    FairnessDegradation,
    ResponsivenessDrop,
    LoadImbalance,
    PriorityInversionIncrease,
}

/// Regression severity levels
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum RegressionSeverity {
    Minor,      // < 10% deviation
    Moderate,   // 10-25% deviation
    Severe,     // 25-50% deviation
    Critical,   // > 50% deviation
}

/// Optimization recommendation engine
pub struct OptimizationRecommender {
    /// Rule-based recommendations
    rule_engine: OptimizationRuleEngine,
    /// ML-based recommendations
    ml_recommender: Option<MLRecommender>,
}

/// Optimization rule engine
pub struct OptimizationRuleEngine {
    /// Optimization rules
    rules: Vec<OptimizationRule>,
}

/// Single optimization rule
#[derive(Debug, Clone)]
pub struct OptimizationRule {
    pub name: String,
    pub condition: OptimizationCondition,
    pub recommendation: OptimizationRecommendation,
    pub confidence: f32,
    pub priority: u8,
}

/// Optimization condition
#[derive(Debug, Clone)]
pub struct OptimizationCondition {
    pub metric: String,
    pub operator: ComparisonOperator,
    pub threshold: f32,
    pub duration: Option<Duration>,
}

/// Optimization recommendation
#[derive(Debug, Clone)]
pub struct OptimizationRecommendation {
    pub title: String,
    pub description: String,
    pub action: OptimizationAction,
    pub expected_improvement: f32,
    pub risk_level: RiskLevel,
    pub implementation_difficulty: ImplementationDifficulty,
}

/// Optimization actions
#[derive(Debug, Clone)]
pub enum OptimizationAction {
    AdjustSchedulerQuantum { new_quantum_us: u64 },
    ChangeSchedulingAlgorithm { new_algorithm: SchedulerAlgorithm },
    EnablePriorityInheritance,
    AdjustLoadBalancingParameters { interval_ms: u64, threshold: f32 },
    TuneContextSwitchThreshold { new_threshold: u64 },
    EnableAging,
    AdjustPriorityLevels { boost_factor: f32 },
}

/// Risk levels for optimizations
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Implementation difficulty
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum ImplementationDifficulty {
    Easy,      // Configuration change
    Medium,    // Algorithm parameter tuning
    Hard,      // Algorithm implementation change
    VeryHard,  // Architecture modification
}

/// Comparison operators
#[derive(Debug, Clone)]
pub enum ComparisonOperator {
    GreaterThan,
    LessThan,
    EqualTo,
    GreaterThanOrEqual,
    LessThanOrEqual,
}

/// Machine learning-based recommender
pub struct MLRecommender {
    /// Trained models
    models: HashMap<String, Box<dyn RecommendationModel>>,
    /// Training data
    training_data: Vec<MLTrainingSample>,
}

/// Training sample for ML recommender
#[derive(Debug, Clone)]
struct MLTrainingSample {
    pub features: Vec<f32>,
    pub label: String,
    pub performance_impact: f32,
}

/// Recommendation model interface
pub trait RecommendationModel {
    fn predict(&self, features: &[f32]) -> Vec<f32>;
    fn train(&mut self, samples: &[MLTrainingSample]) -> Result<(), Box<dyn std::error::Error>>;
    fn get_model_name(&self) -> &str;
}

/// Workload classification and characterization
pub struct WorkloadClassifier {
    /// Workload characteristics database
    workload_database: HashMap<String, WorkloadProfile>,
    /// Classification models
    classifiers: HashMap<String, Box<dyn WorkloadClassifierModel>>,
}

/// Workload profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkloadProfile {
    pub name: String,
    pub characteristics: WorkloadCharacteristics,
    pub recommended_algorithm: SchedulerAlgorithm,
    pub recommended_parameters: SchedulerParameters,
    pub performance_expectations: PerformanceExpectations,
}

/// Workload characteristics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkloadCharacteristics {
    pub cpu_intensity: f32,        // 0.0 = I/O bound, 1.0 = CPU bound
    pub thread_count: u32,
    pub lock_contention_level: f32,
    pub memory_access_pattern: MemoryAccessPattern,
    pub communication_pattern: CommunicationPattern,
    pub deadline_sensitivity: f32, // 0.0 = no deadlines, 1.0 = hard real-time
    pub interactive_requirement: f32, // 0.0 = batch, 1.0 = interactive
}

/// Memory access patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemoryAccessPattern {
    Sequential,
    Random,
    Strided,
    WorkingSet,
    Streaming,
}

/// Communication patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommunicationPattern {
    None,
    SharedMemory,
    MessagePassing,
    ProducerConsumer,
    Collective,
}

/// Scheduler parameters for specific workload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerParameters {
    pub time_quantum_us: u64,
    pub priority_levels: u8,
    pub aging_enabled: bool,
    pub inheritance_enabled: bool,
    pub load_balance_interval_ms: u64,
    pub context_switch_threshold_us: u64,
}

/// Performance expectations for workload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceExpectations {
    pub target_throughput: f64,
    pub max_scheduling_latency_us: u64,
    pub min_fairness_index: f32,
    pub min_responsiveness_score: f32,
    pub max_priority_inversions_per_hour: u32,
}

/// Workload classifier model interface
pub trait WorkloadClassifierModel {
    fn classify(&self, sample: &PerformanceSample) -> WorkloadClassification;
    fn train(&mut self, samples: &[PerformanceSample], labels: &[String]) -> Result<(), Box<dyn std::error::Error>>;
}

/// Workload classification result
#[derive(Debug, Clone)]
pub struct WorkloadClassification {
    pub workload_type: WorkloadType,
    pub confidence: f32,
    pub characteristics: WorkloadCharacteristics,
    pub recommended_algorithm: SchedulerAlgorithm,
}

/// Types of workloads
#[derive(Debug, Clone, PartialEq)]
pub enum WorkloadType {
    CpuBound,
    IoBound,
    MemoryBound,
    CommunicationIntensive,
    RealTime,
    Interactive,
    Batch,
    Mixed,
}

impl AnalysisEngine {
    /// Create new analysis engine
    pub fn new(config: ProfilerConfig) -> Self {
        Self {
            config,
            baselines: HashMap::new(),
            trend_data: TrendAnalysis::new(),
            regression_detector: RegressionDetector::new(),
            optimization_recommender: OptimizationRecommender::new(),
            workload_classifier: WorkloadClassifier::new(),
        }
    }

    /// Analyze performance trends
    pub async fn analyze_trends(&self, samples: &[PerformanceSample]) -> Vec<TrendAnalysisResult> {
        let mut results = Vec::new();

        // Analyze scheduling latency trends
        if let Some(latency_trend) = self.analyze_latency_trend(samples) {
            results.push(TrendAnalysisResult {
                metric: "scheduling_latency".to_string(),
                trend: latency_trend.trend_direction,
                slope: latency_trend.slope,
                r_squared: latency_trend.r_squared,
                confidence: latency_trend.r_squared * 0.8 + 0.1,
                description: self.generate_trend_description(&latency_trend),
            });
        }

        // Analyze throughput trends
        if let Some(throughput_trend) = self.analyze_throughput_trend(samples) {
            results.push(TrendAnalysisResult {
                metric: "throughput".to_string(),
                trend: throughput_trend.trend_direction,
                slope: throughput_trend.slope,
                r_squared: throughput_trend.r_squared,
                confidence: throughput_trend.r_squared * 0.8 + 0.1,
                description: self.generate_trend_description(&throughput_trend),
            });
        }

        // Analyze fairness trends
        if let Some(fairness_trend) = self.analyze_fairness_trend(samples) {
            results.push(TrendAnalysisResult {
                metric: "fairness".to_string(),
                trend: fairness_trend.trend_direction,
                slope: fairness_trend.slope,
                r_squared: fairness_trend.r_squared,
                confidence: fairness_trend.r_squared * 0.8 + 0.1,
                description: self.generate_trend_description(&fairness_trend),
            });
        }

        results
    }

    /// Detect performance regressions
    pub async fn detect_regressions(&self, samples: &[PerformanceSample]) -> Vec<PerformanceRegression> {
        let current_metrics = self.calculate_current_metrics(samples);
        let mut regressions = Vec::new();

        // Check each baseline for regressions
        for (name, baseline) in &self.baselines {
            if current_metrics.avg_scheduling_latency_ns > baseline.metrics.avg_scheduling_latency_ns as f32 * (1.0 + self.regression_detector.latency_regression_threshold) {
                regressions.push(PerformanceRegression {
                    regression_type: RegressionType::LatencyIncrease,
                    severity: self.calculate_severity(
                        current_metrics.avg_scheduling_latency_ns as f32 / baseline.metrics.avg_scheduling_latency_ns as f32 - 1.0
                    ),
                    affected_metrics: vec!["scheduling_latency".to_string()],
                    detected_at: Instant::now(),
                    baseline_name: name.clone(),
                    current_value: current_metrics.avg_scheduling_latency_ns as f32,
                    baseline_value: baseline.metrics.avg_scheduling_latency_ns as f32,
                    deviation_percent: (current_metrics.avg_scheduling_latency_ns as f32 / baseline.metrics.avg_scheduling_latency_ns as f32 - 1.0) * 100.0,
                });
            }

            if current_metrics.avg_throughput < baseline.metrics.avg_throughput as f32 * (1.0 - self.regression_detector.throughput_regression_threshold) {
                regressions.push(PerformanceRegression {
                    regression_type: RegressionType::ThroughputDecrease,
                    severity: self.calculate_severity(
                        1.0 - current_metrics.avg_throughput / baseline.metrics.avg_throughput as f32
                    ),
                    affected_metrics: vec!["throughput".to_string()],
                    detected_at: Instant::now(),
                    baseline_name: name.clone(),
                    current_value: current_metrics.avg_throughput,
                    baseline_value: baseline.metrics.avg_throughput as f32,
                    deviation_percent: (1.0 - current_metrics.avg_throughput / baseline.metrics.avg_throughput as f32) * 100.0,
                });
            }
        }

        regressions
    }

    /// Generate optimization recommendations
    pub async fn generate_recommendations(&self, samples: &[PerformanceSample]) -> Vec<String> {
        let current_metrics = self.calculate_current_metrics(samples);
        let workload_classification = self.workload_classifier.classify_workload(samples).await;
        let recommendations = self.optimization_recommender.generate_recommendations(&current_metrics, &workload_classification);

        recommendations.iter()
            .map(|rec| format!("{}: {} (Expected improvement: {:.1}%)", rec.title, rec.description, rec.expected_improvement * 100.0))
            .collect()
    }

    /// Generate comprehensive performance report
    pub async fn generate_report(&self, samples: &[PerformanceSample]) -> Result<String, Box<dyn std::error::Error>> {
        let mut report = String::new();
        
        // Report header
        report.push_str("# MultiOS Scheduler Performance Report\n\n");
        report.push_str(&format!("Generated: {}\n", Utc::now()));
        report.push_str(&format!("Algorithm: {:?}\n", self.config.algorithm));
        report.push_str(&format!("Core Count: {}\n", self.config.core_count));
        report.push_str(&format!("Sample Count: {}\n\n", samples.len()));

        // Performance summary
        report.push_str("## Performance Summary\n\n");
        let metrics = self.calculate_current_metrics(samples);
        report.push_str(&format!("- Average Scheduling Latency: {:.2} ns\n", metrics.avg_scheduling_latency_ns));
        report.push_str(&format!("- Average Context Switch Overhead: {:.2} µs\n", metrics.avg_context_switch_overhead_us));
        report.push_str(&format!("- Average CPU Utilization: {:.2}%\n", metrics.avg_cpu_utilization * 100.0));
        report.push_str(&format!("- Average Fairness Index: {:.3}\n", metrics.avg_fairness_index));
        report.push_str(&format!("- Average Responsiveness Score: {:.3}\n", metrics.avg_responsiveness_score));
        report.push_str(&format!("- Average Throughput: {:.0} tasks/sec\n\n", metrics.avg_throughput));

        // Trend analysis
        report.push_str("## Trend Analysis\n\n");
        let trends = self.analyze_trends(samples).await;
        for trend in trends {
            report.push_str(&format!("- **{}**: {} (confidence: {:.1}%)\n", 
                trend.metric, trend.description, trend.confidence * 100.0));
        }
        report.push_str("\n");

        // Regression analysis
        report.push_str("## Regression Analysis\n\n");
        let regressions = self.detect_regressions(samples).await;
        if regressions.is_empty() {
            report.push_str("No performance regressions detected.\n\n");
        } else {
            for regression in regressions {
                report.push_str(&format!("- **{:?}**: {:.1}% deviation from baseline '{}' (severity: {:?})\n",
                    regression.regression_type, regression.deviation_percent, regression.baseline_name, regression.severity));
            }
            report.push_str("\n");
        }

        // Workload characterization
        report.push_str("## Workload Characterization\n\n");
        let classification = self.workload_classifier.classify_workload(samples).await;
        report.push_str(&format!("- **Workload Type**: {:?}\n", classification.workload_type));
        report.push_str(&format!("- **CPU Intensity**: {:.2}\n", classification.characteristics.cpu_intensity));
        report.push_str(&format!("- **Lock Contention**: {:.2}\n", classification.characteristics.lock_contention_level));
        report.push_str(&format!("- **Interactive Requirement**: {:.2}\n\n", classification.characteristics.interactive_requirement));

        // Optimization recommendations
        report.push_str("## Optimization Recommendations\n\n");
        let recommendations = self.generate_recommendations(samples).await;
        if recommendations.is_empty() {
            report.push_str("No optimization recommendations at this time.\n");
        } else {
            for rec in recommendations {
                report.push_str(&format!("- {}\n", rec));
            }
        }

        Ok(report)
    }

    /// Calculate current performance metrics from samples
    fn calculate_current_metrics(&self, samples: &[PerformanceSample]) -> CurrentMetrics {
        if samples.is_empty() {
            return CurrentMetrics::default();
        }

        let mut total_latency = 0u64;
        let mut total_overhead = 0.0f32;
        let mut total_utilization = 0.0f32;
        let mut total_fairness = 0.0f32;
        let mut total_responsiveness = 0.0f32;
        let mut total_throughput = 0.0f64;

        for sample in samples {
            total_latency += sample.scheduling_latency.avg_ns as u64;
            total_overhead += sample.context_switch_overhead.avg_microseconds;
            total_utilization += sample.cpu_utilization.iter().sum::<f32>() / sample.cpu_utilization.len() as f32;
            total_fairness += sample.fairness_index;
            total_responsiveness += sample.responsiveness_score;
            total_throughput += sample.throughput;
        }

        let count = samples.len() as f32;

        CurrentMetrics {
            avg_scheduling_latency_ns: total_latency as f32 / count,
            avg_context_switch_overhead_us: total_overhead / count,
            avg_cpu_utilization: total_utilization / count,
            avg_fairness_index: total_fairness / count,
            avg_responsiveness_score: total_responsiveness / count,
            avg_throughput: total_throughput as f32 / count,
            load_balancing_efficiency: samples.last().map(|s| s.load_balancing_efficiency).unwrap_or(0.0),
            priority_inversions_per_hour: 0, // Would be calculated from events
        }
    }

    /// Analyze latency trend
    fn analyze_latency_trend(&self, samples: &[PerformanceSample]) -> Option<TrendLine> {
        if samples.len() < 10 {
            return None;
        }

        let data_points: Vec<(Instant, f32)> = samples.iter()
            .map(|s| (s.timestamp.with_timezone(&Utc).timestamp_nanos_opt().unwrap_or(0) as Instant, s.scheduling_latency.avg_ns as f32))
            .collect();

        self.calculate_trend_line(&data_points)
    }

    /// Analyze throughput trend
    fn analyze_throughput_trend(&self, samples: &[PerformanceSample]) -> Option<TrendLine> {
        if samples.len() < 10 {
            return None;
        }

        let data_points: Vec<(Instant, f32)> = samples.iter()
            .map(|s| (s.timestamp.with_timezone(&Utc).timestamp_nanos_opt().unwrap_or(0) as Instant, s.throughput as f32))
            .collect();

        self.calculate_trend_line(&data_points)
    }

    /// Analyze fairness trend
    fn analyze_fairness_trend(&self, samples: &[PerformanceSample]) -> Option<TrendLine> {
        if samples.len() < 10 {
            return None;
        }

        let data_points: Vec<(Instant, f32)> = samples.iter()
            .map(|s| (s.timestamp.with_timezone(&Utc).timestamp_nanos_opt().unwrap_or(0) as Instant, s.fairness_index))
            .collect();

        self.calculate_trend_line(&data_points)
    }

    /// Calculate trend line from data points
    fn calculate_trend_line(&self, data_points: &[(Instant, f32)]) -> Option<TrendLine> {
        if data_points.len() < 2 {
            return None;
        }

        let n = data_points.len();
        let x_values: Vec<f32> = (0..n).map(|i| i as f32).collect();
        let y_values: Vec<f32> = data_points.iter().map(|(_, y)| *y).collect();

        // Simple linear regression
        let sum_x = x_values.iter().sum::<f32>();
        let sum_y = y_values.iter().sum::<f32>();
        let sum_xy = x_values.iter().zip(y_values.iter()).map(|(x, y)| x * y).sum::<f32>();
        let sum_xx = x_values.iter().map(|x| x * x).sum::<f32>();

        let slope = (n as f32 * sum_xy - sum_x * sum_y) / (n as f32 * sum_xx - sum_x * sum_x);
        let intercept = (sum_y - slope * sum_x) / n as f32;

        // Calculate R-squared
        let y_mean = sum_y / n as f32;
        let ss_tot = y_values.iter().map(|y| (y - y_mean).powi(2)).sum::<f32>();
        let ss_res = y_values.iter().enumerate().map(|(i, y)| {
            let predicted = slope * x_values[i] + intercept;
            (y - predicted).powi(2)
        }).sum::<f32>();

        let r_squared = if ss_tot > 0.0 { 1.0 - ss_res / ss_tot } else { 0.0 };

        let trend_direction = if slope.abs() < 0.01 {
            TrendDirection::Stable
        } else if slope > 0.0 {
            TrendDirection::Improving
        } else {
            TrendDirection::Degrading
        };

        Some(TrendLine {
            data_points: data_points.to_vec(),
            slope,
            r_squared,
            trend_direction,
        })
    }

    /// Generate trend description
    fn generate_trend_description(&self, trend: &TrendLine) -> String {
        match trend.trend_direction {
            TrendDirection::Improving => {
                if trend.r_squared > 0.8 {
                    "Strong upward trend detected".to_string()
                } else {
                    "Moderate improvement trend".to_string()
                }
            },
            TrendDirection::Degrading => {
                if trend.r_squared > 0.8 {
                    "Strong downward trend detected".to_string()
                } else {
                    "Moderate degradation trend".to_string()
                }
            },
            TrendDirection::Stable => "Performance remains stable".to_string(),
            TrendDirection::Unknown => "Trend unclear due to high variance".to_string(),
        }
    }

    /// Calculate regression severity
    fn calculate_severity(&self, deviation: f32) -> RegressionSeverity {
        let abs_deviation = deviation.abs();
        if abs_deviation > 0.5 {
            RegressionSeverity::Critical
        } else if abs_deviation > 0.25 {
            RegressionSeverity::Severe
        } else if abs_deviation > 0.1 {
            RegressionSeverity::Moderate
        } else {
            RegressionSeverity::Minor
        }
    }
}

/// Current performance metrics
#[derive(Debug, Default)]
struct CurrentMetrics {
    pub avg_scheduling_latency_ns: f32,
    pub avg_context_switch_overhead_us: f32,
    pub avg_cpu_utilization: f32,
    pub avg_fairness_index: f32,
    pub avg_responsiveness_score: f32,
    pub avg_throughput: f32,
    pub load_balancing_efficiency: f32,
    pub priority_inversions_per_hour: u32,
}

/// Trend analysis result
#[derive(Debug)]
pub struct TrendAnalysisResult {
    pub metric: String,
    pub trend: TrendDirection,
    pub slope: f32,
    pub r_squared: f32,
    pub confidence: f32,
    pub description: String,
}

impl TrendAnalysis {
    fn new() -> Self {
        Self {
            latency_trend: TrendLine {
                data_points: Vec::new(),
                slope: 0.0,
                r_squared: 0.0,
                trend_direction: TrendDirection::Unknown,
            },
            throughput_trend: TrendLine {
                data_points: Vec::new(),
                slope: 0.0,
                r_squared: 0.0,
                trend_direction: TrendDirection::Unknown,
            },
            fairness_trend: TrendLine {
                data_points: Vec::new(),
                slope: 0.0,
                r_squared: 0.0,
                trend_direction: TrendDirection::Unknown,
            },
            utilization_trend: TrendLine {
                data_points: Vec::new(),
                slope: 0.0,
                r_squared: 0.0,
                trend_direction: TrendDirection::Unknown,
            },
            seasonal_patterns: HashMap::new(),
        }
    }
}

impl RegressionDetector {
    fn new() -> Self {
        Self {
            latency_regression_threshold: 0.20,  // 20% increase
            throughput_regression_threshold: 0.15, // 15% decrease
            fairness_regression_threshold: 0.10,   // 10% decrease
            detected_regressions: Vec::new(),
        }
    }
}

impl OptimizationRecommender {
    fn new() -> Self {
        Self {
            rule_engine: OptimizationRuleEngine::new(),
            ml_recommender: None, // Could be initialized with actual ML model
        }
    }

    fn generate_recommendations(&self, metrics: &CurrentMetrics, classification: &WorkloadClassification) -> Vec<OptimizationRecommendation> {
        let mut recommendations = Vec::new();

        // Rule-based recommendations
        recommendations.extend(self.rule_engine.generate_recommendations(metrics, classification));

        // Add ML-based recommendations if available
        if let Some(ml_rec) = &self.ml_recommender {
            let ml_recommendations = ml_rec.generate_recommendations(metrics, classification);
            recommendations.extend(ml_recommendations);
        }

        // Sort by expected improvement and priority
        recommendations.sort_by(|a, b| b.expected_improvement.partial_cmp(&a.expected_improvement).unwrap_or(std::cmp::Ordering::Equal));
        recommendations
    }
}

impl OptimizationRuleEngine {
    fn new() -> Self {
        Self {
            rules: Self::create_default_rules(),
        }
    }

    fn create_default_rules() -> Vec<OptimizationRule> {
        vec![
            // High latency rule
            OptimizationRule {
                name: "high_scheduling_latency".to_string(),
                condition: OptimizationCondition {
                    metric: "avg_scheduling_latency_ns".to_string(),
                    operator: ComparisonOperator::GreaterThan,
                    threshold: 50000.0, // 50µs
                    duration: Some(Duration::from_secs(30)),
                },
                recommendation: OptimizationRecommendation {
                    title: "Reduce Scheduling Latency".to_string(),
                    description: "Increase scheduler frequency or optimize algorithm implementation".to_string(),
                    action: OptimizationAction::AdjustSchedulerQuantum { new_quantum_us: 10000 },
                    expected_improvement: 0.25,
                    risk_level: RiskLevel::Low,
                    implementation_difficulty: ImplementationDifficulty::Easy,
                },
                confidence: 0.9,
                priority: 1,
            },
            
            // Low fairness rule
            OptimizationRule {
                name: "low_fairness".to_string(),
                condition: OptimizationCondition {
                    metric: "avg_fairness_index".to_string(),
                    operator: ComparisonOperator::LessThan,
                    threshold: 0.8,
                    duration: Some(Duration::from_secs(60)),
                },
                recommendation: OptimizationRecommendation {
                    title: "Improve Fairness".to_string(),
                    description: "Enable aging mechanism to prevent thread starvation".to_string(),
                    action: OptimizationAction::EnableAging,
                    expected_improvement: 0.15,
                    risk_level: RiskLevel::Low,
                    implementation_difficulty: ImplementationDifficulty::Easy,
                },
                confidence: 0.85,
                priority: 2,
            },
            
            // High context switch overhead rule
            OptimizationRule {
                name: "high_context_switch_cost".to_string(),
                condition: OptimizationCondition {
                    metric: "avg_context_switch_overhead_us".to_string(),
                    operator: ComparisonOperator::GreaterThan,
                    threshold: 5.0, // 5µs
                    duration: Some(Duration::from_secs(30)),
                },
                recommendation: OptimizationRecommendation {
                    title: "Optimize Context Switches".to_string(),
                    description: "Increase context switch threshold to reduce unnecessary switches".to_string(),
                    action: OptimizationAction::TuneContextSwitchThreshold { new_threshold: 1000 },
                    expected_improvement: 0.20,
                    risk_level: RiskLevel::Medium,
                    implementation_difficulty: ImplementationDifficulty::Medium,
                },
                confidence: 0.8,
                priority: 3,
            },
        ]
    }

    fn generate_recommendations(&self, metrics: &CurrentMetrics, classification: &WorkloadClassification) -> Vec<OptimizationRecommendation> {
        let mut recommendations = Vec::new();

        for rule in &self.rules {
            if self.evaluate_condition(&rule.condition, metrics) {
                recommendations.push(rule.recommendation.clone());
            }
        }

        // Add workload-specific recommendations
        recommendations.extend(self.generate_workload_specific_recommendations(classification));

        recommendations
    }

    fn evaluate_condition(&self, condition: &OptimizationCondition, metrics: &CurrentMetrics) -> bool {
        let metric_value = match condition.metric.as_str() {
            "avg_scheduling_latency_ns" => metrics.avg_scheduling_latency_ns,
            "avg_context_switch_overhead_us" => metrics.avg_context_switch_overhead_us,
            "avg_cpu_utilization" => metrics.avg_cpu_utilization,
            "avg_fairness_index" => metrics.avg_fairness_index,
            "avg_responsiveness_score" => metrics.avg_responsiveness_score,
            "avg_throughput" => metrics.avg_throughput,
            "load_balancing_efficiency" => metrics.load_balancing_efficiency,
            _ => return false,
        };

        match condition.operator {
            ComparisonOperator::GreaterThan => metric_value > condition.threshold,
            ComparisonOperator::LessThan => metric_value < condition.threshold,
            ComparisonOperator::EqualTo => (metric_value - condition.threshold).abs() < f32::EPSILON,
            ComparisonOperator::GreaterThanOrEqual => metric_value >= condition.threshold,
            ComparisonOperator::LessThanOrEqual => metric_value <= condition.threshold,
        }
    }

    fn generate_workload_specific_recommendations(&self, classification: &WorkloadClassification) -> Vec<OptimizationRecommendation> {
        let mut recommendations = Vec::new();

        match classification.workload_type {
            WorkloadType::CpuBound => {
                recommendations.push(OptimizationRecommendation {
                    title: "Optimize for CPU-bound Workload".to_string(),
                    description: "Use priority-based scheduling to maximize CPU utilization".to_string(),
                    action: OptimizationAction::ChangeSchedulingAlgorithm { 
                        new_algorithm: SchedulerAlgorithm::PriorityBased 
                    },
                    expected_improvement: 0.30,
                    risk_level: RiskLevel::Medium,
                    implementation_difficulty: ImplementationDifficulty::Hard,
                });
            },
            
            WorkloadType::IoBound => {
                recommendations.push(OptimizationRecommendation {
                    title: "Optimize for I/O-bound Workload".to_string(),
                    description: "Use round-robin with aging to ensure fair I/O thread scheduling".to_string(),
                    action: OptimizationAction::ChangeSchedulingAlgorithm { 
                        new_algorithm: SchedulerAlgorithm::RoundRobin 
                    },
                    expected_improvement: 0.25,
                    risk_level: RiskLevel::Low,
                    implementation_difficulty: ImplementationDifficulty::Medium,
                });
            },
            
            WorkloadType::RealTime => {
                recommendations.push(OptimizationRecommendation {
                    title: "Optimize for Real-time Workload".to_string(),
                    description: "Use EDF scheduling for deterministic deadline guarantees".to_string(),
                    action: OptimizationAction::ChangeSchedulingAlgorithm { 
                        new_algorithm: SchedulerAlgorithm::EarliestDeadlineFirst 
                    },
                    expected_improvement: 0.40,
                    risk_level: RiskLevel::High,
                    implementation_difficulty: ImplementationDifficulty::VeryHard,
                });
            },
            
            _ => {
                // Mixed workload - use MLFQ
                recommendations.push(OptimizationRecommendation {
                    title: "Optimize for Mixed Workload".to_string(),
                    description: "Use MLFQ to adaptively handle mixed workload characteristics".to_string(),
                    action: OptimizationAction::ChangeSchedulingAlgorithm { 
                        new_algorithm: SchedulerAlgorithm::MultiLevelFeedbackQueue 
                    },
                    expected_improvement: 0.20,
                    risk_level: RiskLevel::Medium,
                    implementation_difficulty: ImplementationDifficulty::Hard,
                });
            }
        }

        recommendations
    }
}

// Placeholder implementations for ML recommender
impl MLRecommender {
    fn new() -> Self {
        Self {
            models: HashMap::new(),
            training_data: Vec::new(),
        }
    }

    fn generate_recommendations(&self, _metrics: &CurrentMetrics, _classification: &WorkloadClassification) -> Vec<OptimizationRecommendation> {
        // In real implementation, this would use trained ML models
        Vec::new()
    }
}

impl WorkloadClassifier {
    fn new() -> Self {
        Self {
            workload_database: Self::initialize_workload_database(),
            classifiers: HashMap::new(),
        }
    }

    fn initialize_workload_database() -> HashMap<String, WorkloadProfile> {
        let mut database = HashMap::new();

        // CPU-bound workload profile
        database.insert("cpu_bound".to_string(), WorkloadProfile {
            name: "CPU-bound".to_string(),
            characteristics: WorkloadCharacteristics {
                cpu_intensity: 0.9,
                thread_count: 100,
                lock_contention_level: 0.2,
                memory_access_pattern: MemoryAccessPattern::Sequential,
                communication_pattern: CommunicationPattern::None,
                deadline_sensitivity: 0.1,
                interactive_requirement: 0.2,
            },
            recommended_algorithm: SchedulerAlgorithm::PriorityBased,
            recommended_parameters: SchedulerParameters {
                time_quantum_us: 20000,
                priority_levels: 8,
                aging_enabled: false,
                inheritance_enabled: false,
                load_balance_interval_ms: 50,
                context_switch_threshold_us: 1000,
            },
            performance_expectations: PerformanceExpectations {
                target_throughput: 5000.0,
                max_scheduling_latency_us: 1000,
                min_fairness_index: 0.7,
                min_responsiveness_score: 0.6,
                max_priority_inversions_per_hour: 10,
            },
        });

        // I/O-bound workload profile
        database.insert("io_bound".to_string(), WorkloadProfile {
            name: "I/O-bound".to_string(),
            characteristics: WorkloadCharacteristics {
                cpu_intensity: 0.3,
                thread_count: 200,
                lock_contention_level: 0.4,
                memory_access_pattern: MemoryAccessPattern::Random,
                communication_pattern: CommunicationPattern::ProducerConsumer,
                deadline_sensitivity: 0.3,
                interactive_requirement: 0.7,
            },
            recommended_algorithm: SchedulerAlgorithm::RoundRobin,
            recommended_parameters: SchedulerParameters {
                time_quantum_us: 10000,
                priority_levels: 5,
                aging_enabled: true,
                inheritance_enabled: true,
                load_balance_interval_ms: 100,
                context_switch_threshold_us: 500,
            },
            performance_expectations: PerformanceExpectations {
                target_throughput: 3000.0,
                max_scheduling_latency_us: 500,
                min_fairness_index: 0.8,
                min_responsiveness_score: 0.8,
                max_priority_inversions_per_hour: 5,
            },
        });

        // Real-time workload profile
        database.insert("realtime".to_string(), WorkloadProfile {
            name: "Real-time".to_string(),
            characteristics: WorkloadCharacteristics {
                cpu_intensity: 0.7,
                thread_count: 50,
                lock_contention_level: 0.1,
                memory_access_pattern: MemoryAccessPattern::Streaming,
                communication_pattern: CommunicationPattern::MessagePassing,
                deadline_sensitivity: 0.95,
                interactive_requirement: 0.3,
            },
            recommended_algorithm: SchedulerAlgorithm::EarliestDeadlineFirst,
            recommended_parameters: SchedulerParameters {
                time_quantum_us: 5000,
                priority_levels: 10,
                aging_enabled: false,
                inheritance_enabled: true,
                load_balance_interval_ms: 20,
                context_switch_threshold_us: 200,
            },
            performance_expectations: PerformanceExpectations {
                target_throughput: 2000.0,
                max_scheduling_latency_us: 100,
                min_fairness_index: 0.6,
                min_responsiveness_score: 0.9,
                max_priority_inversions_per_hour: 1,
            },
        });

        database
    }

    /// Classify workload based on performance samples
    pub async fn classify_workload(&self, samples: &[PerformanceSample]) -> WorkloadClassification {
        if samples.is_empty() {
            return WorkloadClassification {
                workload_type: WorkloadType::Mixed,
                confidence: 0.0,
                characteristics: WorkloadCharacteristics {
                    cpu_intensity: 0.5,
                    thread_count: 0,
                    lock_contention_level: 0.5,
                    memory_access_pattern: MemoryAccessPattern::Random,
                    communication_pattern: CommunicationPattern::None,
                    deadline_sensitivity: 0.5,
                    interactive_requirement: 0.5,
                },
                recommended_algorithm: SchedulerAlgorithm::RoundRobin,
            };
        }

        // Calculate characteristics from samples
        let characteristics = self.calculate_characteristics(samples);
        let workload_type = self.determine_workload_type(&characteristics);
        let recommended_algorithm = self.select_recommended_algorithm(&characteristics);
        let confidence = self.calculate_classification_confidence(&characteristics);

        WorkloadClassification {
            workload_type,
            confidence,
            characteristics,
            recommended_algorithm,
        }
    }

    /// Calculate workload characteristics from performance samples
    fn calculate_characteristics(&self, samples: &[PerformanceSample]) -> WorkloadCharacteristics {
        let avg_cpu_utilization = samples.iter()
            .map(|s| s.cpu_utilization.iter().sum::<f32>() / s.cpu_utilization.len() as f32)
            .sum::<f32>() / samples.len() as f32;

        let avg_scheduling_latency = samples.iter()
            .map(|s| s.scheduling_latency.avg_ns)
            .sum::<f64>() / samples.len() as f64;

        // Estimate CPU intensity based on utilization and scheduling behavior
        let cpu_intensity = (avg_cpu_utilization * 0.7 + 
                           (1.0 - (avg_scheduling_latency / 100000.0).min(1.0)) * 0.3).min(1.0);

        // Estimate lock contention from scheduling patterns
        let lock_contention_level = (1.0 - (samples.iter()
            .map(|s| s.scheduling_latency.p99_ns)
            .sum::<u64>() as f64 / samples.len() as f64 / 50000.0).min(1.0);

        WorkloadCharacteristics {
            cpu_intensity,
            thread_count: 100, // Would be determined from actual thread count
            lock_contention_level,
            memory_access_pattern: MemoryAccessPattern::Random, // Simplified
            communication_pattern: CommunicationPattern::None, // Simplified
            deadline_sensitivity: 0.5, // Would need more sophisticated analysis
            interactive_requirement: avg_scheduling_latency < 100000.0 && avg_cpu_utilization < 0.8,
        }
    }

    /// Determine workload type from characteristics
    fn determine_workload_type(&self, characteristics: &WorkloadCharacteristics) -> WorkloadType {
        if characteristics.deadline_sensitivity > 0.8 {
            WorkloadType::RealTime
        } else if characteristics.cpu_intensity > 0.7 {
            WorkloadType::CpuBound
        } else if characteristics.cpu_intensity < 0.4 {
            WorkloadType::IoBound
        } else if characteristics.interactive_requirement > 0.7 {
            WorkloadType::Interactive
        } else if characteristics.lock_contention_level > 0.6 {
            WorkloadType::CommunicationIntensive
        } else {
            WorkloadType::Mixed
        }
    }

    /// Select recommended algorithm based on characteristics
    fn select_recommended_algorithm(&self, characteristics: &WorkloadCharacteristics) -> SchedulerAlgorithm {
        match self.determine_workload_type(characteristics) {
            WorkloadType::CpuBound => SchedulerAlgorithm::PriorityBased,
            WorkloadType::IoBound => SchedulerAlgorithm::RoundRobin,
            WorkloadType::RealTime => SchedulerAlgorithm::EarliestDeadlineFirst,
            WorkloadType::Interactive => SchedulerAlgorithm::MultiLevelFeedbackQueue,
            _ => SchedulerAlgorithm::RoundRobin,
        }
    }

    /// Calculate classification confidence
    fn calculate_classification_confidence(&self, _characteristics: &WorkloadCharacteristics) -> f32 {
        // Simplified confidence calculation
        0.85
    }
}

// Simple recommendation model implementations
struct LinearRecommendationModel {
    weights: Vec<f32>,
}

impl RecommendationModel for LinearRecommendationModel {
    fn predict(&self, features: &[f32]) -> Vec<f32> {
        features.iter().zip(&self.weights).map(|(f, w)| f * w).collect()
    }

    fn train(&mut self, samples: &[MLTrainingSample]) -> Result<(), Box<dyn std::error::Error>> {
        // Simple training implementation
        Ok(())
    }

    fn get_model_name(&self) -> &str {
        "linear_regression"
    }
}

struct SimpleWorkloadClassifier;

impl WorkloadClassifierModel for SimpleWorkloadClassifier {
    fn classify(&self, sample: &PerformanceSample) -> WorkloadClassification {
        let cpu_intensity = sample.cpu_utilization.iter().sum::<f32>() / sample.cpu_utilization.len() as f32;
        
        if cpu_intensity > 0.8 {
            WorkloadClassification {
                workload_type: WorkloadType::CpuBound,
                confidence: 0.8,
                characteristics: WorkloadCharacteristics {
                    cpu_intensity,
                    thread_count: 100,
                    lock_contention_level: 0.2,
                    memory_access_pattern: MemoryAccessPattern::Sequential,
                    communication_pattern: CommunicationPattern::None,
                    deadline_sensitivity: 0.1,
                    interactive_requirement: 0.2,
                },
                recommended_algorithm: SchedulerAlgorithm::PriorityBased,
            }
        } else {
            WorkloadClassification {
                workload_type: WorkloadType::IoBound,
                confidence: 0.7,
                characteristics: WorkloadCharacteristics {
                    cpu_intensity,
                    thread_count: 200,
                    lock_contention_level: 0.4,
                    memory_access_pattern: MemoryAccessPattern::Random,
                    communication_pattern: CommunicationPattern::ProducerConsumer,
                    deadline_sensitivity: 0.3,
                    interactive_requirement: 0.7,
                },
                recommended_algorithm: SchedulerAlgorithm::RoundRobin,
            }
        }
    }

    fn train(&mut self, _samples: &[PerformanceSample], _labels: &[String]) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}