// Educational ML Framework - MultiOS Integration Examples
// Demonstrates how to use the ML framework with MultiOS performance monitoring tools

use multi_os_ml::neural_net::models::SimpleNN;
use multi_os_ml::neural_net::layers::{DenseLayer, Conv2DLayer, MaxPool2DLayer};
use multi_os_ml::neural_net::utils::ActivationFunction;
use multi_os_ml::runtime::performance::PerformanceMonitor;
use multi_os_ml::runtime::memory::MemoryManager;
use multi_os_ml::data_pipeline::DataPipeline;
use multi_os_ml::runtime::tensor::Tensor;
use std::collections::HashMap;
use std::time::Instant;

/// MultiOS Integration Configuration
#[derive(Clone)]
pub struct MultiOSIntegrationConfig {
    pub enable_performance_monitoring: bool,
    pub enable_memory_profiling: bool,
    pub enable_resource_monitoring: bool,
    pub monitoring_interval: u64,        // Milliseconds
    pub save_monitoring_data: bool,
    pub performance_report_path: Option<String>,
    pub alert_thresholds: AlertThresholds,
}

/// Alert thresholds for performance monitoring
#[derive(Clone)]
pub struct AlertThresholds {
    pub memory_usage_mb: f64,           // Alert if memory usage exceeds this
    pub training_time_seconds: f64,     // Alert if epoch takes longer than this
    pub gpu_utilization_percent: f64,   // Alert if GPU usage is too high/low
    pub cpu_utilization_percent: f64,   // Alert if CPU usage is abnormal
}

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            memory_usage_mb: 1000.0,     // 1GB threshold
            training_time_seconds: 60.0, // 1 minute per epoch
            gpu_utilization_percent: 95.0, // Alert if GPU usage too high
            cpu_utilization_percent: 90.0, // Alert if CPU usage too high
        }
    }
}

/// Main integration structure for MultiOS ML monitoring
pub struct MultiOSMLMonitor {
    config: MultiOSIntegrationConfig,
    performance_monitor: PerformanceMonitor,
    memory_manager: MemoryManager,
    session_start: Instant,
    monitoring_data: Vec<MonitoringSnapshot>,
}

/// Performance monitoring snapshot
pub struct MonitoringSnapshot {
    timestamp: std::time::Instant,
    memory_usage_mb: f64,
    cpu_utilization_percent: f64,
    gpu_utilization_percent: f64,
    active_tensors: usize,
    training_epoch: usize,
    batch_index: usize,
    epoch_duration_ms: u64,
    batch_duration_ms: u64,
}

impl MultiOSIntegrationConfig {
    pub fn new() -> Self {
        Self {
            enable_performance_monitoring: true,
            enable_memory_profiling: true,
            enable_resource_monitoring: true,
            monitoring_interval: 1000,  // 1 second
            save_monitoring_data: true,
            performance_report_path: Some("monitoring_report.json".to_string()),
            alert_thresholds: AlertThresholds::default(),
        }
    }
    
    /// Disable performance monitoring
    pub fn disable_performance_monitoring(mut self) -> Self {
        self.enable_performance_monitoring = false;
        self
    }
    
    /// Disable memory profiling
    pub fn disable_memory_profiling(mut self) -> Self {
        self.enable_memory_profiling = false;
        self
    }
    
    /// Set custom monitoring interval
    pub fn monitoring_interval_ms(mut self, interval: u64) -> Self {
        self.monitoring_interval = interval;
        self
    }
    
    /// Set custom report path
    pub fn report_path(mut self, path: &str) -> Self {
        self.performance_report_path = Some(path.to_string());
        self
    }
    
    /// Set custom alert thresholds
    pub fn alert_thresholds(mut self, thresholds: AlertThresholds) -> Self {
        self.alert_thresholds = thresholds;
        self
    }
}

impl MultiOSMLMonitor {
    /// Create new MultiOS ML monitor
    pub fn new(config: MultiOSIntegrationConfig) -> Self {
        let performance_monitor = PerformanceMonitor::new();
        let memory_manager = MemoryManager::new();
        
        println!("🔧 MultiOS ML Monitor initialized:");
        println!("   Performance monitoring: {}", config.enable_performance_monitoring);
        println!("   Memory profiling: {}", config.enable_memory_profiling);
        println!("   Resource monitoring: {}", config.enable_resource_monitoring);
        println!("   Monitoring interval: {}ms", config.monitoring_interval);
        
        Self {
            config,
            performance_monitor,
            memory_manager,
            session_start: Instant::now(),
            monitoring_data: Vec::new(),
        }
    }
    
    /// Start training session with full monitoring
    pub fn start_training_session(&mut self) {
        println!("\n🚀 Starting MultiOS-monitored training session...");
        
        if self.config.enable_performance_monitoring {
            println!("   ✓ Performance monitoring enabled");
            self.performance_monitor.start_monitoring();
        }
        
        if self.config.enable_memory_profiling {
            println!("   ✓ Memory profiling enabled");
            self.memory_manager.start_tracking();
        }
        
        println!("   🕐 Session started at: {:?}", self.session_start);
    }
    
    /// Monitor training epoch
    pub fn monitor_training_epoch(&mut self, epoch: usize, total_epochs: usize) -> TrainingMetrics {
        let epoch_start = Instant::now();
        
        // Collect current system metrics
        let snapshot = self.collect_system_metrics(epoch, 0);
        self.monitoring_data.push(snapshot);
        
        // Check for alerts
        self.check_alerts();
        
        // Simulate training metrics (in real implementation, these would come from actual training)
        let training_metrics = self.simulate_training_metrics(epoch, total_epochs);
        
        let epoch_duration = epoch_start.elapsed();
        
        println!("📊 Epoch {}/{} monitoring:", epoch + 1, total_epochs);
        println!("   Duration: {:?}", epoch_duration);
        println!("   Memory usage: {:.1} MB", snapshot.memory_usage_mb);
        println!("   CPU utilization: {:.1}%", snapshot.cpu_utilization_percent);
        
        TrainingMetrics {
            epoch,
            duration_ms: epoch_duration.as_millis() as u64,
            memory_usage_mb: snapshot.memory_usage_mb,
            cpu_utilization: snapshot.cpu_utilization_percent,
            gpu_utilization: snapshot.gpu_utilization_percent,
            loss: training_metrics.loss,
            accuracy: training_metrics.accuracy,
        }
    }
    
    /// Monitor training batch
    pub fn monitor_training_batch(&mut self, epoch: usize, batch: usize, total_batches: usize) -> BatchMetrics {
        let batch_start = Instant::now();
        
        let snapshot = self.collect_system_metrics(epoch, batch);
        self.monitoring_data.push(snapshot);
        
        let batch_duration = batch_start.elapsed();
        
        BatchMetrics {
            epoch,
            batch,
            duration_ms: batch_duration.as_millis() as u64,
            memory_usage_mb: snapshot.memory_usage_mb,
            active_tensors: snapshot.active_tensors,
        }
    }
    
    /// Monitor model inference
    pub fn monitor_inference(&mut self, input_size: usize) -> InferenceMetrics {
        let inference_start = Instant::now();
        
        // Track inference-specific metrics
        let pre_inference_memory = self.memory_manager.get_current_usage();
        
        let snapshot = self.collect_system_metrics(0, 0);
        
        // Simulate inference time (in real implementation, this would be actual inference)
        std::thread::sleep(std::time::Duration::from_millis(10));
        
        let inference_duration = inference_start.elapsed();
        let post_inference_memory = self.memory_manager.get_current_usage();
        
        InferenceMetrics {
            input_size,
            inference_time_ms: inference_duration.as_millis() as u64,
            memory_delta_bytes: post_inference_memory as i64 - pre_inference_memory as i64,
            throughput_samples_per_sec: input_size as f64 / (inference_duration.as_millis() as f64 / 1000.0),
        }
    }
    
    /// Monitor memory optimization
    pub fn monitor_memory_optimization(&self) -> MemoryOptimizationReport {
        let memory_stats = self.memory_manager.get_statistics();
        let optimization_suggestions = self.generate_optimization_suggestions(&memory_stats);
        
        println!("\n💾 Memory Optimization Analysis:");
        println!("   Peak usage: {:.1} MB", memory_stats.peak_usage as f64 / (1024.0 * 1024.0));
        println!("   Active tensors: {}", memory_stats.active_tensors);
        println!("   Memory allocations: {}", memory_stats.total_allocations);
        
        if !optimization_suggestions.is_empty() {
            println!("   💡 Optimization suggestions:");
            for suggestion in &optimization_suggestions {
                println!("     • {}", suggestion);
            }
        }
        
        MemoryOptimizationReport {
            peak_memory_mb: memory_stats.peak_usage as f64 / (1024.0 * 1024.0),
            active_tensors: memory_stats.active_tensors,
            total_allocations: memory_stats.total_allocations,
            optimization_suggestions,
        }
    }
    
    /// Generate comprehensive performance report
    pub fn generate_performance_report(&self) -> String {
        let mut report = String::new();
        
        report.push_str(&format!("# MultiOS ML Performance Report\n"));
        report.push_str(&format!("Generated: {:?}\n\n", self.session_start.elapsed()));
        
        // Session summary
        report.push_str(&format!("## Session Summary\n"));
        report.push_str(&format!("Duration: {:?}\n", self.session_start.elapsed()));
        report.push_str(&format!("Total monitoring snapshots: {}\n\n", self.monitoring_data.len()));
        
        // Performance metrics
        if !self.monitoring_data.is_empty() {
            let avg_memory = self.monitoring_data.iter()
                .map(|s| s.memory_usage_mb)
                .sum::<f64>() / self.monitoring_data.len() as f64;
            
            let peak_memory = self.monitoring_data.iter()
                .map(|s| s.memory_usage_mb)
                .fold(0.0, f64::max);
            
            let avg_cpu = self.monitoring_data.iter()
                .map(|s| s.cpu_utilization_percent)
                .sum::<f64>() / self.monitoring_data.len() as f64;
            
            report.push_str(&format!("## Performance Metrics\n"));
            report.push_str(&format!("Average memory usage: {:.1} MB\n", avg_memory));
            report.push_str(&format!("Peak memory usage: {:.1} MB\n", peak_memory));
            report.push_str(&format!("Average CPU utilization: {:.1}%\n", avg_cpu));
            
            if let Some(gpu_avg) = self.monitoring_data.iter()
                .filter(|s| s.gpu_utilization_percent > 0.0)
                .map(|s| s.gpu_utilization_percent)
                .reduce(|a, b| a + b)
                .map(|sum| sum / self.monitoring_data.iter().filter(|s| s.gpu_utilization_percent > 0.0).count() as f64) {
                report.push_str(&format!("Average GPU utilization: {:.1}%\n", gpu_avg));
            }
        }
        
        // Resource utilization trends
        report.push_str(&format!("\n## Resource Utilization Trends\n"));
        if self.monitoring_data.len() > 1 {
            report.push_str(&format!("Memory trend: {} over session\n", 
                if self.monitoring_data.last().unwrap().memory_usage_mb > self.monitoring_data.first().unwrap().memory_usage_mb {
                    "increasing"
                } else {
                    "stable/decreasing"
                }));
        }
        
        // Recommendations
        let recommendations = self.generate_recommendations();
        if !recommendations.is_empty() {
            report.push_str(&format!("\n## Recommendations\n"));
            for (i, rec) in recommendations.iter().enumerate() {
                report.push_str(&format!("{}. {}\n", i + 1, rec));
            }
        }
        
        report
    }
    
    /// Save performance report to file
    pub fn save_performance_report(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(path) = &self.config.performance_report_path {
            let report = self.generate_performance_report();
            std::fs::write(path, report)?;
            println!("📄 Performance report saved to: {}", path);
        }
        Ok(())
    }
    
    /// End monitoring session
    pub fn end_monitoring_session(&mut self) {
        println!("\n🏁 Ending MultiOS monitoring session...");
        
        let session_duration = self.session_start.elapsed();
        println!("   Session duration: {:?}", session_duration);
        println!("   Total snapshots collected: {}", self.monitoring_data.len());
        
        if self.config.enable_performance_monitoring {
            self.performance_monitor.stop_monitoring();
        }
        
        if self.config.enable_memory_profiling {
            self.memory_manager.stop_tracking();
        }
        
        // Generate final report
        if self.config.save_monitoring_data {
            self.save_performance_report().unwrap_or_else(|e| {
                println!("Warning: Could not save performance report: {}", e);
            });
        }
    }
    
    // Helper methods
    
    fn collect_system_metrics(&self, epoch: usize, batch: usize) -> MonitoringSnapshot {
        MonitoringSnapshot {
            timestamp: Instant::now(),
            memory_usage_mb: self.get_memory_usage_mb(),
            cpu_utilization_percent: self.get_cpu_utilization(),
            gpu_utilization_percent: self.get_gpu_utilization(),
            active_tensors: self.get_active_tensor_count(),
            training_epoch: epoch,
            batch_index: batch,
            epoch_duration_ms: 0,  // Would be filled by caller
            batch_duration_ms: 0,  // Would be filled by caller
        }
    }
    
    fn get_memory_usage_mb(&self) -> f64 {
        if self.config.enable_memory_profiling {
            self.memory_manager.get_current_usage() as f64 / (1024.0 * 1024.0)
        } else {
            0.0
        }
    }
    
    fn get_cpu_utilization(&self) -> f64 {
        if self.config.enable_resource_monitoring {
            // Simulate CPU utilization monitoring
            // In real implementation, would read from system
            45.0 + (self.session_start.elapsed().as_secs() as f64 * 0.1).sin() * 20.0
        } else {
            0.0
        }
    }
    
    fn get_gpu_utilization(&self) -> f64 {
        if self.config.enable_resource_monitoring {
            // Simulate GPU utilization monitoring
            // In real implementation, would query GPU
            78.0 + (self.session_start.elapsed().as_secs() as f64 * 0.05).cos() * 15.0
        } else {
            0.0
        }
    }
    
    fn get_active_tensor_count(&self) -> usize {
        if self.config.enable_memory_profiling {
            // Simulate active tensor count
            self.monitoring_data.len() + 10  // Increment with each snapshot
        } else {
            0
        }
    }
    
    fn check_alerts(&self) {
        if !self.config.enable_resource_monitoring {
            return;
        }
        
        if let Some(latest_snapshot) = self.monitoring_data.last() {
            // Check memory usage
            if latest_snapshot.memory_usage_mb > self.config.alert_thresholds.memory_usage_mb {
                println!("⚠️  ALERT: Memory usage ({:.1} MB) exceeds threshold ({:.1} MB)", 
                    latest_snapshot.memory_usage_mb, self.config.alert_thresholds.memory_usage_mb);
            }
            
            // Check CPU utilization
            if latest_snapshot.cpu_utilization_percent > self.config.alert_thresholds.cpu_utilization_percent {
                println!("⚠️  ALERT: CPU utilization ({:.1}%) is very high", 
                    latest_snapshot.cpu_utilization_percent);
            }
            
            // Check GPU utilization
            if latest_snapshot.gpu_utilization_percent > self.config.alert_thresholds.gpu_utilization_percent {
                println!("⚠️  ALERT: GPU utilization ({:.1}%) is very high", 
                    latest_snapshot.gpu_utilization_percent);
            }
        }
    }
    
    fn simulate_training_metrics(&self, epoch: usize, total_epochs: usize) -> TrainingMetrics {
        // Simulate decreasing loss and increasing accuracy over epochs
        let progress = epoch as f64 / total_epochs as f64;
        let loss = 2.0 * (1.0 - progress).max(0.1);
        let accuracy = progress.min(0.95);
        
        TrainingMetrics {
            epoch,
            duration_ms: 0,
            memory_usage_mb: 0.0,
            cpu_utilization: 0.0,
            gpu_utilization: 0.0,
            loss,
            accuracy,
        }
    }
    
    fn generate_optimization_suggestions(&self, memory_stats: &multi_os_ml::runtime::memory::MemoryStats) -> Vec<String> {
        let mut suggestions = Vec::new();
        
        // Check memory fragmentation
        if memory_stats.total_allocations > 1000 {
            suggestions.push("Consider using memory pools to reduce allocation overhead".to_string());
        }
        
        // Check peak memory usage
        let peak_mb = memory_stats.peak_usage as f64 / (1024.0 * 1024.0);
        if peak_mb > 500.0 {
            suggestions.push("High memory usage detected - consider batch size reduction".to_string());
        }
        
        // Check tensor count
        if memory_stats.active_tensors > 100 {
            suggestions.push("Many active tensors - consider tensor reuse strategies".to_string());
        }
        
        suggestions
    }
    
    fn generate_recommendations(&self) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        if self.monitoring_data.len() > 10 {
            let memory_trend = self.monitoring_data.windows(2)
                .map(|w| w[1].memory_usage_mb - w[0].memory_usage_mb)
                .sum::<f64>() / (self.monitoring_data.len() - 1) as f64;
            
            if memory_trend > 10.0 {
                recommendations.push("Memory usage is increasing over time - consider garbage collection optimization".to_string());
            }
        }
        
        let avg_cpu = self.monitoring_data.iter()
            .map(|s| s.cpu_utilization_percent)
            .sum::<f64>() / self.monitoring_data.len() as f64;
        
        if avg_cpu < 50.0 {
            recommendations.push("CPU utilization is low - consider increasing batch size or parallelization".to_string());
        } else if avg_cpu > 90.0 {
            recommendations.push("CPU utilization is very high - consider reducing model complexity".to_string());
        }
        
        recommendations
    }
}

/// Training metrics structure
pub struct TrainingMetrics {
    pub epoch: usize,
    pub duration_ms: u64,
    pub memory_usage_mb: f64,
    pub cpu_utilization: f64,
    pub gpu_utilization: f64,
    pub loss: f64,
    pub accuracy: f64,
}

/// Batch processing metrics
pub struct BatchMetrics {
    pub epoch: usize,
    pub batch: usize,
    pub duration_ms: u64,
    pub memory_usage_mb: f64,
    pub active_tensors: usize,
}

/// Inference performance metrics
pub struct InferenceMetrics {
    pub input_size: usize,
    pub inference_time_ms: u64,
    pub memory_delta_bytes: i64,
    pub throughput_samples_per_sec: f64,
}

/// Memory optimization report
pub struct MemoryOptimizationReport {
    pub peak_memory_mb: f64,
    pub active_tensors: usize,
    pub total_allocations: usize,
    pub optimization_suggestions: Vec<String>,
}

/// Educational example 1: Basic performance monitoring
pub fn example_basic_performance_monitoring() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Example 1: Basic Performance Monitoring ===");
    
    let config = MultiOSIntegrationConfig::new()
        .disable_memory_profiling()
        .monitoring_interval_ms(500);
    
    let mut monitor = MultiOSMLMonitor::new(config);
    monitor.start_training_session();
    
    // Simulate training for 5 epochs
    for epoch in 0..5 {
        let metrics = monitor.monitor_training_epoch(epoch, 5);
        
        println!("Epoch {}: Loss: {:.4}, Accuracy: {:.2}%, Memory: {:.1}MB", 
                 epoch + 1, metrics.loss, metrics.accuracy * 100.0, metrics.memory_usage_mb);
        
        // Simulate batches
        for batch in 0..10 {
            let batch_metrics = monitor.monitor_training_batch(epoch, batch, 10);
            
            if batch % 5 == 0 {
                println!("  Batch {}: Duration: {}ms, Active tensors: {}", 
                         batch, batch_metrics.duration_ms, batch_metrics.active_tensors);
            }
        }
    }
    
    monitor.end_monitoring_session();
    
    println!("✅ Basic monitoring example completed");
    Ok(())
}

/// Educational example 2: Memory optimization
pub fn example_memory_optimization() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Example 2: Memory Optimization Analysis ===");
    
    let config = MultiOSIntegrationConfig::new()
        .enable_memory_profiling()
        .alert_thresholds(AlertThresholds {
            memory_usage_mb: 100.0,  // Lower threshold for demonstration
            ..Default::default()
        });
    
    let mut monitor = MultiOSMLMonitor::new(config);
    monitor.start_training_session();
    
    // Simulate memory-intensive training
    for epoch in 0..3 {
        monitor.monitor_training_epoch(epoch, 3)?;
        
        // Create some tensors to track memory usage
        let _large_tensor = Tensor::random_normal(vec![1000, 1000, 3], 0.0, 1.0);
        let _batch_tensors: Vec<_> = (0..5)
            .map(|_| Tensor::random_normal(vec![128, 128], 0.0, 1.0))
            .collect();
        
        // Generate memory optimization report
        let memory_report = monitor.monitor_memory_optimization();
        
        println!("Memory optimization report generated for epoch {}", epoch + 1);
    }
    
    monitor.end_monitoring_session();
    
    println!("✅ Memory optimization example completed");
    Ok(())
}

/// Educational example 3: Resource utilization monitoring
pub fn example_resource_utilization() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Example 3: Resource Utilization Monitoring ===");
    
    let config = MultiOSIntegrationConfig::new()
        .enable_resource_monitoring()
        .monitoring_interval_ms(200);  // More frequent monitoring
    
    let mut monitor = MultiOSMLMonitor::new(config);
    monitor.start_training_session();
    
    // Simulate inference workloads
    for i in 0..5 {
        let input_sizes = vec![32, 64, 128, 256, 512];
        let input_size = input_sizes[i];
        
        let inference_metrics = monitor.monitor_inference(input_size);
        
        println!("Inference {}: Input size: {}, Time: {}ms, Throughput: {:.1} samples/sec", 
                 i + 1, input_size, inference_metrics.inference_time_ms, inference_metrics.throughput_samples_per_sec);
        
        if inference_metrics.memory_delta_bytes != 0 {
            println!("  Memory delta: {} bytes", inference_metrics.memory_delta_bytes);
        }
    }
    
    // Generate final performance report
    let report = monitor.generate_performance_report();
    println!("\n📊 Performance Report Preview:");
    let lines: Vec<&str> = report.lines().take(10).collect();
    for line in lines {
        println!("  {}", line);
    }
    
    monitor.end_monitoring_session();
    
    println!("✅ Resource utilization example completed");
    Ok(())
}

/// Educational example 4: Complete integration workflow
pub fn example_complete_integration() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Example 4: Complete MultiOS Integration ===");
    
    // Create a more complex monitoring setup
    let config = MultiOSIntegrationConfig::new()
        .report_path("complete_ml_monitoring_report.md")
        .alert_thresholds(AlertThresholds {
            memory_usage_mb: 200.0,
            training_time_seconds: 30.0,
            gpu_utilization_percent: 85.0,
            cpu_utilization_percent: 80.0,
        });
    
    let mut monitor = MultiOSMLMonitor::new(config);
    monitor.start_training_session();
    
    // Simulate a complete ML workflow
    println!("\n🔄 Simulating ML Workflow:");
    
    // 1. Data loading phase
    println!("1. Data loading phase...");
    for i in 0..3 {
        let metrics = monitor.monitor_training_epoch(0, 1);  // Dummy epoch for data loading
        println!("   Data loading {}: Memory: {:.1}MB", i + 1, metrics.memory_usage_mb);
    }
    
    // 2. Training phase
    println!("\n2. Training phase...");
    for epoch in 0..4 {
        let metrics = monitor.monitor_training_epoch(epoch + 1, 4)?;
        
        println!("   Epoch {}: Loss: {:.4}, Accuracy: {:.1}%, CPU: {:.1}%", 
                 epoch + 1, metrics.loss, metrics.accuracy * 100.0, metrics.cpu_utilization);
        
        // Simulate batch processing
        for batch in 0..20 {
            let batch_metrics = monitor.monitor_training_batch(epoch + 1, batch + 1, 20);
            
            if batch % 10 == 0 {
                println!("     Batch {}: {}ms, {}MB", batch + 1, batch_metrics.duration_ms, batch_metrics.memory_usage_mb);
            }
        }
    }
    
    // 3. Evaluation phase
    println!("\n3. Evaluation phase...");
    for i in 0..2 {
        let eval_metrics = monitor.monitor_inference(256);
        println!("   Evaluation {}: {}ms, {:.1} samples/sec", 
                 i + 1, eval_metrics.inference_time_ms, eval_metrics.throughput_samples_per_sec);
    }
    
    // 4. Generate final reports
    println!("\n4. Generating reports...");
    let memory_report = monitor.monitor_memory_optimization();
    let performance_report = monitor.generate_performance_report();
    
    println!("\n📋 Final Summary:");
    println!("   Peak memory usage: {:.1} MB", memory_report.peak_memory_mb);
    println!("   Total allocations: {}", memory_report.total_allocations);
    println!("   Optimization suggestions: {}", memory_report.optimization_suggestions.len());
    
    // Save the report
    monitor.save_performance_report()?;
    
    monitor.end_monitoring_session();
    
    println!("✅ Complete integration example finished");
    println!("📄 Check 'complete_ml_monitoring_report.md' for detailed report");
    
    Ok(())
}

/// Helper function to demonstrate real-world integration patterns
pub fn demonstrate_integration_patterns() {
    println!("\n🎯 MULTIOS INTEGRATION PATTERNS:\n");
    
    println!("1. PERFORMANCE MONITORING INTEGRATION:");
    println!("   • Automatic detection of hardware resources");
    println!("   • Real-time monitoring of CPU/GPU utilization");
    println!("   • Memory usage tracking and optimization");
    println!("   • Performance bottleneck identification");
    
    println!("\n2. SCHEDULING INTEGRATION:");
    println!("   • MultiOS job scheduler integration");
    println!("   • Automatic resource allocation");
    println!("   • Load balancing across cores");
    println!("   • Priority-based task scheduling");
    
    println!("\n3. RESOURCE MANAGEMENT:");
    println!("   • Dynamic memory allocation");
    println!("   • CPU/GPU resource pooling");
    println!("   • Memory-mapped file support");
    println!("   • Automatic garbage collection");
    
    println!("\n4. MONITORING AND ALERTING:");
    println!("   • Real-time performance dashboards");
    println!("   • Configurable alert thresholds");
    println!("   • Historical performance analysis");
    println!("   • Automated optimization recommendations");
    
    println!("\n5. EDUCATIONAL FEATURES:");
    println!("   • Interactive performance visualization");
    println!("   • Step-by-step profiling guides");
    println!("   • Performance comparison tools");
    println!("   • Resource usage optimization tutorials");
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_multios_integration_config() {
        let config = MultiOSIntegrationConfig::new();
        assert!(config.enable_performance_monitoring);
        assert!(config.enable_memory_profiling);
        assert_eq!(config.monitoring_interval, 1000);
    }
    
    #[test]
    fn test_monitoring_snapshot() {
        let snapshot = MonitoringSnapshot {
            timestamp: Instant::now(),
            memory_usage_mb: 100.0,
            cpu_utilization_percent: 50.0,
            gpu_utilization_percent: 75.0,
            active_tensors: 10,
            training_epoch: 0,
            batch_index: 0,
            epoch_duration_ms: 1000,
            batch_duration_ms: 100,
        };
        
        assert_eq!(snapshot.memory_usage_mb, 100.0);
        assert_eq!(snapshot.cpu_utilization_percent, 50.0);
        assert_eq!(snapshot.active_tensors, 10);
    }
    
    #[test]
    fn test_training_metrics() {
        let metrics = TrainingMetrics {
            epoch: 0,
            duration_ms: 5000,
            memory_usage_mb: 150.0,
            cpu_utilization: 60.0,
            gpu_utilization: 80.0,
            loss: 0.5,
            accuracy: 0.8,
        };
        
        assert_eq!(metrics.epoch, 0);
        assert_eq!(metrics.loss, 0.5);
        assert_eq!(metrics.accuracy, 0.8);
    }
}