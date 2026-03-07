/// Display module for GameBoy LCD output
///
/// Handles frame buffer management.

pub mod frame_buffer;

pub use frame_buffer::{
    create_shared_frame_buffer, FrameBuffer, SharedFrameBuffer,
    SCREEN_WIDTH, SCREEN_HEIGHT, FRAME_BUFFER_SIZE,
};
