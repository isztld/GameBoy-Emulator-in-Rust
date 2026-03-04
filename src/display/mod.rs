/// Display module for GameBoy LCD output
///
/// Handles frame buffer management and Metal rendering.

pub mod frame_buffer;
pub mod metal_renderer;

pub use frame_buffer::{
    create_shared_frame_buffer, FrameBuffer, SharedFrameBuffer,
    SCREEN_WIDTH, SCREEN_HEIGHT, FRAME_BUFFER_SIZE,
};
pub use metal_renderer::MetalRenderer;
