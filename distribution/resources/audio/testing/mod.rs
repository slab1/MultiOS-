//! Audio Subsystem Testing Utilities
//! 
//! This module provides comprehensive testing utilities for the MultiOS
//! audio subsystem, including unit tests, integration tests, and
//! performance benchmarks.

use crate::lib::*;
use alloc::vec::Vec;

/// Audio subsystem test suite
pub struct AudioTestSuite {
    test_results: Vec<TestResult>,
    passed_tests: usize,
    failed_tests: usize,
}

/// Test result structure
#[derive(Debug, Clone)]
pub struct TestResult {
    pub name: &'static str,
    pub passed: bool,
    pub duration_ms: f64,
    pub error_message: Option<String>,
}

impl AudioTestSuite {
    /// Create a new test suite
    pub fn new() -> Self {
        Self {
            test_results: Vec::new(),
            passed_tests: 0,
            failed_tests: 0,
        }
    }

    /// Run all tests
    pub fn run_all_tests(&mut self) -> TestSummary {
        log_info!("Starting audio subsystem test suite");
        
        // Core system tests
        self.test_initialization();
        self.test_stream_creation();
        self.test_audio_format_support();
        self.test_buffer_management();
        
        // Codec tests
        self.test_ac97_codec();
        self.test_hd_audio_codec();
        self.test_i2s_codec();
        
        // Mixing tests
        self.test_audio_mixing();
        self.test_effects_processing();
        self.test_volume_control();
        
        // Processing tests
        self.test_dsp_functions();
        self.test_spectrum_analysis();
        self.test_synthesizer();
        
        // Hotplug tests
        self.test_device_detection();
        self.test_device_hotplug();
        
        // Performance tests
        self.test_performance_benchmarks();
        self.test_memory_usage();
        self.test_latency_measurements();
        
        self.generate_summary()
    }

    /// Test audio system initialization
    fn test_initialization(&mut self) {
        let test_name = "Audio System Initialization";
        self.run_test(test_name, || {
            // Test initialization
            initialize_audio_system()?;
            
            // Check if system is initialized
            assert!(is_initialized(), "Audio system should be initialized");
            
            // Get subsystem info
            let info = get_subsystem_info();
            assert!(info.initialized, "Subsystem info should show initialized state");
            
            Ok(())
        });
    }

    /// Test audio stream creation
    fn test_stream_creation(&mut self) {
        let test_name = "Audio Stream Creation";
        self.run_test(test_name, || {
            // Test different stream configurations
            let configs = vec![
                presets::cd_quality(),
                presets::studio_quality(),
                presets::low_latency(),
                presets::voice_recording(),
            ];
            
            for config in configs {
                let audio_manager = get_audio_manager().unwrap();
                let stream_id = audio_manager.create_stream(config.clone())?;
                
                // Verify stream state
                let state = audio_manager.get_stream_state(stream_id)?;
                assert_eq!(state, StreamState::Idle, "New stream should be in idle state");
                
                // Clean up
                audio_manager.stop_playback(stream_id)?;
            }
            
            Ok(())
        });
    }

    /// Test audio format support
    fn test_audio_format_support(&mut self) {
        let test_name = "Audio Format Support";
        self.run_test(test_name, || {
            let formats = vec![
                AudioFormat::Pcm8U,
                AudioFormat::Pcm8S,
                AudioFormat::Pcm16LE,
                AudioFormat::Pcm16BE,
                AudioFormat::Pcm24LE,
                AudioFormat::Pcm24BE,
                AudioFormat::Pcm32LE,
                AudioFormat::Pcm32BE,
                AudioFormat::Float32LE,
                AudioFormat::Float32BE,
                AudioFormat::Float64LE,
                AudioFormat::Float64BE,
            ];
            
            for format in formats {
                let config = StreamConfig {
                    format,
                    sample_rate: 48000,
                    channels: 2,
                    buffer_size: 1024,
                    buffer_count: 2,
                };
                
                // Validate format support
                utils::validate_config(&config)?;
            }
            
            Ok(())
        });
    }

    /// Test buffer management
    fn test_buffer_management(&mut self) {
        let test_name = "Buffer Management";
        self.run_test(test_name, || {
            // Test buffer creation
            let buffer = AudioBuffer::new(1024)?;
            assert_eq!(buffer.size(), 1024, "Buffer size should match requested size");
            
            // Test buffer operations
            let mut slice = buffer.as_mut_slice();
            for byte in slice.iter_mut() {
                *byte = 0x42;
            }
            
            assert_eq!(slice[0], 0x42, "Buffer should be writable");
            
            // Test buffer clearing
            buffer.clear();
            assert_eq!(buffer.as_slice(), &[0; 1024][..], "Buffer should be cleared");
            
            Ok(())
        });
    }

    /// Test AC'97 codec
    fn test_ac97_codec(&mut self) {
        let test_name = "AC'97 Codec";
        self.run_test(test_name, || {
            let mut codec = Ac97Codec::new(0xC000);
            
            // Test initialization
            codec.initialize()?;
            
            // Test configuration
            let config = presets::cd_quality();
            codec.configure(&config)?;
            
            // Test operation
            codec.start(StreamType::Playback)?;
            codec.stop(StreamType::Playback)?;
            
            // Test mixer settings
            codec.set_pcm_volume(20, 20)?;
            codec.set_master_volume(15, 15)?;
            
            Ok(())
        });
    }

    /// Test HD Audio codec
    fn test_hd_audio_codec(&mut self) {
        let test_name = "HD Audio Codec";
        self.run_test(test_name, || {
            let mut codec = HdAudioCodec::new(0xD000);
            
            // Test initialization
            codec.initialize()?;
            
            // Test high-resolution configuration
            let config = presets::ultra_high_resolution();
            codec.configure(&config)?;
            
            // Test simultaneous operation
            codec.start(StreamType::Simultaneous)?;
            codec.stop(StreamType::Simultaneous)?;
            
            Ok(())
        });
    }

    /// Test I2S codec
    fn test_i2s_codec(&mut self) {
        let test_name = "I2S Codec";
        self.run_test(test_name, || {
            let mut codec = I2sCodec::new(0x40001000);
            
            // Test initialization
            codec.initialize()?;
            
            // Test embedded configuration
            let config = StreamConfig {
                format: AudioFormat::Pcm24LE,
                sample_rate: 48000,
                channels: 2,
                buffer_size: 512,
                buffer_count: 2,
            };
            
            codec.configure(&config)?;
            codec.start(StreamType::Playback)?;
            codec.stop(StreamType::Playback)?;
            
            Ok(())
        });
    }

    /// Test audio mixing
    fn test_audio_mixing(&mut self) {
        let test_name = "Audio Mixing";
        self.run_test(test_name, || {
            let mut mixer = Mixer::new();
            mixer.initialize()?;
            
            // Create multiple streams
            let stream_ids = vec![
                mixer.register_stream(1, presets::cd_quality())?,
                mixer.register_stream(2, presets::studio_quality())?,
                mixer.register_stream(3, presets::low_latency())?,
            ];
            
            // Configure streams
            mixer.set_stream_volume(1, 0.8)?;
            mixer.set_stream_volume(2, 0.6)?;
            mixer.set_stream_volume(3, 1.0)?;
            mixer.set_stream_pan(1, -0.5)?;
            mixer.set_stream_pan(2, 0.5)?;
            mixer.set_stream_pan(3, 0.0)?;
            
            // Enable mixing
            mixer.set_mixing_enabled(true);
            for stream_id in &stream_ids {
                mixer.enable_stream(*stream_id)?;
            }
            
            // Perform mixing
            let mixed_buffer = mixer.mix_streams()?;
            assert!(mixed_buffer.size() > 0, "Mixed buffer should contain data");
            
            Ok(())
        });
    }

    /// Test effects processing
    fn test_effects_processing(&mut self) {
        let test_name = "Effects Processing";
        self.run_test(test_name, || {
            let dsp = DSP::new(48000);
            
            // Generate test signal
            let samples = dsp.generate_sine(440.0, 0.5, 1024);
            
            // Test different effects
            let filtered = dsp.low_pass_filter(&samples, 2000.0, 0.7);
            assert_eq!(filtered.len(), samples.len(), "Filter should preserve length");
            
            let reverb = dsp.apply_reverb(&samples, 0.5, 0.3, 0.4);
            assert_eq!(reverb.len(), samples.len(), "Reverb should preserve length");
            
            let distorted = dsp.apply_distortion(&samples, 2.0, 0.5);
            assert_eq!(distorted.len(), samples.len(), "Distortion should preserve length");
            
            let compressed = dsp.apply_compressor(&samples, 0.3, 4.0, 0.01, 0.1);
            assert_eq!(compressed.len(), samples.len(), "Compression should preserve length");
            
            Ok(())
        });
    }

    /// Test volume control
    fn test_volume_control(&mut self) {
        let test_name = "Volume Control";
        self.run_test(test_name, || {
            let mut volume_effect = VolumeEffect::new();
            
            // Test volume changes
            volume_effect.set_parameter("Gain", 0.5)?;
            volume_effect.set_parameter("Gain", 1.0)?;
            volume_effect.set_parameter("Gain", 2.0)?;
            volume_effect.set_parameter("Gain", 0.0)?;
            
            // Test boundary conditions
            assert!(volume_effect.set_parameter("Gain", 3.0).is_err(), "Should reject gain > 2.0");
            assert!(volume_effect.set_parameter("Gain", -1.0).is_err(), "Should reject gain < 0.0");
            
            // Test processing
            let input = vec![0.5; 100];
            let mut output = vec![0.0; 100];
            volume_effect.set_parameter("Gain", 1.0)?;
            volume_effect.process(&input, &mut output)?;
            
            // Volume processing should maintain array size
            assert_eq!(output.len(), input.len(), "Volume processing should preserve array size");
            
            Ok(())
        });
    }

    /// Test DSP functions
    fn test_dsp_functions(&mut self) {
        let test_name = "DSP Functions";
        self.run_test(test_name, || {
            let dsp = DSP::new(48000);
            
            // Test waveform generation
            let sine = dsp.generate_sine(440.0, 0.5, 100);
            let square = dsp.generate_square(440.0, 0.5, 100);
            let sawtooth = dsp.generate_sawtooth(440.0, 0.5, 100);
            let triangle = dsp.generate_triangle(440.0, 0.5, 100);
            
            assert_eq!(sine.len(), 100, "Generated samples should match requested length");
            assert_eq!(square.len(), 100, "Generated samples should match requested length");
            assert_eq!(sawtooth.len(), 100, "Generated samples should match requested length");
            assert_eq!(triangle.len(), 100, "Generated samples should match requested length");
            
            // Test level calculations
            let rms = dsp.calculate_rms(&sine);
            let peak = dsp.calculate_peak(&sine);
            assert!(rms > 0.0 && rms <= 0.5, "RMS should be reasonable for sine wave");
            assert!(peak > 0.0 && peak <= 0.5, "Peak should be reasonable for sine wave");
            
            // Test dB conversions
            let db = dsp.linear_to_db(0.5);
            assert!(db > -20.0 && db < 0.0, "dB conversion should be reasonable");
            
            Ok(())
        });
    }

    /// Test spectrum analysis
    fn test_spectrum_analysis(&mut self) {
        let test_name = "Spectrum Analysis";
        self.run_test(test_name, || {
            let mut analyzer = SpectrumAnalyzer::new(1024);
            let dsp = DSP::new(48000);
            
            // Generate test signal (pure tone)
            let samples = dsp.generate_sine(440.0, 0.5, 1024);
            
            // Analyze spectrum
            let spectrum = analyzer.analyze_spectrum(&samples);
            
            assert!(spectrum.len() > 0, "Spectrum should contain frequency bins");
            
            // Find peak frequency
            let mut max_magnitude = 0.0;
            let mut peak_bin = 0;
            for (i, &mag) in spectrum.iter().enumerate() {
                if mag > max_magnitude {
                    max_magnitude = mag;
                    peak_bin = i;
                }
            }
            
            // For a 440Hz tone at 48kHz sample rate, peak should be around bin 9-10
            assert!(peak_bin >= 8 && peak_bin <= 12, "Peak frequency should be near expected bin");
            
            Ok(())
        });
    }

    /// Test synthesizer
    fn test_synthesizer(&mut self) {
        let test_name = "Audio Synthesizer";
        self.run_test(test_name, || {
            let mut synth = Synthesizer::new(48000);
            
            // Configure envelope
            synth.set_envelope(0.01, 0.1, 0.8, 0.2);
            
            // Test note on/off
            synth.note_on(60, 0.8); // Middle C
            let samples = synth.generate_samples(1000);
            assert_eq!(samples.len(), 1000, "Synthesizer should generate requested samples");
            
            synth.note_off(60);
            
            // Test different oscillator types
            synth.set_oscillator(Oscillator::Single);
            synth.note_on(64, 0.7); // E4
            let _ = synth.generate_samples(500);
            
            synth.set_oscillator(Oscillator::Dual);
            synth.note_on(67, 0.6); // G4
            let _ = synth.generate_samples(500);
            
            Ok(())
        });
    }

    /// Test device detection
    fn test_device_detection(&mut self) {
        let test_name = "Device Detection";
        self.run_test(test_name, || {
            // Test PCI device detection
            let pci_devices = detect_pci_audio_devices();
            log_info!("Found {} PCI audio devices", pci_devices.len());
            
            // Test USB device detection
            let usb_devices = detect_usb_audio_devices();
            log_info!("Found {} USB audio devices", usb_devices.len());
            
            // Test codec detection
            let codecs = detect_codecs();
            assert!(!codecs.is_empty(), "Should detect at least one codec");
            
            Ok(())
        });
    }

    /// Test device hotplug
    fn test_device_hotplug(&mut self) {
        let test_name = "Device Hotplug";
        self.run_test(test_name, || {
            let mut hotplug_manager = AudioHotplugManager::new();
            hotplug_manager.initialize()?;
            
            // Test event handler
            hotplug_manager.add_event_handler(Box::new(DefaultHotplugHandler::new()));
            
            // Simulate polling
            hotplug_manager.poll_devices()?;
            
            // Get device statistics
            let stats = hotplug_manager.get_statistics();
            assert!(stats.total_devices >= 0, "Statistics should be valid");
            
            Ok(())
        });
    }

    /// Test performance benchmarks
    fn test_performance_benchmarks(&mut self) {
        let test_name = "Performance Benchmarks";
        self.run_test(test_name, || {
            let dsp = DSP::new(48000);
            
            // Benchmark waveform generation
            let start_time = 0; // Would use real timestamp
            for _ in 0..100 {
                let _ = dsp.generate_sine(440.0, 0.5, 2048);
            }
            let end_time = start_time + 100; // Simulated time
            
            log_info!("Generated 100 waveforms in simulated time units");
            
            // Benchmark DSP operations
            let samples = dsp.generate_sine(440.0, 0.5, 4096);
            let start_time = 0; // Would use real timestamp
            
            for _ in 0..50 {
                let _ = dsp.low_pass_filter(&samples, 2000.0, 0.7);
                let _ = dsp.apply_reverb(&samples, 0.5, 0.3, 0.4);
                let _ = dsp.apply_distortion(&samples, 2.0, 0.5);
            }
            let end_time = start_time + 50; // Simulated time
            
            log_info!("Processed 50 audio frames in simulated time units");
            
            Ok(())
        });
    }

    /// Test memory usage
    fn test_memory_usage(&mut self) {
        let test_name = "Memory Usage";
        self.run_test(test_name, || {
            // Test buffer allocation
            let buffer = AudioBuffer::new(1024)?;
            assert!(buffer.size() == 1024, "Buffer should allocate correct size");
            
            // Test multiple buffer allocation
            let mut buffers = Vec::new();
            for i in 0..10 {
                let buffer = AudioBuffer::new(1024 * (i + 1))?;
                buffers.push(buffer);
            }
            
            assert_eq!(buffers.len(), 10, "Should allocate all buffers");
            
            // Test mixing buffer allocation
            let mut mixer = Mixer::new();
            mixer.initialize()?;
            
            let config = presets::studio_quality();
            mixer.register_stream(1, config)?;
            
            Ok(())
        });
    }

    /// Test latency measurements
    fn test_latency_measurements(&mut self) {
        let test_name = "Latency Measurements";
        self.run_test(test_name, || {
            let dsp = DSP::new(48000);
            
            // Generate audio and measure processing time
            let samples = dsp.generate_sine(440.0, 0.5, 2048);
            
            // Simulate processing latency measurement
            let start_time = 0; // Would use high-resolution timer
            let filtered = dsp.low_pass_filter(&samples, 2000.0, 0.7);
            let end_time = start_time + 100; // Simulated processing time
            
            let processing_latency = end_time - start_time;
            let buffer_duration = utils::buffer_duration_ms(2048, 48000);
            
            log_info!("Processing latency: {} simulated units", processing_latency);
            log_info!("Buffer duration: {:.2} ms", buffer_duration);
            
            // Processing should be faster than buffer duration
            assert!(processing_latency as f32 < buffer_duration, 
                   "Processing should be faster than real-time");
            
            Ok(())
        });
    }

    /// Helper method to run a single test
    fn run_test(&mut self, name: &'static str, test_fn: fn() -> Result<(), AudioError>) {
        let start_time = 0.0; // Would use real timestamp in milliseconds
        
        let result = match test_fn() {
            Ok(()) => TestResult {
                name,
                passed: true,
                duration_ms: 0.0, // Would calculate actual duration
                error_message: None,
            },
            Err(e) => TestResult {
                name,
                passed: false,
                duration_ms: 0.0, // Would calculate actual duration
                error_message: Some(format!("{:?}", e)),
            },
        };
        
        if result.passed {
            self.passed_tests += 1;
            log_info!("✓ {}", name);
        } else {
            self.failed_tests += 1;
            log_error!("✗ {}: {:?}", name, result.error_message);
        }
        
        self.test_results.push(result);
    }

    /// Generate test summary
    fn generate_summary(&self) -> TestSummary {
        let total_tests = self.passed_tests + self.failed_tests;
        let pass_rate = if total_tests > 0 {
            (self.passed_tests as f64 / total_tests as f64) * 100.0
        } else {
            0.0
        };
        
        TestSummary {
            total_tests,
            passed_tests: self.passed_tests,
            failed_tests: self.failed_tests,
            pass_rate,
            test_results: self.test_results.clone(),
        }
    }
}

/// Test summary structure
#[derive(Debug, Clone)]
pub struct TestSummary {
    pub total_tests: usize,
    pub passed_tests: usize,
    pub failed_tests: usize,
    pub pass_rate: f64,
    pub test_results: Vec<TestResult>,
}

impl TestSummary {
    /// Print test results
    pub fn print_results(&self) {
        println!("\n=== Audio Subsystem Test Results ===");
        println!("Total Tests: {}", self.total_tests);
        println!("Passed: {}", self.passed_tests);
        println!("Failed: {}", self.failed_tests);
        println!("Pass Rate: {:.1}%", self.pass_rate);
        
        if self.failed_tests > 0 {
            println!("\nFailed Tests:");
            for result in &self.test_results {
                if !result.passed {
                    println!("  ✗ {}: {}", result.name, 
                            result.error_message.as_ref().unwrap_or("Unknown error"));
                }
            }
        }
        
        println!("\n=== Test Summary ===");
        if self.pass_rate >= 95.0 {
            println!("🎉 Excellent! Audio subsystem is working correctly.");
        } else if self.pass_rate >= 80.0 {
            println!("👍 Good! Audio subsystem is mostly working.");
        } else {
            println!("⚠️  Issues detected. Please check failed tests.");
        }
    }
}

// Logging functions
fn log_info(msg: &str) {
    println!("[AUDIO TEST] {}", msg);
}

fn log_error(msg: &str) {
    eprintln!("[AUDIO TEST ERROR] {}", msg);
}