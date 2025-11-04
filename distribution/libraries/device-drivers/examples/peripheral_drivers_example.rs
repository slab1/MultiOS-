//! Advanced Peripheral Drivers Example
//! 
//! This example demonstrates how to use all the advanced peripheral drivers
//! including graphics, storage, network, and audio subsystems.

use crate::device_drivers::{
    self, graphics, storage, network, audio,
    DeviceType, DriverResult,
};

/// Example application demonstrating advanced peripheral driver usage
pub struct PeripheralExampleApp {
    graphics_manager: Option<graphics::GraphicsDriverManager>,
    storage_manager: Option<storage::StorageDriverManager>,
    network_manager: Option<network::NetworkDriverManager>,
    audio_manager: Option<audio::AudioDriverManager>,
}

impl PeripheralExampleApp {
    /// Create new peripheral example application
    pub fn new() -> Self {
        Self {
            graphics_manager: None,
            storage_manager: None,
            network_manager: None,
            audio_manager: None,
        }
    }
    
    /// Initialize all peripheral drivers
    pub fn init(&mut self) -> DriverResult<()> {
        info!("Initializing Advanced Peripheral Drivers Example");
        
        // Initialize device driver framework
        device_drivers::init()?;
        
        // Initialize graphics subsystem
        self.init_graphics()?;
        
        // Initialize storage subsystem
        self.init_storage()?;
        
        // Initialize network subsystem
        self.init_network()?;
        
        // Initialize audio subsystem
        self.init_audio()?;
        
        info!("All peripheral drivers initialized successfully");
        Ok(())
    }
    
    /// Initialize graphics drivers and display startup screen
    fn init_graphics(&mut self) -> DriverResult<()> {
        info!("Initializing Graphics Subsystem");
        
        let mut graphics_manager = graphics::GraphicsDriverManager::new();
        
        // Register all graphics drivers
        graphics_manager.register_vga(0x3CE, 0xA0000)?;
        graphics_manager.register_vesa(0xA0000)?;
        graphics_manager.register_uefi_gop(0xA0000, 8192 * 1024)?;
        
        // Set VESA as default for better resolution
        graphics_manager.set_current_driver(graphics::GraphicsMode::Vesa)?;
        
        // Clear screen and display startup message
        graphics_manager.clear(0x000020)?;  // Dark blue background
        graphics_manager.draw_pixel(100, 100, 0xFFFFFF)?;  // White pixel
        
        self.graphics_manager = Some(graphics_manager);
        info!("Graphics subsystem initialized");
        Ok(())
    }
    
    /// Initialize storage drivers and test disk operations
    fn init_storage(&mut self) -> DriverResult<()> {
        info!("Initializing Storage Subsystem");
        
        let mut storage_manager = storage::StorageDriverManager::new();
        
        // Register all storage controllers
        storage_manager.register_sata(0x1F0, 0x3F6)?;
        storage_manager.register_nvme(0x9000, 0x9200)?;
        storage_manager.register_usb_mass_storage(0, 1, 0x81, 0x02)?;
        
        // Get device information
        let device_info = storage_manager.get_device_info()?;
        info!("Storage device: {} ({} sectors)", 
              device_info.device_name, device_info.total_sectors);
        
        // Test read/write operations
        self.test_storage_operations(&mut storage_manager)?;
        
        self.storage_manager = Some(storage_manager);
        info!("Storage subsystem initialized");
        Ok(())
    }
    
    /// Test storage operations
    fn test_storage_operations(&self, storage: &storage::StorageDriverManager) -> DriverResult<()> {
        info!("Testing storage operations");
        
        // Create test data
        let test_data = vec![0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, 0x11, 0x22];
        let mut read_buffer = vec![0u8; test_data.len()];
        
        // Test write operation
        storage.write_sectors(0, 1, &test_data)?;
        info!("Written {} bytes to storage", test_data.len());
        
        // Test read operation
        storage.read_sectors(0, 1, &mut read_buffer)?;
        info!("Read {} bytes from storage", read_buffer.len());
        
        // Verify data integrity
        assert_eq!(test_data, read_buffer, "Storage read/write test failed");
        info!("Storage operations test passed");
        
        Ok(())
    }
    
    /// Initialize network drivers and test connectivity
    fn init_network(&mut self) -> DriverResult<()> {
        info!("Initializing Network Subsystem");
        
        let mut network_manager = network::NetworkDriverManager::new();
        
        // Register network interfaces
        let ethernet_mac = network::MacAddress::from_string("12:34:56:78:9A:BC")?;
        network_manager.register_ethernet(0x1C00, ethernet_mac)?;
        
        let wifi_mac = network::MacAddress::from_string("FE:DC:BA:98:76:54")?;
        network_manager.register_wifi(wifi_mac, network::WifiRadioType::N)?;
        
        // Test packet operations
        self.test_network_operations(&mut network_manager)?;
        
        // Test WiFi scanning and connection (simulated)
        if let Some(wifi_interface) = network_manager.wifi_interfaces.get(0) {
            let access_points = wifi_interface.scan()?;
            info!("Found {} WiFi access points", access_points.len());
            
            for ap in &access_points {
                info!("  - SSID: {}, Signal: {} dBm, Channel: {}", 
                      ap.ssid, ap.signal_strength, ap.channel);
            }
        }
        
        self.network_manager = Some(network_manager);
        info!("Network subsystem initialized");
        Ok(())
    }
    
    /// Test network packet operations
    fn test_network_operations(&self, network: &network::NetworkDriverManager) -> DriverResult<()> {
        info!("Testing network operations");
        
        // Create test packet
        let packet_data = vec![
            0x01, 0x02, 0x03, 0x04,  // Source IP (simulated)
            0x05, 0x06, 0x07, 0x08,  // Dest IP (simulated)
            0x00, 0x50,              // TCP port (simulated)
            // ... more packet data
        ];
        
        let test_packet = network::NetworkPacket {
            data: packet_data,
            length: packet_data.len(),
            packet_type: network::NetworkPacketType::IPv4,
            timestamp: 1000,
        };
        
        // Test send operation
        let bytes_sent = network.send_packet(&test_packet)?;
        info!("Sent {} bytes over network", bytes_sent);
        
        // Test receive operation
        let received_packet = network.receive_packet()?;
        match received_packet {
            Some(packet) => {
                info!("Received packet: {} bytes, type: {:?}", packet.length, packet.packet_type);
            }
            None => {
                info!("No packets received");
            }
        }
        
        info!("Network operations test completed");
        Ok(())
    }
    
    /// Initialize audio drivers and test sound playback
    fn init_audio(&mut self) -> DriverResult<()> {
        info!("Initializing Audio Subsystem");
        
        let mut audio_manager = audio::AudioDriverManager::new();
        
        // Register audio devices
        audio_manager.register_ac97(0x400, 0)?;
        audio_manager.register_hda(0x9000, 0)?;
        audio_manager.register_usb_audio(0, 2, 0, 0x81, 0x02)?;
        
        // Test audio operations
        self.test_audio_operations(&mut audio_manager)?;
        
        self.audio_manager = Some(audio_manager);
        info!("Audio subsystem initialized");
        Ok(())
    }
    
    /// Test audio operations
    fn test_audio_operations(&self, audio: &mut audio::AudioDriverManager) -> DriverResult<()> {
        info!("Testing audio operations");
        
        // Set master volume to 75%
        let volume = audio::VolumeLevel::new(75);
        audio.set_master_volume(volume)?;
        info!("Set master volume to {}%", volume.as_percent());
        
        // Create test audio buffer (1 second of silence at 44.1kHz stereo)
        let sample_rate = audio::AudioSampleRate::Hz44100 as u32;
        let frame_count = sample_rate;
        let bytes_per_frame = 4; // 2 channels * 2 bytes per sample
        
        let audio_data = vec![0u8; frame_count as usize * bytes_per_frame];
        
        let test_buffer = audio::AudioBuffer {
            data: audio_data,
            format: audio::AudioFormatInfo {
                format: audio::AudioFormat::Pcm16,
                channels: audio::AudioChannels::Stereo,
                sample_rate: audio::AudioSampleRate::Hz44100,
                bit_depth: 16,
                bytes_per_sample: 4,
                bytes_per_frame: 4,
                frames_per_second: 44100,
            },
            frame_count,
            timestamp: 2000,
            direction: audio::AudioDirection::Output,
        };
        
        // Test playback
        let bytes_played = audio.play_buffer(&test_buffer)?;
        info!("Played {} bytes of audio", bytes_played);
        
        info!("Audio operations test completed");
        Ok(())
    }
    
    /// Display system status on graphics screen
    pub fn display_system_status(&self) -> DriverResult<()> {
        if let Some(ref graphics) = self.graphics_manager {
            info!("Displaying system status on screen");
            
            // Clear screen
            graphics.clear(0x001122)?; // Dark background
            
            // Draw system status graphics
            graphics.draw_pixel(200, 50, 0x00FF00)?; // Green indicator
            graphics.draw_pixel(300, 50, 0xFF0000)?; // Red indicator
            graphics.draw_pixel(400, 50, 0xFFFF00)?; // Yellow indicator
        }
        
        Ok(())
    }
    
    /// Perform comprehensive system test
    pub fn run_comprehensive_test(&mut self) -> DriverResult<()> {
        info!("Running comprehensive peripheral drivers test");
        
        // Graphics test
        if let Some(ref graphics) = self.graphics_manager {
            info!("Graphics test: Drawing test patterns");
            graphics.clear(0x000080)?; // Blue background
            
            // Draw test patterns
            for x in 0..100 {
                for y in 0..100 {
                    let color = (x ^ y) as u32;
                    graphics.draw_pixel(100 + x, 100 + y, color)?;
                }
            }
        }
        
        // Storage test
        if let Some(ref storage) = self.storage_manager {
            info!("Storage test: Large data transfer");
            let large_buffer = vec![0u8; 1024 * 1024]; // 1MB buffer
            let mut read_buffer = vec![0u8; 1024 * 1024];
            
            // Test large write
            storage.write_sectors(0, 2048, &large_buffer)?;
            info!("Wrote 1MB to storage");
            
            // Test large read
            storage.read_sectors(0, 2048, &mut read_buffer)?;
            info!("Read 1MB from storage");
        }
        
        // Network test
        if let Some(ref network) = self.network_manager {
            info!("Network test: Packet flood test");
            for i in 0..10 {
                let packet_data = vec![0xFF; 64 + (i % 10) * 64];
                let packet = network::NetworkPacket {
                    data: packet_data,
                    length: packet_data.len(),
                    packet_type: network::NetworkPacketType::Custom,
                    timestamp: 3000 + i,
                };
                
                network.send_packet(&packet)?;
            }
            info!("Sent 10 test packets");
        }
        
        // Audio test
        if let Some(ref mut audio) = self.audio_manager {
            info!("Audio test: Multi-channel volume test");
            for channel in 0..8 {
                let volume = audio::VolumeLevel::new(25 + (channel as u8 * 10));
                if let Some(output) = audio.get_primary_output() {
                    let _ = output.set_volume(channel, volume);
                }
            }
            info!("Set volumes for 8 channels");
        }
        
        info!("Comprehensive test completed successfully");
        Ok(())
    }
    
    /// Get system information from all drivers
    pub fn get_system_info(&self) -> DriverResult<SystemInfo> {
        let mut info = SystemInfo::new();
        
        // Graphics info
        if let Some(ref graphics) = self.graphics_manager {
            if let Some(driver) = graphics.get_current_driver() {
                info.graphics_active = true;
                info.graphics_driver = "VESA/UEFI GOP".to_string();
            }
        }
        
        // Storage info
        if let Some(ref storage) = self.storage_manager {
            let device_info = storage.get_device_info()?;
            info.storage_active = true;
            info.storage_type = device_info.device_name.to_string();
            info.storage_size_mb = device_info.total_sectors * device_info.sector_size as u64 / (1024 * 1024);
        }
        
        // Network info
        if let Some(ref network) = self.network_manager {
            let interfaces = network.list_interfaces();
            info.network_active = !interfaces.is_empty();
            info.network_interfaces = interfaces.len();
            
            if let Some(interface) = interfaces.get(0) {
                info.network_type = format!("{} Mbps", interface.speed_mbps);
                info.mac_address = interface.mac_address.to_string();
            }
        }
        
        // Audio info
        if let Some(ref audio) = self.audio_manager {
            let devices = audio.list_devices();
            info.audio_active = !devices.is_empty();
            info.audio_devices = devices.len();
            
            if let Some(volume) = audio.get_master_volume() {
                info.master_volume = volume.as_percent();
            }
        }
        
        Ok(info)
    }
    
    /// Shutdown all peripheral drivers
    pub fn shutdown(&mut self) -> DriverResult<()> {
        info!("Shutting down peripheral drivers");
        
        // Shutdown in reverse order
        if let Some(ref mut audio) = self.audio_manager {
            if let Some(output) = audio.get_primary_output() {
                let _ = output.set_volume(0, audio::VolumeLevel::new(0)); // Mute
            }
        }
        
        if let Some(ref graphics) = self.graphics_manager {
            graphics.clear(0x000000)?; // Clear screen to black
        }
        
        self.audio_manager = None;
        self.network_manager = None;
        self.storage_manager = None;
        self.graphics_manager = None;
        
        info!("Peripheral drivers shutdown complete");
        Ok(())
    }
}

/// System information gathered from all drivers
#[derive(Debug)]
pub struct SystemInfo {
    pub graphics_active: bool,
    pub graphics_driver: String,
    pub storage_active: bool,
    pub storage_type: String,
    pub storage_size_mb: u64,
    pub network_active: bool,
    pub network_interfaces: usize,
    pub network_type: String,
    pub mac_address: String,
    pub audio_active: bool,
    pub audio_devices: usize,
    pub master_volume: u8,
}

impl SystemInfo {
    /// Create new system info structure
    pub fn new() -> Self {
        Self {
            graphics_active: false,
            graphics_driver: "None".to_string(),
            storage_active: false,
            storage_type: "None".to_string(),
            storage_size_mb: 0,
            network_active: false,
            network_interfaces: 0,
            network_type: "None".to_string(),
            mac_address: "None".to_string(),
            audio_active: false,
            audio_devices: 0,
            master_volume: 0,
        }
    }
    
    /// Print system information
    pub fn print_summary(&self) {
        info!("=== System Information Summary ===");
        
        if self.graphics_active {
            info!("Graphics: {} driver active", self.graphics_driver);
        } else {
            info!("Graphics: No graphics driver active");
        }
        
        if self.storage_active {
            info!("Storage: {} ({:.1} GB)", 
                  self.storage_type, 
                  self.storage_size_mb as f64 / 1024.0);
        } else {
            info!("Storage: No storage device active");
        }
        
        if self.network_active {
            info!("Network: {} interface(s), {} speed, MAC: {}", 
                  self.network_interfaces, 
                  self.network_type,
                  self.mac_address);
        } else {
            info!("Network: No network interface active");
        }
        
        if self.audio_active {
            info!("Audio: {} device(s), volume: {}%", 
                  self.audio_devices, 
                  self.master_volume);
        } else {
            info!("Audio: No audio device active");
        }
        
        info!("=================================");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_peripheral_example_app_creation() {
        let app = PeripheralExampleApp::new();
        assert!(app.graphics_manager.is_none());
        assert!(app.storage_manager.is_none());
        assert!(app.network_manager.is_none());
        assert!(app.audio_manager.is_none());
    }

    #[test]
    fn test_system_info_creation() {
        let info = SystemInfo::new();
        
        assert!(!info.graphics_active);
        assert!(!info.storage_active);
        assert!(!info.network_active);
        assert!(!info.audio_active);
        assert_eq!(info.master_volume, 0);
    }

    #[test]
    fn test_system_info_printing() {
        let info = SystemInfo::new();
        info.print_summary();
        // Just ensure it doesn't panic
    }
}
