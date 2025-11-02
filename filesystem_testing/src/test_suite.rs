//! Test suite base definitions
//! 
//! Provides the base trait and framework for organizing file system tests
//! into logical suites with common initialization, execution, and reporting.

use super::{TestResult, TestStats};

/// Base trait for all test suites
pub trait TestSuite {
    /// Get the name of the test suite
    fn name(&self) -> &str;
    
    /// Get description of what this test suite does
    fn description(&self) -> &str;
    
    /// Run the complete test suite
    fn run(&self) -> TestResult;
    
    /// Optional setup before running tests
    fn setup(&self) -> Result<(), &'static str> {
        Ok(())
    }
    
    /// Optional cleanup after running tests
    fn cleanup(&self) -> Result<(), &'static str> {
        Ok(())
    }
    
    /// Get test statistics for this suite
    fn get_stats(&self) -> &TestStats {
        static STATS: std::sync::OnceLock<TestStats> = std::sync::OnceLock::new();
        STATS.get_or_init(|| TestStats::default())
    }
}

/// Base implementation for test suites with common functionality
pub struct BaseTestSuite {
    name: String,
    description: String,
    stats: TestStats,
}

impl BaseTestSuite {
    pub fn new(name: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            stats: TestStats::default(),
        }
    }

    pub fn add_test_result(&mut self, result: TestResult) {
        self.stats.add_result(result);
    }

    pub fn get_stats(&self) -> &TestStats {
        &self.stats
    }
}

impl TestSuite for BaseTestSuite {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn run(&self) -> TestResult {
        // Base implementation should be overridden by specific suites
        TestResult::Passed
    }
}

/// Trait for test cases within suites
pub trait TestCase {
    /// Get test case name
    fn name(&self) -> &str;
    
    /// Get test case description
    fn description(&self) -> &str;
    
    /// Run the individual test case
    fn run(&self) -> TestResult;
    
    /// Setup before running this test case
    fn setup(&self) -> Result<(), &'static str> {
        Ok(())
    }
    
    /// Cleanup after running this test case
    fn cleanup(&self) -> Result<(), &'static str> {
        Ok(())
    }
    
    /// Check if this test should be skipped
    fn should_skip(&self) -> bool {
        false
    }
    
    /// Get timeout for this test (in milliseconds)
    fn timeout_ms(&self) -> u64 {
        30000 // 30 seconds default
    }
}

/// Base implementation for test cases
pub struct BaseTestCase {
    name: String,
    description: String,
    timeout: u64,
    skip: bool,
}

impl BaseTestCase {
    pub fn new(name: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            timeout: 30000,
            skip: false,
        }
    }

    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout = timeout_ms;
        self
    }

    pub fn skip(mut self, skip: bool) -> Self {
        self.skip = skip;
        self
    }
}

impl TestCase for BaseTestCase {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn run(&self) -> TestResult {
        TestResult::Passed
    }

    fn should_skip(&self) -> bool {
        self.skip
    }

    fn timeout_ms(&self) -> u64 {
        self.timeout
    }
}

/// Helper macro for creating test cases with custom logic
#[macro_export]
macro_rules! test_case {
    ($name:expr, $desc:expr, $test_fn:expr) => {
        struct TestCaseImpl {
            inner: Box<dyn Fn() -> TestResult>,
            name: String,
            description: String,
        }

        impl TestCaseImpl {
            fn new<F>(name: &str, description: &str, test_fn: F) -> Self
            where
                F: Fn() -> TestResult + 'static,
            {
                Self {
                    inner: Box::new(test_fn),
                    name: name.to_string(),
                    description: description.to_string(),
                }
            }
        }

        impl TestCase for TestCaseImpl {
            fn name(&self) -> &str {
                &self.name
            }

            fn description(&self) -> &str {
                &self.description
            }

            fn run(&self) -> TestResult {
                (self.inner)()
            }
        }

        TestCaseImpl::new($name, $desc, $test_fn)
    };
}

/// Helper macro for creating test suites
#[macro_export]
macro_rules! test_suite {
    ($name:expr, $desc:expr, [ $($test_case:expr),* ]) => {
        struct TestSuiteImpl {
            base: BaseTestSuite,
            test_cases: Vec<Box<dyn TestCase>>,
        }

        impl TestSuiteImpl {
            fn new() -> Self {
                let mut suite = Self {
                    base: BaseTestSuite::new($name, $desc),
                    test_cases: Vec::new(),
                };
                
                $(
                    suite.test_cases.push(Box::new($test_case));
                )*
                
                suite
            }
        }

        impl TestSuite for TestSuiteImpl {
            fn name(&self) -> &str {
                self.base.name()
            }

            fn description(&self) -> &str {
                self.base.description()
            }

            fn run(&self) -> TestResult {
                if let Err(e) = self.setup() {
                    error!("Test suite setup failed: {}", e);
                    return TestResult::Error;
                }

                let mut result = TestResult::Passed;
                
                for test_case in &self.test_cases {
                    if test_case.should_skip() {
                        info!("Skipping test: {}", test_case.name());
                        continue;
                    }

                    info!("Running test: {} - {}", test_case.name(), test_case.description());
                    
                    let test_result = test_case.run();
                    
                    match test_result {
                        TestResult::Passed => {
                            info!("✓ {} passed", test_case.name());
                        }
                        TestResult::Failed => {
                            error!("✗ {} failed", test_case.name());
                            result = TestResult::Failed;
                        }
                        TestResult::Error => {
                            error!("✗ {} errored", test_case.name());
                            result = TestResult::Error;
                        }
                        TestResult::Timeout => {
                            warn!("⚠ {} timed out", test_case.name());
                            result = TestResult::Timeout;
                        }
                        TestResult::Skipped => {
                            info!("⊘ {} skipped", test_case.name());
                        }
                    }
                }

                if let Err(e) = self.cleanup() {
                    error!("Test suite cleanup failed: {}", e);
                    result = TestResult::Error;
                }

                result
            }

            fn setup(&self) -> Result<(), &'static str> {
                Ok(())
            }

            fn cleanup(&self) -> Result<(), &'static str> {
                Ok(())
            }
        }

        Box::new(TestSuiteImpl::new())
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base_test_case() {
        let test_case = BaseTestCase::new("test_name", "Test description");
        assert_eq!(test_case.name(), "test_name");
        assert_eq!(test_case.description(), "Test description");
        assert!(!test_case.should_skip());
        assert_eq!(test_case.timeout_ms(), 30000);
    }

    #[test]
    fn test_base_test_case_custom_options() {
        let test_case = BaseTestCase::new("test", "desc")
            .with_timeout(60000)
            .skip(true);
        
        assert_eq!(test_case.timeout_ms(), 60000);
        assert!(test_case.should_skip());
    }

    #[test]
    fn test_test_case_macro() {
        let test_case = test_case!("test_name", "Test description", || {
            // Test logic here
            TestResult::Passed
        });
        
        assert_eq!(test_case.name(), "test_name");
        assert_eq!(test_case.description(), "Test description");
    }
}