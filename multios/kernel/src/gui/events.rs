//! Event system for GUI toolkit
//! 
//! Provides event handling, dispatching, and event types for GUI interactions.

use alloc::vec::Vec;
use alloc::boxed::Box;
use spin::Mutex;
use core::any::Any;

use super::{WidgetId, GUIResult, GUIError};

/// Event types supported by the GUI system
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventType {
    // Mouse events
    MouseMove,
    MouseDown,
    MouseUp,
    MouseClick,
    MouseDoubleClick,
    MouseEnter,
    MouseLeave,
    
    // Keyboard events
    KeyDown,
    KeyUp,
    KeyPress,
    
    // Window events
    Paint,
    Resize,
    Move,
    Show,
    Hide,
    Close,
    
    // Focus events
    Focus,
    Blur,
    
    // Custom events
    Custom,
}

/// Event structure containing event data and metadata
#[derive(Debug, Clone)]
pub struct Event {
    pub event_type: EventType,
    pub widget_id: WidgetId,
    pub timestamp: u64,
    pub consumed: bool,
    pub data: EventData,
}

impl Event {
    pub fn new(event_type: EventType, widget_id: WidgetId) -> Self {
        Self {
            event_type,
            widget_id,
            timestamp: 0, // Would be set by event system
            consumed: false,
            data: EventData::Empty,
        }
    }

    pub fn consume(&mut self) {
        self.consumed = true;
    }

    pub fn is_consumed(&self) -> bool {
        self.consumed
    }

    pub fn set_data(&mut self, data: EventData) {
        self.data = data;
    }

    pub fn get_data(&self) -> &EventData {
        &self.data
    }
}

/// Event data variants for different event types
#[derive(Debug, Clone)]
pub enum EventData {
    Empty,
    
    // Mouse event data
    Mouse(MouseEventData),
    
    // Keyboard event data
    Keyboard(KeyboardEventData),
    
    // Window event data
    Window(WindowEventData),
    
    // Custom event data (for application-specific events)
    Custom(Box<dyn Any>),
}

/// Mouse event data
#[derive(Debug, Clone)]
pub struct MouseEventData {
    pub position: super::graphics::Point,
    pub button: MouseButton,
    pub button_count: u8,
    pub modifiers: u8, // Ctrl, Alt, Shift flags
}

/// Mouse button types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    None,
}

/// Keyboard event data
#[derive(Debug, Clone)]
pub struct KeyboardEventData {
    pub key_code: u8,
    pub key_char: char,
    pub modifiers: u8, // Ctrl, Alt, Shift flags
}

/// Window event data
#[derive(Debug, Clone)]
pub struct WindowEventData {
    pub width: u32,
    pub height: u32,
    pub position: super::graphics::Point,
}

/// Event handler trait for widgets
pub trait EventHandler {
    /// Handle an event
    fn handle_event(&self, event: &mut Event) -> bool;
    
    /// Check if this handler can handle the given event type
    fn can_handle(&self, event_type: EventType) -> bool;
}

/// Event handler wrapper
pub struct WrappedHandler {
    handler: Box<dyn EventHandler>,
}

impl WrappedHandler {
    pub fn new(handler: Box<dyn EventHandler>) -> Self {
        Self { handler }
    }
}

/// Event dispatcher for managing event propagation
pub struct EventDispatcher {
    handlers: Vec<(EventType, WrappedHandler)>,
}

impl EventDispatcher {
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
        }
    }

    /// Add an event handler for a specific event type
    pub fn add_handler(&mut self, event_type: EventType, handler: Box<dyn EventHandler>) {
        self.handlers.push((event_type, WrappedHandler::new(handler)));
    }

    /// Remove all handlers for a specific event type
    pub fn remove_handlers(&mut self, event_type: EventType) {
        self.handlers.retain(|(et, _)| *et != event_type);
    }

    /// Dispatch an event to all relevant handlers
    pub fn dispatch(&mut self, event: &mut Event) {
        for (event_type, wrapped_handler) in &self.handlers.iter() {
            if *event_type == event.event_type && wrapped_handler.handler.can_handle(event.event_type) {
                if wrapped_handler.handler.handle_event(event) {
                    // Handler handled the event, check if it was consumed
                    if event.is_consumed() {
                        break;
                    }
                }
            }
        }
    }

    /// Clear all handlers
    pub fn clear(&mut self) {
        self.handlers.clear();
    }
}

/// Event queue for storing pending events
pub struct EventQueue {
    events: Vec<Event>,
    max_size: usize,
}

impl EventQueue {
    pub fn new(max_size: usize) -> Self {
        Self {
            events: Vec::new(),
            max_size,
        }
    }

    /// Add an event to the queue
    pub fn enqueue(&mut self, event: Event) -> GUIResult<()> {
        if self.events.len() >= self.max_size {
            return Err(GUIError::EventError);
        }
        self.events.push(event);
        Ok(())
    }

    /// Remove and return the next event from the queue
    pub fn dequeue(&mut self) -> Option<Event> {
        if self.events.is_empty() {
            None
        } else {
            Some(self.events.remove(0))
        }
    }

    /// Check if the queue is empty
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Get the current queue size
    pub fn size(&self) -> usize {
        self.events.len()
    }

    /// Clear all events from the queue
    pub fn clear(&mut self) {
        self.events.clear();
    }

    /// Get the next event without removing it
    pub fn peek(&self) -> Option<&Event> {
        self.events.first()
    }
}

/// Global event queue instance
static EVENT_QUEUE: Mutex<EventQueue> = Mutex::new(EventQueue::new(1000));

/// Event system for managing event processing
pub struct EventSystem {
    event_queue: &'static Mutex<EventQueue>,
}

impl EventSystem {
    pub fn new() -> Self {
        Self {
            event_queue: &EVENT_QUEUE,
        }
    }

    /// Post an event to the global event queue
    pub fn post_event(event: Event) -> GUIResult<()> {
        let mut queue = EVENT_QUEUE.lock();
        queue.enqueue(event)
    }

    /// Get the next event from the global event queue
    pub fn get_next_event() -> Option<Event> {
        let mut queue = EVENT_QUEUE.lock();
        queue.dequeue()
    }

    /// Check if there are pending events
    pub fn has_events() -> bool {
        let queue = EVENT_QUEUE.lock();
        !queue.is_empty()
    }

    /// Clear all pending events
    pub fn clear_events() {
        let mut queue = EVENT_QUEUE.lock();
        queue.clear();
    }

    /// Process all pending events with a dispatcher
    pub fn process_events(dispatcher: &mut EventDispatcher) {
        let mut queue = EVENT_QUEUE.lock();
        
        while let Some(mut event) = queue.dequeue() {
            // Process the event
            dispatcher.dispatch(&mut event);
            
            // If event wasn't consumed and needs further processing, put it back
            if !event.is_consumed() && needs_continued_processing(event.event_type) {
                // Re-enqueue for further processing
                if queue.size() < queue.max_size {
                    queue.events.push(event);
                }
            }
        }
    }

    /// Process events with a timeout (for real-time systems)
    pub fn process_events_with_timeout(dispatcher: &mut EventDispatcher, timeout_ms: u32) {
        let start_time = get_current_time();
        let timeout_us = (timeout_ms as u64) * 1000;
        
        let mut queue = EVENT_QUEUE.lock();
        let mut processed_count = 0;
        
        while !queue.is_empty() {
            if let Some(mut event) = queue.dequeue() {
                dispatcher.dispatch(&mut event);
                processed_count += 1;
                
                // Check timeout
                if get_current_time() - start_time > timeout_us {
                    break;
                }
                
                // If event wasn't consumed and needs continued processing
                if !event.is_consumed() && needs_continued_processing(event.event_type) {
                    if queue.size() < queue.max_size {
                        queue.events.push(event);
                    }
                }
            } else {
                break;
            }
        }
    }
}

/// Helper function to check if an event type needs continued processing
fn needs_continued_processing(event_type: EventType) -> bool {
    matches!(event_type, EventType::MouseMove)
}

/// Helper function to get current time (simplified)
fn get_current_time() -> u64 {
    0 // Would be replaced with actual timer implementation
}

/// Convenience functions for creating common events

/// Create a mouse move event
pub fn create_mouse_move_event(widget_id: WidgetId, position: super::graphics::Point) -> Event {
    let mut event = Event::new(EventType::MouseMove, widget_id);
    event.set_data(EventData::Mouse(MouseEventData {
        position,
        button: MouseButton::None,
        button_count: 0,
        modifiers: 0,
    }));
    event
}

/// Create a mouse down event
pub fn create_mouse_down_event(widget_id: WidgetId, position: super::graphics::Point, button: MouseButton) -> Event {
    let mut event = Event::new(EventType::MouseDown, widget_id);
    event.set_data(EventData::Mouse(MouseEventData {
        position,
        button,
        button_count: 1,
        modifiers: 0,
    }));
    event
}

/// Create a mouse up event
pub fn create_mouse_up_event(widget_id: WidgetId, position: super::graphics::Point, button: MouseButton) -> Event {
    let mut event = Event::new(EventType::MouseUp, widget_id);
    event.set_data(EventData::Mouse(MouseEventData {
        position,
        button,
        button_count: 1,
        modifiers: 0,
    }));
    event
}

/// Create a key down event
pub fn create_key_down_event(widget_id: WidgetId, key_code: u8, key_char: char) -> Event {
    let mut event = Event::new(EventType::KeyDown, widget_id);
    event.set_data(EventData::Keyboard(KeyboardEventData {
        key_code,
        key_char,
        modifiers: 0,
    }));
    event
}

/// Create a key up event
pub fn create_key_up_event(widget_id: WidgetId, key_code: u8, key_char: char) -> Event {
    let mut event = Event::new(EventType::KeyUp, widget_id);
    event.set_data(EventData::Keyboard(KeyboardEventData {
        key_code,
        key_char,
        modifiers: 0,
    }));
    event
}

/// Create a paint event
pub fn create_paint_event(widget_id: WidgetId) -> Event {
    Event::new(EventType::Paint, widget_id)
}