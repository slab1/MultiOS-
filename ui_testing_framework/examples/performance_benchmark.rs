//! Performance benchmarking example showing UI performance testing
//! 
//! This example demonstrates:
//! - Measuring frame rates and render times
//! - Monitoring memory usage during UI operations
//! - Profiling event handling performance
//! - Generating performance reports

use multios_ui_testing::*;
use tokio;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting Performance Benchmarking Example");
    
    // Initialize the performance benchmark
    let mut benchmark = PerformanceBenchmark::new().await?;
    
    // Configure benchmark parameters
    let config = BenchmarkConfig::new()
        .with_duration(Duration::from_secs(30))
        .with_sampling_rate(60) // 60 Hz sampling
        .with_memory_monitoring(true)
        .with_cpu_monitoring(true);
    
    // Create performance test scenario
    let scenario = PerformanceTestScenario::new("ui_interaction_benchmark")
        .with_phase(PerformancePhase::Setup, "Initialize UI components")
        .with_phase(PerformancePhase::Interaction, "Simulate user interactions")
        .with_phase(PerformancePhase::Teardown, "Clean up resources");
    
    // Run frame rate benchmark
    println!("Running frame rate benchmark...");
    let frame_benchmark = benchmark.measure_frame_rate(
        "main_window",
        Duration::from_secs(10)
    ).await?;
    
    println!("Frame Rate Results:");
    println!("  Average FPS: {:.2}", frame_benchmark.average_fps);
    println!("  Min FPS: {:.2}", frame_benchmark.min_fps);
    println!("  Max FPS: {:.2}", frame_benchmark.max_fps);
    println!("  Frame drops: {}", frame_benchmark.frame_drops);
    
    // Run render time analysis
    println!("\nRunning render time analysis...");
    let render_benchmark = benchmark.measure_render_time(
        "login_screen",
        100 // Measure 100 render cycles
    ).await?;
    
    println!("Render Time Results:");
    println!("  Average render time: {:.2}ms", render_benchmark.average_time_ms);
    println!("  Min render time: {:.2}ms", render_benchmark.min_time_ms);
    println!("  Max render time: {:.2}ms", render_benchmark.max_time_ms);
    println!("  95th percentile: {:.2}ms", render_benchmark.p95_time_ms);
    
    // Run memory usage analysis
    println!("\nRunning memory usage analysis...");
    let memory_benchmark = benchmark.measure_memory_usage(
        "application_session",
        Duration::from_secs(20)
    ).await?;
    
    println!("Memory Usage Results:");
    println!("  Peak memory: {:.2}MB", memory_benchmark.peak_memory_mb);
    println!("  Average memory: {:.2}MB", memory_benchmark.average_memory_mb);
    println!("  Memory leaks detected: {}", memory_benchmark.leak_count);
    
    // Run event handling benchmark
    println!("\nRunning event handling benchmark...");
    let event_benchmark = benchmark.measure_event_handling(
        "ui_event_queue",
        1000 // Process 1000 events
    ).await?;
    
    println!("Event Handling Results:");
    println!("  Average event latency: {:.2}ms", event_benchmark.average_latency_ms);
    println!("  Event processing rate: {:.0}/sec", event_benchmark.events_per_second);
    println!("  Missed events: {}", event_benchmark.missed_events);
    
    // Generate comprehensive performance report
    let report = benchmark.generate_report(vec![
        frame_benchmark,
        render_benchmark,
        memory_benchmark,
        event_benchmark,
    ]).await?;
    
    // Save report to file
    std::fs::write("test_data/reports/performance_report.html", report)?;
    println!("\nPerformance report saved to test_data/reports/performance_report.html");
    
    // Clean up
    benchmark.shutdown().await?;
    
    Ok(())
}