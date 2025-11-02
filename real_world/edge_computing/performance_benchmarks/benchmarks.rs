//! Performance Benchmarks for Edge Computing Demonstrations
//! MultiOS Edge Computing Demonstrations

use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant, SystemTime};
use serde::{Deserialize, Serialize};
use tokio::sync::{RwLock, mpsc};

/// Benchmark categories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BenchmarkCategory {
    Latency,
    Throughput,
    ResourceUtilization,
    Scalability,
    Reliability,
    EnergyEfficiency,
    CostEffectiveness,
    NetworkPerformance,
    StoragePerformance,
    Security,
}

/// Benchmark suite
#[derive(Debug, Clone)]
pub struct BenchmarkSuite {
    pub suite_id: String,
    pub suite_name: String,
    pub description: String,
    pub category: BenchmarkCategory,
    pub benchmarks: Vec<IndividualBenchmark>,
    pub target_hardware: HardwareProfile,
    pub expected_results: ExpectedSuiteResults,
}

/// Individual benchmark test
#[derive(Debug, Clone)]
pub struct IndividualBenchmark {
    pub benchmark_id: String,
    pub benchmark_name: String,
    pub description: String,
    pub test_function: BenchmarkTestFunction,
    pub iterations: u32,
    pub warmup_iterations: u32,
    pub timeout: Duration,
    pub resource_requirements: BenchmarkResourceRequirements,
    pub metrics_to_collect: Vec<BenchmarkMetric>,
    pub expected_performance: ExpectedBenchmarkPerformance,
}

/// Test function signature
pub type BenchmarkTestFunction = fn(&BenchmarkContext) -> std::pin::Pin<Box<dyn std::future::Future<Output = std::result::Result<BenchmarkResult, BenchmarkError>> + Send>>;

/// Benchmark context
#[derive(Debug, Clone)]
pub struct BenchmarkContext {
    pub test_parameters: TestParameters,
    pub hardware_profile: HardwareProfile,
    pub network_config: NetworkConfiguration,
    pub monitoring_enabled: bool,
}

/// Test parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestParameters {
    pub workload_size: WorkloadSize,
    pub concurrency_level: u32,
    pub data_size_mb: f32,
    pub request_count: u32,
    pub duration_seconds: u32,
}

/// Hardware profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareProfile {
    pub cpu_model: String,
    pub cpu_cores_physical: usize,
    pub cpu_cores_logical: usize,
    pub memory_gb: u64,
    pub storage_type: StorageType,
    pub storage_gb: u64,
    pub gpu_model: Option<String>,
    pub gpu_memory_gb: Option<u32>,
    pub network_interface: String,
    pub network_speed_gbps: f32,
}

/// Storage types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageType {
    Ssd,
    Nvme,
    Hdd,
    Emmc,
}

/// Network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfiguration {
    pub bandwidth_mbps: f32,
    pub latency_ms: f32,
    pub packet_loss_percent: f32,
    pub jitter_ms: f32,
    pub connection_type: ConnectionType,
}

/// Connection types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConnectionType {
    Ethernet,
    Wifi4,
    Wifi5,
    Wifi6,
    Lte,
    FiveG,
}

/// Benchmark result
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub benchmark_id: String,
    pub execution_time: Duration,
    pub metrics: BenchmarkMetrics,
    pub raw_data: Vec<RawMetric>,
    pub passed_criteria: bool,
    pub performance_score: f32,
}

/// Benchmark metrics
#[derive(Debug, Clone)]
pub struct BenchmarkMetrics {
    pub latency_ms: LatencyMetrics,
    pub throughput: ThroughputMetrics,
    pub resource_usage: ResourceUsageMetrics,
    pub quality_metrics: QualityMetrics,
}

/// Latency metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyMetrics {
    pub min_ms: f32,
    pub max_ms: f32,
    pub avg_ms: f32,
    pub median_ms: f32,
    pub p95_ms: f32,
    pub p99_ms: f32,
    pub p999_ms: f32,
    pub standard_deviation_ms: f32,
}

/// Throughput metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThroughputMetrics {
    pub requests_per_second: f32,
    pub bits_per_second: f64,
    pub operations_per_second: f32,
    pub peak_throughput: f32,
}

/// Resource usage metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsageMetrics {
    pub cpu_usage_percent: f32,
    pub memory_usage_percent: f32,
    pub memory_usage_mb: u64,
    pub disk_usage_percent: f32,
    pub disk_iops: u32,
    pub network_usage_percent: f32,
    pub power_consumption_watts: f32,
}

/// Quality metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    pub error_rate_percent: f32,
    pub availability_percent: f32,
    pub success_rate_percent: f32,
    pub data_integrity_score: f32,
    pub reliability_score: f32,
}

/// Raw metric data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawMetric {
    pub timestamp: SystemTime,
    pub metric_name: String,
    pub value: f32,
    pub unit: String,
    pub sample_id: u32,
}

/// Benchmark error types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BenchmarkError {
    Timeout,
    ResourceExhausted,
    NetworkError,
    ConfigurationError,
    HardwareIncompatible,
    Custom { message: String },
}

/// Resource requirements for benchmarks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResourceRequirements {
    pub min_cpu_cores: usize,
    pub min_memory_gb: u64,
    pub min_storage_gb: u64,
    pub required_gpu: bool,
    pub min_network_mbps: u32,
    pub special_requirements: Vec<String>,
}

/// Metrics to collect during benchmarking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BenchmarkMetric {
    Latency,
    Throughput,
    CpuUsage,
    MemoryUsage,
    DiskUsage,
    NetworkUsage,
    PowerConsumption,
    ErrorRate,
    Availability,
    Custom { metric_name: String },
}

/// Expected performance thresholds
#[derive(Debug, Clone)]
pub struct ExpectedBenchmarkPerformance {
    pub minimum_acceptable: PerformanceThresholds,
    pub target_performance: PerformanceThresholds,
    pub excellent_performance: PerformanceThresholds,
}

/// Performance threshold definitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceThresholds {
    pub max_latency_ms: f32,
    pub min_throughput_ops_per_sec: f32,
    pub max_cpu_usage_percent: f32,
    pub max_memory_usage_percent: f32,
    pub max_error_rate_percent: f32,
    pub min_availability_percent: f32,
}

/// Workload sizes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkloadSize {
    Micro,
    Small,
    Medium,
    Large,
    ExtraLarge,
    Custom { requests_per_second: u32 },
}

/// Expected suite results
#[derive(Debug, Clone)]
pub struct ExpectedSuiteResults {
    pub overall_score: f32,
    pub category_scores: HashMap<BenchmarkCategory, f32>,
    pub recommendations: Vec<PerformanceRecommendation>,
    pub comparisons: Vec<BenchmarkComparison>,
}

/// Performance recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceRecommendation {
    pub recommendation_id: String,
    pub category: PerformanceCategory,
    pub priority: RecommendationPriority,
    pub title: String,
    pub description: String,
    pub expected_improvement_percent: f32,
    pub implementation_effort: ImplementationEffort,
}

/// Performance categories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerformanceCategory {
    HardwareUpgrade,
    SoftwareOptimization,
    ConfigurationTuning,
    ArchitectureChange,
    NetworkImprovement,
    AlgorithmOptimization,
}

/// Recommendation priorities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationPriority {
    Critical,
    High,
    Medium,
    Low,
}

/// Implementation effort levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImplementationEffort {
    Quick,
    Medium,
    Complex,
}

/// Benchmark comparisons
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkComparison {
    pub comparison_name: String,
    pub baseline_name: String,
    pub current_name: String,
    pub metrics_compared: Vec<String>,
    pub improvement_percentage: f32,
    pub winner: ComparisonWinner,
}

/// Comparison winners
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComparisonWinner {
    Baseline,
    Current,
    Tie,
}

/// Performance monitoring during benchmarks
#[derive(Debug, Clone)]
pub struct PerformanceMonitor {
    pub monitoring_interval: Duration,
    pub metrics_buffer: VecDeque<RealTimeMetric>,
    pub alert_thresholds: AlertThresholds,
    pub monitoring_active: bool,
}

/// Real-time metric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealTimeMetric {
    pub timestamp: SystemTime,
    pub metric_type: String,
    pub value: f32,
    pub unit: String,
    pub node_id: Option<String>,
}

/// Alert thresholds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertThresholds {
    pub max_cpu_usage: f32,
    pub max_memory_usage: f32,
    pub max_temperature_celsius: f32,
    pub min_performance_score: f32,
}

/// Benchmark runner
pub struct BenchmarkRunner {
    pub suites: HashMap<String, BenchmarkSuite>,
    pub execution_context: ExecutionContext,
    pub performance_monitor: Arc<RwLock<PerformanceMonitor>>,
    pub results_storage: Arc<RwLock<BenchmarkResultsStorage>>,
    pub parallel_execution: bool,
}

/// Execution context
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    pub execution_id: String,
    pub start_time: SystemTime,
    pub environment: ExecutionEnvironment,
    pub constraints: ExecutionConstraints,
}

/// Execution environment
#[derive(Debug, Clone)]
pub struct ExecutionEnvironment {
    pub os_version: String,
    pub kernel_version: String,
    pub runtime_version: String,
    pub container_runtime: Option<String>,
    pub virtualization: Option<String>,
}

/// Execution constraints
#[derive(Debug, Clone)]
pub struct ExecutionConstraints {
    pub max_execution_time: Duration,
    pub resource_limits: BenchmarkResourceRequirements,
    pub network_isolation: bool,
    pub clean_environment: bool,
}

/// Benchmark results storage
#[derive(Debug, Clone)]
pub struct BenchmarkResultsStorage {
    pub all_results: HashMap<String, Vec<BenchmarkExecution>>,
    pub historical_data: Vec<BenchmarkHistory>,
    pub aggregation_stats: AggregationStatistics,
}

/// Individual benchmark execution
#[derive(Debug, Clone)]
pub struct BenchmarkExecution {
    pub execution_id: String,
    pub benchmark_id: String,
    pub start_time: SystemTime,
    pub end_time: SystemTime,
    pub result: BenchmarkResult,
    pub environment: ExecutionEnvironment,
    pub metadata: ExecutionMetadata,
}

/// Execution metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionMetadata {
    pub executor: String,
    pub test_parameters: TestParameters,
    pub notes: Option<String>,
    pub tags: Vec<String>,
}

/// Historical benchmark data
#[derive(Debug, Clone)]
pub struct BenchmarkHistory {
    pub benchmark_id: String,
    pub execution_history: Vec<BenchmarkExecution>,
    pub trend_analysis: TrendAnalysis,
}

/// Trend analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendAnalysis {
    pub performance_trend: TrendDirection,
    pub trend_strength: f32,
    pub seasonality_detected: bool,
    pub predicted_next_performance: Option<f32>,
}

/// Trend directions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrendDirection {
    Improving,
    Declining,
    Stable,
    Volatile,
}

/// Aggregation statistics
#[derive(Debug, Clone)]
pub struct AggregationStatistics {
    pub total_executions: u64,
    pub avg_execution_time: Duration,
    pub success_rate_percent: f32,
    pub most_common_failures: Vec<String>,
}

/// Create comprehensive edge computing benchmark suite
pub fn create_edge_computing_benchmarks() -> Vec<BenchmarkSuite> {
    let mut suites = Vec::new();

    // Edge AI Inference Benchmark Suite
    suites.push(BenchmarkSuite {
        suite_id: "edge-ai-inference".to_string(),
        suite_name: "Edge AI Inference Performance".to_string(),
        description: "Comprehensive benchmarks for AI inference performance at the edge",
        category: BenchmarkCategory::Throughput,
        target_hardware: HardwareProfile {
            cpu_model: "ARM Cortex-A78".to_string(),
            cpu_cores_physical: 4,
            cpu_cores_logical: 8,
            memory_gb: 8,
            storage_type: StorageType::Nvme,
            storage_gb: 256,
            gpu_model: Some("ARM Mali-G78 MP12".to_string()),
            gpu_memory_gb: Some(12),
            network_interface: "Ethernet".to_string(),
            network_speed_gbps: 1.0,
        },
        benchmarks: vec![
            IndividualBenchmark {
                benchmark_id: "classification-throughput".to_string(),
                benchmark_name: "Image Classification Throughput".to_string(),
                description: "Benchmark image classification model inference throughput",
                test_function: benchmark_image_classification_throughput,
                iterations: 1000,
                warmup_iterations: 100,
                timeout: Duration::from_secs(300),
                resource_requirements: BenchmarkResourceRequirements {
                    min_cpu_cores: 2,
                    min_memory_gb: 4,
                    min_storage_gb: 32,
                    required_gpu: false,
                    min_network_mbps: 100,
                    special_requirements: vec!["TensorFlow Lite support".to_string()],
                },
                metrics_to_collect: vec![
                    BenchmarkMetric::Throughput,
                    BenchmarkMetric::Latency,
                    BenchmarkMetric::CpuUsage,
                    BenchmarkMetric::MemoryUsage,
                ],
                expected_performance: ExpectedBenchmarkPerformance {
                    minimum_acceptable: PerformanceThresholds {
                        max_latency_ms: 100.0,
                        min_throughput_ops_per_sec: 5.0,
                        max_cpu_usage_percent: 90.0,
                        max_memory_usage_percent: 80.0,
                        max_error_rate_percent: 1.0,
                        min_availability_percent: 95.0,
                    },
                    target_performance: PerformanceThresholds {
                        max_latency_ms: 20.0,
                        min_throughput_ops_per_sec: 50.0,
                        max_cpu_usage_percent: 70.0,
                        max_memory_usage_percent: 60.0,
                        max_error_rate_percent: 0.1,
                        min_availability_percent: 99.0,
                    },
                    excellent_performance: PerformanceThresholds {
                        max_latency_ms: 5.0,
                        min_throughput_ops_per_sec: 200.0,
                        max_cpu_usage_percent: 50.0,
                        max_memory_usage_percent: 40.0,
                        max_error_rate_percent: 0.01,
                        min_availability_percent: 99.9,
                    },
                },
            }
        ],
        expected_results: ExpectedSuiteResults {
            overall_score: 85.0,
            category_scores: HashMap::from([
                (BenchmarkCategory::Throughput, 88.0),
                (BenchmarkCategory::Latency, 85.0),
                (BenchmarkCategory::ResourceUtilization, 82.0),
            ]),
            recommendations: vec![
                PerformanceRecommendation {
                    recommendation_id: "optimize-model".to_string(),
                    category: PerformanceCategory::AlgorithmOptimization,
                    priority: RecommendationPriority::High,
                    title: "Optimize Model for Edge Deployment".to_string(),
                    description: "Apply quantization and pruning to reduce model size and improve inference speed",
                    expected_improvement_percent: 30.0,
                    implementation_effort: ImplementationEffort::Medium,
                }
            ],
            comparisons: vec![
                BenchmarkComparison {
                    comparison_name: "Edge vs Cloud".to_string(),
                    baseline_name: "Cloud GPU".to_string(),
                    current_name: "Edge Device".to_string(),
                    metrics_compared: vec!["latency".to_string(), "throughput".to_string()],
                    improvement_percentage: 250.0,
                    winner: ComparisonWinner::Current,
                }
            ],
        },
    });

    // Edge Network Performance Benchmark Suite
    suites.push(BenchmarkSuite {
        suite_id: "edge-network-performance".to_string(),
        suite_name: "Edge Network Performance".to_string(),
        description: "Network performance benchmarks for edge computing scenarios",
        category: BenchmarkCategory::NetworkPerformance,
        target_hardware: HardwareProfile {
            cpu_model: "ARM Cortex-A53".to_string(),
            cpu_cores_physical: 4,
            cpu_cores_logical: 4,
            memory_gb: 4,
            storage_type: StorageType::Emmc,
            storage_gb: 64,
            gpu_model: None,
            network_interface: "WiFi 6".to_string(),
            network_speed_gbps: 0.6,
        },
        benchmarks: vec![
            IndividualBenchmark {
                benchmark_id: "network-latency".to_string(),
                benchmark_name: "Network Latency Benchmark".to_string(),
                description: "Measure network latency between edge nodes",
                test_function: benchmark_network_latency,
                iterations: 10000,
                warmup_iterations: 1000,
                timeout: Duration::from_secs(600),
                resource_requirements: BenchmarkResourceRequirements {
                    min_cpu_cores: 1,
                    min_memory_gb: 1,
                    min_storage_gb: 1,
                    required_gpu: false,
                    min_network_mbps: 10,
                    special_requirements: vec!["Network connectivity".to_string()],
                },
                metrics_to_collect: vec![
                    BenchmarkMetric::Latency,
                    BenchmarkMetric::NetworkUsage,
                    BenchmarkMetric::Availability,
                ],
                expected_performance: ExpectedBenchmarkPerformance {
                    minimum_acceptable: PerformanceThresholds {
                        max_latency_ms: 50.0,
                        min_throughput_ops_per_sec: 1000.0,
                        max_cpu_usage_percent: 50.0,
                        max_memory_usage_percent: 50.0,
                        max_error_rate_percent: 5.0,
                        min_availability_percent: 90.0,
                    },
                    target_performance: PerformanceThresholds {
                        max_latency_ms: 10.0,
                        min_throughput_ops_per_sec: 10000.0,
                        max_cpu_usage_percent: 30.0,
                        max_memory_usage_percent: 30.0,
                        max_error_rate_percent: 0.5,
                        min_availability_percent: 99.0,
                    },
                    excellent_performance: PerformanceThresholds {
                        max_latency_ms: 1.0,
                        min_throughput_ops_per_sec: 100000.0,
                        max_cpu_usage_percent: 20.0,
                        max_memory_usage_percent: 20.0,
                        max_error_rate_percent: 0.1,
                        min_availability_percent: 99.9,
                    },
                },
            }
        ],
        expected_results: ExpectedSuiteResults {
            overall_score: 78.0,
            category_scores: HashMap::from([
                (BenchmarkCategory::NetworkPerformance, 82.0),
                (BenchmarkCategory::Latency, 75.0),
                (BenchmarkCategory::Reliability, 80.0),
            ]),
            recommendations: vec![
                PerformanceRecommendation {
                    recommendation_id: "network-optimization".to_string(),
                    category: PerformanceCategory::NetworkImprovement,
                    priority: RecommendationPriority::Medium,
                    title: "Optimize Network Configuration".to_string(),
                    description: "Implement QoS and traffic shaping for critical workloads",
                    expected_improvement_percent: 20.0,
                    implementation_effort: ImplementationEffort::Medium,
                }
            ],
            comparisons: vec![],
        },
    });

    suites
}

/// Sample benchmark test function implementations
pub async fn benchmark_image_classification_throughput(
    context: &BenchmarkContext,
) -> Result<BenchmarkResult, BenchmarkError> {
    let start_time = Instant::now();
    
    // Simulate image classification benchmark
    let mut latencies = Vec::new();
    let mut throughput_count = 0;
    let mut errors = 0;
    
    for i in 0..context.test_parameters.request_count {
        let request_start = Instant::now();
        
        // Simulate AI inference processing
        tokio::time::sleep(Duration::from_millis((rand::random::<u32>() % 50 + 10) as u64)).await;
        
        let request_duration = request_start.elapsed();
        latencies.push(request_duration.as_secs_f32() * 1000.0);
        
        if rand::random::<f32>() < 0.98 {
            throughput_count += 1;
        } else {
            errors += 1;
        }
        
        // Simulate resource usage
        if i % 100 == 0 {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }
    
    let execution_time = start_time.elapsed();
    
    // Calculate metrics
    latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let avg_latency = latencies.iter().sum::<f32>() / latencies.len() as f32;
    let p95_index = (latencies.len() as f32 * 0.95) as usize;
    let p99_index = (latencies.len() as f32 * 0.99) as usize;
    
    let result = BenchmarkResult {
        benchmark_id: "classification-throughput".to_string(),
        execution_time,
        metrics: BenchmarkMetrics {
            latency_ms: LatencyMetrics {
                min_ms: latencies.first().copied().unwrap_or(0.0),
                max_ms: latencies.last().copied().unwrap_or(0.0),
                avg_ms: avg_latency,
                median_ms: latencies[latencies.len() / 2],
                p95_ms: latencies[std::cmp::min(p95_index, latencies.len() - 1)],
                p99_ms: latencies[std::cmp::min(p99_index, latencies.len() - 1)],
                p999_ms: latencies.last().copied().unwrap_or(0.0),
                standard_deviation_ms: calculate_std_dev(&latencies),
            },
            throughput: ThroughputMetrics {
                requests_per_second: throughput_count as f32 / execution_time.as_secs_f32(),
                bits_per_second: 0.0, // Would calculate based on payload size
                operations_per_second: throughput_count as f32 / execution_time.as_secs_f32(),
                peak_throughput: 0.0, // Would track peak during execution
            },
            resource_usage: ResourceUsageMetrics {
                cpu_usage_percent: 65.0, // Simulated
                memory_usage_percent: 45.0,
                memory_usage_mb: 2048,
                disk_usage_percent: 10.0,
                disk_iops: 100,
                network_usage_percent: 25.0,
                power_consumption_watts: 15.0,
            },
            quality_metrics: QualityMetrics {
                error_rate_percent: (errors as f32 / context.test_parameters.request_count as f32) * 100.0,
                availability_percent: 99.5,
                success_rate_percent: ((context.test_parameters.request_count - errors) as f32 / context.test_parameters.request_count as f32) * 100.0,
                data_integrity_score: 100.0,
                reliability_score: 99.5,
            },
        },
        raw_data: Vec::new(),
        passed_criteria: true,
        performance_score: 85.0,
    };
    
    Ok(result)
}

pub async fn benchmark_network_latency(
    context: &BenchmarkContext,
) -> Result<BenchmarkResult, BenchmarkError> {
    let start_time = Instant::now();
    
    // Simulate network latency measurement
    let mut latencies = Vec::new();
    let mut successful_pings = 0;
    let mut failed_pings = 0;
    
    for i in 0..context.test_parameters.iterations {
        let ping_start = Instant::now();
        
        // Simulate network ping
        tokio::time::sleep(Duration::from_millis((rand::random::<u32>() % 20 + 1) as u64)).await;
        
        let ping_duration = ping_start.elapsed();
        latencies.push(ping_duration.as_secs_f32() * 1000.0);
        
        if rand::random::<f32>() < 0.95 {
            successful_pings += 1;
        } else {
            failed_pings += 1;
        }
    }
    
    let execution_time = start_time.elapsed();
    
    latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());
    
    let result = BenchmarkResult {
        benchmark_id: "network-latency".to_string(),
        execution_time,
        metrics: BenchmarkMetrics {
            latency_ms: LatencyMetrics {
                min_ms: latencies.first().copied().unwrap_or(0.0),
                max_ms: latencies.last().copied().unwrap_or(0.0),
                avg_ms: latencies.iter().sum::<f32>() / latencies.len() as f32,
                median_ms: latencies[latencies.len() / 2],
                p95_ms: latencies[(latencies.len() as f32 * 0.95) as usize],
                p99_ms: latencies[(latencies.len() as f32 * 0.99) as usize],
                p999_ms: latencies.last().copied().unwrap_or(0.0),
                standard_deviation_ms: calculate_std_dev(&latencies),
            },
            throughput: ThroughputMetrics {
                requests_per_second: context.test_parameters.iterations as f32 / execution_time.as_secs_f32(),
                bits_per_second: 0.0,
                operations_per_second: successful_pings as f32 / execution_time.as_secs_f32(),
                peak_throughput: 0.0,
            },
            resource_usage: ResourceUsageMetrics {
                cpu_usage_percent: 15.0,
                memory_usage_percent: 5.0,
                memory_usage_mb: 128,
                disk_usage_percent: 1.0,
                disk_iops: 10,
                network_usage_percent: 50.0,
                power_consumption_watts: 5.0,
            },
            quality_metrics: QualityMetrics {
                error_rate_percent: (failed_pings as f32 / context.test_parameters.iterations as f32) * 100.0,
                availability_percent: ((successful_pings as f32 / context.test_parameters.iterations as f32) * 100.0),
                success_rate_percent: ((successful_pings as f32 / context.test_parameters.iterations as f32) * 100.0),
                data_integrity_score: 100.0,
                reliability_score: ((successful_pings as f32 / context.test_parameters.iterations as f32) * 100.0),
            },
        },
        raw_data: Vec::new(),
        passed_criteria: true,
        performance_score: 78.0,
    };
    
    Ok(result)
}

/// Calculate standard deviation
fn calculate_std_dev(values: &[f32]) -> f32 {
    if values.is_empty() {
        return 0.0;
    }
    
    let mean = values.iter().sum::<f32>() / values.len() as f32;
    let variance = values
        .iter()
        .map(|&x| (x - mean).powi(2))
        .sum::<f32>() / values.len() as f32;
    
    variance.sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmark_suite_creation() {
        let suites = create_edge_computing_benchmarks();
        assert_eq!(suites.length(), 2);
        
        let first_suite = &suites[0];
        assert_eq!(first_suite.suite_id, "edge-ai-inference");
        assert_eq!(first_suite.category, BenchmarkCategory::Throughput);
        assert_eq!(first_suite.benchmarks.len(), 1);
    }

    #[test]
    fn test_performance_thresholds() {
        let thresholds = PerformanceThresholds {
            max_latency_ms: 20.0,
            min_throughput_ops_per_sec: 50.0,
            max_cpu_usage_percent: 70.0,
            max_memory_usage_percent: 60.0,
            max_error_rate_percent: 0.1,
            min_availability_percent: 99.0,
        };
        
        assert_eq!(thresholds.max_latency_ms, 20.0);
        assert_eq!(thresholds.min_throughput_ops_per_sec, 50.0);
        assert!(thresholds.min_availability_percent > 95.0);
    }

    #[test]
    fn test_hardware_profile() {
        let profile = HardwareProfile {
            cpu_model: "ARM Cortex-A78".to_string(),
            cpu_cores_physical: 4,
            cpu_cores_logical: 8,
            memory_gb: 8,
            storage_type: StorageType::Nvme,
            storage_gb: 256,
            gpu_model: Some("ARM Mali-G78".to_string()),
            gpu_memory_gb: Some(12),
            network_interface: "Ethernet".to_string(),
            network_speed_gbps: 1.0,
        };
        
        assert_eq!(profile.cpu_cores_physical, 4);
        assert!(profile.gpu_model.is_some());
        assert_eq!(profile.storage_type, StorageType::Nvme);
    }

    #[test]
    fn test_standard_deviation_calculation() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let std_dev = calculate_std_dev(&values);
        
        // Standard deviation of 1,2,3,4,5 should be approximately 1.41
        assert!((std_dev - 1.41421356237).abs() < 0.001);
    }

    #[test]
    fn test_benchmark_result_creation() {
        let result = BenchmarkResult {
            benchmark_id: "test-benchmark".to_string(),
            execution_time: Duration::from_secs(10),
            metrics: BenchmarkMetrics {
                latency_ms: LatencyMetrics {
                    min_ms: 1.0,
                    max_ms: 10.0,
                    avg_ms: 5.0,
                    median_ms: 5.0,
                    p95_ms: 9.0,
                    p99_ms: 9.9,
                    p999_ms: 10.0,
                    standard_deviation_ms: 2.0,
                },
                throughput: ThroughputMetrics {
                    requests_per_second: 100.0,
                    bits_per_second: 1000000.0,
                    operations_per_second: 100.0,
                    peak_throughput: 150.0,
                },
                resource_usage: ResourceUsageMetrics {
                    cpu_usage_percent: 50.0,
                    memory_usage_percent: 40.0,
                    memory_usage_mb: 2048,
                    disk_usage_percent: 20.0,
                    disk_iops: 500,
                    network_usage_percent: 30.0,
                    power_consumption_watts: 25.0,
                },
                quality_metrics: QualityMetrics {
                    error_rate_percent: 0.1,
                    availability_percent: 99.9,
                    success_rate_percent: 99.9,
                    data_integrity_score: 100.0,
                    reliability_score: 99.9,
                },
            },
            raw_data: Vec::new(),
            passed_criteria: true,
            performance_score: 85.0,
        };
        
        assert_eq!(result.benchmark_id, "test-benchmark");
        assert!(result.passed_criteria);
        assert_eq!(result.performance_score, 85.0);
        assert_eq!(result.metrics.latency_ms.avg_ms, 5.0);
    }
}