/// Audio channel implementations

use crate::audio::apu::AudioChannel;

/// Channel types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Channel {
    Square1, // Channel 1: Square wave with sweep
    Square2, // Channel 2: Square wave
    Wave,    // Channel 3: Wave pattern
    Noise,   // Channel 4: Noise
}

/// Square audio channel
#[derive(Debug)]
pub struct SquareChannel {
    enabled: bool,
    frequency: u16,
    duty_cycle: u8,
    volume: u8,
}

impl SquareChannel {
    pub fn new() -> Self {
        SquareChannel {
            enabled: false,
            frequency: 0,
            duty_cycle: 0,
            volume: 0,
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn set_frequency(&mut self, freq: u16) {
        self.frequency = freq;
    }

    pub fn set_duty_cycle(&mut self, duty: u8) {
        self.duty_cycle = duty;
    }

    pub fn set_volume(&mut self, vol: u8) {
        self.volume = vol;
    }
}

impl Default for SquareChannel {
    fn default() -> Self {
        Self::new()
    }
}

/// Wave channel
#[derive(Debug)]
pub struct WaveChannel {
    enabled: bool,
    volume: u8,
    pattern: [u8; 32], // 32 bytes of wave pattern
}

impl WaveChannel {
    pub fn new() -> Self {
        WaveChannel {
            enabled: false,
            volume: 0,
            pattern: [0; 32],
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn set_volume(&mut self, vol: u8) {
        self.volume = vol;
    }

    pub fn set_pattern(&mut self, data: [u8; 32]) {
        self.pattern = data;
    }
}

impl Default for WaveChannel {
    fn default() -> Self {
        Self::new()
    }
}

/// Noise channel
#[derive(Debug)]
#[allow(dead_code)]
pub struct NoiseChannel {
    enabled: bool,
    shift_register: u32,
    clock_rate: u32,
}

impl NoiseChannel {
    pub fn new() -> Self {
        NoiseChannel {
            enabled: false,
            shift_register: 0x7FFF,
            clock_rate: 0,
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn clock(&mut self) {
        if !self.enabled {
            return;
        }
        // LFSR feedback
        let bit0 = self.shift_register & 1;
        let bit1 = (self.shift_register >> 1) & 1;
        let feedback = bit0 ^ bit1;
        self.shift_register >>= 1;
        self.shift_register |= feedback << 14;
    }

    pub fn get_bit(&self) -> u8 {
        if self.shift_register & 1 == 1 { 1 } else { 0 }
    }
}

impl Default for NoiseChannel {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioChannel for SquareChannel {
    fn clock(&mut self) -> bool {
        // Clock the channel
        true
    }

    fn get_output(&self) -> f32 {
        if !self.enabled {
            return 0.0;
        }
        // Simple square wave output based on duty cycle
        let duty_values = [0.125, 0.25, 0.5, 0.75]; // 12.5%, 25%, 50%, 75%
        let duty_index = (self.duty_cycle >> 6) as usize;
        let duty = duty_values[duty_index.min(3)];
        if duty > 0.0 {
            1.0 * duty
        } else {
            0.0
        }
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn reset(&mut self) {
        self.enabled = true;
    }
}

impl AudioChannel for WaveChannel {
    fn clock(&mut self) -> bool {
        true
    }

    fn get_output(&self) -> f32 {
        if !self.enabled {
            return 0.0;
        }
        // Simple wave output using volume
        let vol = (self.volume >> 4) as u8;
        if vol == 0 {
            0.0
        } else {
            1.0 * (vol as f32 / 7.0)
        }
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn reset(&mut self) {
        self.enabled = true;
    }
}

impl AudioChannel for NoiseChannel {
    fn clock(&mut self) -> bool {
        if !self.enabled {
            return false;
        }
        // LFSR feedback
        let bit0 = self.shift_register & 1;
        let bit1 = (self.shift_register >> 1) & 1;
        let feedback = bit0 ^ bit1;
        self.shift_register >>= 1;
        self.shift_register |= feedback << 14;
        true
    }

    fn get_output(&self) -> f32 {
        if !self.enabled {
            return 0.0;
        }
        if self.shift_register & 1 == 1 {
            1.0
        } else {
            0.0
        }
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn reset(&mut self) {
        self.shift_register = 0x7FFF;
        self.enabled = true;
    }
}
