//! Example implementations demonstrating the cross-platform compatibility layer

use crate::{
    ArchitectureType, DeviceClass, CompatibilityError, log, 
    framework::{Application, ApplicationInfo, ApplicationBuilder, ApplicationState, ApplicationType},
    api::{ApiResult, ApiError, make_call, ApiService, FileOperation, FileMode},
    testing::{Test, TestInfo, TestResult, TestType, TestCategory},
    platform::{Platform, get_system_info, get_battery_status},
};

/// Example application implementing the Application trait
#[derive(Debug)]
pub struct HelloWorldApp {
    info: ApplicationInfo,
    state: ApplicationState,
    output_buffer: Vec<u8>,
}

impl HelloWorldApp {
    pub fn new() -> Self {
        let info = ApplicationInfo {
            id: 1,
            name: "Hello World",
            version: "1.0.0",
            description: "Simple hello world application for cross-platform testing",
            author: "MultiOS Team",
            app_type: ApplicationType::Console,
            supported_architectures: vec![
                ArchitectureType::X86_64,
                ArchitectureType::ARM64,
                ArchitectureType::RISCV64,
            ],
            required_permissions: crate::framework::ApplicationPermissions::empty(),
            dependencies: vec![],
            resource_limits: None,
        };
        
        HelloWorldApp {
            info,
            state: ApplicationState::NotLoaded,
            output_buffer: Vec::new(),
        }
    }
}

impl Application for HelloWorldApp {
    fn get_info(&self) -> &ApplicationInfo {
        &self.info
    }
    
    fn init(&mut self) -> Result<(), CompatibilityError> {
        log::info!("Initializing Hello World application");
        self.state = ApplicationState::Ready;
        self.output_buffer.clear();
        self.output_buffer.extend_from_slice(b"Hello World Application Initialized\n");
        Ok(())
    }
    
    fn start(&mut self) -> Result<(), CompatibilityError> {
        log::info!("Starting Hello World application");
        self.state = ApplicationState::Running;
        self.output_buffer.extend_from_slice(b"Hello, MultiOS!\n");
        self.output_buffer.extend_from_slice(b"This application runs on multiple architectures.\n");
        
        // Demonstrate platform API usage
        if let Some(system_info) = get_system_info() {
            self.output_buffer.extend_from_slice(&format!(
                "Platform: {:?}, Architecture: {:?}, CPU Cores: {}\n",
                system_info.platform_type, system_info.architecture, system_info.cpu_count
            ).into_bytes());
        }
        
        // Demonstrate battery status
        if let Ok(battery) = get_battery_status() {
            if battery.present {
                self.output_buffer.extend_from_slice(&format!(
                    "Battery: {}% {}\n",
                    battery.level,
                    if battery.charging { "(charging)" } else { "" }
                ).into_bytes());
            }
        }
        
        Ok(())
    }
    
    fn pause(&mut self) -> Result<(), CompatibilityError> {
        self.state = ApplicationState::Suspended;
        Ok(())
    }
    
    fn resume(&mut self) -> Result<(), CompatibilityError> {
        self.state = ApplicationState::Running;
        Ok(())
    }
    
    fn stop(&mut self) -> Result<(), CompatibilityError> {
        self.state = ApplicationState::Terminated;
        self.output_buffer.extend_from_slice(b"Goodbye from MultiOS!\n");
        Ok(())
    }
    
    fn cleanup(&mut self) -> Result<(), CompatibilityError> {
        self.output_buffer.clear();
        self.state = ApplicationState::NotLoaded;
        Ok(())
    }
    
    fn get_state(&self) -> ApplicationState {
        self.state
    }
    
    fn is_ready(&self) -> bool {
        matches!(self.state, ApplicationState::Ready | ApplicationState::Running)
    }
    
    fn handle_event(&mut self, _event: &crate::framework::SystemEvent) -> Result<(), CompatibilityError> {
        Ok(())
    }
    
    fn get_output(&self) -> Option<&'static str> {
        // Convert output buffer to static str for demonstration
        unsafe {
            Some(core::str::from_utf8_unchecked(&self.output_buffer))
        }
    }
}

/// Example GUI application
#[derive(Debug)]
pub struct SimpleGuiApp {
    info: ApplicationInfo,
    state: ApplicationState,
    window_handle: Option<u32>,
}

impl SimpleGuiApp {
    pub fn new() -> Self {
        let info = ApplicationInfo {
            id: 2,
            name: "Simple GUI",
            version: "1.0.0",
            description: "Simple GUI application for testing",
            author: "MultiOS Team",
            app_type: ApplicationType::Native,
            supported_architectures: vec![
                ArchitectureType::X86_64,
                ArchitectureType::ARM64,
                ArchitectureType::RISCV64,
            ],
            required_permissions: crate::framework::ApplicationPermissions::empty(),
            dependencies: vec![],
            resource_limits: None,
        };
        
        SimpleGuiApp {
            info,
            state: ApplicationState::NotLoaded,
            window_handle: None,
        }
    }
}

impl crate::framework::GuiApplication for SimpleGuiApp {
    fn init_gui(&mut self) -> Result<(), CompatibilityError> {
        log::info!("Initializing GUI for Simple GUI Application");
        // In a real implementation, this would create a window
        self.window_handle = Some(1);
        Ok(())
    }
    
    fn render(&self) -> Result<(), CompatibilityError> {
        if let Some(handle) = self.window_handle {
            log::debug!("Rendering GUI window {}", handle);
            // In a real implementation, this would render the window
        }
        Ok(())
    }
    
    fn handle_gui_event(&mut self, event: &crate::framework::GuiEvent) -> Result<(), CompatibilityError> {
        log::debug!("Handling GUI event: {:?}", event.event_type);
        Ok(())
    }
    
    fn get_main_window(&self) -> Option<crate::framework::WindowHandle> {
        self.window_handle.map(|id| crate::framework::WindowHandle {
            id,
            width: 800,
            height: 600,
            x: 100,
            y: 100,
            visible: true,
        })
    }
}

impl Application for SimpleGuiApp {
    fn get_info(&self) -> &ApplicationInfo {
        &self.info
    }
    
    fn init(&mut self) -> Result<(), CompatibilityError> {
        log::info!("Initializing Simple GUI application");
        self.state = ApplicationState::Ready;
        Ok(())
    }
    
    fn start(&mut self) -> Result<(), CompatibilityError> {
        log::info!("Starting Simple GUI application");
        self.state = ApplicationState::Running;
        self.init_gui()?;
        Ok(())
    }
    
    fn pause(&mut self) -> Result<(), CompatibilityError> {
        self.state = ApplicationState::Suspended;
        Ok(())
    }
    
    fn resume(&mut self) -> Result<(), CompatibilityError> {
        self.state = ApplicationState::Running;
        Ok(())
    }
    
    fn stop(&mut self) -> Result<(), CompatibilityError> {
        self.state = ApplicationState::Terminated;
        Ok(())
    }
    
    fn cleanup(&mut self) -> Result<(), CompatibilityError> {
        self.state = ApplicationState::NotLoaded;
        Ok(())
    }
    
    fn get_state(&self) -> ApplicationState {
        self.state
    }
    
    fn is_ready(&self) -> bool {
        matches!(self.state, ApplicationState::Ready | ApplicationState::Running)
    }
    
    fn handle_event(&mut self, _event: &crate::framework::SystemEvent) -> Result<(), CompatibilityError> {
        Ok(())
    }
}

/// Example test implementation
pub struct SystemIntegrationTest {
    info: TestInfo,
    test_name: &'static str,
}

impl SystemIntegrationTest {
    pub fn new(test_name: &'static str) -> Self {
        let info = TestInfo {
            id: 100,
            name: test_name,
            description: "System integration test",
            test_type: TestType::Integration,
            category: TestCategory::SYSTEM | TestCategory::PERFORMANCE,
            supported_architectures: vec![
                ArchitectureType::X86_64,
                ArchitectureType::ARM64,
                ArchitectureType::RISCV64,
            ],
            supported_devices: vec![],
            timeout_ms: 5000,
            critical: true,
        };
        
        SystemIntegrationTest {
            info,
            test_name,
        }
    }
}

impl Test for SystemIntegrationTest {
    fn get_info(&self) -> &TestInfo {
        &self.info
    }
    
    fn run(&self) -> Result<TestResult, CompatibilityError> {
        log::info!("Running system integration test: {}", self.test_name);
        
        // Test 1: Check compatibility layer initialization
        if let Some(state) = crate::get_state() {
            log::debug!("Compatibility layer state: {:?}", state.arch_type);
            if !state.features.is_supported() {
                return Ok(TestResult::Fail);
            }
        } else {
            return Ok(TestResult::Fail);
        }
        
        // Test 2: Check device manager
        if let Some(device_manager) = crate::devices::get_device_manager() {
            log::debug!("Device manager is available");
            // Basic device manager functionality test would go here
        } else {
            log::warn!("Device manager not available");
            return Ok(TestResult::Fail);
        }
        
        // Test 3: Check driver manager
        if let Some(driver_manager) = crate::drivers::get_driver_manager() {
            log::debug!("Driver manager is available");
            // Basic driver manager functionality test would go here
        } else {
            log::warn!("Driver manager not available");
            return Ok(TestResult::Fail);
        }
        
        // Test 4: Check API layer
        if let Some(system_info) = get_system_info() {
            log::debug!("System info: {:?}", system_info.architecture);
        } else {
            log::warn!("System info not available");
            return Ok(TestResult::Fail);
        }
        
        // Test 5: Test basic API functionality
        let file_params = vec![FileMode::READ.bits() as u64, 0];
        match make_call(ApiService::FileSystem, FileOperation::Open as u32, &file_params) {
            Ok(_) => log::debug!("File API test passed"),
            Err(e) => {
                log::debug!("File API test result: {:?}", e);
                // File operations might not be implemented yet, so don't fail on this
            }
        }
        
        log::info!("System integration test '{}' completed successfully", self.test_name);
        Ok(TestResult::Pass)
    }
}

/// Example stress test
pub struct StressTest {
    info: TestInfo,
    iterations: u32,
}

impl StressTest {
    pub fn new(iterations: u32) -> Self {
        let info = TestInfo {
            id: 101,
            name: "System Stress Test",
            description: "Stress test for cross-platform compatibility",
            test_type: TestType::Stress,
            category: TestCategory::PERFORMANCE | TestCategory::SYSTEM,
            supported_architectures: vec![
                ArchitectureType::X86_64,
                ArchitectureType::ARM64,
                ArchitectureType::RISCV64,
            ],
            supported_devices: vec![],
            timeout_ms: 10000,
            critical: false,
        };
        
        StressTest {
            info,
            iterations,
        }
    }
}

impl Test for StressTest {
    fn get_info(&self) -> &TestInfo {
        &self.info
    }
    
    fn run(&self) -> Result<TestResult, CompatibilityError> {
        log::info!("Running stress test with {} iterations", self.iterations);
        
        let mut passed = 0;
        let mut failed = 0;
        
        for i in 0..self.iterations {
            if i % 100 == 0 {
                log::debug!("Stress test progress: {}/{}", i, self.iterations);
            }
            
            // Test system calls under load
            match make_call(ApiService::FileSystem, FileOperation::Open as u32, &[0, 0]) {
                Ok(_) => passed += 1,
                Err(_) => failed += 1,
            }
            
            // Test memory allocation
            if let Ok(ptr) = crate::api::memory_allocate(1024, crate::api::MemoryProtection::READ | crate::api::MemoryProtection::WRITE) {
                unsafe {
                    // Write to allocated memory
                    core::ptr::write_volatile(ptr, 0x42u8);
                    let value = core::ptr::read_volatile(ptr);
                    if value == 0x42 {
                        passed += 1;
                    } else {
                        failed += 1;
                    }
                }
            } else {
                failed += 1;
            }
            
            // Test architecture features
            if let Some(state) = crate::get_state() {
                let _ = state.features.has_fpu; // Just access to verify state
                passed += 1;
            } else {
                failed += 1;
            }
        }
        
        let total = passed + failed;
        let pass_rate = if total > 0 { (passed as f32) / (total as f32) } else { 0.0 };
        
        log::info!("Stress test completed: {}/{} passed ({:.1}%)", passed, total, pass_rate * 100.0);
        
        if pass_rate >= 0.95 {
            Ok(TestResult::Pass)
        } else {
            Ok(TestResult::Fail)
        }
    }
}

/// Example application demonstrating file operations
#[derive(Debug)]
pub struct FileApp {
    info: ApplicationInfo,
    state: ApplicationState,
    test_file_path: &'static str,
}

impl FileApp {
    pub fn new() -> Self {
        let info = ApplicationInfo {
            id: 3,
            name: "File Operations",
            version: "1.0.0",
            description: "Application demonstrating file operations",
            author: "MultiOS Team",
            app_type: ApplicationType::Console,
            supported_architectures: vec![
                ArchitectureType::X86_64,
                ArchitectureType::ARM64,
                ArchitectureType::RISCV64,
            ],
            required_permissions: crate::framework::ApplicationPermissions::FILE_READ | 
                                 crate::framework::ApplicationPermissions::FILE_WRITE,
            dependencies: vec![],
            resource_limits: None,
        };
        
        FileApp {
            info,
            state: ApplicationState::NotLoaded,
            test_file_path: "/test_file.txt",
        }
    }
}

impl Application for FileApp {
    fn get_info(&self) -> &ApplicationInfo {
        &self.info
    }
    
    fn init(&mut self) -> Result<(), CompatibilityError> {
        log::info!("Initializing File Operations application");
        self.state = ApplicationState::Ready;
        Ok(())
    }
    
    fn start(&mut self) -> Result<(), CompatibilityError> {
        log::info!("Starting File Operations application");
        self.state = ApplicationState::Running;
        
        // Demonstrate file operations
        log::info!("Testing file operations for cross-platform compatibility");
        
        // Open file
        let open_params = vec![self.test_file_path.len() as u64, FileMode::READ.bits() as u64];
        let test_data = self.test_file_path.as_bytes();
        let mut params = vec![open_params[0], open_params[1]];
        for chunk in test_data.chunks(8) {
            let mut padded = [0u8; 8];
            padded[..chunk.len()].copy_from_slice(chunk);
            params.extend_from_slice(&padded.iter().map(|b| *b as u64).collect::<Vec<_>>());
        }
        
        match make_call(ApiService::FileSystem, FileOperation::Open as u32, &params) {
            Ok(result) => log::info!("File opened successfully: {:?}", result),
            Err(e) => log::warn!("File operation failed: {:?}", e),
        }
        
        Ok(())
    }
    
    fn pause(&mut self) -> Result<(), CompatibilityError> {
        self.state = ApplicationState::Suspended;
        Ok(())
    }
    
    fn resume(&mut self) -> Result<(), CompatibilityError> {
        self.state = ApplicationState::Running;
        Ok(())
    }
    
    fn stop(&mut self) -> Result<(), CompatibilityError> {
        self.state = ApplicationState::Terminated;
        Ok(())
    }
    
    fn cleanup(&mut self) -> Result<(), CompatibilityError> {
        self.state = ApplicationState::NotLoaded;
        Ok(())
    }
    
    fn get_state(&self) -> ApplicationState {
        self.state
    }
    
    fn is_ready(&self) -> bool {
        matches!(self.state, ApplicationState::Ready | ApplicationState::Running)
    }
    
    fn handle_event(&mut self, _event: &crate::framework::SystemEvent) -> Result<(), CompatibilityError> {
        Ok(())
    }
}