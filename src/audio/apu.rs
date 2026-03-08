/// Audio Processing Unit (APU) for GameBoy DMG
///
/// Implements the full 4-channel audio system with frame sequencer,
/// volume envelopes, frequency sweep, length counters, and sample output
/// via a shared ring buffer consumed by a cpal audio stream.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use crate::audio::channels::{SquareChannel, WaveChannel, NoiseChannel};

/// Stereo audio output (left and right channels), values in [0.0, 1.0]
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

impl Default for AudioOutput {
    fn default() -> Self {
        Self::new()
    }
}

/// Audio Processing Unit
pub struct AudioProcessor {
    pub ch1: SquareChannel,
    pub ch2: SquareChannel,
    pub ch3: WaveChannel,
    pub ch4: NoiseChannel,
    pub enabled: bool,
    pub master_volume: u8,  // NR50
    pub panning: u8,        // NR51

    // Sample accumulation
    sample_timer: f32,

    /// Shared sample buffer consumed by the cpal audio callback.
    pub audio_buffer: Arc<Mutex<VecDeque<(f32, f32)>>>,

    // Frame sequencer
    frame_seq_timer: u32,
    frame_seq_step: u8,
}

impl AudioProcessor {
    pub fn new() -> Self {
        AudioProcessor {
            ch1: SquareChannel::new(),
            ch2: SquareChannel::new(),
            ch3: WaveChannel::new(),
            ch4: NoiseChannel::new(),
            enabled: false,
            master_volume: 0,
            panning: 0,
            sample_timer: 0.0,
            audio_buffer: Arc::new(Mutex::new(VecDeque::new())),
            frame_seq_timer: 0,
            frame_seq_step: 0,
        }
    }

    /// Clock the APU once per M-cycle (~1,048,576 Hz).
    /// Called from System::step's tick closure.
    pub fn clock(&mut self) {
        if !self.enabled {
            self.accumulate_sample();
            return;
        }

        self.ch1.clock();
        self.ch2.clock();
        self.ch3.clock();
        self.ch4.clock();

        // Frame sequencer: ticks every 8192 M-cycles → 128 Hz
        self.frame_seq_timer += 1;
        if self.frame_seq_timer >= 8192 {
            self.frame_seq_timer = 0;
            match self.frame_seq_step {
                0 | 4 => {
                    self.ch1.clock_length();
                    self.ch2.clock_length();
                    self.ch3.clock_length();
                    self.ch4.clock_length();
                }
                2 | 6 => {
                    self.ch1.clock_length();
                    self.ch2.clock_length();
                    self.ch3.clock_length();
                    self.ch4.clock_length();
                    self.ch1.clock_sweep();
                }
                7 => {
                    self.ch1.clock_envelope();
                    self.ch2.clock_envelope();
                    self.ch4.clock_envelope();
                }
                _ => {}
            }
            self.frame_seq_step = (self.frame_seq_step + 1) & 7;
        }

        self.accumulate_sample();
    }

    fn accumulate_sample(&mut self) {
        self.sample_timer += 1.0;
        // ~23.77 M-cycles per sample at 44100 Hz
        let cycles_per_sample = 1_048_576.0 / 44100.0;
        if self.sample_timer >= cycles_per_sample {
            self.sample_timer -= cycles_per_sample;
            let (left, right) = self.mix();
            let mut buf = self.audio_buffer.lock().unwrap();
            // Cap buffer to ~4 frames of audio to prevent unbounded growth
            if buf.len() < 8192 {
                buf.push_back((left, right));
            }
        }
    }

    fn mix(&self) -> (f32, f32) {
        if !self.enabled {
            return (0.0, 0.0);
        }

        let channels = [
            self.ch1.get_output(),
            self.ch2.get_output(),
            self.ch3.get_output(),
            self.ch4.get_output(),
        ];

        let mut left = 0.0f32;
        let mut right = 0.0f32;

        for i in 0..4 {
            // SO2 = left, bits 7-4 (CH4→SO2 is bit 7, CH1→SO2 is bit 4)
            if self.panning & (0x10 << i) != 0 {
                left += channels[i];
            }
            // SO1 = right, bits 3-0 (CH4→SO1 is bit 3, CH1→SO1 is bit 0)
            if self.panning & (0x01 << i) != 0 {
                right += channels[i];
            }
        }

        let so2_vol = ((self.master_volume >> 4) & 0x07) as f32 / 7.0;
        let so1_vol = (self.master_volume & 0x07) as f32 / 7.0;

        (left / 4.0 * so2_vol, right / 4.0 * so1_vol)
    }

    /// Write to an audio I/O register (0xFF10-0xFF3F).
    pub fn write_io(&mut self, address: u16, value: u8) {
        match address {
            0xFF10..=0xFF26 => self.write_audio_register(address, value),
            0xFF30..=0xFF3F => {
                let index = (address as usize) - 0xFF30;
                self.ch3.write_wave_byte(index, value);
            }
            _ => {}
        }
    }

    fn write_audio_register(&mut self, address: u16, value: u8) {
        match address {
            0xFF10 => self.ch1.write_sweep(value),
            0xFF11 => self.ch1.write_duty_length(value),
            0xFF12 => self.ch1.write_envelope(value),
            0xFF13 => self.ch1.write_freq_lo(value),
            0xFF14 => self.ch1.write_freq_hi(value),
            // 0xFF15 unused
            0xFF16 => self.ch2.write_duty_length(value),
            0xFF17 => self.ch2.write_envelope(value),
            0xFF18 => self.ch2.write_freq_lo(value),
            0xFF19 => self.ch2.write_freq_hi(value),
            0xFF1A => self.ch3.write_dac_enable(value),
            0xFF1B => self.ch3.write_length(value),
            0xFF1C => self.ch3.write_volume(value),
            0xFF1D => self.ch3.write_freq_lo(value),
            0xFF1E => self.ch3.write_freq_hi(value),
            // 0xFF1F unused
            0xFF20 => self.ch4.write_length(value),
            0xFF21 => self.ch4.write_envelope(value),
            0xFF22 => self.ch4.write_poly(value),
            0xFF23 => self.ch4.write_trigger(value),
            0xFF24 => self.master_volume = value,
            0xFF25 => self.panning = value,
            0xFF26 => {
                let was = self.enabled;
                self.enabled = (value & 0x80) != 0;
                if was && !self.enabled {
                    // Power off: reset all channel registers
                    self.ch1 = SquareChannel::new();
                    self.ch2 = SquareChannel::new();
                    self.ch3 = WaveChannel::new();
                    self.ch4 = NoiseChannel::new();
                    self.master_volume = 0;
                    self.panning = 0;
                }
            }
            _ => {}
        }
    }

    /// Get the current stereo audio output (legacy; prefer audio_buffer for streaming).
    pub fn get_output(&self) -> AudioOutput {
        let (left, right) = self.mix();
        AudioOutput { left, right }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

impl Default for AudioProcessor {
    fn default() -> Self {
        Self::new()
    }
}
