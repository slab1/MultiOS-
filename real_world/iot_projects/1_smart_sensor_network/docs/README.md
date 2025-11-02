# Smart Sensor Network - Documentation

## Overview

The Smart Sensor Network is an IoT demonstration project designed for RISC-V architectures. It collects environmental data from temperature, humidity, and motion sensors, performs edge processing, and transmits data using multiple communication protocols.

## Hardware Requirements

### Core Hardware
- **RISC-V Development Board**: SiFive HiFive, Kendryte K210, or compatible
- **Minimum Memory**: 256KB RAM, 128KB Flash
- **Operating Frequency**: 50MHz recommended

### Sensors
1. **DHT22 Temperature/Humidity Sensor**
   - Temperature range: -40°C to +80°C
   - Humidity range: 0-100% RH
   - Accuracy: ±0.5°C, ±2-5% RH
   - Communication: 1-wire (GPIO)

2. **PIR Motion Sensor (HC-SR501)**
   - Detection range: 7 meters
   - Detection angle: 120°
   - Output: Digital (HIGH/LOW)
   - Power: 5V, 3.3V compatible

3. **Battery Voltage Monitoring**
   - ADC input (12-bit resolution)
   - Voltage reference: 3.3V
   - Battery type: Li-Ion/Li-Po recommended

### Display
- **OLED Display (SSD1306 128x64)**
  - Interface: I2C
  - Address: 0x3C
  - Size: 0.96" OLED

### Communication Modules
1. **WiFi Module (ESP8266/ESP32)**
   - UART interface
   - Support for MQTT, HTTP
   - 802.11 b/g/n

2. **LoRa Module (SX1278/RA-02)**
   - SPI interface
   - Frequency: 868/915 MHz
   - Range: Several kilometers

## Hardware Connections

### GPIO Pin Assignments
```
GPIO Pin | Component | Function
---------|-----------|----------
0        | DHT22     | Data (1-wire)
1        | PIR       | Motion detection
2        | Battery   | Voltage monitoring (ADC)
3        | Display   | I2C SDA
4        | Display   | I2C SCL
5        | LoRa      | SPI MISO
6        | LoRa      | SPI MOSI
7        | LoRa      | SPI CLK
8        | LoRa      | CS
9        | LoRa      | Reset
10       | WiFi      | UART TX
11       | WiFi      | UART RX
```

### Power Connections
```
Component     | VCC     | GND    | Notes
--------------|---------|--------|----------------------
DHT22         | 3.3V    | GND    | VDD
PIR           | 3.3V    | GND    | VCC
Display       | 3.3V    | GND    | VCC
WiFi Module   | 3.3V    | GND    | VCC
LoRa Module   | 3.3V    | GND    | VCC
```

## Software Architecture

### Component Overview
```
┌─────────────────────────────────────────┐
│         Smart Sensor Network App        │
├─────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────────────┐│
│  │  Sensors    │  │  Communication      ││
│  │  Manager    │  │  Manager            ││
│  └─────────────┘  └─────────────────────┘│
├─────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────────────┐│
│  │  Display    │  │  Power Management   ││
│  │  Manager    │  │                     ││
│  └─────────────┘  └─────────────────────┘│
├─────────────────────────────────────────┤
│           RISC-V HAL                     │
└─────────────────────────────────────────┘
```

### Key Features

#### 1. Multi-Sensor Data Collection
- **Temperature Monitoring**: Real-time temperature with configurable thresholds
- **Humidity Tracking**: Humidity level monitoring and alerting
- **Motion Detection**: PIR sensor for security and automation
- **Battery Monitoring**: Voltage level tracking for battery-powered devices

#### 2. Edge Processing
- **Threshold Detection**: Configurable alert thresholds
- **Data Aggregation**: Statistical analysis of sensor readings
- **Event Detection**: Motion-triggered actions
- **Local Alerts**: Immediate response to critical conditions

#### 3. Multiple Communication Protocols
- **MQTT**: Cloud-based data transmission for IoT platforms
- **LoRaWAN**: Long-range low-power communication
- **WiFi**: Standard wireless connectivity
- **Local Display**: On-device information display

#### 4. Power Management
- **Adaptive Sampling**: Dynamic interval adjustment
- **Low Power Modes**: Sleep and idle states based on battery level
- **Efficient Algorithms**: Optimized for low power consumption
- **Battery Optimization**: Power-saving features for extended operation

## Configuration

### Device Configuration
```rust
let device_info = DeviceInfo {
    device_id: 0x1234_5678,      // Unique device identifier
    location: String::from("Living Room"), // Installation location
    node_type: String::from("Temperature"), // Node classification
    firmware_version: String::from("1.0.0"), // Software version
};
```

### Sensor Configuration
```rust
let config = SensorConfig {
    temp_threshold: 3000,         // 30.0°C alert threshold
    humidity_threshold: 3000,     // 30.0% low humidity alert
    motion_enabled: true,         // Enable motion detection
    sampling_interval: 30,        // Sensor read every 30 seconds
    transmission_interval: 300,   // Transmit every 5 minutes
};
```

### Communication Configuration
```rust
// WiFi Configuration
let wifi_config = WifiConfig {
    ssid: "MyNetwork",
    password: "password123",
    mqtt_broker: "broker.example.com",
    mqtt_port: 1883,
};

// LoRa Configuration
let lora_config = LoraConfig {
    frequency: 868_100_000,      // 868.1 MHz
    spreading_factor: 7,
    transmission_power: 14,      // 14 dBm
};
```

## Data Format

### Sensor Reading Structure
```json
{
  "device_id": 305419896,
  "temperature": 2350,
  "humidity": 6500,
  "motion": true,
  "timestamp": 1640995200,
  "battery": 3780,
  "location": "Living Room",
  "node_type": "Temperature"
}
```

### Field Descriptions
- **device_id**: Unique 32-bit identifier
- **temperature**: Temperature in deci-celsius (2350 = 23.5°C)
- **humidity**: Humidity in deci-percent (6500 = 65.0%)
- **motion**: Motion detection status (boolean)
- **timestamp**: Unix timestamp (seconds since epoch)
- **battery**: Battery voltage in millivolts
- **location**: Device installation location
- **node_type**: Device classification

## Deployment Guide

### Step 1: Hardware Setup
1. Connect all sensors to the development board
2. Verify power connections
3. Test each component individually
4. Check communication interfaces (I2C, SPI, UART)

### Step 2: Software Compilation
```bash
# Build for RISC-V
cargo build --release --target riscv64gc-unknown-none-elf

# Build with specific features
cargo build --release --features "mqtt,lora,display"
```

### Step 3: Flash to Hardware
```bash
# Using OpenOCD
openocd -f interface/stlink.cfg -f target/riscv.cfg \
        -c "program target/release/smart_sensor_network reset exit"

# Using vendor-specific tools
# (depends on hardware platform)
```

### Step 4: Configuration
1. Configure WiFi credentials
2. Set MQTT broker details
3. Adjust sensor thresholds
4. Test communication protocols

### Step 5: Testing
1. Verify sensor readings
2. Test motion detection
3. Confirm data transmission
4. Check display functionality
5. Validate power management

## Testing and Validation

### Unit Tests
```bash
cargo test --release
```

### Integration Tests
```bash
cargo test --release --features "test_hardware"
```

### Field Testing
1. **Sensor Accuracy**: Compare with calibrated instruments
2. **Communication Range**: Test maximum transmission distance
3. **Battery Life**: Monitor power consumption over time
4. **Environmental Testing**: Temperature, humidity extremes
5. **EMI/EMC**: Interference testing in industrial environments

## Troubleshooting

### Common Issues

#### 1. Sensor Not Responding
- Check power connections
- Verify GPIO pin assignments
- Test with oscilloscope/logic analyzer
- Verify sensor initialization sequence

#### 2. Communication Failures
- Check baud rates and configuration
- Verify antenna connections (LoRa)
- Test with known working modules
- Monitor signal quality

#### 3. Display Issues
- Verify I2C address (0x3C)
- Check wiring connections
- Test with simple I2C scanner
- Verify initialization sequence

#### 4. Power Problems
- Measure actual current draw
- Check battery voltage under load
- Verify power sequencing
- Test with different power supplies

### Debug Tools
- **Logic Analyzer**: Digital signal analysis
- **Oscilloscope**: Analog signal monitoring
- **Multimeter**: Voltage and current measurement
- **Network Analyzer**: RF signal analysis (LoRa)
- **Current Probe**: Power consumption profiling

## Performance Metrics

### Expected Performance
- **Sensor Sampling**: 30-second intervals (configurable)
- **Battery Life**: 6+ months with 2000mAh LiPo
- **LoRa Range**: 2-5 km (line of sight)
- **WiFi Range**: 50-100 meters (typical home)
- **Temperature Accuracy**: ±0.5°C
- **Humidity Accuracy**: ±2-5% RH

### Optimization Tips
1. **Sampling Rate**: Adjust based on application needs
2. **Transmission Frequency**: Balance data freshness vs. power
3. **Data Compression**: Reduce transmission payload size
4. **Deep Sleep**: Maximize power efficiency
5. **Edge Processing**: Process data locally when possible

## Maintenance

### Regular Tasks
1. **Battery Monitoring**: Check voltage levels monthly
2. **Sensor Calibration**: Verify accuracy quarterly
3. **Software Updates**: Apply security patches
4. **Physical Inspection**: Check for damage/weather exposure
5. **Communication Testing**: Verify connectivity monthly

### Replacement Schedule
- **PIR Sensor**: 2-3 years (typical lifetime)
- **Battery**: 1-2 years (depending on usage)
- **Display**: 5-7 years (OLED lifetime)
- **Enclosure**: 5+ years (with proper maintenance)

## Future Enhancements

### Planned Features
1. **Additional Sensors**: Air quality, noise, light level
2. **Machine Learning**: Predictive maintenance, anomaly detection
3. **Mesh Networking**: Multi-hop communication
4. **Edge AI**: On-device neural networks
5. **Remote Configuration**: Over-the-air updates

### Integration Possibilities
1. **Smart Home**: Home Assistant, OpenHAB
2. **Cloud Platforms**: AWS IoT, Azure IoT, Google Cloud IoT
3. **Mobile Apps**: iOS/Android companion apps
4. **Dashboards**: Grafana, Node-RED, custom interfaces
5. **Automation**: IFTTT, Zapier integration

This smart sensor network provides a solid foundation for IoT environmental monitoring with room for expansion and customization based on specific application requirements.