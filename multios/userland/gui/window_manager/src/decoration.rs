//! Window decorations rendering and management

use crate::*;

/// Window decoration styles
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DecorationStyle {
    pub theme: DecorationTheme,
    pub button_size: u32,
    pub border_width: u32,
    pub title_bar_height: u32,
    pub corner_radius: u32,
}

impl Default for DecorationStyle {
    fn default() -> Self {
        Self {
            theme: DecorationTheme::Default,
            button_size: 30,
            border_width: 2,
            title_bar_height: 30,
            corner_radius: 0,
        }
    }
}

impl DecorationStyle {
    pub fn new(theme: DecorationTheme) -> Self {
        Self {
            theme,
            button_size: 30,
            border_width: 2,
            title_bar_height: 30,
            corner_radius: match theme {
                DecorationTheme::Rounded => 8,
                _ => 0,
            },
        }
    }

    pub fn with_button_size(mut self, size: u32) -> Self {
        self.button_size = size;
        self
    }

    pub fn with_border_width(mut self, width: u32) -> Self {
        self.border_width = width;
        self
    }

    pub fn with_title_bar_height(mut self, height: u32) -> Self {
        self.title_bar_height = height;
        self
    }

    pub fn with_corner_radius(mut self, radius: u32) -> Self {
        self.corner_radius = radius;
        self
    }
}

/// Decoration themes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecorationTheme {
    Default,
    Dark,
    Rounded,
    MacOS,
    Windows,
    Linux,
}

/// Colors for decorations
#[derive(Debug, Clone, Copy)]
pub struct DecorationColors {
    pub title_bar: Color,
    pub title_bar_inactive: Color,
    pub border: Color,
    pub border_inactive: Color,
    pub button_background: Color,
    pub button_background_hover: Color,
    pub button_background_active: Color,
    pub button_icon: Color,
}

impl Default for DecorationColors {
    fn default() -> Self {
        Self {
            title_bar: Color::RGB(240, 240, 240),
            title_bar_inactive: Color::RGB(220, 220, 220),
            border: Color::RGB(120, 120, 120),
            border_inactive: Color::RGB(180, 180, 180),
            button_background: Color::RGB(200, 200, 200),
            button_background_hover: Color::RGB(220, 220, 220),
            button_background_active: Color::RGB(180, 180, 180),
            button_icon: Color::RGB(60, 60, 60),
        }
    }
}

impl DecorationColors {
    pub fn for_theme(theme: DecorationTheme) -> Self {
        match theme {
            DecorationTheme::Dark => Self {
                title_bar: Color::RGB(60, 60, 60),
                title_bar_inactive: Color::RGB(45, 45, 45),
                border: Color::RGB(120, 120, 120),
                border_inactive: Color::RGB(80, 80, 80),
                button_background: Color::RGB(80, 80, 80),
                button_background_hover: Color::RGB(100, 100, 100),
                button_background_active: Color::RGB(60, 60, 60),
                button_icon: Color::RGB(220, 220, 220),
            },
            DecorationTheme::MacOS => Self {
                title_bar: Color::RGB(230, 230, 230),
                title_bar_inactive: Color::RGB(210, 210, 210),
                border: Color::RGB(180, 180, 180),
                border_inactive: Color::RGB(200, 200, 200),
                button_background: Color::RGB(220, 220, 220),
                button_background_hover: Color::RGB(235, 235, 235),
                button_background_active: Color::RGB(200, 200, 200),
                button_icon: Color::RGB(80, 80, 80),
            },
            _ => Self::default(),
        }
    }
}

/// Color representation
#[derive(Debug, Clone, Copy)]
pub enum Color {
    RGB(u8, u8, u8),
    RGBA(u8, u8, u8, u8),
    HSL(u16, u8, u8),
}

impl Color {
    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self::RGB(r, g, b)
    }

    pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self::RGBA(r, g, b, a)
    }
}

/// Button states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonState {
    Normal,
    Hover,
    Active,
}

/// Window decoration renderer
pub struct DecorationRenderer {
    style: DecorationStyle,
    colors: DecorationColors,
}

impl DecorationRenderer {
    /// Create a new decoration renderer
    pub fn new(style: DecorationStyle) -> Self {
        Self {
            colors: DecorationColors::for_theme(style.theme),
            style,
        }
    }

    /// Create with default style and theme
    pub fn default() -> Self {
        Self::new(DecorationStyle::default())
    }

    /// Render window decorations
    pub fn render_decorations(
        &self,
        window: &Window,
        context: &mut RenderContext,
    ) -> Result<(), DecorationError> {
        // Render border
        self.render_border(window, context)?;

        // Render title bar
        if window.style.has_title_bar {
            self.render_title_bar(window, context)?;
        }

        // Render buttons
        if window.style.has_title_bar {
            self.render_buttons(window, context)?;
        }

        Ok(())
    }

    /// Render window border
    fn render_border(
        &self,
        window: &Window,
        context: &mut RenderContext,
    ) -> Result<(), DecorationError> {
        let bounds = window.bounds();
        let border_color = if window.has_focus() {
            self.colors.border
        } else {
            self.colors.border_inactive
        };

        // Top border
        context.draw_rectangle(
            bounds.position.x,
            bounds.position.y,
            bounds.size.width,
            self.style.border_width,
            border_color,
        )?;

        // Bottom border
        context.draw_rectangle(
            bounds.position.x,
            bounds.position.y + (bounds.size.height as i32) - (self.style.border_width as i32),
            bounds.size.width,
            self.style.border_width,
            border_color,
        )?;

        // Left border
        context.draw_rectangle(
            bounds.position.x,
            bounds.position.y,
            self.style.border_width,
            bounds.size.height,
            border_color,
        )?;

        // Right border
        context.draw_rectangle(
            bounds.position.x + (bounds.size.width as i32) - (self.style.border_width as i32),
            bounds.position.y,
            self.style.border_width,
            bounds.size.height,
            border_color,
        )?;

        Ok(())
    }

    /// Render title bar
    fn render_title_bar(
        &self,
        window: &Window,
        context: &mut RenderContext,
    ) -> Result<(), DecorationError> {
        let title_bar_bounds = window.decorations().title_bar();
        let title_color = if window.has_focus() {
            self.colors.title_bar
        } else {
            self.colors.title_bar_inactive
        };

        context.draw_rectangle(
            title_bar_bounds.position.x,
            title_bar_bounds.position.y,
            title_bar_bounds.size.width,
            title_bar_bounds.size.height,
            title_color,
        )?;

        // Draw title text
        context.draw_text(
            &window.title(),
            title_bar_bounds.position.x + (title_bar_bounds.size.width as i32) / 4,
            title_bar_bounds.position.y + (title_bar_bounds.size.height as i32) / 3,
            title_color,
        )?;

        Ok(())
    }

    /// Render window buttons
    fn render_buttons(
        &self,
        window: &Window,
        context: &mut RenderContext,
    ) -> Result<(), DecorationError> {
        // Close button
        if let Some(close_bounds) = window.close_button_bounds() {
            self.render_button(
                close_bounds,
                ButtonSymbol::Close,
                ButtonState::Normal, // Would be determined by hover state
                context,
            )?;
        }

        // Minimize button
        if let Some(min_bounds) = window.minimize_button_bounds() {
            self.render_button(
                min_bounds,
                ButtonSymbol::Minimize,
                ButtonState::Normal,
                context,
            )?;
        }

        // Maximize button
        if let Some(max_bounds) = window.maximize_button_bounds() {
            let symbol = if window.state() == WindowState::Maximized {
                ButtonSymbol::Restore
            } else {
                ButtonSymbol::Maximize
            };
            
            self.render_button(
                max_bounds,
                symbol,
                ButtonState::Normal,
                context,
            )?;
        }

        Ok(())
    }

    /// Render individual button
    fn render_button(
        &self,
        bounds: Rectangle,
        symbol: ButtonSymbol,
        state: ButtonState,
        context: &mut RenderContext,
    ) -> Result<(), DecorationError> {
        // Choose colors based on state
        let (bg_color, icon_color) = match state {
            ButtonState::Normal => (self.colors.button_background, self.colors.button_icon),
            ButtonState::Hover => (self.colors.button_background_hover, self.colors.button_icon),
            ButtonState::Active => (self.colors.button_background_active, self.colors.button_icon),
        };

        // Draw button background
        context.draw_rectangle(
            bounds.position.x,
            bounds.position.y,
            bounds.size.width,
            bounds.size.height,
            bg_color,
        )?;

        // Draw button icon
        self.render_button_symbol(bounds, symbol, icon_color, context)?;

        Ok(())
    }

    /// Render button symbol (icon)
    fn render_button_symbol(
        &self,
        bounds: Rectangle,
        symbol: ButtonSymbol,
        color: Color,
        context: &mut RenderContext,
    ) -> Result<(), DecorationError> {
        match symbol {
            ButtonSymbol::Close => {
                // Draw X
                let padding = 8;
                context.draw_line(
                    bounds.position.x + padding,
                    bounds.position.y + padding,
                    bounds.position.x + (bounds.size.width as i32) - padding,
                    bounds.position.y + (bounds.size.height as i32) - padding,
                    color,
                )?;
                context.draw_line(
                    bounds.position.x + padding,
                    bounds.position.y + (bounds.size.height as i32) - padding,
                    bounds.position.x + (bounds.size.width as i32) - padding,
                    bounds.position.y + padding,
                    color,
                )?;
            }
            ButtonSymbol::Minimize => {
                // Draw minus sign
                let y = bounds.position.y + (bounds.size.height as i32) / 2;
                context.draw_line(
                    bounds.position.x + 8,
                    y,
                    bounds.position.x + (bounds.size.width as i32) - 8,
                    y,
                    color,
                )?;
            }
            ButtonSymbol::Maximize => {
                // Draw square
                let padding = 8;
                context.draw_rectangle(
                    bounds.position.x + padding,
                    bounds.position.y + padding,
                    bounds.size.width - padding * 2,
                    bounds.size.height - padding * 2,
                    color,
                )?;
            }
            ButtonSymbol::Restore => {
                // Draw overlapping squares (restore icon)
                let padding = 6;
                context.draw_rectangle(
                    bounds.position.x + padding,
                    bounds.position.y + padding,
                    bounds.size.width - padding * 2,
                    bounds.size.height - padding * 2,
                    color,
                )?;
                let inner_padding = 10;
                context.draw_rectangle(
                    bounds.position.x + inner_padding,
                    bounds.position.y + inner_padding,
                    bounds.size.width - inner_padding * 2,
                    bounds.size.height - inner_padding * 2,
                    color,
                )?;
            }
        }

        Ok(())
    }

    /// Update colors based on theme
    pub fn set_theme(&mut self, theme: DecorationTheme) {
        self.style.theme = theme;
        self.colors = DecorationColors::for_theme(theme);
    }

    /// Get current style
    pub fn style(&self) -> DecorationStyle {
        self.style
    }

    /// Get current colors
    pub fn colors(&self) -> DecorationColors {
        self.colors
    }
}

/// Button symbols for window controls
#[derive(Debug, Clone, Copy)]
pub enum ButtonSymbol {
    Close,
    Minimize,
    Maximize,
    Restore,
}

/// Rendering context interface
pub struct RenderContext {
    // This would be implemented with actual graphics context
    // For now, we'll provide a mock interface
}

impl RenderContext {
    pub fn new() -> Self {
        Self
    }

    pub fn draw_rectangle(
        &self,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        color: Color,
    ) -> Result<(), DecorationError> {
        // Mock implementation - would use actual graphics API
        println!("Drawing rectangle: ({}, {}) {}x{} with color {:?}", x, y, width, height, color);
        Ok(())
    }

    pub fn draw_line(
        &self,
        x1: i32,
        y1: i32,
        x2: i32,
        y2: i32,
        color: Color,
    ) -> Result<(), DecorationError> {
        // Mock implementation - would use actual graphics API
        println!("Drawing line: ({},{}) to ({},{}) with color {:?}", x1, y1, x2, y2, color);
        Ok(())
    }

    pub fn draw_text(
        &self,
        text: &str,
        x: i32,
        y: i32,
        color: Color,
    ) -> Result<(), DecorationError> {
        // Mock implementation - would use actual graphics API
        println!("Drawing text: '{}' at ({},{}) with color {:?}", text, x, y, color);
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum DecorationError {
    RenderFailed,
    InvalidBounds,
}

impl std::fmt::Display for DecorationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DecorationError::RenderFailed => write!(f, "Failed to render decorations"),
            DecorationError::InvalidBounds => write!(f, "Invalid bounds for decoration rendering"),
        }
    }
}

impl std::error::Error for DecorationError {}
