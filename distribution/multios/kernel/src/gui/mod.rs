//! MultiOS GUI Toolkit
//! 
//! A basic GUI toolkit providing common UI widgets, event handling,
//! layout management, and styling system for MultiOS applications.

#![no_std]

extern crate log;
use log::info;

pub mod widgets;
pub mod events;
pub mod layout;
pub mod style;
pub mod manager;
pub mod graphics;
pub mod examples;

pub use events::{Event, EventType, EventHandler, EventDispatcher};
pub use layout::{Layout, LayoutManager, FlowLayout, GridLayout, AbsoluteLayout};
pub use style::{Style, StyleManager, Theme, Color, Font, Border};
pub use widgets::{Widget, Container, Button, Label, TextField, Menu, Dialog, ListBox, ProgressBar};
pub use manager::{GUIManager, Window, WindowManager};
pub use graphics::{Renderer, Point, Rectangle, Size};

// Re-export core types
pub use core::option::Option;
pub use alloc::vec::Vec;
pub use alloc::boxed::Box;
pub use alloc::rc::Rc;
pub use alloc::sync::Arc;

/// GUI error types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GUIError {
    InitializationFailed,
    WidgetNotFound,
    InvalidLayout,
    StyleError,
    RenderingError,
    EventError,
}

/// GUI result type
pub type GUIResult<T> = Result<T, GUIError>;

/// GUI widget identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WidgetId(u32);

impl WidgetId {
    pub fn new() -> Self {
        use alloc::sync::atomic::{AtomicU32, Ordering};
        static COUNTER: AtomicU32 = AtomicU32::new(1);
        WidgetId(COUNTER.fetch_add(1, Ordering::SeqCst))
    }
}

/// GUI application identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AppId(u32);

impl AppId {
    pub fn new() -> Self {
        use alloc::sync::atomic::{AtomicU32, Ordering};
        static COUNTER: AtomicU32 = AtomicU32::new(1);
        AppId(COUNTER.fetch_add(1, Ordering::SeqCst))
    }
}

/// Window identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WindowId(u32);

impl WindowId {
    pub fn new() -> Self {
        use alloc::sync::atomic::{AtomicU32, Ordering};
        static COUNTER: AtomicU32 = AtomicU32::new(1);
        WindowId(COUNTER.fetch_add(1, Ordering::SeqCst))
    }
}

/// Initialize the GUI system
pub fn init() -> GUIResult<()> {
    info!("Initializing MultiOS GUI toolkit...");
    
    // Initialize graphics subsystem
    graphics::init()?;
    
    // Initialize GUI manager
    manager::init()?;
    
    // Initialize default theme and styles
    style::init()?;
    
    info!("GUI toolkit initialized successfully");
    Ok(())
}

/// Shutdown the GUI system
pub fn shutdown() -> GUIResult<()> {
    info!("Shutting down GUI toolkit...");
    
    // Shutdown GUI manager
    manager::shutdown()?;
    
    // Shutdown graphics subsystem
    graphics::shutdown()?;
    
    info!("GUI toolkit shutdown complete");
    Ok(())
}