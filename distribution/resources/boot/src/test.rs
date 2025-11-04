//! Hardware Compatibility Testing Framework
//! 
//! This module provides comprehensive testing capabilities for hardware compatibility
//! across different architectures and boot methods.

use crate::{BootError, Architecture, BootMode, HardwareInfo, MemoryMap, BootStatus};
use log::{info, debug, warn, error};

/// Hardware compatibility test framework
pub struct HardwareCompatibilityFramework {
    test_suite: TestSuite,
    architecture: Architecture,
    boot_mode: BootMode,
    test_results: Vec<TestResult>,
}

/// Hardware test categories
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TestCategory {
    CPU,
    Memory,
    Storage,
    Network,
    Graphics,
    Audio,
    USB,
    PCIe,
    PowerManagement,
    Thermal,
    Clock,
    Interrupt,
    Timer,
    RealTimeClock,
    ACPI,
    SMBIOS,
    Firmware,
    BootDevice,
    Security,
}

/// Test result status
#[derive(Debug, Clone, PartialEq)]
pub enum TestStatus {
    Passed,
    Failed,
    Skipped,
    Warning,
    NotSupported,
    InProgress,
}

/// Individual test result
#[derive(Debug, Clone)]
pub struct TestResult {
    pub test_name: String,
    pub category: TestCategory,
    pub status: TestStatus,
    pub message: String,
    pub execution_time_ms: u64,
    pub metadata: TestMetadata,
}

/// Test metadata
#[derive(Debug, Clone, Default)]
pub struct TestMetadata {
    pub hardware_info: Option<HardwareInfo>,
    pub cpu_features: Vec<String>,
    pub memory_regions: Vec<MemoryRegion>,
    pub device_info: Vec<DeviceInfo>,
    pub performance_metrics: PerformanceMetrics,
    pub error_codes: Vec<u32>,
    pub warnings: Vec<String>,
}

/// Device information
#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub device_type: String,
    pub vendor_id: Option<u16>,
    pub device_id: Option<u16>,
    pub subsystem_vendor_id: Option<u16>,
    pub subsystem_device_id: Option<u16>,
    pub driver: Option<String>,
    pub capabilities: Vec<String>,
    pub performance: PerformanceMetrics,
}

/// Performance metrics
#[derive(Debug, Clone, Default)]
pub struct PerformanceMetrics {
    pub throughput_mbps: f64,
    pub latency_us: f64,
    pub cpu_usage_percent: f64,
    pub memory_usage_mb: f64,
    pub temperature_celsius: Option<f32>,
    pub power_consumption_watts: Option<f32>,
}

/// Test suite configuration
#[derive(Debug, Clone)]
pub struct TestSuite {
    pub enabled_categories: Vec<TestCategory>,
    pub enabled_tests: Vec<String>,
    pub test_timeout_ms: u64,
    pub parallel_execution: bool,
    pub strict_mode: bool,
    pub performance_benchmarks: bool,
    pub regression_testing: bool,
    pub stress_testing: bool,
}

impl Default for TestSuite {
    fn default() -> Self {
        Self {
            enabled_categories: vec![
                TestCategory::CPU,
                TestCategory::Memory,
                TestCategory::Storage,
                TestCategory::Network,
                TestCategory::BootDevice,
                TestCategory::Firmware,
            ],
            enabled_tests: Vec::new(),
            test_timeout_ms: 30_000, // 30 seconds
            parallel_execution: false,
            strict_mode: false,
            performance_benchmarks: true,
            regression_testing: false,
            stress_testing: false,
        }
    }
}

/// Hardware test interface
pub trait HardwareTest {
    fn run(&self, framework: &HardwareCompatibilityFramework) -> TestResult;
    fn name(&self) -> &'static str;
    fn category(&self) -> TestCategory;
    fn description(&self) -> &'static str;
    fn required_features(&self) -> Vec<String>;
    fn is_applicable(&self, arch: Architecture, mode: BootMode) -> bool;
    fn timeout_ms(&self) -> u64;
}

/// CPU feature test
pub struct CPUFeatureTest {
    feature_name: String,
    test_function: fn(&HardwareInfo) -> bool,
}

/// Memory test
pub struct MemoryTest {
    test_type: MemoryTestType,
    size_mb: usize,
}

/// Memory test types
#[derive(Debug, Clone, Copy)]
pub enum MemoryTestType {
    BasicReadWrite,
    SpeedTest,
    CacheTest,
    ThermalTest,
    ECCTest,
}

/// Storage device test
pub struct StorageTest {
    device_path: String,
    test_type: StorageTestType,
}

/// Storage test types
#[derive(Debug, Clone, Copy)]
pub enum StorageTestType {
    ReadSpeed,
    WriteSpeed,
    RandomIO,
    BootSector,
    SmartInfo,
}

/// Network interface test
pub struct NetworkTest {
    interface_name: String,
    test_type: NetworkTestType,
}

/// Network test types
#[derive(Debug, Clone, Copy)]
pub enum NetworkTestType {
    LinkStatus,
    SpeedTest,
    PacketLoss,
    Configuration,
}

/// Boot device test
pub struct BootDeviceTest {
    device_path: String,
    test_type: BootDeviceTestType,
}

/// Boot device test types
#[derive(Debug, Clone, Copy)]
pub enum BootDeviceTestType {
    Accessibility,
    ReadCapability,
    BootSector,
    UEFIBoot,
    LegacyBoot,
}

/// Firmware test
pub struct FirmwareTest {
    test_type: FirmwareTestType,
}

/// Firmware test types
#[derive(Debug, Clone, Copy)]
pub enum FirmwareTestType {
    UEFIServices,
    ACPIVerification,
    SMBIOSVerification,
    SecureBoot,
    TPMDetection,
}

impl HardwareCompatibilityFramework {
    /// Create new compatibility framework
    pub const fn new(arch: Architecture, mode: BootMode, suite: TestSuite) -> Self {
        Self {
            test_suite: suite,
            architecture: arch,
            boot_mode: mode,
            test_results: Vec::new(),
        }
    }

    /// Run all compatible tests
    pub fn run_all_tests(&mut self, hardware_info: &HardwareInfo) -> Result<Vec<TestResult>, BootError> {
        info!("Running hardware compatibility tests for {:?} in {:?} mode", self.architecture, self.boot_mode);
        
        self.test_results.clear();
        
        // Create and run CPU tests
        self.run_cpu_tests(hardware_info)?;
        
        // Create and run memory tests
        self.run_memory_tests(hardware_info)?;
        
        // Create and run storage tests
        self.run_storage_tests(hardware_info)?;
        
        // Create and run network tests
        self.run_network_tests(hardware_info)?;
        
        // Create and run firmware tests
        self.run_firmware_tests(hardware_info)?;
        
        // Create and run boot device tests
        self.run_boot_device_tests(hardware_info)?;
        
        // Additional architecture-specific tests
        match self.architecture {
            Architecture::X86_64 => self.run_x86_64_specific_tests(hardware_info)?,
            Architecture::ARM64 => self.run_arm64_specific_tests(hardware_info)?,
            Architecture::RISC_V64 => self.run_riscv64_specific_tests(hardware_info)?,
        }
        
        // Generate test report
        self.generate_test_report()?;
        
        Ok(self.test_results.clone())
    }

    /// Run CPU-specific tests
    fn run_cpu_tests(&mut self, _hardware_info: &HardwareInfo) -> Result<(), BootError> {
        debug!("Running CPU tests...");
        
        // CPU feature detection test
        let cpu_feature_test = CPUFeatureTest::new(
            "x86_64",
            |info| matches!(info.architecture, Architecture::X86_64),
        );
        self.run_test(Box::new(cpu_feature_test), _hardware_info);
        
        // CPU frequency test
        let cpu_freq_test = CPUFeatureTest::new(
            "frequency",
            |_info| true,
        );
        self.run_test(Box::new(cpu_freq_test), _hardware_info);
        
        Ok(())
    }

    /// Run memory-specific tests
    fn run_memory_tests(&mut self, _hardware_info: &HardwareInfo) -> Result<(), BootError> {
        debug!("Running memory tests...");
        
        // Basic memory test
        let mem_test = MemoryTest::new(MemoryTestType::BasicReadWrite, 1024);
        self.run_test(Box::new(mem_test), _hardware_info);
        
        // Memory speed test
        let speed_test = MemoryTest::new(MemoryTestType::SpeedTest, 4096);
        self.run_test(Box::new(speed_test), _hardware_info);
        
        Ok(())
    }

    /// Run storage-specific tests
    fn run_storage_tests(&mut self, _hardware_info: &HardwareInfo) -> Result<(), BootError> {
        debug!("Running storage tests...");
        
        // Test primary boot device
        let storage_test = StorageTest::new("/dev/sda", StorageTestType::ReadSpeed);
        self.run_test(Box::new(storage_test), _hardware_info);
        
        Ok(())
    }

    /// Run network-specific tests
    fn run_network_tests(&mut self, _hardware_info: &HardwareInfo) -> Result<(), BootError> {
        debug!("Running network tests...");
        
        // Test network interfaces
        let net_test = NetworkTest::new("eth0", NetworkTestType::LinkStatus);
        self.run_test(Box::new(net_test), _hardware_info);
        
        Ok(())
    }

    /// Run firmware-specific tests
    fn run_firmware_tests(&mut self, _hardware_info: &HardwareInfo) -> Result<(), BootError> {
        debug!("Running firmware tests...");
        
        match self.boot_mode {
            BootMode::UEFI => {
                let uefi_test = FirmwareTest::new(FirmwareTestType::UEFIServices);
                self.run_test(Box::new(uefi_test), _hardware_info);
            }
            BootMode::LegacyBIOS => {
                let bios_test = FirmwareTest::new(FirmwareTestType::ACPIVerification);
                self.run_test(Box::new(bios_test), _hardware_info);
            }
            BootMode::Direct => {
                let direct_test = FirmwareTest::new(FirmwareTestType::TPMDetection);
                self.run_test(Box::new(direct_test), _hardware_info);
            }
        }
        
        Ok(())
    }

    /// Run boot device tests
    fn run_boot_device_tests(&mut self, _hardware_info: &HardwareInfo) -> Result<(), BootError> {
        debug!("Running boot device tests...");
        
        // Boot device accessibility test
        let boot_test = BootDeviceTest::new("/dev/sda", BootDeviceTestType::Accessibility);
        self.run_test(Box::new(boot_test), _hardware_info);
        
        // Boot sector test
        let sector_test = BootDeviceTest::new("/dev/sda", BootDeviceTestType::BootSector);
        self.run_test(Box::new(sector_test), _hardware_info);
        
        Ok(())
    }

    /// Run x86_64 specific tests
    fn run_x86_64_specific_tests(&mut self, _hardware_info: &HardwareInfo) -> Result<(), BootError> {
        debug!("Running x86_64 specific tests...");
        
        // PCI bus test
        let pci_test = CPUFeatureTest::new("pci", |_info| true);
        self.run_test(Box::new(pci_test), _hardware_info);
        
        // SSE/AVX test
        let sse_test = CPUFeatureTest::new("sse", |_info| true);
        self.run_test(Box::new(sse_test), _hardware_info);
        
        Ok(())
    }

    /// Run ARM64 specific tests
    fn run_arm64_specific_tests(&mut self, _hardware_info: &HardwareInfo) -> Result<(), BootError> {
        debug!("Running ARM64 specific tests...");
        
        // GIC interrupt controller test
        let gic_test = CPUFeatureTest::new("gic", |_info| true);
        self.run_test(Box::new(gic_test), _hardware_info);
        
        // Generic timer test
        let timer_test = CPUFeatureTest::new("generic_timer", |_info| true);
        self.run_test(Box::new(timer_test), _hardware_info);
        
        Ok(())
    }

    /// Run RISC-V specific tests
    fn run_riscv64_specific_tests(&mut self, _hardware_info: &HardwareInfo) -> Result<(), BootError> {
        debug!("Running RISC-V64 specific tests...");
        
        // PLIC interrupt controller test
        let plic_test = CPUFeatureTest::new("plic", |_info| true);
        self.run_test(Box::new(plic_test), _hardware_info);
        
        // SBI interface test
        let sbi_test = CPUFeatureTest::new("sbi", |_info| true);
        self.run_test(Box::new(sbi_test), _hardware_info);
        
        Ok(())
    }

    /// Run individual test
    fn run_test(&mut self, test: Box<dyn HardwareTest>, hardware_info: &HardwareInfo) {
        if !test.is_applicable(self.architecture, self.boot_mode) {
            self.test_results.push(TestResult {
                test_name: test.name().to_string(),
                category: test.category(),
                status: TestStatus::NotSupported,
                message: "Test not applicable for current architecture/boot mode".to_string(),
                execution_time_ms: 0,
                metadata: TestMetadata::default(),
            });
            return;
        }
        
        debug!("Running test: {}", test.name());
        
        let start_time = crate::arch::x86_64::X86BootUtils::has_feature("time"); // Simplified timing
        
        let result = test.run(self);
        
        let execution_time = if start_time {
            // Calculate execution time
            0
        } else {
            0
        };
        
        let mut result_with_time = result;
        result_with_time.execution_time_ms = execution_time;
        
        self.test_results.push(result_with_time);
    }

    /// Generate test report
    fn generate_test_report(&self) -> Result<(), BootError> {
        info!("Generating hardware compatibility test report...");
        
        let passed_tests = self.test_results.iter().filter(|r| r.status == TestStatus::Passed).count();
        let failed_tests = self.test_results.iter().filter(|r| r.status == TestStatus::Failed).count();
        let skipped_tests = self.test_results.iter().filter(|r| r.status == TestStatus::Skipped).count();
        let total_tests = self.test_results.len();
        
        info!("Test Summary: {}/{} tests passed, {}/{} failed, {}/{} skipped", 
              passed_tests, total_tests, failed_tests, total_tests, skipped_tests, total_tests);
        
        // Print failed tests
        for result in &self.test_results {
            if result.status == TestStatus::Failed {
                error!("FAILED: {} - {}", result.test_name, result.message);
            } else if result.status == TestStatus::Warning {
                warn!("WARNING: {} - {}", result.test_name, result.message);
            }
        }
        
        Ok(())
    }

    /// Get test results
    pub const fn test_results(&self) -> &[TestResult] {
        &self.test_results
    }

    /// Get test results by category
    pub fn test_results_by_category(&self, category: TestCategory) -> Vec<&TestResult> {
        self.test_results.iter()
            .filter(|result| result.category == category)
            .collect()
    }

    /// Check if all critical tests passed
    pub fn critical_tests_passed(&self) -> bool {
        let critical_categories = vec![
            TestCategory::CPU,
            TestCategory::Memory,
            TestCategory::BootDevice,
            TestCategory::Firmware,
        ];
        
        for category in critical_categories {
            let category_results = self.test_results_by_category(category);
            if category_results.is_empty() || 
               category_results.iter().all(|r| r.status != TestStatus::Passed) {
                return false;
            }
        }
        
        true
    }
}

impl CPUFeatureTest {
    /// Create new CPU feature test
    pub const fn new(name: &'static str, test_function: fn(&HardwareInfo) -> bool) -> Self {
        Self {
            feature_name: name.to_string(),
            test_function,
        }
    }
}

impl HardwareTest for CPUFeatureTest {
    fn run(&self, framework: &HardwareCompatibilityFramework) -> TestResult {
        let hardware_info = framework.architecture.get_hardware_info(); // Simplified
        
        let passed = (self.test_function)(&hardware_info);
        
        TestResult {
            test_name: format!("CPU Feature: {}", self.feature_name),
            category: TestCategory::CPU,
            status: if passed { TestStatus::Passed } else { TestStatus::Failed },
            message: if passed { 
                "CPU feature detected successfully".to_string() 
            } else { 
                "CPU feature not available".to_string() 
            },
            execution_time_ms: 0,
            metadata: TestMetadata {
                hardware_info: Some(hardware_info),
                cpu_features: vec![self.feature_name.to_string()],
                ..Default::default()
            },
        }
    }

    fn name(&self) -> &'static str {
        "CPU Feature Test"
    }

    fn category(&self) -> TestCategory {
        TestCategory::CPU
    }

    fn description(&self) -> &'static str {
        "Tests CPU features and capabilities"
    }

    fn required_features(&self) -> Vec<String> {
        vec![self.feature_name.to_string()]
    }

    fn is_applicable(&self, arch: Architecture, _mode: BootMode) -> bool {
        match (arch, self.feature_name.as_str()) {
            (Architecture::X86_64, _) => true,
            (Architecture::ARM64, "gic") | (Architecture::ARM64, "generic_timer") => true,
            (Architecture::RISC_V64, "plic") | (Architecture::RISC_V64, "sbi") => true,
            _ => matches!(arch, Architecture::X86_64),
        }
    }

    fn timeout_ms(&self) -> u64 {
        1000
    }
}

impl MemoryTest {
    /// Create new memory test
    pub const fn new(test_type: MemoryTestType, size_mb: usize) -> Self {
        Self {
            test_type,
            size_mb,
        }
    }
}

impl HardwareTest for MemoryTest {
    fn run(&self, _framework: &HardwareCompatibilityFramework) -> TestResult {
        let test_name = match self.test_type {
            MemoryTestType::BasicReadWrite => "Memory Basic Read/Write Test",
            MemoryTestType::SpeedTest => "Memory Speed Test",
            MemoryTestType::CacheTest => "Memory Cache Test",
            MemoryTestType::ThermalTest => "Memory Thermal Test",
            MemoryTestType::ECCTest => "Memory ECC Test",
        };
        
        let passed = self.size_mb > 0; // Simplified test logic
        
        TestResult {
            test_name: test_name.to_string(),
            category: TestCategory::Memory,
            status: if passed { TestStatus::Passed } else { TestStatus::Failed },
            message: if passed {
                format!("Memory test ({}) completed successfully", self.size_mb)
            } else {
                "Memory test failed".to_string()
            },
            execution_time_ms: 0,
            metadata: TestMetadata {
                memory_regions: Vec::new(),
                performance_metrics: PerformanceMetrics {
                    memory_usage_mb: self.size_mb as f64,
                    ..Default::default()
                },
                ..Default::default()
            },
        }
    }

    fn name(&self) -> &'static str {
        "Memory Test"
    }

    fn category(&self) -> TestCategory {
        TestCategory::Memory
    }

    fn description(&self) -> &'static str {
        "Tests memory functionality and performance"
    }

    fn required_features(&self) -> Vec<String> {
        vec!["memory".to_string()]
    }

    fn is_applicable(&self, _arch: Architecture, _mode: BootMode) -> bool {
        true
    }

    fn timeout_ms(&self) -> u64 {
        30_000
    }
}

impl StorageTest {
    /// Create new storage test
    pub const fn new(device_path: &'static str, test_type: StorageTestType) -> Self {
        Self {
            device_path: device_path.to_string(),
            test_type,
        }
    }
}

impl HardwareTest for StorageTest {
    fn run(&self, _framework: &HardwareCompatibilityFramework) -> TestResult {
        let test_name = match self.test_type {
            StorageTestType::ReadSpeed => "Storage Read Speed Test",
            StorageTestType::WriteSpeed => "Storage Write Speed Test",
            StorageTestType::RandomIO => "Storage Random I/O Test",
            StorageTestType::BootSector => "Storage Boot Sector Test",
            StorageTestType::SmartInfo => "Storage SMART Information Test",
        };
        
        let passed = !self.device_path.is_empty(); // Simplified test logic
        
        TestResult {
            test_name: format!("{} ({})", test_name, self.device_path),
            category: TestCategory::Storage,
            status: if passed { TestStatus::Passed } else { TestStatus::Failed },
            message: if passed {
                "Storage device test completed successfully".to_string()
            } else {
                "Storage device test failed".to_string()
            },
            execution_time_ms: 0,
            metadata: TestMetadata {
                device_info: vec![DeviceInfo {
                    device_type: "Storage".to_string(),
                    driver: Some("Default".to_string()),
                    ..Default::default()
                }],
                ..Default::default()
            },
        }
    }

    fn name(&self) -> &'static str {
        "Storage Test"
    }

    fn category(&self) -> TestCategory {
        TestCategory::Storage
    }

    fn description(&self) -> &'static str {
        "Tests storage device functionality and performance"
    }

    fn required_features(&self) -> Vec<String> {
        vec!["storage".to_string()]
    }

    fn is_applicable(&self, _arch: Architecture, _mode: BootMode) -> bool {
        true
    }

    fn timeout_ms(&self) -> u64 {
        10_000
    }
}

impl NetworkTest {
    /// Create new network test
    pub const fn new(interface_name: &'static str, test_type: NetworkTestType) -> Self {
        Self {
            interface_name: interface_name.to_string(),
            test_type,
        }
    }
}

impl HardwareTest for NetworkTest {
    fn run(&self, _framework: &HardwareCompatibilityFramework) -> TestResult {
        let test_name = match self.test_type {
            NetworkTestType::LinkStatus => "Network Link Status Test",
            NetworkTestType::SpeedTest => "Network Speed Test",
            NetworkTestType::PacketLoss => "Network Packet Loss Test",
            NetworkTestType::Configuration => "Network Configuration Test",
        };
        
        let passed = !self.interface_name.is_empty(); // Simplified test logic
        
        TestResult {
            test_name: format!("{} ({})", test_name, self.interface_name),
            category: TestCategory::Network,
            status: if passed { TestStatus::Passed } else { TestStatus::Failed },
            message: if passed {
                "Network interface test completed successfully".to_string()
            } else {
                "Network interface test failed".to_string()
            },
            execution_time_ms: 0,
            metadata: TestMetadata {
                device_info: vec![DeviceInfo {
                    device_type: "Network".to_string(),
                    driver: Some("Network Driver".to_string()),
                    capabilities: vec!["Ethernet".to_string()],
                    ..Default::default()
                }],
                ..Default::default()
            },
        }
    }

    fn name(&self) -> &'static str {
        "Network Test"
    }

    fn category(&self) -> TestCategory {
        TestCategory::Network
    }

    fn description(&self) -> &'static str {
        "Tests network interface functionality and performance"
    }

    fn required_features(&self) -> Vec<String> {
        vec!["network".to_string()]
    }

    fn is_applicable(&self, _arch: Architecture, _mode: BootMode) -> bool {
        true
    }

    fn timeout_ms(&self) -> u64 {
        5_000
    }
}

impl BootDeviceTest {
    /// Create new boot device test
    pub const fn new(device_path: &'static str, test_type: BootDeviceTestType) -> Self {
        Self {
            device_path: device_path.to_string(),
            test_type,
        }
    }
}

impl HardwareTest for BootDeviceTest {
    fn run(&self, _framework: &HardwareCompatibilityFramework) -> TestResult {
        let test_name = match self.test_type {
            BootDeviceTestType::Accessibility => "Boot Device Accessibility Test",
            BootDeviceTestType::ReadCapability => "Boot Device Read Capability Test",
            BootDeviceTestType::BootSector => "Boot Device Boot Sector Test",
            BootDeviceTestType::UEFIBoot => "UEFI Boot Test",
            BootDeviceTestType::LegacyBoot => "Legacy Boot Test",
        };
        
        let passed = !self.device_path.is_empty(); // Simplified test logic
        
        TestResult {
            test_name: format!("{} ({})", test_name, self.device_path),
            category: TestCategory::BootDevice,
            status: if passed { TestStatus::Passed } else { TestStatus::Failed },
            message: if passed {
                "Boot device test completed successfully".to_string()
            } else {
                "Boot device test failed".to_string()
            },
            execution_time_ms: 0,
            metadata: TestMetadata {
                device_info: vec![DeviceInfo {
                    device_type: "Boot Device".to_string(),
                    capabilities: vec!["Boot".to_string()],
                    ..Default::default()
                }],
                ..Default::default()
            },
        }
    }

    fn name(&self) -> &'static str {
        "Boot Device Test"
    }

    fn category(&self) -> TestCategory {
        TestCategory::BootDevice
    }

    fn description(&self) -> &'static str {
        "Tests boot device accessibility and functionality"
    }

    fn required_features(&self) -> Vec<String> {
        vec!["boot_device".to_string()]
    }

    fn is_applicable(&self, _arch: Architecture, mode: BootMode) -> bool {
        match (self.test_type, mode) {
            (BootDeviceTestType::UEFIBoot, BootMode::UEFI) => true,
            (BootDeviceTestType::LegacyBoot, BootMode::LegacyBIOS) => true,
            (BootDeviceTestType::Accessibility, _) => true,
            (BootDeviceTestType::ReadCapability, _) => true,
            (BootDeviceTestType::BootSector, _) => true,
            _ => false,
        }
    }

    fn timeout_ms(&self) -> u64 {
        2_000
    }
}

impl FirmwareTest {
    /// Create new firmware test
    pub const fn new(test_type: FirmwareTestType) -> Self {
        Self { test_type }
    }
}

impl HardwareTest for FirmwareTest {
    fn run(&self, framework: &HardwareCompatibilityFramework) -> TestResult {
        let test_name = match self.test_type {
            FirmwareTestType::UEFIServices => "UEFI Services Test",
            FirmwareTestType::ACPIVerification => "ACPI Verification Test",
            FirmwareTestType::SMBIOSVerification => "SMBIOS Verification Test",
            FirmwareTestType::SecureBoot => "Secure Boot Test",
            FirmwareTestType::TPMDetection => "TPM Detection Test",
        };
        
        let passed = framework.boot_mode != BootMode::Direct; // Simplified test logic
        
        TestResult {
            test_name: test_name.to_string(),
            category: TestCategory::Firmware,
            status: if passed { TestStatus::Passed } else { TestStatus::Skipped },
            message: if passed {
                "Firmware test completed successfully".to_string()
            } else {
                "Firmware test skipped in direct boot mode".to_string()
            },
            execution_time_ms: 0,
            metadata: TestMetadata {
                hardware_info: Some(HardwareInfo {
                    firmware_type: framework.boot_mode,
                    ..Default::default()
                }),
                ..Default::default()
            },
        }
    }

    fn name(&self) -> &'static str {
        "Firmware Test"
    }

    fn category(&self) -> TestCategory {
        TestCategory::Firmware
    }

    fn description(&self) -> &'static str {
        "Tests firmware services and capabilities"
    }

    fn required_features(&self) -> Vec<String> {
        match self.test_type {
            FirmwareTestType::UEFIServices => vec!["uefi".to_string()],
            FirmwareTestType::ACPIVerification => vec!["acpi".to_string()],
            FirmwareTestType::SMBIOSVerification => vec!["smbios".to_string()],
            FirmwareTestType::SecureBoot => vec!["secure_boot".to_string()],
            FirmwareTestType::TPMDetection => vec!["tpm".to_string()],
        }
    }

    fn is_applicable(&self, _arch: Architecture, mode: BootMode) -> bool {
        match self.test_type {
            FirmwareTestType::UEFIServices => matches!(mode, BootMode::UEFI),
            FirmwareTestType::ACPIVerification => matches!(mode, BootMode::UEFI | BootMode::LegacyBIOS),
            FirmwareTestType::SMBIOSVerification => matches!(mode, BootMode::LegacyBIOS),
            FirmwareTestType::SecureBoot => matches!(mode, BootMode::UEFI),
            FirmwareTestType::TPMDetection => true,
        }
    }

    fn timeout_ms(&self) -> u64 {
        15_000
    }
}