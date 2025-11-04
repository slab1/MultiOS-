//! Z-order management for window layering and stacking

use crate::*;

/// Manages the z-order (stacking order) of windows
#[derive(Debug)]
pub struct ZOrderManager {
    workspaces_z_orders: Vec<ZOrderStack>,
}

impl ZOrderManager {
    /// Create a new z-order manager
    pub fn new() -> Self {
        Self {
            workspaces_z_orders: vec![ZOrderStack::new()],
        }
    }

    /// Add window to workspace z-order
    pub fn add_window(&mut self, window_id: WindowId, workspace_id: usize) {
        if workspace_id >= self.workspaces_z_orders.len() {
            // Extend if necessary
            while self.workspaces_z_orders.len() <= workspace_id {
                self.workspaces_z_orders.push(ZOrderStack::new());
            }
        }

        self.workspaces_z_orders[workspace_id].add_window(window_id);
    }

    /// Remove window from workspace z-order
    pub fn remove_window(&mut self, window_id: WindowId, workspace_id: usize) {
        if workspace_id < self.workspaces_z_orders.len() {
            self.workspaces_z_orders[workspace_id].remove_window(window_id);
        }
    }

    /// Bring window to front (highest z-order)
    pub fn bring_to_front(&mut self, window_id: WindowId, workspace_id: usize) {
        if workspace_id < self.workspaces_z_orders.len() {
            self.workspaces_z_orders[workspace_id].bring_to_front(window_id);
        }
    }

    /// Send window to back (lowest z-order)
    pub fn send_to_back(&mut self, window_id: WindowId, workspace_id: usize) {
        if workspace_id < self.workspaces_z_orders.len() {
            self.workspaces_z_orders[workspace_id].send_to_back(window_id);
        }
    }

    /// Move window up in z-order
    pub fn move_up(&mut self, window_id: WindowId, workspace_id: usize) -> Result<(), ZOrderError> {
        if workspace_id >= self.workspaces_z_orders.len() {
            return Err(ZOrderError::InvalidWorkspace);
        }
        self.workspaces_z_orders[workspace_id].move_up(window_id)
    }

    /// Move window down in z-order
    pub fn move_down(&mut self, window_id: WindowId, workspace_id: usize) -> Result<(), ZOrderError> {
        if workspace_id >= self.workspaces_z_orders.len() {
            return Err(ZOrderError::InvalidWorkspace);
        }
        self.workspaces_z_orders[workspace_id].move_down(window_id)
    }

    /// Get z-index of window
    pub fn get_z_index(&self, window_id: WindowId, workspace_id: usize) -> Option<usize> {
        if workspace_id < self.workspaces_z_orders.len() {
            self.workspaces_z_orders[workspace_id].get_z_index(window_id)
        } else {
            None
        }
    }

    /// Get all windows in workspace sorted by z-order (front to back)
    pub fn windows_in_workspace(&self, workspace_id: usize) -> Vec<WindowId> {
        if workspace_id < self.workspaces_z_orders.len() {
            self.workspaces_z_orders[workspace_id].windows_front_to_back()
        } else {
            Vec::new()
        }
    }

    /// Get all windows in workspace sorted by z-order (back to front)
    pub fn windows_in_workspace_back_to_front(&self, workspace_id: usize) -> Vec<WindowId> {
        if workspace_id < self.workspaces_z_orders.len() {
            self.workspaces_z_orders[workspace_id].windows_back_to_front()
        } else {
            Vec::new()
        }
    }

    /// Check if window is on top
    pub fn is_on_top(&self, window_id: WindowId, workspace_id: usize) -> bool {
        if workspace_id >= self.workspaces_z_orders.len() {
            return false;
        }
        self.workspaces_z_orders[workspace_id].is_on_top(window_id)
    }

    /// Check if window is at bottom
    pub fn is_at_bottom(&self, window_id: WindowId, workspace_id: usize) -> bool {
        if workspace_id >= self.workspaces_z_orders.len() {
            return false;
        }
        self.workspaces_z_orders[workspace_id].is_at_bottom(window_id)
    }

    /// Get window count in workspace
    pub fn window_count(&self, workspace_id: usize) -> usize {
        if workspace_id < self.workspaces_z_orders.len() {
            self.workspaces_z_orders[workspace_id].window_count()
        } else {
            0
        }
    }

    /// Find topmost window at position (hit testing)
    pub fn find_topmost_at_position(&self, position: Point, workspace_id: usize) -> Option<WindowId> {
        if workspace_id >= self.workspaces_z_orders.len() {
            return None;
        }

        self.workspaces_z_orders[workspace_id].find_topmost_at_position(position)
    }

    /// Get window directly below another window
    pub fn get_window_below(&self, window_id: WindowId, workspace_id: usize) -> Option<WindowId> {
        if workspace_id >= self.workspaces_z_orders.len() {
            return None;
        }
        self.workspaces_z_orders[workspace_id].get_window_below(window_id)
    }

    /// Ensure workspace exists in z-order manager
    pub fn ensure_workspace(&mut self, workspace_id: usize) {
        while self.workspaces_z_orders.len() <= workspace_id {
            self.workspaces_z_orders.push(ZOrderStack::new());
        }
    }
}

/// Manages z-order for a single workspace
#[derive(Debug)]
struct ZOrderStack {
    windows: Vec<WindowId>,
}

impl ZOrderStack {
    /// Create empty z-order stack
    pub fn new() -> Self {
        Self {
            windows: Vec::new(),
        }
    }

    /// Add window to z-order
    pub fn add_window(&mut self, window_id: WindowId) {
        if !self.windows.contains(&window_id) {
            self.windows.push(window_id);
        }
    }

    /// Remove window from z-order
    pub fn remove_window(&mut self, window_id: WindowId) {
        self.windows.retain(|&id| id != window_id);
    }

    /// Bring window to front
    pub fn bring_to_front(&mut self, window_id: WindowId) {
        if let Some(pos) = self.windows.iter().position(|&id| id == window_id) {
            let window = self.windows.remove(pos);
            self.windows.push(window);
        }
    }

    /// Send window to back
    pub fn send_to_back(&mut self, window_id: WindowId) {
        if let Some(pos) = self.windows.iter().position(|&id| id == window_id) {
            let window = self.windows.remove(pos);
            self.windows.insert(0, window);
        }
    }

    /// Move window up in z-order
    pub fn move_up(&mut self, window_id: WindowId) -> Result<(), ZOrderError> {
        if let Some(pos) = self.windows.iter().position(|&id| id == window_id) {
            if pos == 0 {
                return Err(ZOrderError::AlreadyAtTop);
            }
            self.windows.swap(pos, pos - 1);
            Ok(())
        } else {
            Err(ZOrderError::WindowNotFound)
        }
    }

    /// Move window down in z-order
    pub fn move_down(&mut self, window_id: WindowId) -> Result<(), ZOrderError> {
        if let Some(pos) = self.windows.iter().position(|&id| id == window_id) {
            if pos == self.windows.len() - 1 {
                return Err(ZOrderError::AlreadyAtBottom);
            }
            self.windows.swap(pos, pos + 1);
            Ok(())
        } else {
            Err(ZOrderError::WindowNotFound)
        }
    }

    /// Get z-index of window
    pub fn get_z_index(&self, window_id: WindowId) -> Option<usize> {
        self.windows.iter().position(|&id| id == window_id)
    }

    /// Get windows from front to back
    pub fn windows_front_to_back(&self) -> Vec<WindowId> {
        self.windows.clone()
    }

    /// Get windows from back to front
    pub fn windows_back_to_front(&self) -> Vec<WindowId> {
        let mut reversed = self.windows.clone();
        reversed.reverse();
        reversed
    }

    /// Check if window is on top
    pub fn is_on_top(&self, window_id: WindowId) -> bool {
        self.windows.last().map_or(false, |&id| id == window_id)
    }

    /// Check if window is at bottom
    pub fn is_at_bottom(&self, window_id: WindowId) -> bool {
        self.windows.first().map_or(false, |&id| id == window_id)
    }

    /// Get window count
    pub fn window_count(&self) -> usize {
        self.windows.len()
    }

    /// Find topmost window at position (for hit testing)
    pub fn find_topmost_at_position(&self, position: Point) -> Option<WindowId> {
        // Search from front to back (highest to lowest z-order)
        for window_id in self.windows.iter().rev() {
            // This would need access to window bounds, which we don't have here
            // In a real implementation, this would need to be provided by the window manager
            return Some(*window_id);
        }
        None
    }

    /// Get window directly below another window
    pub fn get_window_below(&self, window_id: WindowId) -> Option<WindowId> {
        if let Some(pos) = self.windows.iter().position(|&id| id == window_id) {
            if pos > 0 {
                return Some(self.windows[pos - 1]);
            }
        }
        None
    }
}

#[derive(Debug, Clone)]
pub enum ZOrderError {
    WindowNotFound,
    AlreadyAtTop,
    AlreadyAtBottom,
    InvalidWorkspace,
}

impl std::fmt::Display for ZOrderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ZOrderError::WindowNotFound => write!(f, "Window not found in z-order"),
            ZOrderError::AlreadyAtTop => write!(f, "Window is already at top of z-order"),
            ZOrderError::AlreadyAtBottom => write!(f, "Window is already at bottom of z-order"),
            ZOrderError::InvalidWorkspace => write!(f, "Invalid workspace ID"),
        }
    }
}

impl std::error::Error for ZOrderError {}
