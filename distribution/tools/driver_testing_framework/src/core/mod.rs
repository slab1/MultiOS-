//! Core types and interfaces for the driver testing framework

use core::fmt;
use core::time::Duration;
use serde::{Deserialize, Serialize};

/// Error types for driver testing framework
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DriverTestError {
    /// Initialization error
    InitializationError(String),
    /// Test execution error
    TestExecutionError(String),
    /// Validation error
    ValidationError(String),
    /// Hardware simulation error
    HardwareSimulationError(String),
    /// Performance benchmarking error
    PerformanceError(String),
    /// Resource allocation error
    ResourceError(String),
    /// Timeout error
    TimeoutError(Duration),
    /// Framework already initialized
    AlreadyInitialized,
    /// Test not found
    TestNotFound(String),
    /// Configuration error
    ConfigurationError(String),
    /// Driver binding error
    DriverBindingError(String),
    /// Hardware error
    HardwareError(String),
    /// Invalid test configuration
    InvalidConfiguration(String),
}

impl fmt::Display for DriverTestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DriverTestError::InitializationError(msg) => 
                write!(f, "Initialization error: {}", msg),
            DriverTestError::TestExecutionError(msg) => 
                write!(f, "Test execution error: {}", msg),
            DriverTestError::ValidationError(msg) => 
                write!(f, "Validation error: {}", msg),
            DriverTestError::HardwareSimulationError(msg) => 
                write!(f, "Hardware simulation error: {}", msg),
            DriverTestError::PerformanceError(msg) => 
                write!(f, "Performance error: {}", msg),
            DriverTestError::ResourceError(msg) => 
                write!(f, "Resource error: {}", msg),
            DriverTestError::TimeoutError(duration) => 
                write!(f, "Timeout after {:?}", duration),
            DriverTestError::AlreadyInitialized => 
                write!(f, "Testing framework already initialized"),
            DriverTestError::TestNotFound(name) => 
                write!(f, "Test not found: {}", name),
            DriverTestError::ConfigurationError(msg) => 
                write!(f, "Configuration error: {}", msg),
            DriverTestError::DriverBindingError(msg) => 
                write!(f, "Driver binding error: {}", msg),
            DriverTestError::HardwareError(msg) => 
                write!(f, "Hardware error: {}", msg),
            DriverTestError::InvalidConfiguration(msg) => 
                write!(f, "Invalid configuration: {}", msg),
        }
    }
}

impl std::error::Error for DriverTestError {}

/// Test result structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    /// Test name
    pub name: String,
    /// Test status
    pub status: TestStatus,
    /// Test execution duration
    pub duration: Duration,
    /// Test result message
    pub message: String,
    /// Test category
    pub category: TestCategory,
    /// Additional test metadata
    pub metadata: Option<TestMetadata>,
    /// Test performance metrics
    pub metrics: Option<TestMetrics>,
}

impl TestResult {
    /// Create a new successful test result
    pub fn success(name: String, duration: Duration, category: TestCategory) -> Self {
        Self {
            name,
            status: TestStatus::Passed,
            duration,
            message: "Test passed".to_string(),
            category,
            metadata: None,
            metrics: None,
        }
    }
    
    /// Create a new failed test result
    pub fn failure(name: String, duration: Duration, category: TestCategory, message: String) -> Self {
        Self {
            name,
            status: TestStatus::Failed,
            duration,
            message,
            category,
            metadata: None,
            metrics: None,
        }
    }
    
    /// Create a new skipped test result
    pub fn skip(name: String, duration: Duration, category: TestCategory, message: String) -> Self {
        Self {
            name,
            status: TestStatus::Skipped,
            duration,
            message,
            category,
            metadata: None,
            metrics: None,
        }
    }
    
    /// Check if test passed
    pub fn is_success(&self) -> bool {
        self.status == TestStatus::Passed
    }
    
    /// Check if test failed
    pub fn is_failure(&self) -> bool {
        self.status == TestStatus::Failed
    }
}

/// Test status enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TestStatus {
    /// Test passed
    Passed,
    /// Test failed
    Failed,
    /// Test skipped
    Skipped,
    /// Test timed out
    Timeout,
    /// Test error
    Error,
}

impl fmt::Display for TestStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TestStatus::Passed => write!(f, "PASSED"),
            TestStatus::Failed => write!(f, "FAILED"),
            TestStatus::Skipped => write!(f, "SKIPPED"),
            TestStatus::Timeout => write!(f, "TIMEOUT"),
            TestStatus::Error => write!(f, "ERROR"),
        }
    }
}

/// Test category enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TestCategory {
    /// Unit tests
    Unit,
    /// Integration tests
    Integration,
    /// Performance tests
    Performance,
    /// Stress tests
    Stress,
    /// Validation tests
    Validation,
    /// Security tests
    Security,
    /// Compatibility tests
    Compatibility,
    /// Regression tests
    Regression,
    /// Debugging tests
    Debug,
    /// Troubleshooting tests
    Troubleshooting,
}

impl fmt::Display for TestCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TestCategory::Unit => write!(f, "Unit"),
            TestCategory::Integration => write!(f, "Integration"),
            TestCategory::Performance => write!(f, "Performance"),
            TestCategory::Stress => write!(f, "Stress"),
            TestCategory::Validation => write!(f, "Validation"),
            TestCategory::Security => write!(f, "Security"),
            TestCategory::Compatibility => write!(f, "Compatibility"),
            TestCategory::Regression => write!(f, "Regression"),
            TestCategory::Debug => write!(f, "Debug"),
            TestCategory::Troubleshooting => write!(f, "Troubleshooting"),
        }
    }
}

/// Test configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestConfig {
    /// Test name
    pub name: String,
    /// Test category
    pub category: TestCategory,
    /// Test timeout
    pub timeout: Option<Duration>,
    /// Test dependencies
    pub dependencies: Vec<String>,
    /// Test tags for filtering
    pub tags: Vec<String>,
    /// Test priority (higher number = higher priority)
    pub priority: u8,
    /// Test retry count
    pub retry_count: u8,
    /// Test parallel execution flag
    pub parallel: bool,
    /// Test resource requirements
    pub resource_requirements: Option<TestResourceRequirements>,
}

impl TestConfig {
    /// Create a new test configuration
    pub fn new(name: String, category: TestCategory) -> Self {
        Self {
            name,
            category,
            timeout: Some(Duration::from_secs(30)),
            dependencies: Vec::new(),
            tags: Vec::new(),
            priority: 1,
            retry_count: 0,
            parallel: false,
            resource_requirements: None,
        }
    }
}

/// Driver test trait
pub trait DriverTest {
    /// Get test configuration
    fn config(&self) -> &TestConfig;
    
    /// Execute the test
    fn execute(&self) -> Result<TestResult, DriverTestError>;
    
    /// Set up test environment
    fn setup(&mut self) -> Result<(), DriverTestError> {
        Ok(())
    }
    
    /// Clean up test environment
    fn cleanup(&mut self) -> Result<(), DriverTestError> {
        Ok(())
    }
    
    /// Get test description
    fn description(&self) -> &str {
        "Driver test"
    }
    
    /// Get test tags
    fn tags(&self) -> &[String] {
        &[]
    }
}

/// Test metadata structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestMetadata {
    /// Test environment information
    pub environment: EnvironmentInfo,
    /// Test system information
    pub system_info: SystemInfo,
    /// Test driver information
    pub driver_info: DriverInfo,
}

/// Environment information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentInfo {
    /// Operating system
    pub os: String,
    /// Architecture
    pub architecture: String,
    /// Testing framework version
    pub framework_version: String,
    /// Test execution timestamp
    pub timestamp: String,
}

/// System information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    /// CPU model
    pub cpu_model: String,
    /// CPU cores
    pub cpu_cores: u32,
    /// Memory size (bytes)
    pub memory_size: u64,
    /// Available memory (bytes)
    pub available_memory: u64,
}

/// Driver information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriverInfo {
    /// Driver name
    pub name: String,
    /// Driver version
    pub version: String,
    /// Driver type
    pub driver_type: String,
    /// Hardware device
    pub hardware_device: String,
}

/// Performance metrics structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestMetrics {
    /// Execution time
    pub execution_time: Duration,
    /// Memory usage
    pub memory_usage: MemoryMetrics,
    /// CPU usage
    pub cpu_usage: CpuMetrics,
    /// I/O metrics
    pub io_metrics: IoMetrics,
}

/// Memory metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryMetrics {
    /// Peak memory usage (bytes)
    pub peak_usage: u64,
    /// Average memory usage (bytes)
    pub average_usage: u64,
    /// Memory allocations
    pub allocations: u64,
    /// Memory deallocations
    pub deallocations: u64,
    /// Memory leaks (bytes)
    pub leaks: u64,
}

/// CPU metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuMetrics {
    /// CPU usage percentage
    pub usage_percent: f32,
    /// Context switches
    pub context_switches: u64,
    /// System calls
    pub system_calls: u64,
}

/// I/O metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IoMetrics {
    /// Read operations
    pub read_operations: u64,
    /// Write operations
    pub write_operations: u64,
    /// Bytes read
    pub bytes_read: u64,
    /// Bytes written
    pub bytes_written: u64,
    /// I/O errors
    pub errors: u64,
}

/// Test resource requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResourceRequirements {
    /// Minimum memory required (bytes)
    pub min_memory: u64,
    /// Minimum CPU cores required
    pub min_cpu_cores: u32,
    /// Required hardware devices
    pub required_devices: Vec<String>,
    /// Required driver capabilities
    pub required_capabilities: Vec<String>,
}

/// Test environment initializer
pub trait TestEnvironment {
    /// Initialize the test environment
    fn initialize(&mut self) -> Result<(), DriverTestError>;
    
    /// Clean up the test environment
    fn cleanup(&mut self) -> Result<(), DriverTestError>;
    
    /// Check if environment is ready
    fn is_ready(&self) -> bool;
    
    /// Get environment status
    fn status(&self) -> EnvironmentStatus;
}

/// Environment status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EnvironmentStatus {
    /// Environment is ready
    Ready,
    /// Environment is initializing
    Initializing,
    /// Environment is cleaning up
    Cleaning,
    /// Environment is in error state
    Error(String),
    /// Environment is unknown
    Unknown,
}

impl fmt::Display for EnvironmentStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EnvironmentStatus::Ready => write!(f, "Ready"),
            EnvironmentStatus::Initializing => write!(f, "Initializing"),
            EnvironmentStatus::Cleaning => write!(f, "Cleaning"),
            EnvironmentStatus::Error(msg) => write!(f, "Error: {}", msg),
            EnvironmentStatus::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Test executor trait
pub trait TestExecutor {
    /// Execute a test
    fn execute_test(&mut self, test: &dyn DriverTest) -> Result<TestResult, DriverTestError>;
    
    /// Execute multiple tests
    fn execute_tests(&mut self, tests: &[&dyn DriverTest]) -> Result<Vec<TestResult>, DriverTestError>;
    
    /// Get execution statistics
    fn get_statistics(&self) -> ExecutionStatistics;
}

/// Execution statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStatistics {
    /// Total tests executed
    pub total_tests: usize,
    /// Tests that passed
    pub passed_tests: usize,
    /// Tests that failed
    pub failed_tests: usize,
    /// Tests that were skipped
    pub skipped_tests: usize,
    /// Total execution time
    pub total_time: Duration,
    /// Average test time
    pub average_time: Duration,
}

impl ExecutionStatistics {
    /// Create new execution statistics
    pub fn new() -> Self {
        Self {
            total_tests: 0,
            passed_tests: 0,
            failed_tests: 0,
            skipped_tests: 0,
            total_time: Duration::from_secs(0),
            average_time: Duration::from_secs(0),
        }
    }
    
    /// Calculate success rate
    pub fn success_rate(&self) -> f32 {
        if self.total_tests == 0 {
            0.0
        } else {
            (self.passed_tests as f32) / (self.total_tests as f32) * 100.0
        }
    }
    
    /// Check if all tests passed
    pub fn all_passed(&self) -> bool {
        self.failed_tests == 0 && self.total_tests > 0
    }
}

impl Default for ExecutionStatistics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_test_result_creation() {
        let result = TestResult::success(
            "test_validation".to_string(),
            Duration::from_millis(100),
            TestCategory::Validation,
        );
        
        assert!(result.is_success());
        assert!(!result.is_failure());
        assert_eq!(result.category, TestCategory::Validation);
    }
    
    #[test]
    fn test_test_result_failure() {
        let result = TestResult::failure(
            "test_failure".to_string(),
            Duration::from_millis(50),
            TestCategory::Unit,
            "Test failed".to_string(),
        );
        
        assert!(!result.is_success());
        assert!(result.is_failure());
        assert_eq!(result.status, TestStatus::Failed);
    }
    
    #[test]
    fn test_execution_statistics() {
        let mut stats = ExecutionStatistics::new();
        stats.total_tests = 10;
        stats.passed_tests = 8;
        stats.failed_tests = 2;
        stats.total_time = Duration::from_secs(5);
        
        assert_eq!(stats.success_rate(), 80.0);
        assert!(!stats.all_passed());
        
        stats.failed_tests = 0;
        assert!(stats.all_passed());
    }
}
