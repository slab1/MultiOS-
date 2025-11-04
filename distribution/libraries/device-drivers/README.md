# MultiOS Device Driver Framework

A comprehensive device driver framework for the MultiOS hybrid microkernel architecture, providing unified device abstraction, automatic driver binding, and plug-and-play device detection across multiple architectures.

## Features

### Core Framework
- **Device Abstraction Layer**: Unified interface for all hardware devices
- **Driver Manager**: Central registry for device drivers with priority-based binding
- **Hardware Bus Support**: PCI, USB, I2C, SPI, and platform bus interfaces
- **Plug-and-Play**: Automatic device detection and driver binding
- **Safe Interfaces**: Memory-safe device operations with proper error handling

### Supported Architectures
- x86_64
- ARM64 (AArch64)
- RISC-V 64-bit

### Built-in Device Drivers
- **Serial Console**: 16550 UART support
- **Timer System**: 8254 PIT, HPET, and high-resolution timers
- **Keyboard Input**: PS/2 and USB keyboard support
- **PCI Bus**: Automatic device enumeration
- **USB Host**: Device detection and enumeration

## Usage

### Basic Initialization

```rust
use multios_device_drivers::{init, init_console, init_system_timer, init_keyboard};

fn main() -> Result<(), DriverError> {
    // Initialize the device driver framework
    init()?;
    
    // Initialize console output
    let mut console = init_console()?;
    console.println("Hello from MultiOS!")?;
    
    // Initialize system timer
    let timer_manager = init_system_timer()?;
    
    // Initialize keyboard input
    init_keyboard()?;
    
    Ok(())
}
```

### Device Discovery and Management

```rust
use multios_device_drivers::{discover_all_devices, find_devices, DeviceType};

// Discover all available devices
let devices = discover_all_devices()?;
println!("Found {} devices", devices.len());

// Find specific device types
let keyboards = find_devices(DeviceType::Keyboard)?;
let serial_devices = find_devices(DeviceType::UART)?;
```

### Serial Console Operations

```rust
use multios_device_drivers::serial::{SerialConsole, UartConfig, Parity};

// Create custom UART configuration
let config = UartConfig {
    baud_rate: 9600,
    data_bits: 8,
    stop_bits: 1,
    parity: Parity::None,
};

// Initialize console
let mut console = SerialConsole::with_config(0x3F8, config);
console.init()?;

// Print formatted output
console.println("System booted successfully")?;
console.print("Boot time: ")?;
console.println(&format!("{} ms", boot_time))?;

// Read user input
let input = console.read_line()?;
println!("User input: {}", input);
```

### Timer Operations

```rust
use multios_device_drivers::timer::TimerManager;

// Initialize timer system
TimerManager::init_global()?;

// Get elapsed time
if let Some(timer) = TimerManager::get_global() {
    let tick_count = timer.get_tick_count();
    let elapsed_ns = timer.get_elapsed_ns();
    
    println!("Timer: {} Hz, {} ticks, {} ns", 
             timer.get_frequency(), tick_count, elapsed_ns);
}
```

### Keyboard Input

```rust
use multios_device_drivers::keyboard::{get_global_key_event, KeyCode};

// Check for keyboard events
if keyboard::global_keyboard_has_events() {
    if let Some(event) = get_global_key_event() {
        println!("Key pressed: {:?}", event.key_code);
        
        // Convert key code to character
        if let Some(character) = keyboard::Ps2Keyboard::key_code_to_char(
            event.key_code, event.modifiers) {
            println!("Character: {}", character);
        }
    }
}
```

## Architecture

### Device Abstraction

The framework provides a unified `Device` structure that abstracts hardware specifics:

```rust
pub struct Device {
    pub info: DeviceInfo,           // Device metadata
    pub hardware_addr: HardwareAddress, // Hardware address
    pub driver: Option<&'static dyn DeviceDriver>, // Associated driver
}
```

### Driver Interface

All drivers implement the `DeviceDriver` trait:

```rust
pub trait DeviceDriver: Send + Sync {
    fn name(&self) -> &'static str;
    fn supported_devices(&self) -> &[DeviceType];
    fn init(&self, device: &Device) -> DriverResult<()>;
    fn read(&self, device: &Device, buffer: &mut [u8]) -> DriverResult<usize>;
    fn write(&self, device: &Device, buffer: &[u8]) -> DriverResult<usize>;
    fn ioctl(&self, device: &Device, command: u32, data: usize) -> DriverResult<usize>;
    fn capabilities(&self) -> DeviceCapabilities;
}
```

### Hardware Bus Support

The framework includes bus drivers for different hardware interfaces:

- **PCI Bus**: Automatic device enumeration, configuration space access
- **USB Bus**: Device detection, endpoint configuration
- **I2C Bus**: Slave device detection, address scanning
- **SPI Bus**: Chip select management, data transfer
- **Platform Bus**: Memory-mapped device discovery

### Device Hierarchy

```
Driver Manager
├── PCI Bus
│   ├── Host Bridge (0:0:0)
│   ├── SATA Controller (0:1f:2)
│   └── USB Controller (0:1a:0)
├── USB Bus 1
│   ├── Keyboard (port 2)
│   └── Mouse (port 3)
└── Platform Bus
    ├── Timer (0x2c000000)
    └── UART (0x3f8)
```

## Building

Add the framework to your project:

```toml
[dependencies]
multios-device-drivers = { path = "path/to/device-drivers" }
```

Build with specific features:

```bash
# Basic functionality
cargo build

# With PCI support
cargo build --features pci

# With all features
cargo build --all-features

# Run examples
cargo run --example demo
```

## Error Handling

The framework uses `DriverResult<T>` for safe error handling:

```rust
pub type DriverResult<T> = Result<T, DriverError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriverError {
    DeviceNotFound,
    DriverNotSupported,
    InitializationFailed,
    DeviceBusy,
    PermissionDenied,
    HardwareError,
}
```

## Testing

The framework includes comprehensive unit tests:

```bash
# Run all tests
cargo test

# Run integration tests
cargo test --test integration

# Run benchmarks
cargo bench

# Test on specific architectures
cargo test --target x86_64
cargo test --target aarch64
cargo test --target riscv64gc
```

## Integration with MultiOS

The framework is designed to integrate seamlessly with the MultiOS kernel:

1. **Boot-time initialization**: Devices detected during boot are automatically registered
2. **Interrupt handling**: Framework provides hooks for interrupt-driven devices
3. **Memory management**: Safe memory allocation for device structures
4. **Power management**: Support for device power states and hot-plug events

## Future Enhancements

Planned features include:

- Network device drivers
- Graphics display support
- Audio device drivers
- Storage device drivers (block, filesystem)
- Advanced USB device support
- Real-time clock (RTC) support
- Watchdog timer support
- GPIO device support
- PWM device support

## License

MIT OR Apache-2.0

## Contributing

Contributions are welcome! Please see the main MultiOS project for contribution guidelines.