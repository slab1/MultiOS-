# MultiOS Audio Subsystem

A comprehensive, educational audio subsystem implementation for MultiOS, featuring hardware abstraction, multiple codec support, real-time processing, and extensive educational examples.

## 🎵 Overview

The MultiOS Audio Subsystem provides a complete audio infrastructure designed for educational purposes and real-world multimedia applications. It includes support for various audio codecs, hardware abstraction layers, real-time processing capabilities, and comprehensive debugging tools.

## 🏗️ Architecture

### Core Components

1. **Core Audio Framework** (`core/`)
   - Audio manager and stream management
   - Format support and buffer management
   - Device abstraction and configuration

2. **Hardware Abstraction Layer** (`hal/`)
   - PCI audio device support (AC'97, HD Audio)
   - USB audio device support
   - I2S codec support for embedded systems
   - Device detection and enumeration

3. **Codec Support** (`codecs/`)
   - AC'97 audio codec implementation
   - HD Audio (Intel HDA) codec support
   - I2S codec for embedded applications
   - Generic codec interface for extensibility

4. **Audio Mixing Engine** (`mixing/`)
   - Multi-stream audio mixing
   - Real-time effects processing
   - Volume control and routing
   - Hardware/software mixing support

5. **Real-time Processing** (`processing/`)
   - DSP utilities and algorithms
   - FFT spectrum analysis
   - Audio synthesizer with ADSR envelopes
   - Effects processing (reverb, distortion, compression)

6. **Hotplug Support** (`hotplug/`)
   - Dynamic device detection
   - USB audio device support
   - Bluetooth audio device handling
   - Event-driven device management

7. **Educational Examples** (`examples/`)
   - Frequency generation tutorials
   - Audio recording examples
   - Mixing and effects demonstrations
   - Spectrum analysis tutorials
   - Synthesizer programming examples

8. **Debugging Tools** (`debug/`)
   - Real-time performance monitoring
   - Audio visualization tools
   - System health reporting
   - Comprehensive debugging utilities

## 🚀 Features

### Audio Formats Supported
- PCM: 8-bit, 16-bit, 24-bit, 32-bit (signed/unsigned)
- Floating-point: 32-bit, 64-bit (little/big endian)
- Sample rates: 8kHz to 192kHz
- Channel configurations: Mono, Stereo, Surround (up to 8 channels)

### Hardware Support
- **PCI Audio**: AC'97, HD Audio (Intel HDA)
- **USB Audio**: USB Audio Class 1.0 and 2.0
- **Embedded**: I2S audio codecs
- **Future**: Bluetooth A2DP/HFP, HDMI audio

### Educational Features
- **Interactive Tutorials**: Step-by-step audio programming lessons
- **Visualization Tools**: Waveform, spectrum, and level meters
- **Real-time Examples**: Live audio processing demonstrations
- **Comprehensive Documentation**: In-code explanations and guides

### Performance Features
- **Low Latency**: Configurable buffer sizes down to 64 samples
- **High Quality**: Support for studio-quality audio (192kHz/32-bit float)
- **Hardware Acceleration**: Direct hardware mixing when available
- **Real-time Monitoring**: Performance metrics and health reporting

## 📦 Installation

### Prerequisites
- Rust toolchain (1.70+)
- MultiOS kernel environment
- Audio hardware (optional for simulation)

### Building

```bash
# Clone the repository
git clone <repository-url>
cd multios/hardware_support/audio

# Build the audio subsystem
cargo build --release

# Run tests
cargo test

# Run the demo
cargo run --example demo
```

### Integration with MultiOS

1. Add to your `Cargo.toml`:
```toml
[dependencies]
hardware_support = { path = "hardware_support/audio" }
```

2. Initialize in your kernel:
```rust
use hardware_support::audio::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize audio subsystem
    initialize_audio_system()?;
    
    // Run your audio application
    run_audio_application()?;
    
    Ok(())
}
```

## 🎓 Educational Examples

### 1. Frequency Generation
Learn to generate different waveforms and understand audio fundamentals.

```rust
use hardware_support::audio::examples::*;

let mut tutorial = AudioTutorial::new();
tutorial.initialize()?;

// Generate test tones
tutorial.run_example(0)?; // Frequency Generation
```

### 2. Audio Recording
Understand audio input and capture techniques.

```rust
tutorial.run_example(1)?; // Audio Recording
```

### 3. Audio Mixing
Master multi-stream audio mixing and effects.

```rust
tutorial.run_example(3)?; // Audio Mixing
```

### 4. Effects Processing
Learn audio effects including reverb, distortion, and compression.

```rust
tutorial.run_example(4)?; // Effects Processing
```

### 5. Spectrum Analysis
Understand frequency domain analysis using FFT.

```rust
tutorial.run_example(5)?; // Spectrum Analysis
```

### 6. Audio Synthesis
Create musical sounds using digital synthesis.

```rust
tutorial.run_example(6)?; // Audio Synthesizer
```

## 🔧 Configuration Presets

### CD Quality (Standard Audio)
```rust
let config = presets::cd_quality();
// Format: PCM 16-bit, 44.1kHz, Stereo, 1KB buffer
```

### Studio Quality (Professional Audio)
```rust
let config = presets::studio_quality();
// Format: Float 32-bit, 48kHz, Stereo, 512-sample buffer
```

### High Resolution (Audiophile)
```rust
let config = presets::high_resolution();
// Format: Float 32-bit, 96kHz, Stereo, 256-sample buffer
```

### Ultra High Resolution (Mastering)
```rust
let config = presets::ultra_high_resolution();
// Format: Float 32-bit, 192kHz, Stereo, 128-sample buffer
```

### Low Latency (Real-time)
```rust
let config = presets::low_latency();
// Format: Float 32-bit, 48kHz, Stereo, 64-sample buffer
```

### Voice Recording
```rust
let config = presets::voice_recording();
// Format: PCM 16-bit, 16kHz, Mono, 512-sample buffer
```

### Surround Sound
```rust
let config = presets::surround_sound();
// Format: PCM 24-bit, 48kHz, 6-channel, 1KB buffer
```

## 🧪 Testing

### Run Test Suite
```bash
cargo test
```

### Run Performance Tests
```bash
cargo test performance
```

### Run Educational Examples
```bash
cargo run --example demo
```

### Interactive Demo
```bash
cargo run --example demo -- interactive
```

## 📊 Performance Monitoring

### Real-time Statistics
```rust
let mut monitor = AudioSystemMonitor::new();
monitor.start_monitoring();

// Get performance data
let report = monitor.get_performance_report();
println!("CPU Usage: {:.1}%", report.performance_data.cpu_utilization);
println!("Buffer Underruns: {}", report.performance_data.buffer_underruns);
println!("Latency: {:.2} ms", report.performance_data.latency_ms);
```

### Health Monitoring
```rust
let health = monitor.generate_health_report();
println!("System Health: {}/100", health.overall_health);

for recommendation in health.recommendations {
    println!("💡 {}", recommendation);
}
```

## 🔍 Debugging Tools

### Function Tracing
```rust
let mut debugger = AudioDebugger::new();
debugger.enable_tracing();

// Automatic function tracing
debugger.trace_function_entry("audio_processing");
// ... audio processing code ...
debugger.trace_function_exit("audio_processing", duration_ns);
```

### Performance Profiling
```rust
let mut profiler = AudioProfiler::new();

// Profile function execution
let start_time = get_time_ns();
process_audio();
let end_time = get_time_ns();
profiler.profile_function("process_audio", start_time, end_time);

let report = profiler.generate_report();
```

### Waveform Visualization
```rust
let visualizer = WaveformVisualizer::new(1024);
let waveform = visualizer.visualize_waveform(&samples);
println!("{}", waveform);
```

### Spectrum Visualization
```rust
let fft_viz = FFTVisualizer::new(2048);
let spectrum = fft_viz.visualize_spectrum(&magnitude_spectrum, 48000);
println!("{}", spectrum);
```

## 🎛️ Audio Processing Examples

### Generate Sine Wave
```rust
use hardware_support::audio::processing::DSP;

let dsp = DSP::new(48000);
let samples = dsp.generate_sine(440.0, 0.5, 1024); // A4, 0.5 amplitude
```

### Apply Effects
```rust
let filtered = dsp.low_pass_filter(&samples, 2000.0, 0.7);
let reverb = dsp.apply_reverb(&samples, 0.5, 0.3, 0.4);
let distorted = dsp.apply_distortion(&samples, 2.0, 0.5);
let compressed = dsp.apply_compressor(&samples, 0.3, 4.0, 0.01, 0.1);
```

### Audio Synthesis
```rust
use hardware_support::audio::processing::{Synthesizer, Oscillator};

let mut synth = Synthesizer::new(48000);
synth.set_envelope(0.01, 0.2, 0.8, 0.3); // ADSR
synth.note_on(60, 0.8); // Middle C
let samples = synth.generate_samples(1000);
synth.note_off(60);
```

## 🐛 Troubleshooting

### Common Issues

#### 1. No Audio Output
- Check device permissions
- Verify audio hardware is detected
- Ensure correct audio format configuration
- Monitor buffer underruns

#### 2. High Latency
- Use low-latency preset
- Reduce buffer sizes
- Enable hardware mixing
- Check CPU utilization

#### 3. Audio Distortion
- Check volume levels
- Verify format compatibility
- Monitor for clipping
- Adjust gain levels

#### 4. Device Not Detected
- Check hotplug event handlers
- Verify device detection code
- Monitor USB/PCI enumeration
- Check driver loading

### Debug Commands

```bash
# List audio devices
audio_demo interactive
# Then select option 3 for system information

# Run performance test
audio_demo performance

# Run stress test
audio_demo test
```

### Performance Tips

1. **Buffer Size Optimization**
   - Use larger buffers for stability
   - Use smaller buffers for low latency
   - Monitor underrun/overrun rates

2. **CPU Optimization**
   - Enable hardware acceleration
   - Reduce sample rate if possible
   - Use efficient audio formats

3. **Memory Management**
   - Reuse audio buffers
   - Monitor memory usage
   - Clean up unused resources

## 📚 Educational Resources

### Learning Path

1. **Audio Fundamentals**
   - Start with frequency generation example
   - Understand sample rates and bit depths
   - Learn about audio formats

2. **Audio Processing**
   - Study DSP algorithms
   - Learn about filters and effects
   - Understand spectrum analysis

3. **Audio Programming**
   - Practice with mixing examples
   - Learn real-time programming
   - Understand latency and timing

4. **Hardware Integration**
   - Study hardware abstraction layer
   - Learn about codec implementations
   - Understand device hotplugging

### Code Examples

All examples are fully documented with:
- Step-by-step explanations
- Performance considerations
- Educational notes
- Troubleshooting tips

### Interactive Tutorials

The demo program provides:
- Guided tutorials
- Real-time feedback
- Performance monitoring
- Visual demonstrations

## 🤝 Contributing

We welcome contributions to the MultiOS Audio Subsystem! Areas where help is needed:

1. **Hardware Support**: Additional codec drivers
2. **Educational Content**: More tutorials and examples
3. **Performance Optimization**: Real-time improvements
4. **Testing**: Comprehensive test coverage
5. **Documentation**: User guides and API docs

### Development Setup

```bash
# Clone repository
git clone <repository-url>
cd multios/hardware_support/audio

# Install dependencies
cargo install cargo-watch

# Run development server
cargo watch -x run --example demo

# Run tests continuously
cargo watch -x test
```

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.

## 🆘 Support

- **Documentation**: Check the `/docs` directory for detailed guides
- **Examples**: Run the demo program for interactive learning
- **Issues**: Report bugs and feature requests via GitHub issues
- **Community**: Join our Discord server for questions and discussions

## 🎯 Roadmap

### Short Term (v1.1)
- [ ] Bluetooth A2DP support
- [ ] HDMI audio codec
- [ ] VST plugin interface
- [ ] WebAssembly bindings

### Medium Term (v1.2)
- [ ] Spatial audio support
- [ ] Advanced effects library
- [ ] Machine learning audio processing
- [ ] Network audio streaming

### Long Term (v2.0)
- [ ] Multi-platform support (Windows, Linux, macOS)
- [ ] Professional audio workstation features
- [ ] Plugin architecture
- [ ] Cloud audio processing

---

**MultiOS Audio Subsystem** - Empowering the next generation of audio programmers and multimedia applications! 🎵✨