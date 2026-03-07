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
        self.0 = value & 0xF0; // lower 4 bits forced to 0
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
        Flags((self.af as u8) & 0xF0)
    }

    pub fn set_f(&mut self, value: Flags) {
        self.af = (self.af & 0xFF00) | ((value.get() & 0xF0) as u16);
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

    pub fn modify_f(&mut self, op: impl FnOnce(&mut Flags)) {
        let mut f = self.f();
        op(&mut f);
        self.set_f(f);
    }

    pub fn r16(&self, reg: crate::cpu::instructions::R16Register) -> u16 {
        match reg {
            crate::cpu::instructions::R16Register::BC => self.bc,
            crate::cpu::instructions::R16Register::DE => self.de,
            crate::cpu::instructions::R16Register::HL => self.hl,
            crate::cpu::instructions::R16Register::SP => self.sp,
            crate::cpu::instructions::R16Register::AF => self.af,
        }
    }

    pub fn set_r16(&mut self, reg: crate::cpu::instructions::R16Register, value: u16) {
        match reg {
            crate::cpu::instructions::R16Register::BC => self.bc = value,
            crate::cpu::instructions::R16Register::DE => self.de = value,
            crate::cpu::instructions::R16Register::HL => self.hl = value,
            crate::cpu::instructions::R16Register::SP => self.sp = value,
            crate::cpu::instructions::R16Register::AF => self.af = value,
        }
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
    pub ime_pending: bool, // EI sets this; IME is enabled after the next instruction
}

impl CPUState {
    pub fn new() -> Self {
        CPUState {
            registers: Registers::new(),
            ime: false,
            ime_pending: false,
        }
    }
}

impl Default for CPUState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[path = "registers_tests.rs"]
mod tests;
