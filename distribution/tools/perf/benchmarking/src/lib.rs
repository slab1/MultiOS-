//! MultiOS Comprehensive Benchmarking Framework
//! 
//! This framework provides comprehensive benchmarking capabilities for MultiOS,
//! including CPU, memory, file system, network, and system call performance testing.

pub mod cpu;
pub mod memory;
pub mod filesystem;
pub mod network;
pub mod boot_time;
pub mod syscalls;
pub mod utils;
pub mod reporter;

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};

/// Benchmark result structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub name: String,
    pub category: BenchmarkCategory,
    pub duration: Duration,
    pub iterations: u64,
    pub operations_per_second: f64,
    pub throughput: f64,
    pub unit: String,
    pub metadata: std::collections::HashMap<String, String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Benchmark categories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BenchmarkCategory {
    CPU,
    Memory,
    FileSystem,
    Network,
    BootTime,
    Syscalls,
}

/// Benchmark trait for implementing custom benchmarks
pub trait Benchmark: Send + Sync {
    /// Get benchmark name
    fn name(&self) -> &str;
    
    /// Get benchmark category
    fn category(&self) -> BenchmarkCategory;
    
    /// Run benchmark and return result
    fn run(&self, iterations: u64) -> Result<BenchmarkResult, Box<dyn std::error::Error>>;
    
    /// Validate benchmark prerequisites
    fn validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

/// Benchmark runner
pub struct BenchmarkRunner {
    results: Arc<Mutex<Vec<BenchmarkResult>>>,
    verbose: bool,
}

impl BenchmarkRunner {
    /// Create new benchmark runner
    pub fn new(verbose: bool) -> Self {
        Self {
            results: Arc::new(Mutex::new(Vec::new())),
            verbose,
        }
    }
    
    /// Run a single benchmark
    pub fn run_benchmark<T: Benchmark>(&self, benchmark: &T, iterations: u64) -> Result<BenchmarkResult, Box<dyn std::error::Error>> {
        if self.verbose {
            println!("Running benchmark: {}", benchmark.name());
        }
        
        // Validate prerequisites
        benchmark.validate()?;
        
        let start = Instant::now();
        let result = benchmark.run(iterations)?;
        let elapsed = start.elapsed();
        
        if self.verbose {
            println!("Completed benchmark: {} (Duration: {:?})", benchmark.name(), elapsed);
        }
        
        Ok(result)
    }
    
    /// Run multiple benchmarks
    pub fn run_benchmarks<T: Benchmark>(&self, benchmarks: Vec<T>, iterations: u64) -> Result<Vec<BenchmarkResult>, Box<dyn std::error::Error>> {
        let mut results = Vec::new();
        
        for benchmark in benchmarks {
            let result = self.run_benchmark(&benchmark, iterations)?;
            results.push(result);
        }
        
        Ok(results)
    }
    
    /// Get all collected results
    pub fn get_results(&self) -> Vec<BenchmarkResult> {
        self.results.lock().unwrap().clone()
    }
    
    /// Clear results
    pub fn clear_results(&self) {
        self.results.lock().unwrap().clear();
    }
}

/// Benchmark configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkConfig {
    pub iterations: u64,
    pub warmup_iterations: u64,
    pub timeout: Option<Duration>,
    pub batch_size: usize,
    pub verbose: bool,
    pub output_format: OutputFormat,
    pub compare_baseline: bool,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            iterations: 10000,
            warmup_iterations: 100,
            timeout: Some(Duration::from_secs(300)),
            batch_size: 1000,
            verbose: true,
            output_format: OutputFormat::Json,
            compare_baseline: false,
        }
    }
}

/// Output formats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutputFormat {
    Json,
    Csv,
    Human,
    Html,
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            OutputFormat::Json => write!(f, "json"),
            OutputFormat::Csv => write!(f, "csv"),
            OutputFormat::Human => write!(f, "human"),
            OutputFormat::Html => write!(f, "html"),
        }
    }
}

/// System information structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub os_name: String,
    pub os_version: String,
    pub kernel_version: String,
    pub architecture: String,
    pub cpu_model: String,
    pub cpu_cores: usize,
    pub memory_total: u64,
    pub memory_available: u64,
}

impl SystemInfo {
    /// Collect system information
    pub fn collect() -> Result<Self, Box<dyn std::error::Error>> {
        use sysinfo::{System, SystemExt, CpuExt};
        
        let mut sys = System::new_all();
        sys.refresh_all();
        
        Ok(Self {
            os_name: std::env::consts::OS.to_string(),
            os_version: "unknown".to_string(), // Would need platform-specific implementation
            kernel_version: sys.kernel_version().unwrap_or_else(|| "unknown".to_string()),
            architecture: std::env::consts::ARCH.to_string(),
            cpu_model: sys.global_cpu_info().brand().to_string(),
            cpu_cores: sys.physical_core_count().unwrap_or(1),
            memory_total: sys.total_memory(),
            memory_available: sys.available_memory(),
        })
    }
}

/// Performance comparison against baseline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceComparison {
    pub benchmark_name: String,
    pub current_result: BenchmarkResult,
    pub baseline_result: Option<BenchmarkResult>,
    pub percentage_change: f64,
    pub is_improvement: bool,
}

impl PerformanceComparison {
    /// Create comparison from current and baseline results
    pub fn new(current: BenchmarkResult, baseline: Option<BenchmarkResult>) -> Self {
        let percentage_change = if let Some(baseline) = &baseline {
            if baseline.operations_per_second > 0.0 {
                ((current.operations_per_second - baseline.operations_per_second) / baseline.operations_per_second) * 100.0
            } else {
                0.0
            }
        } else {
            0.0
        };
        
        let is_improvement = percentage_change > 0.0;
        
        Self {
            benchmark_name: current.name.clone(),
            current_result: current,
            baseline_result: baseline,
            percentage_change,
            is_improvement,
        }
    }
}