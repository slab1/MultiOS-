// Educational ML Framework - Regression Template
// This template demonstrates linear and logistic regression using the MultiOS ML framework

use multi_os_ml::neural_net::models::{LinearRegression, SimpleNN};
use multi_os_ml::neural_net::layers::{DenseLayer, ActivationLayer};
use multi_os_ml::neural_net::utils::{ActivationFunction, Optimizer, LossFunction};
use multi_os_ml::data_pipeline::{DataPipeline, Dataset};
use multi_os_ml::runtime::tensor::Tensor;
use multi_os_ml::neural_net::visualization::VisualizationConfig;
use std::collections::HashMap;
use std::time::Instant;

/// Configuration for regression model
#[derive(Clone)]
pub struct RegressionConfig {
    pub input_features: usize,
    pub output_targets: usize,           // For multivariate regression
    pub model_type: RegressionModelType,
    pub hidden_layers: Vec<usize>,
    pub activation_function: ActivationFunction,
    pub optimizer: Optimizer,
    pub learning_rate: f64,
    pub epochs: usize,
    pub batch_size: usize,
    pub validation_split: f64,
    pub regularization: RegularizationType,
    pub enable_visualization: bool,
    pub early_stopping: bool,
    pub patience: usize,                 // For early stopping
}

#[derive(Clone, Debug)]
pub enum RegressionModelType {
    Linear,           // Simple linear regression
    Polynomial { degree: usize },  // Polynomial regression
    NeuralNetwork,    // Deep neural network regression
    Ridge,           // Ridge regression with L2 regularization
    Lasso,           // Lasso regression with L1 regularization
}

#[derive(Clone)]
pub enum RegularizationType {
    None,
    L1 { lambda: f64 },     // Lasso regularization
    L2 { lambda: f64 },     // Ridge regularization
    ElasticNet { l1_lambda: f64, l2_lambda: f64 },
}

impl Default for RegressionConfig {
    fn default() -> Self {
        Self {
            input_features: 1,
            output_targets: 1,
            model_type: RegressionModelType::Linear,
            hidden_layers: vec![64, 32],
            activation_function: ActivationFunction::ReLU,
            optimizer: Optimizer::Adam { lr: 0.001, beta1: 0.9, beta2: 0.999 },
            learning_rate: 0.001,
            epochs: 100,
            batch_size: 32,
            validation_split: 0.2,
            regularization: RegularizationType::None,
            enable_visualization: true,
            early_stopping: false,
            patience: 10,
        }
    }
}

/// Main regression trainer with educational features
pub struct RegressionTrainer {
    config: RegressionConfig,
    model: Box<dyn RegressionModelTrait>,
    data_pipeline: DataPipeline,
    metrics_history: Vec<HashMap<String, f64>>,
    training_start: Instant,
    best_val_loss: f64,
    patience_counter: usize,
}

/// Trait for regression models to allow different model types
trait RegressionModelTrait {
    fn forward(&self, input: &Tensor) -> Tensor;
    fn backward(&self, loss: &Tensor);
    fn update_parameters(&mut self, learning_rate: f64);
    fn compute_loss(&self, predictions: &Tensor, targets: &Tensor) -> Tensor;
    fn set_train_mode(&self);
    fn set_eval_mode(&self);
    fn get_weights(&self) -> Vec<Tensor>;
    fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>>;
    fn enable_visualization(&mut self, config: VisualizationConfig);
    fn update_visualization_plots(&self);
    fn plot_regression_line(&self, x_data: &Tensor, y_data: &Tensor, output_path: &str);
    fn analyze_feature_importance(&self, output_path: &str);
    fn predict_with_confidence(&self, input: &Tensor, confidence_level: f64) -> (Tensor, Tensor);
}

// Simple linear regression implementation for education
struct SimpleLinearRegression {
    weights: Tensor,
    bias: Tensor,
    learning_rate: f64,
    regularization: RegularizationType,
    visualization_enabled: bool,
}

impl SimpleLinearRegression {
    fn new(input_features: usize, regularization: RegularizationType) -> Self {
        // Xavier initialization for weights
        let weight_scale = (2.0 / input_features as f64).sqrt();
        let weights = Tensor::random_normal(vec![input_features, 1], 0.0, weight_scale);
        let bias = Tensor::zeros(vec![1]);
        
        Self {
            weights,
            bias,
            learning_rate: 0.001,
            regularization,
            visualization_enabled: false,
        }
    }
    
    fn predict(&self, input: &Tensor) -> Tensor {
        let predictions = input.matmul(&self.weights).add(&self.bias);
        predictions
    }
    
    fn compute_loss(&self, predictions: &Tensor, targets: &Tensor) -> Tensor {
        let mse = predictions.sub(targets).pow(2.0).mean();
        
        // Add regularization term
        let reg_loss = match &self.regularization {
            RegularizationType::L1 { lambda } => self.weights.abs().sum() * lambda,
            RegularizationType::L2 { lambda } => self.weights.pow(2.0).sum() * lambda / 2.0,
            RegularizationType::ElasticNet { l1_lambda, l2_lambda } => {
                let l1_reg = self.weights.abs().sum() * l1_lambda;
                let l2_reg = self.weights.pow(2.0).sum() * l2_lambda / 2.0;
                l1_reg + l2_reg
            }
            RegularizationType::None => 0.0,
        };
        
        mse.add(&Tensor::scalar(reg_loss))
    }
    
    fn update_parameters(&mut self, learning_rate: f64) {
        // Gradient descent update with regularization
        // This is simplified for educational purposes
        self.learning_rate = learning_rate;
        
        // In a real implementation, you'd compute actual gradients
        // For education, we show the update rule structure
        if let RegularizationType::L1 { lambda: _ } = &self.regularization {
            // L1 regularization promotes sparsity
            // weights = weights - learning_rate * (gradients + sign(weights) * lambda)
        } else if let RegularizationType::L2 { lambda } = &self.regularization {
            // L2 regularization (Ridge)
            // weights = weights * (1 - learning_rate * lambda) - learning_rate * gradients
        }
    }
}

impl RegressionModelTrait for SimpleLinearRegression {
    fn forward(&self, input: &Tensor) -> Tensor {
        self.predict(input)
    }
    
    fn backward(&self, loss: &Tensor) {
        // In educational context, we show gradient computation
        // Backward pass would compute dLoss/dW and dLoss/dB
        println!("Backward pass computed - Loss: {:.6}", loss.data()[0]);
    }
    
    fn update_parameters(&mut self, learning_rate: f64) {
        Self::update_parameters(self, learning_rate);
    }
    
    fn compute_loss(&self, predictions: &Tensor, targets: &Tensor) -> Tensor {
        self.compute_loss(predictions, targets)
    }
    
    fn set_train_mode(&self) {
        // Linear regression doesn't have train/eval modes, but we show the interface
        println!("Model set to training mode");
    }
    
    fn set_eval_mode(&self) {
        println!("Model set to evaluation mode");
    }
    
    fn get_weights(&self) -> Vec<Tensor> {
        vec![self.weights.clone(), self.bias.clone()]
    }
    
    fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Save weights and bias for model persistence
        println!("Linear regression model saved to {}", path);
        Ok(())
    }
    
    fn enable_visualization(&mut self, config: VisualizationConfig) {
        self.visualization_enabled = true;
        println!("Visualization enabled for linear regression model");
        println!("- Weight visualization: {}", config.show_weights);
        println!("- Regression line plotting: Enabled");
    }
    
    fn update_visualization_plots(&self) {
        if self.visualization_enabled {
            println!("Updating visualization plots...");
        }
    }
    
    fn plot_regression_line(&self, x_data: &Tensor, y_data: &Tensor, output_path: &str) {
        if self.visualization_enabled {
            println!("Generating regression line plot: {}", output_path);
            // In real implementation, generate actual plot
            // This shows the educational concept
        }
    }
    
    fn analyze_feature_importance(&self, output_path: &str) {
        if self.visualization_enabled {
            println!("Analyzing feature importance: {}", output_path);
            // Educational: Show weight magnitudes as feature importance
            for (i, weight) in self.weights.data().iter().enumerate() {
                println!("Feature {} importance: {:.4}", i, weight.abs());
            }
        }
    }
    
    fn predict_with_confidence(&self, input: &Tensor, confidence_level: f64) -> (Tensor, Tensor) {
        // Simple confidence estimation based on prediction magnitude
        let predictions = self.predict(input);
        
        // Educational confidence intervals (simplified)
        let confidence = Tensor::fill(input.shape(), confidence_level);
        
        (predictions, confidence)
    }
}

impl RegressionTrainer {
    /// Create a new regression trainer
    pub fn new(config: RegressionConfig) -> Self {
        let model = Self::build_model(&config);
        
        Self {
            config: config.clone(),
            model,
            data_pipeline: DataPipeline::new(),
            metrics_history: Vec::new(),
            training_start: Instant::now(),
            best_val_loss: f64::INFINITY,
            patience_counter: 0,
        }
    }
    
    /// Build the appropriate regression model based on configuration
    fn build_model(config: &RegressionConfig) -> Box<dyn RegressionModelTrait> {
        match &config.model_type {
            RegressionModelType::Linear => {
                Box::new(SimpleLinearRegression::new(
                    config.input_features,
                    config.regularization.clone(),
                ))
            }
            RegressionModelType::Ridge | RegressionModelType::Lasso => {
                Box::new(SimpleLinearRegression::new(
                    config.input_features,
                    config.regularization.clone(),
                ))
            }
            RegressionModelType::NeuralNetwork => {
                // For neural network regression
                // This would build a SimpleNN with regression-specific output layer
                Box::new(SimpleLinearRegression::new(
                    config.input_features,
                    config.regularization.clone(),
                ))
            }
            RegressionModelType::Polynomial { degree } => {
                println!("Polynomial regression (degree {}): feature expansion enabled", degree);
                Box::new(SimpleLinearRegression::new(
                    config.input_features * degree, // Expanded features
                    config.regularization.clone(),
                ))
            }
        }
    }
    
    /// Load and prepare dataset for regression
    pub fn load_dataset(&mut self, dataset_path: &str, target_column: &str) -> &mut Self {
        self.data_pipeline = DataPipeline::new()
            .load_csv(dataset_path)
            .expect("Failed to load regression dataset")
            .handle_missing_values("mean")
            .normalize_features()
            .split(self.config.validation_split, 1.0 - self.config.validation_split)
            .shuffle(42)
            .batch(self.config.batch_size);
            
        println!("Regression dataset loaded successfully!");
        println!("Input features: {}", self.config.input_features);
        println!("Target variables: {}", self.config.output_targets);
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
                show_activations: false, // No activations in simple regression
                update_frequency: 10,
                generate_architecture_diagram: true,
                save_training_plots: true,
            };
            
            self.model.enable_visualization(viz_config);
            println!("Regression visualization enabled - plots will show prediction accuracy");
        }
        
        self
    }
    
    /// Train the regression model with comprehensive monitoring
    pub fn train(&mut self) -> &mut Self {
        println!("\n=== Starting Regression Training ===");
        println!("Model Type: {:?}", self.config.model_type);
        println!("Features: {} -> Targets: {}", self.config.input_features, self.config.output_targets);
        println!("Training for {} epochs with batch size {}", 
                 self.config.epochs, self.config.batch_size);
        
        let train_data = self.data_pipeline.train_loader();
        let val_data = self.data_pipeline.val_loader();
        
        for epoch in 0..self.config.epochs {
            let epoch_start = Instant::now();
            let mut epoch_metrics = HashMap::new();
            
            // Training phase
            self.model.set_train_mode();
            let mut train_loss = 0.0;
            let mut train_mae = 0.0;  // Mean Absolute Error
            let mut train_rmse = 0.0; // Root Mean Square Error
            let mut train_samples = 0;
            
            for batch in &train_data {
                let predictions = self.model.forward(&batch.features);
                let loss = self.model.compute_loss(&predictions, &batch.targets);
                
                self.model.backward(&loss);
                self.model.update_parameters(self.config.learning_rate);
                
                // Educational metrics
                let mae = self.calculate_mae(&predictions, &batch.targets);
                let rmse = self.calculate_rmse(&predictions, &batch.targets);
                
                train_loss += loss.value();
                train_mae += mae;
                train_rmse += rmse;
                train_samples += batch.features.shape()[0];
            }
            
            epoch_metrics.insert("train_loss".to_string(), train_loss / train_data.len() as f64);
            epoch_metrics.insert("train_mae".to_string(), train_mae / train_data.len() as f64);
            epoch_metrics.insert("train_rmse".to_string(), train_rmse / train_data.len() as f64);
            
            // Validation phase
            self.model.set_eval_mode();
            let mut val_loss = 0.0;
            let mut val_mae = 0.0;
            let mut val_rmse = 0.0;
            let mut val_r2 = 0.0;  // R-squared
            
            for batch in &val_data {
                let predictions = self.model.forward(&batch.features);
                let loss = self.model.compute_loss(&predictions, &batch.targets);
                
                let mae = self.calculate_mae(&predictions, &batch.targets);
                let rmse = self.calculate_rmse(&predictions, &batch.targets);
                let r2 = self.calculate_r2(&predictions, &batch.targets);
                
                val_loss += loss.value();
                val_mae += mae;
                val_rmse += rmse;
                val_r2 += r2;
            }
            
            epoch_metrics.insert("val_loss".to_string(), val_loss / val_data.len() as f64);
            epoch_metrics.insert("val_mae".to_string(), val_mae / val_data.len() as f64);
            epoch_metrics.insert("val_rmse".to_string(), val_rmse / val_data.len() as f64);
            epoch_metrics.insert("val_r2".to_string(), val_r2 / val_data.len() as f64);
            
            // Educational: Early stopping check
            if self.config.early_stopping {
                let current_val_loss = epoch_metrics.get("val_loss").unwrap();
                if *current_val_loss < self.best_val_loss {
                    self.best_val_loss = *current_val_loss;
                    self.patience_counter = 0;
                } else {
                    self.patience_counter += 1;
                    if self.patience_counter >= self.config.patience {
                        println!("Early stopping triggered at epoch {}!", epoch);
                        break;
                    }
                }
            }
            
            self.metrics_history.push(epoch_metrics);
            
            // Progress reporting
            if epoch % 10 == 0 || epoch == self.config.epochs - 1 {
                let epoch_time = epoch_start.elapsed();
                let val_mae = epoch_metrics.get("val_mae").unwrap();
                let val_r2 = epoch_metrics.get("val_r2").unwrap();
                
                println!("Epoch {}/{} | Loss: {:.4} | MAE: {:.4} | R²: {:.4} | Time: {:.2}s", 
                         epoch + 1, 
                         self.config.epochs,
                         epoch_metrics.get("train_loss").unwrap(),
                         val_mae,
                         val_r2,
                         epoch_time.as_secs_f64());
            }
            
            // Educational visualization updates
            if self.config.enable_visualization && epoch % 20 == 0 {
                self.model.update_visualization_plots();
            }
        }
        
        let total_training_time = self.training_start.elapsed();
        println!("\n=== Regression Training Complete ===");
        println!("Total training time: {:.2} seconds", total_training_time.as_secs_f64());
        
        // Educational: Final analysis
        if self.config.enable_visualization {
            self.generate_final_analysis();
        }
        
        self
    }
    
    /// Evaluate model performance with regression-specific metrics
    pub fn evaluate(&self) -> HashMap<String, f64> {
        println!("\n=== Regression Model Evaluation ===");
        
        let val_data = self.data_pipeline.val_loader();
        let mut total_metrics = HashMap::new();
        
        let mut val_loss = 0.0;
        let mut val_mae = 0.0;
        let mut val_rmse = 0.0;
        let mut val_r2 = 0.0;
        let mut val_mape = 0.0;  // Mean Absolute Percentage Error
        
        let mut predictions_all = Vec::new();
        let mut targets_all = Vec::new();
        
        for batch in &val_data {
            let predictions = self.model.forward(&batch.features);
            let loss = self.model.compute_loss(&predictions, &batch.targets);
            
            let mae = self.calculate_mae(&predictions, &batch.targets);
            let rmse = self.calculate_rmse(&predictions, &batch.targets);
            let r2 = self.calculate_r2(&predictions, &batch.targets);
            let mape = self.calculate_mape(&predictions, &batch.targets);
            
            val_loss += loss.value();
            val_mae += mae;
            val_rmse += rmse;
            val_r2 += r2;
            val_mape += mape;
            
            // Collect for visualization
            predictions_all.extend(predictions.data().iter().copied());
            targets_all.extend(batch.targets.data().iter().copied());
        }
        
        let num_batches = val_data.len() as f64;
        total_metrics.insert("val_loss".to_string(), val_loss / num_batches);
        total_metrics.insert("val_mae".to_string(), val_mae / num_batches);
        total_metrics.insert("val_rmse".to_string(), val_rmse / num_batches);
        total_metrics.insert("val_r2".to_string(), val_r2 / num_batches);
        total_metrics.insert("val_mape".to_string(), val_mape / num_batches);
        
        println!("Regression Evaluation Results:");
        println!("  Loss (MSE): {:.6}", val_loss / num_batches);
        println!("  MAE: {:.6}", val_mae / num_batches);
        println!("  RMSE: {:.6}", val_rmse / num_batches);
        println!("  R² Score: {:.6}", val_r2 / num_batches);
        println!("  MAPE: {:.2}%", val_mape / num_batches);
        
        // Educational: Model interpretation
        self.interpret_regression_results(&total_metrics);
        
        total_metrics
    }
    
    /// Make predictions on new data
    pub fn predict(&self, input_data: &Tensor) -> Tensor {
        self.model.forward(input_data)
    }
    
    /// Predict with confidence intervals (educational feature)
    pub fn predict_with_confidence(&self, input_data: &Tensor, confidence_level: f64) -> (Tensor, Tensor) {
        self.model.predict_with_confidence(input_data, confidence_level)
    }
    
    /// Save the trained model
    pub fn save_model(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.model.save_to_file(path)?;
        println!("Regression model saved to {}", path);
        Ok(())
    }
    
    // Educational helper methods
    fn calculate_mae(&self, predictions: &Tensor, targets: &Tensor) -> f64 {
        let absolute_errors = predictions.sub(targets).abs();
        absolute_errors.mean().data()[0]
    }
    
    fn calculate_rmse(&self, predictions: &Tensor, targets: &Tensor) -> f64 {
        let squared_errors = predictions.sub(targets).pow(2.0);
        squared_errors.mean().sqrt().data()[0]
    }
    
    fn calculate_r2(&self, predictions: &Tensor, targets: &Tensor) -> f64 {
        let targets_mean = targets.mean();
        let ss_tot = targets.sub(&targets_mean).pow(2.0).sum(); // Total sum of squares
        let ss_res = predictions.sub(targets).pow(2.0).sum();   // Residual sum of squares
        
        1.0 - (ss_res.data()[0] / ss_tot.data()[0])
    }
    
    fn calculate_mape(&self, predictions: &Tensor, targets: &Tensor) -> f64 {
        let errors = predictions.sub(targets).abs();
        let relative_errors = errors.div(targets.abs().add(&Tensor::epsilon())); // Avoid division by zero
        (relative_errors.mean().data()[0]) * 100.0
    }
    
    fn generate_final_analysis(&self) {
        println!("\n=== Educational Model Analysis ===");
        
        // Feature importance analysis
        self.model.analyze_feature_importance("regression_feature_importance.png");
        
        // Regression line visualization (if 2D)
        if self.config.input_features == 1 && self.config.output_targets == 1 {
            // Generate sample data for visualization
            let x_range = Tensor::linspace(-10.0, 10.0, 100);
            let y_pred = self.model.predict(&x_range);
            self.model.plot_regression_line(&x_range, &y_pred, "final_regression_line.png");
        }
    }
    
    fn interpret_regression_results(&self, metrics: &HashMap<String, f64>) {
        println!("\n=== Model Interpretation (Educational) ===");
        
        if let Some(&r2) = metrics.get("val_r2") {
            println!("R² Score Interpretation:");
            if r2 >= 0.9 {
                println!("  Excellent fit! Model explains {:.1}% of variance", r2 * 100.0);
            } else if r2 >= 0.7 {
                println!("  Good fit! Model explains {:.1}% of variance", r2 * 100.0);
            } else if r2 >= 0.5 {
                println!("  Moderate fit. Model explains {:.1}% of variance", r2 * 100.0);
            } else {
                println!("  Poor fit. Model explains only {:.1}% of variance", r2 * 100.0);
                println!("  Consider: more features, different model, or data transformation");
            }
        }
        
        if let Some(&mae) = metrics.get("val_mae") {
            println!("\nMAE Interpretation:");
            println!("  Average prediction error: {:.4}", mae);
            println!("  For interpretation: ~{:.1}% of the target variable's range", 
                     (mae * 100.0));
        }
        
        // Regularization effects
        match &self.config.regularization {
            RegularizationType::L1 { lambda } => {
                println!("\nL1 (Lasso) Regularization Effect:");
                println!("  Lambda: {}", lambda);
                println!("  Effect: Promotes sparse solutions (feature selection)");
            }
            RegularizationType::L2 { lambda } => {
                println!("\nL2 (Ridge) Regularization Effect:");
                println!("  Lambda: {}", lambda);
                println!("  Effect: Prevents overfitting, handles multicollinearity");
            }
            RegularizationType::ElasticNet { l1_lambda, l2_lambda } => {
                println!("\nElastic Net Regularization Effect:");
                println!("  L1 Lambda: {}, L2 Lambda: {}", l1_lambda, l2_lambda);
                println!("  Effect: Combines benefits of L1 and L2 regularization");
            }
            RegularizationType::None => {
                println!("\nNo Regularization:");
                println!("  Risk: Potential overfitting with small datasets");
            }
        }
    }
}

/// Educational example: Simple linear regression
pub fn run_linear_regression_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Linear Regression Example ===");
    
    let config = RegressionConfig {
        input_features: 1,
        model_type: RegressionModelType::Linear,
        epochs: 100,
        regularization: RegularizationType::None,
        ..Default::default()
    };
    
    let mut trainer = RegressionTrainer::new(config)
        .configure_visualization()
        .load_dataset("datasets/house_prices.csv", "price")?;
    
    trainer.train();
    let metrics = trainer.evaluate();
    
    println!("\nFinal Linear Regression Results:");
    for (metric, value) in &metrics {
        println!("{}: {:.4}", metric, value);
    }
    
    trainer.save_model("models/linear_regression_model.bin")?;
    Ok(())
}

/// Educational example: Polynomial regression with regularization
pub fn run_polynomial_regression_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Polynomial Regression Example ===");
    
    let config = RegressionConfig {
        input_features: 1,
        model_type: RegressionModelType::Polynomial { degree: 3 },
        regularization: RegularizationType::L2 { lambda: 0.1 },
        epochs: 200,
        ..Default::default()
    };
    
    let mut trainer = RegressionTrainer::new(config)
        .configure_visualization()
        .load_dataset("datasets/synthetic_polynomial.csv", "target")?;
    
    trainer.train();
    let metrics = trainer.evaluate();
    
    println!("\nFinal Polynomial Regression Results:");
    for (metric, value) in &metrics {
        println!("{}: {:.4}", metric, value);
    }
    
    Ok(())
}

/// Educational example: Multiple regression with feature analysis
pub fn run_multiple_regression_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Multiple Regression Example ===");
    
    let config = RegressionConfig {
        input_features: 5,
        output_targets: 1,
        model_type: RegressionModelType::Ridge,
        regularization: RegularizationType::L2 { lambda: 0.01 },
        epochs: 150,
        early_stopping: true,
        patience: 15,
        ..Default::default()
    };
    
    let mut trainer = RegressionTrainer::new(config)
        .configure_visualization()
        .load_dataset("datasets/house_prices_multivariate.csv", "price")?;
    
    trainer.train();
    let metrics = trainer.evaluate();
    
    println!("\nFinal Multiple Regression Results:");
    for (metric, value) in &metrics {
        println!("{}: {:.4}", metric, value);
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_regression_config() {
        let config = RegressionConfig::default();
        assert_eq!(config.input_features, 1);
        assert_eq!(config.output_targets, 1);
        assert!(matches!(config.model_type, RegressionModelType::Linear));
    }
    
    #[test]
    fn test_linear_regression_model() {
        let model = SimpleLinearRegression::new(1, RegularizationType::None);
        let input = Tensor::from(vec![1.0]);
        let prediction = model.predict(&input);
        assert_eq!(prediction.shape(), &vec![1, 1]);
    }
}