//! USB Framework Testing Tools
//!
//! This module provides comprehensive testing utilities for the MultiOS USB
//! framework, including unit tests, integration tests, and hardware validation
//! tools.

use crate::host::{XhciHost, EhciHost, OhciHost};
use crate::classes::{HidDevice, MscDevice, CdcDevice, AudioDevice};
use crate::hub::UsbHub;
use crate::hotplug::HotplugDetector;
use crate::power::{UsbPowerManager, PowerState};
use crate::security::{SecurityManager, SecurityLevel, DeviceFingerprint, TrustState};
use crate::protocol_analyzer::{ProtocolAnalyzer, DescriptorDecoder};
use crate::{UsbResult, UsbError};

/// Test result structure
#[derive(Debug, Clone)]
pub struct TestResult {
    pub test_name: String,
    pub passed: bool,
    pub message: String,
    pub duration_ms: u64,
}

/// Test suite manager
pub struct TestSuite {
    /// Test results
    results: Vec<TestResult>,
    /// Enable verbose output
    verbose: bool,
}

impl TestSuite {
    /// Create new test suite
    pub fn new(verbose: bool) -> Self {
        Self {
            results: Vec::new(),
            verbose,
        }
    }

    /// Add test result
    fn add_result(&mut self, test_name: &str, passed: bool, message: &str, duration_ms: u64) {
        self.results.push(TestResult {
            test_name: test_name.to_string(),
            passed,
            message: message.to_string(),
            duration_ms,
        });

        if self.verbose {
            println!("[{}] {}: {}",
                if passed { "PASS" } else { "FAIL" },
                test_name,
                message);
        }
    }

    /// Run host controller tests
    pub fn test_host_controllers(&mut self) -> UsbResult<()> {
        println!("Testing USB Host Controllers...");

        // Test xHCI controller
        let start_time = crate::timer::current_time_ms();
        match XhciHost::new() {
            Ok(_) => {
                let duration = crate::timer::current_time_ms() - start_time;
                self.add_result("xHCI Controller Initialization", true, 
                    "Controller initialized successfully", duration);
            }
            Err(e) => {
                let duration = crate::timer::current_time_ms() - start_time;
                self.add_result("xHCI Controller Initialization", false,
                    &format!("Initialization failed: {:?}", e), duration);
            }
        }

        // Test EHCI controller
        let start_time = crate::timer::current_time_ms();
        match EhciHost::new() {
            Ok(_) => {
                let duration = crate::timer::current_time_ms() - start_time;
                self.add_result("EHCI Controller Initialization", true,
                    "Controller initialized successfully", duration);
            }
            Err(e) => {
                let duration = crate::timer::current_time_ms() - start_time;
                self.add_result("EHCI Controller Initialization", false,
                    &format!("Initialization failed: {:?}", e), duration);
            }
        }

        // Test OHCI controller
        let start_time = crate::timer::current_time_ms();
        match OhciHost::new() {
            Ok(_) => {
                let duration = crate::timer::current_time_ms() - start_time;
                self.add_result("OHCI Controller Initialization", true,
                    "Controller initialized successfully", duration);
            }
            Err(e) => {
                let duration = crate::timer::current_time_ms() - start_time;
                self.add_result("OHCI Controller Initialization", false,
                    &format!("Initialization failed: {:?}", e), duration);
            }
        }

        Ok(())
    }

    /// Run device class tests
    pub fn test_device_classes(&mut self) -> UsbResult<()> {
        println!("Testing USB Device Classes...");

        // Test HID device
        self.test_hid_device();

        // Test MSC device
        self.test_msc_device();

        // Test CDC device
        self.test_cdc_device();

        // Test Audio device
        self.test_audio_device();

        Ok(())
    }

    /// Test HID device class
    fn test_hid_device(&mut self) {
        let start_time = crate::timer::current_time_ms();

        // Test keyboard processing
        let mut hid_device = HidDevice::new(0x1234, 0x5678, 1);
        if let Err(e) = hid_device.initialize() {
            let duration = crate::timer::current_time_ms() - start_time;
            self.add_result("HID Device Initialization", false,
                &format!("Initialization failed: {:?}", e), duration);
            return;
        }

        // Test keyboard report parsing
        let keyboard_report = vec![0x00, 0x00, 0x04]; // 'a' key
        match hid_device.parse_input_report(&keyboard_report) {
            Ok(_) => {
                let duration = crate::timer::current_time_ms() - start_time;
                self.add_result("HID Keyboard Report Parsing", true,
                    "Keyboard report parsed successfully", duration);
            }
            Err(e) => {
                let duration = crate::timer::current_time_ms() - start_time;
                self.add_result("HID Keyboard Report Parsing", false,
                    &format!("Parsing failed: {:?}", e), duration);
            }
        }

        // Test mouse report parsing
        let mouse_report = vec![0x01, 0x10, 0x00, 0x20, 0x00]; // Left click, move right
        match hid_device.parse_input_report(&mouse_report) {
            Ok(_) => {
                let duration = crate::timer::current_time_ms() - start_time;
                self.add_result("HID Mouse Report Parsing", true,
                    "Mouse report parsed successfully", duration);
            }
            Err(e) => {
                let duration = crate::timer::current_time_ms() - start_time;
                self.add_result("HID Mouse Report Parsing", false,
                    &format!("Parsing failed: {:?}", e), duration);
            }
        }
    }

    /// Test MSC device class
    fn test_msc_device(&mut self) {
        let start_time = crate::timer::current_time_ms();

        let mut msc_device = MscDevice::new(0x1234, 0x5678, 1);
        if let Err(e) = msc_device.initialize() {
            let duration = crate::timer::current_time_ms() - start_time;
            self.add_result("MSC Device Initialization", false,
                &format!("Initialization failed: {:?}", e), duration);
            return;
        }

        // Test SCSI inquiry command
        let inquiry_cmd = vec![0x12, 0x00, 0x00, 0x00, 0x24, 0x00];
        match msc_device.execute_scsi_command(&inquiry_cmd, 36) {
            Ok(data) => {
                if data.len() >= 36 {
                    let duration = crate::timer::current_time_ms() - start_time;
                    self.add_result("MSC SCSI Inquiry Command", true,
                        "SCSI inquiry command executed successfully", duration);
                } else {
                    let duration = crate::timer::current_time_ms() - start_time;
                    self.add_result("MSC SCSI Inquiry Command", false,
                        "Insufficient data received", duration);
                }
            }
            Err(e) => {
                let duration = crate::timer::current_time_ms() - start_time;
                self.add_result("MSC SCSI Inquiry Command", false,
                    &format!("Command failed: {:?}", e), duration);
            }
        }

        // Test read capacity command
        let read_capacity_cmd = vec![0x25, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        match msc_device.execute_scsi_command(&read_capacity_cmd, 8) {
            Ok(_) => {
                let duration = crate::timer::current_time_ms() - start_time;
                self.add_result("MSC SCSI Read Capacity", true,
                    "Read capacity command executed successfully", duration);
            }
            Err(e) => {
                let duration = crate::timer::current_time_ms() - start_time;
                self.add_result("MSC SCSI Read Capacity", false,
                    &format!("Command failed: {:?}", e), duration);
            }
        }
    }

    /// Test CDC device class
    fn test_cdc_device(&mut self) {
        let start_time = crate::timer::current_time_ms();

        let mut cdc_device = CdcDevice::new(0x1234, 0x5678, 1, 2);
        if let Err(e) = cdc_device.initialize() {
            let duration = crate::timer::current_time_ms() - start_time;
            self.add_result("CDC Device Initialization", false,
                &format!("Initialization failed: {:?}", e), duration);
            return;
        }

        // Test line coding configuration
        let line_coding = cdc_device.create_line_coding(
            crate::classes::cdc::LineBaudRate::B115200,
            crate::classes::cdc::DataBits::Eight,
            crate::classes::cdc::StopBits::One,
            crate::classes::cdc::Parity::None,
        );

        if line_coding.baud_rate == 115200 {
            let duration = crate::timer::current_time_ms() - start_time;
            self.add_result("CDC Line Coding Configuration", true,
                "Line coding configured successfully", duration);
        } else {
            let duration = crate::timer::current_time_ms() - start_time;
            self.add_result("CDC Line Coding Configuration", false,
                "Baud rate not set correctly", duration);
        }

        // Test control line state
        match cdc_device.set_control_line_state(
            crate::classes::cdc::ControlLineState::DTR) {
            Ok(_) => {
                let duration = crate::timer::current_time_ms() - start_time;
                self.add_result("CDC Control Line State", true,
                    "Control line state set successfully", duration);
            }
            Err(e) => {
                let duration = crate::timer::current_time_ms() - start_time;
                self.add_result("CDC Control Line State", false,
                    &format!("Setting control lines failed: {:?}", e), duration);
            }
        }
    }

    /// Test Audio device class
    fn test_audio_device(&mut self) {
        let start_time = crate::timer::current_time_ms();

        let mut audio_device = AudioDevice::new(0x1234, 0x5678, 1);
        if let Err(e) = audio_device.initialize() {
            let duration = crate::timer::current_time_ms() - start_time;
            self.add_result("Audio Device Initialization", false,
                &format!("Initialization failed: {:?}", e), duration);
            return;
        }

        // Test audio format configuration
        let format = audio_device.create_audio_format(
            crate::classes::audio::AudioSampleRate::Hz44100,
            crate::classes::audio::AudioSampleBits::Sixteen,
            crate::classes::audio::AudioChannels::Stereo,
        );

        if format.sample_rate == 44100 && format.sample_bits == crate::classes::audio::AudioSampleBits::Sixteen {
            let duration = crate::timer::current_time_ms() - start_time;
            self.add_result("Audio Format Configuration", true,
                "Audio format configured successfully", duration);
        } else {
            let duration = crate::timer::current_time_ms() - start_time;
            self.add_result("Audio Format Configuration", false,
                "Audio format not set correctly", duration);
        }

        // Test volume control
        match audio_device.set_master_volume(0.5) {
            Ok(_) => {
                let duration = crate::timer::current_time_ms() - start_time;
                self.add_result("Audio Volume Control", true,
                    "Volume control set successfully", duration);
            }
            Err(e) => {
                let duration = crate::timer::current_time_ms() - start_time;
                self.add_result("Audio Volume Control", false,
                    &format!("Volume control failed: {:?}", e), duration);
            }
        }
    }

    /// Run security system tests
    pub fn test_security_system(&mut self) -> UsbResult<()> {
        println!("Testing USB Security System...");

        let start_time = crate::timer::current_time_ms();

        // Test security manager
        let mut security_manager = SecurityManager::new(SecurityLevel::Medium);
        
        // Test device fingerprint creation
        let fingerprint = DeviceFingerprint::new(0x1234, 0x5678, (0x03, 0x01, 0x01)); // HID keyboard
        let hash = fingerprint.hash();
        
        if hash != 0 {
            let duration = crate::timer::current_time_ms() - start_time;
            self.add_result("Security Device Fingerprint", true,
                &format!("Device fingerprint created (hash: {:016X})", hash), duration);
        } else {
            let duration = crate::timer::current_time_ms() - start_time;
            self.add_result("Security Device Fingerprint", false,
                "Device fingerprint hash failed", duration);
        }

        // Test device access check
        let trust_state = security_manager.check_device_access(&fingerprint)?;
        
        if trust_state == TrustState::Trusted || trust_state == TrustState::Verified {
            let duration = crate::timer::current_time_ms() - start_time;
            self.add_result("Security Access Control", true,
                &format!("Device access allowed (state: {:?})", trust_state), duration);
        } else {
            let duration = crate::timer::current_time_ms() - start_time;
            self.add_result("Security Access Control", false,
                &format!("Device access denied (state: {:?})", trust_state), duration);
        }

        // Test security policy
        let mut custom_policy = crate::security::SecurityPolicy::default();
        custom_policy.name = "Custom Test Policy".to_string();
        custom_policy.allow_storage = false;
        security_manager.add_policy(custom_policy);

        let report = security_manager.generate_security_report();
        if report.contains("Custom Test Policy") {
            let duration = crate::timer::current_time_ms() - start_time;
            self.add_result("Security Policy Management", true,
                "Security policy added and report generated", duration);
        } else {
            let duration = crate::timer::current_time_ms() - start_time;
            self.add_result("Security Policy Management", false,
                "Security policy not found in report", duration);
        }

        Ok(())
    }

    /// Run protocol analyzer tests
    pub fn test_protocol_analyzer(&mut self) -> UsbResult<()> {
        println!("Testing USB Protocol Analyzer...");

        let start_time = crate::timer::current_time_ms();

        // Test descriptor decoder
        let decoder = DescriptorDecoder::new();
        let device_descriptor = create_test_device_descriptor();
        
        match decoder.decode_descriptor(&device_descriptor) {
            Ok(decoded) => {
                if decoded.contains("Device Descriptor") && decoded.contains("bcdUSB") {
                    let duration = crate::timer::current_time_ms() - start_time;
                    self.add_result("Descriptor Decoding", true,
                        "Device descriptor decoded successfully", duration);
                } else {
                    let duration = crate::timer::current_time_ms() - start_time;
                    self.add_result("Descriptor Decoding", false,
                        "Descriptor decode missing expected content", duration);
                }
            }
            Err(e) => {
                let duration = crate::timer::current_time_ms() - start_time;
                self.add_result("Descriptor Decoding", false,
                    &format!("Descriptor decode failed: {:?}", e), duration);
            }
        }

        // Test protocol analyzer
        let mut analyzer = ProtocolAnalyzer::new();
        analyzer.set_capture_enabled(true);

        if analyzer.generate_analysis_report().contains("Total transactions captured: 0") {
            let duration = crate::timer::current_time_ms() - start_time;
            self.add_result("Protocol Analyzer Generation", true,
                "Protocol analysis report generated", duration);
        } else {
            let duration = crate::timer::current_time_ms() - start_time;
            self.add_result("Protocol Analyzer Generation", false,
                "Protocol analysis report generation failed", duration);
        }

        // Test educational mode
        let tutorial = analyzer.generate_tutorial("basic");
        if tutorial.contains("USB Protocol Analyzer Tutorial") {
            let duration = crate::timer::current_time_ms() - start_time;
            self.add_result("Educational Tutorial Generation", true,
                "Educational tutorial generated successfully", duration);
        } else {
            let duration = crate::timer::current_time_ms() - start_time;
            self.add_result("Educational Tutorial Generation", false,
                "Educational tutorial generation failed", duration);
        }

        Ok(())
    }

    /// Run power management tests
    pub fn test_power_management(&mut self) -> UsbResult<()> {
        println!("Testing USB Power Management...");

        let start_time = crate::timer::current_time_ms();

        let mut power_manager = UsbPowerManager::new();

        // Test power state management
        power_manager.set_power_state(PowerState::Suspended)?;
        
        if power_manager.get_power_state() == PowerState::Suspended {
            let duration = crate::timer::current_time_ms() - start_time;
            self.add_result("Power State Management", true,
                "Power state set and retrieved successfully", duration);
        } else {
            let duration = crate::timer::current_time_ms() - start_time;
            self.add_result("Power State Management", false,
                "Power state not set correctly", duration);
        }

        // Test power budgeting
        let budget_result = power_manager.calculate_power_budget(100, 500)?;
        if budget_result.remaining_capacity > 0 {
            let duration = crate::timer::current_time_ms() - start_time;
            self.add_result("Power Budget Calculation", true,
                &format!("Power budget calculated ({} mA remaining)", 
                    budget_result.remaining_capacity), duration);
        } else {
            let duration = crate::timer::current_time_ms() - start_time;
            self.add_result("Power Budget Calculation", false,
                "Power budget calculation failed", duration);
        }

        Ok(())
    }

    /// Run hub management tests
    pub fn test_hub_management(&mut self) -> UsbResult<()> {
        println!("Testing USB Hub Management...");

        let start_time = crate::timer::current_time_ms();

        let mut hub = UsbHub::new(1, 4); // 4-port hub

        // Test port management
        hub.initialize()?;
        
        // Simulate device connection on port 2
        hub.port_device_connected(2, 0x1234, 0x5678, crate::hub::UsbSpeed::HighSpeed)?;
        
        if hub.port_device_count(2) > 0 {
            let duration = crate::timer::current_time_ms() - start_time;
            self.add_result("Hub Port Management", true,
                "Port device connection handled successfully", duration);
        } else {
            let duration = crate::timer::current_time_ms() - start_time;
            self.add_result("Hub Port Management", false,
                "Port device connection failed", duration);
        }

        // Test power management
        hub.set_port_power(2, true)?;
        if hub.get_port_power(2)? {
            let duration = crate::timer::current_time_ms() - start_time;
            self.add_result("Hub Power Management", true,
                "Port power control working", duration);
        } else {
            let duration = crate::timer::current_time_ms() - start_time;
            self.add_result("Hub Power Management", false,
                "Port power control failed", duration);
        }

        Ok(())
    }

    /// Run hotplug detection tests
    pub fn test_hotplug_detection(&mut self) -> UsbResult<()> {
        println!("Testing USB Hotplug Detection...");

        let start_time = crate::timer::current_time_ms();

        let mut detector = HotplugDetector::new();

        // Test device insertion simulation
        let device_info = detector.simulate_device_insertion(
            0x1234, 0x5678, (0x03, 0x01, 0x01))?;
        
        if device_info.is_some() {
            let duration = crate::timer::current_time_ms() - start_time;
            self.add_result("Hotplug Device Detection", true,
                "Device insertion detected successfully", duration);
        } else {
            let duration = crate::timer::current_time_ms() - start_time;
            self.add_result("Hotplug Device Detection", false,
                "Device insertion not detected", duration);
        }

        // Test device removal simulation
        detector.simulate_device_removal(0x1234, 0x5678)?;
        
        if detector.get_active_device_count() == 0 {
            let duration = crate::timer::current_time_ms() - start_time;
            self.add_result("Hotplug Device Removal", true,
                "Device removal detected successfully", duration);
        } else {
            let duration = crate::timer::current_time_ms() - start_time;
            self.add_result("Hotplug Device Removal", false,
                "Device removal not detected", duration);
        }

        Ok(())
    }

    /// Generate test report
    pub fn generate_report(&self) -> String {
        let mut report = String::new();
        report.push_str("USB Framework Test Report\n");
        report.push_str("=========================\n\n");

        let total_tests = self.results.len();
        let passed_tests = self.results.iter().filter(|r| r.passed).count();
        let failed_tests = total_tests - passed_tests;
        let total_time: u64 = self.results.iter().map(|r| r.duration_ms).sum();

        report.push_str(&format!("Total Tests: {}\n", total_tests));
        report.push_str(&format!("Passed: {}\n", passed_tests));
        report.push_str(&format!("Failed: {}\n", failed_tests));
        report.push_str(&format!("Success Rate: {:.1}%\n", (passed_tests as f64 / total_tests as f64) * 100.0));
        report.push_str(&format!("Total Time: {} ms\n\n", total_time));

        if failed_tests > 0 {
            report.push_str("Failed Tests:\n");
            report.push_str("=============\n");
            for result in &self.results {
                if !result.passed {
                    report.push_str(&format!("✗ {}: {}\n", result.test_name, result.message));
                }
            }
            report.push_str("\n");
        }

        report.push_str("Test Details:\n");
        report.push_str("=============\n");
        for result in &self.results {
            let status = if result.passed { "✓" } else { "✗" };
            report.push_str(&format!("{} {}: {} ({} ms)\n", 
                status, result.test_name, result.message, result.duration_ms));
        }

        report
    }

    /// Get test statistics
    pub fn get_statistics(&self) -> (usize, usize, u64) {
        let total_tests = self.results.len();
        let passed_tests = self.results.iter().filter(|r| r.passed).count();
        let total_time: u64 = self.results.iter().map(|r| r.duration_ms).sum();
        (total_tests, passed_tests, total_time)
    }
}

/// Create test device descriptor
fn create_test_device_descriptor() -> Vec<u8> {
    vec![
        0x12,          // Descriptor length (18 bytes)
        0x01,          // Descriptor type (Device)
        0x02, 0x10,    // bcdUSB (USB 2.0)
        0x00,          // bDeviceClass
        0x00,          // bDeviceSubClass
        0x00,          // bDeviceProtocol
        0x08,          // bMaxPacketSize0 (8 bytes)
        0x34, 0x12,    // idVendor (0x1234)
        0x78, 0x56,    // idProduct (0x5678)
        0x01, 0x00,    // bcdDevice (1.00)
        0x01,          // iManufacturer
        0x02,          // iProduct
        0x03,          // iSerialNumber
        0x01,          // bNumConfigurations
    ]
}

/// Run comprehensive USB framework test suite
pub fn run_comprehensive_tests() -> UsbResult<()> {
    println!("MultiOS USB Framework - Comprehensive Test Suite");
    println!("================================================\n");

    let mut test_suite = TestSuite::new(true);

    // Run all test categories
    test_suite.test_host_controllers()?;
    test_suite.test_device_classes()?;
    test_suite.test_security_system()?;
    test_suite.test_protocol_analyzer()?;
    test_suite.test_power_management()?;
    test_suite.test_hub_management()?;
    test_suite.test_hotplug_detection()?;

    // Generate and display report
    println!("\n{}", test_suite.generate_report());

    let (total, passed, _) = test_suite.get_statistics();
    if passed == total {
        println!("🎉 All tests passed! USB framework is working correctly.");
    } else {
        println!("❌ {} out of {} tests failed. Please check the framework.", 
            total - passed, total);
    }

    Ok(())
}

/// Quick validation test for USB framework
pub fn quick_validation_test() -> UsbResult<bool> {
    println!("USB Framework Quick Validation");
    println!("==============================\n");

    let mut all_passed = true;

    // Test 1: Host controller initialization
    print!("Testing host controllers... ");
    let mut host_tests_passed = 0;
    if XhciHost::new().is_ok() { host_tests_passed += 1; }
    if EhciHost::new().is_ok() { host_tests_passed += 1; }
    if OhciHost::new().is_ok() { host_tests_passed += 1; }
    
    if host_tests_passed > 0 {
        println!("✓ ({}/3 controllers available)", host_tests_passed);
    } else {
        println!("✗ (No host controllers available)");
        all_passed = false;
    }

    // Test 2: Device class initialization
    print!("Testing device classes... ");
    let mut class_tests_passed = 0;
    
    if HidDevice::new(0x1234, 0x5678, 1).initialize().is_ok() { 
        class_tests_passed += 1; 
    }
    if MscDevice::new(0x1234, 0x5678, 1).initialize().is_ok() { 
        class_tests_passed += 1; 
    }
    if CdcDevice::new(0x1234, 0x5678, 1, 2).initialize().is_ok() { 
        class_tests_passed += 1; 
    }
    if AudioDevice::new(0x1234, 0x5678, 1).initialize().is_ok() { 
        class_tests_passed += 1; 
    }
    
    if class_tests_passed == 4 {
        println!("✓ (All device classes initialized)");
    } else {
        println!("✗ ({}/4 device classes initialized)", class_tests_passed);
        all_passed = false;
    }

    // Test 3: Security system
    print!("Testing security system... ");
    let mut security_manager = SecurityManager::new(SecurityLevel::Basic);
    let fingerprint = DeviceFingerprint::new(0x1234, 0x5678, (0x03, 0x01, 0x01));
    
    if security_manager.check_device_access(&fingerprint).is_ok() {
        println!("✓ (Security system functional)");
    } else {
        println!("✗ (Security system failed)");
        all_passed = false;
    }

    // Test 4: Protocol analyzer
    print!("Testing protocol analyzer... ");
    let mut analyzer = ProtocolAnalyzer::new();
    let decoder = DescriptorDecoder::new();
    let descriptor = create_test_device_descriptor();
    
    if decoder.decode_descriptor(&descriptor).is_ok() {
        println!("✓ (Protocol analyzer functional)");
    } else {
        println!("✗ (Protocol analyzer failed)");
        all_passed = false;
    }

    // Test 5: Power management
    print!("Testing power management... ");
    let mut power_manager = UsbPowerManager::new();
    
    if power_manager.calculate_power_budget(1, 500).is_ok() {
        println!("✓ (Power management functional)");
    } else {
        println!("✗ (Power management failed)");
        all_passed = false;
    }

    println!();
    if all_passed {
        println!("🎉 Quick validation passed! USB framework is ready.");
    } else {
        println!("❌ Quick validation failed. Some components need attention.");
    }

    Ok(all_passed)
}

/// Benchmark USB framework performance
pub fn benchmark_framework() -> UsbResult<()> {
    println!("USB Framework Performance Benchmark");
    println!("===================================\n");

    // Benchmark device class operations
    println!("Benchmarking device class operations...");
    
    // HID device benchmark
    let start_time = crate::timer::current_time_ms();
    let mut hid_device = HidDevice::new(0x1234, 0x5678, 1);
    hid_device.initialize()?;
    
    let iterations = 1000;
    for _ in 0..iterations {
        let report = vec![0x00, 0x00, 0x04];
        let _ = hid_device.parse_input_report(&report);
    }
    
    let duration = crate::timer::current_time_ms() - start_time;
    println!("HID report parsing: {} operations in {} ms ({:.1} ops/sec)", 
        iterations, duration, (iterations as f64 / duration as f64) * 1000.0);

    // Protocol analyzer benchmark
    let start_time = crate::timer::current_time_ms();
    let mut analyzer = ProtocolAnalyzer::new();
    let decoder = DescriptorDecoder::new();
    
    for _ in 0..100 {
        let descriptor = create_test_device_descriptor();
        let _ = decoder.decode_descriptor(&descriptor);
    }
    
    let duration = crate::timer::current_time_ms() - start_time;
    println!("Descriptor decoding: {} operations in {} ms ({:.1} ops/sec)", 
        100, duration, (100.0 / duration as f64) * 1000.0);

    // Security manager benchmark
    let start_time = crate::timer::current_time_ms();
    let mut security_manager = SecurityManager::new(SecurityLevel::Medium);
    
    for _ in 0..1000 {
        let fingerprint = DeviceFingerprint::new(0x1234, 0x5678, (0x03, 0x01, 0x01));
        let _ = security_manager.check_device_access(&fingerprint);
    }
    
    let duration = crate::timer::current_time_ms() - start_time;
    println!("Security checks: {} operations in {} ms ({:.1} ops/sec)", 
        1000, duration, (1000.0 / duration as f64) * 1000.0);

    println!("\nBenchmark completed!");
    Ok(())
}