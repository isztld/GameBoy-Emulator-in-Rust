/// Frame Buffer for GameBoy LCD display
///
/// Handles the 160x144 pixel frame buffer used for display output.

use std::sync::{Arc, Mutex};

/// GameBoy screen dimensions
pub const SCREEN_WIDTH: usize = 160;
pub const SCREEN_HEIGHT: usize = 144;

/// Frame buffer size in pixels
pub const FRAME_BUFFER_SIZE: usize = SCREEN_WIDTH * SCREEN_HEIGHT;

/// Frame buffer containing RGBA pixel data
#[derive(Debug, Clone)]
pub struct FrameBuffer {
    /// RGBA pixel data (4 bytes per pixel)
    pub pixels: [u32; FRAME_BUFFER_SIZE],
    /// Whether a new frame is ready
    pub frame_ready: bool,
}

impl FrameBuffer {
    pub fn new() -> Self {
        FrameBuffer {
            pixels: [0u32; FRAME_BUFFER_SIZE],
            frame_ready: false,
        }
    }

    /// Clear the frame buffer to black
    pub fn clear(&mut self) {
        self.pixels.fill(0);
        self.frame_ready = false;
    }

    /// Set a single pixel at (x, y) with color value (0-3 for GameBoy palette)
    pub fn set_pixel(&mut self, x: usize, y: usize, color: u8) {
        if x < SCREEN_WIDTH && y < SCREEN_HEIGHT {
            let idx = y * SCREEN_WIDTH + x;
            self.pixels[idx] = color_to_rgba(color);
        }
    }

    /// Get a reference to the pixel array
    pub fn get_pixels(&self) -> &[u32; FRAME_BUFFER_SIZE] {
        &self.pixels
    }

    /// Mark frame as ready for display
    pub fn mark_frame_ready(&mut self) {
        self.frame_ready = true;
    }

    /// Clear frame ready flag
    pub fn clear_frame_ready(&mut self) {
        self.frame_ready = false;
    }
}

impl Default for FrameBuffer {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert GameBoy color value (0-3) to RGBA
/// GameBoy DMG uses 4 shades of gray:
/// 0 = white, 1 = light gray, 2 = dark gray, 3 = black
#[inline]
pub fn color_to_rgba(color: u8) -> u32 {
    match color {
        0 => 0xFFFFFFFF, // White
        1 => 0xFFB4B4B4, // Light gray
        2 => 0xFF686868, // Dark gray
        3 => 0xFF000000, // Black
        _ => 0xFF000000,
    }
}

/// Shared frame buffer wrapped in Arc<Mutex<>> for thread-safe access
pub type SharedFrameBuffer = Arc<Mutex<FrameBuffer>>;

/// Create a new shared frame buffer
pub fn create_shared_frame_buffer() -> SharedFrameBuffer {
    Arc::new(Mutex::new(FrameBuffer::new()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_buffer_create() {
        let fb = FrameBuffer::new();
        assert_eq!(fb.pixels.len(), FRAME_BUFFER_SIZE);
        assert!(!fb.frame_ready);
    }

    #[test]
    fn test_set_pixel() {
        let mut fb = FrameBuffer::new();
        fb.set_pixel(10, 10, 0); // White
        assert_eq!(fb.pixels[10 * 160 + 10], 0xFFFFFFFF);
    }

    #[test]
    fn test_color_conversion() {
        assert_eq!(color_to_rgba(0), 0xFFFFFFFF); // White
        assert_eq!(color_to_rgba(1), 0xFFB4B4B4); // Light gray
        assert_eq!(color_to_rgba(2), 0xFF686868); // Dark gray
        assert_eq!(color_to_rgba(3), 0xFF000000); // Black
    }

    #[test]
    fn test_pixel_bounds() {
        let mut fb = FrameBuffer::new();
        // Out of bounds should not panic
        fb.set_pixel(200, 200, 0);
        fb.set_pixel(160, 144, 0);
    }
}
