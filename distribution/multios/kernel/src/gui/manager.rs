//! GUI manager for MultiOS
//! 
//! Provides window management, application management, and main GUI system coordination.

use alloc::vec::Vec;
use alloc::boxed::Box;
use alloc::string::String;
use spin::Mutex;

use super::{WidgetId, WindowId, AppId, GUIResult, GUIError};
use super::events::{EventSystem, EventDispatcher, Event};
use super::graphics::{Renderer, FramebufferRenderer, Color, Rectangle, Size};
use super::widgets::{Widget, Container, Button, Label, TextField, Menu, Dialog, ListBox, ProgressBar};
use super::style::StyleManager;

/// Window structure
pub struct Window {
    pub id: WindowId,
    pub title: String,
    pub bounds: Rectangle,
    pub content: Box<dyn Widget>,
    pub dirty: bool,
    pub visible: bool,
    pub resizable: bool,
    pub closable: bool,
    pub minimizable: bool,
    pub maximized: bool,
    pub modal: bool,
}

impl Window {
    pub fn new(title: &str, width: u32, height: u32) -> Self {
        Self {
            id: WindowId::new(),
            title: title.to_string(),
            bounds: Rectangle::new(100, 100, width, height),
            content: Box::new(Container::new("window_content")),
            dirty: true,
            visible: true,
            resizable: true,
            closable: true,
            minimizable: true,
            maximized: false,
            modal: false,
        }
    }

    pub fn set_content(&mut self, content: Box<dyn Widget>) {
        self.content = content;
        self.invalidate();
    }

    pub fn invalidate(&mut self) {
        self.dirty = true;
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn show(&mut self) {
        self.visible = true;
        self.invalidate();
    }

    pub fn hide(&mut self) {
        self.visible = false;
        self.invalidate();
    }

    pub fn close(&mut self) {
        self.visible = false;
        // In a real implementation, this would trigger window close events
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.bounds.width = width;
        self.bounds.height = height;
        self.invalidate();
    }

    pub fn move_to(&mut self, x: i32, y: i32) {
        self.bounds.x = x;
        self.bounds.y = y;
        self.invalidate();
    }

    pub fn bring_to_front(&mut self) {
        // Window ordering logic would be handled by WindowManager
        self.invalidate();
    }

    pub fn send_to_back(&mut self) {
        // Window ordering logic would be handled by WindowManager
        self.invalidate();
    }
}

/// Window manager for organizing and managing multiple windows
pub struct WindowManager {
    windows: Vec<Window>,
    active_window: Option<WindowId>,
    z_order: Vec<WindowId>, // Stack for z-ordering
    desktop_rect: Rectangle,
    dirty: bool,
}

impl WindowManager {
    pub fn new(desktop_width: u32, desktop_height: u32) -> Self {
        Self {
            windows: Vec::new(),
            active_window: None,
            z_order: Vec::new(),
            desktop_rect: Rectangle::new(0, 0, desktop_width, desktop_height),
            dirty: true,
        }
    }

    pub fn create_window(&mut self, title: &str, width: u32, height: u32) -> GUIResult<WindowId> {
        let window = Window::new(title, width, height);
        let window_id = window.id;
        
        self.windows.push(window);
        self.z_order.push(window_id);
        self.dirty = true;
        
        Ok(window_id)
    }

    pub fn get_window(&self, window_id: WindowId) -> Option<&Window> {
        self.windows.iter().find(|w| w.id == window_id)
    }

    pub fn get_window_mut(&mut self, window_id: WindowId) -> Option<&mut Window> {
        self.windows.iter_mut().find(|w| w.id == window_id)
    }

    pub fn get_active_window(&self) -> Option<WindowId> {
        self.active_window
    }

    pub fn set_active_window(&mut self, window_id: Option<WindowId>) {
        self.active_window = window_id;
        
        // Move active window to front in z-order
        if let Some(window_id) = window_id {
            if let Some(pos) = self.z_order.iter().position(|&id| id == window_id) {
                self.z_order.remove(pos);
                self.z_order.push(window_id);
            }
        }
        
        self.dirty = true;
    }

    pub fn close_window(&mut self, window_id: WindowId) -> GUIResult<()> {
        if let Some(pos) = self.windows.iter().position(|w| w.id == window_id) {
            self.windows.remove(pos);
        }
        
        if let Some(pos) = self.z_order.iter().position(|&id| id == window_id) {
            self.z_order.remove(pos);
        }
        
        // Activate next window in z-order
        if let Some(&next_window_id) = self.z_order.last() {
            self.active_window = Some(next_window_id);
        } else {
            self.active_window = None;
        }
        
        self.dirty = true;
        Ok(())
    }

    pub fn get_all_windows(&self) -> &[Window] {
        &self.windows
    }

    pub fn get_windows_by_z_order(&self) -> Vec<&Window> {
        let mut windows_by_z = Vec::new();
        for window_id in &self.z_order {
            if let Some(window) = self.windows.iter().find(|w| w.id == *window_id) {
                windows_by_z.push(window);
            }
        }
        windows_by_z
    }

    pub fn get_dirty_windows(&self) -> Vec<WindowId> {
        self.windows.iter()
            .filter(|w| w.dirty && w.visible)
            .map(|w| w.id)
            .collect()
    }

    pub fn clear_dirty_flags(&mut self) {
        for window in &mut self.windows {
            window.dirty = false;
        }
        self.dirty = false;
    }

    pub fn get_desktop_rect(&self) -> Rectangle {
        self.desktop_rect
    }

    pub fn set_desktop_rect(&mut self, rect: Rectangle) {
        self.desktop_rect = rect;
        self.dirty = true;
    }

    pub fn cascade_windows(&mut self, offset: i32, start_x: i32, start_y: i32) {
        let mut x = start_x;
        let mut y = start_y;
        
        for window in &mut self.windows {
            window.move_to(x, y);
            x += offset;
            y += offset;
            
            // Wrap around if windows go off-screen
            if x + window.bounds.width as i32 > self.desktop_rect.width as i32 {
                x = start_x;
            }
            if y + window.bounds.height as i32 > self.desktop_rect.height as i32 {
                y = start_y;
            }
        }
        self.dirty = true;
    }

    pub fn tile_windows(&mut self, horizontal: bool) {
        let visible_windows: Vec<_> = self.windows.iter()
            .filter(|w| w.visible)
            .collect();
        
        if visible_windows.is_empty() {
            return;
        }

        let window_count = visible_windows.len();
        let tile_width = self.desktop_rect.width / window_count as u32;
        let tile_height = self.desktop_rect.height / window_count as u32;
        
        for (index, window) in visible_windows.into_iter().enumerate() {
            if horizontal {
                let x = (index as u32 * tile_width) as i32;
                let y = 0;
                window.move_to(x, y);
                window.resize(tile_width, self.desktop_rect.height);
            } else {
                let x = 0;
                let y = (index as u32 * tile_height) as i32;
                window.move_to(x, y);
                window.resize(self.desktop_rect.width, tile_height);
            }
        }
        self.dirty = true;
    }
}

/// Application structure
pub struct Application {
    pub id: AppId,
    pub name: String,
    pub windows: Vec<WindowId>,
    pub focused: bool,
    pub running: bool,
}

impl Application {
    pub fn new(name: &str) -> Self {
        Self {
            id: AppId::new(),
            name: name.to_string(),
            windows: Vec::new(),
            focused: false,
            running: false,
        }
    }

    pub fn add_window(&mut self, window_id: WindowId) {
        self.windows.push(window_id);
    }

    pub fn remove_window(&mut self, window_id: WindowId) {
        self.windows.retain(|&id| id != window_id);
    }

    pub fn start(&mut self) {
        self.running = true;
    }

    pub fn stop(&mut self) {
        self.running = false;
    }

    pub fn focus(&mut self) {
        self.focused = true;
    }

    pub fn blur(&mut self) {
        self.focused = false;
    }
}

/// Main GUI manager
pub struct GUIManager {
    window_manager: WindowManager,
    applications: Vec<Application>,
    renderer: FramebufferRenderer,
    event_dispatcher: EventDispatcher,
    style_manager: Option<StyleManager>,
    desktop_color: Color,
    desktop_background: Option<String>, // Wallpaper path
    cursor_position: super::graphics::Point,
    cursor_visible: bool,
    dirty: bool,
    initialized: bool,
}

impl GUIManager {
    pub fn new(desktop_width: u32, desktop_height: u32) -> Self {
        let renderer = FramebufferRenderer::new(desktop_width, desktop_height);
        
        Self {
            window_manager: WindowManager::new(desktop_width, desktop_height),
            applications: Vec::new(),
            renderer,
            event_dispatcher: EventDispatcher::new(),
            style_manager: None,
            desktop_color: Color::WHITE,
            desktop_background: None,
            cursor_position: super::graphics::Point::origin(),
            cursor_visible: true,
            dirty: true,
            initialized: false,
        }
    }

    /// Initialize the GUI manager
    pub fn initialize(&mut self) -> GUIResult<()> {
        if self.initialized {
            return Err(GUIError::InitializationFailed);
        }

        // Initialize renderer
        self.renderer.clear(self.desktop_color);
        
        // Setup default event handlers
        self.setup_default_handlers();
        
        // Create desktop
        self.create_desktop_window()?;
        
        self.initialized = true;
        self.dirty = true;
        
        info!("GUI Manager initialized");
        Ok(())
    }

    /// Setup default event handlers
    fn setup_default_handlers(&mut self) {
        // Add global event handlers here
        // For now, this is a placeholder
    }

    /// Create the desktop window
    fn create_desktop_window(&mut self) -> GUIResult<()> {
        let desktop_id = self.window_manager.create_window("Desktop", 
                                                         self.window_manager.get_desktop_rect().width,
                                                         self.window_manager.get_desktop_rect().height)?;
        
        if let Some(window) = self.window_manager.get_window_mut(desktop_id) {
            window.resizable = false;
            window.closable = false;
            window.minimizable = false;
            
            // Add desktop widgets (taskbar, icons, etc.)
            let mut desktop_container = Container::new("desktop");
            
            // Add taskbar
            let taskbar = Container::new("taskbar");
            desktop_container.add_child(Box::new(taskbar))?;
            
            window.set_content(Box::new(desktop_container));
        }
        
        Ok(())
    }

    /// Create a new application
    pub fn create_application(&mut self, name: &str) -> AppId {
        let app = Application::new(name);
        let app_id = app.id;
        self.applications.push(app);
        app_id
    }

    /// Get application by ID
    pub fn get_application(&self, app_id: AppId) -> Option<&Application> {
        self.applications.iter().find(|app| app.id == app_id)
    }

    /// Get application by ID (mutable)
    pub fn get_application_mut(&mut self, app_id: AppId) -> Option<&mut Application> {
        self.applications.iter_mut().find(|app| app.id == app_id)
    }

    /// Create a new window for an application
    pub fn create_window_for_app(&mut self, app_id: AppId, title: &str, width: u32, height: u32) -> GUIResult<WindowId> {
        let window_id = self.window_manager.create_window(title, width, height)?;
        
        if let Some(app) = self.get_application_mut(app_id) {
            app.add_window(window_id);
        }
        
        Ok(window_id)
    }

    /// Render the entire GUI
    pub fn render(&mut self) -> GUIResult<()> {
        if !self.initialized {
            return Err(GUIError::NotInitialized);
        }

        // Clear screen
        self.renderer.clear(self.desktop_color);
        
        // Draw desktop background
        if let Some(background) = &self.desktop_background {
            // Draw wallpaper if implemented
            // For now, just draw desktop color
        }
        
        // Render all windows in z-order
        let windows_by_z = self.window_manager.get_windows_by_z_order();
        for window in windows_by_z.iter().filter(|w| w.visible) {
            self.render_window(window)?;
        }
        
        // Draw cursor
        if self.cursor_visible {
            self.draw_cursor();
        }
        
        self.window_manager.clear_dirty_flags();
        self.dirty = false;
        
        Ok(())
    }

    /// Render a specific window
    fn render_window(&self, window: &Window) -> GUIResult<()> {
        // Render window background
        self.renderer.fill_rect(window.bounds, Color::WHITE);
        
        // Render window border
        let border = super::graphics::Border::new(2, Color::GRAY, super::graphics::BorderStyle::Solid);
        self.renderer.draw_rect(window.bounds, border);
        
        // Render window title bar
        let title_bar_height = 30;
        let title_bar_rect = Rectangle::new(window.bounds.x, window.bounds.y, 
                                          window.bounds.width, title_bar_height);
        self.renderer.fill_rect(title_bar_rect, Color::DARK_GRAY);
        
        // Draw title text
        // self.renderer.draw_text(&window.title, 
        //                       super::graphics::Point::new(window.bounds.x + 8, window.bounds.y + 16),
        //                       super::graphics::Font::new(14, super::graphics::FontFamily::SansSerif),
        //                       Color::WHITE);
        
        // Render close button
        if window.closable {
            let close_button_rect = Rectangle::new(window.bounds.x + window.bounds.width as i32 - 25,
                                                 window.bounds.y + 5, 15, 15);
            self.renderer.draw_rect(close_button_rect, 
                                  super::graphics::Border::new(1, Color::WHITE, super::graphics::BorderStyle::Solid));
            // self.renderer.draw_text("X", 
            //                      super::graphics::Point::new(close_button_rect.x + 3, close_button_rect.y + 10),
            //                      super::graphics::Font::new(10, super::graphics::FontFamily::Default),
            //                      Color::WHITE);
        }
        
        // Render window content
        let content_rect = Rectangle::new(window.bounds.x + 2,
                                        window.bounds.y + title_bar_height as i32 + 2,
                                        window.bounds.width - 4,
                                        window.bounds.height - title_bar_height - 4);
        
        // Set clip rect for content rendering
        self.renderer.set_clip_rect(Some(content_rect));
        
        // Render the widget content
        window.content.render(&mut self.renderer)?;
        
        // Clear clip rect
        self.renderer.set_clip_rect(None);
        
        Ok(())
    }

    /// Draw the cursor
    fn draw_cursor(&self) {
        let cursor_size = 16;
        let cursor_rect = Rectangle::new(self.cursor_position.x, self.cursor_position.y, 
                                       cursor_size, cursor_size);
        self.renderer.fill_rect(cursor_rect, Color::BLACK);
        self.renderer.draw_rect(cursor_rect, 
                              super::graphics::Border::new(1, Color::WHITE, super::graphics::BorderStyle::Solid));
    }

    /// Process all pending events
    pub fn process_events(&mut self) -> GUIResult<()> {
        // Process events from the event system
        EventSystem::process_events(&mut self.event_dispatcher);
        
        // Mark GUI as dirty if any events were processed
        if EventSystem::has_events() {
            self.dirty = true;
        }
        
        Ok(())
    }

    /// Update the GUI (called periodically)
    pub fn update(&mut self) -> GUIResult<()> {
        // Update animations, timers, etc.
        // For now, just mark as dirty to trigger redraw
        self.dirty = true;
        Ok(())
    }

    /// Get the framebuffer data for display
    pub fn get_framebuffer(&self) -> &[u8] {
        self.renderer.get_buffer()
    }

    /// Get the screen size
    pub fn get_screen_size(&self) -> Size {
        Size::new(self.renderer.width, self.renderer.height)
    }

    /// Set desktop background color
    pub fn set_desktop_color(&mut self, color: Color) {
        self.desktop_color = color;
        self.dirty = true;
    }

    /// Set desktop wallpaper
    pub fn set_desktop_wallpaper(&mut self, wallpaper_path: &str) {
        self.desktop_background = Some(wallpaper_path.to_string());
        self.dirty = true;
    }

    /// Show/hide cursor
    pub fn set_cursor_visible(&mut self, visible: bool) {
        self.cursor_visible = visible;
        self.dirty = true;
    }

    /// Move cursor
    pub fn move_cursor(&mut self, position: super::graphics::Point) {
        self.cursor_position = position;
        self.dirty = true;
    }

    /// Get cursor position
    pub fn get_cursor_position(&self) -> super::graphics::Point {
        self.cursor_position
    }

    /// Check if GUI needs redrawing
    pub fn is_dirty(&self) -> bool {
        self.dirty || self.window_manager.dirty
    }

    /// Get window manager
    pub fn get_window_manager(&self) -> &WindowManager {
        &self.window_manager
    }

    /// Get window manager (mutable)
    pub fn get_window_manager_mut(&mut self) -> &mut WindowManager {
        &mut self.window_manager
    }

    /// Get event dispatcher
    pub fn get_event_dispatcher(&mut self) -> &mut EventDispatcher {
        &mut self.event_dispatcher
    }

    /// Check if manager is initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
}

/// Global GUI manager instance
static GUI_MANAGER: Mutex<Option<GUIManager>> = Mutex::new(None);

/// Initialize the GUI manager
pub fn init() -> GUIResult<()> {
    info!("Initializing GUI manager...");
    
    let mut manager_guard = GUI_MANAGER.lock();
    if manager_guard.is_some() {
        return Err(GUIError::InitializationFailed);
    }
    
    // Create GUI manager with default screen size (800x600)
    let manager = GUIManager::new(800, 600);
    let mut manager = manager;
    manager.initialize()?;
    
    *manager_guard = Some(manager);
    
    info!("GUI manager initialized successfully");
    Ok(())
}

/// Shutdown the GUI manager
pub fn shutdown() -> GUIResult<()> {
    info!("Shutting down GUI manager...");
    
    let mut manager_guard = GUI_MANAGER.lock();
    if let Some(mut manager) = manager_guard.take() {
        // Cleanup operations
        manager.initialized = false;
    }
    
    info!("GUI manager shutdown complete");
    Ok(())
}

/// Get the global GUI manager
pub fn get_gui_manager() -> Option<GUIManager> {
    let manager_guard = GUI_MANAGER.lock();
    manager_guard.clone()
}

/// Get mutable reference to global GUI manager
pub fn get_gui_manager_mut() -> Option<GUIManager> {
    let mut manager_guard = GUI_MANAGER.lock();
    manager_guard.as_ref().cloned()
}

/// Create a simple test window with default widgets
pub fn create_test_window() -> GUIResult<WindowId> {
    let manager = get_gui_manager_mut();
    if let Some(mut manager) = manager {
        if !manager.is_initialized() {
            return Err(GUIError::NotInitialized);
        }
        
        let window_id = manager.get_window_manager_mut().create_window("Test Window", 400, 300)?;
        
        if let Some(window) = manager.get_window_manager_mut().get_window_mut(window_id) {
            let mut container = Container::new("test_content");
            
            // Add some test widgets
            let button = Button::new("Click Me!");
            let label = Label::new("Hello, MultiOS GUI!");
            let text_field = TextField::new();
            let list_box = ListBox::new();
            let progress_bar = ProgressBar::new();
            
            container.add_child(Box::new(button))?;
            container.add_child(Box::new(label))?;
            container.add_child(Box::new(text_field))?;
            container.add_child(Box::new(list_box))?;
            container.add_child(Box::new(progress_bar))?;
            
            window.set_content(Box::new(container));
        }
        
        Ok(window_id)
    } else {
        Err(GUIError::NotInitialized)
    }
}