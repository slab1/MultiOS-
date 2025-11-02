# Educational ML Framework Examples

This directory contains comprehensive examples, templates, and tutorials for the MultiOS ML framework designed for computer science education.

## Directory Structure

```
examples/
├── README.md                   # This file - main documentation
├── templates/                  # ML workflow templates
│   ├── basic_classification.rs     # Binary/multiclass classification
│   ├── regression.rs               # Linear/logistic regression
│   ├── image_recognition.rs        # CNN for image classification
│   ├── text_processing.rs          # RNN/LSTM for text analysis
│   ├── clustering.rs               # Unsupervised learning
│   └── transfer_learning.rs        # Pre-trained model adaptation
├── tutorials/                  # Step-by-step tutorials
│   ├── 01_basic_concepts.rs        # Introduction to tensors and operations
│   ├── 02_first_network.rs         # Building your first neural network
│   ├── 03_visualization.rs         # Using the visual debugger
│   ├── 04_parallel_training.rs     # Distributed training setup
│   ├── 05_performance_optimization.rs # Profiling and optimization
│   └── 06_production_deployment.rs  # Model deployment basics
├── datasets/                   # Educational datasets and loaders
│   ├── synthetic.rs                # Generated datasets for learning
│   ├── iris_classification.rs      # Classic iris dataset
│   ├── house_prices.rs             # Regression dataset
│   ├── mnist_subset.rs             # Simplified MNIST for education
│   └── text_sentiment.rs           # Text classification dataset
├── integration/                # MultiOS integration examples
│   ├── performance_monitoring.rs   # Using MultiOS performance tools
│   ├── scheduling_integration.rs   # MultiOS job scheduling
│   ├── resource_management.rs      # Memory and CPU optimization
│   └── monitoring_dashboard.rs     # Real-time monitoring setup
└── quickstart.rs               # Quick start template for new users
```

## Quick Start

For students new to ML, start with:

1. **Quick Start**: Run `quickstart.rs` for a simple example
2. **Basic Concepts**: Follow `tutorials/01_basic_concepts.rs`
3. **First Network**: Build your first model with `tutorials/02_first_network.rs`

## Template Categories

### 1. Classification Templates (`templates/`)
- **Basic Classification**: Binary and multiclass problems
- **Image Recognition**: Convolutional neural networks
- **Text Processing**: Natural language processing tasks
- **Transfer Learning**: Adapting pre-trained models

### 2. Tutorials (`tutorials/`)
Step-by-step guides covering:
- Tensor operations and basic ML concepts
- Building and training neural networks
- Visual debugging and model interpretation
- Parallel training and distributed systems
- Performance optimization techniques
- Production deployment patterns

### 3. Datasets (`datasets/`)
Educational datasets with:
- Synthetic data generators for concept learning
- Classic datasets (Iris, MNIST subset)
- Regression problems (house prices)
- Text analysis datasets (sentiment classification)
- Data loading and preprocessing examples

### 4. Integration (`integration/`)
MultiOS-specific features:
- Performance monitoring integration
- Resource management and optimization
- Scheduling and parallel execution
- Real-time monitoring dashboards

## Running Examples

### Basic Template
```rust
use multi_os_ml::neural_net::models::SimpleNN;
use multi_os_ml::data_pipeline::DataPipeline;

fn main() {
    // Load your dataset
    let pipeline = DataPipeline::new()
        .load_data("path/to/data.csv")
        .normalize()
        .batch(32);
    
    // Build model
    let mut model = SimpleNN::new(vec![784, 128, 64, 10]);
    
    // Train with visualization
    model.train_with_visualization(&pipeline, 10);
    
    // Evaluate
    let accuracy = model.evaluate(&pipeline.test_data);
    println!("Model accuracy: {:.2}%", accuracy * 100.0);
}
```

### Parallel Training Example
```rust
use multi_os_ml::parallel_training::ParallelTrainer;

fn main() {
    let trainer = ParallelTrainer::new()
        .num_workers(4)                    // Use MultiOS cores
        .batch_size(128)
        .sync_frequency(10);               // Gradient sync frequency
    
    let model = trainer.train_distributed(
        "templates/basic_classification.rs",
        "datasets/mnist_subset.rs",
    );
    
    println!("Training completed on {} workers", trainer.num_workers());
}
```

### Performance Monitoring
```rust
use multi_os_ml::runtime::performance::PerformanceMonitor;
use multios_performance::Profiler;

fn main() {
    let perf_monitor = PerformanceMonitor::new();
    let profiler = Profiler::new();
    
    // Train with full performance monitoring
    let model = perf_monitor.monitor_training(|| {
        SimpleNN::new(vec![784, 128, 10]).train(epochs)
    });
    
    // Generate performance report
    let report = perf_monitor.generate_report();
    println!("Performance Report: {}", report);
}
```

## Educational Features

### Visual Debugging
Every template includes visualization options:
```rust
// Enable real-time visualization
model.enable_visualization(VisualizationConfig {
    show_weights: true,
    show_gradients: true,
    show_activations: true,
    update_frequency: 5,  // Every 5 epochs
});

// Generate network architecture diagram
model.generate_architecture_svg("network.svg");
```

### Interactive Exploration
```rust
use multi_os_ml::interactive::ModelBrowser;

// Start interactive model browser
let browser = ModelBrowser::new();
browser.serve_model(&model, 8080);  // Web interface on port 8080
```

### Performance Profiling
```rust
use multios_performance::{Profiler, ResourceMonitor};

// Monitor memory usage during training
let memory_monitor = ResourceMonitor::new();
let peak_memory = memory_monitor.track_training(|| {
    model.train(dataset, epochs)
});

println!("Peak memory usage: {:.2} MB", peak_memory);
```

## Learning Path Recommendations

### Beginner Track
1. `tutorials/01_basic_concepts.rs` - Learn tensors and basic operations
2. `templates/basic_classification.rs` - Simple classification task
3. `datasets/synthetic.rs` - Generate and work with synthetic data
4. `tutorials/03_visualization.rs` - Understanding model internals

### Intermediate Track
1. `templates/image_recognition.rs` - CNN for image classification
2. `tutorials/04_parallel_training.rs` - Distributed training
3. `integration/performance_monitoring.rs` - Performance optimization
4. `tutorials/05_performance_optimization.rs` - Advanced optimization

### Advanced Track
1. `templates/transfer_learning.rs` - Using pre-trained models
2. `tutorials/06_production_deployment.rs` - Model deployment
3. `integration/scheduling_integration.rs` - MultiOS job scheduling
4. Custom model development with full feature utilization

## Common Patterns

### Data Loading Pattern
```rust
use multi_os_ml::data_pipeline::{DataPipeline, Dataset};

let dataset = IrisClassification::load("data/iris.csv")
    .expect("Failed to load dataset");

let pipeline = DataPipeline::new()
    .load_dataset(Box::new(dataset))
    .shuffle(42)                    // Reproducible shuffling
    .split(0.8, 0.2)               // Train/validation split
    .normalize_features()
    .encode_labels();               // One-hot encoding for classification

let (train_data, test_data) = pipeline.build_data_loaders(32);
```

### Training Loop Pattern
```rust
let mut model = NeuralNetwork::new(config);
let mut trainer = Trainer::new()
    .optimizer(Optimizer::Adam { lr: 0.001 })
    .loss_function(LossFunction::CrossEntropy)
    .metrics(vec![Accuracy, Precision, Recall]);

for epoch in 0..epochs {
    let epoch_start = Instant::now();
    
    for batch in &train_data {
        let predictions = model.forward(&batch.features);
        let loss = trainer.compute_loss(&predictions, &batch.labels);
        
        model.backward(&loss);
        model.update_parameters();
    }
    
    // Validation and logging
    if epoch % 10 == 0 {
        let val_accuracy = model.evaluate(&test_data);
        println!("Epoch {}: {:.4} accuracy in {:.2}s", 
                 epoch, val_accuracy, epoch_start.elapsed().as_secs_f64());
    }
}
```

### Evaluation Pattern
```rust
let evaluation_results = model.evaluate_comprehensive(&test_data, vec![
    Accuracy,
    Precision { average: Macro },
    Recall { average: Macro },
    F1Score { average: Macro },
    ConfusionMatrix,
    ClassificationReport,
]);

println!("{}", evaluation_results.generate_report());
```

## Troubleshooting

### Common Issues
1. **Out of Memory**: Reduce batch size or use gradient accumulation
2. **Slow Training**: Enable parallel training or optimize data pipeline
3. **Poor Performance**: Check data normalization and learning rate
4. **Visualization Issues**: Ensure all visualization dependencies are installed

### Debug Mode
```rust
let mut model = NeuralNetwork::new(config);
model.set_debug_mode(true);

// Enable detailed logging
model.set_verbosity(Verbose);

// Set breakpoints for debugging
model.add_breakpoint("epoch_5", |model| {
    println!("Model state at epoch 5:");
    model.print_weights_summary();
    model.plot_weights_heatmap("weights_epoch_5.png");
});
```

## Support and Resources

- **Documentation**: `/workspace/system_features/ml/README.md`
- **API Reference**: Generated API docs in `docs/`
- **MultiOS Integration**: `integration/` directory examples
- **Performance Tuning**: `tutorials/05_performance_optimization.rs`

## Contributing

To add new examples:
1. Follow the template structure in `templates/`
2. Include comprehensive documentation
3. Add visualization options where appropriate
4. Test with multiple dataset sizes
5. Update this README with new examples

For questions about the ML framework, refer to the main documentation or the tutorials directory.