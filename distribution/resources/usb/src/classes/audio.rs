//! USB Audio Class Driver
//! 
//! Supports USB audio devices like speakers, microphones, audio interfaces, and headphones.
//! Implements USB Audio Class specifications for digital audio streaming and control.

use crate::*;

#[cfg(feature = "std")]
use std::collections::BTreeMap;

/// Audio Class Version
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioClassVersion {
    Version1 = 0x0100,    // USB 1.0 Audio
    Version2 = 0x0200,    // USB 2.0 Audio (deprecated)
    Version3 = 0x0300,    // USB 3.0 Audio
    Unknown,
}

/// Audio Data Format Types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioDataFormat {
    Undefined = 0x00,
    PCM = 0x01,
    PCM8 = 0x02,
    IEEEFloat = 0x03,
    ALaw = 0x04,
    MULaw = 0x05,
    Digital = 0x06,
    Format3 = 0x07,
    FormatType4 = 0x08,
    FormatType5 = 0x09,
    MPEG1Layer2 = 0x0A,
    MPEG1Layer3 = 0x0B,
    MPEG2NoXs = 0x0C,
    MPEG2Layer3Ls = 0x0D,
    IEC958 = 0x0E,
    IEC1936Ac3 = 0x0F,
    MPEG2AAC = 0x10,
    MPEG2Layer2Ls = 0x11,
    MPEG2AACLs = 0x12,
    MPEG4AAC = 0x13,
    MPEG4Type = 0x14,
    MPEG4AacPlain = 0x15,
    MPEG4Type2 = 0x16,
    MPEG4Type3 = 0x17,
    MPEG4Type4 = 0x18,
    MPEG4Type5 = 0x19,
    MPEG4Type6 = 0x1A,
    MPEG4Type7 = 0x1B,
    MPEG4Type8 = 0x1C,
    MPEG4Type9 = 0x1D,
    MPEG4Type10 = 0x1E,
    MPEG4Type11 = 0x1F,
    WMA = 0x20,
    WMAPro = 0x21,
    WMALossless = 0x22,
    OggVorbis = 0x23,
    OpUserDefined = 0x24,
    Extensions = 0x25,
}

/// Audio Terminal Types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioTerminalType {
    Undefined = 0x0000,
    InputUndefined = 0x0100,
    Microphone = 0x0200,
    DesktopMicrophone = 0x0210,
    PersonalMicrophone = 0x0220,
    OmniDirectionalMicrophone = 0x0230,
    MicrophoneArray = 0x0240,
    ProcessingMicrophoneArray = 0x0250,
    OutputUndefined = 0x0300,
    Speaker = 0x0400,
    Headphones = 0x0500,
    HeadMountedDisplayAudio = 0x0600,
    DesktopSpeaker = 0x0700,
    RoomSpeaker = 0x0800,
    CommunicationSpeaker = 0x0900,
    LowFrequencyEffectsSpeaker = 0x0A00,
    HeadphoneSpeakers = 0x0B00,
    SurroundSpeakers = 0x0C00,
    Subwoofer = 0x0D00,
    HeadsetAudioInput = 0x0E00,
    HeadsetAudioOutput = 0x0F00,
    HeadsetCommunication = 0x1000,
    ConsumerAudio = 0x2000,
    StereoSpeakers = 0x3000,
    TVSpeakers = 0x3100,
    TabletSpeakers = 0x3200,
    ProAudioSpeakers = 0x3300,
    Handset = 0x4000,
    Headset = 0x4100,
    Headphone = 0x4200,
    MicrophoneJack = 0x5100,
    PhoneLine = 0x5200,
    DigitalAudioInterface = 0x5300,
    SPDIFInterface = 0x5400,
    DigitalReceiver = 0x5500,
    ModemAudio = 0x5600,
}

/// Audio Processing Types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioProcessingType {
    Undefined = 0x00,
    UpDownMix = 0x01,
    DolbyProLogic = 0x02,
    ProLogicII = 0x03,
    ProLogicIIx = 0x04,
    CircularReverb = 0x05,
    Reverb = 0x06,
    Chorus = 0x07,
    ComplexResolver = 0x08,
    Enhancer = 0x09,
    Compressor = 0x0A,
    NoiseGater = 0x0B,
    AutomaticGain = 0x0C,
    DynamicRangeCompressor = 0x0D,
    ThreeDimensionalExpander = 0x0E,
    Preamp = 0x0F,
    MultiBandComp = 0x10,
    Download = 0x11,
    PeakLimiter = 0x12,
    Vocoder = 0x13,
    Speakercompensation = 0x14,
    SurroundEnhancer = 0x15,
    Centergenerator = 0x16,
    ReverbGates = 0x17,
    Armoring = 0x18,
    FiveBandComp = 0x19,
    SevenBandComp = 0x1A,
    ElevenBandComp = 0x1B,
    GraphicEq = 0x1C,
    ParametricEq = 0x1D,
    SoftLimiter = 0x1E,
}

/// Audio Feature Unit Control Selectors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioFeatureControl {
    Mute = 0x01,
    Volume = 0x02,
    Bass = 0x03,
    Mid = 0x04,
    Treble = 0x05,
    Equalizer = 0x06,
    AutomaticGain = 0x07,
    Delay = 0x08,
    BassBoost =  0x09,
    Loudness = 0x0A,
    InputGain = 0x0B,
    InputGainPad = 0x0C,
    PhaseInverter = 0x0D,
    Underflow = 0x0E,
    Overflow = 0x0F,
}

/// Audio Feature Unit Control
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct AudioFeatureControlData {
    pub request_type: u8,
    pub selector: AudioFeatureControl,
    pub channel: u8,
    pub value: i16,
    pub min_value: i16,
    pub max_value: i16,
    pub resolution: u16,
}

/// Audio Stream Format
#[derive(Debug, Clone)]
pub struct AudioStreamFormat {
    pub format_type: AudioDataFormat,
    pub channels: u8,
    pub bits_per_sample: u8,
    pub sample_rate: u32,
    pub bit_rate: u32,
    pub sync_frames: u8,
}

/// Audio Stream Interface
#[derive(Debug)]
pub struct AudioStreamInterface {
    pub interface_number: u8,
    pub alternate_setting: u8,
    pub format_count: u8,
    pub formats: Vec<AudioStreamFormat>,
    pub endpoint_address: u8,
    pub iso_endpoint_type: IsoEndpointType,
    pub delay: u8,
    pub active: bool,
}

/// ISO Endpoint Types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IsoEndpointType {
    Data = 0x00,
    Feedback = 0x01,
    Implicit = 0x02,
}

/// Audio Device Information
#[derive(Debug)]
pub struct AudioDeviceInfo {
    pub vendor_id: u16,
    pub product_id: u16,
    pub class_version: AudioClassVersion,
    pub stream_interface_count: u8,
    pub stream_interfaces: Vec<AudioStreamInterface>,
    pub terminal_types: Vec<AudioTerminalType>,
    pub processing_units: Vec<AudioProcessingType>,
    pub feature_units: Vec<u8>,
    pub endpoint_controls: Vec<u8>,
    pub sample_rate: u32,
    pub bit_depth: u8,
    pub channels: u8,
    pub sync_endpoint_address: Option<u8>,
}

/// Audio Data Block
#[derive(Debug)]
pub struct AudioDataBlock {
    pub data: Vec<u8>,
    pub timestamp: u64,
    pub sample_count: usize,
    pub channel_count: u8,
    pub sample_rate: u32,
    pub format: AudioDataFormat,
}

/// Audio Buffer
#[derive(Debug)]
pub struct AudioBuffer {
    pub data: Vec<u8>,
    pub capacity: usize,
    pub write_position: usize,
    pub read_position: usize,
    pub filled: bool,
    pub empty: bool,
}

/// Audio Stream State
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioStreamState {
    Stopped,
    Playing,
    Recording,
    Paused,
    Error,
}

/// Audio Driver
pub struct AudioDriver {
    pub device_info: AudioDeviceInfo,
    pub stream_state: AudioStreamState,
    pub current_format: Option<AudioStreamFormat>,
    pub audio_buffer: AudioBuffer,
    pub sync_endpoint_buffer: AudioBuffer,
    pub interrupt_endpoint: Option<u8>,
    pub data_endpoint_in: Option<u8>,
    pub data_endpoint_out: Option<u8>,
    pub sync_endpoint_feedback: Option<u8>,
    pub sync_endpoint_implicit: Option<u8>,
    pub sampling_frequency_locked: bool,
    pub bit_depth_locked: u8,
    pub active_interface: Option<u8>,
    pub volume_levels: Vec<i16>,
    pub mute_states: Vec<bool>,
}

/// Audio Driver Implementation
impl AudioDriver {
    /// Create a new audio driver instance
    pub fn new(device_address: u8) -> Self {
        Self {
            device_info: AudioDeviceInfo {
                vendor_id: 0,
                product_id: 0,
                class_version: AudioClassVersion::Unknown,
                stream_interface_count: 0,
                stream_interfaces: Vec::new(),
                terminal_types: Vec::new(),
                processing_units: Vec::new(),
                feature_units: Vec::new(),
                endpoint_controls: Vec::new(),
                sample_rate: 44100,
                bit_depth: 16,
                channels: 2,
                sync_endpoint_address: None,
            },
            stream_state: AudioStreamState::Stopped,
            current_format: None,
            audio_buffer: AudioBuffer {
                data: Vec::new(),
                capacity: 0,
                write_position: 0,
                read_position: 0,
                filled: false,
                empty: true,
            },
            sync_endpoint_buffer: AudioBuffer {
                data: Vec::new(),
                capacity: 0,
                write_position: 0,
                read_position: 0,
                filled: false,
                empty: true,
            },
            interrupt_endpoint: None,
            data_endpoint_in: None,
            data_endpoint_out: None,
            sync_endpoint_feedback: None,
            sync_endpoint_implicit: None,
            sampling_frequency_locked: false,
            bit_depth_locked: 16,
            active_interface: None,
            volume_levels: Vec::new(),
            mute_states: Vec::new(),
        }
    }

    /// Initialize audio device
    pub fn initialize(&mut self) -> UsbResult<()> {
        log::info!("Initializing USB audio device");

        // Discover audio stream interfaces
        self.discover_stream_interfaces()?;

        // Initialize audio buffers
        self.initialize_audio_buffers()?;

        // Configure default audio format
        self.set_default_format()?;

        log::info!("Audio device initialized successfully");
        Ok(())
    }

    /// Discover audio stream interfaces
    fn discover_stream_interfaces(&mut self) -> UsbResult<()> {
        // This would parse USB descriptors to find audio stream interfaces
        // For now, create a default stereo interface
        let default_format = AudioStreamFormat {
            format_type: AudioDataFormat::PCM,
            channels: 2,
            bits_per_sample: 16,
            sample_rate: 44100,
            bit_rate: 0,
            sync_frames: 0,
        };

        let stream_interface = AudioStreamInterface {
            interface_number: 1,
            alternate_setting: 0,
            format_count: 1,
            formats: vec![default_format.clone()],
            endpoint_address: 0x01,
            iso_endpoint_type: IsoEndpointType::Data,
            delay: 0,
            active: false,
        };

        self.device_info.stream_interfaces.push(stream_interface);
        self.device_info.stream_interface_count = 1;
        self.device_info.sample_rate = 44100;
        self.device_info.bit_depth = 16;
        self.device_info.channels = 2;

        log::info!("Discovered {} audio stream interfaces", self.device_info.stream_interface_count);
        Ok(())
    }

    /// Initialize audio buffers
    fn initialize_audio_buffers(&mut self) -> UsbResult<()> {
        // Allocate buffer for 1 second of audio at current format
        let bytes_per_second = (self.device_info.sample_rate as usize) * 
                               (self.device_info.channels as usize) * 
                               (self.device_info.bit_depth as usize / 8);
        let buffer_size = bytes_per_second;

        self.audio_buffer = AudioBuffer {
            data: vec![0u8; buffer_size],
            capacity: buffer_size,
            write_position: 0,
            read_position: 0,
            filled: false,
            empty: true,
        };

        self.sync_endpoint_buffer = AudioBuffer {
            data: vec![0u8; 256], // Small buffer for sync data
            capacity: 256,
            write_position: 0,
            read_position: 0,
            filled: false,
            empty: true,
        };

        log::info!("Audio buffer initialized: {} bytes", buffer_size);
        Ok(())
    }

    /// Set default audio format
    fn set_default_format(&mut self) -> UsbResult<()> {
        if self.device_info.stream_interfaces.is_empty() {
            return Err(UsbDriverError::UnsupportedFeature);
        }

        let default_interface = &self.device_info.stream_interfaces[0];
        if default_interface.formats.is_empty() {
            return Err(UsbDriverError::UnsupportedFeature);
        }

        self.current_format = Some(default_interface.formats[0].clone());
        log::info!("Set default format: {} Hz, {} bit, {} channels",
                  self.current_format.as_ref().unwrap().sample_rate,
                  self.current_format.as_ref().unwrap().bits_per_sample,
                  self.current_format.as_ref().unwrap().channels);
        Ok(())
    }

    /// Start audio streaming
    pub fn start_streaming(&mut self, mode: AudioStreamState) -> UsbResult<()> {
        match mode {
            AudioStreamState::Playing => self.start_playback(),
            AudioStreamState::Recording => self.start_recording(),
            _ => Err(UsbDriverError::UnsupportedFeature),
        }
    }

    /// Start audio playback
    pub fn start_playback(&mut self) -> UsbResult<()> {
        if self.current_format.is_none() {
            return Err(UsbDriverError::InvalidConfiguration);
        }

        self.stream_state = AudioStreamState::Playing;
        self.audio_buffer.empty = true;
        self.audio_buffer.filled = false;

        log::info!("Started audio playback");
        Ok(())
    }

    /// Start audio recording
    pub fn start_recording(&mut self) -> UsbResult<()> {
        if self.current_format.is_none() {
            return Err(UsbDriverError::InvalidConfiguration);
        }

        self.stream_state = AudioStreamState::Recording;
        self.audio_buffer.empty = true;
        self.audio_buffer.filled = false;

        log::info!("Started audio recording");
        Ok(())
    }

    /// Stop audio streaming
    pub fn stop_streaming(&mut self) -> UsbResult<()> {
        self.stream_state = AudioStreamState::Stopped;
        self.audio_buffer.empty = true;
        self.audio_buffer.filled = false;

        log::info!("Stopped audio streaming");
        Ok(())
    }

    /// Write audio data to buffer
    pub fn write_audio_data(&mut self, data: &[u8]) -> UsbResult<usize> {
        if self.stream_state != AudioStreamState::Playing {
            return Err(UsbDriverError::InvalidConfiguration);
        }

        let format = match &self.current_format {
            Some(f) => f,
            None => return Err(UsbDriverError::InvalidConfiguration),
        };

        // Calculate how much data can be written
        let bytes_available = self.audio_buffer.capacity - 
                              ((self.audio_buffer.write_position - self.audio_buffer.read_position + 
                                self.audio_buffer.capacity) % self.audio_buffer.capacity);

        let bytes_to_write = data.len().min(bytes_available);
        if bytes_to_write == 0 {
            return Ok(0);
        }

        // Write data in two parts if it wraps around the buffer
        if self.audio_buffer.write_position + bytes_to_write > self.audio_buffer.capacity {
            let first_part = self.audio_buffer.capacity - self.audio_buffer.write_position;
            let second_part = bytes_to_write - first_part;

            self.audio_buffer.data[self.audio_buffer.write_position..].copy_from_slice(&data[..first_part]);
            self.audio_buffer.data[..second_part].copy_from_slice(&data[first_part..]);

            self.audio_buffer.write_position = second_part;
        } else {
            self.audio_buffer.data[self.audio_buffer.write_position..self.audio_buffer.write_position + bytes_to_write]
                .copy_from_slice(&data);
            self.audio_buffer.write_position += bytes_to_write;
        }

        self.audio_buffer.empty = false;
        if self.audio_buffer.write_position == self.audio_buffer.read_position {
            self.audio_buffer.filled = true;
        }

        log::debug!("Wrote {} bytes to audio buffer", bytes_to_write);
        Ok(bytes_to_write)
    }

    /// Read audio data from buffer
    pub fn read_audio_data(&mut self, buffer: &mut [u8]) -> UsbResult<usize> {
        if self.stream_state != AudioStreamState::Recording {
            return Err(UsbDriverError::InvalidConfiguration);
        }

        // Calculate how much data is available
        let bytes_available = if self.audio_buffer.filled {
            self.audio_buffer.capacity
        } else {
            (self.audio_buffer.write_position - self.audio_buffer.read_position + 
             self.audio_buffer.capacity) % self.audio_buffer.capacity
        };

        let bytes_to_read = buffer.len().min(bytes_available);
        if bytes_to_read == 0 {
            return Ok(0);
        }

        // Read data in two parts if it wraps around the buffer
        if self.audio_buffer.read_position + bytes_to_read > self.audio_buffer.capacity {
            let first_part = self.audio_buffer.capacity - self.audio_buffer.read_position;
            let second_part = bytes_to_read - first_part;

            buffer[..first_part].copy_from_slice(&self.audio_buffer.data[self.audio_buffer.read_position..]);
            buffer[first_part..].copy_from_slice(&self.audio_buffer.data[..second_part]);

            self.audio_buffer.read_position = second_part;
        } else {
            buffer[..bytes_to_read].copy_from_slice(
                &self.audio_buffer.data[self.audio_buffer.read_position..self.audio_buffer.read_position + bytes_to_read]
            );
            self.audio_buffer.read_position += bytes_to_read;
        }

        self.audio_buffer.filled = false;
        if self.audio_buffer.read_position == self.audio_buffer.write_position {
            self.audio_buffer.empty = true;
        }

        log::debug!("Read {} bytes from audio buffer", bytes_to_read);
        Ok(bytes_to_read)
    }

    /// Set sampling frequency
    pub fn set_sampling_frequency(&mut self, frequency: u32) -> UsbResult<()> {
        let current_format = match &mut self.current_format {
            Some(f) => f,
            None => return Err(UsbDriverError::InvalidConfiguration),
        };

        // Check if the device supports this frequency
        // Implementation would send control request to set frequency
        current_format.sample_rate = frequency;
        self.device_info.sample_rate = frequency;
        self.sampling_frequency_locked = true;

        log::info!("Set sampling frequency to {} Hz", frequency);
        Ok(())
    }

    /// Set bit depth
    pub fn set_bit_depth(&mut self, bit_depth: u8) -> UsbResult<()> {
        let current_format = match &mut self.current_format {
            Some(f) => f,
            None => return Err(UsbDriverError::InvalidConfiguration),
        };

        // Validate bit depth
        if bit_depth != 8 && bit_depth != 16 && bit_depth != 24 && bit_depth != 32 {
            return Err(UsbDriverError::InvalidConfiguration);
        }

        current_format.bits_per_sample = bit_depth;
        self.device_info.bit_depth = bit_depth;
        self.bit_depth_locked = bit_depth;

        log::info!("Set bit depth to {} bits", bit_depth);
        Ok(())
    }

    /// Set channel count
    pub fn set_channels(&mut self, channels: u8) -> UsbResult<()> {
        let current_format = match &mut self.current_format {
            Some(f) => f,
            None => return Err(UsbDriverError::InvalidConfiguration),
        };

        // Validate channel count
        if channels < 1 || channels > 8 {
            return Err(UsbDriverError::InvalidConfiguration);
        }

        current_format.channels = channels;
        self.device_info.channels = channels;

        log::info!("Set channel count to {}", channels);
        Ok(())
    }

    /// Set volume for a channel
    pub fn set_volume(&mut self, channel: u8, volume_db: i16) -> UsbResult<()> {
        if channel as usize >= self.volume_levels.len() {
            // Expand volume array if needed
            while self.volume_levels.len() <= channel as usize {
                self.volume_levels.push(0);
            }
        }

        // Validate volume range (-144 dB to +12 dB typical range)
        if volume_db < -14400 || volume_db > 1200 {
            return Err(UsbDriverError::InvalidConfiguration);
        }

        // Implementation would send control request to set volume
        self.volume_levels[channel as usize] = volume_db;

        log::info!("Set volume for channel {} to {} dB", channel, volume_db);
        Ok(())
    }

    /// Get volume for a channel
    pub fn get_volume(&self, channel: u8) -> UsbResult<i16> {
        if channel as usize >= self.volume_levels.len() {
            return Err(UsbDriverError::InvalidConfiguration);
        }

        Ok(self.volume_levels[channel as usize])
    }

    /// Set mute state for a channel
    pub fn set_mute(&mut self, channel: u8, muted: bool) -> UsbResult<()> {
        if channel as usize >= self.mute_states.len() {
            // Expand mute array if needed
            while self.mute_states.len() <= channel as usize {
                self.mute_states.push(false);
            }
        }

        // Implementation would send control request to set mute
        self.mute_states[channel as usize] = muted;

        log::info!("Set mute for channel {} to {}", channel, muted);
        Ok(())
    }

    /// Get mute state for a channel
    pub fn get_mute(&self, channel: u8) -> UsbResult<bool> {
        if channel as usize >= self.mute_states.len() {
            return Ok(false); // Default to unmuted
        }

        Ok(self.mute_states[channel as usize])
    }

    /// Process implicit sync endpoint data
    pub fn process_sync_data(&mut self, sync_data: &[u8]) -> UsbResult<()> {
        // Process implicit feedback data for sample rate synchronization
        // Implementation would parse sync data and adjust sample rate if needed

        if sync_data.len() >= 4 {
            let sample_rate = ((sync_data[3] as u32) << 24) |
                            ((sync_data[2] as u32) << 16) |
                            ((sync_data[1] as u32) << 8) |
                            (sync_data[0] as u32);

            if self.sampling_frequency_locked && sample_rate != self.device_info.sample_rate {
                log::debug!("Sync endpoint reports {} Hz", sample_rate);
                // Could adjust timing based on sync feedback
            }
        }

        Ok(())
    }

    /// Get current stream format
    pub fn get_current_format(&self) -> Option<&AudioStreamFormat> {
        self.current_format.as_ref()
    }

    /// Check if streaming is active
    pub fn is_streaming(&self) -> bool {
        self.stream_state == AudioStreamState::Playing || 
        self.stream_state == AudioStreamState::Recording
    }

    /// Get audio buffer status
    pub fn get_buffer_status(&self) -> (usize, usize) {
        let bytes_used = if self.audio_buffer.filled {
            self.audio_buffer.capacity
        } else {
            (self.audio_buffer.write_position - self.audio_buffer.read_position + 
             self.audio_buffer.capacity) % self.audio_buffer.capacity
        };

        (bytes_used, self.audio_buffer.capacity)
    }

    /// Set endpoints
    pub fn set_endpoints(&mut self, data_in: Option<u8>, data_out: Option<u8>, 
                        interrupt: Option<u8>, sync_feedback: Option<u8>, sync_implicit: Option<u8>) {
        self.data_endpoint_in = data_in;
        self.data_endpoint_out = data_out;
        self.interrupt_endpoint = interrupt;
        self.sync_endpoint_feedback = sync_feedback;
        self.sync_endpoint_implicit = sync_implicit;
    }

    /// Get device information
    pub fn get_device_info(&self) -> &AudioDeviceInfo {
        &self.device_info
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_driver_creation() {
        let driver = AudioDriver::new(1);
        assert_eq!(driver.is_streaming(), false);
    }

    #[test]
    fn test_audio_stream_format_creation() {
        let format = AudioStreamFormat {
            format_type: AudioDataFormat::PCM,
            channels: 2,
            bits_per_sample: 16,
            sample_rate: 44100,
            bit_rate: 1411200,
            sync_frames: 0,
        };

        assert_eq!(format.channels, 2);
        assert_eq!(format.bits_per_sample, 16);
        assert_eq!(format.sample_rate, 44100);
    }

    #[test]
    fn test_audio_buffer_creation() {
        let buffer = AudioBuffer {
            data: vec![0u8; 1024],
            capacity: 1024,
            write_position: 0,
            read_position: 0,
            filled: false,
            empty: true,
        };

        assert_eq!(buffer.capacity, 1024);
        assert_eq!(buffer.empty, true);
        assert_eq!(buffer.filled, false);
    }

    #[test]
    fn test_audio_data_format_from_u8() {
        assert_eq!(AudioDataFormat::from_u8(0x01), AudioDataFormat::PCM);
        assert_eq!(AudioDataFormat::from_u8(0x03), AudioDataFormat::IEEEFloat);
        assert_eq!(AudioDataFormat::from_u8(0xFF), AudioDataFormat::FormatType11);
    }

    #[test]
    fn test_audio_terminal_type_from_u16() {
        assert_eq!(AudioTerminalType::from_u16(0x0200), AudioTerminalType::Microphone);
        assert_eq!(AudioTerminalType::from_u16(0x0400), AudioTerminalType::Speaker);
        assert_eq!(AudioTerminalType::from_u16(0x0000), AudioTerminalType::Undefined);
    }
}

// Add missing trait implementations
impl From<u8> for AudioDataFormat {
    fn from(value: u8) -> Self {
        match value {
            0x00 => AudioDataFormat::Undefined,
            0x01 => AudioDataFormat::PCM,
            0x02 => AudioDataFormat::PCM8,
            0x03 => AudioDataFormat::IEEEFloat,
            0x04 => AudioDataFormat::ALaw,
            0x05 => AudioDataFormat::MULaw,
            0x06 => AudioDataFormat::Digital,
            0x07 => AudioDataFormat::Format3,
            0x08 => AudioDataFormat::FormatType4,
            0x09 => AudioDataFormat::FormatType5,
            0x0A => AudioDataFormat::MPEG1Layer2,
            0x0B => AudioDataFormat::MPEG1Layer3,
            0x0C => AudioDataFormat::MPEG2NoXs,
            0x0D => AudioDataFormat::MPEG2Layer3Ls,
            0x0E => AudioDataFormat::IEC958,
            0x0F => AudioDataFormat::IEC1936Ac3,
            0x10 => AudioDataFormat::MPEG2AAC,
            0x11 => AudioDataFormat::MPEG2Layer2Ls,
            0x12 => AudioDataFormat::MPEG2AACLs,
            0x13 => AudioDataFormat::MPEG4AAC,
            0x14 => AudioDataFormat::MPEG4Type,
            0x15 => AudioDataFormat::MPEG4AacPlain,
            0x16 => AudioDataFormat::MPEG4Type2,
            0x17 => AudioDataFormat::MPEG4Type3,
            0x18 => AudioDataFormat::MPEG4Type4,
            0x19 => AudioDataFormat::MPEG4Type5,
            0x1A => AudioDataFormat::MPEG4Type6,
            0x1B => AudioDataFormat::MPEG4Type7,
            0x1C => AudioDataFormat::MPEG4Type8,
            0x1D => AudioDataFormat::MPEG4Type9,
            0x1E => AudioDataFormat::MPEG4Type10,
            0x1F => AudioDataFormat::MPEG4Type11,
            0x20 => AudioDataFormat::WMA,
            0x21 => AudioDataFormat::WMAPro,
            0x22 => AudioDataFormat::WMALossless,
            0x23 => AudioDataFormat::OggVorbis,
            0x24 => AudioDataFormat::OpUserDefined,
            0x25 => AudioDataFormat::Extensions,
            _ => AudioDataFormat::FormatType11,
        }
    }
}

impl From<u16> for AudioTerminalType {
    fn from(value: u16) -> Self {
        match value {
            0x0000 => AudioTerminalType::Undefined,
            0x0100 => AudioTerminalType::InputUndefined,
            0x0200 => AudioTerminalType::Microphone,
            0x0210 => AudioTerminalType::DesktopMicrophone,
            0x0220 => AudioTerminalType::PersonalMicrophone,
            0x0230 => AudioTerminalType::OmniDirectionalMicrophone,
            0x0240 => AudioTerminalType::MicrophoneArray,
            0x0250 => AudioTerminalType::ProcessingMicrophoneArray,
            0x0300 => AudioTerminalType::OutputUndefined,
            0x0400 => AudioTerminalType::Speaker,
            0x0500 => AudioTerminalType::Headphones,
            0x0600 => AudioTerminalType::HeadMountedDisplayAudio,
            0x0700 => AudioTerminalType::DesktopSpeaker,
            0x0800 => AudioTerminalType::RoomSpeaker,
            0x0900 => AudioTerminalType::CommunicationSpeaker,
            0x0A00 => AudioTerminalType::LowFrequencyEffectsSpeaker,
            0x0B00 => AudioTerminalType::HeadphoneSpeakers,
            0x0C00 => AudioTerminalType::SurroundSpeakers,
            0x0D00 => AudioTerminalType::Subwoofer,
            0x0E00 => AudioTerminalType::HeadsetAudioInput,
            0x0F00 => AudioTerminalType::HeadsetAudioOutput,
            0x1000 => AudioTerminalType::HeadsetCommunication,
            0x2000 => AudioTerminalType::ConsumerAudio,
            0x3000 => AudioTerminalType::StereoSpeakers,
            0x3100 => AudioTerminalType::TVSpeakers,
            0x3200 => AudioTerminalType::TabletSpeakers,
            0x3300 => AudioTerminalType::ProAudioSpeakers,
            0x4000 => AudioTerminalType::Handset,
            0x4100 => AudioTerminalType::Headset,
            0x4200 => AudioTerminalType::Headphone,
            0x5100 => AudioTerminalType::MicrophoneJack,
            0x5200 => AudioTerminalType::PhoneLine,
            0x5300 => AudioTerminalType::DigitalAudioInterface,
            0x5400 => AudioTerminalType::SPDIFInterface,
            0x5500 => AudioTerminalType::DigitalReceiver,
            0x5600 => AudioTerminalType::ModemAudio,
            _ => AudioTerminalType::Undefined,
        }
    }
}