/// GameBoy CPU Registers
///
/// The GameBoy uses a SM83 CPU (8-bit, GBZ80 compatible).
/// It has the following registers:
/// - AF: Accumulator & Flags (16-bit)
/// - BC: General purpose (16-bit)
/// - DE: General purpose (16-bit)
/// - HL: General purpose (16-bit)
/// - SP: Stack Pointer (16-bit)
/// - PC: Program Counter (16-bit)

/// Flags register (lower 8 bits of AF)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Flags(u8);

impl Flags {
    pub const Z: u8 = 1 << 7; // Zero flag
    pub const N: u8 = 1 << 6; // Subtraction flag (BCD)
    pub const H: u8 = 1 << 5; // Half Carry flag (BCD)
    pub const C: u8 = 1 << 4; // Carry flag

    pub fn new() -> Self {
        Flags(0)
    }

    pub fn is_zero(&self) -> bool {
        self.0 & Self::Z != 0
    }

    pub fn set_zero(&mut self, value: bool) {
        if value {
            self.0 |= Self::Z;
        } else {
            self.0 &= !Self::Z;
        }
    }

    pub fn is_subtraction(&self) -> bool {
        self.0 & Self::N != 0
    }

    pub fn set_subtraction(&mut self, value: bool) {
        if value {
            self.0 |= Self::N;
        } else {
            self.0 &= !Self::N;
        }
    }

    pub fn is_half_carry(&self) -> bool {
        self.0 & Self::H != 0
    }

    pub fn set_half_carry(&mut self, value: bool) {
        if value {
            self.0 |= Self::H;
        } else {
            self.0 &= !Self::H;
        }
    }

    pub fn is_carry(&self) -> bool {
        self.0 & Self::C != 0
    }

    pub fn set_carry(&mut self, value: bool) {
        if value {
            self.0 |= Self::C;
        } else {
            self.0 &= !Self::C;
        }
    }

    pub fn get(&self) -> u8 {
        self.0
    }

    pub fn set(&mut self, value: u8) {
        self.0 = value;
    }
}

/// CPU Registers
#[derive(Debug, Clone, Copy)]
pub struct Registers {
    pub af: u16,
    pub bc: u16,
    pub de: u16,
    pub hl: u16,
    pub sp: u16,
    pub pc: u16,
}

impl Registers {
    pub fn new() -> Self {
        Registers {
            af: 0x0000,
            bc: 0x0000,
            de: 0x0000,
            hl: 0x0000,
            sp: 0xFFFE,
            pc: 0x0000,
        }
    }

    // A register (high byte of AF)
    pub fn a(&self) -> u8 {
        (self.af >> 8) as u8
    }

    pub fn set_a(&mut self, value: u8) {
        self.af = (self.af & 0x00FF) | ((value as u16) << 8);
    }

    // F register (low byte of AF)
    pub fn f(&self) -> Flags {
        Flags(self.af as u8)
    }

    pub fn set_f(&mut self, value: Flags) {
        self.af = (self.af & 0xFF00) | value.get() as u16;
    }

    // BC register pair
    pub fn b(&self) -> u8 {
        (self.bc >> 8) as u8
    }

    pub fn set_b(&mut self, value: u8) {
        self.bc = (self.bc & 0x00FF) | ((value as u16) << 8);
    }

    pub fn c(&self) -> u8 {
        self.bc as u8
    }

    pub fn set_c(&mut self, value: u8) {
        self.bc = (self.bc & 0xFF00) | (value as u16);
    }

    // DE register pair
    pub fn d(&self) -> u8 {
        (self.de >> 8) as u8
    }

    pub fn set_d(&mut self, value: u8) {
        self.de = (self.de & 0x00FF) | ((value as u16) << 8);
    }

    pub fn e(&self) -> u8 {
        self.de as u8
    }

    pub fn set_e(&mut self, value: u8) {
        self.de = (self.de & 0xFF00) | (value as u16);
    }

    // HL register pair
    pub fn h(&self) -> u8 {
        (self.hl >> 8) as u8
    }

    pub fn set_h(&mut self, value: u8) {
        self.hl = (self.hl & 0x00FF) | ((value as u16) << 8);
    }

    pub fn l(&self) -> u8 {
        self.hl as u8
    }

    pub fn set_l(&mut self, value: u8) {
        self.hl = (self.hl & 0xFF00) | (value as u16);
    }
}

impl Default for Registers {
    fn default() -> Self {
        Self::new()
    }
}

/// CPU State
#[derive(Debug, Clone, Copy)]
pub struct CPUState {
    pub registers: Registers,
    pub ime: bool, // Interrupt Master Enable
    pub halted: bool,
    pub stopped: bool,
}

impl CPUState {
    pub fn new() -> Self {
        CPUState {
            registers: Registers::new(),
            ime: false,
            halted: false,
            stopped: false,
        }
    }
}

impl Default for CPUState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flags() {
        let mut flags = Flags::new();
        assert!(!flags.is_zero());
        assert!(!flags.is_carry());

        flags.set_zero(true);
        assert!(flags.is_zero());

        flags.set_carry(true);
        assert!(flags.is_carry());

        flags.set_zero(false);
        assert!(!flags.is_zero());
    }

    #[test]
    fn test_registers() {
        let mut regs = Registers::new();

        // Test A register
        regs.set_a(0xAB);
        assert_eq!(regs.a(), 0xAB);

        // Test F register
        let mut flags = Flags::new();
        flags.set_zero(true);
        flags.set_carry(true);
        regs.set_f(flags);
        assert!(regs.f().is_zero());
        assert!(regs.f().is_carry());

        // Test BC register pair
        regs.set_b(0xCD);
        regs.set_c(0xEF);
        assert_eq!(regs.b(), 0xCD);
        assert_eq!(regs.c(), 0xEF);
        assert_eq!(regs.bc, 0xCDEF);

        // Test HL register pair
        regs.set_h(0x12);
        regs.set_l(0x34);
        assert_eq!(regs.h(), 0x12);
        assert_eq!(regs.l(), 0x34);
        assert_eq!(regs.hl, 0x1234);
    }
}
