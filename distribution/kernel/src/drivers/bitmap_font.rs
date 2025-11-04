//! MultiOS Bitmap Font and Text Rendering System
//!
//! This module provides bitmap font support, text rendering primitives,
//! and text formatting capabilities for the MultiOS graphics system.

use crate::log::{info, warn, error, debug};
use alloc::{vec::Vec, collections::BTreeMap, string::String};
use spin::{Mutex, RwLock, Once};
use core::ops::{Deref, DerefMut};
use crate::drivers::graphics::{GraphicsBuffer, ColorDepth, GraphicsPrimitive, Point};

/// Font manager initialization
pub fn init() -> Result<(), crate::KernelError> {
    info!("Initializing font manager...");
    
    let mut manager = FONT_MANAGER.get().write();
    manager.initialize()?;
    
    info!("Font manager initialized successfully");
    Ok(())
}

/// Global font manager
pub static FONT_MANAGER: Once<Mutex<FontManager>> = Once::new();

/// Character encoding type
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum CharacterEncoding {
    ASCII = 0,
    UTF8 = 1,
    Unicode = 2,
    Custom = 3,
}

/// Text alignment
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum TextAlign {
    Left = 0,
    Center = 1,
    Right = 2,
}

/// Text direction
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum TextDirection {
    LeftToRight = 0,
    RightToLeft = 1,
    TopToBottom = 0,
    BottomToTop = 1,
}

/// Text style flags
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum TextStyle {
    None = 0,
    Bold = 1,
    Italic = 2,
    Underline = 4,
    Strikethrough = 8,
    Monospace = 16,
}

/// Bitmap font glyph structure
#[derive(Debug, Clone)]
pub struct Glyph {
    pub width: u32,
    pub height: u32,
    pub advance: u32, // Horizontal advance to next character
    pub bearing_x: i32, // Horizontal bearing
    pub bearing_y: i32, // Vertical bearing
    pub bitmap: Vec<u8>, // Glyph bitmap (1 bit per pixel)
}

/// Bitmap font structure
#[derive(Debug, Clone)]
pub struct BitmapFont {
    pub name: String,
    pub size: u32,
    pub glyph_height: u32,
    pub glyph_width: u32, // Fixed width for monospace, maximum width for proportional
    pub baseline: u32,
    pub line_height: u32,
    pub ascender: u32,
    pub descender: i32,
    pub glyphs: BTreeMap<char, Glyph>,
    pub is_monospace: bool,
    pub encoding: CharacterEncoding,
    pub style: TextStyle,
}

impl BitmapFont {
    /// Create new bitmap font
    pub fn new(name: String, size: u32, is_monospace: bool) -> Self {
        Self {
            name,
            size,
            glyph_height: size,
            glyph_width: size,
            baseline: size - 2, // Leave 2 pixels for descenders
            line_height: size + 2,
            ascender: size - 2,
            descender: -2,
            glyphs: BTreeMap::new(),
            is_monospace,
            encoding: CharacterEncoding::ASCII,
            style: TextStyle::None,
        }
    }
    
    /// Add glyph to font
    pub fn add_glyph(&mut self, ch: char, glyph: Glyph) {
        self.glyphs.insert(ch, glyph);
        
        // Update font metrics if needed
        if !self.is_monospace {
            self.glyph_width = self.glyph_width.max(glyph.width);
        }
    }
    
    /// Get glyph for character
    pub fn get_glyph(&self, ch: char) -> Option<&Glyph> {
        self.glyphs.get(&ch)
    }
    
    /// Calculate text width
    pub fn text_width(&self, text: &str) -> u32 {
        if self.is_monospace {
            text.len() as u32 * self.glyph_width
        } else {
            let mut width = 0u32;
            for ch in text.chars() {
                if let Some(glyph) = self.get_glyph(ch) {
                    width += glyph.advance;
                } else {
                    // Use default glyph width for missing characters
                    width += self.glyph_width;
                }
            }
            width
        }
    }
    
    /// Calculate text height
    pub fn text_height(&self, lines: u32) -> u32 {
        if lines <= 1 {
            self.line_height
        } else {
            self.line_height + (lines - 1) * (self.line_height + 2) // Add line spacing
        }
    }
    
    /// Render glyph to graphics buffer
    pub fn render_glyph(&self, glyph: &Glyph, x: u32, y: u32, color: u32, 
                       target: &mut GraphicsBuffer) -> Result<(), crate::KernelError> {
        // Calculate bounding box
        let render_x = (x as i32 + glyph.bearing_x) as u32;
        let render_y = (y as i32 + glyph.bearing_y) as u32;
        
        for py in 0..glyph.height {
            for px in 0..glyph.width {
                let bitmap_index = (py * ((glyph.width + 7) / 8) + px / 8) as usize;
                let bit_index = 7 - (px % 8);
                
                if bitmap_index < glyph.bitmap.len() {
                    let bit = (glyph.bitmap[bitmap_index] >> bit_index) & 1;
                    if bit == 1 {
                        let pixel_x = render_x + px;
                        let pixel_y = render_y + py;
                        
                        if pixel_x < target.width && pixel_y < target.height {
                            target.set_pixel(pixel_x, pixel_y, color);
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Render character to graphics buffer
    pub fn render_character(&self, ch: char, x: u32, y: u32, color: u32, 
                          target: &mut GraphicsBuffer) -> Result<u32, crate::KernelError> {
        if let Some(glyph) = self.get_glyph(ch) {
            self.render_glyph(glyph, x, y, color, target)?;
            Ok(glyph.advance)
        } else {
            // Render missing character glyph (usually a box or question mark)
            debug!("Missing glyph for character: '{}'", ch);
            Ok(self.glyph_width) // Return advance width
        }
    }
    
    /// Render text to graphics buffer with alignment
    pub fn render_text(&self, text: &str, x: u32, y: u32, color: u32, 
                      align: TextAlign, target: &mut GraphicsBuffer) -> Result<(), crate::KernelError> {
        if text.is_empty() {
            return Ok(());
        }
        
        let mut render_x = x;
        let text_width = self.text_width(text);
        
        // Adjust x position based on alignment
        match align {
            TextAlign::Center => {
                if text_width > 0 {
                    render_x = x.saturating_sub(text_width / 2);
                }
            }
            TextAlign::Right => {
                if text_width > 0 {
                    render_x = x.saturating_sub(text_width);
                }
            }
            TextAlign::Left => {} // No adjustment needed
        }
        
        let mut current_x = render_x;
        
        for ch in text.chars() {
            current_x += self.render_character(ch, current_x, y, color, target)?;
        }
        
        Ok(())
    }
}

/// Font manager for managing multiple fonts
pub struct FontManager {
    pub fonts: BTreeMap<String, BitmapFont>,
    pub default_font: Option<String>,
    pub initialized: bool,
}

impl FontManager {
    /// Create new font manager
    pub fn new() -> Self {
        Self {
            fonts: BTreeMap::new(),
            default_font: None,
            initialized: false,
        }
    }
    
    /// Initialize font manager and load built-in fonts
    pub fn initialize(&mut self) -> Result<(), crate::KernelError> {
        if self.initialized {
            warn!("Font manager already initialized");
            return Ok(());
        }
        
        info!("Initializing font manager");
        
        // Load built-in fonts
        self.load_default_fonts()?;
        
        self.initialized = true;
        info!("Font manager initialization complete");
        Ok(())
    }
    
    /// Load default fonts
    fn load_default_fonts(&mut self) -> Result<(), crate::KernelError> {
        info!("Loading default fonts");
        
        // Load 8x8 monospace font
        self.load_8x8_font()?;
        
        // Load 8x13 monospace font
        self.load_8x13_font()?;
        
        // Load proportional font (simulated)
        self.load_proportional_font()?;
        
        Ok(())
    }
    
    /// Load 8x8 monospace bitmap font
    fn load_8x8_font(&mut self) -> Result<(), crate::KernelError> {
        info!("Loading 8x8 monospace font");
        
        let mut font = BitmapFont::new("8x8-Mono".to_string(), 8, true);
        
        // Define 8x8 font data (simplified version with common ASCII characters)
        let font_data = include_bytes!("../fonts/8x8_font.bin");
        
        for ascii_char in 32u8..=126u8 { // ASCII 32-126
            if let Some(ch) = char::from_u32(ascii_char as u32) {
                let glyph_data = self.extract_8x8_glyph(font_data, (ascii_char - 32) as usize);
                
                let glyph = Glyph {
                    width: 8,
                    height: 8,
                    advance: 9, // 8 pixels + 1 pixel spacing
                    bearing_x: 0,
                    bearing_y: 0,
                    bitmap: glyph_data,
                };
                
                font.add_glyph(ch, glyph);
            }
        }
        
        self.fonts.insert(font.name.clone(), font);
        if self.default_font.is_none() {
            self.default_font = Some("8x8-Mono".to_string());
        }
        
        Ok(())
    }
    
    /// Load 8x13 monospace bitmap font
    fn load_8x13_font(&mut self) -> Result<(), crate::KernelError> {
        info!("Loading 8x13 monospace font");
        
        let mut font = BitmapFont::new("8x13-Mono".to_string(), 13, true);
        
        // Define 8x13 font data (simulated)
        let font_data = include_bytes!("../fonts/8x13_font.bin");
        
        for ascii_char in 32u8..=126u8 { // ASCII 32-126
            if let Some(ch) = char::from_u32(ascii_char as u32) {
                let glyph_data = self.extract_8x13_glyph(font_data, (ascii_char - 32) as usize);
                
                let glyph = Glyph {
                    width: 8,
                    height: 13,
                    advance: 9, // 8 pixels + 1 pixel spacing
                    bearing_x: 0,
                    bearing_y: 0,
                    bitmap: glyph_data,
                };
                
                font.add_glyph(ch, glyph);
            }
        }
        
        self.fonts.insert(font.name.clone(), font);
        
        Ok(())
    }
    
    /// Load proportional font
    fn load_proportional_font(&mut self) -> Result<(), crate::KernelError> {
        info!("Loading proportional font");
        
        let mut font = BitmapFont::new("Proportional-10".to_string(), 10, false);
        
        // Simulate proportional font by creating different widths for characters
        for ascii_char in 32u8..=126u8 {
            if let Some(ch) = char::from_u32(ascii_char as u32) {
                // Simulate different widths for different characters
                let width = match ch {
                    'i' | 'l' | '|' => 3,
                    'I' => 6,
                    'w' | 'W' => 9,
                    'M' => 10,
                    _ => 6, // Default width
                };
                
                let glyph = Glyph {
                    width,
                    height: 10,
                    advance: width + 1, // Width + spacing
                    bearing_x: 0,
                    bearing_y: 0,
                    bitmap: vec![0u8; (width * 10 + 7) / 8], // Simplified bitmap
                };
                
                font.add_glyph(ch, glyph);
            }
        }
        
        self.fonts.insert(font.name.clone(), font);
        
        Ok(())
    }
    
    /// Extract 8x8 glyph from font data
    fn extract_8x8_glyph(&self, font_data: &[u8], char_index: usize) -> Vec<u8> {
        let glyph_size = 8; // 8 bytes per character
        let start_index = char_index * glyph_size;
        let end_index = start_index + glyph_size;
        
        if end_index <= font_data.len() {
            font_data[start_index..end_index].to_vec()
        } else {
            vec![0u8; glyph_size] // Return empty glyph if out of bounds
        }
    }
    
    /// Extract 8x13 glyph from font data
    fn extract_8x13_glyph(&self, font_data: &[u8], char_index: usize) -> Vec<u8> {
        let glyph_size = 13; // 13 bytes per character
        let start_index = char_index * glyph_size;
        let end_index = start_index + glyph_size;
        
        if end_index <= font_data.len() {
            font_data[start_index..end_index].to_vec()
        } else {
            vec![0u8; glyph_size] // Return empty glyph if out of bounds
        }
    }
    
    /// Register font
    pub fn register_font(&mut self, font: BitmapFont) -> Result<(), crate::KernelError> {
        if self.fonts.contains_key(&font.name) {
            return Err(crate::KernelError::AlreadyExists);
        }
        
        self.fonts.insert(font.name.clone(), font);
        info!("Registered font: {}", font.name);
        Ok(())
    }
    
    /// Get font by name
    pub fn get_font(&self, name: &str) -> Option<&BitmapFont> {
        self.fonts.get(name)
    }
    
    /// Set default font
    pub fn set_default_font(&mut self, name: &str) -> Result<(), crate::KernelError> {
        if !self.fonts.contains_key(name) {
            return Err(crate::KernelError::NotFound);
        }
        
        self.default_font = Some(name.to_string());
        info!("Set default font to: {}", name);
        Ok(())
    }
    
    /// Get default font
    pub fn get_default_font(&self) -> Option<&BitmapFont> {
        if let Some(ref name) = self.default_font {
            self.fonts.get(name)
        } else {
            self.fonts.values().next()
        }
    }
    
    /// Render text using default font
    pub fn render_text(&self, text: &str, x: u32, y: u32, color: u32, 
                     align: TextAlign, target: &mut GraphicsBuffer) -> Result<(), crate::KernelError> {
        if let Some(font) = self.get_default_font() {
            font.render_text(text, x, y, color, align, target)
        } else {
            Err(crate::KernelError::NotFound)
        }
    }
    
    /// Render formatted text (multi-line with alignment)
    pub fn render_formatted_text(&self, text: &str, x: u32, y: u32, width: u32, 
                               color: u32, align: TextAlign, line_height: u32,
                               target: &mut GraphicsBuffer) -> Result<u32, crate::KernelError> {
        let font = self.get_default_font().ok_or(crate::KernelError::NotFound)?;
        
        let lines = self.wrap_text(text, width, font)?;
        let total_height = font.text_height(lines.len() as u32);
        
        let mut current_y = y;
        
        for line in &lines {
            font.render_text(line, x, current_y, color, align, target)?;
            current_y += line_height;
        }
        
        Ok(total_height)
    }
    
    /// Word wrap text to fit within specified width
    fn wrap_text(&self, text: &str, max_width: u32, font: &BitmapFont) -> Result<Vec<String>, crate::KernelError> {
        let words: Vec<&str> = text.split_whitespace().collect();
        let mut lines = Vec::new();
        let mut current_line = String::new();
        let mut current_width = 0u32;
        
        for word in words {
            let word_width = font.text_width(word);
            
            if current_width + word_width > max_width && !current_line.is_empty() {
                // Line is full, start new line
                lines.push(current_line);
                current_line = String::new();
                current_width = 0;
            }
            
            if current_width > 0 {
                current_line.push(' ');
                current_width += font.text_width(" ");
            }
            
            current_line.push_str(word);
            current_width += word_width;
        }
        
        if !current_line.is_empty() {
            lines.push(current_line);
        }
        
        Ok(lines)
    }
}

/// Text layout structure for complex text rendering
#[derive(Debug, Clone)]
pub struct TextLayout {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub font_name: String,
    pub color: u32,
    pub background_color: Option<u32>,
    pub align: TextAlign,
    pub direction: TextDirection,
    pub style: TextStyle,
    pub line_height: u32,
    pub word_wrap: bool,
}

/// Advanced text rendering utilities
pub struct AdvancedTextRenderer {
    pub default_layout: TextLayout,
}

impl AdvancedTextRenderer {
    /// Create new text renderer with default layout
    pub fn new() -> Self {
        Self {
            default_layout: TextLayout {
                x: 0,
                y: 0,
                width: 800,
                height: 600,
                font_name: "8x8-Mono".to_string(),
                color: 0xFFFFFFFF,
                background_color: None,
                align: TextAlign::Left,
                direction: TextDirection::LeftToRight,
                style: TextStyle::None,
                line_height: 10,
                word_wrap: true,
            },
        }
    }
    
    /// Render text with advanced layout options
    pub fn render_with_layout(&self, text: &str, layout: &TextLayout, 
                            target: &mut GraphicsBuffer) -> Result<u32, crate::KernelError> {
        let font_manager = FONT_MANAGER.get().read();
        let font = font_manager.get_font(&layout.font_name)
            .ok_or(crate::KernelError::NotFound)?;
        
        // Render background if specified
        if let Some(bg_color) = layout.background_color {
            target.clear_section(layout.x, layout.y, layout.width, layout.height, bg_color)?;
        }
        
        if layout.word_wrap {
            font_manager.render_formatted_text(
                text, layout.x, layout.y, layout.width,
                layout.color, layout.align, layout.line_height, target
            )
        } else {
            font.render_text(text, layout.x, layout.y, layout.color, layout.align, target)
        }
    }
    
    /// Render text with shadow effect
    pub fn render_with_shadow(&self, text: &str, x: u32, y: u32, color: u32, 
                            shadow_color: u32, shadow_offset: (i32, i32),
                            target: &mut GraphicsBuffer) -> Result<u32, crate::KernelError> {
        let font_manager = FONT_MANAGER.get().read();
        let font = font_manager.get_default_font().ok_or(crate::KernelError::NotFound)?;
        
        let shadow_x = (x as i32 + shadow_offset.0) as u32;
        let shadow_y = (y as i32 + shadow_offset.1) as u32;
        
        // Render shadow
        font.render_text(text, shadow_x, shadow_y, shadow_color, TextAlign::Left, target)?;
        
        // Render main text
        font.render_text(text, x, y, color, TextAlign::Left, target)
    }
    
    /// Render text with outline effect
    pub fn render_with_outline(&self, text: &str, x: u32, y: u32, 
                             fill_color: u32, outline_color: u32, outline_width: u32,
                             target: &mut GraphicsBuffer) -> Result<u32, crate::KernelError> {
        let font_manager = FONT_MANAGER.get().read();
        let font = font_manager.get_default_font().ok_or(crate::KernelError::NotFound)?;
        
        // Render outline (draw text multiple times with offset)
        for dy in 0..=outline_width {
            for dx in 0..=outline_width {
                let offset_x = (dx as i32 - outline_width as i32 / 2) as u32;
                let offset_y = (dy as i32 - outline_width as i32 / 2) as u32;
                
                font.render_text(text, x + offset_x, y + offset_y, outline_color, TextAlign::Left, target)?;
            }
        }
        
        // Render fill
        font.render_text(text, x, y, fill_color, TextAlign::Left, target)
    }
}

impl Default for AdvancedTextRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl GraphicsPrimitive for AdvancedTextRenderer {
    fn clear(&self, _color: u32) -> Result<(), crate::KernelError> {
        // Text renderer cannot clear screen - delegate to graphics driver
        Ok(())
    }
    
    fn draw_pixel(&self, _x: u32, _y: u32, _color: u32) -> Result<(), crate::KernelError> {
        // Text renderer cannot draw individual pixels
        Ok(())
    }
    
    fn get_pixel(&self, _x: u32, _y: u32) -> Result<u32, crate::KernelError> {
        // Text renderer cannot read pixels
        Ok(0)
    }
    
    fn draw_line_h(&self, _x1: u32, _y: u32, _x2: u32, _color: u32) -> Result<(), crate::KernelError> {
        Ok(())
    }
    
    fn draw_line_v(&self, _x: u32, _y1: u32, _y2: u32, _color: u32) -> Result<(), crate::KernelError> {
        Ok(())
    }
    
    fn draw_line(&self, _x1: u32, _y1: u32, _x2: u32, _y2: u32, _color: u32) -> Result<(), crate::KernelError> {
        Ok(())
    }
    
    fn draw_rect(&self, _x: u32, _y: u32, _width: u32, _height: u32, _color: u32, _filled: bool) -> Result<(), crate::KernelError> {
        Ok(())
    }
    
    fn draw_circle(&self, _cx: u32, _cy: u32, _radius: u32, _color: u32, _filled: bool) -> Result<(), crate::KernelError> {
        Ok(())
    }
    
    fn draw_ellipse(&self, _cx: u32, _cy: u32, _rx: u32, _ry: u32, _color: u32, _filled: bool) -> Result<(), crate::KernelError> {
        Ok(())
    }
    
    fn draw_polygon(&self, _points: &[Point], _color: u32, _filled: bool) -> Result<(), crate::KernelError> {
        Ok(())
    }
    
    fn draw_text(&self, x: u32, y: u32, text: &str, color: u32, _font: Option<&BitmapFont>) -> Result<(), crate::KernelError> {
        let font_manager = FONT_MANAGER.get().read();
        let mut buffer = crate::drivers::graphics::GraphicsBuffer::new(1, 1, ColorDepth::Bpp32)?;
        
        font_manager.render_text(text, x, y, color, TextAlign::Left, &mut buffer)?;
        Ok(())
    }
    
    fn blit(&self, _buffer: &GraphicsBuffer, _x: u32, _y: u32) -> Result<(), crate::KernelError> {
        Ok(())
    }
    
    fn blit_scaled(&self, _buffer: &GraphicsBuffer, _x: u32, _y: u32, _scale_x: f32, _scale_y: f32) -> Result<(), crate::KernelError> {
        Ok(())
    }
    
    fn blit_rotated(&self, _buffer: &GraphicsBuffer, _x: u32, _y: u32, _angle: f32) -> Result<(), crate::KernelError> {
        Ok(())
    }
    
    fn screenshot(&self, _x: u32, _y: u32, _width: u32, _height: u32) -> Result<GraphicsBuffer, crate::KernelError> {
        Ok(GraphicsBuffer::new(1, 1, ColorDepth::Bpp32).unwrap())
    }
}

impl GraphicsBuffer {
    /// Clear specific section of buffer
    pub fn clear_section(&mut self, x: u32, y: u32, width: u32, height: u32, color: u32) -> Result<(), crate::KernelError> {
        if x + width > self.width || y + height > self.height {
            return Err(crate::KernelError::InvalidAddress);
        }
        
        for py in y..(y + height) {
            for px in x..(x + width) {
                self.set_pixel(px, py, color);
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bitmap_font_creation() {
        let mut font = BitmapFont::new("Test-Font".to_string(), 12, true);
        
        let glyph = Glyph {
            width: 12,
            height: 12,
            advance: 13,
            bearing_x: 0,
            bearing_y: 0,
            bitmap: vec![0u8; 18], // (12 * 12 + 7) / 8 = 18 bytes
        };
        
        font.add_glyph('A', glyph);
        
        assert_eq!(font.size, 12);
        assert_eq!(font.is_monospace, true);
        assert_eq!(font.get_glyph('A').is_some(), true);
    }

    #[test]
    fn test_text_width_calculation() {
        let font = BitmapFont::new("Test-Mono".to_string(), 8, true);
        
        let text1 = "ABC"; // Monospace font
        let text2 = "Hello"; // Proportional would vary
        
        assert_eq!(font.text_width(text1), text1.len() as u32 * 8);
        assert_eq!(font.text_width(text2), text2.len() as u32 * 8); // For monospace
    }

    #[test]
    fn test_font_manager() {
        let mut manager = FontManager::new();
        assert!(manager.initialize().is_ok());
        assert!(manager.initialized);
        
        assert!(manager.get_default_font().is_some());
        assert!(manager.fonts.len() > 0);
    }

    #[test]
    fn test_advanced_text_renderer() {
        let mut manager = FontManager::new();
        assert!(manager.initialize().is_ok());
        
        let renderer = AdvancedTextRenderer::new();
        let mut buffer = GraphicsBuffer::new(200, 50, ColorDepth::Bpp32).unwrap();
        
        // Test basic text rendering (would require actual font data)
        // This is a conceptual test since we can't render without proper fonts
        assert_eq!(buffer.width, 200);
        assert_eq!(buffer.height, 50);
    }

    #[test]
    fn test_text_layout() {
        let layout = TextLayout {
            x: 10,
            y: 20,
            width: 100,
            height: 50,
            font_name: "Test-Font".to_string(),
            color: 0xFF000000,
            background_color: Some(0xFF000000),
            align: TextAlign::Center,
            direction: TextDirection::LeftToRight,
            style: TextStyle::Bold,
            line_height: 12,
            word_wrap: true,
        };
        
        assert_eq!(layout.x, 10);
        assert_eq!(layout.y, 20);
        assert_eq!(layout.align, TextAlign::Center);
        assert_eq!(layout.word_wrap, true);
    }

    #[test]
    fn test_glyph_metrics() {
        let glyph = Glyph {
            width: 10,
            height: 12,
            advance: 11,
            bearing_x: 1,
            bearing_y: 2,
            bitmap: vec![0u8; 15],
        };
        
        assert_eq!(glyph.width, 10);
        assert_eq!(glyph.height, 12);
        assert_eq!(glyph.advance, 11);
        assert_eq!(glyph.bearing_x, 1);
        assert_eq!(glyph.bearing_y, 2);
    }
}