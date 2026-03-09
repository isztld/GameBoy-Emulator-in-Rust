/// Audio channel implementations for GameBoy DMG APU

const DUTY_TABLE: [[u8; 8]; 4] = [
    [0, 0, 0, 0, 0, 0, 0, 1], // 12.5%
    [1, 0, 0, 0, 0, 0, 0, 1], // 25%
    [1, 0, 0, 0, 1, 1, 1, 1], // 50%
    [0, 1, 1, 1, 1, 1, 1, 0], // 75%
];

// T-cycle divisors [8,16,32,48,64,80,96,112] divided by 4 for M-cycle clocking
const NOISE_DIVISORS: [u32; 8] = [2, 4, 8, 12, 16, 20, 24, 28];

/// Square wave channel (used for CH1 and CH2; CH1 uses the sweep fields)
#[derive(Debug)]
pub struct SquareChannel {
    pub enabled: bool,
    dac_enabled: bool,
    // Period timer
    frequency: u16,
    timer: i32,
    // Duty
    duty: u8,
    duty_pos: u8,
    // Volume envelope
    volume: u8,
    env_initial: u8,
    env_add: bool,
    env_period: u8,
    env_timer: u8,
    // Length counter
    length: u16,
    length_enabled: bool,
    // CH1 sweep (unused by CH2 but harmless)
    sweep_period: u8,
    sweep_negate: bool,
    sweep_shift: u8,
    sweep_timer: u8,
    sweep_enabled: bool,
    sweep_shadow: u16,
}

impl SquareChannel {
    pub fn new() -> Self {
        SquareChannel {
            enabled: false,
            dac_enabled: false,
            frequency: 0,
            timer: 1,
            duty: 0,
            duty_pos: 0,
            volume: 0,
            env_initial: 0,
            env_add: false,
            env_period: 0,
            env_timer: 0,
            length: 0,
            length_enabled: false,
            sweep_period: 0,
            sweep_negate: false,
            sweep_shift: 0,
            sweep_timer: 0,
            sweep_enabled: false,
            sweep_shadow: 0,
        }
    }

    pub fn write_sweep(&mut self, val: u8) {
        self.sweep_period = (val >> 4) & 7;
        self.sweep_negate = (val & 0x08) != 0;
        self.sweep_shift = val & 7;
    }

    pub fn write_duty_length(&mut self, val: u8) {
        self.duty = (val >> 6) & 3;
        self.length = 64 - (val & 63) as u16;
    }

    pub fn write_envelope(&mut self, val: u8) {
        self.env_initial = (val >> 4) & 15;
        self.env_add = (val & 0x08) != 0;
        self.env_period = val & 7;
        self.dac_enabled = (val & 0xF8) != 0;
        if !self.dac_enabled {
            self.enabled = false;
        }
    }

    pub fn write_freq_lo(&mut self, val: u8) {
        self.frequency = (self.frequency & 0x700) | (val as u16);
    }

    pub fn write_freq_hi(&mut self, val: u8) {
        self.frequency = (self.frequency & 0xFF) | (((val & 7) as u16) << 8);
        self.length_enabled = (val & 0x40) != 0;
        if (val & 0x80) != 0 {
            self.trigger();
        }
    }

    fn trigger(&mut self) {
        if self.dac_enabled {
            self.enabled = true;
        }
        if self.length == 0 {
            self.length = 64;
        }
        self.timer = ((2048 - self.frequency) as i32).max(1);
        self.volume = self.env_initial;
        self.env_timer = self.env_period;

        // Sweep init
        self.sweep_shadow = self.frequency;
        self.sweep_timer = if self.sweep_period > 0 { self.sweep_period } else { 8 };
        self.sweep_enabled = self.sweep_period > 0 || self.sweep_shift > 0;
        // Overflow check on trigger
        if self.sweep_shift > 0 {
            self.calculate_sweep_freq(); // side-effect: may disable channel
        }
    }

    fn calculate_sweep_freq(&mut self) -> u16 {
        let delta = self.sweep_shadow >> self.sweep_shift;
        let new_freq = if self.sweep_negate {
            self.sweep_shadow.wrapping_sub(delta)
        } else {
            self.sweep_shadow + delta
        };
        if new_freq > 2047 {
            self.enabled = false;
        }
        new_freq
    }

    pub fn clock(&mut self) {
        self.timer -= 1;
        if self.timer <= 0 {
            self.timer = ((2048 - self.frequency) as i32).max(1);
            self.duty_pos = (self.duty_pos + 1) & 7;
        }
    }

    pub fn clock_length(&mut self) {
        if self.length_enabled && self.length > 0 {
            self.length -= 1;
            if self.length == 0 {
                self.enabled = false;
            }
        }
    }

    pub fn clock_envelope(&mut self) {
        if self.env_period == 0 {
            return;
        }
        if self.env_timer > 0 {
            self.env_timer -= 1;
        }
        if self.env_timer == 0 {
            self.env_timer = self.env_period;
            if self.env_add && self.volume < 15 {
                self.volume += 1;
            } else if !self.env_add && self.volume > 0 {
                self.volume -= 1;
            }
        }
    }

    pub fn clock_sweep(&mut self) {
        if self.sweep_timer > 0 {
            self.sweep_timer -= 1;
        }
        if self.sweep_timer == 0 {
            self.sweep_timer = if self.sweep_period > 0 { self.sweep_period } else { 8 };
            if self.sweep_enabled && self.sweep_period > 0 {
                let new_freq = self.calculate_sweep_freq();
                if new_freq <= 2047 {
                    self.frequency = new_freq;
                    self.sweep_shadow = new_freq;
                    // Second overflow check
                    if self.sweep_shift > 0 {
                        self.calculate_sweep_freq();
                    }
                }
            }
        }
    }

    pub fn get_output(&self) -> f32 {
        if !self.enabled || !self.dac_enabled {
            return 0.0;
        }
        let high = DUTY_TABLE[self.duty as usize][self.duty_pos as usize];
        if high == 1 {
            self.volume as f32 / 15.0
        } else {
            0.0
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

impl Default for SquareChannel {
    fn default() -> Self {
        Self::new()
    }
}

/// Wave channel (CH3)
#[derive(Debug)]
pub struct WaveChannel {
    pub enabled: bool,
    dac_enabled: bool,
    frequency: u16,
    timer: i32,
    position: u8,
    volume_shift: u8,
    pub pattern: [u8; 16],
    current_sample: u8,
    length: u16,
    length_enabled: bool,
}

impl WaveChannel {
    pub fn new() -> Self {
        WaveChannel {
            enabled: false,
            dac_enabled: false,
            frequency: 0,
            timer: 1,
            position: 0,
            volume_shift: 0,
            // DMG power-on wave RAM state (left by boot ROM)
            pattern: [0x84, 0x40, 0x43, 0xAA, 0x2D, 0x78, 0x92, 0x3C,
                      0x60, 0x59, 0xAD, 0xA1, 0x0C, 0xE2, 0xF3, 0x44],
            current_sample: 0,
            length: 0,
            length_enabled: false,
        }
    }

    pub fn write_dac_enable(&mut self, val: u8) {
        self.dac_enabled = (val & 0x80) != 0;
        if !self.dac_enabled {
            self.enabled = false;
        }
    }

    pub fn write_length(&mut self, val: u8) {
        self.length = 256 - val as u16;
    }

    pub fn write_volume(&mut self, val: u8) {
        self.volume_shift = (val >> 5) & 3;
    }

    pub fn write_freq_lo(&mut self, val: u8) {
        self.frequency = (self.frequency & 0x700) | (val as u16);
    }

    pub fn write_freq_hi(&mut self, val: u8) {
        self.frequency = (self.frequency & 0xFF) | (((val & 7) as u16) << 8);
        self.length_enabled = (val & 0x40) != 0;
        if (val & 0x80) != 0 {
            self.trigger();
        }
    }

    fn trigger(&mut self) {
        if self.dac_enabled {
            self.enabled = true;
        }
        if self.length == 0 {
            self.length = 256;
        }
        self.timer = (((2048 - self.frequency) / 2) as i32).max(1);
        self.position = 0;
    }

    pub fn write_wave_byte(&mut self, index: usize, value: u8) {
        if index < 16 {
            self.pattern[index] = value;
        }
    }

    pub fn clock(&mut self) {
        self.timer -= 1;
        if self.timer <= 0 {
            self.timer = (((2048 - self.frequency) / 2) as i32).max(1);
            self.position = (self.position + 1) & 31;
            let byte_idx = self.position / 2;
            self.current_sample = if self.position & 1 == 0 {
                (self.pattern[byte_idx as usize] >> 4) & 0xF
            } else {
                self.pattern[byte_idx as usize] & 0xF
            };
        }
    }

    pub fn clock_length(&mut self) {
        if self.length_enabled && self.length > 0 {
            self.length -= 1;
            if self.length == 0 {
                self.enabled = false;
            }
        }
    }

    pub fn get_output(&self) -> f32 {
        if !self.enabled || !self.dac_enabled {
            return 0.0;
        }
        let s = match self.volume_shift {
            0 => 0,
            1 => self.current_sample,
            2 => self.current_sample >> 1,
            3 => self.current_sample >> 2,
            _ => 0,
        };
        s as f32 / 15.0
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

impl Default for WaveChannel {
    fn default() -> Self {
        Self::new()
    }
}

/// Noise channel (CH4) with LFSR
#[derive(Debug)]
pub struct NoiseChannel {
    pub enabled: bool,
    dac_enabled: bool,
    lfsr: u16,
    clock_shift: u8,
    width_mode: bool,
    divisor_code: u8,
    timer: i32,
    volume: u8,
    env_initial: u8,
    env_add: bool,
    env_period: u8,
    env_timer: u8,
    length: u16,
    length_enabled: bool,
}

impl NoiseChannel {
    pub fn new() -> Self {
        NoiseChannel {
            enabled: false,
            dac_enabled: false,
            lfsr: 0x7FFF,
            clock_shift: 0,
            width_mode: false,
            divisor_code: 0,
            timer: 1,
            volume: 0,
            env_initial: 0,
            env_add: false,
            env_period: 0,
            env_timer: 0,
            length: 0,
            length_enabled: false,
        }
    }

    pub fn write_length(&mut self, val: u8) {
        self.length = (64 - (val & 63)) as u16;
    }

    pub fn write_envelope(&mut self, val: u8) {
        self.env_initial = (val >> 4) & 15;
        self.env_add = (val & 0x08) != 0;
        self.env_period = val & 7;
        self.dac_enabled = (val & 0xF8) != 0;
        if !self.dac_enabled {
            self.enabled = false;
        }
    }

    pub fn write_poly(&mut self, val: u8) {
        self.clock_shift = (val >> 4) & 15;
        self.width_mode = (val & 0x08) != 0;
        self.divisor_code = val & 7;
    }

    pub fn write_trigger(&mut self, val: u8) {
        self.length_enabled = (val & 0x40) != 0;
        if (val & 0x80) != 0 {
            self.trigger();
        }
    }

    fn trigger(&mut self) {
        if self.dac_enabled {
            self.enabled = true;
        }
        if self.length == 0 {
            self.length = 64;
        }
        let divisor = NOISE_DIVISORS[self.divisor_code as usize];
        self.timer = ((divisor << self.clock_shift) as i32).max(1);
        self.lfsr = 0x7FFF;
        self.volume = self.env_initial;
        self.env_timer = self.env_period;
    }

    pub fn clock(&mut self) {
        self.timer -= 1;
        if self.timer <= 0 {
            let divisor = NOISE_DIVISORS[self.divisor_code as usize].max(1);
            self.timer = ((divisor << self.clock_shift) as i32).max(1);
            // Clock LFSR
            let xor = (self.lfsr & 1) ^ ((self.lfsr >> 1) & 1);
            self.lfsr >>= 1;
            self.lfsr |= xor << 14;
            if self.width_mode {
                // Also set bit 6 for 7-bit mode
                self.lfsr = (self.lfsr & !0x40) | (xor << 6);
            }
        }
    }

    pub fn clock_length(&mut self) {
        if self.length_enabled && self.length > 0 {
            self.length -= 1;
            if self.length == 0 {
                self.enabled = false;
            }
        }
    }

    pub fn clock_envelope(&mut self) {
        if self.env_period == 0 {
            return;
        }
        if self.env_timer > 0 {
            self.env_timer -= 1;
        }
        if self.env_timer == 0 {
            self.env_timer = self.env_period;
            if self.env_add && self.volume < 15 {
                self.volume += 1;
            } else if !self.env_add && self.volume > 0 {
                self.volume -= 1;
            }
        }
    }

    pub fn get_output(&self) -> f32 {
        if !self.enabled || !self.dac_enabled {
            return 0.0;
        }
        // LFSR bit 0 low = output high (inverted)
        if self.lfsr & 1 == 0 {
            self.volume as f32 / 15.0
        } else {
            0.0
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

impl Default for NoiseChannel {
    fn default() -> Self {
        Self::new()
    }
}
