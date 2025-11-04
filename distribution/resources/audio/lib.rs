//! MultiOS Audio Subsystem Library
//! 
//! This is the main library module that provides a complete audio subsystem
//! for MultiOS, including device management, codec support, mixing, processing,
//! and educational examples.

pub mod core;
pub mod hal;
pub mod codecs;
pub mod mixing;
pub mod processing;
pub mod hotplug;
pub mod examples;
pub mod debug;
pub mod testing;

pub use crate::core::*;
pub use crate::hal::*;
pub use crate::codecs::*;
pub use crate::mixing::*;
pub use crate::processing::*;
pub use crate::hotplug::*;
pub use crate::examples::*;
pub use crate::debug::*;
pub use crate::testing::*;

/// Audio subsystem version information
pub const AUDIO_VERSION: &str = "1.0.0";
pub const AUDIO_BUILD_DATE: &str = "2025-11-03";

/// Initialize the complete audio subsystem
pub fn initialize_audio_system() -> Result<(), AudioError> {
    log_info!("Initializing MultiOS Audio Subsystem v{}", AUDIO_VERSION);
    
    // Initialize the core audio manager
    init_audio_system(48000)?;
    
    // Initialize hotplug manager
    let mut hotplug_manager = AudioHotplugManager::new();
    hotplug_manager.initialize()?;
    
    // Add default event handler
    hotplug_manager.add_event_handler(Box::new(DefaultHotplugHandler::new()));
    
    log_info!("MultiOS Audio Subsystem initialized successfully");
    Ok(())
}

/// Shutdown the audio subsystem
pub fn shutdown_audio_system() -> Result<(), AudioError> {
    log_info!("Shutting down MultiOS Audio Subsystem");
    
    // Stop all active streams
    if let Some(audio_manager) = get_audio_manager() {
        // Get all stream IDs and stop them
        // Implementation would iterate through active streams
    }
    
    log_info!("MultiOS Audio Subsystem shutdown complete");
    Ok(())
}

/// Get audio subsystem information
pub fn get_subsystem_info() -> SubsystemInfo {
    SubsystemInfo {
        version: AUDIO_VERSION,
        build_date: AUDIO_BUILD_DATE,
        capabilities: get_capabilities(),
        initialized: is_initialized(),
    }
}

/// Audio subsystem information
#[derive(Debug, Clone)]
pub struct SubsystemInfo {
    pub version: &'static str,
    pub build_date: &'static str,
    pub capabilities: AudioCapabilities,
    pub initialized: bool,
}

/// Check if audio subsystem is initialized
fn is_initialized() -> bool {
    unsafe {
        AUDIO_MANAGER.is_some()
    }
}

/// Get global audio capabilities
fn get_capabilities() -> AudioCapabilities {
    AudioCapabilities {
        supported_formats: vec![
            AudioFormat::Pcm16LE,
            AudioFormat::Pcm24LE,
            AudioFormat::Pcm32LE,
            AudioFormat::Float32LE,
        ],
        max_sample_rate: 192000,
        max_channels: 8,
        max_buffer_size: 1024 * 1024, // 1MB
        hardware_mixing: true,
        software_mixing: true,
    }
}

/// Educational tutorial runner
pub struct AudioEducationSystem {
    tutorial: AudioTutorial,
    monitor: AudioSystemMonitor,
    debugger: AudioDebugger,
}

impl AudioEducationSystem {
    /// Create a new education system
    pub fn new() -> Self {
        Self {
            tutorial: AudioTutorial::new(),
            monitor: AudioSystemMonitor::new(),
            debugger: AudioDebugger::new(),
        }
    }

    /// Initialize the education system
    pub fn initialize(&mut self) -> Result<(), AudioError> {
        log_info!("Initializing Audio Education System");
        
        // Initialize tutorial system
        self.tutorial.initialize()?;
        
        // Start monitoring
        self.monitor.start_monitoring();
        
        // Enable debugging for educational purposes
        self.debugger.enable_tracing();
        self.debugger.enable_performance_monitoring();
        
        log_info!("Audio Education System initialized");
        Ok(())
    }

    /// Run a specific educational example
    pub fn run_example(&mut self, example_index: usize) -> Result<(), AudioError> {
        log_info!("Running educational example {}", example_index);
        
        self.debugger.trace_function_entry("run_example");
        let result = self.tutorial.run_example(example_index);
        self.debugger.trace_function_exit("run_example", 0);
        
        result
    }

    /// Run all educational examples
    pub fn run_all_examples(&mut self) -> Result<(), AudioError> {
        log_info!("Running all educational examples");
        
        let examples = self.tutorial.list_examples();
        log_info!("Available examples:");
        for (i, example) in examples.iter().enumerate() {
            log_info!("  {}. {}", i + 1, example);
        }
        
        // Run examples sequentially
        while self.tutorial.get_current_example() < examples.len() {
            self.tutorial.run_next_example()?;
        }
        
        log_info!("All educational examples completed");
        Ok(())
    }

    /// List available examples
    pub fn list_examples(&self) -> Vec<String> {
        self.tutorial.list_examples()
    }

    /// Get performance monitoring data
    pub fn get_performance_data(&self) -> PerformanceReport {
        self.monitor.get_performance_report()
    }

    /// Get debug information
    pub fn get_debug_info(&self) -> DebugReport {
        self.debugger.generate_report()
    }

    /// Generate comprehensive system report
    pub fn generate_system_report(&self) -> ComprehensiveSystemReport {
        ComprehensiveSystemReport {
            subsystem_info: get_subsystem_info(),
            performance_report: self.get_performance_data(),
            debug_report: self.get_debug_info(),
            health_report: self.monitor.generate_health_report(),
            available_examples: self.list_examples(),
        }
    }
}

/// Comprehensive system report for educational purposes
#[derive(Debug, Clone)]
pub struct ComprehensiveSystemReport {
    pub subsystem_info: SubsystemInfo,
    pub performance_report: PerformanceReport,
    pub debug_report: DebugReport,
    pub health_report: HealthReport,
    pub available_examples: Vec<String>,
}

/// Audio configuration presets for common use cases
pub mod presets {
    use super::*;

    /// High-quality audio configuration (CD quality)
    pub fn cd_quality() -> StreamConfig {
        StreamConfig {
            format: AudioFormat::Pcm16LE,
            sample_rate: 44100,
            channels: 2,
            buffer_size: 1024,
            buffer_count: 4,
        }
    }

    /// Studio-quality audio configuration
    pub fn studio_quality() -> StreamConfig {
        StreamConfig {
            format: AudioFormat::Float32LE,
            sample_rate: 48000,
            channels: 2,
            buffer_size: 512,
            buffer_count: 8,
        }
    }

    /// High-resolution audio configuration
    pub fn high_resolution() -> StreamConfig {
        StreamConfig {
            format: AudioFormat::Float32LE,
            sample_rate: 96000,
            channels: 2,
            buffer_size: 256,
            buffer_count: 8,
        }
    }

    /// Ultra-high-resolution audio configuration
    pub fn ultra_high_resolution() -> StreamConfig {
        StreamConfig {
            format: AudioFormat::Float32LE,
            sample_rate: 192000,
            channels: 2,
            buffer_size: 128,
            buffer_count: 16,
        }
    }

    /// Low-latency real-time configuration
    pub fn low_latency() -> StreamConfig {
        StreamConfig {
            format: AudioFormat::Float32LE,
            sample_rate: 48000,
            channels: 2,
            buffer_size: 64,
            buffer_count: 4,
        }
    }

    /// Voice recording configuration
    pub fn voice_recording() -> StreamConfig {
        StreamConfig {
            format: AudioFormat::Pcm16LE,
            sample_rate: 16000,
            channels: 1,
            buffer_size: 512,
            buffer_count: 4,
        }
    }

    /// Surround sound configuration
    pub fn surround_sound() -> StreamConfig {
        StreamConfig {
            format: AudioFormat::Pcm24LE,
            sample_rate: 48000,
            channels: 6,
            buffer_size: 1024,
            buffer_count: 4,
        }
    }
}

/// Audio system utilities
pub mod utils {
    use super::*;

    /// Convert linear gain to decibels
    pub fn gain_to_db(gain: f32) -> f32 {
        if gain > 0.0 {
            20.0 * gain.log10()
        } else {
            -120.0
        }
    }

    /// Convert decibels to linear gain
    pub fn db_to_gain(db: f32) -> f32 {
        10.0_f32.powf(db / 20.0)
    }

    /// Convert sample rate to quality level
    pub fn sample_rate_to_quality(sample_rate: u32) -> &'static str {
        match sample_rate {
            8000 => "Telephone Quality",
            16000 => "Speech Quality",
            22050 => "AM Radio Quality",
            44100 => "CD Quality",
            48000 => "Professional Quality",
            96000 => "Studio Quality",
            192000 => "Ultra High Quality",
            _ => "Unknown Quality",
        }
    }

    /// Calculate buffer duration in seconds
    pub fn buffer_duration_samples(samples: usize, sample_rate: u32) -> f32 {
        samples as f32 / sample_rate as f32
    }

    /// Calculate buffer duration in milliseconds
    pub fn buffer_duration_ms(samples: usize, sample_rate: u32) -> f32 {
        buffer_duration_samples(samples, sample_rate) * 1000.0
    }

    /// Calculate bytes per second for audio data
    pub fn bytes_per_second(config: &StreamConfig) -> usize {
        let bytes_per_sample = match config.format {
            AudioFormat::Pcm8U | AudioFormat::Pcm8S => 1,
            AudioFormat::Pcm16LE | AudioFormat::Pcm16BE => 2,
            AudioFormat::Pcm24LE | AudioFormat::Pcm24BE => 3,
            AudioFormat::Pcm32LE | AudioFormat::Pcm32BE |
            AudioFormat::Float32LE | AudioFormat::Float32BE => 4,
            AudioFormat::Float64LE | AudioFormat::Float64BE => 8,
        };
        
        config.sample_rate as usize * config.channels as usize * bytes_per_sample
    }

    /// Validate audio configuration
    pub fn validate_config(config: &StreamConfig) -> Result<(), AudioError> {
        // Check sample rate
        if config.sample_rate < 8000 || config.sample_rate > 192000 {
            return Err(AudioError::InvalidSampleRate);
        }

        // Check channel count
        if config.channels == 0 || config.channels > 32 {
            return Err(AudioError::InvalidChannelCount);
        }

        // Check buffer size
        if config.buffer_size == 0 || config.buffer_size > 1048576 {
            return Err(AudioError::BufferOverflow);
        }

        // Check buffer count
        if config.buffer_count == 0 || config.buffer_count > 32 {
            return Err(AudioError::BufferOverflow);
        }

        Ok(())
    }

    /// Generate test tone
    pub fn generate_test_tone(frequency: f32, duration: f32, sample_rate: u32, amplitude: f32) -> Vec<f32> {
        let length = (duration * sample_rate as f32) as usize;
        let phase_increment = 2.0 * core::f32::consts::PI * frequency / sample_rate as f32;
        let mut samples = Vec::with_capacity(length);
        
        let mut phase = 0.0;
        for _ in 0..length {
            samples.push(amplitude * phase.sin());
            phase += phase_increment;
        }
        
        samples
    }

    /// Calculate RMS (Root Mean Square) level
    pub fn calculate_rms(samples: &[f32]) -> f32 {
        if samples.is_empty() {
            return 0.0;
        }
        
        let sum_squares: f32 = samples.iter().map(|&s| s * s).sum();
        (sum_squares / samples.len() as f32).sqrt()
    }

    /// Calculate peak level
    pub fn calculate_peak(samples: &[f32]) -> f32 {
        samples.iter().fold(0.0f32, |acc, &s| acc.max(s.abs()))
    }

    /// Calculate dynamic range in dB
    pub fn calculate_dynamic_range(samples: &[f32]) -> f32 {
        let peak = calculate_peak(samples);
        let rms = calculate_rms(samples);
        
        if rms > 0.0 {
            gain_to_db(peak / rms)
        } else {
            0.0
        }
    }
}

/// Testing utilities
pub mod testing {
    use super::*;

    /// Create test audio stream
    pub fn create_test_stream(audio_manager: &mut AudioManager, config: StreamConfig) -> Result<u32, AudioError> {
        let stream_id = audio_manager.create_stream(config)?;
        
        // Generate test tone
        let samples = super::utils::generate_test_tone(440.0, 1.0, config.sample_rate, 0.5);
        
        // In real implementation, would fill buffer with test data
        log_info!("Created test stream {} with configuration: {:?}Hz, {} channels", 
                 stream_id, config.sample_rate, config.channels);
        
        Ok(stream_id)
    }

    /// Performance test for audio subsystem
    pub fn run_performance_test(audio_manager: &mut AudioManager, duration: f32) -> PerformanceTestResult {
        log_info!("Running audio subsystem performance test for {:.1} seconds", duration);
        
        let start_time = 0; // Would use real timestamp
        let mut samples_processed = 0;
        
        // Simulate audio processing load
        for _ in 0..((duration * 48000.0) as usize / 1024) {
            // Simulate processing a buffer
            samples_processed += 1024;
            
            // Simulate work
            for _ in 0..1000 {
                let _ = 1.0 + 2.0;
            }
        }
        
        let end_time = start_time + (duration * 1000.0) as u64; // Simulate time
        
        PerformanceTestResult {
            duration,
            samples_processed,
            throughput: samples_processed as f32 / duration,
            cpu_cycles: samples_processed * 100, // Estimated
        }
    }

    /// Stress test for audio mixing
    pub fn run_mixing_stress_test(audio_manager: &mut AudioManager, stream_count: usize) -> MixingStressResult {
        log_info!("Running mixing stress test with {} streams", stream_count);
        
        let mut streams = Vec::new();
        
        // Create multiple streams
        for i in 0..stream_count {
            let config = StreamConfig {
                format: AudioFormat::Pcm16LE,
                sample_rate: 48000,
                channels: 2,
                buffer_size: 1024,
                buffer_count: 2,
            };
            
            match audio_manager.create_stream(config) {
                Ok(stream_id) => streams.push(stream_id),
                Err(e) => {
                    log_error!("Failed to create stream {}: {:?}", i, e);
                    break;
                }
            }
        }
        
        // Enable all streams
        for &stream_id in &streams {
            let _ = audio_manager.start_playback(stream_id);
        }
        
        log_info!("Mixing stress test: created {} streams", streams.len());
        
        MixingStressResult {
            streams_created: streams.len(),
            max_streams: stream_count,
            test_passed: streams.len() == stream_count,
        }
    }
}

/// Performance test result
#[derive(Debug, Clone)]
pub struct PerformanceTestResult {
    pub duration: f32,
    pub samples_processed: usize,
    pub throughput: f32,
    pub cpu_cycles: usize,
}

/// Mixing stress test result
#[derive(Debug, Clone)]
pub struct MixingStressResult {
    pub streams_created: usize,
    pub max_streams: usize,
    pub test_passed: bool,
}

// Logging function
fn log_info(msg: &str) {
    println!("[AUDIO SYSTEM] {}", msg);
}

fn log_error(msg: &str) {
    eprintln!("[AUDIO SYSTEM ERROR] {}", msg);
}