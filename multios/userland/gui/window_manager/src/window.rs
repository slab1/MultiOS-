//! Window types and core window management

use crate::*;

/// Unique window identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WindowId(pub u32);

impl WindowId {
    pub fn new(id: u32) -> Self {
        WindowId(id)
    }

    pub fn next(&self) -> Self {
        WindowId(self.0 + 1)
    }
}

/// 2D Point
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

/// 2D Size
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

impl Size {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

/// Rectangle (position + size)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Rectangle {
    pub position: Point,
    pub size: Size,
}

impl Rectangle {
    pub fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self {
            position: Point::new(x, y),
            size: Size::new(width, height),
        }
    }

    pub fn contains_point(&self, point: Point) -> bool {
        point.x >= self.position.x
            && point.y >= self.position.y
            && (point.x as u32) < self.position.x as u32 + self.size.width
            && (point.y as u32) < self.position.y as u32 + self.size.height
    }
}

/// Window state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowState {
    Normal,
    Minimized,
    Maximized,
}

/// Window style flags
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowStyle {
    pub resizable: bool,
    pub maximizable: bool,
    pub minimizable: bool,
    pub closable: bool,
    pub has_title_bar: bool,
    pub has_border: bool,
    pub always_on_top: bool,
}

impl Default for WindowStyle {
    fn default() -> Self {
        Self {
            resizable: true,
            maximizable: true,
            minimizable: true,
            closable: true,
            has_title_bar: true,
            has_border: true,
            always_on_top: false,
        }
    }
}

/// Window error types
#[derive(Debug, Clone)]
pub enum WindowError {
    WindowNotFound,
    InvalidBounds,
    InvalidWorkspace,
    InvalidState,
    PermissionDenied,
}

/// Window represents a single application window
#[derive(Debug)]
pub struct Window {
    id: WindowId,
    title: String,
    bounds: Rectangle,
    state: WindowState,
    style: WindowStyle,
    is_focused: bool,
    decorations: WindowDecorations,
}

impl Window {
    /// Create a new window
    pub fn new(
        id: WindowId,
        title: String,
        bounds: Rectangle,
        style: WindowStyle,
    ) -> Result<Self, WindowError> {
        // Validate bounds
        if bounds.size.width == 0 || bounds.size.height == 0 {
            return Err(WindowError::InvalidBounds);
        }

        Ok(Self {
            id,
            title,
            bounds,
            state: WindowState::Normal,
            style,
            is_focused: false,
            decorations: WindowDecorations::new(style),
        })
    }

    /// Get window ID
    pub fn id(&self) -> WindowId {
        self.id
    }

    /// Get window title
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Set window title
    pub fn set_title(&mut self, title: String) {
        self.title = title;
    }

    /// Get window bounds
    pub fn bounds(&self) -> Rectangle {
        self.bounds
    }

    /// Set window position
    pub fn set_position(&mut self, position: Point) {
        self.bounds.position = position;
    }

    /// Set window size
    pub fn set_size(&mut self, size: Size) {
        self.bounds.size = size;
    }

    /// Get window state
    pub fn state(&self) -> WindowState {
        self.state
    }

    /// Set window state
    pub fn set_state(&mut self, state: WindowState) {
        self.state = state;
    }

    /// Check if window is resizable
    pub fn is_resizable(&self) -> bool {
        self.style.resizable && self.state == WindowState::Normal
    }

    /// Check if window is maximizable
    pub fn is_maximizable(&self) -> bool {
        self.style.maximizable
    }

    /// Check if window is minimizable
    pub fn is_minimizable(&self) -> bool {
        self.style.minimizable
    }

    /// Check if window is closable
    pub fn is_closable(&self) -> bool {
        self.style.closable
    }

    /// Check if window has focus
    pub fn has_focus(&self) -> bool {
        self.is_focused
    }

    /// Set focus state
    pub fn set_focused(&mut self, focused: bool) {
        self.is_focused = focused;
    }

    /// Get window decorations
    pub fn decorations(&self) -> &WindowDecorations {
        &self.decorations
    }

    /// Check if point is within window bounds
    pub fn contains_point(&self, point: Point) -> bool {
        self.bounds.contains_point(point)
    }

    /// Check if point is within title bar
    pub fn contains_title_bar(&self, point: Point) -> bool {
        if !self.style.has_title_bar || self.state != WindowState::Normal {
            return false;
        }

        self.decorations.title_bar.contains_point(point)
    }

    /// Get close button bounds
    pub fn close_button_bounds(&self) -> Option<Rectangle> {
        if !self.style.has_title_bar {
            return None;
        }
        self.decorations.close_button_bounds()
    }

    /// Get minimize button bounds
    pub fn minimize_button_bounds(&self) -> Option<Rectangle> {
        if !self.style.has_title_bar || !self.style.minimizable {
            return None;
        }
        self.decorations.minimize_button_bounds()
    }

    /// Get maximize button bounds
    pub fn maximize_button_bounds(&self) -> Option<Rectangle> {
        if !self.style.has_title_bar || !self.style.maximizable {
            return None;
        }
        self.decorations.maximize_button_bounds()
    }

    /// Update decorations after bounds change
    pub fn update_decorations(&mut self) {
        self.decorations.update_bounds(self.bounds);
    }
}

/// Window decorations (title bar, buttons, borders)
#[derive(Debug)]
pub struct WindowDecorations {
    title_bar: Rectangle,
    close_button: Rectangle,
    minimize_button: Rectangle,
    maximize_button: Rectangle,
    style: WindowStyle,
}

impl WindowDecorations {
    pub fn new(style: WindowStyle) -> Self {
        let title_bar_height = if style.has_title_bar { 30 } else { 0 };
        
        Self {
            title_bar: Rectangle::new(0, 0, 0, title_bar_height),
            close_button: Rectangle::new(0, 0, 30, title_bar_height),
            minimize_button: Rectangle::new(0, 0, 30, title_bar_height),
            maximize_button: Rectangle::new(0, 0, 30, title_bar_height),
            style,
        }
    }

    /// Update decoration bounds based on window bounds
    pub fn update_bounds(&mut self, window_bounds: Rectangle) {
        self.title_bar.position = window_bounds.position;
        self.title_bar.size = Size::new(window_bounds.size.width, self.title_bar.size.height);

        // Position buttons in title bar
        if self.style.has_title_bar {
            let button_width = 30;
            let padding = 5;
            let mut x = window_bounds.position.x + (window_bounds.size.width as i32) - padding - button_width;

            // Close button (rightmost)
            self.close_button.position = Point::new(x, window_bounds.position.y);
            self.close_button.size = Size::new(button_width, self.title_bar.size.height);

            // Maximize button
            x -= (button_width + padding) as i32;
            self.maximize_button.position = Point::new(x, window_bounds.position.y);
            self.maximize_button.size = Size::new(button_width, self.title_bar.size.height);

            // Minimize button
            x -= (button_width + padding) as i32;
            self.minimize_button.position = Point::new(x, window_bounds.position.y);
            self.minimize_button.size = Size::new(button_width, self.title_bar.size.height);
        }
    }

    /// Get title bar bounds
    pub fn title_bar(&self) -> Rectangle {
        self.title_bar
    }

    /// Get close button bounds
    pub fn close_button_bounds(&self) -> Option<Rectangle> {
        if self.style.has_title_bar {
            Some(self.close_button)
        } else {
            None
        }
    }

    /// Get minimize button bounds
    pub fn minimize_button_bounds(&self) -> Option<Rectangle> {
        if self.style.has_title_bar && self.style.minimizable {
            Some(self.minimize_button)
        } else {
            None
        }
    }

    /// Get maximize button bounds
    pub fn maximize_button_bounds(&self) -> Option<Rectangle> {
        if self.style.has_title_bar && self.style.maximizable {
            Some(self.maximize_button)
        } else {
            None
        }
    }
}
