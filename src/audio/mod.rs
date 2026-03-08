/// Audio Processing Unit (APU) module
///
/// The APU handles audio generation with 4 channels:
/// - Channel 1: Square wave with sweep
/// - Channel 2: Square wave
/// - Channel 3: Wave pattern
/// - Channel 4: Noise

pub mod apu;
pub mod channels;

pub use apu::AudioProcessor;
pub use channels::{SquareChannel, WaveChannel, NoiseChannel};
