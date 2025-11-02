//! MultiOS Educational ML Runtime
//! 
//! This module provides a basic machine learning runtime and interpreter
//! designed specifically for computer science education.

pub mod interpreter;
pub mod tensor;
pub mod ops;
pub mod memory;
pub mod debug;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};

/// Educational ML Runtime Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    pub max_memory_mb: usize,
    pub max_compute_units: usize,
    pub debug_level: DebugLevel,
    pub educational_mode: bool,
    pub visualization_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DebugLevel {
    None,
    Basic,
    Verbose,
    Educational,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            max_memory_mb: 512,
            max_compute_units: 4,
            debug_level: DebugLevel::Educational,
            educational_mode: true,
            visualization_enabled: true,
        }
    }
}

/// Educational ML Runtime
/// 
/// Provides a high-level interface for executing ML workloads
/// with educational features like step-by-step debugging and
/// visualization support.
pub struct EducationalMLRuntime {
    config: RuntimeConfig,
    interpreter: interpreter::MLInterpreter,
    memory_manager: memory::EducationalMemoryManager,
    debug_trace: Arc<Mutex<Vec<DebugEvent>>>,
    performance_monitor: performance::MLPerformanceMonitor,
}

impl EducationalMLRuntime {
    /// Create a new educational ML runtime
    pub fn new(config: RuntimeConfig) -> Self {
        Self {
            interpreter: interpreter::MLInterpreter::new(),
            memory_manager: memory::EducationalMemoryManager::new(config.max_memory_mb),
            debug_trace: Arc::new(Mutex::new(Vec::new())),
            performance_monitor: performance::MLPerformanceMonitor::new(),
            config,
        }
    }

    /// Execute ML program with educational features
    pub fn execute_ml_program(
        &self,
        program: &MLProgram,
    ) -> Result<MLResult, MLExecutionError> {
        let start_time = std::time::Instant::now();
        
        // Log execution start for educational tracking
        self.log_event(DebugEvent::ProgramStart {
            program_id: program.id.clone(),
            timestamp: start_time,
        });

        // Execute with educational monitoring
        let result = self.execute_with_debugging(program)?;

        // Record performance metrics
        let execution_time = start_time.elapsed();
        self.performance_monitor.record_execution(
            &program.id,
            execution_time,
            self.memory_manager.get_current_usage(),
        );

        Ok(result)
    }

    /// Get educational debug information
    pub fn get_debug_trace(&self) -> Vec<DebugEvent> {
        self.debug_trace.lock().unwrap().clone()
    }

    /// Get memory usage statistics
    pub fn get_memory_stats(&self) -> memory::MemoryStats {
        self.memory_manager.get_stats()
    }

    /// Get performance metrics
    pub fn get_performance_stats(&self) -> performance::PerformanceStats {
        self.performance_monitor.get_stats()
    }

    fn execute_with_debugging(
        &self,
        program: &MLProgram,
    ) -> Result<MLResult, MLExecutionError> {
        // Educational step-by-step execution
        for (step_id, operation) in program.operations.iter().enumerate() {
            self.log_event(DebugEvent::OperationStart {
                step_id,
                operation: operation.clone(),
                timestamp: std::time::Instant::now(),
            });

            // Execute with educational monitoring
            let _ = self.interpreter.execute_operation(operation, &self.memory_manager)?;

            self.log_event(DebugEvent::OperationComplete {
                step_id,
                timestamp: std::time::Instant::now(),
            });
        }

        Ok(MLResult {
            output_tensors: Vec::new(),
            debug_info: Some(self.get_debug_trace()),
            performance_metrics: Some(self.get_performance_stats()),
        })
    }

    fn log_event(&self, event: DebugEvent) {
        if let Ok(mut trace) = self.debug_trace.lock() {
            trace.push(event);
        }
    }
}

/// ML Program representation for educational purposes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MLProgram {
    pub id: String,
    pub name: String,
    pub operations: Vec<MLOperation>,
    pub metadata: ProgramMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgramMetadata {
    pub description: String,
    pub difficulty_level: DifficultyLevel,
    pub learning_objectives: Vec<String>,
    pub estimated_duration: std::time::Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DifficultyLevel {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}

/// ML Operation for educational execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MLOperation {
    TensorCreate { name: String, shape: Vec<usize>, data: Option<Vec<f32>> },
    MatrixMultiply { a: String, b: String, result: String },
    Activation { input: String, activation: ActivationType, result: String },
    Forward { layers: Vec<String>, input: String, output: String },
    Backward { loss: String, gradients: Vec<String> },
    Optimize { parameters: Vec<String>, learning_rate: f32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActivationType {
    ReLU,
    Sigmoid,
    Tanh,
    Softmax,
}

/// ML Execution Result with educational information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MLResult {
    pub output_tensors: Vec<tensor::EducationalTensor>,
    pub debug_info: Option<Vec<DebugEvent>>,
    pub performance_metrics: Option<performance::PerformanceStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugEvent {
    pub timestamp: std::time::Instant,
    pub event_type: String,
    pub details: serde_json::Value,
}

/// Error types for ML execution with educational context
#[derive(Debug, thiserror::Error)]
pub enum MLExecutionError {
    #[error("Memory allocation failed: {0}")]
    MemoryError(String),
    #[error("Invalid tensor operation: {0}")]
    TensorError(String),
    #[error("Educational validation failed: {0}")]
    ValidationError(String),
    #[error("Runtime error: {0}")]
    RuntimeError(String),
}

impl From<memory::MemoryError> for MLExecutionError {
    fn from(err: memory::MemoryError) -> Self {
        MLExecutionError::MemoryError(err.to_string())
    }
}

impl From<tensor::TensorError> for MLExecutionError {
    fn from(err: tensor::TensorError) -> Self {
        MLExecutionError::TensorError(err.to_string())
    }
}

// Re-export modules for convenience
pub mod performance;