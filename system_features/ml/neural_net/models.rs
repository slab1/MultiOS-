//! Educational Neural Network Models
//! 
//! Provides pre-built educational neural network models with various
//! complexity levels and learning objectives.

use super::layers::{DenseLayer, ConvLayer, MaxPoolLayer, DropoutLayer, BatchNormLayer};
use super::{EducationalLayer, EducationalNNConfig, EducationalModel};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Educational Model Builder
/// 
/// Provides a builder pattern for creating educational neural network models
/// with appropriate complexity levels and learning objectives.
pub struct EducationalModelBuilder {
    layers: Vec<EducationalLayer>,
    config: EducationalNNConfig,
    educational_features: ModelEducationalFeatures,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelEducationalFeatures {
    pub difficulty_level: ModelDifficultyLevel,
    pub learning_objectives: Vec<ModelLearningObjective>,
    pub prerequisite_concepts: Vec<String>,
    pub assessment_criteria: Vec<AssessmentCriteria>,
    pub interactive_elements: Vec<InteractiveModelElement>,
    pub visualization_requirements: VisualizationRequirements,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelDifficultyLevel {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelLearningObjective {
    pub objective: String,
    pub description: String,
    pub key_concepts: Vec<String>,
    pub assessment_method: AssessmentMethod,
    pub mastery_criteria: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AssessmentMethod {
    InteractiveQuiz,
    PerformanceBenchmark,
    ConceptualUnderstanding,
    ImplementationTask,
    DebuggingExercise,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssessmentCriteria {
    pub criterion: String,
    pub weight: f32,
    pub measurement_method: String,
    pub educational_rationale: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InteractiveModelElement {
    WeightInspector,
    ActivationVisualizer,
    GradientFlowExplorer,
    PerformanceProfiler,
    EducationalAnnotations,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationRequirements {
    pub required_visualizations: Vec<VisualizationType>,
    pub detail_level: DetailLevel,
    pub educational_annotations: bool,
    pub interactive_features: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VisualizationType {
    ArchitectureDiagram,
    WeightHeatmaps,
    ActivationDistributions,
    GradientFlows,
    PerformanceMetrics,
    EducationalOverlays,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DetailLevel {
    Overview,
    Intermediate,
    Detailed,
    Expert,
}

/// Pre-built Educational Models
pub struct EducationalModels;

impl EducationalModels {
    /// Create a beginner-friendly dense network for basic concepts
    pub fn create_beginner_dense_network(input_size: usize, hidden_sizes: Vec<usize>, output_size: usize) -> EducationalModel {
        let config = EducationalNNConfig {
            learning_rate: 0.1,
            batch_size: 1,
            epochs: 5,
            educational_mode: true,
            visualization_enabled: true,
            step_by_step: true,
            auto_grad: false,
        };

        let mut layers = Vec::new();

        // Input layer (represented as first dense layer)
        layers.push(EducationalLayer::Dense {
            input_size,
            output_size: hidden_sizes[0],
            activation: super::ActivationType::ReLU,
            weights: None,
            biases: None,
        });

        // Hidden layers
        for i in 0..hidden_sizes.len() - 1 {
            layers.push(EducationalLayer::Dense {
                input_size: hidden_sizes[i],
                output_size: hidden_sizes[i + 1],
                activation: super::ActivationType::ReLU,
                weights: None,
                biases: None,
            });
        }

        // Output layer
        layers.push(EducationalLayer::Dense {
            input_size: hidden_sizes[hidden_sizes.len() - 1],
            output_size,
            activation: super::ActivationType::Linear,
            weights: None,
            biases: None,
        });

        EducationalModel::new(layers, config)
    }

    /// Create a CNN for image classification education
    pub fn create_educational_cnn() -> EducationalModel {
        let config = EducationalNNConfig {
            learning_rate: 0.01,
            batch_size: 16,
            epochs: 10,
            educational_mode: true,
            visualization_enabled: true,
            step_by_step: false,
            auto_grad: true,
        };

        let layers = vec![
            // First conv block
            EducationalLayer::Conv2D {
                input_channels: 1,
                output_channels: 8,
                kernel_size: 3,
                stride: 1,
                padding: 1,
                activation: super::ActivationType::ReLU,
            },
            EducationalLayer::MaxPool2D {
                pool_size: 2,
                stride: 2,
            },
            // Second conv block
            EducationalLayer::Conv2D {
                input_channels: 8,
                output_channels: 16,
                kernel_size: 3,
                stride: 1,
                padding: 1,
                activation: super::ActivationType::ReLU,
            },
            EducationalLayer::MaxPool2D {
                pool_size: 2,
                stride: 2,
            },
            // Fully connected layers
            EducationalLayer::Flatten,
            EducationalLayer::Dense {
                input_size: 16 * 7 * 7, // Assuming 28x28 input
                output_size: 10,
                activation: super::ActivationType::Linear,
                weights: None,
                biases: None,
            },
        ];

        EducationalModel::new(layers, config)
    }

    /// Create a simple regression network
    pub fn create_simple_regressor(input_features: usize, hidden_size: usize) -> EducationalModel {
        let config = EducationalNNConfig {
            learning_rate: 0.01,
            batch_size: 32,
            epochs: 50,
            educational_mode: true,
            visualization_enabled: true,
            step_by_step: true,
            auto_grad: false,
        };

        let layers = vec![
            EducationalLayer::Dense {
                input_size: input_features,
                output_size: hidden_size,
                activation: super::ActivationType::ReLU,
                weights: None,
                biases: None,
            },
            EducationalLayer::Dense {
                input_size: hidden_size,
                output_size: 1,
                activation: super::ActivationType::Linear,
                weights: None,
                biases: None,
            },
        ];

        EducationalModel::new(layers, config)
    }

    /// Create a network with dropout for regularization education
    pub fn create_regularized_network(input_size: usize, hidden_sizes: Vec<usize>, output_size: usize, dropout_rate: f32) -> EducationalModel {
        let config = EducationalNNConfig {
            learning_rate: 0.001,
            batch_size: 64,
            epochs: 20,
            educational_mode: true,
            visualization_enabled: true,
            step_by_step: false,
            auto_grad: true,
        };

        let mut layers = Vec::new();

        // Input layer
        layers.push(EducationalLayer::Dense {
            input_size,
            output_size: hidden_sizes[0],
            activation: super::ActivationType::ReLU,
            weights: None,
            biases: None,
        });
        layers.push(EducationalLayer::Dropout { rate: dropout_rate });

        // Hidden layers with dropout
        for i in 0..hidden_sizes.len() - 1 {
            layers.push(EducationalLayer::Dense {
                input_size: hidden_sizes[i],
                output_size: hidden_sizes[i + 1],
                activation: super::ActivationType::ReLU,
                weights: None,
                biases: None,
            });
            layers.push(EducationalLayer::Dropout { rate: dropout_rate });
        }

        // Output layer (no dropout)
        layers.push(EducationalLayer::Dense {
            input_size: hidden_sizes[hidden_sizes.len() - 1],
            output_size,
            activation: super::ActivationType::Linear,
            weights: None,
            biases: None,
        });

        EducationalModel::new(layers, config)
    }

    /// Create a residual network for advanced education
    pub fn create_simple_resnet() -> EducationalModel {
        let config = EducationalNNConfig {
            learning_rate: 0.001,
            batch_size: 32,
            epochs: 30,
            educational_mode: true,
            visualization_enabled: true,
            step_by_step: false,
            auto_grad: true,
        };

        // Simplified ResNet-like architecture for education
        let layers = vec![
            EducationalLayer::Conv2D {
                input_channels: 3,
                output_channels: 16,
                kernel_size: 3,
                stride: 1,
                padding: 1,
                activation: super::ActivationType::ReLU,
            },
            EducationalLayer::Conv2D {
                input_channels: 16,
                output_channels: 16,
                kernel_size: 3,
                stride: 1,
                padding: 1,
                activation: super::ActivationType::ReLU,
            },
            EducationalLayer::Conv2D {
                input_channels: 16,
                output_channels: 32,
                kernel_size: 3,
                stride: 2,
                padding: 1,
                activation: super::ActivationType::ReLU,
            },
            EducationalLayer::Conv2D {
                input_channels: 32,
                output_channels: 32,
                kernel_size: 3,
                stride: 1,
                padding: 1,
                activation: super::ActivationType::ReLU,
            },
            EducationalLayer::Flatten,
            EducationalLayer::Dense {
                input_size: 32 * 7 * 7, // Assuming 28x28 input
                output_size: 10,
                activation: super::ActivationType::Linear,
                weights: None,
                biases: None,
            },
        ];

        EducationalModel::new(layers, config)
    }

    /// Create an autoencoder for unsupervised learning education
    pub fn create_simple_autoencoder(input_size: usize, encoding_size: usize) -> EducationalModel {
        let config = EducationalNNConfig {
            learning_rate: 0.01,
            batch_size: 64,
            epochs: 100,
            educational_mode: true,
            visualization_enabled: true,
            step_by_step: false,
            auto_grad: true,
        };

        let layers = vec![
            // Encoder
            EducationalLayer::Dense {
                input_size,
                output_size: (input_size + encoding_size) / 2,
                activation: super::ActivationType::ReLU,
                weights: None,
                biases: None,
            },
            EducationalLayer::Dense {
                input_size: (input_size + encoding_size) / 2,
                output_size: encoding_size,
                activation: super::ActivationType::ReLU,
                weights: None,
                biases: None,
            },
            // Decoder
            EducationalLayer::Dense {
                input_size: encoding_size,
                output_size: (input_size + encoding_size) / 2,
                activation: super::ActivationType::ReLU,
                weights: None,
                biases: None,
            },
            EducationalLayer::Dense {
                input_size: (input_size + encoding_size) / 2,
                output_size,
                activation: super::ActivationType::Sigmoid,
                weights: None,
                biases: None,
            },
        ];

        EducationalModel::new(layers, config)
    }

    /// Create a GAN component (Generator or Discriminator)
    pub fn create_gan_component(component_type: GANComponentType, input_size: usize) -> EducationalModel {
        let config = EducationalNNConfig {
            learning_rate: 0.0002,
            batch_size: 64,
            epochs: 100,
            educational_mode: true,
            visualization_enabled: true,
            step_by_step: false,
            auto_grad: true,
        };

        let layers = match component_type {
            GANComponentType::Generator => vec![
                EducationalLayer::Dense {
                    input_size: 100, // Noise vector
                    output_size: 128,
                    activation: super::ActivationType::ReLU,
                    weights: None,
                    biases: None,
                },
                EducationalLayer::Dense {
                    input_size: 128,
                    output_size: 256,
                    activation: super::ActivationType::ReLU,
                    weights: None,
                    biases: None,
                },
                EducationalLayer::Dense {
                    input_size: 256,
                    output_size: input_size,
                    activation: super::ActivationType::Tanh,
                    weights: None,
                    biases: None,
                },
            ],
            GANComponentType::Discriminator => vec![
                EducationalLayer::Dense {
                    input_size,
                    output_size: 256,
                    activation: super::ActivationType::LeakyReLU,
                    weights: None,
                    biases: None,
                },
                EducationalLayer::Dropout { rate: 0.3 },
                EducationalLayer::Dense {
                    input_size: 256,
                    output_size: 128,
                    activation: super::ActivationType::LeakyReLU,
                    weights: None,
                    biases: None,
                },
                EducationalLayer::Dropout { rate: 0.3 },
                EducationalLayer::Dense {
                    input_size: 128,
                    output_size: 1,
                    activation: super::ActivationType::Sigmoid,
                    weights: None,
                    biases: None,
                },
            ],
        };

        EducationalModel::new(layers, config)
    }
}

/// Model Training Educational Features
#[derive(Debug, Clone)]
pub struct EducationalTrainingSession {
    pub session_id: String,
    pub model: EducationalModel,
    pub training_config: TrainingConfig,
    pub learning_tracker: LearningProgressTracker,
    pub assessment_engine: EducationalAssessmentEngine,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingConfig {
    pub learning_objectives: Vec<TrainingLearningObjective>,
    pub assessment_frequency: AssessmentFrequency,
    pub feedback_level: FeedbackLevel,
    pub intervention_triggers: Vec<InterventionTrigger>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingLearningObjective {
    pub objective: String,
    pub target_competency: CompetencyLevel,
    pub assessment_method: TrainingAssessmentMethod,
    pub success_criteria: SuccessCriteria,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompetencyLevel {
    Awareness,
    Understanding,
    Application,
    Analysis,
    Synthesis,
    Evaluation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrainingAssessmentMethod {
    TheoreticalQuiz,
    PracticalImplementation,
    PerformanceBenchmark,
    PeerReview,
    SelfReflection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuccessCriteria {
    pub threshold: f32,
    pub measurement_metric: String,
    pub validation_method: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AssessmentFrequency {
    Continuous,
    EndOfEpoch,
    EndOfTraining,
    OnPerformanceDrop,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FeedbackLevel {
    Minimal,
    Standard,
    Detailed,
    Comprehensive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InterventionTrigger {
    AccuracyBelow(f32),
    LossPlateau,
    HighOverfitting,
    SlowConvergence,
    ConceptualMisunderstanding,
}

/// Learning Progress Tracking
#[derive(Debug, Clone)]
pub struct LearningProgressTracker {
    pub objectives_progress: HashMap<String, ObjectiveProgress>,
    pub skill_development: SkillDevelopmentMap,
    pub challenge_history: Vec<ChallengeRecord>,
    pub milestone_achievements: Vec<MilestoneRecord>,
    pub adaptive_recommendations: Vec<AdaptiveRecommendation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectiveProgress {
    pub objective: String,
    pub completion_percentage: f32,
    pub competency_level: CompetencyLevel,
    pub evidence: Vec<String>,
    pub next_steps: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct SkillDevelopmentMap {
    pub programming_skills: SkillProgression,
    pub mathematical_concepts: SkillProgression,
    pub algorithmic_thinking: SkillProgression,
    pub debugging_skills: SkillProgression,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
pub struct ChallengeRecord {
    pub challenge_id: String,
    pub challenge_type: ChallengeType,
    pub difficulty_level: ChallengeDifficulty,
    pub outcome: ChallengeOutcome,
    pub learning_points: Vec<String>,
    pub timestamp: std::time::SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChallengeType {
    Implementation,
    Debugging,
    Optimization,
    Theoretical,
    Creative,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChallengeDifficulty {
    Easy,
    Medium,
    Hard,
    Expert,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChallengeOutcome {
    CompletedSuccessfully,
    CompletedWithHints,
    PartiallyCompleted,
    Attempted,
    Skipped,
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
pub enum RecommendationPriority {
    Low,
    Medium,
    High,
    Critical,
}

/// Educational Assessment Engine
#[derive(Debug, Clone)]
pub struct EducationalAssessmentEngine {
    pub assessment_history: Vec<AssessmentRecord>,
    pub competency_map: CompetencyAssessmentMap,
    pub learning_gaps: Vec<LearningGap>,
    pub personalized_path: PersonalizedLearningPath,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssessmentRecord {
    pub assessment_id: String,
    pub assessment_type: AssessmentType,
    pub score: f32,
    pub max_score: f32,
    pub time_taken: std::time::Duration,
    pub competency_demonstrated: Vec<CompetencyEvidence>,
    pub feedback: AssessmentFeedback,
    pub timestamp: std::time::SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AssessmentType {
    MultipleChoice,
    Implementation,
    Debugging,
    Performance,
    Portfolio,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompetencyEvidence {
    pub competency: String,
    pub level_demonstrated: CompetencyLevel,
    pub confidence_score: f32,
    pub supporting_evidence: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssessmentFeedback {
    pub overall_feedback: String,
    pub strengths: Vec<String>,
    pub improvements: Vec<String>,
    pub recommendations: Vec<String>,
    pub next_steps: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct CompetencyAssessmentMap {
    pub programming_competency: CompetencyProfile,
    pub mathematical_competency: CompetencyProfile,
    pub algorithmic_competency: CompetencyProfile,
    pub ml_concepts_competency: CompetencyProfile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompetencyProfile {
    pub current_level: CompetencyLevel,
    pub proficiency_score: f32,
    pub strengths: Vec<String>,
    pub weaknesses: Vec<String>,
    pub development_plan: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningGap {
    pub gap_area: String,
    pub severity: GapSeverity,
    pub impact_on_learning: String,
    pub remediation_strategy: String,
    pub estimated_time: std::time::Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GapSeverity {
    Minor,
    Moderate,
    Major,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalizedLearningPath {
    pub current_stage: LearningStage,
    pub recommended_activities: Vec<RecommendedActivity>,
    pub estimated_completion: std::time::SystemTime,
    pub adaptive_adjustments: Vec<PathAdjustment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LearningStage {
    Foundation,
    Application,
    Integration,
    Mastery,
    Innovation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendedActivity {
    pub activity_type: ActivityType,
    pub description: String,
    pub difficulty: ActivityDifficulty,
    pub estimated_duration: std::time::Duration,
    pub learning_outcomes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActivityType {
    Tutorial,
    Exercise,
    Project,
    Challenge,
    Assessment,
    Reflection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActivityDifficulty {
    Scaffolded,
    Guided,
    Independent,
    Challenging,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathAdjustment {
    pub adjustment_reason: String,
    pub change_description: String,
    pub impact_assessment: String,
    pub implementation_notes: String,
}

/// GAN Component Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GANComponentType {
    Generator,
    Discriminator,
}

/// Builder Implementation
impl EducationalModelBuilder {
    /// Create a new model builder
    pub fn new() -> Self {
        Self {
            layers: Vec::new(),
            config: EducationalNNConfig::default(),
            educational_features: ModelEducationalFeatures {
                difficulty_level: ModelDifficultyLevel::Beginner,
                learning_objectives: Vec::new(),
                prerequisite_concepts: Vec::new(),
                assessment_criteria: Vec::new(),
                interactive_elements: Vec::new(),
                visualization_requirements: VisualizationRequirements {
                    required_visualizations: vec![VisualizationType::ArchitectureDiagram],
                    detail_level: DetailLevel::Overview,
                    educational_annotations: true,
                    interactive_features: false,
                },
            },
        }
    }

    /// Add a dense layer
    pub fn add_dense_layer(
        &mut self,
        input_size: usize,
        output_size: usize,
        activation: super::ActivationType,
    ) -> &mut Self {
        self.layers.push(EducationalLayer::Dense {
            input_size,
            output_size,
            activation,
            weights: None,
            biases: None,
        });
        self
    }

    /// Add a convolutional layer
    pub fn add_conv_layer(
        &mut self,
        input_channels: usize,
        output_channels: usize,
        kernel_size: usize,
        activation: super::ActivationType,
    ) -> &mut Self {
        self.layers.push(EducationalLayer::Conv2D {
            input_channels,
            output_channels,
            kernel_size,
            stride: 1,
            padding: 1,
            activation,
        });
        self
    }

    /// Add max pooling layer
    pub fn add_max_pool(&mut self, pool_size: usize) -> &mut Self {
        self.layers.push(EducationalLayer::MaxPool2D {
            pool_size,
            stride: pool_size,
        });
        self
    }

    /// Add dropout layer
    pub fn add_dropout(&mut self, rate: f32) -> &mut Self {
        self.layers.push(EducationalLayer::Dropout { rate });
        self
    }

    /// Add flatten layer
    pub fn add_flatten(&mut self) -> &mut Self {
        self.layers.push(EducationalLayer::Flatten);
        self
    }

    /// Set difficulty level
    pub fn set_difficulty(&mut self, level: ModelDifficultyLevel) -> &mut Self {
        self.educational_features.difficulty_level = level;
        self
    }

    /// Add learning objective
    pub fn add_learning_objective(
        &mut self,
        objective: &str,
        description: &str,
        key_concepts: Vec<&str>,
    ) -> &mut Self {
        self.educational_features.learning_objectives.push(ModelLearningObjective {
            objective: objective.to_string(),
            description: description.to_string(),
            key_concepts: key_concepts.into_iter().map(|s| s.to_string()).collect(),
            assessment_method: AssessmentMethod::ConceptualUnderstanding,
            mastery_criteria: "Demonstrated understanding".to_string(),
        });
        self
    }

    /// Set learning rate
    pub fn set_learning_rate(&mut self, rate: f32) -> &mut Self {
        self.config.learning_rate = rate;
        self
    }

    /// Set batch size
    pub fn set_batch_size(&mut self, size: usize) -> &mut Self {
        self.config.batch_size = size;
        self
    }

    /// Set number of epochs
    pub fn set_epochs(&mut self, epochs: usize) -> &mut Self {
        self.config.epochs = epochs;
        self
    }

    /// Build the educational model
    pub fn build(&self) -> EducationalModel {
        EducationalModel::new(self.layers.clone(), self.config.clone())
    }
}

impl Default for EducationalModelBuilder {
    fn default() -> Self {
        Self::new()
    }
}