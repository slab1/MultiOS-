// Educational ML Framework - Tutorial 01: Basic Concepts
// Introduction to tensors, operations, and fundamental ML concepts
// This tutorial is designed for students learning machine learning fundamentals

use multi_os_ml::runtime::tensor::Tensor;
use multi_os_ml::runtime::memory::MemoryManager;
use multi_os_ml::runtime::performance::PerformanceMonitor;
use multi_os_ml::neural_net::utils::{ActivationFunction, Optimizer, LossFunction};
use std::time::Instant;

/// Tutorial 01: Basic Concepts in Machine Learning
/// 
/// Learning Objectives:
/// 1. Understand tensors as the fundamental data structure in ML
/// 2. Learn basic tensor operations
/// 3. Explore different data types and shapes
/// 4. Understand memory management concepts
/// 5. Perform simple mathematical operations
/// 6. Learn about computational graphs
/// 7. Introduction to gradients and backpropagation concepts

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== MultiOS ML Framework - Tutorial 01: Basic Concepts ===");
    println!("Welcome to your first steps into machine learning!\n");
    
    // Welcome and overview
    tutorial_introduction();
    
    // Section 1: Tensors - The Foundation
    section_1_tensors_introduction();
    
    // Section 2: Basic Tensor Operations
    section_2_tensor_operations();
    
    // Section 3: Mathematical Operations
    section_3_mathematical_operations();
    
    // Section 4: Memory Management
    section_4_memory_management();
    
    // Section 5: Performance Monitoring
    section_5_performance_monitoring();
    
    // Section 6: Gradients and Backpropagation
    section_6_gradients_introduction();
    
    // Section 7: Computational Graphs
    section_7_computational_graphs();
    
    // Summary and next steps
    tutorial_summary();
    
    Ok(())
}

fn tutorial_introduction() {
    println!("🎯 LEARNING OBJECTIVES:");
    println!("After completing this tutorial, you will understand:");
    println!("• What tensors are and why they're essential in ML");
    println!("• Basic tensor operations and manipulations");
    println!("• Memory management in ML systems");
    println!("• How to monitor performance and memory usage");
    println!("• The basics of gradients and backpropagation");
    println!("• How computational graphs represent ML computations\n");
    
    println!("📚 WHAT IS A TENSOR?");
    println!("A tensor is a multi-dimensional array that stores data.");
    println!("• 0D tensor = Scalar (single number)");
    println!("• 1D tensor = Vector (array of numbers)");
    println!("• 2D tensor = Matrix (table of numbers)");
    println!("• 3D+ tensors = Higher-dimensional arrays\n");
    
    println!("🖥️  MULTIOS INTEGRATION:");
    println!("This framework integrates with MultiOS for:");
    println!("• Optimized memory management");
    println!("• Performance monitoring and profiling");
    println!("• Efficient computation scheduling");
    println!("• Educational visualization tools\n");
    
    press_continue();
}

fn section_1_tensors_introduction() {
    println!("=== SECTION 1: INTRODUCTION TO TENSORS ===\n");
    
    println!("🔤 CREATING DIFFERENT TENSOR TYPES:\n");
    
    // Scalar (0D tensor)
    println!("1. SCALAR (0D tensor) - A single number:");
    let scalar = Tensor::from(3.14);
    println!("   Value: {:?}", scalar);
    println!("   Shape: {:?}", scalar.shape());
    println!("   Dimensions: {}", scalar.shape().len());
    
    // Vector (1D tensor)
    println!("\n2. VECTOR (1D tensor) - An array of numbers:");
    let vector = Tensor::from(vec![1.0, 2.0, 3.0, 4.0, 5.0]);
    println!("   Values: {:?}", vector);
    println!("   Shape: {:?}", vector.shape());
    println!("   Length: {}", vector.shape()[0]);
    
    // Matrix (2D tensor)
    println!("\n3. MATRIX (2D tensor) - A table of numbers:");
    let matrix_data = vec![
        vec![1.0, 2.0, 3.0],
        vec![4.0, 5.0, 6.0],
        vec![7.0, 8.0, 9.0],
    ];
    let matrix = Tensor::from_2d(&matrix_data);
    println!("   Values:");
    for row in matrix_data {
        println!("   {:?}", row);
    }
    println!("   Shape: {:?} ({} rows × {} columns)", 
             matrix.shape(), matrix.shape()[0], matrix.shape()[1]);
    
    // 3D tensor (common in image processing)
    println!("\n4. 3D TENSOR - Used for images (height × width × channels):");
    let image_tensor = Tensor::random_normal(vec![4, 4, 3], 0.0, 1.0);
    println!("   Shape: {:?} (4×4 image with 3 color channels)", image_tensor.shape());
    println!("   Total elements: {}", image_tensor.num_elements());
    
    // 4D tensor (common in batch processing)
    println!("\n5. 4D TENSOR - Used for batches of images:");
    let batch_tensor = Tensor::zeros(vec![8, 28, 28, 1]);
    println!("   Shape: {:?} (batch of 8, 28×28 grayscale images)", batch_tensor.shape());
    println!("   Total elements: {}\n", batch_tensor.num_elements());
    
    println!("💡 EDUCATIONAL INSIGHTS:");
    println!("• Shape tells us the dimensions of our data");
    println!("• Number of elements = product of all dimensions");
    println!("• Different shapes serve different purposes in ML");
    println!("• Understanding shapes is crucial for debugging models\n");
    
    press_continue();
}

fn section_2_tensor_operations() {
    println!("=== SECTION 2: BASIC TENSOR OPERATIONS ===\n");
    
    println!("🔧 COMMON TENSOR CREATION METHODS:\n");
    
    // Zeros tensor
    println!("1. ZEROS - Initialize with zeros:");
    let zeros = Tensor::zeros(vec![3, 4]);
    println!("   3×4 zeros tensor: shape {:?}", zeros.shape());
    
    // Ones tensor
    println!("\n2. ONES - Initialize with ones:");
    let ones = Tensor::ones(vec![2, 3]);
    println!("   2×3 ones tensor: shape {:?}", ones.shape());
    
    // Random normal distribution
    println!("\n3. RANDOM NORMAL - Gaussian distribution:");
    let random_normal = Tensor::random_normal(vec![3, 3], 0.0, 1.0);
    println!("   3×3 random tensor (μ=0, σ=1):");
    display_tensor_data(&random_normal);
    
    // Random uniform distribution
    println!("\n4. RANDOM UNIFORM - Uniform distribution [0,1):");
    let random_uniform = Tensor::random_uniform(vec![2, 4], 0.0, 1.0);
    println!("   2×4 random tensor:");
    display_tensor_data(&random_uniform);
    
    // Range tensor
    println!("\n5. RANGE - Sequential numbers:");
    let range_tensor = Tensor::range(1.0, 10.0, 1.0);
    println!("   Range 1 to 10: {:?}", range_tensor);
    
    // Identity matrix
    println!("\n6. IDENTITY - Square matrix with 1s on diagonal:");
    let identity = Tensor::eye(4);
    println!("   4×4 identity matrix:");
    display_tensor_data(&identity);
    
    println!("\n🔍 SHAPE MANIPULATION:\n");
    
    let original = Tensor::from(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
    println!("Original 1D tensor: {:?}", original);
    
    // Reshape
    let reshaped = original.reshape(vec![2, 3]);
    println!("Reshaped to 2×3: {:?}", reshaped);
    
    // Transpose
    let transposed = reshaped.transpose();
    println!("Transposed: {:?}", transposed);
    
    // Squeeze and unsqueeze
    let tensor_with_extra_dim = Tensor::from_3d(&[[[1.0, 2.0]]]);
    println!("3D tensor with extra dimension: shape {:?}", tensor_with_extra_dim.shape());
    
    let squeezed = tensor_with_extra_dim.squeeze();
    println!("After squeezing: shape {:?}", squeezed.shape());
    
    println!("\n💡 EDUCATIONAL INSIGHTS:");
    println!("• Reshaping changes how we view the same data");
    println!("• Transpose swaps rows and columns (important for matrix operations)");
    println!("• Squeeze removes dimensions of size 1");
    println!("• Understanding shape manipulation helps in data preprocessing\n");
    
    press_continue();
}

fn section_3_mathematical_operations() {
    println!("=== SECTION 3: MATHEMATICAL OPERATIONS ===\n");
    
    println!("🧮 ELEMENT-WISE OPERATIONS:\n");
    
    let a = Tensor::from(vec![1.0, 2.0, 3.0, 4.0]);
    let b = Tensor::from(vec![5.0, 6.0, 7.0, 8.0]);
    
    println!("Tensor A: {:?}", a);
    println!("Tensor B: {:?}", b);
    
    // Addition
    println!("\n1. ADDITION (element-wise):");
    let sum = a.add(&b);
    println!("   A + B = {:?}", sum);
    
    // Subtraction
    println!("\n2. SUBTRACTION (element-wise):");
    let difference = b.sub(&a);
    println!("   B - A = {:?}", difference);
    
    // Multiplication
    println!("\n3. MULTIPLICATION (element-wise):");
    let product = a.mul(&b);
    println!("   A × B = {:?}", product);
    
    // Division
    println!("\n4. DIVISION (element-wise):");
    let division = b.div(&a);
    println!("   B ÷ A = {:?}", division);
    
    // Power
    println!("\n5. POWER:");
    let power = a.pow(2.0);
    println!("   A² = {:?}", power);
    
    // Square root
    println!("\n6. SQUARE ROOT:");
    let sqrt_vals = Tensor::from(vec![4.0, 9.0, 16.0, 25.0]).sqrt();
    println!("   √[4,9,16,25] = {:?}", sqrt_vals);
    
    println!("\n🔢 MATRIX OPERATIONS:\n");
    
    let matrix_a = Tensor::from_2d(&[
        vec![1.0, 2.0],
        vec![3.0, 4.0]
    ]);
    let matrix_b = Tensor::from_2d(&[
        vec![5.0, 6.0],
        vec![7.0, 8.0]
    ]);
    
    println!("Matrix A:");
    display_tensor_data(&matrix_a);
    println!("Matrix B:");
    display_tensor_data(&matrix_b);
    
    // Matrix multiplication
    println!("\n1. MATRIX MULTIPLICATION:");
    let matmul_result = matrix_a.matmul(&matrix_b);
    println!("   A × B = ");
    display_tensor_data(&matmul_result);
    
    // Matrix transpose
    println!("\n2. MATRIX TRANSPOSE:");
    let transposed = matrix_a.transpose();
    println!("   Aᵀ = ");
    display_tensor_data(&transposed);
    
    // Matrix inverse (educational note)
    println!("\n3. MATRIX INVERSE:");
    println!("   Note: Inverse operations are computationally expensive");
    println!("   Used in linear regression and certain optimization algorithms");
    
    println!("\n📊 STATISTICAL OPERATIONS:\n");
    
    let data = Tensor::from(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0]);
    println!("Dataset: {:?}", data);
    
    println!("\n1. SUM:");
    let sum = data.sum();
    println!("   Sum = {:.2}", sum);
    
    println!("\n2. MEAN:");
    let mean = data.mean();
    println!("   Mean = {:.2}", mean);
    
    println!("\n3. STANDARD DEVIATION:");
    let std = data.std();
    println!("   Standard Deviation = {:.2}", std);
    
    println!("\n4. MIN/MAX:");
    let (min_val, max_val) = data.min_max();
    println!("   Min = {:.2}, Max = {:.2}", min_val, max_val);
    
    println!("\n💡 EDUCATIONAL INSIGHTS:");
    println!("• Element-wise operations apply to each element independently");
    println!("• Matrix multiplication is fundamental in neural networks");
    println!("• Statistical operations help understand data distribution");
    println!("• These operations form the building blocks of ML algorithms\n");
    
    press_continue();
}

fn section_4_memory_management() {
    println!("=== SECTION 4: MEMORY MANAGEMENT ===\n");
    
    println!("🧠 WHY MEMORY MANAGEMENT MATTERS:\n");
    println!("• ML models often work with large datasets");
    println!("• Efficient memory usage prevents out-of-memory errors");
    println!("• MultiOS provides optimized memory management");
    println!("• Understanding memory helps optimize performance\n");
    
    println!("💾 MEMORY POOLS:\n");
    
    let memory_manager = MemoryManager::new();
    
    println!("1. CREATING MEMORY POOL:");
    let pool = memory_manager.create_pool(1024 * 1024); // 1MB pool
    println!("   Created pool of size: {} bytes", pool.size());
    
    println!("\n2. ALLOCATING FROM POOL:");
    let allocation = pool.allocate(1024); // Allocate 1KB
    println!("   Allocated: {} bytes", allocation.size());
    
    println!("\n3. MEMORY POOLS HELP BY:");
    println!("   • Reducing allocation overhead");
    println!("   • Improving cache locality");
    println!("   • Preventing memory fragmentation");
    println!("   • Enabling efficient garbage collection");
    
    println!("\n📏 MEMORY TRACKING:\n");
    
    let mut tracker = memory_manager.create_tracker();
    
    println!("4. TRACKING MEMORY USAGE:");
    
    // Create some tensors to track
    let tensor1 = Tensor::zeros(vec![1000, 1000]);
    let tensor2 = Tensor::random_normal(vec![500, 500], 0.0, 1.0);
    
    let usage1 = tracker.get_current_usage();
    println!("   After creating tensors: {:.2} MB", usage1 as f64 / (1024.0 * 1024.0));
    
    // Memory statistics
    let stats = tracker.get_statistics();
    println!("   Total allocations: {}", stats.total_allocations);
    println!("   Peak usage: {:.2} MB", stats.peak_usage as f64 / (1024.0 * 1024.0));
    println!("   Active tensors: {}", stats.active_tensors);
    
    println!("\n🔍 TENSOR MEMORY ANALYSIS:\n");
    
    let large_tensor = Tensor::random_normal(vec![1000, 1000, 3], 0.0, 1.0);
    let memory_info = large_tensor.get_memory_info();
    
    println!("5. ANALYZING TENSOR MEMORY:");
    println!("   Tensor shape: {:?}", large_tensor.shape());
    println!("   Memory size: {:.2} MB", memory_info.size as f64 / (1024.0 * 1024.0));
    println!("   Memory layout: {:?}", memory_info.layout);
    println!("   Contiguous: {}", memory_info.is_contiguous);
    
    println!("\n💡 EDUCATIONAL INSIGHTS:");
    println!("• Memory management is crucial for scaling ML models");
    println!("• MultiOS optimizes memory allocation patterns");
    println!("• Understanding memory usage helps in model design");
    println!("• Always monitor memory usage with large datasets\n");
    
    press_continue();
}

fn section_5_performance_monitoring() {
    println!("=== SECTION 5: PERFORMANCE MONITORING ===\n");
    
    println!("⚡ WHY PERFORMANCE MONITORING MATTERS:\n");
    println!("• Identify bottlenecks in ML pipelines");
    println!("• Optimize training and inference speed");
    println!("• Understand resource utilization");
    println!("• Debug performance issues");
    
    println!("\n📊 PERFORMANCE MONITOR:\n");
    
    let performance_monitor = PerformanceMonitor::new();
    
    println!("1. CREATING PERFORMANCE MONITOR:");
    let monitor = performance_monitor.create_monitor();
    println!("   Monitor initialized successfully");
    
    println!("\n2. MONITORING TENSOR OPERATIONS:");
    
    // Create some operations to monitor
    let start_time = Instant::now();
    
    let tensor_a = Tensor::random_normal(vec![500, 500], 0.0, 1.0);
    let tensor_b = Tensor::random_normal(vec![500, 500], 0.0, 1.0);
    
    let matrix_mult = tensor_a.matmul(&tensor_b);
    let operation_time = start_time.elapsed();
    
    println!("   Matrix multiplication completed in: {:?}", operation_time);
    
    let op_stats = monitor.get_operation_stats("matmul");
    println!("   Operation: {}", op_stats.operation_name);
    println!("   Executions: {}", op_stats.executions);
    println!("   Average time: {:?}", op_stats.average_time);
    println!("   Total time: {:?}", op_stats.total_time);
    
    println!("\n3. PROFILING MULTIPLE OPERATIONS:");
    
    let mut operations = Vec::new();
    
    // Profile different operations
    let operations_to_profile = [
        ("addition", || { Tensor::ones(vec![1000, 1000]).add(&Tensor::zeros(vec![1000, 1000])); }),
        ("multiplication", || { Tensor::ones(vec![1000, 1000]).mul(&Tensor::ones(vec![1000, 1000])); }),
        ("transpose", || { Tensor::random_normal(vec![1000, 500], 0.0, 1.0).transpose(); }),
    ];
    
    for (name, operation) in &operations_to_profile {
        let start = Instant::now();
        operation();
        let duration = start.elapsed();
        
        monitor.record_operation(name, duration);
        println!("   {} took: {:?}", name, duration);
    }
    
    println!("\n4. GENERATING PERFORMANCE REPORT:");
    let report = monitor.generate_report();
    println!("   Report generated:");
    println!("   {}", report);
    
    println!("\n🎯 PERFORMANCE OPTIMIZATION TIPS:\n");
    
    println!("• Use appropriate data types (float32 vs float64)");
    println!("• Leverage SIMD instructions for vectorized operations");
    println!("• Minimize memory allocations and copies");
    println!("• Use efficient algorithms (e.g., Strassen for large matrix mult)");
    println!("• Profile code to identify actual bottlenecks");
    println!("• Consider MultiOS parallel processing capabilities");
    
    println!("\n💡 EDUCATIONAL INSIGHTS:");
    println!("• Performance monitoring is essential for production ML systems");
    println!("• MultiOS provides integrated profiling tools");
    println!("• Understanding performance helps in algorithm selection");
    println!("• Always measure before optimizing\n");
    
    press_continue();
}

fn section_6_gradients_introduction() {
    println!("=== SECTION 6: GRADIENTS AND BACKPROPAGATION ===\n");
    
    println!("📈 WHAT ARE GRADIENTS?\n");
    println!("Gradients tell us how to change parameters to improve our model:");
    println!("• Gradient = direction and rate of fastest increase");
    println!("• Negative gradient = direction of fastest decrease");
    println!("• Used to minimize loss functions");
    println!("• Fundamental to training neural networks");
    
    println!("\n🎯 SIMPLE GRADIENT EXAMPLE:\n");
    
    println!("Let's understand gradients with a simple function: f(x) = x²");
    println!("The derivative (gradient) is: f'(x) = 2x");
    
    let x = Tensor::from(3.0);
    let y = x.pow(2.0);
    let gradient = x.mul(&Tensor::from(2.0)); // Manual gradient for f(x) = x²
    
    println!("\nAt x = 3.0:");
    println!("  f(x) = x² = {:.2}", y.data()[0]);
    println!("  f'(x) = 2x = {:.2}", gradient.data()[0]);
    println!("  Gradient points in direction of increasing function");
    println!("  To minimize: move in opposite direction (subtract gradient)");
    
    println!("\n🔄 GRADIENT DESCENT:\n");
    
    println!("Simple gradient descent algorithm:");
    println!("1. Start with initial guess");
    println!("2. Compute gradient at current position");
    println!("3. Move opposite to gradient (downhill)");
    println!("4. Repeat until convergence");
    
    // Simulate gradient descent for x²
    println!("\nExample: Minimizing f(x) = x²");
    let mut current_x = Tensor::from(5.0);
    let learning_rate = 0.1;
    
    println!("Starting at x = 5.0");
    for iteration in 0..5 {
        let current_value = current_x.pow(2.0);
        let gradient = current_x.mul(&Tensor::from(2.0));
        
        // Update: x = x - learning_rate * gradient
        current_x = current_x.sub(&gradient.mul(&Tensor::from(learning_rate)));
        
        println!("  Iteration {}: x = {:.3}, f(x) = {:.3}, gradient = {:.3}", 
                 iteration + 1, 
                 current_x.data()[0], 
                 current_value.data()[0], 
                 gradient.data()[0]);
    }
    
    println!("\n📚 BACKPROPAGATION:\n");
    
    println!("Backpropagation is how neural networks learn:");
    println!("1. Forward pass: compute predictions and loss");
    println!("2. Backward pass: compute gradients through chain rule");
    println!("3. Update weights using gradients");
    println!("4. Repeat for many training examples");
    
    println!("\nChain Rule Example: f(g(x))");
    println!("  f(u) = u², u = g(x) = 2x");
    println!("  df/dx = df/du × du/dx = 2u × 2 = 4x");
    
    let x_val = Tensor::from(2.0);
    let u_val = x_val.mul(&Tensor::from(2.0));
    let f_val = u_val.pow(2.0);
    let df_du = u_val.mul(&Tensor::from(2.0));
    let du_dx = Tensor::from(2.0);
    let df_dx = df_du.mul(&du_dx);
    
    println!("\nAt x = 2.0:");
    println!("  u = 2x = {:.2}", u_val.data()[0]);
    println!("  f(u) = u² = {:.2}", f_val.data()[0]);
    println!("  df/du = 2u = {:.2}", df_du.data()[0]);
    println!("  du/dx = 2");
    println!("  df/dx = df/du × du/dx = {:.2}", df_dx.data()[0]);
    
    println!("\n⚠️  GRADIENT PROBLEMS:\n");
    
    println!("Common issues in deep networks:");
    println!("• Vanishing gradients: gradients become too small");
    println!("• Exploding gradients: gradients become too large");
    println!("• Local minima: getting stuck in suboptimal solutions");
    println!("• Saddle points: flat regions in loss landscape");
    
    println!("\nSolutions:");
    println!("• Proper weight initialization");
    println!("• Use ReLU or other non-saturating activations");
    println!("• Batch normalization");
    println!("• Residual connections (skip connections)");
    println!("• Adaptive learning rate methods (Adam, RMSprop)");
    
    println!("\n💡 EDUCATIONAL INSIGHTS:");
    println!("• Gradients are the 'learning signal' in ML");
    println!("• Understanding gradients helps debug training issues");
    println!("• MultiOS provides automatic differentiation capabilities");
    println!("• Gradients guide parameter updates in optimization\n");
    
    press_continue();
}

fn section_7_computational_graphs() {
    println!("=== SECTION 7: COMPUTATIONAL GRAPHS ===\n");
    
    println!("🔗 WHAT ARE COMPUTATIONAL GRAPHS?\n");
    
    println!("Computational graphs represent ML computations as nodes and edges:");
    println!("• Nodes: operations or variables");
    println!("• Edges: data flow between operations");
    println!("• Enable automatic differentiation");
    println!("• Optimize computation order");
    println!("• Enable parallel execution");
    
    println!("\n📊 SIMPLE GRAPH EXAMPLE:\n");
    
    println!("Consider computing: z = (x + y) × w");
    println!("This can be represented as a graph:");
    println!("  x → [+1] → [a] → [×2] → z");
    println!("  y →     ↗                   ↗");
    println!("  w →                   ↗");
    
    // Simulate the computation
    println!("\nManual computation:");
    let x = Tensor::from(3.0);
    let y = Tensor::from(4.0);
    let w = Tensor::from(2.0);
    
    let addition = x.add(&y);
    let multiplication = addition.mul(&w);
    
    println!("  Step 1: x + y = {} + {} = {}", 
             x.data()[0], y.data()[0], addition.data()[0]);
    println!("  Step 2: (x + y) × w = {} × {} = {}", 
             addition.data()[0], w.data()[0], multiplication.data()[0]);
    
    println!("\n🎯 BENEFITS OF COMPUTATIONAL GRAPHS:\n");
    
    println!("1. AUTOMATIC DIFFERENTIATION:");
    println!("   • Compute gradients automatically");
    println!("   • Apply chain rule systematically");
    println!("   • Handle complex nested operations");
    
    println!("\n2. OPTIMIZATION:");
    println!("   • Reorder operations for efficiency");
    println!("   • Eliminate common subexpressions");
    println!("   • Fuse operations when possible");
    
    println!("\n3. PARALLELIZATION:");
    println!("   • Identify independent operations");
    println!("   • Execute in parallel on multiple cores");
    println!("   • Optimize memory access patterns");
    
    println!("\n4. MEMORY MANAGEMENT:");
    println!("   • Track tensor lifetimes");
    println!("   • Free unnecessary intermediate values");
    println!("   • Optimize memory allocation");
    
    println!("\n📈 FORWARD AND BACKWARD PASSES:\n");
    
    println!("In a computational graph:");
    println!("\nFORWARD PASS (Evaluation):");
    println!("  • Compute operations in topological order");
    println!("  • Store intermediate results");
    println!("  • Calculate final output");
    
    println!("\nBACKWARD PASS (Gradient Computation):");
    println!("  • Start from output and work backwards");
    println!("  • Apply chain rule at each node");
    println!("  • Accumulate gradients from multiple paths");
    
    println!("\n🔧 MULTIOS INTEGRATION:\n");
    
    println!("MultiOS enhances computational graphs with:");
    println!("• Distributed graph execution across cores");
    println!("• Automatic load balancing");
    println!("• Memory-aware operation scheduling");
    println!("• Performance profiling and optimization");
    println!("• Integration with hardware accelerators");
    
    println!("\n💡 EDUCATIONAL INSIGHTS:");
    println!("• Computational graphs are fundamental to modern ML frameworks");
    println!("• Understanding graphs helps in debugging complex models");
    println!("• MultiOS provides optimized graph execution");
    println!("• Graphs enable both efficiency and automatic differentiation\n");
    
    press_continue();
}

fn tutorial_summary() {
    println!("=== TUTORIAL 01 SUMMARY ===\n");
    
    println!("🎓 WHAT YOU'VE LEARNED:\n");
    
    println!("✅ TENSORS:");
    println!("   • Created scalars, vectors, matrices, and higher-dimensional arrays");
    println!("   • Understood shape concepts and dimensions");
    println!("   • Learned tensor creation methods (zeros, ones, random, etc.)");
    
    println!("\n✅ OPERATIONS:");
    println!("   • Performed element-wise operations (add, sub, mul, div)");
    println!("   • Applied matrix operations (matmul, transpose)");
    println!("   • Used statistical functions (sum, mean, std, min/max)");
    
    println!("\n✅ MEMORY MANAGEMENT:");
    println!("   • Understood memory pools and allocation strategies");
    println!("   • Learned about memory tracking and analysis");
    println!("   • Explored tensor memory information");
    
    println!("\n✅ PERFORMANCE MONITORING:");
    println!("   • Profiled tensor operations");
    println!("   • Generated performance reports");
    println!("   • Learned optimization principles");
    
    println!("\n✅ GRADIENTS AND BACKPROPAGATION:");
    println!("   • Understood what gradients are and why they matter");
    println!("   • Learned gradient descent principles");
    println!("   • Explored chain rule and automatic differentiation");
    
    println!("\n✅ COMPUTATIONAL GRAPHS:");
    println!("   • Understood graph representation of computations");
    println!("   • Learned about forward and backward passes");
    println!("   • Explored MultiOS integration benefits");
    
    println!("\n🚀 NEXT STEPS:\n");
    
    println!("Recommended progression:");
    println!("1. Tutorial 02: Building Your First Neural Network");
    println!("2. Tutorial 03: Visualization and Debugging");
    println!("3. Tutorial 04: Parallel Training");
    println!("4. Tutorial 05: Performance Optimization");
    println!("5. Tutorial 06: Production Deployment");
    
    println!("\n📚 PRACTICE IDEAS:");
    println!("• Experiment with different tensor shapes and operations");
    println!("• Monitor performance with various tensor sizes");
    println!("• Implement simple gradient descent for different functions");
    println!("• Explore MultiOS memory management with large tensors");
    
    println!("\n🛠️  EXPLORE THE FRAMEWORK:");
    println!("• Try the classification template");
    println!("• Experiment with regression models");
    println!("• Build CNNs for image recognition");
    println!("• Create RNNs for text processing");
    
    println!("\n💡 KEY TAKEAWAYS:");
    println!("• Tensors are the foundation of ML - master their manipulation");
    println!("• Memory management is crucial for scaling ML models");
    println!("• Performance monitoring helps optimize ML pipelines");
    println!("• Understanding gradients is key to understanding learning");
    println!("• Computational graphs enable efficient, automatic differentiation");
    
    println!("\n🎉 Congratulations on completing Tutorial 01!");
    println!("You're now ready to build your first neural network!\n");
}

// Helper functions
fn display_tensor_data(tensor: &Tensor) {
    let data = tensor.data();
    let shape = tensor.shape();
    
    if shape.len() == 1 {
        println!("   {:?}", data);
    } else if shape.len() == 2 {
        let rows = shape[0];
        let cols = shape[1];
        for i in 0..rows {
            let start = i * cols;
            let end = start + cols;
            println!("   {:?}", &data[start..end]);
        }
    } else {
        println!("   Tensor shape: {:?}", shape);
        println!("   (Display truncated for higher dimensions)");
    }
}

fn press_continue() {
    println!("\n" + &"=".repeat(60));
    println!("Press Enter to continue to the next section...");
    println!("" + &"=".repeat(60));
    
    // In a real tutorial, this would wait for user input
    // For now, we'll just add a pause
    std::thread::sleep(std::time::Duration::from_millis(500));
}