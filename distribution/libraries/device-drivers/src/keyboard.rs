//! Keyboard Driver
//! 
//! Provides support for PS/2 keyboard and USB keyboard input.

use crate::{DeviceType, DriverResult, DriverError, device::{Device, DeviceDriver, DeviceCapabilities}};
use spin::{Mutex, Once};
use alloc::collections::VecDeque;
use log::{info, warn, error};

/// Key codes (PS/2 keyboard scan codes)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum KeyCode {
    // Special keys
    Escape = 0x01,
    F1 = 0x3B,
    F2 = 0x3C,
    F3 = 0x3D,
    F4 = 0x3E,
    F5 = 0x3F,
    F6 = 0x40,
    F7 = 0x41,
    F8 = 0x42,
    F9 = 0x43,
    F10 = 0x44,
    F11 = 0x57,
    F12 = 0x58,
    
    // Number row
    Backquote = 0x29,
    Digit1 = 0x02,
    Digit2 = 0x03,
    Digit3 = 0x04,
    Digit4 = 0x05,
    Digit5 = 0x06,
    Digit6 = 0x07,
    Digit7 = 0x08,
    Digit8 = 0x09,
    Digit9 = 0x0A,
    Digit0 = 0x0B,
    Minus = 0x0C,
    Equal = 0x0D,
    Backspace = 0x0E,
    
    // Top letter row
    Tab = 0x0F,
    KeyQ = 0x10,
    KeyW = 0x11,
    KeyE = 0x12,
    KeyR = 0x13,
    KeyT = 0x14,
    KeyY = 0x15,
    KeyU = 0x16,
    KeyI = 0x17,
    KeyO = 0x18,
    KeyP = 0x19,
    LeftBracket = 0x1A,
    RightBracket = 0x1B,
    Backslash = 0x2B,
    
    // Home row
    CapsLock = 0x3A,
    KeyA = 0x1E,
    KeyS = 0x1F,
    KeyD = 0x20,
    KeyF = 0x21,
    KeyG = 0x22,
    KeyH = 0x23,
    KeyJ = 0x24,
    KeyK = 0x25,
    KeyL = 0x26,
    Semicolon = 0x27,
    Quote = 0x28,
    Enter = 0x1C,
    
    // Bottom row
    LeftShift = 0x2A,
    KeyZ = 0x2C,
    KeyX = 0x2D,
    KeyC = 0x2E,
    KeyV = 0x2F,
    KeyB = 0x30,
    KeyN = 0x31,
    KeyM = 0x32,
    Comma = 0x33,
    Period = 0x34,
    Slash = 0x35,
    RightShift = 0x36,
    
    // Bottom modifiers
    LeftControl = 0x1D,
    LeftAlt = 0x38,
    Space = 0x39,
    RightAlt = 0xE1,
    RightControl = 0xE0,
    
    // Navigation
    PrintScreen = 0x37,
    ScrollLock = 0x46,
    Pause = 0x45,
    Insert = 0x52,
    Home = 0x47,
    PageUp = 0x49,
    Delete = 0x53,
    End = 0x4F,
    PageDown = 0x51,
    
    // Arrow keys
    ArrowUp = 0x48,
    ArrowDown = 0x50,
    ArrowLeft = 0x4B,
    ArrowRight = 0x4D,
    
    // Numpad
    NumLock = 0x45,
    Divide = 0x35,
    Multiply = 0x37,
    Subtract = 0x4A,
    Add = 0x4E,
    EnterNumpad = 0x1C,
    Decimal = 0x53,
    Num0 = 0x52,
    Num1 = 0x4F,
    Num2 = 0x50,
    Num3 = 0x51,
    Num4 = 0x4B,
    Num5 = 0x4C,
    Num6 = 0x4D,
    Num7 = 0x47,
    Num8 = 0x48,
    Num9 = 0x49,
}

/// Key event structure
#[derive(Debug, Clone, Copy)]
pub struct KeyEvent {
    pub key_code: KeyCode,
    pub is_pressed: bool,
    pub modifiers: KeyModifiers,
    pub timestamp: u64,
}

/// Keyboard modifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyModifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub caps_lock: bool,
    pub num_lock: bool,
    pub scroll_lock: bool,
}

impl Default for KeyModifiers {
    fn default() -> Self {
        Self {
            shift: false,
            ctrl: false,
            alt: false,
            caps_lock: false,
            num_lock: false,
            scroll_lock: false,
        }
    }
}

impl KeyModifiers {
    /// Check if any modifier is active
    pub fn is_active(&self) -> bool {
        self.shift || self.ctrl || self.alt || self.caps_lock || self.num_lock || self.scroll_lock
    }
}

/// Keyboard driver type
#[derive(Debug, Clone, Copy)]
pub enum KeyboardType {
    Ps2Keyboard,
    UsbKeyboard,
}

/// PS/2 Keyboard Driver
pub struct Ps2Keyboard {
    pub port: u16,
    pub keyboard_type: KeyboardType,
    pub key_queue: Mutex<VecDeque<KeyEvent>>,
    pub modifiers: Mutex<KeyModifiers>,
    pub caps_lock_state: Mutex<bool>,
}

impl Ps2Keyboard {
    /// Create new PS/2 keyboard driver
    pub fn new(port: u16) -> Self {
        Self {
            port,
            keyboard_type: KeyboardType::Ps2Keyboard,
            key_queue: Mutex::new(VecDeque::new()),
            modifiers: Mutex::new(KeyModifiers::default()),
            caps_lock_state: Mutex::new(false),
        }
    }

    /// Initialize PS/2 keyboard
    pub fn init(&self) -> DriverResult<()> {
        info!("Initializing PS/2 keyboard at port 0x{:04x}", self.port);
        
        // Send reset command
        self.send_command(0xFF)?;
        
        // Enable keyboard
        self.send_command(0xF4)?;
        
        info!("PS/2 keyboard initialized");
        Ok(())
    }

    /// Send command to keyboard
    fn send_command(&self, command: u8) -> DriverResult<()> {
        // Wait for input buffer to be empty
        let mut timeout = 1000;
        while timeout > 0 {
            let status = unsafe { core::ptr::read_volatile((self.port + 4) as *const u8) };
            if status & 0x02 == 0 {
                break; // Input buffer empty
            }
            timeout -= 1;
        }
        
        if timeout == 0 {
            return Err(DriverError::HardwareError);
        }
        
        // Send command
        unsafe { core::ptr::write_volatile(self.port as *mut u8, command) };
        
        Ok(())
    }

    /// Read scan code from keyboard
    pub fn read_scan_code(&self) -> Option<u8> {
        let status = unsafe { core::ptr::read_volatile((self.port + 4) as *const u8) };
        
        if status & 0x01 != 0 {
            // Data available
            Some(unsafe { core::ptr::read_volatile(self.port as *const u8) })
        } else {
            None
        }
    }

    /// Process scan code and generate key event
    pub fn process_scan_code(&self, scan_code: u8) -> Option<KeyEvent> {
        let mut modifiers = *self.modifiers.lock();
        let mut is_pressed = true;
        let mut key_code = KeyCode::Escape;
        
        // Handle special codes
        if scan_code == 0xF0 {
            // Release code prefix - wait for next code
            return None;
        } else if scan_code == 0xE0 {
            // Extended code prefix - wait for next code
            return None;
        }
        
        // Convert scan code to key code (simplified mapping)
        key_code = match scan_code {
            0x01 => KeyCode::Escape,
            0x02 => KeyCode::Digit1,
            0x03 => KeyCode::Digit2,
            0x04 => KeyCode::Digit3,
            0x05 => KeyCode::Digit4,
            0x06 => KeyCode::Digit5,
            0x07 => KeyCode::Digit6,
            0x08 => KeyCode::Digit7,
            0x09 => KeyCode::Digit8,
            0x0A => KeyCode::Digit9,
            0x0B => KeyCode::Digit0,
            0x0C => KeyCode::Minus,
            0x0D => KeyCode::Equal,
            0x0E => KeyCode::Backspace,
            0x0F => KeyCode::Tab,
            0x10 => KeyCode::KeyQ,
            0x11 => KeyCode::KeyW,
            0x12 => KeyCode::KeyE,
            0x13 => KeyCode::KeyR,
            0x14 => KeyCode::KeyT,
            0x15 => KeyCode::KeyY,
            0x16 => KeyCode::KeyU,
            0x17 => KeyCode::KeyI,
            0x18 => KeyCode::KeyO,
            0x19 => KeyCode::KeyP,
            0x1A => KeyCode::LeftBracket,
            0x1B => KeyCode::RightBracket,
            0x1C => KeyCode::Enter,
            0x1D => KeyCode::LeftControl,
            0x1E => KeyCode::KeyA,
            0x1F => KeyCode::KeyS,
            0x20 => KeyCode::KeyD,
            0x21 => KeyCode::KeyF,
            0x22 => KeyCode::KeyG,
            0x23 => KeyCode::KeyH,
            0x24 => KeyCode::KeyJ,
            0x25 => KeyCode::KeyK,
            0x26 => KeyCode::KeyL,
            0x27 => KeyCode::Semicolon,
            0x28 => KeyCode::Quote,
            0x29 => KeyCode::Backquote,
            0x2A => KeyCode::LeftShift,
            0x2B => KeyCode::Backslash,
            0x2C => KeyCode::KeyZ,
            0x2D => KeyCode::KeyX,
            0x2E => KeyCode::KeyC,
            0x2F => KeyCode::KeyV,
            0x30 => KeyCode::KeyB,
            0x31 => KeyCode::KeyN,
            0x32 => KeyCode::KeyM,
            0x33 => KeyCode::Comma,
            0x34 => KeyCode::Period,
            0x35 => KeyCode::Slash,
            0x36 => KeyCode::RightShift,
            0x37 => KeyCode::PrintScreen,
            0x38 => KeyCode::LeftAlt,
            0x39 => KeyCode::Space,
            0x3A => KeyCode::CapsLock,
            0x3B => KeyCode::F1,
            0x3C => KeyCode::F2,
            0x3D => KeyCode::F3,
            0x3E => KeyCode::F4,
            0x3F => KeyCode::F5,
            0x40 => KeyCode::F6,
            0x41 => KeyCode::F7,
            0x42 => KeyCode::F8,
            0x43 => KeyCode::F9,
            0x44 => KeyCode::F10,
            0x45 => KeyCode::NumLock,
            0x46 => KeyCode::ScrollLock,
            0x47 => KeyCode::Num7,
            0x48 => KeyCode::Num8,
            0x49 => KeyCode::Num9,
            0x4A => KeyCode::Subtract,
            0x4B => KeyCode::Num4,
            0x4C => KeyCode::Num5,
            0x4D => KeyCode::Num6,
            0x4E => KeyCode::Add,
            0x4F => KeyCode::Num1,
            0x50 => KeyCode::Num2,
            0x51 => KeyCode::Num3,
            0x52 => KeyCode::Num0,
            0x53 => KeyCode::Decimal,
            _ => KeyCode::Escape, // Unknown key
        };
        
        // Handle modifier keys
        match key_code {
            KeyCode::LeftShift => modifiers.shift = is_pressed,
            KeyCode::RightShift => modifiers.shift = is_pressed,
            KeyCode::LeftControl => modifiers.ctrl = is_pressed,
            KeyCode::RightControl => modifiers.ctrl = is_pressed,
            KeyCode::LeftAlt => modifiers.alt = is_pressed,
            KeyCode::RightAlt => modifiers.alt = is_pressed,
            KeyCode::CapsLock if is_pressed => {
                *self.caps_lock_state.lock() = !*self.caps_lock_state.lock();
                modifiers.caps_lock = *self.caps_lock_state.lock();
            }
            KeyCode::NumLock if is_pressed => {
                modifiers.num_lock = !modifiers.num_lock;
            }
            KeyCode::ScrollLock if is_pressed => {
                modifiers.scroll_lock = !modifiers.scroll_lock;
            }
            _ => {}
        }
        
        // Update global modifiers
        *self.modifiers.lock() = modifiers;
        
        // Create and return key event
        Some(KeyEvent {
            key_code,
            is_pressed,
            modifiers,
            timestamp: crate::timer::TimerManager::get_global_tick_count().unwrap_or(0),
        })
    }

    /// Add key event to queue
    pub fn add_key_event(&self, event: KeyEvent) {
        let mut queue = self.key_queue.lock();
        queue.push_back(event);
        
        // Limit queue size to prevent memory issues
        if queue.len() > 100 {
            queue.pop_front();
        }
    }

    /// Get next key event (non-blocking)
    pub fn get_key_event(&self) -> Option<KeyEvent> {
        self.key_queue.lock().pop_front()
    }

    /// Check if key event queue has events
    pub fn has_events(&self) -> bool {
        !self.key_queue.lock().is_empty()
    }

    /// Convert key code to character
    pub fn key_code_to_char(&self, key_code: KeyCode, modifiers: KeyModifiers) -> Option<char> {
        let shift = modifiers.shift;
        let caps_lock = modifiers.caps_lock;
        
        // Handle caps lock interaction
        let use_shift = shift ^ caps_lock;
        
        match key_code {
            KeyCode::Digit1 => Some(if use_shift { '!' } else { '1' }),
            KeyCode::Digit2 => Some(if use_shift { '@' } else { '2' }),
            KeyCode::Digit3 => Some(if use_shift { '#' } else { '3' }),
            KeyCode::Digit4 => Some(if use_shift { '$' } else { '4' }),
            KeyCode::Digit5 => Some(if use_shift { '%' } else { '5' }),
            KeyCode::Digit6 => Some(if use_shift { '^' } else { '6' }),
            KeyCode::Digit7 => Some(if use_shift { '&' } else { '7' }),
            KeyCode::Digit8 => Some(if use_shift { '*' } else { '8' }),
            KeyCode::Digit9 => Some(if use_shift { '(' } else { '9' }),
            KeyCode::Digit0 => Some(if use_shift { ')' } else { '0' }),
            KeyCode::Minus => Some(if use_shift { '_' } else { '-' }),
            KeyCode::Equal => Some(if use_shift { '+' } else { '=' }),
            KeyCode::Backspace => None,
            KeyCode::Tab => Some('\t'),
            KeyCode::KeyQ => Some(if use_shift { 'Q' } else { 'q' }),
            KeyCode::KeyW => Some(if use_shift { 'W' } else { 'w' }),
            KeyCode::KeyE => Some(if use_shift { 'E' } else { 'e' }),
            KeyCode::KeyR => Some(if use_shift { 'R' } else { 'r' }),
            KeyCode::KeyT => Some(if use_shift { 'T' } else { 't' }),
            KeyCode::KeyY => Some(if use_shift { 'Y' } else { 'y' }),
            KeyCode::KeyU => Some(if use_shift { 'U' } else { 'u' }),
            KeyCode::KeyI => Some(if use_shift { 'I' } else { 'i' }),
            KeyCode::KeyO => Some(if use_shift { 'O' } else { 'o' }),
            KeyCode::KeyP => Some(if use_shift { 'P' } else { 'p' }),
            KeyCode::LeftBracket => Some(if use_shift { '{' } else { '[' }),
            KeyCode::RightBracket => Some(if use_shift { '}' } else { ']' }),
            KeyCode::Backslash => Some(if use_shift { '|' } else { '\\' }),
            KeyCode::KeyA => Some(if use_shift { 'A' } else { 'a' }),
            KeyCode::KeyS => Some(if use_shift { 'S' } else { 's' }),
            KeyCode::KeyD => Some(if use_shift { 'D' } else { 'd' }),
            KeyCode::KeyF => Some(if use_shift { 'F' } else { 'f' }),
            KeyCode::KeyG => Some(if use_shift { 'G' } else { 'g' }),
            KeyCode::KeyH => Some(if use_shift { 'H' } else { 'h' }),
            KeyCode::KeyJ => Some(if use_shift { 'J' } else { 'j' }),
            KeyCode::KeyK => Some(if use_shift { 'K' } else { 'k' }),
            KeyCode::KeyL => Some(if use_shift { 'L' } else { 'l' }),
            KeyCode::Semicolon => Some(if use_shift { ':' } else { ';' }),
            KeyCode::Quote => Some(if use_shift { '"' } else { '\'' }),
            KeyCode::Backquote => Some(if use_shift { '~' } else { '`' }),
            KeyCode::KeyZ => Some(if use_shift { 'Z' } else { 'z' }),
            KeyCode::KeyX => Some(if use_shift { 'X' } else { 'x' }),
            KeyCode::KeyC => Some(if use_shift { 'C' } else { 'c' }),
            KeyCode::KeyV => Some(if use_shift { 'V' } else { 'v' }),
            KeyCode::KeyB => Some(if use_shift { 'B' } else { 'b' }),
            KeyCode::KeyN => Some(if use_shift { 'N' } else { 'n' }),
            KeyCode::KeyM => Some(if use_shift { 'M' } else { 'm' }),
            KeyCode::Comma => Some(if use_shift { '<' } else { ',' }),
            KeyCode::Period => Some(if use_shift { '>' } else { '.' }),
            KeyCode::Slash => Some(if use_shift { '?' } else { '/' }),
            KeyCode::Space => Some(' '),
            KeyCode::Enter => Some('\n'),
            _ => None,
        }
    }
}

impl DeviceDriver for Ps2Keyboard {
    fn name(&self) -> &'static str {
        "PS/2 Keyboard Driver"
    }

    fn supported_devices(&self) -> &[DeviceType] {
        &[DeviceType::Keyboard]
    }

    fn init(&self, device: &Device) -> DriverResult<()> {
        info!("Initializing PS/2 keyboard: {}", device.info.name);
        
        // Extract port from hardware address
        let port = match device.hardware_addr {
            crate::device::HardwareAddress::Port(port) => port,
            _ => return Err(DriverError::HardwareError),
        };
        
        // Port should match our expected port
        if port != self.port {
            return Err(DriverError::HardwareError);
        }
        
        self.init()
    }

    fn remove(&self, device: &Device) -> DriverResult<()> {
        info!("Removing PS/2 keyboard: {}", device.info.name);
        
        // Disable keyboard
        let _ = self.send_command(0xF5);
        
        Ok(())
    }

    fn read(&self, device: &Device, buffer: &mut [u8]) -> DriverResult<usize> {
        if let Some(event) = self.get_key_event() {
            if buffer.len() >= core::mem::size_of::<KeyEvent>() {
                let event_bytes = unsafe {
                    core::slice::from_raw_parts(
                        &event as *const KeyEvent as *const u8,
                        core::mem::size_of::<KeyEvent>(),
                    )
                };
                buffer[..event_bytes.len()].copy_from_slice(event_bytes);
                Ok(event_bytes.len())
            } else {
                Err(DriverError::PermissionDenied)
            }
        } else {
            Err(DriverError::DeviceNotFound)
        }
    }

    fn write(&self, device: &Device, buffer: &[u8]) -> DriverResult<usize> {
        // Keyboard is read-only from application perspective
        // Control commands could be sent here
        warn!("Write to keyboard device not supported");
        Err(DriverError::PermissionDenied)
    }

    fn ioctl(&self, device: &Device, command: u32, data: usize) -> DriverResult<usize> {
        match command {
            0x4001 => Ok(self.has_events() as usize), // Check if events available
            0x4002 => { // Get modifiers state
                let modifiers = *self.modifiers.lock();
                let state = (modifiers.shift as usize) |
                          (modifiers.ctrl as usize) << 1 |
                          (modifiers.alt as usize) << 2 |
                          (modifiers.caps_lock as usize) << 3 |
                          (modifiers.num_lock as usize) << 4 |
                          (modifiers.scroll_lock as usize) << 5;
                Ok(state)
            }
            0x4003 => { // Clear key queue
                self.key_queue.lock().clear();
                Ok(0)
            }
            0x4004 => Ok(self.key_queue.lock().len()), // Get queue size
            _ => Err(DriverError::PermissionDenied),
        }
    }

    fn capabilities(&self) -> DeviceCapabilities {
        DeviceCapabilities::READ | DeviceCapabilities::INTERRUPT
    }
}

/// USB Keyboard Driver (simplified)
pub struct UsbKeyboard {
    pub bus_id: u8,
    pub device_id: u8,
    pub key_queue: Mutex<VecDeque<KeyEvent>>,
    pub modifiers: Mutex<KeyModifiers>,
}

impl UsbKeyboard {
    /// Create new USB keyboard driver
    pub fn new(bus_id: u8, device_id: u8) -> Self {
        Self {
            bus_id,
            device_id,
            key_queue: Mutex::new(VecDeque::new()),
            modifiers: Mutex::new(KeyModifiers::default()),
        }
    }

    /// Initialize USB keyboard
    pub fn init(&self) -> DriverResult<()> {
        info!("Initializing USB keyboard on bus {}, device {}", self.bus_id, self.device_id);
        
        // USB keyboard initialization would be more complex in reality
        // involving USB protocol initialization, endpoint configuration, etc.
        
        info!("USB keyboard initialized");
        Ok(())
    }
}

impl DeviceDriver for UsbKeyboard {
    fn name(&self) -> &'static str {
        "USB Keyboard Driver"
    }

    fn supported_devices(&self) -> &[DeviceType] {
        &[DeviceType::Keyboard]
    }

    fn init(&self, device: &Device) -> DriverResult<()> {
        info!("Initializing USB keyboard: {}", device.info.name);
        self.init()
    }

    fn remove(&self, device: &Device) -> DriverResult<()> {
        info!("Removing USB keyboard: {}", device.info.name);
        Ok(())
    }

    fn read(&self, device: &Device, buffer: &mut [u8]) -> DriverResult<usize> {
        if let Some(event) = self.key_queue.lock().pop_front() {
            if buffer.len() >= core::mem::size_of::<KeyEvent>() {
                let event_bytes = unsafe {
                    core::slice::from_raw_parts(
                        &event as *const KeyEvent as *const u8,
                        core::mem::size_of::<KeyEvent>(),
                    )
                };
                buffer[..event_bytes.len()].copy_from_slice(event_bytes);
                Ok(event_bytes.len())
            } else {
                Err(DriverError::PermissionDenied)
            }
        } else {
            Err(DriverError::DeviceNotFound)
        }
    }

    fn write(&self, device: &Device, buffer: &[u8]) -> DriverResult<usize> {
        warn!("Write to USB keyboard device not supported");
        Err(DriverError::PermissionDenied)
    }

    fn ioctl(&self, device: &Device, command: u32, data: usize) -> DriverResult<usize> {
        match command {
            0x4001 => Ok(self.key_queue.lock().len()),
            0x4002 => Ok(0), // USB keyboard modifiers
            0x4003 => {
                self.key_queue.lock().clear();
                Ok(0)
            }
            _ => Err(DriverError::PermissionDenied),
        }
    }

    fn capabilities(&self) -> DeviceCapabilities {
        DeviceCapabilities::READ | DeviceCapabilities::INTERRUPT | DeviceCapabilities::HOT_PLUG
    }
}

/// Global keyboard instance
static KEYBOARD: Once<Mutex<Box<dyn KeyboardDriver + Send + Sync>>> = Once::new();

/// Keyboard driver trait for abstraction
pub trait KeyboardDriver {
    fn name(&self) -> &'static str;
    fn get_key_event(&self) -> Option<KeyEvent>;
    fn has_events(&self) -> bool;
    fn get_modifiers(&self) -> KeyModifiers;
    fn key_code_to_char(&self, key_code: KeyCode, modifiers: KeyModifiers) -> Option<char>;
}

/// Initialize global keyboard system
pub fn init_global_keyboard(keyboard: Box<dyn KeyboardDriver + Send + Sync>) {
    KEYBOARD.call_once(|| Mutex::new(keyboard));
}

/// Get global keyboard reference
pub fn get_global_keyboard() -> Option<spin::MutexGuard<dyn KeyboardDriver + Send + Sync>> {
    KEYBOARD.get().map(|keyboard| keyboard.lock())
}

/// Get next key event from global keyboard
pub fn get_global_key_event() -> Option<KeyEvent> {
    get_global_keyboard().and_then(|kb| kb.get_key_event())
}

/// Check if global keyboard has events
pub fn global_keyboard_has_events() -> bool {
    get_global_keyboard().map_or(false, |kb| kb.has_events())
}

impl KeyboardDriver for Ps2Keyboard {
    fn name(&self) -> &'static str {
        "PS/2 Keyboard"
    }

    fn get_key_event(&self) -> Option<KeyEvent> {
        self.get_key_event()
    }

    fn has_events(&self) -> bool {
        self.has_events()
    }

    fn get_modifiers(&self) -> KeyModifiers {
        *self.modifiers.lock()
    }

    fn key_code_to_char(&self, key_code: KeyCode, modifiers: KeyModifiers) -> Option<char> {
        self.key_code_to_char(key_code, modifiers)
    }
}

impl KeyboardDriver for UsbKeyboard {
    fn name(&self) -> &'static str {
        "USB Keyboard"
    }

    fn get_key_event(&self) -> Option<KeyEvent> {
        self.key_queue.lock().pop_front()
    }

    fn has_events(&self) -> bool {
        !self.key_queue.lock().is_empty()
    }

    fn get_modifiers(&self) -> KeyModifiers {
        *self.modifiers.lock()
    }

    fn key_code_to_char(&self, _key_code: KeyCode, _modifiers: KeyModifiers) -> Option<char> {
        // USB keyboard character conversion would be similar to PS/2
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ps2_keyboard_creation() {
        let keyboard = Ps2Keyboard::new(0x60);
        assert_eq!(keyboard.port, 0x60);
        assert_eq!(keyboard.keyboard_type, KeyboardType::Ps2Keyboard);
    }

    #[test]
    fn test_key_modifiers() {
        let mut modifiers = KeyModifiers::default();
        assert!(!modifiers.is_active());
        
        modifiers.shift = true;
        assert!(modifiers.is_active());
        
        modifiers.ctrl = true;
        assert!(modifiers.is_active());
    }

    #[test]
    fn test_key_code_conversion() {
        let keyboard = Ps2Keyboard::new(0x60);
        let modifiers = KeyModifiers { shift: true, ..Default::default() };
        
        assert_eq!(keyboard.key_code_to_char(KeyCode::Digit1, modifiers), Some('!'));
        assert_eq!(keyboard.key_code_to_char(KeyCode::Digit2, modifiers), Some('@'));
        
        let modifiers = KeyModifiers::default();
        assert_eq!(keyboard.key_code_to_char(KeyCode::KeyA, modifiers), Some('a'));
        
        let modifiers = KeyModifiers { caps_lock: true, ..Default::default() };
        assert_eq!(keyboard.key_code_to_char(KeyCode::KeyA, modifiers), Some('A'));
    }

    #[test]
    fn test_key_events() {
        let keyboard = Ps2Keyboard::new(0x60);
        
        let event = KeyEvent {
            key_code: KeyCode::KeyA,
            is_pressed: true,
            modifiers: KeyModifiers::default(),
            timestamp: 0,
        };
        
        keyboard.add_key_event(event);
        assert!(keyboard.has_events());
        
        let retrieved = keyboard.get_key_event().unwrap();
        assert_eq!(retrieved.key_code, KeyCode::KeyA);
        assert!(retrieved.is_pressed);
    }

    #[test]
    fn test_keyboard_capabilities() {
        let keyboard = Ps2Keyboard::new(0x60);
        let caps = keyboard.capabilities();
        
        assert!(caps.contains(DeviceCapabilities::READ));
        assert!(caps.contains(DeviceCapabilities::INTERRUPT));
        assert!(!caps.contains(DeviceCapabilities::WRITE));
    }

    #[test]
    fn test_scan_code_processing() {
        let keyboard = Ps2Keyboard::new(0x60);
        
        // Test common scan code
        if let Some(event) = keyboard.process_scan_code(0x1E) {
            assert_eq!(event.key_code, KeyCode::KeyA);
            assert!(event.is_pressed);
        } else {
            panic!("Failed to process scan code for 'A' key");
        }
    }
}