//! Educational Tensor Implementation
//! 
//! Provides tensor operations with educational features including:
//! - Step-by-step operation visualization
//! - Educational error messages
//! - Performance metrics
//! - Memory usage tracking

use serde::{Deserialize, Serialize};
use std::fmt::{self, Debug, Display};
use std::ops::{Add, Sub, Mul, Div, Index, IndexMut};
use std::sync::{Arc, Mutex};

/// Educational Tensor with visualization and debugging capabilities
#[derive(Clone, Serialize, Deserialize)]
pub struct EducationalTensor {
    data: Vec<f32>,
    shape: Vec<usize>,
    metadata: TensorMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TensorMetadata {
    pub name: Option<String>,
    pub created_at: std::time::SystemTime,
    pub operation_history: Vec<OperationTrace>,
    pub educational_notes: Vec<String>,
    pub visualization_hints: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationTrace {
    pub operation: String,
    pub timestamp: std::time::SystemTime,
    pub input_shapes: Vec<Vec<usize>>,
    pub output_shape: Vec<usize>,
}

impl EducationalTensor {
    /// Create a new tensor with educational metadata
    pub fn new(data: Vec<f32>, shape: Vec<usize>) -> Result<Self, TensorError> {
        // Validate dimensions
        let expected_size: usize = shape.iter().product();
        if data.len() != expected_size {
            return Err(TensorError::DimensionMismatch {
                expected: expected_size,
                actual: data.len(),
            });
        }

        Ok(Self {
            data,
            shape,
            metadata: TensorMetadata {
                name: None,
                created_at: std::time::SystemTime::now(),
                operation_history: Vec::new(),
                educational_notes: Vec::new(),
                visualization_hints: vec!["show_values".to_string(), "highlight_shape".to_string()],
            },
        })
    }

    /// Create tensor from data with educational features
    pub fn from_data(data: &[f32], shape: Vec<usize>) -> Result<Self, TensorError> {
        Self::new(data.to_vec(), shape)
    }

    /// Create a zero tensor
    pub fn zeros(shape: Vec<usize>) -> Self {
        let size: usize = shape.iter().product();
        Self::new(vec![0.0; size], shape).expect("Failed to create zeros tensor")
    }

    /// Create a ones tensor
    pub fn ones(shape: Vec<usize>) -> Self {
        let size: usize = shape.iter().product();
        Self::new(vec![1.0; size], shape).expect("Failed to create ones tensor")
    }

    /// Create a random tensor with educational seed
    pub fn random(shape: Vec<usize>, seed: u64) -> Self {
        let size: usize = shape.iter().product();
        let mut data = Vec::with_capacity(size);
        let mut rng = EducationalRng::new(seed);

        for _ in 0..size {
            data.push(rng.next_f32());
        }

        Self::new(data, shape).expect("Failed to create random tensor")
    }

    /// Get tensor shape
    pub fn shape(&self) -> &[usize] {
        &self.shape
    }

    /// Get tensor size (total number of elements)
    pub fn size(&self) -> usize {
        self.data.len()
    }

    /// Get dimension count
    pub fn dims(&self) -> usize {
        self.shape.len()
    }

    /// Access tensor element with educational bounds checking
    pub fn get(&self, indices: &[usize]) -> Result<f32, TensorError> {
        if indices.len() != self.shape.len() {
            return Err(TensorError::DimensionMismatch {
                expected: self.shape.len(),
                actual: indices.len(),
            });
        }

        for (i, &index) in indices.iter().enumerate() {
            if index >= self.shape[i] {
                return Err(TensorError::IndexOutOfBounds {
                    dimension: i,
                    index,
                    max: self.shape[i] - 1,
                });
            }
        }

        let linear_index = self.flatten_indices(indices);
        Ok(self.data[linear_index])
    }

    /// Set tensor element with educational validation
    pub fn set(&mut self, indices: &[usize], value: f32) -> Result<(), TensorError> {
        if indices.len() != self.shape.len() {
            return Err(TensorError::DimensionMismatch {
                expected: self.shape.len(),
                actual: indices.len(),
            });
        }

        let linear_index = self.flatten_indices(indices);
        self.data[linear_index] = value;
        
        // Add educational note about modification
        self.metadata.educational_notes.push(
            format!("Element at {:?} set to {}", indices, value)
        );

        Ok(())
    }

    /// Matrix multiplication with educational tracing
    pub fn matrix_multiply(&self, other: &EducationalTensor) -> Result<EducationalTensor, TensorError> {
        // Educational validation
        self.validate_matrix_compatibility(other)?;

        let (m, k) = (self.shape[0], self.shape[1]);
        let (_k, n) = (other.shape[0], other.shape[1]);

        let mut result_data = vec![0.0; m * n];

        // Educational step-by-step multiplication
        for i in 0..m {
            for j in 0..n {
                let mut sum = 0.0;
                for kk in 0..k {
                    let a_val = self.get(&[i, kk])?;
                    let b_val = other.get(&[kk, j])?;
                    sum += a_val * b_val;
                    
                    // Educational tracing
                    if i == 0 && j == 0 && kk < 3 {
                        println!("Educational: Computing element [{},{}], step {}: {} × {} = {}, running sum: {}", 
                                i, j, kk, a_val, b_val, a_val * b_val, sum);
                    }
                }
                result_data[i * n + j] = sum;
            }
        }

        let mut result = EducationalTensor::new(result_data, vec![m, n])?;
        
        // Add operation trace for educational visualization
        result.metadata.operation_history.push(OperationTrace {
            operation: "matrix_multiply".to_string(),
            timestamp: std::time::SystemTime::now(),
            input_shapes: vec![self.shape.clone(), other.shape.clone()],
            output_shape: result.shape.clone(),
        });

        Ok(result)
    }

    /// Apply activation function with educational notes
    pub fn apply_activation(
        &self,
        activation: &super::ActivationType,
    ) -> Result<EducationalTensor, TensorError> {
        let mut result_data = Vec::with_capacity(self.data.len());

        for &value in &self.data {
            let activated = match activation {
                super::ActivationType::ReLU => value.max(0.0),
                super::ActivationType::Sigmoid => 1.0 / (1.0 + (-value).exp()),
                super::ActivationType::Tanh => value.tanh(),
                super::ActivationType::Softmax => {
                    // Note: Proper softmax requires normalizing across a dimension
                    // This is a simplified version for educational purposes
                    value.exp()
                }
            };
            result_data.push(activated);
        }

        let mut result = EducationalTensor::new(result_data, self.shape.clone())?;
        
        // Add educational notes
        result.metadata.educational_notes.push(
            format!("Applied {:?} activation function", activation)
        );

        Ok(result)
    }

    /// Element-wise addition with educational validation
    pub fn element_add(&self, other: &EducationalTensor) -> Result<EducationalTensor, TensorError> {
        self.validate_broadcast_compatibility(other)?;

        let mut result_data = Vec::with_capacity(self.data.len());

        for i in 0..self.data.len() {
            result_data.push(self.data[i] + other.data[i]);
        }

        Ok(EducationalTensor::new(result_data, self.shape.clone())?)
    }

    /// Calculate mean with educational tracking
    pub fn mean(&self) -> f32 {
        let sum: f32 = self.data.iter().sum();
        sum / self.data.len() as f32
    }

    /// Calculate standard deviation
    pub fn std(&self) -> f32 {
        let mean = self.mean();
        let variance: f32 = self.data
            .iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f32>() / self.data.len() as f32;
        variance.sqrt()
    }

    /// Get educational visualization hints
    pub fn get_visualization_hints(&self) -> &[String] {
        &self.metadata.visualization_hints
    }

    /// Get operation history for educational analysis
    pub fn get_operation_history(&self) -> &[OperationTrace] {
        &self.metadata.operation_history
    }

    /// Add educational metadata
    pub fn with_metadata(mut self, metadata: OperationMetadata) -> Self {
        self.metadata.operation_history.push(OperationTrace {
            operation: metadata.operation_type,
            timestamp: std::time::SystemTime::now(),
            input_shapes: metadata.input_shapes,
            output_shape: self.shape.clone(),
        });
        self.metadata.educational_notes.extend(metadata.educational_notes);
        self
    }

    /// Set tensor name for educational identification
    pub fn with_name(mut self, name: &str) -> Self {
        self.metadata.name = Some(name.to_string());
        self
    }

    /// Flatten multi-dimensional indices to linear index
    fn flatten_indices(&self, indices: &[usize]) -> usize {
        let mut linear_index = 0;
        let mut stride = 1;

        for i in (0..indices.len()).rev() {
            linear_index += indices[i] * stride;
            stride *= self.shape[i];
        }

        linear_index
    }

    /// Educational matrix compatibility validation
    fn validate_matrix_compatibility(&self, other: &EducationalTensor) -> Result<(), TensorError> {
        if self.dims() != 2 || other.dims() != 2 {
            return Err(TensorError::Educational(
                "Matrix multiplication requires 2D tensors".to_string()
            ));
        }

        if self.shape[1] != other.shape[0] {
            return Err(TensorError::DimensionMismatch {
                expected: self.shape[1],
                actual: other.shape[0],
            });
        }

        Ok(())
    }

    /// Educational broadcast compatibility validation
    fn validate_broadcast_compatibility(&self, other: &EducationalTensor) -> Result<(), TensorError> {
        if self.shape != other.shape {
            // Simple validation - in practice would implement NumPy-style broadcasting
            return Err(TensorError::Educational(
                "Element-wise operations require same shape tensors".to_string()
            ));
        }

        Ok(())
    }

    /// Display tensor with educational formatting
    pub fn display_educational(&self) -> String {
        let mut output = format!("Tensor(shape: {:?}, size: {})\n", self.shape, self.size());
        
        if let Some(ref name) = self.metadata.name {
            output.push_str(&format!("Name: {}\n", name));
        }

        output.push_str("Data (first 10 elements):\n");
        for (i, &value) in self.data.iter().take(10).enumerate() {
            output.push_str(&format!("{:.4} ", value));
            if (i + 1) % 5 == 0 {
                output.push('\n');
            }
        }
        if self.data.len() > 10 {
            output.push_str(&format!("... ({} more elements)", self.data.len() - 10));
        }

        if !self.metadata.educational_notes.is_empty() {
            output.push_str("\nEducational Notes:\n");
            for note in &self.metadata.educational_notes {
                output.push_str(&format!("- {}\n", note));
            }
        }

        output
    }
}

impl Debug for EducationalTensor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Tensor({:?}, size: {})", self.shape, self.size())
    }
}

impl Display for EducationalTensor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_educational())
    }
}

impl Index<&[usize]> for EducationalTensor {
    type Output = f32;

    fn index(&self, indices: &[usize]) -> &Self::Output {
        &self.data[self.flatten_indices(indices)]
    }
}

impl IndexMut<&[usize]> for EducationalTensor {
    fn index_mut(&mut self, indices: &[usize]) -> &mut Self::Output {
        let linear_index = self.flatten_indices(indices);
        &mut self.data[linear_index]
    }
}

/// Educational random number generator
#[derive(Debug)]
struct EducationalRng {
    state: u64,
}

impl EducationalRng {
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    pub fn next_f32(&mut self) -> f32 {
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1);
        let value = (self.state >> 32) as u32;
        (value as f32) / (u32::MAX as f32)
    }
}

/// Error types with educational context
#[derive(Debug, thiserror::Error)]
pub enum TensorError {
    #[error("Dimension mismatch: expected {expected}, got {actual}")]
    DimensionMismatch { expected: usize, actual: usize },
    
    #[error("Index out of bounds: dimension {dimension}, index {index}, max {max}")]
    IndexOutOfBounds { dimension: usize, index: usize, max: usize },
    
    #[error("Educational error: {0}")]
    Educational(String),
    
    #[error("Memory error: {0}")]
    MemoryError(String),
}

use super::OperationMetadata;