//! Audio Subsystem Drivers
//! 
//! Support for audio devices including AC'97, HDA, and USB audio with audio processing operations.

use crate::{DeviceType, DriverResult, DriverError, Device, DeviceHandle, DeviceInfo, HardwareAddress, BusHandle, BusType, DeviceCapabilities, DeviceState, DeviceDriver};
use crate::device::DeviceCapability;
use spin::{Mutex, RwLock};
use alloc::{vec:: Vec, collections::BTreeMap};
use log::{info, warn, error};

/// Audio device types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum AudioType {
    Unknown = 0,
    Ac97 = 1,
    Hda = 2,
    UsbAudio = 3,
    PciAudio = 4,
    Bluetooth = 5,
}

/// Audio format types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum AudioFormat {
    Unknown = 0,
    Pcm8 = 1,
    Pcm16 = 2,
    Pcm24 = 3,
    Pcm32 = 4,
    Float32 = 5,
}

/// Audio channel configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum AudioChannels {
    Mono = 1,
    Stereo = 2,
    Quad = 4,
    FivePointOne = 6,
    SevenPointOne = 8,
}

/// Audio sample rate
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum AudioSampleRate {
    Unknown = 0,
    Hz8000 = 8000,
    Hz11025 = 11025,
    Hz16000 = 16000,
    Hz22050 = 22050,
    Hz44100 = 44100,
    Hz48000 = 48000,
    Hz88200 = 88200,
    Hz96000 = 96000,
    Hz192000 = 192000,
}

/// Audio stream direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum AudioDirection {
    Output = 0,
    Input = 1,
    Bidirectional = 2,
}

/// Audio format information
#[derive(Debug, Clone, Copy)]
pub struct AudioFormatInfo {
    pub format: AudioFormat,
    pub channels: AudioChannels,
    pub sample_rate: AudioSampleRate,
    pub bit_depth: u8,
    pub bytes_per_sample: u8,
    pub bytes_per_frame: u8,
    pub frames_per_second: u32,
}

/// Audio buffer for streaming
#[derive(Debug, Clone)]
pub struct AudioBuffer {
    pub data: Vec<u8>,
    pub format: AudioFormatInfo,
    pub frame_count: u32,
    pub timestamp: u64,
    pub direction: AudioDirection,
}

/// Audio stream information
#[derive(Debug, Clone)]
pub struct AudioStreamInfo {
    pub stream_id: u32,
    pub direction: AudioDirection,
    pub format: AudioFormatInfo,
    pub buffer_size: usize,
    pub period_size: usize,
    pub latency_frames: u32,
    pub is_active: bool,
}

/// Audio device capabilities
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum AudioCapability {
    None = 0,
    Playback = 1 << 0,
    Recording = 1 << 1,
    Duplex = 1 << 2,
    HardwareMixer = 1 << 3,
    VolumeControl = 1 << 4,
    MuteControl = 1 << 5,
    SampleRateConversion = 1 << 6,
    DigitalAudio = 1 << 7,
}

bitflags! {
    /// Audio capability flags
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct AudioCapabilities: u32 {
        const NONE = AudioCapability::None as u32;
        const PLAYBACK = AudioCapability::Playback as u32;
        const RECORDING = AudioCapability::Recording as u32;
        const DUPLEX = AudioCapability::Duplex as u32;
        const HARDWARE_MIXER = AudioCapability::HardwareMixer as u32;
        const VOLUME_CONTROL = AudioCapability::VolumeControl as u32;
        const MUTE_CONTROL = AudioCapability::MuteControl as u32;
        const SAMPLE_RATE_CONVERSION = AudioCapability::SampleRateConversion as u32;
        const DIGITAL_AUDIO = AudioCapability::DigitalAudio as u32;
    }
}

/// Volume level (0-100%)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VolumeLevel(pub u8);

impl VolumeLevel {
    pub const MIN: Self = Self(0);
    pub const MAX: Self = Self(100);
    
    /// Create new volume level
    pub fn new(level: u8) -> Self {
        Self(level.clamp(0, 100))
    }
    
    /// Get volume as percentage
    pub fn as_percent(&self) -> u8 {
        self.0
    }
    
    /// Get volume as hardware value (0-65535)
    pub fn as_hardware(&self) -> u16 {
        ((self.0 as u32 * 65535) / 100) as u16
    }
    
    /// Set volume from percentage
    pub fn from_percent(percent: u8) -> Self {
        Self(percent.clamp(0, 100))
    }
    
    /// Check if muted
    pub fn is_muted(&self) -> bool {
        self.0 == 0
    }
}

/// Audio device operations
pub trait AudioDevice: Send + Sync {
    /// Start audio playback/recording
    fn start(&self, stream_id: u32) -> DriverResult<()>;
    
    /// Stop audio playback/recording
    fn stop(&self, stream_id: u32) -> DriverResult<()>;
    
    /// Write audio data
    fn write(&self, stream_id: u32, data: &[u8]) -> DriverResult<usize>;
    
    /// Read audio data
    fn read(&self, stream_id: u32, data: &mut [u8]) -> DriverResult<usize>;
    
    /// Set volume
    fn set_volume(&self, channel: u8, volume: VolumeLevel) -> DriverResult<()>;
    
    /// Get volume
    fn get_volume(&self, channel: u8) -> DriverResult<VolumeLevel>;
    
    /// Mute/unmute channel
    fn set_mute(&self, channel: u8, muted: bool) -> DriverResult<()>;
    
    /// Get supported formats
    fn get_supported_formats(&self, direction: AudioDirection) -> Vec<AudioFormatInfo>;
    
    /// Get current format
    fn get_current_format(&self, stream_id: u32) -> DriverResult<AudioFormatInfo>;
    
    /// Get device information
    fn get_device_info(&self) -> DriverResult<AudioDeviceInfo>;
}

/// Audio device information
#[derive(Debug, Clone)]
pub struct AudioDeviceInfo {
    pub device_name: &'static str,
    pub audio_type: AudioType,
    pub capabilities: AudioCapabilities,
    pub input_channels: u8,
    pub output_channels: u8,
    pub sample_rates: Vec<AudioSampleRate>,
    pub formats: Vec<AudioFormat>,
    pub max_sample_rate: AudioSampleRate,
    pub max_bit_depth: u8,
    pub latency_ms: u32,
}

/// AC'97 Audio Driver
pub struct Ac97Driver {
    io_base: u64,
    codec_address: u8,
    input_buffer: Vec<u8>,
    output_buffer: Vec<u8>,
    current_format: AudioFormatInfo,
    volume_levels: Vec<u8>, // Volume for each channel
    muted: Vec<bool>,
    capabilities: AudioCapabilities,
}

impl Ac97Driver {
    /// Create new AC'97 driver
    pub fn new(io_base: u64, codec_address: u8) -> Self {
        let default_format = AudioFormatInfo {
            format: AudioFormat::Pcm16,
            channels: AudioChannels::Stereo,
            sample_rate: AudioSampleRate::Hz48000,
            bit_depth: 16,
            bytes_per_sample: 4, // 2 channels * 2 bytes per sample
            bytes_per_frame: 4,
            frames_per_second: 48000,
        };
        
        Self {
            io_base,
            codec_address,
            input_buffer: vec![0u8; 16384],
            output_buffer: vec![0u8; 16384],
            current_format: default_format,
            volume_levels: vec![50, 50], // Default 50% volume for stereo
            muted: vec![false, false],
            capabilities: AudioCapabilities::PLAYBACK | 
                         AudioCapabilities::RECORDING | 
                         AudioCapabilities::VOLUME_CONTROL | 
                         AudioCapabilities::MUTE_CONTROL,
        }
    }
    
    /// Initialize AC'97 codec
    pub fn init(&mut self) -> DriverResult<()> {
        info!("Initializing AC'97 audio codec");
        
        // Reset codec
        self.reset_codec()?;
        
        // Enable power controls
        self.enable_power_controls()?;
        
        // Configure default format
        self.configure_format(self.current_format)?;
        
        // Set default volumes
        self.set_default_volumes()?;
        
        info!("AC'97 codec initialized");
        Ok(())
    }
    
    /// Reset AC'97 codec
    fn reset_codec(&self) -> DriverResult<()> {
        info!("Resetting AC'97 codec");
        
        // Send reset command
        // Wait for reset completion
        // Verify codec is ready
        
        Ok(())
    }
    
    /// Enable power controls
    fn enable_power_controls(&self) -> DriverResult<()> {
        info!("Enabling AC'97 power controls");
        
        // Enable VREF and power on speakers
        Ok(())
    }
    
    /// Configure audio format
    fn configure_format(&self, format: AudioFormatInfo) -> DriverResult<()> {
        info!("Configuring AC'97 format: {} channels, {} Hz, {} bit", 
              format.channels as u8, format.sample_rate as u32, format.bit_depth);
        
        // Set sample rate
        self.set_sample_rate(format.sample_rate)?;
        
        // Set format (stereo, 16-bit)
        self.set_format_bits(format.format)?;
        
        // Set number of channels
        self.set_channels(format.channels)?;
        
        Ok(())
    }
    
    /// Set sample rate
    fn set_sample_rate(&self, sample_rate: AudioSampleRate) -> DriverResult<()> {
        info!("Setting AC'97 sample rate to {} Hz", sample_rate as u32);
        
        // Write to sample rate register
        // Verify rate was set correctly
        
        Ok(())
    }
    
    /// Set format bits
    fn set_format_bits(&self, format: AudioFormat) -> DriverResult<()> {
        match format {
            AudioFormat::Pcm8 | AudioFormat::Pcm16 | AudioFormat::Pcm24 | AudioFormat::Pcm32 => {
                info!("Setting AC'97 PCM format: {:?}", format);
            }
            _ => return Err(DriverError::DeviceNotFound),
        }
        
        Ok(())
    }
    
    /// Set number of channels
    fn set_channels(&self, channels: AudioChannels) -> DriverResult<()> {
        match channels {
            AudioChannels::Mono | AudioChannels::Stereo => {
                info!("Setting AC'97 channels: {:?}", channels);
            }
            _ => return Err(DriverError::DeviceNotFound),
        }
        
        Ok(())
    }
    
    /// Set default volumes
    fn set_default_volumes(&self) -> DriverResult<()> {
        info!("Setting AC'97 default volumes");
        
        // Set master volume
        // Set PCM volume
        // Set microphone volume if available
        
        Ok(())
    }
    
    /// Write to AC'97 registers
    fn write_register(&self, reg: u16, value: u16) -> DriverResult<()> {
        // Write to I/O port
        let port = self.io_base + (reg as u64);
        
        unsafe {
            core::ptr::write_volatile(port as *mut u16, value);
        }
        
        Ok(())
    }
    
    /// Read from AC'97 registers
    fn read_register(&self, reg: u16) -> DriverResult<u16> {
        let port = self.io_base + (reg as u64);
        
        unsafe {
            Ok(core::ptr::read_volatile(port as *const u16))
        }
    }
    
    /// Play audio samples
    fn play_samples(&self, data: &[u8]) -> DriverResult<usize> {
        info!("Playing AC'97 audio: {} bytes", data.len());
        
        // Convert samples to appropriate format
        // Write to output buffer/DMA
        // Start playback if not already running
        
        let bytes_written = data.len().min(self.output_buffer.len());
        self.output_buffer[..bytes_written].copy_from_slice(&data[..bytes_written]);
        
        Ok(bytes_written)
    }
    
    /// Record audio samples
    fn record_samples(&self, buffer: &mut [u8]) -> DriverResult<usize> {
        info!("Recording AC'97 audio into buffer of {} bytes", buffer.len());
        
        // Start recording if not already running
        // Read from input buffer/DMA
        // Convert samples to required format
        
        let bytes_read = buffer.len().min(self.input_buffer.len());
        buffer[..bytes_read].copy_from_slice(&self.input_buffer[..bytes_read]);
        
        Ok(bytes_read)
    }
}

impl DeviceDriver for Ac97Driver {
    fn name(&self) -> &'static str {
        "AC'97 Audio Driver"
    }
    
    fn supported_devices(&self) -> &[DeviceType] {
        &[DeviceType::Audio]
    }
    
    fn init(&self, _device: &Device) -> DriverResult<()> {
        info!("Initializing AC'97 driver");
        Ok(())
    }
    
    fn remove(&self, _device: &Device) -> DriverResult<()> {
        info!("Removing AC'97 driver");
        Ok(())
    }
    
    fn read(&self, _device: &Device, _buffer: &mut [u8]) -> DriverResult<usize> {
        Ok(0)
    }
    
    fn write(&self, _device: &Device, _buffer: &[u8]) -> DriverResult<usize> {
        Ok(0)
    }
    
    fn ioctl(&self, _device: &Device, _command: u32, _data: usize) -> DriverResult<usize> {
        Ok(0)
    }
    
    fn capabilities(&self) -> DeviceCapabilities {
        DeviceCapabilities::READ | DeviceCapabilities::WRITE | DeviceCapabilities::INTERRUPT | DeviceCapabilities::DMA
    }
}

impl AudioDevice for Ac97Driver {
    fn start(&self, _stream_id: u32) -> DriverResult<()> {
        info!("Starting AC'97 audio stream");
        Ok(())
    }
    
    fn stop(&self, _stream_id: u32) -> DriverResult<()> {
        info!("Stopping AC'97 audio stream");
        Ok(())
    }
    
    fn write(&self, _stream_id: u32, data: &[u8]) -> DriverResult<usize> {
        self.play_samples(data)
    }
    
    fn read(&self, _stream_id: u32, data: &mut [u8]) -> DriverResult<usize> {
        self.record_samples(data)
    }
    
    fn set_volume(&self, channel: u8, volume: VolumeLevel) -> DriverResult<()> {
        if (channel as usize) >= self.volume_levels.len() {
            return Err(DriverError::DeviceNotFound);
        }
        
        info!("Setting AC'97 volume for channel {} to {}%", channel, volume.as_percent());
        
        self.volume_levels[channel as usize] = volume.as_percent();
        
        // Write to volume register
        let hardware_volume = volume.as_hardware();
        self.write_register(0x02, hardware_volume)?; // Master volume register
        
        Ok(())
    }
    
    fn get_volume(&self, channel: u8) -> DriverResult<VolumeLevel> {
        if (channel as usize) >= self.volume_levels.len() {
            return Err(DriverError::DeviceNotFound);
        }
        
        Ok(VolumeLevel::new(self.volume_levels[channel as usize]))
    }
    
    fn set_mute(&self, channel: u8, muted: bool) -> DriverResult<()> {
        if (channel as usize) >= self.muted.len() {
            return Err(DriverError::DeviceNotFound);
        }
        
        self.muted[channel as usize] = muted;
        
        info!("AC'97 channel {} {}", 
              if muted { "muted" } else { "unmuted" }, channel);
        
        Ok(())
    }
    
    fn get_supported_formats(&self, direction: AudioDirection) -> Vec<AudioFormatInfo> {
        let mut formats = Vec::new();
        
        // AC'97 typically supports PCM 8/16/24 bit at various sample rates
        for &sample_rate in &[AudioSampleRate::Hz8000, AudioSampleRate::Hz44100, AudioSampleRate::Hz48000] {
            for &channels in &[AudioChannels::Mono, AudioChannels::Stereo] {
                for &format in &[AudioFormat::Pcm16, AudioFormat::Pcm24] {
                    if (direction == AudioDirection::Output || direction == AudioDirection::Bidirectional) ||
                       (direction == AudioDirection::Input && matches!(format, AudioFormat::Pcm16)) {
                        let bit_depth = match format {
                            AudioFormat::Pcm8 => 8,
                            AudioFormat::Pcm16 => 16,
                            AudioFormat::Pcm24 => 24,
                            AudioFormat::Pcm32 => 32,
                            _ => 16,
                        };
                        
                        formats.push(AudioFormatInfo {
                            format,
                            channels,
                            sample_rate,
                            bit_depth,
                            bytes_per_sample: ((channels as u8 * bit_depth) / 8) as u8,
                            bytes_per_frame: ((channels as u8 * bit_depth) / 8) as u8,
                            frames_per_second: sample_rate as u32,
                        });
                    }
                }
            }
        }
        
        formats
    }
    
    fn get_current_format(&self, _stream_id: u32) -> DriverResult<AudioFormatInfo> {
        Ok(self.current_format)
    }
    
    fn get_device_info(&self) -> DriverResult<AudioDeviceInfo> {
        Ok(AudioDeviceInfo {
            device_name: "AC'97 Audio",
            audio_type: AudioType::Ac97,
            capabilities: self.capabilities,
            input_channels: 1, // AC'97 typically has mono/stereo input
            output_channels: 2, // AC'97 typically has stereo output
            sample_rates: vec![
                AudioSampleRate::Hz8000,
                AudioSampleRate::Hz11025,
                AudioSampleRate::Hz22050,
                AudioSampleRate::Hz44100,
                AudioSampleRate::Hz48000,
            ],
            formats: vec![AudioFormat::Pcm16, AudioFormat::Pcm24],
            max_sample_rate: AudioSampleRate::Hz48000,
            max_bit_depth: 24,
            latency_ms: 10, // Typical AC'97 latency
        })
    }
}

/// Intel HDA Audio Driver
pub struct HdaDriver {
    base_address: u64,
    codec_address: u8,
    widget_count: u8,
    streams: BTreeMap<u32, AudioStreamInfo>,
    current_streams: Vec<u32>,
}

impl HdaDriver {
    /// Create new HDA driver
    pub fn new(base_address: u64, codec_address: u8) -> Self {
        Self {
            base_address,
            codec_address,
            widget_count: 0,
            streams: BTreeMap::new(),
            current_streams: Vec::new(),
        }
    }
    
    /// Initialize HDA controller
    pub fn init(&mut self) -> DriverResult<()> {
        info!("Initializing Intel HDA audio controller");
        
        // Reset HDA controller
        self.reset_controller()?;
        
        // Initialize codec
        self.init_codec()?;
        
        // Discover audio widgets
        self.discover_widgets()?;
        
        // Initialize streams
        self.init_streams()?;
        
        info!("HDA controller initialized with {} widgets", self.widget_count);
        Ok(())
    }
    
    /// Reset HDA controller
    fn reset_controller(&self) -> DriverResult<()> {
        info!("Resetting HDA controller");
        
        // Perform controller reset
        // Clear all states
        // Wait for ready state
        
        Ok(())
    }
    
    /// Initialize codec
    fn init_codec(&self) -> DriverResult<()> {
        info!("Initializing HDA codec at address {}", self.codec_address);
        
        // Verify codec is present
        // Initialize codec state
        // Enable power controls
        
        Ok(())
    }
    
    /// Discover audio widgets
    fn discover_widgets(&mut self) -> DriverResult<()> {
        info!("Discovering HDA audio widgets");
        
        // Enumerate all widgets in codec
        // Identify audio output widgets (speakers)
        // Identify audio input widgets (microphone)
        // Count total widgets
        
        self.widget_count = 32; // Example: typical HDA codec has many widgets
        
        info!("Found {} HDA widgets", self.widget_count);
        Ok(())
    }
    
    /// Initialize audio streams
    fn init_streams(&mut self) -> DriverResult<()> {
        info!("Initializing HDA audio streams");
        
        // Set up playback streams
        // Set up capture streams
        // Configure stream buffers
        
        // Add example stream
        let stream_info = AudioStreamInfo {
            stream_id: 1,
            direction: AudioDirection::Output,
            format: AudioFormatInfo {
                format: AudioFormat::Pcm16,
                channels: AudioChannels::Stereo,
                sample_rate: AudioSampleRate::Hz48000,
                bit_depth: 16,
                bytes_per_sample: 4,
                bytes_per_frame: 4,
                frames_per_second: 48000,
            },
            buffer_size: 4096,
            period_size: 1024,
            latency_frames: 128,
            is_active: false,
        };
        
        self.streams.insert(1, stream_info);
        
        Ok(())
    }
    
    /// Configure stream
    fn configure_stream(&self, stream_id: u32, format: AudioFormatInfo) -> DriverResult<()> {
        info!("Configuring HDA stream {}: {} channels, {} Hz", 
              stream_id, format.channels as u8, format.sample_rate as u32);
        
        // Set stream format
        // Configure DMA pointers
        // Enable stream interrupts
        
        Ok(())
    }
    
    /// Start stream
    fn start_stream(&mut self, stream_id: u32) -> DriverResult<()> {
        if let Some(stream) = self.streams.get_mut(&stream_id) {
            stream.is_active = true;
            self.current_streams.push(stream_id);
            info!("Started HDA stream {}", stream_id);
            Ok(())
        } else {
            Err(DriverError::DeviceNotFound)
        }
    }
    
    /// Stop stream
    fn stop_stream(&mut self, stream_id: u32) -> DriverResult<()> {
        if let Some(stream) = self.streams.get_mut(&stream_id) {
            stream.is_active = false;
            self.current_streams.retain(|&id| id != stream_id);
            info!("Stopped HDA stream {}", stream_id);
            Ok(())
        } else {
            Err(DriverError::DeviceNotFound)
        }
    }
}

impl DeviceDriver for HdaDriver {
    fn name(&self) -> &'static str {
        "Intel HDA Audio Driver"
    }
    
    fn supported_devices(&self) -> &[DeviceType] {
        &[DeviceType::Audio]
    }
    
    fn init(&self, _device: &Device) -> DriverResult<()> {
        info!("Initializing HDA driver");
        Ok(())
    }
    
    fn remove(&self, _device: &Device) -> DriverResult<()> {
        info!("Removing HDA driver");
        Ok(())
    }
    
    fn read(&self, _device: &Device, _buffer: &mut [u8]) -> DriverResult<usize> {
        Ok(0)
    }
    
    fn write(&self, _device: &Device, _buffer: &[u8]) -> DriverResult<usize> {
        Ok(0)
    }
    
    fn ioctl(&self, _device: &Device, _command: u32, _data: usize) -> DriverResult<usize> {
        Ok(0)
    }
    
    fn capabilities(&self) -> DeviceCapabilities {
        DeviceCapabilities::READ | DeviceCapabilities::WRITE | DeviceCapabilities::INTERRUPT | DeviceCapabilities::DMA
    }
}

impl AudioDevice for HdaDriver {
    fn start(&self, stream_id: u32) -> DriverResult<()> {
        if let Some(stream) = self.streams.get(&stream_id) {
            self.configure_stream(stream_id, stream.format)?;
            Ok(())
        } else {
            Err(DriverError::DeviceNotFound)
        }
    }
    
    fn stop(&self, _stream_id: u32) -> DriverResult<()> {
        Ok(())
    }
    
    fn write(&self, _stream_id: u32, data: &[u8]) -> DriverResult<usize> {
        info!("HDA write: {} bytes", data.len());
        Ok(data.len())
    }
    
    fn read(&self, _stream_id: u32, _data: &mut [u8]) -> DriverResult<usize> {
        Ok(0)
    }
    
    fn set_volume(&self, _channel: u8, _volume: VolumeLevel) -> DriverResult<()> {
        info!("Setting HDA volume");
        Ok(())
    }
    
    fn get_volume(&self, _channel: u8) -> DriverResult<VolumeLevel> {
        Ok(VolumeLevel::new(50))
    }
    
    fn set_mute(&self, _channel: u8, _muted: bool) -> DriverResult<()> {
        Ok(())
    }
    
    fn get_supported_formats(&self, _direction: AudioDirection) -> Vec<AudioFormatInfo> {
        // HDA supports high-quality formats
        vec![
            AudioFormatInfo {
                format: AudioFormat::Pcm16,
                channels: AudioChannels::Stereo,
                sample_rate: AudioSampleRate::Hz48000,
                bit_depth: 16,
                bytes_per_sample: 4,
                bytes_per_frame: 4,
                frames_per_second: 48000,
            },
            AudioFormatInfo {
                format: AudioFormat::Pcm24,
                channels: AudioChannels::Stereo,
                sample_rate: AudioSampleRate::Hz48000,
                bit_depth: 24,
                bytes_per_sample: 6,
                bytes_per_frame: 6,
                frames_per_second: 48000,
            },
        ]
    }
    
    fn get_current_format(&self, stream_id: u32) -> DriverResult<AudioFormatInfo> {
        if let Some(stream) = self.streams.get(&stream_id) {
            Ok(stream.format)
        } else {
            Err(DriverError::DeviceNotFound)
        }
    }
    
    fn get_device_info(&self) -> DriverResult<AudioDeviceInfo> {
        Ok(AudioDeviceInfo {
            device_name: "Intel HDA Audio",
            audio_type: AudioType::Hda,
            capabilities: AudioCapabilities::PLAYBACK | 
                         AudioCapabilities::RECORDING | 
                         AudioCapabilities::DUPLEX | 
                         AudioCapabilities::VOLUME_CONTROL | 
                         AudioCapabilities::SAMPLE_RATE_CONVERSION,
            input_channels: 2,
            output_channels: 8, // HDA supports more channels
            sample_rates: vec![
                AudioSampleRate::Hz48000,
                AudioSampleRate::Hz96000,
                AudioSampleRate::Hz192000,
            ],
            formats: vec![AudioFormat::Pcm16, AudioFormat::Pcm24, AudioFormat::Pcm32],
            max_sample_rate: AudioSampleRate::Hz192000,
            max_bit_depth: 32,
            latency_ms: 5, // Lower latency than AC'97
        })
    }
}

/// USB Audio Driver
pub struct UsbAudioDriver {
    controller_id: u8,
    device_address: u8,
    endpoint_in: u8,
    endpoint_out: u8,
    interface_number: u8,
    is_class_compliant: bool,
}

impl UsbAudioDriver {
    /// Create new USB audio driver
    pub fn new(controller_id: u8, device_address: u8, interface_number: u8, 
               endpoint_in: u8, endpoint_out: u8) -> Self {
        Self {
            controller_id,
            device_address,
            interface_number,
            endpoint_in,
            endpoint_out,
            is_class_compliant: true,
        }
    }
    
    /// Initialize USB audio device
    pub fn init(&mut self) -> DriverResult<()> {
        info!("Initializing USB audio device");
        
        // Configure USB audio streaming interface
        self.configure_interface()?;
        
        // Set initial format
        self.set_initial_format()?;
        
        info!("USB audio device initialized");
        Ok(())
    }
    
    /// Configure USB interface
    fn configure_interface(&self) -> DriverResult<()> {
        info!("Configuring USB audio interface");
        
        // Configure alternate settings
        // Set endpoint parameters
        // Enable streaming interface
        
        Ok(())
    }
    
    /// Set initial audio format
    fn set_initial_format(&self) -> DriverResult<()> {
        info!("Setting initial USB audio format");
        
        // Set stereo, 16-bit, 44.1kHz as default
        Ok(())
    }
}

impl DeviceDriver for UsbAudioDriver {
    fn name(&self) -> &'static str {
        "USB Audio Driver"
    }
    
    fn supported_devices(&self) -> &[DeviceType] {
        &[DeviceType::Audio]
    }
    
    fn init(&self, _device: &Device) -> DriverResult<()> {
        info!("Initializing USB audio driver");
        Ok(())
    }
    
    fn remove(&self, _device: &Device) -> DriverResult<()> {
        info!("Removing USB audio driver");
        Ok(())
    }
    
    fn read(&self, _device: &Device, _buffer: &mut [u8]) -> DriverResult<usize> {
        Ok(0)
    }
    
    fn write(&self, _device: &Device, _buffer: &[u8]) -> DriverResult<usize> {
        Ok(0)
    }
    
    fn ioctl(&self, _device: &Device, _command: u32, _data: usize) -> DriverResult<usize> {
        Ok(0)
    }
    
    fn capabilities(&self) -> DeviceCapabilities {
        DeviceCapabilities::READ | DeviceCapabilities::WRITE | DeviceCapabilities::INTERRUPT | DeviceCapabilities::HOT_PLUG
    }
}

impl AudioDevice for UsbAudioDriver {
    fn start(&self, _stream_id: u32) -> DriverResult<()> {
        info!("Starting USB audio stream");
        Ok(())
    }
    
    fn stop(&self, _stream_id: u32) -> DriverResult<()> {
        info!("Stopping USB audio stream");
        Ok(())
    }
    
    fn write(&self, _stream_id: u32, data: &[u8]) -> DriverResult<usize> {
        info!("USB audio write: {} bytes", data.len());
        Ok(data.len())
    }
    
    fn read(&self, _stream_id: u32, data: &mut [u8]) -> DriverResult<usize> {
        info!("USB audio read: {} bytes", data.len());
        Ok(0)
    }
    
    fn set_volume(&self, _channel: u8, _volume: VolumeLevel) -> DriverResult<()> {
        info!("Setting USB audio volume");
        Ok(())
    }
    
    fn get_volume(&self, _channel: u8) -> DriverResult<VolumeLevel> {
        Ok(VolumeLevel::new(75))
    }
    
    fn set_mute(&self, _channel: u8, _muted: bool) -> DriverResult<()> {
        Ok(())
    }
    
    fn get_supported_formats(&self, _direction: AudioDirection) -> Vec<AudioFormatInfo> {
        vec![
            AudioFormatInfo {
                format: AudioFormat::Pcm16,
                channels: AudioChannels::Stereo,
                sample_rate: AudioSampleRate::Hz44100,
                bit_depth: 16,
                bytes_per_sample: 4,
                bytes_per_frame: 4,
                frames_per_second: 44100,
            },
        ]
    }
    
    fn get_current_format(&self, _stream_id: u32) -> DriverResult<AudioFormatInfo> {
        Ok(AudioFormatInfo {
            format: AudioFormat::Pcm16,
            channels: AudioChannels::Stereo,
            sample_rate: AudioSampleRate::Hz44100,
            bit_depth: 16,
            bytes_per_sample: 4,
            bytes_per_frame: 4,
            frames_per_second: 44100,
        })
    }
    
    fn get_device_info(&self) -> DriverResult<AudioDeviceInfo> {
        Ok(AudioDeviceInfo {
            device_name: "USB Audio Device",
            audio_type: AudioType::UsbAudio,
            capabilities: AudioCapabilities::PLAYBACK | 
                         AudioCapabilities::RECORDING | 
                         AudioCapabilities::VOLUME_CONTROL,
            input_channels: 1,
            output_channels: 2,
            sample_rates: vec![AudioSampleRate::Hz44100, AudioSampleRate::Hz48000],
            formats: vec![AudioFormat::Pcm16],
            max_sample_rate: AudioSampleRate::Hz48000,
            max_bit_depth: 16,
            latency_ms: 20, // USB has higher latency
        })
    }
}

/// Audio driver manager
pub struct AudioDriverManager {
    ac97_drivers: Vec<Ac97Driver>,
    hda_drivers: Vec<HdaDriver>,
    usb_audio_drivers: Vec<UsbAudioDriver>,
    primary_output: Option<&'static dyn AudioDevice>,
    primary_input: Option<&'static dyn AudioDevice>,
}

impl AudioDriverManager {
    /// Create new audio driver manager
    pub fn new() -> Self {
        Self {
            ac97_drivers: Vec::new(),
            hda_drivers: Vec::new(),
            usb_audio_drivers: Vec::new(),
            primary_output: None,
            primary_input: None,
        }
    }
    
    /// Register AC'97 audio driver
    pub fn register_ac97(&mut self, io_base: u64, codec_address: u8) -> DriverResult<()> {
        let mut driver = Ac97Driver::new(io_base, codec_address);
        driver.init()?;
        
        self.ac97_drivers.push(driver);
        
        if self.primary_output.is_none() {
            self.primary_output = Some(self.ac97_drivers.last().unwrap());
        }
        if self.primary_input.is_none() {
            self.primary_input = Some(self.ac97_drivers.last().unwrap());
        }
        
        info!("AC'97 audio driver registered");
        Ok(())
    }
    
    /// Register HDA audio driver
    pub fn register_hda(&mut self, base_address: u64, codec_address: u8) -> DriverResult<()> {
        let mut driver = HdaDriver::new(base_address, codec_address);
        driver.init()?;
        
        self.hda_drivers.push(driver);
        
        // HDA typically preferred over AC'97
        if self.primary_output.is_none() {
            self.primary_output = Some(self.hda_drivers.last().unwrap());
        }
        if self.primary_input.is_none() {
            self.primary_input = Some(self.hda_drivers.last().unwrap());
        }
        
        info!("HDA audio driver registered");
        Ok(())
    }
    
    /// Register USB audio driver
    pub fn register_usb_audio(&mut self, controller_id: u8, device_address: u8, 
                             interface_number: u8, endpoint_in: u8, endpoint_out: u8) -> DriverResult<()> {
        let mut driver = UsbAudioDriver::new(controller_id, device_address, interface_number, 
                                           endpoint_in, endpoint_out);
        driver.init()?;
        
        self.usb_audio_drivers.push(driver);
        
        info!("USB audio driver registered");
        Ok(())
    }
    
    /// Get primary output device
    pub fn get_primary_output(&self) -> Option<&dyn AudioDevice> {
        self.primary_output.map(|dev| *dev)
    }
    
    /// Get primary input device
    pub fn get_primary_input(&self) -> Option<&dyn AudioDevice> {
        self.primary_input.map(|dev| *dev)
    }
    
    /// Play audio buffer
    pub fn play_buffer(&self, buffer: &AudioBuffer) -> DriverResult<usize> {
        if let Some(output) = self.get_primary_output() {
            // Convert buffer to raw data
            output.write(0, &buffer.data)
        } else {
            Err(DriverError::DeviceNotFound)
        }
    }
    
    /// Record audio buffer
    pub fn record_buffer(&self, buffer: &mut AudioBuffer) -> DriverResult<usize> {
        if let Some(input) = self.get_primary_input() {
            input.read(0, &mut buffer.data)
        } else {
            Err(DriverError::DeviceNotFound)
        }
    }
    
    /// Set master volume
    pub fn set_master_volume(&self, volume: VolumeLevel) -> DriverResult<()> {
        if let Some(output) = self.get_primary_output() {
            output.set_volume(0, volume)
        } else {
            Err(DriverError::DeviceNotFound)
        }
    }
    
    /// Get master volume
    pub fn get_master_volume(&self) -> DriverResult<VolumeLevel> {
        if let Some(output) = self.get_primary_output() {
            output.get_volume(0)
        } else {
            Err(DriverError::DeviceNotFound)
        }
    }
    
    /// List all audio devices
    pub fn list_devices(&self) -> Vec<AudioDeviceInfo> {
        let mut devices = Vec::new();
        
        for ac97 in &self.ac97_drivers {
            if let Ok(info) = ac97.get_device_info() {
                devices.push(info);
            }
        }
        
        for hda in &self.hda_drivers {
            if let Ok(info) = hda.get_device_info() {
                devices.push(info);
            }
        }
        
        for usb_audio in &self.usb_audio_drivers {
            if let Ok(info) = usb_audio.get_device_info() {
                devices.push(info);
            }
        }
        
        devices
    }
    
    /// Check if audio is available
    pub fn is_audio_available(&self) -> bool {
        !self.ac97_drivers.is_empty() || 
        !self.hda_drivers.is_empty() || 
        !self.usb_audio_drivers.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_volume_level() {
        let volume = VolumeLevel::new(75);
        assert_eq!(volume.as_percent(), 75);
        assert_eq!(volume.as_hardware(), 49151);
        assert!(!volume.is_muted());
        
        let muted = VolumeLevel::new(0);
        assert!(muted.is_muted());
    }

    #[test]
    fn test_mac_address_from_invalid_string() {
        assert!(MacAddress::from_string("invalid").is_err());
        assert!(MacAddress::from_string("12:34:56:78").is_err());
    }

    #[test]
    fn test_audio_format_info() {
        let format = AudioFormatInfo {
            format: AudioFormat::Pcm16,
            channels: AudioChannels::Stereo,
            sample_rate: AudioSampleRate::Hz44100,
            bit_depth: 16,
            bytes_per_sample: 4,
            bytes_per_frame: 4,
            frames_per_second: 44100,
        };
        
        assert_eq!(format.channels as u8, 2);
        assert_eq!(format.sample_rate as u32, 44100);
    }

    #[test]
    fn test_ac97_driver_creation() {
        let driver = Ac97Driver::new(0x400, 0);
        assert!(!driver.capabilities.is_empty());
        assert_eq!(driver.capabilities, AudioCapabilities::PLAYBACK | 
                   AudioCapabilities::RECORDING | 
                   AudioCapabilities::VOLUME_CONTROL | 
                   AudioCapabilities::MUTE_CONTROL);
    }

    #[test]
    fn test_audio_driver_manager() {
        let mut manager = AudioDriverManager::new();
        
        // Register AC'97 driver
        assert!(manager.register_ac97(0x400, 0).is_ok());
        assert!(manager.is_audio_available());
        
        // Get master volume
        let volume = manager.get_master_volume();
        assert!(volume.is_ok());
        
        // List devices
        let devices = manager.list_devices();
        assert!(!devices.is_empty());
    }
}
