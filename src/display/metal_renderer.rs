/// Metal renderer for GameBoy LCD display
///
/// Handles Metal device setup and frame buffer management.
/// For a complete window implementation, integrate with winit or similar.

use metal::{Device, CommandQueue};

use super::frame_buffer::{FrameBuffer, SCREEN_WIDTH, SCREEN_HEIGHT};

/// Metal renderer for displaying the GameBoy frame buffer
pub struct MetalRenderer {
    /// Metal device
    device: Device,
    /// Command queue for rendering commands
    command_queue: CommandQueue,
    /// Width of the display in pixels
    width: u32,
    /// Height of the display in pixels
    height: u32,
    /// Scale factor for display
    scale: u32,
}

impl MetalRenderer {
    /// Create a new Metal renderer
    pub fn new() -> Option<Self> {
        // Get the default Metal device
        let device = Device::system_default()?;

        // Create a command queue
        let command_queue = device.new_command_queue();

        // Scale factor for better visibility (3x = 480x432)
        let scale = 3;

        Some(MetalRenderer {
            device,
            command_queue,
            width: SCREEN_WIDTH as u32 * scale,
            height: SCREEN_HEIGHT as u32 * scale,
            scale,
        })
    }

    /// Get the scale factor
    pub fn scale(&self) -> u32 {
        self.scale
    }

    /// Update texture with frame buffer data
    /// This updates the internal representation of the frame.
    /// For actual display, integrate with a windowing library.
    pub fn update_texture(&self, frame_buffer: &FrameBuffer) {
        // Process the frame buffer
        // In a full implementation, this would:
        // 1. Create a CAMetalLayer for the window
        // 2. Copy frame buffer data to a Metal texture
        // 3. Render the texture to the screen

        let _pixels = frame_buffer.get_pixels();
    }

    /// Get the command queue
    pub fn get_command_queue(&self) -> &CommandQueue {
        &self.command_queue
    }

    /// Get the device
    pub fn get_device(&self) -> &Device {
        &self.device
    }

    /// Get the display width (scaled)
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Get the display height (scaled)
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Print frame buffer statistics (for debugging)
    pub fn debug_frame(&self, frame_buffer: &FrameBuffer) {
        let pixels = frame_buffer.get_pixels();
        let mut white_count = 0;
        let mut black_count = 0;

        for &pixel in pixels.iter() {
            if pixel == 0xFFFFFFFF { white_count += 1; }
            if pixel == 0xFF000000 { black_count += 1; }
        }

        println!("Frame: {} white pixels, {} black pixels",
            white_count, black_count);
    }
}

impl Default for MetalRenderer {
    fn default() -> Self {
        Self::new().expect("Failed to create Metal renderer")
    }
}
