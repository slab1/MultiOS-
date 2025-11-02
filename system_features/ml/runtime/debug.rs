//! Debug utilities for educational ML runtime
//! 
//! Provides comprehensive debugging features for ML education including:
//! - Step-by-step execution tracing
//! - Educational variable inspection
//! - Performance debugging
//! - Learning feedback and hints

use super::tensor::EducationalTensor;
use super::memory::MemoryStats;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};

/// Educational Debug Manager
/// 
/// Provides debugging capabilities with educational features:
/// - Step-by-step execution tracing
/// - Educational breakpoints
 Variable inspection and analysis
/// - Performance debugging
/// - Learning feedback system
pub struct EducationalDebugger {
    debug_session: DebugSession,
    breakpoints: HashMap<String, Breakpoint>,
    trace_history: Vec<DebugTrace>,
    educational_hints: EducationalHintSystem,
    visualization_config: VisualizationConfig,
}

#[derive(Debug, Clone)]
struct DebugSession {
    pub session_id: String,
    pub start_time: SystemTime,
    pub current_step: usize,
    pub is_active: bool,
    pub step_mode: bool,
    pub educational_mode: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Breakpoint {
    pub location: String,
    pub condition: Option<BreakpointCondition>,
    pub hit_count: usize,
    pub enabled: bool,
    pub educational_note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BreakpointCondition {
    TensorValue { tensor_name: String, condition: String },
    MemoryUsage { threshold_mb: usize },
    OperationTime { threshold_ms: usize },
    Custom { expression: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugTrace {
    pub step_number: usize,
    pub timestamp: SystemTime,
    pub operation: String,
    pub operation_details: OperationDetails,
    pub tensor_states: HashMap<String, TensorDebugInfo>,
    pub memory_state: MemoryDebugInfo,
    pub performance_metrics: PerformanceDebugInfo,
    pub educational_context: EducationalContext,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationDetails {
    pub operation_type: String,
    pub parameters: HashMap<String, serde_json::Value>,
    pub input_tensors: Vec<String>,
    pub output_tensors: Vec<String>,
    pub execution_time: Duration,
    pub educational_notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TensorDebugInfo {
    pub name: String,
    pub shape: Vec<usize>,
    pub data_preview: Vec<f32>,
    pub statistical_summary: TensorStats,
    pub memory_usage_kb: usize,
    pub creation_location: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TensorStats {
    pub min_value: f32,
    pub max_value: f32,
    pub mean: f32,
    pub std_dev: f32,
    pub zero_count: usize,
    pub nan_count: usize,
    pub inf_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryDebugInfo {
    pub current_usage_mb: usize,
    pub peak_usage_mb: usize,
    pub tensor_count: usize,
    pub allocation_rate: f32,
    pub memory_fragments: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceDebugInfo {
    pub operation_time: Duration,
    pub memory_delta_mb: isize,
    pub cpu_usage_estimate: f32,
    pub cache_hit_rate: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EducationalContext {
    pub learning_objective: String,
    pub difficulty_level: DifficultyLevel,
    pub step_explanation: String,
    pub hints_available: Vec<String>,
    pub common_mistakes: Vec<String>,
    pub next_steps: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DifficultyLevel {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}

#[derive(Debug, Default)]
struct EducationalHintSystem {
    pub current_hints: Vec<EducationalHint>,
    pub hint_history: Vec<(usize, String)>,
    pub learning_progress: HashMap<String, f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct EducationalHint {
    pub hint_type: HintType,
    pub content: String,
    pub priority: HintPriority,
    pub context: String,
    pub learning_benefit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HintType {
    Performance,
    Algorithm,
    Memory,
    Visualization,
    BestPractice,
    CommonMistake,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HintPriority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationConfig {
    pub show_tensor_values: bool,
    pub show_memory_usage: bool,
    pub show_performance_metrics: bool,
    pub educational_annotations: bool,
    pub real_time_updates: bool,
    pub color_coding: bool,
}

impl EducationalDebugger {
    /// Create a new educational debugger
    pub fn new(session_id: &str) -> Self {
        Self {
            debug_session: DebugSession {
                session_id: session_id.to_string(),
                start_time: SystemTime::now(),
                current_step: 0,
                is_active: true,
                step_mode: false,
                educational_mode: true,
            },
            breakpoints: HashMap::new(),
            trace_history: Vec::new(),
            educational_hints: EducationalHintSystem::default(),
            visualization_config: VisualizationConfig {
                show_tensor_values: true,
                show_memory_usage: true,
                show_performance_metrics: true,
                educational_annotations: true,
                real_time_updates: true,
                color_coding: true,
            },
        }
    }

    /// Start a new debug session
    pub fn start_session(&mut self, session_id: &str) {
        self.debug_session = DebugSession {
            session_id: session_id.to_string(),
            start_time: SystemTime::now(),
            current_step: 0,
            is_active: true,
            step_mode: false,
            educational_mode: true,
        };
        self.trace_history.clear();
    }

    /// Set a breakpoint with educational context
    pub fn set_breakpoint(
        &mut self,
        location: &str,
        educational_note: Option<&str>,
    ) {
        self.breakpoints.insert(
            location.to_string(),
            Breakpoint {
                location: location.to_string(),
                condition: None,
                hit_count: 0,
                enabled: true,
                educational_note: educational_note.map(|s| s.to_string()),
            },
        );
    }

    /// Set a conditional breakpoint
    pub fn set_conditional_breakpoint(
        &mut self,
        location: &str,
        condition: BreakpointCondition,
        educational_note: Option<&str>,
    ) {
        self.breakpoints.insert(
            location.to_string(),
            Breakpoint {
                location: location.to_string(),
                condition: Some(condition),
                hit_count: 0,
                enabled: true,
                educational_note: educational_note.map(|s| s.to_string()),
            },
        );
    }

    /// Trace an operation step with educational context
    pub fn trace_operation(
        &mut self,
        operation: &str,
        parameters: HashMap<String, serde_json::Value>,
        input_tensors: &[String],
        output_tensors: &[String],
        execution_time: Duration,
        memory_before_mb: usize,
        memory_after_mb: usize,
        learning_objective: &str,
        step_explanation: &str,
    ) -> DebugAction {
        self.debug_session.current_step += 1;
        let step_number = self.debug_session.current_step;

        // Check for breakpoints
        if self.should_break_at(operation) {
            return DebugAction::Break;
        }

        // Create tensor debug information
        let mut tensor_states = HashMap::new();
        for tensor_name in input_tensors.iter().chain(output_tensors.iter()) {
            // This would integrate with actual tensor storage in a real implementation
            let debug_info = self.create_tensor_debug_info(tensor_name);
            if let Some(info) = debug_info {
                tensor_states.insert(tensor_name.clone(), info);
            }
        }

        // Create memory debug information
        let memory_state = MemoryDebugInfo {
            current_usage_mb: memory_after_mb,
            peak_usage_mb: memory_after_mb.max(memory_before_mb),
            tensor_count: tensor_states.len(),
            allocation_rate: (memory_after_mb - memory_before_mb) as f32 / execution_time.as_millis() as f32,
            memory_fragments: 0, // Would calculate in real implementation
        };

        // Create performance debug information
        let performance_metrics = PerformanceDebugInfo {
            operation_time: execution_time,
            memory_delta_mb: memory_after_mb as isize - memory_before_mb as isize,
            cpu_usage_estimate: self.estimate_cpu_usage(execution_time),
            cache_hit_rate: 0.8, // Would calculate in real implementation
        };

        // Create educational context
        let educational_context = EducationalContext {
            learning_objective: learning_objective.to_string(),
            difficulty_level: self.infer_difficulty_level(operation),
            step_explanation: step_explanation.to_string(),
            hints_available: self.generate_hints(operation, execution_time),
            common_mistakes: self.identify_common_mistakes(operation),
            next_steps: self.suggest_next_steps(operation),
        };

        // Create operation details
        let operation_details = OperationDetails {
            operation_type: operation.to_string(),
            parameters,
            input_tensors: input_tensors.to_vec(),
            output_tensors: output_tensors.to_vec(),
            execution_time,
            educational_notes: self.generate_operation_notes(operation),
        };

        // Create and store debug trace
        let debug_trace = DebugTrace {
            step_number,
            timestamp: SystemTime::now(),
            operation: operation.to_string(),
            operation_details,
            tensor_states,
            memory_state,
            performance_metrics,
            educational_context,
        };

        self.trace_history.push(debug_trace);

        // Generate educational hints
        self.update_educational_hints(operation, execution_time);

        DebugAction::Continue
    }

    /// Get current debug trace
    pub fn get_current_trace(&self) -> Option<&DebugTrace> {
        self.trace_history.last()
    }

    /// Get debug trace history
    pub fn get_trace_history(&self, limit: Option<usize>) -> &[DebugTrace] {
        let limit = limit.unwrap_or(100);
        let len = self.trace_history.len();
        &self.trace_history[len.saturating_sub(limit)..]
    }

    /// Get educational summary
    pub fn get_educational_summary(&self) -> EducationalSummary {
        let total_steps = self.trace_history.len();
        let total_time: Duration = self.trace_history
            .iter()
            .map(|trace| trace.operation_details.execution_time)
            .fold(Duration::from_nanos(0), |acc, d| acc + d);

        let operations_by_type: HashMap<String, usize> = self.trace_history
            .iter()
            .map(|trace| (trace.operation.clone(), 1))
            .fold(HashMap::new(), |mut acc, (op, count)| {
                *acc.entry(op).or_insert(0) += count;
                acc
            });

        let most_common_operation = operations_by_type
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(op, _)| op.clone());

        let learning_progress = self.calculate_learning_progress();

        EducationalSummary {
            total_execution_time: total_time,
            total_steps,
            average_step_time: if total_steps > 0 {
                Duration::from_nanos(total_time.as_nanos() as u64 / total_steps as u64)
            } else {
                Duration::from_nanos(0)
            },
            most_common_operation,
            learning_objectives_completed: learning_progress.objectives_completed,
            common_mistakes_made: learning_progress.mistakes_made,
            performance_insights: self.generate_performance_insights(),
            educational_score: self.calculate_educational_score(),
        }
    }

    /// Enable/disable step mode for debugging
    pub fn set_step_mode(&mut self, enabled: bool) {
        self.debug_session.step_mode = enabled;
    }

    /// Get available educational hints
    pub fn get_available_hints(&self) -> &[EducationalHint] {
        &self.educational_hints.current_hints
    }

    /// Get visualization configuration
    pub fn get_visualization_config(&self) -> &VisualizationConfig {
        &self.visualization_config
    }

    /// Update visualization configuration
    pub fn update_visualization_config(&mut self, config: VisualizationConfig) {
        self.visualization_config = config;
    }

    /// Check if should break at location
    fn should_break_at(&self, operation: &str) -> bool {
        // Check for unconditional breakpoints
        for (location, breakpoint) in &self.breakpoints {
            if breakpoint.enabled && location == operation {
                return true;
            }
        }

        // Check for conditional breakpoints
        // This would evaluate conditions in a real implementation
        false
    }

    /// Create tensor debug information
    fn create_tensor_debug_info(&self, tensor_name: &str) -> Option<TensorDebugInfo> {
        // In a real implementation, this would retrieve actual tensor data
        // For educational purposes, we'll create mock data
        Some(TensorDebugInfo {
            name: tensor_name.to_string(),
            shape: vec![2, 3], // Mock shape
            data_preview: vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
            statistical_summary: TensorStats {
                min_value: 1.0,
                max_value: 6.0,
                mean: 3.5,
                std_dev: 1.707,
                zero_count: 0,
                nan_count: 0,
                inf_count: 0,
            },
            memory_usage_kb: 24, // Mock size
            creation_location: Some(format!("line_{}_col_10", self.debug_session.current_step)),
        })
    }

    /// Estimate CPU usage (simplified)
    fn estimate_cpu_usage(&self, execution_time: Duration) -> f32 {
        // Simplified CPU usage estimation
        let time_ms = execution_time.as_millis();
        if time_ms < 10 {
            0.1
        } else if time_ms < 100 {
            0.5
        } else {
            0.9
        }
    }

    /// Infer difficulty level from operation
    fn infer_difficulty_level(&self, operation: &str) -> DifficultyLevel {
        match operation {
            "tensor_create" | "matrix_multiply" => DifficultyLevel::Beginner,
            "activation_function" => DifficultyLevel::Intermediate,
            "forward_pass" | "backward_pass" => DifficultyLevel::Advanced,
            _ => DifficultyLevel::Intermediate,
        }
    }

    /// Generate hints for operation
    fn generate_hints(&self, operation: &str, execution_time: Duration) -> Vec<String> {
        let mut hints = Vec::new();
        
        match operation {
            "matrix_multiply" => {
                hints.push("Matrix multiplication combines information from rows and columns".to_string());
                hints.push("Check dimensions: columns of first matrix must equal rows of second".to_string());
            }
            "activation_function" => {
                hints.push("Activation functions introduce non-linearity".to_string());
                hints.push("ReLU is computationally efficient for deep networks".to_string());
            }
            _ => {
                hints.push(format!("Understanding {} operation fundamentals".to_string(), operation));
            }
        }

        if execution_time.as_millis() > 100 {
            hints.push("This operation took longer than expected. Consider optimization.".to_string());
        }

        hints
    }

    /// Identify common mistakes for operation
    fn identify_common_mistakes(&self, operation: &str) -> Vec<String> {
        match operation {
            "matrix_multiply" => vec![
                "Forgetting to check matrix dimensions".to_string(),
                "Confusing element-wise multiplication with matrix multiplication".to_string(),
            ],
            "activation_function" => vec![
                "Not understanding the mathematical properties".to_string(),
                "Applying activation to wrong tensor".to_string(),
            ],
            _ => vec!["General implementation error".to_string()],
        }
    }

    /// Suggest next steps for learning
    fn suggest_next_steps(&self, operation: &str) -> Vec<String> {
        match operation {
            "tensor_create" => vec![
                "Practice creating tensors with different shapes".to_string(),
                "Learn about tensor indexing and slicing".to_string(),
            ],
            "matrix_multiply" => vec![
                "Implement batch matrix multiplication".to_string(),
                "Study memory-efficient multiplication algorithms".to_string(),
            ],
            _ => vec!["Continue with next operation".to_string()],
        }
    }

    /// Generate operation notes
    fn generate_operation_notes(&self, operation: &str) -> Vec<String> {
        match operation {
            "tensor_create" => vec!["Tensor creation is fundamental to all ML operations".to_string()],
            "matrix_multiply" => vec!["Core operation for linear transformations".to_string()],
            "forward_pass" => vec!["Forward pass computes network predictions".to_string()],
            _ => vec![],
        }
    }

    /// Update educational hints based on recent activity
    fn update_educational_hints(&mut self, operation: &str, execution_time: Duration) {
        // Add performance hints if operations are slow
        if execution_time.as_millis() > 50 {
            let hint = EducationalHint {
                hint_type: HintType::Performance,
                content: format!("Operation {} took {:?}. Consider optimization strategies.", 
                               operation, execution_time),
                priority: HintPriority::Medium,
                context: "performance".to_string(),
                learning_benefit: "Understanding performance optimization".to_string(),
            };
            self.educational_hints.current_hints.push(hint);
        }

        // Keep only the most recent hints
        if self.educational_hints.current_hints.len() > 10 {
            self.educational_hints.current_hints.drain(0..5);
        }
    }

    /// Calculate learning progress
    fn calculate_learning_progress(&self) -> LearningProgress {
        let objectives_completed = self.trace_history
            .iter()
            .map(|trace| trace.educational_context.learning_objective.clone())
            .collect::<std::collections::HashSet<_>>()
            .len();

        let mistakes_made = self.trace_history
            .iter()
            .flat_map(|trace| trace.educational_context.common_mistakes.clone())
            .collect::<std::collections::HashSet<_>>()
            .len();

        LearningProgress {
            objectives_completed,
            mistakes_made,
            total_operations: self.trace_history.len(),
        }
    }

    /// Generate performance insights
    fn generate_performance_insights(&self) -> Vec<String> {
        let mut insights = Vec::new();

        let avg_time: Duration = if !self.trace_history.is_empty() {
            let total_time: Duration = self.trace_history
                .iter()
                .map(|t| t.operation_details.execution_time)
                .fold(Duration::from_nanos(0), |acc, d| acc + d);
            Duration::from_nanos(total_time.as_nanos() as u64 / self.trace_history.len() as u64)
        } else {
            Duration::from_nanos(0)
        };

        insights.push(format!("Average operation time: {:?}", avg_time));

        let slowest_op = self.trace_history
            .iter()
            .max_by_key(|t| t.operation_details.execution_time);
        
        if let Some(op) = slowest_op {
            insights.push(format!("Slowest operation: {} ({:?})", 
                op.operation, op.operation_details.execution_time));
        }

        insights
    }

    /// Calculate educational score
    fn calculate_educational_score(&self) -> f32 {
        if self.trace_history.is_empty() {
            return 0.0;
        }

        let progress = self.calculate_learning_progress();
        let base_score = (progress.objectives_completed as f32 / 10.0).min(1.0);
        let mistake_penalty = (progress.mistakes_made as f32 / 5.0).min(0.3);
        let exploration_bonus = (self.trace_history.len() as f32 / 20.0).min(0.2);

        (base_score - mistake_penalty + exploration_bonus).max(0.0).min(1.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DebugAction {
    Continue,
    Break,
    Step,
    Next,
    Finish,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EducationalSummary {
    pub total_execution_time: Duration,
    pub total_steps: usize,
    pub average_step_time: Duration,
    pub most_common_operation: Option<String>,
    pub learning_objectives_completed: usize,
    pub common_mistakes_made: usize,
    pub performance_insights: Vec<String>,
    pub educational_score: f32,
}

#[derive(Debug, Clone)]
struct LearningProgress {
    pub objectives_completed: usize,
    pub mistakes_made: usize,
    pub total_operations: usize,
}

impl Default for EducationalDebugger {
    fn default() -> Self {
        Self::new("default_session")
    }
}