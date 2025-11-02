//! MultiOS Scheduler Performance Profiler
//! 
//! Comprehensive profiling and optimization system for all 4 MultiOS scheduling algorithms:
//! - Round-Robin
//! - Priority-based  
//! - Multi-Level Feedback Queue (MLFQ)
//! - Earliest Deadline First (EDF)
//!
//! Features:
//! 1. Real-time scheduler performance monitoring
//! 2. Context switch overhead measurement
//! 3. CPU utilization and load balancing analysis
//! 4. Priority inversion detection and mitigation
//! 5. Fairness and responsiveness metrics
//! 6. Multi-core scalability testing
//! 7. Workload-specific scheduler selection
//! 8. Interactive scheduler configuration and tuning tools

use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::{AtomicU64, AtomicUsize, AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use tokio::time::{interval, sleep};
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;

use crate::monitoring::*;
use crate::analysis::*;
use crate::tuning::*;
use crate::benchmarking::*;
use crate::visualization::*;
use crate::types::*;

mod monitoring;
mod analysis;
mod tuning;
mod benchmarking;
mod visualization;
mod types;

#[derive(Parser, Debug)]
#[command(name = "scheduler_profiler")]
#[command(about = "MultiOS Scheduler Performance Profiler")]
#[command(version = "1.0.0")]
struct Cli {
    /// Scheduler algorithm to profile
    #[arg(short, long, default_value = "round_robin")]
    algorithm: SchedulerAlgorithm,

    /// Number of CPU cores to simulate/test
    #[arg(short, long, default_value = "8")]
    cores: usize,

    /// Monitoring interval in milliseconds
    #[arg(short, long, default_value = "100")]
    interval_ms: u64,

    /// Enable detailed performance counters
    #[arg(short, long)]
    counters: bool,

    /// Enable real-time visualization
    #[arg(short, long)]
    dashboard: bool,

    /// Output directory for reports and data
    #[arg(short, long, default_value = "./output")]
    output: String,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Run scalability testing across multiple core counts
    Scalability {
        /// Core counts to test (comma-separated)
        cores: String,
        /// Algorithms to test (comma-separated)
        algorithms: String,
    },
    
    /// Interactive scheduler tuning
    Tune {
        /// Target workload type
        #[arg(default_value = "mixed")]
        workload: String,
        /// Target throughput
        #[arg(default_value = "1000")]
        throughput: f64,
    },
    
    /// Performance analysis and reporting
    Analyze {
        /// Input data file
        input: String,
        /// Output report file
        output: String,
    },
    
    /// Start web dashboard
    Dashboard {
        /// Port for web interface
        #[arg(short, long, default_value = "8080")]
        port: u16,
    },
}

/// Main scheduler profiler system
pub struct SchedulerProfiler {
    /// Configuration
    config: ProfilerConfig,
    
    /// Performance monitor
    monitor: Arc<PerformanceMonitor>,
    
    /// Analysis engine
    analyzer: Arc<AnalysisEngine>,
    
    /// Auto-tuning system
    tuner: Arc<AutoTuner>,
    
    /// Benchmarking system
    benchmarker: Arc<Benchmarker>,
    
    /// Visualization system
    visualizer: Arc<VisualizationEngine>,
    
    /// Runtime data storage
    data_store: Arc<RwLock<DataStore>>,
    
    /// Control flags
    running: AtomicBool,
    
    /// Performance statistics
    stats: ProfilerStats,
}

/// Profiler configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfilerConfig {
    pub algorithm: SchedulerAlgorithm,
    pub core_count: usize,
    pub monitoring_interval: Duration,
    pub enable_counters: bool,
    pub enable_dashboard: bool,
    pub output_directory: String,
    pub max_history_samples: usize,
    pub enable_prediction: bool,
    pub enable_auto_tuning: bool,
    pub enable_benchmarking: bool,
}

impl Default for ProfilerConfig {
    fn default() -> Self {
        Self {
            algorithm: SchedulerAlgorithm::RoundRobin,
            core_count: 8,
            monitoring_interval: Duration::from_millis(100),
            enable_counters: true,
            enable_dashboard: true,
            output_directory: "./output".to_string(),
            max_history_samples: 10000,
            enable_prediction: true,
            enable_auto_tuning: true,
            enable_benchmarking: true,
        }
    }
}

/// Profiler runtime statistics
#[derive(Debug, Default)]
pub struct ProfilerStats {
    pub start_time: Instant,
    pub total_samples: AtomicU64,
    pub context_switches: AtomicU64,
    pub scheduling_decisions: AtomicU64,
    pub performance_alerts: AtomicU64,
    pub auto_tuning_actions: AtomicU64,
    pub benchmark_runs: AtomicU64,
    pub total_measurement_time: AtomicU64, // microseconds
}

/// Thread-safe data store for performance metrics
#[derive(Debug)]
pub struct DataStore {
    pub samples: Vec<PerformanceSample>,
    pub context_switches: Vec<ContextSwitchEvent>,
    pub priority_inversions: Vec<PriorityInversionEvent>,
    pub load_balancing_events: Vec<LoadBalanceEvent>,
    pub fairness_metrics: Vec<FairnessMetric>,
    pub responsiveness_metrics: Vec<ResponsivenessMetric>,
}

/// Performance sample with comprehensive metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSample {
    pub timestamp: DateTime<Utc>,
    pub cpu_utilization: Vec<f32>,
    pub scheduling_latency: SchedulingLatency,
    pub context_switch_overhead: ContextSwitchOverhead,
    pub load_balancing_efficiency: f32,
    pub fairness_index: f32,
    pub responsiveness_score: f32,
    pub throughput: f64,
    pub algorithm: SchedulerAlgorithm,
    pub core_count: usize,
}

/// Context switch overhead measurements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextSwitchOverhead {
    pub min_cycles: u64,
    pub max_cycles: u64,
    pub avg_cycles: f64,
    pub min_microseconds: u64,
    pub max_microseconds: u64,
    pub avg_microseconds: f64,
    pub total_switches: u64,
}

/// Scheduling latency metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulingLatency {
    pub min_ns: u64,
    pub max_ns: u64,
    pub avg_ns: f64,
    pub p95_ns: u64,
    pub p99_ns: u64,
}

/// Context switch event detail
#[derive(Debug, Clone)]
pub struct ContextSwitchEvent {
    pub timestamp: Instant,
    pub from_thread: u64,
    pub to_thread: u64,
    pub from_cpu: usize,
    pub to_cpu: usize,
    pub overhead_cycles: u64,
    pub overhead_microseconds: u64,
}

/// Priority inversion detection event
#[derive(Debug, Clone)]
pub struct PriorityInversionEvent {
    pub timestamp: Instant,
    pub low_priority_thread: u64,
    pub high_priority_thread: u64,
    pub lock_held: u64,
    pub inversion_duration: Duration,
    pub resolution_strategy: String,
}

/// Load balancing event
#[derive(Debug, Clone)]
pub struct LoadBalanceEvent {
    pub timestamp: Instant,
    pub source_cpu: usize,
    pub target_cpu: usize,
    pub thread_id: u64,
    pub migration_reason: String,
    pub load_before: u32,
    pub load_after: u32,
}

/// Fairness metric calculation
#[derive(Debug, Clone)]
pub struct FairnessMetric {
    pub timestamp: Instant,
    pub jains_fairness_index: f32,
    pub thread_burst_times: Vec<f32>,
    pub fairness_variance: f32,
}

/// Responsiveness metric
#[derive(Debug, Clone)]
pub struct ResponsivenessMetric {
    pub timestamp: Instant,
    pub interactive_threads: u32,
    pub avg_response_time: Duration,
    pub max_response_time: Duration,
    pub response_time_variance: f32,
}

impl SchedulerProfiler {
    /// Create new scheduler profiler
    pub fn new(config: ProfilerConfig) -> Self {
        let monitor = Arc::new(PerformanceMonitor::new(config.clone()));
        let analyzer = Arc::new(AnalysisEngine::new(config.clone()));
        let tuner = Arc::new(AutoTuner::new(config.clone()));
        let benchmarker = Arc::new(Benchmarker::new(config.clone()));
        let visualizer = Arc::new(VisualizationEngine::new(config.output_directory.clone()));
        let data_store = Arc::new(RwLock::new(DataStore::new()));

        Self {
            config,
            monitor,
            analyzer,
            tuner,
            benchmarker,
            visualizer,
            data_store,
            running: AtomicBool::new(false),
            stats: ProfilerStats::default(),
        }
    }

    /// Start the profiler
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.running.store(true, Ordering::SeqCst);
        
        println!("Starting MultiOS Scheduler Profiler...");
        println!("Algorithm: {:?}", self.config.algorithm);
        println!("Core count: {}", self.config.core_count);
        println!("Monitoring interval: {:?}", self.config.monitoring_interval);

        // Start monitoring loop
        let monitoring_handle = self.start_monitoring_loop().await?;
        
        // Start analysis loop
        let analysis_handle = self.start_analysis_loop().await?;
        
        // Start tuning loop if enabled
        let mut tuning_handles = Vec::new();
        if self.config.enable_auto_tuning {
            tuning_handles.push(self.start_tuning_loop().await?);
        }
        
        // Start benchmarking loop if enabled
        let mut benchmark_handles = Vec::new();
        if self.config.enable_benchmarking {
            benchmark_handles.push(self.start_benchmarking_loop().await?);
        }

        println!("Profiler started successfully. Press Ctrl+C to stop.");
        
        // Wait for all tasks
        tokio::select! {
            _ = monitoring_handle => {},
            _ = analysis_handle => {},
            _ = tokio::signal::ctrl_c() => {
                println!("\nShutting down profiler...");
                self.shutdown().await?;
            },
        }

        Ok(())
    }

    /// Start the main monitoring loop
    async fn start_monitoring_loop(&self) -> Result<JoinHandle<()>, Box<dyn std::error::Error>> {
        let monitor = self.monitor.clone();
        let data_store = self.data_store.clone();
        let config = self.config.clone();
        let stats = &self.stats;
        let running = &self.running;

        let handle = tokio::spawn(async move {
            let mut interval = interval(config.monitoring_interval);
            
            while running.load(Ordering::SeqCst) {
                interval.tick().await;
                
                // Collect performance metrics
                let sample = monitor.collect_sample().await;
                
                if let Ok(sample) = sample {
                    // Store sample
                    {
                        let mut store = data_store.write().unwrap();
                        store.samples.push(sample);
                        
                        // Limit history size
                        if store.samples.len() > config.max_history_samples {
                            store.samples.drain(0..1000);
                        }
                    }
                    
                    stats.total_samples.fetch_add(1, Ordering::SeqCst);
                }
                
                // Analyze for anomalies and alerts
                if config.enable_prediction {
                    monitor.check_anomalies().await;
                }
            }
        });

        Ok(handle)
    }

    /// Start the analysis loop
    async fn start_analysis_loop(&self) -> Result<JoinHandle<()>, Box<dyn std::error::Error>> {
        let analyzer = self.analyzer.clone();
        let data_store = self.data_store.clone();
        let config = self.config.clone();
        let running = &self.running;

        let handle = tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(1));
            
            while running.load(Ordering::SeqCst) {
                interval.tick().await;
                
                // Perform analysis
                let samples = {
                    let store = data_store.read().unwrap();
                    store.samples.clone()
                };
                
                if !samples.is_empty() {
                    // Analyze performance trends
                    let trends = analyzer.analyze_trends(&samples).await;
                    
                    // Detect performance regressions
                    let regressions = analyzer.detect_regressions(&samples).await;
                    
                    // Generate optimization recommendations
                    let recommendations = analyzer.generate_recommendations(&samples).await;
                    
                    if !recommendations.is_empty() {
                        println!("Optimization recommendations generated:");
                        for rec in &recommendations {
                            println!("  - {}", rec);
                        }
                    }
                }
            }
        });

        Ok(handle)
    }

    /// Start the auto-tuning loop
    async fn start_tuning_loop(&self) -> Result<JoinHandle<()>, Box<dyn std::error::Error>> {
        let tuner = self.tuner.clone();
        let data_store = self.data_store.clone();
        let config = self.config.clone();
        let running = &self.running;

        let handle = tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(5));
            
            while running.load(Ordering::SeqCst) {
                interval.tick().await;
                
                let samples = {
                    let store = data_store.read().unwrap();
                    store.samples.clone()
                };
                
                if !samples.is_empty() {
                    tuner.analyze_and_tune(&samples).await;
                }
            }
        });

        Ok(handle)
    }

    /// Start the benchmarking loop
    async fn start_benchmarking_loop(&self) -> Result<JoinHandle<()>, Box<dyn std::error::Error>> {
        let benchmarker = self.benchmarker.clone();
        let config = self.config.clone();
        let running = &self.running;

        let handle = tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(30));
            
            while running.load(Ordering::SeqCst) {
                interval.tick().await;
                
                // Run performance benchmarks
                benchmarker.run_benchmarks().await;
            }
        });

        Ok(handle)
    }

    /// Shutdown the profiler
    pub async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.running.store(false, Ordering::SeqCst);
        
        // Generate final report
        self.generate_report().await?;
        
        // Export data
        self.export_data().await?;
        
        println!("Profiler shutdown complete.");
        Ok(())
    }

    /// Generate comprehensive performance report
    pub async fn generate_report(&self) -> Result<(), Box<dyn std::error::Error>> {
        let store = self.data_store.read().unwrap();
        let report = self.analyzer.generate_report(&store.samples).await?;
        
        let report_path = format!("{}/performance_report.md", self.config.output_directory);
        std::fs::write(&report_path, report)?;
        
        println!("Performance report generated: {}", report_path);
        Ok(())
    }

    /// Export performance data in multiple formats
    pub async fn export_data(&self) -> Result<(), Box<dyn std::error::Error>> {
        let store = self.data_store.read().unwrap();
        
        // Export JSON
        let json_data = serde_json::to_string_pretty(&store.samples)?;
        let json_path = format!("{}/performance_data.json", self.config.output_directory);
        std::fs::write(&json_path, json_data)?;
        
        // Export CSV
        let csv_data = self.visualizer.samples_to_csv(&store.samples)?;
        let csv_path = format!("{}/performance_data.csv", self.config.output_directory);
        std::fs::write(&csv_path, csv_data)?;
        
        // Generate visualization
        let chart_path = format!("{}/performance_chart.png", self.config.output_directory);
        self.visualizer.generate_performance_charts(&store.samples, &chart_path)?;
        
        println!("Data exported to: {}", self.config.output_directory);
        Ok(())
    }

    /// Get current profiler statistics
    pub fn get_stats(&self) -> &ProfilerStats {
        &self.stats
    }

    /// Get current performance data
    pub fn get_current_data(&self) -> Vec<PerformanceSample> {
        let store = self.data_store.read().unwrap();
        store.samples.clone()
    }
}

impl DataStore {
    fn new() -> Self {
        Self {
            samples: Vec::new(),
            context_switches: Vec::new(),
            priority_inversions: Vec::new(),
            load_balancing_events: Vec::new(),
            fairness_metrics: Vec::new(),
            responsiveness_metrics: Vec::new(),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    
    // Create output directory
    std::fs::create_dir_all(&cli.output)?;
    
    let config = ProfilerConfig {
        algorithm: cli.algorithm,
        core_count: cli.cores,
        monitoring_interval: Duration::from_millis(cli.interval_ms),
        enable_counters: cli.counters,
        enable_dashboard: cli.dashboard,
        output_directory: cli.output,
        max_history_samples: 10000,
        enable_prediction: true,
        enable_auto_tuning: true,
        enable_benchmarking: true,
    };

    match cli.command {
        Some(command) => {
            match command {
                Commands::Scalability { cores, algorithms } => {
                    let core_counts: Vec<usize> = cores.split(',')
                        .map(|s| s.trim().parse().unwrap())
                        .collect();
                    let algos: Vec<SchedulerAlgorithm> = algorithms.split(',')
                        .map(|s| s.trim().parse().unwrap_or(SchedulerAlgorithm::RoundRobin))
                        .collect();
                    
                    run_scalability_test(core_counts, algos, cli.output).await?;
                },
                Commands::Tune { workload, throughput } => {
                    run_interactive_tuning(workload, throughput, cli.output).await?;
                },
                Commands::Analyze { input, output } => {
                    run_performance_analysis(input, output).await?;
                },
                Commands::Dashboard { port } => {
                    run_dashboard_server(port).await?;
                },
            }
        },
        None => {
            // Start normal profiling session
            let profiler = SchedulerProfiler::new(config);
            profiler.start().await?;
        },
    }

    Ok(())
}

async fn run_scalability_test(
    core_counts: Vec<usize>,
    algorithms: Vec<SchedulerAlgorithm>,
    output_dir: String,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Running scalability test...");
    println!("Core counts: {:?}", core_counts);
    println!("Algorithms: {:?}", algorithms);

    // Implementation would run benchmarks across different core counts and algorithms
    // Generate comparison charts and reports
    
    Ok(())
}

async fn run_interactive_tuning(
    workload: String,
    target_throughput: f64,
    output_dir: String,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting interactive tuning...");
    println!("Workload: {}", workload);
    println!("Target throughput: {}", target_throughput);

    // Implementation would analyze workload and suggest optimal scheduler parameters
    
    Ok(())
}

async fn run_performance_analysis(
    input_file: String,
    output_file: String,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Analyzing performance data...");
    println!("Input: {}", input_file);
    println!("Output: {}", output_file);

    // Implementation would read performance data and generate analysis report
    
    Ok(())
}

async fn run_dashboard_server(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting dashboard server on port {}...", port);

    // Implementation would start web dashboard for real-time visualization
    
    Ok(())
}