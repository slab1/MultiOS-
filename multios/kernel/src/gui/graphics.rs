//! Graphics subsystem for GUI toolkit
//! 
//! Provides basic graphics operations including drawing primitives,
//! color management, and framebuffer operations.

use alloc::vec::Vec;
use spin::Mutex;

use super::{GUIResult, GUIError};

/// Basic point structure
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn origin() -> Self {
        Self { x: 0, y: 0 }
    }
}

/// Rectangle structure for drawing areas
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rectangle {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl Rectangle {
    pub fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self { x, y, width, height }
    }

    pub fn from_points(p1: Point, p2: Point) -> Self {
        let x = p1.x.min(p2.x);
        let y = p1.y.min(p2.y);
        let width = (p2.x - p1.x).unsigned_abs();
        let height = (p2.y - p1.y).unsigned_abs();
        Self::new(x, y, width, height)
    }

    pub fn contains(&self, point: Point) -> bool {
        point.x >= self.x && 
        point.x < (self.x + self.width as i32) &&
        point.y >= self.y && 
        point.y < (self.y + self.height as i32)
    }
}

/// Size structure
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

impl Size {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    pub fn zero() -> Self {
        Self { width: 0, height: 0 }
    }

    pub fn is_zero(&self) -> bool {
        self.width == 0 && self.height == 0
    }
}

/// Color representation (RGBA)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const TRANSPARENT: Color = Color { r: 0, g: 0, b: 0, a: 0 };
    pub const BLACK: Color = Color { r: 0, g: 0, b: 0, a: 255 };
    pub const WHITE: Color = Color { r: 255, g: 255, b: 255, a: 255 };
    pub const RED: Color = Color { r: 255, g: 0, b: 0, a: 255 };
    pub const GREEN: Color = Color { r: 0, g: 255, b: 0, a: 255 };
    pub const BLUE: Color = Color { r: 0, g: 0, b: 255, a: 255 };
    pub const YELLOW: Color = Color { r: 255, g: 255, b: 0, a: 255 };
    pub const GRAY: Color = Color { r: 128, g: 128, b: 128, a: 255 };
    pub const LIGHT_GRAY: Color = Color { r: 211, g: 211, b: 211, a: 255 };
    pub const DARK_GRAY: Color = Color { r: 64, g: 64, b: 64, a: 255 };

    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub fn blend(&self, other: Color) -> Color {
        let inv_a = 255u16 - self.a as u16;
        let inv_b = 255u16 - other.a as u16;
        
        Color {
            r: ((self.r as u16 * inv_a + other.r as u16 * other.a as u16) / 255) as u8,
            g: ((self.g as u16 * inv_a + other.g as u16 * other.a as u16) / 255) as u8,
            b: ((self.b as u16 * inv_a + other.b as u16 * other.a as u16) / 255) as u8,
            a: ((self.a as u16 * inv_a + other.a as u16 * inv_b) / 255) as u8,
        }
    }
}

/// Font representation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Font {
    pub size: u32,
    pub family: FontFamily,
}

impl Font {
    pub fn new(size: u32, family: FontFamily) -> Self {
        Self { size, family }
    }
}

/// Font families
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontFamily {
    Default,
    Serif,
    SansSerif,
    Monospace,
}

/// Border styles
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Border {
    pub width: u32,
    pub color: Color,
    pub style: BorderStyle,
}

impl Border {
    pub fn new(width: u32, color: Color, style: BorderStyle) -> Self {
        Self { width, color, style }
    }

    pub fn none() -> Self {
        Self::new(0, Color::TRANSPARENT, BorderStyle::Solid)
    }
}

/// Border style variants
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BorderStyle {
    Solid,
    Dashed,
    Dotted,
    Double,
}

/// Graphics renderer trait
pub trait Renderer {
    /// Clear the entire screen with a color
    fn clear(&mut self, color: Color);
    
    /// Draw a filled rectangle
    fn fill_rect(&mut self, rect: Rectangle, color: Color);
    
    /// Draw a rectangle outline
    fn draw_rect(&mut self, rect: Rectangle, border: Border);
    
    /// Draw a line between two points
    fn draw_line(&mut self, p1: Point, p2: Point, color: Color, width: u32);
    
    /// Draw text at a position
    fn draw_text(&mut self, text: &str, position: Point, font: Font, color: Color);
    
    /// Draw an ellipse
    fn draw_ellipse(&mut self, center: Point, radius_x: u32, radius_y: u32, color: Color, filled: bool);
    
    /// Draw a circle (simplified ellipse)
    fn draw_circle(&mut self, center: Point, radius: u32, color: Color, filled: bool) {
        self.draw_ellipse(center, radius, radius, color, filled);
    }
    
    /// Copy pixels from one area to another
    fn copy_area(&mut self, source_rect: Rectangle, destination: Point);
    
    /// Get the current clipping rectangle
    fn get_clip_rect(&self) -> Option<Rectangle>;
    
    /// Set the clipping rectangle
    fn set_clip_rect(&mut self, rect: Option<Rectangle>);
}

/// Simple framebuffer renderer for testing
pub struct FramebufferRenderer {
    width: u32,
    height: u32,
    buffer: Vec<u8>,
    clip_rect: Option<Rectangle>,
}

impl FramebufferRenderer {
    pub fn new(width: u32, height: u32) -> Self {
        let buffer_size = (width * height * 4) as usize; // RGBA
        Self {
            width,
            height,
            buffer: vec![0; buffer_size],
            clip_rect: None,
        }
    }

    /// Get the raw framebuffer data
    pub fn get_buffer(&self) -> &[u8] {
        &self.buffer
    }

    /// Convert a point to a buffer index
    fn point_to_index(&self, point: Point) -> Option<usize> {
        if point.x < 0 || point.y < 0 {
            return None;
        }
        let x = point.x as u32;
        let y = point.y as u32;
        if x >= self.width || y >= self.height {
            return None;
        }
        let index = (y * self.width + x) * 4;
        Some(index as usize)
    }

    /// Check if a point is clipped
    fn is_clipped(&self, point: Point) -> bool {
        if let Some(clip) = self.clip_rect {
            if !clip.contains(point) {
                return true;
            }
        }
        false
    }
}

impl Renderer for FramebufferRenderer {
    fn clear(&mut self, color: Color) {
        for i in (0..self.buffer.len()).step_by(4) {
            self.buffer[i] = color.b;
            self.buffer[i + 1] = color.g;
            self.buffer[i + 2] = color.r;
            self.buffer[i + 3] = color.a;
        }
    }
    
    fn fill_rect(&mut self, rect: Rectangle, color: Color) {
        let start_y = rect.y.max(0);
        let end_y = (rect.y + rect.height as i32).min(self.height as i32);
        let start_x = rect.x.max(0);
        let end_x = (rect.x + rect.width as i32).min(self.width as i32);
        
        for y in start_y..end_y {
            for x in start_x..end_x {
                let point = Point::new(x, y);
                if !self.is_clipped(point) {
                    if let Some(index) = self.point_to_index(point) {
                        let pixel_start = index;
                        self.buffer[pixel_start] = color.b;
                        self.buffer[pixel_start + 1] = color.g;
                        self.buffer[pixel_start + 2] = color.r;
                        self.buffer[pixel_start + 3] = color.a;
                    }
                }
            }
        }
    }
    
    fn draw_rect(&mut self, rect: Rectangle, border: Border) {
        if border.width == 0 {
            return;
        }
        
        // Draw four sides
        let left = rect.x;
        let right = rect.x + rect.width as i32 - 1;
        let top = rect.y;
        let bottom = rect.y + rect.height as i32 - 1;
        
        // Top and bottom sides
        for i in 0..rect.width {
            let x = rect.x + i as i32;
            let top_point = Point::new(x, top);
            let bottom_point = Point::new(x, bottom);
            
            if !self.is_clipped(top_point) {
                if let Some(index) = self.point_to_index(top_point) {
                    self.buffer[index] = border.color.b;
                    self.buffer[index + 1] = border.color.g;
                    self.buffer[index + 2] = border.color.r;
                    self.buffer[index + 3] = border.color.a;
                }
            }
            
            if !self.is_clipped(bottom_point) {
                if let Some(index) = self.point_to_index(bottom_point) {
                    self.buffer[index] = border.color.b;
                    self.buffer[index + 1] = border.color.g;
                    self.buffer[index + 2] = border.color.r;
                    self.buffer[index + 3] = border.color.a;
                }
            }
        }
        
        // Left and right sides
        for i in 0..rect.height {
            let y = rect.y + i as i32;
            let left_point = Point::new(left, y);
            let right_point = Point::new(right, y);
            
            if !self.is_clipped(left_point) {
                if let Some(index) = self.point_to_index(left_point) {
                    self.buffer[index] = border.color.b;
                    self.buffer[index + 1] = border.color.g;
                    self.buffer[index + 2] = border.color.r;
                    self.buffer[index + 3] = border.color.a;
                }
            }
            
            if !self.is_clipped(right_point) {
                if let Some(index) = self.point_to_index(right_point) {
                    self.buffer[index] = border.color.b;
                    self.buffer[index + 1] = border.color.g;
                    self.buffer[index + 2] = border.color.r;
                    self.buffer[index + 3] = border.color.a;
                }
            }
        }
    }
    
    fn draw_line(&mut self, p1: Point, p2: Point, color: Color, _width: u32) {
        let dx = (p2.x - p1.x).abs();
        let dy = (p2.y - p1.y).abs();
        let sx = if p1.x < p2.x { 1 } else { -1 };
        let sy = if p1.y < p2.y { 1 } else { -1 };
        let mut err = dx as i32 - dy as i32;
        
        let mut x = p1.x;
        let mut y = p1.y;
        
        while x != p2.x || y != p2.y {
            let point = Point::new(x, y);
            if !self.is_clipped(point) {
                if let Some(index) = self.point_to_index(point) {
                    self.buffer[index] = color.b;
                    self.buffer[index + 1] = color.g;
                    self.buffer[index + 2] = color.r;
                    self.buffer[index + 3] = color.a;
                }
            }
            
            let err2 = 2 * err;
            if err2 > -dy {
                err -= dy;
                x += sx;
            }
            if err2 < dx {
                err += dx;
                y += sy;
            }
        }
    }
    
    fn draw_text(&mut self, _text: &str, _position: Point, _font: Font, _color: Color) {
        // For now, just draw a placeholder - actual text rendering would require a font library
        // This is a simplified implementation for demonstration
    }
    
    fn draw_ellipse(&mut self, center: Point, radius_x: u32, radius_y: u32, color: Color, filled: bool) {
        // Simplified ellipse drawing using midpoint algorithm
        let mut x = 0;
        let mut y = radius_y as i32;
        let mut d1 = (radius_y as i32 * radius_y as i32) - (radius_x as i32 * radius_y as i32) + (radius_x as i32 * radius_x as i32) / 4;
        
        while (2 * radius_x as i32 * x) <= (2 * radius_y as i32 * y) {
            let points = [
                Point::new(center.x + x, center.y + y),
                Point::new(center.x - x, center.y + y),
                Point::new(center.x + x, center.y - y),
                Point::new(center.x - x, center.y - y),
            ];
            
            for point in points {
                if filled {
                    let rect = Rectangle::new(point.x - 1, point.y - 1, 2, 2);
                    self.fill_rect(rect, color);
                } else {
                    if !self.is_clipped(point) {
                        if let Some(index) = self.point_to_index(point) {
                            self.buffer[index] = color.b;
                            self.buffer[index + 1] = color.g;
                            self.buffer[index + 2] = color.r;
                            self.buffer[index + 3] = color.a;
                        }
                    }
                }
            }
            
            let mut next_d1 = d1;
            if next_d1 < 0 {
                x += 1;
                next_d1 += 2 * radius_y as i32 * x + radius_y as i32 * radius_y as i32;
            } else {
                x += 1;
                y -= 1;
                next_d1 += 2 * radius_y as i32 * x + radius_y as i32 * radius_y as i32 - 2 * radius_x as i32 * y;
            }
            d1 = next_d1;
        }
    }
    
    fn copy_area(&mut self, source_rect: Rectangle, destination: Point) {
        // Simplified copy operation
        for y in 0..source_rect.height {
            for x in 0..source_rect.width {
                let src_x = source_rect.x + x as i32;
                let src_y = source_rect.y + y as i32;
                let dest_x = destination.x + x as i32;
                let dest_y = destination.y + y as i32;
                
                let src_point = Point::new(src_x, src_y);
                let dest_point = Point::new(dest_x, dest_y);
                
                if !self.is_clipped(dest_point) {
                    if let (Some(src_index), Some(dest_index)) = 
                        (self.point_to_index(src_point), self.point_to_index(dest_point)) {
                        self.buffer[dest_index] = self.buffer[src_index];
                        self.buffer[dest_index + 1] = self.buffer[src_index + 1];
                        self.buffer[dest_index + 2] = self.buffer[src_index + 2];
                        self.buffer[dest_index + 3] = self.buffer[src_index + 3];
                    }
                }
            }
        }
    }
    
    fn get_clip_rect(&self) -> Option<Rectangle> {
        self.clip_rect
    }
    
    fn set_clip_rect(&mut self, rect: Option<Rectangle>) {
        self.clip_rect = rect;
    }
}

/// Global renderer instance
static RENDERER: Mutex<Option<FramebufferRenderer>> = Mutex::new(None);

/// Initialize the graphics subsystem
pub fn init() -> GUIResult<()> {
    info!("Initializing graphics subsystem...");
    
    // Create a default framebuffer renderer (800x600)
    let renderer = FramebufferRenderer::new(800, 600);
    
    let mut renderer_guard = RENDERER.lock();
    *renderer_guard = Some(renderer);
    
    info!("Graphics subsystem initialized with 800x600 framebuffer");
    Ok(())
}

/// Shutdown the graphics subsystem
pub fn shutdown() -> GUIResult<()> {
    info!("Shutting down graphics subsystem...");
    
    let mut renderer_guard = RENDERER.lock();
    *renderer_guard = None;
    
    info!("Graphics subsystem shutdown complete");
    Ok(())
}

/// Get the global renderer instance
pub fn get_renderer() -> Option<impl Renderer + 'static> {
    let renderer_guard = RENDERER.lock();
    renderer_guard.as_ref().map(|r| FramebufferRenderer {
        width: r.width,
        height: r.height,
        buffer: r.buffer.clone(),
        clip_rect: r.clip_rect,
    })
}

/// Clear the screen
pub fn clear(color: Color) {
    if let Some(renderer) = get_renderer() {
        let mut renderer = renderer;
        renderer.clear(color);
    }
}

/// Fill a rectangle
pub fn fill_rect(rect: Rectangle, color: Color) {
    if let Some(renderer) = get_renderer() {
        let mut renderer = renderer;
        renderer.fill_rect(rect, color);
    }
}

/// Draw a rectangle outline
pub fn draw_rect(rect: Rectangle, border: Border) {
    if let Some(renderer) = get_renderer() {
        let mut renderer = renderer;
        renderer.draw_rect(rect, border);
    }
}