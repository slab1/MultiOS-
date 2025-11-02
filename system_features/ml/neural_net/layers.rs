//! Neural Network Layers for Educational ML
//! 
//! Provides educational implementations of common neural network layers
//! with debugging, visualization, and learning features.

use super::super::runtime::tensor::EducationalTensor;
use serde::{Deserialize, Serialize};

/// Educational Dense Layer Implementation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DenseLayer {
    pub input_size: usize,
    pub output_size: usize,
    pub weights: Vec<Vec<f32>>,
    pub biases: Vec<f32>,
    pub activation: super::ActivationType,
    pub educational_metadata: LayerEducationalMetadata,
    pub forward_cache: Option<ForwardCache>,
    pub gradient_cache: Option<GradientCache>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerEducationalMetadata {
    pub layer_id: String,
    pub learning_objectives: Vec<String>,
    pub prerequisite_concepts: Vec<String>,
    pub common_mistakes: Vec<String>,
    pub debugging_tips: Vec<String>,
    pub visualization_hints: Vec<String>,
    pub parameter_analysis: ParameterAnalysis,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterAnalysis {
    pub weight_initialization: InitializationStrategy,
    pub gradient_flow_health: GradientHealth,
    pub computational_complexity: ComplexityAnalysis,
    pub memory_efficiency: MemoryEfficiency,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InitializationStrategy {
    Xavier,
    He,
    Random,
    Constant,
    Educational,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GradientHealth {
    pub health_score: f32,
    pub issues: Vec<GradientIssue>,
    pub recommendations: Vec<HealthRecommendation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GradientIssue {
    pub issue_type: GradientIssueType,
    pub severity: IssueSeverity,
    pub description: String,
    pub solution: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GradientIssueType {
    VanishingGradient,
    ExplodingGradient,
    DeadNeurons,
    UnstableTraining,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IssueSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthRecommendation {
    pub recommendation: String,
    pub implementation: String,
    pub expected_improvement: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityAnalysis {
    pub forward_complexity: String,
    pub backward_complexity: String,
    pub memory_complexity: String,
    pub educational_explanation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEfficiency {
    pub parameter_count: usize,
    pub memory_usage_kb: usize,
    pub efficiency_score: f32,
    pub optimization_suggestions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForwardCache {
    pub input: EducationalTensor,
    pub pre_activation: EducationalTensor,
    pub output: EducationalTensor,
    pub timestamp: std::time::SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GradientCache {
    pub input_gradient: EducationalTensor,
    pub weight_gradients: Vec<Vec<f32>>,
    pub bias_gradients: Vec<f32>,
    pub output_gradient: EducationalTensor,
    pub timestamp: std::time::SystemTime,
}

/// Educational Convolutional Layer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvLayer {
    pub input_channels: usize,
    pub output_channels: usize,
    pub kernel_size: usize,
    pub stride: usize,
    pub padding: usize,
    pub weights: Vec<Vec<Vec<Vec<f32>>>>, // [output][input][height][width]
    pub biases: Vec<f32>,
    pub activation: super::ActivationType,
    pub educational_metadata: LayerEducationalMetadata,
    pub forward_cache: Option<ConvForwardCache>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvForwardCache {
    pub input: Vec<Vec<Vec<f32>>>, // [batch][height][width]
    pub convolved: Vec<Vec<Vec<f32>>>,
    pub output: Vec<Vec<Vec<f32>>>,
    pub timestamp: std::time::SystemTime,
}

/// Educational Pooling Layer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaxPoolLayer {
    pub pool_size: usize,
    pub stride: usize,
    pub educational_metadata: LayerEducationalMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvgPoolLayer {
    pub pool_size: usize,
    pub stride: usize,
    pub educational_metadata: LayerEducationalMetadata,
}

/// Educational Dropout Layer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DropoutLayer {
    pub rate: f32,
    pub mask: Option<Vec<bool>>,
    pub educational_metadata: LayerEducationalMetadata,
}

/// Educational Batch Normalization Layer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchNormLayer {
    pub num_features: usize,
    pub epsilon: f32,
    pub momentum: f32,
    pub gamma: Vec<f32>,
    pub beta: Vec<f32>,
    pub running_mean: Vec<f32>,
    pub running_var: Vec<f32>,
    pub training: bool,
    pub educational_metadata: LayerEducationalMetadata,
}

impl DenseLayer {
    /// Create a new educational dense layer
    pub fn new(
        input_size: usize,
        output_size: usize,
        activation: super::ActivationType,
        initialization: InitializationStrategy,
    ) -> Self {
        let weights = Self::initialize_weights(input_size, output_size, &initialization);
        let biases = vec![0.0; output_size];

        let educational_metadata = LayerEducationalMetadata {
            layer_id: format!("dense_{}_{}", input_size, output_size),
            learning_objectives: vec![
                "Understanding linear transformations".to_string(),
                "Matrix multiplication in neural networks".to_string(),
                "Activation function application".to_string(),
            ],
            prerequisite_concepts: vec![
                "Linear algebra basics".to_string(),
                "Matrix operations".to_string(),
                "Function composition".to_string(),
            ],
            common_mistakes: vec![
                "Forgetting to add biases".to_string(),
                "Dimension mismatches".to_string(),
                "Incorrect weight initialization".to_string(),
            ],
            debugging_tips: vec![
                "Check input/output dimensions".to_string(),
                "Verify weight matrix shape".to_string(),
                "Monitor activation statistics".to_string(),
            ],
            visualization_hints: vec![
                "Weight matrix heatmap".to_string(),
                "Activation distribution".to_string(),
                "Gradient flow diagram".to_string(),
            ],
            parameter_analysis: ParameterAnalysis {
                weight_initialization: initialization,
                gradient_flow_health: GradientHealth {
                    health_score: 1.0,
                    issues: Vec::new(),
                    recommendations: Vec::new(),
                },
                computational_complexity: ComplexityAnalysis {
                    forward_complexity: format!("O({} × {})", input_size, output_size),
                    backward_complexity: format!("O({} × {})", input_size, output_size),
                    memory_complexity: format!("O({} + {})", input_size * output_size, output_size),
                    educational_explanation: "Dense layer multiplies input by weights and adds biases".to_string(),
                },
                memory_efficiency: MemoryEfficiency {
                    parameter_count: input_size * output_size + output_size,
                    memory_usage_kb: (input_size * output_size + output_size) * 4 / 1024,
                    efficiency_score: 1.0,
                    optimization_suggestions: vec!["Consider sparse connections for large layers".to_string()],
                },
            },
        };

        Self {
            input_size,
            output_size,
            weights,
            biases,
            activation,
            educational_metadata,
            forward_cache: None,
            gradient_cache: None,
        }
    }

    /// Initialize weights according to educational strategy
    fn initialize_weights(
        input_size: usize,
        output_size: usize,
        strategy: &InitializationStrategy,
    ) -> Vec<Vec<f32>> {
        let mut weights = Vec::with_capacity(output_size);
        
        match strategy {
            InitializationStrategy::Xavier => {
                let limit = (6.0 / (input_size + output_size) as f32).sqrt();
                for _ in 0..output_size {
                    let mut row = Vec::with_capacity(input_size);
                    for _ in 0..input_size {
                        row.push((rand::random::<f32>() - 0.5) * 2.0 * limit);
                    }
                    weights.push(row);
                }
            }
            InitializationStrategy::He => {
                let std = (2.0 / input_size as f32).sqrt();
                for _ in 0..output_size {
                    let mut row = Vec::with_capacity(input_size);
                    for _ in 0..input_size {
                        row.push(Self::generate_normal_random(std));
                    }
                    weights.push(row);
                }
            }
            InitializationStrategy::Random => {
                for _ in 0..output_size {
                    let mut row = Vec::with_capacity(input_size);
                    for _ in 0..input_size {
                        row.push((rand::random::<f32>() - 0.5) * 2.0);
                    }
                    weights.push(row);
                }
            }
            InitializationStrategy::Constant => {
                for _ in 0..output_size {
                    let mut row = Vec::with_capacity(input_size);
                    for _ in 0..input_size {
                        row.push(0.1);
                    }
                    weights.push(row);
                }
            }
            InitializationStrategy::Educational => {
                // Simple educational initialization
                for i in 0..output_size {
                    let mut row = Vec::with_capacity(input_size);
                    for j in 0..input_size {
                        row.push(if i == j { 1.0 } else { 0.0 });
                    }
                    weights.push(row);
                }
            }
        }

        weights
    }

    /// Generate normally distributed random number (simplified)
    fn generate_normal_random(std: f32) -> f32 {
        let u1: f32 = rand::random::<f32>().max(f32::EPSILON);
        let u2: f32 = rand::random::<f32>();
        let z0 = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f32::consts::PI * u2).cos();
        z0 * std
    }

    /// Educational forward pass
    pub fn educational_forward(
        &mut self,
        input: &[f32],
    ) -> Result<LayerForwardResult, LayerError> {
        // Validate input
        if input.len() != self.input_size {
            return Err(LayerError::DimensionMismatch {
                expected: self.input_size,
                actual: input.len(),
            });
        }

        // Compute pre-activation (z = Wx + b)
        let mut pre_activation = vec![0.0; self.output_size];
        for i in 0..self.output_size {
            for j in 0..self.input_size {
                pre_activation[i] += input[j] * self.weights[i][j];
            }
            pre_activation[i] += self.biases[i];
        }

        // Apply activation function
        let output = self.apply_activation(&pre_activation)?;

        // Create educational cache
        let forward_cache = ForwardCache {
            input: EducationalTensor::from_data(input, vec![self.input_size])?,
            pre_activation: EducationalTensor::from_data(&pre_activation, vec![self.output_size])?,
            output: EducationalTensor::from_data(&output, vec![self.output_size])?,
            timestamp: std::time::SystemTime::now(),
        };

        self.forward_cache = Some(forward_cache.clone());

        // Generate educational insights
        let educational_insights = self.generate_forward_insights(&pre_activation, &output);

        Ok(LayerForwardResult {
            output,
            cache: forward_cache,
            educational_insights,
            performance_metrics: self.calculate_performance_metrics(&pre_activation, &output),
        })
    }

    /// Apply activation function with educational validation
    fn apply_activation(&self, pre_activation: &[f32]) -> Result<Vec<f32>, LayerError> {
        match self.activation {
            super::ActivationType::ReLU => {
                Ok(pre_activation.iter().map(|&x| x.max(0.0)).collect())
            }
            super::ActivationType::Sigmoid => {
                Ok(pre_activation.iter().map(|&x| 1.0 / (1.0 + (-x).exp())).collect())
            }
            super::ActivationType::Tanh => {
                Ok(pre_activation.iter().map(|&x| x.tanh()).collect())
            }
            super::ActivationType::Softmax => {
                let max_val = pre_activation.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
                let mut exp_vals: Vec<f32> = pre_activation.iter().map(|&x| (x - max_val).exp()).collect();
                let sum: f32 = exp_vals.iter().sum();
                for val in &mut exp_vals {
                    *val /= sum;
                }
                Ok(exp_vals)
            }
            super::ActivationType::Linear => {
                Ok(pre_activation.to_vec())
            }
        }
    }

    /// Generate educational insights from forward pass
    fn generate_forward_insights(
        &self,
        pre_activation: &[f32],
        output: &[f32],
    ) -> LayerEducationalInsights {
        let activation_stats = self.calculate_activation_statistics(output);
        let weight_analysis = self.analyze_weight_patterns();
        let performance_notes = self.generate_performance_notes(pre_activation, output);

        LayerEducationalInsights {
            activation_analysis: activation_stats,
            weight_analysis,
            performance_notes,
            debugging_hints: self.generate_debugging_hints(pre_activation, output),
            optimization_suggestions: self.generate_optimization_suggestions(),
            learning_objectives_met: self.assess_learning_objectives(pre_activation, output),
        }
    }

    /// Calculate activation statistics
    fn calculate_activation_statistics(&self, activations: &[f32]) -> ActivationAnalysis {
        let mean = activations.iter().sum::<f32>() / activations.len() as f32;
        let variance = activations.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f32>() / activations.len() as f32;
        let std_dev = variance.sqrt();

        let min_val = activations.iter().fold(f32::INFINITY, |a, &b| a.min(b));
        let max_val = activations.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));

        let zero_count = activations.iter().filter(|&&x| x == 0.0).count();
        let zero_ratio = zero_count as f32 / activations.len() as f32;

        let positive_count = activations.iter().filter(|&&x| x > 0.0).count();
        let positive_ratio = positive_count as f32 / activations.len() as f32;

        ActivationAnalysis {
            mean,
            std_dev,
            min_value: min_val,
            max_value: max_val,
            zero_ratio,
            positive_ratio,
            activation_health: self.assess_activation_health(mean, std_dev, zero_ratio),
            educational_notes: self.generate_activation_notes(mean, std_dev, zero_ratio),
        }
    }

    /// Assess activation health
    fn assess_activation_health(&self, mean: f32, std_dev: f32, zero_ratio: f32) -> ActivationHealth {
        let mut score = 1.0;
        let mut issues = Vec::new();
        let mut recommendations = Vec::new();

        // Check for dead neurons
        if zero_ratio > 0.5 {
            score -= 0.3;
            issues.push("High ratio of dead neurons".to_string());
            recommendations.push("Consider using Leaky ReLU".to_string());
        }

        // Check for exploding activations
        if std_dev > 2.0 {
            score -= 0.2;
            issues.push("High activation variance".to_string());
            recommendations.push("Reduce learning rate".to_string());
        }

        // Check for vanishing activations
        if std_dev < 0.1 {
            score -= 0.2;
            issues.push("Low activation variance".to_string());
            recommendations.push("Check weight initialization".to_string());
        }

        ActivationHealth {
            score: score.max(0.0),
            issues,
            recommendations,
            overall_status: if score > 0.7 {
                HealthStatus::Good
            } else if score > 0.4 {
                HealthStatus::Fair
            } else {
                HealthStatus::Poor
            },
        }
    }

    /// Analyze weight patterns
    fn analyze_weight_patterns(&self) -> WeightPatternAnalysis {
        let mut weights_flat = Vec::new();
        for row in &self.weights {
            weights_flat.extend(row);
        }

        let mean = weights_flat.iter().sum::<f32>() / weights_flat.len() as f32;
        let variance = weights_flat.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f32>() / weights_flat.len() as f32;

        let large_weights: usize = weights_flat.iter().filter(|&&x| x.abs() > 2.0).count();
        let small_weights: usize = weights_flat.iter().filter(|&&x| x.abs() < 0.1).count();

        WeightPatternAnalysis {
            weight_statistics: WeightStatistics {
                mean,
                std_dev: variance.sqrt(),
                large_weight_ratio: large_weights as f32 / weights_flat.len() as f32,
                small_weight_ratio: small_weights as f32 / weights_flat.len() as f32,
            },
            pattern_classification: self.classify_weight_pattern(),
            educational_insights: self.generate_weight_insights(),
        }
    }

    /// Classify weight patterns
    fn classify_weight_pattern(&self) -> WeightPatternType {
        let mut weights_flat = Vec::new();
        for row in &self.weights {
            weights_flat.extend(row);
        }

        let mean = weights_flat.iter().sum::<f32>() / weights_flat.len() as f32;
        let std_dev = (weights_flat.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f32>() / weights_flat.len() as f32).sqrt();

        if std_dev < 0.1 {
            WeightPatternType::Uniform
        } else if mean.abs() > 1.0 {
            WeightPatternType::Biased
        } else {
            WeightPatternType::Normal
        }
    }

    /// Generate performance notes
    fn generate_performance_notes(
        &self,
        pre_activation: &[f32],
        output: &[f32],
    ) -> Vec<PerformanceNote> {
        vec![
            PerformanceNote {
                note_type: "computation".to_string(),
                content: format!(
                    "Computed {} multiplications and {} additions",
                    self.input_size * self.output_size,
                    self.input_size * self.output_size + self.output_size
                ),
                significance: PerformanceSignificance::Medium,
            },
            PerformanceNote {
                note_type: "activation".to_string(),
                content: format!("Applied {} activation to {} neurons", 
                    format!("{:?}", self.activation), 
                    self.output_size),
                significance: PerformanceSignificance::High,
            },
        ]
    }

    /// Generate debugging hints
    fn generate_debugging_hints(
        &self,
        pre_activation: &[f32],
        output: &[f32],
    ) -> Vec<DebuggingHint> {
        let mut hints = Vec::new();

        // Check for NaN or infinite values
        let has_nan = pre_activation.iter().any(|&x| x.is_nan()) || output.iter().any(|&x| x.is_nan());
        let has_inf = pre_activation.iter().any(|&x| x.is_infinite()) || output.iter().any(|&x| x.is_infinite());

        if has_nan {
            hints.push(DebuggingHint {
                hint_type: "numerical_stability".to_string(),
                severity: HintSeverity::High,
                description: "NaN values detected in activations".to_string(),
                solution: "Check for division by zero or invalid operations".to_string(),
                educational_value: "Understanding numerical stability in ML".to_string(),
            });
        }

        if has_inf {
            hints.push(DebuggingHint {
                hint_type: "numerical_stability".to_string(),
                severity: HintSeverity::High,
                description: "Infinite values detected".to_string(),
                solution: "Check for overflow or invalid exponential operations".to_string(),
                educational_value: "Understanding numerical limits".to_string(),
            });
        }

        hints
    }

    /// Generate optimization suggestions
    fn generate_optimization_suggestions(&self) -> Vec<OptimizationSuggestion> {
        vec![
            OptimizationSuggestion {
                suggestion: "Consider weight regularization".to_string(),
                category: OptimizationCategory::Regularization,
                expected_benefit: "Improved generalization".to_string(),
                implementation_effort: ImplementationEffort::Medium,
                educational_benefit: "Understanding regularization techniques".to_string(),
            },
        ]
    }

    /// Assess learning objectives
    fn assess_learning_objectives(
        &self,
        pre_activation: &[f32],
        output: &[f32],
    ) -> HashMap<String, LearningObjectiveStatus> {
        let mut objectives = HashMap::new();

        // Linear transformation understanding
        let linear_correct = pre_activation.len() == self.output_size && 
            pre_activation.iter().all(|&x| x.is_finite());
        objectives.insert(
            "linear_transformation".to_string(),
            LearningObjectiveStatus {
                achieved: linear_correct,
                confidence: if linear_correct { 0.9 } else { 0.3 },
                evidence: if linear_correct {
                    "Correct linear transformation computed".to_string()
                } else {
                    "Linear transformation may have issues".to_string()
                },
            }
        );

        // Activation function application
        let activation_applied = output.len() == pre_activation.len() &&
            output.iter().all(|&x| x.is_finite());
        objectives.insert(
            "activation_application".to_string(),
            LearningObjectiveStatus {
                achieved: activation_applied,
                confidence: if activation_applied { 0.9 } else { 0.2 },
                evidence: if activation_applied {
                    "Activation function applied correctly".to_string()
                } else {
                    "Activation function may have issues".to_string()
                },
            }
        );

        objectives
    }

    /// Calculate performance metrics
    fn calculate_performance_metrics(
        &self,
        pre_activation: &[f32],
        output: &[f32],
    ) -> LayerPerformanceMetrics {
        LayerPerformanceMetrics {
            computation_time: std::time::Duration::from_millis(1), // Simplified
            memory_accesses: self.input_size * self.output_size + self.output_size,
            numerical_stability_score: self.calculate_stability_score(pre_activation, output),
            educational_value_score: 0.8,
        }
    }

    /// Calculate numerical stability score
    fn calculate_stability_score(&self, pre_activation: &[f32], output: &[f32]) -> f32 {
        let mut score = 1.0;

        // Check for NaN or infinite values
        if pre_activation.iter().any(|&x| x.is_nan() || x.is_infinite()) ||
           output.iter().any(|&x| x.is_nan() || x.is_infinite()) {
            score = 0.0;
        }

        score
    }

    /// Generate activation notes
    fn generate_activation_notes(&self, mean: f32, std_dev: f32, zero_ratio: f32) -> Vec<String> {
        let mut notes = Vec::new();

        if zero_ratio > 0.5 {
            notes.push("High ratio of zero activations - may indicate dead ReLU problem".to_string());
        }

        if std_dev > 2.0 {
            notes.push("High variance in activations - consider gradient clipping".to_string());
        }

        if mean.abs() < 0.1 && std_dev < 0.5 {
            notes.push("Low activation values - may indicate vanishing gradients".to_string());
        }

        if notes.is_empty() {
            notes.push("Healthy activation distribution".to_string());
        }

        notes
    }

    /// Generate weight insights
    fn generate_weight_insights(&self) -> Vec<String> {
        vec![
            "Weight values show the learned features".to_string(),
            "Weight magnitudes indicate feature importance".to_string(),
            "Regular weight patterns suggest good learning".to_string(),
        ]
    }
}

/// Supporting types and enums
#[derive(Debug, Clone)]
pub struct LayerForwardResult {
    pub output: Vec<f32>,
    pub cache: ForwardCache,
    pub educational_insights: LayerEducationalInsights,
    pub performance_metrics: LayerPerformanceMetrics,
}

#[derive(Debug, Clone)]
pub struct LayerEducationalInsights {
    pub activation_analysis: ActivationAnalysis,
    pub weight_analysis: WeightPatternAnalysis,
    pub performance_notes: Vec<PerformanceNote>,
    pub debugging_hints: Vec<DebuggingHint>,
    pub optimization_suggestions: Vec<OptimizationSuggestion>,
    pub learning_objectives_met: HashMap<String, LearningObjectiveStatus>,
}

#[derive(Debug, Clone)]
pub struct ActivationAnalysis {
    pub mean: f32,
    pub std_dev: f32,
    pub min_value: f32,
    pub max_value: f32,
    pub zero_ratio: f32,
    pub positive_ratio: f32,
    pub activation_health: ActivationHealth,
    pub educational_notes: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ActivationHealth {
    pub score: f32,
    pub issues: Vec<String>,
    pub recommendations: Vec<String>,
    pub overall_status: HealthStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Good,
    Fair,
    Poor,
    Critical,
}

#[derive(Debug, Clone)]
pub struct WeightPatternAnalysis {
    pub weight_statistics: WeightStatistics,
    pub pattern_classification: WeightPatternType,
    pub educational_insights: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WeightPatternType {
    Normal,
    Uniform,
    Biased,
    Sparse,
    Educational,
}

#[derive(Debug, Clone)]
pub struct WeightStatistics {
    pub mean: f32,
    pub std_dev: f32,
    pub large_weight_ratio: f32,
    pub small_weight_ratio: f32,
}

#[derive(Debug, Clone)]
pub struct PerformanceNote {
    pub note_type: String,
    pub content: String,
    pub significance: PerformanceSignificance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerformanceSignificance {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone)]
pub struct DebuggingHint {
    pub hint_type: String,
    pub severity: HintSeverity,
    pub description: String,
    pub solution: String,
    pub educational_value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HintSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone)]
pub struct OptimizationSuggestion {
    pub suggestion: String,
    pub category: OptimizationCategory,
    pub expected_benefit: String,
    pub implementation_effort: ImplementationEffort,
    pub educational_benefit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationCategory {
    Performance,
    Memory,
    Regularization,
    Architecture,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImplementationEffort {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone)]
pub struct LearningObjectiveStatus {
    pub achieved: bool,
    pub confidence: f32,
    pub evidence: String,
}

#[derive(Debug, Clone)]
pub struct LayerPerformanceMetrics {
    pub computation_time: std::time::Duration,
    pub memory_accesses: usize,
    pub numerical_stability_score: f32,
    pub educational_value_score: f32,
}

#[derive(Debug, thiserror::Error)]
pub enum LayerError {
    #[error("Dimension mismatch: expected {expected}, got {actual}")]
    DimensionMismatch { expected: usize, actual: usize },
    
    #[error("Invalid activation function: {0}")]
    InvalidActivation(String),
    
    #[error("Numerical instability: {0}")]
    NumericalError(String),
    
    #[error("Educational validation error: {0}")]
    Educational(String),
}