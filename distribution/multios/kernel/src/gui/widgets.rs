//! Widget system for GUI toolkit
//! 
//! Provides base widget classes and common UI widgets including
//! buttons, labels, text fields, menus, dialogs, list boxes, and progress bars.

use alloc::vec::Vec;
use alloc::string::String;
use alloc::boxed::Box;
use spin::Mutex;

use super::{WidgetId, GUIResult, GUIError};
use super::events::{Event, EventType, EventHandler, EventDispatcher};
use super::style::{Style, StyleManager};
use super::layout::{LayoutItem, LayoutConstraints, LayoutManager, Layout};
use super::graphics::{Rectangle, Size, Point, Renderer, Color, Border};

/// Base widget trait
pub trait Widget: EventHandler + LayoutItem {
    /// Get the widget type name
    fn get_widget_type(&self) -> &'static str;
    
    /// Get the current style
    fn get_style(&self) -> &Style;
    
    /// Set a new style
    fn set_style(&mut self, style: Style);
    
    /// Mark the widget for redraw
    fn invalidate(&mut self);
    
    /// Check if the widget needs redrawing
    fn is_dirty(&self) -> bool;
    
    /// Get the parent widget
    fn get_parent(&self) -> Option<&dyn Widget>;
    
    /// Set the parent widget
    fn set_parent(&mut self, parent: Option<Box<dyn Widget>>);
    
    /// Add a child widget
    fn add_child(&mut self, child: Box<dyn Widget>) -> GUIResult<()>;
    
    /// Remove a child widget
    fn remove_child(&mut self, widget_id: WidgetId) -> GUIResult<()>;
    
    /// Get children widgets
    fn get_children(&self) -> &[Box<dyn Widget>];
    
    /// Render the widget
    fn render(&self, renderer: &mut dyn Renderer) -> GUIResult<()>;
    
    /// Handle focus events
    fn on_focus(&mut self) {
        self.invalidate();
    }
    
    /// Handle blur events
    fn on_blur(&mut self) {
        self.invalidate();
    }
    
    /// Check if widget accepts focus
    fn can_accept_focus(&self) -> bool {
        true
    }
    
    /// Check if widget is enabled
    fn is_enabled(&self) -> bool {
        self.get_style().enabled.unwrap_or(true)
    }
    
    /// Set enabled state
    fn set_enabled(&mut self, enabled: bool) {
        let mut style = self.get_style().clone();
        style.enabled = Some(enabled);
        self.set_style(style);
        self.invalidate();
    }
    
    /// Check if widget is visible
    fn is_visible(&self) -> bool {
        self.get_style().visible.unwrap_or(true)
    }
    
    /// Set visible state
    fn set_visible(&mut self, visible: bool) {
        let mut style = self.get_style().clone();
        style.visible = Some(visible);
        self.set_style(style);
        self.invalidate();
    }
}

/// Container widget that can hold other widgets
pub struct Container {
    id: WidgetId,
    bounds: Rectangle,
    style: Style,
    children: Vec<Box<dyn Widget>>,
    event_dispatcher: EventDispatcher,
    layout_manager: Option<Box<dyn LayoutManager>>,
    dirty: bool,
    parent: Option<Box<dyn Widget>>,
    name: String,
}

impl Container {
    pub fn new(name: &str) -> Self {
        Self {
            id: WidgetId::new(),
            bounds: Rectangle::new(0, 0, 0, 0),
            style: Style::new("container"),
            children: Vec::new(),
            event_dispatcher: EventDispatcher::new(),
            layout_manager: None,
            dirty: true,
            parent: None,
            name: name.to_string(),
        }
    }

    pub fn with_layout<L: LayoutManager + 'static>(mut self, layout_manager: L) -> Self {
        self.layout_manager = Some(Box::new(layout_manager));
        self
    }

    pub fn get_id(&self) -> WidgetId {
        self.id
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// Apply layout to all children
    pub fn apply_layout(&mut self) -> GUIResult<()> {
        if let Some(layout_manager) = &self.layout_manager {
            layout_manager.layout(self.bounds, &mut self.children)?;
        }
        Ok(())
    }
}

impl Widget for Container {
    fn get_widget_type(&self) -> &'static str {
        "Container"
    }

    fn get_style(&self) -> &Style {
        &self.style
    }

    fn set_style(&mut self, style: Style) {
        self.style = style;
        self.invalidate();
    }

    fn invalidate(&mut self) {
        self.dirty = true;
    }

    fn is_dirty(&self) -> bool {
        self.dirty
    }

    fn get_parent(&self) -> Option<&dyn Widget> {
        self.parent.as_ref().map(|p| p.as_ref())
    }

    fn set_parent(&mut self, parent: Option<Box<dyn Widget>>) {
        self.parent = parent;
    }

    fn add_child(&mut self, child: Box<dyn Widget>) -> GUIResult<()> {
        child.set_parent(Some(Box::new(self as &dyn Widget)));
        self.children.push(child);
        self.invalidate();
        Ok(())
    }

    fn remove_child(&mut self, widget_id: WidgetId) -> GUIResult<()> {
        self.children.retain(|child| child.get_widget_id() != widget_id);
        self.invalidate();
        Ok(())
    }

    fn get_children(&self) -> &[Box<dyn Widget>] {
        &self.children
    }

    fn render(&self, renderer: &mut dyn Renderer) -> GUIResult<()> {
        // Render background if set
        if let Some(bg_color) = self.style.background_color {
            renderer.fill_rect(self.bounds, bg_color);
        }

        // Render border if set
        if let Some(border) = self.style.border {
            renderer.draw_rect(self.bounds, border);
        }

        // Render children
        for child in &self.children {
            if child.is_visible() {
                child.render(renderer)?;
            }
        }

        Ok(())
    }

    fn can_handle(&self, event_type: EventType) -> bool {
        matches!(event_type, 
            EventType::MouseMove | EventType::MouseDown | EventType::MouseUp | 
            EventType::KeyDown | EventType::KeyUp | EventType::Paint | 
            EventType::Resize | EventType::Focus | EventType::Blur)
    }

    fn handle_event(&self, event: &mut Event) -> bool {
        // Container handles events by propagating to children
        match event.event_type {
            EventType::MouseMove | EventType::MouseDown | EventType::MouseUp => {
                // Check if event is within this container's bounds
                if !self.bounds.contains(Point::new(0, 0)) {
                    return false;
                }
            }
            _ => {}
        }
        
        // Propagate to children
        for child in &self.children {
            if child.handle_event(event) && event.is_consumed() {
                return true;
            }
        }
        
        false
    }

    fn get_widget_id(&self) -> WidgetId {
        self.id
    }

    fn get_constraints(&self) -> &LayoutConstraints {
        &LayoutConstraints::new()
    }

    fn get_bounds(&self) -> Rectangle {
        self.bounds
    }

    fn set_bounds(&mut self, rect: Rectangle) {
        self.bounds = rect;
        self.invalidate();
    }

    fn get_preferred_size(&self) -> Size {
        if let Some(layout_manager) = &self.layout_manager {
            layout_manager.preferred_size(&self.children)
        } else {
            // Default size calculation
            let mut width = 0u32;
            let mut height = 0u32;
            
            for child in &self.children {
                if child.is_visible() {
                    let child_size = child.get_preferred_size();
                    width = width.max(child_size.width);
                    height = height.max(child_size.height);
                }
            }
            
            Size::new(width, height)
        }
    }

    fn is_visible(&self) -> bool {
        self.style.visible.unwrap_or(true) && self.parent.map(|p| p.is_visible()).unwrap_or(true)
    }
}

impl EventHandler for Container {
    fn handle_event(&self, event: &mut Event) -> bool {
        Widget::handle_event(self, event)
    }

    fn can_handle(&self, event_type: EventType) -> bool {
        Widget::can_handle(self, event_type)
    }
}

impl LayoutItem for Container {
    fn get_widget_id(&self) -> WidgetId {
        self.id
    }

    fn get_constraints(&self) -> &LayoutConstraints {
        &LayoutConstraints::new()
    }

    fn get_bounds(&self) -> Rectangle {
        self.bounds
    }

    fn set_bounds(&mut self, rect: Rectangle) {
        self.bounds = rect;
        self.invalidate();
    }

    fn get_preferred_size(&self) -> Size {
        Widget::get_preferred_size(self)
    }

    fn is_visible(&self) -> bool {
        Widget::is_visible(self)
    }
}

/// Button widget
pub struct Button {
    id: WidgetId,
    bounds: Rectangle,
    style: Style,
    text: String,
    dirty: bool,
    parent: Option<Box<dyn Widget>>,
    pressed: bool,
    hovered: bool,
}

impl Button {
    pub fn new(text: &str) -> Self {
        Self {
            id: WidgetId::new(),
            bounds: Rectangle::new(0, 0, 80, 30),
            style: Style::new("button")
                .background_color(Color::LIGHT_GRAY)
                .foreground_color(Color::BLACK)
                .border(Border::new(1, Color::DARK_GRAY, super::graphics::BorderStyle::Solid))
                .padding(8),
            text: text.to_string(),
            dirty: true,
            parent: None,
            pressed: false,
            hovered: false,
        }
    }

    pub fn set_text(&mut self, text: &str) {
        self.text = text.to_string();
        self.invalidate();
    }

    pub fn get_text(&self) -> &str {
        &self.text
    }

    pub fn is_pressed(&self) -> bool {
        self.pressed
    }

    pub fn is_hovered(&self) -> bool {
        self.hovered
    }
}

impl Widget for Button {
    fn get_widget_type(&self) -> &'static str {
        "Button"
    }

    fn get_style(&self) -> &Style {
        &self.style
    }

    fn set_style(&mut self, style: Style) {
        self.style = style;
        self.invalidate();
    }

    fn invalidate(&mut self) {
        self.dirty = true;
    }

    fn is_dirty(&self) -> bool {
        self.dirty
    }

    fn get_parent(&self) -> Option<&dyn Widget> {
        self.parent.as_ref().map(|p| p.as_ref())
    }

    fn set_parent(&mut self, parent: Option<Box<dyn Widget>>) {
        self.parent = parent;
    }

    fn add_child(&mut self, _child: Box<dyn Widget>) -> GUIResult<()> {
        Err(GUIError::InvalidLayout)
    }

    fn remove_child(&mut self, _widget_id: WidgetId) -> GUIResult<()> {
        Err(GUIError::InvalidLayout)
    }

    fn get_children(&self) -> &[Box<dyn Widget>] {
        &[]
    }

    fn render(&self, renderer: &mut dyn Renderer) -> GUIResult<()> {
        // Apply hover/pressed state to background color
        let mut bg_color = self.style.background_color.unwrap_or(Color::LIGHT_GRAY);
        
        if self.pressed {
            bg_color = Color::new(bg_color.r.saturating_sub(30), 
                                bg_color.g.saturating_sub(30), 
                                bg_color.b.saturating_sub(30), 
                                bg_color.a);
        } else if self.hovered {
            bg_color = Color::new(bg_color.r.saturating_add(20), 
                                bg_color.g.saturating_add(20), 
                                bg_color.b.saturating_add(20), 
                                bg_color.a);
        }

        // Draw background
        renderer.fill_rect(self.bounds, bg_color);
        
        // Draw border
        if let Some(border) = self.style.border {
            renderer.draw_rect(self.bounds, border);
        }

        // Draw text (simplified)
        let text_color = self.style.foreground_color.unwrap_or(Color::BLACK);
        // renderer.draw_text(&self.text, Point::new(self.bounds.x + 8, self.bounds.y + 16), 
        //                   Font::new(12, super::graphics::FontFamily::Default), text_color);

        self.dirty = false;
        Ok(())
    }

    fn can_handle(&self, event_type: EventType) -> bool {
        matches!(event_type, EventType::MouseDown | EventType::MouseUp | EventType::MouseMove | EventType::Paint)
    }

    fn handle_event(&self, event: &mut Event) -> bool {
        match event.event_type {
            EventType::MouseDown => {
                if let super::events::EventData::Mouse(mouse_data) = event.get_data() {
                    if self.bounds.contains(mouse_data.position) {
                        self.pressed = true;
                        self.dirty = true;
                        return true;
                    }
                }
            }
            EventType::MouseUp => {
                if self.pressed {
                    self.pressed = false;
                    self.dirty = true;
                    
                    // Check if mouse is still over button (click)
                    if let super::events::EventData::Mouse(mouse_data) = event.get_data() {
                        if self.bounds.contains(mouse_data.position) {
                            event.consume();
                            // In a real implementation, this would trigger a click event
                        }
                    }
                    return true;
                }
            }
            EventType::MouseMove => {
                if let super::events::EventData::Mouse(mouse_data) = event.get_data() {
                    let new_hovered = self.bounds.contains(mouse_data.position);
                    if new_hovered != self.hovered {
                        self.hovered = new_hovered;
                        self.dirty = true;
                        return true;
                    }
                }
            }
            _ => {}
        }
        
        false
    }

    fn get_widget_id(&self) -> WidgetId {
        self.id
    }

    fn get_constraints(&self) -> &LayoutConstraints {
        &LayoutConstraints::new()
    }

    fn get_bounds(&self) -> Rectangle {
        self.bounds
    }

    fn set_bounds(&mut self, rect: Rectangle) {
        self.bounds = rect;
        self.invalidate();
    }

    fn get_preferred_size(&self) -> Size {
        // Calculate based on text size + padding
        let text_width = self.text.len() as u32 * 8; // Simplified text measurement
        let width = text_width + self.style.padding.unwrap_or(8) * 2;
        let height = 30; // Default button height
        
        Size::new(width.max(80), height.max(30))
    }

    fn is_visible(&self) -> bool {
        self.style.visible.unwrap_or(true) && self.parent.map(|p| p.is_visible()).unwrap_or(true)
    }
}

impl EventHandler for Button {
    fn handle_event(&self, event: &mut Event) -> bool {
        Widget::handle_event(self, event)
    }

    fn can_handle(&self, event_type: EventType) -> bool {
        Widget::can_handle(self, event_type)
    }
}

impl LayoutItem for Button {
    fn get_widget_id(&self) -> WidgetId {
        self.id
    }

    fn get_constraints(&self) -> &LayoutConstraints {
        &LayoutConstraints::new()
    }

    fn get_bounds(&self) -> Rectangle {
        self.bounds
    }

    fn set_bounds(&mut self, rect: Rectangle) {
        self.bounds = rect;
        self.invalidate();
    }

    fn get_preferred_size(&self) -> Size {
        Widget::get_preferred_size(self)
    }

    fn is_visible(&self) -> bool {
        Widget::is_visible(self)
    }
}

/// Label widget
pub struct Label {
    id: WidgetId,
    bounds: Rectangle,
    style: Style,
    text: String,
    alignment: LabelAlignment,
    dirty: bool,
    parent: Option<Box<dyn Widget>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LabelAlignment {
    Left,
    Center,
    Right,
}

impl Label {
    pub fn new(text: &str) -> Self {
        Self {
            id: WidgetId::new(),
            bounds: Rectangle::new(0, 0, 100, 20),
            style: Style::new("label")
                .background_color(Color::TRANSPARENT)
                .foreground_color(Color::BLACK)
                .padding(4),
            text: text.to_string(),
            alignment: LabelAlignment::Left,
            dirty: true,
            parent: None,
        }
    }

    pub fn set_text(&mut self, text: &str) {
        self.text = text.to_string();
        self.invalidate();
    }

    pub fn get_text(&self) -> &str {
        &self.text
    }

    pub fn set_alignment(&mut self, alignment: LabelAlignment) {
        self.alignment = alignment;
        self.invalidate();
    }

    pub fn get_alignment(&self) -> LabelAlignment {
        self.alignment
    }
}

impl Widget for Label {
    fn get_widget_type(&self) -> &'static str {
        "Label"
    }

    fn get_style(&self) -> &Style {
        &self.style
    }

    fn set_style(&mut self, style: Style) {
        self.style = style;
        self.invalidate();
    }

    fn invalidate(&mut self) {
        self.dirty = true;
    }

    fn is_dirty(&self) -> bool {
        self.dirty
    }

    fn get_parent(&self) -> Option<&dyn Widget> {
        self.parent.as_ref().map(|p| p.as_ref())
    }

    fn set_parent(&mut self, parent: Option<Box<dyn Widget>>) {
        self.parent = parent;
    }

    fn add_child(&mut self, _child: Box<dyn Widget>) -> GUIResult<()> {
        Err(GUIError::InvalidLayout)
    }

    fn remove_child(&mut self, _widget_id: WidgetId) -> GUIResult<()> {
        Err(GUIError::InvalidLayout)
    }

    fn get_children(&self) -> &[Box<dyn Widget>] {
        &[]
    }

    fn render(&self, renderer: &mut dyn Renderer) -> GUIResult<()> {
        // Labels don't draw backgrounds by default (transparent)
        if let Some(bg_color) = self.style.background_color {
            if bg_color != Color::TRANSPARENT {
                renderer.fill_rect(self.bounds, bg_color);
            }
        }

        let text_color = self.style.foreground_color.unwrap_or(Color::BLACK);
        
        // Calculate text position based on alignment
        let padding = self.style.padding.unwrap_or(4);
        let text_x = match self.alignment {
            LabelAlignment::Left => self.bounds.x + padding as i32,
            LabelAlignment::Center => self.bounds.x + (self.bounds.width as i32 - (self.text.len() as i32 * 8)) / 2,
            LabelAlignment::Right => self.bounds.x + self.bounds.width as i32 - (self.text.len() as i32 * 8) - padding as i32,
        };
        let text_y = self.bounds.y + (self.bounds.height as i32 / 2);

        // Draw text (simplified)
        // renderer.draw_text(&self.text, Point::new(text_x, text_y), 
        //                  Font::new(12, super::graphics::FontFamily::Default), text_color);

        self.dirty = false;
        Ok(())
    }

    fn can_handle(&self, event_type: EventType) -> bool {
        matches!(event_type, EventType::Paint)
    }

    fn handle_event(&self, _event: &mut Event) -> bool {
        // Labels don't handle mouse/keyboard events
        false
    }

    fn can_accept_focus(&self) -> bool {
        false
    }

    fn get_widget_id(&self) -> WidgetId {
        self.id
    }

    fn get_constraints(&self) -> &LayoutConstraints {
        &LayoutConstraints::new()
    }

    fn get_bounds(&self) -> Rectangle {
        self.bounds
    }

    fn set_bounds(&mut self, rect: Rectangle) {
        self.bounds = rect;
        self.invalidate();
    }

    fn get_preferred_size(&self) -> Size {
        let text_width = self.text.len() as u32 * 8; // Simplified text measurement
        let width = text_width + self.style.padding.unwrap_or(4) * 2;
        let height = 20; // Default label height
        
        Size::new(width, height.max(20))
    }

    fn is_visible(&self) -> bool {
        self.style.visible.unwrap_or(true) && self.parent.map(|p| p.is_visible()).unwrap_or(true)
    }
}

impl EventHandler for Label {
    fn handle_event(&self, event: &mut Event) -> bool {
        Widget::handle_event(self, event)
    }

    fn can_handle(&self, event_type: EventType) -> bool {
        Widget::can_handle(self, event_type)
    }
}

impl LayoutItem for Label {
    fn get_widget_id(&self) -> WidgetId {
        self.id
    }

    fn get_constraints(&self) -> &LayoutConstraints {
        &LayoutConstraints::new()
    }

    fn get_bounds(&self) -> Rectangle {
        self.bounds
    }

    fn set_bounds(&mut self, rect: Rectangle) {
        self.bounds = rect;
        self.invalidate();
    }

    fn get_preferred_size(&self) -> Size {
        Widget::get_preferred_size(self)
    }

    fn is_visible(&self) -> bool {
        Widget::is_visible(self)
    }
}

/// Text field widget
pub struct TextField {
    id: WidgetId,
    bounds: Rectangle,
    style: Style,
    text: String,
    max_length: Option<usize>,
    dirty: bool,
    parent: Option<Box<dyn Widget>>,
    focused: bool,
    cursor_position: usize,
}

impl TextField {
    pub fn new() -> Self {
        Self {
            id: WidgetId::new(),
            bounds: Rectangle::new(0, 0, 150, 25),
            style: Style::new("textfield")
                .background_color(Color::WHITE)
                .foreground_color(Color::BLACK)
                .border(Border::new(1, Color::GRAY, super::graphics::BorderStyle::Solid))
                .padding(4),
            text: String::new(),
            max_length: None,
            dirty: true,
            parent: None,
            focused: false,
            cursor_position: 0,
        }
    }

    pub fn set_text(&mut self, text: &str) {
        let truncated_text = if let Some(max_len) = self.max_length {
            let mut text = text.to_string();
            if text.len() > max_len {
                text.truncate(max_len);
            }
            text
        } else {
            text.to_string()
        };
        
        self.text = truncated_text;
        self.cursor_position = self.text.len().min(self.cursor_position);
        self.invalidate();
    }

    pub fn get_text(&self) -> &str {
        &self.text
    }

    pub fn set_max_length(&mut self, max_length: usize) {
        self.max_length = Some(max_length);
        if self.text.len() > max_length {
            self.text.truncate(max_length);
            self.cursor_position = self.text.len();
        }
        self.invalidate();
    }

    pub fn clear(&mut self) {
        self.text.clear();
        self.cursor_position = 0;
        self.invalidate();
    }

    pub fn append_char(&mut self, ch: char) {
        if let Some(max_len) = self.max_length {
            if self.text.len() >= max_len {
                return;
            }
        }
        
        let mut chars: Vec<char> = self.text.chars().collect();
        chars.insert(self.cursor_position, ch);
        self.text = chars.into_iter().collect();
        self.cursor_position += 1;
        self.invalidate();
    }

    pub fn delete_char_before_cursor(&mut self) {
        if self.cursor_position > 0 {
            let mut chars: Vec<char> = self.text.chars().collect();
            chars.remove(self.cursor_position - 1);
            self.text = chars.into_iter().collect();
            self.cursor_position -= 1;
            self.invalidate();
        }
    }

    pub fn move_cursor(&mut self, position: usize) {
        self.cursor_position = position.min(self.text.len());
        self.invalidate();
    }

    pub fn get_cursor_position(&self) -> usize {
        self.cursor_position
    }

    pub fn is_focused(&self) -> bool {
        self.focused
    }
}

impl Widget for TextField {
    fn get_widget_type(&self) -> &'static str {
        "TextField"
    }

    fn get_style(&self) -> &Style {
        &self.style
    }

    fn set_style(&mut self, style: Style) {
        self.style = style;
        self.invalidate();
    }

    fn invalidate(&mut self) {
        self.dirty = true;
    }

    fn is_dirty(&self) -> bool {
        self.dirty
    }

    fn get_parent(&self) -> Option<&dyn Widget> {
        self.parent.as_ref().map(|p| p.as_ref())
    }

    fn set_parent(&mut self, parent: Option<Box<dyn Widget>>) {
        self.parent = parent;
    }

    fn add_child(&mut self, _child: Box<dyn Widget>) -> GUIResult<()> {
        Err(GUIError::InvalidLayout)
    }

    fn remove_child(&mut self, _widget_id: WidgetId) -> GUIResult<()> {
        Err(GUIError::InvalidLayout)
    }

    fn get_children(&self) -> &[Box<dyn Widget>] {
        &[]
    }

    fn render(&self, renderer: &mut dyn Renderer) -> GUIResult<()> {
        // Draw background
        let bg_color = self.style.background_color.unwrap_or(Color::WHITE);
        if self.focused {
            // Slightly different color when focused
            let focused_bg = Color::new(bg_color.r.saturating_add(10), 
                                      bg_color.g.saturating_add(10), 
                                      bg_color.b.saturating_add(10), 
                                      bg_color.a);
            renderer.fill_rect(self.bounds, focused_bg);
        } else {
            renderer.fill_rect(self.bounds, bg_color);
        }

        // Draw border
        if let Some(border) = self.style.border {
            renderer.draw_rect(self.bounds, border);
        }

        let text_color = self.style.foreground_color.unwrap_or(Color::BLACK);
        let padding = self.style.padding.unwrap_or(4);

        // Draw text
        let text_x = self.bounds.x + padding as i32;
        let text_y = self.bounds.y + (self.bounds.height as i32 / 2);

        // renderer.draw_text(&self.text, Point::new(text_x, text_y), 
        //                  Font::new(12, super::graphics::FontFamily::Default), text_color);

        // Draw cursor if focused
        if self.focused {
            let cursor_x = text_x + (self.cursor_position as i32 * 8);
            let cursor_rect = Rectangle::new(cursor_x, text_y - 8, 2, 16);
            renderer.fill_rect(cursor_rect, text_color);
        }

        self.dirty = false;
        Ok(())
    }

    fn on_focus(&mut self) {
        self.focused = true;
        self.invalidate();
    }

    fn on_blur(&mut self) {
        self.focused = false;
        self.invalidate();
    }

    fn can_handle(&self, event_type: EventType) -> bool {
        matches!(event_type, EventType::KeyDown | EventType::KeyUp | EventType::KeyPress | 
                           EventType::MouseDown | EventType::Paint | EventType::Focus | EventType::Blur)
    }

    fn handle_event(&self, event: &mut Event) -> bool {
        match event.event_type {
            EventType::MouseDown => {
                if let super::events::EventData::Mouse(mouse_data) = event.get_data() {
                    if self.bounds.contains(mouse_data.position) {
                        event.consume();
                        // Focus the text field
                        return true;
                    }
                }
            }
            EventType::KeyDown => {
                if self.focused {
                    if let super::events::EventData::Keyboard(keyboard_data) = event.get_data() {
                        match keyboard_data.key_char {
                            '\n' | '\r' => {
                                // Enter key - could trigger validation or submission
                            }
                            '\x08' => {
                                // Backspace
                                // self.delete_char_before_cursor();
                                event.consume();
                                return true;
                            }
                            ch if ch.is_ascii() => {
                                // Regular character
                                // self.append_char(ch);
                                event.consume();
                                return true;
                            }
                            _ => {}
                        }
                    }
                }
            }
            EventType::Focus => {
                self.focused = true;
                self.invalidate();
                event.consume();
                return true;
            }
            EventType::Blur => {
                self.focused = false;
                self.invalidate();
                event.consume();
                return true;
            }
            _ => {}
        }
        
        false
    }

    fn can_accept_focus(&self) -> bool {
        true
    }

    fn get_widget_id(&self) -> WidgetId {
        self.id
    }

    fn get_constraints(&self) -> &LayoutConstraints {
        &LayoutConstraints::new()
    }

    fn get_bounds(&self) -> Rectangle {
        self.bounds
    }

    fn set_bounds(&mut self, rect: Rectangle) {
        self.bounds = rect;
        self.invalidate();
    }

    fn get_preferred_size(&self) -> Size {
        let text_width = self.text.len().max(10) as u32 * 8; // Minimum 10 chars wide
        let width = text_width + self.style.padding.unwrap_or(4) * 2;
        let height = 25; // Default text field height
        
        Size::new(width.max(150), height)
    }

    fn is_visible(&self) -> bool {
        self.style.visible.unwrap_or(true) && self.parent.map(|p| p.is_visible()).unwrap_or(true)
    }
}

impl EventHandler for TextField {
    fn handle_event(&self, event: &mut Event) -> bool {
        Widget::handle_event(self, event)
    }

    fn can_handle(&self, event_type: EventType) -> bool {
        Widget::can_handle(self, event_type)
    }
}

impl LayoutItem for TextField {
    fn get_widget_id(&self) -> WidgetId {
        self.id
    }

    fn get_constraints(&self) -> &LayoutConstraints {
        &LayoutConstraints::new()
    }

    fn get_bounds(&self) -> Rectangle {
        self.bounds
    }

    fn set_bounds(&mut self, rect: Rectangle) {
        self.bounds = rect;
        self.invalidate();
    }

    fn get_preferred_size(&self) -> Size {
        Widget::get_preferred_size(self)
    }

    fn is_visible(&self) -> bool {
        Widget::is_visible(self)
    }
}

/// Additional GUI widgets for MultiOS
//! 
//! Extended widget implementations including Menu, Dialog, ListBox, and ProgressBar.

use super::events::MouseButton;

/// Menu widget
pub struct Menu {
    id: WidgetId,
    bounds: Rectangle,
    style: Style,
    items: Vec<MenuItem>,
    dirty: bool,
    parent: Option<Box<dyn Widget>>,
    is_open: bool,
    selected_index: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct MenuItem {
    pub text: String,
    pub enabled: bool,
    pub checked: Option<bool>, // For radio/checkbox items
    pub submenu: Option<Box<Menu>>,
}

impl Menu {
    pub fn new() -> Self {
        Self {
            id: WidgetId::new(),
            bounds: Rectangle::new(0, 0, 120, 25),
            style: Style::new("menu")
                .background_color(Color::WHITE)
                .foreground_color(Color::BLACK)
                .border(Border::new(1, Color::GRAY, super::graphics::BorderStyle::Solid))
                .padding(4),
            items: Vec::new(),
            dirty: true,
            parent: None,
            is_open: false,
            selected_index: None,
        }
    }

    pub fn add_item(&mut self, text: &str) -> usize {
        let item = MenuItem {
            text: text.to_string(),
            enabled: true,
            checked: None,
            submenu: None,
        };
        self.items.push(item);
        self.items.len() - 1
    }

    pub fn add_checkbox_item(&mut self, text: &str, checked: bool) -> usize {
        let item = MenuItem {
            text: text.to_string(),
            enabled: true,
            checked: Some(checked),
            submenu: None,
        };
        self.items.push(item);
        self.items.len() - 1
    }

    pub fn add_submenu(&mut self, text: &str, submenu: Menu) -> usize {
        let item = MenuItem {
            text: text.to_string(),
            enabled: true,
            checked: None,
            submenu: Some(Box::new(submenu)),
        };
        self.items.push(item);
        self.items.len() - 1
    }

    pub fn remove_item(&mut self, index: usize) {
        if index < self.items.len() {
            self.items.remove(index);
            if let Some(selected) = self.selected_index {
                if selected >= index && selected > 0 {
                    self.selected_index = Some(selected - 1);
                }
            }
            self.invalidate();
        }
    }

    pub fn set_item_enabled(&mut self, index: usize, enabled: bool) {
        if let Some(item) = self.items.get_mut(index) {
            item.enabled = enabled;
            self.invalidate();
        }
    }

    pub fn set_item_checked(&mut self, index: usize, checked: bool) {
        if let Some(item) = self.items.get_mut(index) {
            if let Some(ref mut check_state) = item.checked {
                *check_state = checked;
                self.invalidate();
            }
        }
    }

    pub fn is_item_checked(&self, index: usize) -> Option<bool> {
        self.items.get(index).and_then(|item| item.checked)
    }

    pub fn get_selected_index(&self) -> Option<usize> {
        self.selected_index
    }

    pub fn select_item(&mut self, index: usize) {
        if index < self.items.len() && self.items[index].enabled {
            self.selected_index = Some(index);
            self.invalidate();
        }
    }

    pub fn open(&mut self) {
        self.is_open = true;
        self.invalidate();
    }

    pub fn close(&mut self) {
        self.is_open = false;
        self.selected_index = None;
        self.invalidate();
    }

    pub fn toggle(&mut self) {
        if self.is_open {
            self.close();
        } else {
            self.open();
        }
    }

    fn get_item_height(&self) -> u32 {
        24 // Height of each menu item
    }

    fn get_total_height(&self) -> u32 {
        self.items.len() as u32 * self.get_item_height()
    }
}

impl Widget for Menu {
    fn get_widget_type(&self) -> &'static str {
        "Menu"
    }

    fn get_style(&self) -> &Style {
        &self.style
    }

    fn set_style(&mut self, style: Style) {
        self.style = style;
        self.invalidate();
    }

    fn invalidate(&mut self) {
        self.dirty = true;
    }

    fn is_dirty(&self) -> bool {
        self.dirty
    }

    fn get_parent(&self) -> Option<&dyn Widget> {
        self.parent.as_ref().map(|p| p.as_ref())
    }

    fn set_parent(&mut self, parent: Option<Box<dyn Widget>>) {
        self.parent = parent;
    }

    fn add_child(&mut self, _child: Box<dyn Widget>) -> GUIResult<()> {
        Err(GUIError::InvalidLayout)
    }

    fn remove_child(&mut self, _widget_id: WidgetId) -> GUIResult<()> {
        Err(GUIError::InvalidLayout)
    }

    fn get_children(&self) -> &[Box<dyn Widget>] {
        &[]
    }

    fn render(&self, renderer: &mut dyn Renderer) -> GUIResult<()> {
        if !self.is_visible() {
            return Ok(());
        }

        // Draw main menu bar
        if !self.is_open {
            renderer.fill_rect(self.bounds, self.style.background_color.unwrap_or(Color::WHITE));
            
            if let Some(border) = self.style.border {
                renderer.draw_rect(self.bounds, border);
            }

            // Draw menu items text (simplified)
            let text_color = self.style.foreground_color.unwrap_or(Color::BLACK);
            let padding = self.style.padding.unwrap_or(4);
            
            // renderer.draw_text("File", Point::new(self.bounds.x + padding as i32, self.bounds.y + (self.bounds.height as i32 / 2)), 
            //                  Font::new(12, super::graphics::FontFamily::Default), text_color);
        } else {
            // Draw dropdown menu
            let menu_height = self.get_total_height();
            let dropdown_rect = Rectangle::new(self.bounds.x, self.bounds.y + self.bounds.height as i32, 
                                             self.bounds.width, menu_height);
            
            renderer.fill_rect(dropdown_rect, self.style.background_color.unwrap_or(Color::WHITE));
            
            if let Some(border) = self.style.border {
                renderer.draw_rect(dropdown_rect, border);
            }

            // Draw menu items
            for (index, item) in self.items.iter().enumerate() {
                let item_rect = Rectangle::new(
                    dropdown_rect.x,
                    dropdown_rect.y + (index as u32 * self.get_item_height()) as i32,
                    dropdown_rect.width,
                    self.get_item_height()
                );

                // Highlight selected item
                if Some(index) == self.selected_index {
                    renderer.fill_rect(item_rect, Color::BLUE);
                }

                let item_color = if item.enabled {
                    self.style.foreground_color.unwrap_or(Color::BLACK)
                } else {
                    Color::GRAY
                };

                // Draw item text
                let padding = self.style.padding.unwrap_or(4);
                // renderer.draw_text(&item.text, Point::new(item_rect.x + padding as i32, item_rect.y + (item_rect.height as i32 / 2)), 
                //                  Font::new(12, super::graphics::FontFamily::Default), item_color);

                // Draw checkbox if present
                if let Some(checked) = item.checked {
                    let checkbox_rect = Rectangle::new(item_rect.x + (item_rect.width as i32 - 20), 
                                                     item_rect.y + 4, 12, 12);
                    if checked {
                        renderer.fill_rect(checkbox_rect, Color::BLACK);
                    } else {
                        renderer.draw_rect(checkbox_rect, Border::new(1, Color::BLACK, super::graphics::BorderStyle::Solid));
                    }
                }
            }
        }

        self.dirty = false;
        Ok(())
    }

    fn can_handle(&self, event_type: EventType) -> bool {
        matches!(event_type, EventType::MouseDown | EventType::MouseUp | EventType::MouseMove | EventType::Paint)
    }

    fn handle_event(&self, event: &mut Event) -> bool {
        match event.event_type {
            EventType::MouseDown => {
                if let super::events::EventData::Mouse(mouse_data) = event.get_data() {
                    if self.bounds.contains(mouse_data.position) {
                        // Toggle menu
                        event.consume();
                        return true;
                    }
                }
            }
            EventType::MouseMove => {
                if let super::events::EventData::Mouse(mouse_data) = event.get_data() {
                    if self.is_open {
                        // Check which menu item is hovered
                        let item_height = self.get_item_height();
                        let relative_y = mouse_data.position.y - (self.bounds.y + self.bounds.height as i32);
                        
                        if relative_y >= 0 {
                            let index = (relative_y as u32 / item_height) as usize;
                            if index < self.items.len() && self.items[index].enabled {
                                if Some(index) != self.selected_index {
                                    // Update selection
                                    return true;
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        }
        
        false
    }

    fn get_widget_id(&self) -> WidgetId {
        self.id
    }

    fn get_constraints(&self) -> &LayoutConstraints {
        &LayoutConstraints::new()
    }

    fn get_bounds(&self) -> Rectangle {
        self.bounds
    }

    fn set_bounds(&mut self, rect: Rectangle) {
        self.bounds = rect;
        self.invalidate();
    }

    fn get_preferred_size(&self) -> Size {
        if self.is_open {
            Size::new(self.bounds.width, self.get_total_height())
        } else {
            Size::new(80, 25) // Menu bar size
        }
    }

    fn is_visible(&self) -> bool {
        self.style.visible.unwrap_or(true) && self.parent.map(|p| p.is_visible()).unwrap_or(true)
    }
}

impl EventHandler for Menu {
    fn handle_event(&self, event: &mut Event) -> bool {
        Widget::handle_event(self, event)
    }

    fn can_handle(&self, event_type: EventType) -> bool {
        Widget::can_handle(self, event_type)
    }
}

impl LayoutItem for Menu {
    fn get_widget_id(&self) -> WidgetId {
        self.id
    }

    fn get_constraints(&self) -> &LayoutConstraints {
        &LayoutConstraints::new()
    }

    fn get_bounds(&self) -> Rectangle {
        self.bounds
    }

    fn set_bounds(&mut self, rect: Rectangle) {
        self.bounds = rect;
        self.invalidate();
    }

    fn get_preferred_size(&self) -> Size {
        Widget::get_preferred_size(self)
    }

    fn is_visible(&self) -> bool {
        Widget::is_visible(self)
    }
}

/// Dialog widget
pub struct Dialog {
    id: WidgetId,
    bounds: Rectangle,
    style: Style,
    title: String,
    content: Option<Box<dyn Widget>>,
    buttons: Vec<DialogButton>,
    dirty: bool,
    parent: Option<Box<dyn Widget>>,
    modal: bool,
    closable: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogButton {
    Ok,
    Cancel,
    Yes,
    No,
    Custom(&'static str),
}

impl Dialog {
    pub fn new(title: &str) -> Self {
        Self {
            id: WidgetId::new(),
            bounds: Rectangle::new(0, 0, 300, 200),
            style: Style::new("dialog")
                .background_color(Color::WHITE)
                .border(Border::new(2, Color::GRAY, super::graphics::BorderStyle::Solid))
                .padding(16),
            title: title.to_string(),
            content: None,
            buttons: Vec::new(),
            dirty: true,
            parent: None,
            modal: true,
            closable: true,
        }
    }

    pub fn set_content(&mut self, content: Box<dyn Widget>) {
        self.content = Some(content);
        self.invalidate();
    }

    pub fn add_button(&mut self, button: DialogButton) {
        self.buttons.push(button);
        self.invalidate();
    }

    pub fn set_modal(&mut self, modal: bool) {
        self.modal = modal;
    }

    pub fn set_closable(&mut self, closable: bool) {
        self.closable = closable;
    }

    pub fn show(&mut self) -> GUIResult<()> {
        // Center the dialog on the screen
        // This would need access to screen dimensions
        self.invalidate();
        Ok(())
    }

    pub fn hide(&mut self) {
        self.invalidate();
    }

    pub fn close(&mut self) {
        self.hide();
        // In a real implementation, this would trigger a close event
    }

    fn get_button_height(&self) -> u32 {
        30
    }

    fn get_title_bar_height(&self) -> u32 {
        30
    }
}

impl Widget for Dialog {
    fn get_widget_type(&self) -> &'static str {
        "Dialog"
    }

    fn get_style(&self) -> &Style {
        &self.style
    }

    fn set_style(&mut self, style: Style) {
        self.style = style;
        self.invalidate();
    }

    fn invalidate(&mut self) {
        self.dirty = true;
    }

    fn is_dirty(&self) -> bool {
        self.dirty
    }

    fn get_parent(&self) -> Option<&dyn Widget> {
        self.parent.as_ref().map(|p| p.as_ref())
    }

    fn set_parent(&mut self, parent: Option<Box<dyn Widget>>) {
        self.parent = parent;
    }

    fn add_child(&mut self, child: Box<dyn Widget>) -> GUIResult<()> {
        if self.content.is_none() {
            self.set_content(child);
            Ok(())
        } else {
            Err(GUIError::InvalidLayout)
        }
    }

    fn remove_child(&mut self, _widget_id: WidgetId) -> GUIResult<()> {
        self.content = None;
        Ok(())
    }

    fn get_children(&self) -> &[Box<dyn Widget>] {
        if let Some(ref content) = self.content {
            core::slice::from_ref(content.as_ref())
        } else {
            &[]
        }
    }

    fn render(&self, renderer: &mut dyn Renderer) -> GUIResult<()> {
        if !self.is_visible() {
            return Ok(());
        }

        let title_bar_height = self.get_title_bar_height();
        let button_height = if !self.buttons.is_empty() { self.get_button_height() } else { 0 };

        // Draw dialog background
        renderer.fill_rect(self.bounds, self.style.background_color.unwrap_or(Color::WHITE));

        // Draw border
        if let Some(border) = self.style.border {
            renderer.draw_rect(self.bounds, border);
        }

        // Draw title bar
        let title_bar_rect = Rectangle::new(self.bounds.x, self.bounds.y, self.bounds.width, title_bar_height);
        renderer.fill_rect(title_bar_rect, Color::DARK_GRAY);

        // Draw title text
        let title_color = Color::WHITE;
        // renderer.draw_text(&self.title, Point::new(self.bounds.x + 8, self.bounds.y + (title_bar_height as i32 / 2)), 
        //                  Font::new(14, super::graphics::FontFamily::Default), title_color);

        // Draw close button if closable
        if self.closable {
            let close_button_rect = Rectangle::new(self.bounds.x + self.bounds.width as i32 - 25, 
                                                  self.bounds.y + 5, 15, 15);
            renderer.draw_rect(close_button_rect, Border::new(1, Color::WHITE, super::graphics::BorderStyle::Solid));
            // renderer.draw_text("X", Point::new(close_button_rect.x + 3, close_button_rect.y + 10), 
            //                  Font::new(10, super::graphics::FontFamily::Default), Color::WHITE);
        }

        // Draw content area
        let content_rect = Rectangle::new(
            self.bounds.x + self.style.padding.unwrap_or(16) as i32,
            self.bounds.y + title_bar_height as i32 + 8,
            self.bounds.width - (self.style.padding.unwrap_or(16) * 2) as u32,
            self.bounds.height - title_bar_height - button_height - 24
        );

        // Draw content if present
        if let Some(ref content) = self.content {
            content.render(renderer)?;
        }

        // Draw buttons
        if !self.buttons.is_empty() {
            let buttons_rect = Rectangle::new(
                self.bounds.x + self.style.padding.unwrap_or(16) as i32,
                self.bounds.y + self.bounds.height as i32 - button_height as i32 - 8,
                self.bounds.width - (self.style.padding.unwrap_or(16) * 2) as u32,
                button_height
            );

            let button_width = buttons_rect.width / self.buttons.len() as u32;
            
            for (index, button) in self.buttons.iter().enumerate() {
                let button_rect = Rectangle::new(
                    buttons_rect.x + (index as u32 * button_width) as i32,
                    buttons_rect.y,
                    button_width - 4,
                    button_height
                );

                // Draw button background
                renderer.fill_rect(button_rect, Color::LIGHT_GRAY);
                renderer.draw_rect(button_rect, Border::new(1, Color::GRAY, super::graphics::BorderStyle::Solid));

                // Draw button text
                let button_text = match button {
                    DialogButton::Ok => "OK",
                    DialogButton::Cancel => "Cancel",
                    DialogButton::Yes => "Yes",
                    DialogButton::No => "No",
                    DialogButton::Custom(text) => text,
                };

                let button_color = Color::BLACK;
                // renderer.draw_text(button_text, Point::new(button_rect.x + 8, button_rect.y + (button_height as i32 / 2)), 
                //                  Font::new(12, super::graphics::FontFamily::Default), button_color);
            }
        }

        self.dirty = false;
        Ok(())
    }

    fn can_handle(&self, event_type: EventType) -> bool {
        matches!(event_type, EventType::MouseDown | EventType::Paint)
    }

    fn handle_event(&self, event: &mut Event) -> bool {
        match event.event_type {
            EventType::MouseDown => {
                if let super::events::EventData::Mouse(mouse_data) = event.get_data() {
                    // Handle close button click
                    if self.closable {
                        let close_button_rect = Rectangle::new(self.bounds.x + self.bounds.width as i32 - 25, 
                                                              self.bounds.y + 5, 15, 15);
                        if close_button_rect.contains(mouse_data.position) {
                            self.close();
                            event.consume();
                            return true;
                        }
                    }
                }
            }
            _ => {}
        }
        
        false
    }

    fn get_widget_id(&self) -> WidgetId {
        self.id
    }

    fn get_constraints(&self) -> &LayoutConstraints {
        &LayoutConstraints::new()
    }

    fn get_bounds(&self) -> Rectangle {
        self.bounds
    }

    fn set_bounds(&mut self, rect: Rectangle) {
        self.bounds = rect;
        self.invalidate();
    }

    fn get_preferred_size(&self) -> Size {
        Size::new(300, 200)
    }

    fn is_visible(&self) -> bool {
        self.style.visible.unwrap_or(true) && self.parent.map(|p| p.is_visible()).unwrap_or(true)
    }
}

impl EventHandler for Dialog {
    fn handle_event(&self, event: &mut Event) -> bool {
        Widget::handle_event(self, event)
    }

    fn can_handle(&self, event_type: EventType) -> bool {
        Widget::can_handle(self, event_type)
    }
}

impl LayoutItem for Dialog {
    fn get_widget_id(&self) -> WidgetId {
        self.id
    }

    fn get_constraints(&self) -> &LayoutConstraints {
        &LayoutConstraints::new()
    }

    fn get_bounds(&self) -> Rectangle {
        self.bounds
    }

    fn set_bounds(&mut self, rect: Rectangle) {
        self.bounds = rect;
        self.invalidate();
    }

    fn get_preferred_size(&self) -> Size {
        Widget::get_preferred_size(self)
    }

    fn is_visible(&self) -> bool {
        Widget::is_visible(self)
    }
}

/// List box widget
pub struct ListBox {
    id: WidgetId,
    bounds: Rectangle,
    style: Style,
    items: Vec<String>,
    selected_index: Option<usize>,
    dirty: bool,
    parent: Option<Box<dyn Widget>>,
    scroll_offset: usize,
    item_height: u32,
}

impl ListBox {
    pub fn new() -> Self {
        Self {
            id: WidgetId::new(),
            bounds: Rectangle::new(0, 0, 200, 150),
            style: Style::new("listbox")
                .background_color(Color::WHITE)
                .foreground_color(Color::BLACK)
                .border(Border::new(1, Color::GRAY, super::graphics::BorderStyle::Solid))
                .padding(4),
            items: Vec::new(),
            selected_index: None,
            dirty: true,
            parent: None,
            scroll_offset: 0,
            item_height: 20,
        }
    }

    pub fn add_item(&mut self, text: &str) {
        self.items.push(text.to_string());
        self.invalidate();
    }

    pub fn remove_item(&mut self, index: usize) {
        if index < self.items.len() {
            self.items.remove(index);
            if let Some(selected) = self.selected_index {
                if selected >= index {
                    self.selected_index = Some(selected.saturating_sub(1));
                }
            }
            self.invalidate();
        }
    }

    pub fn clear(&mut self) {
        self.items.clear();
        self.selected_index = None;
        self.scroll_offset = 0;
        self.invalidate();
    }

    pub fn get_item_count(&self) -> usize {
        self.items.len()
    }

    pub fn get_selected_index(&self) -> Option<usize> {
        self.selected_index
    }

    pub fn get_selected_item(&self) -> Option<&str> {
        self.selected_index.and_then(|i| self.items.get(i).map(|s| s.as_str()))
    }

    pub fn select_item(&mut self, index: usize) {
        if index < self.items.len() {
            self.selected_index = Some(index);
            
            // Ensure selected item is visible
            let visible_items = self.bounds.height / self.item_height;
            if index < self.scroll_offset {
                self.scroll_offset = index;
            } else if index >= self.scroll_offset + visible_items as usize {
                self.scroll_offset = index - visible_items as usize + 1;
            }
            
            self.invalidate();
        }
    }

    pub fn set_items(&mut self, items: Vec<String>) {
        self.items = items;
        self.selected_index = None;
        self.scroll_offset = 0;
        self.invalidate();
    }

    fn get_visible_item_count(&self) -> usize {
        (self.bounds.height / self.item_height) as usize
    }
}

impl Widget for ListBox {
    fn get_widget_type(&self) -> &'static str {
        "ListBox"
    }

    fn get_style(&self) -> &Style {
        &self.style
    }

    fn set_style(&mut self, style: Style) {
        self.style = style;
        self.invalidate();
    }

    fn invalidate(&mut self) {
        self.dirty = true;
    }

    fn is_dirty(&self) -> bool {
        self.dirty
    }

    fn get_parent(&self) -> Option<&dyn Widget> {
        self.parent.as_ref().map(|p| p.as_ref())
    }

    fn set_parent(&mut self, parent: Option<Box<dyn Widget>>) {
        self.parent = parent;
    }

    fn add_child(&mut self, _child: Box<dyn Widget>) -> GUIResult<()> {
        Err(GUIError::InvalidLayout)
    }

    fn remove_child(&mut self, _widget_id: WidgetId) -> GUIResult<()> {
        Err(GUIError::InvalidLayout)
    }

    fn get_children(&self) -> &[Box<dyn Widget>] {
        &[]
    }

    fn render(&self, renderer: &mut dyn Renderer) -> GUIResult<()> {
        if !self.is_visible() {
            return Ok(());
        }

        // Draw background
        renderer.fill_rect(self.bounds, self.style.background_color.unwrap_or(Color::WHITE));

        // Draw border
        if let Some(border) = self.style.border {
            renderer.draw_rect(self.bounds, border);
        }

        let padding = self.style.padding.unwrap_or(4);
        let visible_count = self.get_visible_item_count();
        
        // Draw visible items
        for i in 0..visible_count {
            let item_index = self.scroll_offset + i;
            if item_index >= self.items.len() {
                break;
            }

            let item_y = self.bounds.y + (i as u32 * self.item_height) as i32;
            let item_rect = Rectangle::new(self.bounds.x, item_y, self.bounds.width, self.item_height);

            // Highlight selected item
            if Some(item_index) == self.selected_index {
                renderer.fill_rect(item_rect, Color::BLUE);
            }

            // Draw item text
            let item_color = if Some(item_index) == self.selected_index {
                Color::WHITE
            } else {
                self.style.foreground_color.unwrap_or(Color::BLACK)
            };

            // renderer.draw_text(&self.items[item_index], Point::new(self.bounds.x + padding as i32, item_y + (self.item_height as i32 / 2)), 
            //                  Font::new(12, super::graphics::FontFamily::Default), item_color);
        }

        // Draw scroll bar if needed
        if self.items.len() > visible_count {
            let scrollbar_width = 16;
            let scrollbar_rect = Rectangle::new(
                self.bounds.x + self.bounds.width as i32 - scrollbar_width as i32,
                self.bounds.y,
                scrollbar_width,
                self.bounds.height
            );

            renderer.fill_rect(scrollbar_rect, Color::LIGHT_GRAY);
            renderer.draw_rect(scrollbar_rect, Border::new(1, Color::GRAY, super::graphics::BorderStyle::Solid));

            // Draw scrollbar thumb
            let thumb_height = (self.bounds.height * visible_count as u32) / self.items.len() as u32;
            let thumb_y = self.bounds.y + (self.scroll_offset as u32 * self.bounds.height) / self.items.len() as u32;
            let thumb_rect = Rectangle::new(scrollbar_rect.x + 2, thumb_y as i32, scrollbar_width - 4, thumb_height);
            
            renderer.fill_rect(thumb_rect, Color::GRAY);
        }

        self.dirty = false;
        Ok(())
    }

    fn can_handle(&self, event_type: EventType) -> bool {
        matches!(event_type, EventType::MouseDown | EventType::MouseUp | EventType::MouseMove | EventType::Paint)
    }

    fn handle_event(&self, event: &mut Event) -> bool {
        match event.event_type {
            EventType::MouseDown => {
                if let super::events::EventData::Mouse(mouse_data) = event.get_data() {
                    if self.bounds.contains(mouse_data.position) {
                        // Calculate which item was clicked
                        let relative_y = mouse_data.position.y - self.bounds.y;
                        let item_index = self.scroll_offset + (relative_y as u32 / self.item_height) as usize;
                        
                        if item_index < self.items.len() {
                            // Select the item
                            self.select_item(item_index);
                            event.consume();
                            return true;
                        }
                    }
                }
            }
            EventType::MouseMove => {
                if let super::events::EventData::Mouse(mouse_data) = event.get_data() {
                    if self.bounds.contains(mouse_data.position) {
                        // Could implement hover effects
                        return true;
                    }
                }
            }
            _ => {}
        }
        
        false
    }

    fn can_accept_focus(&self) -> bool {
        true
    }

    fn get_widget_id(&self) -> WidgetId {
        self.id
    }

    fn get_constraints(&self) -> &LayoutConstraints {
        &LayoutConstraints::new()
    }

    fn get_bounds(&self) -> Rectangle {
        self.bounds
    }

    fn set_bounds(&mut self, rect: Rectangle) {
        self.bounds = rect;
        self.invalidate();
    }

    fn get_preferred_size(&self) -> Size {
        Size::new(200, 150)
    }

    fn is_visible(&self) -> bool {
        self.style.visible.unwrap_or(true) && self.parent.map(|p| p.is_visible()).unwrap_or(true)
    }
}

impl EventHandler for ListBox {
    fn handle_event(&self, event: &mut Event) -> bool {
        Widget::handle_event(self, event)
    }

    fn can_handle(&self, event_type: EventType) -> bool {
        Widget::can_handle(self, event_type)
    }
}

impl LayoutItem for ListBox {
    fn get_widget_id(&self) -> WidgetId {
        self.id
    }

    fn get_constraints(&self) -> &LayoutConstraints {
        &LayoutConstraints::new()
    }

    fn get_bounds(&self) -> Rectangle {
        self.bounds
    }

    fn set_bounds(&mut self, rect: Rectangle) {
        self.bounds = rect;
        self.invalidate();
    }

    fn get_preferred_size(&self) -> Size {
        Widget::get_preferred_size(self)
    }

    fn is_visible(&self) -> bool {
        Widget::is_visible(self)
    }
}

/// Progress bar widget
pub struct ProgressBar {
    id: WidgetId,
    bounds: Rectangle,
    style: Style,
    value: f32, // 0.0 to 1.0
    dirty: bool,
    parent: Option<Box<dyn Widget>>,
    indeterminate: bool,
}

impl ProgressBar {
    pub fn new() -> Self {
        Self {
            id: WidgetId::new(),
            bounds: Rectangle::new(0, 0, 200, 20),
            style: Style::new("progressbar")
                .background_color(Color::LIGHT_GRAY)
                .foreground_color(Color::BLUE)
                .border(Border::new(1, Color::GRAY, super::graphics::BorderStyle::Solid)),
            value: 0.0,
            dirty: true,
            parent: None,
            indeterminate: false,
        }
    }

    pub fn set_value(&mut self, value: f32) {
        self.value = value.clamp(0.0, 1.0);
        self.indeterminate = false;
        self.invalidate();
    }

    pub fn get_value(&self) -> f32 {
        self.value
    }

    pub fn set_indeterminate(&mut self, indeterminate: bool) {
        self.indeterminate = indeterminate;
        self.invalidate();
    }

    pub fn is_indeterminate(&self) -> bool {
        self.indeterminate
    }

    pub fn reset(&mut self) {
        self.value = 0.0;
        self.indeterminate = false;
        self.invalidate();
    }

    pub fn complete(&mut self) {
        self.value = 1.0;
        self.invalidate();
    }
}

impl Widget for ProgressBar {
    fn get_widget_type(&self) -> &'static str {
        "ProgressBar"
    }

    fn get_style(&self) -> &Style {
        &self.style
    }

    fn set_style(&mut self, style: Style) {
        self.style = style;
        self.invalidate();
    }

    fn invalidate(&mut self) {
        self.dirty = true;
    }

    fn is_dirty(&self) -> bool {
        self.dirty
    }

    fn get_parent(&self) -> Option<&dyn Widget> {
        self.parent.as_ref().map(|p| p.as_ref())
    }

    fn set_parent(&mut self, parent: Option<Box<dyn Widget>>) {
        self.parent = parent;
    }

    fn add_child(&mut self, _child: Box<dyn Widget>) -> GUIResult<()> {
        Err(GUIError::InvalidLayout)
    }

    fn remove_child(&mut self, _widget_id: WidgetId) -> GUIResult<()> {
        Err(GUIError::InvalidLayout)
    }

    fn get_children(&self) -> &[Box<dyn Widget>] {
        &[]
    }

    fn render(&self, renderer: &mut dyn Renderer) -> GUIResult<()> {
        if !self.is_visible() {
            return Ok(());
        }

        // Draw background
        renderer.fill_rect(self.bounds, self.style.background_color.unwrap_or(Color::LIGHT_GRAY));

        // Draw border
        if let Some(border) = self.style.border {
            renderer.draw_rect(self.bounds, border);
        }

        // Draw progress
        if self.indeterminate {
            // Draw animated indeterminate pattern (simplified)
            let pattern_width = self.bounds.width / 4;
            for i in 0..4 {
                let x = self.bounds.x + (i as i32 * pattern_width as i32);
                let pattern_rect = Rectangle::new(x, self.bounds.y + 2, pattern_width / 2, self.bounds.height - 4);
                renderer.fill_rect(pattern_rect, self.style.foreground_color.unwrap_or(Color::BLUE));
            }
        } else {
            // Draw progress bar
            let progress_width = (self.bounds.width as f32 * self.value) as u32;
            if progress_width > 0 {
                let progress_rect = Rectangle::new(
                    self.bounds.x + 1,
                    self.bounds.y + 1,
                    progress_width - 2,
                    self.bounds.height - 2
                );
                renderer.fill_rect(progress_rect, self.style.foreground_color.unwrap_or(Color::BLUE));
            }
        }

        self.dirty = false;
        Ok(())
    }

    fn can_handle(&self, event_type: EventType) -> bool {
        matches!(event_type, EventType::Paint)
    }

    fn handle_event(&self, _event: &mut Event) -> bool {
        // Progress bars don't handle user input
        false
    }

    fn can_accept_focus(&self) -> bool {
        false
    }

    fn get_widget_id(&self) -> WidgetId {
        self.id
    }

    fn get_constraints(&self) -> &LayoutConstraints {
        &LayoutConstraints::new()
    }

    fn get_bounds(&self) -> Rectangle {
        self.bounds
    }

    fn set_bounds(&mut self, rect: Rectangle) {
        self.bounds = rect;
        self.invalidate();
    }

    fn get_preferred_size(&self) -> Size {
        Size::new(200, 20)
    }

    fn is_visible(&self) -> bool {
        self.style.visible.unwrap_or(true) && self.parent.map(|p| p.is_visible()).unwrap_or(true)
    }
}

impl EventHandler for ProgressBar {
    fn handle_event(&self, event: &mut Event) -> bool {
        Widget::handle_event(self, event)
    }

    fn can_handle(&self, event_type: EventType) -> bool {
        Widget::can_handle(self, event_type)
    }
}

impl LayoutItem for ProgressBar {
    fn get_widget_id(&self) -> WidgetId {
        self.id
    }

    fn get_constraints(&self) -> &LayoutConstraints {
        &LayoutConstraints::new()
    }

    fn get_bounds(&self) -> Rectangle {
        self.bounds
    }

    fn set_bounds(&mut self, rect: Rectangle) {
        self.bounds = rect;
        self.invalidate();
    }

    fn get_preferred_size(&self) -> Size {
        Widget::get_preferred_size(self)
    }

    fn is_visible(&self) -> bool {
        Widget::is_visible(self)
    }
}