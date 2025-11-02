//! Utility functions for educational neural networks
//! 
//! Provides helper functions, validation utilities, and educational tools
//! for working with neural networks in an educational context.

use super::EducationalLayer;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Educational utilities for neural network validation and analysis
pub struct EducationalNNUtils;

impl EducationalNNUtils {
    /// Validate neural network architecture for educational purposes
    pub fn validate_architecture(layers: &[EducationalLayer]) -> Result<ValidationReport, ValidationError> {
        let mut issues = Vec::new();
        let mut warnings = Vec::new();
        let mut recommendations = Vec::new();

        // Check for empty network
        if layers.is_empty() {
            issues.push(ValidationIssue {
                severity: IssueSeverity::Critical,
                category: ValidationCategory::Architecture,
                message: "Network has no layers".to_string(),
                suggestion: "Add at least one layer".to_string(),
                educational_rationale: "A neural network must have at least one layer to process data".to_string(),
            });
        }

        // Validate layer transitions
        for i in 0..layers.len() {
            let layer_issues = Self::validate_layer_transition(layers, i);
            issues.extend(layer_issues.issues);
            warnings.extend(layer_issues.warnings);
        }

        // Check for common architectural issues
        let common_issues = Self::check_common_issues(layers);
        issues.extend(common_issues.issues);
        warnings.extend(common_issues.warnings);
        recommendations.extend(common_issues.recommendations);

        // Generate educational recommendations
        let educational_recs = Self::generate_educational_recommendations(layers);
        recommendations.extend(educational_recs);

        Ok(ValidationReport {
            is_valid: issues.is_empty(),
            issues,
            warnings,
            recommendations,
            educational_summary: self.generate_educational_summary(layers),
        })
    }

    /// Analyze network complexity for educational purposes
    pub fn analyze_complexity(layers: &[EducationalLayer]) -> ComplexityAnalysis {
        let parameter_count = Self::count_parameters(layers);
        let depth = layers.len();
        let max_width = Self::find_max_width(layers);
        let computational_complexity = Self::estimate_computational_complexity(layers);
        let memory_requirements = Self::estimate_memory_requirements(layers);

        // Determine educational difficulty
        let educational_difficulty = Self::assess_educational_difficulty(
            parameter_count,
            depth,
            max_width,
            &computational_complexity,
        );

        ComplexityAnalysis {
            parameter_count,
            depth,
            max_width,
            computational_complexity,
            memory_requirements_mb: memory_requirements / (1024 * 1024),
            educational_difficulty,
            learning_curve_estimate: Self::estimate_learning_curve(educational_difficulty),
            recommended_prerequisites: Self::identify_prerequisites(layers),
        }
    }

    /// Generate educational network summary
    pub fn generate_educational_summary(
        layers: &[EducationalLayer],
        performance_metrics: Option<&NetworkPerformanceMetrics>,
    ) -> EducationalNetworkSummary {
        let network_concepts = Self::extract_network_concepts(layers);
        let learning_objectives = Self::generate_learning_objectives(layers);
        let common_mistakes = Self::identify_common_student_mistakes(layers);
        let debugging_strategies = Self::generate_debugging_strategies(layers);
        let optimization_tips = Self::generate_optimization_tips(layers);

        EducationalNetworkSummary {
            network_type: Self::classify_network_type(layers),
            core_concepts: network_concepts,
            learning_objectives,
            common_student_mistakes: common_mistakes,
            debugging_strategies,
            optimization_recommendations: optimization_tips,
            assessment_criteria: Self::generate_assessment_criteria(layers),
            practical_applications: Self::identify_applications(layers),
            further_reading: Self::suggest_further_reading(layers),
        }
    }

    /// Create educational progress tracker
    pub fn create_progress_tracker(
        layers: &[EducationalLayer],
        student_skill_level: StudentSkillLevel,
    ) -> EducationalProgressTracker {
        let learning_objectives = Self::generate_learning_objectives(layers);
        let assessment_points = Self::generate_assessment_points(layers, &student_skill_level);

        EducationalProgressTracker {
            session_id: format!("session_{}", std::time::SystemTime::now().elapsed().unwrap().as_secs()),
            model_architecture: Self::summarize_architecture(layers),
            learning_objectives,
            assessment_points,
            skill_development_map: Self::create_skill_development_map(&learning_objectives),
            adaptive_recommendations: Vec::new(),
            milestone_tracking: Vec::new(),
        }
    }

    /// Helper methods

    fn validate_layer_transition(
        layers: &[EducationalLayer],
        layer_index: usize,
    ) -> ValidationResult {
        let mut issues = Vec::new();
        let mut warnings = Vec::new();

        if layer_index >= layers.len() {
            return ValidationResult { issues, warnings };
        }

        match &layers[layer_index] {
            EducationalLayer::Dense {
                input_size,
                output_size,
                ..
            } => {
                // Check for reasonable layer sizes
                if *output_size == 0 {
                    issues.push(ValidationIssue {
                        severity: IssueSeverity::Critical,
                        category: ValidationCategory::LayerSize,
                        message: "Dense layer has zero output size".to_string(),
                        suggestion: "Set output_size to a positive integer".to_string(),
                        educational_rationale: "Output size must be positive for meaningful computation".to_string(),
                    });
                }

                if *input_size == 0 {
                    issues.push(ValidationIssue {
                        severity: IssueSeverity::Critical,
                        category: ValidationCategory::LayerSize,
                        message: "Dense layer has zero input size".to_string(),
                        suggestion: "Set input_size to match previous layer output".to_string(),
                        educational_rationale: "Input size must match data dimensions".to_string(),
                    });
                }

                // Check for extremely large layers (educational warning)
                if *output_size > 10000 {
                    warnings.push(ValidationWarning {
                        category: ValidationCategory::Performance,
                        message: format!("Dense layer with {} outputs may be computationally expensive", output_size),
                        suggestion: "Consider reducing layer size for educational purposes".to_string(),
                        educational_benefit: "Learning about computational efficiency".to_string(),
                    });
                }
            }
            EducationalLayer::Conv2D {
                input_channels,
                output_channels,
                kernel_size,
                ..
            } => {
                if *kernel_size % 2 == 0 {
                    warnings.push(ValidationWarning {
                        category: ValidationCategory::Architecture,
                        message: "Even kernel size may cause padding issues".to_string(),
                        suggestion: "Consider using odd kernel sizes for symmetric padding".to_string(),
                        educational_benefit: "Understanding convolution padding".to_string(),
                    });
                }
            }
            _ => {
                // Other layer validations
            }
        }

        // Check transition to next layer
        if layer_index < layers.len() - 1 {
            let transition_issues = Self::validate_layer_transition_internal(
                &layers[layer_index],
                &layers[layer_index + 1],
            );
            issues.extend(transition_issues.issues);
            warnings.extend(transition_issues.warnings);
        }

        ValidationResult { issues, warnings }
    }

    fn validate_layer_transition_internal(
        current: &EducationalLayer,
        next: &EducationalLayer,
    ) -> ValidationResult {
        let mut issues = Vec::new();
        let mut warnings = Vec::new();

        // Check dimension compatibility
        match (current, next) {
            (
                EducationalLayer::Dense {
                    output_size: current_output,
                    ..
                },
                EducationalLayer::Dense {
                    input_size: next_input,
                    ..
                },
            ) => {
                if current_output != next_input {
                    issues.push(ValidationIssue {
                        severity: IssueSeverity::Critical,
                        category: ValidationCategory::DimensionCompatibility,
                        message: format!("Dense layer output size ({}) doesn't match next layer input size ({})", 
                                       current_output, next_input),
                        suggestion: "Ensure consecutive dense layers have compatible dimensions".to_string(),
                        educational_rationale: "Dimension matching is crucial for proper data flow".to_string(),
                    });
                }
            }
            (
                EducationalLayer::Flatten,
                EducationalLayer::Dense { input_size, .. },
            ) => {
                // Flatten can connect to any dense layer, but warn about large inputs
                if *input_size > 1000 {
                    warnings.push(ValidationWarning {
                        category: ValidationCategory::Performance,
                        message: "Large flattened input may cause memory issues".to_string(),
                        suggestion: "Consider adding intermediate layers or using CNN features".to_string(),
                        educational_benefit: "Understanding feature extraction benefits".to_string(),
                    });
                }
            }
            _ => {
                // Other transitions are generally OK
            }
        }

        ValidationResult { issues, warnings }
    }

    fn check_common_issues(layers: &[EducationalLayer]) -> ValidationResult {
        let mut issues = Vec::new();
        let mut warnings = Vec::new();
        let mut recommendations = Vec::new();

        // Check for missing activation functions
        let mut dense_layers_without_activation = 0;
        for layer in layers {
            if let EducationalLayer::Dense {
                activation: super::ActivationType::Linear,
                ..
            } = layer
            {
                dense_layers_without_activation += 1;
            }
        }

        if dense_layers_without_activation > 1 {
            warnings.push(ValidationWarning {
                category: ValidationCategory::Architecture,
                message: format!("{} dense layers use linear activation", dense_layers_without_activation),
                suggestion: "Consider using non-linear activations in hidden layers".to_string(),
                educational_benefit: "Understanding the importance of non-linearity".to_string(),
            });
        }

        // Check for reasonable network depth
        if layers.len() > 20 {
            recommendations.push(ValidationRecommendation {
                category: ValidationCategory::Complexity,
                priority: RecommendationPriority::Medium,
                message: "Very deep network may be difficult to train".to_string(),
                suggestion: "Consider residual connections or shallower architecture for education".to_string(),
                educational_benefit: "Learning about deep learning challenges".to_string(),
            });
        }

        ValidationResult { issues, warnings, recommendations }
    }

    fn generate_educational_recommendations(
        layers: &[EducationalLayer],
    ) -> Vec<ValidationRecommendation> {
        let mut recommendations = Vec::new();

        // Recommend appropriate complexity based on layer types
        let has_conv = layers.iter().any(|l| matches!(l, EducationalLayer::Conv2D { .. }));
        let has_dense = layers.iter().any(|l| matches!(l, EducationalLayer::Dense { .. }));

        if has_conv && has_dense {
            recommendations.push(ValidationRecommendation {
                category: ValidationCategory::Educational,
                priority: RecommendationPriority::Low,
                message: "CNN detected - good for image classification education".to_string(),
                suggestion: "Explain convolution concepts and spatial feature learning".to_string(),
                educational_benefit: "Understanding computer vision basics".to_string(),
            });
        }

        if layers.iter().any(|l| matches!(l, EducationalLayer::Dropout { .. })) {
            recommendations.push(ValidationRecommendation {
                category: ValidationCategory::Regularization,
                priority: RecommendationPriority::Medium,
                message: "Dropout regularization detected".to_string(),
                suggestion: "Explain overfitting and regularization concepts".to_string(),
                educational_benefit: "Understanding generalization techniques".to_string(),
            });
        }

        recommendations
    }

    fn count_parameters(layers: &[EducationalLayer]) -> usize {
        layers
            .iter()
            .map(|layer| match layer {
                EducationalLayer::Dense {
                    input_size,
                    output_size,
                    ..
                } => input_size * output_size + output_size, // weights + biases
                EducationalLayer::Conv2D {
                    input_channels,
                    output_channels,
                    kernel_size,
                    ..
                } => input_channels * output_channels * kernel_size * kernel_size + output_channels,
                _ => 0,
            })
            .sum()
    }

    fn find_max_width(layers: &[EducationalLayer]) -> usize {
        layers
            .iter()
            .map(|layer| match layer {
                EducationalLayer::Dense { output_size, .. } => *output_size,
                EducationalLayer::Conv2D { output_channels, .. } => *output_channels,
                _ => 0,
            })
            .max()
            .unwrap_or(0)
    }

    fn estimate_computational_complexity(layers: &[EducationalLayer]) -> String {
        let mut complexity_components = Vec::new();

        for layer in layers {
            match layer {
                EducationalLayer::Dense {
                    input_size,
                    output_size,
                    ..
                } => {
                    complexity_components.push(format!("Dense({}, {})", input_size, output_size));
                }
                EducationalLayer::Conv2D {
                    input_channels,
                    output_channels,
                    kernel_size,
                    stride,
                    ..
                } => {
                    let ops_per_position = input_channels * output_channels * kernel_size * kernel_size;
                    let positions = 28 * 28 / stride / stride; // Simplified for educational purposes
                    complexity_components.push(format!("Conv({}×{}, {}×{})", 
                        input_channels, output_channels, kernel_size, kernel_size));
                }
                _ => {}
            }
        }

        complexity_components.join(" + ")
    }

    fn estimate_memory_requirements(layers: &[EducationalLayer]) -> usize {
        // Rough estimation: parameters + activations
        let parameter_count = Self::count_parameters(layers);
        let activation_estimate = layers
            .iter()
            .map(|layer| match layer {
                EducationalLayer::Dense { output_size, .. } => *output_size,
                EducationalLayer::Conv2D { output_channels, .. } => output_channels * 28 * 28, // Simplified
                _ => 0,
            })
            .sum::<usize>();

        (parameter_count + activation_estimate) * 4 // 4 bytes per float
    }

    fn assess_educational_difficulty(
        parameter_count: usize,
        depth: usize,
        max_width: usize,
        computational_complexity: &str,
    ) -> EducationalDifficultyLevel {
        let complexity_score = 
            (parameter_count as f32 / 1000.0).min(10.0) + 
            (depth as f32 / 5.0).min(5.0) + 
            (max_width as f32 / 100.0).min(3.0);

        if complexity_score < 2.0 {
            EducationalDifficultyLevel::Beginner
        } else if complexity_score < 5.0 {
            EducationalDifficultyLevel::Intermediate
        } else if complexity_score < 8.0 {
            EducationalDifficultyLevel::Advanced
        } else {
            EducationalDifficultyLevel::Expert
        }
    }

    fn estimate_learning_curve(difficulty: EducationalDifficultyLevel) -> LearningCurveEstimate {
        match difficulty {
            EducationalDifficultyLevel::Beginner => LearningCurveEstimate {
                initial_concepts_hours: 2,
                hands_on_practice_hours: 4,
                mastery_hours: 8,
                prerequisite_review_hours: 1,
            },
            EducationalDifficultyLevel::Intermediate => LearningCurveEstimate {
                initial_concepts_hours: 4,
                hands_on_practice_hours: 8,
                mastery_hours: 20,
                prerequisite_review_hours: 2,
            },
            EducationalDifficultyLevel::Advanced => LearningCurveEstimate {
                initial_concepts_hours: 8,
                hands_on_practice_hours: 20,
                mastery_hours: 50,
                prerequisite_review_hours: 4,
            },
            EducationalDifficultyLevel::Expert => LearningCurveEstimate {
                initial_concepts_hours: 16,
                hands_on_practice_hours: 40,
                mastery_hours: 100,
                prerequisite_review_hours: 8,
            },
        }
    }

    fn identify_prerequisites(layers: &[EducationalLayer]) -> Vec<String> {
        let mut prerequisites = HashSet::new();
        prerequisites.insert("Basic linear algebra".to_string());
        prerequisites.insert("Matrix operations".to_string());

        let has_conv = layers.iter().any(|l| matches!(l, EducationalLayer::Conv2D { .. }));
        if has_conv {
            prerequisites.insert("Image processing basics".to_string());
            prerequisites.insert("Spatial transformations".to_string());
        }

        let has_dropout = layers.iter().any(|l| matches!(l, EducationalLayer::Dropout { .. }));
        if has_dropout {
            prerequisites.insert("Statistics basics".to_string());
            prerequisites.insert("Probability concepts".to_string());
        }

        prerequisites.into_iter().collect()
    }

    fn extract_network_concepts(layers: &[EducationalLayer]) -> Vec<NetworkConcept> {
        let mut concepts = Vec::new();

        if layers.iter().any(|l| matches!(l, EducationalLayer::Dense { .. })) {
            concepts.push(NetworkConcept {
                concept: "Fully Connected Layers".to_string(),
                description: "Layers where each neuron connects to all neurons in the next layer".to_string(),
                importance: ConceptImportance::High,
                learning_objectives: vec![
                    "Understand matrix multiplication".to_string(),
                    "Learn about parameter sharing".to_string(),
                ],
            });
        }

        if layers.iter().any(|l| matches!(l, EducationalLayer::Conv2D { .. })) {
            concepts.push(NetworkConcept {
                concept: "Convolutional Layers".to_string(),
                description: "Layers that apply convolution operations for feature extraction".to_string(),
                importance: ConceptImportance::High,
                learning_objectives: vec![
                    "Understand spatial feature learning".to_string(),
                    "Learn about parameter efficiency".to_string(),
                ],
            });
        }

        if layers.iter().any(|l| matches!(l, EducationalLayer::Dropout { .. })) {
            concepts.push(NetworkConcept {
                concept: "Regularization".to_string(),
                description: "Techniques to prevent overfitting".to_string(),
                importance: ConceptImportance::Medium,
                learning_objectives: vec![
                    "Understand overfitting".to_string(),
                    "Learn regularization benefits".to_string(),
                ],
            });
        }

        concepts
    }

    fn generate_learning_objectives(layers: &[EducationalLayer]) -> Vec<LearningObjective> {
        vec![
            LearningObjective {
                objective: "Understand neural network architecture".to_string(),
                description: "Comprehend how layers connect and process data".to_string(),
                assessment_method: "Interactive exploration".to_string(),
                success_criteria: "Can explain each layer's function".to_string(),
            },
            LearningObjective {
                objective: "Analyze network complexity".to_string(),
                description: "Understand computational and memory requirements".to_string(),
                assessment_method: "Performance analysis".to_string(),
                success_criteria: "Can estimate resource requirements".to_string(),
            },
        ]
    }

    fn identify_common_student_mistakes(layers: &[EducationalLayer]) -> Vec<CommonMistake> {
        vec![
            CommonMistake {
                mistake: "Incorrect dimension matching".to_string(),
                description: "Not ensuring consecutive layers have compatible dimensions".to_string(),
                prevention_strategy: "Always validate layer connections".to_string(),
                educational_focus: "Understanding data flow".to_string(),
            },
            CommonMistake {
                mistake: "Choosing inappropriate activation functions".to_string(),
                description: "Using linear activations in hidden layers".to_string(),
                prevention_strategy: "Learn activation function properties".to_string(),
                educational_focus: "Non-linearity importance".to_string(),
            },
        ]
    }

    fn generate_debugging_strategies(layers: &[EducationalLayer]) -> Vec<DebuggingStrategy> {
        vec![
            DebuggingStrategy {
                strategy: "Layer-by-layer validation".to_string(),
                description: "Check output dimensions and values at each layer".to_string(),
                implementation: "Use forward pass debugging".to_string(),
                educational_value: "Understanding data flow".to_string(),
            },
            DebuggingStrategy {
                strategy: "Gradient flow analysis".to_string(),
                description: "Monitor gradients to ensure they don't vanish or explode".to_string(),
                implementation: "Track gradient magnitudes".to_string(),
                educational_value: "Understanding optimization".to_string(),
            },
        ]
    }

    fn generate_optimization_tips(layers: &[EducationalLayer]) -> Vec<OptimizationTip> {
        vec![
            OptimizationTip {
                tip: "Start with simpler architectures".to_string(),
                description: "Begin with basic networks before complex ones".to_string(),
                expected_benefit: "Better understanding and faster debugging".to_string(),
                educational_rationale: "Progressive complexity learning".to_string(),
            },
            OptimizationTip {
                tip: "Monitor parameter count".to_string(),
                description: "Keep network size appropriate for the problem".to_string(),
                expected_benefit: "Reduced overfitting and training time".to_string(),
                educational_rationale: "Understanding model complexity".to_string(),
            },
        ]
    }

    fn classify_network_type(layers: &[EducationalLayer]) -> NetworkType {
        let has_conv = layers.iter().any(|l| matches!(l, EducationalLayer::Conv2D { .. }));
        let has_recurrent = layers.iter().any(|l| matches!(l, EducationalLayer::RNN { .. }));
        
        if has_conv {
            NetworkType::Convolutional
        } else if has_recurrent {
            NetworkType::Recurrent
        } else {
            NetworkType::FeedForward
        }
    }

    fn generate_assessment_criteria(layers: &[EducationalLayer]) -> Vec<AssessmentCriterion> {
        vec![
            AssessmentCriterion {
                criterion: "Architecture Understanding".to_string(),
                weight: 0.4,
                measurement_method: "Conceptual questions".to_string(),
                educational_rationale: "Foundation knowledge assessment".to_string(),
            },
            AssessmentCriterion {
                criterion: "Implementation Skills".to_string(),
                weight: 0.3,
                measurement_method: "Code implementation".to_string(),
                educational_rationale: "Practical skill development".to_string(),
            },
            AssessmentCriterion {
                criterion: "Debugging Ability".to_string(),
                weight: 0.3,
                measurement_method: "Problem-solving exercises".to_string(),
                educational_rationale: "Troubleshooting skill development".to_string(),
            },
        ]
    }

    fn identify_applications(layers: &[EducationalLayer]) -> Vec<ApplicationArea> {
        let has_conv = layers.iter().any(|l| matches!(l, EducationalLayer::Conv2D { .. }));
        
        if has_conv {
            vec![
                ApplicationArea {
                    area: "Image Classification".to_string(),
                    description: "Classifying images into categories".to_string(),
                    educational_relevance: "Understanding computer vision".to_string(),
                },
                ApplicationArea {
                    area: "Object Detection".to_string(),
                    description: "Locating objects in images".to_string(),
                    educational_relevance: "Spatial understanding".to_string(),
                },
            ]
        } else {
            vec![
                ApplicationArea {
                    area: "Classification".to_string(),
                    description: "Predicting discrete categories".to_string(),
                    educational_relevance: "Fundamental ML task".to_string(),
                },
                ApplicationArea {
                    area: "Regression".to_string(),
                    description: "Predicting continuous values".to_string(),
                    educational_relevance: "Understanding function approximation".to_string(),
                },
            ]
        }
    }

    fn suggest_further_reading(layers: &[EducationalLayer]) -> Vec<ReadingRecommendation> {
        vec![
            ReadingRecommendation {
                resource: "Deep Learning by Ian Goodfellow".to_string(),
                relevance: "Comprehensive neural network theory".to_string(),
                difficulty_match: true,
                educational_value: "High".to_string(),
            },
            ReadingRecommendation {
                resource: "Neural Networks and Deep Learning by Michael Nielsen".to_string(),
                relevance: "Accessible introduction with educational focus".to_string(),
                difficulty_match: true,
                educational_value: "High".to_string(),
            },
        ]
    }

    fn generate_educational_summary(&self, layers: &[EducationalLayer]) -> EducationalSummary {
        let complexity = self.analyze_complexity(layers);
        
        EducationalSummary {
            network_overview: format!("{} layer neural network", layers.len()),
            complexity_assessment: format!("{} difficulty with {} parameters", 
                format!("{:?}", complexity.educational_difficulty),
                complexity.parameter_count),
            learning_recommendations: self.generate_learning_recommendations(&complexity),
        }
    }

    fn generate_learning_recommendations(&self, complexity: &ComplexityAnalysis) -> Vec<String> {
        match complexity.educational_difficulty {
            EducationalDifficultyLevel::Beginner => vec![
                "Focus on understanding basic concepts first".to_string(),
                "Practice with simple examples".to_string(),
                "Use interactive visualizations".to_string(),
            ],
            EducationalDifficultyLevel::Intermediate => vec![
                "Implement networks from scratch".to_string(),
                "Experiment with different architectures".to_string(),
                "Analyze performance metrics".to_string(),
            ],
            EducationalDifficultyLevel::Advanced => vec![
                "Study advanced optimization techniques".to_string(),
                "Understand regularization methods".to_string(),
                "Explore state-of-the-art architectures".to_string(),
            ],
            EducationalDifficultyLevel::Expert => vec![
                "Research novel architectures".to_string(),
                "Contribute to open-source projects".to_string(),
                "Teach concepts to others".to_string(),
            ],
        }
    }

    fn summarize_architecture(layers: &[EducationalLayer]) -> String {
        layers
            .iter()
            .map(|layer| match layer {
                EducationalLayer::Dense { output_size, .. } => format!("Dense({})", output_size),
                EducationalLayer::Conv2D { output_channels, kernel_size, .. } => 
                    format!("Conv2D({}x{})", output_channels, kernel_size),
                EducationalLayer::MaxPool2D { pool_size, .. } => format!("MaxPool({})", pool_size),
                EducationalLayer::Flatten => "Flatten".to_string(),
                EducationalLayer::Dropout { rate } => format!("Dropout({:.1})", rate),
            })
            .collect::<Vec<_>>()
            .join(" -> ")
    }

    fn create_skill_development_map(&self, objectives: &[LearningObjective]) -> SkillDevelopmentMap {
        let mut skill_map = SkillDevelopmentMap::default();
        
        for objective in objectives {
            if objective.objective.contains("architecture") {
                skill_map.architectural_thinking.current_level = SkillLevel::Beginner;
                skill_map.architectural_thinking.recommended_practice.push(
                    "Design simple networks".to_string()
                );
            }
        }
        
        skill_map
    }

    fn create_progress_tracker(&self, layers: &[EducationalLayer], skill_level: StudentSkillLevel) -> EducationalProgressTracker {
        let learning_objectives = self.generate_learning_objectives(layers);
        let assessment_points = self.generate_assessment_points(layers, &skill_level);
        
        EducationalProgressTracker {
            session_id: format!("tracker_{}", std::time::SystemTime::now().elapsed().unwrap().as_secs()),
            model_architecture: self.summarize_architecture(layers),
            learning_objectives,
            assessment_points,
            skill_development_map: self.create_skill_development_map(&learning_objectives),
            adaptive_recommendations: Vec::new(),
            milestone_tracking: Vec::new(),
        }
    }

    fn generate_assessment_points(&self, layers: &[EducationalLayer], skill_level: &StudentSkillLevel) -> Vec<AssessmentPoint> {
        vec![
            AssessmentPoint {
                point_id: "architecture_understanding".to_string(),
                description: "Can explain network architecture".to_string(),
                difficulty: match skill_level {
                    StudentSkillLevel::Beginner => AssessmentDifficulty::Basic,
                    StudentSkillLevel::Intermediate => AssessmentDifficulty::Intermediate,
                    StudentSkillLevel::Advanced => AssessmentDifficulty::Advanced,
                },
                assessment_method: "Interactive Q&A".to_string(),
            },
        ]
    }
}

/// Supporting types and structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    pub is_valid: bool,
    pub issues: Vec<ValidationIssue>,
    pub warnings: Vec<ValidationWarning>,
    pub recommendations: Vec<ValidationRecommendation>,
    pub educational_summary: EducationalSummary,
}

#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub issues: Vec<ValidationIssue>,
    pub warnings: Vec<ValidationWarning>,
    pub recommendations: Vec<ValidationRecommendation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationIssue {
    pub severity: IssueSeverity,
    pub category: ValidationCategory,
    pub message: String,
    pub suggestion: String,
    pub educational_rationale: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationWarning {
    pub category: ValidationCategory,
    pub message: String,
    pub suggestion: String,
    pub educational_benefit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRecommendation {
    pub category: ValidationCategory,
    pub priority: RecommendationPriority,
    pub message: String,
    pub suggestion: String,
    pub educational_benefit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EducationalSummary {
    pub network_overview: String,
    pub complexity_assessment: String,
    pub learning_recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IssueSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationCategory {
    Architecture,
    LayerSize,
    DimensionCompatibility,
    Performance,
    Educational,
    Regularization,
    Complexity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationPriority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityAnalysis {
    pub parameter_count: usize,
    pub depth: usize,
    pub max_width: usize,
    pub computational_complexity: String,
    pub memory_requirements_mb: usize,
    pub educational_difficulty: EducationalDifficultyLevel,
    pub learning_curve_estimate: LearningCurveEstimate,
    pub recommended_prerequisites: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EducationalDifficultyLevel {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningCurveEstimate {
    pub initial_concepts_hours: u32,
    pub hands_on_practice_hours: u32,
    pub mastery_hours: u32,
    pub prerequisite_review_hours: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EducationalNetworkSummary {
    pub network_type: NetworkType,
    pub core_concepts: Vec<NetworkConcept>,
    pub learning_objectives: Vec<LearningObjective>,
    pub common_student_mistakes: Vec<CommonMistake>,
    pub debugging_strategies: Vec<DebuggingStrategy>,
    pub optimization_recommendations: Vec<OptimizationTip>,
    pub assessment_criteria: Vec<AssessmentCriterion>,
    pub practical_applications: Vec<ApplicationArea>,
    pub further_reading: Vec<ReadingRecommendation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkType {
    FeedForward,
    Convolutional,
    Recurrent,
    Hybrid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConcept {
    pub concept: String,
    pub description: String,
    pub importance: ConceptImportance,
    pub learning_objectives: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConceptImportance {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningObjective {
    pub objective: String,
    pub description: String,
    pub assessment_method: String,
    pub success_criteria: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommonMistake {
    pub mistake: String,
    pub description: String,
    pub prevention_strategy: String,
    pub educational_focus: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebuggingStrategy {
    pub strategy: String,
    pub description: String,
    pub implementation: String,
    pub educational_value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationTip {
    pub tip: String,
    pub description: String,
    pub expected_benefit: String,
    pub educational_rationale: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssessmentCriterion {
    pub criterion: String,
    pub weight: f32,
    pub measurement_method: String,
    pub educational_rationale: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationArea {
    pub area: String,
    pub description: String,
    pub educational_relevance: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadingRecommendation {
    pub resource: String,
    pub relevance: String,
    pub difficulty_match: bool,
    pub educational_value: String,
}

#[derive(Debug, Clone)]
pub struct EducationalProgressTracker {
    pub session_id: String,
    pub model_architecture: String,
    pub learning_objectives: Vec<LearningObjective>,
    pub assessment_points: Vec<AssessmentPoint>,
    pub skill_development_map: SkillDevelopmentMap,
    pub adaptive_recommendations: Vec<AdaptiveRecommendation>,
    pub milestone_tracking: Vec<MilestoneRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssessmentPoint {
    pub point_id: String,
    pub description: String,
    pub difficulty: AssessmentDifficulty,
    pub assessment_method: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AssessmentDifficulty {
    Basic,
    Intermediate,
    Advanced,
    Expert,
}

#[derive(Debug, Clone)]
pub struct SkillDevelopmentMap {
    pub programming_skills: SkillProgression,
    pub mathematical_concepts: SkillProgression,
    pub algorithmic_thinking: SkillProgression,
    pub architectural_thinking: SkillProgression,
}

#[derive(Debug, Clone, Default)]
pub struct SkillProgression {
    pub current_level: SkillLevel,
    pub experience_points: u32,
    pub unlocked_concepts: Vec<String>,
    pub recommended_practice: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SkillLevel {
    Novice,
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptiveRecommendation {
    pub recommendation_type: RecommendationType,
    pub content: String,
    pub priority: RecommendationPriority,
    pub rationale: String,
    pub implementation_guidance: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationType {
    AdditionalPractice,
    ConceptualReview,
    AlternativeApproach,
    AdvancedTopic,
    CollaborativeLearning,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MilestoneRecord {
    pub milestone: String,
    pub achievement_date: std::time::SystemTime,
    pub significance: MilestoneSignificance,
    pub unlocking: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MilestoneSignificance {
    Minor,
    Moderate,
    Major,
    Transformative,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StudentSkillLevel {
    Beginner,
    Intermediate,
    Advanced,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkPerformanceMetrics {
    pub accuracy: f32,
    pub loss: f32,
    pub training_time: std::time::Duration,
    pub memory_usage: usize,
    pub convergence_epoch: Option<u32>,
}

#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Invalid architecture: {0}")]
    ArchitectureError(String),
    #[error("Educational validation failed: {0}")]
    Educational(String),
}