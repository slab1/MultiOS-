//! Portable Application Framework
//! 
//! This module provides a unified framework for building applications that
//! can run across different MultiOS architectures with minimal modifications.

use crate::{ArchitectureType, CompatibilityError, log};
use core::sync::atomic::{AtomicU32, Ordering};
use spin::Mutex;
use bitflags::bitflags;

/// Application identifier
pub type ApplicationId = u32;

/// Application state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApplicationState {
    NotLoaded,
    Loading,
    Ready,
    Running,
    Suspended,
    Terminated,
    Error,
}

/// Application types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ApplicationType {
    Native,
    Interpreted,
    Virtualized,
    Web,
    System,
}

/// Application permissions
bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct ApplicationPermissions: u64 {
        const FILE_READ = 0x001;
        const FILE_WRITE = 0x002;
        const FILE_CREATE = 0x004;
        const FILE_DELETE = 0x008;
        const NETWORK_ACCESS = 0x010;
        const HARDWARE_ACCESS = 0x020;
        const INTERPROCESS_COMMUNICATION = 0x040;
        const MEMORY_ACCESS = 0x080;
        const SYSTEM_CALLS = 0x100;
        const POWER_MANAGEMENT = 0x200;
    }
}

/// Resource limits for applications
#[derive(Debug, Clone)]
pub struct ResourceLimits {
    pub max_memory: usize,
    pub max_cpu_time: u64,
    pub max_file_size: usize,
    pub max_open_files: u32,
    pub max_threads: u32,
    pub max_network_connections: u32,
}

/// Application information structure
#[derive(Debug, Clone)]
pub struct ApplicationInfo {
    pub id: ApplicationId,
    pub name: &'static str,
    pub version: &'static str,
    pub description: &'static str,
    pub author: &'static str,
    pub app_type: ApplicationType,
    pub supported_architectures: Vec<ArchitectureType>,
    pub required_permissions: ApplicationPermissions,
    pub dependencies: Vec<ApplicationId>,
    pub resource_limits: Option<ResourceLimits>,
}

/// Base application trait
pub trait Application: Send + Sync {
    /// Get application information
    fn get_info(&self) -> &ApplicationInfo;
    
    /// Initialize application
    fn init(&mut self) -> Result<(), CompatibilityError>;
    
    /// Start application
    fn start(&mut self) -> Result<(), CompatibilityError>;
    
    /// Pause application
    fn pause(&mut self) -> Result<(), CompatibilityError>;
    
    /// Resume application
    fn resume(&mut self) -> Result<(), CompatibilityError>;
    
    /// Stop application
    fn stop(&mut self) -> Result<(), CompatibilityError>;
    
    /// Cleanup resources
    fn cleanup(&mut self) -> Result<(), CompatibilityError>;
    
    /// Get current state
    fn get_state(&self) -> ApplicationState;
    
    /// Check if application is ready to run
    fn is_ready(&self) -> bool;
    
    /// Handle system events
    fn handle_event(&mut self, event: &SystemEvent) -> Result<(), CompatibilityError>;
}

/// GUI application trait
pub trait GuiApplication: Application {
    /// Initialize GUI
    fn init_gui(&mut self) -> Result<(), CompatibilityError>;
    
    /// Render GUI
    fn render(&self) -> Result<(), CompatibilityError>;
    
    /// Handle GUI events
    fn handle_gui_event(&mut self, event: &GuiEvent) -> Result<(), CompatibilityError>;
    
    /// Get main window handle
    fn get_main_window(&self) -> Option<WindowHandle>;
}

/// Console application trait
pub trait ConsoleApplication: Application {
    /// Run main loop
    fn run_main_loop(&mut self) -> Result<(), CompatibilityError>;
    
    /// Handle console input
    fn handle_input(&mut self, input: &str) -> Result<(), CompatibilityError>;
    
    /// Output text to console
    fn output(&self, text: &str) -> Result<(), CompatibilityError>;
    
    /// Get console prompt
    fn get_prompt(&self) -> Option<&'static str>;
}

/// Network application trait
pub trait NetworkApplication: Application {
    /// Start network listener
    fn start_listener(&mut self, port: u16) -> Result<(), CompatibilityError>;
    
    /// Connect to remote host
    fn connect(&mut self, host: &str, port: u16) -> Result<(), CompatibilityError>;
    
    /// Send data over network
    fn send_data(&self, data: &[u8]) -> Result<(), CompatibilityError>;
    
    /// Receive data from network
    fn receive_data(&self, buffer: &mut [u8]) -> Result<usize, CompatibilityError>;
    
    /// Handle network event
    fn handle_network_event(&mut self, event: &NetworkEvent) -> Result<(), CompatibilityError>;
}

/// System event types
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum SystemEventType {
    Power,
    Memory,
    Storage,
    Network,
    Timer,
    Custom,
}

/// System event structure
#[derive(Debug, Clone)]
pub struct SystemEvent {
    pub event_type: SystemEventType,
    pub timestamp: u64,
    pub source: &'static str,
    pub data: [u64; 4],
}

/// GUI event types
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum GuiEventType {
    MouseButton,
    MouseMove,
    KeyPress,
    KeyRelease,
    WindowClose,
    WindowResize,
    Focus,
    Custom,
}

/// GUI event structure
#[derive(Debug, Clone)]
pub struct GuiEvent {
    pub event_type: GuiEventType,
    pub timestamp: u64,
    pub window: Option<WindowHandle>,
    pub data: [u64; 4],
}

/// Window handle
pub struct WindowHandle {
    pub id: u32,
    pub width: u32,
    pub height: u32,
    pub x: i32,
    pub y: i32,
    pub visible: bool,
}

/// Network event types
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum NetworkEventType {
    ConnectionEstablished,
    ConnectionLost,
    DataReceived,
    Error,
}

/// Network event structure
#[derive(Debug, Clone)]
pub struct NetworkEvent {
    pub event_type: NetworkEventType,
    pub timestamp: u64,
    pub connection_id: u32,
    pub data: Vec<u8>,
}

/// Application manager
pub struct ApplicationManager {
    applications: Mutex<Vec<Box<dyn Application>>>,
    running_apps: Mutex<Vec<ApplicationId>>,
    app_id_counter: AtomicU32,
}

impl ApplicationManager {
    pub fn new() -> Self {
        ApplicationManager {
            applications: Mutex::new(Vec::new()),
            running_apps: Mutex::new(Vec::new()),
            app_id_counter: AtomicU32::new(1),
        }
    }
    
    /// Register an application
    pub fn register_application(&self, mut app: Box<dyn Application>) -> Result<ApplicationId, CompatibilityError> {
        let app_id = self.app_id_counter.fetch_add(1, Ordering::SeqCst);
        
        // Check architecture compatibility
        let arch_type = crate::get_state()
            .map(|s| s.arch_type)
            .ok_or(CompatibilityError::InitializationFailed("Compatibility state not initialized"))?;
        
        if !app.get_info().supported_architectures.contains(&arch_type) {
            return Err(CompatibilityError::DriverNotCompatible);
        }
        
        {
            let mut apps = self.applications.lock();
            apps.push(app);
        }
        
        Ok(app_id)
    }
    
    /// Load and initialize application
    pub fn load_application(&self, app_id: ApplicationId) -> Result<(), CompatibilityError> {
        let apps = self.applications.lock();
        for app in apps.iter() {
            if app.get_info().id == app_id {
                return app.init();
            }
        }
        Err(CompatibilityError::DeviceNotFound)
    }
    
    /// Start application
    pub fn start_application(&self, app_id: ApplicationId) -> Result<(), CompatibilityError> {
        let apps = self.applications.lock();
        for app in apps.iter() {
            if app.get_info().id == app_id {
                let result = app.start();
                if result.is_ok() {
                    let mut running = self.running_apps.lock();
                    running.push(app_id);
                }
                return result;
            }
        }
        Err(CompatibilityError::DeviceNotFound)
    }
    
    /// Stop application
    pub fn stop_application(&self, app_id: ApplicationId) -> Result<(), CompatibilityError> {
        let apps = self.applications.lock();
        for app in apps.iter() {
            if app.get_info().id == app_id {
                let result = app.stop();
                if result.is_ok() {
                    let mut running = self.running_apps.lock();
                    running.retain(|&id| id != app_id);
                }
                return result;
            }
        }
        Err(CompatibilityError::DeviceNotFound)
    }
    
    /// Get list of running applications
    pub fn get_running_applications(&self) -> Vec<ApplicationId> {
        self.running_apps.lock().clone()
    }
    
    /// Find application by ID
    pub fn find_application(&self, app_id: ApplicationId) -> Option<&dyn Application> {
        let apps = self.applications.lock();
        for app in apps.iter() {
            if app.get_info().id == app_id {
                return Some(app.as_ref());
            }
        }
        None
    }
    
    /// Get all registered applications
    pub fn get_applications(&self) -> Vec<&dyn Application> {
        let apps = self.applications.lock();
        apps.iter().map(|app| app.as_ref()).collect()
    }
}

/// Global application manager
static APP_MANAGER: spin::Mutex<Option<ApplicationManager>> = spin::Mutex::new(None);

/// Initialize application framework
pub fn init() -> Result<(), CompatibilityError> {
    let mut manager_lock = APP_MANAGER.lock();
    
    if manager_lock.is_some() {
        return Ok(());
    }
    
    *manager_lock = Some(ApplicationManager::new());
    
    log::info!("Application framework initialized");
    
    Ok(())
}

/// Register an application
pub fn register_application(app: Box<dyn Application>) -> Result<ApplicationId, CompatibilityError> {
    let manager = APP_MANAGER.lock();
    let manager_ref = manager.as_ref()
        .ok_or(CompatibilityError::InitializationFailed("Application manager not initialized"))?;
    
    manager_ref.register_application(app)
}

/// Load application
pub fn load_application(app_id: ApplicationId) -> Result<(), CompatibilityError> {
    let manager = APP_MANAGER.lock();
    let manager_ref = manager.as_ref()
        .ok_or(CompatibilityError::InitializationFailed("Application manager not initialized"))?;
    
    manager_ref.load_application(app_id)
}

/// Start application
pub fn start_application(app_id: ApplicationId) -> Result<(), CompatibilityError> {
    let manager = APP_MANAGER.lock();
    let manager_ref = manager.as_ref()
        .ok_or(CompatibilityError::InitializationFailed("Application manager not initialized"))?;
    
    manager_ref.start_application(app_id)
}

/// Stop application
pub fn stop_application(app_id: ApplicationId) -> Result<(), CompatibilityError> {
    let manager = APP_MANAGER.lock();
    let manager_ref = manager.as_ref()
        .ok_or(CompatibilityError::InitializationFailed("Application manager not initialized"))?;
    
    manager_ref.stop_application(app_id)
}

/// Get running applications
pub fn get_running_applications() -> Vec<ApplicationId> {
    let manager = APP_MANAGER.lock();
    let manager_ref = match manager.as_ref() {
        Some(manager) => manager,
        None => return Vec::new(),
    };
    manager_ref.get_running_applications()
}

/// Application builder for creating applications
pub struct ApplicationBuilder {
    name: &'static str,
    version: &'static str,
    app_type: ApplicationType,
    supported_archs: Vec<ArchitectureType>,
    permissions: ApplicationPermissions,
}

impl ApplicationBuilder {
    pub fn new(name: &'static str, app_type: ApplicationType) -> Self {
        ApplicationBuilder {
            name,
            version: "1.0.0",
            app_type,
            supported_archs: vec![
                ArchitectureType::X86_64,
                ArchitectureType::ARM64,
                ArchitectureType::RISCV64,
            ],
            permissions: ApplicationPermissions::empty(),
        }
    }
    
    pub fn version(mut self, version: &'static str) -> Self {
        self.version = version;
        self
    }
    
    pub fn description(mut self, description: &'static str) -> Self {
        // Would store description in builder
        self
    }
    
    pub fn author(mut self, author: &'static str) -> Self {
        // Would store author in builder
        self
    }
    
    pub fn permissions(mut self, permissions: ApplicationPermissions) -> Self {
        self.permissions = permissions;
        self
    }
    
    pub fn supported_architectures(mut self, architectures: Vec<ArchitectureType>) -> Self {
        self.supported_archs = architectures;
        self
    }
    
    pub fn build<T: Application + 'static>(self, implementation: T) -> Box<dyn Application> {
        // This would create a wrapper application that uses the provided implementation
        // For now, just return the boxed implementation
        Box::new(implementation)
    }
}