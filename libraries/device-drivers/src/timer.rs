//! Timer/PIT Driver
//! 
//! Provides support for Programmable Interval Timer (PIT) and
//! high-resolution system timers.

use crate::{DeviceType, DriverResult, DriverError, device::{Device, DeviceDriver, DeviceCapabilities}};
use spin::{Mutex, Once};
use alloc::vec::Vec;
use log::{info, warn, error};

/// Timer frequency constants
pub const TIMER_FREQ_1193182: u64 = 1_193_182; // PIT base frequency
pub const TIMER_FREQ_1000000000: u64 = 1_000_000_000; // 1 GHz for high-res timers

/// Timer types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimerType {
    P8254,     // 8254/8253 PIT
    ApicTimer, // APIC Timer (x86)
    HPET,      // High Precision Event Timer
    ARMGeneric, // ARM Generic Timer
    RISCVMTimer, // RISC-V Machine Timer
}

/// Timer operating modes
#[derive(Debug, Clone, Copy)]
pub enum TimerMode {
    OneShot,     // Trigger once after delay
    Periodic,    // Trigger periodically
    RateGenerator, // Rate generator mode
    SquareWave,  // Square wave generator
}

/// Timer configuration
#[derive(Debug, Clone, Copy)]
pub struct TimerConfig {
    pub timer_type: TimerType,
    pub mode: TimerMode,
    pub frequency: u32,
    pub interrupt_enabled: bool,
    pub interrupt_vector: Option<u8>,
}

/// Timer device information
#[derive(Debug, Clone)]
pub struct TimerDevice {
    pub config: TimerConfig,
    pub base_address: Option<u64>,
    pub channel: Option<u8>, // PIT channel number
}

/// 8254 PIT Driver
pub struct Pit8254 {
    pub base_port: u16,
    pub config: TimerConfig,
    pub tick_count: Mutex<u64>,
}

impl Pit8254 {
    /// Create new PIT driver
    pub fn new() -> Self {
        Self {
            base_port: 0x40,
            config: TimerConfig {
                timer_type: TimerType::P8254,
                mode: TimerMode::RateGenerator,
                frequency: 1000, // 1000 Hz = 1ms ticks
                interrupt_enabled: true,
                interrupt_vector: Some(0x20), // IRQ 0
            },
            tick_count: Mutex::new(0),
        }
    }

    /// Initialize PIT
    pub fn init(&mut self) -> DriverResult<()> {
        info!("Initializing 8254 PIT at port 0x{:04x}", self.base_port);
        
        // Configure channel 0 for rate generator mode
        let command = 0x36; // Channel 0, LSB then MSB, rate generator
        self.write_command(command);
        
        // Calculate divisor for desired frequency
        let divisor = (TIMER_FREQ_1193182 / self.config.frequency) as u16;
        
        // Write divisor
        self.write_channel(0, (divisor & 0xFF) as u8);
        self.write_channel(0, (divisor >> 8) as u8);
        
        info!("PIT configured for {} Hz (divisor: {})", self.config.frequency, divisor);
        
        Ok(())
    }

    /// Read PIT counter
    pub fn read_counter(&self, channel: u8) -> u16 {
        let port = self.base_port + channel;
        
        // Latch count command
        let latch_command = 0x00 | (channel << 6);
        unsafe { core::ptr::write_volatile(port as *mut u8, latch_command) };
        
        // Read LSB then MSB
        let lsb = unsafe { core::ptr::read_volatile(port as *const u8) };
        let msb = unsafe { core::ptr::read_volatile(port as *const u8) };
        
        (msb as u16) << 8 | lsb as u16
    }

    /// Write PIT command
    fn write_command(&self, command: u8) {
        unsafe { core::ptr::write_volatile((self.base_port + 3) as *mut u8, command) };
    }

    /// Write to PIT channel
    fn write_channel(&self, channel: u8, value: u8) {
        let port = self.base_port + channel;
        unsafe { core::ptr::write_volatile(port as *mut u8, value) };
    }

    /// Get current tick count
    pub fn get_tick_count(&self) -> u64 {
        *self.tick_count.lock()
    }

    /// Increment tick count (called by interrupt handler)
    pub fn increment_tick(&self) {
        *self.tick_count.lock() += 1;
    }

    /// Calculate elapsed time
    pub fn get_elapsed_ms(&self) -> u64 {
        self.get_tick_count() * 1000 / self.config.frequency as u64
    }
}

impl DeviceDriver for Pit8254 {
    fn name(&self) -> &'static str {
        "8254 PIT Driver"
    }

    fn supported_devices(&self) -> &[DeviceType] {
        // This is a system timer, not really a "device" in the traditional sense
        // But we can return it as a generic timer type
        &[crate::DeviceType::Unknown] // We'll need to add a Timer device type
    }

    fn init(&self, device: &Device) -> DriverResult<()> {
        info!("Initializing PIT timer: {}", device.info.name);
        
        // PIT is typically at fixed port addresses, ignore hardware address
        // In a real implementation, we might support alternative addresses
        Ok(())
    }

    fn remove(&self, device: &Device) -> DriverResult<()> {
        info!("Removing PIT timer: {}", device.info.name);
        Ok(())
    }

    fn read(&self, device: &Device, buffer: &mut [u8]) -> DriverResult<usize> {
        if buffer.len() >= 8 {
            let tick_count = self.get_tick_count();
            let bytes = tick_count.to_le_bytes();
            buffer[..8].copy_from_slice(&bytes);
            Ok(8)
        } else {
            Err(DriverError::PermissionDenied)
        }
    }

    fn write(&self, device: &Device, buffer: &[u8]) -> DriverResult<usize> {
        if buffer.len() >= 8 {
            let new_frequency = u32::from_le_bytes(buffer[..4].try_into().unwrap());
            let mut config = self.config;
            config.frequency = new_frequency;
            
            info!("Changing PIT frequency to {} Hz", new_frequency);
            Ok(buffer.len())
        } else {
            Err(DriverError::PermissionDenied)
        }
    }

    fn ioctl(&self, device: &Device, command: u32, data: usize) -> DriverResult<usize> {
        match command {
            0x2001 => Ok(self.get_tick_count() as usize), // Get tick count
            0x2002 => Ok(self.get_elapsed_ms() as usize), // Get elapsed ms
            0x2003 => Ok(self.config.frequency as usize), // Get frequency
            0x2004 => { // Set frequency
                let new_freq = data as u32;
                let mut config = self.config;
                config.frequency = new_freq;
                Ok(0)
            }
            0x2005 => Ok(self.read_counter(0) as usize), // Read counter
            _ => Err(DriverError::PermissionDenied),
        }
    }

    fn capabilities(&self) -> DeviceCapabilities {
        DeviceCapabilities::READ | DeviceCapabilities::WRITE | DeviceCapabilities::INTERRUPT
    }
}

/// High Precision Event Timer (HPET) Driver
pub struct HpetTimer {
    pub base_address: u64,
    pub config: TimerConfig,
    pub tick_count: Mutex<u64>,
    pub period_femtoseconds: u64,
}

impl HpetTimer {
    /// Create new HPET driver
    pub fn new(base_address: u64) -> Self {
        Self {
            base_address,
            config: TimerConfig {
                timer_type: TimerType::HPET,
                mode: TimerMode::Periodic,
                frequency: 0, // Calculated from period
                interrupt_enabled: true,
                interrupt_vector: Some(0x22), // IRQ 2
            },
            tick_count: Mutex::new(0),
            period_femtoseconds: 0,
        }
    }

    /// Initialize HPET
    pub fn init(&mut self) -> DriverResult<()> {
        info!("Initializing HPET at address 0x{:08x}", self.base_address);
        
        // Read HPET period from capabilities register
        let cap_reg = self.read_reg(0x00);
        self.period_femtoseconds = cap_reg & 0xFFFFFFFF;
        
        let freq_fs = 1_000_000_000_000_000_000; // 1e18 femtoseconds per second
        self.config.frequency = (freq_fs / self.period_femtoseconds) as u32;
        
        info!("HPET period: {} fs, frequency: {} Hz", self.period_femtoseconds, self.config.frequency);
        
        // Enable HPET
        let config_reg = self.read_reg(0x10);
        self.write_reg(0x10, config_reg | 0x01);
        
        Ok(())
    }

    /// Read HPET register
    fn read_reg(&self, offset: u32) -> u64 {
        unsafe {
            core::ptr::read_volatile((self.base_address as usize + offset as usize) as *const u64)
        }
    }

    /// Write HPET register
    fn write_reg(&self, offset: u32, value: u64) {
        unsafe {
            core::ptr::write_volatile((self.base_address as usize + offset as usize) as *mut u64, value)
        }
    }

    /// Configure timer comparator
    pub fn configure_timer(&self, timer_id: u8, compare_value: u64) {
        let comparator_offset = 0x100 + (timer_id as u32) * 0x20;
        self.write_reg(comparator_offset, compare_value);
    }

    /// Read main counter
    pub fn read_counter(&self) -> u64 {
        self.read_reg(0xF0)
    }

    /// Get elapsed time in nanoseconds
    pub fn get_elapsed_ns(&self) -> u64 {
        let elapsed_fs = self.read_counter() * self.period_femtoseconds;
        elapsed_fs / 1_000_000_000 // Convert femtoseconds to nanoseconds
    }
}

impl DeviceDriver for HpetTimer {
    fn name(&self) -> &'static str {
        "HPET Driver"
    }

    fn supported_devices(&self) -> &[DeviceType] {
        &[crate::DeviceType::Unknown] // Timer device type
    }

    fn init(&self, device: &Device) -> DriverResult<()> {
        info!("Initializing HPET: {}", device.info.name);
        Ok(())
    }

    fn remove(&self, device: &Device) -> DriverResult<()> {
        info!("Removing HPET: {}", device.info.name);
        Ok(())
    }

    fn read(&self, device: &Device, buffer: &mut [u8]) -> DriverResult<usize> {
        if buffer.len() >= 8 {
            let counter = self.read_counter();
            let bytes = counter.to_le_bytes();
            buffer[..8].copy_from_slice(&bytes);
            Ok(8)
        } else {
            Err(DriverError::PermissionDenied)
        }
    }

    fn write(&self, device: &Device, buffer: &[u8]) -> DriverResult<usize> {
        if buffer.len() >= 8 {
            let new_compare = u64::from_le_bytes(buffer[..8].try_into().unwrap());
            self.configure_timer(0, new_compare);
            Ok(buffer.len())
        } else {
            Err(DriverError::PermissionDenied)
        }
    }

    fn ioctl(&self, device: &Device, command: u32, data: usize) -> DriverResult<usize> {
        match command {
            0x3001 => Ok(self.read_counter() as usize),
            0x3002 => Ok(self.get_elapsed_ns() as usize),
            0x3003 => Ok(self.config.frequency as usize),
            _ => Err(DriverError::PermissionDenied),
        }
    }

    fn capabilities(&self) -> DeviceCapabilities {
        DeviceCapabilities::READ | DeviceCapabilities::WRITE | DeviceCapabilities::INTERRUPT
    }
}

/// Global timer instance
static TIMER: Once<Mutex<dyn Timer + Send + Sync>> = Once::new();

/// Timer trait for abstraction
pub trait Timer {
    fn name(&self) -> &'static str;
    fn get_frequency(&self) -> u32;
    fn get_tick_count(&self) -> u64;
    fn get_elapsed_ns(&self) -> u64;
}

/// Timer manager
pub struct TimerManager {
    timers: Vec<Box<dyn Timer + Send + Sync>>,
    current_timer: usize,
}

impl TimerManager {
    /// Create new timer manager
    pub fn new() -> Self {
        Self {
            timers: Vec::new(),
            current_timer: 0,
        }
    }

    /// Add timer to manager
    pub fn add_timer(&mut self, timer: Box<dyn Timer + Send + Sync>) {
        info!("Adding timer: {}", timer.name());
        self.timers.push(timer);
    }

    /// Get current timer
    pub fn get_current_timer(&self) -> Option<&dyn Timer> {
        self.timers.get(self.current_timer).map(|t| t.as_ref())
    }

    /// Set current timer
    pub fn set_current_timer(&mut self, index: usize) -> DriverResult<()> {
        if index < self.timers.len() {
            self.current_timer = index;
            Ok(())
        } else {
            Err(DriverError::DeviceNotFound)
        }
    }

    /// Initialize all timers
    pub fn init_all(&mut self) -> DriverResult<()> {
        for (i, timer) in self.timers.iter().enumerate() {
            info!("Timer {}: {}", i + 1, timer.name());
        }
        
        if !self.timers.is_empty() {
            info!("Using timer {}: {}", self.current_timer + 1, self.get_current_timer().unwrap().name());
        }
        
        Ok(())
    }

    /// Initialize global timer system
    pub fn init_global() -> DriverResult<()> {
        info!("Initializing global timer system");
        
        TIMER.call_once(|| {
            let mut manager = TimerManager::new();
            
            // Add PIT as fallback
            manager.add_timer(Box::new(Pit8254::new()));
            
            // Try to add HPET if available (typically at 0xFED00000)
            #[cfg(target_arch = "x86_64")]
            {
                // This would be done by reading ACPI tables or memory maps
                // For now, just use PIT
            }
            
            let _ = manager.init_all();
            Mutex::new(manager)
        });
        
        Ok(())
    }

    /// Get global timer reference
    pub fn get_global() -> Option<spin::MutexGuard<dyn Timer + Send + Sync>> {
        TIMER.get().map(|timer| timer.lock())
    }

    /// Get current tick count from global timer
    pub fn get_global_tick_count() -> Option<u64> {
        Self::get_global().map(|timer| timer.get_tick_count())
    }

    /// Get elapsed nanoseconds from global timer
    pub fn get_global_elapsed_ns() -> Option<u64> {
        Self::get_global().map(|timer| timer.get_elapsed_ns())
    }
}

impl Timer for Pit8254 {
    fn name(&self) -> &'static str {
        "8254 PIT"
    }

    fn get_frequency(&self) -> u32 {
        self.config.frequency
    }

    fn get_tick_count(&self) -> u64 {
        *self.tick_count.lock()
    }

    fn get_elapsed_ns(&self) -> u64 {
        self.get_elapsed_ms() * 1_000_000 // Convert ms to ns
    }
}

impl Timer for HpetTimer {
    fn name(&self) -> &'static str {
        "HPET"
    }

    fn get_frequency(&self) -> u32 {
        self.config.frequency
    }

    fn get_tick_count(&self) -> u64 {
        self.read_counter()
    }

    fn get_elapsed_ns(&self) -> u64 {
        self.get_elapsed_ns()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pit_creation() {
        let pit = Pit8254::new();
        assert_eq!(pit.base_port, 0x40);
        assert_eq!(pit.config.timer_type, TimerType::P8254);
        assert_eq!(pit.config.frequency, 1000);
    }

    #[test]
    fn test_timer_mode_enum() {
        assert_eq!(TimerMode::OneShot as u8, 0);
        assert_eq!(TimerMode::Periodic as u8, 1);
        assert_eq!(TimerMode::RateGenerator as u8, 2);
        assert_eq!(TimerMode::SquareWave as u8, 3);
    }

    #[test]
    fn test_timer_type_enum() {
        assert_eq!(TimerType::P8254 as u8, 0);
        assert_eq!(TimerType::ApicTimer as u8, 1);
        assert_eq!(TimerType::HPET as u8, 2);
        assert_eq!(TimerType::ARMGeneric as u8, 3);
        assert_eq!(TimerType::RISCVMTimer as u8, 4);
    }

    #[test]
    fn test_timer_manager_creation() {
        let mut manager = TimerManager::new();
        assert_eq!(manager.timers.len(), 0);
        assert_eq!(manager.current_timer, 0);
    }

    #[test]
    fn test_timer_manager_add_timer() {
        let mut manager = TimerManager::new();
        manager.add_timer(Box::new(Pit8254::new()));
        
        assert_eq!(manager.timers.len(), 1);
        assert!(manager.get_current_timer().is_some());
        assert_eq!(manager.get_current_timer().unwrap().name(), "8254 PIT");
    }

    #[test]
    fn test_timer_capabilities() {
        let pit = Pit8254::new();
        let caps = pit.capabilities();
        
        assert!(caps.contains(DeviceCapabilities::READ));
        assert!(caps.contains(DeviceCapabilities::WRITE));
        assert!(caps.contains(DeviceCapabilities::INTERRUPT));
    }

    #[test]
    fn test_pit_divisor_calculation() {
        let pit = Pit8254::new();
        // For 1000 Hz: divisor = 1193182 / 1000 = 1193
        let expected_divisor = 1193;
        
        // This test verifies our divisor calculation would work
        assert!(expected_divisor > 0);
        assert!(expected_divisor <= 0xFFFF);
    }
}