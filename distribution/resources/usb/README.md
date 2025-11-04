# MultiOS USB Device Driver Framework

A comprehensive USB driver framework for the MultiOS operating system, providing full USB host controller support, device class drivers, and advanced features including security isolation and protocol analysis.

## Features

### 🔧 Core Components
- **USB Host Controller Drivers**: Full support for xHCI (USB 3.0+), EHCI (USB 2.0), and OHCI (USB 1.1)
- **USB Device Class Drivers**: Support for HID, Mass Storage, CDC (Communications), and Audio devices
- **USB Hub Management**: Comprehensive hub detection and port management
- **Hotplug Detection**: Real-time device connection/disconnection handling
- **USB Power Management**: Advanced power state management and charging protocols

### 🔒 Security & Analysis
- **USB Security Isolation**: Device fingerprinting, permission management, and attack detection
- **Educational Protocol Analyzer**: USB traffic capture, descriptor decoding, and educational tutorials
- **Comprehensive Testing Tools**: Unit tests, integration tests, and performance benchmarks

## Quick Start

### Basic Usage

```rust
use usb_framework::prelude::*;

// Initialize the USB framework
let mut framework = UsbFramework::new();

// Initialize all available host controllers
framework.initialize_host_controllers()?;

// Scan for connected devices
let devices = framework.scan_for_devices()?;

println!("Found {} USB devices:", devices.len());
for device in &devices {
    println!("  - {} (VID:{:04X}, PID:{:04X})", 
        device.product_name(), device.vendor_id(), device.product_id());
}
```

### Host Controller Management

```rust
use usb_framework::host::{XhciHost, EhciHost, OhciHost};

// Create host manager
let mut host_manager = UsbHostManager::new();

// Initialize all controllers
host_manager.initialize_all()?;

// Get host information
println!("{}", host_manager.get_host_info());

// Scan for devices
host_manager.scan_for_devices()?;
```

### Device Class Drivers

#### HID (Human Interface Devices)

```rust
use usb_framework::classes::HidDevice;

// Create and initialize HID device
let mut hid_device = HidDevice::new(0x1234, 0x5678, 1);
hid_device.initialize()?;

// Parse keyboard input report
let keyboard_report = vec![0x00, 0x00, 0x04]; // 'a' key
let parsed_input = hid_device.parse_input_report(&keyboard_report)?;
println!("Key pressed: {:?}", parsed_input.key_codes);
```

#### Mass Storage

```rust
use usb_framework::classes::MscDevice;

// Create and initialize MSC device
let mut msc_device = MscDevice::new(0x1234, 0x5678, 1);
msc_device.initialize()?;

// Execute SCSI command
let read_capacity_cmd = vec![
    0x25, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
];
let response = msc_device.execute_scsi_command(&read_capacity_cmd, 8)?;
```

#### Communications (CDC)

```rust
use usb_framework::classes::CdcDevice;
use usb_framework::classes::cdc::{LineBaudRate, DataBits, StopBits, Parity};

// Create CDC device
let mut cdc_device = CdcDevice::new(0x1234, 0x5678, 1, 2);
cdc_device.initialize()?;

// Configure serial parameters
let line_coding = cdc_device.create_line_coding(
    LineBaudRate::B115200,
    DataBits::Eight,
    StopBits::One,
    Parity::None,
);

// Send data
let data = b"Hello USB!";
let bytes_sent = cdc_device.send_data(data)?;
```

#### Audio

```rust
use usb_framework::classes::AudioDevice;
use usb_framework::classes::audio::{AudioSampleRate, AudioSampleBits, AudioChannels};

// Create audio device
let mut audio_device = AudioDevice::new(0x1234, 0x5678, 1);
audio_device.initialize()?;

// Configure audio format
let format = audio_device.create_audio_format(
    AudioSampleRate::Hz44100,
    AudioSampleBits::Sixteen,
    AudioChannels::Stereo,
);

// Control volume
audio_device.set_master_volume(0.8)?;

// Stream audio data
let audio_data = create_audio_data();
audio_device.stream_audio_data(&audio_data)?;
```

### Security Management

```rust
use usb_framework::security::{SecurityManager, SecurityLevel, DeviceFingerprint};

// Create security manager
let mut security_manager = SecurityManager::new(SecurityLevel::Medium);

// Create device fingerprint
let fingerprint = DeviceFingerprint::new(0x1234, 0x5678, (0x03, 0x01, 0x01));

// Check device access
let trust_state = security_manager.check_device_access(&fingerprint)?;
println!("Device trust state: {:?}", trust_state);

// Trust or block device
security_manager.trust_device(&fingerprint);
// or
security_manager.block_device(&fingerprint);

// Generate security report
println!("{}", security_manager.generate_security_report());
```

### Protocol Analysis

```rust
use usb_framework::protocol_analyzer::{ProtocolAnalyzer, DescriptorDecoder};

// Create analyzer
let mut analyzer = ProtocolAnalyzer::new();
analyzer.set_capture_enabled(true);

// Decode USB descriptor
let descriptor_data = get_usb_descriptor_data();
let decoder = DescriptorDecoder::new();
let decoded_descriptor = decoder.decode_descriptor(&descriptor_data)?;
println!("{}", decoded_descriptor);

// Generate analysis report
let report = analyzer.generate_analysis_report();
println!("{}", report);

// Get educational tutorial
let tutorial = analyzer.generate_tutorial("enumeration");
println!("{}", tutorial);
```

### Hub Management

```rust
use usb_framework::hub::UsbHub;

// Create 4-port hub
let mut hub = UsbHub::new(1, 4); // Hub address 1, 4 ports
hub.initialize()?;

// Monitor port status
for port in 0..4 {
    if hub.port_connected(port)? {
        println!("Device connected on port {}", port + 1);
        
        // Power on port
        hub.set_port_power(port, true)?;
        
        // Get device info
        if let Some(device_info) = hub.get_port_device_info(port) {
            println!("  VID: {:04X}, PID: {:04X}", 
                device_info.vendor_id, device_info.product_id);
        }
    }
}
```

### Hotplug Detection

```rust
use usb_framework::hotplug::HotplugDetector;

// Create hotplug detector
let mut detector = HotplugDetector::new();

// Monitor for device events
loop {
    if let Some(event) = detector.check_for_events()? {
        match event {
            HotplugEvent::DeviceConnected { vendor_id, product_id, device_class } => {
                println!("Device connected: VID:{:04X}, PID:{:04X}", 
                    vendor_id, product_id);
                
                // Start device enumeration
                let device_info = detector.enumerate_device(
                    vendor_id, product_id, device_class)?;
            }
            HotplugEvent::DeviceDisconnected { vendor_id, product_id } => {
                println!("Device disconnected: VID:{:04X}, PID:{:04X}", 
                    vendor_id, product_id);
            }
        }
    }
}
```

### Power Management

```rust
use usb_framework::power::{UsbPowerManager, PowerState};

// Create power manager
let mut power_manager = UsbPowerManager::new();

// Set power state
power_manager.set_power_state(PowerState::Suspended)?;

// Calculate power budget
let budget = power_manager.calculate_power_budget(3, 100)?; // 3 devices, 100mA each
println!("Power budget: {}mA total, {}mA remaining", 
    budget.total_budget, budget.remaining_capacity);

// Configure charging
power_manager.enable_usb_pd_charging(5000, 2000)?; // 5V, 2A max
```

## Testing

### Run Comprehensive Tests

```rust
use usb_framework::tests::run_comprehensive_tests;

// Run full test suite
run_comprehensive_tests()?;
```

### Quick Validation

```rust
use usb_framework::tests::quick_validation_test;

// Quick validation test
let is_valid = quick_validation_test()?;
if is_valid {
    println!("✓ USB framework is working correctly");
}
```

### Performance Benchmarking

```rust
use usb_framework::tests::benchmark_framework;

// Run performance benchmarks
benchmark_framework()?;
```

## Examples

The framework includes comprehensive examples:

- **Host Controller Example** (`examples/host_controller_example.rs`)
  - Multi-controller initialization
  - Device enumeration process
  - USB protocol fundamentals

- **Device Classes Example** (`examples/device_classes_example.rs`)
  - HID keyboard/mouse processing
  - Mass storage SCSI commands
  - Serial communication setup
  - Audio streaming

## Architecture

```
USB Framework
├── Host Controllers
│   ├── xHCI (USB 3.0+)
│   ├── EHCI (USB 2.0)
│   └── OHCI (USB 1.1)
├── Device Classes
│   ├── HID (Human Interface)
│   ├── MSC (Mass Storage)
│   ├── CDC (Communications)
│   └── Audio
├── System Services
│   ├── Hub Management
│   ├── Hotplug Detection
│   └── Power Management
├── Security
│   ├── Device Isolation
│   ├── Access Control
│   └── Audit Logging
└── Analysis Tools
    ├── Protocol Analyzer
    ├── Descriptor Decoder
    └── Educational Tools
```

## Educational Content

The framework includes extensive educational resources:

- **USB Protocol Fundamentals**: Understanding USB communication
- **Device Enumeration Process**: Step-by-step device discovery
- **Transfer Types**: Control, bulk, interrupt, and isochronous transfers
- **Descriptor Structure**: Device capabilities and configuration
- **Security Concepts**: Device isolation and access control

### Generate Tutorials

```rust
let tutorial = analyzer.generate_tutorial("enumeration");
println!("{}", tutorial);

// Available topics:
// - "enumeration" - Device discovery process
// - "transfer_types" - USB data transfer methods
// - "descriptors" - Device capability descriptions
// - "protocol" - Low-level communication details
// - "basic" - Introduction to USB
```

## Configuration

### Security Levels

```rust
pub enum SecurityLevel {
    None,        // No restrictions
    Basic,       // Basic device verification
    Medium,      // Fingerprinting and permission checks
    High,        // Strict isolation with monitoring
    Maximum,     // Only explicitly trusted devices
}
```

### Power Management

```rust
pub enum PowerState {
    Active,
    Suspended,
    PoweredDown,
    SelectiveSuspend,
}
```

## Requirements

- Rust 1.70+ (for no_std support)
- Target architecture with embedded capabilities
- Platform-specific USB host controller access

## Building

```bash
# Build the framework
cargo build

# Run examples
cargo run --example host_controller_example
cargo run --example device_classes_example

# Run tests
cargo test

# Run comprehensive test suite
cargo test --test comprehensive
```

## Contributing

The framework is designed to be:
- **Extensible**: Easy to add new device classes
- **Educational**: Rich documentation and tutorials
- **Secure**: Built-in security and isolation features
- **Testable**: Comprehensive testing framework
- **Performant**: Optimized for embedded systems

## License

This USB framework is part of the MultiOS project and follows the same licensing terms.

## Documentation

- **API Documentation**: Generated via `cargo doc`
- **Educational Resources**: Built-in tutorials and examples
- **Testing Guide**: Comprehensive test suite and benchmarks
- **Security Guide**: Security features and configuration
- **Protocol Analysis**: USB protocol education and analysis tools

---

*Built with ❤️ for the MultiOS operating system*