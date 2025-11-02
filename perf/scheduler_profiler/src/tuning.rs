//! Automated Tuning Module
//! 
//! Dynamic scheduler optimization and tuning system including:
//! - Real-time parameter adjustment
//! - Adaptive algorithm selection
//! - Performance-driven tuning decisions
//! - Risk-aware optimization

use crate::*;
use std::time::{Duration, Instant};
use std::collections::{HashMap, VecDeque};

/// Automated tuning system for scheduler optimization
pub struct AutoTuner {
    config: ProfilerConfig,
    /// Current tuning parameters
    current_parameters: TunableParameters,
    /// Tuning history
    tuning_history: Vec<TuningAction>,
    /// Performance baseline for comparison
    performance_baseline: Option<PerformanceBaseline>,
    /// Risk assessment engine
    risk_assessor: RiskAssessor,
    /// Convergence detector
    convergence_detector: ConvergenceDetector,
}

/// Tunable scheduler parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunableParameters {
    pub scheduler_algorithm: SchedulerAlgorithm,
    pub time_quantum_us: u64,
    pub priority_levels: u8,
    pub aging_enabled: bool,
    pub priority_inheritance_enabled: bool,
    pub load_balance_interval_ms: u64,
    pub context_switch_threshold_us: u64,
    pub cpu_affinity_strict: bool,
    pub numa_aware_scheduling: bool,
    pub real_time_priority_boost: bool,
}

/// Tuning action record
#[derive(Debug, Clone)]
pub struct TuningAction {
    pub timestamp: Instant,
    pub parameter: String,
    pub old_value: String,
    pub new_value: String,
    pub reason: TuningReason,
    pub expected_impact: f32,
    pub actual_impact: f32,
    pub success: bool,
    pub rollback_applied: bool,
}

/// Reasons for tuning actions
#[derive(Debug, Clone)]
pub enum TuningReason {
    PerformanceRegression,
    LoadImbalance,
    FairnessDegradation,
    LatencyIncrease,
    ThroughputOptimization,
    EnergyOptimization,
    WorkloadChange,
    ManualRequest,
}

/// Risk assessment engine for tuning decisions
pub struct RiskAssessor {
    /// Risk models for different parameters
    risk_models: HashMap<String, RiskModel>,
    /// Historical tuning outcomes
    tuning_outcomes: VecDeque<TuningOutcome>,
}

/// Risk model for parameter tuning
#[derive(Debug, Clone)]
pub struct RiskModel {
    pub parameter: String,
    pub risk_factors: Vec<RiskFactor>,
    pub mitigation_strategies: Vec<MitigationStrategy>,
}

/// Risk factors affecting tuning decisions
#[derive(Debug, Clone)]
pub struct RiskFactor {
    pub name: String,
    pub weight: f32,
    pub threshold: f32,
    pub probability: f32,
}

/// Mitigation strategies for identified risks
#[derive(Debug, Clone)]
pub struct MitigationStrategy {
    pub description: String,
    pub implementation_cost: f32,
    pub effectiveness: f32,
}

/// Tuning outcome record
#[derive(Debug, Clone)]
pub struct TuningOutcome {
    pub action: TuningAction,
    pub performance_before: f32,
    pub performance_after: f32,
    pub stability_score: f32,
    pub user_satisfaction: f32,
    pub system_stability: SystemStability,
}

/// System stability assessment
#[derive(Debug, Clone, PartialEq)]
pub enum SystemStability {
    Stable,
    SlightlyUnstable,
    ModeratelyUnstable,
    HighlyUnstable,
    Critical,
}

/// Convergence detection for tuning optimization
pub struct ConvergenceDetector {
    /// Performance convergence threshold
    convergence_threshold: f32,
    /// Convergence history
    convergence_history: VecDeque<f32>,
    /// Best known configuration
    best_configuration: Option<TunableParameters>,
    /// Optimization iterations
    iterations: u32,
    /// Convergence status
    converged: bool,
}

impl AutoTuner {
    /// Create new auto-tuner
    pub fn new(config: ProfilerConfig) -> Self {
        let default_parameters = TunableParameters::default();
        
        Self {
            config,
            current_parameters: default_parameters.clone(),
            tuning_history: Vec::new(),
            performance_baseline: None,
            risk_assessor: RiskAssessor::new(),
            convergence_detector: ConvergenceDetector::new(default_parameters),
        }
    }

    /// Analyze performance and apply optimizations
    pub async fn analyze_and_tune(&mut self, samples: &[PerformanceSample]) {
        if samples.len() < 10 {
            return; // Need sufficient data for analysis
        }

        // Calculate current performance metrics
        let current_metrics = self.calculate_performance_metrics(samples);
        
        // Check if tuning is needed
        if self.should_tune(&current_metrics).await {
            // Assess risks for potential tuning actions
            let tuning_candidates = self.identify_tuning_candidates(&current_metrics).await;
            
            // Evaluate and select best tuning action
            if let Some(selected_action) = self.select_tuning_action(tuning_candidates, &current_metrics).await {
                // Apply the tuning action
                self.apply_tuning_action(selected_action.clone()).await;
                
                // Record the action in history
                self.tuning_history.push(TuningAction {
                    timestamp: Instant::now(),
                    parameter: selected_action.parameter.clone(),
                    old_value: selected_action.old_value.clone(),
                    new_value: selected_action.new_value.clone(),
                    reason: selected_action.reason.clone(),
                    expected_impact: selected_action.expected_impact,
                    actual_impact: 0.0, // Will be updated later
                    success: false, // Will be determined after observation period
                    rollback_applied: false,
                });
            }
        }

        // Check for convergence
        self.convergence_detector.check_convergence(&current_metrics);
        
        // Update performance baseline if significant changes detected
        if self.should_update_baseline(&current_metrics).await {
            self.update_performance_baseline(samples).await;
        }
    }

    /// Calculate comprehensive performance metrics
    fn calculate_performance_metrics(&self, samples: &[PerformanceSample]) -> CurrentTuningMetrics {
        if samples.is_empty() {
            return CurrentTuningMetrics::default();
        }

        let mut total_latency = 0.0f64;
        let mut total_throughput = 0.0f64;
        let mut total_fairness = 0.0f32;
        let mut total_responsiveness = 0.0f32;
        let mut total_cpu_util = 0.0f32;
        let mut total_load_balance = 0.0f32;

        for sample in samples {
            total_latency += sample.scheduling_latency.avg_ns as f64;
            total_throughput += sample.throughput;
            total_fairness += sample.fairness_index;
            total_responsiveness += sample.responsiveness_score;
            total_cpu_util += sample.cpu_utilization.iter().sum::<f32>() / sample.cpu_utilization.len() as f32;
            total_load_balance += sample.load_balancing_efficiency;
        }

        let count = samples.len() as f32;

        CurrentTuningMetrics {
            avg_scheduling_latency_ms: (total_latency / count as f64) / 1_000_000.0,
            avg_throughput: total_throughput as f32 / count,
            avg_fairness_index: total_fairness / count,
            avg_responsiveness_score: total_responsiveness / count,
            avg_cpu_utilization: total_cpu_util / count,
            load_balancing_efficiency: total_load_balance / count,
            performance_score: self.calculate_overall_performance_score(samples),
            stability_score: self.calculate_stability_score(samples),
        }
    }

    /// Calculate overall performance score (0-1)
    fn calculate_overall_performance_score(&self, samples: &[PerformanceSample]) -> f32 {
        if samples.is_empty() {
            return 0.0;
        }

        // Weighted scoring of key metrics
        let latency_score = self.calculate_latency_score(samples);
        let throughput_score = self.calculate_throughput_score(samples);
        let fairness_score = self.calculate_fairness_score(samples);
        let responsiveness_score = self.calculate_responsiveness_score(samples);
        let utilization_score = self.calculate_utilization_score(samples);

        // Weight the components (sum should equal 1.0)
        let weights = [0.25, 0.25, 0.2, 0.2, 0.1];
        let scores = [latency_score, throughput_score, fairness_score, responsiveness_score, utilization_score];
        
        scores.iter().zip(weights.iter()).map(|(s, w)| s * w).sum()
    }

    /// Calculate latency score (higher is better)
    fn calculate_latency_score(&self, samples: &[PerformanceSample]) -> f32 {
        let avg_p99_latency = samples.iter()
            .map(|s| s.scheduling_latency.p99_ns as f32)
            .sum::<f32>() / samples.len() as f32;
        
        // Normalize: 100µs = 1.0, 1ms = 0.1, 10ms = 0.01
        (100000.0 / avg_p99_latency).min(1.0).max(0.0)
    }

    /// Calculate throughput score (higher is better)
    fn calculate_throughput_score(&self, samples: &[PerformanceSample]) -> f32 {
        let avg_throughput = samples.iter()
            .map(|s| s.throughput)
            .sum::<f64>() / samples.len() as f64;
        
        // Normalize: assume 10,000 tasks/sec = 1.0
        (avg_throughput / 10000.0).min(1.0).max(0.0)
    }

    /// Calculate fairness score
    fn calculate_fairness_score(&self, samples: &[PerformanceSample]) -> f32 {
        let avg_fairness = samples.iter()
            .map(|s| s.fairness_index)
            .sum::<f32>() / samples.len() as f32;
        
        avg_fairness // Already normalized 0-1
    }

    /// Calculate responsiveness score
    fn calculate_responsiveness_score(&self, samples: &[PerformanceSample]) -> f32 {
        let avg_responsiveness = samples.iter()
            .map(|s| s.responsiveness_score)
            .sum::<f32>() / samples.len() as f32;
        
        avg_responsiveness // Already normalized 0-1
    }

    /// Calculate CPU utilization score (optimal around 80%)
    fn calculate_utilization_score(&self, samples: &[PerformanceSample]) -> f32 {
        let avg_utilization = samples.iter()
            .map(|s| s.cpu_utilization.iter().sum::<f32>() / s.cpu_utilization.len() as f32)
            .sum::<f32>() / samples.len() as f32;
        
        // Optimal utilization is around 80%
        let optimal_utilization = 0.8;
        let deviation = (avg_utilization - optimal_utilization).abs();
        
        (1.0 - deviation).max(0.0) // Score decreases as utilization moves away from optimal
    }

    /// Calculate stability score based on metric variance
    fn calculate_stability_score(&self, samples: &[PerformanceSample]) -> f32 {
        if samples.len() < 2 {
            return 1.0;
        }

        // Calculate coefficient of variation for key metrics
        let throughput_values: Vec<f32> = samples.iter().map(|s| s.throughput as f32).collect();
        let latency_values: Vec<f32> = samples.iter().map(|s| s.scheduling_latency.avg_ns as f32).collect();
        
        let throughput_cv = self.calculate_coefficient_of_variation(&throughput_values);
        let latency_cv = self.calculate_coefficient_of_variation(&latency_values);
        
        // Lower CV = higher stability
        let avg_cv = (throughput_cv + latency_cv) / 2.0;
        
        (1.0 / (1.0 + avg_cv)).max(0.0)
    }

    /// Calculate coefficient of variation
    fn calculate_coefficient_of_variation(&self, values: &[f32]) -> f32 {
        if values.is_empty() {
            return 0.0;
        }

        let mean = values.iter().sum::<f32>() / values.len() as f32;
        let variance = values.iter()
            .map(|&v| (v - mean).powi(2))
            .sum::<f32>() / values.len() as f32;
        let std_dev = variance.sqrt();
        
        if mean.abs() < f32::EPSILON {
            return 0.0;
        }
        
        std_dev / mean.abs()
    }

    /// Determine if tuning is needed
    async fn should_tune(&self, metrics: &CurrentTuningMetrics) -> bool {
        // Don't tune if performance is already excellent
        if metrics.performance_score > 0.9 {
            return false;
        }

        // Don't tune if unstable
        if metrics.stability_score < 0.7 {
            return false;
        }

        // Tune if any critical metric is poor
        if metrics.avg_fairness_index < 0.7 || 
           metrics.avg_responsiveness_score < 0.6 ||
           metrics.load_balancing_efficiency < 0.7 {
            return true;
        }

        // Tune if performance is significantly below baseline
        if let Some(baseline) = &self.performance_baseline {
            let baseline_score = self.calculate_baseline_score(baseline);
            if metrics.performance_score < baseline_score * 0.9 {
                return true;
            }
        }

        false
    }

    /// Identify potential tuning candidates
    async fn identify_tuning_candidates(&self, metrics: &CurrentTuningMetrics) -> Vec<TuningCandidate> {
        let mut candidates = Vec::new();

        // Low fairness - consider enabling aging
        if metrics.avg_fairness_index < 0.8 {
            candidates.push(TuningCandidate {
                parameter: "aging_enabled".to_string(),
                current_value: self.current_parameters.aging_enabled.to_string(),
                proposed_value: (!self.current_parameters.aging_enabled).to_string(),
                expected_improvement: 0.15,
                risk_level: RiskLevel::Low,
                reason: TuningReason::FairnessDegradation,
                confidence: 0.8,
            });
        }

        // High latency - reduce time quantum
        if metrics.avg_scheduling_latency_ms > 0.1 {
            let new_quantum = (self.current_parameters.time_quantum_us / 2).max(5000);
            candidates.push(TuningCandidate {
                parameter: "time_quantum_us".to_string(),
                current_value: self.current_parameters.time_quantum_us.to_string(),
                proposed_value: new_quantum.to_string(),
                expected_improvement: 0.20,
                risk_level: RiskLevel::Medium,
                reason: TuningReason::LatencyIncrease,
                confidence: 0.75,
            });
        }

        // Poor load balancing - adjust balancing interval
        if metrics.load_balancing_efficiency < 0.8 {
            let new_interval = (self.current_parameters.load_balance_interval_ms / 2).max(20);
            candidates.push(TuningCandidate {
                parameter: "load_balance_interval_ms".to_string(),
                current_value: self.current_parameters.load_balance_interval_ms.to_string(),
                proposed_value: new_interval.to_string(),
                expected_improvement: 0.12,
                risk_level: RiskLevel::Low,
                reason: TuningReason::LoadImbalance,
                confidence: 0.7,
            });
        }

        // Algorithm change recommendations based on performance patterns
        let algorithm_recommendation = self.recommend_algorithm_change(metrics);
        if let Some(algorithm) = algorithm_recommendation {
            candidates.push(TuningCandidate {
                parameter: "scheduler_algorithm".to_string(),
                current_value: format!("{:?}", self.current_parameters.scheduler_algorithm),
                proposed_value: format!("{:?}", algorithm),
                expected_improvement: 0.25,
                risk_level: RiskLevel::High,
                reason: TuningReason::ThroughputOptimization,
                confidence: 0.6,
            });
        }

        candidates.sort_by(|a, b| b.expected_improvement.partial_cmp(&a.expected_improvement).unwrap_or(std::cmp::Ordering::Equal));
        candidates
    }

    /// Recommend algorithm change based on current performance
    fn recommend_algorithm_change(&self, metrics: &CurrentTuningMetrics) -> Option<SchedulerAlgorithm> {
        // High CPU utilization with low fairness -> Priority-based
        if metrics.avg_cpu_utilization > 0.8 && metrics.avg_fairness_index < 0.7 {
            if self.current_parameters.scheduler_algorithm != SchedulerAlgorithm::PriorityBased {
                return Some(SchedulerAlgorithm::PriorityBased);
            }
        }

        // High latency with moderate utilization -> MLFQ
        if metrics.avg_scheduling_latency_ms > 0.05 && metrics.avg_throughput < 8000.0 {
            if self.current_parameters.scheduler_algorithm != SchedulerAlgorithm::MultiLevelFeedbackQueue {
                return Some(SchedulerAlgorithm::MultiLevelFeedbackQueue);
            }
        }

        // Real-time workload characteristics -> EDF
        if metrics.avg_responsiveness_score > 0.8 && metrics.avg_cpu_utilization < 0.7 {
            if self.current_parameters.scheduler_algorithm != SchedulerAlgorithm::EarliestDeadlineFirst {
                return Some(SchedulerAlgorithm::EarliestDeadlineFirst);
            }
        }

        None
    }

    /// Select the best tuning action
    async fn select_tuning_action(&self, candidates: Vec<TuningCandidate>, _metrics: &CurrentTuningMetrics) -> Option<TuningAction> {
        if candidates.is_empty() {
            return None;
        }

        // Select the highest-scoring candidate that passes risk assessment
        for candidate in candidates {
            // Risk assessment
            if self.risk_assessor.assess_risk(&candidate).await {
                return Some(TuningAction {
                    timestamp: Instant::now(),
                    parameter: candidate.parameter,
                    old_value: candidate.current_value,
                    new_value: candidate.proposed_value,
                    reason: candidate.reason,
                    expected_impact: candidate.expected_improvement,
                    actual_impact: 0.0,
                    success: false,
                    rollback_applied: false,
                });
            }
        }

        None
    }

    /// Apply tuning action to scheduler parameters
    async fn apply_tuning_action(&mut self, action: TuningAction) {
        println!("Applying tuning action: {} = {} (reason: {:?})", 
                 action.parameter, action.new_value, action.reason);

        // Apply the parameter change
        match action.parameter.as_str() {
            "aging_enabled" => {
                if action.new_value.parse::<bool>().unwrap_or(false) {
                    self.current_parameters.aging_enabled = true;
                } else {
                    self.current_parameters.aging_enabled = false;
                }
            },
            "time_quantum_us" => {
                if let Ok(quantum) = action.new_value.parse::<u64>() {
                    self.current_parameters.time_quantum_us = quantum;
                }
            },
            "load_balance_interval_ms" => {
                if let Ok(interval) = action.new_value.parse::<u64>() {
                    self.current_parameters.load_balance_interval_ms = interval;
                }
            },
            "scheduler_algorithm" => {
                if let Ok(algorithm) = action.new_value.parse::<SchedulerAlgorithm>() {
                    self.current_parameters.scheduler_algorithm = algorithm;
                }
            },
            _ => {
                println!("Unknown parameter for tuning: {}", action.parameter);
                return;
            }
        }

        // In real implementation, this would communicate with the actual scheduler
        // to apply the parameter changes
        self.notify_scheduler_parameter_change(&action).await;
    }

    /// Notify scheduler of parameter change
    async fn notify_scheduler_parameter_change(&self, action: &TuningAction) {
        // In real implementation, this would:
        // 1. Send parameter change to scheduler via IPC
        // 2. Validate the change is applied
        // 3. Handle any errors or rollback if needed
        
        println!("Notified scheduler of parameter change: {:?}", action);
        
        // Simulate notification delay
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    /// Update performance baseline
    async fn update_performance_baseline(&mut self, samples: &[PerformanceSample]) {
        let metrics = self.calculate_performance_metrics(samples);
        
        self.performance_baseline = Some(PerformanceBaseline {
            name: format!("baseline_{}", Instant::now().elapsed().as_secs()),
            algorithm: self.current_parameters.scheduler_algorithm,
            core_count: self.config.core_count,
            metrics: BaselineMetrics {
                avg_scheduling_latency_ns: (metrics.avg_scheduling_latency_ms * 1000.0) as u64,
                avg_context_switch_overhead_us: 5.0, // Would be measured
                avg_cpu_utilization: metrics.avg_cpu_utilization,
                avg_fairness_index: metrics.avg_fairness_index,
                avg_responsiveness_score: metrics.avg_responsiveness_score,
                avg_throughput: metrics.avg_throughput as f64,
                load_balancing_efficiency: metrics.load_balancing_efficiency,
                priority_inversions_per_hour: 0, // Would be counted
            },
            created_at: Instant::now(),
            valid_until: Instant::now() + Duration::from_secs(3600), // Valid for 1 hour
            confidence_level: 0.8,
        });
    }

    /// Check if baseline should be updated
    async fn should_update_baseline(&self, metrics: &CurrentTuningMetrics) -> bool {
        if let Some(baseline) = &self.performance_baseline {
            // Update baseline if it's expired
            if Instant::now() > baseline.valid_until {
                return true;
            }

            // Update baseline if performance has significantly improved
            let current_score = metrics.performance_score;
            let baseline_score = self.calculate_baseline_score(baseline);
            
            if current_score > baseline_score * 1.2 {
                return true;
            }
        } else {
            return true;
        }

        false
    }

    /// Calculate baseline score
    fn calculate_baseline_score(&self, baseline: &PerformanceBaseline) -> f32 {
        // Simplified score calculation based on baseline metrics
        (baseline.metrics.avg_fairness_index * 0.3 +
         baseline.metrics.avg_responsiveness_score * 0.3 +
         (baseline.metrics.avg_throughput as f32 / 10000.0).min(1.0) * 0.4)
    }

    /// Get current tuning parameters
    pub fn get_current_parameters(&self) -> &TunableParameters {
        &self.current_parameters
    }

    /// Get tuning history
    pub fn get_tuning_history(&self) -> &[TuningAction] {
        &self.tuning_history
    }

    /// Get convergence status
    pub fn get_convergence_status(&self) -> ConvergenceStatus {
        self.convergence_detector.get_status()
    }
}

/// Tuning candidate for evaluation
#[derive(Debug, Clone)]
struct TuningCandidate {
    parameter: String,
    current_value: String,
    proposed_value: String,
    expected_improvement: f32,
    risk_level: RiskLevel,
    reason: TuningReason,
    confidence: f32,
}

/// Current tuning metrics
#[derive(Debug, Default)]
struct CurrentTuningMetrics {
    pub avg_scheduling_latency_ms: f64,
    pub avg_throughput: f32,
    pub avg_fairness_index: f32,
    pub avg_responsiveness_score: f32,
    pub avg_cpu_utilization: f32,
    pub load_balancing_efficiency: f32,
    pub performance_score: f32,
    pub stability_score: f32,
}

impl TunableParameters {
    fn default() -> Self {
        Self {
            scheduler_algorithm: SchedulerAlgorithm::RoundRobin,
            time_quantum_us: 20000,
            priority_levels: 5,
            aging_enabled: false,
            priority_inheritance_enabled: true,
            load_balance_interval_ms: 100,
            context_switch_threshold_us: 1000,
            cpu_affinity_strict: false,
            numa_aware_scheduling: true,
            real_time_priority_boost: false,
        }
    }
}

impl RiskAssessor {
    fn new() -> Self {
        Self {
            risk_models: Self::initialize_risk_models(),
            tuning_outcomes: VecDeque::with_capacity(100),
        }
    }

    fn initialize_risk_models() -> HashMap<String, RiskModel> {
        let mut models = HashMap::new();

        // Algorithm change risk model
        models.insert("scheduler_algorithm".to_string(), RiskModel {
            parameter: "scheduler_algorithm".to_string(),
            risk_factors: vec![
                RiskFactor {
                    name: "complexity_increase".to_string(),
                    weight: 0.4,
                    threshold: 0.5,
                    probability: 0.3,
                },
                RiskFactor {
                    name: "backward_compatibility".to_string(),
                    weight: 0.3,
                    threshold: 0.2,
                    probability: 0.1,
                },
            ],
            mitigation_strategies: vec![
                MitigationStrategy {
                    description: "Gradual algorithm transition".to_string(),
                    implementation_cost: 0.3,
                    effectiveness: 0.8,
                },
                MitigationStrategy {
                    description: "Rollback capability".to_string(),
                    implementation_cost: 0.1,
                    effectiveness: 0.9,
                },
            ],
        });

        // Time quantum risk model
        models.insert("time_quantum_us".to_string(), RiskModel {
            parameter: "time_quantum_us".to_string(),
            risk_factors: vec![
                RiskFactor {
                    name: "context_switch_overhead".to_string(),
                    weight: 0.5,
                    threshold: 0.3,
                    probability: 0.4,
                },
            ],
            mitigation_strategies: vec![
                MitigationStrategy {
                    description: "Gradual adjustment".to_string(),
                    implementation_cost: 0.2,
                    effectiveness: 0.7,
                },
            ],
        });

        models
    }

    /// Assess risk of proposed tuning action
    async fn assess_risk(&self, candidate: &TuningCandidate) -> bool {
        if let Some(model) = self.risk_models.get(&candidate.parameter) {
            // Calculate overall risk score
            let total_risk_score: f32 = model.risk_factors
                .iter()
                .map(|factor| factor.weight * factor.probability)
                .sum();

            // Consider risk tolerance based on candidate risk level
            let max_acceptable_risk = match candidate.risk_level {
                RiskLevel::Low => 0.3,
                RiskLevel::Medium => 0.2,
                RiskLevel::High => 0.1,
                RiskLevel::Critical => 0.05,
            };

            // Check if risk is acceptable
            if total_risk_score <= max_acceptable_risk {
                return true;
            }
        } else {
            // No risk model - assume low risk
            return true;
        }

        false
    }
}

impl ConvergenceDetector {
    fn new(initial_parameters: TunableParameters) -> Self {
        Self {
            convergence_threshold: 0.02, // 2% improvement threshold
            convergence_history: VecDeque::with_capacity(50),
            best_configuration: Some(initial_parameters),
            iterations: 0,
            converged: false,
        }
    }

    /// Check for convergence
    fn check_convergence(&mut self, metrics: &CurrentTuningMetrics) {
        self.iterations += 1;
        
        // Add current performance score to history
        if self.convergence_history.len() >= 10 {
            self.convergence_history.pop_front();
        }
        self.convergence_history.push_back(metrics.performance_score);

        // Check convergence only after sufficient history
        if self.convergence_history.len() >= 10 {
            let recent_scores: Vec<f32> = self.convergence_history.iter().rev().take(5).cloned().collect();
            
            if recent_scores.len() >= 2 {
                let improvement = recent_scores[0] - recent_scores.last().copied().unwrap_or(0.0);
                
                // Check if improvement is below threshold for multiple iterations
                if improvement < self.convergence_threshold {
                    self.converged = true;
                }
            }
        }

        // Update best configuration if current is better
        if let Some(best) = &self.best_configuration {
            if metrics.performance_score > 0.9 {
                // Current performance is excellent - may need to update baseline
            }
        }
    }

    /// Get convergence status
    fn get_status(&self) -> ConvergenceStatus {
        if self.converged {
            ConvergenceStatus::Converged
        } else if self.convergence_history.len() < 5 {
            ConvergenceStatus::InsufficientData
        } else if self.iterations > 100 {
            ConvergenceStatus::MaxIterationsReached
        } else {
            ConvergenceStatus::Optimizing
        }
    }
}

/// Convergence status
#[derive(Debug, Clone)]
pub enum ConvergenceStatus {
    Optimizing,
    Converged,
    InsufficientData,
    MaxIterationsReached,
}