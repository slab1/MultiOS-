// MultiOS Example Projects
// A collection of hands-on examples for learning MultiOS development

//=============================================================================
// PROJECT 1: Hello World Kernel Module
//=============================================================================
// Demonstrates basic kernel module development with logging and initialization

use multios::prelude::*;
use multios::kernel::Module;

/// Simple hello world kernel module
#[multios_kernel_module]
pub struct HelloModule {
    message_count: usize,
}

impl HelloModule {
    pub fn new() -> Self {
        HelloModule {
            message_count: 0,
        }
    }
}

impl Module for HelloModule {
    fn init(&mut self) -> Result<(), ModuleError> {
        self.message_count = 0;
        info!("Hello World MultiOS Kernel Module initialized!");
        info!("Module loaded successfully");
        Ok(())
    }
    
    fn cleanup(&mut self) {
        info!("Hello World module cleaned up. Total messages: {}", self.message_count);
    }
    
    fn handle_message(&mut self, message: &str) -> Result<(), ModuleError> {
        self.message_count += 1;
        info!("Module received message #{}: {}", self.message_count, message);
        Ok(())
    }
}

// Example usage:
// let module = HelloModule::new();
// module.init()?;
// module.handle_message("Test message")?;

//=============================================================================
// PROJECT 2: Character Device Driver
//=============================================================================
// Implements a simple character device for demonstrating device driver concepts

use multios::device::{Device, DeviceDriver, DeviceError};
use multios::sync::SpinLock;
use multios::alloc::Vec;

pub struct EchoDevice {
    buffer: SpinLock<Vec<u8>>,
    read_position: SpinLock<usize>,
}

impl EchoDevice {
    pub fn new() -> Self {
        EchoDevice {
            buffer: SpinLock::new(Vec::new()),
            read_position: SpinLock::new(0),
        }
    }
}

impl Device for EchoDevice {
    fn device_name(&self) -> &'static str {
        "echo_device"
    }
    
    fn device_type(&self) -> DeviceType {
        DeviceType::Character
    }
}

impl DeviceDriver for EchoDevice {
    fn read(&self, buf: &mut [u8]) -> Result<usize, DeviceError> {
        let mut buffer = self.buffer.lock();
        let mut read_pos = self.read_position.lock();
        
        if *read_pos >= buffer.len() {
            return Ok(0); // No more data to read
        }
        
        let available_data = &buffer[*read_pos..];
        let bytes_to_read = available_data.len().min(buf.len());
        
        buf[..bytes_to_read].copy_from_slice(&available_data[..bytes_to_read]);
        *read_pos += bytes_to_read;
        
        Ok(bytes_to_read)
    }
    
    fn write(&self, buf: &[u8]) -> Result<usize, DeviceError> {
        let mut buffer = self.buffer.lock();
        
        // Echo the data back (append to buffer)
        buffer.extend_from_slice(buf);
        
        // Reset read position for new data
        let mut read_pos = self.read_position.lock();
        *read_pos = 0;
        
        info!("Echo device: received {} bytes, echoing back", buf.len());
        
        Ok(buf.len())
    }
    
    fn ioctl(&self, command: u32, arg: usize) -> Result<usize, DeviceError> {
        match command {
            // Clear device buffer
            0x1000 => {
                let mut buffer = self.buffer.lock();
                let mut read_pos = self.read_position.lock();
                buffer.clear();
                *read_pos = 0;
                info!("Echo device buffer cleared");
                Ok(0)
            }
            // Get buffer size
            0x1001 => {
                let buffer = self.buffer.lock();
                Ok(buffer.len())
            }
            _ => Err(DeviceError::InvalidCommand),
        }
    }
}

// Example usage:
// let device = EchoDevice::new();
// let data = b"Hello MultiOS!";
// device.write(data)?;

//=============================================================================
// PROJECT 3: Simple File System
//=============================================================================
// Demonstrates file system implementation using the MultiOS VFS

use multios::fs::{FileSystem, Inode, FileType, FsError, DirectoryEntry};
use multios::alloc::HashMap;
use multios::sync::SpinLock;

#[derive(Clone)]
pub struct SimpleInode {
    pub inode_number: usize,
    pub file_type: FileType,
    pub size: usize,
    pub data: Vec<u8>,
    pub name: String,
}

pub struct SimpleFileSystem {
    pub next_inode: SpinLock<usize>,
    pub inodes: SpinLock<HashMap<usize, SimpleInode>>,
    pub root_inode: usize,
}

impl SimpleFileSystem {
    pub fn new() -> Self {
        let mut inodes = HashMap::new();
        let root_inode = 1;
        
        // Create root directory
        inodes.insert(root_inode, SimpleInode {
            inode_number: root_inode,
            file_type: FileType::Directory,
            size: 0,
            data: Vec::new(),
            name: String::from("/"),
        });
        
        SimpleFileSystem {
            next_inode: SpinLock::new(2), // Start from inode 2
            inodes: SpinLock::new(inodes),
            root_inode,
        }
    }
    
    fn allocate_inode(&self, file_type: FileType, name: &str) -> Result<SimpleInode, FsError> {
        let mut next_inode = self.next_inode.lock();
        let inode_num = *next_inode;
        *next_inode += 1;
        
        Ok(SimpleInode {
            inode_number: inode_num,
            file_type,
            size: 0,
            data: Vec::new(),
            name: String::from(name),
        })
    }
}

impl FileSystem for SimpleFileSystem {
    fn lookup(&self, path: &str) -> Result<Inode, FsError> {
        if path == "/" || path.is_empty() {
            return Ok(Inode::new(self.root_inode));
        }
        
        let inodes = self.inodes.lock();
        
        // Simple path parsing - just find by name for now
        for (_num, inode) in inodes.iter() {
            if inode.name == path {
                return Ok(Inode::new(inode.inode_number));
            }
        }
        
        Err(FsError::NotFound)
    }
    
    fn create(&self, path: &str, file_type: FileType) -> Result<Inode, FsError> {
        if path.is_empty() || path.starts_with('/') == false {
            return Err(FsError::InvalidPath);
        }
        
        let name = path.trim_start_matches('/');
        let inode = self.allocate_inode(file_type, name)?;
        let inode_num = inode.inode_number;
        
        let mut inodes = self.inodes.lock();
        inodes.insert(inode_num, inode);
        
        info!("Simple FS: created {} '{}'", file_type, name);
        Ok(Inode::new(inode_num))
    }
    
    fn read(&self, inode: Inode, offset: usize, buf: &mut [u8]) -> Result<usize, FsError> {
        let inodes = self.inodes.lock();
        let inode_data = inodes.get(&inode.number())
            .ok_or(FsError::NotFound)?;
        
        if inode_data.file_type == FileType::Directory {
            return Err(FsError::IsDirectory);
        }
        
        let available_data = &inode_data.data[offset..];
        let bytes_to_read = available_data.len().min(buf.len());
        
        buf[..bytes_to_read].copy_from_slice(&available_data[..bytes_to_read]);
        
        Ok(bytes_to_read)
    }
    
    fn write(&self, inode: Inode, offset: usize, buf: &[u8]) -> Result<usize, FsError> {
        let mut inodes = self.inodes.lock();
        let inode_data = inodes.get_mut(&inode.number())
            .ok_or(FsError::NotFound)?;
        
        if inode_data.file_type == FileType::Directory {
            return Err(FsError::IsDirectory);
        }
        
        // Ensure we have enough space
        let needed_size = offset + buf.len();
        if needed_size > inode_data.data.len() {
            inode_data.data.resize(needed_size, 0);
        }
        
        inode_data.data[offset..offset + buf.len()].copy_from_slice(buf);
        inode_data.size = inode_data.data.len();
        
        Ok(buf.len())
    }
    
    fn readdir(&self, inode: Inode) -> Result<Vec<DirectoryEntry>, FsError> {
        let inodes = self.inodes.lock();
        let inode_data = inodes.get(&inode.number())
            .ok_or(FsError::NotFound)?;
        
        if inode_data.file_type != FileType::Directory {
            return Err(FsError::NotDirectory);
        }
        
        let mut entries = Vec::new();
        
        // Return directory entries (simplified)
        for (_num, dir_inode) in inodes.iter() {
            if dir_inode.inode_number == self.root_inode {
                continue; // Skip root in directory
            }
            
            entries.push(DirectoryEntry {
                name: dir_inode.name.clone(),
                inode: Inode::new(dir_inode.inode_number),
                file_type: dir_inode.file_type,
            });
        }
        
        Ok(entries)
    }
}

// Example usage:
// let fs = SimpleFileSystem::new();
// let file_inode = fs.create("test.txt", FileType::File)?;
// fs.write(file_inode, 0, b"Hello MultiOS!")?;

//=============================================================================
// PROJECT 4: Network Echo Server
//=============================================================================
// Demonstrates networking by implementing a simple TCP echo server

use multios::net::{TcpSocket, UdpSocket, SocketAddr, IpAddr, NetworkError};
use multios::sync::Arc;
use multios::thread::{Thread, ThreadSpawner};
use multios::time::{Duration, Instant};

pub struct EchoServer {
    tcp_socket: TcpSocket,
    udp_socket: UdpSocket,
    running: Arc<SpinLock<bool>>,
}

impl EchoServer {
    pub fn bind(port: u16) -> Result<Self, NetworkError> {
        let tcp_addr = SocketAddr::new(IpAddr::new_v4(0, 0, 0, 0), port);
        let udp_addr = SocketAddr::new(IpAddr::new_v4(0, 0, 0, 0), port);
        
        let tcp_socket = TcpSocket::bind(tcp_addr)?;
        let udp_socket = UdpSocket::bind(udp_addr)?;
        
        Ok(EchoServer {
            tcp_socket,
            udp_socket,
            running: Arc::new(SpinLock::new(false)),
        })
    }
    
    pub fn start(&self) -> Result<(), NetworkError> {
        {
            let mut running = self.running.lock();
            *running = true;
        }
        
        info!("Echo server starting on TCP and UDP...");
        
        // Start TCP echo server
        let tcp_running = Arc::clone(&self.running);
        ThreadSpawner::new()
            .spawn(move || {
                echo_server_loop(&self.tcp_socket, &tcp_running)
            })?;
        
        // Start UDP echo server
        let udp_running = Arc::clone(&self.running);
        ThreadSpawner::new()
            .spawn(move || {
                udp_echo_server_loop(&self.udp_socket, &udp_running)
            })?;
        
        Ok(())
    }
    
    pub fn stop(&self) {
        let mut running = self.running.lock();
        *running = false;
        info!("Echo server stopping...");
    }
}

fn echo_server_loop(socket: &TcpSocket, running: &Arc<SpinLock<bool>>) {
    info!("TCP echo server started");
    
    while *running.lock() {
        match socket.accept() {
            Ok((client_socket, _)) => {
                let client_running = Arc::clone(running);
                ThreadSpawner::new()
                    .spawn(move || {
                        handle_tcp_client(client_socket, &client_running)
                    }).unwrap_or_else(|e| error!("Failed to spawn client thread: {:?}", e));
            }
            Err(e) => error!("TCP accept failed: {:?}", e),
        }
    }
}

fn handle_tcp_client(mut socket: TcpSocket, running: &Arc<SpinLock<bool>>) {
    info!("New TCP client connected");
    
    let mut buffer = [0u8; 1024];
    
    while *running.lock() {
        match socket.read(&mut buffer) {
            Ok(0) => break, // Connection closed
            Ok(n) => {
                info!("TCP echo: {} bytes", n);
                if let Err(e) = socket.write(&buffer[..n]) {
                    error!("TCP write failed: {:?}", e);
                    break;
                }
            }
            Err(e) => {
                error!("TCP read failed: {:?}", e);
                break;
            }
        }
    }
    
    info!("TCP client disconnected");
}

fn udp_echo_server_loop(socket: &UdpSocket, running: &Arc<SpinLock<bool>>) {
    info!("UDP echo server started");
    
    let mut buffer = [0u8; 1024];
    
    while *running.lock() {
        match socket.recv_from(&mut buffer) {
            Ok((n, _addr)) => {
                info!("UDP echo: {} bytes", n);
                if let Err(e) = socket.send_to(&buffer[..n], _addr) {
                    error!("UDP send failed: {:?}", e);
                }
            }
            Err(e) => error!("UDP recv failed: {:?}", e),
        }
    }
}

// Example usage:
// let server = EchoServer::bind(8080)?;
// server.start()?;

//=============================================================================
// PROJECT 5: GUI Application
//=============================================================================
// Demonstrates GUI development with window creation and event handling

use multios::gui::{Window, WindowEvent, EventHandler, Application, Point, Size};
use multios::events::{Event, EventType, MouseEvent, KeyEvent};

pub struct SimpleTextEditor {
    window: Window,
    text_content: String,
    cursor_position: usize,
}

impl SimpleTextEditor {
    pub fn new() -> Result<Self, GuiError> {
        let window = Window::new(
            Point::new(100, 100),
            Size::new(800, 600),
            "MultiOS Text Editor",
        )?;
        
        Ok(SimpleTextEditor {
            window,
            text_content: String::new(),
            cursor_position: 0,
        })
    }
}

impl EventHandler for SimpleTextEditor {
    fn handle_event(&mut self, event: &Event) {
        match event {
            Event::Window(event) => self.handle_window_event(event),
            Event::Mouse(event) => self.handle_mouse_event(event),
            Event::Key(event) => self.handle_key_event(event),
            _ => {}
        }
    }
}

impl SimpleTextEditor {
    fn handle_window_event(&self, event: &WindowEvent) {
        match event {
            WindowEvent::Close => {
                info!("Text editor window closing");
            }
            WindowEvent::Resize(size) => {
                info!("Window resized to: {:?}", size);
            }
            WindowEvent::Move(point) => {
                info!("Window moved to: {:?}", point);
            }
        }
    }
    
    fn handle_mouse_event(&mut self, event: &MouseEvent) {
        match event {
            MouseEvent::Click { position, button } => {
                info!("Mouse clicked at {:?} with button {:?}", position, button);
                // TODO: Calculate cursor position from mouse coordinates
            }
            MouseEvent::DoubleClick { position } => {
                info!("Mouse double-clicked at {:?}", position);
                // TODO: Select word under cursor
            }
            _ => {}
        }
    }
    
    fn handle_key_event(&mut self, event: &KeyEvent) {
        match event {
            KeyEvent::Press(key) => {
                match key {
                    Key::Character(c) => {
                        self.insert_character(*c);
                    }
                    Key::Backspace => {
                        self.delete_character();
                    }
                    Key::Enter => {
                        self.insert_character('\n');
                    }
                    Key::ArrowLeft => {
                        if self.cursor_position > 0 {
                            self.cursor_position -= 1;
                        }
                    }
                    Key::ArrowRight => {
                        if self.cursor_position < self.text_content.len() {
                            self.cursor_position += 1;
                        }
                    }
                    _ => {}
                }
                self.redraw();
            }
            _ => {}
        }
    }
    
    fn insert_character(&mut self, c: char) {
        self.text_content.insert(self.cursor_position, c);
        self.cursor_position += 1;
    }
    
    fn delete_character(&mut self) {
        if self.cursor_position > 0 {
            self.text_content.remove(self.cursor_position - 1);
            self.cursor_position -= 1;
        }
    }
    
    fn redraw(&self) {
        // TODO: Implement window redrawing with text content
        info!("Redrawing text editor with content length: {}", self.text_content.len());
    }
}

#[derive(Debug)]
pub enum GuiError {
    WindowCreationFailed,
    RendererError,
    InvalidState,
}

// Example usage:
// let mut editor = SimpleTextEditor::new()?;
// let mut application = Application::new()?;
// application.add_window(editor.window);
// application.run()?;

//=============================================================================
// PROJECT 6: Memory Allocator
//=============================================================================
// Demonstrates custom memory allocation for educational purposes

use multios::memory::{PhysicalFrame, PageSize, allocate_frame, deallocate_frame};

pub struct SimpleAllocator {
    free_list: Vec<PhysicalFrame>,
}

impl SimpleAllocator {
    pub fn new() -> Self {
        SimpleAllocator {
            free_list: Vec::new(),
        }
    }
    
    pub fn initialize(&mut self, total_frames: usize) {
        // Initialize allocator with a pool of physical frames
        for frame_num in 0..total_frames {
            let frame = PhysicalFrame::new(frame_num * PageSize::SIZE_4K as usize);
            self.free_list.push(frame);
        }
        
        info!("Simple allocator initialized with {} frames", total_frames);
    }
    
    pub fn allocate(&mut self, size: usize) -> Option<PhysicalFrame> {
        // Find the first frame that can accommodate the requested size
        for (index, frame) in self.free_list.iter().enumerate() {
            if frame.size() >= size {
                let allocated_frame = self.free_list.remove(index);
                return Some(allocated_frame);
            }
        }
        
        None
    }
    
    pub fn deallocate(&mut self, frame: PhysicalFrame) {
        self.free_list.push(frame);
    }
    
    pub fn stats(&self) -> AllocatorStats {
        AllocatorStats {
            total_frames: self.free_list.len() + 1, // Approximate
            free_frames: self.free_list.len(),
            allocated_frames: 0, // Would need tracking for accurate count
        }
    }
}

#[derive(Debug, Clone)]
pub struct AllocatorStats {
    pub total_frames: usize,
    pub free_frames: usize,
    pub allocated_frames: usize,
}

pub struct PoolAllocator {
    pools: Vec<MemoryPool>,
}

struct MemoryPool {
    block_size: usize,
    free_blocks: Vec<*mut u8>,
    total_blocks: usize,
}

impl PoolAllocator {
    pub fn new() -> Self {
        PoolAllocator {
            pools: Vec::new(),
        }
    }
    
    pub fn add_pool(&mut self, block_size: usize, num_blocks: usize) {
        // Allocate memory for the pool
        let pool_memory = unsafe {
            let ptr = multios_alloc(num_blocks * block_size);
            std::slice::from_raw_parts_mut(ptr, num_blocks * block_size)
        };
        
        // Create free list
        let mut free_blocks = Vec::new();
        for i in 0..num_blocks {
            let block_ptr = &mut pool_memory[i * block_size] as *mut u8;
            free_blocks.push(block_ptr);
        }
        
        self.pools.push(MemoryPool {
            block_size,
            free_blocks,
            total_blocks: num_blocks,
        });
        
        info!("Added pool: {} blocks of size {} bytes", num_blocks, block_size);
    }
    
    pub fn allocate(&mut self, size: usize) -> Option<*mut u8> {
        // Find the smallest pool that can accommodate the requested size
        for pool in &mut self.pools {
            if pool.block_size >= size && !pool.free_blocks.is_empty() {
                return pool.free_blocks.pop();
            }
        }
        
        None
    }
    
    pub fn deallocate(&mut self, ptr: *mut u8, size: usize) {
        // Find the appropriate pool and return the block
        for pool in &mut self.pools {
            if pool.block_size == size {
                pool.free_blocks.push(ptr);
                return;
            }
        }
        
        // If no matching pool found, this is an error
        error!("Attempted to deallocate block of unknown size: {}", size);
    }
}

// Unsafe allocator functions (for demonstration only)
unsafe fn multios_alloc(size: usize) -> *mut u8 {
    // This would integrate with MultiOS's actual memory allocation
    // For now, just return null to indicate unimplemented
    std::ptr::null_mut()
}

// Example usage:
// let mut allocator = SimpleAllocator::new();
// allocator.initialize(1024);
// let frame = allocator.allocate(4096).expect("Allocation failed");
// allocator.deallocate(frame);

//=============================================================================
// PROJECT 7: Process Manager
//=============================================================================
// Demonstrates process creation and management

use multios::process::{Process, ProcessId, ProcessState};
use multios::thread::{Thread, ThreadId};
use multios::signal::{Signal, SignalHandler};
use multios::time::{Duration, Instant};

pub struct ProcessManager {
    processes: HashMap<ProcessId, Process>,
    next_pid: SpinLock<ProcessId>,
}

impl ProcessManager {
    pub fn new() -> Self {
        ProcessManager {
            processes: HashMap::new(),
            next_pid: SpinLock::new(1),
        }
    }
    
    pub fn create_process(&mut self, name: &str, entry_point: fn()) -> Result<ProcessId, ProcessError> {
        let mut next_pid = self.next_pid.lock();
        let pid = *next_pid;
        *next_pid += 1;
        
        let process = Process::new(pid, String::from(name), entry_point)?;
        
        // Insert into process table
        self.processes.insert(pid, process);
        
        info!("Created process '{}' with PID {}", name, pid);
        Ok(pid)
    }
    
    pub fn kill_process(&mut self, pid: ProcessId) -> Result<(), ProcessError> {
        if let Some(process) = self.processes.remove(&pid) {
            // Clean up process resources
            process.terminate()?;
            
            info!("Terminated process with PID {}", pid);
            Ok(())
        } else {
            Err(ProcessError::ProcessNotFound)
        }
    }
    
    pub fn get_process(&self, pid: ProcessId) -> Option<&Process> {
        self.processes.get(&pid)
    }
    
    pub fn list_processes(&self) -> Vec<ProcessInfo> {
        self.processes.values().map(|proc| ProcessInfo {
            pid: proc.pid(),
            name: proc.name().to_string(),
            state: proc.state(),
            priority: proc.priority(),
        }).collect()
    }
    
    pub fn set_process_priority(&mut self, pid: ProcessId, priority: u8) -> Result<(), ProcessError> {
        if let Some(process) = self.processes.get_mut(&pid) {
            process.set_priority(priority);
            info!("Set priority {} for process {}", priority, pid);
            Ok(())
        } else {
            Err(ProcessError::ProcessNotFound)
        }
    }
    
    pub fn send_signal(&mut self, pid: ProcessId, signal: Signal) -> Result<(), ProcessError> {
        if let Some(process) = self.processes.get_mut(&pid) {
            process.send_signal(signal)?;
            info!("Sent signal {:?} to process {}", signal, pid);
            Ok(())
        } else {
            Err(ProcessError::ProcessNotFound)
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: ProcessId,
    pub name: String,
    pub state: ProcessState,
    pub priority: u8,
}

#[derive(Debug)]
pub enum ProcessError {
    ProcessNotFound,
    ProcessCreationFailed,
    InvalidPriority,
}

// Example usage:
// let mut pm = ProcessManager::new();
// let pid = pm.create_process("echo", echo_main)?;
// pm.set_process_priority(pid, 5)?;

//=============================================================================
// PROJECT 8: Configuration Manager
//=============================================================================
// Demonstrates system configuration management

use multios::config::{Config, ConfigValue};
use multios::fs::{File, FileMode};
use multios::serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SystemConfig {
    pub kernel: KernelConfig,
    pub memory: MemoryConfig,
    pub network: NetworkConfig,
    pub gui: GuiConfig,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KernelConfig {
    pub version: String,
    pub scheduler_algorithm: String,
    pub max_processes: usize,
    pub time_slice_ms: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MemoryConfig {
    pub total_memory_mb: usize,
    pub page_size: usize,
    pub enable_swap: bool,
    pub swap_file: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NetworkConfig {
    pub ip_address: String,
    pub subnet_mask: String,
    pub gateway: String,
    pub dns_servers: Vec<String>,
    pub enable_dhcp: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GuiConfig {
    pub resolution: (u32, u32),
    pub color_depth: u8,
    pub enable_vsync: bool,
    pub theme: String,
}

impl SystemConfig {
    pub fn new() -> Self {
        SystemConfig {
            kernel: KernelConfig {
                version: String::from("1.0.0"),
                scheduler_algorithm: String::from("round_robin"),
                max_processes: 1000,
                time_slice_ms: 10,
            },
            memory: MemoryConfig {
                total_memory_mb: 1024,
                page_size: 4096,
                enable_swap: true,
                swap_file: String::from("/swapfile"),
            },
            network: NetworkConfig {
                ip_address: String::from("192.168.1.100"),
                subnet_mask: String::from("255.255.255.0"),
                gateway: String::from("192.168.1.1"),
                dns_servers: vec![String::from("8.8.8.8")],
                enable_dhcp: false,
            },
            gui: GuiConfig {
                resolution: (1024, 768),
                color_depth: 32,
                enable_vsync: true,
                theme: String::from("default"),
            },
        }
    }
    
    pub fn load_from_file(&mut self, config_path: &str) -> Result<(), ConfigError> {
        let mut file = File::open(config_path, FileMode::Read)?;
        
        // Read configuration file content
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        
        // Parse configuration (using JSON for simplicity)
        let config: Self = serde_json::from_str(&content)
            .map_err(|e| ConfigError::ParseError(e.to_string()))?;
        
        *self = config;
        info!("Loaded configuration from {}", config_path);
        Ok(())
    }
    
    pub fn save_to_file(&self, config_path: &str) -> Result<(), ConfigError> {
        let mut file = File::create(config_path, FileMode::Write)?;
        
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| ConfigError::SerializeError(e.to_string()))?;
        
        file.write_all(content.as_bytes())?;
        
        info!("Saved configuration to {}", config_path);
        Ok(())
    }
    
    pub fn get_kernel_version(&self) -> &str {
        &self.kernel.version
    }
    
    pub fn set_memory_size(&mut self, size_mb: usize) {
        self.memory.total_memory_mb = size_mb;
    }
    
    pub fn get_network_config(&self) -> &NetworkConfig {
        &self.network
    }
}

#[derive(Debug)]
pub enum ConfigError {
    FileNotFound,
    ParseError(String),
    SerializeError(String),
    InvalidValue,
}

// Example usage:
// let mut config = SystemConfig::new();
// config.load_from_file("/etc/multios.conf")?;
// config.set_memory_size(2048);
// config.save_to_file("/etc/multios.conf")?;

//=============================================================================
// PROJECT 9: Logging System
//=============================================================================
// Demonstrates comprehensive logging and monitoring

use multios::sync::{Mutex, Arc};
use multios::time::Instant;
use multios::thread::ThreadSpawner;
use multios::fs::{File, FileMode};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LogLevel {
    Error = 0,
    Warn = 1,
    Info = 2,
    Debug = 3,
    Trace = 4,
}

pub struct LogEntry {
    pub timestamp: Instant,
    pub level: LogLevel,
    pub component: String,
    pub message: String,
    pub thread_id: usize,
}

pub struct Logger {
    log_level: LogLevel,
    components: HashSet<String>,
    log_file: Option<File>,
    buffer: Arc<Mutex<Vec<LogEntry>>>,
    async_writer: bool,
}

impl Logger {
    pub fn new() -> Self {
        Logger {
            log_level: LogLevel::Info,
            components: HashSet::new(),
            log_file: None,
            buffer: Arc::new(Mutex::new(Vec::new())),
            async_writer: true,
        }
    }
    
    pub fn with_file(&mut self, filename: &str) -> Result<(), LoggerError> {
        let file = File::create(filename, FileMode::Write)
            .map_err(|e| LoggerError::FileError(e.to_string()))?;
        self.log_file = Some(file);
        Ok(())
    }
    
    pub fn set_log_level(&mut self, level: LogLevel) {
        self.log_level = level;
    }
    
    pub fn enable_component(&mut self, component: &str) {
        self.components.insert(String::from(component));
    }
    
    pub fn log(&self, level: LogLevel, component: &str, message: &str) {
        if level as usize > self.log_level as usize {
            return;
        }
        
        if !self.components.is_empty() && !self.components.contains(component) {
            return;
        }
        
        let entry = LogEntry {
            timestamp: Instant::now(),
            level,
            component: String::from(component),
            message: String::from(message),
            thread_id: 0, // TODO: Get actual thread ID
        };
        
        if self.async_writer {
            self.log_async(entry);
        } else {
            self.log_sync(entry);
        }
    }
    
    pub fn error(&self, component: &str, message: &str) {
        self.log(LogLevel::Error, component, message);
    }
    
    pub fn warn(&self, component: &str, message: &str) {
        self.log(LogLevel::Warn, component, message);
    }
    
    pub fn info(&self, component: &str, message: &str) {
        self.log(LogLevel::Info, component, message);
    }
    
    pub fn debug(&self, component: &str, message: &str) {
        self.log(LogLevel::Debug, component, message);
    }
    
    pub fn trace(&self, component: &str, message: &str) {
        self.log(LogLevel::Trace, component, message);
    }
    
    fn log_async(&self, entry: LogEntry) {
        let buffer = Arc::clone(&self.buffer);
        ThreadSpawner::new()
            .spawn(move || {
                let mut buf = buffer.lock();
                buf.push(entry);
                
                // Flush buffer if it gets too large
                if buf.len() > 1000 {
                    buf.clear(); // TODO: Write to file
                }
            })
            .unwrap_or_else(|e| eprintln!("Logger thread spawn failed: {:?}", e));
    }
    
    fn log_sync(&self, entry: LogEntry) {
        let formatted = self.format_entry(&entry);
        
        if let Some(ref mut file) = self.log_file {
            let _ = file.write_all(formatted.as_bytes());
        } else {
            // Print to console
            match entry.level {
                LogLevel::Error => eprintln!("{}", formatted),
                LogLevel::Warn => eprintln!("{}", formatted),
                LogLevel::Info => println!("{}", formatted),
                LogLevel::Debug | LogLevel::Trace => println!("{}", formatted),
            }
        }
    }
    
    fn format_entry(&self, entry: &LogEntry) -> String {
        let level_str = match entry.level {
            LogLevel::Error => "ERROR",
            LogLevel::Warn => "WARN",
            LogLevel::Info => "INFO",
            LogLevel::Debug => "DEBUG",
            LogLevel::Trace => "TRACE",
        };
        
        format!("[{}] {} - {} - {}\n",
                entry.timestamp.elapsed().as_millis(),
                level_str,
                entry.component,
                entry.message)
    }
}

#[derive(Debug)]
pub enum LoggerError {
    FileError(String),
    BufferFull,
    WriteFailed,
}

// Global logger instance
lazy_static::lazy_static! {
    pub static ref GLOBAL_LOGGER: Arc<Mutex<Logger>> = Arc::new(Mutex::new(Logger::new()));
}

// Example usage:
// GLOBAL_LOGGER.lock().info("main", "MultiOS started successfully");
// GLOBAL_LOGGER.lock().error("network", "Failed to connect to server");

//=============================================================================
// PROJECT 10: System Monitor
//=============================================================================
// Demonstrates system monitoring and metrics collection

use multios::process::{ProcessManager, ProcessId};
use multios::memory::{MemoryInfo, MemoryStats};
use multios::cpu::{CpuInfo, CpuUsage};
use multios::time::{Duration, Instant};

pub struct SystemMonitor {
    process_manager: ProcessManager,
    memory_info: MemoryInfo,
    cpu_info: CpuInfo,
    history: CircularBuffer<SystemMetrics>,
}

impl SystemMonitor {
    pub fn new() -> Self {
        SystemMonitor {
            process_manager: ProcessManager::new(),
            memory_info: MemoryInfo::new(),
            cpu_info: CpuInfo::new(),
            history: CircularBuffer::new(1000), // Keep last 1000 samples
        }
    }
    
    pub fn collect_metrics(&mut self) -> SystemMetrics {
        let timestamp = Instant::now();
        
        let cpu_usage = self.cpu_info.get_usage();
        let memory_stats = self.memory_info.get_stats();
        let process_count = self.process_manager.list_processes().len();
        
        let metrics = SystemMetrics {
            timestamp,
            cpu_usage,
            memory_stats,
            process_count,
        };
        
        self.history.push(metrics.clone());
        
        metrics
    }
    
    pub fn get_cpu_usage(&self) -> f32 {
        if let Some(latest) = self.history.latest() {
            latest.cpu_usage.total_usage()
        } else {
            0.0
        }
    }
    
    pub fn get_memory_usage(&self) -> MemoryUsage {
        if let Some(latest) = self.history.latest() {
            MemoryUsage {
                used_mb: latest.memory_stats.used_pages * 4,
                total_mb: latest.memory_stats.total_pages * 4,
                percentage: (latest.memory_stats.used_pages as f32 / 
                           latest.memory_stats.total_pages as f32) * 100.0,
            }
        } else {
            MemoryUsage::default()
        }
    }
    
    pub fn get_top_processes(&self, count: usize) -> Vec<ProcessMetrics> {
        let processes = self.process_manager.list_processes();
        
        processes.into_iter()
            .map(|proc| ProcessMetrics {
                pid: proc.pid,
                name: proc.name,
                cpu_usage: 0.0, // TODO: Calculate actual CPU usage
                memory_mb: 0,   // TODO: Calculate actual memory usage
                state: proc.state,
            })
            .take(count)
            .collect()
    }
    
    pub fn get_load_average(&self) -> LoadAverage {
        let recent_metrics: Vec<_> = self.history.get_recent(60); // Last minute
        
        if recent_metrics.is_empty() {
            return LoadAverage::default();
        }
        
        let avg_1min = recent_metrics.iter()
            .map(|m| m.cpu_usage.total_usage())
            .sum::<f32>() / recent_metrics.len() as f32;
        
        LoadAverage {
            one_minute: avg_1min,
            five_minutes: avg_1min * 0.8, // Simplified calculation
            fifteen_minutes: avg_1min * 0.6,
        }
    }
    
    pub fn generate_report(&self) -> String {
        let cpu_usage = self.get_cpu_usage();
        let memory = self.get_memory_usage();
        let load_avg = self.get_load_average();
        let top_processes = self.get_top_processes(5);
        
        format!(
            "=== MultiOS System Report ===
CPU Usage: {:.1}%
Memory: {}/{} MB ({:.1}%)
Load Average: 1min: {:.2}, 5min: {:.2}, 15min: {:.2}

Top Processes:
{}
=== End Report ===",
            cpu_usage,
            memory.used_mb,
            memory.total_mb,
            memory.percentage,
            load_avg.one_minute,
            load_avg.five_minutes,
            load_avg.fifteen_minutes,
            top_processes.iter()
                .map(|p| format!("{} (PID: {}, CPU: {:.1}%, Mem: {} MB)", 
                               p.name, p.pid, p.cpu_usage, p.memory_mb))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
}

#[derive(Debug, Clone)]
pub struct SystemMetrics {
    pub timestamp: Instant,
    pub cpu_usage: CpuUsage,
    pub memory_stats: MemoryStats,
    pub process_count: usize,
}

#[derive(Debug, Clone)]
pub struct MemoryUsage {
    pub used_mb: usize,
    pub total_mb: usize,
    pub percentage: f32,
}

impl Default for MemoryUsage {
    fn default() -> Self {
        MemoryUsage {
            used_mb: 0,
            total_mb: 0,
            percentage: 0.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProcessMetrics {
    pub pid: ProcessId,
    pub name: String,
    pub cpu_usage: f32,
    pub memory_mb: usize,
    pub state: ProcessState,
}

#[derive(Debug, Default, Clone)]
pub struct LoadAverage {
    pub one_minute: f32,
    pub five_minutes: f32,
    pub fifteen_minutes: f32,
}

struct CircularBuffer<T> {
    buffer: Vec<T>,
    capacity: usize,
    write_index: usize,
    filled: bool,
}

impl<T: Clone> CircularBuffer<T> {
    pub fn new(capacity: usize) -> Self {
        CircularBuffer {
            buffer: Vec::with_capacity(capacity),
            capacity,
            write_index: 0,
            filled: false,
        }
    }
    
    pub fn push(&mut self, item: T) {
        if self.buffer.len() < self.capacity {
            self.buffer.push(item);
        } else {
            self.buffer[self.write_index] = item;
            self.write_index = (self.write_index + 1) % self.capacity;
            if self.write_index == 0 {
                self.filled = true;
            }
        }
    }
    
    pub fn latest(&self) -> Option<&T> {
        if self.buffer.is_empty() {
            None
        } else {
            Some(&self.buffer[(self.write_index + self.buffer.len() - 1) % self.buffer.len()])
        }
    }
    
    pub fn get_recent(&self, count: usize) -> Vec<T> {
        if self.buffer.is_empty() {
            return Vec::new();
        }
        
        let actual_count = count.min(self.buffer.len());
        let mut items = Vec::new();
        
        for i in 0..actual_count {
            let index = (self.write_index + self.buffer.len() + i - actual_count) % self.buffer.len();
            items.push(self.buffer[index].clone());
        }
        
        items
    }
}

// Example usage:
// let mut monitor = SystemMonitor::new();
// loop {
//     let metrics = monitor.collect_metrics();
//     println!("CPU: {:.1}%, Memory: {:.1}%", 
//              monitor.get_cpu_usage(), 
//              monitor.get_memory_usage().percentage);
//     std::thread::sleep(Duration::from_secs(5));
// }

//=============================================================================
// End of Example Projects
//=============================================================================

// This collection of example projects demonstrates:
// 1. Basic kernel module development
// 2. Character device driver implementation
// 3. Simple file system creation
// 4. Network server implementation
// 5. GUI application development
// 6. Memory allocation strategies
// 7. Process management
// 8. Configuration handling
// 9. Logging and monitoring
// 10. System metrics collection
//
// Each project can be compiled and run independently to demonstrate
// specific aspects of MultiOS development.