//! I/O Hardware Abstraction Layer
//!
//! This module provides unified I/O interfaces across architectures for
//! port-mapped I/O, memory-mapped I/O, and device operations.

use crate::log::{info, warn, error};
use crate::{KernelError, Result};
use spin::RwLock;
use core::sync::atomic::{AtomicU64, Ordering};

/// I/O subsystem initialization
pub fn init() -> Result<()> {
    info!("Initializing I/O HAL...");
    
    // Detect I/O architecture
    detect_io_architecture()?;
    
    // Initialize I/O controllers
    init_io_controllers()?;
    
    // Set up device management
    setup_device_management()?;
    
    // Initialize I/O memory management
    init_io_memory_management()?;
    
    Ok(())
}

/// I/O subsystem shutdown
pub fn shutdown() -> Result<()> {
    info!("Shutting down I/O HAL...");
    Ok(())
}

/// I/O architectures
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum IoArchitecture {
    PortMapped = 0,      // x86 I/O ports
    MemoryMapped = 1,    // Memory-mapped I/O
    MMIO = 1,           // Alias for memory-mapped
    VirtIO = 2,         // Virtual I/O (paravirtualized)
    Custom = 3,
}

/// I/O operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum IoOperation {
    Read8 = 0,
    Read16 = 1,
    Read32 = 2,
    Read64 = 3,
    Write8 = 4,
    Write16 = 5,
    Write32 = 6,
    Write64 = 7,
    ReadBuffer = 8,
    WriteBuffer = 9,
}

/// I/O device types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum IoDeviceType {
    Keyboard = 0,
    Serial = 1,
    Vga = 2,
    Network = 3,
    Storage = 4,
    Usb = 5,
    Audio = 6,
    Timer = 7,
    Interrupt = 8,
    Random = 9,
}

/// I/O device descriptor
#[derive(Debug, Clone)]
pub struct IoDevice {
    pub device_type: IoDeviceType,
    pub name: &'static str,
    pub base_address: usize,
    pub size: usize,
    pub io_architecture: IoArchitecture,
    pub enabled: bool,
    pub interrupts: Vec<u32>,
}

/// I/O statistics
#[derive(Debug, Clone, Copy)]
pub struct IoStats {
    pub io_operations: AtomicU64,
    pub bytes_read: AtomicU64,
    pub bytes_written: AtomicU64,
    pub io_errors: AtomicU64,
    pub device_errors: AtomicU64,
    pub latency_ns: AtomicU64,
}

/// I/O capabilities
#[derive(Debug, Clone)]
pub struct IoCapabilities {
    pub supports_port_io: bool,
    pub supports_mmio: bool,
    pub max_io_space: u64,
    pub supported_devices: Vec<IoDeviceType>,
}

/// Device I/O table
static DEVICES: RwLock<Vec<IoDevice>> = RwLock::new(Vec::new());

/// I/O architecture
static IO_ARCH: RwLock<IoArchitecture> = RwLock::new(IoArchitecture::Custom);

/// I/O capabilities
static IO_CAPABILITIES: RwLock<IoCapabilities> = RwLock::new(IoCapabilities {
    supports_port_io: false,
    supports_mmio: false,
    max_io_space: 0,
    supported_devices: Vec::new(),
});

/// I/O statistics
static IO_STATS: IoStats = IoStats {
    io_operations: AtomicU64::new(0),
    bytes_read: AtomicU64::new(0),
    bytes_written: AtomicU64::new(0),
    io_errors: AtomicU64::new(0),
    device_errors: AtomicU64::new(0),
    latency_ns: AtomicU64::new(0),
};

/// Detect I/O architecture
fn detect_io_architecture() -> Result<()> {
    info!("Detecting I/O architecture...");
    
    let arch = detect_io_architecture_arch()?;
    *IO_ARCH.write() = arch;
    
    match arch {
        IoArchitecture::PortMapped => info!("Detected port-mapped I/O architecture"),
        IoArchitecture::MemoryMapped => info!("Detected memory-mapped I/O architecture"),
        IoArchitecture::VirtIO => info!("Detected VirtIO architecture"),
        IoArchitecture::Custom => warn!("Using custom I/O architecture"),
    }
    
    // Detect capabilities
    detect_io_capabilities();
    
    Ok(())
}

/// Architecture-specific I/O detection
#[cfg(target_arch = "x86_64")]
fn detect_io_architecture_arch() -> Result<IoArchitecture> {
    // x86_64 supports both port and memory-mapped I/O
    Ok(IoArchitecture::PortMapped)
}

#[cfg(target_arch = "aarch64")]
fn detect_io_architecture_arch() -> Result<IoArchitecture> {
    // ARM64 only supports memory-mapped I/O
    Ok(IoArchitecture::MemoryMapped)
}

#[cfg(target_arch = "riscv64")]
fn detect_io_architecture_arch() -> Result<IoArchitecture> {
    // RISC-V only supports memory-mapped I/O
    Ok(IoArchitecture::MemoryMapped)
}

/// Detect I/O capabilities
fn detect_io_capabilities() {
    let mut capabilities = IoCapabilities {
        supports_port_io: false,
        supports_mmio: false,
        max_io_space: 0,
        supported_devices: Vec::new(),
    };
    
    match *IO_ARCH.read() {
        IoArchitecture::PortMapped => {
            capabilities.supports_port_io = true;
            capabilities.max_io_space = 0xFFFF; // 64KB I/O space
            info!("Port I/O capabilities: 64KB I/O space");
        }
        IoArchitecture::MemoryMapped => {
            capabilities.supports_mmio = true;
            capabilities.max_io_space = u64::MAX; // Virtually unlimited
            info!("Memory-mapped I/O capabilities: Virtually unlimited space");
        }
        _ => {}
    }
    
    // Standard devices for all architectures
    capabilities.supported_devices.extend_from_slice(&[
        IoDeviceType::Serial,
        IoDeviceType::Timer,
        IoDeviceType::Interrupt,
    ]);
    
    #[cfg(target_arch = "x86_64")]
    {
        capabilities.supported_devices.push(IoDeviceType::Keyboard);
        capabilities.supported_devices.push(IoDeviceType::Vga);
    }
    
    *IO_CAPABILITIES.write() = capabilities;
}

/// Initialize I/O controllers
fn init_io_controllers() -> Result<()> {
    info!("Initializing I/O controllers...");
    
    // Initialize standard devices
    init_standard_devices()?;
    
    // Configure I/O controllers
    configure_io_controllers()?;
    
    Ok(())
}

/// Initialize standard devices
fn init_standard_devices() -> Result<()> {
    // Initialize serial port
    init_serial_device()?;
    
    // Initialize timer device
    init_timer_device()?;
    
    // Initialize interrupt controller I/O
    init_interrupt_device()?;
    
    #[cfg(target_arch = "x86_64")]
    {
        // Initialize keyboard controller
        init_keyboard_device()?;
        
        // Initialize VGA controller
        init_vga_device()?;
    }
    
    Ok(())
}

/// Initialize serial device
fn init_serial_device() -> Result<()> {
    info!("Initializing serial device...");
    
    let serial_device = IoDevice {
        device_type: IoDeviceType::Serial,
        name: "Serial Port",
        base_address: 0x3F8, // COM1
        size: 8,
        io_architecture: IoArchitecture::PortMapped,
        enabled: true,
        interrupts: vec![4], // IRQ4
    };
    
    add_device(serial_device)?;
    Ok(())
}

/// Initialize timer device
fn init_timer_device() -> Result<()> {
    info!("Initializing timer device...");
    
    let timer_device = IoDevice {
        device_type: IoDeviceType::Timer,
        name: "System Timer",
        base_address: get_timer_base_address(),
        size: 64,
        io_architecture: IoArchitecture::MemoryMapped,
        enabled: true,
        interrupts: vec![0], // Timer interrupt
    };
    
    add_device(timer_device)?;
    Ok(())
}

/// Initialize interrupt device
fn init_interrupt_device() -> Result<()> {
    info!("Initializing interrupt controller device...");
    
    let interrupt_device = IoDevice {
        device_type: IoDeviceType::Interrupt,
        name: "Interrupt Controller",
        base_address: get_interrupt_controller_address(),
        size: 4096,
        io_architecture: IoArchitecture::MemoryMapped,
        enabled: true,
        interrupts: Vec::new(), // Master interrupt controller
    };
    
    add_device(interrupt_device)?;
    Ok(())
}

#[cfg(target_arch = "x86_64")]
fn init_keyboard_device() -> Result<()> {
    info!("Initializing keyboard device...");
    
    let keyboard_device = IoDevice {
        device_type: IoDeviceType::Keyboard,
        name: "PS/2 Keyboard",
        base_address: 0x60, // Keyboard data port
        size: 4,
        io_architecture: IoArchitecture::PortMapped,
        enabled: true,
        interrupts: vec![1], // IRQ1
    };
    
    add_device(keyboard_device)?;
    Ok(())
}

#[cfg(target_arch = "x86_64")]
fn init_vga_device() -> Result<()> {
    info!("Initializing VGA device...");
    
    let vga_device = IoDevice {
        device_type: IoDeviceType::Vga,
        name: "VGA Display",
        base_address: 0xB8000, // VGA text mode memory
        size: 32 * 1024, // 32KB VGA memory
        io_architecture: IoArchitecture::MemoryMapped,
        enabled: true,
        interrupts: Vec::new(),
    };
    
    add_device(vga_device)?;
    Ok(())
}

/// Configure I/O controllers
fn configure_io_controllers() -> Result<()> {
    info!("Configuring I/O controllers...");
    
    let arch = *IO_ARCH.read();
    
    match arch {
        IoArchitecture::PortMapped => configure_port_io(),
        IoArchitecture::MemoryMapped => configure_mmio(),
        _ => warn!("Unknown I/O architecture for configuration"),
    }
    
    Ok(())
}

fn configure_port_io() {
    #[cfg(target_arch = "x86_64")]
    {
        info!("Configuring x86_64 port I/O");
        // Configure PIC for port I/O
        // Configure serial ports
    }
}

fn configure_mmio() {
    info!("Configuring memory-mapped I/O");
    // Configure memory-mapped device regions
}

/// Setup device management
fn setup_device_management() -> Result<()> {
    info!("Setting up device management...");
    
    // Register device discovery mechanisms
    setup_device_discovery()?;
    
    // Set up device interrupt handling
    setup_device_interrupts()?;
    
    Ok(())
}

/// Setup device discovery
fn setup_device_discovery() -> Result<()> {
    info!("Setting up device discovery...");
    
    // Set up ACPI/PCI discovery in the future
    info!("Device discovery initialized");
    
    Ok(())
}

/// Setup device interrupts
fn setup_device_interrupts() -> Result<()> {
    info!("Setting up device interrupts...");
    
    // Enable interrupts for configured devices
    let devices = DEVICES.read();
    for device in devices.iter() {
        if device.enabled && !device.interrupts.is_empty() {
            info!("Enabling interrupts for {} (IRQ {:?})", device.name, device.interrupts);
        }
    }
    
    Ok(())
}

/// Initialize I/O memory management
fn init_io_memory_management() -> Result<()> {
    info!("Initializing I/O memory management...");
    
    // Reserve I/O memory regions
    reserve_io_memory_regions()?;
    
    // Set up device memory mappings
    setup_device_memory_mappings()?;
    
    Ok(())
}

/// Reserve I/O memory regions
fn reserve_io_memory_regions() -> Result<()> {
    info!("Reserving I/O memory regions...");
    
    // Reserve standard device memory regions
    let regions = get_standard_io_regions();
    
    for region in &regions {
        info!("Reserving I/O region: {:#x} - {:#x} ({})", 
              region.0, region.0 + region.1, region.2);
    }
    
    Ok(())
}

/// Get standard I/O regions
fn get_standard_io_regions() -> Vec<(usize, usize, &'static str)> {
    let mut regions = Vec::new();
    
    #[cfg(target_arch = "x86_64")]
    {
        regions.push((0xB8000, 32 * 1024, "VGA Text Memory"));
        regions.push((0xFEC00000, 4 * 1024, "IO-APIC"));
        regions.push((0xFED00000, 64 * 1024, "HPET"));
    }
    
    #[cfg(target_arch = "aarch64")]
    {
        regions.push((0x1C000000, 256 * 1024, "GICv2"));
        regions.push((0x1C010000, 256 * 1024, "GICv2 CPU Interface"));
    }
    
    #[cfg(target_arch = "riscv64")]
    {
        regions.push((0x2000000, 256 * 1024, "PLIC"));
        regions.push((0x2004000, 4 * 1024, "CLINT"));
    }
    
    regions
}

/// Setup device memory mappings
fn setup_device_memory_mappings() -> Result<()> {
    info!("Setting up device memory mappings...");
    
    // Map device memory regions
    let devices = DEVICES.read();
    for device in devices.iter() {
        if device.io_architecture == IoArchitecture::MemoryMapped {
            info!("Mapping {} at {:#x} - {:#x}", 
                  device.name, device.base_address, device.base_address + device.size);
        }
    }
    
    Ok(())
}

/// Add device to device table
fn add_device(device: IoDevice) -> Result<()> {
    let mut devices = DEVICES.write();
    devices.push(device);
    Ok(())
}

/// Get device by type
pub fn get_device_by_type(device_type: IoDeviceType) -> Option<IoDevice> {
    let devices = DEVICES.read();
    devices.iter().find(|d| d.device_type == device_type).cloned()
}

/// Get all devices
pub fn get_all_devices() -> Vec<IoDevice> {
    DEVICES.read().clone()
}

/// Get I/O architecture
pub fn get_io_architecture() -> IoArchitecture {
    *IO_ARCH.read()
}

/// Get I/O capabilities
pub fn get_io_capabilities() -> IoCapabilities {
    *IO_CAPABILITIES.read()
}

/// Port I/O operations (x86_64 only)
#[cfg(target_arch = "x86_64")]
pub mod port_io {
    use super::*;
    
    /// Read 8-bit from I/O port
    pub fn inb(port: u16) -> u8 {
        let value: u8;
        unsafe {
            core::arch::asm!("inb {}", in(reg) port, out(reg) value);
        }
        IO_STATS.io_operations.fetch_add(1, Ordering::SeqCst);
        value
    }
    
    /// Read 16-bit from I/O port
    pub fn inw(port: u16) -> u16 {
        let value: u16;
        unsafe {
            core::arch::asm!("inw {}", in(reg) port, out(reg) value);
        }
        IO_STATS.io_operations.fetch_add(1, Ordering::SeqCst);
        value
    }
    
    /// Read 32-bit from I/O port
    pub fn inl(port: u16) -> u32 {
        let value: u32;
        unsafe {
            core::arch::asm!("inl {}", in(reg) port, out(reg) value);
        }
        IO_STATS.io_operations.fetch_add(1, Ordering::SeqCst);
        value
    }
    
    /// Write 8-bit to I/O port
    pub fn outb(port: u16, value: u8) {
        unsafe {
            core::arch::asm!("outb {}, {}", in(reg) value, in(reg) port);
        }
        IO_STATS.io_operations.fetch_add(1, Ordering::SeqCst);
    }
    
    /// Write 16-bit to I/O port
    pub fn outw(port: u16, value: u16) {
        unsafe {
            core::arch::asm!("outw {}, {}", in(reg) value, in(reg) port);
        }
        IO_STATS.io_operations.fetch_add(1, Ordering::SeqCst);
    }
    
    /// Write 32-bit to I/O port
    pub fn outl(port: u16, value: u32) {
        unsafe {
            core::arch::asm!("outl {}, {}", in(reg) value, in(reg) port);
        }
        IO_STATS.io_operations.fetch_add(1, Ordering::SeqCst);
    }
    
    /// Read buffer from I/O port
    pub fn inb_buffer(port: u16, buffer: &mut [u8]) {
        unsafe {
            core::arch::asm!(
                "rep insb",
                in("dx") port,
                in("rdi") buffer.as_mut_ptr(),
                in("rcx") buffer.len(),
                out("rax") _, // clobbered
            );
        }
        IO_STATS.bytes_read.fetch_add(buffer.len() as u64, Ordering::SeqCst);
        IO_STATS.io_operations.fetch_add(1, Ordering::SeqCst);
    }
    
    /// Write buffer to I/O port
    pub fn outb_buffer(port: u16, buffer: &[u8]) {
        unsafe {
            core::arch::asm!(
                "rep outsb",
                in("dx") port,
                in("rsi") buffer.as_ptr(),
                in("rcx") buffer.len(),
                out("rax") _, // clobbered
            );
        }
        IO_STATS.bytes_written.fetch_add(buffer.len() as u64, Ordering::SeqCst);
        IO_STATS.io_operations.fetch_add(1, Ordering::SeqCst);
    }
}

/// Memory-mapped I/O operations
pub mod mmio {
    use super::*;
    use volatile::Volatile;
    
    /// Read 8-bit from memory-mapped I/O
    pub fn read8(address: usize) -> u8 {
        let value = unsafe { Volatile::new(address as *const u8).read() };
        IO_STATS.io_operations.fetch_add(1, Ordering::SeqCst);
        value
    }
    
    /// Read 16-bit from memory-mapped I/O
    pub fn read16(address: usize) -> u16 {
        let value = unsafe { Volatile::new(address as *const u16).read() };
        IO_STATS.io_operations.fetch_add(1, Ordering::SeqCst);
        value
    }
    
    /// Read 32-bit from memory-mapped I/O
    pub fn read32(address: usize) -> u32 {
        let value = unsafe { Volatile::new(address as *const u32).read() };
        IO_STATS.io_operations.fetch_add(1, Ordering::SeqCst);
        value
    }
    
    /// Read 64-bit from memory-mapped I/O
    pub fn read64(address: usize) -> u64 {
        let value = unsafe { Volatile::new(address as *const u64).read() };
        IO_STATS.io_operations.fetch_add(1, Ordering::SeqCst);
        value
    }
    
    /// Write 8-bit to memory-mapped I/O
    pub fn write8(address: usize, value: u8) {
        unsafe { Volatile::new(address as *mut u8).write(value) };
        IO_STATS.io_operations.fetch_add(1, Ordering::SeqCst);
    }
    
    /// Write 16-bit to memory-mapped I/O
    pub fn write16(address: usize, value: u16) {
        unsafe { Volatile::new(address as *mut u16).write(value) };
        IO_STATS.io_operations.fetch_add(1, Ordering::SeqCst);
    }
    
    /// Write 32-bit to memory-mapped I/O
    pub fn write32(address: usize, value: u32) {
        unsafe { Volatile::new(address as *mut u32).write(value) };
        IO_STATS.io_operations.fetch_add(1, Ordering::SeqCst);
    }
    
    /// Write 64-bit to memory-mapped I/O
    pub fn write64(address: usize, value: u64) {
        unsafe { Volatile::new(address as *mut u64).write(value) };
        IO_STATS.io_operations.fetch_add(1, Ordering::SeqCst);
    }
    
    /// Read buffer from memory-mapped I/O
    pub fn read_buffer(address: usize, buffer: &mut [u8]) {
        unsafe {
            let src = address as *const u8;
            for (i, dst) in buffer.iter_mut().enumerate() {
                *dst = Volatile::new(src.add(i)).read();
            }
        }
        IO_STATS.bytes_read.fetch_add(buffer.len() as u64, Ordering::SeqCst);
        IO_STATS.io_operations.fetch_add(1, Ordering::SeqCst);
    }
    
    /// Write buffer to memory-mapped I/O
    pub fn write_buffer(address: usize, buffer: &[u8]) {
        unsafe {
            let dst = address as *mut u8;
            for (i, &src) in buffer.iter().enumerate() {
                Volatile::new(dst.add(i)).write(src);
            }
        }
        IO_STATS.bytes_written.fetch_add(buffer.len() as u64, Ordering::SeqCst);
        IO_STATS.io_operations.fetch_add(1, Ordering::SeqCst);
    }
}

/// Get I/O statistics
pub fn get_stats() -> IoStats {
    IO_STATS
}

/// Utility functions
pub mod utils {
    use super::*;
    
    /// Check if I/O address is valid
    pub fn is_valid_io_address(address: usize, size: usize) -> bool {
        let capabilities = get_io_capabilities();
        
        match get_io_architecture() {
            IoArchitecture::PortMapped => {
                address < capabilities.max_io_space && address + size <= capabilities.max_io_space
            }
            IoArchitecture::MemoryMapped => {
                // Assume all addresses in system RAM range are valid for now
                address >= 0x1000 && address + size <= 0xFFFF_FFFF_FFFF
            }
            _ => false,
        }
    }
    
    /// Get device base address
    pub fn get_device_base_address(device_type: IoDeviceType) -> Option<usize> {
        get_device_by_type(device_type).map(|d| d.base_address)
    }
    
    /// Check if device is available
    pub fn is_device_available(device_type: IoDeviceType) -> bool {
        get_device_by_type(device_type)
            .map(|d| d.enabled)
            .unwrap_or(false)
    }
    
    /// Enable device
    pub fn enable_device(device_type: IoDeviceType) -> Result<()> {
        let mut devices = DEVICES.write();
        for device in &mut devices {
            if device.device_type == device_type {
                device.enabled = true;
                info!("Enabled device: {}", device.name);
                break;
            }
        }
        Ok(())
    }
    
    /// Disable device
    pub fn disable_device(device_type: IoDeviceType) -> Result<()> {
        let mut devices = DEVICES.write();
        for device in &mut devices {
            if device.device_type == device_type {
                device.enabled = false;
                info!("Disabled device: {}", device.name);
                break;
            }
        }
        Ok(())
    }
}

// Helper functions to get device addresses
fn get_timer_base_address() -> usize {
    #[cfg(target_arch = "x86_64")]
    return 0xFED00000; // HPET base address
    
    #[cfg(target_arch = "aarch64")]
    return 0x1C000000; // GIC timer base
    
    #[cfg(target_arch = "riscv64")]
    return 0x2004000; // CLINT base
    
    0
}

fn get_interrupt_controller_address() -> usize {
    #[cfg(target_arch = "x86_64")]
    return 0xFEC00000; // IO-APIC base
    
    #[cfg(target_arch = "aarch64")]
    return 0x1C000000; // GIC base
    
    #[cfg(target_arch = "riscv64")]
    return 0x2000000; // PLIC base
    
    0
}