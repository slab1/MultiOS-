//! Basic Edge Service Example
//! MultiOS Edge Computing Demonstrations

use edge_computing_demos::shared_utils::edge_config::{
    create_default_config, EdgeOrchestrator, EdgeDevice, DeviceCapabilities, 
    DeviceStatus, create_sample_device, EdgeTask, WorkloadType, ResourceRequirements
};
use edge_computing_demos::shared_utils::performance_benchmark::{PerformanceBenchmark, BenchmarkMetrics};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting MultiOS Edge Computing Basic Service Demo");
    println!("=" * 60);
    
    // Initialize edge orchestrator
    let config = create_default_config();
    let orchestrator = Arc::new(Mutex::new(EdgeOrchestrator::new(config)));
    
    // Register edge devices
    let device1 = create_sample_device("edge-gateway-001", "IoT Gateway");
    let device2 = create_sample_device("edge-server-002", "Edge Server");
    let device3 = create_sample_device("edge-ai-003", "AI Edge Device");
    
    orchestrator.lock().await.register_device(device1).await?;
    orchestrator.lock().await.register_device(device2).await?;
    orchestrator.lock().await.register_device(device3).await?;
    
    println!("✅ Registered 3 edge devices");
    
    // Initialize performance benchmark
    let mut benchmark = PerformanceBenchmark::new(1000);
    benchmark.start_benchmark();
    
    // Submit various workloads
    let workloads = create_sample_workloads();
    for workload in workloads {
        orchestrator.lock().await.submit_task(workload).await?;
        benchmark.record_operation();
    }
    
    println!("📊 Submitted {} workloads for processing", workloads.len());
    
    // Monitor performance while processing
    let performance_handle = {
        let orchestrator = orchestrator.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(5));
            loop {
                interval.tick().await;
                let orchestrator = orchestrator.lock().await;
                let metrics = orchestrator.get_performance_stats().await;
                
                println!("📈 Performance Stats:");
                println!("   Total devices: {}", metrics.len());
                println!("   Average CPU usage: {:.1}%", 
                    metrics.values().map(|m| m.cpu_usage).sum::<f32>() / metrics.len() as f32);
                println!("   Total throughput: {:.1} ops/sec", 
                    metrics.values().map(|m| m.throughput_ops_per_sec).sum::<f32>());
                
                if metrics.len() > 0 {
                    println!("   System efficiency: {:.1}%", 
                        metrics.values().map(|m| m.power_efficiency_ops_per_watt).sum::<f32>() / metrics.len() as f32);
                }
            }
        })
    };
    
    // Process workloads
    let processing_handle = {
        let orchestrator = orchestrator.clone();
        tokio::spawn(async move {
            loop {
                // Simulate workload processing
                let start = std::time::Instant::now();
                
                // Get best device for next workload
                // This is a simplified version - in real implementation,
                // this would select from actual queued workloads
                
                tokio::time::sleep(Duration::from_millis(100)).await;
                
                let processing_time = start.elapsed();
                if processing_time > Duration::from_millis(500) {
                    println!("⚠️  High processing time detected: {:?}", processing_time);
                }
            }
        })
    };
    
    // Run for a while to demonstrate functionality
    println!("⏱️  Running demonstration for 30 seconds...");
    tokio::time::sleep(Duration::from_secs(30)).await;
    
    // Get final performance metrics
    if let Some(final_metrics) = benchmark.get_comprehensive_metrics() {
        println!("\n🎯 Final Performance Summary:");
        println!("   Average latency: {:.2}ms", final_metrics.avg_latency_ms);
        println!("   95th percentile latency: {:.2}ms", final_metrics.p95_latency_ms);
        println!("   Throughput: {:.2} ops/sec", final_metrics.throughput_ops_per_sec);
        println!("   Success rate: {:.2}%", final_metrics.success_rate_percent);
        println!("   Error rate: {:.2}%", final_metrics.error_rate_percent);
    }
    
    // Shutdown
    performance_handle.abort();
    processing_handle.abort();
    
    println!("\n✅ Basic Edge Service Demo completed successfully!");
    println!("💡 Next steps: Try the other demonstration programs:");
    println!("   cargo run --bin edge_ai_inference_demo");
    println!("   cargo run --bin video_processing_demo");
    println!("   cargo run --bin predictive_maintenance_demo");
    
    Ok(())
}

fn create_sample_workloads() -> Vec<EdgeTask> {
    vec![
        EdgeTask {
            task_id: "ai-inference-001".to_string(),
            workload_type: WorkloadType::AiInference,
            priority: 8,
            estimated_duration_ms: 2000,
            resource_requirements: ResourceRequirements {
                min_cpu_cores: 2,
                min_memory_mb: 1024,
                requires_gpu: true,
                requires_tensor_accel: true,
            },
            data_size_mb: 50.0,
        },
        EdgeTask {
            task_id: "video-processing-001".to_string(),
            workload_type: WorkloadType::VideoProcessing,
            priority: 7,
            estimated_duration_ms: 5000,
            resource_requirements: ResourceRequirements {
                min_cpu_cores: 4,
                min_memory_mb: 2048,
                requires_gpu: true,
                requires_tensor_accel: false,
            },
            data_size_mb: 100.0,
        },
        EdgeTask {
            task_id: "sensor-data-001".to_string(),
            workload_type: WorkloadType::SensorDataProcessing,
            priority: 5,
            estimated_duration_ms: 1000,
            resource_requirements: ResourceRequirements {
                min_cpu_cores: 1,
                min_memory_mb: 512,
                requires_gpu: false,
                requires_tensor_accel: false,
            },
            data_size_mb: 10.0,
        },
        EdgeTask {
            task_id: "maintenance-001".to_string(),
            workload_type: WorkloadType::PredictiveMaintenance,
            priority: 6,
            estimated_duration_ms: 3000,
            resource_requirements: ResourceRequirements {
                min_cpu_cores: 2,
                min_memory_mb: 1024,
                requires_gpu: false,
                requires_tensor_accel: false,
            },
            data_size_mb: 25.0,
        },
        EdgeTask {
            task_id: "smart-city-001".to_string(),
            workload_type: WorkloadType::SmartCityMonitoring,
            priority: 4,
            estimated_duration_ms: 1500,
            resource_requirements: ResourceRequirements {
                min_cpu_cores: 2,
                min_memory_mb: 1024,
                requires_gpu: false,
                requires_tensor_accel: false,
            },
            data_size_mb: 75.0,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_edge_service() {
        let result = main().await;
        assert!(result.is_ok());
    }
}