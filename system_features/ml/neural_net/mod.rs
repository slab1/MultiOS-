//! Educational Neural Network Library
//! 
//! Provides educational neural network implementations with visual debugging
//! and step-by-step learning features.

pub mod layers;
pub mod models;
pub mod visualization;
pub mod utils;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Educational Neural Network Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EducationalNNConfig {
    pub learning_rate: f32,
    pub batch_size: usize,
    pub epochs: usize,
    pub educational_mode: bool,
    pub visualization_enabled: bool,
    pub step_by_step: bool,
    pub auto_grad: bool,
}

impl Default for EducationalNNConfig {
    fn default() -> Self {
        Self {
            learning_rate: 0.01,
            batch_size: 32,
            epochs: 10,
            educational_mode: true,
            visualization_enabled: true,
            step_by_step: true,
            auto_grad: true,
        }
    }
}

/// Educational Neural Network Layer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EducationalLayer {
    Dense {
        input_size: usize,
        output_size: usize,
        activation: ActivationType,
        weights: Option<Vec<Vec<f32>>>,
        biases: Option<Vec<f32>>,
    },
    Conv2D {
        input_channels: usize,
        output_channels: usize,
        kernel_size: usize,
        stride: usize,
        padding: usize,
        activation: ActivationType,
    },
    MaxPool2D {
        pool_size: usize,
        stride: usize,
    },
    Flatten,
    Dropout {
        rate: f32,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActivationType {
    ReLU,
    Sigmoid,
    Tanh,
    Softmax,
    Linear,
}

/// Educational Neural Network Model
#[derive(Debug, Clone)]
pub struct EducationalModel {
    pub layers: Vec<EducationalLayer>,
    pub config: EducationalNNConfig,
    pub forward_cache: Vec<ForwardCacheEntry>,
    pub gradient_cache: Vec<GradientCacheEntry>,
    pub visualization_data: VisualizationData,
    pub educational_metadata: EducationalMetadata,
}

#[derive(Debug, Clone)]
pub struct ForwardCacheEntry {
    pub layer_index: usize,
    pub input: Vec<f32>,
    pub output: Vec<f32>,
    pub activation_applied: bool,
    pub timestamp: std::time::SystemTime,
}

#[derive(Debug, Clone)]
pub struct GradientCacheEntry {
    pub layer_index: usize,
    pub input_gradient: Vec<f32>,
    pub output_gradient: Vec<f32>,
    pub parameter_gradients: Option<ParameterGradients>,
}

#[derive(Debug, Clone)]
pub struct ParameterGradients {
    pub weight_gradients: Vec<Vec<f32>>,
    pub bias_gradients: Vec<f32>,
}

#[derive(Debug, Clone, Default)]
pub struct VisualizationData {
    pub layer_activations: Vec<Vec<f32>>,
    pub gradient_flows: Vec<Vec<f32>>,
    pub weight_matrices: Vec<Vec<Vec<f32>>>,
    pub performance_metrics: PerformanceMetrics,
    pub educational_annotations: Vec<Annotation>,
}

#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub forward_pass_time: std::time::Duration,
    pub backward_pass_time: std::time::Duration,
    pub parameter_update_time: std::time::Duration,
    pub memory_usage_mb: usize,
    pub accuracy: f32,
    pub loss: f32,
}

#[derive(Debug, Clone)]
pub struct Annotation {
    pub layer_index: usize,
    pub annotation_type: AnnotationType,
    pub content: String,
    pub timestamp: std::time::SystemTime,
}

#[derive(Debug, Clone)]
pub enum AnnotationType {
    WeightUpdate,
    ActivationPeak,
    GradientExplosion,
    DeadNeuron,
    OptimizationSuggestion,
}

#[derive(Debug, Clone, Default)]
pub struct EducationalMetadata {
    pub model_complexity: ModelComplexity,
    pub learning_objectives: Vec<String>,
    pub prerequisite_concepts: Vec<String>,
    pub estimated_understanding_time: std::time::Duration,
    pub interactive_features: Vec<InteractiveFeature>,
}

#[derive(Debug, Clone)]
pub enum ModelComplexity {
    Simple,
    Moderate,
    Complex,
    Advanced,
}

#[derive(Debug, Clone)]
pub enum InteractiveFeature {
    WeightSlider,
    ActivationHeatmap,
    GradientVisualizer,
    ParameterInspector,
    PerformanceProfiler,
}

/// Educational Forward Pass Result
#[derive(Debug, Clone)]
pub struct EducationalForwardResult {
    pub output: Vec<f32>,
    pub forward_cache: Vec<ForwardCacheEntry>,
    pub visualization_data: VisualizationData,
    pub educational_insights: EducationalInsights,
    pub performance_metrics: PerformanceMetrics,
}

#[derive(Debug, Clone)]
pub struct EducationalInsights {
    pub layer_analysis: Vec<LayerAnalysis>,
    pub activation_patterns: Vec<ActivationPattern>,
    pub performance_notes: Vec<String>,
    pub learning_suggestions: Vec<String>,
    pub common_mistakes: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct LayerAnalysis {
    pub layer_index: usize,
    pub layer_type: String,
    pub activation_stats: ActivationStats,
    pub parameter_count: usize,
    pub computational_complexity: String,
    pub educational_notes: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ActivationPattern {
    pub layer_index: usize,
    pub pattern_type: PatternType,
    pub description: String,
    pub significance: String,
    pub optimization_opportunities: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum PatternType {
    Normal,
    Saturated,
    Dead,
    Exploding,
    Vanishing,
}

#[derive(Debug, Clone)]
pub struct ActivationStats {
    pub mean: f32,
    pub std_dev: f32,
    pub min_value: f32,
    pub max_value: f32,
    pub zero_ratio: f32,
    pub active_neurons: usize,
}

impl EducationalModel {
    /// Create a new educational neural network model
    pub fn new(layers: Vec<EducationalLayer>, config: EducationalNNConfig) -> Self {
        let mut model = Self {
            layers,
            config,
            forward_cache: Vec::new(),
            gradient_cache: Vec::new(),
            visualization_data: VisualizationData::default(),
            educational_metadata: EducationalMetadata::default(),
        };

        // Initialize educational metadata
        model.initialize_educational_metadata();
        model
    }

    /// Educational forward pass with step-by-step analysis
    pub fn educational_forward(
        &mut self,
        input: &[f32],
    ) -> Result<EducationalForwardResult, NNEducationError> {
        let start_time = std::time::Instant::now();
        let mut current_input = input.to_vec();
        let mut forward_cache = Vec::new();
        let mut layer_activations = Vec::new();

        // Clear previous cache
        self.forward_cache.clear();
        self.visualization_data.layer_activations.clear();

        // Step-by-step forward pass
        for (layer_index, layer) in self.layers.iter().enumerate() {
            self.debug_layer_start(layer_index, layer, &current_input);

            let (output, cache_entry) = self.process_layer(layer_index, layer, &current_input)?;
            
            // Store for visualization
            forward_cache.push(cache_entry.clone());
            layer_activations.push(output.clone());
            
            // Update for next layer
            current_input = output;

            self.debug_layer_end(layer_index, &cache_entry);
        }

        let forward_time = start_time.elapsed();

        // Generate educational insights
        let insights = self.generate_educational_insights(&layer_activations);
        
        // Update visualization data
        self.update_visualization_data(&layer_activations, &forward_cache);

        // Create result
        Ok(EducationalForwardResult {
            output: current_input,
            forward_cache,
            visualization_data: self.visualization_data.clone(),
            educational_insights: insights,
            performance_metrics: PerformanceMetrics {
                forward_pass_time: forward_time,
                backward_pass_time: std::time::Duration::from_nanos(0), // Set during backward pass
                parameter_update_time: std::time::Duration::from_nanos(0), // Set during optimization
                memory_usage_mb: self.estimate_memory_usage(),
                accuracy: 0.0, // Would be calculated during training
                loss: 0.0, // Would be calculated during training
            },
        })
    }

    /// Process individual layer with educational tracking
    fn process_layer(
        &self,
        layer_index: usize,
        layer: &EducationalLayer,
        input: &[f32],
    ) -> Result<(Vec<f32>, ForwardCacheEntry), NNEducationError> {
        match layer {
            EducationalLayer::Dense {
                input_size,
                output_size,
                activation,
                weights,
                biases,
            } => self.process_dense_layer(layer_index, input_size, output_size, activation, weights, biases, input),
            
            EducationalLayer::Conv2D { .. } => {
                // Simplified convolution for educational purposes
                self.process_conv2d_layer(layer_index, layer, input)
            }
            
            EducationalLayer::MaxPool2D { .. } => {
                self.process_maxpool_layer(layer_index, layer, input)
            }
            
            EducationalLayer::Flatten => {
                self.process_flatten_layer(layer_index, input)
            }
            
            EducationalLayer::Dropout { rate } => {
                self.process_dropout_layer(layer_index, rate, input)
            }
        }
    }

    /// Process dense layer with educational analysis
    fn process_dense_layer(
        &self,
        layer_index: usize,
        input_size: &usize,
        output_size: &usize,
        activation: &ActivationType,
        weights: &Option<Vec<Vec<f32>>>,
        biases: &Option<Vec<f32>>,
        input: &[f32],
    ) -> Result<(Vec<f32>, ForwardCacheEntry), NNEducationError> {
        // Educational validation
        if input.len() != *input_size {
            return Err(NNEducationError::DimensionMismatch {
                expected: *input_size,
                actual: input.len(),
            });
        }

        // Initialize weights if not provided
        let weights = match weights {
            Some(w) => w.clone(),
            None => self.initialize_weights(*input_size, *output_size),
        };

        let biases = match biases {
            Some(b) => b.clone(),
            None => vec![0.0; *output_size],
        };

        // Perform matrix multiplication with educational tracing
        let mut output = vec![0.0; *output_size];
        
        // Educational step-by-step multiplication
        for i in 0..*output_size {
            let mut sum = biases[i];
            for j in 0..*input_size {
                sum += input[j] * weights[i][j];
                
                // Educational tracking for first few operations
                if i < 2 && j < 3 {
                    println!("Educational: Computing output[{}] += input[{}] * weight[{}][{}] = {} * {} = {}", 
                            i, j, i, j, input[j], weights[i][j], input[j] * weights[i][j]);
                }
            }
            output[i] = sum;
        }

        // Apply activation function
        let activation_output = self.apply_activation(activation, &output)?;
        let activation_applied = matches!(activation, ActivationType::ReLU | ActivationType::Sigmoid | ActivationType::Tanh | ActivationType::Softmax);

        // Create cache entry
        let cache_entry = ForwardCacheEntry {
            layer_index,
            input: input.to_vec(),
            output: activation_output.clone(),
            activation_applied,
            timestamp: std::time::SystemTime::now(),
        };

        Ok((activation_output, cache_entry))
    }

    /// Process convolution layer (simplified for education)
    fn process_conv2d_layer(
        &self,
        layer_index: usize,
        layer: &EducationalLayer,
        input: &[f32],
    ) -> Result<(Vec<f32>, ForwardCacheEntry), NNEducationError> {
        // Simplified 2D convolution for educational purposes
        let output = input.to_vec(); // Placeholder
        
        let cache_entry = ForwardCacheEntry {
            layer_index,
            input: input.to_vec(),
            output: output.clone(),
            activation_applied: true,
            timestamp: std::time::SystemTime::now(),
        };

        Ok((output, cache_entry))
    }

    /// Process max pooling layer
    fn process_maxpool_layer(
        &self,
        layer_index: usize,
        layer: &EducationalLayer,
        input: &[f32],
    ) -> Result<(Vec<f32>, ForwardCacheEntry), NNEducationError> {
        // Simplified max pooling
        let output = input.to_vec(); // Placeholder
        
        let cache_entry = ForwardCacheEntry {
            layer_index,
            input: input.to_vec(),
            output: output.clone(),
            activation_applied: false,
            timestamp: std::time::SystemTime::now(),
        };

        Ok((output, cache_entry))
    }

    /// Process flatten layer
    fn process_flatten_layer(
        &self,
        layer_index: usize,
        input: &[f32],
    ) -> Result<(Vec<f32>, ForwardCacheEntry), NNEducationError> {
        let cache_entry = ForwardCacheEntry {
            layer_index,
            input: input.to_vec(),
            output: input.to_vec(),
            activation_applied: false,
            timestamp: std::time::SystemTime::now(),
        };

        Ok((input.to_vec(), cache_entry))
    }

    /// Process dropout layer
    fn process_dropout_layer(
        &self,
        layer_index: usize,
        rate: &f32,
        input: &[f32],
    ) -> Result<(Vec<f32>, ForwardCacheEntry), NNEducationError> {
        // Simplified dropout (in practice would use random masks)
        let output = input.to_vec(); // Placeholder
        
        let cache_entry = ForwardCacheEntry {
            layer_index,
            input: input.to_vec(),
            output: output.clone(),
            activation_applied: false,
            timestamp: std::time::SystemTime::now(),
        };

        Ok((output, cache_entry))
    }

    /// Apply activation function with educational notes
    fn apply_activation(
        &self,
        activation: &ActivationType,
        input: &[f32],
    ) -> Result<Vec<f32>, NNEducationError> {
        match activation {
            ActivationType::ReLU => {
                let mut output = Vec::with_capacity(input.len());
                for &x in input {
                    output.push(x.max(0.0));
                }
                Ok(output)
            }
            ActivationType::Sigmoid => {
                let mut output = Vec::with_capacity(input.len());
                for &x in input {
                    output.push(1.0 / (1.0 + (-x).exp()));
                }
                Ok(output)
            }
            ActivationType::Tanh => {
                let mut output = Vec::with_capacity(input.len());
                for &x in input {
                    output.push(x.tanh());
                }
                Ok(output)
            }
            ActivationType::Softmax => {
                // Simplified softmax for educational purposes
                let max_val = input.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
                let mut exp_vals: Vec<f32> = input.iter().map(|&x| (x - max_val).exp()).collect();
                let sum: f32 = exp_vals.iter().sum();
                for val in &mut exp_vals {
                    *val /= sum;
                }
                Ok(exp_vals)
            }
            ActivationType::Linear => Ok(input.to_vec()),
        }
    }

    /// Initialize weights with educational options
    fn initialize_weights(&self, input_size: usize, output_size: usize) -> Vec<Vec<f32>> {
        let mut weights = Vec::with_capacity(output_size);
        
        // Xavier/Glorot initialization for educational purposes
        let limit = (6.0 / (input_size + output_size) as f32).sqrt();
        
        for _ in 0..output_size {
            let mut row = Vec::with_capacity(input_size);
            for _ in 0..input_size {
                // Simplified random initialization
                row.push((rand::random::<f32>() - 0.5) * 2.0 * limit);
            }
            weights.push(row);
        }
        
        weights
    }

    /// Debug layer start with educational information
    fn debug_layer_start(&self, layer_index: usize, layer: &EducationalLayer, input: &[f32]) {
        println!("Educational: Processing layer {}: {:?}", layer_index, self.get_layer_description(layer));
        println!("Educational: Input shape: [{}]", input.len());
    }

    /// Debug layer end with results
    fn debug_layer_end(&self, layer_index: usize, cache_entry: &ForwardCacheEntry) {
        let output_stats = self.calculate_activation_stats(&cache_entry.output);
        println!("Educational: Layer {} output stats: mean={:.4}, std={:.4}, active_ratio={:.2}", 
                layer_index, output_stats.mean, output_stats.std_dev, output_stats.active_neurons as f32 / cache_entry.output.len() as f32);
    }

    /// Get educational layer description
    fn get_layer_description(&self, layer: &EducationalLayer) -> String {
        match layer {
            EducationalLayer::Dense { input_size, output_size, activation, .. } => {
                format!("Dense({}, {}, {:?})", input_size, output_size, activation)
            }
            EducationalLayer::Conv2D { input_channels, output_channels, kernel_size, .. } => {
                format!("Conv2D({}->{} channels, {}x{} kernel)", input_channels, output_channels, kernel_size, kernel_size)
            }
            EducationalLayer::MaxPool2D { pool_size, stride } => {
                format!("MaxPool2D({}x{}, stride={})", pool_size, pool_size, stride)
            }
            EducationalLayer::Flatten => "Flatten".to_string(),
            EducationalLayer::Dropout { rate } => {
                format!("Dropout({:.1}%)", rate * 100.0)
            }
        }
    }

    /// Calculate activation statistics
    fn calculate_activation_stats(&self, activations: &[f32]) -> ActivationStats {
        if activations.is_empty() {
            return ActivationStats {
                mean: 0.0,
                std_dev: 0.0,
                min_value: 0.0,
                max_value: 0.0,
                zero_ratio: 0.0,
                active_neurons: 0,
            };
        }

        let mean = activations.iter().sum::<f32>() / activations.len() as f32;
        let variance = activations.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f32>() / activations.len() as f32;
        let std_dev = variance.sqrt();

        let min_val = activations.iter().fold(f32::INFINITY, |a, &b| a.min(b));
        let max_val = activations.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
        
        let zero_count = activations.iter().filter(|&&x| x == 0.0).count();
        let zero_ratio = zero_count as f32 / activations.len() as f32;
        
        let active_neurons = activations.iter().filter(|&&x| x != 0.0).count();

        ActivationStats {
            mean,
            std_dev,
            min_value: min_val,
            max_value: max_val,
            zero_ratio,
            active_neurons,
        }
    }

    /// Generate educational insights from layer activations
    fn generate_educational_insights(&self, layer_activations: &[Vec<f32>]) -> EducationalInsights {
        let mut layer_analysis = Vec::new();
        let mut activation_patterns = Vec::new();

        for (layer_index, activations) in layer_activations.iter().enumerate() {
            let stats = self.calculate_activation_stats(activations);
            
            // Analyze activation patterns
            let pattern = self.analyze_activation_pattern(activations, layer_index);
            activation_patterns.push(pattern);

            // Create layer analysis
            layer_analysis.push(LayerAnalysis {
                layer_index,
                layer_type: self.get_layer_type(layer_index),
                activation_stats: stats,
                parameter_count: self.estimate_parameter_count(layer_index),
                computational_complexity: self.get_computational_complexity(layer_index),
                educational_notes: self.generate_layer_notes(layer_index, &stats),
            });
        }

        EducationalInsights {
            layer_analysis,
            activation_patterns,
            performance_notes: self.generate_performance_notes(),
            learning_suggestions: self.generate_learning_suggestions(),
            common_mistakes: self.identify_common_mistakes(),
        }
    }

    /// Analyze activation patterns for educational insights
    fn analyze_activation_pattern(&self, activations: &[f32], layer_index: usize) -> ActivationPattern {
        let stats = self.calculate_activation_stats(activations);
        
        let pattern_type = if stats.zero_ratio > 0.9 {
            PatternType::Dead
        } else if stats.std_dev < 0.1 {
            PatternType::Saturated
        } else if stats.std_dev > 2.0 {
            PatternType::Exploding
        } else if stats.mean.abs() < 0.1 && stats.std_dev < 0.5 {
            PatternType::Vanishing
        } else {
            PatternType::Normal
        };

        let (description, significance, optimizations) = match pattern_type {
            PatternType::Dead => (
                "Many neurons are not activating".to_string(),
                "This may indicate a dead ReLU problem".to_string(),
                vec!["Consider using Leaky ReLU".to_string(), "Check weight initialization".to_string()],
            ),
            PatternType::Saturated => (
                "Activation values are clustering".to_string(),
                "Saturated activations can slow learning".to_string(),
                vec!["Consider different activation function".to_string(), "Adjust learning rate".to_string()],
            ),
            PatternType::Exploding => (
                "Activation values are growing very large".to_string(),
                "Exploding gradients can cause instability".to_string(),
                vec!["Use gradient clipping".to_string(), "Check weight initialization".to_string()],
            ),
            PatternType::Vanishing => (
                "Activation values are very small".to_string(),
                "Vanishing gradients can slow learning".to_string(),
                vec!["Consider residual connections".to_string(), "Check activation function".to_string()],
            ),
            PatternType::Normal => (
                "Healthy activation distribution".to_string(),
                "Good sign for training convergence".to_string(),
                vec!["Continue current approach".to_string()],
            ),
        };

        ActivationPattern {
            layer_index,
            pattern_type,
            description,
            significance,
            optimization_opportunities: optimizations,
        }
    }

    /// Update visualization data
    fn update_visualization_data(&mut self, layer_activations: &[Vec<f32>], forward_cache: &[ForwardCacheEntry]) {
        self.visualization_data.layer_activations = layer_activations.clone();
        
        // Add educational annotations
        for (layer_index, activations) in layer_activations.iter().enumerate() {
            let stats = self.calculate_activation_stats(activations);
            
            if stats.zero_ratio > 0.5 {
                self.visualization_data.educational_annotations.push(Annotation {
                    layer_index,
                    annotation_type: AnnotationType::DeadNeuron,
                    content: format!("Warning: {:.1}% dead neurons detected", stats.zero_ratio * 100.0),
                    timestamp: std::time::SystemTime::now(),
                });
            }
        }
    }

    /// Initialize educational metadata
    fn initialize_educational_metadata(&mut self) {
        self.educational_metadata.model_complexity = self.assess_model_complexity();
        self.educational_metadata.learning_objectives = self.generate_learning_objectives();
        self.educational_metadata.prerequisite_concepts = self.identify_prerequisites();
        self.educational_metadata.estimated_understanding_time = self.estimate_understanding_time();
        self.educational_metadata.interactive_features = self.suggest_interactive_features();
    }

    /// Helper methods for educational analysis
    fn get_layer_type(&self, layer_index: usize) -> String {
        match &self.layers[layer_index] {
            EducationalLayer::Dense { .. } => "Dense".to_string(),
            EducationalLayer::Conv2D { .. } => "Convolutional".to_string(),
            EducationalLayer::MaxPool2D { .. } => "MaxPooling".to_string(),
            EducationalLayer::Flatten => "Flatten".to_string(),
            EducationalLayer::Dropout { .. } => "Dropout".to_string(),
        }
    }

    fn estimate_parameter_count(&self, layer_index: usize) -> usize {
        match &self.layers[layer_index] {
            EducationalLayer::Dense { input_size, output_size, .. } => input_size * output_size + output_size,
            EducationalLayer::Conv2D { input_channels, output_channels, kernel_size, .. } => {
                input_channels * output_channels * kernel_size * kernel_size + output_channels
            }
            _ => 0,
        }
    }

    fn get_computational_complexity(&self, layer_index: usize) -> String {
        match &self.layers[layer_index] {
            EducationalLayer::Dense { input_size, output_size, .. } => {
                format!("O({} × {})", input_size, output_size)
            }
            EducationalLayer::Conv2D { input_channels, output_channels, kernel_size, .. } => {
                format!("O({} × {} × {}²)", input_channels, output_channels, kernel_size)
            }
            _ => "O(1)".to_string(),
        }
    }

    fn generate_layer_notes(&self, layer_index: usize, stats: &ActivationStats) -> Vec<String> {
        let mut notes = Vec::new();
        
        notes.push(format!("Active neurons: {}/{} ({:.1}%)", 
                          stats.active_neurons, 
                          if stats.active_neurons > 0 { stats.active_neurons } else { 1 },
                          stats.active_neurons as f32 / 100.0)); // Simplified calculation
        
        if stats.mean.abs() < 0.1 {
            notes.push("Mean activation is close to zero - good for many activation functions".to_string());
        }
        
        if stats.std_dev > 1.0 {
            notes.push("High variance in activations - may indicate need for normalization".to_string());
        }

        notes
    }

    fn generate_performance_notes(&self) -> Vec<String> {
        vec![
            "Forward pass completed successfully".to_string(),
            "Consider monitoring gradient flow during training".to_string(),
        ]
    }

    fn generate_learning_suggestions(&self) -> Vec<String> {
        vec![
            "Practice understanding each layer's function".to_string(),
            "Experiment with different activation functions".to_string(),
            "Monitor activation patterns during training".to_string(),
        ]
    }

    fn identify_common_mistakes(&self) -> Vec<String> {
        vec![
            "Forgetting to apply activation functions".to_string(),
            "Incorrect weight initialization".to_string(),
            "Not checking tensor dimensions".to_string(),
        ]
    }

    fn assess_model_complexity(&self) -> ModelComplexity {
        let total_params: usize = self.layers
            .iter()
            .map(|layer| self.estimate_parameter_count_from_layer(layer))
            .sum();

        match total_params {
            0..=100 => ModelComplexity::Simple,
            101..=1000 => ModelComplexity::Moderate,
            1001..=10000 => ModelComplexity::Complex,
            _ => ModelComplexity::Advanced,
        }
    }

    fn estimate_parameter_count_from_layer(&self, layer: &EducationalLayer) -> usize {
        match layer {
            EducationalLayer::Dense { input_size, output_size, .. } => input_size * output_size + output_size,
            EducationalLayer::Conv2D { input_channels, output_channels, kernel_size, .. } => {
                input_channels * output_channels * kernel_size * kernel_size + output_channels
            }
            _ => 0,
        }
    }

    fn generate_learning_objectives(&self) -> Vec<String> {
        vec![
            "Understand neural network forward propagation".to_string(),
            "Learn about different activation functions".to_string(),
            "Practice debugging activation patterns".to_string(),
        ]
    }

    fn identify_prerequisites(&self) -> Vec<String> {
        vec![
            "Basic linear algebra".to_string(),
            "Understanding of functions and derivatives".to_string(),
            "Programming basics".to_string(),
        ]
    }

    fn estimate_understanding_time(&self) -> std::time::Duration {
        std::time::Duration::from_secs(60 * 30) // 30 minutes estimate
    }

    fn suggest_interactive_features(&self) -> Vec<InteractiveFeature> {
        vec![
            InteractiveFeature::WeightSlider,
            InteractiveFeature::ActivationHeatmap,
            InteractiveFeature::GradientVisualizer,
        ]
    }

    fn estimate_memory_usage(&self) -> usize {
        // Simplified memory estimation
        let mut total_size = 0;
        for layer in &self.layers {
            match layer {
                EducationalLayer::Dense { input_size, output_size, .. } => {
                    total_size += input_size * output_size + output_size;
                }
                _ => {}
            }
        }
        total_size * 4 / (1024 * 1024) // Convert to MB (assuming 4 bytes per float)
    }
}

/// Error types for educational neural networks
#[derive(Debug, thiserror::Error)]
pub enum NNEducationError {
    #[error("Dimension mismatch: expected {expected}, got {actual}")]
    DimensionMismatch { expected: usize, actual: usize },
    
    #[error("Invalid activation function: {0}")]
    InvalidActivation(String),
    
    #[error("Educational validation error: {0}")]
    Educational(String),
    
    #[error("Model not trained")]
    NotTrained,
}