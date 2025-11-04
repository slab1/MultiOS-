//! Basic Driver Test Example
//!
//! This example demonstrates how to use the driver testing framework
//! to create and run basic driver tests.

use driver_testing_framework::{
    DriverTestSuite, TestCategory, TestResult, TestStatus, DriverTestError,
    core::DriverTest, core::TestConfig
};
use std::time::Duration;

/// Example driver for testing purposes
pub struct ExampleSerialDriver {
    /// Driver state
    initialized: bool,
    data_buffer: Vec<u8>,
}

/// Custom test implementation for the example driver
pub struct SerialDriverTest {
    config: TestConfig,
    driver: ExampleSerialDriver,
}

impl SerialDriverTest {
    pub fn new() -> Self {
        Self {
            config: TestConfig::new(
                "serial_driver_basic_test".to_string(),
                TestCategory::Unit
            ),
            driver: ExampleSerialDriver {
                initialized: false,
                data_buffer: Vec::new(),
            },
        }
    }
}

impl DriverTest for SerialDriverTest {
    fn config(&self) -> &TestConfig {
        &self.config
    }
    
    fn setup(&mut self) -> Result<(), DriverTestError> {
        log::info!("Setting up serial driver test");
        
        // Initialize the driver
        self.driver.initialized = true;
        self.driver.data_buffer.clear();
        
        Ok(())
    }
    
    fn execute(&self) -> Result<TestResult, DriverTestError> {
        log::info!("Executing serial driver test");
        
        let start_time = std::time::Instant::now();
        
        // Test driver initialization
        if !self.driver.initialized {
            return Ok(TestResult::failure(
                self.config.name.clone(),
                start_time.elapsed(),
                self.config.category,
                "Driver not initialized".to_string()
            ));
        }
        
        // Test data buffer operations
        let test_data = vec![0x48, 0x65, 0x6C, 0x6C, 0x6F]; // "Hello" in ASCII
        self.driver.data_buffer.extend_from_slice(&test_data);
        
        if self.driver.data_buffer.len() != test_data.len() {
            return Ok(TestResult::failure(
                self.config.name.clone(),
                start_time.elapsed(),
                self.config.category,
                "Data buffer length mismatch".to_string()
            ));
        }
        
        // Test data buffer content
        if &self.driver.data_buffer[..] != test_data {
            return Ok(TestResult::failure(
                self.config.name.clone(),
                start_time.elapsed(),
                self.config.category,
                "Data buffer content mismatch".to_string()
            ));
        }
        
        // Test buffer clear
        let original_len = self.driver.data_buffer.len();
        self.driver.data_buffer.clear();
        
        if !self.driver.data_buffer.is_empty() {
            return Ok(TestResult::failure(
                self.config.name.clone(),
                start_time.elapsed(),
                self.config.category,
                "Data buffer not cleared properly".to_string()
            ));
        }
        
        Ok(TestResult::success(
            self.config.name.clone(),
            start_time.elapsed(),
            self.config.category
        ))
    }
    
    fn cleanup(&mut self) -> Result<(), DriverTestError> {
        log::info!("Cleaning up serial driver test");
        
        // Clean up resources
        self.driver.data_buffer.clear();
        self.driver.initialized = false;
        
        Ok(())
    }
    
    fn description(&self) -> &str {
        "Tests basic serial driver functionality including initialization, buffer operations, and cleanup"
    }
}

/// Keyboard driver test
pub struct KeyboardDriverTest {
    config: TestConfig,
    scan_code_buffer: Vec<u8>,
}

impl KeyboardDriverTest {
    pub fn new() -> Self {
        Self {
            config: TestConfig::new(
                "keyboard_driver_test".to_string(),
                TestCategory::Unit
            ),
            scan_code_buffer: Vec::new(),
        }
    }
}

impl DriverTest for KeyboardDriverTest {
    fn config(&self) -> &TestConfig {
        &self.config
    }
    
    fn setup(&mut self) -> Result<(), DriverTestError> {
        log::info!("Setting up keyboard driver test");
        self.scan_code_buffer.clear();
        Ok(())
    }
    
    fn execute(&self) -> Result<TestResult, DriverTestError> {
        log::info!("Executing keyboard driver test");
        
        let start_time = std::time::Instant::now();
        
        // Test scan code processing
        let test_scan_codes = vec![0x1C, 0xF0, 0x1C]; // Enter key press and release
        
        for &scan_code in &test_scan_codes {
            if scan_code == 0xF0 {
                // Release code - test that we handle it
                continue;
            }
            
            // Simulate scan code processing
            self.scan_code_buffer.push(scan_code);
        }
        
        // Verify scan codes were processed
        if self.scan_code_buffer.len() != 2 { // Only non-release codes
            return Ok(TestResult::failure(
                self.config.name.clone(),
                start_time.elapsed(),
                self.config.category,
                "Scan code processing failed".to_string()
            ));
        }
        
        // Test key event generation
        let key_event = self.generate_key_event(self.scan_code_buffer[0]);
        
        if key_event.is_none() {
            return Ok(TestResult::failure(
                self.config.name.clone(),
                start_time.elapsed(),
                self.config.category,
                "Key event generation failed".to_string()
            ));
        }
        
        Ok(TestResult::success(
            self.config.name.clone(),
            start_time.elapsed(),
            self.config.category
        ))
    }
    
    fn cleanup(&mut self) -> Result<(), DriverTestError> {
        log::info!("Cleaning up keyboard driver test");
        self.scan_code_buffer.clear();
        Ok(())
    }
    
    fn description(&self) -> &str {
        "Tests keyboard driver functionality including scan code processing and key event generation"
    }
}

impl KeyboardDriverTest {
    /// Generate key event from scan code
    fn generate_key_event(&self, scan_code: u8) -> Option<String> {
        match scan_code {
            0x1C => Some("Enter".to_string()),
            0x14 => Some("Left Shift".to_string()),
            0x11 => Some("Alt".to_string()),
            _ => None,
        }
    }
}

/// Timer driver test
pub struct TimerDriverTest {
    config: TestConfig,
    tick_count: u32,
    timer_enabled: bool,
}

impl TimerDriverTest {
    pub fn new() -> Self {
        Self {
            config: TestConfig::new(
                "timer_driver_test".to_string(),
                TestCategory::Unit
            ),
            tick_count: 0,
            timer_enabled: false,
        }
    }
}

impl DriverTest for TimerDriverTest {
    fn config(&self) -> &TestConfig {
        &self.config
    }
    
    fn setup(&mut self) -> Result<(), DriverTestError> {
        log::info!("Setting up timer driver test");
        self.tick_count = 0;
        self.timer_enabled = false;
        Ok(())
    }
    
    fn execute(&self) -> Result<TestResult, DriverTestError> {
        log::info!("Executing timer driver test");
        
        let start_time = std::time::Instant::now();
        
        // Test timer initialization
        if self.timer_enabled {
            return Ok(TestResult::failure(
                self.config.name.clone(),
                start_time.elapsed(),
                self.config.category,
                "Timer should not be enabled initially".to_string()
            ));
        }
        
        // Enable timer
        self.timer_enabled = true;
        
        if !self.timer_enabled {
            return Ok(TestResult::failure(
                self.config.name.clone(),
                start_time.elapsed(),
                self.config.category,
                "Failed to enable timer".to_string()
            ));
        }
        
        // Simulate timer ticks
        for _ in 0..10 {
            self.simulate_timer_tick();
        }
        
        // Verify tick count
        if self.tick_count != 10 {
            return Ok(TestResult::failure(
                self.config.name.clone(),
                start_time.elapsed(),
                self.config.category,
                format!("Expected 10 ticks, got {}", self.tick_count)
            ));
        }
        
        // Test timer disable
        self.timer_enabled = false;
        
        if self.timer_enabled {
            return Ok(TestResult::failure(
                self.config.name.clone(),
                start_time.elapsed(),
                self.config.category,
                "Timer should be disabled".to_string()
            ));
        }
        
        Ok(TestResult::success(
            self.config.name.clone(),
            start_time.elapsed(),
            self.config.category
        ))
    }
    
    fn cleanup(&mut self) -> Result<(), DriverTestError> {
        log::info!("Cleaning up timer driver test");
        self.timer_enabled = false;
        Ok(())
    }
    
    fn description(&self) -> &str {
        "Tests timer driver functionality including initialization, tick counting, and disable/enable operations"
    }
}

impl TimerDriverTest {
    /// Simulate a timer tick
    fn simulate_timer_tick(&mut self) {
        if self.timer_enabled {
            self.tick_count = self.tick_count.wrapping_add(1);
        }
    }
}

/// Example of using the testing framework
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();
    
    log::info!("Starting basic driver test example");
    
    // Create test suite with default configuration
    let mut test_suite = DriverTestSuite::new();
    
    // Create test instances
    let mut serial_test = SerialDriverTest::new();
    let mut keyboard_test = KeyboardDriverTest::new();
    let mut timer_test = TimerDriverTest::new();
    
    // Run individual tests
    log::info!("Running individual driver tests");
    
    // Run serial driver test
    let serial_result = run_driver_test(&mut serial_test).await?;
    log::info!("Serial driver test result: {}", serial_result.status);
    
    // Run keyboard driver test
    let keyboard_result = run_driver_test(&mut keyboard_test).await?;
    log::info!("Keyboard driver test result: {}", keyboard_result.status);
    
    // Run timer driver test
    let timer_result = run_driver_test(&mut timer_test).await?;
    log::info!("Timer driver test result: {}", timer_result.status);
    
    // Run all tests using the test suite
    log::info!("Running comprehensive test suite");
    let test_results = test_suite.run_all_tests().await?;
    
    // Display results
    println!("\n=== Test Results Summary ===");
    println!("Total tests: {}", test_results.total_tests());
    println!("Passed: {}", test_results.passed_tests());
    println!("Failed: {}", test_results.failed_tests());
    println!("Skipped: {}", test_results.skipped_tests());
    
    // Generate and display report
    let report = test_suite.generate_test_report(&test_results)?;
    println!("\n{}", report);
    
    log::info!("Basic driver test example completed");
    
    Ok(())
}

/// Helper function to run a driver test
async fn run_driver_test(test: &mut dyn DriverTest) -> Result<TestResult, DriverTestError> {
    // Setup
    test.setup()?;
    
    // Execute
    let result = test.execute();
    
    // Cleanup
    let _ = test.cleanup();
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_serial_driver_test() {
        let mut test = SerialDriverTest::new();
        
        // Test setup
        test.setup().unwrap();
        assert!(test.driver.initialized);
        
        // Test execution
        let result = test.execute().unwrap();
        assert!(result.is_success());
        
        // Test cleanup
        test.cleanup().unwrap();
        assert!(!test.driver.initialized);
    }
    
    #[tokio::test]
    async fn test_keyboard_driver_test() {
        let mut test = KeyboardDriverTest::new();
        
        // Test setup
        test.setup().unwrap();
        assert!(test.scan_code_buffer.is_empty());
        
        // Test execution
        let result = test.execute().unwrap();
        assert!(result.is_success());
        
        // Test cleanup
        test.cleanup().unwrap();
        assert!(test.scan_code_buffer.is_empty());
    }
    
    #[tokio::test]
    async fn test_timer_driver_test() {
        let mut test = TimerDriverTest::new();
        
        // Test setup
        test.setup().unwrap();
        assert!(!test.timer_enabled);
        assert_eq!(test.tick_count, 0);
        
        // Test execution
        let result = test.execute().unwrap();
        assert!(result.is_success());
        
        // Test cleanup
        test.cleanup().unwrap();
        assert!(!test.timer_enabled);
    }
}
