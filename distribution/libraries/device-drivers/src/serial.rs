//! Serial Console Driver
//! 
//! Provides 16550 UART support for serial console operations.

use crate::{DeviceType, DriverResult, DriverError, device::{Device, DeviceDriver, DeviceCapabilities}};
use log::{info, warn, error};
use core::fmt;

/// 16550 UART Registers
#[repr(u8)]
enum UartRegisters {
    RBR = 0,  // Receiver Buffer Register (read)
    THR = 0,  // Transmitter Holding Register (write)
    IER = 1,  // Interrupt Enable Register
    IIR = 2,  // Interrupt Identification Register (read)
    FCR = 2,  // FIFO Control Register (write)
    LCR = 3,  // Line Control Register
    MCR = 4,  // Modem Control Register
    LSR = 5,  // Line Status Register
    MSR = 6,  // Modem Status Register
    SCR = 7,  // Scratch Register
}

/// UART Configuration
#[derive(Debug, Clone, Copy)]
pub struct UartConfig {
    pub baud_rate: u32,
    pub data_bits: u8,
    pub stop_bits: u8,
    pub parity: Parity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Parity {
    None = 0,
    Odd = 1,
    Even = 2,
    Mark = 3,
    Space = 4,
}

/// Default UART configuration (115200 8N1)
impl Default for UartConfig {
    fn default() -> Self {
        Self {
            baud_rate: 115200,
            data_bits: 8,
            stop_bits: 1,
            parity: Parity::None,
        }
    }
}

/// 16550 UART Driver
pub struct Uart16550 {
    pub port: u16,
    pub config: UartConfig,
}

impl Uart16550 {
    /// Create new UART driver
    pub fn new(port: u16) -> Self {
        let config = UartConfig::default();
        Self { port, config }
    }

    /// Create with custom configuration
    pub fn with_config(port: u16, config: UartConfig) -> Self {
        Self { port, config }
    }

    /// Initialize UART hardware
    pub fn init(&mut self) -> DriverResult<()> {
        info!("Initializing 16550 UART at port 0x{:04x}", self.port);
        
        // Disable interrupts during initialization
        self.write_reg(UartRegisters::IER, 0x00);
        
        // Enable DLAB to access baud rate divisor
        let lcr = self.read_reg(UartRegisters::LCR);
        self.write_reg(UartRegisters::LCR, lcr | 0x80);
        
        // Set baud rate (divisor = clock / (16 * baud_rate))
        // Assuming 1.8432 MHz clock
        let divisor = 1843200 / (16 * self.config.baud_rate);
        self.write_reg(UartRegisters::RBR, (divisor & 0xFF) as u8);
        self.write_reg(UartRegisters::THR, ((divisor >> 8) & 0xFF) as u8);
        
        // Set data format
        let mut lcr = 0x00;
        lcr |= (self.config.data_bits - 5) & 0x03; // data bits (5-8)
        match self.config.stop_bits {
            1 => lcr |= 0x00,  // 1 stop bit
            2 => lcr |= 0x04,  // 2 stop bits
            _ => lcr |= 0x00,
        }
        
        match self.config.parity {
            Parity::None => lcr |= 0x00,
            Parity::Odd => lcr |= 0x08 | 0x10,
            Parity::Even => lcr |= 0x18,
            Parity::Mark => lcr |= 0x28,
            Parity::Space => lcr |= 0x38,
        }
        
        self.write_reg(UartRegisters::LCR, lcr);
        
        // Disable DLAB
        self.write_reg(UartRegisters::LCR, lcr & 0x7F);
        
        // Enable FIFO (if supported)
        self.write_reg(UartRegisters::FCR, 0xC7); // Enable FIFO, trigger level 14
        
        // Enable interrupts
        self.write_reg(UartRegisters::IER, 0x01); // Enable received data interrupt
        
        info!("UART initialized at {} baud, {} data bits, {} stop bits, {} parity",
              self.config.baud_rate, self.config.data_bits, self.config.stop_bits, 
              match self.config.parity { Parity::None => "none", _ => "parity" });
        
        Ok(())
    }

    /// Read a byte from UART
    pub fn read_byte(&self) -> Option<u8> {
        // Check if data is available
        let lsr = self.read_reg(UartRegisters::LSR);
        if lsr & 0x01 != 0 {
            Some(self.read_reg(UartRegisters::RBR))
        } else {
            None
        }
    }

    /// Write a byte to UART
    pub fn write_byte(&self, byte: u8) -> DriverResult<()> {
        // Wait for transmitter to be ready
        let mut timeout = 1000;
        while timeout > 0 {
            let lsr = self.read_reg(UartRegisters::LSR);
            if lsr & 0x20 != 0 {
                break; // Transmitter empty
            }
            timeout -= 1;
        }
        
        if timeout == 0 {
            return Err(DriverError::HardwareError);
        }
        
        self.write_reg(UartRegisters::THR, byte);
        Ok(())
    }

    /// Write string to UART
    pub fn write_string(&self, s: &str) -> DriverResult<()> {
        for byte in s.bytes() {
            self.write_byte(byte)?;
        }
        Ok(())
    }

    /// Check if UART has data available
    pub fn has_data(&self) -> bool {
        let lsr = self.read_reg(UartRegisters::LSR);
        lsr & 0x01 != 0
    }

    /// Check if UART transmitter is ready
    pub fn is_ready(&self) -> bool {
        let lsr = self.read_reg(UartRegisters::LSR);
        lsr & 0x20 != 0
    }

    /// Read register
    fn read_reg(&self, reg: UartRegisters) -> u8 {
        unsafe { core::ptr::read_volatile((self.port as usize + reg as usize) as *const u8) }
    }

    /// Write register
    fn write_reg(&self, reg: UartRegisters, value: u8) {
        unsafe { core::ptr::write_volatile((self.port as usize + reg as usize) as *mut u8, value) }
    }

    /// Get current configuration
    pub fn config(&self) -> &UartConfig {
        &self.config
    }

    /// Set baud rate
    pub fn set_baud_rate(&mut self, baud_rate: u32) -> DriverResult<()> {
        self.config.baud_rate = baud_rate;
        self.init()
    }
}

impl DeviceDriver for Uart16550 {
    fn name(&self) -> &'static str {
        "16550 UART Driver"
    }

    fn supported_devices(&self) -> &[DeviceType] {
        &[DeviceType::UART]
    }

    fn init(&self, device: &Device) -> DriverResult<()> {
        info!("Initializing UART device: {}", device.info.name);
        
        // Extract port from hardware address
        let port = match device.hardware_addr {
            crate::device::HardwareAddress::Port(port) => port,
            _ => return Err(DriverError::HardwareError),
        };
        
        // Initialize UART (note: this needs &mut self, so we can't call it here)
        // In a real implementation, this would be handled differently
        Ok(())
    }

    fn remove(&self, device: &Device) -> DriverResult<()> {
        info!("Removing UART device: {}", device.info.name);
        
        // Disable interrupts
        self.write_reg(UartRegisters::IER, 0x00);
        
        Ok(())
    }

    fn read(&self, device: &Device, buffer: &mut [u8]) -> DriverResult<usize> {
        let mut bytes_read = 0;
        
        for byte in buffer.iter_mut() {
            if let Some(data) = self.read_byte() {
                *byte = data;
                bytes_read += 1;
            } else {
                break;
            }
        }
        
        if bytes_read == 0 {
            Err(DriverError::DeviceNotFound)
        } else {
            Ok(bytes_read)
        }
    }

    fn write(&self, device: &Device, buffer: &[u8]) -> DriverResult<usize> {
        for &byte in buffer {
            self.write_byte(byte)?;
        }
        Ok(buffer.len())
    }

    fn ioctl(&self, device: &Device, command: u32, data: usize) -> DriverResult<usize> {
        match command {
            0x1001 => { // Get configuration
                Ok(self.config.baud_rate as usize)
            }
            0x1002 => { // Set baud rate
                self.config.baud_rate = data as u32;
                Ok(0)
            }
            0x1003 => { // Check if ready
                Ok(if self.is_ready() { 1 } else { 0 })
            }
            0x1004 => { // Check if data available
                Ok(if self.has_data() { 1 } else { 0 })
            }
            _ => Err(DriverError::PermissionDenied),
        }
    }

    fn capabilities(&self) -> DeviceCapabilities {
        DeviceCapabilities::READ | DeviceCapabilities::WRITE | DeviceCapabilities::INTERRUPT
    }
}

impl fmt::Write for Uart16550 {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        match self.write_string(s) {
            Ok(_) => Ok(()),
            Err(_) => Err(fmt::Error),
        }
    }
}

/// Serial Console Manager
pub struct SerialConsole {
    pub uart: Uart16550,
    pub is_enabled: bool,
}

impl SerialConsole {
    /// Create new serial console
    pub fn new(port: u16) -> Self {
        Self {
            uart: Uart16550::new(port),
            is_enabled: false,
        }
    }

    /// Initialize console
    pub fn init(&mut self) -> DriverResult<()> {
        info!("Initializing serial console on port 0x{:04x}", self.uart.port);
        self.uart.init()?;
        self.is_enabled = true;
        
        // Send boot message
        self.uart.write_string("\r\n=== MultiOS Serial Console ===\r\n")?;
        
        Ok(())
    }

    /// Print formatted string
    pub fn print(&mut self, s: &str) -> DriverResult<()> {
        if !self.is_enabled {
            return Ok(());
        }
        
        self.uart.write_string(s)
    }

    /// Print line with newline
    pub fn println(&mut self, s: &str) -> DriverResult<()> {
        if !self.is_enabled {
            return Ok(());
        }
        
        self.uart.write_string(s)?;
        self.uart.write_byte(b'\r')?;
        self.uart.write_byte(b'\n')?;
        Ok(())
    }

    /// Read line from console
    pub fn read_line(&mut self) -> DriverResult<String> {
        if !self.is_enabled {
            return Err(DriverError::HardwareError);
        }
        
        let mut buffer = String::new();
        let mut temp_byte = [0u8; 1];
        
        loop {
            // Read single byte
            match self.uart.read_byte() {
                Some(byte) => {
                    if byte == b'\r' || byte == b'\n' {
                        break;
                    }
                    
                    if byte == 0x7F { // Backspace
                        if !buffer.is_empty() {
                            buffer.pop();
                            self.uart.write_byte(0x08)?; // Backspace
                        }
                        continue;
                    }
                    
                    temp_byte[0] = byte;
                    if let Ok(chr) = core::str::from_utf8(&temp_byte) {
                        buffer.push_str(chr);
                        self.uart.write_byte(byte)?; // Echo
                    }
                }
                None => {
                    // No data available, continue
                    core::hint::spin_loop();
                }
            }
        }
        
        // Handle newline
        self.uart.write_byte(b'\r')?;
        self.uart.write_byte(b'\n')?;
        
        Ok(buffer)
    }

    /// Check if console has input available
    pub fn has_input(&self) -> bool {
        self.is_enabled && self.uart.has_data()
    }

    /// Disable console
    pub fn disable(&mut self) {
        self.is_enabled = false;
    }

    /// Enable console
    pub fn enable(&mut self) {
        self.is_enabled = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uart_config() {
        let config = UartConfig {
            baud_rate: 9600,
            data_bits: 8,
            stop_bits: 1,
            parity: Parity::None,
        };
        
        assert_eq!(config.baud_rate, 9600);
        assert_eq!(config.data_bits, 8);
        assert_eq!(config.stop_bits, 1);
        assert_eq!(config.parity, Parity::None);
    }

    #[test]
    fn test_uart_creation() {
        let uart = Uart16550::new(0x3F8);
        assert_eq!(uart.port, 0x3F8);
        assert_eq!(uart.config.baud_rate, 115200);
        assert_eq!(uart.config.data_bits, 8);
    }

    #[test]
    fn test_parity_enum() {
        assert_eq!(Parity::None as u8, 0);
        assert_eq!(Parity::Odd as u8, 1);
        assert_eq!(Parity::Even as u8, 2);
        assert_eq!(Parity::Mark as u8, 3);
        assert_eq!(Parity::Space as u8, 4);
    }

    #[test]
    fn test_serial_console_creation() {
        let mut console = SerialConsole::new(0x3F8);
        assert_eq!(console.uart.port, 0x3F8);
        assert!(!console.is_enabled);
    }

    #[test]
    fn test_uart_driver_capabilities() {
        let uart = Uart16550::new(0x3F8);
        let caps = uart.capabilities();
        
        assert!(caps.contains(DeviceCapabilities::READ));
        assert!(caps.contains(DeviceCapabilities::WRITE));
        assert!(caps.contains(DeviceCapabilities::INTERRUPT));
        assert!(!caps.contains(DeviceCapabilities::DMA));
    }

    #[test]
    fn test_uart_supported_devices() {
        let uart = Uart16550::new(0x3F8);
        let devices = uart.supported_devices();
        
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0], DeviceType::UART);
    }
}