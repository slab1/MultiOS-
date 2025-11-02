//! Performance monitoring for educational ML workloads
//! 
//! Provides comprehensive performance tracking and analysis for ML operations
//! with educational insights and optimization recommendations.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

/// ML Performance Monitor
/// 
/// Tracks performance metrics for ML operations with educational features:
/// - Operation timing and profiling
/// - Memory usage analysis
/// - Bottleneck identification
/// - Educational optimization suggestions
pub struct MLPerformanceMonitor {
    operation_metrics: HashMap<String, OperationMetrics>,
    timeline: VecDeque<TimelineEvent>,
    memory_peak: usize,
    current_memory: usize,
    educational_insights: EducationalInsights,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OperationMetrics {
    pub operation_type: String,
    pub call_count: usize,
    pub total_duration: Duration,
    pub min_duration: Duration,
    pub max_duration: Duration,
    pub average_duration: Duration,
    pub memory_usage_mb: usize,
    pub success_rate: f32,
    pub educational_notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TimelineEvent {
    pub timestamp: Instant,
    pub event_type: String,
    pub operation: String,
    pub duration: Option<Duration>,
    pub memory_before: usize,
    pub memory_after: usize,
    pub educational_context: String,
}

#[derive(Debug, Clone, Default)]
struct EducationalInsights {
    pub bottlenecks: Vec<Bottleneck>,
    pub optimization_suggestions: Vec<OptimizationSuggestion>,
    pub learning_objectives: Vec<String>,
    pub common_mistakes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceStats {
    pub total_operations: usize,
    pub total_execution_time: Duration,
    pub average_operation_time: Duration,
    pub peak_memory_usage_mb: usize,
    pub current_memory_usage_mb: usize,
    pub slowest_operations: Vec<OperationSummary>,
    pub memory_intensive_operations: Vec<OperationSummary>,
    pub educational_metrics: EducationalMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationSummary {
    pub operation_type: String,
    pub count: usize,
    pub total_time: Duration,
    pub average_time: Duration,
    pub memory_usage_mb: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EducationalMetrics {
    pub learning_efficiency_score: f32,
    pub memory_usage_score: f32,
    pub operation_optimization_score: f32,
    pub overall_educational_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bottleneck {
    pub operation: String,
    pub severity: BottleneckSeverity,
    pub impact_score: f32,
    pub description: String,
    pub suggested_optimizations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BottleneckSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationSuggestion {
    pub category: String,
    pub priority: OptimizationPriority,
    pub description: String,
    pub implementation_difficulty: String,
    pub expected_improvement: String,
    pub educational_benefit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationPriority {
    Low,
    Medium,
    High,
    Critical,
}

impl MLPerformanceMonitor {
    /// Create a new performance monitor
    pub fn new() -> Self {
        Self {
            operation_metrics: HashMap::new(),
            timeline: VecDeque::new(),
            memory_peak: 0,
            current_memory: 0,
            educational_insights: EducationalInsights::default(),
        }
    }

    /// Record an operation execution
    pub fn record_operation(
        &mut self,
        operation_type: &str,
        start_time: Instant,
        end_time: Instant,
        memory_before_mb: usize,
        memory_after_mb: usize,
        success: bool,
        educational_context: &str,
    ) {
        let duration = end_time.duration_since(start_time);
        
        // Update timeline
        self.timeline.push_back(TimelineEvent {
            timestamp: end_time,
            event_type: "operation_complete".to_string(),
            operation: operation_type.to_string(),
            Some(duration),
            memory_before: memory_before_mb,
            memory_after: memory_after_mb,
            educational_context: educational_context.to_string(),
        });

        // Keep timeline manageable (last 1000 events)
        if self.timeline.len() > 1000 {
            self.timeline.pop_front();
        }

        // Update operation metrics
        let metrics = self.operation_metrics
            .entry(operation_type.to_string())
            .or_insert_with(|| OperationMetrics {
                operation_type: operation_type.to_string(),
                call_count: 0,
                total_duration: Duration::from_nanos(0),
                min_duration: Duration::from_nanos(u64::MAX),
                max_duration: Duration::from_nanos(0),
                average_duration: Duration::from_nanos(0),
                memory_usage_mb: 0,
                success_rate: 0.0,
                educational_notes: Vec::new(),
            });

        metrics.call_count += 1;
        metrics.total_duration += duration;
        
        if duration < metrics.min_duration {
            metrics.min_duration = duration;
        }
        if duration > metrics.max_duration {
            metrics.max_duration = duration;
        }
        
        metrics.average_duration = Duration::from_nanos(
            metrics.total_duration.as_nanos() as u64 / metrics.call_count as u64
        );

        // Update memory tracking
        self.current_memory = memory_after_mb;
        if memory_after_mb > self.memory_peak {
            self.memory_peak = memory_after_mb;
        }

        // Update success rate (simplified)
        if success {
            metrics.success_rate = (metrics.success_rate * (metrics.call_count - 1) as f32 + 1.0) / metrics.call_count as f32;
        } else {
            metrics.success_rate = (metrics.success_rate * (metrics.call_count - 1) as f32) / metrics.call_count as f32;
        }

        // Add educational insights
        self.update_educational_insights(operation_type, duration, success);
    }

    /// Record execution for a program
    pub fn record_execution(
        &mut self,
        program_id: &str,
        execution_time: Duration,
        memory_usage_mb: usize,
    ) {
        // Record overall program execution
        self.timeline.push_back(TimelineEvent {
            timestamp: Instant::now(),
            event_type: "program_complete".to_string(),
            operation: format!("program_{}", program_id),
            Some(execution_time),
            memory_before: 0,
            memory_after: memory_usage_mb,
            educational_context: format!("Program {} completed in {:?}", program_id, execution_time),
        });
    }

    /// Get comprehensive performance statistics
    pub fn get_stats(&self) -> PerformanceStats {
        let total_operations = self.operation_metrics.values().map(|m| m.call_count).sum();
        let total_execution_time = self.operation_metrics
            .values()
            .map(|m| m.total_duration)
            .fold(Duration::from_nanos(0), |acc, d| acc + d);
        
        let average_operation_time = if total_operations > 0 {
            Duration::from_nanos(total_execution_time.as_nanos() as u64 / total_operations as u64)
        } else {
            Duration::from_nanos(0)
        };

        // Generate operation summaries
        let mut operation_summaries: Vec<OperationSummary> = self.operation_metrics
            .values()
            .map(|m| OperationSummary {
                operation_type: m.operation_type.clone(),
                count: m.call_count,
                total_time: m.total_duration,
                average_time: m.average_duration,
                memory_usage_mb: m.memory_usage_mb,
            })
            .collect();

        // Sort for top lists
        operation_summaries.sort_by(|a, b| b.total_time.cmp(&a.total_time));
        let slowest_operations = operation_summaries.iter().take(5).cloned().collect();

        operation_summaries.sort_by(|a, b| b.memory_usage_mb.cmp(&a.memory_usage_mb));
        let memory_intensive_operations = operation_summaries.iter().take(5).cloned().collect();

        // Calculate educational metrics
        let educational_metrics = self.calculate_educational_metrics();

        PerformanceStats {
            total_operations,
            total_execution_time,
            average_operation_time,
            peak_memory_usage_mb: self.memory_peak,
            current_memory_usage_mb: self.current_memory,
            slowest_operations,
            memory_intensive_operations,
            educational_metrics,
        }
    }

    /// Get performance report for educational purposes
    pub fn get_educational_report(&self) -> EducationalPerformanceReport {
        let stats = self.get_stats();
        let insights = &self.educational_insights;

        EducationalPerformanceReport {
            performance_summary: stats,
            bottlenecks: insights.bottlenecks.clone(),
            optimization_suggestions: insights.optimization_suggestions.clone(),
            learning_objectives: insights.learning_objectives.clone(),
            common_mistakes: insights.common_mistakes.clone(),
            timeline_summary: self.summarize_timeline(),
        }
    }

    /// Get bottleneck analysis
    pub fn get_bottleneck_analysis(&self) -> Vec<Bottleneck> {
        self.educational_insights.bottlenecks.clone()
    }

    /// Get optimization recommendations
    pub fn get_optimization_recommendations(&self) -> Vec<OptimizationSuggestion> {
        self.educational_insights.optimization_suggestions.clone()
    }

    /// Get operation timeline for visualization
    pub fn get_timeline(&self, limit: Option<usize>) -> Vec<TimelineEvent> {
        let limit = limit.unwrap_or(100);
        self.timeline.iter()
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }

    /// Clear performance data for new session
    pub fn reset(&mut self) {
        self.operation_metrics.clear();
        self.timeline.clear();
        self.memory_peak = 0;
        self.current_memory = 0;
        self.educational_insights = EducationalInsights::default();
    }

    /// Update educational insights based on current metrics
    fn update_educational_insights(&mut self, operation_type: &str, duration: Duration, success: bool) {
        // Identify slow operations
        if duration.as_millis() > 100 {
            let bottleneck = Bottleneck {
                operation: operation_type.to_string(),
                severity: if duration.as_millis() > 500 { BottleneckSeverity::High } else { BottleneckSeverity::Medium },
                impact_score: duration.as_millis() as f32 / 1000.0,
                description: format!("Operation {} took {:?}, which may indicate a performance issue", operation_type, duration),
                suggested_optimizations: self.get_optimization_suggestions(operation_type),
            };
            
            // Avoid duplicates
            if !self.educational_insights.bottlenecks.iter().any(|b| b.operation == bottleneck.operation) {
                self.educational_insights.bottlenecks.push(bottleneck);
            }
        }

        // Generate optimization suggestions based on patterns
        if let Some(metrics) = self.operation_metrics.get(operation_type) {
            if metrics.call_count > 10 {
                let suggestion = OptimizationSuggestion {
                    category: "Performance".to_string(),
                    priority: OptimizationPriority::Medium,
                    description: format!("{} has been called {} times. Consider caching or optimization.", 
                                       operation_type, metrics.call_count),
                    implementation_difficulty: "Medium".to_string(),
                    expected_improvement: "20-30%".to_string(),
                    educational_benefit: "Understanding performance optimization in ML".to_string(),
                };
                
                if !self.educational_insights.optimization_suggestions.iter().any(|s| s.description.contains(operation_type)) {
                    self.educational_insights.optimization_suggestions.push(suggestion);
                }
            }
        }

        // Add learning objectives based on operation types
        self.add_learning_objectives(operation_type);
        self.add_common_mistakes(operation_type, success);
    }

    /// Generate optimization suggestions for specific operations
    fn get_optimization_suggestions(&self, operation_type: &str) -> Vec<String> {
        match operation_type {
            "matrix_multiply" => vec![
                "Consider using optimized BLAS libraries".to_string(),
                "Check if matrices can be pre-processed".to_string(),
                "Evaluate if batch operations are possible".to_string(),
            ],
            "activation_function" => vec![
                "ReLU is computationally efficient".to_string(),
                "Consider vectorization opportunities".to_string(),
            ],
            "backward_pass" => vec![
                "Check gradient computation efficiency".to_string(),
                "Consider memory optimization techniques".to_string(),
            ],
            _ => vec!["Review operation implementation".to_string()],
        }
    }

    /// Add educational learning objectives
    fn add_learning_objectives(&mut self, operation_type: &str) {
        let objectives = match operation_type {
            "matrix_multiply" => vec!["Understanding matrix operations", "Performance optimization"],
            "activation_function" => vec!["Non-linear transformations", "Gradient flow"],
            "backward_pass" => vec!["Gradient computation", "Chain rule application"],
            _ => vec!["ML algorithm understanding"],
        };

        for obj in objectives {
            if !self.educational_insights.learning_objectives.contains(&obj.to_string()) {
                self.educational_insights.learning_objectives.push(obj.to_string());
            }
        }
    }

    /// Identify common mistakes based on operation patterns
    fn add_common_mistakes(&mut self, operation_type: &str, success: bool) {
        if !success {
            let mistakes = match operation_type {
                "matrix_multiply" => vec![
                    "Dimension mismatch between matrices".to_string(),
                    "Incorrect matrix orientation".to_string(),
                ],
                "activation_function" => vec![
                    "Invalid input ranges".to_string(),
                    "Numerical instability".to_string(),
                ],
                _ => vec!["General implementation error".to_string()],
            };

            for mistake in mistakes {
                if !self.educational_insights.common_mistakes.contains(&mistake) {
                    self.educational_insights.common_mistakes.push(mistake);
                }
            }
        }
    }

    /// Calculate educational performance metrics
    fn calculate_educational_metrics(&self) -> EducationalMetrics {
        let total_operations = self.operation_metrics.values().map(|m| m.call_count).sum();
        
        // Learning efficiency based on success rate and operation count
        let avg_success_rate = if total_operations > 0 {
            self.operation_metrics.values()
                .map(|m| m.success_rate)
                .sum::<f32>() / self.operation_metrics.len() as f32
        } else {
            1.0
        };
        let learning_efficiency_score = avg_success_rate * (1.0 + (total_operations as f32 / 100.0).min(1.0));

        // Memory usage score (inverse of memory pressure)
        let memory_usage_score = if self.current_memory > 0 {
            (1.0 - (self.current_memory as f32 / 1024.0)).max(0.0)
        } else {
            1.0
        };

        // Operation optimization score based on operation diversity and efficiency
        let operation_optimization_score = if !self.operation_metrics.is_empty() {
            let avg_duration = self.operation_metrics.values()
                .map(|m| m.average_duration.as_millis() as f32)
                .sum::<f32>() / self.operation_metrics.len() as f32;
            (1.0 - (avg_duration / 1000.0)).max(0.0)
        } else {
            1.0
        };

        // Overall score
        let overall_educational_score = (learning_efficiency_score + memory_usage_score + operation_optimization_score) / 3.0;

        EducationalMetrics {
            learning_efficiency_score,
            memory_usage_score,
            operation_optimization_score,
            overall_educational_score,
        }
    }

    /// Summarize timeline for educational visualization
    fn summarize_timeline(&self) -> TimelineSummary {
        let event_counts: HashMap<String, usize> = self.timeline
            .iter()
            .map(|e| (e.event_type.clone(), 1))
            .fold(HashMap::new(), |mut acc, (event_type, count)| {
                *acc.entry(event_type).or_insert(0) += count;
                acc
            });

        let total_duration = if let (Some(first), Some(last)) = (self.timeline.front(), self.timeline.back()) {
            last.timestamp.duration_since(first.timestamp)
        } else {
            Duration::from_nanos(0)
        };

        TimelineSummary {
            total_events: self.timeline.len(),
            event_type_distribution: event_counts,
            total_duration,
            average_event_frequency: if total_duration.as_millis() > 0 {
                self.timeline.len() as f32 / (total_duration.as_millis() as f32 / 1000.0)
            } else {
                0.0
            },
        }
    }
}

impl Default for MLPerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EducationalPerformanceReport {
    pub performance_summary: PerformanceStats,
    pub bottlenecks: Vec<Bottleneck>,
    pub optimization_suggestions: Vec<OptimizationSuggestion>,
    pub learning_objectives: Vec<String>,
    pub common_mistakes: Vec<String>,
    pub timeline_summary: TimelineSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineSummary {
    pub total_events: usize,
    pub event_type_distribution: HashMap<String, usize>,
    pub total_duration: Duration,
    pub average_event_frequency: f32,
}