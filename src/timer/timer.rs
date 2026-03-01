/// Timer module for GameBoy
///
/// Handles the divider register (DIV), timer counter (TIMA),
/// and timer modulo (TMA) for GameBoy timers.

/// Timer control register
#[derive(Debug, Clone, Copy)]
pub struct TAC {
    /// Bit 2: Timer enable (0=disable, 1=enable)
    pub enabled: bool,
    /// Bits 1-0: Input clock select
    /// 00: 4096 Hz
    /// 01: 262144 Hz
    /// 10: 65536 Hz
    /// 11: 16384 Hz
    pub clock_select: u8,
}

impl TAC {
    pub fn new() -> Self {
        TAC {
            enabled: false,
            clock_select: 0,
        }
    }

    pub fn from_byte(value: u8) -> Self {
        TAC {
            enabled: (value & 0x04) != 0,
            clock_select: value & 0x03,
        }
    }

    pub fn to_byte(&self) -> u8 {
        (if self.enabled { 0x04 } else { 0x00 }) | (self.clock_select & 0x03)
    }

    /// Get the clock frequency in Hz based on clock select
    pub fn clock_frequency(&self) -> u32 {
        match self.clock_select {
            0 => 4096,
            1 => 262144,
            2 => 65536,
            3 => 16384,
            _ => 0,
        }
    }
}

impl Default for TAC {
    fn default() -> Self {
        Self::new()
    }
}

/// Timer
#[derive(Debug)]
pub struct Timer {
    pub div: u8,      // Divider register (00-FF, increments at 16384 Hz)
    pub tac: TAC,     // Timer control
    pub tima: u8,     // Timer counter
    pub tma: u8,      // Timer modulo
    pub clock_counter: u32, // Internal clock counter
    pub interrupt_pending: bool,
}

impl Timer {
    pub fn new() -> Self {
        Timer {
            div: 0,
            tac: TAC::new(),
            tima: 0,
            tma: 0,
            clock_counter: 0,
            interrupt_pending: false,
        }
    }

    /// Increment the divider register
    /// Called at 16384 Hz (every 4 CPU cycles)
    pub fn increment_div(&mut self) {
        self.div = self.div.wrapping_add(1);
    }

    /// Clock the timer
    /// Called at the rate specified by TAC
    pub fn clock(&mut self) {
        if !self.tac.enabled {
            return;
        }

        self.clock_counter += 1;

        // Get the period based on clock frequency
        let period = 1024 / self.tac.clock_frequency() as u32; // Approximate

        if self.clock_counter >= period {
            self.clock_counter = 0;
            self.tima = self.tima.wrapping_add(1);

            if self.tima == 0 {
                // Timer overflow
                self.tima = self.tma;
                self.interrupt_pending = true;
            }
        }
    }

    /// Reset the timer
    pub fn reset(&mut self) {
        self.div = 0;
        self.tima = 0;
        self.tac = TAC::new();
        self.clock_counter = 0;
        self.interrupt_pending = false;
    }

    /// Write to DIV register (reset divider)
    pub fn write_div(&mut self, _value: u8) {
        self.div = 0;
    }

    /// Write to TIMA register
    pub fn write_tima(&mut self, value: u8) {
        self.tima = value;
    }

    /// Write to TMA register
    pub fn write_tma(&mut self, value: u8) {
        self.tma = value;
    }

    /// Write to TAC register
    pub fn write_tac(&mut self, value: u8) {
        self.tac = TAC::from_byte(value);
    }

    /// Check if timer interrupt is pending
    pub fn is_interrupt_pending(&self) -> bool {
        self.interrupt_pending
    }

    /// Acknowledge timer interrupt
    pub fn acknowledge_interrupt(&mut self) {
        self.interrupt_pending = false;
    }

    /// Get current TIMA value
    pub fn get_tima(&self) -> u8 {
        self.tima
    }
}

impl Default for Timer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tac_clock_select() {
        let tac = TAC::from_byte(0x00);
        assert_eq!(tac.clock_frequency(), 4096);

        let tac = TAC::from_byte(0x01);
        assert_eq!(tac.clock_frequency(), 262144);

        let tac = TAC::from_byte(0x02);
        assert_eq!(tac.clock_frequency(), 65536);

        let tac = TAC::from_byte(0x03);
        assert_eq!(tac.clock_frequency(), 16384);
    }

    #[test]
    fn test_timer_div_increment() {
        let mut timer = Timer::new();
        timer.increment_div();
        assert_eq!(timer.div, 1);
        timer.increment_div();
        assert_eq!(timer.div, 2);
    }
}
