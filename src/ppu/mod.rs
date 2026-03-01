/// Pixel Processing Unit (PPU) module
///
/// The PPU handles rendering, VRAM access, and display generation.

pub mod video;
pub mod oam;
pub mod rendering;

pub use video::VideoController;
pub use oam::OAM;
pub use rendering::Renderer;
