# Smart Sensor Network - Tutorial Guide

## Introduction

This tutorial will guide you through setting up, configuring, and deploying the Smart Sensor Network project on RISC-V hardware. We'll cover everything from hardware assembly to cloud integration.

## Prerequisites

### Hardware
- RISC-V development board (SiFive HiFive or compatible)
- DHT22 temperature/humidity sensor
- PIR motion sensor
- OLED display (SSD1306 128x64)
- Breadboard and jumper wires
- USB cable for programming
- 3.7V LiPo battery (optional, for wireless operation)

### Software
- Rust toolchain with RISC-V target
- OpenOCD for debugging
- QEMU for testing
- Git for version control

## Tutorial 1: Basic Setup

### Step 1: Install Dependencies
```bash
# Clone the repository
git clone <repository-url>
cd iot_projects

# Install dependencies
cd setup
./install_deps.sh
```

### Step 2: Build the Project
```bash
# Navigate to the smart sensor project
cd ../1_smart_sensor_network

# Build for RISC-V
cargo build --release --target riscv64gc-unknown-none-elf

# Or use the build script
cd ../setup
./build_riscv.sh smart_sensor_network
```

### Step 3: Test in QEMU Emulator
```bash
# Run in QEMU emulator
./emulate.sh smart_sensor_network
```

You should see the system boot and sensor initialization messages.

## Tutorial 2: Hardware Assembly

### Step 1: Connect the DHT22 Sensor

**Connection Diagram:**
```
RISC-V Board    DHT22
VCC (3.3V)  ->  VDD
GND          ->  GND
GPIO0        ->  DATA
```

**Connection Steps:**
1. Insert DHT22 into breadboard
2. Connect VDD pin to 3.3V supply
3. Connect GND pin to board ground
4. Connect DATA pin to GPIO0 (Pin 0)
5. Add a 10kΩ pull-up resistor between VDD and DATA

### Step 2: Connect the PIR Motion Sensor

**Connection Diagram:**
```
RISC-V Board    PIR Sensor
VCC (3.3V)  ->  VCC
GND          ->  GND
GPIO1        ->  OUT
```

**Connection Steps:**
1. Connect PIR sensor to breadboard
2. Connect VCC to 3.3V supply
3. Connect GND to board ground
4. Connect OUT pin to GPIO1 (Pin 1)
5. Adjust sensitivity and delay potentiometers if needed

### Step 3: Connect the OLED Display

**Connection Diagram:**
```
RISC-V Board    OLED Display
VCC (3.3V)  ->  VCC
GND          ->  GND
GPIO3        ->  SDA
GPIO4        ->  SCL
```

**Connection Steps:**
1. Connect OLED display to breadboard
2. Connect VCC to 3.3V supply
3. Connect GND to board ground
4. Connect SDA to GPIO3 (Pin 3)
5. Connect SCL to GPIO4 (Pin 4)
6. Add 4.7kΩ pull-up resistors for SDA and SCL lines

### Step 4: Final Assembly

**Complete Pin Connections:**
```
Component     | GPIO Pin | Notes
--------------|----------|------------------
DHT22 Data    | Pin 0    | 1-wire protocol
PIR Output    | Pin 1    | Digital HIGH/LOW
Battery Mon   | Pin 2    | ADC input (optional)
Display SDA   | Pin 3    | I2C data
Display SCL   | Pin 4    | I2C clock
LoRa MISO     | Pin 5    | SPI data (optional)
LoRa MOSI     | Pin 6    | SPI data (optional)
LoRa CLK      | Pin 7    | SPI clock (optional)
LoRa CS       | Pin 8    | Chip select (optional)
LoRa Reset    | Pin 9    | Reset (optional)
WiFi TX       | Pin 10   | UART (optional)
WiFi RX       | Pin 11   | UART (optional)
```

## Tutorial 3: Software Configuration

### Step 1: Modify Device Information

Edit `src/main.rs` to customize your device:

```rust
let device_info = DeviceInfo {
    device_id: 0x1234_5678,      // Change to your unique ID
    location: String::from("Your Location"), // Your location
    node_type: String::from("Temperature"), // Your node type
    firmware_version: String::from("1.0.0"), // Version
};
```

### Step 2: Configure Thresholds

Adjust sensor thresholds based on your environment:

```rust
let config = SensorConfig {
    temp_threshold: 2500,        // 25.0°C (adjust as needed)
    humidity_threshold: 3000,    // 30.0% (adjust as needed)
    motion_enabled: true,
    sampling_interval: 60,       // 1 minute (60 seconds)
    transmission_interval: 600,  // 10 minutes (600 seconds)
};
```

### Step 3: Enable Communication Protocols

Choose which communication methods to enable in `Cargo.toml`:

```toml
[features]
default = ["mqtt", "lora", "display", "sensors"]
mqtt = ["iot_communication/mqtt"]    # Enable MQTT
lora = ["iot_communication/lora"]    # Enable LoRaWAN
display = []                         # Enable OLED display
sensors = []                         # Enable sensor reading
```

## Tutorial 4: Testing Individual Components

### Test 1: Sensor Testing

**Temperature/Humidity Sensor:**
```rust
// Add this code temporarily for testing
let (temp, humidity) = app.read_dht22().unwrap();
println!("Temperature: {}°C, Humidity: {}%", 
         temp as f32 / 10.0, humidity as f32 / 10.0);
```

**Motion Sensor:**
```rust
// Add this code temporarily for testing
let motion = GPIO_DRIVER.read_input(1);
println!("Motion detected: {}", motion);
```

**Battery Monitor:**
```rust
// Add this code temporarily for testing
let voltage = app.read_battery_voltage();
println!("Battery voltage: {}mV", voltage);
```

### Test 2: Display Testing

Create a test pattern to verify display functionality:

```rust
// Clear display
for page in 0..8 {
    I2C_DRIVER.start();
    I2C_DRIVER.write_byte(0x78);
    I2C_DRIVER.write_byte(0xB0 + page);
    I2C_DRIVER.write_byte(0x00);
    I2C_DRIVER.write_byte(0x10);
    
    // Fill with pattern
    for col in 0..128 {
        I2C_DRIVER.write_byte(if (page + col) % 2 == 0 { 0xFF } else { 0x00 });
    }
    
    I2C_DRIVER.stop();
}
```

### Test 3: Communication Testing

**WiFi Test:**
```rust
// Test WiFi connectivity
wifi.init("YourWiFiName", "YourPassword")?;
println!("WiFi connected successfully");
```

**MQTT Test:**
```rust
// Test MQTT connection
let mut client = MqttClient::new(&wifi_transport, String::from("test_device"));
client.connect("broker.example.com", None, None)?;
println!("MQTT connected successfully");
```

## Tutorial 5: Integration Testing

### Step 1: Complete System Test

1. Build and flash the complete application
2. Monitor serial output for initialization messages
3. Verify all sensors are responding
4. Check display shows sensor readings
5. Test motion detection by waving in front of PIR sensor

### Step 2: Communication Test

1. **MQTT Test:**
   ```bash
   # Subscribe to your topic
   mosquitto_sub -h broker.example.com -t sensors/+/data
   ```

2. **LoRa Test:**
   ```bash
   # Use a LoRa receiver to monitor transmissions
   # Verify packets are received
   ```

### Step 3: Power Consumption Test

1. Connect current meter to power supply
2. Measure idle current (should be < 10mA)
3. Measure active current during transmission (can be 100-200mA)
4. Calculate expected battery life

## Tutorial 6: Cloud Integration

### Step 1: Setup MQTT Broker

**Using Mosquitto:**
```bash
# Install Mosquitto
sudo apt-get install mosquitto mosquitto-clients

# Start the broker
sudo systemctl start mosquitto

# Test connection
mosquitto_pub -h localhost -t "test/topic" -m "Hello IoT"
```

### Step 2: Cloud Platform Integration

**AWS IoT Core:**
```rust
// Configure AWS IoT
let aws_config = AwsConfig {
    endpoint: "your-iot-endpoint-ats.iot.region.amazonaws.com",
    client_id: "smart_sensor_001",
    certificates: aws_certificates,
};
```

**Google Cloud IoT:**
```rust
// Configure Google Cloud IoT
let gcp_config = GcpConfig {
    project_id: "your-project-id",
    registry_id: "sensor_registry",
    device_id: "device_001",
    private_key: gcp_private_key,
};
```

**Azure IoT Hub:**
```rust
// Configure Azure IoT
let azure_config = AzureConfig {
    connection_string: "HostName=your-hub.azure-devices.net;SharedAccessKeyName=device;SharedAccessKey=key",
    device_id: "device_001",
};
```

### Step 3: Dashboard Integration

**Grafana Dashboard:**
```json
{
  "dashboard": {
    "title": "Smart Sensor Network",
    "panels": [
      {
        "title": "Temperature",
        "type": "graph",
        "targets": [
          {
            "expr": "avg(sensor_temperature{device_id=~\"$device\"})",
            "legendFormat": "Temperature (°C)"
          }
        ]
      },
      {
        "title": "Humidity",
        "type": "graph", 
        "targets": [
          {
            "expr": "avg(sensor_humidity{device_id=~\"$device\"})",
            "legendFormat": "Humidity (%)"
          }
        ]
      }
    ]
  }
}
```

## Tutorial 7: Advanced Configuration

### Step 1: Custom Alert Rules

Implement custom alert logic:

```rust
fn check_alerts(&self, reading: &SensorReading) {
    // Temperature alerts
    if reading.temperature > self.config.temp_threshold {
        self.send_alert("Temperature high", AlertLevel::Warning);
    }
    
    // Humidity alerts
    if reading.humidity < self.config.humidity_threshold {
        self.send_alert("Humidity low", AlertLevel::Warning);
    }
    
    // Motion alerts
    if reading.motion {
        self.send_alert("Motion detected", AlertLevel::Info);
    }
    
    // Battery alerts
    if reading.battery_voltage < 3000 {
        self.send_alert("Low battery", AlertLevel::Critical);
    }
}
```

### Step 2: Data Logging

Implement local data storage:

```rust
struct DataLog {
    readings: Vec<SensorReading, 256>, // Store 256 readings
    index: u8,
}

impl DataLog {
    fn log_reading(&mut self, reading: SensorReading) {
        self.readings[self.index as usize] = reading;
        self.index = (self.index + 1) % self.readings.len();
    }
    
    fn export_data(&self) -> Vec<u8, 1024> {
        // Export data in JSON format
        let mut buffer = Vec::new();
        buffer.extend_from_slice(b"[");
        
        for (i, reading) in self.readings.iter().enumerate() {
            if i > 0 {
                buffer.push(b',');
            }
            // Serialize reading to JSON
            let json = format!(r#"{{"temp":{},"humidity":{},"motion":{},"timestamp":{}}}"#,
                             reading.temperature, reading.humidity, 
                             reading.motion, reading.timestamp);
            buffer.extend_from_slice(json.as_bytes());
        }
        
        buffer.extend_from_slice(b"]");
        buffer
    }
}
```

### Step 3: Over-the-Air Updates

Implement OTA update mechanism:

```rust
fn check_for_updates(&mut self) -> Result<(), UpdateError> {
    // Check firmware version on server
    let version = self.get_latest_version()?;
    
    if version > self.get_current_version() {
        // Download update
        let firmware = self.download_firmware(&version)?;
        
        // Verify checksum
        if self.verify_checksum(&firmware) {
            // Apply update
            self.apply_update(&firmware)?;
        }
    }
    
    Ok(())
}
```

## Tutorial 8: Performance Optimization

### Step 1: Power Optimization

**Low Power Mode:**
```rust
fn optimize_power(&mut self) {
    let battery_level = self.read_battery_voltage();
    
    match battery_level {
        level if level < 2800 => {
            // Deep sleep mode
            self.config.sampling_interval = 300; // 5 minutes
            self.config.transmission_interval = 3600; // 1 hour
            PowerMode::Sleep.enter();
        },
        level if level < 3000 => {
            // Power saving mode
            self.config.sampling_interval = 60; // 1 minute
            self.config.transmission_interval = 1800; // 30 minutes
            PowerMode::Idle.enter();
        },
        _ => {
            // Normal mode
            self.config.sampling_interval = 30; // 30 seconds
            self.config.transmission_interval = 600; // 10 minutes
        }
    }
}
```

**Sleep Optimization:**
```rust
fn smart_sleep(&self, seconds: u32) {
    for _ in 0..seconds {
        unsafe { wfi() }
        self.check_interrupts();
    }
}
```

### Step 2: Memory Optimization

**Buffer Management:**
```rust
struct OptimizedBuffer {
    data: [u8; 128],
    length: u8,
    index: u8,
}

impl OptimizedBuffer {
    fn new() -> Self {
        Self {
            data: [0; 128],
            length: 0,
            index: 0,
        }
    }
    
    fn append(&mut self, byte: u8) {
        if self.length < 128 {
            self.data[self.index as usize] = byte;
            self.length += 1;
            self.index = (self.index + 1) % 128;
        }
    }
}
```

### Step 3: Processing Optimization

**Edge Processing:**
```rust
fn analyze_reading(&self, reading: &SensorReading) -> Analysis {
    let mut analysis = Analysis::new();
    
    // Temperature trend analysis
    if let Some(&prev) = self.history.last() {
        let temp_change = reading.temperature - prev.temperature;
        analysis.temp_trend = match temp_change {
            temp if temp.abs() < 5 => Trend::Stable,
            temp if temp > 0 => Trend::Rising,
            _ => Trend::Falling,
        };
    }
    
    // Humidity prediction
    let humidity_rate = self.calculate_humidity_rate();
    if humidity_rate > 10 {
        analysis.humidity_warning = "Rapid increase detected";
    }
    
    analysis
}
```

## Tutorial 9: Troubleshooting

### Common Issues and Solutions

#### Issue 1: Sensors Not Responding
**Symptoms:** No temperature/humidity readings
**Solutions:**
1. Check power connections
2. Verify GPIO pin assignments
3. Add pull-up resistor (10kΩ)
4. Test with oscilloscope
5. Check sensor initialization sequence

#### Issue 2: Display Not Working
**Symptoms:** Blank or garbled display
**Solutions:**
1. Verify I2C address (0x3C)
2. Check SDA/SCL connections
3. Add pull-up resistors (4.7kΩ)
4. Test with I2C scanner
5. Verify initialization sequence

#### Issue 3: Communication Failures
**Symptoms:** No MQTT/LoRa transmission
**Solutions:**
1. Check antenna connections (LoRa)
2. Verify baud rates and configuration
3. Test with known working components
4. Monitor signal quality
5. Check for interference

#### Issue 4: High Power Consumption
**Symptoms:** Battery drains quickly
**Solutions:**
1. Measure actual current draw
2. Optimize sampling intervals
3. Implement proper sleep modes
4. Reduce transmission frequency
5. Check for firmware loops

### Debug Tools

**Logic Analyzer:**
```bash
# Use sigrok/pulseview to analyze digital signals
pulseview &
```

**Serial Monitor:**
```bash
# Monitor serial output
screen /dev/ttyUSB0 115200

# Or use minicom
minicom -b 115200 -D /dev/ttyUSB0
```

**Network Monitor:**
```bash
# Monitor MQTT traffic
sudo tcpdump -i wlan0 port 1883

# Monitor LoRa traffic (requires RTL-SDR)
rtl_fm -f 868000000 -s 2000000 -g 0 -r 80000 | 
urh -f 868000000 -s 2000000 -c 13.5e3
```

## Tutorial 10: Project Extensions

### Extension 1: Multi-Sensor Array
Add more sensors to create a comprehensive environmental monitoring system:
- **Air Quality**: MQ-135, PM2.5, CO2 sensors
- **Light Level**: LDR, photodiode sensors
- **Noise Level**: Microphone, sound level sensor
- **Soil Moisture**: Capacitive soil moisture sensor

### Extension 2: Mesh Network
Implement a mesh network for extended coverage:
```rust
struct MeshNode {
    node_id: u32,
    neighbors: FnvIndexMap<u32, u16, 16>, // Neighbor ID -> Signal strength
    routing_table: FnvIndexMap<u32, u32, 64>, // Destination -> Next hop
}
```

### Extension 3: Edge AI
Add machine learning capabilities for predictive analysis:
```rust
fn predict_temperature(&self, historical_data: &[SensorReading]) -> i16 {
    // Simple linear regression for temperature prediction
    // In practice, use more sophisticated ML models
    
    if historical_data.len() < 2 {
        return 2300; // Default 23.0°C
    }
    
    let slope = self.calculate_trend(historical_data);
    let last_reading = historical_data[historical_data.len() - 1].temperature;
    
    last_reading + slope
}
```

### Extension 4: Mobile App
Create a companion mobile application:
- Real-time monitoring dashboard
- Remote sensor configuration
- Push notifications for alerts
- Historical data visualization
- OTA firmware updates

This tutorial provides a comprehensive guide to setting up and customizing the Smart Sensor Network for your specific needs. Each tutorial builds upon the previous ones, gradually adding complexity and functionality.

## Next Steps

After completing these tutorials, consider:
1. Integrating with your existing IoT infrastructure
2. Customizing for your specific use case
3. Contributing improvements back to the project
4. Sharing your implementation with the community
5. Scaling to production deployments