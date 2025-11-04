//! Style system for GUI toolkit
//! 
//! Provides styling, theming, and appearance management for GUI widgets.

use alloc::vec::Vec;
use alloc::string::String;
use spin::Mutex;

use super::{GUIResult, GUIError};
use super::graphics::{Color, Font, Border, FontFamily};

/// Style for GUI widgets
#[derive(Debug, Clone)]
pub struct Style {
    pub name: String,
    pub background_color: Option<Color>,
    pub foreground_color: Option<Color>,
    pub border: Option<Border>,
    pub font: Option<Font>,
    pub padding: Option<u32>,
    pub margin: Option<u32>,
    pub min_width: Option<u32>,
    pub min_height: Option<u32>,
    pub max_width: Option<u32>,
    pub max_height: Option<u32>,
    pub opacity: Option<f32>,
    pub visible: Option<bool>,
    pub enabled: Option<bool>,
}

impl Style {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            background_color: None,
            foreground_color: None,
            border: None,
            font: None,
            padding: None,
            margin: None,
            min_width: None,
            min_height: None,
            max_width: None,
            max_height: None,
            opacity: None,
            visible: None,
            enabled: None,
        }
    }

    /// Set background color
    pub fn background_color(mut self, color: Color) -> Self {
        self.background_color = Some(color);
        self
    }

    /// Set foreground color
    pub fn foreground_color(mut self, color: Color) -> Self {
        self.foreground_color = Some(color);
        self
    }

    /// Set border
    pub fn border(mut self, border: Border) -> Self {
        self.border = Some(border);
        self
    }

    /// Set font
    pub fn font(mut self, font: Font) -> Self {
        self.font = Some(font);
        self
    }

    /// Set padding
    pub fn padding(mut self, padding: u32) -> Self {
        self.padding = Some(padding);
        self
    }

    /// Set margin
    pub fn margin(mut self, margin: u32) -> Self {
        self.margin = Some(margin);
        self
    }

    /// Set minimum dimensions
    pub fn min_size(mut self, width: u32, height: u32) -> Self {
        self.min_width = Some(width);
        self.min_height = Some(height);
        self
    }

    /// Set maximum dimensions
    pub fn max_size(mut self, width: u32, height: u32) -> Self {
        self.max_width = Some(width);
        self.max_height = Some(height);
        self
    }

    /// Set opacity (0.0 to 1.0)
    pub fn opacity(mut self, opacity: f32) -> Self {
        self.opacity = Some(opacity);
        self
    }

    /// Set visibility
    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = Some(visible);
        self
    }

    /// Set enabled state
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = Some(enabled);
        self
    }

    /// Merge another style into this one
    pub fn merge(mut self, other: &Style) -> Self {
        if other.background_color.is_some() {
            self.background_color = other.background_color;
        }
        if other.foreground_color.is_some() {
            self.foreground_color = other.foreground_color;
        }
        if other.border.is_some() {
            self.border = other.border;
        }
        if other.font.is_some() {
            self.font = other.font;
        }
        if other.padding.is_some() {
            self.padding = other.padding;
        }
        if other.margin.is_some() {
            self.margin = other.margin;
        }
        if other.min_width.is_some() {
            self.min_width = other.min_width;
        }
        if other.min_height.is_some() {
            self.min_height = other.min_height;
        }
        if other.max_width.is_some() {
            self.max_width = other.max_width;
        }
        if other.max_height.is_some() {
            self.max_height = other.max_height;
        }
        if other.opacity.is_some() {
            self.opacity = other.opacity;
        }
        if other.visible.is_some() {
            self.visible = other.visible;
        }
        if other.enabled.is_some() {
            self.enabled = other.enabled;
        }
        self
    }

    /// Check if style has any properties set
    pub fn is_empty(&self) -> bool {
        self.background_color.is_none() &&
        self.foreground_color.is_none() &&
        self.border.is_none() &&
        self.font.is_none() &&
        self.padding.is_none() &&
        self.margin.is_none() &&
        self.min_width.is_none() &&
        self.min_height.is_none() &&
        self.max_width.is_none() &&
        self.max_height.is_none() &&
        self.opacity.is_none() &&
        self.visible.is_none() &&
        self.enabled.is_none()
    }
}

/// Theme for consistent styling across the application
#[derive(Debug, Clone)]
pub struct Theme {
    pub name: String,
    pub styles: Vec<Style>,
    pub base_style: Style,
}

impl Theme {
    pub fn new(name: &str) -> Self {
        let mut theme = Self {
            name: name.to_string(),
            styles: Vec::new(),
            base_style: Style::new("base"),
        };

        // Set default base style properties
        theme.base_style = theme.base_style
            .background_color(Color::WHITE)
            .foreground_color(Color::BLACK)
            .padding(8)
            .margin(4)
            .min_size(0, 0)
            .max_size(0, 0)
            .opacity(1.0)
            .visible(true)
            .enabled(true);

        theme
    }

    /// Add a style to the theme
    pub fn add_style(mut self, style: Style) -> Self {
        self.styles.push(style);
        self
    }

    /// Get a style by name
    pub fn get_style(&self, name: &str) -> Option<&Style> {
        for style in &self.styles {
            if style.name == name {
                return Some(style);
            }
        }
        None
    }

    /// Create a new style derived from theme properties
    pub fn create_style(&self, name: &str) -> Style {
        let mut style = Style::new(name);
        
        // Start with base style
        style = style.merge(&self.base_style);
        
        // Apply theme-specific styles if they exist
        if let Some(theme_style) = self.get_style(name) {
            style = style.merge(theme_style);
        }
        
        style
    }
}

/// Built-in theme variants
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeVariant {
    Default,
    Light,
    Dark,
    HighContrast,
}

/// Style manager for managing themes and styles
pub struct StyleManager {
    current_theme: Theme,
    themes: Vec<Theme>,
}

impl StyleManager {
    pub fn new() -> Self {
        let mut manager = Self {
            current_theme: Theme::new("default"),
            themes: Vec::new(),
        };

        // Initialize default theme with built-in styles
        manager.initialize_default_theme();
        manager
    }

    /// Initialize the default theme with common widget styles
    fn initialize_default_theme(&mut self) {
        // Button styles
        let button_style = Style::new("button")
            .background_color(Color::LIGHT_GRAY)
            .foreground_color(Color::BLACK)
            .border(Border::new(1, Color::DARK_GRAY, super::graphics::BorderStyle::Solid))
            .padding(8)
            .min_size(80, 30);

        // Label styles
        let label_style = Style::new("label")
            .background_color(Color::TRANSPARENT)
            .foreground_color(Color::BLACK)
            .padding(4);

        // Text field styles
        let text_field_style = Style::new("textfield")
            .background_color(Color::WHITE)
            .foreground_color(Color::BLACK)
            .border(Border::new(1, Color::GRAY, super::graphics::BorderStyle::Solid))
            .padding(4)
            .min_size(150, 25);

        // Window styles
        let window_style = Style::new("window")
            .background_color(Color::WHITE)
            .border(Border::new(2, Color::GRAY, super::graphics::BorderStyle::Solid))
            .padding(8);

        // List box styles
        let list_box_style = Style::new("listbox")
            .background_color(Color::WHITE)
            .foreground_color(Color::BLACK)
            .border(Border::new(1, Color::GRAY, super::graphics::BorderStyle::Solid))
            .padding(4)
            .min_size(200, 100);

        // Progress bar styles
        let progress_bar_style = Style::new("progressbar")
            .background_color(Color::LIGHT_GRAY)
            .foreground_color(Color::BLUE)
            .border(Border::new(1, Color::GRAY, super::graphics::BorderStyle::Solid))
            .min_size(200, 20);

        // Menu styles
        let menu_style = Style::new("menu")
            .background_color(Color::WHITE)
            .foreground_color(Color::BLACK)
            .border(Border::new(1, Color::GRAY, super::graphics::BorderStyle::Solid))
            .padding(4);

        self.current_theme = self.current_theme
            .add_style(button_style)
            .add_style(label_style)
            .add_style(text_field_style)
            .add_style(window_style)
            .add_style(list_box_style)
            .add_style(progress_bar_style)
            .add_style(menu_style);

        self.themes.push(self.current_theme.clone());
    }

    /// Load a built-in theme variant
    pub fn load_theme_variant(&mut self, variant: ThemeVariant) -> GUIResult<()> {
        match variant {
            ThemeVariant::Default => {
                // Already loaded as default
                Ok(())
            }
            ThemeVariant::Light => {
                self.create_light_theme()
            }
            ThemeVariant::Dark => {
                self.create_dark_theme()
            }
            ThemeVariant::HighContrast => {
                self.create_high_contrast_theme()
            }
        }
    }

    /// Create a light theme
    fn create_light_theme(&mut self) -> GUIResult<()> {
        let mut theme = Theme::new("light");

        let button_style = Style::new("button")
            .background_color(Color::WHITE)
            .foreground_color(Color::BLACK)
            .border(Border::new(1, Color::DARK_GRAY, super::graphics::BorderStyle::Solid));

        let label_style = Style::new("label")
            .background_color(Color::TRANSPARENT)
            .foreground_color(Color::BLACK);

        let text_field_style = Style::new("textfield")
            .background_color(Color::WHITE)
            .foreground_color(Color::BLACK)
            .border(Border::new(1, Color::GRAY, super::graphics::BorderStyle::Solid));

        let window_style = Style::new("window")
            .background_color(Color::WHITE)
            .border(Border::new(2, Color::GRAY, super::graphics::BorderStyle::Solid));

        theme = theme
            .add_style(button_style)
            .add_style(label_style)
            .add_style(text_field_style)
            .add_style(window_style);

        self.themes.push(theme);
        Ok(())
    }

    /// Create a dark theme
    fn create_dark_theme(&mut self) -> GUIResult<()> {
        let mut theme = Theme::new("dark");

        let button_style = Style::new("button")
            .background_color(Color::DARK_GRAY)
            .foreground_color(Color::WHITE)
            .border(Border::new(1, Color::WHITE, super::graphics::BorderStyle::Solid));

        let label_style = Style::new("label")
            .background_color(Color::TRANSPARENT)
            .foreground_color(Color::WHITE);

        let text_field_style = Style::new("textfield")
            .background_color(Color::BLACK)
            .foreground_color(Color::WHITE)
            .border(Border::new(1, Color::WHITE, super::graphics::BorderStyle::Solid));

        let window_style = Style::new("window")
            .background_color(Color::DARK_GRAY)
            .border(Border::new(2, Color::WHITE, super::graphics::BorderStyle::Solid));

        theme = theme
            .add_style(button_style)
            .add_style(label_style)
            .add_style(text_field_style)
            .add_style(window_style);

        self.themes.push(theme);
        Ok(())
    }

    /// Create a high contrast theme
    fn create_high_contrast_theme(&mut self) -> GUIResult<()> {
        let mut theme = Theme::new("high_contrast");

        let button_style = Style::new("button")
            .background_color(Color::BLACK)
            .foreground_color(Color::WHITE)
            .border(Border::new(2, Color::WHITE, super::graphics::BorderStyle::Double));

        let label_style = Style::new("label")
            .background_color(Color::TRANSPARENT)
            .foreground_color(Color::BLACK);

        let text_field_style = Style::new("textfield")
            .background_color(Color::WHITE)
            .foreground_color(Color::BLACK)
            .border(Border::new(2, Color::BLACK, super::graphics::BorderStyle::Double));

        let window_style = Style::new("window")
            .background_color(Color::WHITE)
            .border(Border::new(3, Color::BLACK, super::graphics::BorderStyle::Double));

        theme = theme
            .add_style(button_style)
            .add_style(label_style)
            .add_style(text_field_style)
            .add_style(window_style);

        self.themes.push(theme);
        Ok(())
    }

    /// Set the current theme
    pub fn set_current_theme(&mut self, theme_name: &str) -> GUIResult<()> {
        for theme in &self.themes {
            if theme.name == theme_name {
                self.current_theme = theme.clone();
                return Ok(());
            }
        }
        Err(GUIError::StyleError)
    }

    /// Get the current theme
    pub fn get_current_theme(&self) -> &Theme {
        &self.current_theme
    }

    /// Create a style for a widget type
    pub fn create_widget_style(&self, widget_type: &str) -> Style {
        self.current_theme.create_style(widget_type)
    }

    /// Add a custom theme
    pub fn add_theme(&mut self, theme: Theme) {
        self.themes.push(theme);
    }

    /// Get all available themes
    pub fn get_available_themes(&self) -> Vec<String> {
        self.themes.iter().map(|t| t.name.clone()).collect()
    }
}

/// Global style manager instance
static STYLE_MANAGER: Mutex<Option<StyleManager>> = Mutex::new(None);

/// Initialize the style system
pub fn init() -> GUIResult<()> {
    info!("Initializing style system...");
    
    let mut manager = StyleManager::new();
    
    // Load all built-in theme variants
    manager.load_theme_variant(ThemeVariant::Default)?;
    manager.load_theme_variant(ThemeVariant::Light)?;
    manager.load_theme_variant(ThemeVariant::Dark)?;
    manager.load_theme_variant(ThemeVariant::HighContrast)?;
    
    let mut manager_guard = STYLE_MANAGER.lock();
    *manager_guard = Some(manager);
    
    info!("Style system initialized with multiple themes");
    Ok(())
}

/// Shutdown the style system
pub fn shutdown() -> GUIResult<()> {
    info!("Shutting down style system...");
    
    let mut manager_guard = STYLE_MANAGER.lock();
    *manager_guard = None;
    
    info!("Style system shutdown complete");
    Ok(())
}

/// Get the global style manager
pub fn get_style_manager() -> Option<StyleManager> {
    let manager_guard = STYLE_MANAGER.lock();
    manager_guard.clone()
}

/// Create a default style
pub fn create_default_style() -> Style {
    let manager = get_style_manager();
    if let Some(manager) = manager {
        manager.create_widget_style("default")
    } else {
        Style::new("default")
    }
}

/// Create a button style
pub fn create_button_style() -> Style {
    let manager = get_style_manager();
    if let Some(manager) = manager {
        manager.create_widget_style("button")
    } else {
        Style::new("button")
            .background_color(Color::LIGHT_GRAY)
            .foreground_color(Color::BLACK)
            .padding(8)
    }
}

/// Create a label style
pub fn create_label_style() -> Style {
    let manager = get_style_manager();
    if let Some(manager) = manager {
        manager.create_widget_style("label")
    } else {
        Style::new("label")
            .background_color(Color::TRANSPARENT)
            .foreground_color(Color::BLACK)
    }
}

/// Create a text field style
pub fn create_text_field_style() -> Style {
    let manager = get_style_manager();
    if let Some(manager) = manager {
        manager.create_widget_style("textfield")
    } else {
        Style::new("textfield")
            .background_color(Color::WHITE)
            .foreground_color(Color::BLACK)
            .padding(4)
    }
}