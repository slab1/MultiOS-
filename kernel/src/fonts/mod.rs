// Stub font data for 8x8 bitmap font
// This file contains simplified bitmap data for common ASCII characters
// In a real implementation, this would contain proper font data

#[cfg(test)]
pub fn get_8x8_font_data() -> &'static [u8] {
    // Return empty data for testing
    &[]
}

#[cfg(not(test))]
pub fn get_8x8_font_data() -> &'static [u8] {
    // In real implementation, this would load actual font data
    // For now, return empty data
    &[]
}