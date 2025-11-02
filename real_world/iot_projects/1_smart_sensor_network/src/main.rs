//! Smart Sensor Network - Main Application
//! 
//! This application demonstrates a RISC-V IoT sensor network with temperature,
//! humidity, and motion sensors. It collects data from multiple sensors,
//! performs edge processing, and transmits data via MQTT and LoRaWAN.
//!
//! Hardware Requirements:
//! - RISC-V development board (SiFive HiFive, Kendryte K210, etc.)
//! - DHT22 temperature/humidity sensor
//! - PIR motion sensor
//! - I2C OLED display
//! - Optional: LoRa module for long-range communication

#![no_std]
#![no_main]

use core::fmt::Write;
use heapless::{String, Vec, FnvIndexMap};

use riscv_hal::*;
use iot_communication::*;

// Sensor readings
#[derive(Clone, Copy, Debug)]
struct SensorReading {
    temperature: i16,  // deci-celsius (e.g., 235 = 23.5°C)
    humidity: u16,     // deci-percent (e.g., 650 = 65.0%)
    motion: bool,
    timestamp: u32,    // Unix timestamp
    battery_voltage: u16, // milli-volts
}

// Sensor configuration
#[derive(Clone, Copy)]
struct SensorConfig {
    temp_threshold: i16,
    humidity_threshold: u16,
    motion_enabled: bool,
    sampling_interval: u32, // seconds
    transmission_interval: u32, // seconds
}

// Device information
#[derive(Clone, Copy)]
struct DeviceInfo {
    device_id: u32,
    location: String<32>,
    node_type: String<16>,
    firmware_version: String<16>,
}

// Application state
struct SensorNetworkApp {
    device_info: DeviceInfo,
    config: SensorConfig,
    comm_manager: CommunicationManager,
    current_reading: Option<SensorReading>,
    reading_count: u32,
    last_transmission: u32,
    motion_detected: bool,
    display_buffer: Vec<u8, 128>,
}

impl SensorNetworkApp {
    pub fn new(device_info: DeviceInfo, config: SensorConfig) -> Self {
        Self {
            device_info,
            config,
            comm_manager: CommunicationManager::new(),
            current_reading: None,
            reading_count: 0,
            last_transmission: 0,
            motion_detected: false,
            display_buffer: Vec::new(),
        }
    }

    /// Initialize the sensor network application
    pub fn init(&mut self) -> Result<(), SensorNetworkError> {
        // Initialize hardware
        self.init_hardware()?;
        
        // Initialize sensors
        self.init_sensors()?;
        
        // Initialize communication
        self.init_communication()?;
        
        // Initialize display
        self.init_display()?;
        
        // Setup interrupts
        self.setup_interrupts()?;
        
        println!("\n🌡️  Smart Sensor Network Initialized");
        println!("📡 Device ID: {}", self.device_info.device_id);
        println!("📍 Location: {}", self.device_info.location);
        println!("⏱️  Sampling interval: {} seconds", self.config.sampling_interval);
        
        Ok(())
    }

    fn init_hardware(&self) -> Result<(), SensorNetworkError> {
        let config = SystemConfig {
            core_frequency_hz: 50_000_000, // 50 MHz
            memory_size: 256 * 1024,       // 256KB
            interrupt_controller: InterruptType::PLIC,
            power_management: PowerMode::Normal,
        };
        
        init_system(config);
        
        // Configure GPIO pins
        let gpio_config = GpioConfig {
            pin_number: 0, // Temperature sensor
            mode: GpioMode::Input,
            pull_type: PullType::Up,
            drive_strength: DriveStrength::Low,
        };
        
        GPIO_DRIVER.configure(gpio_config);
        
        println!("✅ Hardware initialized");
        Ok(())
    }

    fn init_sensors(&mut self) -> Result<(), SensorNetworkError> {
        // Initialize DHT22 temperature/humidity sensor
        self.init_dht22_sensor()?;
        
        // Initialize PIR motion sensor
        self.init_pir_sensor()?;
        
        // Initialize battery voltage monitoring
        self.init_battery_monitor()?;
        
        println!("✅ Sensors initialized");
        Ok(())
    }

    fn init_dht22_sensor(&self) -> Result<(), SensorNetworkError> {
        // DHT22 uses 1-wire protocol via GPIO
        // This implementation uses a simplified protocol
        
        let config = GpioConfig {
            pin_number: 0, // DHT22 data pin
            mode: GpioMode::Output,
            pull_type: PullType::Up,
            drive_strength: DriveStrength::Medium,
        };
        
        GPIO_DRIVER.configure(config);
        
        // Send start signal
        GPIO_DRIVER.set_output(0, false);
        delay_ms(18); // 18ms low pulse
        
        GPIO_DRIVER.set_output(0, true);
        delay_ms(1); // 1ms high pulse
        
        // Switch to input mode to read response
        let input_config = GpioConfig {
            pin_number: 0,
            mode: GpioMode::Input,
            pull_type: PullType::Up,
            drive_strength: DriveStrength::Low,
        };
        
        GPIO_DRIVER.configure(input_config);
        
        Ok(())
    }

    fn init_pir_sensor(&self) -> Result<(), SensorNetworkError> {
        let config = GpioConfig {
            pin_number: 1, // PIR motion sensor
            mode: GpioMode::Input,
            pull_type: PullType::Up,
            drive_strength: DriveStrength::Low,
        };
        
        GPIO_DRIVER.configure(config);
        
        Ok(())
    }

    fn init_battery_monitor(&self) -> Result<(), SensorNetworkError> {
        // Initialize ADC for battery voltage monitoring
        let config = GpioConfig {
            pin_number: 2, // Battery voltage input
            mode: GpioMode::Analog,
            pull_type: PullType::None,
            drive_strength: DriveStrength::Low,
        };
        
        GPIO_DRIVER.configure(config);
        
        Ok(())
    }

    fn init_communication(&mut self) -> Result<(), SensorNetworkError> {
        // Initialize WiFi for MQTT communication
        #[cfg(feature = "wifi")]
        {
            let uart = UART_DRIVER.borrow();
            let static_uart = unsafe {
                // SAFETY: This is safe as UART_DRIVER is static and lives for program lifetime
                &*(uart as *const Uart)
            };
            
            self.comm_manager.init_wifi(static_uart, "MyWiFi", "password123")?;
        }
        
        // Initialize LoRaWAN for long-range communication
        #[cfg(feature = "lora")]
        {
            self.comm_manager.init_lora(SPI_DRIVER)?;
        }
        
        println!("✅ Communication initialized");
        Ok(())
    }

    fn init_display(&mut self) -> Result<(), SensorNetworkError> {
        // Initialize I2C OLED display (SSD1306)
        let display_config = GpioConfig {
            pin_number: 3, // SDA
            mode: GpioMode::AlternateFunction,
            pull_type: PullType::Up,
            drive_strength: DriveStrength::Medium,
        };
        
        let sda_config = GpioConfig {
            pin_number: 4, // SCL
            mode: GpioMode::AlternateFunction,
            pull_type: PullType::Up,
            drive_strength: DriveStrength::Medium,
        };
        
        GPIO_DRIVER.configure(display_config);
        GPIO_DRIVER.configure(sda_config);
        
        // Initialize display with I2C
        self.init_oled_display()?;
        
        // Show initial message
        self.update_display()?;
        
        Ok(())
    }

    fn init_oled_display(&self) -> Result<(), SensorNetworkError> {
        // I2C address 0x3C for SSD1306
        I2C_DRIVER.start();
        I2C_DRIVER.write_byte(0x78); // I2C address with write flag
        
        // Send initialization sequence
        let init_commands = [
            0xAE, // Display off
            0x20, // Memory addressing mode
            0x10, // Horizontal addressing
            0xB0, // Set page address
            0xC8, // Set COM scan direction
            0x00, // Set lower column address
            0x10, // Set higher column address
            0x40, // Set start line address
            0x81, // Set contrast
            0xFF, // Maximum contrast
            0xA1, // Set segment remap
            0xA6, // Normal display mode
            0xA8, // Set multiplex ratio
            0x3F, // 64 duty
            0xA4, // Output RAM to display
            0xD3, // Set display offset
            0x00, // No offset
            0xD5, // Set display clock
            0xF0, // Set divide ratio
            0xD9, // Set pre-charge period
            0xF1, // Pre-charge = 15 clocks
            0xDA, // Set com pins hardware configuration
            0x12, // Alternative COM pin configuration
            0xDB, // Set VCOMH deselect level
            0x20, // ~0.77 V
            0x8D, // Charge pump setting
            0x14, // Enable charge pump
            0xAF, // Display on
        ];
        
        for &cmd in &init_commands {
            I2C_DRIVER.start();
            I2C_DRIVER.write_byte(0x78); // Device address
            I2C_DRIVER.write_byte(cmd);   // Command byte
            I2C_DRIVER.write_byte(0x00); // Data byte
            I2C_DRIVER.stop();
            delay_ms(1);
        }
        
        Ok(())
    }

    fn setup_interrupts(&self) -> Result<(), SensorNetworkError> {
        // Setup timer interrupt for periodic sampling
        set_timer_interrupt();
        
        // Setup GPIO interrupt for motion sensor
        unsafe {
            // Configure GPIO interrupt enable
            let mut ie = core::ptr::read_volatile(0x1001_2004 as *const u32);
            ie |= 0x02; // Enable interrupt for pin 1
            core::ptr::write_volatile(0x1001_2004 as *mut u32, ie);
        }
        
        Ok(())
    }

    /// Main application loop
    pub fn run(&mut self) -> ! {
        loop {
            // Sample sensors
            self.sample_sensors();
            
            // Process samples
            self.process_samples();
            
            // Update display
            self.update_display();
            
            // Check if transmission is needed
            if self.should_transmit() {
                self.transmit_data();
            }
            
            // Process incoming messages
            self.comm_manager.process_messages();
            
            // Check for motion detection
            self.check_motion();
            
            // Power management
            self.manage_power();
            
            // Small delay to prevent excessive CPU usage
            delay_ms(100);
        }
    }

    fn sample_sensors(&mut self) {
        let current_time = get_time().0; // Use seconds from RTC
        
        if current_time % self.config.sampling_interval == 0 {
            // Read temperature and humidity
            if let Ok(reading) = self.read_dht22() {
                self.current_reading = Some(SensorReading {
                    temperature: reading.0,
                    humidity: reading.1,
                    motion: self.motion_detected,
                    timestamp: current_time,
                    battery_voltage: self.read_battery_voltage(),
                });
                
                self.reading_count += 1;
                self.motion_detected = false; // Reset motion flag
            }
        }
    }

    fn read_dht22(&self) -> Result<(i16, u16), SensorNetworkError> {
        // Simplified DHT22 read implementation
        // In a real implementation, this would handle the 1-wire protocol
        
        // For demo purposes, generate realistic sensor values
        let temperature = 2300 + (self.reading_count % 500) as i16; // 23.0-28.0°C
        let humidity = 4500 + (self.reading_count % 2000) as u16;   // 45.0-65.0%
        
        Ok((temperature, humidity))
    }

    fn read_battery_voltage(&self) -> u16 {
        // Read battery voltage using ADC
        let reading = ADC_DRIVER.read_channel(0); // Channel 0 for battery
        let voltage_mv = (reading as u32 * 3300) / 4095; // 3.3V reference, 12-bit ADC
        voltage_mv as u16
    }

    fn check_motion(&mut self) {
        // Check PIR sensor
        self.motion_detected = GPIO_DRIVER.read_input(1);
        
        if self.motion_detected {
            println!("🚨 Motion detected!");
            
            // Trigger immediate transmission
            self.transmit_data();
        }
    }

    fn process_samples(&self) {
        if let Some(reading) = self.current_reading {
            // Check for threshold violations
            if reading.temperature > self.config.temp_threshold {
                println!("⚠️  Temperature threshold exceeded: {}°C", 
                        reading.temperature as f32 / 10.0);
            }
            
            if reading.humidity < self.config.humidity_threshold {
                println!("⚠️  Low humidity detected: {}%", 
                        reading.humidity as f32 / 10.0);
            }
            
            if reading.battery_voltage < 3000 {
                println!("🔋 Low battery: {}mV", reading.battery_voltage);
            }
        }
    }

    fn should_transmit(&self) -> bool {
        if self.current_reading.is_none() {
            return false;
        }
        
        let current_time = get_time().0;
        current_time - self.last_transmission >= self.config.transmission_interval
    }

    fn transmit_data(&mut self) {
        if let Some(reading) = self.current_reading {
            // Format data for transmission
            let mut data_string = String::<128>::new();
            write!(&mut data_string, 
                   "{{\"device_id\":{},\"temp\":{},\"humidity\":{},\"motion\":{},\"timestamp\":{},\"battery\":{}}}", 
                   self.device_info.device_id,
                   reading.temperature,
                   reading.humidity,
                   reading.motion,
                   reading.timestamp,
                   reading.battery_voltage).unwrap();
            
            // Transmit via MQTT
            #[cfg(feature = "mqtt")]
            {
                let topic = format!("sensors/{}/data", self.device_info.device_id);
                if let Ok(_) = self.comm_manager.send_message(
                    data_string.as_bytes(), 
                    CommunicationProtocol::MQTT
                ) {
                    println!("📡 Data transmitted via MQTT");
                }
            }
            
            // Transmit via LoRaWAN
            #[cfg(feature = "lora")]
            {
                if let Ok(_) = self.comm_manager.send_message(
                    data_string.as_bytes(),
                    CommunicationProtocol::LoRa
                ) {
                    println!("📡 Data transmitted via LoRa");
                }
            }
            
            self.last_transmission = get_time().0;
        }
    }

    fn update_display(&mut self) -> Result<(), SensorNetworkError> {
        // Clear display buffer
        self.display_buffer.clear();
        
        // Add current readings to buffer
        if let Some(reading) = self.current_reading {
            write!(&mut self.display_buffer, 
                   "Temp: {}°C\nHum: {}%\nMotion: {}\nBattery: {}mV\n\nTransmit: {}s", 
                   reading.temperature as f32 / 10.0,
                   reading.humidity as f32 / 10.0,
                   if reading.motion { "YES" } else { "NO" },
                   reading.battery_voltage,
                   self.config.transmission_interval).unwrap();
            
            // Send buffer to display
            self.send_to_display(&self.display_buffer)?;
        } else {
            write!(&mut self.display_buffer, 
                   "Smart Sensor\nNetwork v1.0\n\nDevice: {}\nStatus: Sampling",
                   self.device_info.device_id).unwrap();
        }
        
        Ok(())
    }

    fn send_to_display(&self, data: &[u8]) -> Result<(), SensorNetworkError> {
        // Send data to OLED display via I2C
        let lines: Vec<&[u8], 8> = data.split(|&b| b == b'\n').collect();
        
        for (i, line) in lines.iter().enumerate() {
            if line.is_empty() {
                continue;
            }
            
            // Set cursor position
            I2C_DRIVER.start();
            I2C_DRIVER.write_byte(0x78); // Device address
            
            // Set page address and column
            let page_addr = 0xB0 + i as u8;
            I2C_DRIVER.write_byte(page_addr);
            I2C_DRIVER.write_byte(0x00); // Lower column
            I2C_DRIVER.write_byte(0x10); // Higher column
            
            // Send character data (simplified - would need font table)
            for &byte in line {
                // Convert ASCII to display bitmap (simplified)
                let display_byte = match byte {
                    b'T' | b't' => 0x7E, // Simplified 'T'
                    b'E' | b'e' => 0x7D, // Simplified 'E'
                    b'M' | b'm' => 0x6D, // Simplified 'M'
                    b'P' | b'p' => 0x75, // Simplified 'P'
                    _ => 0x00, // Space or unknown
                };
                
                I2C_DRIVER.write_byte(display_byte);
            }
            
            I2C_DRIVER.stop();
        }
        
        Ok(())
    }

    fn manage_power(&self) {
        // Implement power management based on battery level and activity
        let battery_mv = self.read_battery_voltage();
        
        if battery_mv < 2800 {
            // Low power mode
            let config = SystemConfig {
                core_frequency_hz: 10_000_000, // 10 MHz
                memory_size: 256 * 1024,
                interrupt_controller: InterruptType::PLIC,
                power_management: PowerMode::Sleep,
            };
            init_system(config);
            
            println!("🔋 Entering low power mode");
        }
    }
}

// Error types
#[derive(Debug)]
pub enum SensorNetworkError {
    HardwareInitFailed,
    SensorInitFailed,
    CommunicationError,
    DisplayError,
    InvalidData,
}

// Global application instance
static mut SENSOR_APP: Option<SensorNetworkApp> = None;

// RISC-V entry point
#[no_mangle]
pub extern "C" fn main() -> ! {
    // Initialize system
    let config = SystemConfig::default();
    init_system(config);
    
    // Device information
    let device_info = DeviceInfo {
        device_id: 0x1234_5678,
        location: String::from("Living Room"),
        node_type: String::from("Temperature"),
        firmware_version: String::from("1.0.0"),
    };
    
    // Sensor configuration
    let config = SensorConfig {
        temp_threshold: 3000, // 30.0°C
        humidity_threshold: 3000, // 30.0%
        motion_enabled: true,
        sampling_interval: 30, // 30 seconds
        transmission_interval: 300, // 5 minutes
    };
    
    // Create application
    let app = SensorNetworkApp::new(device_info, config);
    
    // Initialize application
    if let Ok(_) = app.init() {
        app.run();
    } else {
        println!("❌ Failed to initialize sensor network");
        loop {}
    }
}

// Interrupt service routine
#[no_mangle]
pub extern "C" fn timer_interrupt() {
    // Update real-time clock
    RTC.tick();
}

// GPIO interrupt service routine
#[no_mangle]
pub extern "C" fn gpio_interrupt() {
    // Handle motion sensor interrupt
    unsafe {
        SENSOR_APP.as_mut().unwrap().motion_detected = true;
    }
}