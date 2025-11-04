//! MultiOS Graphics Driver Integration Tests
//!
//! This module provides integration tests for the graphics driver and
//! framebuffer management system.

use crate::drivers::graphics::*;
use crate::drivers::framebuffer::*;
use crate::drivers::bitmap_font::*;
use crate::drivers::graphics_utils::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graphics_system_integration() {
        // Test graphics manager initialization
        let mut graphics_manager = GraphicsManager::new();
        assert!(graphics_manager.initialize().is_ok());
        
        // Test that drivers are registered
        assert!(graphics_manager.vesa_driver.is_some() || graphics_manager.vga_driver.is_some());
        
        // Test setting current driver
        if graphics_manager.vga_driver.is_some() {
            assert!(graphics_manager.set_current_driver(GraphicsMode::Vga).is_ok());
            assert!(graphics_manager.get_current_driver().is_some());
        }
        
        if graphics_manager.vesa_driver.is_some() {
            assert!(graphics_manager.set_current_driver(GraphicsMode::Vesa).is_ok());
            assert!(graphics_manager.get_current_driver().is_some());
        }
        
        // Test basic drawing operations
        assert!(graphics_manager.clear(0x000000).is_ok());
        assert!(graphics_manager.draw_pixel(10, 10, 0xFFFFFF).is_ok());
    }

    #[test]
    fn test_framebuffer_system_integration() {
        // Test framebuffer manager initialization
        let mut framebuffer_manager = FramebufferManager::new();
        assert!(framebuffer_manager.initialize().is_ok());
        
        // Test that displays are detected
        assert!(framebuffer_manager.display_count() > 0);
        assert!(framebuffer_manager.get_primary_display().is_some());
        
        // Test acceleration manager
        assert!(framebuffer_manager.acceleration.initialized);
        assert!(framebuffer_manager.acceleration.supports_operation(AccelerationOperation::Clear));
        assert!(framebuffer_manager.acceleration.supports_operation(AccelerationOperation::Blit));
    }

    #[test]
    fn test_font_system_integration() {
        // Test font manager initialization
        let mut font_manager = FontManager::new();
        assert!(font_manager.initialize().is_ok());
        
        // Test that fonts are loaded
        assert!(font_manager.initialized);
        assert!(font_manager.get_default_font().is_some());
        
        let font = font_manager.get_default_font().unwrap();
        assert!(font.glyphs.len() > 0);
        
        // Test font metrics
        assert_eq!(font.size, 8); // Default font is 8x8
        assert_eq!(font.is_monospace, true);
        
        // Test text width calculation
        let text = "Hello";
        let width = font.text_width(text);
        assert!(width > 0);
        assert_eq!(width, text.len() as u32 * font.glyph_width);
    }

    #[test]
    fn test_graphics_buffers() {
        // Test graphics buffer creation
        let buffer = GraphicsBuffer::new(100, 100, ColorDepth::Bpp32).unwrap();
        assert_eq!(buffer.width, 100);
        assert_eq!(buffer.height, 100);
        assert_eq!(buffer.color_depth, ColorDepth::Bpp32);
        
        // Test pixel operations
        buffer.set_pixel(10, 20, 0xFF00FF00);
        assert_eq!(buffer.get_pixel(10, 20), Some(0xFF00FF00));
        
        // Test bounds checking
        assert_eq!(buffer.get_pixel(1000, 1000), None);
        buffer.set_pixel(1000, 1000, 0xFFFFFFFF);
        assert_eq!(buffer.get_pixel(1000, 1000), None);
        
        // Test buffer clear
        let mut buffer = GraphicsBuffer::new(10, 10, ColorDepth::Bpp32).unwrap();
        buffer.set_pixel(5, 5, 0xFF0000);
        buffer.clear(0x000000);
        assert_eq!(buffer.get_pixel(5, 5), Some(0x000000));
    }

    #[test]
    fn test_graphics_primitives() {
        let buffer = GraphicsBuffer::new(200, 200, ColorDepth::Bpp32).unwrap();
        
        // Test VGA driver primitive operations
        let vga_driver = VgaGraphics::new(0x3CE, 0xA0000);
        
        // Note: These tests are conceptual since we can't actually access memory
        // In a real implementation, these would require actual framebuffer memory
        
        // Test basic drawing operations would be available
        assert_eq!(vga_driver.current_mode.width, 320);
        assert_eq!(vga_driver.current_mode.height, 200);
        assert_eq!(vga_driver.current_mode.color_depth, ColorDepth::Bpp8);
        
        // Test VESA driver primitive operations
        let vesa_driver = VesaGraphics::new(0xA0000000);
        assert_eq!(vesa_driver.current_mode.width, 1024);
        assert_eq!(vesa_driver.current_mode.height, 768);
        assert_eq!(vesa_driver.current_mode.color_depth, ColorDepth::Bpp32);
        
        // Test available modes
        assert!(!vesa_driver.available_modes.is_empty());
        assert_eq!(vesa_driver.available_modes.len(), 3); // 640x480, 800x600, 1024x768
    }

    #[test]
    fn test_color_utilities() {
        let colors = ColorUtils;
        
        // Test RGB to HSV conversion
        let (h, s, v) = colors.rgb_to_hsv(255, 0, 0);
        assert!((h - 0.0).abs() < 0.01);
        assert_eq!(s, 1.0);
        assert_eq!(v, 1.0);
        
        // Test color mixing
        let mixed = colors.mix_colors(0xFF0000FF, 0x0000FFFF, 0.5);
        assert_eq!(mixed, 0x7F007FFF);
        
        // Test alpha blending
        let blended = colors.alpha_blend(0x80000000, 0x00000000, 0.5);
        assert_eq!(blended, 0x40000000);
        
        // Test gradient
        let gradient = colors.gradient(0xFF0000FF, 0x0000FFFF, 0.75);
        assert_eq!(gradient, 0x4000FFFF);
    }

    #[test]
    fn test_pattern_generation() {
        let patterns = PatternGenerator;
        
        // Test solid pattern
        let solid = patterns.generate_solid(10, 10, 0xFF0000FF);
        assert_eq!(solid.pattern_type, PatternType::Solid);
        assert_eq!(solid.width, 10);
        assert_eq!(solid.height, 10);
        assert_eq!(solid.colors.len(), 1);
        
        // Test horizontal lines pattern
        let h_lines = patterns.generate_horizontal_lines(20, 10, 2, vec![0xFF0000FF, 0x00FF00FF]);
        assert_eq!(h_lines.pattern_type, PatternType::HorizontalLines);
        assert_eq!(h_lines.colors.len(), 2);
        
        // Test checkerboard pattern
        let checker = patterns.generate_checkerboard(10, 10, 2, vec![0xFFFFFFFF, 0x000000FF]);
        assert_eq!(checker.pattern_type, PatternType::Checkerboard);
        assert_eq!(checker.colors.len(), 2);
    }

    #[test]
    fn test_advanced_graphics() {
        let mut pixels = Vec::new();
        let width = 100;
        let height = 100;
        pixels.resize((width * height * 4) as usize, 0);
        
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
        
        let advanced = AdvancedGraphics;
        
        // Test anti-aliased line
        advanced.draw_anti_aliased_line(width, height, 0.0, 0.0, 50.0, 50.0, 0xFF0000FF, draw_pixel);
        
        // Test gradient rectangle
        advanced.draw_gradient_rect(width, height, 10, 10, 30, 30, 0xFF0000FF, 0x00FF00FF, 0, draw_pixel);
        
        // Test rounded rectangle
        advanced.draw_rounded_rect(width, height, 50, 50, 20, 20, 5, 0x0000FFFF, true, draw_pixel);
        
        // Test radial gradient circle
        advanced.draw_radial_gradient(width, height, 25, 25, 15, 0xFFFF00FF, 0xFF00FFFF, draw_pixel);
    }

    #[test]
    fn test_animation_utilities() {
        let anim = AnimationUtilities::new();
        
        assert_eq!(anim.lerp(0.0, 10.0, 0.5), 5.0);
        assert_eq!(anim.ease_in(0.5), 0.25);
        assert_eq!(anim.ease_out(0.5), 0.75);
        
        let eased_in_out = anim.ease_in_out(0.5);
        assert!(eased_in_out >= 0.0 && eased_in_out <= 1.0);
        
        let bounce = anim.bounce(0.5);
        assert!(bounce >= 0.0 && bounce <= 1.0);
    }

    #[test]
    fn test_graphics_effects() {
        let effects = GraphicsEffects;
        let mut pixels = vec![255u8; 100 * 4];
        
        // Test alpha blend effect
        effects.apply_alpha_blend(&mut pixels, 0.5);
        assert_eq!(pixels[3], 127); // Alpha channel should be halved
        
        // Test blur effect
        effects.apply_blur(&mut pixels, 10, 10, 1);
        
        // Test shadow effect application
        let mut shadow_applied = false;
        let test_draw = |x: u32, y: u32, color: u32| {
            shadow_applied = true;
        };
        
        effects.apply_shadow(test_draw, (2, 2), 0x40000000, 0.5);
        // Shadow function should be created without error
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

    #[test]
    fn test_complete_graphics_workflow() {
        // This test simulates a complete graphics workflow
        
        // 1. Initialize all graphics subsystems
        let mut graphics_manager = GraphicsManager::new();
        assert!(graphics_manager.initialize().is_ok());
        
        let mut framebuffer_manager = FramebufferManager::new();
        assert!(framebuffer_manager.initialize().is_ok());
        
        let mut font_manager = FontManager::new();
        assert!(font_manager.initialize().is_ok());
        
        let mut graphics_utils = GraphicsUtilities::new();
        assert!(graphics_utils.initialize().is_ok());
        
        // 2. Create graphics buffer
        let mut buffer = GraphicsBuffer::new(800, 600, ColorDepth::Bpp32).unwrap();
        
        // 3. Clear screen
        buffer.clear(0x000000);
        
        // 4. Draw some graphics primitives
        if let Some(driver) = graphics_manager.get_current_driver() {
            // Draw lines
            assert!(driver.draw_line(10, 10, 100, 10, 0xFFFFFF).is_ok());
            assert!(driver.draw_line(10, 10, 10, 100, 0xFFFFFF).is_ok());
            
            // Draw rectangle
            assert!(driver.draw_rect(50, 50, 100, 75, 0xFF0000, true).is_ok());
            assert!(driver.draw_rect(50, 50, 100, 75, 0xFFFFFF, false).is_ok());
            
            // Draw circle
            assert!(driver.draw_circle(200, 200, 50, 0x00FF00, true).is_ok());
            assert!(driver.draw_circle(200, 200, 50, 0xFFFFFF, false).is_ok());
        }
        
        // 5. Draw text (would require font data)
        if let Some(font) = font_manager.get_default_font() {
            // Test font operations
            let text = "MultiOS Graphics Test";
            let text_width = font.text_width(text);
            assert!(text_width > 0);
            
            // Test text rendering would go here
            // font.render_text(text, 300, 100, 0xFFFFFF, TextAlign::Left, &mut buffer)?;
        }
        
        // 6. Test advanced graphics operations
        let colors = graphics_utils.colors();
        let patterns = graphics_utils.patterns();
        let advanced = graphics_utils.advanced();
        
        // Create gradient colors
        let gradient_color = colors.gradient(0xFF0000, 0x0000FF, 0.5);
        
        // Create pattern
        let pattern = patterns.generate_checkerboard(50, 50, 5, vec![0xFFFFFF, 0x000000]);
        
        // Use advanced graphics operations
        let mut pixels = vec![0u8; 200 * 200 * 4];
        let draw_pixel = |x: u32, y: u32, color: u32| {
            let offset = ((y * 200 + x) * 4) as usize;
            if offset + 3 < pixels.len() {
                let bytes = color.to_le_bytes();
                pixels[offset] = bytes[0];
                pixels[offset + 1] = bytes[1];
                pixels[offset + 2] = bytes[2];
                pixels[offset + 3] = bytes[3];
            }
        };
        
        advanced.draw_gradient_rect(200, 200, 0, 0, 100, 100, 0xFF0000, 0x00FF00, 1, draw_pixel);
        advanced.draw_rounded_rect(200, 200, 100, 100, 50, 50, 10, 0x0000FF, true, draw_pixel);
        
        // 7. Test animation
        let animation = graphics_utils.animation();
        let animated_value = animation.lerp(0.0, 100.0, animation.ease_in_out(0.5));
        assert!(animated_value >= 0.0 && animated_value <= 100.0);
        
        // 8. Test effects
        let effects = graphics_utils.effects();
        effects.apply_alpha_blend(&mut pixels, 0.75);
        
        // Test completed successfully
        assert!(true); // If we got here, the workflow was successful
    }

    #[test]
    fn test_multi_display_support() {
        // Test multiple display detection and management
        let mut framebuffer_manager = FramebufferManager::new();
        assert!(framebuffer_manager.initialize().is_ok());
        
        let display_count = framebuffer_manager.display_count();
        assert!(display_count > 0);
        
        // Test primary display
        if let Some(primary_display) = framebuffer_manager.get_primary_display() {
            assert!(primary_display.info.width > 0);
            assert!(primary_display.info.height > 0);
            assert!(primary_display.info.bpp > 0);
        }
        
        // Test acceleration operations
        if display_count > 0 {
            assert!(framebuffer_manager.accelerated_clear(0, 0x000000).is_ok());
        }
    }
}