//! Core type definitions for the scheduler profiler
//! 
//! This module contains all shared types and data structures used across
//! the scheduler profiling system.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Core scheduler algorithms supported by MultiOS
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SchedulerAlgorithm {
    /// Round-Robin scheduling
    RoundRobin,
    /// Priority-based scheduling
    Priority,
    /// Multi-Level Feedback Queue
    MLFQ,
    /// Earliest Deadline First
    EDF,
}

/// Lock acquisition tracking for priority inversion detection
#[derive(Debug, Clone)]
pub struct LockAcquisition {
    pub thread_id: u64,
    pub lock_address: u64,
    pub acquisition_time: Instant,
    pub priority: u8,
}

/// Fairness calculator for scheduler performance
#[derive(Debug)]
pub struct FairnessCalculator {
    /// Fairness history buffer
    history: Vec<f32>,
}

/// Load balancing analyzer
#[derive(Debug)]
pub struct LoadBalancingAnalyzer {
    /// Per-core load statistics
    core_loads: HashMap<usize, f32>,
    /// Load balancing events
    events: Vec<LoadBalancingEvent>,
}

/// Load balancing event
#[derive(Debug, Clone)]
pub struct LoadBalancingEvent {
    pub timestamp: Instant,
    pub from_core: usize,
    pub to_core: usize,
    pub thread_id: u64,
    pub load_difference: f32,
}

/// ML-based optimization recommender
pub struct MLRecommender {
    /// Model weights (placeholder)
    weights: Vec<f32>,
    /// Training data
    training_data: Vec<(Vec<f32>, f32)>,
}

/// Optimization condition for rule engine
#[derive(Debug, Clone)]
pub enum OptimizationCondition {
    /// Latency threshold exceeded
    LatencyAboveThreshold { threshold_ns: u64 },
    /// Fairness below threshold
    FairnessBelowThreshold { threshold: f32 },
    /// CPU utilization imbalance
    LoadImbalance { threshold: f32 },
    /// Priority inversions detected
    PriorityInversionsDetected { count_threshold: u32 },
    /// Throughput below target
    ThroughputBelowTarget { target: f64 },
}

/// Optimization recommendation
#[derive(Debug, Clone)]
pub struct OptimizationRecommendation {
    pub title: String,
    pub description: String,
    pub action: OptimizationAction,
    pub expected_improvement: f32,
    pub confidence: f32,
}

/// Optimization action to take
#[derive(Debug, Clone)]
pub enum OptimizationAction {
    /// Change scheduler algorithm
    ChangeAlgorithm { algorithm: SchedulerAlgorithm },
    /// Adjust algorithm parameters
    AdjustParameters { parameter: String, value: f32 },
    /// Rebalance load across cores
    RebalanceLoad,
    /// Adjust thread priorities
    AdjustPriorities { thread_id: u64, new_priority: u8 },
    /// Enable/disable specific features
    ToggleFeature { feature: String, enabled: bool },
}

/// Workload classifier for automatic configuration
#[derive(Debug)]
pub struct WorkloadClassifier {
    /// Classification history
    classifications: Vec<WorkloadClassification>,
}

/// Workload classification result
#[derive(Debug, Clone)]
pub struct WorkloadClassification {
    pub timestamp: Instant,
    pub workload_type: WorkloadType,
    pub characteristics: WorkloadCharacteristics,
    pub recommended_algorithm: SchedulerAlgorithm,
    pub confidence: f32,
}

/// Types of workloads
#[derive(Debug, Clone, PartialEq)]
pub enum WorkloadType {
    /// CPU-intensive computational tasks
    CPUIntensive,
    /// I/O-bound tasks
    IOBound,
    /// Real-time tasks with deadlines
    RealTime,
    /// Mixed workload
    Mixed,
    /// Background tasks
    Background,
}

/// Workload characteristics
#[derive(Debug, Clone)]
pub struct WorkloadCharacteristics {
    pub avg_cpu_utilization: f32,
    pub cpu_intensity: f32,
    pub io_intensity: f32,
    pub parallelism_degree: usize,
    pub priority_distribution: HashMap<u8, f32>,
    pub deadline_sensitivity: f32,
}