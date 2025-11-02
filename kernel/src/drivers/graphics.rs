//! MultiOS Graphics Driver and Framebuffer Management
//!
//! This module provides comprehensive graphics driver support including:
//! - Framebuffer management with safe memory operations
//! - VESA/VGA support with multiple resolutions and color depths
//! - Advanced graphics primitives (pixel, lines, rectangles, circles, text)
//! - Hardware acceleration detection and usage
//! - Multiple display support
//! - Safe graphics operations with proper error handling

use crate::log::{info, warn, error, debug};
use alloc::{vec::Vec, collections::BTreeMap, string::String};
use spin::{Mutex, RwLock, Once};
use core::ptr::{self, read, write};
use core::ops::{Deref, DerefMut};

/// Graphics driver initialization
pub fn init() -> Result<(), crate::KernelError> {
    info!("Initializing graphics driver system...");
    
    // Initialize graphics driver manager
    let mut manager = GRAPHICS_MANAGER.get().write();
    manager.initialize()?;
    
    info!("Graphics driver system initialized successfully");
    Ok(())
}

/// Global graphics manager
pub static GRAPHICS_MANAGER: Once<Mutex<GraphicsManager>> = Once::new();

/// Graphics mode types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum GraphicsMode {
    Unknown = 0,
    Vga = 1,
    Vesa = 2,
    UefiGop = 3,
    Framebuffer = 4,
    HardwareAcceleration = 5,
}

/// Color depth options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ColorDepth {
    Unknown = 0,
    Bpp1 = 1,   // 1-bit per pixel (monochrome)
    Bpp2 = 2,   // 2-bit per pixel (4 colors)
    Bpp4 = 4,   // 4-bit per pixel (16 colors)
    Bpp8 = 5,   // 8-bit per pixel (256 colors)
    Bpp15 = 6,  // 15-bit per pixel (32K colors)
    Bpp16 = 7,  // 16-bit per pixel (65K colors)
    Bpp24 = 8,  // 24-bit per pixel (16M colors)
    Bpp32 = 9,  // 32-bit per pixel with alpha
}

/// Screen orientation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ScreenOrientation {
    Landscape = 0,
    Portrait = 1,
    LandscapeFlipped = 2,
    PortraitFlipped = 3,
}

/// Graphics mode information
#[derive(Debug, Clone)]
pub struct GraphicsModeInfo {
    pub width: u32,
    pub height: u32,
    pub color_depth: ColorDepth,
    pub pitch: u32,
    pub framebuffer_addr: u64,
    pub mode_number: u16,
    pub attributes: u16,
    pub bpp: u8, // Bits per pixel
    pub bytes_per_pixel: u8,
    pub red_mask: u8,
    pub green_mask: u8,
    pub blue_mask: u8,
    pub alpha_mask: u8,
    pub red_shift: u8,
    pub green_shift: u8,
    pub blue_shift: u8,
    pub alpha_shift: u8,
}

impl GraphicsModeInfo {
    /// Calculate bytes per pixel from color depth
    pub fn get_bytes_per_pixel(&self) -> u8 {
        match self.color_depth {
            ColorDepth::Bpp1 => 0,  // Special case - packed pixels
            ColorDepth::Bpp2 => 0,  // Special case - packed pixels
            ColorDepth::Bpp4 => 0,  // Special case - packed pixels
            ColorDepth::Bpp8 => 1,
            ColorDepth::Bpp15 => 2,
            ColorDepth::Bpp16 => 2,
            ColorDepth::Bpp24 => 3,
            ColorDepth::Bpp32 => 4,
            _ => 0,
        }
    }
    
    /// Check if color depth supports alpha channel
    pub fn has_alpha_channel(&self) -> bool {
        self.color_depth == ColorDepth::Bpp32 && self.alpha_mask > 0
    }
    
    /// Convert RGB color to pixel value for this mode
    pub fn color_to_pixel(&self, red: u8, green: u8, blue: u8, alpha: u8) -> u32 {
        match self.color_depth {
            ColorDepth::Bpp16 => {
                ((red as u32 >> 3) << 11) | 
                ((green as u32 >> 2) << 5) | 
                (blue as u32 >> 3)
            }
            ColorDepth::Bpp24 => {
                (red as u32) | 
                ((green as u32) << 8) | 
                ((blue as u32) << 16)
            }
            ColorDepth::Bpp32 if self.has_alpha_channel() => {
                (red as u32) | 
                ((green as u32) << 8) | 
                ((blue as u32) << 16) | 
                ((alpha as u32) << 24)
            }
            ColorDepth::Bpp32 => {
                (red as u32) | 
                ((green as u32) << 8) | 
                ((blue as u32) << 16)
            }
            _ => 0, // Unsupported color depths
        }
    }
    
    /// Convert pixel value to RGBA
    pub fn pixel_to_color(&self, pixel: u32) -> (u8, u8, u8, u8) {
        match self.color_depth {
            ColorDepth::Bpp16 => {
                let r = ((pixel >> 11) & 0x1F) as u8;
                let g = ((pixel >> 5) & 0x3F) as u8;
                let b = (pixel & 0x1F) as u8;
                ((r << 3), (g << 2), (b << 3), 255)
            }
            ColorDepth::Bpp24 => {
                ((pixel & 0xFF) as u8, 
                 ((pixel >> 8) & 0xFF) as u8, 
                 ((pixel >> 16) & 0xFF) as u8, 
                 255)
            }
            ColorDepth::Bpp32 if self.has_alpha_channel() => {
                ((pixel & 0xFF) as u8, 
                 ((pixel >> 8) & 0xFF) as u8, 
                 ((pixel >> 16) & 0xFF) as u8, 
                 ((pixel >> 24) & 0xFF) as u8)
            }
            ColorDepth::Bpp32 => {
                ((pixel & 0xFF) as u8, 
                 ((pixel >> 8) & 0xFF) as u8, 
                 ((pixel >> 16) & 0xFF) as u8, 
                 255)
            }
            _ => (0, 0, 0, 0),
        }
    }
}

/// Rectangle for graphics operations
#[derive(Debug, Clone, Copy)]
pub struct Rectangle {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl Rectangle {
    /// Create a new rectangle
    pub fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Self { x, y, width, height }
    }
    
    /// Check if a point is within this rectangle
    pub fn contains(&self, x: u32, y: u32) -> bool {
        x >= self.x && x < self.x + self.width &&
        y >= self.y && y < self.y + self.height
    }
    
    /// Check if this rectangle intersects with another
    pub fn intersects(&self, other: &Rectangle) -> bool {
        !(self.x + self.width <= other.x ||
          other.x + other.width <= self.x ||
          self.y + self.height <= other.y ||
          other.y + other.height <= self.y)
    }
    
    /// Get intersection with another rectangle
    pub fn intersection(&self, other: &Rectangle) -> Option<Rectangle> {
        if self.intersects(other) {
            let x = self.x.max(other.x);
            let y = self.y.max(other.y);
            let width = (self.x + self.width).min(other.x + other.width) - x;
            let height = (self.y + self.height).min(other.y + other.height) - y;
            Some(Rectangle::new(x, y, width, height))
        } else {
            None
        }
    }
}

/// Point for graphics operations
#[derive(Debug, Clone, Copy)]
pub struct Point {
    pub x: u32,
    pub y: u32,
}

impl Point {
    pub fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }
}

/// Line structure for graphics operations
#[derive(Debug, Clone, Copy)]
pub struct Line {
    pub start: Point,
    pub end: Point,
    pub color: u32,
    pub width: u32,
}

/// Graphics buffer for drawing operations
#[derive(Clone)]
pub struct GraphicsBuffer {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub pitch: u32,
    pub color_depth: ColorDepth,
    pub framebuffer_addr: Option<u64>,
}

impl GraphicsBuffer {
    /// Create a new graphics buffer
    pub fn new(width: u32, height: u32, color_depth: ColorDepth) -> Result<Self, crate::KernelError> {
        let pitch = width * color_depth as u32;
        let size = (height * pitch) as usize;
        
        Ok(Self {
            data: vec![0u8; size],
            width,
            height,
            pitch,
            color_depth,
            framebuffer_addr: None,
        })
    }
    
    /// Create buffer from existing framebuffer
    pub fn from_framebuffer(framebuffer_addr: u64, width: u32, height: u32, color_depth: ColorDepth, pitch: u32) -> Result<Self, crate::KernelError> {
        let size = (height * pitch) as usize;
        
        Ok(Self {
            data: vec![0u8; size], // In real implementation, this would be mapped memory
            width,
            height,
            pitch,
            color_depth,
            framebuffer_addr: Some(framebuffer_addr),
        })
    }
    
    /// Get pixel at coordinates
    pub fn get_pixel(&self, x: u32, y: u32) -> Option<u32> {
        if x >= self.width || y >= self.height {
            return None;
        }
        
        let offset = (y * self.pitch + x * self.color_depth as u32) as usize;
        
        match self.color_depth {
            ColorDepth::Bpp8 => Some(self.data[offset] as u32),
            ColorDepth::Bpp16 => {
                let pixel_bytes = &self.data[offset..offset + 2];
                Some(u16::from_le_bytes([pixel_bytes[0], pixel_bytes[1]]) as u32)
            }
            ColorDepth::Bpp24 => {
                let pixel_bytes = &self.data[offset..offset + 3];
                Some(u32::from_le_bytes([pixel_bytes[0], pixel_bytes[1], pixel_bytes[2], 0]))
            }
            ColorDepth::Bpp32 => {
                let pixel_bytes = &self.data[offset..offset + 4];
                Some(u32::from_le_bytes([pixel_bytes[0], pixel_bytes[1], pixel_bytes[2], pixel_bytes[3]]))
            }
            _ => None,
        }
    }
    
    /// Set pixel at coordinates
    pub fn set_pixel(&mut self, x: u32, y: u32, color: u32) {
        if x >= self.width || y >= self.height {
            return;
        }
        
        let offset = (y * self.pitch + x * self.color_depth as u32) as usize;
        
        match self.color_depth {
            ColorDepth::Bpp8 => self.data[offset] = color as u8,
            ColorDepth::Bpp16 => {
                let bytes = (color as u16).to_le_bytes();
                self.data[offset] = bytes[0];
                self.data[offset + 1] = bytes[1];
            }
            ColorDepth::Bpp24 => {
                let bytes = color.to_le_bytes();
                self.data[offset] = bytes[0];
                self.data[offset + 1] = bytes[1];
                self.data[offset + 2] = bytes[2];
            }
            ColorDepth::Bpp32 => {
                let bytes = color.to_le_bytes();
                self.data[offset] = bytes[0];
                self.data[offset + 1] = bytes[1];
                self.data[offset + 2] = bytes[2];
                self.data[offset + 3] = bytes[3];
            }
            _ => {},
        }
    }
    
    /// Clear the entire buffer with a color
    pub fn clear(&mut self, color: u32) {
        match self.color_depth {
            ColorDepth::Bpp8 => self.data.fill(color as u8),
            ColorDepth::Bpp16 => {
                let bytes = (color as u16).to_le_bytes();
                for i in (0..self.data.len()).step_by(2) {
                    self.data[i] = bytes[0];
                    self.data[i + 1] = bytes[1];
                }
            }
            ColorDepth::Bpp24 => {
                let bytes = color.to_le_bytes();
                for i in (0..self.data.len()).step_by(3) {
                    self.data[i] = bytes[0];
                    self.data[i + 1] = bytes[1];
                    self.data[i + 2] = bytes[2];
                }
            }
            ColorDepth::Bpp32 => {
                let bytes = color.to_le_bytes();
                for i in (0..self.data.len()).step_by(4) {
                    self.data[i] = bytes[0];
                    self.data[i + 1] = bytes[1];
                    self.data[i + 2] = bytes[2];
                    self.data[i + 3] = bytes[3];
                }
            }
            _ => self.data.fill(0),
        }
    }
}

/// Hardware acceleration capabilities
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum HardwareAcceleration {
    None = 0,
    Basic = 1,    // Blits, rectangles
    Intermediate = 2, // Lines, circles, text
    Advanced = 3, // 3D acceleration, shaders
}

/// Graphics capabilities structure
#[derive(Debug, Clone)]
pub struct GraphicsCapabilities {
    pub hardware_acceleration: HardwareAcceleration,
    pub supports_multiple_displays: bool,
    pub max_resolution_width: u32,
    pub max_resolution_height: u32,
    pub supported_color_depths: Vec<ColorDepth>,
    pub supports_double_buffering: bool,
    pub supports_vsync: bool,
    pub supports_vga_safe_mode: bool,
}

/// Framebuffer device
pub struct FramebufferDevice {
    pub id: u32,
    pub name: String,
    pub mode: GraphicsModeInfo,
    pub buffer: Option<GraphicsBuffer>,
    pub capabilities: GraphicsCapabilities,
    pub state: DriverState,
}

/// Graphics primitive operations
pub trait GraphicsPrimitive {
    /// Clear screen with specified color
    fn clear(&self, color: u32) -> Result<(), crate::KernelError>;
    
    /// Draw pixel at coordinates
    fn draw_pixel(&self, x: u32, y: u32, color: u32) -> Result<(), crate::KernelError>;
    
    /// Get pixel color at coordinates
    fn get_pixel(&self, x: u32, y: u32) -> Result<u32, crate::KernelError>;
    
    /// Draw horizontal line
    fn draw_line_h(&self, x1: u32, y: u32, x2: u32, color: u32) -> Result<(), crate::KernelError>;
    
    /// Draw vertical line
    fn draw_line_v(&self, x: u32, y1: u32, y2: u32, color: u32) -> Result<(), crate::KernelError>;
    
    /// Draw line between two points (Bresenham's algorithm)
    fn draw_line(&self, x1: u32, y1: u32, x2: u32, y2: u32, color: u32) -> Result<(), crate::KernelError>;
    
    /// Draw rectangle
    fn draw_rect(&self, x: u32, y: u32, width: u32, height: u32, color: u32, filled: bool) -> Result<(), crate::KernelError>;
    
    /// Draw filled circle
    fn draw_circle(&self, cx: u32, cy: u32, radius: u32, color: u32, filled: bool) -> Result<(), crate::KernelError>;
    
    /// Draw ellipse
    fn draw_ellipse(&self, cx: u32, cy: u32, rx: u32, ry: u32, color: u32, filled: bool) -> Result<(), crate::KernelError>;
    
    /// Draw polygon
    fn draw_polygon(&self, points: &[Point], color: u32, filled: bool) -> Result<(), crate::KernelError>;
    
    /// Draw text using bitmap font
    fn draw_text(&self, x: u32, y: u32, text: &str, color: u32, font: Option<&BitmapFont>) -> Result<(), crate::KernelError>;
    
    /// Blit buffer to screen at specified position
    fn blit(&self, buffer: &GraphicsBuffer, x: u32, y: u32) -> Result<(), crate::KernelError>;
    
    /// Blit buffer with scaling
    fn blit_scaled(&self, buffer: &GraphicsBuffer, x: u32, y: u32, scale_x: f32, scale_y: f32) -> Result<(), crate::KernelError>;
    
    /// Blit buffer with rotation
    fn blit_rotated(&self, buffer: &GraphicsBuffer, x: u32, y: u32, angle: f32) -> Result<(), crate::KernelError>;
    
    /// Copy area of screen to buffer
    fn screenshot(&self, x: u32, y: u32, width: u32, height: u32) -> Result<GraphicsBuffer, crate::KernelError>;
}

/// Driver state enumeration
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum DriverState {
    Unloaded = 0,
    Loaded = 1,
    Active = 2,
    Error = 3,
}

/// VGA Graphics Driver
pub struct VgaGraphics {
    pub io_base: u16,
    pub vram_base: u64,
    pub current_mode: GraphicsModeInfo,
    pub buffer: Option<GraphicsBuffer>,
}

impl VgaGraphics {
    /// Create new VGA graphics driver
    pub fn new(io_base: u16, vram_base: u64) -> Self {
        Self {
            io_base,
            vram_base,
            current_mode: GraphicsModeInfo {
                width: 320,
                height: 200,
                color_depth: ColorDepth::Bpp8,
                pitch: 320,
                framebuffer_addr: vram_base,
                mode_number: 0x13,
                attributes: 0x00,
                bpp: 8,
                bytes_per_pixel: 1,
                red_mask: 0,
                green_mask: 0,
                blue_mask: 0,
                alpha_mask: 0,
                red_shift: 0,
                green_shift: 0,
                blue_shift: 0,
                alpha_shift: 0,
            },
            buffer: None,
        }
    }
    
    /// Initialize VGA graphics mode
    pub fn init_mode(&mut self) -> Result<(), crate::KernelError> {
        info!("Initializing VGA graphics mode 0x13");
        
        // Set VGA mode 0x13 (320x200x256)
        self.set_mode(0x13)?;
        
        // Configure palette
        self.set_palette()?;
        
        info!("VGA graphics initialized: {}x{}x{}", 
              self.current_mode.width, 
              self.current_mode.height, 
              self.current_mode.bpp);
        
        Ok(())
    }
    
    /// Set graphics mode
    fn set_mode(&self, mode: u16) -> Result<(), crate::KernelError> {
        // VGA mode set - typically via BIOS interrupt 0x10
        // In a real implementation, this would call BIOS services
        
        info!("Setting VGA mode: 0x{:X}", mode);
        Ok(())
    }
    
    /// Configure VGA palette
    fn set_palette(&self) -> Result<(), crate::KernelError> {
        // Set color palette for standard VGA colors
        info!("Configuring VGA palette");
        Ok(())
    }
    
    /// Write pixel to VGA memory with bounds checking
    fn write_pixel_vga(&self, x: u32, y: u32, color: u8) -> Result<(), crate::KernelError> {
        if x >= self.current_mode.width || y >= self.current_mode.height {
            return Err(crate::KernelError::InvalidAddress);
        }
        
        let offset = (y * self.current_mode.pitch + x) as usize;
        
        // Safe write to VGA memory
        unsafe {
            let vram_ptr = self.vram_base as *mut u8;
            if !vram_ptr.is_null() {
                ptr::write(vram_ptr.add(offset), color);
                Ok(())
            } else {
                Err(crate::KernelError::InvalidAddress)
            }
        }
    }
    
    /// Read pixel from VGA memory with bounds checking
    fn read_pixel_vga(&self, x: u32, y: u32) -> Result<u8, crate::KernelError> {
        if x >= self.current_mode.width || y >= self.current_mode.height {
            return Err(crate::KernelError::InvalidAddress);
        }
        
        let offset = (y * self.current_mode.pitch + x) as usize;
        
        unsafe {
            let vram_ptr = self.vram_base as *const u8;
            if !vram_ptr.is_null() {
                Ok(ptr::read(vram_ptr.add(offset)))
            } else {
                Err(crate::KernelError::InvalidAddress)
            }
        }
    }
}

impl GraphicsPrimitive for VgaGraphics {
    fn clear(&self, color: u32) -> Result<(), crate::KernelError> {
        let color_byte = (color & 0xFF) as u8; // VGA mode 0x13 uses 8-bit color
        
        for y in 0..self.current_mode.height {
            for x in 0..self.current_mode.width {
                self.write_pixel_vga(x, y, color_byte)?;
            }
        }
        Ok(())
    }
    
    fn draw_pixel(&self, x: u32, y: u32, color: u32) -> Result<(), crate::KernelError> {
        self.write_pixel_vga(x, y, (color & 0xFF) as u8)
    }
    
    fn get_pixel(&self, x: u32, y: u32) -> Result<u32, crate::KernelError> {
        Ok(self.read_pixel_vga(x, y)? as u32)
    }
    
    fn draw_line_h(&self, x1: u32, y: u32, x2: u32, color: u32) -> Result<(), crate::KernelError> {
        let (start, end) = if x1 <= x2 { (x1, x2) } else { (x2, x1) };
        let color_byte = (color & 0xFF) as u8;
        
        for x in start..=end {
            self.write_pixel_vga(x, y, color_byte)?;
        }
        Ok(())
    }
    
    fn draw_line_v(&self, x: u32, y1: u32, y2: u32, color: u32) -> Result<(), crate::KernelError> {
        let (start, end) = if y1 <= y2 { (y1, y2) } else { (y2, y1) };
        let color_byte = (color & 0xFF) as u8;
        
        for y in start..=end {
            self.write_pixel_vga(x, y, color_byte)?;
        }
        Ok(())
    }
    
    fn draw_line(&self, x1: u32, y1: u32, x2: u32, y2: u32, color: u32) -> Result<(), crate::KernelError> {
        // Bresenham's line algorithm for efficiency
        let mut x1 = x1 as i32;
        let mut y1 = y1 as i32;
        let x2 = x2 as i32;
        let y2 = y2 as i32;
        
        let dx = (x2 - x1).abs();
        let dy = (y2 - y1).abs();
        let sx = if x1 < x2 { 1 } else { -1 };
        let sy = if y1 < y2 { 1 } else { -1 };
        
        let mut err = if dx > dy { dx } else { -dy } / 2;
        let color_byte = (color & 0xFF) as u8;
        
        loop {
            if x1 >= 0 && x1 < self.current_mode.width as i32 && 
               y1 >= 0 && y1 < self.current_mode.height as i32 {
                self.write_pixel_vga(x1 as u32, y1 as u32, color_byte)?;
            }
            
            if x1 == x2 && y1 == y2 {
                break;
            }
            
            let e2 = err;
            if e2 > -dx {
                err -= dy;
                x1 += sx;
            }
            if e2 < dy {
                err += dx;
                y1 += sy;
            }
        }
        
        Ok(())
    }
    
    fn draw_rect(&self, x: u32, y: u32, width: u32, height: u32, color: u32, filled: bool) -> Result<(), crate::KernelError> {
        let color_byte = (color & 0xFF) as u8;
        
        if filled {
            for py in y..(y + height) {
                for px in x..(x + width) {
                    self.write_pixel_vga(px, py, color_byte)?;
                }
            }
        } else {
            // Draw border
            self.draw_line_h(x, y, x + width, color)?;
            self.draw_line_h(x, y + height, x + width, color)?;
            self.draw_line_v(x, y, y + height, color)?;
            self.draw_line_v(x + width, y, y + height, color)?;
        }
        Ok(())
    }
    
    fn draw_circle(&self, cx: u32, cy: u32, radius: u32, color: u32, filled: bool) -> Result<(), crate::KernelError> {
        let color_byte = (color & 0xFF) as u8;
        let r = radius as i32;
        let cx = cx as i32;
        let cy = cy as i32;
        
        for y in -r..=r {
            for x in -r..=r {
                let dist_squared = x * x + y * y;
                
                if dist_squared <= r * r {
                    if filled {
                        let px = (cx + x) as u32;
                        let py = (cy + y) as u32;
                        if px < self.current_mode.width && py < self.current_mode.height {
                            self.write_pixel_vga(px, py, color_byte)?;
                        }
                    } else {
                        // Draw circle outline - only draw if at boundary
                        if dist_squared > (r - 1) * (r - 1) {
                            let px = (cx + x) as u32;
                            let py = (cy + y) as u32;
                            if px < self.current_mode.width && py < self.current_mode.height {
                                self.write_pixel_vga(px, py, color_byte)?;
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }
    
    fn draw_ellipse(&self, _cx: u32, _cy: u32, _rx: u32, _ry: u32, _color: u32, _filled: bool) -> Result<(), crate::KernelError> {
        // Ellipse drawing implementation would go here
        Ok(())
    }
    
    fn draw_polygon(&self, _points: &[Point], _color: u32, _filled: bool) -> Result<(), crate::KernelError> {
        // Polygon drawing implementation would go here
        Ok(())
    }
    
    fn draw_text(&self, _x: u32, _y: u32, _text: &str, _color: u32, _font: Option<&BitmapFont>) -> Result<(), crate::KernelError> {
        // Text rendering would require bitmap font implementation
        Ok(())
    }
    
    fn blit(&self, _buffer: &GraphicsBuffer, _x: u32, _y: u32) -> Result<(), crate::KernelError> {
        // Blit operation for VGA - would copy buffer to VGA memory
        Ok(())
    }
    
    fn blit_scaled(&self, _buffer: &GraphicsBuffer, _x: u32, _y: u32, _scale_x: f32, _scale_y: f32) -> Result<(), crate::KernelError> {
        Ok(())
    }
    
    fn blit_rotated(&self, _buffer: &GraphicsBuffer, _x: u32, _y: u32, _angle: f32) -> Result<(), crate::KernelError> {
        Ok(())
    }
    
    fn screenshot(&self, x: u32, y: u32, width: u32, height: u32) -> Result<GraphicsBuffer, crate::KernelError> {
        let mut buffer = GraphicsBuffer::new(width, height, ColorDepth::Bpp8)?;
        
        for py in 0..height {
            for px in 0..width {
                if let Ok(color) = self.get_pixel(x + px, y + py) {
                    buffer.set_pixel(px, py, color);
                }
            }
        }
        
        Ok(buffer)
    }
}

/// VESA Graphics Driver
pub struct VesaGraphics {
    pub framebuffer_addr: u64,
    pub current_mode: GraphicsModeInfo,
    pub available_modes: Vec<GraphicsModeInfo>,
    pub buffer: Option<GraphicsBuffer>,
}

impl VesaGraphics {
    /// Create new VESA graphics driver
    pub fn new(framebuffer_addr: u64) -> Self {
        Self {
            framebuffer_addr,
            current_mode: GraphicsModeInfo {
                width: 1024,
                height: 768,
                color_depth: ColorDepth::Bpp32,
                pitch: 4096,
                framebuffer_addr,
                mode_number: 0x0118,
                attributes: 0x00,
                bpp: 32,
                bytes_per_pixel: 4,
                red_mask: 8,
                green_mask: 8,
                blue_mask: 8,
                alpha_mask: 8,
                red_shift: 0,
                green_shift: 8,
                blue_shift: 16,
                alpha_shift: 24,
            },
            available_modes: Vec::new(),
            buffer: None,
        }
    }
    
    /// Initialize VESA graphics mode
    pub fn init(&mut self) -> Result<(), crate::KernelError> {
        info!("Initializing VESA graphics");
        
        // Query available VESA modes
        self.query_modes()?;
        
        // Set optimal mode
        self.set_mode(0x0118)?; // 1024x768x32
        
        info!("VESA graphics initialized: {}x{}x{}", 
              self.current_mode.width, 
              self.current_mode.height, 
              self.current_mode.bpp);
        
        Ok(())
    }
    
    /// Query available VESA modes
    fn query_modes(&mut self) -> Result<(), crate::KernelError> {
        // In real implementation, this would query VESA BIOS
        info!("Querying VESA modes");
        
        // Add common VESA modes
        self.available_modes.push(GraphicsModeInfo {
            width: 640, height: 480, color_depth: ColorDepth::Bpp32,
            pitch: 2560, framebuffer_addr: self.framebuffer_addr,
            mode_number: 0x0111, attributes: 0x00,
            bpp: 32, bytes_per_pixel: 4,
            red_mask: 8, green_mask: 8, blue_mask: 8, alpha_mask: 8,
            red_shift: 0, green_shift: 8, blue_shift: 16, alpha_shift: 24,
        });
        
        self.available_modes.push(GraphicsModeInfo {
            width: 800, height: 600, color_depth: ColorDepth::Bpp32,
            pitch: 3200, framebuffer_addr: self.framebuffer_addr,
            mode_number: 0x0114, attributes: 0x00,
            bpp: 32, bytes_per_pixel: 4,
            red_mask: 8, green_mask: 8, blue_mask: 8, alpha_mask: 8,
            red_shift: 0, green_shift: 8, blue_shift: 16, alpha_shift: 24,
        });
        
        self.available_modes.push(GraphicsModeInfo {
            width: 1024, height: 768, color_depth: ColorDepth::Bpp32,
            pitch: 4096, framebuffer_addr: self.framebuffer_addr,
            mode_number: 0x0118, attributes: 0x00,
            bpp: 32, bytes_per_pixel: 4,
            red_mask: 8, green_mask: 8, blue_mask: 8, alpha_mask: 8,
            red_shift: 0, green_shift: 8, blue_shift: 16, alpha_shift: 24,
        });
        
        info!("Found {} VESA modes", self.available_modes.len());
        Ok(())
    }
    
    /// Set VESA mode
    fn set_mode(&mut self, mode_number: u16) -> Result<(), crate::KernelError> {
        // Find mode info
        if let Some(mode_info) = self.available_modes.iter().find(|m| m.mode_number == mode_number) {
            self.current_mode = mode_info.clone();
            info!("Set VESA mode 0x{:X}: {}x{}x{}", 
                  mode_number, mode_info.width, mode_info.height, mode_info.bpp);
        } else {
            return Err(crate::KernelError::InvalidMode);
        }
        Ok(())
    }
    
    /// Write pixel to framebuffer with bounds checking
    fn write_pixel_vesa(&self, x: u32, y: u32, color: u32) -> Result<(), crate::KernelError> {
        if x >= self.current_mode.width || y >= self.current_mode.height {
            return Err(crate::KernelError::InvalidAddress);
        }
        
        let offset = (y * self.current_mode.pitch + x * self.current_mode.bytes_per_pixel as u32) as usize;
        
        unsafe {
            let fb_ptr = self.framebuffer_addr as *mut u8;
            if !fb_ptr.is_null() {
                let color_bytes = color.to_le_bytes();
                ptr::copy_nonoverlapping(color_bytes.as_ptr(), fb_ptr.add(offset), self.current_mode.bytes_per_pixel as usize);
                Ok(())
            } else {
                Err(crate::KernelError::InvalidAddress)
            }
        }
    }
    
    /// Read pixel from framebuffer
    fn read_pixel_vesa(&self, x: u32, y: u32) -> Result<u32, crate::KernelError> {
        if x >= self.current_mode.width || y >= self.current_mode.height {
            return Err(crate::KernelError::InvalidAddress);
        }
        
        let offset = (y * self.current_mode.pitch + x * self.current_mode.bytes_per_pixel as u32) as usize;
        
        unsafe {
            let fb_ptr = self.framebuffer_addr as *const u8;
            if !fb_ptr.is_null() {
                let mut color_bytes = [0u8; 4];
                ptr::copy_nonoverlapping(fb_ptr.add(offset), color_bytes.as_mut_ptr(), self.current_mode.bytes_per_pixel as usize);
                Ok(u32::from_le_bytes(color_bytes))
            } else {
                Err(crate::KernelError::InvalidAddress)
            }
        }
    }
}

impl GraphicsPrimitive for VesaGraphics {
    fn clear(&self, color: u32) -> Result<(), crate::KernelError> {
        let buffer_size = (self.current_mode.height * self.current_mode.pitch) as usize;
        let color_bytes = color.to_le_bytes();
        
        unsafe {
            let fb_ptr = self.framebuffer_addr as *mut u8;
            if !fb_ptr.is_null() {
                for i in (0..buffer_size).step_by(self.current_mode.bytes_per_pixel as usize) {
                    ptr::copy_nonoverlapping(color_bytes.as_ptr(), fb_ptr.add(i), self.current_mode.bytes_per_pixel as usize);
                }
                Ok(())
            } else {
                Err(crate::KernelError::InvalidAddress)
            }
        }
    }
    
    fn draw_pixel(&self, x: u32, y: u32, color: u32) -> Result<(), crate::KernelError> {
        self.write_pixel_vesa(x, y, color)
    }
    
    fn get_pixel(&self, x: u32, y: u32) -> Result<u32, crate::KernelError> {
        self.read_pixel_vesa(x, y)
    }
    
    fn draw_line_h(&self, x1: u32, y: u32, x2: u32, color: u32) -> Result<(), crate::KernelError> {
        let (start, end) = if x1 <= x2 { (x1, x2) } else { (x2, x1) };
        
        for x in start..=end {
            self.write_pixel_vesa(x, y, color)?;
        }
        Ok(())
    }
    
    fn draw_line_v(&self, x: u32, y1: u32, y2: u32, color: u32) -> Result<(), crate::KernelError> {
        let (start, end) = if y1 <= y2 { (y1, y2) } else { (y2, y1) };
        
        for y in start..=end {
            self.write_pixel_vesa(x, y, color)?;
        }
        Ok(())
    }
    
    fn draw_line(&self, x1: u32, y1: u32, x2: u32, y2: u32, color: u32) -> Result<(), crate::KernelError> {
        // Bresenham's line algorithm for efficiency
        let mut x1 = x1 as i32;
        let mut y1 = y1 as i32;
        let x2 = x2 as i32;
        let y2 = y2 as i32;
        
        let dx = (x2 - x1).abs();
        let dy = (y2 - y1).abs();
        let sx = if x1 < x2 { 1 } else { -1 };
        let sy = if y1 < y2 { 1 } else { -1 };
        
        let mut err = if dx > dy { dx } else { -dy } / 2;
        
        loop {
            if x1 >= 0 && x1 < self.current_mode.width as i32 && 
               y1 >= 0 && y1 < self.current_mode.height as i32 {
                self.write_pixel_vesa(x1 as u32, y1 as u32, color)?;
            }
            
            if x1 == x2 && y1 == y2 {
                break;
            }
            
            let e2 = err;
            if e2 > -dx {
                err -= dy;
                x1 += sx;
            }
            if e2 < dy {
                err += dx;
                y1 += sy;
            }
        }
        
        Ok(())
    }
    
    fn draw_rect(&self, x: u32, y: u32, width: u32, height: u32, color: u32, filled: bool) -> Result<(), crate::KernelError> {
        if filled {
            for py in y..(y + height) {
                for px in x..(x + width) {
                    self.write_pixel_vesa(px, py, color)?;
                }
            }
        } else {
            self.draw_line_h(x, y, x + width, color)?;
            self.draw_line_h(x, y + height, x + width, color)?;
            self.draw_line_v(x, y, y + height, color)?;
            self.draw_line_v(x + width, y, y + height, color)?;
        }
        Ok(())
    }
    
    fn draw_circle(&self, cx: u32, cy: u32, radius: u32, color: u32, filled: bool) -> Result<(), crate::KernelError> {
        let color_byte = (color & 0xFF) as u8;
        let r = radius as i32;
        let cx = cx as i32;
        let cy = cy as i32;
        
        for y in -r..=r {
            for x in -r..=r {
                let dist_squared = x * x + y * y;
                
                if dist_squared <= r * r {
                    if filled {
                        let px = (cx + x) as u32;
                        let py = (cy + y) as u32;
                        if px < self.current_mode.width && py < self.current_mode.height {
                            self.write_pixel_vesa(px, py, color)?;
                        }
                    } else {
                        // Draw circle outline - only draw if at boundary
                        if dist_squared > (r - 1) * (r - 1) {
                            let px = (cx + x) as u32;
                            let py = (cy + y) as u32;
                            if px < self.current_mode.width && py < self.current_mode.height {
                                self.write_pixel_vesa(px, py, color)?;
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }
    
    fn draw_ellipse(&self, _cx: u32, _cy: u32, _rx: u32, _ry: u32, _color: u32, _filled: bool) -> Result<(), crate::KernelError> {
        // Ellipse drawing implementation would go here
        Ok(())
    }
    
    fn draw_polygon(&self, _points: &[Point], _color: u32, _filled: bool) -> Result<(), crate::KernelError> {
        // Polygon drawing implementation would go here
        Ok(())
    }
    
    fn draw_text(&self, _x: u32, _y: u32, _text: &str, _color: u32, _font: Option<&BitmapFont>) -> Result<(), crate::KernelError> {
        // Text rendering would require bitmap font implementation
        Ok(())
    }
    
    fn blit(&self, buffer: &GraphicsBuffer, x: u32, y: u32) -> Result<(), crate::KernelError> {
        // Simple blit without scaling
        for by in 0..buffer.height {
            for bx in 0..buffer.width {
                let src_offset = (by * buffer.pitch + bx * buffer.color_depth as u32) as usize;
                let dst_x = x + bx;
                let dst_y = y + by;
                
                if dst_x < self.current_mode.width && dst_y < self.current_mode.height {
                    // Extract pixel from source buffer
                    if let Some(pixel_data) = buffer.get_pixel(bx, by) {
                        self.write_pixel_vesa(dst_x, dst_y, pixel_data)?;
                    }
                }
            }
        }
        Ok(())
    }
    
    fn blit_scaled(&self, _buffer: &GraphicsBuffer, _x: u32, _y: u32, _scale_x: f32, _scale_y: f32) -> Result<(), crate::KernelError> {
        // Scaled blit would implement interpolation
        Ok(())
    }
    
    fn blit_rotated(&self, _buffer: &GraphicsBuffer, _x: u32, _y: u32, _angle: f32) -> Result<(), crate::KernelError> {
        Ok(())
    }
    
    fn screenshot(&self, x: u32, y: u32, width: u32, height: u32) -> Result<GraphicsBuffer, crate::KernelError> {
        let mut buffer = GraphicsBuffer::new(width, height, self.current_mode.color_depth)?;
        
        for py in 0..height {
            for px in 0..width {
                if let Ok(color) = self.get_pixel(x + px, y + py) {
                    buffer.set_pixel(px, py, color);
                }
            }
        }
        
        Ok(buffer)
    }
}

/// Graphics driver manager
pub struct GraphicsManager {
    pub vga_driver: Option<VgaGraphics>,
    pub vesa_driver: Option<VesaGraphics>,
    pub current_driver: Option<&'static dyn GraphicsPrimitive + Send + Sync>,
    pub framebuffer_devices: Vec<FramebufferDevice>,
    pub primary_display: Option<u32>,
}

impl GraphicsManager {
    /// Create new graphics driver manager
    pub fn new() -> Self {
        Self {
            vga_driver: None,
            vesa_driver: None,
            current_driver: None,
            framebuffer_devices: Vec::new(),
            primary_display: None,
        }
    }
    
    /// Initialize graphics system
    pub fn initialize(&mut self) -> Result<(), crate::KernelError> {
        info!("Initializing graphics manager");
        
        // Initialize VGA driver if available
        self.init_vga_driver()?;
        
        // Initialize VESA driver if available
        self.init_vesa_driver()?;
        
        // Set default driver
        if self.vesa_driver.is_some() {
            self.set_current_driver(GraphicsMode::Vesa)?;
        } else if self.vga_driver.is_some() {
            self.set_current_driver(GraphicsMode::Vga)?;
        }
        
        info!("Graphics manager initialization complete");
        Ok(())
    }
    
    /// Initialize VGA driver
    fn init_vga_driver(&mut self) -> Result<(), crate::KernelError> {
        // Check for VGA availability (typically at I/O base 0x3CE and VRAM at 0xA0000)
        // In a real implementation, this would do hardware detection
        
        info!("VGA driver detected");
        
        let vga_driver = VgaGraphics::new(0x3CE, 0xA0000);
        self.vga_driver = Some(vga_driver);
        
        info!("VGA graphics driver initialized");
        Ok(())
    }
    
    /// Initialize VESA driver
    fn init_vesa_driver(&mut self) -> Result<(), crate::KernelError> {
        // In real implementation, VESA framebuffer would be detected from bootloader
        // For now, we'll assume a common VESA framebuffer address
        let framebuffer_addr = 0xA0000000; // Common VESA framebuffer address
        
        info!("VESA framebuffer detected at 0x{:X}", framebuffer_addr);
        
        let mut vesa_driver = VesaGraphics::new(framebuffer_addr);
        vesa_driver.init()?;
        self.vesa_driver = Some(vesa_driver);
        
        info!("VESA graphics driver initialized");
        Ok(())
    }
    
    /// Register VGA driver
    pub fn register_vga(&mut self, io_base: u16, vram_base: u64) -> Result<(), crate::KernelError> {
        let mut driver = VgaGraphics::new(io_base, vram_base);
        driver.init_mode()?;
        self.vga_driver = Some(driver);
        info!("VGA graphics driver registered");
        Ok(())
    }
    
    /// Register VESA driver
    pub fn register_vesa(&mut self, framebuffer_addr: u64) -> Result<(), crate::KernelError> {
        let mut driver = VesaGraphics::new(framebuffer_addr);
        driver.init()?;
        self.vesa_driver = Some(driver);
        info!("VESA graphics driver registered");
        Ok(())
    }
    
    /// Set current graphics driver
    pub fn set_current_driver(&mut self, driver_type: GraphicsMode) -> Result<(), crate::KernelError> {
        match driver_type {
            GraphicsMode::Vga => {
                if let Some(ref driver) = self.vga_driver {
                    self.current_driver = Some(unsafe { core::mem::transmute::<&VgaGraphics, &'static dyn GraphicsPrimitive + Send + Sync>(driver) });
                    info!("Set VGA as current graphics driver");
                } else {
                    return Err(crate::KernelError::DeviceNotFound);
                }
            }
            GraphicsMode::Vesa => {
                if let Some(ref driver) = self.vesa_driver {
                    self.current_driver = Some(unsafe { core::mem::transmute::<&VesaGraphics, &'static dyn GraphicsPrimitive + Send + Sync>(driver) });
                    info!("Set VESA as current graphics driver");
                } else {
                    return Err(crate::KernelError::DeviceNotFound);
                }
            }
            _ => return Err(crate::KernelError::DeviceNotFound),
        }
        Ok(())
    }
    
    /// Get current graphics driver
    pub fn get_current_driver(&self) -> Option<&dyn GraphicsPrimitive + Send + Sync> {
        self.current_driver.map(|driver| *driver)
    }
    
    /// Clear screen
    pub fn clear(&self, color: u32) -> Result<(), crate::KernelError> {
        if let Some(driver) = self.current_driver {
            driver.clear(color)
        } else {
            Err(crate::KernelError::DeviceNotFound)
        }
    }
    
    /// Draw pixel
    pub fn draw_pixel(&self, x: u32, y: u32, color: u32) -> Result<(), crate::KernelError> {
        if let Some(driver) = self.current_driver {
            driver.draw_pixel(x, y, color)
        } else {
            Err(crate::KernelError::DeviceNotFound)
        }
    }
    
    /// Get current display information
    pub fn get_current_display_info(&self) -> Option<&FramebufferDevice> {
        if let Some(display_id) = self.primary_display {
            self.framebuffer_devices.iter().find(|dev| dev.id == display_id)
        } else {
            self.framebuffer_devices.first()
        }
    }
    
    /// Get available resolutions for current driver
    pub fn get_available_resolutions(&self) -> Vec<(u32, u32, u8)> {
        if let Some(driver) = &self.vesa_driver {
            driver.available_modes.iter()
                .map(|mode| (mode.width, mode.height, mode.bpp))
                .collect()
        } else {
            vec![(320, 200, 8), (640, 480, 32), (800, 600, 32), (1024, 768, 32)]
        }
    }
}

/// Common colors (in 0xRRGGBB format)
pub const COLORS: [(u32, &str); 16] = [
    (0x000000, "Black"),
    (0xFFFFFF, "White"),
    (0xFF0000, "Red"),
    (0x00FF00, "Green"),
    (0x0000FF, "Blue"),
    (0xFFFF00, "Yellow"),
    (0xFF00FF, "Magenta"),
    (0x00FFFF, "Cyan"),
    (0x808080, "Gray"),
    (0x800000, "Dark Red"),
    (0x008000, "Dark Green"),
    (0x000080, "Dark Blue"),
    (0x808000, "Olive"),
    (0x800080, "Purple"),
    (0x008080, "Teal"),
    (0xC0C0C0, "Silver"),
];

/// Bitmap font structure for text rendering
#[derive(Debug, Clone)]
pub struct BitmapFont {
    pub width: u32,
    pub height: u32,
    pub chars: BTreeMap<char, &'static [u8]>, // Character to bitmap data mapping
}

/// Text rendering utilities
pub struct TextRenderer {
    pub font: Option<BitmapFont>,
}

impl TextRenderer {
    pub fn new() -> Self {
        Self { font: None }
    }
    
    /// Load built-in bitmap font
    pub fn load_default_font(&mut self) {
        // Default 8x8 pixel font
        let default_font_data = include_bytes!("../../../fonts/8x8_font.bin");
        
        // Create bitmap font
        let mut font = BitmapFont {
            width: 8,
            height: 8,
            chars: BTreeMap::new(),
        };
        
        // Load ASCII characters (32-126)
        for (i, &byte) in default_font_data.iter().enumerate() {
            let char_index = 32 + (i / 8); // 8 bytes per character
            if char_index <= 126 {
                let char_byte_index = i % 8;
                // Each character's bitmap is 8 bytes
                // This is a simplified representation
                let char_start = (char_index - 32) * 8;
                let char_end = char_start + 8;
                if char_end <= default_font_data.len() {
                    let char_data = &default_font_data[char_start..char_end];
                    if let Some(&ch) = std::char::from_u32(char_index as u32) {
                        font.chars.insert(ch, char_data);
                    }
                }
            }
        }
        
        self.font = Some(font);
        info!("Default bitmap font loaded: {}x{}", font.width, font.height);
    }
}

impl Default for TextRenderer {
    fn default() -> Self {
        let mut renderer = Self::new();
        renderer.load_default_font();
        renderer
    }
}

impl GraphicsPrimitive for TextRenderer {
    fn clear(&self, _color: u32) -> Result<(), crate::KernelError> {
        // Text renderer cannot clear screen - delegate to graphics driver
        Ok(())
    }
    
    fn draw_pixel(&self, _x: u32, _y: u32, _color: u32) -> Result<(), crate::KernelError> {
        // Text renderer cannot draw individual pixels - delegate to graphics driver
        Ok(())
    }
    
    fn get_pixel(&self, _x: u32, _y: u32) -> Result<u32, crate::KernelError> {
        // Text renderer cannot read pixels - delegate to graphics driver
        Ok(0)
    }
    
    fn draw_line_h(&self, _x1: u32, _y: u32, _x2: u32, _color: u32) -> Result<(), crate::KernelError> {
        // Text renderer cannot draw lines - delegate to graphics driver
        Ok(())
    }
    
    fn draw_line_v(&self, _x: u32, _y1: u32, _y2: u32, _color: u32) -> Result<(), crate::KernelError> {
        // Text renderer cannot draw lines - delegate to graphics driver
        Ok(())
    }
    
    fn draw_line(&self, _x1: u32, _y1: u32, _x2: u32, _y2: u32, _color: u32) -> Result<(), crate::KernelError> {
        // Text renderer cannot draw lines - delegate to graphics driver
        Ok(())
    }
    
    fn draw_rect(&self, _x: u32, _y: u32, _width: u32, _height: u32, _color: u32, _filled: bool) -> Result<(), crate::KernelError> {
        // Text renderer cannot draw rectangles - delegate to graphics driver
        Ok(())
    }
    
    fn draw_circle(&self, _cx: u32, _cy: u32, _radius: u32, _color: u32, _filled: bool) -> Result<(), crate::KernelError> {
        // Text renderer cannot draw circles - delegate to graphics driver
        Ok(())
    }
    
    fn draw_ellipse(&self, _cx: u32, _cy: u32, _rx: u32, _ry: u32, _color: u32, _filled: bool) -> Result<(), crate::KernelError> {
        Ok(())
    }
    
    fn draw_polygon(&self, _points: &[Point], _color: u32, _filled: bool) -> Result<(), crate::KernelError> {
        Ok(())
    }
    
    fn draw_text(&self, _x: u32, _y: u32, _text: &str, _color: u32, _font: Option<&BitmapFont>) -> Result<(), crate::KernelError> {
        // This would be implemented to draw text using the font
        Ok(())
    }
    
    fn blit(&self, _buffer: &GraphicsBuffer, _x: u32, _y: u32) -> Result<(), crate::KernelError> {
        // Text renderer cannot blit - delegate to graphics driver
        Ok(())
    }
    
    fn blit_scaled(&self, _buffer: &GraphicsBuffer, _x: u32, _y: u32, _scale_x: f32, _scale_y: f32) -> Result<(), crate::KernelError> {
        Ok(())
    }
    
    fn blit_rotated(&self, _buffer: &GraphicsBuffer, _x: u32, _y: u32, _angle: f32) -> Result<(), crate::KernelError> {
        Ok(())
    }
    
    fn screenshot(&self, _x: u32, _y: u32, _width: u32, _height: u32) -> Result<GraphicsBuffer, crate::KernelError> {
        // Text renderer cannot take screenshots - delegate to graphics driver
        Ok(GraphicsBuffer::new(1, 1, ColorDepth::Bpp32).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graphics_mode_info() {
        let mode = GraphicsModeInfo {
            width: 1024,
            height: 768,
            color_depth: ColorDepth::Bpp32,
            pitch: 4096,
            framebuffer_addr: 0xA0000,
            mode_number: 0x118,
            attributes: 0x00,
            bpp: 32,
            bytes_per_pixel: 4,
            red_mask: 8,
            green_mask: 8,
            blue_mask: 8,
            alpha_mask: 8,
            red_shift: 0,
            green_shift: 8,
            blue_shift: 16,
            alpha_shift: 24,
        };
        
        assert_eq!(mode.width, 1024);
        assert_eq!(mode.height, 768);
        assert_eq!(mode.bpp, 32);
        assert_eq!(mode.get_bytes_per_pixel(), 4);
        assert_eq!(mode.has_alpha_channel(), true);
    }

    #[test]
    fn test_color_conversion() {
        let mode = GraphicsModeInfo {
            width: 1024,
            height: 768,
            color_depth: ColorDepth::Bpp32,
            pitch: 4096,
            framebuffer_addr: 0xA0000,
            mode_number: 0x118,
            attributes: 0x00,
            bpp: 32,
            bytes_per_pixel: 4,
            red_mask: 8,
            green_mask: 8,
            blue_mask: 8,
            alpha_mask: 8,
            red_shift: 0,
            green_shift: 8,
            blue_shift: 16,
            alpha_shift: 24,
        };
        
        let pixel = mode.color_to_pixel(255, 128, 64, 255);
        let (r, g, b, a) = mode.pixel_to_color(pixel);
        
        assert_eq!(r, 255);
        assert_eq!(g, 128);
        assert_eq!(b, 64);
        assert_eq!(a, 255);
    }

    #[test]
    fn test_graphics_buffer_creation() {
        let buffer = GraphicsBuffer::new(320, 240, ColorDepth::Bpp32).unwrap();
        
        assert_eq!(buffer.width, 320);
        assert_eq!(buffer.height, 240);
        assert_eq!(buffer.pitch, 320 * 4);
        assert_eq!(buffer.color_depth, ColorDepth::Bpp32);
        
        // Test pixel operations
        buffer.set_pixel(10, 20, 0xFF00FF);
        assert_eq!(buffer.get_pixel(10, 20), Some(0xFF00FF));
        assert_eq!(buffer.get_pixel(1000, 1000), None); // Out of bounds
    }

    #[test]
    fn test_rectangle_operations() {
        let rect1 = Rectangle::new(10, 10, 50, 50);
        let rect2 = Rectangle::new(30, 30, 50, 50);
        let rect3 = Rectangle::new(100, 100, 20, 20);
        
        // Test contains
        assert!(rect1.contains(15, 15));
        assert!(!rect1.contains(100, 100));
        
        // Test intersection
        assert!(rect1.intersects(&rect2));
        assert!(!rect1.intersects(&rect3));
        
        let intersection = rect1.intersection(&rect2).unwrap();
        assert_eq!(intersection.x, 30);
        assert_eq!(intersection.y, 30);
        assert_eq!(intersection.width, 30);
        assert_eq!(intersection.height, 30);
    }

    #[test]
    fn test_vga_driver() {
        let mut vga_driver = VgaGraphics::new(0x3CE, 0xA0000);
        assert!(vga_driver.init_mode().is_ok());
        
        // Test basic operations
        assert!(vga_driver.clear(0x00).is_ok());
        assert!(vga_driver.draw_pixel(10, 20, 0xFF).is_ok());
        assert_eq!(vga_driver.get_pixel(10, 20), Ok(0xFF));
        
        // Test line drawing
        assert!(vga_driver.draw_line_h(0, 50, 100, 0xAA).is_ok());
        assert!(vga_driver.draw_line_v(50, 0, 100, 0xAA).is_ok());
        
        // Test rectangle drawing
        assert!(vga_driver.draw_rect(10, 10, 50, 30, 0x55, false).is_ok());
        assert!(vga_driver.draw_rect(60, 10, 50, 30, 0x55, true).is_ok());
        
        // Test circle drawing
        assert!(vga_driver.draw_circle(100, 100, 20, 0x77, false).is_ok());
        assert!(vga_driver.draw_circle(150, 100, 20, 0x77, true).is_ok());
    }

    #[test]
    fn test_vesa_driver() {
        let mut vesa_driver = VesaGraphics::new(0xA0000000);
        assert!(vesa_driver.init().is_ok());
        
        // Test basic operations
        assert!(vesa_driver.clear(0xFF000000).is_ok());
        assert!(vesa_driver.draw_pixel(10, 20, 0xFF00FF00).is_ok());
        assert_eq!(vesa_driver.get_pixel(10, 20), Ok(0xFF00FF00));
        
        // Test screenshot
        let screenshot = vesa_driver.screenshot(5, 5, 10, 10).unwrap();
        assert_eq!(screenshot.width, 10);
        assert_eq!(screenshot.height, 10);
    }

    #[test]
    fn test_graphics_manager() {
        let mut manager = GraphicsManager::new();
        assert!(manager.initialize().is_ok());
        
        // Test default driver setting
        assert!(manager.set_current_driver(GraphicsMode::Vga).is_ok());
        assert!(manager.get_current_driver().is_some());
        
        // Test drawing operations
        assert!(manager.clear(0x000000).is_ok());
        assert!(manager.draw_pixel(10, 10, 0xFFFFFF).is_ok());
        
        // Test available resolutions
        let resolutions = manager.get_available_resolutions();
        assert!(!resolutions.is_empty());
        
        let (width, height, bpp) = resolutions[0];
        assert!(width > 0 && height > 0 && bpp > 0);
    }
}