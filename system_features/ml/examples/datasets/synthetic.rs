// Educational ML Framework - Synthetic Dataset Generator
// Provides various synthetic datasets for educational ML practice

use multi_os_ml::runtime::tensor::Tensor;
use std::collections::HashMap;

/// Synthetic dataset configurations for educational purposes
/// 
/// This module provides functions to generate synthetic datasets with known properties,
/// making them ideal for learning and testing ML algorithms without external dependencies.

/// Configuration for generating synthetic datasets
#[derive(Clone, Debug)]
pub struct SyntheticDatasetConfig {
    pub num_samples: usize,
    pub num_features: usize,
    pub num_classes: Option<usize>,  // None for regression
    pub noise_level: f64,            // Amount of noise to add (0.0 to 1.0)
    pub cluster_separation: f64,     // Distance between clusters (for classification)
    pub random_seed: u64,            // For reproducible results
    pub file_path: Option<String>,   // Where to save the dataset
}

/// Supported synthetic dataset types
#[derive(Clone, Debug)]
pub enum SyntheticDatasetType {
    Classification(ClassificationType),
    Regression(RegressionType),
    Clustering,
    TimeSeries,
}

/// Classification dataset types
#[derive(Clone, Debug)]
pub enum ClassificationType {
    LinearSeparable,      // Linearly separable classes
    NonLinearSeparable,   // Classes requiring non-linear decision boundary
    ConcentricCircles,    // Concentric circle patterns
    Moons,                // Two crescent moons
    SwissRoll,            // 3D Swiss roll projected to 2D
    RandomBlobs,          // Random Gaussian clusters
    Imbalanced,           // Classes with different sizes
}

/// Regression dataset types
#[derive(Clone, Debug)]
pub enum RegressionType {
    Linear,               // Linear relationship with noise
    Polynomial,           // Polynomial relationship
    SineWave,             // Sinusoidal pattern
    StepFunction,         // Step function
    MultiModal,           // Multiple modes
    Periodic,             // Periodic function
}

/// Generated dataset result
pub struct GeneratedDataset {
    pub features: Tensor,
    pub targets: Tensor,
    pub feature_names: Vec<String>,
    pub target_name: String,
    pub metadata: HashMap<String, String>,
}

impl SyntheticDatasetConfig {
    /// Create configuration for classification dataset
    pub fn classification(num_samples: usize, num_features: usize, num_classes: usize) -> Self {
        Self {
            num_samples,
            num_features,
            num_classes: Some(num_classes),
            noise_level: 0.1,
            cluster_separation: 1.0,
            random_seed: 42,
            file_path: None,
        }
    }
    
    /// Create configuration for regression dataset
    pub fn regression(num_samples: usize, num_features: usize) -> Self {
        Self {
            num_samples,
            num_features,
            num_classes: None,
            noise_level: 0.1,
            cluster_separation: 1.0,
            random_seed: 42,
            file_path: None,
        }
    }
    
    /// Set noise level (0.0 = no noise, 1.0 = maximum noise)
    pub fn noise_level(mut self, level: f64) -> Self {
        self.noise_level = level.clamp(0.0, 1.0);
        self
    }
    
    /// Set cluster separation for classification
    pub fn cluster_separation(mut self, separation: f64) -> Self {
        self.cluster_separation = separation.max(0.1);
        self
    }
    
    /// Set random seed for reproducibility
    pub fn random_seed(mut self, seed: u64) -> Self {
        self.random_seed = seed;
        self
    }
    
    /// Set file path to save the dataset
    pub fn save_to_file(mut self, path: &str) -> Self {
        self.file_path = Some(path.to_string());
        self
    }
}

/// Generate synthetic classification dataset
pub fn generate_classification_dataset(
    config: SyntheticDatasetConfig,
    dataset_type: ClassificationType,
) -> GeneratedDataset {
    println!("Generating {} classification dataset...", dataset_name(&dataset_type));
    
    let features = match dataset_type {
        ClassificationType::LinearSeparable => generate_linear_separable(&config),
        ClassificationType::NonLinearSeparable => generate_non_linear_separable(&config),
        ClassificationType::ConcentricCircles => generate_concentric_circles(&config),
        ClassificationType::Moons => generate_moons(&config),
        ClassificationType::SwissRoll => generate_swiss_roll(&config),
        ClassificationType::RandomBlobs => generate_random_blobs(&config),
        ClassificationType::Imbalanced => generate_imbalanced(&config),
    };
    
    let targets = generate_classification_labels(&config, &features);
    let feature_names = generate_feature_names(config.num_features, "feature");
    let target_name = "class".to_string();
    
    let mut metadata = HashMap::new();
    metadata.insert("dataset_type".to_string(), format!("{:?}", dataset_type));
    metadata.insert("num_samples".to_string(), config.num_samples.to_string());
    metadata.insert("num_features".to_string(), config.num_features.to_string());
    metadata.insert("num_classes".to_string(), config.num_classes.unwrap_or(0).to_string());
    metadata.insert("noise_level".to_string(), config.noise_level.to_string());
    
    let dataset = GeneratedDataset {
        features,
        targets,
        feature_names,
        target_name,
        metadata,
    };
    
    // Save dataset if file path specified
    if let Some(path) = config.file_path {
        save_dataset_to_csv(&dataset, &path).unwrap_or_else(|e| {
            println!("Warning: Could not save dataset to {}: {}", path, e);
        });
    }
    
    println!("Generated {} samples with {} features", config.num_samples, config.num_features);
    dataset
}

/// Generate synthetic regression dataset
pub fn generate_regression_dataset(
    config: SyntheticDatasetConfig,
    dataset_type: RegressionType,
) -> GeneratedDataset {
    println!("Generating {} regression dataset...", dataset_name(&dataset_type));
    
    let features = generate_regression_features(&config);
    let targets = match dataset_type {
        RegressionType::Linear => generate_linear_targets(&features, &config),
        RegressionType::Polynomial => generate_polynomial_targets(&features, &config),
        RegressionType::SineWave => generate_sine_targets(&features, &config),
        RegressionType::StepFunction => generate_step_targets(&features, &config),
        RegressionType::MultiModal => generate_multimodal_targets(&features, &config),
        RegressionType::Periodic => generate_periodic_targets(&features, &config),
    };
    
    let feature_names = generate_feature_names(config.num_features, "x");
    let target_name = "y".to_string();
    
    let mut metadata = HashMap::new();
    metadata.insert("dataset_type".to_string(), format!("{:?}", dataset_type));
    metadata.insert("num_samples".to_string(), config.num_samples.to_string());
    metadata.insert("num_features".to_string(), config.num_features.to_string());
    metadata.insert("noise_level".to_string(), config.noise_level.to_string());
    
    let dataset = GeneratedDataset {
        features,
        targets,
        feature_names,
        target_name,
        metadata,
    };
    
    // Save dataset if file path specified
    if let Some(path) = config.file_path {
        save_dataset_to_csv(&dataset, &path).unwrap_or_else(|e| {
            println!("Warning: Could not save dataset to {}: {}", path, e);
        });
    }
    
    println!("Generated {} samples with {} features", config.num_samples, config.num_features);
    dataset
}

// Classification dataset generators

fn generate_linear_separable(config: &SyntheticDatasetConfig) -> Tensor {
    let num_classes = config.num_classes.unwrap_or(2);
    let samples_per_class = config.num_samples / num_classes;
    let mut all_features = Vec::new();
    
    for class in 0..num_classes {
        for _ in 0..samples_per_class {
            let center_x = (class as f64 - (num_classes as f64 - 1.0) / 2.0) * config.cluster_separation;
            let mut sample = Vec::new();
            
            for feature_idx in 0..config.num_features {
                let base_value = if feature_idx == 0 { center_x } else { 0.0 };
                let noise = random_normal(0.0, 0.2) * config.noise_level;
                sample.push(base_value + noise);
            }
            
            all_features.extend(sample);
        }
    }
    
    Tensor::from(all_features).reshape(vec![samples_per_class * num_classes, config.num_features])
}

fn generate_non_linear_separable(config: &SyntheticDatasetConfig) -> Tensor {
    let num_classes = config.num_classes.unwrap_or(2);
    let samples_per_class = config.num_samples / num_classes;
    let mut all_features = Vec::new();
    
    for class in 0..num_classes {
        for _ in 0..samples_per_class {
            let angle = class as f64 * std::f64::consts::PI * 2.0 / num_classes as f64;
            let radius = config.cluster_separation;
            
            let x = radius * angle.cos() + random_normal(0.0, 0.2) * config.noise_level;
            let y = radius * angle.sin() + random_normal(0.0, 0.2) * config.noise_level;
            
            let mut sample = vec![x, y];
            
            // Add additional features with some correlation
            for feature_idx in 2..config.num_features {
                let correlation = 0.3;
                let noise = random_normal(0.0, 0.1) * config.noise_level;
                let value = correlation * (x + y) + noise;
                sample.push(value);
            }
            
            all_features.extend(sample);
        }
    }
    
    Tensor::from(all_features).reshape(vec![samples_per_class * num_classes, config.num_features])
}

fn generate_concentric_circles(config: &SyntheticDatasetConfig) -> Tensor {
    let num_classes = config.num_classes.unwrap_or(2);
    let samples_per_class = config.num_samples / num_classes;
    let mut all_features = Vec::new();
    
    for class in 0..num_classes {
        for _ in 0..samples_per_class {
            let radius = (class as f64 + 1.0) * config.cluster_separation;
            let angle = rand_range(0.0, 2.0 * std::f64::consts::PI);
            
            let x = radius * angle.cos() + random_normal(0.0, 0.1) * config.noise_level;
            let y = radius * angle.sin() + random_normal(0.0, 0.1) * config.noise_level;
            
            let mut sample = vec![x, y];
            
            // Add additional features
            for _ in 2..config.num_features {
                sample.push(rand_range(-1.0, 1.0));
            }
            
            all_features.extend(sample);
        }
    }
    
    Tensor::from(all_features).reshape(vec![samples_per_class * num_classes, config.num_features])
}

fn generate_moons(config: &SyntheticDatasetConfig) -> Tensor {
    let num_classes = 2;
    let samples_per_class = config.num_samples / num_classes;
    let mut all_features = Vec::new();
    
    for class in 0..num_classes {
        for _ in 0..samples_per_class {
            let angle = rand_range(0.0, std::f64::consts::PI);
            let offset = if class == 0 { 0.0 } else { 0.4 };
            
            let x = angle.cos();
            let y = angle.sin() + offset;
            
            // Add noise
            let x = x + random_normal(0.0, 0.1) * config.noise_level;
            let y = y + random_normal(0.0, 0.1) * config.noise_level;
            
            let mut sample = vec![x, y];
            
            // Add additional features
            for _ in 2..config.num_features {
                sample.push(rand_range(-2.0, 2.0));
            }
            
            all_features.extend(sample);
        }
    }
    
    Tensor::from(all_features).reshape(vec![samples_per_class * num_classes, config.num_features])
}

fn generate_swiss_roll(config: &SyntheticDatasetConfig) -> Tensor {
    let num_classes = config.num_classes.unwrap_or(2);
    let samples_per_class = config.num_samples / num_classes;
    let mut all_features = Vec::new();
    
    for class in 0..num_classes {
        for _ in 0..samples_per_class {
            let t = rand_range(1.5 * std::f64::consts::PI, 4.5 * std::f64::consts::PI);
            let height = rand_range(0.0, 10.0);
            
            let x = t * t.cos();
            let z = t * t.sin();
            let y = height;
            
            // Add noise and class-specific shift
            let class_shift = (class as f64 - 0.5) * 5.0;
            let x = x + random_normal(0.0, 0.5) * config.noise_level + class_shift;
            let y = y + random_normal(0.0, 1.0) * config.noise_level;
            let z = z + random_normal(0.0, 0.5) * config.noise_level;
            
            let mut sample = vec![x, z];  // Use x and z for 2D projection
            
            // Add additional features
            for _ in 2..config.num_features {
                sample.push(rand_range(-20.0, 20.0));
            }
            
            all_features.extend(sample);
        }
    }
    
    Tensor::from(all_features).reshape(vec![samples_per_class * num_classes, config.num_features])
}

fn generate_random_blobs(config: &SyntheticDatasetConfig) -> Tensor {
    let num_classes = config.num_classes.unwrap_or(3);
    let samples_per_class = config.num_samples / num_classes;
    let mut all_features = Vec::new();
    
    for class in 0..num_classes {
        // Random cluster centers
        let centers = (0..config.num_features)
            .map(|_| rand_range(-5.0, 5.0))
            .collect::<Vec<_>>();
        
        for _ in 0..samples_per_class {
            let mut sample = Vec::new();
            
            for feature_idx in 0..config.num_features {
                let center = centers[feature_idx];
                let value = center + random_normal(0.0, 1.0) * config.noise_level * 2.0;
                sample.push(value);
            }
            
            all_features.extend(sample);
        }
    }
    
    Tensor::from(all_features).reshape(vec![samples_per_class * num_classes, config.num_features])
}

fn generate_imbalanced(config: &SyntheticDatasetConfig) -> Tensor {
    let num_classes = config.num_classes.unwrap_or(3);
    let mut all_features = Vec::new();
    
    // Create imbalanced class sizes
    let class_sizes = match num_classes {
        2 => vec![config.num_samples * 8 / 10, config.num_samples * 2 / 10],
        3 => vec![config.num_samples * 6 / 10, config.num_samples * 3 / 10, config.num_samples * 1 / 10],
        _ => vec![config.num_samples / num_classes; num_classes],
    };
    
    for (class_idx, &class_size) in class_sizes.iter().enumerate() {
        for _ in 0..class_size {
            let center_x = (class_idx as f64 - (num_classes as f64 - 1.0) / 2.0) * config.cluster_separation;
            let mut sample = Vec::new();
            
            for feature_idx in 0..config.num_features {
                let base_value = if feature_idx == 0 { center_x } else { rand_range(-1.0, 1.0) };
                let noise = random_normal(0.0, 0.3) * config.noise_level;
                sample.push(base_value + noise);
            }
            
            all_features.extend(sample);
        }
    }
    
    Tensor::from(all_features).reshape(vec![config.num_samples, config.num_features])
}

// Regression dataset generators

fn generate_regression_features(config: &SyntheticDatasetConfig) -> Tensor {
    let mut all_features = Vec::new();
    
    for _ in 0..config.num_samples {
        let mut sample = Vec::new();
        
        for _ in 0..config.num_features {
            sample.push(rand_range(-2.0, 2.0));
        }
        
        all_features.extend(sample);
    }
    
    Tensor::from(all_features).reshape(vec![config.num_samples, config.num_features])
}

fn generate_linear_targets(features: &Tensor, config: &SyntheticDatasetConfig) -> Tensor {
    let num_samples = features.shape()[0];
    let weights: Vec<f64> = (0..config.num_features)
        .map(|i| rand_range(-1.0, 1.0))
        .collect();
    
    let mut targets = Vec::new();
    
    for sample_idx in 0..num_samples {
        let mut prediction = 0.0;
        
        for feature_idx in 0..config.num_features {
            let feature_value = features.data()[sample_idx * config.num_features + feature_idx];
            prediction += feature_value * weights[feature_idx];
        }
        
        // Add noise
        let noise = random_normal(0.0, 1.0) * config.noise_level;
        targets.push(prediction + noise);
    }
    
    Tensor::from(targets).reshape(vec![num_samples, 1])
}

fn generate_polynomial_targets(features: &Tensor, config: &SyntheticDatasetConfig) -> Tensor {
    let num_samples = features.shape()[0];
    let mut targets = Vec::new();
    
    for sample_idx in 0..num_samples {
        let x = features.data()[sample_idx * config.num_features];  // Use first feature as x
        
        // y = 0.5*x^2 + 2*x + 1 + noise
        let prediction = 0.5 * x * x + 2.0 * x + 1.0;
        let noise = random_normal(0.0, 1.0) * config.noise_level;
        targets.push(prediction + noise);
    }
    
    Tensor::from(targets).reshape(vec![num_samples, 1])
}

fn generate_sine_targets(features: &Tensor, config: &SyntheticDatasetConfig) -> Tensor {
    let num_samples = features.shape()[0];
    let mut targets = Vec::new();
    
    for sample_idx in 0..num_samples {
        let x = features.data()[sample_idx * config.num_features];  // Use first feature as x
        
        // y = sin(x) + noise
        let prediction = x.sin();
        let noise = random_normal(0.0, 0.3) * config.noise_level;
        targets.push(prediction + noise);
    }
    
    Tensor::from(targets).reshape(vec![num_samples, 1])
}

fn generate_step_targets(features: &Tensor, config: &SyntheticDatasetConfig) -> Tensor {
    let num_samples = features.shape()[0];
    let mut targets = Vec::new();
    
    for sample_idx in 0..num_samples {
        let x = features.data()[sample_idx * config.num_features];  // Use first feature as x
        
        // y = step function + noise
        let prediction = if x < -1.0 {
            -1.0
        } else if x < 0.0 {
            0.0
        } else if x < 1.0 {
            1.0
        } else {
            2.0
        };
        
        let noise = random_normal(0.0, 0.1) * config.noise_level;
        targets.push(prediction + noise);
    }
    
    Tensor::from(targets).reshape(vec![num_samples, 1])
}

fn generate_multimodal_targets(features: &Tensor, config: &SyntheticDatasetConfig) -> Tensor {
    let num_samples = features.shape()[0];
    let mut targets = Vec::new();
    
    for sample_idx in 0..num_samples {
        let x = features.data()[sample_idx * config.num_features];  // Use first feature as x
        
        // y = mixture of Gaussians + noise
        let center1 = -1.0;
        let center2 = 1.0;
        let weight = rand_range(0.0, 1.0);
        
        let gaussian1 = (-0.5 * (x - center1) * (x - center1)).exp();
        let gaussian2 = (-0.5 * (x - center2) * (x - center2)).exp();
        
        let prediction = weight * gaussian1 + (1.0 - weight) * gaussian2;
        let noise = random_normal(0.0, 0.1) * config.noise_level;
        targets.push(prediction + noise);
    }
    
    Tensor::from(targets).reshape(vec![num_samples, 1])
}

fn generate_periodic_targets(features: &Tensor, config: &SyntheticDatasetConfig) -> Tensor {
    let num_samples = features.shape()[0];
    let mut targets = Vec::new();
    
    for sample_idx in 0..num_samples {
        let x = features.data()[sample_idx * config.num_features];  // Use first feature as x
        
        // y = sin(2*x) + 0.5*sin(5*x) + noise
        let prediction = (2.0 * x).sin() + 0.5 * (5.0 * x).sin();
        let noise = random_normal(0.0, 0.3) * config.noise_level;
        targets.push(prediction + noise);
    }
    
    Tensor::from(targets).reshape(vec![num_samples, 1])
}

// Helper functions

fn generate_classification_labels(config: &SyntheticDatasetConfig, features: &Tensor) -> Tensor {
    let num_samples = features.shape()[0];
    let num_classes = config.num_classes.unwrap_or(2);
    let samples_per_class = num_samples / num_classes;
    
    let mut labels = Vec::new();
    
    for class in 0..num_classes {
        for _ in 0..samples_per_class {
            labels.push(class as f64);
        }
    }
    
    Tensor::from(labels).reshape(vec![num_samples, 1])
}

fn generate_feature_names(num_features: usize, prefix: &str) -> Vec<String> {
    (0..num_features)
        .map(|i| format!("{}{}", prefix, i))
        .collect()
}

fn dataset_name(dataset_type: &dyn std::fmt::Display) -> String {
    format!("{:?}", dataset_type)
}

fn save_dataset_to_csv(dataset: &GeneratedDataset, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    use std::fs::File;
    use std::io::Write;
    
    let mut file = File::create(file_path)?;
    
    // Write header
    let header: Vec<String> = dataset.feature_names.iter()
        .chain(std::iter::once(&dataset.target_name))
        .cloned()
        .collect();
    writeln!(file, "{}", header.join(","))?;
    
    // Write data
    let num_samples = dataset.features.shape()[0];
    let num_features = dataset.feature_names.len();
    
    for sample_idx in 0..num_samples {
        let mut row = Vec::new();
        
        // Features
        for feature_idx in 0..num_features {
            let value = dataset.features.data()[sample_idx * num_features + feature_idx];
            row.push(format!("{:.6}", value));
        }
        
        // Target
        let target_value = dataset.targets.data()[sample_idx];
        row.push(format!("{:.6}", target_value));
        
        writeln!(file, "{}", row.join(","))?;
    }
    
    println!("Dataset saved to {}", file_path);
    Ok(())
}

// Random number generation utilities

fn rand_range(min: f64, max: f64) -> f64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    (std::time::Instant::now().elapsed().as_nanos() as u64).hash(&mut hasher);
    let hash = hasher.finish();
    
    let normalized = (hash as f64) / (u64::MAX as f64);
    min + normalized * (max - min)
}

fn random_normal(mean: f64, std_dev: f64) -> f64 {
    // Box-Muller transform for generating normal distribution
    let u1 = rand_range(0.0, 1.0);
    let u2 = rand_range(0.0, 1.0);
    
    let z0 = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
    mean + std_dev * z0
}

/// Educational examples using synthetic datasets

/// Example 1: Simple linear classification
pub fn example_linear_classification() {
    println!("=== Example: Linear Classification ===");
    
    let config = SyntheticDatasetConfig::classification(1000, 2, 2)
        .noise_level(0.1)
        .cluster_separation(2.0)
        .save_to_file("datasets/linear_classification.csv");
    
    let dataset = generate_classification_dataset(config, ClassificationType::LinearSeparable);
    
    println!("Generated linear separable dataset:");
    println!("  Samples: {}", dataset.features.shape()[0]);
    println!("  Features: {}", dataset.features.shape()[1]);
    println!("  Classes: {}", dataset.targets.data().len());
    
    println!("\nSample data:");
    for i in 0..5 {
        let features: Vec<f64> = dataset.features.data()[i*2..i*2+2].to_vec();
        let label = dataset.targets.data()[i] as usize;
        println!("  Features: {:?}, Label: {}", features, label);
    }
    
    println!("\nThis dataset is perfect for:");
    println!("  • Learning basic classification");
    println!("  • Testing perceptrons and simple neural networks");
    println!("  • Understanding decision boundaries");
}

/// Example 2: Non-linear classification
pub fn example_nonlinear_classification() {
    println!("\n=== Example: Non-linear Classification ===");
    
    let config = SyntheticDatasetConfig::classification(500, 2, 3)
        .noise_level(0.2)
        .cluster_separation(1.5);
    
    let dataset = generate_classification_dataset(config, ClassificationType::ConcentricCircles);
    
    println!("Generated concentric circles dataset:");
    println!("  Samples: {}", dataset.features.shape()[0]);
    println!("  Features: {}", dataset.features.shape()[1]);
    
    println!("\nThis dataset demonstrates:");
    println!("  • Non-linear decision boundaries");
    println!("  • Need for non-linear classifiers");
    println!("  • Feature transformation importance");
}

/// Example 3: Regression with noise
pub fn example_regression_with_noise() {
    println!("\n=== Example: Regression with Noise ===");
    
    let config = SyntheticDatasetConfig::regression(200, 1)
        .noise_level(0.3)
        .save_to_file("datasets/polynomial_regression.csv");
    
    let dataset = generate_regression_dataset(config, RegressionType::Polynomial);
    
    println!("Generated polynomial regression dataset:");
    println!("  Samples: {}", dataset.features.shape()[0]);
    println!("  Features: {}", dataset.features.shape()[1]);
    
    println!("\nTarget values (first 10):");
    for i in 0..10 {
        let x = dataset.features.data()[i];
        let y = dataset.targets.data()[i];
        println!("  x: {:.3}, y: {:.3}", x, y);
    }
    
    println!("\nThis dataset helps learn:");
    println!("  • Polynomial regression concepts");
    println!("  • Impact of noise on model performance");
    println!("  • Overfitting vs underfitting");
}

/// Example 4: Time series regression
pub fn example_time_series() {
    println!("\n=== Example: Time Series Data ===");
    
    let config = SyntheticDatasetConfig::regression(100, 3)
        .noise_level(0.1);
    
    let dataset = generate_regression_dataset(config, RegressionType::Periodic);
    
    println!("Generated periodic time series dataset:");
    println!("  Samples: {}", dataset.features.shape()[0]);
    println!("  Features: {}", dataset.features.shape()[1]);
    
    println!("\nPeriodic pattern data (first 10 points):");
    for i in 0..10 {
        let x = dataset.features.data()[i];
        let y = dataset.targets.data()[i];
        println!("  Time step {}: x: {:.3}, y: {:.3}", i, x, y);
    }
    
    println!("\nThis dataset teaches:");
    println!("  • Time series modeling concepts");
    println!("  • Periodic pattern recognition");
    println!("  • Temporal dependencies in data");
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_linear_classification_config() {
        let config = SyntheticDatasetConfig::classification(100, 4, 3);
        assert_eq!(config.num_samples, 100);
        assert_eq!(config.num_features, 4);
        assert_eq!(config.num_classes, Some(3));
        assert!(config.noise_level >= 0.0);
    }
    
    #[test]
    fn test_regression_config() {
        let config = SyntheticDatasetConfig::regression(200, 2);
        assert_eq!(config.num_samples, 200);
        assert_eq!(config.num_features, 2);
        assert_eq!(config.num_classes, None);
    }
    
    #[test]
    fn test_dataset_generation() {
        let config = SyntheticDatasetConfig::classification(50, 2, 2)
            .noise_level(0.0)
            .cluster_separation(2.0);
        
        let dataset = generate_classification_dataset(config, ClassificationType::LinearSeparable);
        
        assert_eq!(dataset.features.shape()[0], 50);
        assert_eq!(dataset.features.shape()[1], 2);
        assert_eq!(dataset.targets.shape()[0], 50);
        assert_eq!(dataset.targets.shape()[1], 1);
    }
}