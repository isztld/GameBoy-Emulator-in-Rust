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

    pub fn f_mut(&mut self) -> &mut Flags {
        unsafe { &mut *(self as *mut Registers as *mut Flags) }
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
    pub halted: bool,
    pub stopped: bool,
}

impl CPUState {
    pub fn new() -> Self {
        CPUState {
            registers: Registers::new(),
            ime: false,
            ime_pending: false,
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
    fn test_flags_initial_state() {
        let flags = Flags::new();
        // All flags should be 0 initially
        assert!(!flags.is_zero());
        assert!(!flags.is_subtraction());
        assert!(!flags.is_half_carry());
        assert!(!flags.is_carry());
    }

    #[test]
    fn test_flags_set_and_clear() {
        let mut flags = Flags::new();

        // Test Zero flag
        flags.set_zero(true);
        assert!(flags.is_zero());
        flags.set_zero(false);
        assert!(!flags.is_zero());

        // Test Subtraction flag
        flags.set_subtraction(true);
        assert!(flags.is_subtraction());
        flags.set_subtraction(false);
        assert!(!flags.is_subtraction());

        // Test Half Carry flag
        flags.set_half_carry(true);
        assert!(flags.is_half_carry());
        flags.set_half_carry(false);
        assert!(!flags.is_half_carry());

        // Test Carry flag
        flags.set_carry(true);
        assert!(flags.is_carry());
        flags.set_carry(false);
        assert!(!flags.is_carry());
    }

    #[test]
    fn test_flags_get_set() {
        let mut flags = Flags::new();

        flags.set_zero(true);
        flags.set_carry(true);

        // Z and C set
        assert_eq!(flags.get(), Flags::Z | Flags::C);

        // Overwrite with all bits set
        flags.set(0xFF);

        // Lower nibble must be zero (hardware behavior)
        assert_eq!(flags.get(), 0xF0);

        assert!(flags.is_zero());
        assert!(flags.is_subtraction());
        assert!(flags.is_half_carry());
        assert!(flags.is_carry());
    }

    #[test]
    fn test_flags_constants() {
        assert_eq!(Flags::Z, 0x80);
        assert_eq!(Flags::N, 0x40);
        assert_eq!(Flags::H, 0x20);
        assert_eq!(Flags::C, 0x10);
    }

    #[test]
    fn test_registers_new() {
        let regs = Registers::new();
        assert_eq!(regs.af, 0x0000);
        assert_eq!(regs.bc, 0x0000);
        assert_eq!(regs.de, 0x0000);
        assert_eq!(regs.hl, 0x0000);
        assert_eq!(regs.sp, 0xFFFE);
        assert_eq!(regs.pc, 0x0000);
    }

    #[test]
    fn test_registers_a_f() {
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

        // Verify AF register is properly composed (A=0xAB, F with Z and C set = 0x90)
        assert_eq!(regs.af, 0xAB90);
    }

    #[test]
    fn test_registers_bc() {
        let mut regs = Registers::new();

        regs.set_b(0xCD);
        regs.set_c(0xEF);
        assert_eq!(regs.b(), 0xCD);
        assert_eq!(regs.c(), 0xEF);
        assert_eq!(regs.bc, 0xCDEF);

        // Test setting full 16-bit value
        regs.bc = 0x1234;
        assert_eq!(regs.b(), 0x12);
        assert_eq!(regs.c(), 0x34);
    }

    #[test]
    fn test_registers_de() {
        let mut regs = Registers::new();

        regs.set_d(0x56);
        regs.set_e(0x78);
        assert_eq!(regs.d(), 0x56);
        assert_eq!(regs.e(), 0x78);
        assert_eq!(regs.de, 0x5678);
    }

    #[test]
    fn test_registers_hl() {
        let mut regs = Registers::new();

        regs.set_h(0x9A);
        regs.set_l(0xBC);
        assert_eq!(regs.h(), 0x9A);
        assert_eq!(regs.l(), 0xBC);
        assert_eq!(regs.hl, 0x9ABC);
    }

    #[test]
    fn test_registers_sp() {
        let mut regs = Registers::new();
        assert_eq!(regs.sp, 0xFFFE);

        regs.sp = 0xC000;
        assert_eq!(regs.sp, 0xC000);
    }

    #[test]
    fn test_registers_pc() {
        let mut regs = Registers::new();
        assert_eq!(regs.pc, 0x0000);

        regs.pc = 0x0100;
        assert_eq!(regs.pc, 0x0100);
    }

    #[test]
    fn test_all_registers_together() {
        let mut regs = Registers::new();
        regs.af = 0x1234;
        regs.bc = 0x5678;
        regs.de = 0x9ABC;
        regs.hl = 0xDEF0;
        regs.sp = 0xFF00;
        regs.pc = 0x0100;

        assert_eq!(regs.a(), 0x12);
        assert_eq!(regs.f().get(), 0x30); // F register only uses upper 4 bits (hardware behavior)
        assert_eq!(regs.b(), 0x56);
        assert_eq!(regs.c(), 0x78);
        assert_eq!(regs.d(), 0x9A);
        assert_eq!(regs.e(), 0xBC);
        assert_eq!(regs.h(), 0xDE);
        assert_eq!(regs.l(), 0xF0);
        assert_eq!(regs.sp, 0xFF00);
        assert_eq!(regs.pc, 0x0100);
    }

    #[test]
    fn test_cpu_state() {
        let state = CPUState::new();
        assert_eq!(state.registers.pc, 0x0000);
        assert!(!state.ime);
        assert!(!state.halted);
        assert!(!state.stopped);
    }

    #[test]
    fn test_registers_default() {
        let regs = Registers::default();
        // After reset, PC should be 0x0100 for GameBoy boot
        // But Registers::new() initializes to 0, CPU::reset sets PC to 0x0100
        assert_eq!(regs.pc, 0x0000);
        assert_eq!(regs.sp, 0xFFFE);
    }
}
