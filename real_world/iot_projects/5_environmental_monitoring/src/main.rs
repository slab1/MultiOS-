//! Environmental Monitoring System
//! 
//! Comprehensive environmental monitoring with air quality and noise detection
//! for RISC-V architectures. Monitors air pollution, weather conditions,
//! noise pollution, and provides real-time environmental data analysis.

#![no_std]
#![main]

use core::fmt::Write;
use heapless::{String, Vec, FnvIndexMap};

use riscv_hal::*;
use iot_communication::*;

// Air quality data
#[derive(Clone, Copy, Debug)]
struct AirQualityData {
    pm25: u16,           // PM2.5 µg/m³
    pm10: u16,           // PM10 µg/m³
    co2: u16,            // CO2 ppm
    co: u16,             // CO ppm
    no2: u16,            // NO2 µg/m³
    o3: u16,             // O3 µg/m³
    so2: u16,            // SO2 µg/m³
    voc_level: u16,      // VOCs ppm
    aqi: u8,             // Air Quality Index (0-500)
    timestamp: u32,
}

// Noise pollution data
#[derive(Clone, Copy, Debug)]
struct NoiseData {
    sound_level: u8,     // dB
    frequency_spectrum: [u8; 16], // Frequency analysis
    peak_level: u8,      // Peak dB
    average_level: u8,   // Average dB
    noise_events: u8,    // Number of events per hour
    noise_type: NoiseType,
    timestamp: u32,
}

#[derive(Clone, Copy, Debug)]
enum NoiseType {
    Traffic = 1,
    Industrial = 2,
    Construction = 3,
    Aircraft = 4,
    Emergency = 5,
    Residential = 6,
    Commercial = 7,
    Other = 8,
}

// Weather station data
#[derive(Clone, Copy, Debug)]
struct WeatherData {
    temperature: i16,     // deci-celsius
    humidity: u16,        // deci-percent
    pressure: u32,        // hPa * 100
    wind_speed: u16,      // m/s * 10
    wind_direction: u16,  // degrees
    rainfall: u16,        // mm
    uv_index: u8,         // 0-11
    solar_radiation: u32, // W/m²
    visibility: u16,      // km * 10
    timestamp: u32,
}

// Environmental monitoring station
struct EnvironmentalStation {
    station_id: String<16>,
    location: String<32>,
    coordinates: (f32, f32), // latitude, longitude
    air_quality_data: Vec<AirQualityData, 168>, // 1 week of hourly data
    noise_data: Vec<NoiseData, 24>,             // 24 hours of data
    weather_data: Vec<WeatherData, 24>,         // 24 hours of data
    communication_manager: CommunicationManager,
    alert_system: AlertSystem,
}

struct AlertSystem {
    alerts: Vec<EnvironmentalAlert, 50>,
    threshold_config: AlertThresholds,
}

#[derive(Clone, Copy, Debug)]
struct AlertThresholds {
    pm25_warning: u16,    // µg/m³
    pm25_critical: u16,
    noise_warning: u8,    // dB
    noise_critical: u8,
    temperature_warning: i16,
    temperature_critical: i16,
}

#[derive(Clone, Copy, Debug)]
struct EnvironmentalAlert {
    alert_id: u32,
    severity: AlertSeverity,
    category: EnvironmentalCategory,
    message: String<128>,
    threshold_value: f32,
    current_value: f32,
    timestamp: u32,
}

#[derive(Clone, Copy, Debug)]
enum AlertSeverity {
    Info = 1,
    Warning = 2,
    Critical = 3,
    Emergency = 4,
}

#[derive(Clone, Copy, Debug)]
enum EnvironmentalCategory {
    AirQuality = 1,
    NoisePollution = 2,
    Weather = 3,
    Climate = 4,
}

impl EnvironmentalStation {
    pub fn new(station_id: String<16>, location: String<32>) -> Self {
        Self {
            station_id,
            location,
            coordinates: (0.0, 0.0),
            air_quality_data: Vec::new(),
            noise_data: Vec::new(),
            weather_data: Vec::new(),
            communication_manager: CommunicationManager::new(),
            alert_system: AlertSystem {
                alerts: Vec::new(),
                threshold_config: AlertThresholds {
                    pm25_warning: 25,    // WHO guideline
                    pm25_critical: 50,
                    noise_warning: 65,   // WHO recommendation
                    noise_critical: 75,
                    temperature_warning: 3500, // 35°C
                    temperature_critical: 4000, // 40°C
                },
            },
        }
    }

    pub fn init(&mut self) -> Result<(), EnvironmentalMonitoringError> {
        // Initialize sensors
        self.init_air_quality_sensors()?;
        self.init_noise_sensors()?;
        self.init_weather_sensors()?;
        
        // Initialize communication
        self.init_communication()?;
        
        println!("🌍 Environmental Monitoring Station Initialized");
        println!("📍 Location: {}", self.location);
        println!("🆔 Station ID: {}", self.station_id);
        
        Ok(())
    }

    fn init_air_quality_sensors(&self) -> Result<(), EnvironmentalMonitoringError> {
        // Initialize PM sensors, gas sensors, etc.
        println!("  - Air quality sensors initialized");
        Ok(())
    }

    fn init_noise_sensors(&self) -> Result<(), EnvironmentalMonitoringError> {
        // Initialize microphones, audio processing
        println!("  - Noise monitoring sensors initialized");
        Ok(())
    }

    fn init_weather_sensors(&self) -> Result<(), EnvironmentalMonitoringError> {
        // Initialize weather station sensors
        println!("  - Weather station sensors initialized");
        Ok(())
    }

    fn init_communication(&mut self) -> Result<(), EnvironmentalMonitoringError> {
        // Initialize cloud reporting
        #[cfg(feature = "wifi")]
        {
            println!("  - WiFi communication enabled");
        }
        
        #[cfg(feature = "lora")]
        {
            println!("  - LoRaWAN communication enabled");
        }
        
        Ok(())
    }

    pub fn run(&mut self) -> ! {
        let mut sample_counter = 0u32;
        
        loop {
            // Sample air quality (every 10 minutes)
            if sample_counter % 600 == 0 { // 10 minutes
                self.sample_air_quality();
            }
            
            // Sample noise level (every 1 minute)
            if sample_counter % 60 == 0 { // 1 minute
                self.sample_noise_level();
            }
            
            // Sample weather data (every 5 minutes)
            if sample_counter % 300 == 0 { // 5 minutes
                self.sample_weather_data();
            }
            
            // Process data and generate alerts (every 5 minutes)
            if sample_counter % 300 == 0 {
                self.process_environmental_data();
            }
            
            // Transmit data (every 30 minutes)
            if sample_counter % 1800 == 0 { // 30 minutes
                self.transmit_environmental_data();
            }
            
            sample_counter = sample_counter.wrapping_add(1);
            if sample_counter == 0 {
                sample_counter = 1;
            }
            
            delay_ms(1000); // 1 second
        }
    }

    fn sample_air_quality(&mut self) {
        // Read air quality sensors (simplified)
        let air_data = AirQualityData {
            pm25: 15 + (sample_counter % 50) as u16,   // 15-65 µg/m³
            pm10: 25 + (sample_counter % 75) as u16,   // 25-100 µg/m³
            co2: 400 + (sample_counter % 200) as u16,  // 400-600 ppm
            co: 1 + (sample_counter % 5) as u16,       // 1-6 ppm
            no2: 20 + (sample_counter % 40) as u16,    // 20-60 µg/m³
            o3: 50 + (sample_counter % 100) as u16,    // 50-150 µg/m³
            so2: 5 + (sample_counter % 25) as u16,     // 5-30 µg/m³
            voc_level: 300 + (sample_counter % 500) as u16, // 300-800 ppb
            aqi: 50 + (sample_counter % 100) as u8,    // 50-150
            timestamp: get_time().0,
        };
        
        self.air_quality_data.push(air_data).unwrap_or(());
        
        // Keep only last 168 readings (1 week of hourly data)
        if self.air_quality_data.len() > 168 {
            self.air_quality_data.remove(0);
        }
        
        println!("🌫️  Air Quality: PM2.5={}µg/m³, CO2={}ppm, AQI={}", 
                air_data.pm25, air_data.co2, air_data.aqi);
    }

    fn sample_noise_level(&mut self) {
        // Sample noise level
        let noise_data = NoiseData {
            sound_level: 45 + (sample_counter % 40) as u8, // 45-85 dB
            frequency_spectrum: [0; 16],
            peak_level: 60 + (sample_counter % 30) as u8, // 60-90 dB
            average_level: 50 + (sample_counter % 25) as u8, // 50-75 dB
            noise_events: (sample_counter % 10) as u8,
            noise_type: NoiseType::Traffic,
            timestamp: get_time().0,
        };
        
        self.noise_data.push(noise_data).unwrap_or(());
        
        // Keep only last 24 readings (24 hours)
        if self.noise_data.len() > 24 {
            self.noise_data.remove(0);
        }
        
        println!("🔊 Noise Level: {}dB, Events: {}", noise_data.sound_level, noise_data.noise_events);
    }

    fn sample_weather_data(&mut self) {
        // Sample weather sensors
        let weather_data = WeatherData {
            temperature: 2000 + (sample_counter % 1500) as i16, // 20-35°C
            humidity: 5000 + (sample_counter % 3000) as u16,    // 50-80%
            pressure: 101300 + (sample_counter % 2000) as u32,  // 1013-1015 hPa
            wind_speed: 10 + (sample_counter % 20) as u16,      // 1-3 m/s
            wind_direction: (sample_counter % 360) as u16,
            rainfall: if sample_counter % 720 == 0 { 5 } else { 0 }, // Occasional rain
            uv_index: (sample_counter % 11) as u8,
            solar_radiation: 400 + (sample_counter % 600) as u32, // 400-1000 W/m²
            visibility: 80 + (sample_counter % 20) as u16,      // 8-10 km
            timestamp: get_time().0,
        };
        
        self.weather_data.push(weather_data).unwrap_or(());
        
        // Keep only last 24 readings
        if self.weather_data.len() > 24 {
            self.weather_data.remove(0);
        }
        
        println!("🌤️  Weather: {}°C, {}% humidity, {}hPa, UV:{}", 
                weather_data.temperature as f32 / 10.0,
                weather_data.humidity as f32 / 10.0,
                weather_data.pressure,
                weather_data.uv_index);
    }

    fn process_environmental_data(&mut self) {
        // Analyze data and generate alerts
        if let Some(&air_data) = self.air_quality_data.last() {
            self.check_air_quality_alerts(&air_data);
        }
        
        if let Some(&noise_data) = self.noise_data.last() {
            self.check_noise_alerts(&noise_data);
        }
        
        if let Some(&weather_data) = self.weather_data.last() {
            self.check_weather_alerts(&weather_data);
        }
    }

    fn check_air_quality_alerts(&mut self, air_data: &AirQualityData) {
        // PM2.5 alerts
        if air_data.pm25 > self.alert_system.threshold_config.pm25_warning {
            let severity = if air_data.pm25 > self.alert_system.threshold_config.pm25_critical {
                AlertSeverity::Critical
            } else {
                AlertSeverity::Warning
            };
            
            let alert = EnvironmentalAlert {
                alert_id: sample_counter,
                severity,
                category: EnvironmentalCategory::AirQuality,
                message: String::from(&format!("PM2.5 level high: {} µg/m³", air_data.pm25)),
                threshold_value: self.alert_system.threshold_config.pm25_warning as f32,
                current_value: air_data.pm25 as f32,
                timestamp: get_time().0,
            };
            
            self.alert_system.alerts.push(alert).unwrap_or(());
            
            println!("⚠️  Air Quality Alert: PM2.5={}µg/m³", air_data.pm25);
        }
        
        // AQI alerts
        if air_data.aqi > 150 {
            let alert = EnvironmentalAlert {
                alert_id: sample_counter,
                severity: AlertSeverity::Warning,
                category: EnvironmentalCategory::AirQuality,
                message: String::from(&format!("Air Quality Index unhealthy: {}", air_data.aqi)),
                threshold_value: 150.0,
                current_value: air_data.aqi as f32,
                timestamp: get_time().0,
            };
            
            self.alert_system.alerts.push(alert).unwrap_or(());
            
            println!("🌫️  AQI Alert: Unhealthy for sensitive groups (AQI={})", air_data.aqi);
        }
    }

    fn check_noise_alerts(&mut self, noise_data: &NoiseData) {
        if noise_data.sound_level > self.alert_system.threshold_config.noise_warning {
            let severity = if noise_data.sound_level > self.alert_system.threshold_config.noise_critical {
                AlertSeverity::Critical
            } else {
                AlertSeverity::Warning
            };
            
            let alert = EnvironmentalAlert {
                alert_id: sample_counter,
                severity,
                category: EnvironmentalCategory::NoisePollution,
                message: String::from(&format!("Noise level high: {}dB", noise_data.sound_level)),
                threshold_value: self.alert_system.threshold_config.noise_warning as f32,
                current_value: noise_data.sound_level as f32,
                timestamp: get_time().0,
            };
            
            self.alert_system.alerts.push(alert).unwrap_or(());
            
            println!("🔊 Noise Alert: {}dB exceeds threshold", noise_data.sound_level);
        }
    }

    fn check_weather_alerts(&mut self, weather_data: &WeatherData) {
        // Temperature alerts
        if weather_data.temperature > self.alert_system.threshold_config.temperature_warning {
            let severity = if weather_data.temperature > self.alert_system.threshold_config.temperature_critical {
                AlertSeverity::Critical
            } else {
                AlertSeverity::Warning
            };
            
            let alert = EnvironmentalAlert {
                alert_id: sample_counter,
                severity,
                category: EnvironmentalCategory::Weather,
                message: String::from(&format!("High temperature: {}°C", weather_data.temperature as f32 / 10.0)),
                threshold_value: self.alert_system.threshold_config.temperature_warning as f32,
                current_value: weather_data.temperature as f32,
                timestamp: get_time().0,
            };
            
            self.alert_system.alerts.push(alert).unwrap_or(());
            
            println!("🌡️  Temperature Alert: {}°C", weather_data.temperature as f32 / 10.0);
        }
        
        // UV alerts
        if weather_data.uv_index >= 8 {
            let alert = EnvironmentalAlert {
                alert_id: sample_counter,
                severity: AlertSeverity::Warning,
                category: EnvironmentalCategory::Weather,
                message: String::from(&format!("High UV index: {}", weather_data.uv_index)),
                threshold_value: 8.0,
                current_value: weather_data.uv_index as f32,
                timestamp: get_time().0,
            };
            
            self.alert_system.alerts.push(alert).unwrap_or(());
            
            println!("☀️  UV Alert: Very High (UV={})", weather_data.uv_index);
        }
    }

    fn transmit_environmental_data(&mut self) {
        // Prepare comprehensive environmental report
        let mut report = String::<512>::new();
        
        write!(&mut report, 
               "{{\"station_id\":\"{}\",\"location\":\"{}\",\"timestamp\":{},\"data\":{{",
               self.station_id, self.location, get_time().0).unwrap();
        
        // Add latest air quality data
        if let Some(&air_data) = self.air_quality_data.last() {
            write!(&mut report, 
                   "\"air_quality\":{{\"pm25\":{},\"pm10\":{},\"co2\":{},\"aqi\":{}}},",
                   air_data.pm25, air_data.pm10, air_data.co2, air_data.aqi).unwrap();
        }
        
        // Add latest noise data
        if let Some(&noise_data) = self.noise_data.last() {
            write!(&mut report, 
                   "\"noise\":{{\"level\":{},\"events\":{}}},",
                   noise_data.sound_level, noise_data.noise_events).unwrap();
        }
        
        // Add latest weather data
        if let Some(&weather_data) = self.weather_data.last() {
            write!(&mut report, 
                   "\"weather\":{{\"temp\":{},\"humidity\":{},\"pressure\":{}}}",
                   weather_data.temperature, weather_data.humidity, weather_data.pressure).unwrap();
        }
        
        report.push('}');
        report.push('}');
        
        // Transmit via available protocols
        #[cfg(feature = "mqtt")]
        {
            let topic = format!("environmental/{}/data", self.station_id);
            if let Ok(_) = self.communication_manager.send_message(
                report.as_bytes(),
                CommunicationProtocol::MQTT
            ) {
                println!("📡 Environmental data transmitted");
            }
        }
        
        #[cfg(feature = "lora")]
        {
            if let Ok(_) = self.communication_manager.send_message(
                report.as_bytes(),
                CommunicationProtocol::LoRa
            ) {
                println!("📡 Environmental data transmitted via LoRa");
            }
        }
    }
}

#[derive(Debug)]
pub enum EnvironmentalMonitoringError {
    SensorInitFailed,
    CommunicationError,
    DataProcessingError,
}

// RISC-V entry point
#[no_mangle]
pub extern "C" fn main() -> ! {
    // Initialize system
    let config = SystemConfig {
        core_frequency_hz: 50_000_000,
        memory_size: 256 * 1024,
        interrupt_controller: InterruptType::PLIC,
        power_management: PowerMode::Normal,
    };
    init_system(config);
    
    // Create environmental monitoring station
    let station = EnvironmentalStation::new(
        String::from("ENV-001"),
        String::from("Central Park, New York")
    );
    
    if let Ok(_) = station.init() {
        station.run();
    } else {
        println!("❌ Failed to initialize environmental monitoring station");
        loop {}
    }
}