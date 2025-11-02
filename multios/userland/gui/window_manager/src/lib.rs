//! MultiOS Window Management System
//! 
//! This module provides comprehensive window management functionality including:
//! - Window creation, destruction, movement, and resizing
//! - Z-order management and layering
//! - Window decorations (title bars, borders, controls)
//! - State management (normal, minimized, maximized)
//! - Focus management
//! - Multiple desktop workspaces
//! - Event handling system
//!
//! # Examples
//!
//! Basic window operations:
//! ```no_run
//! use window_manager::*;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut window_manager = WindowManager::new();
//!     
//!     let window_id = window_manager.create_window(
//!         "My Window".to_string(),
////!         Rectangle::new(100, 100, 800, 600),
//!         WindowStyle::default(),
//!     )?;
//!
//!     window_manager.move_window(window_id, Point::new(200, 200))?;
//!     window_manager.set_focus(window_id)?;
//!     
//!     Ok(())
//! }
//! ```
//!
//! Workspace management:
//! ```no_run
//! use window_manager::*;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut window_manager = WindowManager::new();
//!     
//!     // Create additional workspaces
//!     let dev_workspace = window_manager.create_workspace("Development".to_string())?;
//!     let design_workspace = window_manager.create_workspace("Design".to_string())?;
//!     
//!     // Switch workspaces
//!     window_manager.switch_to_workspace(dev_workspace)?;
//!     
//!     Ok(())
//! }
//! ```

pub mod window;
pub mod decoration;
pub mod workspace;
pub mod focus_manager;
pub mod z_order;
pub mod event_manager;

pub use window::*;
pub use decoration::*;
pub use workspace::*;
pub use focus_manager::*;
pub use z_order::*;
pub use event_manager::*;

use std::collections::HashMap;

/// Window Manager - Main orchestrator for all window operations
pub struct WindowManager {
    windows: HashMap<WindowId, Window>,
    z_order: ZOrderManager,
    focus_manager: FocusManager,
    workspaces: Vec<Workspace>,
    active_workspace: usize,
    event_manager: EventManager,
    next_window_id: WindowId,
}

impl WindowManager {
    /// Create a new window manager
    pub fn new() -> Self {
        let mut workspaces = Vec::new();
        workspaces.push(Workspace::new("Desktop 1"));
        
        Self {
            windows: HashMap::new(),
            z_order: ZOrderManager::new(),
            focus_manager: FocusManager::new(),
            workspaces,
            active_workspace: 0,
            event_manager: EventManager::new(),
            next_window_id: WindowId::new(1),
        }
    }

    /// Create a new window
    pub fn create_window(
        &mut self,
        title: String,
        bounds: Rectangle,
        style: WindowStyle,
    ) -> Result<WindowId, WindowError> {
        let window_id = self.next_window_id;
        self.next_window_id = self.next_window_id.next();

        let window = Window::new(window_id, title, bounds, style)?;
        
        self.windows.insert(window_id, window);
        self.z_order.add_window(window_id, self.active_workspace);
        self.focus_manager.set_focus(window_id);
        
        self.event_manager.send_event(
            window_id,
            Event::WindowCreated { window_id }
        );
        
        Ok(window_id)
    }

    /// Destroy a window
    pub fn destroy_window(&mut self, window_id: WindowId) -> Result<(), WindowError> {
        if !self.windows.contains_key(&window_id) {
            return Err(WindowError::WindowNotFound);
        }

        // If this window has focus, remove focus
        if self.focus_manager.has_focus(window_id) {
            self.focus_manager.clear_focus();
        }

        // Remove from z-order
        self.z_order.remove_window(window_id, self.active_workspace);

        // Send destroy event
        self.event_manager.send_event(
            window_id,
            Event::WindowDestroyed { window_id }
        );

        self.windows.remove(&window_id);
        Ok(())
    }

    /// Move a window
    pub fn move_window(&mut self, window_id: WindowId, position: Point) -> Result<(), WindowError> {
        let window = self.windows.get_mut(&window_id)
            .ok_or(WindowError::WindowNotFound)?;
        
        window.set_position(position);
        self.event_manager.send_event(
            window_id,
            Event::WindowMoved { 
                window_id, 
                position: window.bounds().position 
            }
        );
        Ok(())
    }

    /// Resize a window
    pub fn resize_window(
        &mut self, 
        window_id: WindowId, 
        size: Size
    ) -> Result<(), WindowError> {
        let window = self.windows.get_mut(&window_id)
            .ok_or(WindowError::WindowNotFound)?;
        
        window.set_size(size);
        self.event_manager.send_event(
            window_id,
            Event::WindowResized { 
                window_id, 
                size: window.bounds().size 
            }
        );
        Ok(())
    }

    /// Minimize a window
    pub fn minimize_window(&mut self, window_id: WindowId) -> Result<(), WindowError> {
        let window = self.windows.get_mut(&window_id)
            .ok_or(WindowError::WindowNotFound)?;
        
        window.set_state(WindowState::Minimized);
        self.event_manager.send_event(window_id, Event::WindowMinimized { window_id });
        Ok(())
    }

    /// Maximize a window
    pub fn maximize_window(&mut self, window_id: WindowId) -> Result<(), WindowError> {
        let window = self.windows.get_mut(&window_id)
            .ok_or(WindowError::WindowNotFound)?;
        
        window.set_state(WindowState::Maximized);
        self.event_manager.send_event(window_id, Event::WindowMaximized { window_id });
        Ok(())
    }

    /// Restore a window from minimized/maximized state
    pub fn restore_window(&mut self, window_id: WindowId) -> Result<(), WindowError> {
        let window = self.windows.get_mut(&window_id)
            .ok_or(WindowError::WindowNotFound)?;
        
        window.set_state(WindowState::Normal);
        self.event_manager.send_event(window_id, Event::WindowRestored { window_id });
        Ok(())
    }

    /// Bring window to front (change z-order)
    pub fn bring_to_front(&mut self, window_id: WindowId) -> Result<(), WindowError> {
        if !self.windows.contains_key(&window_id) {
            return Err(WindowError::WindowNotFound);
        }

        self.z_order.bring_to_front(window_id, self.active_workspace);
        self.event_manager.send_event(window_id, Event::WindowBroughtToFront { window_id });
        Ok(())
    }

    /// Send window to back (change z-order)
    pub fn send_to_back(&mut self, window_id: WindowId) -> Result<(), WindowError> {
        if !self.windows.contains_key(&window_id) {
            return Err(WindowError::WindowNotFound);
        }

        self.z_order.send_to_back(window_id, self.active_workspace);
        self.event_manager.send_event(window_id, Event::WindowSentToBack { window_id });
        Ok(())
    }

    /// Set window focus
    pub fn set_focus(&mut self, window_id: WindowId) -> Result<(), WindowError> {
        if !self.windows.contains_key(&window_id) {
            return Err(WindowError::WindowNotFound);
        }

        self.focus_manager.set_focus(window_id);
        self.event_manager.send_event(window_id, Event::WindowFocused { window_id });
        Ok(())
    }

    /// Clear window focus
    pub fn clear_focus(&mut self) {
        self.focus_manager.clear_focus();
    }

    /// Get focused window
    pub fn focused_window(&self) -> Option<WindowId> {
        self.focus_manager.focused_window()
    }

    /// Create a new workspace
    pub fn create_workspace(&mut self, name: String) -> Result<usize, WindowError> {
        if self.workspaces.len() >= 32 {
            return Err(WindowError::InvalidWorkspace);
        }
        
        let workspace_id = self.workspaces.len();
        self.workspaces.push(Workspace::new(name));
        Ok(workspace_id)
    }

    /// Switch to a different workspace
    pub fn switch_to_workspace(&mut self, workspace_id: usize) -> Result<(), WindowError> {
        if workspace_id >= self.workspaces.len() {
            return Err(WindowError::InvalidWorkspace);
        }
        
        self.active_workspace = workspace_id;
        Ok(())
    }

    /// Get current workspace
    pub fn active_workspace(&self) -> usize {
        self.active_workspace
    }

    /// Get window by ID
    pub fn get_window(&self, window_id: WindowId) -> Option<&Window> {
        self.windows.get(&window_id)
    }

    /// Get mutable window by ID
    pub fn get_window_mut(&mut self, window_id: WindowId) -> Option<&mut Window> {
        self.windows.get_mut(&window_id)
    }

    /// Get all windows in current workspace
    pub fn windows_in_workspace(&self, workspace_id: usize) -> Vec<WindowId> {
        self.z_order.windows_in_workspace(workspace_id)
    }

    /// Get all windows in active workspace
    pub fn active_windows(&self) -> Vec<WindowId> {
        self.windows_in_workspace(self.active_workspace)
    }

    /// Process events
    pub fn process_events(&mut self) -> Vec<Event> {
        self.event_manager.drain_events()
    }
}

impl Default for WindowManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_manager_creation() {
        let manager = WindowManager::new();
        assert_eq!(manager.active_workspace(), 0);
        assert_eq!(manager.active_windows().len(), 0);
    }

    #[test]
    fn test_window_creation_and_destruction() -> Result<(), Box<dyn std::error::Error>> {
        let mut manager = WindowManager::new();
        
        let window_id = manager.create_window(
            "Test Window".to_string(),
            Rectangle::new(100, 100, 400, 300),
            WindowStyle::default(),
        )?;
        
        assert!(manager.get_window(window_id).is_some());
        assert_eq!(manager.active_windows().len(), 1);
        
        manager.destroy_window(window_id)?;
        assert!(manager.get_window(window_id).is_none());
        assert_eq!(manager.active_windows().len(), 0);
        
        Ok(())
    }

    #[test]
    fn test_window_move_and_resize() -> Result<(), Box<dyn std::error::Error>> {
        let mut manager = WindowManager::new();
        
        let window_id = manager.create_window(
            "Test Window".to_string(),
            Rectangle::new(100, 100, 400, 300),
            WindowStyle::default(),
        )?;
        
        manager.move_window(window_id, Point::new(200, 200))?;
        manager.resize_window(window_id, Size::new(600, 400))?;
        
        let window = manager.get_window(window_id).unwrap();
        assert_eq!(window.bounds().position.x, 200);
        assert_eq!(window.bounds().position.y, 200);
        assert_eq!(window.bounds().size.width, 600);
        assert_eq!(window.bounds().size.height, 400);
        
        Ok(())
    }

    #[test]
    fn test_window_state_changes() -> Result<(), Box<dyn std::error::Error>> {
        let mut manager = WindowManager::new();
        
        let window_id = manager.create_window(
            "Test Window".to_string(),
            Rectangle::new(100, 100, 400, 300),
            WindowStyle::default(),
        )?;
        
        // Test minimize
        manager.minimize_window(window_id)?;
        let window = manager.get_window(window_id).unwrap();
        assert_eq!(window.state(), WindowState::Minimized);
        
        // Test maximize
        manager.maximize_window(window_id)?;
        let window = manager.get_window(window_id).unwrap();
        assert_eq!(window.state(), WindowState::Maximized);
        
        // Test restore
        manager.restore_window(window_id)?;
        let window = manager.get_window(window_id).unwrap();
        assert_eq!(window.state(), WindowState::Normal);
        
        Ok(())
    }

    #[test]
    fn test_focus_management() -> Result<(), Box<dyn std::error::Error>> {
        let mut manager = WindowManager::new();
        
        let window1_id = manager.create_window(
            "Window 1".to_string(),
            Rectangle::new(100, 100, 400, 300),
            WindowStyle::default(),
        )?;
        
        let window2_id = manager.create_window(
            "Window 2".to_string(),
            Rectangle::new(150, 150, 400, 300),
            WindowStyle::default(),
        )?;
        
        assert_eq!(manager.focused_window(), Some(window2_id)); // Last created has focus
        
        manager.set_focus(window1_id)?;
        assert_eq!(manager.focused_window(), Some(window1_id));
        
        manager.clear_focus();
        assert_eq!(manager.focused_window(), None);
        
        Ok(())
    }

    #[test]
    fn test_workspace_management() -> Result<(), Box<dyn std::error::Error>> {
        let mut manager = WindowManager::new();
        
        // Create additional workspaces
        let workspace1_id = manager.create_workspace("Development".to_string())?;
        let workspace2_id = manager.create_workspace("Design".to_string())?;
        
        assert_eq!(manager.active_workspace(), 0);
        assert!(workspace1_id > 0);
        assert!(workspace2_id > workspace1_id);
        
        // Switch to workspace 1
        manager.switch_to_workspace(workspace1_id)?;
        assert_eq!(manager.active_workspace(), workspace1_id);
        
        // Switch to workspace 2
        manager.switch_to_workspace(workspace2_id)?;
        assert_eq!(manager.active_workspace(), workspace2_id);
        
        Ok(())
    }

    #[test]
    fn test_window_error_handling() {
        let mut manager = WindowManager::new();
        
        // Test non-existent window operations
        assert_eq!(
            manager.destroy_window(WindowId::new(999)),
            Err(WindowError::WindowNotFound)
        );
        
        assert_eq!(
            manager.move_window(WindowId::new(999), Point::new(0, 0)),
            Err(WindowError::WindowNotFound)
        );
        
        assert_eq!(
            manager.set_focus(WindowId::new(999)),
            Err(WindowError::WindowNotFound)
        );
        
        // Test invalid workspace
        assert_eq!(
            manager.switch_to_workspace(999),
            Err(WindowError::InvalidWorkspace)
        );
        
        // Test invalid bounds
        assert_eq!(
            manager.create_window(
                "Test".to_string(),
                Rectangle::new(0, 0, 0, 0),
                WindowStyle::default(),
            ),
            Err(WindowError::InvalidBounds)
        );
    }

    #[test]
    fn test_window_hit_testing() -> Result<(), Box<dyn std::error::Error>> {
        let mut manager = WindowManager::new();
        
        let window_id = manager.create_window(
            "Test Window".to_string(),
            Rectangle::new(100, 100, 400, 300),
            WindowStyle::default(),
        )?;
        
        let window = manager.get_window(window_id).unwrap();
        
        // Test inside window
        assert!(window.contains_point(Point::new(200, 200)));
        
        // Test outside window
        assert!(!window.contains_point(Point::new(50, 50)));
        assert!(!window.contains_point(Point::new(600, 500)));
        
        Ok(())
    }

    #[test]
    fn test_z_order_operations() -> Result<(), Box<dyn std::error::Error>> {
        let mut manager = WindowManager::new();
        
        let window1_id = manager.create_window(
            "Window 1".to_string(),
            Rectangle::new(100, 100, 400, 300),
            WindowStyle::default(),
        )?;
        
        let window2_id = manager.create_window(
            "Window 2".to_string(),
            Rectangle::new(150, 150, 400, 300),
            WindowStyle::default(),
        )?;
        
        let window3_id = manager.create_window(
            "Window 3".to_string(),
            Rectangle::new(200, 200, 400, 300),
            WindowStyle::default(),
        )?;
        
        // Bring window 1 to front
        manager.bring_to_front(window1_id)?;
        
        // Send window 3 to back
        manager.send_to_back(window3_id)?;
        
        // Verify z-order through window list
        let windows = manager.active_windows();
        assert_eq!(windows[0], window1_id); // Frontmost
        assert_eq!(windows[2], window3_id); // Backmost
        
        Ok(())
    }

    #[test]
    fn test_event_processing() -> Result<(), Box<dyn std::error::Error>> {
        let mut manager = WindowManager::new();
        
        let window_id = manager.create_window(
            "Test Window".to_string(),
            Rectangle::new(100, 100, 400, 300),
            WindowStyle::default(),
        )?;
        
        // Process events
        let events = manager.process_events();
        assert_eq!(events.len(), 1);
        
        if let Event::WindowCreated { window_id: created_id } = &events[0] {
            assert_eq!(*created_id, window_id);
        }
        
        Ok(())
    }

    #[test]
    fn test_decorations() -> Result<(), Box<dyn std::error::Error>> {
        let mut manager = WindowManager::new();
        
        let window_id = manager.create_window(
            "Test Window".to_string(),
            Rectangle::new(100, 100, 400, 300),
            WindowStyle::default(),
        )?;
        
        let window = manager.get_window(window_id).unwrap();
        
        // Test title bar decoration
        if window.style.has_title_bar {
            let title_bar_bounds = window.decorations().title_bar();
            assert_eq!(title_bar_bounds.position.x, 100);
            assert_eq!(title_bar_bounds.position.y, 100);
            assert_eq!(title_bar_bounds.size.width, 400);
            assert_eq!(title_bar_bounds.size.height, 30);
        }
        
        // Test button bounds
        assert!(window.close_button_bounds().is_some());
        assert!(window.minimize_button_bounds().is_some());
        assert!(window.maximize_button_bounds().is_some());
        
        Ok(())
    }
}
