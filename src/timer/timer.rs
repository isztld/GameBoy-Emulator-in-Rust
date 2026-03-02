/// Timer module for GameBoy
///
/// DIV increments at 16384 Hz (every 64 machine cycles at 1,048,576 machine cycles/sec).
/// TIMA increments at the rate selected by TAC bits 1-0:
///   00: 4096 Hz   → every 256 machine cycles
///   01: 262144 Hz → every 4 machine cycles
///   10: 65536 Hz  → every 16 machine cycles
///   11: 16384 Hz  → every 64 machine cycles

const CPU_MACHINE_HZ: u32 = 1_048_576; // machine cycles per second (4 MHz / 4)

/// Number of machine cycles between DIV increments (16384 Hz).
const DIV_PERIOD: u32 = CPU_MACHINE_HZ / 16384; // 64

#[derive(Debug, Clone, Copy)]
pub struct TAC {
    pub enabled: bool,
    pub clock_select: u8,
}

impl TAC {
    pub fn new() -> Self {
        TAC { enabled: false, clock_select: 0 }
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

    /// Machine-cycle period between TIMA increments for this clock select.
    pub fn tima_period(&self) -> u32 {
        match self.clock_select {
            0 => CPU_MACHINE_HZ / 4_096,   // 256
            1 => CPU_MACHINE_HZ / 262_144, // 4
            2 => CPU_MACHINE_HZ / 65_536,  // 16
            3 => CPU_MACHINE_HZ / 16_384,  // 64
            _ => 256,
        }
    }
}

impl Default for TAC {
    fn default() -> Self { Self::new() }
}

#[derive(Debug)]
pub struct Timer {
    pub div: u8,
    pub tac: TAC,
    pub tima: u8,
    pub tma: u8,

    /// Internal counter driving DIV (resets every DIV_PERIOD machine cycles).
    div_counter: u32,
    /// Internal counter driving TIMA (resets every tac.tima_period() machine cycles).
    tima_counter: u32,
}

impl Timer {
    pub fn new() -> Self {
        Timer {
            div: 0,
            tac: TAC::new(),
            tima: 0,
            tma: 0,
            div_counter: 0,
            tima_counter: 0,
        }
    }

    /// Advance the timer by one machine cycle.
    /// Writes the timer interrupt bit (bit 2) into IF at `0xFF0F` on overflow.
    pub fn tick(&mut self, bus: &mut crate::memory::MemoryBus) {
        // --- DIV ---
        self.div_counter += 1;
        if self.div_counter >= DIV_PERIOD {
            self.div_counter = 0;
            self.div = self.div.wrapping_add(1);
            // Keep the bus DIV register in sync.
            // Write directly to the internal io array to avoid the reset-on-write behaviour.
            bus.io[0x04] = self.div;
        }

        // --- TIMA ---
        if !self.tac.enabled {
            return;
        }

        self.tima_counter += 1;
        if self.tima_counter >= self.tac.tima_period() {
            self.tima_counter = 0;
            let (new_tima, overflow) = self.tima.overflowing_add(1);
            if overflow {
                // Reload from TMA and request timer interrupt (IF bit 2).
                self.tima = self.tma;
                let if_val = bus.read(0xFF0F);
                bus.write(0xFF0F, if_val | 0x04);
            } else {
                self.tima = new_tima;
            }
            // Keep bus TIMA register in sync.
            bus.io[0x05] = self.tima;
        }
    }

    pub fn reset(&mut self) {
        self.div = 0;
        self.tima = 0;
        self.tma = 0;
        self.tac = TAC::new();
        self.div_counter = 0;
        self.tima_counter = 0;
    }

    pub fn write_div(&mut self) {
        // Any write to DIV resets both the register and the internal counter
        // (the counter reset prevents a partial-period increment after the write).
        self.div = 0;
        self.div_counter = 0;
    }

    pub fn write_tima(&mut self, value: u8) { self.tima = value; }
    pub fn write_tma(&mut self, value: u8)  { self.tma = value; }

    pub fn write_tac(&mut self, value: u8) {
        // Changing clock select resets the TIMA counter to avoid a spurious
        // early increment when switching to a faster clock.
        let new_tac = TAC::from_byte(value);
        if new_tac.clock_select != self.tac.clock_select {
            self.tima_counter = 0;
        }
        self.tac = new_tac;
    }

    pub fn get_tima(&self) -> u8 { self.tima }
    pub fn get_div(&self)  -> u8 { self.div }
}

impl Default for Timer {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::MemoryBus;

    fn make_bus() -> MemoryBus {
        MemoryBus::new(vec![0u8; 32768])
    }

    #[test]
    fn test_tac_tima_periods() {
        assert_eq!(TAC::from_byte(0x00).tima_period(), 256);
        assert_eq!(TAC::from_byte(0x01).tima_period(), 4);
        assert_eq!(TAC::from_byte(0x02).tima_period(), 16);
        assert_eq!(TAC::from_byte(0x03).tima_period(), 64);
    }

    #[test]
    fn test_tac_enabled_flag() {
        assert!(!TAC::from_byte(0x00).enabled);
        assert!( TAC::from_byte(0x04).enabled);
        assert!( TAC::from_byte(0x07).enabled);
    }

    #[test]
    fn test_tac_roundtrip() {
        for v in 0u8..=7 {
            assert_eq!(TAC::from_byte(v).to_byte(), v);
        }
    }

    #[test]
    fn test_div_increments_every_64_cycles() {
        let mut timer = Timer::new();
        let mut bus = make_bus();

        for _ in 0..63 {
            timer.tick(&mut bus);
        }
        assert_eq!(timer.div, 0, "DIV should not increment before 64 cycles");

        timer.tick(&mut bus);
        assert_eq!(timer.div, 1, "DIV should increment after 64 cycles");

        for _ in 0..64 {
            timer.tick(&mut bus);
        }
        assert_eq!(timer.div, 2);
    }

    #[test]
    fn test_div_wraps() {
        let mut timer = Timer::new();
        let mut bus = make_bus();
        timer.div = 0xFF;
        // One more full DIV_PERIOD to trigger the increment
        for _ in 0..DIV_PERIOD {
            timer.tick(&mut bus);
        }
        assert_eq!(timer.div, 0);
    }

    #[test]
    fn test_write_div_resets_counter_and_register() {
        let mut timer = Timer::new();
        let mut bus = make_bus();
        // Advance most of the way through a DIV period
        for _ in 0..63 {
            timer.tick(&mut bus);
        }
        timer.write_div();
        assert_eq!(timer.div, 0);
        assert_eq!(timer.div_counter, 0);
        // After reset, should take another full 64 cycles before incrementing
        for _ in 0..63 {
            timer.tick(&mut bus);
        }
        assert_eq!(timer.div, 0, "DIV must not increment early after reset");
        timer.tick(&mut bus);
        assert_eq!(timer.div, 1);
    }

    #[test]
    fn test_tima_disabled_by_default() {
        let mut timer = Timer::new();
        let mut bus = make_bus();
        for _ in 0..1000 {
            timer.tick(&mut bus);
        }
        assert_eq!(timer.tima, 0, "TIMA must not increment when timer is disabled");
    }

    #[test]
    fn test_tima_increments_at_4096hz() {
        // TAC = 0x04: enabled, clock select 0 (4096 Hz → period 256)
        let mut timer = Timer::new();
        let mut bus = make_bus();
        timer.write_tac(0x04);

        for _ in 0..255 {
            timer.tick(&mut bus);
        }
        assert_eq!(timer.tima, 0, "TIMA must not increment before 256 cycles");

        timer.tick(&mut bus);
        assert_eq!(timer.tima, 1);
    }

    #[test]
    fn test_tima_overflow_reloads_tma_and_sets_if() {
        let mut timer = Timer::new();
        let mut bus = make_bus();
        timer.write_tac(0x04); // enabled, 4096 Hz
        timer.write_tma(0x42);
        timer.tima = 0xFF;
        timer.tima_counter = 255; // one tick away from firing

        timer.tick(&mut bus);

        assert_eq!(timer.tima, 0x42, "TIMA must reload from TMA on overflow");
        assert_eq!(bus.read(0xFF0F) & 0x04, 0x04, "timer interrupt bit must be set in IF");
    }

    #[test]
    fn test_tima_fastest_clock() {
        // TAC clock select 1: 262144 Hz → period 4
        let mut timer = Timer::new();
        let mut bus = make_bus();
        timer.write_tac(0x05); // enabled, clock select 1

        for _ in 0..4 {
            timer.tick(&mut bus);
        }
        assert_eq!(timer.tima, 1);

        for _ in 0..4 {
            timer.tick(&mut bus);
        }
        assert_eq!(timer.tima, 2);
    }
}
