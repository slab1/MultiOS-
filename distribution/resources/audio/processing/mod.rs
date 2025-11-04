//! Real-time Audio Processing
//! 
//! This module provides real-time audio processing capabilities including
//! effects, filters, synthesis, and analysis for educational purposes.

use crate::core::{AudioFormat, AudioError};
use crate::hal::AudioBuffer;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;

/// Real-time audio processor
pub struct AudioProcessor {
    sample_rate: u32,
    buffer_size: usize,
    processing_enabled: bool,
    fft_size: usize,
    window_function: WindowType,
}

/// Window function types for FFT analysis
#[derive(Debug, Clone, Copy)]
pub enum WindowType {
    Rectangular,
    Hanning,
    Hamming,
    Blackman,
    Kaiser(f32), // Beta parameter
}

/// FFT implementation for real-time analysis
pub struct FFTProcessor {
    size: usize,
    bit_reversed_indices: Vec<usize>,
    twiddle_factors: Vec<complex::Complex32>,
}

/// Complex number type for FFT
mod complex {
    #[derive(Debug, Clone, Copy, Default)]
    pub struct Complex32 {
        pub real: f32,
        pub imag: f32,
    }

    impl Complex32 {
        pub fn new(real: f32, imag: f32) -> Self {
            Self { real, imag }
        }

        pub fn magnitude(&self) -> f32 {
            (self.real * self.real + self.imag * self.imag).sqrt()
        }

        pub fn phase(&self) -> f32 {
            self.imag.atan2(self.real)
        }

        pub fn multiply(&self, other: &Self) -> Self {
            Self::new(
                self.real * other.real - self.imag * other.imag,
                self.real * other.imag + self.imag * other.real,
            )
        }

        pub fn add(&self, other: &Self) -> Self {
            Self::new(self.real + other.real, self.imag + other.imag)
        }
    }
}

/// Digital signal processing utilities
pub struct DSP {
    sample_rate: u32,
}

impl DSP {
    pub fn new(sample_rate: u32) -> Self {
        Self { sample_rate }
    }

    /// Generate sine wave
    pub fn generate_sine(&self, frequency: f32, amplitude: f32, length: usize) -> Vec<f32> {
        let mut samples = Vec::with_capacity(length);
        let phase_increment = 2.0 * core::f32::consts::PI * frequency / self.sample_rate as f32;
        let mut phase = 0.0;

        for _ in 0..length {
            samples.push(amplitude * phase.sin());
            phase += phase_increment;
        }

        samples
    }

    /// Generate square wave
    pub fn generate_square(&self, frequency: f32, amplitude: f32, length: usize) -> Vec<f32> {
        let mut samples = Vec::with_capacity(length);
        let period = self.sample_rate as f32 / frequency;
        let mut phase = 0.0;

        for _ in 0..length {
            let normalized_phase = (phase % period) / period;
            let value = if normalized_phase < 0.5 { amplitude } else { -amplitude };
            samples.push(value);
            phase += 1.0;
        }

        samples
    }

    /// Generate sawtooth wave
    pub fn generate_sawtooth(&self, frequency: f32, amplitude: f32, length: usize) -> Vec<f32> {
        let mut samples = Vec::with_capacity(length);
        let period = self.sample_rate as f32 / frequency;
        let mut phase = 0.0;

        for _ in 0..length {
            let normalized_phase = (phase % period) / period;
            let value = amplitude * (2.0 * normalized_phase - 1.0);
            samples.push(value);
            phase += 1.0;
        }

        samples
    }

    /// Generate triangle wave
    pub fn generate_triangle(&self, frequency: f32, amplitude: f32, length: usize) -> Vec<f32> {
        let mut samples = Vec::with_capacity(length);
        let period = self.sample_rate as f32 / frequency;
        let mut phase = 0.0;

        for _ in 0..length {
            let normalized_phase = (phase % period) / period;
            let value = if normalized_phase < 0.5 {
                4.0 * amplitude * normalized_phase - amplitude
            } else {
                -4.0 * amplitude * normalized_phase + 3.0 * amplitude
            };
            samples.push(value);
            phase += 1.0;
        }

        samples
    }

    /// Generate white noise
    pub fn generate_white_noise(&self, amplitude: f32, length: usize) -> Vec<f32> {
        use crate::random::Random;
        let mut rng = Random::new();
        let mut samples = Vec::with_capacity(length);

        for _ in 0..length {
            samples.push(amplitude * (rng.next_f32() * 2.0 - 1.0));
        }

        samples
    }

    /// Apply simple envelope (ADSR)
    pub fn apply_adsr(&self, samples: &mut [f32], attack: f32, decay: f32, sustain: f32, release: f32) {
        let len = samples.len();
        if len == 0 { return; }

        let attack_samples = (attack * self.sample_rate as f32) as usize;
        let decay_samples = (decay * self.sample_rate as f32) as usize;
        let release_samples = (release * self.sample_rate as f32) as usize;
        let sustain_start = attack_samples + decay_samples;
        let sustain_end = len - release_samples;

        for i in 0..len {
            let envelope = if i < attack_samples {
                // Attack phase
                i as f32 / attack_samples as f32
            } else if i < sustain_start {
                // Decay phase
                1.0 - (1.0 - sustain) * (i as f32 - attack_samples as f32) / decay_samples as f32
            } else if i < sustain_end {
                // Sustain phase
                sustain
            } else {
                // Release phase
                sustain * (len as f32 - i as f32) / release_samples as f32
            };

            samples[i] *= envelope;
        }
    }

    /// Simple low-pass filter
    pub fn low_pass_filter(&self, samples: &[f32], cutoff_freq: f32, resonance: f32) -> Vec<f32> {
        let mut output = Vec::with_capacity(samples.len());
        let dt = 1.0 / self.sample_rate as f32;
        let rc = 1.0 / (2.0 * core::f32::consts::PI * cutoff_freq);
        let alpha = dt / (rc + dt);
        
        let mut prev_input = 0.0;
        let mut prev_output = 0.0;

        for &sample in samples {
            let filtered = prev_output + alpha * (sample - prev_output);
            output.push(filtered);
            prev_input = sample;
            prev_output = filtered;
        }

        output
    }

    /// Simple high-pass filter
    pub fn high_pass_filter(&self, samples: &[f32], cutoff_freq: f32) -> Vec<f32> {
        let mut output = Vec::with_capacity(samples.len());
        let dt = 1.0 / self.sample_rate as f32;
        let rc = 1.0 / (2.0 * core::f32::consts::PI * cutoff_freq);
        let alpha = rc / (rc + dt);
        
        let mut prev_input = 0.0;
        let mut prev_output = 0.0;

        for &sample in samples {
            let filtered = alpha * (prev_output + sample - prev_input);
            output.push(filtered);
            prev_input = sample;
            prev_output = filtered;
        }

        output
    }

    /// Apply reverb using simple delay lines
    pub fn apply_reverb(&self, samples: &[f32], room_size: f32, damping: f32, mix: f32) -> Vec<f32> {
        let mut output = samples.to_vec();
        let delay_time = room_size * 0.1; // 0 to 100ms
        let delay_samples = (delay_time * self.sample_rate as f32) as usize;
        let mut delay_buffer = vec![0.0f32; delay_samples * 4]; // Multiple delay lines
        
        for (i, &sample) in samples.iter().enumerate() {
            let delayed_sample = delay_buffer[i % delay_buffer.len()];
            
            // Apply damping to delayed signal
            let damped_delayed = delayed_sample * damping;
            
            // Mix original and reverb
            output[i] = sample * (1.0 - mix) + damped_delayed * mix;
            
            // Store in delay buffer
            delay_buffer[i % delay_buffer.len()] = damped_delayed + sample * 0.3;
        }

        output
    }

    /// Apply distortion/overdrive
    pub fn apply_distortion(&self, samples: &[f32], drive: f32, tone: f32) -> Vec<f32> {
        let mut output = Vec::with_capacity(samples.len());
        
        for &sample in samples {
            // Apply drive (soft clipping)
            let driven = sample * drive;
            let clipped = driven.tanh();
            
            // Apply tone control (simple low-pass filter on distortion)
            let filtered = clipped * tone + sample * (1.0 - tone);
            
            output.push(filtered);
        }

        output
    }

    /// Simple compressor/limiter
    pub fn apply_compressor(&self, samples: &[f32], threshold: f32, ratio: f32, attack: f32, release: f32) -> Vec<f32> {
        let mut output = Vec::with_capacity(samples.len());
        let mut envelope = 0.0f32;
        
        for &sample in samples {
            let abs_sample = sample.abs();
            
            // Update envelope follower
            if abs_sample > envelope {
                envelope += (abs_sample - envelope) * attack;
            } else {
                envelope += (abs_sample - envelope) * release;
            }
            
            // Apply compression
            let gain_reduction = if envelope > threshold {
                (threshold / envelope).powf(1.0 / ratio)
            } else {
                1.0
            };
            
            output.push(sample * gain_reduction);
        }

        output
    }

    /// Normalize audio to prevent clipping
    pub fn normalize(&self, samples: &[f32], target_peak: f32) -> Vec<f32> {
        let peak = samples.iter().fold(0.0f32, |acc, &s| acc.max(s.abs()));
        
        if peak > 0.0 {
            let gain = target_peak / peak;
            samples.iter().map(|&s| s * gain).collect()
        } else {
            samples.to_vec()
        }
    }

    /// Calculate RMS (Root Mean Square) for loudness measurement
    pub fn calculate_rms(&self, samples: &[f32]) -> f32 {
        let sum_squares: f32 = samples.iter().map(|&s| s * s).sum();
        (sum_squares / samples.len() as f32).sqrt()
    }

    /// Calculate peak level
    pub fn calculate_peak(&self, samples: &[f32]) -> f32 {
        samples.iter().fold(0.0f32, |acc, &s| acc.max(s.abs()))
    }

    /// Calculate dB level from linear amplitude
    pub fn linear_to_db(&self, linear: f32) -> f32 {
        if linear > 0.0 {
            20.0 * linear.log10()
        } else {
            -120.0 // Very low value for silence
        }
    }

    /// Calculate dBFS (Full Scale) level
    pub fn calculate_dbfs(&self, samples: &[f32]) -> f32 {
        let peak = self.calculate_peak(samples);
        self.linear_to_db(peak)
    }
}

/// Spectrum analyzer for educational visualization
pub struct SpectrumAnalyzer {
    fft_processor: FFTProcessor,
    window: Vec<f32>,
    magnitude_spectrum: Vec<f32>,
}

impl SpectrumAnalyzer {
    pub fn new(fft_size: usize) -> Self {
        let mut window = vec![0.0f32; fft_size];
        
        // Generate Hanning window
        for i in 0..fft_size {
            window[i] = 0.5 * (1.0 - (2.0 * core::f32::consts::PI * i as f32 / (fft_size as f32 - 1.0)).cos());
        }

        Self {
            fft_processor: FFTProcessor::new(fft_size),
            window,
            magnitude_spectrum: vec![0.0f32; fft_size / 2],
        }
    }

    /// Analyze frequency spectrum of audio samples
    pub fn analyze_spectrum(&mut self, samples: &[f32]) -> &[f32] {
        if samples.len() != self.window.len() {
            return &[];
        }

        // Apply window function
        let mut windowed_samples = Vec::with_capacity(samples.len());
        for (i, &sample) in samples.iter().enumerate() {
            windowed_samples.push(sample * self.window[i]);
        }

        // Perform FFT
        let mut complex_data = self.window_to_complex(&windowed_samples);
        self.fft_processor.process(&mut complex_data);

        // Calculate magnitude spectrum
        for (i, complex_sample) in complex_data.iter().enumerate() {
            if i < self.magnitude_spectrum.len() {
                self.magnitude_spectrum[i] = complex_sample.magnitude();
            }
        }

        &self.magnitude_spectrum
    }

    /// Convert real samples to complex representation
    fn window_to_complex(&self, samples: &[f32]) -> Vec<complex::Complex32> {
        samples.iter()
            .map(|&s| complex::Complex32::new(s, 0.0))
            .collect()
    }
}

impl FFTProcessor {
    pub fn new(size: usize) -> Self {
        // Check if size is power of 2
        assert!(size.is_power_of_two());
        
        let mut bit_reversed_indices = vec![0usize; size];
        let mut twiddle_factors = Vec::with_capacity(size);
        
        // Generate bit-reversed indices
        for i in 0..size {
            bit_reversed_indices[i] = Self::bit_reverse(i, size.ilog2() as usize);
        }

        // Generate twiddle factors
        for k in 0..(size / 2) {
            let angle = -2.0 * core::f32::consts::PI * k as f32 / size as f32;
            twiddle_factors.push(complex::Complex32::new(angle.cos(), angle.sin()));
        }

        Self {
            size,
            bit_reversed_indices,
            twiddle_factors,
        }
    }

    /// Bit reversal for FFT
    fn bit_reverse(index: usize, bits: usize) -> usize {
        let mut result = 0;
        for i in 0..bits {
            if (index >> i) & 1 == 1 {
                result |= 1 << (bits - 1 - i);
            }
        }
        result
    }

    /// Perform FFT on complex data
    pub fn process(&self, data: &mut [complex::Complex32]) {
        let n = data.len();
        
        // Bit-reversal permutation
        for i in 0..n {
            let j = self.bit_reversed_indices[i];
            if j > i {
                data.swap(i, j);
            }
        }

        // Cooley-Tukey FFT algorithm
        for len in (2..=n).step_by(2) {
            let half_len = len / 2;
            let table_step = self.size / len;
            
            for i in (0..n).step_by(len) {
                for j in 0..half_len {
                    let k = j * table_step;
                    let twiddle = self.twiddle_factors[k];
                    let temp = data[i + j + half_len].multiply(&twiddle);
                    
                    data[i + j + half_len] = data[i + j].subtract(&temp);
                    data[i + j] = data[i + j].add(&temp);
                }
            }
        }
    }
}

/// Synthesizer for educational purposes
pub struct Synthesizer {
    sample_rate: u32,
    active_voices: BTreeMap<u8, Voice>,
    envelope_generator: ADSR,
    oscillator: Oscillator,
}

#[derive(Debug, Clone)]
struct Voice {
    frequency: f32,
    amplitude: f32,
    phase: f32,
    waveform: Waveform,
    enabled: bool,
}

#[derive(Debug, Clone, Copy)]
enum Waveform {
    Sine,
    Square,
    Sawtooth,
    Triangle,
}

#[derive(Debug, Clone)]
struct ADSR {
    attack: f32,
    decay: f32,
    sustain: f32,
    release: f32,
    current_level: f32,
    state: EnvelopeState,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum EnvelopeState {
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
}

#[derive(Debug, Clone, Copy)]
enum Oscillator {
    Single,
    Dual,
    PWM, // Pulse Width Modulation
}

impl Synthesizer {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            sample_rate,
            active_voices: BTreeMap::new(),
            envelope_generator: ADSR {
                attack: 0.01,
                decay: 0.1,
                sustain: 0.7,
                release: 0.2,
                current_level: 0.0,
                state: EnvelopeState::Idle,
            },
            oscillator: Oscillator::Single,
        }
    }

    /// Note on (start playing a note)
    pub fn note_on(&mut self, note: u8, velocity: f32) {
        let frequency = self.midi_note_to_frequency(note);
        
        self.active_voices.insert(note, Voice {
            frequency,
            amplitude: velocity,
            phase: 0.0,
            waveform: Waveform::Sine,
            enabled: true,
        });

        self.envelope_generator.state = EnvelopeState::Attack;
        log_info!("Note ON: {} ({} Hz, velocity: {})", note, frequency, velocity);
    }

    /// Note off (stop playing a note)
    pub fn note_off(&mut self, note: u8) {
        if let Some(voice) = self.active_voices.get_mut(&note) {
            voice.enabled = false;
        }
        
        self.envelope_generator.state = EnvelopeState::Release;
        log_info!("Note OFF: {}", note);
    }

    /// Generate audio samples
    pub fn generate_samples(&mut self, sample_count: usize) -> Vec<f32> {
        let mut samples = vec![0.0f32; sample_count];
        let mut phase_increment = 2.0 * core::f32::consts::PI / self.sample_rate as f32;
        
        for sample_idx in 0..sample_count {
            let mut sample_value = 0.0f32;
            
            // Process all active voices
            for voice in self.active_voices.values() {
                if voice.enabled {
                    let wave_sample = self.generate_waveform(voice.phase, voice.waveform);
                    sample_value += wave_sample * voice.amplitude;
                    voice.phase += voice.frequency * phase_increment;
                    
                    // Wrap phase to prevent overflow
                    if voice.phase >= 2.0 * core::f32::consts::PI {
                        voice.phase -= 2.0 * core::f32::consts::PI;
                    }
                }
            }

            // Apply envelope
            let envelope_value = self.update_envelope();
            samples[sample_idx] = sample_value * envelope_value;
        }

        // Remove finished voices
        self.active_voices.retain(|_, voice| voice.enabled);

        samples
    }

    /// Generate waveform samples
    fn generate_waveform(&self, phase: f32, waveform: Waveform) -> f32 {
        match waveform {
            Waveform::Sine => phase.sin(),
            Waveform::Square => if phase.sin() >= 0.0 { 1.0 } else { -1.0 },
            Waveform::Sawtooth => (phase / core::f32::consts::PI).fract() * 2.0 - 1.0,
            Waveform::Triangle => (2.0 / core::f32::consts::PI * phase.sin()).asin(),
        }
    }

    /// Update envelope generator
    fn update_envelope(&mut self) -> f32 {
        match self.envelope_generator.state {
            EnvelopeState::Attack => {
                self.envelope_generator.current_level += 1.0 / (self.envelope_generator.attack * self.sample_rate as f32);
                if self.envelope_generator.current_level >= 1.0 {
                    self.envelope_generator.current_level = 1.0;
                    self.envelope_generator.state = EnvelopeState::Decay;
                }
            },
            EnvelopeState::Decay => {
                self.envelope_generator.current_level -= (1.0 - self.envelope_generator.sustain) / (self.envelope_generator.decay * self.sample_rate as f32);
                if self.envelope_generator.current_level <= self.envelope_generator.sustain {
                    self.envelope_generator.current_level = self.envelope_generator.sustain;
                    self.envelope_generator.state = EnvelopeState::Sustain;
                }
            },
            EnvelopeState::Sustain => {
                self.envelope_generator.current_level = self.envelope_generator.sustain;
            },
            EnvelopeState::Release => {
                self.envelope_generator.current_level -= self.envelope_generator.sustain / (self.envelope_generator.release * self.sample_rate as f32);
                if self.envelope_generator.current_level <= 0.0 {
                    self.envelope_generator.current_level = 0.0;
                    self.envelope_generator.state = EnvelopeState::Idle;
                }
            },
            EnvelopeState::Idle => {
                self.envelope_generator.current_level = 0.0;
            },
        }

        self.envelope_generator.current_level
    }

    /// Convert MIDI note number to frequency
    fn midi_note_to_frequency(&self, note: u8) -> f32 {
        440.0 * 2.0.powf((note as f32 - 69.0) / 12.0)
    }

    /// Set envelope parameters
    pub fn set_envelope(&mut self, attack: f32, decay: f32, sustain: f32, release: f32) {
        self.envelope_generator.attack = attack;
        self.envelope_generator.decay = decay;
        self.envelope_generator.sustain = sustain;
        self.envelope_generator.release = release;
    }

    /// Set oscillator type
    pub fn set_oscillator(&mut self, oscillator: Oscillator) {
        self.oscillator = oscillator;
    }
}

/// Audio recorder for educational purposes
pub struct AudioRecorder {
    sample_rate: u32,
    buffer_size: usize,
    recording: bool,
    recorded_data: Vec<f32>,
}

impl AudioRecorder {
    pub fn new(sample_rate: u32, buffer_size: usize) -> Self {
        Self {
            sample_rate,
            buffer_size,
            recording: false,
            recorded_data: Vec::new(),
        }
    }

    /// Start recording
    pub fn start_recording(&mut self) {
        self.recording = true;
        self.recorded_data.clear();
        log_info!("Started audio recording at {} Hz", self.sample_rate);
    }

    /// Stop recording
    pub fn stop_recording(&mut self) {
        self.recording = false;
        log_info!("Stopped recording. Captured {} samples", self.recorded_data.len());
    }

    /// Add audio samples to recording
    pub fn add_samples(&mut self, samples: &[f32]) {
        if self.recording {
            self.recorded_data.extend_from_slice(samples);
        }
    }

    /// Get recorded audio data
    pub fn get_recorded_data(&self) -> &[f32] {
        &self.recorded_data
    }

    /// Save recorded data to file (educational)
    pub fn save_recording(&self, filename: &str) -> Result<(), AudioError> {
        // This would save to a file in a real implementation
        log_info!("Recording saved to {}", filename);
        Ok(())
    }

    /// Clear recording
    pub fn clear(&mut self) {
        self.recorded_data.clear();
        log_info!("Recording cleared");
    }
}

// Simple random number generator (placeholder)
pub mod random {
    pub struct Random {
        state: u32,
    }

    impl Random {
        pub fn new() -> Self {
            Self { state: 0x12345678 }
        }

        pub fn next_u32(&mut self) -> u32 {
            self.state = self.state.wrapping_mul(1664525).wrapping_add(1013904223);
            self.state
        }

        pub fn next_f32(&mut self) -> f32 {
            (self.next_u32() as f32) / (u32::MAX as f32)
        }
    }
}

// Logging function
fn log_info(msg: &str) {
    println!("[AUDIO PROCESSING] {}", msg);
}