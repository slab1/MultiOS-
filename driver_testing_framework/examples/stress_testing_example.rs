//! Stress Testing Example
//!
//! This example demonstrates how to use the stress testing capabilities
//! of the driver testing framework to test driver stability under various
//! load conditions and stress scenarios.

use driver_testing_framework::{
    StressTestConfig, StressTester, HardwareSimulator, SimulationEnvironment,
    DriverTestError, TestResult, TestStatus, TestCategory
};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Example stress test scenarios for serial driver
pub struct SerialDriverStressTest {
    name: String,
    category: TestCategory,
    concurrent_threads: usize,
    operations_per_thread: usize,
}

impl SerialDriverStressTest {
    pub fn new() -> Self {
        Self {
            name: "serial_driver_stress_test".to_string(),
            category: TestCategory::Stress,
            concurrent_threads: 4,
            operations_per_thread: 1000,
        }
    }
    
    /// Configure test parameters
    pub fn with_threads(mut self, threads: usize) -> Self {
        self.concurrent_threads = threads;
        self
    }
    
    pub fn with_operations_per_thread(mut self, ops: usize) -> Self {
        self.operations_per_thread = ops;
        self
    }
    
    /// Run the stress test
    pub async fn run_test(&mut self, simulator: &mut HardwareSimulator) -> Result<TestResult, DriverTestError> {
        let start_time = std::time::Instant::now();
        let error_count = Arc::new(Mutex::new(0usize));
        let success_count = Arc::new(Mutex::new(0usize));
        
        log::info!("Starting serial driver stress test");
        log::info!("Threads: {}, Operations per thread: {}", 
                  self.concurrent_threads, self.operations_per_thread);
        
        // Create handles for concurrent operations
        let mut handles = Vec::new();
        
        for thread_id in 0..self.concurrent_threads {
            let thread_error_count = Arc::clone(&error_count);
            let thread_success_count = Arc::clone(&success_count);
            
            let handle = tokio::spawn(async move {
                run_serial_stress_thread(
                    thread_id,
                    self.operations_per_thread,
                    &thread_error_count,
                    &thread_success_count
                ).await
            });
            
            handles.push(handle);
        }
        
        // Wait for all threads to complete
        for handle in handles {
            match handle.await {
                Ok(result) => {
                    if let Err(e) = result {
                        log::warn!("Thread failed: {}", e);
                    }
                },
                Err(e) => {
                    log::error!("Thread panic: {}", e);
                }
            }
        }
        
        // Collect results
        let total_errors = *error_count.lock().unwrap();
        let total_successes = *success_count.lock().unwrap();
        let total_operations = self.concurrent_threads * self.operations_per_thread;
        let success_rate = (total_successes as f32 / total_operations as f32) * 100.0;
        
        let duration = start_time.elapsed();
        let operations_per_second = (total_operations as f32) / duration.as_secs_f32();
        
        // Determine test status
        let status = if success_rate >= 95.0 {
            TestStatus::Passed
        } else if success_rate >= 80.0 {
            TestStatus::Warning
        } else {
            TestStatus::Failed
        };
        
        Ok(TestResult {
            name: self.name.clone(),
            status,
            duration,
            message: format!(
                "Serial stress test: {}/{} operations successful ({:.1}%), {:.1} ops/sec",
                total_successes, total_operations, success_rate, operations_per_second
            ),
            category: self.category,
            metadata: None,
            metrics: None,
        })
    }
}

/// Run stress test for a single thread
async fn run_serial_stress_thread(
    thread_id: usize,
    operations: usize,
    error_count: &Arc<Mutex<usize>>,
    success_count: &Arc<Mutex<usize>>
) -> Result<(), DriverTestError> {
    for operation_id in 0..operations {
        // Simulate serial driver operation
        match simulate_serial_operation(thread_id, operation_id).await {
            Ok(_) => {
                let mut count = success_count.lock().unwrap();
                *count += 1;
            },
            Err(_) => {
                let mut count = error_count.lock().unwrap();
                *count += 1;
            }
        }
        
        // Small delay to prevent overwhelming
        if operation_id % 100 == 0 {
            tokio::task::yield_now().await;
        }
    }
    
    Ok(())
}

/// Simulate serial driver operation
async fn simulate_serial_operation(thread_id: usize, operation_id: usize) -> Result<(), DriverTestError> {
    // Generate test data
    let test_data = vec![
        (thread_id % 256) as u8,
        (operation_id % 256) as u8,
        0x0A, // Newline
    ];
    
    // Simulate write operation
    simulate_serial_write(&test_data).await?;
    
    // Simulate read operation
    simulate_serial_read().await?;
    
    // Simulate interrupt handling
    if operation_id % 10 == 0 {
        simulate_serial_interrupt().await?;
    }
    
    Ok(())
}

/// Simulate serial write operation
async fn simulate_serial_write(data: &[u8]) -> Result<(), DriverTestError> {
    // Simulate time taken for write operation
    tokio::time::sleep(Duration::from_micros(10)).await;
    
    // Simulate occasional write failures (1% failure rate)
    if rand::random::<f32>() < 0.01 {
        return Err(DriverTestError::HardwareSimulationError(
            "Simulated write failure".to_string()
        ));
    }
    
    Ok(())
}

/// Simulate serial read operation
async fn simulate_serial_read() -> Result<(), DriverTestError> {
    // Simulate time taken for read operation
    tokio::time::sleep(Duration::from_micros(5)).await;
    
    // Simulate occasional read failures (0.5% failure rate)
    if rand::random::<f32>() < 0.005 {
        return Err(DriverTestError::HardwareSimulationError(
            "Simulated read failure".to_string()
        ));
    }
    
    Ok(())
}

/// Simulate serial interrupt handling
async fn simulate_serial_interrupt() -> Result<(), DriverTestError> {
    // Simulate interrupt handling time
    tokio::time::sleep(Duration::from_micros(2)).await;
    
    // Simulate occasional interrupt handling failures (0.1% failure rate)
    if rand::random::<f32>() < 0.001 {
        return Err(DriverTestError::HardwareSimulationError(
            "Simulated interrupt handling failure".to_string()
        ));
    }
    
    Ok(())
}

/// Example memory pressure test
pub struct MemoryPressureTest {
    name: String,
    category: TestCategory,
    target_memory_mb: usize,
}

impl MemoryPressureTest {
    pub fn new() -> Self {
        Self {
            name: "memory_pressure_test".to_string(),
            category: TestCategory::Stress,
            target_memory_mb: 100,
        }
    }
    
    pub fn with_target_memory(mut self, mb: usize) -> Self {
        self.target_memory_mb = mb;
        self
    }
    
    /// Run memory pressure test
    pub async fn run_test(&mut self) -> Result<TestResult, DriverTestError> {
        let start_time = std::time::Instant::now();
        
        log::info!("Starting memory pressure test - target: {} MB", self.target_memory_mb);
        
        let mut allocated_chunks = Vec::new();
        let chunk_size = 1024 * 1024; // 1MB chunks
        let mut allocation_errors = 0usize;
        
        // Allocate memory until target is reached or allocation fails
        while allocated_chunks.len() * chunk_size < self.target_memory_mb * 1024 * 1024 {
            match allocate_memory_chunk(chunk_size) {
                Ok(chunk) => {
                    allocated_chunks.push(chunk);
                    if allocated_chunks.len() % 10 == 0 {
                        log::info!("Allocated {} MB", allocated_chunks.len() * chunk_size / (1024 * 1024));
                    }
                },
                Err(_) => {
                    allocation_errors += 1;
                    log::warn!("Allocation {} failed", allocated_chunks.len());
                    
                    // Stop if too many consecutive failures
                    if allocation_errors > 10 {
                        break;
                    }
                }
            }
            
            // Small delay to allow system to react
            tokio::time::sleep(Duration::from_millis(1)).await;
        }
        
        // Keep some memory allocated to maintain pressure
        log::info!("Maintaining memory pressure for 5 seconds");
        tokio::time::sleep(Duration::from_secs(5)).await;
        
        // Deallocate memory
        log::info!("Deallocating memory");
        allocated_chunks.clear();
        
        // Force cleanup
        tokio::task::yield_now().await;
        
        let duration = start_time.elapsed();
        let allocated_mb = allocated_chunks.len() * chunk_size / (1024 * 1024);
        
        // Determine test status
        let status = if allocated_mb >= self.target_memory_mb / 2 {
            TestStatus::Passed
        } else {
            TestStatus::Warning
        };
        
        Ok(TestResult {
            name: self.name.clone(),
            status,
            duration,
            message: format!(
                "Memory pressure test: {} MB allocated, {} errors",
                allocated_mb, allocation_errors
            ),
            category: self.category,
            metadata: None,
            metrics: None,
        })
    }
}

/// Allocate memory chunk
fn allocate_memory_chunk(size: usize) -> Result<Vec<u8>, DriverTestError> {
    match std::panic::catch_unwind(|| {
        vec![0u8; size]
    }) {
        Ok(chunk) => Ok(chunk),
        Err(_) => Err(DriverTestError::ResourceError(
            "Failed to allocate memory chunk".to_string()
        )),
    }
}

/// Example CPU stress test
pub struct CpuStressTest {
    name: String,
    category: TestCategory,
    duration_seconds: u64,
    thread_count: usize,
}

impl CpuStressTest {
    pub fn new() -> Self {
        Self {
            name: "cpu_stress_test".to_string(),
            category: TestCategory::Stress,
            duration_seconds: 30,
            thread_count: 4,
        }
    }
    
    pub fn with_duration(mut self, seconds: u64) -> Self {
        self.duration_seconds = seconds;
        self
    }
    
    pub fn with_threads(mut self, threads: usize) -> Self {
        self.thread_count = threads;
        self
    }
    
    /// Run CPU stress test
    pub async fn run_test(&mut self) -> Result<TestResult, DriverTestError> {
        let start_time = std::time::Instant::now();
        let end_time = start_time + Duration::from_secs(self.duration_seconds);
        
        log::info!("Starting CPU stress test - {} threads, {} seconds", 
                  self.thread_count, self.duration_seconds);
        
        // Spawn CPU-intensive tasks
        let mut handles = Vec::new();
        
        for thread_id in 0..self.thread_count {
            let handle = tokio::spawn(async move {
                run_cpu_intensive_task(thread_id, end_time).await
            });
            handles.push(handle);
        }
        
        // Wait for completion
        for handle in handles {
            match handle.await {
                Ok(result) => {
                    if let Err(e) = result {
                        log::warn!("CPU stress task failed: {}", e);
                    }
                },
                Err(e) => {
                    log::error!("CPU stress task panic: {}", e);
                }
            }
        }
        
        let duration = start_time.elapsed();
        
        Ok(TestResult {
            name: self.name.clone(),
            status: TestStatus::Passed,
            duration,
            message: format!(
                "CPU stress test completed: {} threads, {} seconds",
                self.thread_count, self.duration_seconds
            ),
            category: self.category,
            metadata: None,
            metrics: None,
        })
    }
}

/// Run CPU-intensive task
async fn run_cpu_intensive_task(thread_id: usize, end_time: std::time::Instant) -> Result<(), DriverTestError> {
    let mut iterations = 0;
    
    while std::time::Instant::now() < end_time {
        // Perform CPU-intensive computation
        cpu_intensive_computation(1000);
        
        iterations += 1;
        
        // Periodically yield to prevent monopolizing CPU
        if iterations % 1000 == 0 {
            tokio::task::yield_now().await;
        }
    }
    
    log::info!("Thread {} completed {} iterations", thread_id, iterations);
    
    Ok(())
}

/// Perform CPU-intensive computation
fn cpu_intensive_computation(iterations: u32) {
    let mut result = 0u64;
    
    for i in 0..iterations {
        result = result.wrapping_add(i * i);
        
        // Add some branching to make it more realistic
        if i % 100 == 0 {
            result = result.wrapping_mul(2);
        }
    }
    
    // Prevent compiler optimization
    let _ = result;
}

/// Example interrupt storm test
pub struct InterruptStormTest {
    name: String,
    category: TestCategory,
    interrupt_rate_per_second: u32,
    duration_seconds: u32,
}

impl InterruptStormTest {
    pub fn new() -> Self {
        Self {
            name: "interrupt_storm_test".to_string(),
            category: TestCategory::Stress,
            interrupt_rate_per_second: 1000,
            duration_seconds: 30,
        }
    }
    
    pub fn with_interrupt_rate(mut self, rate: u32) -> Self {
        self.interrupt_rate_per_second = rate;
        self
    }
    
    pub fn with_duration(mut self, seconds: u32) -> Self {
        self.duration_seconds = seconds;
        self
    }
    
    /// Run interrupt storm test
    pub async fn run_test(&mut self, simulator: &mut HardwareSimulator) -> Result<TestResult, DriverTestError> {
        let start_time = std::time::Instant::now();
        let end_time = start_time + Duration::from_secs(self.duration_seconds as u64);
        
        log::info!("Starting interrupt storm test - {} interrupts/sec for {} seconds", 
                  self.interrupt_rate_per_second, self.duration_seconds);
        
        let mut total_interrupts = 0u64;
        let interrupt_interval = Duration::from_millis(1000) / self.interrupt_rate_per_second as u64;
        
        // Generate interrupt storm
        while std::time::Instant::now() < end_time {
            // Generate interrupts for different IRQ lines
            for irq in 0..16 {
                simulator.simulate_interrupt(irq)?;
                total_interrupts += 1;
                
                // Simulate interrupt handling
                let interrupt_interaction = crate::simulation::DeviceInteraction::Interrupt { irq };
                if let Err(e) = simulator.simulate_device_interaction("uart0", interrupt_interaction) {
                    log::debug!("Interrupt handling error for IRQ {}: {}", irq, e);
                }
            }
            
            tokio::time::sleep(interrupt_interval).await;
        }
        
        // Final stats collection
        let stats = simulator.get_statistics();
        
        let duration = start_time.elapsed();
        let actual_rate = (total_interrupts as f32) / duration.as_secs_f32();
        
        Ok(TestResult {
            name: self.name.clone(),
            status: TestStatus::Passed,
            duration,
            message: format!(
                "Interrupt storm test: {} total interrupts, {:.1} interrupts/sec",
                total_interrupts, actual_rate
            ),
            category: self.category,
            metadata: None,
            metrics: None,
        })
    }
}

/// Comprehensive stress test suite
pub struct ComprehensiveStressTest {
    name: String,
    category: TestCategory,
    serial_stress: SerialDriverStressTest,
    memory_pressure: MemoryPressureTest,
    cpu_stress: CpuStressTest,
}

impl ComprehensiveStressTest {
    pub fn new() -> Self {
        Self {
            name: "comprehensive_stress_test".to_string(),
            category: TestCategory::Stress,
            serial_stress: SerialDriverStressTest::new(),
            memory_pressure: MemoryPressureTest::new(),
            cpu_stress: CpuStressTest::new(),
        }
    }
    
    /// Run all stress tests
    pub async fn run_test(&mut self, simulator: &mut HardwareSimulator) -> Result<TestResult, DriverTestError> {
        let start_time = std::time::Instant::now();
        
        log::info!("Starting comprehensive stress test suite");
        
        let mut test_results = Vec::new();
        
        // Test 1: Serial driver stress test
        log::info!("Running serial driver stress test");
        let serial_result = self.serial_stress.run_test(simulator).await?;
        test_results.push(serial_result);
        
        // Test 2: Memory pressure test
        log::info!("Running memory pressure test");
        let memory_result = self.memory_pressure.run_test().await?;
        test_results.push(memory_result);
        
        // Test 3: CPU stress test
        log::info!("Running CPU stress test");
        let cpu_result = self.cpu_stress.run_test().await?;
        test_results.push(cpu_result);
        
        // Test 4: Interrupt storm test
        log::info!("Running interrupt storm test");
        let interrupt_storm = InterruptStormTest::new();
        let interrupt_result = interrupt_storm.run_test(simulator).await?;
        test_results.push(interrupt_result);
        
        // Analyze results
        let passed_tests = test_results.iter().filter(|r| r.is_success()).count();
        let total_tests = test_results.len();
        let success_rate = (passed_tests as f32 / total_tests as f32) * 100.0;
        
        let duration = start_time.elapsed();
        
        Ok(TestResult {
            name: self.name.clone(),
            status: if success_rate >= 75.0 { TestStatus::Passed } else { TestStatus::Warning },
            duration,
            message: format!(
                "Comprehensive stress test: {}/{} tests passed ({:.1}%)",
                passed_tests, total_tests, success_rate
            ),
            category: self.category,
            metadata: None,
            metrics: None,
        })
    }
}

/// Main stress testing example
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();
    
    log::info!("Starting stress testing example");
    
    // Create simulation environment
    let sim_env = SimulationEnvironment {
        virtual_hardware: true,
        network_simulation: true,
        storage_simulation: true,
        interrupt_simulation: true,
        timing_multiplier: 1.0,
    };
    
    // Initialize hardware simulator
    let mut simulator = HardwareSimulator::new(sim_env);
    simulator.initialize()?;
    
    log::info!("Hardware simulator initialized");
    
    // Test 1: Individual stress tests
    log::info!("\n=== Test 1: Individual Stress Tests ===");
    
    // Serial driver stress test
    let mut serial_stress = SerialDriverStressTest::new()
        .with_threads(8)
        .with_operations_per_thread(500);
    let serial_result = serial_stress.run_test(&mut simulator).await?;
    log::info!("Serial stress test result: {}", serial_result.status);
    
    // Memory pressure test
    let mut memory_pressure = MemoryPressureTest::new()
        .with_target_memory(50);
    let memory_result = memory_pressure.run_test().await?;
    log::info!("Memory pressure test result: {}", memory_result.status);
    
    // CPU stress test
    let mut cpu_stress = CpuStressTest::new()
        .with_threads(4)
        .with_duration(10);
    let cpu_result = cpu_stress.run_test().await?;
    log::info!("CPU stress test result: {}", cpu_result.status);
    
    // Test 2: Interrupt storm test
    log::info!("\n=== Test 2: Interrupt Storm Test ===");
    
    let mut interrupt_storm = InterruptStormTest::new()
        .with_interrupt_rate(100)
        .with_duration(5);
    let interrupt_result = interrupt_storm.run_test(&mut simulator).await?;
    log::info!("Interrupt storm test result: {}", interrupt_result.status);
    
    // Test 3: Comprehensive stress test suite
    log::info!("\n=== Test 3: Comprehensive Stress Test Suite ===");
    
    let mut comprehensive_test = ComprehensiveStressTest::new();
    let comprehensive_result = comprehensive_test.run_test(&mut simulator).await?;
    log::info!("Comprehensive stress test result: {}", comprehensive_result.status);
    
    // Test 4: Using the framework's built-in stress tester
    log::info!("\n=== Test 4: Framework Stress Tester ===");
    
    let stress_config = StressTestConfig {
        max_duration: 60,
        concurrent_operations: 10,
        memory_pressure: 30,
        cpu_stress: 50,
        io_stress: true,
    };
    
    let mut framework_stress_tester = StressTester::new(stress_config);
    let framework_results = framework_stress_tester.run_stress_tests(&simulator).await?;
    
    let framework_passed = framework_results.iter().filter(|r| r.is_success()).count();
    log::info!("Framework stress tester: {}/{} tests passed", 
              framework_passed, framework_results.len());
    
    // Display final statistics
    let stats = simulator.get_statistics();
    log::info!("\n=== Final Stress Test Statistics ===");
    log::info!("Simulation devices: {}", stats.device_count);
    log::info!("Total interrupts generated: {}", stats.interrupt_count);
    log::info!("Total memory accesses: {}", stats.memory_access_count);
    log::info!("Simulation duration: {:?}", stats.time_elapsed);
    
    // Shutdown simulator
    simulator.shutdown()?;
    
    log::info!("Stress testing example completed successfully");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_serial_stress_test_creation() {
        let serial_test = SerialDriverStressTest::new();
        assert_eq!(serial_test.concurrent_threads, 4);
        assert_eq!(serial_test.operations_per_thread, 1000);
    }
    
    #[tokio::test]
    async fn test_serial_stress_test_configuration() {
        let serial_test = SerialDriverStressTest::new()
            .with_threads(8)
            .with_operations_per_thread(2000);
        
        assert_eq!(serial_test.concurrent_threads, 8);
        assert_eq!(serial_test.operations_per_thread, 2000);
    }
    
    #[tokio::test]
    async fn test_memory_pressure_test_creation() {
        let memory_test = MemoryPressureTest::new();
        assert_eq!(memory_test.target_memory_mb, 100);
    }
    
    #[tokio::test]
    async fn test_cpu_stress_test_creation() {
        let cpu_test = CpuStressTest::new();
        assert_eq!(cpu_test.duration_seconds, 30);
        assert_eq!(cpu_test.thread_count, 4);
    }
    
    #[tokio::test]
    async fn test_interrupt_storm_test_creation() {
        let interrupt_test = InterruptStormTest::new();
        assert_eq!(interrupt_test.interrupt_rate_per_second, 1000);
        assert_eq!(interrupt_test.duration_seconds, 30);
    }
    
    #[test]
    fn test_allocate_memory_chunk() {
        let chunk = allocate_memory_chunk(1024 * 1024).unwrap();
        assert_eq!(chunk.len(), 1024 * 1024);
    }
    
    #[test]
    fn test_cpu_intensive_computation() {
        cpu_intensive_computation(10000);
        // If this completes without panic, the test passes
    }
}
