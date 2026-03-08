/// Instruction decoding module
///
/// This module handles decoding opcodes into Instructions.

use crate::memory::MemoryBus;
use crate::cpu::instructions::{Instruction, R8Register, R16Register, R16Mem, Condition};

/// Decode an instruction from the opcode
pub fn decode_instruction(
    bus: &MemoryBus,
    pc: u16,
    opcode: u8,
) -> (Instruction, u8) {
    match opcode {
        0x00 => (Instruction::NOP, 1),
        0x01 => {
            let value = read_u16(bus, pc);
            (Instruction::LdR16Imm16 { dest: R16Register::BC, value }, 3)
        }
        0x02 => (Instruction::LdIndR16A { src: R16Mem::BC }, 1),
        0x03 => (Instruction::IncR16 { reg: R16Register::BC }, 1),
        0x04 => (Instruction::IncR8 { reg: R8Register::B }, 1),
        0x05 => (Instruction::DecR8 { reg: R8Register::B }, 1),
        0x06 => (Instruction::LdR8Imm8 { dest: R8Register::B, value: bus.read(pc + 1) }, 2),
        0x07 => (Instruction::RLCA, 1),
        0x08 => (Instruction::LdIndImm16Sp { address: read_u16(bus, pc) }, 3),
        0x09 => (Instruction::AddHlR16 { reg: R16Register::BC }, 1),
        0x0A => (Instruction::LdAIndR16 { dest: R16Mem::BC }, 1),
        0x0B => (Instruction::DecR16 { reg: R16Register::BC }, 1),
        0x0C => (Instruction::IncR8 { reg: R8Register::C }, 1),
        0x0D => (Instruction::DecR8 { reg: R8Register::C }, 1),
        0x0E => (Instruction::LdR8Imm8 { dest: R8Register::C, value: bus.read(pc + 1) }, 2),
        0x0F => (Instruction::RRCA, 1),
        0x10 => (Instruction::STOP, 2), // STOP is a 2-byte instruction (0x10 0x00)
        0x11 => {
            let value = read_u16(bus, pc);
            (Instruction::LdR16Imm16 { dest: R16Register::DE, value }, 3)
        }
        0x12 => (Instruction::LdIndR16A { src: R16Mem::DE }, 1),
        0x13 => (Instruction::IncR16 { reg: R16Register::DE }, 1),
        0x14 => (Instruction::IncR8 { reg: R8Register::D }, 1),
        0x15 => (Instruction::DecR8 { reg: R8Register::D }, 1),
        0x16 => (Instruction::LdR8Imm8 { dest: R8Register::D, value: bus.read(pc + 1) }, 2),
        0x17 => (Instruction::RLA, 1),
        0x18 => (Instruction::JrImm8 { offset: bus.read(pc + 1) as i8 }, 2),
        0x19 => (Instruction::AddHlR16 { reg: R16Register::DE }, 1),
        0x1A => (Instruction::LdAIndR16 { dest: R16Mem::DE }, 1),
        0x1B => (Instruction::DecR16 { reg: R16Register::DE }, 1),
        0x1C => (Instruction::IncR8 { reg: R8Register::E }, 1),
        0x1D => (Instruction::DecR8 { reg: R8Register::E }, 1),
        0x1E => (Instruction::LdR8Imm8 { dest: R8Register::E, value: bus.read(pc + 1) }, 2),
        0x1F => (Instruction::RRA, 1),
        0x20 => (Instruction::JrCondImm8 { cond: Condition::NZ, offset: bus.read(pc + 1) as i8 }, 2),
        0x21 => {
            let value = read_u16(bus, pc);
            (Instruction::LdR16Imm16 { dest: R16Register::HL, value }, 3)
        }
        0x22 => (Instruction::LdIndR16A { src: R16Mem::HLPlus }, 1),
        0x23 => (Instruction::IncR16 { reg: R16Register::HL }, 1),
        0x24 => (Instruction::IncR8 { reg: R8Register::H }, 1),
        0x25 => (Instruction::DecR8 { reg: R8Register::H }, 1),
        0x26 => (Instruction::LdR8Imm8 { dest: R8Register::H, value: bus.read(pc + 1) }, 2),
        0x27 => (Instruction::DAA, 1),
        0x28 => (Instruction::JrCondImm8 { cond: Condition::Z, offset: bus.read(pc + 1) as i8 }, 2),
        0x29 => (Instruction::AddHlR16 { reg: R16Register::HL }, 1),
        0x2A => (Instruction::LdAIndR16 { dest: R16Mem::HLPlus }, 1),
        0x2B => (Instruction::DecR16 { reg: R16Register::HL }, 1),
        0x2C => (Instruction::IncR8 { reg: R8Register::L }, 1),
        0x2D => (Instruction::DecR8 { reg: R8Register::L }, 1),
        0x2E => (Instruction::LdR8Imm8 { dest: R8Register::L, value: bus.read(pc + 1) }, 2),
        0x2F => (Instruction::CPL, 1),
        0x30 => (Instruction::JrCondImm8 { cond: Condition::NC, offset: bus.read(pc + 1) as i8 }, 2),
        0x31 => {
            let value = read_u16(bus, pc);
            (Instruction::LdR16Imm16 { dest: R16Register::SP, value }, 3)
        }
        0x32 => (Instruction::LdIndR16A { src: R16Mem::HLMinus }, 1),
        0x33 => (Instruction::IncR16 { reg: R16Register::SP }, 1),
        0x34 => (Instruction::IncR8 { reg: R8Register::HL }, 1),
        0x35 => (Instruction::DecR8 { reg: R8Register::HL }, 1),
        0x36 => (Instruction::LdR8Imm8 { dest: R8Register::HL, value: bus.read(pc + 1) }, 2),
        0x37 => (Instruction::SCF, 1),
        0x38 => (Instruction::JrCondImm8 { cond: Condition::C, offset: bus.read(pc + 1) as i8 }, 2),
        0x39 => (Instruction::AddHlR16 { reg: R16Register::SP }, 1),
        0x3A => (Instruction::LdAIndR16 { dest: R16Mem::HLMinus }, 1),
        0x3B => (Instruction::DecR16 { reg: R16Register::SP }, 1),
        0x3C => (Instruction::IncR8 { reg: R8Register::A }, 1),
        0x3D => (Instruction::DecR8 { reg: R8Register::A }, 1),
        0x3E => (Instruction::LdR8Imm8 { dest: R8Register::A, value: bus.read(pc + 1) }, 2),
        0x3F => (Instruction::CCF, 1),

        // Block 1: 0x40–0x7F — LD r8, r8
        // Encoding: 0b01_DDD_SSS — bits 3-5 are dest, bits 0-2 are src.
        // 0x76 is HALT, not LD (HL),(HL).
        0x76 => (Instruction::HALT, 1),
        0x40..=0x7F => {
            let dest = R8Register::from_byte((opcode >> 3) & 0x07);
            let src  = R8Register::from_byte(opcode & 0x07);
            (Instruction::LdR8R8 { dest, src }, 1)
        }

        // Block 2: 0x80–0xBF — 8-bit arithmetic
        0x80..=0xBF => {
            let reg = R8Register::from_byte(opcode & 0x07);
            match opcode & 0xF8 {
                0x80 => (Instruction::AddAR8 { reg }, 1),
                0x88 => (Instruction::AdcAR8 { reg }, 1),
                0x90 => (Instruction::SubAR8 { reg }, 1),
                0x98 => (Instruction::SbcAR8 { reg }, 1),
                0xA0 => (Instruction::AndAR8 { reg }, 1),
                0xA8 => (Instruction::XorAR8 { reg }, 1),
                0xB0 => (Instruction::OrAR8 { reg }, 1),
                0xB8 => (Instruction::CpAR8 { reg }, 1),
                _    => (Instruction::NOP, 1),
            }
        }

        // Block 3: 0xC0–0xFF
        0xC0 => (Instruction::RetCond { cond: Condition::NZ }, 1),
        0xC1 => (Instruction::PopR16 { reg: R16Register::BC }, 1),
        0xC2 => (Instruction::JpCondImm16 { cond: Condition::NZ, address: read_u16(bus, pc) }, 3),
        0xC3 => (Instruction::JpImm16 { address: read_u16(bus, pc) }, 3),
        0xC4 => (Instruction::CallCondImm16 { cond: Condition::NZ, address: read_u16(bus, pc) }, 3),
        0xC5 => (Instruction::PushR16 { reg: R16Register::BC }, 1),
        0xC6 => (Instruction::AddAImm8 { value: bus.read(pc + 1) }, 2),
        0xC7 => (Instruction::RST { target: 0x00 }, 1),
        0xC8 => (Instruction::RetCond { cond: Condition::Z }, 1),
        0xC9 => (Instruction::RET, 1),
        0xCA => (Instruction::JpCondImm16 { cond: Condition::Z, address: read_u16(bus, pc) }, 3),
        0xCB => {
            let cb_opcode = bus.read(pc + 1);
            let cb_instr = crate::cpu::instructions::CBInstruction::from_byte(opcode, cb_opcode);
            (Instruction::CB { cb_instr }, 2)
        }
        0xCC => (Instruction::CallCondImm16 { cond: Condition::Z, address: read_u16(bus, pc) }, 3),
        0xCD => (Instruction::CallImm16 { address: read_u16(bus, pc) }, 3),
        0xCE => (Instruction::AdcAImm8 { value: bus.read(pc + 1) }, 2),
        0xCF => (Instruction::RST { target: 0x08 }, 1),
        0xD0 => (Instruction::RetCond { cond: Condition::NC }, 1),
        0xD1 => (Instruction::PopR16 { reg: R16Register::DE }, 1),
        0xD2 => (Instruction::JpCondImm16 { cond: Condition::NC, address: read_u16(bus, pc) }, 3),
        // 0xD3 — invalid
        0xD4 => (Instruction::CallCondImm16 { cond: Condition::NC, address: read_u16(bus, pc) }, 3),
        0xD5 => (Instruction::PushR16 { reg: R16Register::DE }, 1),
        0xD6 => (Instruction::SubAImm8 { value: bus.read(pc + 1) }, 2),
        0xD7 => (Instruction::RST { target: 0x10 }, 1),
        0xD8 => (Instruction::RetCond { cond: Condition::C }, 1),
        0xD9 => (Instruction::RETI, 1),
        0xDA => (Instruction::JpCondImm16 { cond: Condition::C, address: read_u16(bus, pc) }, 3),
        // 0xDB — invalid
        0xDC => (Instruction::CallCondImm16 { cond: Condition::C, address: read_u16(bus, pc) }, 3),
        // 0xDD — invalid
        0xDE => (Instruction::SbcAImm8 { value: bus.read(pc + 1) }, 2),
        0xDF => (Instruction::RST { target: 0x18 }, 1),
        0xE0 => (Instruction::LdhIndImm8A { address: bus.read(pc + 1) }, 2),
        0xE1 => (Instruction::PopR16 { reg: R16Register::HL }, 1),
        0xE2 => (Instruction::LdhIndCA, 1),
        // 0xE3, 0xE4 — invalid
        0xE5 => (Instruction::PushR16 { reg: R16Register::HL }, 1),
        0xE6 => (Instruction::AndAImm8 { value: bus.read(pc + 1) }, 2),
        0xE7 => (Instruction::RST { target: 0x20 }, 1),
        0xE8 => (Instruction::AddSpImm8 { value: bus.read(pc + 1) as i8 }, 2),
        0xE9 => (Instruction::JpHl, 1),
        0xEA => (Instruction::LdIndImm16A { address: read_u16(bus, pc) }, 3),
        // 0xEB, 0xEC, 0xED — invalid
        0xEE => (Instruction::XorAImm8 { value: bus.read(pc + 1) }, 2),
        0xEF => (Instruction::RST { target: 0x28 }, 1),
        0xF0 => (Instruction::LdhAIndImm8 { address: bus.read(pc + 1) }, 2),
        0xF1 => (Instruction::PopR16 { reg: R16Register::AF }, 1),
        0xF2 => (Instruction::LdhAC, 1),
        0xF3 => (Instruction::DI, 1),
        // 0xF4 — invalid
        0xF5 => (Instruction::PushR16 { reg: R16Register::AF }, 1),
        0xF6 => (Instruction::OrAImm8 { value: bus.read(pc + 1) }, 2),
        0xF7 => (Instruction::RST { target: 0x30 }, 1),
        0xF8 => (Instruction::LdHlSpImm8 { value: bus.read(pc + 1) as i8 }, 2),
        0xF9 => (Instruction::LdSpHl, 1),
        0xFA => (Instruction::LdAIndImm16 { address: read_u16(bus, pc) }, 3),
        0xFB => (Instruction::EI, 1),
        // 0xFC, 0xFD — invalid
        0xFE => (Instruction::CpAImm8 { value: bus.read(pc + 1) }, 2),
        0xFF => (Instruction::RST { target: 0x38 }, 1),
        _ => (Instruction::NOP, 1), // invalid opcodes
    }
}

/// Read a little-endian u16 from pc+1 and pc+2.
fn read_u16(bus: &MemoryBus, pc: u16) -> u16 {
    let lo = bus.read(pc.wrapping_add(1)) as u16;
    let hi = bus.read(pc.wrapping_add(2)) as u16;
    (hi << 8) | lo
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::MemoryBus;

    fn make_bus() -> MemoryBus {
        MemoryBus::new(vec![0u8; 32768])
    }

    /// Write a fake instruction into WRAM starting at 0xC000 and decode from there.
    /// `bytes` is the full instruction including opcode at index 0.
    fn decode_with_operands(bytes: &[u8]) -> (Instruction, u8) {
        let mut bus = make_bus();
        let pc: u16 = 0xC000;
        for (i, &b) in bytes.iter().enumerate() {
            bus.write(pc + i as u16, b);
        }
        decode_instruction(&bus, pc, bytes[0])
    }

    fn decode_no_operands(opcode: u8) -> (Instruction, u8) {
        decode_with_operands(&[opcode])
    }

    // -----------------------------------------------------------------------
    // Block 0
    // -----------------------------------------------------------------------

    #[test]
    fn test_decode_nop() {
        let (instr, len) = decode_no_operands(0x00);
        assert_eq!(instr, Instruction::NOP);
        assert_eq!(len, 1);
    }

    #[test]
    fn test_decode_stop_is_2_bytes() {
        let (instr, len) = decode_no_operands(0x10);
        assert_eq!(instr, Instruction::STOP);
        assert_eq!(len, 2);
    }

    #[test]
    fn test_decode_ld_r16_imm16_all() {
        for (opcode, expected_dest) in [
            (0x01, R16Register::BC),
            (0x11, R16Register::DE),
            (0x21, R16Register::HL),
            (0x31, R16Register::SP),
        ] {
            let (instr, len) = decode_with_operands(&[opcode, 0x34, 0x12]);
            assert_eq!(len, 3, "opcode {opcode:#04X}");
            match instr {
                Instruction::LdR16Imm16 { dest, value } => {
                    assert_eq!(dest, expected_dest, "opcode {opcode:#04X}");
                    assert_eq!(value, 0x1234, "opcode {opcode:#04X}");
                }
                _ => panic!("opcode {opcode:#04X}: expected LdR16Imm16"),
            }
        }
    }

    #[test]
    fn test_decode_ld_ind_r16_a() {
        assert_eq!(decode_no_operands(0x02).0, Instruction::LdIndR16A { src: R16Mem::BC });
        assert_eq!(decode_no_operands(0x12).0, Instruction::LdIndR16A { src: R16Mem::DE });
        assert_eq!(decode_no_operands(0x22).0, Instruction::LdIndR16A { src: R16Mem::HLPlus });
        assert_eq!(decode_no_operands(0x32).0, Instruction::LdIndR16A { src: R16Mem::HLMinus });
    }

    #[test]
    fn test_decode_ld_a_ind_r16() {
        assert_eq!(decode_no_operands(0x0A).0, Instruction::LdAIndR16 { dest: R16Mem::BC });
        assert_eq!(decode_no_operands(0x1A).0, Instruction::LdAIndR16 { dest: R16Mem::DE });
        assert_eq!(decode_no_operands(0x2A).0, Instruction::LdAIndR16 { dest: R16Mem::HLPlus });
        assert_eq!(decode_no_operands(0x3A).0, Instruction::LdAIndR16 { dest: R16Mem::HLMinus });
    }

    #[test]
    fn test_decode_ld_ind_imm16_sp() {
        let (instr, len) = decode_with_operands(&[0x08, 0x34, 0x12]);
        assert_eq!(len, 3);
        assert_eq!(instr, Instruction::LdIndImm16Sp { address: 0x1234 });
    }

    #[test]
    fn test_decode_inc_dec_r16() {
        assert_eq!(decode_no_operands(0x03).0, Instruction::IncR16 { reg: R16Register::BC });
        assert_eq!(decode_no_operands(0x13).0, Instruction::IncR16 { reg: R16Register::DE });
        assert_eq!(decode_no_operands(0x23).0, Instruction::IncR16 { reg: R16Register::HL });
        assert_eq!(decode_no_operands(0x33).0, Instruction::IncR16 { reg: R16Register::SP });
        assert_eq!(decode_no_operands(0x0B).0, Instruction::DecR16 { reg: R16Register::BC });
        assert_eq!(decode_no_operands(0x1B).0, Instruction::DecR16 { reg: R16Register::DE });
        assert_eq!(decode_no_operands(0x2B).0, Instruction::DecR16 { reg: R16Register::HL });
        assert_eq!(decode_no_operands(0x3B).0, Instruction::DecR16 { reg: R16Register::SP });
    }

    #[test]
    fn test_decode_inc_dec_r8_all() {
        for (op, reg) in [
            (0x04, R8Register::B),  (0x0C, R8Register::C),
            (0x14, R8Register::D),  (0x1C, R8Register::E),
            (0x24, R8Register::H),  (0x2C, R8Register::L),
            (0x34, R8Register::HL), (0x3C, R8Register::A),
        ] {
            assert_eq!(decode_no_operands(op).0, Instruction::IncR8 { reg }, "INC {op:#04X}");
        }
        for (op, reg) in [
            (0x05, R8Register::B),  (0x0D, R8Register::C),
            (0x15, R8Register::D),  (0x1D, R8Register::E),
            (0x25, R8Register::H),  (0x2D, R8Register::L),
            (0x35, R8Register::HL), (0x3D, R8Register::A),
        ] {
            assert_eq!(decode_no_operands(op).0, Instruction::DecR8 { reg }, "DEC {op:#04X}");
        }
    }

    #[test]
    fn test_decode_ld_r8_imm8_all() {
        for (op, reg) in [
            (0x06, R8Register::B),  (0x0E, R8Register::C),
            (0x16, R8Register::D),  (0x1E, R8Register::E),
            (0x26, R8Register::H),  (0x2E, R8Register::L),
            (0x36, R8Register::HL), (0x3E, R8Register::A),
        ] {
            let (instr, len) = decode_with_operands(&[op, 0x42]);
            assert_eq!(len, 2, "op={op:#04X}");
            assert_eq!(instr, Instruction::LdR8Imm8 { dest: reg, value: 0x42 }, "op={op:#04X}");
        }
    }

    #[test]
    fn test_decode_misc_block0() {
        assert_eq!(decode_no_operands(0x07).0, Instruction::RLCA);
        assert_eq!(decode_no_operands(0x0F).0, Instruction::RRCA);
        assert_eq!(decode_no_operands(0x17).0, Instruction::RLA);
        assert_eq!(decode_no_operands(0x1F).0, Instruction::RRA);
        assert_eq!(decode_no_operands(0x27).0, Instruction::DAA);
        assert_eq!(decode_no_operands(0x2F).0, Instruction::CPL);
        assert_eq!(decode_no_operands(0x37).0, Instruction::SCF);
        assert_eq!(decode_no_operands(0x3F).0, Instruction::CCF);
    }

    #[test]
    fn test_decode_add_hl_r16() {
        assert_eq!(decode_no_operands(0x09).0, Instruction::AddHlR16 { reg: R16Register::BC });
        assert_eq!(decode_no_operands(0x19).0, Instruction::AddHlR16 { reg: R16Register::DE });
        assert_eq!(decode_no_operands(0x29).0, Instruction::AddHlR16 { reg: R16Register::HL });
        assert_eq!(decode_no_operands(0x39).0, Instruction::AddHlR16 { reg: R16Register::SP });
    }

    #[test]
    fn test_decode_jr_instructions() {
        assert_eq!(decode_with_operands(&[0x18, 0x05]).0, Instruction::JrImm8 { offset: 5 });
        assert_eq!(decode_with_operands(&[0x20, 0x05]).0, Instruction::JrCondImm8 { cond: Condition::NZ, offset: 5 });
        assert_eq!(decode_with_operands(&[0x28, 0x05]).0, Instruction::JrCondImm8 { cond: Condition::Z,  offset: 5 });
        assert_eq!(decode_with_operands(&[0x30, 0x05]).0, Instruction::JrCondImm8 { cond: Condition::NC, offset: 5 });
        assert_eq!(decode_with_operands(&[0x38, 0x05]).0, Instruction::JrCondImm8 { cond: Condition::C,  offset: 5 });
    }

    // -----------------------------------------------------------------------
    // Block 1
    // -----------------------------------------------------------------------

    #[test]
    fn test_decode_halt() {
        assert_eq!(decode_no_operands(0x76).0, Instruction::HALT);
    }

    #[test]
    fn test_decode_ld_r8_r8_all() {
        for dest_idx in 0u8..=7 {
            for src_idx in 0u8..=7 {
                let opcode = 0x40u8 | (dest_idx << 3) | src_idx;
                if opcode == 0x76 { continue; }
                let (instr, len) = decode_no_operands(opcode);
                assert_eq!(len, 1, "opcode={opcode:#04X}");
                match instr {
                    Instruction::LdR8R8 { dest, src } => {
                        assert_eq!(dest, R8Register::from_byte(dest_idx), "opcode={opcode:#04X} dest");
                        assert_eq!(src,  R8Register::from_byte(src_idx),  "opcode={opcode:#04X} src");
                    }
                    _ => panic!("opcode={opcode:#04X}: expected LdR8R8, got {instr:?}"),
                }
            }
        }
    }

    // -----------------------------------------------------------------------
    // Block 2
    // -----------------------------------------------------------------------

    #[test]
    fn test_decode_arithmetic_block2() {
        assert_eq!(decode_no_operands(0x80).0, Instruction::AddAR8 { reg: R8Register::B });
        assert_eq!(decode_no_operands(0x87).0, Instruction::AddAR8 { reg: R8Register::A });
        assert_eq!(decode_no_operands(0x88).0, Instruction::AdcAR8 { reg: R8Register::B });
        assert_eq!(decode_no_operands(0x90).0, Instruction::SubAR8 { reg: R8Register::B });
        assert_eq!(decode_no_operands(0x98).0, Instruction::SbcAR8 { reg: R8Register::B });
        assert_eq!(decode_no_operands(0xA0).0, Instruction::AndAR8 { reg: R8Register::B });
        assert_eq!(decode_no_operands(0xA8).0, Instruction::XorAR8 { reg: R8Register::B });
        assert_eq!(decode_no_operands(0xB0).0, Instruction::OrAR8  { reg: R8Register::B });
        assert_eq!(decode_no_operands(0xB8).0, Instruction::CpAR8  { reg: R8Register::B });
    }

    // -----------------------------------------------------------------------
    // Block 3
    // -----------------------------------------------------------------------

    #[test]
    fn test_decode_return_instructions() {
        assert_eq!(decode_no_operands(0xC0).0, Instruction::RetCond { cond: Condition::NZ });
        assert_eq!(decode_no_operands(0xC8).0, Instruction::RetCond { cond: Condition::Z  });
        assert_eq!(decode_no_operands(0xD0).0, Instruction::RetCond { cond: Condition::NC });
        assert_eq!(decode_no_operands(0xD8).0, Instruction::RetCond { cond: Condition::C  });
        assert_eq!(decode_no_operands(0xC9).0, Instruction::RET);
        assert_eq!(decode_no_operands(0xD9).0, Instruction::RETI);
    }

    #[test]
    fn test_decode_pop_push_instructions() {
        assert_eq!(decode_no_operands(0xC1).0, Instruction::PopR16  { reg: R16Register::BC });
        assert_eq!(decode_no_operands(0xD1).0, Instruction::PopR16  { reg: R16Register::DE });
        assert_eq!(decode_no_operands(0xE1).0, Instruction::PopR16  { reg: R16Register::HL });
        assert_eq!(decode_no_operands(0xF1).0, Instruction::PopR16  { reg: R16Register::AF });
        assert_eq!(decode_no_operands(0xC5).0, Instruction::PushR16 { reg: R16Register::BC });
        assert_eq!(decode_no_operands(0xD5).0, Instruction::PushR16 { reg: R16Register::DE });
        assert_eq!(decode_no_operands(0xE5).0, Instruction::PushR16 { reg: R16Register::HL });
        assert_eq!(decode_no_operands(0xF5).0, Instruction::PushR16 { reg: R16Register::AF });
    }

    #[test]
    fn test_decode_jump_instructions() {
        assert_eq!(decode_with_operands(&[0xC3, 0x34, 0x12]).0, Instruction::JpImm16 { address: 0x1234 });
        assert_eq!(decode_no_operands(0xE9).0, Instruction::JpHl);
        assert_eq!(decode_with_operands(&[0xC2, 0x34, 0x12]).0, Instruction::JpCondImm16 { cond: Condition::NZ, address: 0x1234 });
        assert_eq!(decode_with_operands(&[0xCA, 0x34, 0x12]).0, Instruction::JpCondImm16 { cond: Condition::Z,  address: 0x1234 });
        assert_eq!(decode_with_operands(&[0xD2, 0x34, 0x12]).0, Instruction::JpCondImm16 { cond: Condition::NC, address: 0x1234 });
        assert_eq!(decode_with_operands(&[0xDA, 0x34, 0x12]).0, Instruction::JpCondImm16 { cond: Condition::C,  address: 0x1234 });
    }

    #[test]
    fn test_decode_call_instructions() {
        assert_eq!(decode_with_operands(&[0xCD, 0x34, 0x12]).0, Instruction::CallImm16 { address: 0x1234 });
        assert_eq!(decode_with_operands(&[0xC4, 0x34, 0x12]).0, Instruction::CallCondImm16 { cond: Condition::NZ, address: 0x1234 });
        assert_eq!(decode_with_operands(&[0xCC, 0x34, 0x12]).0, Instruction::CallCondImm16 { cond: Condition::Z,  address: 0x1234 });
        assert_eq!(decode_with_operands(&[0xD4, 0x34, 0x12]).0, Instruction::CallCondImm16 { cond: Condition::NC, address: 0x1234 });
        assert_eq!(decode_with_operands(&[0xDC, 0x34, 0x12]).0, Instruction::CallCondImm16 { cond: Condition::C,  address: 0x1234 });
    }

    #[test]
    fn test_decode_rst_instructions() {
        for (op, target) in [
            (0xC7, 0x00u8), (0xCF, 0x08), (0xD7, 0x10), (0xDF, 0x18),
            (0xE7, 0x20),   (0xEF, 0x28), (0xF7, 0x30), (0xFF, 0x38),
        ] {
            assert_eq!(decode_no_operands(op).0, Instruction::RST { target }, "op={op:#04X}");
        }
    }

    #[test]
    fn test_decode_arithmetic_imm8() {
        for (op, expected) in [
            (0xC6, Instruction::AddAImm8 { value: 0x42 }),
            (0xCE, Instruction::AdcAImm8 { value: 0x42 }),
            (0xD6, Instruction::SubAImm8 { value: 0x42 }),
            (0xDE, Instruction::SbcAImm8 { value: 0x42 }),
            (0xE6, Instruction::AndAImm8 { value: 0x42 }),
            (0xEE, Instruction::XorAImm8 { value: 0x42 }),
            (0xF6, Instruction::OrAImm8  { value: 0x42 }),
            (0xFE, Instruction::CpAImm8  { value: 0x42 }),
        ] {
            assert_eq!(decode_with_operands(&[op, 0x42]).0, expected, "op={op:#04X}");
        }
    }

    #[test]
    fn test_decode_ldh_instructions() {
        assert_eq!(decode_with_operands(&[0xE0, 0x10]).0, Instruction::LdhIndImm8A { address: 0x10 });
        assert_eq!(decode_with_operands(&[0xF0, 0x10]).0, Instruction::LdhAIndImm8 { address: 0x10 });
        assert_eq!(decode_no_operands(0xE2).0, Instruction::LdhIndCA);
        assert_eq!(decode_no_operands(0xF2).0, Instruction::LdhAC);
    }

    #[test]
    fn test_decode_ld_imm16_indirect() {
        assert_eq!(decode_with_operands(&[0xEA, 0x34, 0x12]).0, Instruction::LdIndImm16A  { address: 0x1234 });
        assert_eq!(decode_with_operands(&[0xFA, 0x34, 0x12]).0, Instruction::LdAIndImm16  { address: 0x1234 });
    }

    #[test]
    fn test_decode_sp_instructions() {
        assert_eq!(decode_with_operands(&[0xE8, 0x05]).0, Instruction::AddSpImm8 { value: 5 });
        assert_eq!(decode_with_operands(&[0xF8, 0x05]).0, Instruction::LdHlSpImm8 { value: 5 });
        assert_eq!(decode_no_operands(0xF9).0, Instruction::LdSpHl);
    }

    #[test]
    fn test_decode_control_instructions() {
        assert_eq!(decode_no_operands(0xF3).0, Instruction::DI);
        assert_eq!(decode_no_operands(0xFB).0, Instruction::EI);
    }

    #[test]
    fn test_decode_cb_prefix() {
        let (instr, len) = decode_with_operands(&[0xCB, 0x00]); // RLC B
        assert_eq!(len, 2);
        assert_eq!(
            instr,
            Instruction::CB {
                cb_instr: crate::cpu::instructions::CBInstruction::RLCR8 { reg: R8Register::B }
            }
        );
    }

    #[test]
    fn test_decode_all_cb_opcodes_do_not_panic() {
        for cb_opcode in 0x00u8..=0xFF {
            let (instr, len) = decode_with_operands(&[0xCB, cb_opcode]);
            assert_eq!(len, 2, "cb={cb_opcode:#04X}");
            assert!(matches!(instr, Instruction::CB { .. }), "cb={cb_opcode:#04X}");
        }
    }

    #[test]
    fn test_decode_invalid_opcodes_return_nop() {
        for op in [0xD3u8, 0xDB, 0xDD, 0xE3, 0xE4, 0xEB, 0xEC, 0xED, 0xF4, 0xFC, 0xFD] {
            let (instr, len) = decode_no_operands(op);
            assert_eq!(instr, Instruction::NOP, "op={op:#04X}");
            assert_eq!(len, 1, "op={op:#04X}");
        }
    }
}
