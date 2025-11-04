//! MultiOS GUI Toolkit Example
//! 
//! Demonstrates the usage of the MultiOS GUI toolkit with a simple application.
//! This file serves as both documentation and a working example.

#![allow(dead_code)]

use crate::gui::{
    init, shutdown, GUIResult, GUIError,
    manager::{GUIManager, get_gui_manager_mut, create_test_window},
    widgets::{
        Widget, Container, Button, Label, TextField, Menu, Dialog, ListBox, ProgressBar,
        DialogButton
    },
    layout::{FlowLayout, GridLayout, LayoutManager},
    style::{Color, Border},
    events::{create_mouse_down_event, create_key_down_event, EventType},
    graphics::{Point, Rectangle, Size}
};

/// Example GUI application
pub struct ExampleApp {
    gui_manager: GUIManager,
    main_window_id: super::WindowId,
    counter: u32,
}

impl ExampleApp {
    /// Create a new example application
    pub fn new() -> GUIResult<Self> {
        // Initialize GUI system
        init()?;
        
        // Create GUI manager
        let gui_manager = GUIManager::new(1024, 768);
        let mut gui_manager = gui_manager;
        
        // Initialize the GUI manager
        gui_manager.initialize()?;
        
        // Create main application
        let app_id = gui_manager.create_application("Example App");
        
        // Create main window
        let main_window_id = gui_manager.create_window_for_app(
            app_id, 
            "MultiOS GUI Example", 
            600, 
            400
        )?;
        
        // Setup window content
        Self::setup_main_window(&mut gui_manager, main_window_id)?;
        
        Ok(Self {
            gui_manager,
            main_window_id,
            counter: 0,
        })
    }
    
    /// Setup the main window with example widgets
    fn setup_main_window(gui_manager: &mut GUIManager, window_id: super::WindowId) -> GUIResult<()> {
        if let Some(window) = gui_manager.get_window_manager_mut().get_window_mut(window_id) {
            // Create main container with grid layout
            let layout = GridLayout::with_dimensions(2, 2)
                .spacing(16)
                .padding(20);
            
            let mut main_container = Container::new("main_container").with_layout(layout);
            
            // Create example widgets
            
            // 1. Button section
            let mut button_container = Container::new("button_section");
            let button_layout = FlowLayout::vertical()
                .spacing(8)
                .padding(10);
            button_container = button_container.with_layout(button_layout);
            
            let counter_label = Label::new("Counter: 0");
            let increment_button = Button::new("Increment");
            let decrement_button = Button::new("Decrement");
            let reset_button = Button::new("Reset");
            
            button_container.add_child(Box::new(counter_label))?;
            button_container.add_child(Box::new(increment_button))?;
            button_container.add_child(Box::new(decrement_button))?;
            button_container.add_child(Box::new(reset_button))?;
            
            // 2. Text input section
            let mut input_container = Container::new("input_section");
            let input_layout = FlowLayout::vertical()
                .spacing(8)
                .padding(10);
            input_container = input_container.with_layout(input_layout);
            
            let name_label = Label::new("Name:");
            let name_input = TextField::new();
            name_input.set_max_length(50);
            
            let message_label = Label::new("Message:");
            let message_input = TextField::new();
            message_input.set_max_length(100);
            
            input_container.add_child(Box::new(name_label))?;
            input_container.add_child(Box::new(name_input))?;
            input_container.add_child(Box::new(message_label))?;
            input_container.add_child(Box::new(message_input))?;
            
            // 3. List and progress section
            let mut list_container = Container::new("list_section");
            let list_layout = FlowLayout::vertical()
                .spacing(8)
                .padding(10);
            list_container = list_container.with_layout(list_layout);
            
            let fruits_label = Label::new("Fruits:");
            let fruits_list = ListBox::new();
            fruits_list.add_item("Apple");
            fruits_list.add_item("Banana");
            fruits_list.add_item("Orange");
            fruits_list.add_item("Grape");
            fruits_list.add_item("Strawberry");
            
            let progress_label = Label::new("Progress:");
            let progress_bar = ProgressBar::new();
            progress_bar.set_value(0.5); // 50% progress
            
            list_container.add_child(Box::new(fruits_label))?;
            list_container.add_child(Box::new(fruits_list))?;
            list_container.add_child(Box::new(progress_label))?;
            list_container.add_child(Box::new(progress_bar))?;
            
            // 4. Menu and dialog section
            let mut menu_container = Container::new("menu_section");
            let menu_layout = FlowLayout::vertical()
                .spacing(8)
                .padding(10);
            menu_container = menu_container.with_layout(menu_layout);
            
            let file_menu = Menu::new();
            file_menu.add_item("New");
            file_menu.add_item("Open");
            file_menu.add_item("Save");
            file_menu.add_checkbox_item("Auto Save", false);
            
            let help_menu = Menu::new();
            help_menu.add_item("Help Topics");
            help_menu.add_item("About");
            
            let dialog_button = Button::new("Show Dialog");
            
            menu_container.add_child(Box::new(file_menu))?;
            menu_container.add_child(Box::new(help_menu))?;
            menu_container.add_child(Box::new(dialog_button))?;
            
            // Add all sections to main container
            main_container.add_child(Box::new(button_container))?;
            main_container.add_child(Box::new(input_container))?;
            main_container.add_child(Box::new(list_container))?;
            main_container.add_child(Box::new(menu_container))?;
            
            // Set the content of the window
            window.set_content(Box::new(main_container));
        }
        
        Ok(())
    }
    
    /// Run the application
    pub fn run(&mut self) -> GUIResult<()> {
        info!("Starting MultiOS GUI Example Application");
        
        // Show the main window
        if let Some(window) = self.gui_manager.get_window_manager_mut().get_window_mut(self.main_window_id) {
            window.show();
        }
        
        // Application main loop
        self.main_loop()?;
        
        Ok(())
    }
    
    /// Application main loop
    fn main_loop(&mut self) -> GUIResult<()> {
        loop {
            // Process pending events
            self.gui_manager.process_events()?;
            
            // Update GUI state
            self.gui_manager.update()?;
            
            // Render if needed
            if self.gui_manager.is_dirty() {
                self.gui_manager.render()?;
            }
            
            // Check for application exit condition
            // In a real implementation, this would check for quit events
            if self.should_exit() {
                break;
            }
            
            // Sleep briefly to avoid busy waiting
            self.sleep_ms(16); // ~60 FPS
        }
        
        Ok(())
    }
    
    /// Check if application should exit
    fn should_exit(&self) -> bool {
        // In a real implementation, check for quit events
        self.counter > 1000 // Example exit condition
    }
    
    /// Sleep for specified milliseconds
    fn sleep_ms(&self, ms: u32) {
        // In a real implementation, this would use proper timing
        // For now, just a placeholder
    }
    
    /// Handle button clicks
    pub fn handle_button_click(&mut self, button_text: &str) -> GUIResult<()> {
        match button_text {
            "Increment" => {
                self.counter += 1;
                self.update_counter_display()?;
            }
            "Decrement" => {
                self.counter = self.counter.saturating_sub(1);
                self.update_counter_display()?;
            }
            "Reset" => {
                self.counter = 0;
                self.update_counter_display()?;
            }
            "Show Dialog" => {
                self.show_example_dialog()?;
            }
            _ => {}
        }
        
        Ok(())
    }
    
    /// Update counter display
    fn update_counter_display(&mut self) -> GUIResult<()> {
        if let Some(window) = self.gui_manager.get_window_manager_mut().get_window_mut(self.main_window_id) {
            // Update counter label text
            // In a real implementation, this would find and update the specific widget
            info!("Counter updated to: {}", self.counter);
        }
        
        Ok(())
    }
    
    /// Show example dialog
    fn show_example_dialog(&mut self) -> GUIResult<()> {
        let mut dialog = Dialog::new("Example Dialog");
        
        // Add content to dialog
        let dialog_content = Container::new("dialog_content");
        let message_label = Label::new("This is an example dialog!");
        dialog_content.add_child(Box::new(message_label))?;
        
        dialog.set_content(Box::new(dialog_content));
        
        // Add standard buttons
        dialog.add_button(DialogButton::Ok);
        dialog.add_button(DialogButton::Cancel);
        
        // Show dialog
        dialog.show()?;
        
        info!("Example dialog shown");
        Ok(())
    }
    
    /// Get the framebuffer data for display
    pub fn get_framebuffer_data(&self) -> &[u8] {
        self.gui_manager.get_framebuffer()
    }
    
    /// Get screen size
    pub fn get_screen_size(&self) -> Size {
        self.gui_manager.get_screen_size()
    }
    
    /// Clean up resources
    pub fn cleanup(&mut self) {
        // Close all windows
        // Stop all applications
        // Clean up resources
        
        // Shutdown GUI system
        let _ = shutdown();
        
        info!("GUI Example Application cleanup complete");
    }
}

impl Drop for ExampleApp {
    fn drop(&mut self) {
        self.cleanup();
    }
}

/// Example of creating a custom widget
pub struct CustomWidget {
    id: super::WidgetId,
    bounds: Rectangle,
    value: f32,
    dirty: bool,
}

impl CustomWidget {
    pub fn new() -> Self {
        Self {
            id: super::WidgetId::new(),
            bounds: Rectangle::new(0, 0, 100, 50),
            value: 0.0,
            dirty: true,
        }
    }
    
    pub fn set_value(&mut self, value: f32) {
        self.value = value;
        self.dirty = true;
    }
    
    pub fn get_value(&self) -> f32 {
        self.value
    }
}

impl Widget for CustomWidget {
    fn get_widget_type(&self) -> &'static str {
        "CustomWidget"
    }
    
    fn get_style(&self) -> &super::style::Style {
        // Return default style for now
        static STYLE: std::sync::OnceLock<super::style::Style> = std::sync::OnceLock::new();
        STYLE.get_or_init(|| {
            super::style::Style::new("custom")
                .background_color(Color::BLUE)
                .foreground_color(Color::WHITE)
        })
    }
    
    fn set_style(&mut self, _style: super::style::Style) {
        self.invalidate();
    }
    
    fn invalidate(&mut self) {
        self.dirty = true;
    }
    
    fn is_dirty(&self) -> bool {
        self.dirty
    }
    
    fn get_parent(&self) -> Option<&dyn Widget> {
        None
    }
    
    fn set_parent(&mut self, _parent: Option<Box<dyn Widget>>) {
        // No-op for this simple widget
    }
    
    fn add_child(&mut self, _child: Box<dyn Widget>) -> GUIResult<()> {
        Err(GUIError::InvalidLayout)
    }
    
    fn remove_child(&mut self, _widget_id: super::WidgetId) -> GUIResult<()> {
        Err(GUIError::InvalidLayout)
    }
    
    fn get_children(&self) -> &[Box<dyn Widget>] {
        &[]
    }
    
    fn render(&self, renderer: &mut dyn super::graphics::Renderer) -> GUIResult<()> {
        if !self.is_visible() {
            return Ok(());
        }
        
        // Draw custom widget content
        renderer.fill_rect(self.bounds, Color::BLUE);
        renderer.draw_rect(self.bounds, Border::new(2, Color::WHITE, super::graphics::BorderStyle::Solid));
        
        // Draw value (simplified)
        // renderer.draw_text(
        //     &format!("{:.1}", self.value),
        //     Point::new(self.bounds.x + 10, self.bounds.y + 25),
        //     super::graphics::Font::new(12, super::graphics::FontFamily::Default),
        //     Color::WHITE
        // );
        
        Ok(())
    }
    
    fn can_handle(&self, event_type: EventType) -> bool {
        matches!(event_type, EventType::MouseDown | EventType::Paint)
    }
    
    fn handle_event(&self, _event: &mut super::events::Event) -> bool {
        // Handle custom widget events
        false
    }
    
    fn get_widget_id(&self) -> super::WidgetId {
        self.id
    }
    
    fn get_constraints(&self) -> &super::layout::LayoutConstraints {
        &super::layout::LayoutConstraints::new()
    }
    
    fn get_bounds(&self) -> Rectangle {
        self.bounds
    }
    
    fn set_bounds(&mut self, rect: Rectangle) {
        self.bounds = rect;
        self.invalidate();
    }
    
    fn get_preferred_size(&self) -> Size {
        Size::new(100, 50)
    }
    
    fn is_visible(&self) -> bool {
        true
    }
}

/// Example of widget factory functions
pub mod widget_factory {
    use super::*;
    
    /// Create a styled button
    pub fn create_styled_button(text: &str, primary: bool) -> Button {
        let mut button = Button::new(text);
        
        let style = if primary {
            super::style::Style::new("primary_button")
                .background_color(Color::BLUE)
                .foreground_color(Color::WHITE)
                .border(Border::new(2, Color::DARK_GRAY, super::graphics::BorderStyle::Solid))
        } else {
            super::style::Style::new("secondary_button")
                .background_color(Color::LIGHT_GRAY)
                .foreground_color(Color::BLACK)
                .border(Border::new(1, Color::GRAY, super::graphics::BorderStyle::Solid))
        };
        
        button.set_style(style);
        button
    }
    
    /// Create a form container with standard styling
    pub fn create_form_container() -> Container {
        let layout = GridLayout::with_dimensions(2, 1)
            .spacing(8)
            .padding(16)
            .alignment(super::layout::Alignment::Start, super::layout::Alignment::Center);
            
        Container::new("form").with_layout(layout)
    }
    
    /// Create a status bar
    pub fn create_status_bar() -> Container {
        let layout = FlowLayout::horizontal()
            .spacing(16)
            .padding(8);
            
        let mut status_bar = Container::new("status_bar").with_layout(layout);
        
        // Add status labels
        let status_label = Label::new("Ready");
        let progress_label = Label::new("100%");
        
        status_bar.add_child(Box::new(status_label)).ok();
        status_bar.add_child(Box::new(progress_label)).ok();
        
        status_bar
    }
}

/// Example of event handling
pub mod event_handlers {
    use super::*;
    
    /// Generic button click handler
    pub fn handle_button_click<W: Widget>(
        widget: &W,
        event: &mut super::events::Event,
        callback: impl Fn(&str),
    ) -> bool {
        if let super::events::EventData::Mouse(mouse_data) = event.get_data() {
            if widget.get_bounds().contains(mouse_data.position) {
                if matches!(event.event_type, EventType::MouseDown) {
                    // Extract button text from widget
                    let button_text = "Button"; // In real implementation, extract from widget
                    callback(button_text);
                    event.consume();
                    return true;
                }
            }
        }
        false
    }
    
    /// Handle keyboard input for text fields
    pub fn handle_keyboard_input(
        event: &mut super::events::Event,
        text_field: &mut TextField,
    ) -> bool {
        if let super::events::EventData::Keyboard(keyboard_data) = event.get_data() {
            match event.event_type {
                EventType::KeyDown => {
                    match keyboard_data.key_char {
                        '\n' | '\r' => {
                            // Enter pressed - could trigger form submission
                            event.consume();
                            return true;
                        }
                        '\x08' => {
                            // Backspace
                            text_field.delete_char_before_cursor();
                            event.consume();
                            return true;
                        }
                        ch if ch.is_ascii() && ch.is_printable() => {
                            // Regular character
                            text_field.append_char(ch);
                            event.consume();
                            return true;
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_example_app_creation() {
        // Test creating an example application
        let app = ExampleApp::new();
        assert!(app.is_ok());
    }
    
    #[test]
    fn test_custom_widget() {
        let widget = CustomWidget::new();
        assert_eq!(widget.get_widget_type(), "CustomWidget");
        assert_eq!(widget.get_value(), 0.0);
        
        widget.set_value(42.0);
        assert_eq!(widget.get_value(), 42.0);
    }
    
    #[test]
    fn test_widget_factory() {
        let button = widget_factory::create_styled_button("Test", true);
        assert_eq!(button.get_text(), "Test");
        
        let form = widget_factory::create_form_container();
        assert_eq!(form.get_name(), "form");
        
        let status_bar = widget_factory::create_status_bar();
        assert_eq!(status_bar.get_name(), "status_bar");
    }
}