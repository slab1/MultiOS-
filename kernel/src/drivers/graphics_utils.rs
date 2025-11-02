//! MultiOS Graphics Utilities and Advanced Drawing Operations
//!
//! This module provides advanced graphics utilities including:
//! - Enhanced graphics primitives (anti-aliased lines, gradients, patterns)
//! - Graphics effects (shadows, blur, transparency)
//! - Animation and rendering utilities
//! - Graphics helpers and utilities

use crate::log::{info, warn, error, debug};
use alloc::{vec::Vec, collections::BTreeMap};
use spin::{Mutex, RwLock, Once};
use core::ops::{Deref, DerefMut};

/// Graphics utilities initialization
pub fn init() -> Result<(), crate::KernelError> {
    info!("Initializing graphics utilities...");
    
    let mut utilities = GRAPHICS_UTILITIES.get().write();
    utilities.initialize()?;
    
    info!("Graphics utilities initialized successfully");
    Ok(())
}

/// Global graphics utilities
pub static GRAPHICS_UTILITIES: Once<Mutex<GraphicsUtilities>> = Once::new();

/// Color utilities for advanced color operations
pub struct ColorUtils;

impl ColorUtils {
    /// Convert RGB to HSV
    pub fn rgb_to_hsv(r: u8, g: u8, b: u8) -> (f32, f32, f32) {
        let r = r as f32 / 255.0;
        let g = g as f32 / 255.0;
        let b = b as f32 / 255.0;
        
        let max = r.max(g.max(b));
        let min = r.min(g.min(b));
        let delta = max - min;
        
        let mut h = 0.0;
        let s = if max == 0.0 { 0.0 } else { delta / max };
        let v = max;
        
        if delta != 0.0 {
            h = match max {
                _ if r == max => (g - b) / delta + if g < b { 6.0 } else { 0.0 },
                _ if g == max => (b - r) / delta + 2.0,
                _ => (r - g) / delta + 4.0,
            };
            h /= 6.0;
        }
        
        (h, s, v)
    }
    
    /// Convert HSV to RGB
    pub fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (u8, u8, u8) {
        let h = h % 1.0;
        let i = (h * 6.0).floor();
        let f = h * 6.0 - i;
        let p = v * (1.0 - s);
        let q = v * (1.0 - s * f);
        let t = v * (1.0 - s * (1.0 - f));
        
        let (r, g, b) = match i as u32 % 6 {
            0 => (v, t, p),
            1 => (q, v, p),
            2 => (p, v, t),
            3 => (p, q, v),
            4 => (t, p, v),
            _ => (v, p, q),
        };
        
        ((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
    }
    
    /// Mix two colors
    pub fn mix_colors(color1: u32, color2: u32, t: f32) -> u32 {
        let r1 = (color1 & 0xFF) as u8;
        let g1 = ((color1 >> 8) & 0xFF) as u8;
        let b1 = ((color1 >> 16) & 0xFF) as u8;
        let a1 = ((color1 >> 24) & 0xFF) as u8;
        
        let r2 = (color2 & 0xFF) as u8;
        let g2 = ((color2 >> 8) & 0xFF) as u8;
        let b2 = ((color2 >> 16) & 0xFF) as u8;
        let a2 = ((color2 >> 24) & 0xFF) as u8;
        
        let r = (r1 as f32 * (1.0 - t) + r2 as f32 * t) as u32;
        let g = (g1 as f32 * (1.0 - t) + g2 as f32 * t) as u32;
        let b = (b1 as f32 * (1.0 - t) + b2 as f32 * t) as u32;
        let a = (a1 as f32 * (1.0 - t) + a2 as f32 * t) as u32;
        
        r | (g << 8) | (b << 16) | (a << 24)
    }
    
    /// Calculate alpha blend of two colors
    pub fn alpha_blend(foreground: u32, background: u32, alpha: f32) -> u32 {
        let fg_r = (foreground & 0xFF) as u8;
        let fg_g = ((foreground >> 8) & 0xFF) as u8;
        let fg_b = ((foreground >> 16) & 0xFF) as u8;
        let fg_a = ((foreground >> 24) & 0xFF) as u8;
        
        let bg_r = (background & 0xFF) as u8;
        let bg_g = ((background >> 8) & 0xFF) as u8;
        let bg_b = ((background >> 16) & 0xFF) as u8;
        
        let alpha_f = alpha.min(1.0).max(0.0);
        let fg_alpha = (fg_a as f32 / 255.0) * alpha_f;
        
        let r = (fg_r as f32 * fg_alpha + bg_r as f32 * (1.0 - fg_alpha)) as u32;
        let g = (fg_g as f32 * fg_alpha + bg_g as f32 * (1.0 - fg_alpha)) as u32;
        let b = (fg_b as f32 * fg_alpha + bg_b as f32 * (1.0 - fg_alpha)) as u32;
        
        r | (g << 8) | (b << 16) | 0xFF000000
    }
    
    /// Generate gradient color
    pub fn gradient(start_color: u32, end_color: u32, t: f32) -> u32 {
        Self::mix_colors(start_color, end_color, t)
    }
}

/// Pattern types for filling
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum PatternType {
    Solid = 0,
    HorizontalLines = 1,
    VerticalLines = 2,
    Grid = 3,
    DiagonalLines = 4,
    Checkerboard = 5,
    Dither = 6,
}

/// Pattern definition
#[derive(Debug, Clone)]
pub struct Pattern {
    pub pattern_type: PatternType,
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>, // Pattern bitmap
    pub colors: Vec<u32>, // Pattern colors
}

/// Pattern generator
pub struct PatternGenerator;

impl PatternGenerator {
    /// Generate solid pattern
    pub fn generate_solid(width: u32, height: u32, color: u32) -> Pattern {
        Pattern {
            pattern_type: PatternType::Solid,
            width,
            height,
            data: vec![0u8; (width * height) as usize],
            colors: vec![color],
        }
    }
    
    /// Generate horizontal lines pattern
    pub fn generate_horizontal_lines(width: u32, height: u32, line_spacing: u32, colors: Vec<u32>) -> Pattern {
        let mut data = vec![0u8; (width * height) as usize];
        let mut pattern = Pattern {
            pattern_type: PatternType::HorizontalLines,
            width,
            height,
            data,
            colors: colors.into_iter().take(2).collect(),
        };
        
        if pattern.colors.len() < 2 {
            pattern.colors.push(0x00000000); // Black
            pattern.colors.push(0xFFFFFFFF); // White
        }
        
        for y in 0..height {
            let color_index = (y / line_spacing) % 2;
            let color = pattern.colors[color_index as usize];
            
            for x in 0..width {
                let r = (color & 0xFF) as u8;
                let g = ((color >> 8) & 0xFF) as u8;
                let b = ((color >> 16) & 0xFF) as u8;
                
                pattern.data[(y * width + x) as usize] = ((r + g + b) / 3) as u8;
            }
        }
        
        pattern
    }
    
    /// Generate checkerboard pattern
    pub fn generate_checkerboard(width: u32, height: u32, square_size: u32, colors: Vec<u32>) -> Pattern {
        let mut data = vec![0u8; (width * height) as usize];
        let mut pattern = Pattern {
            pattern_type: PatternType::Checkerboard,
            width,
            height,
            data,
            colors: colors.into_iter().take(2).collect(),
        };
        
        if pattern.colors.len() < 2 {
            pattern.colors.push(0xFF000000); // Black
            pattern.colors.push(0xFFFFFFFF); // White
        }
        
        for y in 0..height {
            for x in 0..width {
                let square_x = x / square_size;
                let square_y = y / square_size;
                let color_index = (square_x + square_y) % 2;
                let color = pattern.colors[color_index as usize];
                
                pattern.data[(y * width + x) as usize] = ((color & 0xFF) as u8);
            }
        }
        
        pattern
    }
    
    /// Apply pattern to graphics primitive
    pub fn apply_pattern<F>(pattern: &Pattern, mut apply_pixel: F) 
    where F: FnMut(u32, u32, u32) {
        for y in 0..pattern.height {
            for x in 0..pattern.width {
                let pixel_index = (y * pattern.width + x) as usize;
                if pixel_index < pattern.data.len() {
                    let intensity = pattern.data[pixel_index];
                    let color_index = (intensity as usize) % pattern.colors.len();
                    let color = pattern.colors[color_index];
                    apply_pixel(x, y, color);
                }
            }
        }
    }
}

/// Advanced drawing operations
pub struct AdvancedGraphics;

impl AdvancedGraphics {
    /// Draw anti-aliased line (simplified version)
    pub fn draw_anti_aliased_line<F>(width: u32, height: u32, x1: f32, y1: f32, x2: f32, y2: f32, color: u32, mut set_pixel: F)
    where F: FnMut(u32, u32, u32) {
        let dx = x2 - x1;
        let dy = y2 - y1;
        let length = (dx * dx + dy * dy).sqrt();
        
        if length == 0.0 {
            return;
        }
        
        let steps = length.ceil() as u32;
        
        for i in 0..=steps {
            let t = i as f32 / steps as f32;
            let x = x1 + t * dx;
            let y = y1 + t * dy;
            
            let pixel_x = x.round() as u32;
            let pixel_y = y.round() as u32;
            
            if pixel_x < width && pixel_y < height {
                let alpha = 255 - ((t * 255.0).abs() * 0.5) as u8; // Simple anti-aliasing
                let alpha_color = (color & 0xFFFFFF) | ((alpha as u32) << 24);
                set_pixel(pixel_x, pixel_y, alpha_color);
            }
        }
    }
    
    /// Draw gradient rectangle
    pub fn draw_gradient_rect<F>(width: u32, height: u32, x: u32, y: u32, 
                               rect_width: u32, rect_height: u32, 
                               start_color: u32, end_color: u32,
                               direction: u32, // 0=horizontal, 1=vertical
                               mut set_pixel: F)
    where F: FnMut(u32, u32, u32) {
        for py in y..(y + rect_height) {
            for px in x..(x + rect_width) {
                let t = if direction == 0 {
                    (px - x) as f32 / rect_width as f32
                } else {
                    (py - y) as f32 / rect_height as f32
                };
                
                let color = ColorUtils::gradient(start_color, end_color, t);
                set_pixel(px, py, color);
            }
        }
    }
    
    /// Draw rounded rectangle
    pub fn draw_rounded_rect<F>(width: u32, height: u32, x: u32, y: u32, 
                              rect_width: u32, rect_height: u32, 
                              corner_radius: u32, color: u32, filled: bool,
                              mut set_pixel: F)
    where F: FnMut(u32, u32, u32) {
        let cx = x + rect_width / 2;
        let cy = y + rect_height / 2;
        let r = corner_radius;
        let r_squared = (r * r) as f32;
        
        for py in y..(y + rect_height) {
            for px in x..(x + rect_width) {
                let dx = (px as i32 - cx as i32).abs() as f32;
                let dy = (py as i32 - cy as i32).abs() as f32;
                
                let distance_squared = dx * dx + dy * dy;
                
                if distance_squared <= r_squared || !filled {
                    if filled {
                        set_pixel(px, py, color);
                    } else {
                        // Draw border only
                        if distance_squared > (r - 1).pow(2) as f32 && distance_squared <= r_squared {
                            set_pixel(px, py, color);
                        }
                    }
                }
            }
        }
    }
    
    /// Draw radial gradient circle
    pub fn draw_radial_gradient<F>(width: u32, height: u32, cx: u32, cy: u32, 
                                 radius: u32, center_color: u32, edge_color: u32,
                                 mut set_pixel: F)
    where F: FnMut(u32, u32, u32) {
        let r_squared = radius * radius;
        let cx_f = cx as f32;
        let cy_f = cy as f32;
        
        for y in (cy.saturating_sub(radius))..(cy + radius) {
            for x in (cx.saturating_sub(radius))..(cx + radius) {
                let dx = x as f32 - cx_f;
                let dy = y as f32 - cy_f;
                let dist_squared = dx * dx + dy * dy;
                
                if dist_squared <= r_squared as f32 {
                    let t = (dist_squared.sqrt() / radius as f32).min(1.0);
                    let color = ColorUtils::gradient(center_color, edge_color, t);
                    set_pixel(x, y, color);
                }
            }
        }
    }
    
    /// Draw polygon with enhanced algorithms
    pub fn draw_polygon<F>(points: &[crate::drivers::graphics::Point], color: u32, 
                         filled: bool, mut set_pixel: F)
    where F: FnMut(u32, u32, u32) {
        if points.len() < 3 {
            return;
        }
        
        if filled {
            // Use scanline algorithm for filled polygon
            let min_y = points.iter().map(|p| p.y).min().unwrap_or(0);
            let max_y = points.iter().map(|p| p.y).max().unwrap_or(0);
            
            for y in min_y..=max_y {
                let mut intersections = Vec::new();
                
                for i in 0..points.len() {
                    let p1 = points[i];
                    let p2 = points[(i + 1) % points.len()];
                    
                    if (p1.y <= y && p2.y > y) || (p2.y <= y && p1.y > y) {
                        let t = (y as f32 - p1.y as f32) / (p2.y as f32 - p1.y as f32);
                        let x = p1.x as f32 + t * (p2.x as f32 - p1.x as f32);
                        intersections.push(x);
                    }
                }
                
                intersections.sort_by(|a, b| a.partial_cmp(b).unwrap());
                
                for i in (0..intersections.len()).step_by(2) {
                    if i + 1 < intersections.len() {
                        let x1 = intersections[i].round() as u32;
                        let x2 = intersections[i + 1].round() as u32;
                        
                        for x in x1..=x2 {
                            if x < crate::drivers::graphics::GraphicsBuffer::new(1, 1, crate::drivers::graphics::ColorDepth::Bpp32).unwrap().width {
                                set_pixel(x, y, color);
                            }
                        }
                    }
                }
            }
        } else {
            // Draw polygon outline
            for i in 0..points.len() {
                let p1 = points[i];
                let p2 = points[(i + 1) % points.len()];
                
                let dx = (p2.x as i32 - p1.x as i32).abs();
                let dy = (p2.y as i32 - p1.y as i32).abs();
                let steps = dx.max(dy) as u32;
                
                for step in 0..=steps {
                    let t = step as f32 / steps as f32;
                    let x = p1.x as f32 + t * (p2.x as f32 - p1.x as f32);
                    let y = p1.y as f32 + t * (p2.y as f32 - p1.y as f32);
                    
                    let pixel_x = x.round() as u32;
                    let pixel_y = y.round() as u32;
                    
                    set_pixel(pixel_x, pixel_y, color);
                }
            }
        }
    }
}

/// Animation utilities
pub struct AnimationUtilities;

impl AnimationUtilities {
    /// Create new animation utilities
    pub fn new() -> Self {
        Self
    }

    /// Linear interpolation between two values
    pub fn lerp(start: f32, end: f32, t: f32) -> f32 {
        start + t * (end - start)
    }
    
    /// Easing function: ease-in
    pub fn ease_in(t: f32) -> f32 {
        t * t
    }
    
    /// Easing function: ease-out
    pub fn ease_out(t: f32) -> f32 {
        1.0 - (1.0 - t) * (1.0 - t)
    }
    
    /// Easing function: ease-in-out
    pub fn ease_in_out(t: f32) -> f32 {
        if t < 0.5 {
            2.0 * t * t
        } else {
            1.0 - 2.0 * (1.0 - t) * (1.0 - t)
        }
    }
    
    /// Bounce easing function
    pub fn bounce(t: f32) -> f32 {
        if t < 1.0 / 2.75 {
            7.5625 * t * t
        } else if t < 2.0 / 2.75 {
            t -= 1.5 / 2.75;
            7.5625 * t * t + 0.75
        } else if t < 2.5 / 2.75 {
            t -= 2.25 / 2.75;
            7.5625 * t * t + 0.9375
        } else {
            t -= 2.625 / 2.75;
            7.5625 * t * t + 0.984375
        }
    }
}

/// Graphics effects
pub struct GraphicsEffects;

impl GraphicsEffects {
    /// Apply shadow effect to graphics
    pub fn apply_shadow<F>(mut draw_pixel: F, shadow_offset: (i32, i32), shadow_color: u32, shadow_alpha: f32)
    where F: FnMut(u32, u32, u32) {
        let original_draw = draw_pixel;
        
        // Shadow will be drawn before main graphics
        move |x: u32, y: u32, color: u32| {
            let shadow_x = (x as i32 + shadow_offset.0) as u32;
            let shadow_y = (y as i32 + shadow_offset.1) as u32;
            let shadow_color_with_alpha = (shadow_color & 0xFFFFFF) | 
                ((shadow_alpha * 255.0) as u32) << 24;
            
            original_draw(shadow_x, shadow_y, shadow_color_with_alpha);
            original_draw(x, y, color);
        }
    }
    
    /// Apply blur effect (simplified box blur)
    pub fn apply_blur(pixels: &mut [u8], width: u32, height: u32, radius: u32) {
        let mut temp = pixels.to_vec();
        let radius = radius.min(5); // Limit blur radius
        
        for y in 0..height {
            for x in 0..width {
                let mut r_sum = 0u32;
                let mut g_sum = 0u32;
                let mut b_sum = 0u32;
                let mut count = 0u32;
                
                for dy in 0..=radius {
                    for dx in 0..=radius {
                        let px = x.saturating_add(dx).min(width - 1);
                        let py = y.saturating_add(dy).min(height - 1);
                        let offset = ((py * width + px) * 4) as usize;
                        
                        if offset + 3 < pixels.len() {
                            r_sum += pixels[offset] as u32;
                            g_sum += pixels[offset + 1] as u32;
                            b_sum += pixels[offset + 2] as u32;
                            count += 1;
                        }
                    }
                }
                
                if count > 0 {
                    let offset = ((y * width + x) * 4) as usize;
                    if offset + 3 < pixels.len() {
                        temp[offset] = (r_sum / count) as u8;
                        temp[offset + 1] = (g_sum / count) as u8;
                        temp[offset + 2] = (b_sum / count) as u8;
                    }
                }
            }
        }
        
        pixels.copy_from_slice(&temp);
    }
    
    /// Apply alpha blending effect
    pub fn apply_alpha_blend(pixels: &mut [u8], alpha: f32) {
        for chunk in pixels.chunks_mut(4) {
            if chunk.len() >= 3 {
                chunk[3] = (chunk[3] as f32 * alpha).min(255.0).max(0.0) as u8;
            }
        }
    }
}

/// Graphics utilities manager
pub struct GraphicsUtilities {
    pub initialized: bool,
}

impl GraphicsUtilities {
    /// Create new graphics utilities manager
    pub fn new() -> Self {
        Self {
            initialized: false,
        }
    }
    
    /// Initialize graphics utilities
    pub fn initialize(&mut self) -> Result<(), crate::KernelError> {
        if self.initialized {
            warn!("Graphics utilities already initialized");
            return Ok(());
        }
        
        info!("Initializing graphics utilities");
        
        self.initialized = true;
        info!("Graphics utilities initialization complete");
        Ok(())
    }
    
    /// Get color utilities
    pub fn colors(&self) -> ColorUtils {
        ColorUtils
    }
    
    /// Get pattern generator
    pub fn patterns(&self) -> PatternGenerator {
        PatternGenerator
    }
    
    /// Get advanced graphics operations
    pub fn advanced(&self) -> AdvancedGraphics {
        AdvancedGraphics
    }
    
    /// Get animation utilities
    pub fn animation(&self) -> AnimationUtilities {
        AnimationUtilities
    }
    
    /// Get graphics effects
    pub fn effects(&self) -> GraphicsEffects {
        GraphicsEffects
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_utils() {
        // Test RGB to HSV conversion
        let (h, s, v) = ColorUtils::rgb_to_hsv(255, 0, 0);
        assert!((h - 0.0).abs() < 0.01);
        assert_eq!(s, 1.0);
        assert_eq!(v, 1.0);
        
        // Test color mixing
        let mixed = ColorUtils::mix_colors(0xFF0000FF, 0x0000FFFF, 0.5);
        assert_eq!(mixed, 0x7F007FFF);
        
        // Test alpha blending
        let blended = ColorUtils::alpha_blend(0x80000000, 0x00000000, 0.5);
        assert_eq!(blended, 0x40000000);
    }

    #[test]
    fn test_pattern_generator() {
        let pattern = PatternGenerator::generate_solid(10, 10, 0xFF0000FF);
        assert_eq!(pattern.pattern_type, PatternType::Solid);
        assert_eq!(pattern.width, 10);
        assert_eq!(pattern.height, 10);
        assert_eq!(pattern.colors.len(), 1);
        
        let h_lines = PatternGenerator::generate_horizontal_lines(20, 10, 2, vec![0xFF0000FF, 0x00FF00FF]);
        assert_eq!(h_lines.pattern_type, PatternType::HorizontalLines);
        
        let checker = PatternGenerator::generate_checkerboard(10, 10, 2, vec![0xFFFFFFFF, 0x000000FF]);
        assert_eq!(checker.pattern_type, PatternType::Checkerboard);
    }

    #[test]
    fn test_advanced_graphics() {
        let mut pixels = Vec::new();
        let width = 100;
        let height = 100;
        
        let draw_pixel = |x: u32, y: u32, color: u32| {
            let offset = ((y * width + x) * 4) as usize;
            if offset + 3 < pixels.len() {
                let bytes = color.to_le_bytes();
                pixels[offset] = bytes[0];
                pixels[offset + 1] = bytes[1];
                pixels[offset + 2] = bytes[2];
                pixels[offset + 3] = bytes[3];
            }
        };
        
        // Test drawing anti-aliased line
        pixels.resize((width * height * 4) as usize, 0);
        AdvancedGraphics::draw_anti_aliased_line(width, height, 0.0, 0.0, 50.0, 50.0, 0xFF0000FF, draw_pixel);
        
        // Test gradient rectangle
        AdvancedGraphics::draw_gradient_rect(width, height, 10, 10, 30, 30, 0xFF0000FF, 0x00FF00FF, 0, draw_pixel);
        
        // Test rounded rectangle
        AdvancedGraphics::draw_rounded_rect(width, height, 50, 50, 20, 20, 5, 0x0000FFFF, true, draw_pixel);
        
        // Test radial gradient circle
        AdvancedGraphics::draw_radial_gradient(width, height, 25, 25, 15, 0xFFFF00FF, 0xFF00FFFF, draw_pixel);
    }

    #[test]
    fn test_animation_utilities() {
        let anim = AnimationUtilities::new();
        
        assert_eq!(anim.lerp(0.0, 10.0, 0.5), 5.0);
        assert_eq!(anim.ease_in(0.5), 0.25);
        assert_eq!(anim.ease_out(0.5), 0.75);
        
        let eased = anim.ease_in_out(0.5);
        assert!(eased >= 0.0 && eased <= 1.0);
    }

    #[test]
    fn test_graphics_effects() {
        let mut pixels = vec![0xFFu8; 100 * 4];
        
        // Test alpha blend effect
        GraphicsEffects::apply_alpha_blend(&mut pixels, 0.5);
        assert_eq!(pixels[3], 127); // Alpha channel should be halved
        
        // Test blur effect (simplified)
        GraphicsEffects::apply_blur(&mut pixels, 10, 10, 1);
        // Blur test - pixels should be modified
    }

    #[test]
    fn test_graphics_utilities_manager() {
        let mut utils = GraphicsUtilities::new();
        assert!(utils.initialize().is_ok());
        assert!(utils.initialized);
        
        // Test getting utility components
        let _colors = utils.colors();
        let _patterns = utils.patterns();
        let _advanced = utils.advanced();
        let _animation = utils.animation();
        let _effects = utils.effects();
    }
}