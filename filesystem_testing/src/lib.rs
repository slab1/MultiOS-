//! MultiOS File System Testing Framework
//! 
//! Comprehensive testing, validation, and debugging tools for file system operations.
//! Provides stress testing, integrity checking, recovery tools, performance benchmarking,
//! and automated testing for various file system types and edge cases.

#![no_std]

#[cfg(feature = "std")]
extern crate std;

extern crate alloc;
use alloc::vec::Vec;
use alloc::string::String;
use alloc::boxed::Box;
use alloc::collections::BTreeMap;

use spin::Mutex;
use log::{info, warn, error, debug};
use chrono::{DateTime, Utc};

pub mod stress_testing;
pub mod integrity_checking;
pub mod recovery_tools;
pub mod performance_benchmarking;
pub mod disk_analysis;
pub mod image_creation;
pub mod automated_testing;
pub mod test_suite;
pub mod validation_framework;
pub mod examples;

/// Test result types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestResult {
    Passed,
    Failed,
    Skipped,
    Timeout,
    Error,
}

/// File system test statistics
#[derive(Debug, Clone)]
pub struct TestStats {
    pub total_tests: u64,
    pub passed_tests: u64,
    pub failed_tests: u64,
    pub skipped_tests: u64,
    pub timeout_tests: u64,
    pub error_tests: u64,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
}

impl Default for TestStats {
    fn default() -> Self {
        Self {
            total_tests: 0,
            passed_tests: 0,
            failed_tests: 0,
            skipped_tests: 0,
            timeout_tests: 0,
            error_tests: 0,
            start_time: Utc::now(),
            end_time: None,
        }
    }
}

impl TestStats {
    pub fn add_result(&mut self, result: TestResult) {
        self.total_tests += 1;
        match result {
            TestResult::Passed => self.passed_tests += 1,
            TestResult::Failed => self.failed_tests += 1,
            TestResult::Skipped => self.skipped_tests += 1,
            TestResult::Timeout => self.timeout_tests += 1,
            TestResult::Error => self.error_tests += 1,
        }
    }

    pub fn success_rate(&self) -> f64 {
        if self.total_tests == 0 { return 0.0; }
        (self.passed_tests as f64 / self.total_tests as f64) * 100.0
    }

    pub fn duration(&self) -> Option<chrono::Duration> {
        self.end_time.map(|end| end - self.start_time)
    }
}

/// Global test coordinator
pub static TEST_COORDINATOR: Mutex<Option<TestCoordinator>> = Mutex::new(None);

/// Main test coordinator for orchestrating all file system tests
pub struct TestCoordinator {
    stats: TestStats,
    test_suites: BTreeMap<String, Box<dyn test_suite::TestSuite>>,
    results: BTreeMap<String, TestResult>,
}

impl TestCoordinator {
    pub fn new() -> Self {
        info!("Initializing File System Test Coordinator");
        Self {
            stats: TestStats::default(),
            test_suites: BTreeMap::new(),
            results: BTreeMap::new(),
        }
    }

    pub fn register_suite(&mut self, name: String, suite: Box<dyn test_suite::TestSuite>) {
        info!("Registering test suite: {}", name);
        self.test_suites.insert(name, suite);
    }

    pub fn run_all_tests(&mut self) -> TestResult {
        info!("Starting comprehensive file system test suite");
        let mut overall_result = TestResult::Passed;

        for (suite_name, suite) in &self.test_suites {
            info!("Running test suite: {}", suite_name);
            
            match suite.run() {
                TestResult::Passed => {
                    info!("✓ Suite '{}' passed", suite_name);
                    self.results.insert(suite_name.clone(), TestResult::Passed);
                },
                result => {
                    error!("✗ Suite '{}' failed: {:?}", suite_name, result);
                    self.results.insert(suite_name.clone(), result);
                    if result != TestResult::Skipped {
                        overall_result = TestResult::Failed;
                    }
                }
            }
        }

        self.stats.end_time = Some(Utc::now());
        info!("Test suite completed. Success rate: {:.2}%", self.stats.success_rate());
        overall_result
    }

    pub fn run_suite(&mut self, name: &str) -> TestResult {
        if let Some(suite) = self.test_suites.get_mut(name) {
            info!("Running individual test suite: {}", name);
            let result = suite.run();
            self.results.insert(name.to_string(), result);
            result
        } else {
            warn!("Test suite '{}' not found", name);
            TestResult::Failed
        }
    }

    pub fn get_statistics(&self) -> &TestStats {
        &self.stats
    }

    pub fn get_results(&self) -> &BTreeMap<String, TestResult> {
        &self.results
    }
}

/// Initialize the testing framework
pub fn init_testing() -> Result<(), &'static str> {
    let mut coordinator = TEST_COORDINATOR.lock();
    
    // Create coordinator if not exists
    if coordinator.is_none() {
        *coordinator = Some(TestCoordinator::new());
    }
    
    // Register built-in test suites
    if let Some(coord) = coordinator.as_mut() {
        coord.register_suite(
            "stress_testing".to_string(),
            Box::new(stress_testing::StressTestSuite::new())
        );
        
        coord.register_suite(
            "integrity_checking".to_string(),
            Box::new(integrity_checking::IntegrityTestSuite::new())
        );
        
        coord.register_suite(
            "performance_benchmarking".to_string(),
            Box::new(performance_benchmarking::BenchmarkTestSuite::new())
        );
        
        coord.register_suite(
            "disk_analysis".to_string(),
            Box::new(disk_analysis::DiskAnalysisSuite::new())
        );
        
        coord.register_suite(
            "automated_testing".to_string(),
            Box::new(automated_testing::AutomatedTestSuite::new())
        );
    }
    
    info!("File system testing framework initialized");
    Ok(())
}

/// Run the complete test suite
pub fn run_full_test_suite() -> TestResult {
    let mut coordinator = TEST_COORDINATOR.lock();
    
    match coordinator.as_mut() {
        Some(coord) => coord.run_all_tests(),
        None => {
            error!("Test coordinator not initialized");
            TestResult::Error
        }
    }
}

/// Run a specific test suite
pub fn run_test_suite(name: &str) -> TestResult {
    let mut coordinator = TEST_COORDINATOR.lock();
    
    match coordinator.as_mut() {
        Some(coord) => coord.run_suite(name),
        None => {
            error!("Test coordinator not initialized");
            TestResult::Error
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_test_result_ordering() {
        assert_eq!(TestResult::Passed as u8, 0);
        assert_eq!(TestResult::Failed as u8, 1);
        assert_eq!(TestResult::Skipped as u8, 2);
        assert_eq!(TestResult::Timeout as u8, 3);
        assert_eq!(TestResult::Error as u8, 4);
    }

    #[test]
    fn test_test_stats() {
        let mut stats = TestStats::default();
        assert_eq!(stats.total_tests, 0);
        
        stats.add_result(TestResult::Passed);
        stats.add_result(TestResult::Failed);
        assert_eq!(stats.total_tests, 2);
        assert_eq!(stats.passed_tests, 1);
        assert_eq!(stats.failed_tests, 1);
        assert!((stats.success_rate() - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_test_coordinator() {
        let mut coordinator = TestCoordinator::new();
        
        // Add a mock test suite
        coordinator.register_suite(
            "mock_suite".to_string(),
            Box::new(MockTestSuite::new())
        );
        
        let result = coordinator.run_suite("mock_suite");
        assert_eq!(result, TestResult::Passed);
        
        assert!(coordinator.get_results().contains_key("mock_suite"));
    }

    // Mock test suite for testing
    struct MockTestSuite {
        result: TestResult,
    }

    impl MockTestSuite {
        fn new() -> Self {
            Self { result: TestResult::Passed }
        }
    }

    impl test_suite::TestSuite for MockTestSuite {
        fn name(&self) -> &str {
            "MockTestSuite"
        }
        
        fn description(&self) -> &str {
            "Mock test suite for testing framework"
        }
        
        fn run(&self) -> TestResult {
            self.result
        }
    }
}