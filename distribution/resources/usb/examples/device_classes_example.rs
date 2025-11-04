//! USB Device Class Driver Examples
//!
//! This example demonstrates how to use the MultiOS USB device class
//! framework for HID, Mass Storage, CDC, and Audio devices.

use crate::device::UsbDevice;
use crate::classes::{HidDevice, MscDevice, CdcDevice, AudioDevice, UsbDeviceClass};
use crate::descriptor::{UsbDescriptor, UsbDescriptorType};
use crate::{UsbError, UsbResult};

/// Example: Human Interface Device (HID) Class Driver
pub fn example_hid_device() -> UsbResult<()> {
    println!("HID Device Class Example");
    println!("========================\n");

    // Create a mock HID device
    let mut hid_device = HidDevice::new(
        0x1234, // Vendor ID
        0x5678, // Product ID
        1,      // Interface number
    );

    // Initialize HID device
    hid_device.initialize()?;
    println!("✓ HID device initialized");

    // Parse HID descriptors
    let descriptor_data = create_mock_hid_descriptor();
    let parsed_descriptor = hid_device.parse_hid_descriptor(&descriptor_data)?;
    println!("✓ HID descriptor parsed:");
    println!("  Country code: {}", parsed_descriptor.country_code);
    println!("  Report descriptors: {} bytes", parsed_descriptor.report_descriptor.len());

    // Example keyboard input processing
    println!("\nKeyboard Input Processing Example:");
    println!("=================================");

    // Simulate keyboard key press (Escape key)
    let keyboard_report = [
        0x00,   // Modifier keys (none)
        0x00,   // Reserved
        0x01,   // Key code for Escape
    ];

    let parsed_input = hid_device.parse_input_report(&keyboard_report)?;
    println!("✓ Parsed keyboard input:");
    println!("  Key code: 0x{:02X} ({})", 
        parsed_input.key_codes[0], 
        get_key_name(parsed_input.key_codes[0]));

    // Example mouse input processing
    println!("\nMouse Input Processing Example:");
    println!("==============================");

    let mouse_report = [
        0x01,   // Button 1 pressed
        0x00,   // Button 2 not pressed
        0x00,   // Button 3 not pressed
        0x10,   // X movement (16 pixels right)
        0x00,   // X movement continuation
        0xF0,   // Y movement (16 pixels up, using two's complement)
        0xFF,   // Y movement continuation
    ];

    let parsed_input = hid_device.parse_input_report(&mouse_report)?;
    println!("✓ Parsed mouse input:");
    println!("  Buttons: {}", 
        if parsed_input.mouse_data.button1 { "Left" } else { "None" });
    println!("  X movement: {}", parsed_input.mouse_data.x_movement as i16);
    println!("  Y movement: {}", parsed_input.mouse_data.y_movement as i16);

    // Example gamepad processing
    println!("\nGamepad Input Processing Example:");
    println!("================================");

    let gamepad_report = [
        0x01,   // D-pad up
        0x00,   // Buttons (none pressed)
        0x80,   // Left stick X (center)
        0x80,   // Left stick Y (center)
        0x80,   // Right stick X (center)
        0x80,   // Right stick Y (center)
        0x64,   // Left trigger (100%)
        0x00,   // Right trigger (0%)
    ];

    let parsed_input = hid_device.parse_input_report(&gamepad_report)?;
    println!("✓ Parsed gamepad input:");
    println!("  D-pad: {:?}", parsed_input.gamepad_data.dpad_up);
    println!("  Left stick: ({}, {})", 
        parsed_input.gamepad_data.left_stick_x,
        parsed_input.gamepad_data.left_stick_y);
    println!("  Right stick: ({}, {})",
        parsed_input.gamepad_data.right_stick_x,
        parsed_input.gamepad_data.right_stick_y);
    println!("  Triggers: L={}%, R={}%",
        parsed_input.gamepad_data.left_trigger,
        parsed_input.gamepad_data.right_trigger);

    println!("\nHID device example completed successfully!");
    Ok(())
}

/// Example: Mass Storage Class (MSC) Driver
pub fn example_mass_storage_device() -> UsbResult<()> {
    println!("Mass Storage Device Class Example");
    println!("==================================\n");

    // Create a mock mass storage device
    let mut msc_device = MscDevice::new(
        0xABCD, // Vendor ID
        0xEF01, // Product ID
        1,      // Interface number
    );

    // Initialize MSC device
    msc_device.initialize()?;
    println!("✓ Mass storage device initialized");

    // Parse SCSI capabilities
    println!("\nSCSI Capabilities:");
    println!("==================");
    println!("Vendor: {}", msc_device.vendor_string());
    println!("Product: {}", msc_device.product_string());
    println!("Revision: {}", msc_device.revision_string());
    println!("Removable: {}", if msc_device.is_removable() { "Yes" } else { "No" });

    // Example read capacity command
    println!("\nRead Capacity Example:");
    println!("=====================");

    let read_capacity_cmd = [
        0x25, // READ CAPACITY (10) command opcode
        0x00, // Reserved
        0x00, // LBA (high byte)
        0x00,
        0x00,
        0x00, // LBA (low byte)
        0x00, // Reserved
        0x00, // Reserved
        0x00, // Reserved
        0x00, // Control
    ];

    let result = msc_device.execute_scsi_command(&read_capacity_cmd, 8)?;
    println!("✓ Read capacity command executed");
    println!("  Response: {} bytes", result.len());

    // Parse capacity response
    if result.len() >= 8 {
        let logical_block_address = ((result[0] as u32) << 24) |
                                   ((result[1] as u32) << 16) |
                                   ((result[2] as u32) << 8) |
                                   (result[3] as u32);
        let block_length = ((result[4] as u32) << 24) |
                          ((result[5] as u32) << 16) |
                          ((result[6] as u32) << 8) |
                          (result[7] as u32);

        println!("  Last logical block: {}", logical_block_address);
        println!("  Block length: {} bytes", block_length);
        println!("  Capacity: {} MB", 
            (logical_block_address + 1) * block_length / 1024 / 1024);
    }

    // Example read data command
    println!("\nRead Data Example:");
    println!("=================");

    let read_cmd = [
        0x28, // READ (10) command opcode
        0x00, // Flags
        0x00, // LBA (high byte)
        0x00,
        0x00,
        0x10, // LBA = block 16
        0x00, // Transfer length (high byte)
        0x01, // Transfer length = 1 block
        0x00, // Reserved
        0x00, // Control
    ];

    let block_size = 512;
    let result = msc_device.execute_scsi_command(&read_cmd, block_size)?;
    println!("✓ Read command executed");
    println!("  Read {} bytes from block 16", result.len());

    // Example: USB bulk-only transport
    println!("\nBulk-Only Transport Example:");
    println!("===========================");
    println!("Command Wrapper (CBW):");
    println!("  Signature: 0x43425355 (USBC)");
    println!("  Tag: 0x00001234");
    println!("  DataTransferLength: {}", block_size);
    println!("  Flags: 0x80 (Data-in from device)");
    println!("  LUN: 0");
    println!("  CBLength: 10");

    println!("\nData Phase:");
    println!("  Transfer direction: Device → Host");
    println!("  Data length: {} bytes", block_size);

    println!("\nStatus Wrapper (CSW):");
    println!("  Signature: 0x53425355 (USBS)");
    println!("  Tag: 0x00001234");
    println!("  DataResidue: 0");
    println!("  Status: 0x00 (Success)");

    println!("\nMass storage device example completed successfully!");
    Ok(())
}

/// Example: Communications Device Class (CDC) Driver
pub fn example_communications_device() -> UsbResult<()> {
    println!("Communications Device Class Example");
    println!("===================================\n");

    // Create a mock CDC device
    let mut cdc_device = CdcDevice::new(
        0x1234, // Vendor ID
        0x5678, // Product ID
        1,      // Interface number (control)
        2,      // Interface number (data)
    );

    // Initialize CDC device
    cdc_device.initialize()?;
    println!("✓ CDC device initialized");

    // Example: Serial communication setup
    println!("\nSerial Communication Setup:");
    println!("==========================");

    // Set line coding (baud rate, data bits, stop bits, parity)
    let line_coding = cdc_device.create_line_coding(
        crate::classes::cdc::LineBaudRate::B115200,
        crate::classes::cdc::DataBits::Eight,
        crate::classes::cdc::StopBits::One,
        crate::classes::cdc::Parity::None,
    );
    println!("✓ Line coding configured:");
    println!("  Baud rate: {} bps", line_coding.baud_rate);
    println!("  Data bits: {}", line_coding.data_bits as u8);
    println!("  Stop bits: {}", line_coding.stop_bits as u8);
    println!("  Parity: {:?}", line_coding.parity);

    // Set control line state
    cdc_device.set_control_line_state(
        crate::classes::cdc::ControlLineState::DTR | 
        crate::classes::cdc::ControlLineState::RTS,
    )?;
    println!("✓ Control lines set: DTR, RTS");

    // Example: Send data over serial port
    println!("\nSerial Data Transfer Example:");
    println!("============================");

    let test_message = b"Hello from MultiOS USB!";
    let bytes_sent = cdc_device.send_data(test_message)?;
    println!("✓ Sent {} bytes: \"{}\"", bytes_sent, String::from_utf8_lossy(test_message));

    // Example: USB modem functionality
    println!("\nModem Functionality Example:");
    println!("===========================");

    // Dial a number (AT command)
    let dial_cmd = b"ATD555-1234\r";
    cdc_device.send_data(dial_cmd)?;
    println!("✓ Sent AT command: \"{}\"", String::from_utf8_lossy(dial_cmd));

    // Example: USB network adapter functionality
    println!("\nUSB Network Adapter Example:");
    println!("===========================");

    println!("• Network interface detected");
    println!("• MAC address: 00:01:02:03:04:05");
    println!("• IP configuration: DHCP");
    println!("• Connection status: Connected");
    println!("• Transmitted: 1024 bytes");
    println!("• Received: 2048 bytes");

    println!("\nCommunications device example completed successfully!");
    Ok(())
}

/// Example: USB Audio Class Driver
pub fn example_audio_device() -> UsbResult<()> {
    println!("USB Audio Device Class Example");
    println!("==============================\n");

    // Create a mock audio device
    let mut audio_device = AudioDevice::new(
        0x1234, // Vendor ID
        0x5678, // Product ID
        1,      // Interface number (streaming)
    );

    // Initialize audio device
    audio_device.initialize()?;
    println!("✓ Audio device initialized");

    // Example: Audio format configuration
    println!("\nAudio Format Configuration:");
    println!("===========================");

    let format = audio_device.create_audio_format(
        crate::classes::audio::AudioSampleRate::Hz44100,
        crate::classes::audio::AudioSampleBits::Sixteen,
        crate::classes::audio::AudioChannels::Stereo,
    );
    println!("✓ Audio format configured:");
    println!("  Sample rate: {} Hz", format.sample_rate);
    println!("  Sample bits: {}", format.sample_bits as u8);
    println!("  Channels: {}", format.channels as u8);
    println!("  Bit rate: {} kbps", format.bit_rate / 1000);

    // Example: Volume control
    println!("\nVolume Control Example:");
    println!("======================");

    // Set master volume to 75%
    let master_volume = 0.75;
    audio_device.set_master_volume(master_volume)?;
    println!("✓ Master volume set to {}%", (master_volume * 100.0) as u8);

    // Set left channel volume to 80%
    audio_device.set_channel_volume(0, 0.8)?;
    println!("✓ Left channel volume set to 80%");

    // Set right channel volume to 70%
    audio_device.set_channel_volume(1, 0.7)?;
    println!("✓ Right channel volume set to 70%");

    // Example: Audio streaming
    println!("\nAudio Streaming Example:");
    println!("=======================");

    // Simulate audio data transmission
    let sample_data = create_mock_audio_data();
    let frames_transmitted = audio_device.stream_audio_data(&sample_data)?;
    println!("✓ Transmitted {} audio frames", frames_transmitted);
    println!("  Data size: {} bytes", sample_data.len());
    println!("  Duration: {:.1} ms", (sample_data.len() as f32 / 44100.0 / 4.0) * 1000.0);

    // Example: Audio device capabilities
    println!("\nAudio Device Capabilities:");
    println!("=========================");

    println!("Device type: {:?}", audio_device.get_device_type());
    println!("Streaming interface: ✓");
    println!("Control interface: ✓");
    println!("Volume control: ✓");
    println!("Mute control: ✓");
    println!("Sample rate support: 44.1 kHz, 48 kHz");
    println!("Sample depth support: 16-bit, 24-bit");
    println!("Channel support: Mono, Stereo");

    // Example: Audio synchronization
    println!("\nAudio Synchronization:");
    println!("=====================");

    println!("• Feedback endpoint configured");
    println!("• Sample rate adaptation: Active");
    println!("• Clock synchronization: Master");
    println!("• Buffer underrun protection: Enabled");
    println!("• Latency: 10 ms");

    println!("\nAudio device example completed successfully!");
    Ok(())
}

/// Create mock HID descriptor data
fn create_mock_hid_descriptor() -> Vec<u8> {
    vec![
        0x09, 0x21,  // HID descriptor type
        0x11, 0x01,  // HID version 1.11
        0x00,        // Country code (not localized)
        0x01,        // Number of descriptors
        0x22,        // Descriptor type (report)
        0x3F, 0x00,  // Report descriptor length (63 bytes)
    ]
}

/// Get human-readable key name for HID key code
fn get_key_name(key_code: u8) -> &'static str {
    match key_code {
        0x00 => "None",
        0x01 => "Keyboard ErrorRollOver",
        0x02 => "Keyboard POSTFail",
        0x03 => "Keyboard ErrorUndefined",
        0x04 => "a/A",
        0x05 => "b/B",
        // ... many more keys would be defined here
        0x1C => "Enter",
        0x1D => "Left Control",
        0x1E => "Left Shift",
        0x1F => "Left Alt",
        0x20 => "Left GUI",
        0x21 => "Right Control",
        0x22 => "Right Shift",
        0x23 => "Right Alt",
        0x24 => "Right GUI",
        _ => "Unknown key",
    }
}

/// Create mock audio data for streaming example
fn create_mock_audio_data() -> Vec<u8> {
    // Create 44100 samples * 2 channels * 2 bytes per sample = 176400 bytes
    // That's about 4 seconds of audio at 44.1kHz
    let sample_count = 44100;
    let channel_count = 2;
    let bytes_per_sample = 2;
    let total_bytes = sample_count * channel_count * bytes_per_sample;
    
    // Create sine wave data
    let mut audio_data = Vec::with_capacity(total_bytes);
    let frequency = 440.0; // A4 note
    let sample_rate = 44100.0;
    let amplitude = 0.3;
    
    for i in 0..sample_count {
        let t = (i as f32) / sample_rate;
        let sample = (frequency * 2.0 * core::f32::consts::PI * t).sin() * amplitude;
        
        // Convert to 16-bit signed integer
        let sample_int = (sample * 32767.0) as i16;
        
        // Add sample for both channels (stereo)
        let sample_bytes = sample_int.to_le_bytes();
        audio_data.extend_from_slice(&sample_bytes);
        audio_data.extend_from_slice(&sample_bytes);
    }
    
    audio_data
}

/// Example: Combined Device Class Usage
pub fn example_combined_classes() -> UsbResult<()> {
    println!("Combined Device Class Example");
    println!("=============================\n");

    println!("Simulating a composite USB device with multiple functions:");
    println!("========================================================\n");

    // Simulate a USB headset with multiple interfaces
    println!("1. Audio Interface (USB Audio Class)");
    println!("   • Stereo speakers output");
    println!("   • Microphone input");
    println!("   • Volume controls");
    println!("   • Sample rate: 44.1 kHz, 16-bit");

    println!("\n2. HID Interface (Human Interface Device)");
    println!("   • Media playback controls (play/pause, next, previous)");
    println!("   • Volume up/down buttons");
    println!("   • Mute toggle button");

    println!("\n3. CDC Interface (Communications Device Class)");
    println!("   • Software updates");
    println!("   • Device configuration");
    println!("   • Firmware management");

    println!("\nSimulating simultaneous operation:");
    println!("================================");

    // Simulate audio playback
    println!("• Audio: Playing music through speakers");
    println!("• Audio: Recording voice input from microphone");
    println!("• HID: Volume up button pressed - increasing volume");
    println!("• CDC: Checking for software updates in background");
    println!("• HID: Play/pause button toggled - music paused");
    println!("• CDC: Software update downloaded successfully");

    println!("\nMulti-interface synchronization:");
    println!("===============================:");
    println!("• Audio and HID interfaces share device state");
    println!("• Volume changes affect both audio output and metadata");
    println!("• CDC interface provides device management without interrupting audio");
    println!("• All interfaces operate concurrently on the same physical device");

    Ok(())
}