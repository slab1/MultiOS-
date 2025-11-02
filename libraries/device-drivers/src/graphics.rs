//! Graphics Display Drivers
//! 
//! Support for VGA, VESA, and UEFI GOP graphics modes with primitive drawing operations.

use crate::{DeviceType, DriverResult, DriverError, Device, DeviceHandle, DeviceInfo, HardwareAddress, BusHandle, BusType, DeviceCapabilities, DeviceState, DeviceDriver};
use crate::device::DeviceCapability;
use spin::{Mutex, RwLock};
use alloc::{vec::Vec, collections::BTreeMap};
use log::{info, warn, error};

/// Graphics device types
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

/// Graphics color depth
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ColorDepth {
    Unknown = 0,
    Bpp8 = 1,
    Bpp16 = 2,
    Bpp24 = 3,
    Bpp32 = 4,
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
}

/// Graphics buffer for drawing operations
#[derive(Clone)]
pub struct GraphicsBuffer {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub pitch: u32,
    pub color_depth: ColorDepth,
}

/// Graphics primitive operations
pub trait GraphicsPrimitive {
    /// Clear screen with specified color
    fn clear(&self, color: u32) -> DriverResult<()>;
    
    /// Draw pixel at coordinates
    fn draw_pixel(&self, x: u32, y: u32, color: u32) -> DriverResult<()>;
    
    /// Get pixel color at coordinates
    fn get_pixel(&self, x: u32, y: u32) -> DriverResult<u32>;
    
    /// Draw horizontal line
    fn draw_line_h(&self, x1: u32, y: u32, x2: u32, color: u32) -> DriverResult<()>;
    
    /// Draw vertical line
    fn draw_line_v(&self, x: u32, y1: u32, y2: u32, color: u32) -> DriverResult<()>;
    
    /// Draw rectangle
    fn draw_rect(&self, x: u32, y: u32, width: u32, height: u32, color: u32, filled: bool) -> DriverResult<()>;
    
    /// Draw circle
    fn draw_circle(&self, cx: u32, cy: u32, radius: u32, color: u32, filled: bool) -> DriverResult<()>;
    
    /// Draw text using bitmap font
    fn draw_text(&self, x: u32, y: u32, text: &str, color: u32, font: Option<&BitmapFont>) -> DriverResult<()>;
    
    /// Blit buffer to screen
    fn blit(&self, buffer: &GraphicsBuffer, x: u32, y: u32) -> DriverResult<()>;
    
    /// Blit buffer with scaling
    fn blit_scaled(&self, buffer: &GraphicsBuffer, x: u32, y: u32, scale: f32) -> DriverResult<()>;
}

/// Bitmap font for text rendering
#[derive(Debug, Clone)]
pub struct BitmapFont {
    pub width: u32,
    pub height: u32,
    pub data: &'static [u8], // Font bitmap data
}

/// Common colors
pub const COLORS: &[u32] = &[
    0xFF000000, // Black
    0xFFFFFFFF, // White
    0xFF0000FF, // Red
    0xFF00FF00, // Green
    0xFFFF0000, // Blue
    0xFFFFFF00, // Yellow
    0xFFFF00FF, // Magenta
    0xFF00FFFF, // Cyan
    0xFF808080, // Gray
    0xFF800000, // Dark Red
    0xFF008000, // Dark Green
    0xFF000080, // Dark Blue
];

/// VGA Graphics Driver
pub struct VgaGraphics {
    io_base: u16,
    vram_base: u64,
    current_mode: GraphicsModeInfo,
    buffer: Option<GraphicsBuffer>,
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
            },
            buffer: None,
        }
    }
    
    /// Initialize VGA graphics mode
    pub fn init_mode(&mut self) -> DriverResult<()> {
        info!("Initializing VGA graphics mode 0x13");
        
        // Set VGA mode 0x13 (320x200x256)
        self.set_mode(0x13)?;
        
        // Configure palette
        self.set_palette()?;
        
        info!("VGA graphics initialized: {}x{}x{}", 
              self.current_mode.width, 
              self.current_mode.height, 
              self.current_mode.color_depth as u8 * 8);
        
        Ok(())
    }
    
    /// Set graphics mode
    fn set_mode(&self, mode: u16) -> DriverResult<()> {
        // VGA mode set - typically via BIOS interrupt 0x10
        // In a real implementation, this would call BIOS services
        
        info!("Setting VGA mode: 0x{:X}", mode);
        Ok(())
    }
    
    /// Configure VGA palette
    fn set_palette(&self) -> DriverResult<()> {
        // Set color palette for standard VGA colors
        info!("Configuring VGA palette");
        Ok(())
    }
    
    /// Draw pixel in VGA mode
    fn write_pixel_vga(&self, x: u32, y: u32, color: u8) -> DriverResult<()> {
        if x >= self.current_mode.width || y >= self.current_mode.height {
            return Err(DriverError::DeviceNotFound);
        }
        
        let offset = (y * self.current_mode.pitch + x) as usize;
        
        // Write pixel to VGA memory
        unsafe {
            let vram_ptr = self.vram_base as *mut u8;
            core::ptr::write(vram_ptr.add(offset), color);
        }
        
        Ok(())
    }
    
    /// Read pixel from VGA memory
    fn read_pixel_vga(&self, x: u32, y: u32) -> DriverResult<u8> {
        if x >= self.current_mode.width || y >= self.current_mode.height {
            return Err(DriverError::DeviceNotFound);
        }
        
        let offset = (y * self.current_mode.pitch + x) as usize;
        
        unsafe {
            let vram_ptr = self.vram_base as *const u8;
            Ok(core::ptr::read(vram_ptr.add(offset)))
        }
    }
}

impl DeviceDriver for VgaGraphics {
    fn name(&self) -> &'static str {
        "VGA Graphics Driver"
    }
    
    fn supported_devices(&self) -> &[DeviceType] {
        &[DeviceType::Display]
    }
    
    fn init(&self, _device: &Device) -> DriverResult<()> {
        info!("Initializing VGA graphics driver");
        Ok(())
    }
    
    fn remove(&self, _device: &Device) -> DriverResult<()> {
        info!("Removing VGA graphics driver");
        Ok(())
    }
    
    fn read(&self, _device: &Device, _buffer: &mut [u8]) -> DriverResult<usize> {
        Ok(0)
    }
    
    fn write(&self, _device: &Device, _buffer: &[u8]) -> DriverResult<usize> {
        Ok(0)
    }
    
    fn ioctl(&self, _device: &Device, command: u32, _data: usize) -> DriverResult<usize> {
        // Handle graphics-specific ioctl commands
        match command {
            // Add graphics-specific commands here
            _ => Ok(0),
        }
    }
    
    fn capabilities(&self) -> DeviceCapabilities {
        DeviceCapabilities::WRITE | DeviceCapabilities::INTERRUPT
    }
}

impl GraphicsPrimitive for VgaGraphics {
    fn clear(&self, color: u32) -> DriverResult<()> {
        for y in 0..self.current_mode.height {
            for x in 0..self.current_mode.width {
                self.write_pixel_vga(x, y, color as u8)?;
            }
        }
        Ok(())
    }
    
    fn draw_pixel(&self, x: u32, y: u32, color: u32) -> DriverResult<()> {
        self.write_pixel_vga(x, y, color as u8)
    }
    
    fn get_pixel(&self, x: u32, y: u32) -> DriverResult<u32> {
        Ok(self.read_pixel_vga(x, y)? as u32)
    }
    
    fn draw_line_h(&self, x1: u32, y: u32, x2: u32, color: u32) -> DriverResult<()> {
        let (start, end) = if x1 <= x2 { (x1, x2) } else { (x2, x1) };
        
        for x in start..=end {
            self.write_pixel_vga(x, y, color as u8)?;
        }
        Ok(())
    }
    
    fn draw_line_v(&self, x: u32, y1: u32, y2: u32, color: u32) -> DriverResult<()> {
        let (start, end) = if y1 <= y2 { (y1, y2) } else { (y2, y1) };
        
        for y in start..=end {
            self.write_pixel_vga(x, y, color as u8)?;
        }
        Ok(())
    }
    
    fn draw_rect(&self, x: u32, y: u32, width: u32, height: u32, color: u32, filled: bool) -> DriverResult<()> {
        if filled {
            for py in y..(y + height) {
                for px in x..(x + width) {
                    self.write_pixel_vga(px, py, color as u8)?;
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
    
    fn draw_circle(&self, cx: u32, cy: u32, radius: u32, color: u32, filled: bool) -> DriverResult<()> {
        let r_squared = radius * radius;
        
        for y in 0..radius {
            for x in 0..radius {
                let dist_squared = x * x + y * y;
                
                if dist_squared <= r_squared {
                    if filled {
                        // Draw filled circle
                        for px in (cx.saturating_sub(x))..(cx + radius) {
                            for py in (cy.saturating_sub(y))..(cy + radius) {
                                self.write_pixel_vga(px, py, color as u8)?;
                            }
                        }
                    } else {
                        // Draw circle outline
                        let px = cx + x;
                        let py = cy + y;
                        self.write_pixel_vga(px, py, color as u8)?;
                        self.write_pixel_vga(cx + x, cy - y, color as u8)?;
                        self.write_pixel_vga(cx - x, cy + y, color as u8)?;
                        self.write_pixel_vga(cx - x, cy - y, color as u8)?;
                    }
                }
            }
        }
        Ok(())
    }
    
    fn draw_text(&self, _x: u32, _y: u32, _text: &str, _color: u32, _font: Option<&BitmapFont>) -> DriverResult<()> {
        // Text rendering would require bitmap font implementation
        Ok(())
    }
    
    fn blit(&self, _buffer: &GraphicsBuffer, _x: u32, _y: u32) -> DriverResult<()> {
        // Blit operation for VGA
        Ok(())
    }
    
    fn blit_scaled(&self, _buffer: &GraphicsBuffer, _x: u32, _y: u32, _scale: f32) -> DriverResult<()> {
        Ok(())
    }
}

/// VESA Graphics Driver
pub struct VesaGraphics {
    framebuffer_addr: u64,
    current_mode: GraphicsModeInfo,
    available_modes: Vec<GraphicsModeInfo>,
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
            },
            available_modes: Vec::new(),
        }
    }
    
    /// Initialize VESA graphics mode
    pub fn init(&mut self) -> DriverResult<()> {
        info!("Initializing VESA graphics");
        
        // Query available VESA modes
        self.query_modes()?;
        
        // Set optimal mode
        self.set_mode(0x0118)?; // 1024x768x32
        
        info!("VESA graphics initialized: {}x{}x{}", 
              self.current_mode.width, 
              self.current_mode.height, 
              self.current_mode.color_depth as u8 * 8);
        
        Ok(())
    }
    
    /// Query available VESA modes
    fn query_modes(&mut self) -> DriverResult<()> {
        // In real implementation, this would query VESA BIOS
        info!("Querying VESA modes");
        
        // Add common VESA modes
        self.available_modes.push(GraphicsModeInfo {
            width: 640, height: 480, color_depth: ColorDepth::Bpp32,
            pitch: 2560, framebuffer_addr: self.framebuffer_addr,
            mode_number: 0x0111, attributes: 0x00,
        });
        
        self.available_modes.push(GraphicsModeInfo {
            width: 800, height: 600, color_depth: ColorDepth::Bpp32,
            pitch: 3200, framebuffer_addr: self.framebuffer_addr,
            mode_number: 0x0114, attributes: 0x00,
        });
        
        self.available_modes.push(GraphicsModeInfo {
            width: 1024, height: 768, color_depth: ColorDepth::Bpp32,
            pitch: 4096, framebuffer_addr: self.framebuffer_addr,
            mode_number: 0x0118, attributes: 0x00,
        });
        
        info!("Found {} VESA modes", self.available_modes.len());
        Ok(())
    }
    
    /// Set VESA mode
    fn set_mode(&mut self, mode_number: u16) -> DriverResult<()> {
        // Find mode info
        if let Some(mode_info) = self.available_modes.iter().find(|m| m.mode_number == mode_number) {
            self.current_mode = mode_info.clone();
            info!("Set VESA mode 0x{:X}: {}x{}x{}", 
                  mode_number, mode_info.width, mode_info.height, 
                  mode_info.color_depth as u8 * 8);
        } else {
            return Err(DriverError::DeviceNotFound);
        }
        Ok(())
    }
    
    /// Write pixel to framebuffer
    fn write_pixel_vesa(&self, x: u32, y: u32, color: u32) -> DriverResult<()> {
        if x >= self.current_mode.width || y >= self.current_mode.height {
            return Err(DriverError::DeviceNotFound);
        }
        
        let offset = (y * self.current_mode.pitch + x * 4) as usize;
        
        unsafe {
            let fb_ptr = self.framebuffer_addr as *mut u8;
            let color_bytes = color.to_le_bytes();
            core::ptr::copy_nonoverlapping(color_bytes.as_ptr(), fb_ptr.add(offset), 4);
        }
        
        Ok(())
    }
    
    /// Read pixel from framebuffer
    fn read_pixel_vesa(&self, x: u32, y: u32) -> DriverResult<u32> {
        if x >= self.current_mode.width || y >= self.current_mode.height {
            return Err(DriverError::DeviceNotFound);
        }
        
        let offset = (y * self.current_mode.pitch + x * 4) as usize;
        
        unsafe {
            let fb_ptr = self.framebuffer_addr as *const u8;
            let mut color_bytes = [0u8; 4];
            core::ptr::copy_nonoverlapping(fb_ptr.add(offset), color_bytes.as_mut_ptr(), 4);
            Ok(u32::from_le_bytes(color_bytes))
        }
    }
}

impl DeviceDriver for VesaGraphics {
    fn name(&self) -> &'static str {
        "VESA Graphics Driver"
    }
    
    fn supported_devices(&self) -> &[DeviceType] {
        &[DeviceType::Display]
    }
    
    fn init(&self, _device: &Device) -> DriverResult<()> {
        info!("Initializing VESA graphics driver");
        Ok(())
    }
    
    fn remove(&self, _device: &Device) -> DriverResult<()> {
        info!("Removing VESA graphics driver");
        Ok(())
    }
    
    fn read(&self, _device: &Device, _buffer: &mut [u8]) -> DriverResult<usize> {
        Ok(0)
    }
    
    fn write(&self, _device: &Device, _buffer: &[u8]) -> DriverResult<usize> {
        Ok(0)
    }
    
    fn ioctl(&self, _device: &Device, _command: u32, _data: usize) -> DriverResult<usize> {
        Ok(0)
    }
    
    fn capabilities(&self) -> DeviceCapabilities {
        DeviceCapabilities::WRITE | DeviceCapabilities::INTERRUPT
    }
}

impl GraphicsPrimitive for VesaGraphics {
    fn clear(&self, color: u32) -> DriverResult<()> {
        let buffer_size = (self.current_mode.height * self.current_mode.pitch) as usize;
        let color_bytes = color.to_le_bytes();
        
        unsafe {
            let fb_ptr = self.framebuffer_addr as *mut u8;
            for i in (0..buffer_size).step_by(4) {
                core::ptr::copy_nonoverlapping(color_bytes.as_ptr(), fb_ptr.add(i), 4);
            }
        }
        Ok(())
    }
    
    fn draw_pixel(&self, x: u32, y: u32, color: u32) -> DriverResult<()> {
        self.write_pixel_vesa(x, y, color)
    }
    
    fn get_pixel(&self, x: u32, y: u32) -> DriverResult<u32> {
        self.read_pixel_vesa(x, y)
    }
    
    fn draw_line_h(&self, x1: u32, y: u32, x2: u32, color: u32) -> DriverResult<()> {
        let (start, end) = if x1 <= x2 { (x1, x2) } else { (x2, x1) };
        
        for x in start..=end {
            self.write_pixel_vesa(x, y, color)?;
        }
        Ok(())
    }
    
    fn draw_line_v(&self, x: u32, y1: u32, y2: u32, color: u32) -> DriverResult<()> {
        let (start, end) = if y1 <= y2 { (y1, y2) } else { (y2, y1) };
        
        for y in start..=end {
            self.write_pixel_vesa(x, y, color)?;
        }
        Ok(())
    }
    
    fn draw_rect(&self, x: u32, y: u32, width: u32, height: u32, color: u32, filled: bool) -> DriverResult<()> {
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
    
    fn draw_circle(&self, cx: u32, cy: u32, radius: u32, color: u32, filled: bool) -> DriverResult<()> {
        let r_squared = radius * radius;
        
        for y in 0..radius {
            for x in 0..radius {
                let dist_squared = x * x + y * y;
                
                if dist_squared <= r_squared {
                    if filled {
                        for px in (cx.saturating_sub(x))..(cx + radius) {
                            for py in (cy.saturating_sub(y))..(cy + radius) {
                                self.write_pixel_vesa(px, py, color)?;
                            }
                        }
                    } else {
                        let px = cx + x;
                        let py = cy + y;
                        self.write_pixel_vesa(px, py, color)?;
                        self.write_pixel_vesa(cx + x, cy - y, color)?;
                        self.write_pixel_vesa(cx - x, cy + y, color)?;
                        self.write_pixel_vesa(cx - x, cy - y, color)?;
                    }
                }
            }
        }
        Ok(())
    }
    
    fn draw_text(&self, _x: u32, _y: u32, _text: &str, _color: u32, _font: Option<&BitmapFont>) -> DriverResult<()> {
        Ok(())
    }
    
    fn blit(&self, buffer: &GraphicsBuffer, x: u32, y: u32) -> DriverResult<()> {
        // Simple blit without scaling
        for by in 0..buffer.height {
            for bx in 0..buffer.width {
                let src_offset = (by * buffer.pitch + bx * 4) as usize;
                let dst_x = x + bx;
                let dst_y = y + by;
                
                if dst_x < self.current_mode.width && dst_y < self.current_mode.height {
                    // Extract pixel from source buffer (assuming RGBA)
                    let pixel_data = &buffer.data[src_offset..src_offset + 4];
                    let color = u32::from_le_bytes([pixel_data[0], pixel_data[1], pixel_data[2], pixel_data[3]]);
                    
                    self.write_pixel_vesa(dst_x, dst_y, color)?;
                }
            }
        }
        Ok(())
    }
    
    fn blit_scaled(&self, _buffer: &GraphicsBuffer, _x: u32, _y: u32, _scale: f32) -> DriverResult<()> {
        // Scaled blit would implement interpolation
        Ok(())
    }
}

/// UEFI GOP Graphics Driver
pub struct UefiGopGraphics {
    framebuffer_addr: u64,
    framebuffer_size: usize,
    current_mode: GraphicsModeInfo,
}

impl UefiGopGraphics {
    /// Create new UEFI GOP graphics driver
    pub fn new(framebuffer_addr: u64, framebuffer_size: usize) -> Self {
        Self {
            framebuffer_addr,
            framebuffer_size,
            current_mode: GraphicsModeInfo {
                width: 1920,
                height: 1080,
                color_depth: ColorDepth::Bpp32,
                pitch: 7680,
                framebuffer_addr,
                mode_number: 0,
                attributes: 0x00,
            },
        }
    }
    
    /// Initialize GOP graphics
    pub fn init(&mut self) -> DriverResult<()> {
        info!("Initializing UEFI GOP graphics");
        
        // Query current GOP mode
        // In real implementation, this would use UEFI GOP protocol
        
        info!("UEFI GOP initialized: {}x{}x{}", 
              self.current_mode.width, 
              self.current_mode.height, 
              self.current_mode.color_depth as u8 * 8);
        
        Ok(())
    }
}

impl DeviceDriver for UefiGopGraphics {
    fn name(&self) -> &'static str {
        "UEFI GOP Graphics Driver"
    }
    
    fn supported_devices(&self) -> &[DeviceType] {
        &[DeviceType::Display]
    }
    
    fn init(&self, _device: &Device) -> DriverResult<()> {
        info!("Initializing UEFI GOP graphics driver");
        Ok(())
    }
    
    fn remove(&self, _device: &Device) -> DriverResult<()> {
        info!("Removing UEFI GOP graphics driver");
        Ok(())
    }
    
    fn read(&self, _device: &Device, _buffer: &mut [u8]) -> DriverResult<usize> {
        Ok(0)
    }
    
    fn write(&self, _device: &Device, _buffer: &[u8]) -> DriverResult<usize> {
        Ok(0)
    }
    
    fn ioctl(&self, _device: &Device, _command: u32, _data: usize) -> DriverResult<usize> {
        Ok(0)
    }
    
    fn capabilities(&self) -> DeviceCapabilities {
        DeviceCapabilities::WRITE | DeviceCapabilities::INTERRUPT
    }
}

impl GraphicsPrimitive for UefiGopGraphics {
    fn clear(&self, color: u32) -> DriverResult<()> {
        let color_bytes = color.to_le_bytes();
        
        unsafe {
            let fb_ptr = self.framebuffer_addr as *mut u8;
            for i in (0..self.framebuffer_size).step_by(4) {
                core::ptr::copy_nonoverlapping(color_bytes.as_ptr(), fb_ptr.add(i), 4);
            }
        }
        Ok(())
    }
    
    fn draw_pixel(&self, _x: u32, _y: u32, _color: u32) -> DriverResult<()> {
        Ok(())
    }
    
    fn get_pixel(&self, _x: u32, _y: u32) -> DriverResult<u32> {
        Ok(0)
    }
    
    fn draw_line_h(&self, _x1: u32, _y: u32, _x2: u32, _color: u32) -> DriverResult<()> {
        Ok(())
    }
    
    fn draw_line_v(&self, _x: u32, _y1: u32, _y2: u32, _color: u32) -> DriverResult<()> {
        Ok(())
    }
    
    fn draw_rect(&self, _x: u32, _y: u32, _width: u32, _height: u32, _color: u32, _filled: bool) -> DriverResult<()> {
        Ok(())
    }
    
    fn draw_circle(&self, _cx: u32, _cy: u32, _radius: u32, _color: u32, _filled: bool) -> DriverResult<()> {
        Ok(())
    }
    
    fn draw_text(&self, _x: u32, _y: u32, _text: &str, _color: u32, _font: Option<&BitmapFont>) -> DriverResult<()> {
        Ok(())
    }
    
    fn blit(&self, _buffer: &GraphicsBuffer, _x: u32, _y: u32) -> DriverResult<()> {
        Ok(())
    }
    
    fn blit_scaled(&self, _buffer: &GraphicsBuffer, _x: u32, _y: u32, _scale: f32) -> DriverResult<()> {
        Ok(())
    }
}

/// Graphics driver manager
pub struct GraphicsDriverManager {
    vga_driver: Option<VgaGraphics>,
    vesa_driver: Option<VesaGraphics>,
    uefi_gop_driver: Option<UefiGopGraphics>,
    current_driver: Option<&'static dyn GraphicsPrimitive + Send + Sync>,
}

impl GraphicsDriverManager {
    /// Create new graphics driver manager
    pub fn new() -> Self {
        Self {
            vga_driver: None,
            vesa_driver: None,
            uefi_gop_driver: None,
            current_driver: None,
        }
    }
    
    /// Register VGA driver
    pub fn register_vga(&mut self, io_base: u16, vram_base: u64) -> DriverResult<()> {
        let driver = VgaGraphics::new(io_base, vram_base);
        self.vga_driver = Some(driver);
        info!("VGA graphics driver registered");
        Ok(())
    }
    
    /// Register VESA driver
    pub fn register_vesa(&mut self, framebuffer_addr: u64) -> DriverResult<()> {
        let mut driver = VesaGraphics::new(framebuffer_addr);
        driver.init()?;
        self.vesa_driver = Some(driver);
        info!("VESA graphics driver registered");
        Ok(())
    }
    
    /// Register UEFI GOP driver
    pub fn register_uefi_gop(&mut self, framebuffer_addr: u64, framebuffer_size: usize) -> DriverResult<()> {
        let mut driver = UefiGopGraphics::new(framebuffer_addr, framebuffer_size);
        driver.init()?;
        self.uefi_gop_driver = Some(driver);
        info!("UEFI GOP graphics driver registered");
        Ok(())
    }
    
    /// Set current graphics driver
    pub fn set_current_driver(&mut self, driver_type: GraphicsMode) -> DriverResult<()> {
        match driver_type {
            GraphicsMode::Vga => {
                if let Some(ref driver) = self.vga_driver {
                    self.current_driver = Some(driver);
                    info!("Set VGA as current graphics driver");
                } else {
                    return Err(DriverError::DeviceNotFound);
                }
            }
            GraphicsMode::Vesa => {
                if let Some(ref driver) = self.vesa_driver {
                    self.current_driver = Some(driver);
                    info!("Set VESA as current graphics driver");
                } else {
                    return Err(DriverError::DeviceNotFound);
                }
            }
            GraphicsMode::UefiGop => {
                if let Some(ref driver) = self.uefi_gop_driver {
                    self.current_driver = Some(driver);
                    info!("Set UEFI GOP as current graphics driver");
                } else {
                    return Err(DriverError::DeviceNotFound);
                }
            }
            _ => return Err(DriverError::DeviceNotFound),
        }
        Ok(())
    }
    
    /// Get current graphics driver
    pub fn get_current_driver(&self) -> Option<&dyn GraphicsPrimitive + Send + Sync> {
        self.current_driver.map(|driver| *driver)
    }
    
    /// Clear screen
    pub fn clear(&self, color: u32) -> DriverResult<()> {
        if let Some(driver) = self.current_driver {
            driver.clear(color)
        } else {
            Err(DriverError::DeviceNotFound)
        }
    }
    
    /// Draw pixel
    pub fn draw_pixel(&self, x: u32, y: u32, color: u32) -> DriverResult<()> {
        if let Some(driver) = self.current_driver {
            driver.draw_pixel(x, y, color)
        } else {
            Err(DriverError::DeviceNotFound)
        }
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
        };
        
        assert_eq!(mode.width, 1024);
        assert_eq!(mode.height, 768);
    }

    #[test]
    fn test_graphics_buffer_creation() {
        let buffer = GraphicsBuffer {
            data: vec![0u8; 320 * 240 * 4],
            width: 320,
            height: 240,
            pitch: 1280,
            color_depth: ColorDepth::Bpp32,
        };
        
        assert_eq!(buffer.width, 320);
        assert_eq!(buffer.height, 240);
    }

    #[test]
    fn test_graphics_driver_manager() {
        let mut manager = GraphicsDriverManager::new();
        
        // Register VGA driver
        assert!(manager.register_vga(0x3CE, 0xA0000).is_ok());
        assert!(manager.vga_driver.is_some());
        
        // Set current driver
        assert!(manager.set_current_driver(GraphicsMode::Vga).is_ok());
        assert!(manager.get_current_driver().is_some());
    }
}
