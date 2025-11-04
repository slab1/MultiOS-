//! Event management system for windows

use crate::*;

/// Event types handled by window manager
#[derive(Debug, Clone)]
pub enum Event {
    // Window lifecycle events
    WindowCreated { window_id: WindowId },
    WindowDestroyed { window_id: WindowId },
    WindowMinimized { window_id: WindowId },
    WindowMaximized { window_id: WindowId },
    WindowRestored { window_id: WindowId },
    
    // Window state events
    WindowMoved { window_id: WindowId, position: Point },
    WindowResized { window_id: WindowId, size: Size },
    WindowFocused { window_id: WindowId },
    WindowUnfocused { window_id: WindowId },
    
    // Z-order events
    WindowBroughtToFront { window_id: WindowId },
    WindowSentToBack { window_id: WindowId },
    
    // Mouse events
    MouseClicked { window_id: WindowId, position: Point, button: MouseButton },
    MouseReleased { window_id: WindowId, position: Point, button: MouseButton },
    MouseMoved { window_id: WindowId, position: Point },
    MouseWheel { window_id: WindowId, delta: i32 },
    
    // Keyboard events
    KeyPressed { window_id: WindowId, key: KeyCode, modifiers: KeyModifiers },
    KeyReleased { window_id: WindowId, key: KeyCode, modifiers: KeyModifiers },
    
    // System events
    WorkspaceChanged { from_workspace: usize, to_workspace: usize },
    FocusChanged { old_focus: Option<WindowId>, new_focus: Option<WindowId> },
}

/// Mouse button types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    X1,
    X2,
}

/// Key codes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyCode {
    // Function keys
    Escape,
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
    
    // Number keys
    Num0, Num1, Num2, Num3, Num4, Num5, Num6, Num7, Num8, Num9,
    
    // Letter keys
    A, B, C, D, E, F, G, H, I, J, K, L, M,
    N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
    
    // Symbol keys
    Backquote, Hyphen, Equal, Backspace,
    Tab, LeftBracket, RightBracket, Backslash,
    Semicolon, Quote, Enter,
    Comma, Period, Slash,
    
    // Control keys
    CapsLock, ShiftLeft, ShiftRight, ControlLeft, ControlRight,
    AltLeft, AltRight, Space, MetaLeft, MetaRight,
    
    // Navigation keys
    ArrowUp, ArrowDown, ArrowLeft, ArrowRight,
    Home, End, PageUp, PageDown,
    
    // Editing keys
    Insert, Delete,
    
    // Custom keys (extendable)
    Other(char),
}

/// Keyboard modifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct KeyModifiers {
    pub shift: bool,
    pub control: bool,
    pub alt: bool,
    pub meta: bool, // Windows key or Command key
}

impl KeyModifiers {
    pub fn new(shift: bool, control: bool, alt: bool, meta: bool) -> Self {
        Self { shift, control, alt, meta }
    }

    pub fn none() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        !self.shift && !self.control && !self.alt && !self.meta
    }

    pub fn with_shift(mut self) -> Self {
        self.shift = true;
        self
    }

    pub fn with_control(mut self) -> Self {
        self.control = true;
        self
    }

    pub fn with_alt(mut self) -> Self {
        self.alt = true;
        self
    }

    pub fn with_meta(mut self) -> Self {
        self.meta = true;
        self
    }
}

/// Event queue manager
#[derive(Debug)]
pub struct EventManager {
    events: Vec<Event>,
    pending_events: Vec<Event>,
    event_handlers: HashMap<EventType, Vec<EventHandler>>,
}

/// Event type for dispatching
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventType {
    WindowCreated,
    WindowDestroyed,
    WindowMoved,
    WindowResized,
    WindowFocused,
    MouseClicked,
    MouseMoved,
    KeyPressed,
    KeyReleased,
    WorkspaceChanged,
}

/// Event handler function
pub type EventHandler = Box<dyn FnMut(&Event) + Send + Sync>;

impl EventManager {
    /// Create a new event manager
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            pending_events: Vec::new(),
            event_handlers: HashMap::new(),
        }
    }

    /// Send event to be processed
    pub fn send_event(&mut self, _window_id: WindowId, event: Event) {
        self.pending_events.push(event);
    }

    /// Process all pending events
    pub fn drain_events(&mut self) -> Vec<Event> {
        // Move pending events to main queue
        self.events.extend(self.pending_events.drain(..));
        
        // Process events with handlers
        let mut processed_events = Vec::new();
        let mut i = 0;
        
        while i < self.events.len() {
            let event = self.events[i].clone();
            let event_type = self.get_event_type(&event);
            
            // Call registered handlers
            if let Some(handlers) = self.event_handlers.get_mut(&event_type) {
                for handler in handlers.iter_mut() {
                    handler(&event);
                }
            }
            
            processed_events.push(event);
            i += 1;
        }

        // Clear processed events
        self.events.clear();
        processed_events
    }

    /// Register event handler
    pub fn register_handler<F>(&mut self, event_type: EventType, handler: F)
    where
        F: FnMut(&Event) + Send + Sync + 'static,
    {
        let handler = Box::new(handler) as EventHandler;
        self.event_handlers
            .entry(event_type)
            .or_insert_with(Vec::new)
            .push(handler);
    }

    /// Unregister all handlers for event type
    pub fn unregister_handlers(&mut self, event_type: EventType) {
        self.event_handlers.remove(&event_type);
    }

    /// Get pending event count
    pub fn pending_count(&self) -> usize {
        self.pending_events.len()
    }

    /// Clear all events
    pub fn clear(&mut self) {
        self.events.clear();
        self.pending_events.clear();
    }

    /// Check if event matches type
    fn get_event_type(&self, event: &Event) -> EventType {
        match event {
            Event::WindowCreated { .. } => EventType::WindowCreated,
            Event::WindowDestroyed { .. } => EventType::WindowDestroyed,
            Event::WindowMoved { .. } => EventType::WindowMoved,
            Event::WindowResized { .. } => EventType::WindowResized,
            Event::WindowFocused { .. } => EventType::WindowFocused,
            Event::MouseClicked { .. } => EventType::MouseClicked,
            Event::MouseMoved { .. } => EventType::MouseMoved,
            Event::KeyPressed { .. } => EventType::KeyPressed,
            Event::KeyReleased { .. } => EventType::KeyReleased,
            Event::WorkspaceChanged { .. } => EventType::WorkspaceChanged,
            _ => EventType::WindowCreated, // Default fallback
        }
    }
}

impl Default for EventManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Utility functions for event creation
impl EventManager {
    /// Create window created event
    pub fn create_window_created_event(window_id: WindowId) -> Event {
        Event::WindowCreated { window_id }
    }

    /// Create mouse clicked event
    pub fn create_mouse_click_event(
        window_id: WindowId, 
        position: Point, 
        button: MouseButton
    ) -> Event {
        Event::MouseClicked { window_id, position, button }
    }

    /// Create key pressed event
    pub fn create_key_pressed_event(
        window_id: WindowId, 
        key: KeyCode, 
        modifiers: KeyModifiers
    ) -> Event {
        Event::KeyPressed { window_id, key, modifiers }
    }
}

/// Hit testing utilities
pub struct HitTestResult {
    pub window_id: WindowId,
    pub hit_part: HitTestPart,
    pub position: Point,
}

/// Parts of a window that can be hit tested
#[derive(Debug, Clone, Copy)]
pub enum HitTestPart {
    Client,        // Window content area
    TitleBar,      // Title bar area
    CloseButton,   // Close button
    MinimizeButton, // Minimize button
    MaximizeButton, // Maximize button
    TopBorder,     // Top border (resize)
    BottomBorder,  // Bottom border (resize)
    LeftBorder,    // Left border (resize)
    RightBorder,   // Right border (resize)
    TopLeftCorner, // Top-left corner (resize)
    TopRightCorner, // Top-right corner (resize)
    BottomLeftCorner, // Bottom-left corner (resize)
    BottomRightCorner, // Bottom-right corner (resize)
}

/// Hit test a window at given position
pub fn hit_test_window(
    window: &Window,
    position: Point
) -> Option<HitTestPart> {
    if !window.contains_point(position) {
        return None;
    }

    // Check decorations first
    if let Some(title_bar) = window.close_button_bounds() {
        if title_bar.contains_point(position) {
            return Some(HitTestPart::CloseButton);
        }
    }

    if let Some(title_bar) = window.minimize_button_bounds() {
        if title_bar.contains_point(position) {
            return Some(HitTestPart::MinimizeButton);
        }
    }

    if let Some(title_bar) = window.maximize_button_bounds() {
        if title_bar.contains_point(position) {
            return Some(HitTestPart::MaximizeButton);
        }
    }

    if window.contains_title_bar(position) {
        return Some(HitTestPart::TitleBar);
    }

    // Check borders and corners for resizing
    let bounds = window.bounds();
    let border_size = 4; // Border thickness in pixels

    // Check corners first
    let corner_size = border_size;
    if position.x < bounds.position.x + corner_size as i32 {
        if position.y < bounds.position.y + corner_size as i32 {
            return Some(HitTestPart::TopLeftCorner);
        }
        if position.y > bounds.position.y + (bounds.size.height as i32) - corner_size as i32 {
            return Some(HitTestPart::BottomLeftCorner);
        }
        return Some(HitTestPart::LeftBorder);
    }

    if position.x > bounds.position.x + (bounds.size.width as i32) - corner_size as i32 {
        if position.y < bounds.position.y + corner_size as i32 {
            return Some(HitTestPart::TopRightCorner);
        }
        if position.y > bounds.position.y + (bounds.size.height as i32) - corner_size as i32 {
            return Some(HitTestPart::BottomRightCorner);
        }
        return Some(HitTestPart::RightBorder);
    }

    // Check top/bottom borders
    if position.y < bounds.position.y + border_size as i32 {
        return Some(HitTestPart::TopBorder);
    }
    if position.y > bounds.position.y + (bounds.size.height as i32) - border_size as i32 {
        return Some(HitTestPart::BottomBorder);
    }

    // Default to client area
    Some(HitTestPart::Client)
}
