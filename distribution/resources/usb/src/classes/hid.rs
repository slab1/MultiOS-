//! USB HID (Human Interface Device) Class Driver
//! 
//! Supports HID devices like keyboards, mice, game controllers, and other input devices.
//! Implements HID protocol parsing, report descriptor processing, and device event handling.

use crate::*;

#[cfg(feature = "std")]
use std::collections::BTreeMap;

/// HID Report Types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HidReportType {
    Input = 0x01,
    Output = 0x02,
    Feature = 0x03,
}

/// HID Usage Pages
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HidUsagePage {
    GenericDesktop = 0x01,
    Simulation = 0x02,
    VR = 0x03,
    Sport = 0x04,
    Game = 0x05,
    GenericDeviceControls = 0x06,
    Keyboard = 0x07,
    LED = 0x08,
    Button = 0x09,
    Ordinal = 0x0A,
    Telephony = 0x0B,
    Consumer = 0x0C,
    Digitizer = 0x0D,
    Haptics = 0x0E,
    PID = 0x0F,
    Unicode = 0x10,
    AlphanumericDisplay = 0x14,
    Camera = 0x21,
    BarcodeScanner = 0x8C,
    Scale = 0x8D,
    MSR = 0x8E,
    CameraAutoFocus = 0x8F,
    Gaming = 0x91,
    VendorSpecific = 0xFF00,
    Undefined = 0x00,
}

/// HID Usage Pages for Generic Desktop
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HidGenericDesktopUsage {
    Pointer = 0x01,
    Mouse = 0x02,
    Joystick = 0x04,
    GamePad = 0x05,
    Keyboard = 0x06,
    Keypad = 0x07,
    MultiAxisController = 0x08,
    TabletPCSystemControls = 0x09,
    X = 0x30,
    Y = 0x31,
    Z = 0x32,
    Rx = 0x33,
    Ry = 0x34,
    Rz = 0x35,
    Slider = 0x36,
    Dial = 0x37,
    Wheel = 0x38,
    HatSwitch = 0x39,
    CountedBuffer = 0x3A,
    ByteCount = 0x3B,
    MotionWakeup = 0x3C,
    VX = 0x3D,
    VY = 0x3E,
    VZ = 0x3F,
    Vbrx = 0x40,
   Vbry = 0x41,
    Vbrz = 0x42,
    Vno = 0x43,
    SystemControl = 0x80,
    SystemPowerDown = 0x81,
    SystemSleep = 0x82,
    SystemWakeUp = 0x83,
    SystemContextMenu = 0x84,
    SystemMainMenu = 0x85,
    SystemAppMenu = 0x86,
    SystemMenuHelp = 0x87,
    SystemMenuExit = 0x88,
    SystemMenuSelect = 0x89,
    SystemRightArrow = 0x8A,
    SystemLeftArrow = 0x8B,
    SystemUpArrow = 0x8C,
    SystemDownArrow = 0x8D,
    SystemNext = 0x8E,
    SystemPrevious = 0x8F,
    SystemStop = 0x90,
    SystemPlay = 0x91,
    SystemPause = 0x92,
    SystemRecord = 0x93,
    SystemFastForward = 0x94,
    SystemRewind = 0x95,
    SystemScanNextTrack = 0x96,
    SystemScanPreviousTrack = 0x97,
    SystemStopEject = 0x98,
    SystemPlayPause = 0x99,
    SystemPlaySkip = 0x9A,
}

/// HID Report Descriptor Item
#[derive(Debug, Clone)]
pub struct HidReportItem {
    pub tag: u8,
    pub size: u8,
    pub data: u32,
    pub name: String,
}

/// HID Report Field
#[derive(Debug, Clone)]
pub struct HidReportField {
    pub usage_page: HidUsagePage,
    pub usage: u32,
    pub logical_minimum: i32,
    pub logical_maximum: i32,
    pub physical_minimum: i32,
    pub physical_maximum: i32,
    pub unit_exponent: i32,
    pub unit: u32,
    pub report_size: u32,
    pub report_count: u32,
    pub report_id: u32,
    pub is_absolute: bool,
    pub is_variable: bool,
    pub is_wrapped: bool,
    pub is_non_linear: bool,
    pub has_preferred_state: bool,
    pub is_null_state: bool,
    pub is_volatile: bool,
    pub is_buffered_bytes: bool,
}

/// HID Report
#[derive(Debug, Clone)]
pub struct HidReport {
    pub report_type: HidReportType,
    pub report_id: u32,
    pub fields: Vec<HidReportField>,
    pub size: u32,
}

/// HID Device Information
#[derive(Debug)]
pub struct HidDeviceInfo {
    pub vendor_id: u16,
    pub product_id: u16,
    pub version: u16,
    pub usage_page: HidUsagePage,
    pub usage: u32,
    pub num_reports: u8,
    pub reports: Vec<HidReport>,
    pub protocol: u8,
    pub country_code: u8,
    pub descriptor_type: u8,
}

/// HID Keyboard Event
#[derive(Debug, Clone)]
pub struct HidKeyboardEvent {
    pub key_code: u8,
    pub modifiers: HidKeyboardModifier,
    pub pressed: bool,
}

/// HID Keyboard Modifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HidKeyboardModifier {
    pub left_control: bool,
    pub left_shift: bool,
    pub left_alt: bool,
    pub left_gui: bool,
    pub right_control: bool,
    pub right_shift: bool,
    pub right_alt: bool,
    pub right_gui: bool,
}

impl HidKeyboardModifier {
    pub fn new() -> Self {
        Self {
            left_control: false,
            left_shift: false,
            left_alt: false,
            left_gui: false,
            right_control: false,
            right_shift: false,
            right_alt: false,
            right_gui: false,
        }
    }

    pub fn from_byte(byte: u8) -> Self {
        Self {
            left_control: (byte & 0x01) != 0,
            left_shift: (byte & 0x02) != 0,
            left_alt: (byte & 0x04) != 0,
            left_gui: (byte & 0x08) != 0,
            right_control: (byte & 0x10) != 0,
            right_shift: (byte & 0x20) != 0,
            right_alt: (byte & 0x40) != 0,
            right_gui: (byte & 0x80) != 0,
        }
    }

    pub fn to_byte(&self) -> u8 {
        (self.left_control as u8) |
        ((self.left_shift as u8) << 1) |
        ((self.left_alt as u8) << 2) |
        ((self.left_gui as u8) << 3) |
        ((self.right_control as u8) << 4) |
        ((self.right_shift as u8) << 5) |
        ((self.right_alt as u8) << 6) |
        ((self.right_gui as u8) << 7)
    }
}

/// HID Mouse Event
#[derive(Debug, Clone)]
pub struct HidMouseEvent {
    pub buttons: u8,
    pub x: i16,
    pub y: i16,
    pub wheel: i8,
}

/// HID Game Controller Event
#[derive(Debug, Clone)]
pub struct HidGamepadEvent {
    pub buttons: u32,
    pub x: i16,
    pub y: i16,
    pub z: i16,
    pub rx: i16,
    pub ry: i16,
    pub rz: i16,
    pub hat: i8,
    pub sliders: [i16; 2],
    pub dials: [i16; 4],
}

/// HID Generic Event
#[derive(Debug, Clone)]
pub struct HidGenericEvent {
    pub usage_page: HidUsagePage,
    pub usage: u32,
    pub value: i32,
    pub is_relative: bool,
    pub timestamp: u64,
}

/// HID Event Type
#[derive(Debug, Clone)]
pub enum HidEvent {
    Keyboard(HidKeyboardEvent),
    Mouse(HidMouseEvent),
    Gamepad(HidGamepadEvent),
    Generic(HidGenericEvent),
}

/// HID Device Driver
pub struct HidDriver {
    pub device_info: HidDeviceInfo,
    pub current_reports: BTreeMap<u32, Vec<u8>>,
    pub pending_events: Vec<HidEvent>,
    pub event_callback: Option<fn(HidEvent)>,
    pub interrupt_endpoint_in: Option<u8>,
    pub interrupt_endpoint_out: Option<u8>,
    pub polling_interval: u8,
    pub active: bool,
}

impl HidDriver {
    /// Create a new HID driver instance
    pub fn new(device_address: u8) -> Self {
        Self {
            device_info: HidDeviceInfo {
                vendor_id: 0,
                product_id: 0,
                version: 0,
                usage_page: HidUsagePage::GenericDesktop,
                usage: 0,
                num_reports: 0,
                reports: Vec::new(),
                protocol: 0,
                country_code: 0,
                descriptor_type: 0,
            },
            current_reports: BTreeMap::new(),
            pending_events: Vec::new(),
            event_callback: None,
            interrupt_endpoint_in: None,
            interrupt_endpoint_out: None,
            polling_interval: 1,
            active: false,
        }
    }

    /// Initialize HID device
    pub fn initialize(&mut self) -> UsbResult<()> {
        log::info!("Initializing HID device at address {}", 0);
        self.active = true;
        Ok(())
    }

    /// Parse HID report descriptor
    pub fn parse_report_descriptor(&mut self, descriptor: &[u8]) -> UsbResult<()> {
        let mut offset = 0;
        let mut reports = Vec::new();
        let mut current_report_type = HidReportType::Input;
        let mut current_report_id = 0u32;
        let mut current_fields = Vec::new();

        while offset < descriptor.len() {
            let item = self.parse_hid_item(&descriptor[offset..])?;
            offset += item.size + 1; // +1 for tag byte

            match item.tag & 0x0F {
                0x0A => { // Usage Page
                    let usage_page = HidUsagePage::from_u16(item.data as u16);
                    for field in &mut current_fields {
                        field.usage_page = usage_page;
                    }
                }
                0x08 => { // Usage
                    let usage = item.data;
                    for field in &mut current_fields {
                        field.usage = usage;
                    }
                }
                0x09 => { // Designator Index
                    // Handle designator if needed
                }
                0x0B => { // String Index
                    // Handle string index if needed
                }
                0x0C => { // String Descriptor
                    // Handle string descriptor if needed
                }
                0x0D => { // Delimiter
                    // Handle delimiter if needed
                }
                _ => { // Main items
                    match item.tag & 0x0F {
                        0x08 => { // Report Count
                            for field in &mut current_fields {
                                field.report_count = item.data as u32;
                            }
                        }
                        0x07 => { // Report Size
                            for field in &mut current_fields {
                                field.report_size = item.data as u32;
                            }
                        }
                        0x05 => { // Logical Minimum
                            for field in &mut current_fields {
                                field.logical_minimum = item.data as i32;
                            }
                        }
                        0x06 => { // Logical Maximum
                            for field in &mut current_fields {
                                field.logical_maximum = item.data as i32;
                            }
                        }
                        0x03 => { // Input, Output, or Feature
                            match item.tag >> 2 {
                                0x20 => current_report_type = HidReportType::Input,
                                0x21 => current_report_type = HidReportType::Output,
                                0x22 => current_report_type = HidReportType::Feature,
                                _ => {}
                            }

                            // Extract main item flags
                            if !current_fields.is_empty() {
                                let flags = item.data;
                                for field in &mut current_fields {
                                    field.is_absolute = (flags & 0x01) == 0;
                                    field.is_variable = (flags & 0x02) != 0;
                                    field.has_preferred_state = (flags & 0x04) == 0;
                                    field.is_non_linear = (flags & 0x08) != 0;
                                    field.is_null_state = (flags & 0x10) != 0;
                                }
                            }
                        }
                        0x04 => { // Collection
                            // End current report and start new one
                            if !current_fields.is_empty() {
                                let report = HidReport {
                                    report_type: current_report_type,
                                    report_id: current_report_id,
                                    fields: current_fields.clone(),
                                    size: 0,
                                };
                                reports.push(report);
                                current_fields.clear();
                            }
                        }
                        0x06 => { // End Collection
                            // Report is complete
                            if !current_fields.is_empty() {
                                let report = HidReport {
                                    report_type: current_report_type,
                                    report_id: current_report_id,
                                    fields: current_fields.clone(),
                                    size: 0,
                                };
                                reports.push(report);
                                current_fields.clear();
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        // Add any remaining fields to a report
        if !current_fields.is_empty() {
            let report = HidReport {
                report_type: current_report_type,
                report_id: current_report_id,
                fields: current_fields.clone(),
                size: 0,
            };
            reports.push(report);
        }

        self.device_info.reports = reports;
        self.device_info.num_reports = reports.len() as u8;

        log::info!("Parsed {} HID reports", reports.len());
        Ok(())
    }

    /// Parse a single HID item from descriptor data
    pub fn parse_hid_item(&self, data: &[u8]) -> UsbResult<HidReportItem> {
        if data.is_empty() {
            return Err(UsbDriverError::ProtocolError);
        }

        let tag_byte = data[0];
        let tag = tag_byte >> 4;
        let _type = (tag_byte >> 2) & 0x03;
        let size = tag_byte & 0x03;

        let mut value = 0u32;
        match size {
            0 => value = 0,
            1 if data.len() >= 2 => value = data[1] as u32,
            2 if data.len() >= 3 => value = ((data[1] as u32) | ((data[2] as u32) << 8)) as u32,
            3 if data.len() >= 5 => {
                value = (data[1] as u32) | 
                       ((data[2] as u32) << 8) |
                       ((data[3] as u32) << 16) |
                       ((data[4] as u32) << 24);
            }
            _ => return Err(UsbDriverError::ProtocolError),
        }

        Ok(HidReportItem {
            tag,
            size,
            data: value,
            name: self.get_item_name(tag).to_string(),
        })
    }

    /// Get item name for debugging
    fn get_item_name(&self, tag: u8) -> &'static str {
        match tag {
            0x0 => "Input",
            0x1 => "Output", 
            0x2 => "Feature",
            0x3 => "Collection",
            0x4 => "End Collection",
            0x5 => "Usage Page",
            0x6 => "Logical Minimum",
            0x7 => "Logical Maximum",
            0x8 => "Physical Minimum",
            0x9 => "Physical Maximum",
            0xA => "Unit Exponent",
            0xB => "Unit",
            0xC => "Report Size",
            0xD => "Report ID",
            0xE => "Report Count",
            0xF => "Push",
            _ => "Unknown",
        }
    }

    /// Process HID report data
    pub fn process_report(&mut self, report_type: HidReportType, report_id: u32, data: &[u8]) {
        // Store current report state
        self.current_reports.insert(report_id, data.to_vec());

        // Parse event based on device type
        match self.device_info.usage_page {
            HidUsagePage::GenericDesktop => {
                self.process_generic_desktop_report(report_type, report_id, data);
            }
            HidUsagePage::Keyboard => {
                self.process_keyboard_report(report_type, report_id, data);
            }
            HidUsagePage::Button => {
                self.process_button_report(report_type, report_id, data);
            }
            _ => {
                // Generic processing for unknown device types
                self.process_generic_report(report_type, report_id, data);
            }
        }
    }

    /// Process generic desktop device reports (mouse, joystick, etc.)
    fn process_generic_desktop_report(&mut self, report_type: HidReportType, report_id: u32, data: &[u8]) {
        if report_type != HidReportType::Input {
            return;
        }

        // Look for common patterns
        if self.device_info.usage == HidGenericDesktopUsage::Mouse as u32 {
            self.process_mouse_report(data);
        } else if self.device_info.usage == HidGenericDesktopUsage::Joystick as u32 || 
                  self.device_info.usage == HidGenericDesktopUsage::GamePad as u32 {
            self.process_gamepad_report(data);
        } else {
            self.process_keyboard_report(report_type, report_id, data);
        }
    }

    /// Process mouse report
    fn process_mouse_report(&mut self, data: &[u8]) {
        if data.len() < 3 {
            return;
        }

        let mut buttons = 0u8;
        if data.len() > 0 {
            buttons = data[0];
        }

        let mut x = 0i16;
        let mut y = 0i16;
        let mut wheel = 0i8;

        if data.len() >= 5 {
            x = (data[1] as i8) as i16;
            y = (data[2] as i8) as i16;
            if data.len() > 3 {
                wheel = data[3] as i8;
            }
        }

        let event = HidEvent::Mouse(HidMouseEvent {
            buttons,
            x,
            y,
            wheel,
        });

        self.queue_event(event);
    }

    /// Process keyboard report
    fn process_keyboard_report(&mut self, _report_type: HidReportType, _report_id: u32, data: &[u8]) {
        if data.len() < 8 {
            return;
        }

        let modifiers = HidKeyboardModifier::from_byte(data[0]);
        let mut key_codes = Vec::new();

        // Extract key codes (data[2] to data[7] for 6-key rollover)
        for i in 2..8.min(data.len()) {
            if data[i] != 0 {
                key_codes.push(data[i]);
            }
        }

        // Generate events for each key
        for &key_code in &key_codes {
            let event = HidEvent::Keyboard(HidKeyboardEvent {
                key_code,
                modifiers,
                pressed: true,
            });
            self.queue_event(event);
        }
    }

    /// Process gamepad/joystick report
    fn process_gamepad_report(&mut self, data: &[u8]) {
        let mut gamepad_event = HidGamepadEvent {
            buttons: 0,
            x: 0,
            y: 0,
            z: 0,
            rx: 0,
            ry: 0,
            rz: 0,
            hat: -1,
            sliders: [0; 2],
            dials: [0; 4],
        };

        let mut offset = 0;

        // Parse buttons (typically first 2-4 bytes)
        if data.len() >= 4 {
            gamepad_event.buttons = (data[0] as u32) | 
                                   ((data[1] as u32) << 8) |
                                   ((data[2] as u32) << 16) |
                                   ((data[3] as u32) << 24);
            offset = 4;
        }

        // Parse analog axes (typically 2-6 bytes)
        if data.len() >= offset + 1 {
            gamepad_event.x = data[offset] as i16;
            offset += 1;
        }
        if data.len() >= offset + 1 {
            gamepad_event.y = data[offset] as i16;
            offset += 1;
        }
        if data.len() >= offset + 1 {
            gamepad_event.z = data[offset] as i16;
            offset += 1;
        }
        if data.len() >= offset + 1 {
            gamepad_event.rx = data[offset] as i16;
            offset += 1;
        }
        if data.len() >= offset + 1 {
            gamepad_event.ry = data[offset] as i16;
            offset += 1;
        }
        if data.len() >= offset + 1 {
            gamepad_event.rz = data[offset] as i16;
            offset += 1;
        }

        // Parse hat switch
        if data.len() >= offset + 1 {
            gamepad_event.hat = data[offset] as i8;
        }

        let event = HidEvent::Gamepad(gamepad_event);
        self.queue_event(event);
    }

    /// Process button-only devices
    fn process_button_report(&mut self, _report_type: HidReportType, _report_id: u32, data: &[u8]) {
        let mut button_count = 0;
        for &byte in data {
            for bit in 0..8 {
                if byte & (1 << bit) != 0 {
                    button_count += 1;
                    
                    let event = HidEvent::Generic(HidGenericEvent {
                        usage_page: HidUsagePage::Button,
                        usage: button_count,
                        value: 1,
                        is_relative: false,
                        timestamp: 0, // TODO: Add timestamp
                    });
                    
                    self.queue_event(event);
                }
            }
        }
    }

    /// Process generic HID reports
    fn process_generic_report(&mut self, _report_type: HidReportType, _report_id: u32, data: &[u8]) {
        for (i, &byte) in data.iter().enumerate() {
            let event = HidEvent::Generic(HidGenericEvent {
                usage_page: self.device_info.usage_page,
                usage: i as u32,
                value: byte as i32,
                is_relative: false,
                timestamp: 0, // TODO: Add timestamp
            });
            
            self.queue_event(event);
        }
    }

    /// Queue an HID event
    fn queue_event(&mut self, event: HidEvent) {
        self.pending_events.push(event);
        
        // Call event callback if set
        if let Some(callback) = self.event_callback {
            callback(event.clone());
        }
    }

    /// Set event callback function
    pub fn set_event_callback(&mut self, callback: fn(HidEvent)) {
        self.event_callback = Some(callback);
    }

    /// Get next pending event
    pub fn get_next_event(&mut self) -> Option<HidEvent> {
        self.pending_events.pop()
    }

    /// Clear pending events
    pub fn clear_events(&mut self) {
        self.pending_events.clear();
    }

    /// Get device info
    pub fn get_device_info(&self) -> &HidDeviceInfo {
        &self.device_info
    }

    /// Set interrupt endpoint
    pub fn set_interrupt_endpoint(&mut self, endpoint_in: Option<u8>, endpoint_out: Option<u8>) {
        self.interrupt_endpoint_in = endpoint_in;
        self.interrupt_endpoint_out = endpoint_out;
    }

    /// Check if device is active
    pub fn is_active(&self) -> bool {
        self.active
    }
}

/// Usage page conversion functions
impl HidUsagePage {
    pub fn from_u16(usage_page: u16) -> Self {
        match usage_page {
            0x01 => HidUsagePage::GenericDesktop,
            0x02 => HidUsagePage::Simulation,
            0x03 => HidUsagePage::VR,
            0x04 => HidUsagePage::Sport,
            0x05 => HidUsagePage::Game,
            0x06 => HidUsagePage::GenericDeviceControls,
            0x07 => HidUsagePage::Keyboard,
            0x08 => HidUsagePage::LED,
            0x09 => HidUsagePage::Button,
            0x0A => HidUsagePage::Ordinal,
            0x0B => HidUsagePage::Telephony,
            0x0C => HidUsagePage::Consumer,
            0x0D => HidUsagePage::Digitizer,
            0x0E => HidUsagePage::Haptics,
            0x0F => HidUsagePage::PID,
            0x10 => HidUsagePage::Unicode,
            0x14 => HidUsagePage::AlphanumericDisplay,
            0x21 => HidUsagePage::Camera,
            0x8C => HidUsagePage::BarcodeScanner,
            0x8D => HidUsagePage::Scale,
            0x8E => HidUsagePage::MSR,
            0x8F => HidUsagePage::CameraAutoFocus,
            0x91 => HidUsagePage::Gaming,
            0xFF00..=0xFFFF => HidUsagePage::VendorSpecific,
            _ => HidUsagePage::Undefined,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hid_driver_creation() {
        let driver = HidDriver::new(1);
        assert_eq!(driver.is_active(), false);
    }

    #[test]
    fn test_keyboard_modifier_parsing() {
        let modifier = HidKeyboardModifier::from_byte(0x00);
        assert_eq!(modifier.left_control, false);
        assert_eq!(modifier.left_shift, false);

        let modifier = HidKeyboardModifier::from_byte(0xFF);
        assert_eq!(modifier.left_control, true);
        assert_eq!(modifier.left_shift, true);
        assert_eq!(modifier.left_alt, true);
        assert_eq!(modifier.left_gui, true);
        assert_eq!(modifier.right_control, true);
        assert_eq!(modifier.right_shift, true);
        assert_eq!(modifier.right_alt, true);
        assert_eq!(modifier.right_gui, true);
    }

    #[test]
    fn test_keyboard_modifier_serialization() {
        let modifier = HidKeyboardModifier::from_byte(0x55);
        assert_eq!(modifier.to_byte(), 0x55);
    }

    #[test]
    fn test_usage_page_parsing() {
        assert_eq!(HidUsagePage::from_u16(0x01), HidUsagePage::GenericDesktop);
        assert_eq!(HidUsagePage::from_u16(0x07), HidUsagePage::Keyboard);
        assert_eq!(HidUsagePage::from_u16(0xFF00), HidUsagePage::VendorSpecific);
    }
}