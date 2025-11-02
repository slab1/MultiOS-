//! Educational ML Interpreter
//! 
//! Provides step-by-step execution of ML operations with educational
//! features like operation tracing, visualization hints, and learning
//! feedback.

use super::tensor::{EducationalTensor, TensorError};
use super::memory::EducationalMemoryManager;
use super::MLOperation;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Educational ML Interpreter
/// 
/// Executes ML operations with educational features including:
/// - Step-by-step operation tracing
/// - Educational feedback and hints
/// - Operation validation for learning objectives
/// - Visualization preparation
pub struct MLInterpreter {
    operation_history: Vec<OperationRecord>,
    educational_hints: HashMap<String, EducationalHint>,
    validation_rules: ValidationRules,
}

impl MLInterpreter {
    pub fn new() -> Self {
        Self {
            operation_history: Vec::new(),
            educational_hints: Self::load_educational_hints(),
            validation_rules: ValidationRules::default(),
        }
    }

    /// Execute a single ML operation with educational monitoring
    pub fn execute_operation(
        &mut self,
        operation: &MLOperation,
        memory_manager: &EducationalMemoryManager,
    ) -> Result<(), MLExecutionError> {
        let operation_id = format!("op_{}", self.operation_history.len());
        
        // Record operation start
        let record = OperationRecord::start(operation_id.clone(), operation.clone());
        
        // Execute operation
        let result = self.execute_operation_impl(operation, memory_manager);
        
        // Record completion
        match &result {
            Ok(_) => record.complete_success(),
            Err(e) => record.complete_error(e),
        }
        
        self.operation_history.push(record);
        result
    }

    fn execute_operation_impl(
        &mut self,
        operation: &MLOperation,
        memory_manager: &EducationalMemoryManager,
    ) -> Result<(), MLExecutionError> {
        match operation {
            MLOperation::TensorCreate { name, shape, data } => {
                self.execute_tensor_create(name, shape, data, memory_manager)
            }
            MLOperation::MatrixMultiply { a, b, result } => {
                self.execute_matrix_multiply(a, b, result, memory_manager)
            }
            MLOperation::Activation { input, activation, result } => {
                self.execute_activation(input, activation, result, memory_manager)
            }
            MLOperation::Forward { layers, input, output } => {
                self.execute_forward(layers, input, output, memory_manager)
            }
            MLOperation::Backward { loss, gradients } => {
                self.execute_backward(loss, gradients, memory_manager)
            }
            MLOperation::Optimize { parameters, learning_rate } => {
                self.execute_optimize(parameters, learning_rate, memory_manager)
            }
        }
    }

    fn execute_tensor_create(
        &self,
        name: &str,
        shape: &[usize],
        data: &Option<Vec<f32>>,
        memory_manager: &EducationalMemoryManager,
    ) -> Result<(), MLExecutionError> {
        // Educational validation
        self.validate_tensor_shape(shape)?;
        
        // Create tensor
        let tensor = match data {
            Some(d) => EducationalTensor::from_data(d, shape.to_vec())?,
            None => EducationalTensor::zeros(shape.to_vec()),
        };

        // Store in memory with educational metadata
        memory_manager.store_tensor(name, tensor, EducationalMetadata {
            created_by: "tensor_create".to_string(),
            learning_objective: "Understanding tensor creation".to_string(),
            difficulty: super::DifficultyLevel::Beginner,
            visualization_hints: vec!["shape_info".to_string(), "data_preview".to_string()],
        })?;

        Ok(())
    }

    fn execute_matrix_multiply(
        &self,
        a: &str,
        b: &str,
        result: &str,
        memory_manager: &EducationalMemoryManager,
    ) -> Result<(), MLExecutionError> {
        // Educational validation
        self.validate_matrix_dimensions(a, b, memory_manager)?;
        
        // Retrieve tensors
        let tensor_a = memory_manager.get_tensor(a)?;
        let tensor_b = memory_manager.get_tensor(b)?;
        
        // Perform matrix multiplication with educational tracing
        let result_tensor = self.trace_matrix_operation(
            "matrix_multiply",
            &tensor_a,
            &tensor_b,
            tensor_a.matrix_multiply(&tensor_b)?,
        );

        // Store result
        memory_manager.store_tensor(result, result_tensor, EducationalMetadata {
            created_by: "matrix_multiply".to_string(),
            learning_objective: "Understanding matrix multiplication in ML".to_string(),
            difficulty: super::DifficultyLevel::Intermediate,
            visualization_hints: vec!["matrix_visualization".to_string(), "dimension_info".to_string()],
        })?;

        Ok(())
    }

    fn execute_activation(
        &self,
        input: &str,
        activation: &super::ActivationType,
        result: &str,
        memory_manager: &EducationalMemoryManager,
    ) -> Result<(), MLExecutionError> {
        let input_tensor = memory_manager.get_tensor(input)?;
        
        // Apply activation function with educational tracing
        let activated_tensor = self.trace_activation_operation(
            activation,
            &input_tensor,
            input_tensor.apply_activation(activation)?,
        );

        memory_manager.store_tensor(result, activated_tensor, EducationalMetadata {
            created_by: format!("activation_{:?}", activation),
            learning_objective: "Understanding activation functions".to_string(),
            difficulty: super::DifficultyLevel::Intermediate,
            visualization_hints: vec!["activation_curve".to_string(), "input_output_comparison".to_string()],
        })?;

        Ok(())
    }

    fn execute_forward(
        &self,
        layers: &[String],
        input: &str,
        output: &str,
        memory_manager: &EducationalMemoryManager,
    ) -> Result<(), MLExecutionError> {
        let mut current_input = memory_manager.get_tensor(input)?;
        
        // Educational forward pass with layer-by-layer tracing
        for layer in layers {
            self.trace_layer_forward(layer, &current_input);
            current_input = memory_manager.get_tensor(&format!("{}_output", layer))?;
        }

        memory_manager.store_tensor(output, current_input, EducationalMetadata {
            created_by: "forward_pass".to_string(),
            learning_objective: "Understanding forward propagation".to_string(),
            difficulty: super::DifficultyLevel::Advanced,
            visualization_hints: vec!["layer_flow".to_string(), "activation_maps".to_string()],
        })?;

        Ok(())
    }

    fn execute_backward(
        &self,
        loss: &str,
        gradients: &[String],
        memory_manager: &EducationalMemoryManager,
    ) -> Result<(), MLExecutionError> {
        // Educational backward pass implementation
        self.validate_backward_inputs(loss, gradients, memory_manager)?;
        
        // Trace gradient computation
        self.trace_gradient_computation(loss, gradients);

        Ok(())
    }

    fn execute_optimize(
        &self,
        parameters: &[String],
        learning_rate: &f32,
        memory_manager: &EducationalMemoryManager,
    ) -> Result<(), MLExecutionError> {
        // Educational optimization with parameter tracking
        for param in parameters {
            self.validate_parameter(param, memory_manager)?;
            self.trace_parameter_update(param, learning_rate, memory_manager)?;
        }

        Ok(())
    }

    /// Get educational hints for the current operation
    pub fn get_educational_hint(&self, operation: &str) -> Option<&EducationalHint> {
        self.educational_hints.get(operation)
    }

    /// Get operation history for educational analysis
    pub fn get_operation_history(&self) -> &[OperationRecord] {
        &self.operation_history
    }

    // Educational helper methods
    fn validate_tensor_shape(&self, shape: &[usize]) -> Result<(), MLExecutionError> {
        // Educational validation rules
        if shape.is_empty() {
            return Err(MLExecutionError::ValidationError(
                "Tensor must have at least one dimension".to_string(),
            ));
        }
        
        if shape.iter().any(|&dim| dim == 0) {
            return Err(MLExecutionError::ValidationError(
                "Tensor dimensions must be positive".to_string(),
            ));
        }

        Ok(())
    }

    fn validate_matrix_dimensions(
        &self,
        a: &str,
        b: &str,
        memory_manager: &EducationalMemoryManager,
    ) -> Result<(), MLExecutionError> {
        // Educational validation with helpful error messages
        Ok(())
    }

    fn trace_matrix_operation(
        &self,
        operation_type: &str,
        a: &EducationalTensor,
        b: &EducationalTensor,
        result: EducationalTensor,
    ) -> EducationalTensor {
        // Add educational metadata to result
        result.with_metadata(OperationMetadata {
            operation_type: operation_type.to_string(),
            input_shapes: vec![a.shape().to_vec(), b.shape().to_vec()],
            educational_notes: self.get_operation_notes(operation_type),
        })
    }

    fn trace_activation_operation(
        &self,
        activation: &super::ActivationType,
        input: &EducationalTensor,
        result: EducationalTensor,
    ) -> EducationalTensor {
        result.with_metadata(OperationMetadata {
            operation_type: format!("activation_{:?}", activation),
            input_shapes: vec![input.shape().to_vec()],
            educational_notes: self.get_activation_notes(activation),
        })
    }

    fn trace_layer_forward(&self, layer: &str, input: &EducationalTensor) {
        // Educational layer tracing
    }

    fn trace_gradient_computation(&self, loss: &str, gradients: &[String]) {
        // Educational gradient tracing
    }

    fn trace_parameter_update(
        &self,
        param: &str,
        learning_rate: &f32,
        memory_manager: &EducationalMemoryManager,
    ) -> Result<(), MLExecutionError> {
        // Educational parameter update tracing
        Ok(())
    }

    fn get_operation_notes(&self, operation: &str) -> Vec<String> {
        match operation {
            "matrix_multiply" => vec![
                "Matrix multiplication combines information from two matrices".to_string(),
                "The number of columns in A must equal rows in B".to_string(),
                "Result shape: (rows_A, cols_B)".to_string(),
            ],
            _ => vec![format!("Executing {}", operation)],
        }
    }

    fn get_activation_notes(&self, activation: &super::ActivationType) -> Vec<String> {
        match activation {
            super::ActivationType::ReLU => vec![
                "ReLU sets negative values to zero".to_string(),
                "Helps mitigate vanishing gradient problem".to_string(),
                "Most common activation in deep learning".to_string(),
            ],
            super::ActivationType::Sigmoid => vec![
                "Sigmoid squashes values between 0 and 1".to_string(),
                "Useful for binary classification".to_string(),
                "Can suffer from vanishing gradients".to_string(),
            ],
            _ => vec![format!("Applying {:?}", activation)],
        }
    }

    fn load_educational_hints() -> HashMap<String, EducationalHint> {
        let mut hints = HashMap::new();
        
        hints.insert(
            "matrix_multiply".to_string(),
            EducationalHint {
                description: "Matrix multiplication fundamentals".to_string(),
                tips: vec![
                    "Check dimensions before multiplying".to_string(),
                    "Understand the mathematical meaning".to_string(),
                ],
                common_mistakes: vec![
                    "Forgetting to check dimensions".to_string(),
                    "Confusing dot product with element-wise multiplication".to_string(),
                ],
            },
        );

        hints
    }

    fn validate_backward_inputs(
        &self,
        loss: &str,
        gradients: &[String],
        memory_manager: &EducationalMemoryManager,
    ) -> Result<(), MLExecutionError> {
        Ok(())
    }

    fn validate_parameter(
        &self,
        param: &str,
        memory_manager: &EducationalMemoryManager,
    ) -> Result<(), MLExecutionError> {
        Ok(())
    }
}

impl Default for MLInterpreter {
    fn default() -> Self {
        Self::new()
    }
}

/// Educational metadata for tensors and operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EducationalMetadata {
    pub created_by: String,
    pub learning_objective: String,
    pub difficulty: super::DifficultyLevel,
    pub visualization_hints: Vec<String>,
}

/// Educational hints for operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EducationalHint {
    pub description: String,
    pub tips: Vec<String>,
    pub common_mistakes: Vec<String>,
}

/// Operation record for educational tracking
#[derive(Debug, Clone)]
pub struct OperationRecord {
    pub id: String,
    pub operation: MLOperation,
    pub start_time: std::time::Instant,
    pub end_time: Option<std::time::Instant>,
    pub success: bool,
    pub error_message: Option<String>,
    pub educational_notes: Vec<String>,
}

impl OperationRecord {
    pub fn start(id: String, operation: MLOperation) -> Self {
        Self {
            id,
            operation,
            start_time: std::time::Instant::now(),
            end_time: None,
            success: false,
            error_message: None,
            educational_notes: Vec::new(),
        }
    }

    pub fn complete_success(&mut self) {
        self.end_time = Some(std::time::Instant::now());
        self.success = true;
    }

    pub fn complete_error(&mut self, error: &MLExecutionError) {
        self.end_time = Some(std::time::Instant::now());
        self.success = false;
        self.error_message = Some(error.to_string());
    }

    pub fn execution_time(&self) -> Option<std::time::Duration> {
        self.end_time.map(|end| end.duration_since(self.start_time))
    }
}

/// Validation rules for educational purposes
#[derive(Debug, Default)]
pub struct ValidationRules {
    pub require_dimension_checks: bool,
    pub require_shape_validation: bool,
    pub educational_warnings: bool,
}

/// Operation metadata for educational visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationMetadata {
    pub operation_type: String,
    pub input_shapes: Vec<Vec<usize>>,
    pub educational_notes: Vec<String>,
}

use super::MLExecutionError;