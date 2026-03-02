/// Register utility functions for instruction executors
use crate::memory::MemoryBus;
use crate::cpu::instructions::{R16Register, R8Register};
use crate::cpu::registers::Registers;

/// Get 16-bit register value
pub fn r16(registers: &Registers, reg: R16Register) -> u16 {
    match reg {
        R16Register::BC => registers.bc,
        R16Register::DE => registers.de,
        R16Register::HL => registers.hl,
        R16Register::SP => registers.sp,
        R16Register::AF => registers.af,
    }
}

/// Set 16-bit register value
pub fn set_r16(registers: &mut Registers, reg: R16Register, value: u16) {
    match reg {
        R16Register::BC => registers.bc = value,
        R16Register::DE => registers.de = value,
        R16Register::HL => registers.hl = value,
        R16Register::SP => registers.sp = value,
        R16Register::AF => {
            // Lower 4 bits of F are always zero in hardware
            registers.af = value & 0xFFF0;
        }
    }
}

/// Get 8-bit register value (or memory at HL)
pub fn get_r8(registers: &Registers, bus: &mut MemoryBus, reg: R8Register) -> u8 {
    match reg {
        R8Register::B => registers.b(),
        R8Register::C => registers.c(),
        R8Register::D => registers.d(),
        R8Register::E => registers.e(),
        R8Register::H => registers.h(),
        R8Register::L => registers.l(),
        R8Register::HL => bus.read(registers.hl),
        R8Register::A => registers.a(),
    }
}

/// Set 8-bit register value (or memory at HL)
pub fn set_r8(registers: &mut Registers, bus: &mut MemoryBus, reg: R8Register, value: u8) {
    match reg {
        R8Register::B => registers.set_b(value),
        R8Register::C => registers.set_c(value),
        R8Register::D => registers.set_d(value),
        R8Register::E => registers.set_e(value),
        R8Register::H => registers.set_h(value),
        R8Register::L => registers.set_l(value),
        R8Register::HL => bus.write(registers.hl, value),
        R8Register::A => registers.set_a(value),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::MemoryBus;

    #[test]
    fn test_r16_get() {
        let mut registers = Registers::new();
        registers.bc = 0x1234;
        registers.de = 0x5678;
        registers.hl = 0x9ABC;
        registers.sp = 0xFFFE;
        registers.af = 0x1234;

        assert_eq!(r16(&registers, R16Register::BC), 0x1234);
        assert_eq!(r16(&registers, R16Register::DE), 0x5678);
        assert_eq!(r16(&registers, R16Register::HL), 0x9ABC);
        assert_eq!(r16(&registers, R16Register::SP), 0xFFFE);
        assert_eq!(r16(&registers, R16Register::AF), 0x1234);
    }

    #[test]
    fn test_set_r16() {
        let mut registers = Registers::new();

        set_r16(&mut registers, R16Register::BC, 0x1234);
        assert_eq!(registers.bc, 0x1234);

        set_r16(&mut registers, R16Register::DE, 0x5678);
        assert_eq!(registers.de, 0x5678);

        set_r16(&mut registers, R16Register::HL, 0x9ABC);
        assert_eq!(registers.hl, 0x9ABC);

        set_r16(&mut registers, R16Register::SP, 0xFF00);
        assert_eq!(registers.sp, 0xFF00);

        // AF register should have lower 4 bits of F cleared
        set_r16(&mut registers, R16Register::AF, 0xABCD);
        assert_eq!(registers.af, 0xABC0);
    }

    #[test]
    fn test_set_r16_af_preserves_high_nibble() {
        let mut registers = Registers::new();

        // Set F with various flag bits
        registers.set_a(0xFF);
        registers.f_mut().set_zero(true);
        registers.f_mut().set_carry(true);

        let af = registers.af;
        set_r16(&mut registers, R16Register::AF, 0x1234);

        // Should preserve high nibble of F (the flags)
        assert_eq!(registers.af & 0xF0, af & 0xF0);
    }

    #[test]
    fn test_get_r8_normal_registers() {
        let mut registers = Registers::new();
        registers.set_b(0xAB);
        registers.set_c(0xCD);
        registers.set_d(0xEF);
        registers.set_e(0x12);
        registers.set_h(0x34);
        registers.set_l(0x56);
        registers.set_a(0x78);

        let mut bus = MemoryBus::new(vec![0; 32768]);

        assert_eq!(get_r8(&registers, &mut bus, R8Register::B), 0xAB);
        assert_eq!(get_r8(&registers, &mut bus, R8Register::C), 0xCD);
        assert_eq!(get_r8(&registers, &mut bus, R8Register::D), 0xEF);
        assert_eq!(get_r8(&registers, &mut bus, R8Register::E), 0x12);
        assert_eq!(get_r8(&registers, &mut bus, R8Register::H), 0x34);
        assert_eq!(get_r8(&registers, &mut bus, R8Register::L), 0x56);
        assert_eq!(get_r8(&registers, &mut bus, R8Register::A), 0x78);
    }

    #[test]
    fn test_get_r8_hl_memory() {
        let mut registers = Registers::new();
        registers.hl = 0xC000;

        let mut bus = MemoryBus::new(vec![0; 32768]);
        bus.write(0xC000, 0x42);

        assert_eq!(get_r8(&registers, &mut bus, R8Register::HL), 0x42);
    }

    #[test]
    fn test_set_r8_normal_registers() {
        let mut registers = Registers::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);

        set_r8(&mut registers, &mut bus, R8Register::B, 0xAB);
        assert_eq!(registers.b(), 0xAB);

        set_r8(&mut registers, &mut bus, R8Register::C, 0xCD);
        assert_eq!(registers.c(), 0xCD);

        set_r8(&mut registers, &mut bus, R8Register::D, 0xEF);
        assert_eq!(registers.d(), 0xEF);

        set_r8(&mut registers, &mut bus, R8Register::E, 0x12);
        assert_eq!(registers.e(), 0x12);

        set_r8(&mut registers, &mut bus, R8Register::H, 0x34);
        assert_eq!(registers.h(), 0x34);

        set_r8(&mut registers, &mut bus, R8Register::L, 0x56);
        assert_eq!(registers.l(), 0x56);

        set_r8(&mut registers, &mut bus, R8Register::A, 0x78);
        assert_eq!(registers.a(), 0x78);
    }

    #[test]
    fn test_set_r8_hl_memory() {
        let mut registers = Registers::new();
        registers.hl = 0xC000;

        let mut bus = MemoryBus::new(vec![0; 32768]);

        set_r8(&mut registers, &mut bus, R8Register::HL, 0x42);

        assert_eq!(bus.read(0xC000), 0x42);
    }
}
