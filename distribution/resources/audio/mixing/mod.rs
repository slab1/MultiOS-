//! Audio Mixing and Routing Engine
//! 
//! This module provides audio mixing, routing, and effects processing
//! for multiple simultaneous audio streams in MultiOS.

use crate::core::{AudioFormat, AudioError, StreamConfig, StreamState};
use crate::hal::{AudioBuffer};
use alloc::vec::Vec;
use alloc::collections::BTreeMap;

/// Audio stream reference for mixing
pub struct AudioStream {
    pub id: u32,
    pub state: StreamState,
    pub config: StreamConfig,
    pub buffer: AudioBuffer,
    pub volume: f32,
    pub pan: f32,
    pub muted: bool,
    pub priority: u8,
    pub effects: Vec<Box<dyn AudioEffect>>,
}

/// Audio effect trait for processing streams
pub trait AudioEffect {
    /// Process audio data
    fn process(&mut self, input: &[f32], output: &mut [f32]) -> Result<(), AudioError>;
    
    /// Update effect parameters
    fn set_parameter(&mut self, param: &str, value: f32) -> Result<(), AudioError>;
    
    /// Get effect information
    fn get_info(&self) -> EffectInfo;
}

/// Effect information structure
#[derive(Debug, Clone)]
pub struct EffectInfo {
    pub name: &'static str,
    pub version: &'static str,
    pub parameters: Vec<EffectParameter>,
}

/// Effect parameter definition
#[derive(Debug, Clone)]
pub struct EffectParameter {
    pub name: &'static str,
    pub min_value: f32,
    pub max_value: f32,
    pub default_value: f32,
    pub current_value: f32,
}

/// Volume control effect
pub struct VolumeEffect {
    gain: f32,
    info: EffectInfo,
}

impl VolumeEffect {
    pub fn new() -> Self {
        Self {
            gain: 1.0,
            info: EffectInfo {
                name: "Volume Control",
                version: "1.0",
                parameters: vec![
                    EffectParameter {
                        name: "Gain",
                        min_value: 0.0,
                        max_value: 2.0,
                        default_value: 1.0,
                        current_value: 1.0,
                    }
                ],
            },
        }
    }
}

impl AudioEffect for VolumeEffect {
    fn process(&mut self, input: &[f32], output: &mut [f32]) -> Result<(), AudioError> {
        let len = input.len().min(output.len());
        for i in 0..len {
            output[i] = input[i] * self.gain;
        }
        Ok(())
    }

    fn set_parameter(&mut self, param: &str, value: f32) -> Result<(), AudioError> {
        if param == "Gain" {
            self.gain = value.clamp(0.0, 2.0);
            self.info.parameters[0].current_value = self.gain;
            Ok(())
        } else {
            Err(AudioError::InvalidState)
        }
    }

    fn get_info(&self) -> EffectInfo {
        self.info.clone()
    }
}

/// Pan control effect
pub struct PanEffect {
    pan: f32,
    info: EffectInfo,
}

impl PanEffect {
    pub fn new() -> Self {
        Self {
            pan: 0.0, // Center
            info: EffectInfo {
                name: "Pan Control",
                version: "1.0",
                parameters: vec![
                    EffectParameter {
                        name: "Pan",
                        min_value: -1.0,
                        max_value: 1.0,
                        default_value: 0.0,
                        current_value: 0.0,
                    }
                ],
            },
        }
    }

    /// Calculate left and right channel gains
    fn calculate_gains(&self) -> (f32, f32) {
        if self.pan <= 0.0 {
            (1.0, 1.0 + self.pan)
        } else {
            (1.0 - self.pan, 1.0)
        }
    }
}

impl AudioEffect for PanEffect {
    fn process(&mut self, input: &[f32], output: &mut [f32]) -> Result<(), AudioError> {
        if input.len() < 2 {
            return Err(AudioError::InvalidState);
        }

        let (left_gain, right_gain) = self.calculate_gains();
        
        // Assuming stereo input: [L, R, L, R, ...]
        for chunk in input.chunks(2) {
            if chunk.len() == 2 {
                // Left channel (even indices)
                output[chunk[0] as usize] = chunk[0] * left_gain;
                // Right channel (odd indices) 
                output[chunk[1] as usize] = chunk[1] * right_gain;
            }
        }
        
        Ok(())
    }

    fn set_parameter(&mut self, param: &str, value: f32) -> Result<(), AudioError> {
        if param == "Pan" {
            self.pan = value.clamp(-1.0, 1.0);
            self.info.parameters[0].current_value = self.pan;
            Ok(())
        } else {
            Err(AudioError::InvalidState)
        }
    }

    fn get_info(&self) -> EffectInfo {
        self.info.clone()
    }
}

/// Simple low-pass filter effect
pub struct LowPassFilter {
    cutoff_frequency: f32,
    resonance: f32,
    sample_rate: f32,
    // Filter state variables
    prev_input: f32,
    prev_output: f32,
    info: EffectInfo,
}

impl LowPassFilter {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            cutoff_frequency: sample_rate / 4.0, // Default to quarter sample rate
            resonance: 0.7,
            sample_rate,
            prev_input: 0.0,
            prev_output: 0.0,
            info: EffectInfo {
                name: "Low-Pass Filter",
                version: "1.0",
                parameters: vec![
                    EffectParameter {
                        name: "Cutoff",
                        min_value: 20.0,
                        max_value: sample_rate / 2.0,
                        default_value: sample_rate / 4.0,
                        current_value: sample_rate / 4.0,
                    },
                    EffectParameter {
                        name: "Resonance",
                        min_value: 0.1,
                        max_value: 1.0,
                        default_value: 0.7,
                        current_value: 0.7,
                    }
                ],
            },
        }
    }

    /// Calculate filter coefficient
    fn calculate_coefficient(&self) -> f32 {
        let omega = 2.0 * core::f32::consts::PI * self.cutoff_frequency / self.sample_rate;
        let alpha = omega.sin() / (2.0 * self.resonance);
        let b0 = (1.0 - omega.cos()) / 2.0;
        let b1 = 1.0 - omega.cos();
        let b2 = b0;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * omega.cos();
        let a2 = 1.0 - alpha;
        
        b0 / a0 // Simplified coefficient calculation
    }
}

impl AudioEffect for LowPassFilter {
    fn process(&mut self, input: &[f32], output: &mut [f32]) -> Result<(), AudioError> {
        if input.len() != output.len() {
            return Err(AudioError::InvalidState);
        }

        let coefficient = self.calculate_coefficient();
        
        for (i, &sample) in input.iter().enumerate() {
            // Simple first-order low-pass filter
            self.prev_output = self.prev_output + coefficient * (sample - self.prev_output);
            output[i] = self.prev_output;
            self.prev_input = sample;
        }
        
        Ok(())
    }

    fn set_parameter(&mut self, param: &str, value: f32) -> Result<(), AudioError> {
        match param {
            "Cutoff" => {
                self.cutoff_frequency = value.clamp(20.0, self.sample_rate / 2.0);
                self.info.parameters[0].current_value = self.cutoff_frequency;
                Ok(())
            },
            "Resonance" => {
                self.resonance = value.clamp(0.1, 1.0);
                self.info.parameters[1].current_value = self.resonance;
                Ok(())
            },
            _ => Err(AudioError::InvalidState),
        }
    }

    fn get_info(&self) -> EffectInfo {
        self.info.clone()
    }
}

/// Audio mixing engine
pub struct Mixer {
    streams: BTreeMap<u32, AudioStream>,
    master_volume: f32,
    master_mute: bool,
    sample_rate: u32,
    buffer_size: usize,
    output_buffer: AudioBuffer,
    mixing_enabled: bool,
}

impl Mixer {
    /// Create a new audio mixer
    pub fn new() -> Self {
        let sample_rate = 48000;
        let buffer_size = 1024;
        
        Self {
            streams: BTreeMap::new(),
            master_volume: 1.0,
            master_mute: false,
            sample_rate,
            buffer_size,
            output_buffer: AudioBuffer::new(buffer_size * 4).unwrap(), // 32-bit float buffer
            mixing_enabled: true,
        }
    }

    /// Initialize the mixer
    pub fn initialize(&mut self) -> Result<(), AudioError> {
        log_info!("Initializing audio mixer");
        self.output_buffer.clear();
        log_info!("Audio mixer initialized with {} Hz sample rate", self.sample_rate);
        Ok(())
    }

    /// Register a new audio stream
    pub fn register_stream(&mut self, stream_id: u32, config: &StreamConfig) -> Result<(), AudioError> {
        if config.channels > 8 {
            return Err(AudioError::InvalidChannelCount);
        }

        let stream = AudioStream {
            id: stream_id,
            state: StreamState::Idle,
            config: config.clone(),
            buffer: AudioBuffer::new(config.buffer_size)?,
            volume: 1.0,
            pan: 0.0,
            muted: false,
            priority: 5, // Default priority (1-10, 10 is highest)
            effects: Vec::new(),
        };

        self.streams.insert(stream_id, stream);
        log_info!("Registered audio stream {} with {} channels", stream_id, config.channels);
        Ok(())
    }

    /// Unregister an audio stream
    pub fn unregister_stream(&mut self, stream_id: u32) -> Result<(), AudioError> {
        if self.streams.remove(&stream_id).is_some() {
            log_info!("Unregistered audio stream {}", stream_id);
            Ok(())
        } else {
            Err(AudioError::StreamNotFound)
        }
    }

    /// Enable a stream for mixing
    pub fn enable_stream(&mut self, stream_id: u32) -> Result<(), AudioError> {
        if let Some(stream) = self.streams.get_mut(&stream_id) {
            stream.state = StreamState::Playing;
            log_info!("Enabled audio stream {}", stream_id);
            Ok(())
        } else {
            Err(AudioError::StreamNotFound)
        }
    }

    /// Disable a stream
    pub fn disable_stream(&mut self, stream_id: u32) -> Result<(), AudioError> {
        if let Some(stream) = self.streams.get_mut(&stream_id) {
            stream.state = StreamState::Stopped;
            log_info!("Disabled audio stream {}", stream_id);
            Ok(())
        } else {
            Err(AudioError::StreamNotFound)
        }
    }

    /// Pause a stream
    pub fn pause_stream(&mut self, stream_id: u32) -> Result<(), AudioError> {
        if let Some(stream) = self.streams.get_mut(&stream_id) {
            stream.state = StreamState::Paused;
            log_info!("Paused audio stream {}", stream_id);
            Ok(())
        } else {
            Err(AudioError::StreamNotFound)
        }
    }

    /// Resume a stream
    pub fn resume_stream(&mut self, stream_id: u32) -> Result<(), AudioError> {
        if let Some(stream) = self.streams.get_mut(&stream_id) {
            stream.state = StreamState::Playing;
            log_info!("Resumed audio stream {}", stream_id);
            Ok(())
        } else {
            Err(AudioError::StreamNotFound)
        }
    }

    /// Set stream volume (0.0 to 2.0)
    pub fn set_stream_volume(&mut self, stream_id: u32, volume: f32) -> Result<(), AudioError> {
        if let Some(stream) = self.streams.get_mut(&stream_id) {
            stream.volume = volume.clamp(0.0, 2.0);
            log_info!("Set stream {} volume to {}", stream_id, stream.volume);
            Ok(())
        } else {
            Err(AudioError::StreamNotFound)
        }
    }

    /// Get stream volume
    pub fn get_stream_volume(&self, stream_id: u32) -> Result<f32, AudioError> {
        if let Some(stream) = self.streams.get(&stream_id) {
            Ok(stream.volume)
        } else {
            Err(AudioError::StreamNotFound)
        }
    }

    /// Set stream pan (-1.0 left to 1.0 right)
    pub fn set_stream_pan(&mut self, stream_id: u32, pan: f32) -> Result<(), AudioError> {
        if let Some(stream) = self.streams.get_mut(&stream_id) {
            stream.pan = pan.clamp(-1.0, 1.0);
            log_info!("Set stream {} pan to {}", stream_id, stream.pan);
            Ok(())
        } else {
            Err(AudioError::StreamNotFound)
        }
    }

    /// Mute/unmute a stream
    pub fn set_stream_mute(&mut self, stream_id: u32, muted: bool) -> Result<(), AudioError> {
        if let Some(stream) = self.streams.get_mut(&stream_id) {
            stream.muted = muted;
            log_info!("Set stream {} mute to {}", stream_id, muted);
            Ok(())
        } else {
            Err(AudioError::StreamNotFound)
        }
    }

    /// Set stream priority (1-10)
    pub fn set_stream_priority(&mut self, stream_id: u32, priority: u8) -> Result<(), AudioError> {
        if let Some(stream) = self.streams.get_mut(&stream_id) {
            if priority >= 1 && priority <= 10 {
                stream.priority = priority;
                log_info!("Set stream {} priority to {}", stream_id, stream.priority);
                Ok(())
            } else {
                Err(AudioError::InvalidState)
            }
        } else {
            Err(AudioError::StreamNotFound)
        }
    }

    /// Add effect to a stream
    pub fn add_effect(&mut self, stream_id: u32, effect: Box<dyn AudioEffect>) -> Result<(), AudioError> {
        if let Some(stream) = self.streams.get_mut(&stream_id) {
            stream.effects.push(effect);
            log_info!("Added effect to stream {}", stream_id);
            Ok(())
        } else {
            Err(AudioError::StreamNotFound)
        }
    }

    /// Remove effect from a stream
    pub fn remove_effect(&mut self, stream_id: u32, effect_index: usize) -> Result<(), AudioError> {
        if let Some(stream) = self.streams.get_mut(&stream_id) {
            if effect_index < stream.effects.len() {
                stream.effects.remove(effect_index);
                log_info!("Removed effect {} from stream {}", effect_index, stream_id);
                Ok(())
            } else {
                Err(AudioError::InvalidState)
            }
        } else {
            Err(AudioError::StreamNotFound)
        }
    }

    /// Set master volume (0.0 to 2.0)
    pub fn set_master_volume(&mut self, volume: f32) {
        self.master_volume = volume.clamp(0.0, 2.0);
        log_info!("Set master volume to {}", self.master_volume);
    }

    /// Get master volume
    pub fn get_master_volume(&self) -> f32 {
        self.master_volume
    }

    /// Mute/unmute master
    pub fn set_master_mute(&mut self, muted: bool) {
        self.master_mute = muted;
        log_info!("Set master mute to {}", muted);
    }

    /// Mix all active streams
    pub fn mix_streams(&mut self) -> Result<&AudioBuffer, AudioError> {
        if !self.mixing_enabled {
            return Ok(&self.output_buffer);
        }

        // Clear output buffer
        self.output_buffer.clear();

        // Convert buffer to float slice for mixing
        let output_samples = unsafe {
            core::slice::from_raw_parts_mut(
                self.output_buffer.as_mut_slice().as_mut_ptr() as *mut f32,
                self.output_buffer.size() / 4
            )
        };

        // Mix active streams by priority
        let mut active_streams: Vec<_> = self.streams.values()
            .filter(|s| s.state == StreamState::Playing && !s.muted)
            .collect();
        
        active_streams.sort_by(|a, b| b.priority.cmp(&a.priority));

        for stream in active_streams {
            self.mix_stream(stream, output_samples)?;
        }

        // Apply master volume and check for clipping
        if self.master_mute {
            for sample in output_samples.iter_mut() {
                *sample = 0.0;
            }
        } else {
            for sample in output_samples.iter_mut() {
                *sample *= self.master_volume;
                
                // Simple clipping protection
                if *sample > 1.0 {
                    *sample = 1.0;
                } else if *sample < -1.0 {
                    *sample = -1.0;
                }
            }
        }

        Ok(&self.output_buffer)
    }

    /// Mix a single stream into the output
    fn mix_stream(&mut self, stream: &AudioStream, output: &mut [f32]) -> Result<(), AudioError> {
        // Convert stream buffer to float samples
        let stream_samples = unsafe {
            core::slice::from_raw_parts(
                stream.buffer.as_slice().as_ptr() as *const f32,
                stream.buffer.size() / 4
            )
        };

        // Apply effects to stream
        let mut processed_samples = stream_samples.to_vec();
        for effect in &mut stream.effects {
            let mut temp_buffer = vec![0.0; processed_samples.len()];
            effect.process(&processed_samples, &mut temp_buffer)?;
            processed_samples.copy_from_slice(&temp_buffer);
        }

        // Apply volume and pan
        let (left_gain, right_gain) = self.calculate_pan_gains(stream.pan);
        let volume_gain = stream.volume;

        // Mix into output (assuming stereo)
        for (i, &sample) in processed_samples.iter().enumerate() {
            if i < output.len() {
                // Simple stereo mixing: alternate channels
                if i % 2 == 0 {
                    // Left channel
                    output[i] += sample * volume_gain * left_gain;
                } else {
                    // Right channel
                    output[i] += sample * volume_gain * right_gain;
                }
            }
        }

        Ok(())
    }

    /// Calculate pan gains for left and right channels
    fn calculate_pan_gains(&self, pan: f32) -> (f32, f32) {
        if pan <= 0.0 {
            (1.0, 1.0 + pan)
        } else {
            (1.0 - pan, 1.0)
        }
    }

    /// Get mixer information
    pub fn get_info(&self) -> MixerInfo {
        MixerInfo {
            active_streams: self.streams.values()
                .filter(|s| s.state == StreamState::Playing)
                .count(),
            total_streams: self.streams.len(),
            master_volume: self.master_volume,
            master_mute: self.master_mute,
            sample_rate: self.sample_rate,
            buffer_size: self.buffer_size,
            mixing_enabled: self.mixing_enabled,
        }
    }

    /// Enable/disable mixing
    pub fn set_mixing_enabled(&mut self, enabled: bool) {
        self.mixing_enabled = enabled;
        log_info!("Mixing {}", if enabled { "enabled" } else { "disabled" });
    }
}

/// Mixer information structure
#[derive(Debug, Clone)]
pub struct MixerInfo {
    pub active_streams: usize,
    pub total_streams: usize,
    pub master_volume: f32,
    pub master_mute: bool,
    pub sample_rate: u32,
    pub buffer_size: usize,
    pub mixing_enabled: bool,
}

/// Audio routing manager
pub struct AudioRouter {
    input_sources: BTreeMap<String, AudioStream>,
    output_destinations: BTreeMap<String, AudioStream>,
    routing_matrix: BTreeMap<(String, String), f32>, // (input, output) -> gain
}

impl AudioRouter {
    pub fn new() -> Self {
        Self {
            input_sources: BTreeMap::new(),
            output_destinations: BTreeMap::new(),
            routing_matrix: BTreeMap::new(),
        }
    }

    /// Add input source
    pub fn add_input_source(&mut self, name: String, stream: AudioStream) {
        self.input_sources.insert(name, stream);
        log_info!("Added input source: {}", name);
    }

    /// Add output destination
    pub fn add_output_destination(&mut self, name: String, stream: AudioStream) {
        self.output_destinations.insert(name, stream);
        log_info!("Added output destination: {}", name);
    }

    /// Create audio route
    pub fn create_route(&mut self, input: &str, output: &str, gain: f32) -> Result<(), AudioError> {
        if self.input_sources.contains_key(input) && self.output_destinations.contains_key(output) {
            self.routing_matrix.insert((input.to_string(), output.to_string()), gain);
            log_info!("Created route: {} -> {} (gain: {})", input, output, gain);
            Ok(())
        } else {
            Err(AudioError::InvalidState)
        }
    }

    /// Remove audio route
    pub fn remove_route(&mut self, input: &str, output: &str) -> Result<(), AudioError> {
        if self.routing_matrix.remove(&(input.to_string(), output.to_string())).is_some() {
            log_info!("Removed route: {} -> {}", input, output);
            Ok(())
        } else {
            Err(AudioError::InvalidState)
        }
    }
}

// Logging function
fn log_info(msg: &str) {
    println!("[AUDIO MIXER] {}", msg);
}