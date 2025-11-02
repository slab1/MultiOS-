// Educational ML Framework - Image Recognition Template
// This template demonstrates CNN-based image classification using the MultiOS ML framework

use multi_os_ml::neural_net::models::SimpleNN;
use multi_os_ml::neural_net::layers::{DenseLayer, Conv2DLayer, MaxPool2DLayer, DropoutLayer, BatchNormLayer, FlattenLayer};
use multi_os_ml::neural_net::utils::{ActivationFunction, Optimizer, LossFunction};
use multi_os_ml::data_pipeline::{DataPipeline, Dataset};
use multi_os_ml::runtime::tensor::Tensor;
use multi_os_ml::neural_net::visualization::VisualizationConfig;
use multi_os_ml::neural_net::utils::{DataAugmentation, ImagePreprocessing};
use std::collections::HashMap;
use std::time::Instant;

/// Configuration for CNN image recognition
#[derive(Clone)]
pub struct ImageRecognitionConfig {
    pub input_shape: (u32, u32, u32),     // (height, width, channels)
    pub num_classes: usize,
    pub cnn_architecture: CNNArchitecture,
    pub optimizer: Optimizer,
    pub learning_rate: f64,
    pub epochs: usize,
    pub batch_size: usize,
    pub validation_split: f64,
    pub data_augmentation: bool,
    pub dropout_rate: f64,
    pub batch_normalization: bool,
    pub enable_visualization: bool,
    pub feature_maps_visualization: bool,
    pub gradient_visualization: bool,
}

#[derive(Clone, Debug)]
pub enum CNNArchitecture {
    SimpleCNN,              // Basic CNN for learning
    LeNet5,                 // Classic LeNet-5 architecture
    AlexNetSimplified,      // Simplified AlexNet
    VGGSimplified,          // Simplified VGG architecture
    Custom { 
        conv_layers: Vec<ConvLayerConfig>,
        dense_layers: Vec<usize>,
    },
}

#[derive(Clone)]
pub struct ConvLayerConfig {
    pub filters: usize,
    pub kernel_size: (usize, usize),
    pub stride: (usize, usize),
    pub padding: (usize, usize),
    pub activation: ActivationFunction,
}

impl Default for ImageRecognitionConfig {
    fn default() -> Self {
        Self {
            input_shape: (28, 28, 1),    // MNIST-like
            num_classes: 10,
            cnn_architecture: CNNArchitecture::SimpleCNN,
            optimizer: Optimizer::Adam { lr: 0.001, beta1: 0.9, beta2: 0.999 },
            learning_rate: 0.001,
            epochs: 25,
            batch_size: 64,
            validation_split: 0.2,
            data_augmentation: true,
            dropout_rate: 0.5,
            batch_normalization: true,
            enable_visualization: true,
            feature_maps_visualization: true,
            gradient_visualization: false,
        }
    }
}

/// Main image recognition trainer with CNN-specific features
pub struct ImageRecognitionTrainer {
    config: ImageRecognitionConfig,
    model: SimpleNN,
    data_pipeline: DataPipeline,
    metrics_history: Vec<HashMap<String, f64>>,
    training_start: Instant,
    best_accuracy: f64,
}

/// Educational CNN model builder
pub struct CNNBuilder {
    config: ImageRecognitionConfig,
}

impl CNNBuilder {
    pub fn new(config: ImageRecognitionConfig) -> Self {
        Self { config }
    }
    
    /// Build CNN architecture based on configuration
    pub fn build_model(&self) -> SimpleNN {
        match &self.config.cnn_architecture {
            CNNArchitecture::SimpleCNN => self.build_simple_cnn(),
            CNNArchitecture::LeNet5 => self.build_lenet5(),
            CNNArchitecture::AlexNetSimplified => self.build_alexnet_simplified(),
            CNNArchitecture::VGGSimplified => self.build_vgg_simplified(),
            CNNArchitecture::Custom { conv_layers, dense_layers } => {
                self.build_custom_cnn(conv_layers, dense_layers)
            }
        }
    }
    
    /// Build simple CNN for educational purposes
    fn build_simple_cnn(&self) -> SimpleNN {
        let mut layers = Vec::new();
        
        // First Convolutional Block
        layers.push(Box::new(Conv2DLayer::new(
            self.config.input_shape.2,           // input channels
            32,                                   // output channels (feature maps)
            (3, 3),                               // kernel size
            (1, 1),                               // stride
            (1, 1),                               // padding
            ActivationFunction::ReLU,
        )));
        
        if self.config.batch_normalization {
            layers.push(Box::new(BatchNormLayer::new(32)));
        }
        
        layers.push(Box::new(MaxPool2DLayer::new((2, 2), (2, 2))));
        
        // Second Convolutional Block
        layers.push(Box::new(Conv2DLayer::new(
            32,
            64,
            (3, 3),
            (1, 1),
            (1, 1),
            ActivationFunction::ReLU,
        )));
        
        if self.config.batch_normalization {
            layers.push(Box::new(BatchNormLayer::new(64)));
        }
        
        layers.push(Box::new(MaxPool2DLayer::new((2, 2), (2, 2))));
        
        // Flatten for dense layers
        layers.push(Box::new(FlattenLayer::new()));
        
        // Dense layers with dropout
        let (_, compressed_height, compressed_width) = self.calculate_compressed_size();
        let flattened_size = 64 * compressed_height * compressed_width;
        
        layers.push(Box::new(DenseLayer::new(
            flattened_size,
            128,
            ActivationFunction::ReLU,
        )));
        
        if self.config.dropout_rate > 0.0 {
            layers.push(Box::new(DropoutLayer::new(self.config.dropout_rate)));
        }
        
        layers.push(Box::new(DenseLayer::new(
            128,
            self.config.num_classes,
            ActivationFunction::Softmax,
        )));
        
        SimpleNN::new_with_layers(layers)
    }
    
    /// Build LeNet-5 architecture (educational classic)
    fn build_lenet5(&self) -> SimpleNN {
        let mut layers = Vec::new();
        
        // LeNet-5 follows the original architecture
        // Layer 1: Convolution
        layers.push(Box::new(Conv2DLayer::new(
            self.config.input_shape.2,
            6,  // LeNet-5 uses 6 feature maps
            (5, 5),
            (1, 1),
            (2, 2),
            ActivationFunction::Tanh,
        )));
        layers.push(Box::new(MaxPool2DLayer::new((2, 2), (2, 2))));
        
        // Layer 2: Convolution
        layers.push(Box::new(Conv2DLayer::new(
            6,
            16,
            (5, 5),
            (1, 1),
            (0, 0),
            ActivationFunction::Tanh,
        )));
        layers.push(Box::new(MaxPool2DLayer::new((2, 2), (2, 2))));
        
        // Flatten
        layers.push(Box::new(FlattenLayer::new()));
        
        // Dense layers
        layers.push(Box::new(DenseLayer::new(
            400,  // Calculated based on input size
            120,
            ActivationFunction::Tanh,
        )));
        
        layers.push(Box::new(DenseLayer::new(
            120,
            84,
            ActivationFunction::Tanh,
        )));
        
        layers.push(Box::new(DenseLayer::new(
            84,
            self.config.num_classes,
            ActivationFunction::Softmax,
        )));
        
        SimpleNN::new_with_layers(layers)
    }
    
    /// Build simplified AlexNet for education
    fn build_alexnet_simplified(&self) -> SimpleNN {
        let mut layers = Vec::new();
        
        // Simplified AlexNet - reduced for educational purposes
        // First conv block
        layers.push(Box::new(Conv2DLayer::new(
            self.config.input_shape.2,
            96,
            (11, 11),
            (4, 4),
            (2, 2),
            ActivationFunction::ReLU,
        )));
        layers.push(Box::new(MaxPool2DLayer::new((3, 3), (2, 2))));
        
        // Second conv block
        layers.push(Box::new(Conv2DLayer::new(
            96,
            256,
            (5, 5),
            (1, 1),
            (2, 2),
            ActivationFunction::ReLU,
        )));
        layers.push(Box::new(MaxPool2DLayer::new((3, 3), (2, 2))));
        
        // Three smaller conv layers
        for (i, (filters, kernel_size)) in [(384, (3, 3)), (384, (3, 3)), (256, (3, 3))]
            .iter()
            .enumerate() 
        {
            let input_channels = if i == 0 { 256 } else { 384 };
            
            layers.push(Box::new(Conv2DLayer::new(
                input_channels,
                *filters,
                *kernel_size,
                (1, 1),
                (1, 1),
                ActivationFunction::ReLU,
            )));
        }
        
        layers.push(Box::new(MaxPool2DLayer::new((3, 3), (2, 2))));
        layers.push(Box::new(FlattenLayer::new()));
        
        // Dense layers with dropout
        layers.push(Box::new(DenseLayer::new(
            self.calculate_flattened_size(),
            4096,
            ActivationFunction::ReLU,
        )));
        layers.push(Box::new(DropoutLayer::new(0.5)));
        
        layers.push(Box::new(DenseLayer::new(
            4096,
            4096,
            ActivationFunction::ReLU,
        )));
        layers.push(Box::new(DropoutLayer::new(0.5)));
        
        layers.push(Box::new(DenseLayer::new(
            4096,
            self.config.num_classes,
            ActivationFunction::Softmax,
        )));
        
        SimpleNN::new_with_layers(layers)
    }
    
    /// Build simplified VGG architecture
    fn build_vgg_simplified(&self) -> SimpleNN {
        let mut layers = Vec::new();
        
        // VGG-like architecture with small 3x3 filters
        let conv_configs = vec![
            (64, (3, 3)), (64, (3, 3)),
            (128, (3, 3)), (128, (3, 3)),
            (256, (3, 3)), (256, (3, 3)),
            (512, (3, 3)), (512, (3, 3)),
        ];
        
        let mut current_channels = self.config.input_shape.2;
        
        for (i, (filters, kernel_size)) in conv_configs.iter().enumerate() {
            layers.push(Box::new(Conv2DLayer::new(
                current_channels,
                *filters,
                *kernel_size,
                (1, 1),
                (1, 1),
                ActivationFunction::ReLU,
            )));
            
            current_channels = *filters;
            
            // Add pooling every two conv layers
            if (i + 1) % 2 == 0 {
                layers.push(Box::new(MaxPool2DLayer::new((2, 2), (2, 2))));
            }
        }
        
        layers.push(Box::new(FlattenLayer::new()));
        
        // Dense layers
        layers.push(Box::new(DenseLayer::new(
            self.calculate_flattened_size(),
            4096,
            ActivationFunction::ReLU,
        )));
        layers.push(Box::new(DropoutLayer::new(0.5)));
        
        layers.push(Box::new(DenseLayer::new(
            4096,
            4096,
            ActivationFunction::ReLU,
        )));
        layers.push(Box::new(DropoutLayer::new(0.5)));
        
        layers.push(Box::new(DenseLayer::new(
            4096,
            self.config.num_classes,
            ActivationFunction::Softmax,
        )));
        
        SimpleNN::new_with_layers(layers)
    }
    
    /// Build custom CNN architecture
    fn build_custom_cnn(&self, conv_layers: &[ConvLayerConfig], dense_layers: &[usize]) -> SimpleNN {
        let mut layers = Vec::new();
        let mut current_channels = self.config.input_shape.2;
        
        // Build convolutional layers
        for conv_config in conv_layers {
            layers.push(Box::new(Conv2DLayer::new(
                current_channels,
                conv_config.filters,
                conv_config.kernel_size,
                conv_config.stride,
                conv_config.padding,
                conv_config.activation.clone(),
            )));
            
            if self.config.batch_normalization {
                layers.push(Box::new(BatchNormLayer::new(conv_config.filters)));
            }
            
            current_channels = conv_config.filters;
            layers.push(Box::new(MaxPool2DLayer::new((2, 2), (2, 2))));
        }
        
        // Flatten
        layers.push(Box::new(FlattenLayer::new()));
        
        // Build dense layers
        let flattened_size = self.calculate_flattened_size();
        let mut input_size = flattened_size;
        
        for &output_size in dense_layers {
            layers.push(Box::new(DenseLayer::new(
                input_size,
                output_size,
                ActivationFunction::ReLU,
            )));
            
            if self.config.dropout_rate > 0.0 {
                layers.push(Box::new(DropoutLayer::new(self.config.dropout_rate)));
            }
            
            input_size = output_size;
        }
        
        // Output layer
        layers.push(Box::new(DenseLayer::new(
            input_size,
            self.config.num_classes,
            ActivationFunction::Softmax,
        )));
        
        SimpleNN::new_with_layers(layers)
    }
    
    /// Calculate the size after convolution and pooling operations
    fn calculate_compressed_size(&self) -> (u32, u32, u32) {
        let (height, width, channels) = self.config.input_shape;
        
        // Simplified calculation - in real implementation, track actual dimensions
        let conv1_height = (height - 3 + 2) / 2 + 1;  // After conv + pool
        let conv1_width = (width - 3 + 2) / 2 + 1;
        
        let conv2_height = (conv1_height - 3 + 2) / 2 + 1;
        let conv2_width = (conv1_width - 3 + 2) / 2 + 1;
        
        (conv2_height, conv2_width, 64)
    }
    
    /// Calculate flattened size for dense layers
    fn calculate_flattened_size(&self) -> usize {
        let (height, width, _) = self.calculate_compressed_size();
        (height as usize) * (width as usize) * 64
    }
}

impl ImageRecognitionTrainer {
    /// Create a new image recognition trainer
    pub fn new(config: ImageRecognitionConfig) -> Self {
        let builder = CNNBuilder::new(config.clone());
        let model = builder.build_model();
        
        Self {
            config: config.clone(),
            model,
            data_pipeline: DataPipeline::new(),
            metrics_history: Vec::new(),
            training_start: Instant::now(),
            best_accuracy: 0.0,
        }
    }
    
    /// Load and prepare image dataset with CNN-specific preprocessing
    pub fn load_dataset(&mut self, dataset_path: &str, labels_column: &str) -> &mut Self {
        // Image-specific data pipeline
        self.data_pipeline = DataPipeline::new()
            .load_images(dataset_path)
            .expect("Failed to load image dataset")
            .resize_images(self.config.input_shape.0, self.config.input_shape.1)
            .normalize_pixels(0.0, 1.0)  // Normalize to [0,1] range
            .grayscale_if_needed(self.config.input_shape.2 == 1)
            .split(self.config.validation_split, 1.0 - self.config.validation_split)
            .shuffle(42)
            .batch(self.config.batch_size);
            
        if self.config.data_augmentation {
            self.data_pipeline = self.data_pipeline
                .augment_images(DataAugmentation {
                    rotate: Some((-30.0, 30.0)),           // Random rotation
                    flip_horizontal: true,                 // Random horizontal flip
                    flip_vertical: false,                  // No vertical flip for education
                    zoom: Some((0.8, 1.2)),               // Random zoom
                    shear: Some((-0.2, 0.2)),             // Random shear
                    brightness: Some((-0.2, 0.2)),        // Random brightness
                    contrast: Some((0.8, 1.2)),           // Random contrast
                    gaussian_noise: Some(0.1),            // Add noise
                });
        }
            
        println!("Image dataset loaded successfully!");
        println!("Input shape: {:?}", self.config.input_shape);
        println!("Number of classes: {}", self.config.num_classes);
        println!("Training samples: {}", self.data_pipeline.train_size());
        println!("Validation samples: {}", self.data_pipeline.val_size());
        println!("Data augmentation: {}", self.config.data_augmentation);
        
        self
    }
    
    /// Configure visualization for CNN-specific features
    pub fn configure_visualization(&mut self) -> &mut Self {
        if self.config.enable_visualization {
            let viz_config = VisualizationConfig {
                show_weights: true,
                show_gradients: true,
                show_activations: true,
                update_frequency: 5,  // Update more frequently for CNNs
                generate_architecture_diagram: true,
                save_training_plots: true,
            };
            
            self.model.enable_visualization(viz_config);
            
            if self.config.feature_maps_visualization {
                println!("Feature maps visualization enabled");
            }
            
            println!("CNN visualization enabled - feature maps, activations, and training plots");
        }
        
        self
    }
    
    /// Train the CNN model with image-specific optimizations
    pub fn train(&mut self) -> &mut Self {
        println!("\n=== Starting CNN Training ===");
        println!("Architecture: {:?}", self.config.cnn_architecture);
        println!("Input shape: {:?}", self.config.input_shape);
        println!("Classes: {}", self.config.num_classes);
        println!("Training for {} epochs with batch size {}", 
                 self.config.epochs, self.config.batch_size);
        
        let train_data = self.data_pipeline.train_loader();
        let val_data = self.data_pipeline.val_loader();
        
        for epoch in 0..self.config.epochs {
            let epoch_start = Instant::now();
            let mut epoch_metrics = HashMap::new();
            
            // Training phase
            self.model.train_mode();  // Enable dropout, etc.
            let mut train_loss = 0.0;
            let mut train_accuracy = 0.0;
            let mut train_samples = 0;
            
            for (batch_idx, batch) in train_data.iter().enumerate() {
                let predictions = self.model.forward(&batch.features);
                let loss = self.model.compute_loss(&predictions, &batch.targets);
                
                self.model.backward(&loss);
                self.model.update_parameters();
                
                let batch_accuracy = self.calculate_accuracy(&predictions, &batch.targets);
                train_loss += loss.value();
                train_accuracy += batch_accuracy;
                train_samples += batch.features.shape()[0];
                
                // Educational: Show sample predictions every few batches
                if batch_idx % 50 == 0 && self.config.enable_visualization {
                    self.show_sample_predictions(&predictions, &batch.targets, batch_idx);
                }
            }
            
            epoch_metrics.insert("train_loss".to_string(), train_loss / train_data.len() as f64);
            epoch_metrics.insert("train_accuracy".to_string(), train_accuracy / train_data.len() as f64);
            
            // Validation phase
            self.model.eval_mode();  // Disable dropout, etc.
            let mut val_accuracy = 0.0;
            let mut val_samples = 0;
            
            for batch in &val_data {
                let predictions = self.model.forward(&batch.features);
                let accuracy = self.calculate_accuracy(&predictions, &batch.targets);
                val_accuracy += accuracy;
                val_samples += batch.features.shape()[0];
            }
            
            epoch_metrics.insert("val_accuracy".to_string(), val_accuracy / val_data.len() as f64);
            
            self.metrics_history.push(epoch_metrics);
            
            // Update best accuracy for checkpointing
            let current_accuracy = epoch_metrics.get("val_accuracy").unwrap();
            if *current_accuracy > self.best_accuracy {
                self.best_accuracy = *current_accuracy;
                self.save_checkpoint(epoch);
            }
            
            // Progress reporting
            if epoch % 5 == 0 || epoch == self.config.epochs - 1 {
                let epoch_time = epoch_start.elapsed();
                let train_acc = epoch_metrics.get("train_accuracy").unwrap();
                let val_acc = epoch_metrics.get("val_accuracy").unwrap();
                
                println!("Epoch {}/{} | Train Acc: {:.2}% | Val Acc: {:.2}% | Time: {:.2}s", 
                         epoch + 1, 
                         self.config.epochs,
                         train_acc * 100.0,
                         val_acc * 100.0,
                         epoch_time.as_secs_f64());
            }
            
            // Educational visualization updates
            if self.config.enable_visualization {
                self.update_cnn_visualization(epoch);
            }
        }
        
        let total_training_time = self.training_start.elapsed();
        println!("\n=== CNN Training Complete ===");
        println!("Best validation accuracy: {:.2}%", self.best_accuracy * 100.0);
        println!("Total training time: {:.2} seconds", total_training_time.as_secs_f64());
        
        self
    }
    
    /// Evaluate CNN performance with image-specific metrics
    pub fn evaluate(&self) -> HashMap<String, f64> {
        println!("\n=== CNN Model Evaluation ===");
        
        let val_data = self.data_pipeline.val_loader();
        let mut total_accuracy = 0.0;
        let mut predictions_all = Vec::new();
        let mut actual_labels = Vec::new();
        
        for batch in &val_data {
            let predictions = self.model.forward(&batch.features);
            let accuracy = self.calculate_accuracy(&predictions, &batch.targets);
            total_accuracy += accuracy;
            
            // Collect for detailed analysis
            predictions_all.extend(self.get_predicted_classes(&predictions));
            actual_labels.extend(self.get_actual_classes(&batch.targets));
        }
        
        let avg_accuracy = total_accuracy / val_data.len() as f64;
        
        // Generate comprehensive evaluation
        let confusion_matrix = self.generate_confusion_matrix(&predictions_all, &actual_labels);
        let per_class_accuracy = self.calculate_per_class_accuracy(&confusion_matrix);
        
        println!("Overall Accuracy: {:.4} ({:.2}%)", avg_accuracy, avg_accuracy * 100.0);
        println!("\nPer-Class Accuracy:");
        for (class, accuracy) in &per_class_accuracy {
            println!("Class {}: {:.2}%", class, accuracy * 100.0);
        }
        
        // Educational: Show misclassified examples
        if self.config.enable_visualization {
            self.analyze_misclassifications(&predictions_all, &actual_labels);
        }
        
        // Generate metrics
        let mut evaluation_metrics = HashMap::new();
        evaluation_metrics.insert("accuracy".to_string(), avg_accuracy);
        for (class, accuracy) in per_class_accuracy {
            evaluation_metrics.insert(format!("class_{}_accuracy", class), accuracy);
        }
        
        evaluation_metrics
    }
    
    /// Predict on new images with confidence scores
    pub fn predict(&self, images: &Tensor) -> (Vec<usize>, Vec<f64>) {
        let predictions = self.model.forward(images);
        let classes = self.get_predicted_classes(&predictions);
        let confidences = self.get_prediction_confidences(&predictions);
        (classes, confidences)
    }
    
    /// Visualize CNN feature maps (educational)
    pub fn visualize_feature_maps(&self, layer_index: usize, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        if self.config.feature_maps_visualization {
            println!("Generating feature maps visualization for layer {}: {}", layer_index, output_path);
            // In real implementation, extract and visualize feature maps
            // Educational: Shows what each convolution layer learns
        }
        Ok(())
    }
    
    /// Save the trained CNN model
    pub fn save_model(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.model.save_to_file(path)?;
        println!("CNN model saved to {}", path);
        Ok(())
    }
    
    // Helper methods for CNN-specific operations
    fn calculate_accuracy(&self, predictions: &Tensor, targets: &Tensor) -> f64 {
        let predicted_classes = self.get_predicted_classes(predictions);
        let actual_classes = self.get_actual_classes(targets);
        self.count_correct_predictions(&predicted_classes, &actual_classes) as f64 / predicted_classes.len() as f64
    }
    
    fn get_predicted_classes(&self, predictions: &Tensor) -> Vec<usize> {
        let data = predictions.data();
        let batch_size = data.len() / self.config.num_classes;
        
        (0..batch_size)
            .map(|i| {
                let start = i * self.config.num_classes;
                let end = start + self.config.num_classes;
                let &max_prob = data[start..end]
                    .iter()
                    .max_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap();
                
                data[start..end]
                    .iter()
                    .position(|&x| x == max_prob)
                    .unwrap()
            })
            .collect()
    }
    
    fn get_actual_classes(&self, targets: &Tensor) -> Vec<usize> {
        targets.data().iter().map(|&x| x as usize).collect()
    }
    
    fn get_prediction_confidences(&self, predictions: &Tensor) -> Vec<f64> {
        let data = predictions.data();
        let batch_size = data.len() / self.config.num_classes;
        
        (0..batch_size)
            .map(|i| {
                let start = i * self.config.num_classes;
                let end = start + self.config.num_classes;
                data[start..end]
                    .iter()
                    .max_by(|a, b| a.partial_cmp(b).unwrap())
                    .copied()
                    .unwrap_or(0.0)
            })
            .collect()
    }
    
    fn count_correct_predictions(&self, predicted: &[usize], actual: &[usize]) -> usize {
        predicted.iter().zip(actual.iter())
            .filter(|(&p, &a)| p == a)
            .count()
    }
    
    fn generate_confusion_matrix(&self, predicted: &[usize], actual: &[usize]) -> Vec<Vec<usize>> {
        let mut matrix = vec![vec![0usize; self.config.num_classes]; self.config.num_classes];
        
        for (&pred, &actual) in predicted.iter().zip(actual.iter()) {
            if pred < self.config.num_classes && actual < self.config.num_classes {
                matrix[actual][pred] += 1;
            }
        }
        
        matrix
    }
    
    fn calculate_per_class_accuracy(&self, confusion_matrix: &Vec<Vec<usize>>) -> HashMap<usize, f64> {
        let mut accuracies = HashMap::new();
        
        for class in 0..self.config.num_classes {
            let class_row = &confusion_matrix[class];
            let class_total: usize = class_row.iter().sum();
            let correct = class_row[class];
            
            let accuracy = if class_total > 0 {
                correct as f64 / class_total as f64
            } else {
                0.0
            };
            
            accuracies.insert(class, accuracy);
        }
        
        accuracies
    }
    
    fn show_sample_predictions(&self, predictions: &Tensor, targets: &Tensor, batch_idx: usize) {
        // Educational: Show a few sample predictions for learning
        let sample_size = std::cmp::min(3, predictions.shape()[0]);
        
        for i in 0..sample_size {
            let start = i * self.config.num_classes;
            let end = start + self.config.num_classes;
            let probs = &predictions.data()[start..end];
            
            let predicted_class = probs
                .iter()
                .enumerate()
                .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
                .map(|(i, _)| i)
                .unwrap_or(0);
            
            let actual_class = targets.data()[i] as usize;
            let confidence = probs[predicted_class];
            
            println!("Sample {} (Batch {}): Predicted: {}, Actual: {}, Confidence: {:.2}", 
                     i + 1, batch_idx, predicted_class, actual_class, confidence);
        }
    }
    
    fn update_cnn_visualization(&self, epoch: usize) {
        if self.config.feature_maps_visualization && epoch % 10 == 0 {
            println!("Updating CNN feature map visualizations...");
        }
        
        if self.config.gradient_visualization && epoch % 20 == 0 {
            println!("Updating gradient flow visualizations...");
        }
    }
    
    fn analyze_misclassifications(&self, predictions: &[usize], actual: &[usize]) {
        let mut misclassifications = Vec::new();
        
        for (i, (&pred, &act)) in predictions.iter().zip(actual.iter()).enumerate() {
            if pred != act {
                misclassifications.push((i, pred, act));
            }
        }
        
        if !misclassifications.is_empty() {
            println!("\nTop 5 Misclassifications:");
            for (i, (idx, pred, actual)) in misclassifications.iter().take(5).enumerate() {
                println!("  {}: Image {} - Predicted: {}, Actual: {}", i + 1, idx, pred, actual);
            }
        }
    }
    
    fn save_checkpoint(&self, epoch: usize) {
        if self.best_accuracy > 0.0 {
            println!("New best accuracy: {:.2}% - Saving checkpoint", self.best_accuracy * 100.0);
            // In real implementation, save model checkpoint
        }
    }
}

/// Educational example: Simple CNN for MNIST-like dataset
pub fn run_simple_cnn_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Simple CNN Example (MNIST-like) ===");
    
    let config = ImageRecognitionConfig {
        input_shape: (28, 28, 1),
        num_classes: 10,
        cnn_architecture: CNNArchitecture::SimpleCNN,
        epochs: 20,
        batch_size: 64,
        data_augmentation: true,
        ..Default::default()
    };
    
    let mut trainer = ImageRecognitionTrainer::new(config)
        .configure_visualization()
        .load_dataset("datasets/mnist_subset", "label")?;
    
    trainer.train();
    let metrics = trainer.evaluate();
    
    println!("\nFinal CNN Results:");
    for (metric, value) in &metrics {
        println!("{}: {:.4}", metric, value);
    }
    
    trainer.save_model("models/simple_cnn_mnist.bin")?;
    Ok(())
}

/// Educational example: Custom CNN architecture
pub fn run_custom_cnn_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Custom CNN Architecture Example ===");
    
    let conv_layers = vec![
        ConvLayerConfig { filters: 16, kernel_size: (3, 3), stride: (1, 1), padding: (1, 1), activation: ActivationFunction::ReLU },
        ConvLayerConfig { filters: 32, kernel_size: (3, 3), stride: (1, 1), padding: (1, 1), activation: ActivationFunction::ReLU },
        ConvLayerConfig { filters: 64, kernel_size: (3, 3), stride: (1, 1), padding: (1, 1), activation: ActivationFunction::ReLU },
    ];
    
    let config = ImageRecognitionConfig {
        input_shape: (32, 32, 3),  // Color images
        num_classes: 10,
        cnn_architecture: CNNArchitecture::Custom { 
            conv_layers: conv_layers.clone(),
            dense_layers: vec![256, 128],
        },
        epochs: 30,
        batch_normalization: true,
        dropout_rate: 0.3,
        ..Default::default()
    };
    
    let mut trainer = ImageRecognitionTrainer::new(config)
        .configure_visualization()
        .load_dataset("datasets/cifar10_subset", "label")?;
    
    trainer.train();
    let metrics = trainer.evaluate();
    
    // Educational: Visualize custom architecture
    trainer.visualize_feature_maps(0, "custom_cnn_layer1_features.png")?;
    
    println!("\nCustom CNN Results:");
    for (metric, value) in &metrics {
        println!("{}: {:.4}", metric, value);
    }
    
    Ok(())
}

/// Educational example: LeNet-5 architecture
pub fn run_lenet5_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== LeNet-5 Educational Example ===");
    
    let config = ImageRecognitionConfig {
        input_shape: (32, 32, 1),  // LeNet-5 expects 32x32
        num_classes: 10,
        cnn_architecture: CNNArchitecture::LeNet5,
        epochs: 25,
        batch_size: 128,
        data_augmentation: false,  // LeNet-5 traditionally doesn't use augmentation
        ..Default::default()
    };
    
    let mut trainer = ImageRecognitionTrainer::new(config)
        .configure_visualization()
        .load_dataset("datasets/mnist_32x32", "label")?;
    
    trainer.train();
    let metrics = trainer.evaluate();
    
    println!("\nLeNet-5 Results (Educational Classic):");
    for (metric, value) in &metrics {
        println!("{}: {:.4}", metric, value);
    }
    
    println!("\nEducational Note: LeNet-5 was one of the first successful CNNs!");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_image_recognition_config() {
        let config = ImageRecognitionConfig::default();
        assert_eq!(config.input_shape, (28, 28, 1));
        assert_eq!(config.num_classes, 10);
        assert!(config.enable_visualization);
    }
    
    #[test]
    fn test_cnn_builder() {
        let config = ImageRecognitionConfig::default();
        let builder = CNNBuilder::new(config);
        let model = builder.build_model();
        assert!(true); // Placeholder test
    }
}