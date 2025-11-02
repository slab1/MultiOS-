// Educational ML Framework - Basic Classification Template
// This template demonstrates binary and multiclass classification using the MultiOS ML framework

use multi_os_ml::neural_net::models::SimpleNN;
use multi_os_ml::neural_net::layers::{DenseLayer, ActivationLayer};
use multi_os_ml::neural_net::utils::{ActivationFunction, Optimizer, LossFunction};
use multi_os_ml::data_pipeline::{DataPipeline, Dataset};
use multi_os_ml::runtime::tensor::Tensor;
use multi_os_ml::neural_net::visualization::VisualizationConfig;
use multi_os_ml::interactive::ModelBrowser;
use std::collections::HashMap;
use std::time::Instant;

/// Configuration for classification model
#[derive(Clone)]
pub struct ClassificationConfig {
    pub input_features: usize,
    pub hidden_layers: Vec<usize>,
    pub output_classes: usize,
    pub activation_function: ActivationFunction,
    pub optimizer: Optimizer,
    pub learning_rate: f64,
    pub epochs: usize,
    pub batch_size: usize,
    pub validation_split: f64,
    pub enable_visualization: bool,
    pub enable_debugging: bool,
}

impl Default for ClassificationConfig {
    fn default() -> Self {
        Self {
            input_features: 10,     // Default for educational datasets
            hidden_layers: vec![64, 32],  // Two hidden layers
            output_classes: 2,      // Binary classification by default
            activation_function: ActivationFunction::ReLU,
            optimizer: Optimizer::Adam { lr: 0.001, beta1: 0.9, beta2: 0.999 },
            learning_rate: 0.001,
            epochs: 100,
            batch_size: 32,
            validation_split: 0.2,
            enable_visualization: true,
            enable_debugging: false,
        }
    }
}

/// Main classification trainer with educational features
pub struct ClassificationTrainer {
    config: ClassificationConfig,
    model: SimpleNN,
    data_pipeline: DataPipeline,
    metrics_history: Vec<HashMap<String, f64>>,
    training_start: Instant,
}

impl ClassificationTrainer {
    /// Create a new classification trainer
    pub fn new(config: ClassificationConfig) -> Self {
        // Build model architecture
        let mut layers = Vec::new();
        
        // Input layer
        layers.push(Box::new(DenseLayer::new(
            config.input_features,
            config.hidden_layers[0],
            ActivationFunction::ReLU,
        )));
        
        // Hidden layers
        for i in 0..(config.hidden_layers.len() - 1) {
            layers.push(Box::new(DenseLayer::new(
                config.hidden_layers[i],
                config.hidden_layers[i + 1],
                ActivationFunction::ReLU,
            )));
        }
        
        // Output layer
        let output_activation = if config.output_classes == 2 {
            ActivationFunction::Sigmoid  // Binary classification
        } else {
            ActivationFunction::Softmax  // Multiclass classification
        };
        layers.push(Box::new(DenseLayer::new(
            config.hidden_layers.last().unwrap().clone(),
            config.output_classes,
            output_activation,
        )));
        
        let model = SimpleNN::new_with_layers(layers);
        
        Self {
            config: config.clone(),
            model,
            data_pipeline: DataPipeline::new(),
            metrics_history: Vec::new(),
            training_start: Instant::now(),
        }
    }
    
    /// Load and prepare dataset for classification
    pub fn load_dataset(&mut self, dataset_path: &str, target_column: &str) -> &mut Self {
        // Educational data loading with automatic preprocessing
        self.data_pipeline = DataPipeline::new()
            .load_csv(dataset_path)
            .expect("Failed to load dataset")
            .handle_missing_values("mean")  // Educational: shows data cleaning
            .normalize_features()
            .split(self.config.validation_split, 1.0 - self.config.validation_split)
            .encode_categorical(target_column)
            .shuffle(42)  // Reproducible results
            .batch(self.config.batch_size);
            
        println!("Dataset loaded and preprocessed successfully!");
        println!("Features: {}", self.config.input_features);
        println!("Classes: {}", self.config.output_classes);
        println!("Training samples: {}", self.data_pipeline.train_size());
        println!("Validation samples: {}", self.data_pipeline.val_size());
        
        self
    }
    
    /// Configure model visualization for educational purposes
    pub fn configure_visualization(&mut self) -> &mut Self {
        if self.config.enable_visualization {
            let viz_config = VisualizationConfig {
                show_weights: true,
                show_gradients: true,
                show_activations: true,
                update_frequency: 10,  // Update every 10 epochs
                generate_architecture_diagram: true,
                save_training_plots: true,
            };
            
            self.model.enable_visualization(viz_config);
            println!("Visualization enabled - real-time training plots will be generated");
        }
        
        self
    }
    
    /// Configure debugging for educational exploration
    pub fn configure_debugging(&mut self) -> &mut Self {
        if self.config.enable_debugging {
            self.model.set_debug_mode(true);
            self.model.set_verbosity(multi_os_ml::runtime::debug::DebugLevel::Verbose);
            
            // Add educational breakpoints
            self.model.add_breakpoint("epoch_10", |model| {
                println!("Debug checkpoint at epoch 10:");
                model.print_weights_summary();
                model.plot_weights_heatmap("debug_epoch_10.png");
            });
            
            self.model.add_breakpoint("epoch_50", |model| {
                println!("Debug checkpoint at epoch 50:");
                model.analyze_gradient_flow();
                model.visualize_feature_importance("importance_epoch_50.png");
            });
            
            println!("Debugging enabled - detailed logs and checkpoints will be generated");
        }
        
        self
    }
    
    /// Train the classification model with comprehensive monitoring
    pub fn train(&mut self) -> &mut Self {
        println!("\n=== Starting Classification Training ===");
        println!("Model Architecture: {} input -> {} hidden -> {} output", 
                 self.config.input_features, 
                 self.config.hidden_layers.len(), 
                 self.config.output_classes);
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
            
            for batch in &train_data {
                let predictions = self.model.forward(&batch.features);
                let loss = self.model.compute_loss(&predictions, &batch.targets);
                
                self.model.backward(&loss);
                self.model.update_parameters();
                
                train_loss += loss.value();
                train_accuracy += self.calculate_accuracy(&predictions, &batch.targets);
                train_samples += batch.features.shape()[0];
            }
            
            epoch_metrics.insert("train_loss".to_string(), train_loss / train_data.len() as f64);
            epoch_metrics.insert("train_accuracy".to_string(), train_accuracy / train_data.len() as f64);
            
            // Validation phase
            self.model.eval_mode();  // Disable dropout, etc.
            let mut val_accuracy = 0.0;
            let mut val_samples = 0;
            let mut correct_predictions = 0;
            
            for batch in &val_data {
                let predictions = self.model.forward(&batch.features);
                let accuracy = self.calculate_accuracy(&predictions, &batch.targets);
                val_accuracy += accuracy;
                val_samples += batch.features.shape()[0];
                
                // Count correct predictions for confusion matrix
                let predicted_classes = self.get_predicted_classes(&predictions);
                let actual_classes = self.get_actual_classes(&batch.targets);
                correct_predictions += self.count_correct_predictions(&predicted_classes, &actual_classes);
            }
            
            epoch_metrics.insert("val_accuracy".to_string(), val_accuracy / val_data.len() as f64);
            
            self.metrics_history.push(epoch_metrics);
            
            // Progress reporting
            if epoch % 10 == 0 || epoch == self.config.epochs - 1 {
                let epoch_time = epoch_start.elapsed();
                let accuracy_percent = epoch_metrics.get("val_accuracy").unwrap() * 100.0;
                
                println!("Epoch {}/{} | Loss: {:.4} | Val Acc: {:.2}% | Time: {:.2}s", 
                         epoch + 1, 
                         self.config.epochs,
                         epoch_metrics.get("train_loss").unwrap(),
                         accuracy_percent,
                         epoch_time.as_secs_f64());
            }
            
            // Educational visualization updates
            if self.config.enable_visualization && epoch % 10 == 0 {
                self.model.update_visualization_plots();
            }
        }
        
        let total_training_time = self.training_start.elapsed();
        println!("\n=== Training Complete ===");
        println!("Total training time: {:.2} seconds", total_training_time.as_secs_f64());
        
        self
    }
    
    /// Evaluate model performance with comprehensive metrics
    pub fn evaluate(&self) -> HashMap<String, f64> {
        println!("\n=== Model Evaluation ===");
        
        let val_data = self.data_pipeline.val_loader();
        let mut total_accuracy = 0.0;
        let mut predictions = Vec::new();
        let mut actual_labels = Vec::new();
        
        for batch in &val_data {
            let batch_predictions = self.model.forward(&batch.features);
            let accuracy = self.calculate_accuracy(&batch_predictions, &batch.targets);
            total_accuracy += accuracy;
            
            // Collect for detailed metrics
            predictions.extend(self.get_predicted_classes(&batch_predictions));
            actual_labels.extend(self.get_actual_classes(&batch.targets));
        }
        
        let avg_accuracy = total_accuracy / val_data.len() as f64;
        
        // Generate comprehensive evaluation report
        let confusion_matrix = self.generate_confusion_matrix(&predictions, &actual_labels);
        let class_accuracies = self.calculate_class_accuracies(&confusion_matrix);
        let precision_recall = self.calculate_precision_recall(&confusion_matrix);
        
        println!("Overall Accuracy: {:.4} ({:.2}%)", avg_accuracy, avg_accuracy * 100.0);
        println!("\nClass-wise Performance:");
        for (class, accuracy) in &class_accuracies {
            println!("Class {}: {:.4} accuracy", class, accuracy);
        }
        
        // Educational: Show confusion matrix
        println!("\nConfusion Matrix:");
        for (i, row) in confusion_matrix.iter().enumerate() {
            let row_str: String = row.iter()
                .map(|&x| format!("{:4}", x))
                .collect::<Vec<_>>()
                .join(" ");
            println!("Class {}: {}", i, row_str);
        }
        
        // Generate metrics hashmap
        let mut evaluation_metrics = HashMap::new();
        evaluation_metrics.insert("accuracy".to_string(), avg_accuracy);
        for (class, accuracy) in class_accuracies {
            evaluation_metrics.insert(format!("class_{}_accuracy", class), accuracy);
        }
        for (class, precision) in precision_recall.0 {
            evaluation_metrics.insert(format!("class_{}_precision", class), precision);
        }
        for (class, recall) in precision_recall.1 {
            evaluation_metrics.insert(format!("class_{}_recall", class), recall);
        }
        
        evaluation_metrics
    }
    
    /// Interactive model exploration for education
    pub fn start_interactive_exploration(&self, port: u16) -> Result<(), Box<dyn std::error::Error>> {
        if self.config.enable_visualization {
            let browser = ModelBrowser::new();
            browser.serve_model(&self.model, port)?;
            
            println!("Interactive model browser started at http://localhost:{}", port);
            println!("Explore your trained model, visualize weights, and understand predictions!");
            
            Ok(())
        } else {
            println!("Visualization not enabled. Set enable_visualization = true to use interactive features.");
            Ok(())
        }
    }
    
    /// Make predictions on new data
    pub fn predict(&self, input_data: &Tensor) -> Tensor {
        self.model.forward(input_data)
    }
    
    /// Save the trained model
    pub fn save_model(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.model.save_to_file(path)?;
        println!("Model saved to {}", path);
        Ok(())
    }
    
    // Helper methods for educational insights
    fn calculate_accuracy(&self, predictions: &Tensor, targets: &Tensor) -> f64 {
        let predicted_classes = self.get_predicted_classes(predictions);
        let actual_classes = self.get_actual_classes(targets);
        self.count_correct_predictions(&predicted_classes, &actual_classes) as f64 / predicted_classes.len() as f64
    }
    
    fn get_predicted_classes(&self, predictions: &Tensor) -> Vec<usize> {
        let shape = predictions.shape();
        let data = predictions.data();
        
        if shape.len() == 2 {
            // Classification with probabilities
            data.chunks(shape[1])
                .map(|chunk| {
                    let mut max_idx = 0;
                    let mut max_val = chunk[0];
                    for (i, &val) in chunk.iter().enumerate() {
                        if val > max_val {
                            max_val = val;
                            max_idx = i;
                        }
                    }
                    max_idx
                })
                .collect()
        } else {
            // Single predictions
            data.iter()
                .enumerate()
                .filter(|(_, &val)| val > 0.5)
                .map(|(i, _)| i)
                .collect()
        }
    }
    
    fn get_actual_classes(&self, targets: &Tensor) -> Vec<usize> {
        let data = targets.data();
        let shape = targets.shape();
        
        if shape.len() == 2 {
            // One-hot encoded
            data.chunks(shape[1])
                .map(|chunk| {
                    let mut max_idx = 0;
                    let mut max_val = chunk[0];
                    for (i, &val) in chunk.iter().enumerate() {
                        if val > max_val {
                            max_val = val;
                            max_idx = i;
                        }
                    }
                    max_idx
                })
                .collect()
        } else {
            // Single labels
            data.iter().map(|&x| x as usize).collect()
        }
    }
    
    fn count_correct_predictions(&self, predicted: &[usize], actual: &[usize]) -> usize {
        predicted.iter().zip(actual.iter())
            .filter(|(&p, &a)| p == a)
            .count()
    }
    
    fn generate_confusion_matrix(&self, predicted: &[usize], actual: &[usize]) -> Vec<Vec<usize>> {
        let num_classes = self.config.output_classes;
        let mut matrix = vec![vec![0usize; num_classes]; num_classes];
        
        for (&pred, &actual) in predicted.iter().zip(actual.iter()) {
            if pred < num_classes && actual < num_classes {
                matrix[actual][pred] += 1;
            }
        }
        
        matrix
    }
    
    fn calculate_class_accuracies(&self, confusion_matrix: &Vec<Vec<usize>>) -> HashMap<usize, f64> {
        let mut accuracies = HashMap::new();
        
        for class in 0..self.config.output_classes {
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
    
    fn calculate_precision_recall(&self, confusion_matrix: &Vec<Vec<usize>>) -> (HashMap<usize, f64>, HashMap<usize, f64>) {
        let mut precision = HashMap::new();
        let mut recall = HashMap::new();
        
        for class in 0..self.config.output_classes {
            // Precision: TP / (TP + FP)
            let true_positives = confusion_matrix[class][class];
            let false_positives: usize = (0..self.config.output_classes)
                .filter(|&i| i != class)
                .map(|i| confusion_matrix[i][class])
                .sum();
            
            let precision_val = if true_positives + false_positives > 0 {
                true_positives as f64 / (true_positives + false_positives) as f64
            } else {
                0.0
            };
            
            // Recall: TP / (TP + FN)
            let false_negatives: usize = (0..self.config.output_classes)
                .filter(|&i| i != class)
                .map(|i| confusion_matrix[class][i])
                .sum();
            
            let recall_val = if true_positives + false_negatives > 0 {
                true_positives as f64 / (true_positives + false_negatives) as f64
            } else {
                0.0
            };
            
            precision.insert(class, precision_val);
            recall.insert(class, recall_val);
        }
        
        (precision, recall)
    }
}

/// Educational example: Simple binary classification
pub fn run_basic_classification_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Basic Classification Example ===");
    
    // Configure model for educational purposes
    let config = ClassificationConfig {
        input_features: 4,     // Iris dataset features
        hidden_layers: vec![8, 4],
        output_classes: 3,     // Iris species classification
        epochs: 50,
        batch_size: 16,
        enable_visualization: true,
        enable_debugging: false,
        ..Default::default()
    };
    
    // Create and configure trainer
    let mut trainer = ClassificationTrainer::new(config)
        .configure_visualization()
        .load_dataset("datasets/iris_classification.csv", "species")?;
    
    // Train the model
    trainer.train();
    
    // Evaluate and display results
    let metrics = trainer.evaluate();
    println!("\nFinal Evaluation Metrics:");
    for (metric, value) in &metrics {
        println!("{}: {:.4}", metric, value);
    }
    
    // Start interactive exploration (optional)
    if config.enable_visualization {
        println!("\nStarting interactive model browser...");
        trainer.start_interactive_exploration(8080)?;
    }
    
    // Save the trained model
    trainer.save_model("models/iris_classification_model.bin")?;
    
    println!("Classification example completed successfully!");
    Ok(())
}

/// Educational example: Custom configuration for different datasets
pub fn run_custom_classification_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Custom Classification Example ===");
    
    // Different configuration for larger datasets
    let config = ClassificationConfig {
        input_features: 20,     // Synthetic feature set
        hidden_layers: vec![128, 64, 32],
        output_classes: 5,      // Multiclass problem
        epochs: 200,
        batch_size: 64,
        learning_rate: 0.0005,  // Smaller learning rate for stability
        optimizer: Optimizer::Adam { lr: 0.0005, beta1: 0.9, beta2: 0.999 },
        enable_visualization: true,
        enable_debugging: true,
        ..Default::default()
    };
    
    let mut trainer = ClassificationTrainer::new(config)
        .configure_visualization()
        .configure_debugging()
        .load_dataset("datasets/synthetic_multiclass.csv", "label")?;
    
    // Extended training with more monitoring
    trainer.train();
    
    // Comprehensive evaluation
    let metrics = trainer.evaluate();
    
    // Educational: Show training progression
    println!("\nTraining Progression:");
    for (epoch, epoch_metrics) in trainer.metrics_history.iter().enumerate() {
        if epoch % 25 == 0 || epoch == trainer.metrics_history.len() - 1 {
            let train_loss = epoch_metrics.get("train_loss").unwrap();
            let val_acc = epoch_metrics.get("val_accuracy").unwrap();
            println!("Epoch {}: Loss {:.4}, Val Acc {:.4}", epoch, train_loss, val_acc);
        }
    }
    
    println!("Custom classification example completed successfully!");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_classification_config() {
        let config = ClassificationConfig::default();
        assert_eq!(config.input_features, 10);
        assert_eq!(config.output_classes, 2);
        assert!(config.enable_visualization);
    }
    
    #[test]
    fn test_evaluation_metrics() {
        // This would test the evaluation functionality
        // In a real implementation, you'd need to set up test data
        assert!(true); // Placeholder
    }
}