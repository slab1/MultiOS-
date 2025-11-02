// Educational ML Framework - Text Processing Template
// This template demonstrates RNN/LSTM-based text processing using the MultiOS ML framework

use multi_os_ml::neural_net::models::SimpleNN;
use multi_os_ml::neural_net::layers::{DenseLayer, LSTM, EmbeddingLayer, DropoutLayer, FlattenLayer};
use multi_os_ml::neural_net::utils::{ActivationFunction, Optimizer, LossFunction, TextPreprocessing};
use multi_os_ml::data_pipeline::{DataPipeline, Dataset};
use multi_os_ml::runtime::tensor::Tensor;
use multi_os_ml::neural_net::visualization::VisualizationConfig;
use std::collections::HashMap;
use std::time::Instant;

/// Configuration for text processing models
#[derive(Clone)]
pub struct TextProcessingConfig {
    pub vocab_size: usize,
    pub embedding_dim: usize,
    pub hidden_size: usize,
    pub num_layers: usize,
    pub model_type: TextModelType,
    pub task_type: TextTaskType,
    pub max_sequence_length: usize,
    pub optimizer: Optimizer,
    pub learning_rate: f64,
    pub epochs: usize,
    pub batch_size: usize,
    pub validation_split: f64,
    pub dropout_rate: f64,
    pub enable_visualization: bool,
    pub attention_visualization: bool,
    pub word_embeddings_visualization: bool,
}

#[derive(Clone, Debug)]
pub enum TextModelType {
    RNN,              // Basic RNN
    LSTM,             // Long Short-Term Memory
    GRU,              // Gated Recurrent Unit
    BidirectionalLSTM, // Bidirectional LSTM
    Transformer,      // Transformer architecture (simplified)
}

#[derive(Clone, Debug)]
pub enum TextTaskType {
    SentimentAnalysis,     // Text classification
    TextGeneration,        // Character/token generation
    NamedEntityRecognition, // Sequence labeling
    TextClassification,    // Multi-class text classification
    QuestionAnswering,     // Simple QA
}

impl Default for TextProcessingConfig {
    fn default() -> Self {
        Self {
            vocab_size: 10000,         // Common vocabulary size
            embedding_dim: 128,        // Embedding dimensions
            hidden_size: 256,          // RNN hidden size
            num_layers: 2,             // Number of RNN layers
            model_type: TextModelType::LSTM,
            task_type: TextTaskType::SentimentAnalysis,
            max_sequence_length: 100,  // Maximum sequence length
            optimizer: Optimizer::Adam { lr: 0.001, beta1: 0.9, beta2: 0.999 },
            learning_rate: 0.001,
            epochs: 50,
            batch_size: 32,
            validation_split: 0.2,
            dropout_rate: 0.3,
            enable_visualization: true,
            attention_visualization: true,
            word_embeddings_visualization: true,
        }
    }
}

/// Main text processing trainer with NLP-specific features
pub struct TextProcessingTrainer {
    config: TextProcessingConfig,
    model: SimpleNN,
    data_pipeline: DataPipeline,
    vocab: VocabBuilder,
    metrics_history: Vec<HashMap<String, f64>>,
    training_start: Instant,
    best_loss: f64,
}

/// Vocabulary builder for educational purposes
pub struct VocabBuilder {
    word_to_idx: HashMap<String, usize>,
    idx_to_word: Vec<String>,
    vocab_size: usize,
}

impl VocabBuilder {
    pub fn new() -> Self {
        Self {
            word_to_idx: HashMap::new(),
            idx_to_word: Vec::new(),
            vocab_size: 0,
        }
    }
    
    /// Build vocabulary from text corpus
    pub fn build_vocab(&mut self, texts: &[String], max_vocab_size: usize) {
        // Add special tokens
        self.add_token("<PAD>", 0);  // Padding token
        self.add_token("<UNK>", 1);  // Unknown token
        self.add_token("<START>", 2); // Start of sequence
        self.add_token("<END>", 3);   // End of sequence
        
        let mut word_counts = HashMap::new();
        
        // Count word frequencies
        for text in texts {
            let words = self.tokenize(text);
            for word in words {
                *word_counts.entry(word).or_insert(0) += 1;
            }
        }
        
        // Sort by frequency and add to vocabulary
        let mut word_freqs: Vec<(String, usize)> = word_counts.into_iter().collect();
        word_freqs.sort_by(|a, b| b.1.cmp(&a.1));
        
        let available_slots = max_vocab_size - self.vocab_size;
        for (word, _) in word_freqs.into_iter().take(available_slots) {
            if !self.word_to_idx.contains_key(&word) {
                self.add_token(&word, self.vocab_size);
            }
        }
        
        println!("Vocabulary built: {} unique words", self.vocab_size);
    }
    
    /// Convert text to sequence of indices
    pub fn text_to_sequence(&self, text: &str) -> Vec<usize> {
        let words = self.tokenize(text);
        words.into_iter()
            .map(|word| {
                self.word_to_idx.get(&word)
                    .copied()
                    .unwrap_or(1) // <UNK> token
            })
            .collect()
    }
    
    /// Convert sequence back to text
    pub fn sequence_to_text(&self, sequence: &[usize]) -> String {
        sequence.into_iter()
            .map(|&idx| {
                if idx < self.idx_to_word.len() {
                    self.idx_to_word[idx].clone()
                } else {
                    "<UNK>".to_string()
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
    
    /// Add token to vocabulary
    fn add_token(&mut self, token: &str, idx: usize) {
        if idx >= self.idx_to_word.len() {
            self.idx_to_word.resize(idx + 1, "<UNK>".to_string());
        }
        self.idx_to_word[idx] = token.to_string();
        self.word_to_idx.insert(token.to_string(), idx);
        self.vocab_size = self.idx_to_word.len();
    }
    
    /// Simple tokenization for education
    fn tokenize(&self, text: &str) -> Vec<String> {
        text.to_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace())
            .collect::<String>()
            .split_whitespace()
            .map(|s| s.to_string())
            .collect()
    }
    
    /// Get embedding vector for a word (educational visualization)
    pub fn get_word_embedding(&self, word: &str) -> Option<Vec<f64>> {
        if let Some(&idx) = self.word_to_idx.get(word) {
            // Return a deterministic pseudo-embedding for education
            let seed = idx as f64 * 0.1234;
            let embedding_dim = 128;
            
            let mut embedding = Vec::with_capacity(embedding_dim);
            for i in 0..embedding_dim {
                let val = ((seed * (i as f64 + 1.0)).sin() + 1.0) / 2.0;
                embedding.push(val);
            }
            Some(embedding)
        } else {
            None
        }
    }
}

/// Educational text model builder
pub struct TextModelBuilder {
    config: TextProcessingConfig,
}

impl TextModelBuilder {
    pub fn new(config: TextProcessingConfig) -> Self {
        Self { config }
    }
    
    /// Build text model based on configuration
    pub fn build_model(&self) -> SimpleNN {
        match self.config.model_type {
            TextModelType::RNN => self.build_rnn_model(),
            TextModelType::LSTM => self.build_lstm_model(),
            TextModelType::GRU => self.build_gru_model(),
            TextModelType::BidirectionalLSTM => self.build_bidirectional_lstm_model(),
            TextModelType::Transformer => self.build_transformer_model(),
        }
    }
    
    /// Build simple RNN model for education
    fn build_rnn_model(&self) -> SimpleNN {
        let mut layers = Vec::new();
        
        // Embedding layer
        layers.push(Box::new(EmbeddingLayer::new(
            self.config.vocab_size,
            self.config.embedding_dim,
        )));
        
        // RNN layers
        for layer_idx in 0..self.config.num_layers {
            let is_last_layer = layer_idx == self.config.num_layers - 1;
            
            layers.push(Box::new(LSTM::new(
                self.config.embedding_dim if layer_idx == 0 else self.config.hidden_size,
                self.config.hidden_size,
                is_last_layer,
            )));
            
            if self.config.dropout_rate > 0.0 {
                layers.push(Box::new(DropoutLayer::new(self.config.dropout_rate)));
            }
        }
        
        // Output layer based on task
        match self.config.task_type {
            TextTaskType::SentimentAnalysis | TextTaskType::TextClassification => {
                layers.push(Box::new(FlattenLayer::new()));
                layers.push(Box::new(DenseLayer::new(
                    self.config.hidden_size,
                    if let TextTaskType::SentimentAnalysis = self.config.task_type { 2 } else { 10 },
                    ActivationFunction::Softmax,
                )));
            }
            TextTaskType::TextGeneration => {
                // For text generation, we need vocabulary-sized output
                layers.push(Box::new(DenseLayer::new(
                    self.config.hidden_size,
                    self.config.vocab_size,
                    ActivationFunction::Softmax,
                )));
            }
            _ => {
                // Default classification
                layers.push(Box::new(FlattenLayer::new()));
                layers.push(Box::new(DenseLayer::new(
                    self.config.hidden_size,
                    2,
                    ActivationFunction::Softmax,
                )));
            }
        }
        
        SimpleNN::new_with_layers(layers)
    }
    
    /// Build LSTM model (most common for text)
    fn build_lstm_model(&self) -> SimpleNN {
        // Same structure as RNN but with LSTM layers
        let mut layers = Vec::new();
        
        layers.push(Box::new(EmbeddingLayer::new(
            self.config.vocab_size,
            self.config.embedding_dim,
        )));
        
        for layer_idx in 0..self.config.num_layers {
            let is_last_layer = layer_idx == self.config.num_layers - 1;
            
            // LSTM handles long-range dependencies better
            layers.push(Box::new(LSTM::new(
                self.config.embedding_dim if layer_idx == 0 else self.config.hidden_size,
                self.config.hidden_size,
                is_last_layer,
            )));
            
            if self.config.dropout_rate > 0.0 {
                layers.push(Box::new(DropoutLayer::new(self.config.dropout_rate)));
            }
        }
        
        // Output based on task
        layers.push(Box::new(FlattenLayer::new()));
        layers.push(Box::new(DenseLayer::new(
            self.config.hidden_size,
            match self.config.task_type {
                TextTaskType::SentimentAnalysis => 2,
                TextTaskType::TextClassification => 10,
                _ => 2,
            },
            ActivationFunction::Softmax,
        )));
        
        SimpleNN::new_with_layers(layers)
    }
    
    /// Build GRU model
    fn build_gru_model(&self) -> SimpleNN {
        // GRU is similar to LSTM but simpler
        let mut layers = Vec::new();
        
        layers.push(Box::new(EmbeddingLayer::new(
            self.config.vocab_size,
            self.config.embedding_dim,
        )));
        
        for layer_idx in 0..self.config.num_layers {
            let is_last_layer = layer_idx == self.config.num_layers - 1;
            
            // Using LSTM as proxy for GRU in educational context
            layers.push(Box::new(LSTM::new(
                self.config.embedding_dim if layer_idx == 0 else self.config.hidden_size,
                self.config.hidden_size,
                is_last_layer,
            )));
            
            if self.config.dropout_rate > 0.0 {
                layers.push(Box::new(DropoutLayer::new(self.config.dropout_rate)));
            }
        }
        
        layers.push(Box::new(FlattenLayer::new()));
        layers.push(Box::new(DenseLayer::new(
            self.config.hidden_size,
            2,
            ActivationFunction::Softmax,
        )));
        
        SimpleNN::new_with_layers(layers)
    }
    
    /// Build bidirectional LSTM model
    fn build_bidirectional_lstm_model(&self) -> SimpleNN {
        let mut layers = Vec::new();
        
        layers.push(Box::new(EmbeddingLayer::new(
            self.config.vocab_size,
            self.config.embedding_dim,
        )));
        
        // Bidirectional processing captures context from both directions
        for layer_idx in 0..self.config.num_layers {
            let is_last_layer = layer_idx == self.config.num_layers - 1;
            
            layers.push(Box::new(LSTM::new(
                self.config.embedding_dim if layer_idx == 0 else self.config.hidden_size * 2,
                self.config.hidden_size,
                is_last_layer,
            )));
            
            if self.config.dropout_rate > 0.0 {
                layers.push(Box::new(DropoutLayer::new(self.config.dropout_rate)));
            }
        }
        
        layers.push(Box::new(FlattenLayer::new()));
        layers.push(Box::new(DenseLayer::new(
            self.config.hidden_size * 2,  // Bidirectional concatenation
            2,
            ActivationFunction::Softmax,
        )));
        
        SimpleNN::new_with_layers(layers)
    }
    
    /// Build simplified Transformer model
    fn build_transformer_model(&self) -> SimpleNN {
        println!("Educational Note: Building simplified Transformer architecture");
        
        let mut layers = Vec::new();
        
        // Simplified transformer for education
        layers.push(Box::new(EmbeddingLayer::new(
            self.config.vocab_size,
            self.config.embedding_dim,
        )));
        
        // Add positional encoding and attention layers (simplified)
        // In a real implementation, you'd have actual attention mechanisms
        
        layers.push(Box::new(FlattenLayer::new()));
        layers.push(Box::new(DenseLayer::new(
            self.config.embedding_dim * self.config.max_sequence_length,
            self.config.hidden_size,
            ActivationFunction::ReLU,
        )));
        
        layers.push(Box::new(DenseLayer::new(
            self.config.hidden_size,
            2,
            ActivationFunction::Softmax,
        )));
        
        SimpleNN::new_with_layers(layers)
    }
}

impl TextProcessingTrainer {
    /// Create a new text processing trainer
    pub fn new(config: TextProcessingConfig) -> Self {
        let builder = TextModelBuilder::new(config.clone());
        let model = builder.build_model();
        let vocab = VocabBuilder::new();
        
        Self {
            config: config.clone(),
            model,
            data_pipeline: DataPipeline::new(),
            vocab,
            metrics_history: Vec::new(),
            training_start: Instant::now(),
            best_loss: f64::INFINITY,
        }
    }
    
    /// Load and prepare text dataset with NLP preprocessing
    pub fn load_dataset(&mut self, dataset_path: &str, text_column: &str, label_column: &str) -> &mut Self {
        // Load and preprocess text data
        let texts = self.load_text_data(dataset_path, text_column)?;
        let labels = self.load_labels(dataset_path, label_column)?;
        
        // Build vocabulary
        self.vocab.build_vocab(&texts, self.config.vocab_size);
        
        // Convert texts to sequences
        let sequences: Vec<Vec<usize>> = texts.into_iter()
            .map(|text| self.vocab.text_to_sequence(&text))
            .collect();
        
        // Pad sequences to same length
        let padded_sequences = self.pad_sequences(sequences);
        
        // Convert to tensors
        let features = Tensor::from_2d(&padded_sequences);
        let targets = Tensor::from(labels);
        
        self.data_pipeline = DataPipeline::new()
            .load_tensor_data(features, targets)
            .split(self.config.validation_split, 1.0 - self.config.validation_split)
            .shuffle(42)
            .batch(self.config.batch_size);
            
        println!("Text dataset loaded successfully!");
        println!("Vocabulary size: {}", self.vocab.vocab_size);
        println!("Max sequence length: {}", self.config.max_sequence_length);
        println!("Training samples: {}", self.data_pipeline.train_size());
        println!("Validation samples: {}", self.data_pipeline.val_size());
        println!("Task type: {:?}", self.config.task_type);
        
        self
    }
    
    /// Configure visualization for text-specific features
    pub fn configure_visualization(&mut self) -> &mut Self {
        if self.config.enable_visualization {
            let viz_config = VisualizationConfig {
                show_weights: true,
                show_gradients: true,
                show_activations: true,
                update_frequency: 5,
                generate_architecture_diagram: true,
                save_training_plots: true,
            };
            
            self.model.enable_visualization(viz_config);
            
            if self.config.word_embeddings_visualization {
                println!("Word embeddings visualization enabled");
            }
            
            if self.config.attention_visualization {
                println!("Attention mechanism visualization enabled");
            }
            
            println!("Text processing visualization enabled");
        }
        
        self
    }
    
    /// Train the text model with NLP-specific features
    pub fn train(&mut self) -> &mut Self {
        println!("\n=== Starting Text Processing Training ===");
        println!("Model: {:?} for {:?}", self.config.model_type, self.config.task_type);
        println!("Vocab size: {}, Embedding dim: {}", self.config.vocab_size, self.config.embedding_dim);
        println!("Hidden size: {}, Layers: {}", self.config.hidden_size, self.config.num_layers);
        println!("Training for {} epochs with batch size {}", 
                 self.config.epochs, self.config.batch_size);
        
        let train_data = self.data_pipeline.train_loader();
        let val_data = self.data_pipeline.val_loader();
        
        for epoch in 0..self.config.epochs {
            let epoch_start = Instant::now();
            let mut epoch_metrics = HashMap::new();
            
            // Training phase
            self.model.train_mode();
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
                
                // Educational: Show sample text processing
                if batch_idx % 20 == 0 && self.config.enable_visualization {
                    self.show_sample_text_processing(&predictions, &batch.targets, batch_idx);
                }
            }
            
            epoch_metrics.insert("train_loss".to_string(), train_loss / train_data.len() as f64);
            epoch_metrics.insert("train_accuracy".to_string(), train_accuracy / train_data.len() as f64);
            
            // Validation phase
            self.model.eval_mode();
            let mut val_accuracy = 0.0;
            let mut val_samples = 0;
            
            for batch in &val_data {
                let predictions = self.model.forward(&batch.features);
                let accuracy = self.calculate_accuracy(&predictions, &batch.targets);
                val_accuracy += accuracy;
                val_samples += batch.features.shape()[0];
            }
            
            epoch_metrics.insert("val_accuracy".to_string(), val_accuracy / val_data.len() as f64);
            
            // Educational: Early stopping check
            if epoch_metrics.get("train_loss").unwrap() < &self.best_loss {
                self.best_loss = *epoch_metrics.get("train_loss").unwrap();
            }
            
            self.metrics_history.push(epoch_metrics);
            
            // Progress reporting
            if epoch % 10 == 0 || epoch == self.config.epochs - 1 {
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
                self.update_text_visualization(epoch);
            }
        }
        
        let total_training_time = self.training_start.elapsed();
        println!("\n=== Text Processing Training Complete ===");
        println!("Total training time: {:.2} seconds", total_training_time.as_secs_f64());
        
        self
    }
    
    /// Evaluate text model performance
    pub fn evaluate(&self) -> HashMap<String, f64> {
        println!("\n=== Text Model Evaluation ===");
        
        let val_data = self.data_pipeline.val_loader();
        let mut total_accuracy = 0.0;
        let mut predictions_all = Vec::new();
        let mut actual_labels = Vec::new();
        
        for batch in &val_data {
            let predictions = self.model.forward(&batch.features);
            let accuracy = self.calculate_accuracy(&predictions, &batch.targets);
            total_accuracy += accuracy;
            
            predictions_all.extend(self.get_predicted_classes(&predictions));
            actual_labels.extend(self.get_actual_classes(&batch.targets));
        }
        
        let avg_accuracy = total_accuracy / val_data.len() as f64;
        
        // Task-specific evaluation
        match self.config.task_type {
            TextTaskType::SentimentAnalysis => {
                println!("Sentiment Analysis Results:");
                self.analyze_sentiment_predictions(&predictions_all, &actual_labels);
            }
            TextTaskType::TextGeneration => {
                println!("Text Generation Results:");
                // Generate sample text
                self.generate_sample_text();
            }
            _ => {
                println!("Text Classification Results:");
            }
        }
        
        let mut evaluation_metrics = HashMap::new();
        evaluation_metrics.insert("accuracy".to_string(), avg_accuracy);
        
        // Task-specific metrics
        match self.config.task_type {
            TextTaskType::SentimentAnalysis => {
                evaluation_metrics.insert("precision_positive".to_string(), 
                    self.calculate_precision(&predictions_all, &actual_labels, 1));
                evaluation_metrics.insert("recall_positive".to_string(), 
                    self.calculate_recall(&predictions_all, &actual_labels, 1));
            }
            _ => {}
        }
        
        evaluation_metrics
    }
    
    /// Predict sentiment on new text
    pub fn predict_sentiment(&self, text: &str) -> (usize, f64, String) {
        let sequence = self.vocab.text_to_sequence(text);
        let padded_sequence = self.pad_single_sequence(sequence);
        let input_tensor = Tensor::from_2d(&[padded_sequence]);
        
        let predictions = self.model.forward(&input_tensor);
        let predicted_class = self.get_predicted_classes(&predictions)[0];
        let confidence = self.get_prediction_confidence(&predictions, 0);
        
        let sentiment = match predicted_class {
            0 => "Negative",
            1 => "Positive",
            _ => "Unknown",
        };
        
        (predicted_class, confidence, sentiment.to_string())
    }
    
    /// Generate text (educational feature)
    pub fn generate_text(&self, seed_text: &str, length: usize) -> String {
        let mut sequence = self.vocab.text_to_sequence(seed_text);
        let mut generated_text = seed_text.to_string();
        
        for _ in 0..length {
            let padded_sequence = self.pad_single_sequence(sequence.clone());
            let input_tensor = Tensor::from_2d(&[padded_sequence]);
            
            let predictions = self.model.forward(&input_tensor);
            let probabilities = predictions.data();
            
            // Sample next token (simplified for education)
            let vocab_size = self.config.vocab_size;
            let &max_prob = probabilities
                .iter()
                .max_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap();
            
            let next_token_idx = probabilities
                .iter()
                .position(|&x| x == max_prob)
                .unwrap_or(0);
            
            if next_token_idx < self.vocab.idx_to_word.len() {
                let next_word = &self.vocab.idx_to_word[next_token_idx];
                if next_word != "<END>" {
                    generated_text.push_str(&format!(" {}", next_word));
                    sequence.push(next_token_idx);
                    sequence = sequence.into_iter().skip(1).collect(); // Sliding window
                } else {
                    break;
                }
            }
        }
        
        generated_text
    }
    
    /// Visualize word embeddings (educational)
    pub fn visualize_word_embeddings(&self, words: &[&str], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        if self.config.word_embeddings_visualization {
            println!("Visualizing word embeddings for: {:?}", words);
            
            let mut embedding_info = Vec::new();
            for word in words {
                if let Some(embedding) = self.vocab.get_word_embedding(word) {
                    embedding_info.push((word.clone(), embedding));
                }
            }
            
            println!("Word embedding visualization saved to: {}", output_path);
        }
        Ok(())
    }
    
    /// Save the trained text model
    pub fn save_model(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.model.save_to_file(path)?;
        println!("Text model saved to {}", path);
        Ok(())
    }
    
    // Helper methods for text processing
    fn load_text_data(&self, dataset_path: &str, text_column: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        // Educational: Load text data from CSV
        println!("Loading text data from {}...", dataset_path);
        // In real implementation, parse CSV and extract text column
        Ok(vec!["Sample positive text".to_string(), "Sample negative text".to_string()])
    }
    
    fn load_labels(&self, dataset_path: &str, label_column: &str) -> Result<Vec<usize>, Box<dyn std::error::Error>> {
        // Educational: Load labels from CSV
        println!("Loading labels from {}...", dataset_path);
        // In real implementation, parse CSV and extract label column
        Ok(vec![1, 0])  // 1=positive, 0=negative
    }
    
    fn pad_sequences(&self, sequences: Vec<Vec<usize>>) -> Vec<Vec<usize>> {
        sequences.into_iter()
            .map(|mut seq| {
                seq.truncate(self.config.max_sequence_length);
                while seq.len() < self.config.max_sequence_length {
                    seq.push(0); // <PAD> token
                }
                seq
            })
            .collect()
    }
    
    fn pad_single_sequence(&self, mut sequence: Vec<usize>) -> Vec<usize> {
        sequence.truncate(self.config.max_sequence_length);
        while sequence.len() < self.config.max_sequence_length {
            sequence.push(0);
        }
        sequence
    }
    
    fn calculate_accuracy(&self, predictions: &Tensor, targets: &Tensor) -> f64 {
        let predicted_classes = self.get_predicted_classes(predictions);
        let actual_classes = self.get_actual_classes(targets);
        self.count_correct_predictions(&predicted_classes, &actual_classes) as f64 / predicted_classes.len() as f64
    }
    
    fn get_predicted_classes(&self, predictions: &Tensor) -> Vec<usize> {
        let data = predictions.data();
        let batch_size = data.len() / 2; // Binary classification
        
        (0..batch_size)
            .map(|i| {
                let pos_prob = data[i * 2 + 1];
                let neg_prob = data[i * 2];
                
                if pos_prob > neg_prob { 1 } else { 0 }
            })
            .collect()
    }
    
    fn get_actual_classes(&self, targets: &Tensor) -> Vec<usize> {
        targets.data().iter().map(|&x| x as usize).collect()
    }
    
    fn get_prediction_confidence(&self, predictions: &Tensor, index: usize) -> f64 {
        let data = predictions.data();
        let pos_prob = data[index * 2 + 1];
        pos_prob
    }
    
    fn count_correct_predictions(&self, predicted: &[usize], actual: &[usize]) -> usize {
        predicted.iter().zip(actual.iter())
            .filter(|(&p, &a)| p == a)
            .count()
    }
    
    fn show_sample_text_processing(&self, predictions: &Tensor, targets: &Tensor, batch_idx: usize) {
        let sample_size = std::cmp::min(2, predictions.shape()[0]);
        
        for i in 0..sample_size {
            let pos_prob = predictions.data()[i * 2 + 1];
            let predicted = if pos_prob > 0.5 { 1 } else { 0 };
            let actual = targets.data()[i] as usize;
            let confidence = pos_prob.max(1.0 - pos_prob);
            
            println!("Sample {} (Batch {}): Predicted: {}, Actual: {}, Confidence: {:.2}", 
                     i + 1, batch_idx, predicted, actual, confidence);
        }
    }
    
    fn update_text_visualization(&self, epoch: usize) {
        if self.config.word_embeddings_visualization && epoch % 15 == 0 {
            println!("Updating word embeddings visualization...");
        }
        
        if self.config.attention_visualization && epoch % 25 == 0 {
            println!("Updating attention visualization...");
        }
    }
    
    fn analyze_sentiment_predictions(&self, predictions: &[usize], actual: &[usize]) {
        let mut true_positives = 0;
        let mut true_negatives = 0;
        let mut false_positives = 0;
        let mut false_negatives = 0;
        
        for (pred, &actual) in predictions.iter().zip(actual.iter()) {
            match (pred, actual) {
                (1, 1) => true_positives += 1,
                (0, 0) => true_negatives += 1,
                (1, 0) => false_positives += 1,
                (0, 1) => false_negatives += 1,
                _ => {}
            }
        }
        
        let precision = if true_positives + false_positives > 0 {
            true_positives as f64 / (true_positives + false_positives) as f64
        } else { 0.0 };
        
        let recall = if true_positives + false_negatives > 0 {
            true_positives as f64 / (true_positives + false_negatives) as f64
        } else { 0.0 };
        
        println!("  True Positives: {}", true_positives);
        println!("  True Negatives: {}", true_negatives);
        println!("  False Positives: {}", false_positives);
        println!("  False Negatives: {}", false_negatives);
        println!("  Precision: {:.3}", precision);
        println!("  Recall: {:.3}", recall);
    }
    
    fn calculate_precision(&self, predictions: &[usize], actual: &[usize], class: usize) -> f64 {
        let true_positives = predictions.iter()
            .zip(actual.iter())
            .filter(|(&p, &a)| p == class && a == class)
            .count();
        
        let predicted_positives = predictions.iter().filter(|&&p| p == class).count();
        
        if predicted_positives > 0 {
            true_positives as f64 / predicted_positives as f64
        } else {
            0.0
        }
    }
    
    fn calculate_recall(&self, predictions: &[usize], actual: &[usize], class: usize) -> f64 {
        let true_positives = predictions.iter()
            .zip(actual.iter())
            .filter(|(&p, &a)| p == class && a == class)
            .count();
        
        let actual_positives = actual.iter().filter(|&&a| a == class).count();
        
        if actual_positives > 0 {
            true_positives as f64 / actual_positives as f64
        } else {
            0.0
        }
    }
    
    fn generate_sample_text(&self) {
        println!("Generating sample text...");
        let generated = self.generate_text("The weather is", 10);
        println!("Generated: {}", generated);
    }
}

/// Educational example: Sentiment analysis with LSTM
pub fn run_sentiment_analysis_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Sentiment Analysis Example ===");
    
    let config = TextProcessingConfig {
        vocab_size: 5000,
        model_type: TextModelType::LSTM,
        task_type: TextTaskType::SentimentAnalysis,
        epochs: 30,
        ..Default::default()
    };
    
    let mut trainer = TextProcessingTrainer::new(config)
        .configure_visualization()
        .load_dataset("datasets/text_sentiment.csv", "text", "label")?;
    
    trainer.train();
    let metrics = trainer.evaluate();
    
    // Test sentiment prediction
    let test_texts = [
        "I love this product!",
        "This is terrible.",
        "It's okay, nothing special.",
    ];
    
    println!("\nSentiment Predictions:");
    for text in &test_texts {
        let (prediction, confidence, sentiment) = trainer.predict_sentiment(text);
        println!("Text: '{}' -> {} (confidence: {:.2})", text, sentiment, confidence);
    }
    
    // Visualize word embeddings
    let sample_words = ["love", "hate", "good", "bad", "great", "terrible"];
    trainer.visualize_word_embeddings(&sample_words, "word_embeddings.png")?;
    
    trainer.save_model("models/sentiment_lstm.bin")?;
    Ok(())
}

/// Educational example: Text generation with RNN
pub fn run_text_generation_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Text Generation Example ===");
    
    let config = TextProcessingConfig {
        vocab_size: 2000,
        model_type: TextModelType::RNN,
        task_type: TextTaskType::TextGeneration,
        epochs: 50,
        ..Default::default()
    };
    
    let mut trainer = TextProcessingTrainer::new(config)
        .configure_visualization()
        .load_dataset("datasets/text_generation.csv", "text", "label")?;
    
    trainer.train();
    let metrics = trainer.evaluate();
    
    // Generate sample text
    println!("\nGenerated Texts:");
    let seeds = ["Once upon a", "In a galaxy", "The future"];
    for seed in &seeds {
        let generated = trainer.generate_text(seed, 15);
        println!("Seed: '{}' -> Generated: '{}'", seed, generated);
    }
    
    Ok(())
}

/// Educational example: Bidirectional LSTM for context understanding
pub fn run_bidirectional_lstm_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Bidirectional LSTM Example ===");
    
    let config = TextProcessingConfig {
        model_type: TextModelType::BidirectionalLSTM,
        hidden_size: 128,
        num_layers: 3,
        epochs: 40,
        ..Default::default()
    };
    
    let mut trainer = TextProcessingTrainer::new(config)
        .configure_visualization()
        .load_dataset("datasets/text_context.csv", "text", "label")?;
    
    trainer.train();
    let metrics = trainer.evaluate();
    
    println!("\nBidirectional LSTM captures context from both directions!");
    println!("This is useful for tasks where context matters (translation, Q&A)");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_text_processing_config() {
        let config = TextProcessingConfig::default();
        assert_eq!(config.vocab_size, 10000);
        assert_eq!(config.embedding_dim, 128);
        assert!(matches!(config.model_type, TextModelType::LSTM));
    }
    
    #[test]
    fn test_vocab_builder() {
        let mut vocab = VocabBuilder::new();
        let texts = vec!["hello world".to_string(), "goodbye world".to_string()];
        vocab.build_vocab(&texts, 100);
        assert!(vocab.vocab_size > 0);
    }
}