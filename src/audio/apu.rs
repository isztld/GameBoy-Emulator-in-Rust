/// Audio Processing Unit (APU) for GameBoy
///
/// The APU handles audio generation with 4 channels:
/// - Channel 1: Square wave with sweep
/// - Channel 2: Square wave
/// - Channel 3: Wave pattern
/// - Channel 4: Noise

/// Audio output (left and right channels)
#[derive(Debug, Clone, Copy)]
pub struct AudioOutput {
    pub left: f32,
    pub right: f32,
}

impl AudioOutput {
    pub fn new() -> Self {
        AudioOutput { left: 0.0, right: 0.0 }
    }

    pub fn silence() -> Self {
        AudioOutput { left: 0.0, right: 0.0 }
    }
}

/// Audio Channel trait
pub trait AudioChannel {
    fn clock(&mut self) -> bool; // Returns true if output changed
    fn get_output(&self) -> f32;
    fn is_enabled(&self) -> bool;
    fn reset(&mut self);
}

// Re-export channel types from channels module to avoid duplicate definitions
pub use crate::audio::channels::{SquareChannel, WaveChannel, NoiseChannel};

/// Audio Processing Unit
pub struct AudioProcessor {
    pub channels: [Box<dyn AudioChannel>; 4],
    pub enabled: bool,
    pub master_volume: u8, // NR50
    pub output_select: u8, // NR51
    wave_pattern: [u8; 32], // Wave pattern RAM (0xFF30-0xFF3F)
}

impl AudioProcessor {
    pub fn new() -> Self {
        // Initialize channels
        // Note: SquareChannel from channels.rs doesn't take channel_id
        AudioProcessor {
            channels: [
                Box::new(SquareChannel::new()),
                Box::new(SquareChannel::new()),
                Box::new(WaveChannel::new()),
                Box::new(NoiseChannel::new()),
            ],
            enabled: false,
            master_volume: 0x00,
            output_select: 0x00,
            wave_pattern: [0; 32],
        }
    }

    /// Clock the audio processor
    /// Called at 2x CPU frequency (8388 Hz)
    pub fn clock(&mut self) {
        for channel in &mut self.channels {
            channel.clock();
        }
    }

    /// Get audio output for current frame
    pub fn get_output(&self) -> AudioOutput {
        if !self.enabled {
            return AudioOutput::silence();
        }

        let left_sum: f32 = self.channels
            .iter()
            .enumerate()
            .filter(|(_, ch)| ch.is_enabled() && (self.output_select & 0x01) != 0)
            .map(|(_i, ch)| {
                let vol = (self.master_volume >> 4) & 0x07;
                ch.get_output() * (vol as f32 / 7.0)
            })
            .sum();

        let right_sum: f32 = self.channels
            .iter()
            .enumerate()
            .filter(|(_, ch)| ch.is_enabled() && (self.output_select & 0x10) != 0)
            .map(|(_i, ch)| {
                let vol = self.master_volume & 0x07;
                ch.get_output() * (vol as f32 / 7.0)
            })
            .sum();

        AudioOutput {
            left: left_sum,
            right: right_sum,
        }
    }

    /// Write to I/O register
    pub fn write_io(&mut self, address: u16, value: u8) {
        match address {
            0xFF10..=0xFF26 => {
                // Audio registers
                self.write_audio_register(address, value);
            }
            0xFF30..=0xFF3F => {
                // Wave pattern RAM
                self.write_wave_pattern(address, value);
            }
            _ => {}
        }
    }

    fn write_audio_register(&mut self, address: u16, value: u8) {
        let offset = address as usize;
        match offset {
            0xFF10 => {
                // NR10 - Channel 1 sweep register
                // TODO: Implement sweep
            }
            0xFF11 => {
                // NR11 - Channel 1 pattern/length
                // TODO: Implement
            }
            0xFF12 => {
                // NR12 - Channel 1 envelope
                // TODO: Implement
            }
            0xFF13 => {
                // NR13 - Channel 1 frequency low
                // TODO: Implement
            }
            0xFF14 => {
                // NR14 - Channel 1 frequency high
                // TODO: Implement
            }
            0xFF15 => {
                // NR21 - Channel 2 pattern/length
                // TODO: Implement
            }
            0xFF16 => {
                // NR22 - Channel 2 envelope
                // TODO: Implement
            }
            0xFF17 => {
                // NR23 - Channel 2 frequency low
                // TODO: Implement
            }
            0xFF18 => {
                // NR24 - Channel 2 frequency high
                // TODO: Implement
            }
            0xFF19 => {
                // NR30 - Channel 3 enable
                self.channels[2].reset();
            }
            0xFF1A => {
                // NR31 - Channel 3 pattern/length
            }
            0xFF1B => {
                // NR32 - Channel 3 wave pattern
            }
            0xFF1C => {
                // NR33 - Channel 3 frequency low
            }
            0xFF1D => {
                // NR34 - Channel 3 frequency high
            }
            0xFF1E => {
                // NR41 - Channel 4 length
            }
            0xFF1F => {
                // NR42 - Channel 4 envelope
            }
            0xFF20 => {
                // NR43 - Channel 4 polynomial counter
            }
            0xFF21 => {
                // NR44 - Channel 4 counter/consecutive
            }
            0xFF22 => {
                // NR50 - Volume control
                self.master_volume = value;
            }
            0xFF23 => {
                // NR51 - Output select
                self.output_select = value;
            }
            0xFF24 => {
                // NR52 - Master enable
                self.enabled = (value & 0x80) != 0;
            }
            _ => {}
        }
    }

    fn write_wave_pattern(&mut self, address: u16, value: u8) {
        // Wave pattern RAM at 0xFF30-0xFF3F
        // Each address corresponds to one byte of the 32-byte wave pattern
        let index = (address as usize) - 0xFF30;
        self.wave_pattern[index] = value;
    }

    /// Check if audio is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

impl Default for AudioProcessor {
    fn default() -> Self {
        Self::new()
    }
}
