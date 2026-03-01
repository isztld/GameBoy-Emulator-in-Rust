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
