//! Window focus management

use crate::*;

/// Manages window focus and keyboard input routing
#[derive(Debug)]
pub struct FocusManager {
    focused_window: Option<WindowId>,
    window_history: Vec<WindowId>,
    max_history: usize,
}

impl FocusManager {
    /// Create a new focus manager
    pub fn new() -> Self {
        Self {
            focused_window: None,
            window_history: Vec::new(),
            max_history: 10, // Keep track of last 10 focused windows
        }
    }

    /// Set focus to a window
    pub fn set_focus(&mut self, window_id: WindowId) {
        // Remove from history if already present
        self.window_history.retain(|&id| id != window_id);
        
        // Add to front of history
        self.window_history.insert(0, window_id);
        
        // Trim history to max size
        if self.window_history.len() > self.max_history {
            self.window_history.pop();
        }

        self.focused_window = Some(window_id);
    }

    /// Clear focus (no window has focus)
    pub fn clear_focus(&mut self) {
        self.focused_window = None;
    }

    /// Get currently focused window
    pub fn focused_window(&self) -> Option<WindowId> {
        self.focused_window
    }

    /// Check if a specific window has focus
    pub fn has_focus(&self, window_id: WindowId) -> bool {
        self.focused_window == Some(window_id)
    }

    /// Check if any window has focus
    pub fn has_focus_any(&self) -> bool {
        self.focused_window.is_some()
    }

    /// Get focus history (most recently focused first)
    pub fn focus_history(&self) -> &[WindowId] {
        &self.window_history
    }

    /// Set focus to next window in history
    pub fn focus_previous(&mut self) -> Option<WindowId> {
        if !self.window_history.is_empty() {
            // Skip current focused window (at index 0)
            if self.window_history.len() > 1 {
                let prev_window = self.window_history[1];
                self.set_focus(prev_window);
                return Some(prev_window);
            }
        }
        None
    }

    /// Set focus to next window in history (same as focus_next_from_history)
    pub fn focus_next(&mut self) -> Option<WindowId> {
        self.focus_next_from_history()
    }

    /// Set focus to next window in history
    pub fn focus_next_from_history(&mut self) -> Option<WindowId> {
        if !self.window_history.is_empty() {
            // Skip current focused window (at index 0)
            if self.window_history.len() > 1 {
                let next_window = self.window_history[1];
                self.set_focus(next_window);
                return Some(next_window);
            }
        }
        None
    }

    /// Get previously focused window (ignoring current)
    pub fn previous_focused(&self) -> Option<WindowId> {
        if self.window_history.len() > 1 {
            Some(self.window_history[1])
        } else {
            None
        }
    }

    /// Focus window cycling through available windows
    pub fn cycle_focus(&mut self, available_windows: &[WindowId]) {
        if available_windows.is_empty() {
            return;
        }

        let current = self.focused_window;
        
        // Find current window index
        let current_index = current.and_then(|id| {
            available_windows.iter().position(|&w| w == id)
        });

        // Move to next window
        let next_index = match current_index {
            Some(i) => {
                if i + 1 < available_windows.len() {
                    i + 1
                } else {
                    0 // Wrap around to first
                }
            }
            None => 0, // No current focus, start with first window
        };

        self.set_focus(available_windows[next_index]);
    }

    /// Set focus to first window in list
    pub fn focus_first(&mut self, available_windows: &[WindowId]) {
        if !available_windows.is_empty() {
            self.set_focus(available_windows[0]);
        }
    }

    /// Set focus to last window in list
    pub fn focus_last(&mut self, available_windows: &[WindowId]) {
        if !available_windows.is_empty() {
            self.set_focus(available_windows[available_windows.len() - 1]);
        }
    }

    /// Remove window from focus history
    pub fn remove_window_from_history(&mut self, window_id: WindowId) {
        self.window_history.retain(|&id| id != window_id);
        
        // If the removed window was focused, clear focus
        if self.focused_window == Some(window_id) {
            self.clear_focus();
            
            // Try to focus another window from history
            if !self.window_history.is_empty() {
                let next_focus = self.window_history[0];
                self.focused_window = Some(next_focus);
            }
        }
    }

    /// Clear focus history
    pub fn clear_history(&mut self) {
        self.window_history.clear();
        // Keep current focus but clear history
    }

    /// Get focus statistics
    pub fn focus_stats(&self) -> FocusStats {
        FocusStats {
            current_focus: self.focused_window,
            has_focus: self.has_focus_any(),
            history_count: self.window_history.len(),
            max_history: self.max_history,
        }
    }

    /// Set maximum history size
    pub fn set_max_history(&mut self, max_size: usize) {
        self.max_history = max_size;
        if self.window_history.len() > max_size {
            self.window_history.truncate(max_size);
        }
    }

    /// Get maximum history size
    pub fn max_history(&self) -> usize {
        self.max_history
    }
}

/// Focus statistics
#[derive(Debug, Clone)]
pub struct FocusStats {
    pub current_focus: Option<WindowId>,
    pub has_focus: bool,
    pub history_count: usize,
    pub max_history: usize,
}

/// Window activation modes
#[derive(Debug, Clone, Copy)]
pub enum ActivationMode {
    /// Activate normally (may change focus)
    Normal,
    /// Activate but don't change focus
    Passive,
    /// Bring to front and focus
    Foreground,
    /// Bring to front but don't focus
    RaiseOnly,
}

/// Focus event types
#[derive(Debug)]
pub enum FocusEvent {
    FocusGained(WindowId),
    FocusLost(WindowId),
    FocusChanged(Option<WindowId>, Option<WindowId>),
    FocusRequested(WindowId),
    FocusCycle(WindowId),
}

impl FocusManager {
    /// Handle focus request from external source
    pub fn request_focus(&mut self, window_id: WindowId) {
        self.set_focus(window_id);
    }

    /// Handle window activation
    pub fn activate_window(&mut self, window_id: WindowId, mode: ActivationMode) {
        match mode {
            ActivationMode::Normal => {
                self.set_focus(window_id);
            }
            ActivationMode::Passive => {
                // Don't change current focus
            }
            ActivationMode::Foreground => {
                self.set_focus(window_id);
            }
            ActivationMode::RaiseOnly => {
                // Just bring to front, don't change focus
                // This would typically involve z-order changes
            }
        }
    }

    /// Check if window should receive keyboard events
    pub fn should_receive_keyboard(&self, window_id: WindowId) -> bool {
        self.has_focus(window_id)
    }

    /// Check if window should receive mouse events
    pub fn should_receive_mouse(&self, _window_id: WindowId) -> bool {
        // All visible windows can receive mouse events
        // Focus primarily affects keyboard input
        true
    }

    /// Get all windows that should receive input
    pub fn get_input_targets(&self) -> InputTargets {
        InputTargets {
            keyboard_target: self.focused_window,
            // Mouse targets would be determined by hit testing
            mouse_targets: Vec::new(),
        }
    }
}

/// Input targets for different input types
#[derive(Debug)]
pub struct InputTargets {
    pub keyboard_target: Option<WindowId>,
    pub mouse_targets: Vec<WindowId>, // Ordered by z-order for mouse events
}

impl InputTargets {
    /// Create empty input targets
    pub fn new() -> Self {
        Self {
            keyboard_target: None,
            mouse_targets: Vec::new(),
        }
    }

    /// Check if keyboard should go to specific window
    pub fn should_send_keys_to(&self, window_id: WindowId) -> bool {
        self.keyboard_target == Some(window_id)
    }

    /// Get keyboard target or none
    pub fn keyboard_target(&self) -> Option<WindowId> {
        self.keyboard_target
    }
}
