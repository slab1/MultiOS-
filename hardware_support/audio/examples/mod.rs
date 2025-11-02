//! Audio Education Examples
//! 
//! This module provides educational examples and tutorials for learning
//! audio programming concepts in MultiOS, including frequency generation,
//! recording, playback, effects processing, and audio analysis.

use crate::core::{AudioManager, AudioFormat, StreamConfig, StreamState, AudioError, init_audio_system};
use crate::processing::{DSP, SpectrumAnalyzer, Synthesizer, AudioRecorder};
use crate::mixing::{Mixer, VolumeEffect, PanEffect, LowPassFilter};
use crate::hotplug::{AudioHotplugManager, DefaultHotplugHandler};
use alloc::vec::Vec;
use alloc::string::String;

/// Educational audio tutorial runner
pub struct AudioTutorial {
    examples: Vec<AudioExample>,
    current_example: usize,
    audio_manager: AudioManager,
    dsp: DSP,
}

impl AudioTutorial {
    /// Create a new tutorial runner
    pub fn new() -> Self {
        let mut examples = Vec::new();
        
        // Register all educational examples
        examples.push(Box::new(FrequencyGenerationExample::new()) as Box<dyn AudioExample>);
        examples.push(Box::new(AudioRecordingExample::new()) as Box<dyn AudioExample>);
        examples.push(Box::new(AudioPlaybackExample::new()) as Box<dyn AudioExample>);
        examples.push(Box::new(MixingExample::new()) as Box<dyn AudioExample>);
        examples.push(Box::new(EffectsProcessingExample::new()) as Box<dyn AudioExample>);
        examples.push(Box::new(SpectrumAnalysisExample::new()) as Box<dyn AudioExample>);
        examples.push(Box::new(SynthesizerExample::new()) as Box<dyn AudioExample>);
        examples.push(Box::new(HotplugExample::new()) as Box<dyn AudioExample>);

        Self {
            examples,
            current_example: 0,
            audio_manager: AudioManager::new(48000),
            dsp: DSP::new(48000),
        }
    }

    /// Initialize the tutorial system
    pub fn initialize(&mut self) -> Result<(), AudioError> {
        // Initialize audio manager
        self.audio_manager.initialize()?;
        
        // Add default hotplug handler
        let hotplug_manager = AudioHotplugManager::new();
        hotplug_manager.add_event_handler(Box::new(DefaultHotplugHandler::new()));
        
        log_info!("Audio tutorial system initialized");
        log_info!("Available examples: {}", self.examples.len());
        
        Ok(())
    }

    /// Run the next example
    pub fn run_next_example(&mut self) -> Result<(), AudioError> {
        if self.current_example >= self.examples.len() {
            log_info!("All examples completed!");
            return Ok(());
        }

        let example_name = self.examples[self.current_example].get_name();
        log_info!("\n=== Running Example {}: {} ===", 
                 self.current_example + 1, example_name);

        // Run the current example
        self.examples[self.current_example].run(&mut self.audio_manager, &self.dsp)?;
        
        self.current_example += 1;
        Ok(())
    }

    /// Run a specific example by index
    pub fn run_example(&mut self, index: usize) -> Result<(), AudioError> {
        if index < self.examples.len() {
            let example_name = self.examples[index].get_name();
            log_info!("\n=== Running Example {}: {} ===", 
                     index + 1, example_name);
            self.examples[index].run(&mut self.audio_manager, &self.dsp)?;
            Ok(())
        } else {
            Err(AudioError::InvalidState)
        }
    }

    /// List all available examples
    pub fn list_examples(&self) -> Vec<String> {
        self.examples.iter().enumerate()
            .map(|(i, ex)| format!("{}. {}", i + 1, ex.get_name()))
            .collect()
    }

    /// Get current example index
    pub fn get_current_example(&self) -> usize {
        self.current_example
    }

    /// Reset to first example
    pub fn reset(&mut self) {
        self.current_example = 0;
        log_info!("Tutorial reset to first example");
    }
}

/// Generic audio example trait
pub trait AudioExample {
    fn get_name(&self) -> &'static str;
    fn get_description(&self) -> &'static str;
    fn run(&mut self, audio_manager: &mut AudioManager, dsp: &DSP) -> Result<(), AudioError>;
}

/// 1. Frequency Generation Example
pub struct FrequencyGenerationExample {
    frequencies: Vec<f32>,
    duration: f32,
}

impl FrequencyGenerationExample {
    pub fn new() -> Self {
        Self {
            frequencies: vec![440.0, 523.25, 659.25, 783.99, 880.0], // A4, C5, E5, G5, A5
            duration: 2.0,
        }
    }
}

impl AudioExample for FrequencyGenerationExample {
    fn get_name(&self) -> &'static str {
        "Frequency Generation"
    }

    fn get_description(&self) -> &'static str {
        "Generate sine waves at different frequencies using DSP"
    }

    fn run(&mut self, _audio_manager: &mut AudioManager, dsp: &DSP) -> Result<(), AudioError> {
        log_info!("Generating frequency test tones...");
        
        for (i, &freq) in self.frequencies.iter().enumerate() {
            log_info!("Generating {} Hz tone...", freq);
            
            // Generate sine wave
            let sample_rate = dsp.sample_rate;
            let length = (self.duration * sample_rate as f32) as usize;
            let samples = dsp.generate_sine(freq, 0.5, length);
            
            // Calculate RMS and peak levels
            let rms = dsp.calculate_rms(&samples);
            let peak = dsp.calculate_peak(&samples);
            let rms_db = dsp.linear_to_db(rms);
            let peak_db = dsp.linear_to_db(peak);
            
            log_info!("  RMS: {:.3} ({:.1} dB), Peak: {:.3} ({:.1} dB)", 
                     rms, rms_db, peak, peak_db);
            
            // Generate other waveforms for comparison
            let square_samples = dsp.generate_square(freq, 0.3, length / 4);
            let sawtooth_samples = dsp.generate_sawtooth(freq, 0.3, length / 4);
            let triangle_samples = dsp.generate_triangle(freq, 0.3, length / 4);
            
            log_info!("  Generated {} samples of sine, square, sawtooth, and triangle waves", 
                     samples.len());
            
            // Simulate playback delay
            log_info!("  Playing tone for {:.1} seconds...", self.duration);
        }

        log_info!("Frequency generation example completed");
        Ok(())
    }
}

/// 2. Audio Recording Example
pub struct AudioRecordingExample {
    record_duration: f32,
    sample_rate: u32,
}

impl AudioRecordingExample {
    pub fn new() -> Self {
        Self {
            record_duration: 5.0,
            sample_rate: 48000,
        }
    }
}

impl AudioExample for AudioRecordingExample {
    fn get_name(&self) -> &'static str {
        "Audio Recording"
    }

    fn get_description(&self) -> &'static str {
        "Record audio from microphone or line input"
    }

    fn run(&mut self, _audio_manager: &mut AudioManager, dsp: &DSP) -> Result<(), AudioError> {
        log_info!("Starting audio recording example...");
        
        // Create audio recorder
        let mut recorder = AudioRecorder::new(self.sample_rate, 1024);
        
        // Start recording
        recorder.start_recording();
        log_info!("Recording started for {:.1} seconds...", self.record_duration);
        
        // Simulate recording process
        let total_samples = (self.record_duration * self.sample_rate as f32) as usize;
        let chunk_size = 1024;
        
        for chunk_start in (0..total_samples).step_by(chunk_size) {
            let chunk_end = (chunk_start + chunk_size).min(total_samples);
            let chunk_length = chunk_end - chunk_start;
            
            // Simulate audio input (normally would come from hardware)
            let samples = dsp.generate_white_noise(0.1, chunk_length);
            recorder.add_samples(&samples);
            
            let progress = (chunk_end as f32 / total_samples as f32) * 100.0;
            log_info!("Recording progress: {:.1}%", progress);
        }
        
        // Stop recording
        recorder.stop_recording();
        
        // Analyze recorded data
        let recorded_data = recorder.get_recorded_data();
        if !recorded_data.is_empty() {
            let rms = dsp.calculate_rms(recorded_data);
            let peak = dsp.calculate_peak(recorded_data);
            let duration = recorded_data.len() as f32 / self.sample_rate as f32;
            
            log_info!("Recording completed:");
            log_info!("  Duration: {:.2} seconds", duration);
            log_info!("  Sample count: {}", recorded_data.len());
            log_info!("  RMS level: {:.3}", rms);
            log_info!("  Peak level: {:.3}", peak);
            log_info!("  Average level: {:.3}", 
                     recorded_data.iter().sum::<f32>() / recorded_data.len() as f32);
        }
        
        log_info!("Audio recording example completed");
        Ok(())
    }
}

/// 3. Audio Playback Example
pub struct AudioPlaybackExample {
    test_duration: f32,
}

impl AudioPlaybackExample {
    pub fn new() -> Self {
        Self {
            test_duration: 3.0,
        }
    }
}

impl AudioExample for AudioExample for AudioPlaybackExample {
    fn get_name(&self) -> &'static str {
        "Audio Playback"
    }

    fn get_description(&self) -> &'static str {
        "Play back audio samples with volume and format control"
    }

    fn run(&mut self, audio_manager: &mut AudioManager, dsp: &DSP) -> Result<(), AudioError> {
        log_info!("Starting audio playback example...");
        
        // Create stream configuration for high-quality playback
        let config = StreamConfig {
            format: AudioFormat::Float32LE,
            sample_rate: 48000,
            channels: 2,
            buffer_size: 1024,
            buffer_count: 4,
        };
        
        // Create audio stream
        let stream_id = audio_manager.create_stream(config.clone())?;
        log_info!("Created audio stream {} for playback", stream_id);
        
        // Generate test audio (sweep from 100Hz to 1000Hz)
        let sample_rate = 48000;
        let length = (self.test_duration * sample_rate as f32) as usize;
        let mut samples = Vec::with_capacity(length);
        
        log_info!("Generating frequency sweep test audio...");
        for i in 0..length {
            let t = i as f32 / sample_rate as f32;
            let freq = 100.0 + 900.0 * (t / self.test_duration); // Linear sweep
            let sample = (2.0 * core::f32::consts::PI * freq * t).sin() * 0.3;
            samples.push(sample);
        }
        
        // Apply ADSR envelope to prevent clicks
        let mut envelope_samples = samples.clone();
        dsp.apply_adsr(&mut envelope_samples, 0.1, 0.1, 0.8, 0.2);
        
        // Normalize audio to prevent clipping
        let normalized_samples = dsp.normalize(&envelope_samples, 0.8);
        
        log_info!("Starting playback...");
        audio_manager.start_playback(stream_id)?;
        
        // Simulate buffer streaming (normally done by audio callback)
        let chunk_size = 512;
        for chunk_start in (0..normalized_samples.len()).step_by(chunk_size) {
            let chunk_end = (chunk_start + chunk_size).min(normalized_samples.len());
            let chunk = &normalized_samples[chunk_start..chunk_end];
            
            // Simulate sending data to audio hardware
            let progress = (chunk_end as f32 / normalized_samples.len() as f32) * 100.0;
            log_info!("Playback progress: {:.1}%", progress);
            
            // Simulate timing delay
            core::thread::sleep(core::time::Duration::from_millis(10));
        }
        
        // Stop playback
        audio_manager.stop_playback(stream_id)?;
        
        log_info!("Audio playback example completed");
        Ok(())
    }
}

/// 4. Audio Mixing Example
pub struct MixingExample {
    stream_count: usize,
}

impl MixingExample {
    pub fn new() -> Self {
        Self {
            stream_count: 4,
        }
    }
}

impl AudioExample for MixingExample {
    fn get_name(&self) -> &'static str {
        "Audio Mixing"
    }

    fn get_description(&self) -> &'static str {
        "Mix multiple audio streams with different volumes and effects"
    }

    fn run(&mut self, _audio_manager: &mut AudioManager, dsp: &DSP) -> Result<(), AudioError> {
        log_info!("Starting audio mixing example...");
        
        // Create mixer
        let mut mixer = Mixer::new();
        mixer.initialize()?;
        
        // Create multiple audio streams with different properties
        let sample_rate = 48000;
        let duration = 2.0;
        let length = (duration * sample_rate as f32) as usize;
        
        let frequencies = [440.0, 554.37, 659.25, 880.0]; // Musical notes
        let volumes = [1.0, 0.7, 0.5, 0.3];
        let pans = [-0.8, -0.4, 0.4, 0.8];
        
        log_info!("Creating {} audio streams for mixing...", self.stream_count);
        
        // Create audio streams
        for i in 0..self.stream_count {
            let config = StreamConfig {
                format: AudioFormat::Float32LE,
                sample_rate,
                channels: 2,
                buffer_size: length,
                buffer_count: 2,
            };
            
            let stream_id = mixer.register_stream(i as u32, config.clone())?;
            
            // Generate tone for this stream
            let samples = dsp.generate_sine(frequencies[i], volumes[i], length);
            
            // Set stream properties
            mixer.set_stream_volume(stream_id, volumes[i])?;
            mixer.set_stream_pan(stream_id, pans[i])?;
            
            // Add effects
            let volume_effect = Box::new(VolumeEffect::new()) as Box<dyn crate::mixing::AudioEffect>;
            mixer.add_effect(stream_id, volume_effect)?;
            
            let pan_effect = Box::new(PanEffect::new()) as Box<dyn crate::mixing::AudioEffect>;
            mixer.add_effect(stream_id, pan_effect)?;
            
            log_info!("Stream {}: {:.1} Hz, volume {:.1}, pan {:.1}", 
                     i, frequencies[i], volumes[i], pans[i]);
        }
        
        // Enable all streams
        for i in 0..self.stream_count {
            mixer.enable_stream(i as u32)?;
        }
        
        // Perform mixing
        log_info!("Mixing audio streams...");
        let mixed_buffer = mixer.mix_streams()?;
        
        // Analyze mixed result
        let mixed_samples = unsafe {
            core::slice::from_raw_parts(
                mixed_buffer.as_slice().as_ptr() as *const f32,
                mixed_buffer.size() / 4
            )
        };
        
        let rms = dsp.calculate_rms(mixed_samples);
        let peak = dsp.calculate_peak(mixed_samples);
        
        log_info!("Mixed audio analysis:");
        log_info!("  RMS level: {:.3}", rms);
        log_info!("  Peak level: {:.3}", peak);
        log_info!("  Dynamic range: {:.1} dB", 
                 dsp.linear_to_db(peak / rms.max(0.001)));
        
        // Show mixer info
        let mixer_info = mixer.get_info();
        log_info!("Mixer statistics:");
        log_info!("  Active streams: {}", mixer_info.active_streams);
        log_info!("  Total streams: {}", mixer_info.total_streams);
        log_info!("  Master volume: {:.2}", mixer_info.master_volume);
        
        log_info!("Audio mixing example completed");
        Ok(())
    }
}

/// 5. Effects Processing Example
pub struct EffectsProcessingExample {
    effect_count: usize,
}

impl EffectsProcessingExample {
    pub fn new() -> Self {
        Self {
            effect_count: 3,
        }
    }
}

impl AudioExample for EffectsProcessingExample {
    fn get_name(&self) -> &'static str {
        "Effects Processing"
    }

    fn get_description(&self) -> &'static str {
        "Apply various audio effects: reverb, distortion, compression"
    }

    fn run(&mut self, _audio_manager: &mut AudioManager, dsp: &DSP) -> Result<(), AudioError> {
        log_info!("Starting effects processing example...");
        
        // Generate test signal (guitar-like tone)
        let sample_rate = 48000;
        let duration = 3.0;
        let length = (duration * sample_rate as f32) as usize;
        
        log_info!("Generating test signal...");
        let mut samples = Vec::new();
        
        // Generate harmonic-rich tone (simulating guitar)
        for i in 0..length {
            let t = i as f32 / sample_rate as f32;
            let fundamental = 220.0; // A3
            let harmonic2 = fundamental * 2.0;
            let harmonic3 = fundamental * 3.0;
            let harmonic4 = fundamental * 4.0;
            
            let sample = 0.3 * (fundamental * t).sin() +
                        0.2 * (harmonic2 * t).sin() +
                        0.1 * (harmonic3 * t).sin() +
                        0.05 * (harmonic4 * t).sin();
            
            samples.push(sample);
        }
        
        // Apply ADSR envelope
        dsp.apply_adsr(&mut samples, 0.01, 0.2, 0.7, 0.5);
        
        // Apply different effects
        let original_rms = dsp.calculate_rms(&samples);
        let original_peak = dsp.calculate_peak(&samples);
        
        log_info!("Original signal: RMS {:.3}, Peak {:.3}", 
                 original_rms, original_peak);
        
        // 1. Apply reverb
        log_info!("Applying reverb effect...");
        let reverb_samples = dsp.apply_reverb(&samples, 0.7, 0.3, 0.4);
        let reverb_rms = dsp.calculate_rms(&reverb_samples);
        let reverb_peak = dsp.calculate_peak(&reverb_samples);
        log_info!("  Reverb result: RMS {:.3} ({:.1} dB), Peak {:.3}", 
                 reverb_rms, dsp.linear_to_db(reverb_rms / original_rms), reverb_peak);
        
        // 2. Apply distortion
        log_info!("Applying distortion effect...");
        let distorted_samples = dsp.apply_distortion(&samples, 3.0, 0.5);
        let distorted_rms = dsp.calculate_rms(&distorted_samples);
        let distorted_peak = dsp.calculate_peak(&distorted_samples);
        log_info!("  Distortion result: RMS {:.3} ({:.1} dB), Peak {:.3}", 
                 distorted_rms, dsp.linear_to_db(distorted_rms / original_rms), distorted_peak);
        
        // 3. Apply compression
        log_info!("Applying compression effect...");
        let compressed_samples = dsp.apply_compressor(&samples, 0.3, 4.0, 0.01, 0.1);
        let compressed_rms = dsp.calculate_rms(&compressed_samples);
        let compressed_peak = dsp.calculate_peak(&compressed_samples);
        log_info!("  Compression result: RMS {:.3} ({:.1} dB), Peak {:.3}", 
                 compressed_rms, dsp.linear_to_db(compressed_rms / original_rms), compressed_peak);
        
        // 4. Apply filter chain (low-pass + high-pass)
        log_info!("Applying filter effects...");
        let low_passed = dsp.low_pass_filter(&samples, 2000.0, 0.7);
        let band_passed = dsp.high_pass_filter(&samples, 80.0);
        let filtered_rms = dsp.calculate_rms(&band_passed);
        log_info!("  Filtering result: RMS {:.3} ({:.1} dB)", 
                 filtered_rms, dsp.linear_to_db(filtered_rms / original_rms));
        
        // Combine multiple effects
        log_info!("Applying effect chain...");
        let mut chained_samples = samples.clone();
        dsp.apply_adsr(&mut chained_samples, 0.01, 0.1, 0.8, 0.3);
        let chained_samples = dsp.apply_distortion(&chained_samples, 2.0, 0.3);
        let chained_samples = dsp.apply_reverb(&chained_samples, 0.5, 0.4, 0.3);
        let chained_samples = dsp.apply_compressor(&chained_samples, 0.4, 3.0, 0.01, 0.1);
        let chained_rms = dsp.calculate_rms(&chained_samples);
        log_info!("  Effect chain result: RMS {:.3} ({:.1} dB)", 
                 chained_rms, dsp.linear_to_db(chained_rms / original_rms));
        
        log_info!("Effects processing example completed");
        Ok(())
    }
}

/// 6. Spectrum Analysis Example
pub struct SpectrumAnalysisExample {
    fft_size: usize,
}

impl SpectrumAnalysisExample {
    pub fn new() -> Self {
        Self {
            fft_size: 2048,
        }
    }
}

impl AudioExample for SpectrumAnalysisExample {
    fn get_name(&self) -> &'static str {
        "Spectrum Analysis"
    }

    fn get_description(&self) -> &'static str {
        "Analyze frequency spectrum of audio signals using FFT"
    }

    fn run(&mut self, _audio_manager: &mut AudioManager, dsp: &DSP) -> Result<(), AudioError> {
        log_info!("Starting spectrum analysis example...");
        
        // Create spectrum analyzer
        let mut analyzer = SpectrumAnalyzer::new(self.fft_size);
        
        // Test with different signals
        let test_cases = vec![
            ("Pure Sine Wave", 440.0, 0.5),
            ("Two Tones", 440.0, 0.3),
            ("White Noise", 0.0, 0.1),
            ("Sweep Signal", 0.0, 0.3),
        ];
        
        for (name, freq, amplitude) in test_cases {
            log_info!("\nAnalyzing: {}", name);
            
            let mut samples = Vec::with_capacity(self.fft_size);
            
            match name {
                "Pure Sine Wave" => {
                    samples = dsp.generate_sine(freq, amplitude, self.fft_size);
                },
                "Two Tones" => {
                    let mut sine1 = dsp.generate_sine(440.0, amplitude, self.fft_size);
                    let sine2 = dsp.generate_sine(880.0, amplitude * 0.7, self.fft_size);
                    for (s1, s2) in sine1.iter_mut().zip(sine2.iter()) {
                        *s1 += *s2;
                    }
                    samples = sine1;
                },
                "White Noise" => {
                    samples = dsp.generate_white_noise(amplitude, self.fft_size);
                },
                "Sweep Signal" => {
                    for i in 0..self.fft_size {
                        let t = i as f32 / dsp.sample_rate as f32;
                        let sweep_freq = 100.0 + 2000.0 * (t / 0.1); // 100Hz to 2100Hz in 0.1s
                        let sample = amplitude * (2.0 * core::f32::consts::PI * sweep_freq * t).sin();
                        samples.push(sample);
                    }
                },
                _ => {},
            }
            
            // Analyze spectrum
            let spectrum = analyzer.analyze_spectrum(&samples);
            
            // Find peak frequency
            let mut max_magnitude = 0.0;
            let mut peak_bin = 0;
            for (i, &mag) in spectrum.iter().enumerate() {
                if mag > max_magnitude {
                    max_magnitude = mag;
                    peak_bin = i;
                }
            }
            
            let bin_width = dsp.sample_rate as f32 / self.fft_size as f32;
            let peak_frequency = peak_bin as f32 * bin_width;
            
            // Calculate frequency resolution
            let resolution = bin_width;
            
            log_info!("  Peak frequency: {:.1} Hz", peak_frequency);
            log_info!("  Frequency resolution: {:.1} Hz", resolution);
            log_info!("  Peak magnitude: {:.3}", max_magnitude);
            
            // Show spectrum profile
            let db_spectrum: Vec<f32> = spectrum.iter()
                .map(|&mag| if mag > 0.001 { 20.0 * mag.log10() } else { -60.0 })
                .collect();
            
            let low_bins: f32 = db_spectrum[0..100].iter().sum::<f32>() / 100.0;
            let mid_bins: f32 = db_spectrum[100..1000].iter().sum::<f32>() / 900.0;
            let high_bins: f32 = db_spectrum[1000..].iter().sum::<f32>() / (db_spectrum.len() - 1000) as f32;
            
            log_info!("  Average levels - Low: {:.1} dB, Mid: {:.1} dB, High: {:.1} dB", 
                     low_bins, mid_bins, high_bins);
        }
        
        log_info!("Spectrum analysis example completed");
        Ok(())
    }
}

/// 7. Synthesizer Example
pub struct SynthesizerExample {
    note_sequence: Vec<u8>,
}

impl SynthesizerExample {
    pub fn new() -> Self {
        // C major scale
        Self {
            note_sequence: vec![60, 62, 64, 65, 67, 69, 71, 72], // C4 to C5
        }
    }
}

impl AudioExample for SynthesizerExample {
    fn get_name(&self) -> &'static str {
        "Audio Synthesizer"
    }

    fn get_description(&self) -> &'static str {
        "Generate music using FM synthesis and ADSR envelopes"
    }

    fn run(&mut self, _audio_manager: &mut AudioManager, dsp: &DSP) -> Result<(), AudioError> {
        log_info!("Starting synthesizer example...");
        
        // Create synthesizer
        let mut synth = Synthesizer::new(48000);
        
        // Configure envelope (quick attack, sustain, medium release)
        synth.set_envelope(0.01, 0.2, 0.8, 0.3);
        
        log_info!("Playing C major scale...");
        
        // Play each note in the scale
        for (i, &note) in self.note_sequence.iter().enumerate() {
            let frequency = 440.0 * 2.0.powf((note as f32 - 69.0) / 12.0);
            let velocity = 0.7 + 0.2 * (i as f32 / self.note_sequence.len() as f32);
            
            log_info!("Playing note {} (MIDI {}): {:.1} Hz, velocity {:.2}", 
                     i + 1, note, frequency, velocity);
            
            // Note on
            synth.note_on(note, velocity);
            
            // Generate audio for note duration
            let note_duration = 0.5; // 500ms per note
            let sample_count = (note_duration * 48000.0) as usize;
            let samples = synth.generate_samples(sample_count);
            
            // Analyze generated note
            let rms = dsp.calculate_rms(&samples);
            let peak = dsp.calculate_peak(&samples);
            log_info!("  Generated samples: {}, RMS: {:.3}, Peak: {:.3}", 
                     samples.len(), rms, peak);
            
            // Note off
            synth.note_off(note);
            
            // Short pause between notes
            log_info!("  Note off, pausing...");
        }
        
        // Demonstrate different waveforms
        log_info("\nDemonstrating different waveforms:");
        let waveforms = vec![
            ("Sine", crate::processing::Oscillator::Single),
            ("Dual Oscillator", crate::processing::Oscillator::Dual),
        ];
        
        for (name, osc_type) in waveforms {
            synth.set_oscillator(osc_type);
            
            // Play a chord
            let chord_notes = [60, 64, 67]; // C major chord
            let chord_velocities = [0.8, 0.6, 0.6];
            
            log_info!("Playing {} chord...", name);
            for (i, (&note, &velocity)) in chord_notes.iter().zip(chord_velocities.iter()).enumerate() {
                synth.note_on(note, velocity);
                log_info!("  Note {}: MIDI {}", i + 1, note);
            }
            
            // Generate chord audio
            let chord_duration = 1.0;
            let sample_count = (chord_duration * 48000.0) as usize;
            let chord_samples = synth.generate_samples(sample_count);
            
            // Analyze chord
            let rms = dsp.calculate_rms(&chord_samples);
            let peak = dsp.calculate_peak(&chord_samples);
            log_info!("  Chord RMS: {:.3}, Peak: {:.3}", rms, peak);
            
            // Turn off all notes
            for &note in &chord_notes {
                synth.note_off(note);
            }
        }
        
        log_info!("Synthesizer example completed");
        Ok(())
    }
}

/// 8. Hotplug Example
pub struct HotplugExample {
    simulation_duration: f32,
}

impl HotplugExample {
    pub fn new() -> Self {
        Self {
            simulation_duration: 5.0,
        }
    }
}

impl AudioExample for HotplugExample {
    fn get_name(&self) -> &'static str {
        "Device Hotplug"
    }

    fn get_description(&self) -> &'static str {
        "Simulate audio device detection and hotplug events"
    }

    fn run(&mut self, _audio_manager: &mut AudioManager, _dsp: &DSP) -> Result<(), AudioError> {
        log_info!("Starting device hotplug simulation...");
        
        // Create hotplug manager
        let mut hotplug_manager = AudioHotplugManager::new();
        hotplug_manager.initialize()?;
        
        // Add event handlers
        hotplug_manager.add_event_handler(Box::new(DefaultHotplugHandler::new()));
        
        log_info!("Device monitoring started...");
        
        // Simulate device detection over time
        let start_time = _dsp.sample_rate; // Using sample rate as time base
        
        for second in 0..(self.simulation_duration as u32) {
            log_info!("\n--- Second {} ---", second);
            
            // Poll for devices
            hotplug_manager.poll_devices()?;
            
            // Simulate specific events
            match second {
                1 => {
                    log_info!("Simulating USB headset connection...");
                    // This would be triggered by actual USB events
                },
                2 => {
                    log_info!("Simulating Bluetooth speaker pairing...");
                    // This would be triggered by actual Bluetooth events
                },
                3 => {
                    log_info!("Simulating audio interface detection...");
                    // This would be triggered by actual device events
                },
                4 => {
                    log_info!("Getting device list...");
                    let devices = hotplug_manager.list_devices();
                    log_info!("Current devices: {}", devices.len());
                    
                    for device in &devices {
                        log_info!("  Device: {} (ID: {})", device.name, device.id);
                        log_info!("    Type: {:?}", device.device_type);
                        log_info!("    Capabilities: {} playback channels, {} recording channels", 
                                 device.capabilities.playback_channels, 
                                 device.capabilities.recording_channels);
                        log_info!("    Sample rates: {:?}", device.capabilities.sample_rates);
                    }
                },
                _ => {},
            }
        }
        
        // Show final statistics
        let stats = hotplug_manager.get_statistics();
        log_info!("\nHotplug Statistics:");
        log_info!("  Total devices: {}", stats.total_devices);
        log_info!("  Active devices: {}", stats.active_devices);
        log_info!("  Polling enabled: {}", stats.polling_enabled);
        
        log_info!("Device hotplug example completed");
        Ok(())
    }
}

// Logging function
fn log_info(msg: &str) {
    println!("[AUDIO TUTORIAL] {}", msg);
}