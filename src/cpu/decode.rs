/// Instruction decoding module
///
/// This module handles decoding opcodes into Instructions.

use crate::memory::MemoryBus;
use crate::cpu::instructions::{Instruction, R8Register, R16Register, R16Mem, Condition};

/// Decode an instruction from the opcode
pub fn decode_instruction(_cpu_state: &crate::cpu::CPUState, bus: &MemoryBus, pc: u16, opcode: u8) -> (Instruction, u8) {
    match opcode {
        // Block 0: 00-3F
        0x00 => (Instruction::NOP, 1),
        0x01 => {
            let low = bus.read(pc + 1) as u8;
            let high = bus.read(pc + 2) as u8;
            let value = (high as u16) << 8 | low as u16;
            (Instruction::LdR16Imm16 { dest: R16Register::BC, value }, 3)
        }
        0x02 => (Instruction::LdIndR16A { src: R16Mem::BC }, 1),
        0x03 => (Instruction::IncR16 { reg: R16Register::BC }, 1),
        0x04 => (Instruction::IncR8 { reg: R8Register::B }, 1),
        0x05 => (Instruction::DecR8 { reg: R8Register::B }, 1),
        0x06 => {
            let value = bus.read(pc + 1);
            (Instruction::LdR8Imm8 { dest: R8Register::B, value }, 2)
        }
        0x07 => (Instruction::RLCA, 1),
        0x08 => {
            let low = bus.read(pc + 1) as u8;
            let high = bus.read(pc + 2) as u8;
            let address = (high as u16) << 8 | low as u16;
            (Instruction::LdIndImm16Sp { address }, 3)
        }
        0x09 => (Instruction::AddHlR16 { reg: R16Register::BC }, 1),
        0x0A => (Instruction::LdAIndR16 { dest: R16Mem::BC }, 1),
        0x0B => (Instruction::DecR16 { reg: R16Register::BC }, 1),
        0x0C => (Instruction::IncR8 { reg: R8Register::C }, 1),
        0x0D => (Instruction::DecR8 { reg: R8Register::C }, 1),
        0x0E => {
            let value = bus.read(pc + 1);
            (Instruction::LdR8Imm8 { dest: R8Register::C, value }, 2)
        }
        0x0F => (Instruction::RRCA, 1),
        0x10 => (Instruction::STOP, 1),
        0x11 => {
            let low = bus.read(pc + 1) as u8;
            let high = bus.read(pc + 2) as u8;
            let value = (high as u16) << 8 | low as u16;
            (Instruction::LdR16Imm16 { dest: R16Register::DE, value }, 3)
        }
        0x12 => (Instruction::LdIndR16A { src: R16Mem::DE }, 1),
        0x13 => (Instruction::IncR16 { reg: R16Register::DE }, 1),
        0x14 => (Instruction::IncR8 { reg: R8Register::D }, 1),
        0x15 => (Instruction::DecR8 { reg: R8Register::D }, 1),
        0x16 => {
            let value = bus.read(pc + 1);
            (Instruction::LdR8Imm8 { dest: R8Register::D, value }, 2)
        }
        0x17 => (Instruction::RLA, 1),
        0x18 => {
            let offset = bus.read(pc + 1) as i8;
            (Instruction::JrImm8 { offset }, 2)
        }
        0x19 => (Instruction::AddHlR16 { reg: R16Register::DE }, 1),
        0x1A => (Instruction::LdAIndR16 { dest: R16Mem::DE }, 1),
        0x1B => (Instruction::DecR16 { reg: R16Register::DE }, 1),
        0x1C => (Instruction::IncR8 { reg: R8Register::E }, 1),
        0x1D => (Instruction::DecR8 { reg: R8Register::E }, 1),
        0x1E => {
            let value = bus.read(pc + 1);
            (Instruction::LdR8Imm8 { dest: R8Register::E, value }, 2)
        }
        0x1F => (Instruction::RRA, 1),
        0x20 => {
            let offset = bus.read(pc + 1) as i8;
            (Instruction::JrCondImm8 { cond: Condition::NZ, offset }, 2)
        }
        0x21 => {
            let low = bus.read(pc + 1) as u8;
            let high = bus.read(pc + 2) as u8;
            let value = (high as u16) << 8 | low as u16;
            (Instruction::LdR16Imm16 { dest: R16Register::HL, value }, 3)
        }
        0x22 => (Instruction::LdIndR16A { src: R16Mem::HLPlus }, 1),
        0x23 => (Instruction::IncR16 { reg: R16Register::HL }, 1),
        0x24 => (Instruction::IncR8 { reg: R8Register::H }, 1),
        0x25 => (Instruction::DecR8 { reg: R8Register::H }, 1),
        0x26 => {
            let value = bus.read(pc + 1);
            (Instruction::LdR8Imm8 { dest: R8Register::H, value }, 2)
        }
        0x27 => (Instruction::DAA, 1),
        0x28 => {
            let offset = bus.read(pc + 1) as i8;
            (Instruction::JrCondImm8 { cond: Condition::Z, offset }, 2)
        }
        0x29 => (Instruction::AddHlR16 { reg: R16Register::HL }, 1),
        0x2A => (Instruction::LdAIndR16 { dest: R16Mem::HLPlus }, 1),
        0x2B => (Instruction::DecR16 { reg: R16Register::HL }, 1),
        0x2C => (Instruction::IncR8 { reg: R8Register::L }, 1),
        0x2D => (Instruction::DecR8 { reg: R8Register::L }, 1),
        0x2E => {
            let value = bus.read(pc + 1);
            (Instruction::LdR8Imm8 { dest: R8Register::L, value }, 2)
        }
        0x2F => (Instruction::CPL, 1),
        0x30 => {
            let offset = bus.read(pc + 1) as i8;
            (Instruction::JrCondImm8 { cond: Condition::NC, offset }, 2)
        }
        0x31 => {
            let low = bus.read(pc + 1) as u8;
            let high = bus.read(pc + 2) as u8;
            let value = (high as u16) << 8 | low as u16;
            (Instruction::LdR16Imm16 { dest: R16Register::SP, value }, 3)
        }
        0x32 => (Instruction::LdIndR16A { src: R16Mem::HLMinus }, 1),
        0x33 => (Instruction::IncR16 { reg: R16Register::SP }, 1),
        0x34 => (Instruction::IncR8 { reg: R8Register::HL }, 1),
        0x35 => (Instruction::DecR8 { reg: R8Register::HL }, 1),
        0x36 => {
            let value = bus.read(pc + 1);
            (Instruction::LdR8Imm8 { dest: R8Register::HL, value }, 2)
        }
        0x37 => (Instruction::SCF, 1),
        0x38 => {
            let offset = bus.read(pc + 1) as i8;
            (Instruction::JrCondImm8 { cond: Condition::C, offset }, 2)
        }
        0x39 => (Instruction::AddHlR16 { reg: R16Register::SP }, 1),
        0x3A => (Instruction::LdAIndR16 { dest: R16Mem::HLMinus }, 1),
        0x3B => (Instruction::DecR16 { reg: R16Register::SP }, 1),
        0x3C => (Instruction::IncR8 { reg: R8Register::A }, 1),
        0x3D => (Instruction::DecR8 { reg: R8Register::A }, 1),
        0x3E => {
            let value = bus.read(pc + 1);
            (Instruction::LdR8Imm8 { dest: R8Register::A, value }, 2)
        }
        0x3F => (Instruction::CCF, 1),

        // Block 1: 40-7F (8-bit register-to-register loads)
        // Opcode format: dd ddd ddd where upper 3 bits are src, lower 3 bits are dest
        0x40..=0x7F => {
            let reg_src = R8Register::from_byte((opcode >> 3) & 0x07);
            let reg_dest = R8Register::from_byte(opcode & 0x07);
            (Instruction::LdR8R8 { dest: reg_dest, src: reg_src }, 1)
        }

        // Block 2: 80-BF (8-bit arithmetic with accumulator)
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
                _ => (Instruction::NOP, 1),
            }
        }

        // Block 3: C0-FF (jumps, calls, returns, stack operations)
        0xC0 => (Instruction::RetCond { cond: Condition::NZ }, 1),
        0xC1 => (Instruction::PopR16 { reg: R16Register::BC }, 1),
        0xC2 => {
            let low = bus.read(pc + 1) as u8;
            let high = bus.read(pc + 2) as u8;
            let address = (high as u16) << 8 | low as u16;
            (Instruction::JpCondImm16 { cond: Condition::NZ, address }, 3)
        }
        0xC3 => {
            let low = bus.read(pc + 1) as u8;
            let high = bus.read(pc + 2) as u8;
            let address = (high as u16) << 8 | low as u16;
            (Instruction::JpImm16 { address }, 3)
        }
        0xC4 => {
            let low = bus.read(pc + 1) as u8;
            let high = bus.read(pc + 2) as u8;
            let address = (high as u16) << 8 | low as u16;
            (Instruction::CallCondImm16 { cond: Condition::NZ, address }, 3)
        }
        0xC5 => (Instruction::PushR16 { reg: R16Register::BC }, 1),
        0xC6 => {
            let value = bus.read(pc + 1);
            (Instruction::AddAImm8 { value }, 2)
        }
        0xC7 => (Instruction::RST { target: 0x00 }, 1),
        0xC8 => (Instruction::RetCond { cond: Condition::Z }, 1),
        0xC9 => (Instruction::RET, 1),
        0xCA => {
            let low = bus.read(pc + 1) as u8;
            let high = bus.read(pc + 2) as u8;
            let address = (high as u16) << 8 | low as u16;
            (Instruction::JpCondImm16 { cond: Condition::Z, address }, 3)
        }
        0xCB => {
            let cb_opcode = bus.read(pc + 1);
            let cb_instr = crate::cpu::instructions::CBInstruction::from_byte(opcode, cb_opcode);
            (Instruction::CB { cb_instr }, 2)
        }
        0xCC => {
            let low = bus.read(pc + 1) as u8;
            let high = bus.read(pc + 2) as u8;
            let address = (high as u16) << 8 | low as u16;
            (Instruction::CallCondImm16 { cond: Condition::Z, address }, 3)
        }
        0xCD => {
            let low = bus.read(pc + 1) as u8;
            let high = bus.read(pc + 2) as u8;
            let address = (high as u16) << 8 | low as u16;
            (Instruction::CallImm16 { address }, 3)
        }
        0xCE => {
            let value = bus.read(pc + 1);
            (Instruction::AdcAImm8 { value }, 2)
        }
        0xCF => (Instruction::RST { target: 0x08 }, 1),
        0xD0 => (Instruction::RetCond { cond: Condition::NC }, 1),
        0xD1 => (Instruction::PopR16 { reg: R16Register::DE }, 1),
        0xD2 => {
            let low = bus.read(pc + 1) as u8;
            let high = bus.read(pc + 2) as u8;
            let address = (high as u16) << 8 | low as u16;
            (Instruction::JpCondImm16 { cond: Condition::NC, address }, 3)
        }
        0xD4 => {
            let low = bus.read(pc + 1) as u8;
            let high = bus.read(pc + 2) as u8;
            let address = (high as u16) << 8 | low as u16;
            (Instruction::CallCondImm16 { cond: Condition::NC, address }, 3)
        }
        0xD5 => (Instruction::PushR16 { reg: R16Register::DE }, 1),
        0xD6 => {
            let value = bus.read(pc + 1);
            (Instruction::SubAImm8 { value }, 2)
        }
        0xD7 => (Instruction::RST { target: 0x10 }, 1),
        0xD8 => (Instruction::RetCond { cond: Condition::C }, 1),
        0xD9 => (Instruction::RETI, 1),
        0xDA => {
            let low = bus.read(pc + 1) as u8;
            let high = bus.read(pc + 2) as u8;
            let address = (high as u16) << 8 | low as u16;
            (Instruction::JpCondImm16 { cond: Condition::C, address }, 3)
        }
        0xDB => (Instruction::LdhAC, 1),
        0xDC => {
            let low = bus.read(pc + 1) as u8;
            let high = bus.read(pc + 2) as u8;
            let address = (high as u16) << 8 | low as u16;
            (Instruction::CallCondImm16 { cond: Condition::C, address }, 3)
        }
        0xDE => {
            let value = bus.read(pc + 1);
            (Instruction::SbcAImm8 { value }, 2)
        }
        0xDF => (Instruction::RST { target: 0x18 }, 1),
        0xE0 => {
            let value = bus.read(pc + 1);
            (Instruction::LdhIndImm8A { address: value }, 2)
        }
        0xE1 => (Instruction::PopR16 { reg: R16Register::HL }, 1),
        0xE2 => (Instruction::LdhIndCA, 1),
        0xE5 => (Instruction::PushR16 { reg: R16Register::HL }, 1),
        0xE6 => {
            let value = bus.read(pc + 1);
            (Instruction::AndAImm8 { value }, 2)
        }
        0xE7 => (Instruction::RST { target: 0x20 }, 1),
        0xE8 => {
            let value = bus.read(pc + 1) as i8;
            (Instruction::AddSpImm8 { value }, 2)
        }
        0xE9 => (Instruction::JpHl, 1),
        0xEA => {
            let low = bus.read(pc + 1) as u8;
            let high = bus.read(pc + 2) as u8;
            let address = (high as u16) << 8 | low as u16;
            (Instruction::LdIndImm16A { address }, 3)
        }
        0xEE => {
            let value = bus.read(pc + 1);
            (Instruction::XorAImm8 { value }, 2)
        }
        0xEF => (Instruction::RST { target: 0x28 }, 1),
        0xF0 => {
            let value = bus.read(pc + 1);
            (Instruction::LdhAIndImm8 { address: value }, 2)
        }
        0xF1 => (Instruction::PopR16 { reg: R16Register::AF }, 1),
        0xF2 => (Instruction::LdhAC, 1),
        0xF3 => (Instruction::DI, 1),
        0xF5 => (Instruction::PushR16 { reg: R16Register::AF }, 1),
        0xF6 => {
            let value = bus.read(pc + 1);
            (Instruction::OrAImm8 { value }, 2)
        }
        0xF7 => (Instruction::RST { target: 0x30 }, 1),
        0xF8 => {
            let value = bus.read(pc + 1) as i8;
            (Instruction::LdHlSpImm8 { value }, 2)
        }
        0xF9 => (Instruction::LdSpHl, 1),
        0xFA => {
            let low = bus.read(pc + 1) as u8;
            let high = bus.read(pc + 2) as u8;
            let address = (high as u16) << 8 | low as u16;
            (Instruction::LdAIndImm16 { address }, 3)
        }
        0xFB => (Instruction::EI, 1),
        0xFE => {
            let value = bus.read(pc + 1);
            (Instruction::CpAImm8 { value }, 2)
        }
        0xFF => (Instruction::RST { target: 0x38 }, 1),
        _ => (Instruction::NOP, 1),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::MemoryBus;

    // ==================== BLOCK 0 (00-3F) INSTRUCTIONS ====================

    #[test]
    fn test_decode_nop() {
        let bus = MemoryBus::new(vec![0; 32768]);
        let (instr, bytes) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0x00);
        assert_eq!(instr, Instruction::NOP);
        assert_eq!(bytes, 1);
    }

    #[test]
    fn test_decode_ld_r16_imm16_all() {
        let mut bus = MemoryBus::new(vec![0; 32768]);
        bus.write(0x0001, 0x34);
        bus.write(0x0002, 0x12);

        // BC
        let (instr, bytes) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0x01);
        match instr {
            Instruction::LdR16Imm16 { dest, value } => {
                assert_eq!(dest, R16Register::BC);
                assert_eq!(value, 0x1234);
            }
            _ => panic!("Expected LdR16Imm16"),
        }
        assert_eq!(bytes, 3);

        // DE
        let (instr, bytes) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0x11);
        match instr {
            Instruction::LdR16Imm16 { dest, value } => {
                assert_eq!(dest, R16Register::DE);
                assert_eq!(value, 0x1234);
            }
            _ => panic!("Expected LdR16Imm16"),
        }

        // HL
        let (instr, bytes) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0x21);
        match instr {
            Instruction::LdR16Imm16 { dest, value } => {
                assert_eq!(dest, R16Register::HL);
                assert_eq!(value, 0x1234);
            }
            _ => panic!("Expected LdR16Imm16"),
        }

        // SP
        let (instr, bytes) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0x31);
        match instr {
            Instruction::LdR16Imm16 { dest, value } => {
                assert_eq!(dest, R16Register::SP);
                assert_eq!(value, 0x1234);
            }
            _ => panic!("Expected LdR16Imm16"),
        }
    }

    #[test]
    fn test_decode_ld_ind_r16_a() {
        let bus = MemoryBus::new(vec![0; 32768]);

        let (instr, bytes) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0x02);
        assert_eq!(instr, Instruction::LdIndR16A { src: R16Mem::BC });
        assert_eq!(bytes, 1);

        let (instr, bytes) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0x12);
        assert_eq!(instr, Instruction::LdIndR16A { src: R16Mem::DE });
        assert_eq!(bytes, 1);
    }

    #[test]
    fn test_decode_ld_a_ind_r16() {
        let bus = MemoryBus::new(vec![0; 32768]);

        let (instr, bytes) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0x0A);
        assert_eq!(instr, Instruction::LdAIndR16 { dest: R16Mem::BC });
        assert_eq!(bytes, 1);

        let (instr, bytes) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0x1A);
        assert_eq!(instr, Instruction::LdAIndR16 { dest: R16Mem::DE });
        assert_eq!(bytes, 1);
    }

    #[test]
    fn test_decode_ld_ind_imm16_sp() {
        let mut bus = MemoryBus::new(vec![0; 32768]);
        bus.write(0x0001, 0x34);
        bus.write(0x0002, 0x12);

        let (instr, bytes) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0x08);
        match instr {
            Instruction::LdIndImm16Sp { address } => {
                assert_eq!(address, 0x1234);
            }
            _ => panic!("Expected LdIndImm16Sp"),
        }
        assert_eq!(bytes, 3);
    }

    #[test]
    fn test_decode_inc_dec_r16() {
        let bus = MemoryBus::new(vec![0; 32768]);

        // INC r16
        let (instr, _) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0x03);
        assert_eq!(instr, Instruction::IncR16 { reg: R16Register::BC });

        let (instr, _) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0x13);
        assert_eq!(instr, Instruction::IncR16 { reg: R16Register::DE });

        let (instr, _) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0x23);
        assert_eq!(instr, Instruction::IncR16 { reg: R16Register::HL });

        let (instr, _) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0x33);
        assert_eq!(instr, Instruction::IncR16 { reg: R16Register::SP });

        // DEC r16
        let (instr, _) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0x0B);
        assert_eq!(instr, Instruction::DecR16 { reg: R16Register::BC });

        let (instr, _) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0x1B);
        assert_eq!(instr, Instruction::DecR16 { reg: R16Register::DE });

        let (instr, _) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0x2B);
        assert_eq!(instr, Instruction::DecR16 { reg: R16Register::HL });

        let (instr, _) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0x3B);
        assert_eq!(instr, Instruction::DecR16 { reg: R16Register::SP });
    }

    #[test]
    fn test_decode_inc_dec_r8_all() {
        let bus = MemoryBus::new(vec![0; 32768]);

        // INC r8
        let (instr, _) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0x04);
        assert_eq!(instr, Instruction::IncR8 { reg: R8Register::B });

        let (instr, _) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0x0C);
        assert_eq!(instr, Instruction::IncR8 { reg: R8Register::C });

        let (instr, _) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0x14);
        assert_eq!(instr, Instruction::IncR8 { reg: R8Register::D });

        let (instr, _) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0x1C);
        assert_eq!(instr, Instruction::IncR8 { reg: R8Register::E });

        let (instr, _) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0x24);
        assert_eq!(instr, Instruction::IncR8 { reg: R8Register::H });

        let (instr, _) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0x2C);
        assert_eq!(instr, Instruction::IncR8 { reg: R8Register::L });

        let (instr, _) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0x34);
        assert_eq!(instr, Instruction::IncR8 { reg: R8Register::HL });

        let (instr, _) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0x3C);
        assert_eq!(instr, Instruction::IncR8 { reg: R8Register::A });

        // DEC r8
        let (instr, _) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0x05);
        assert_eq!(instr, Instruction::DecR8 { reg: R8Register::B });

        let (instr, _) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0x0D);
        assert_eq!(instr, Instruction::DecR8 { reg: R8Register::C });

        let (instr, _) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0x15);
        assert_eq!(instr, Instruction::DecR8 { reg: R8Register::D });

        let (instr, _) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0x1D);
        assert_eq!(instr, Instruction::DecR8 { reg: R8Register::E });

        let (instr, _) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0x25);
        assert_eq!(instr, Instruction::DecR8 { reg: R8Register::H });

        let (instr, _) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0x2D);
        assert_eq!(instr, Instruction::DecR8 { reg: R8Register::L });

        let (instr, _) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0x35);
        assert_eq!(instr, Instruction::DecR8 { reg: R8Register::HL });

        let (instr, _) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0x3D);
        assert_eq!(instr, Instruction::DecR8 { reg: R8Register::A });
    }

    #[test]
    fn test_decode_ld_r8_imm8_all() {
        let mut bus = MemoryBus::new(vec![0; 32768]);
        bus.write(0x0001, 0x42);

        // LD B, d8
        let (instr, bytes) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0x06);
        match instr {
            Instruction::LdR8Imm8 { dest, value } => {
                assert_eq!(dest, R8Register::B);
                assert_eq!(value, 0x42);
            }
            _ => panic!("Expected LdR8Imm8"),
        }
        assert_eq!(bytes, 2);

        // LD C, d8
        let (instr, bytes) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0x0E);
        match instr {
            Instruction::LdR8Imm8 { dest, value } => {
                assert_eq!(dest, R8Register::C);
                assert_eq!(value, 0x42);
            }
            _ => panic!("Expected LdR8Imm8"),
        }

        // LD A, d8
        let (instr, bytes) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0x3E);
        match instr {
            Instruction::LdR8Imm8 { dest, value } => {
                assert_eq!(dest, R8Register::A);
                assert_eq!(value, 0x42);
            }
            _ => panic!("Expected LdR8Imm8"),
        }
    }

    #[test]
    fn test_decode_rotate_shift_instructions() {
        let bus = MemoryBus::new(vec![0; 32768]);

        let (instr, _) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0x07);
        assert_eq!(instr, Instruction::RLCA);

        let (instr, _) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0x0F);
        assert_eq!(instr, Instruction::RRCA);

        let (instr, _) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0x17);
        assert_eq!(instr, Instruction::RLA);

        let (instr, _) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0x1F);
        assert_eq!(instr, Instruction::RRA);

        let (instr, _) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0x27);
        assert_eq!(instr, Instruction::DAA);

        let (instr, _) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0x2F);
        assert_eq!(instr, Instruction::CPL);

        let (instr, _) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0x37);
        assert_eq!(instr, Instruction::SCF);

        let (instr, _) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0x3F);
        assert_eq!(instr, Instruction::CCF);
    }

    #[test]
    fn test_decode_jr_instructions() {
        let mut bus = MemoryBus::new(vec![0; 32768]);
        bus.write(0x0001, 0x05); // offset = 5

        // JR r8
        let (instr, bytes) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0x18);
        match instr {
            Instruction::JrImm8 { offset } => {
                assert_eq!(offset, 5);
            }
            _ => panic!("Expected JrImm8"),
        }
        assert_eq!(bytes, 2);

        // JR NZ, r8
        let (instr, bytes) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0x20);
        match instr {
            Instruction::JrCondImm8 { cond, offset } => {
                assert_eq!(cond, Condition::NZ);
                assert_eq!(offset, 5);
            }
            _ => panic!("Expected JrCondImm8"),
        }

        // JR Z, r8
        let (instr, bytes) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0x28);
        match instr {
            Instruction::JrCondImm8 { cond, offset } => {
                assert_eq!(cond, Condition::Z);
                assert_eq!(offset, 5);
            }
            _ => panic!("Expected JrCondImm8"),
        }

        // JR NC, r8
        let (instr, bytes) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0x30);
        match instr {
            Instruction::JrCondImm8 { cond, offset } => {
                assert_eq!(cond, Condition::NC);
                assert_eq!(offset, 5);
            }
            _ => panic!("Expected JrCondImm8"),
        }

        // JR C, r8
        let (instr, bytes) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0x38);
        match instr {
            Instruction::JrCondImm8 { cond, offset } => {
                assert_eq!(cond, Condition::C);
                assert_eq!(offset, 5);
            }
            _ => panic!("Expected JrCondImm8"),
        }
    }

    #[test]
    fn test_decode_add_hl_r16() {
        let bus = MemoryBus::new(vec![0; 32768]);

        let (instr, _) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0x09);
        assert_eq!(instr, Instruction::AddHlR16 { reg: R16Register::BC });

        let (instr, _) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0x19);
        assert_eq!(instr, Instruction::AddHlR16 { reg: R16Register::DE });

        let (instr, _) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0x29);
        assert_eq!(instr, Instruction::AddHlR16 { reg: R16Register::HL });

        let (instr, _) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0x39);
        assert_eq!(instr, Instruction::AddHlR16 { reg: R16Register::SP });
    }

    // ==================== BLOCK 1 (40-7F) INSTRUCTIONS ====================

    #[test]
    fn test_decode_ld_r8_r8_all() {
        let bus = MemoryBus::new(vec![0; 32768]);

        // Test all LD r8, r8 combinations
        // LD r8, r8 opcodes are 0x40-0x7F
        // Encoding: dd dd dd = ddd ddd where upper 3 bits are src, lower 3 bits are dest
        for src in 0..=7 {
            for dest in 0..=7 {
                let opcode = 0x40 | ((src & 0x07) << 3) | (dest & 0x07);
                let (instr, _) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, opcode as u8);

                match instr {
                    Instruction::LdR8R8 { dest: d, src: s } => {
                        assert_eq!(d, R8Register::from_byte(dest as u8));
                        assert_eq!(s, R8Register::from_byte(src as u8));
                    }
                    _ => panic!("Expected LdR8R8 for opcode={:02X}", opcode),
                }
            }
        }
    }

    // ==================== BLOCK 2 (80-BF) INSTRUCTIONS ====================

    #[test]
    fn test_decode_arithmetic_instructions() {
        let bus = MemoryBus::new(vec![0; 32768]);

        // ADD A, r8
        let (instr, _) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0x80);
        assert_eq!(instr, Instruction::AddAR8 { reg: R8Register::B });

        let (instr, _) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0x87);
        assert_eq!(instr, Instruction::AddAR8 { reg: R8Register::A });

        // ADC A, r8
        let (instr, _) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0x88);
        assert_eq!(instr, Instruction::AdcAR8 { reg: R8Register::B });

        // SUB A, r8
        let (instr, _) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0x90);
        assert_eq!(instr, Instruction::SubAR8 { reg: R8Register::B });

        // SBC A, r8
        let (instr, _) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0x98);
        assert_eq!(instr, Instruction::SbcAR8 { reg: R8Register::B });

        // AND A, r8
        let (instr, _) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0xA0);
        assert_eq!(instr, Instruction::AndAR8 { reg: R8Register::B });

        // XOR A, r8
        let (instr, _) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0xA8);
        assert_eq!(instr, Instruction::XorAR8 { reg: R8Register::B });

        // OR A, r8
        let (instr, _) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0xB0);
        assert_eq!(instr, Instruction::OrAR8 { reg: R8Register::B });

        // CP A, r8
        let (instr, _) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0xB8);
        assert_eq!(instr, Instruction::CpAR8 { reg: R8Register::B });
    }

    // ==================== BLOCK 3 (C0-FF) INSTRUCTIONS ====================

    #[test]
    fn test_decode_return_instructions() {
        let bus = MemoryBus::new(vec![0; 32768]);

        let (instr, _) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0xC0);
        assert_eq!(instr, Instruction::RetCond { cond: Condition::NZ });

        let (instr, _) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0xC8);
        assert_eq!(instr, Instruction::RetCond { cond: Condition::Z });

        let (instr, _) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0xD0);
        assert_eq!(instr, Instruction::RetCond { cond: Condition::NC });

        let (instr, _) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0xD8);
        assert_eq!(instr, Instruction::RetCond { cond: Condition::C });

        let (instr, _) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0xC9);
        assert_eq!(instr, Instruction::RET);

        let (instr, _) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0xD9);
        assert_eq!(instr, Instruction::RETI);
    }

    #[test]
    fn test_decode_pop_instructions() {
        let bus = MemoryBus::new(vec![0; 32768]);

        let (instr, _) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0xC1);
        assert_eq!(instr, Instruction::PopR16 { reg: R16Register::BC });

        let (instr, _) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0xD1);
        assert_eq!(instr, Instruction::PopR16 { reg: R16Register::DE });

        let (instr, _) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0xE1);
        assert_eq!(instr, Instruction::PopR16 { reg: R16Register::HL });

        let (instr, _) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0xF1);
        assert_eq!(instr, Instruction::PopR16 { reg: R16Register::AF });
    }

    #[test]
    fn test_decode_push_instructions() {
        let bus = MemoryBus::new(vec![0; 32768]);

        let (instr, _) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0xC5);
        assert_eq!(instr, Instruction::PushR16 { reg: R16Register::BC });

        let (instr, _) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0xD5);
        assert_eq!(instr, Instruction::PushR16 { reg: R16Register::DE });

        let (instr, _) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0xE5);
        assert_eq!(instr, Instruction::PushR16 { reg: R16Register::HL });

        let (instr, _) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0xF5);
        assert_eq!(instr, Instruction::PushR16 { reg: R16Register::AF });
    }

    #[test]
    fn test_decode_jump_instructions() {
        let mut bus = MemoryBus::new(vec![0; 32768]);
        bus.write(0x0001, 0x34);
        bus.write(0x0002, 0x12);

        // JP a16
        let (instr, bytes) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0xC3);
        match instr {
            Instruction::JpImm16 { address } => {
                assert_eq!(address, 0x1234);
            }
            _ => panic!("Expected JpImm16"),
        }
        assert_eq!(bytes, 3);

        // JP NZ, a16
        let (instr, bytes) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0xC2);
        match instr {
            Instruction::JpCondImm16 { cond, address } => {
                assert_eq!(cond, Condition::NZ);
                assert_eq!(address, 0x1234);
            }
            _ => panic!("Expected JpCondImm16"),
        }

        // JP Z, a16
        let (instr, bytes) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0xCA);
        match instr {
            Instruction::JpCondImm16 { cond, address } => {
                assert_eq!(cond, Condition::Z);
                assert_eq!(address, 0x1234);
            }
            _ => panic!("Expected JpCondImm16"),
        }

        // JP NC, a16
        let (instr, bytes) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0xD2);
        match instr {
            Instruction::JpCondImm16 { cond, address } => {
                assert_eq!(cond, Condition::NC);
                assert_eq!(address, 0x1234);
            }
            _ => panic!("Expected JpCondImm16"),
        }

        // JP C, a16
        let (instr, bytes) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0xDA);
        match instr {
            Instruction::JpCondImm16 { cond, address } => {
                assert_eq!(cond, Condition::C);
                assert_eq!(address, 0x1234);
            }
            _ => panic!("Expected JpCondImm16"),
        }

        // JP (HL)
        let (instr, _) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0xE9);
        assert_eq!(instr, Instruction::JpHl);
    }

    #[test]
    fn test_decode_call_instructions() {
        let mut bus = MemoryBus::new(vec![0; 32768]);
        bus.write(0x0001, 0x34);
        bus.write(0x0002, 0x12);

        // CALL a16
        let (instr, bytes) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0xCD);
        match instr {
            Instruction::CallImm16 { address } => {
                assert_eq!(address, 0x1234);
            }
            _ => panic!("Expected CallImm16"),
        }
        assert_eq!(bytes, 3);

        // CALL NZ, a16
        let (instr, bytes) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0xC4);
        match instr {
            Instruction::CallCondImm16 { cond, address } => {
                assert_eq!(cond, Condition::NZ);
                assert_eq!(address, 0x1234);
            }
            _ => panic!("Expected CallCondImm16"),
        }

        // CALL Z, a16
        let (instr, bytes) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0xCC);
        match instr {
            Instruction::CallCondImm16 { cond, address } => {
                assert_eq!(cond, Condition::Z);
                assert_eq!(address, 0x1234);
            }
            _ => panic!("Expected CallCondImm16"),
        }

        // CALL NC, a16
        let (instr, bytes) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0xD4);
        match instr {
            Instruction::CallCondImm16 { cond, address } => {
                assert_eq!(cond, Condition::NC);
                assert_eq!(address, 0x1234);
            }
            _ => panic!("Expected CallCondImm16"),
        }

        // CALL C, a16
        let (instr, bytes) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0xDC);
        match instr {
            Instruction::CallCondImm16 { cond, address } => {
                assert_eq!(cond, Condition::C);
                assert_eq!(address, 0x1234);
            }
            _ => panic!("Expected CallCondImm16"),
        }
    }

    #[test]
    fn test_decode_rst_instructions() {
        let bus = MemoryBus::new(vec![0; 32768]);

        let (instr, _) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0xC7);
        assert_eq!(instr, Instruction::RST { target: 0x00 });

        let (instr, _) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0xCF);
        assert_eq!(instr, Instruction::RST { target: 0x08 });

        let (instr, _) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0xD7);
        assert_eq!(instr, Instruction::RST { target: 0x10 });

        let (instr, _) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0xDF);
        assert_eq!(instr, Instruction::RST { target: 0x18 });

        let (instr, _) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0xE7);
        assert_eq!(instr, Instruction::RST { target: 0x20 });

        let (instr, _) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0xEF);
        assert_eq!(instr, Instruction::RST { target: 0x28 });

        let (instr, _) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0xF7);
        assert_eq!(instr, Instruction::RST { target: 0x30 });

        let (instr, _) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0xFF);
        assert_eq!(instr, Instruction::RST { target: 0x38 });
    }

    #[test]
    fn test_decode_ld_ind_imm16_a() {
        let mut bus = MemoryBus::new(vec![0; 32768]);
        bus.write(0x0001, 0x34);
        bus.write(0x0002, 0x12);

        let (instr, bytes) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0xEA);
        match instr {
            Instruction::LdIndImm16A { address } => {
                assert_eq!(address, 0x1234);
            }
            _ => panic!("Expected LdIndImm16A"),
        }
        assert_eq!(bytes, 3);
    }

    #[test]
    fn test_decode_ld_a_ind_imm16() {
        let mut bus = MemoryBus::new(vec![0; 32768]);
        bus.write(0x0001, 0x34);
        bus.write(0x0002, 0x12);

        let (instr, bytes) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0xFA);
        match instr {
            Instruction::LdAIndImm16 { address } => {
                assert_eq!(address, 0x1234);
            }
            _ => panic!("Expected LdAIndImm16"),
        }
        assert_eq!(bytes, 3);
    }

    #[test]
    fn test_decode_ldh_ind_imm8_a() {
        let mut bus = MemoryBus::new(vec![0; 32768]);
        bus.write(0x0001, 0x10);

        let (instr, bytes) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0xE0);
        match instr {
            Instruction::LdhIndImm8A { address } => {
                assert_eq!(address, 0x10);
            }
            _ => panic!("Expected LdhIndImm8A"),
        }
        assert_eq!(bytes, 2);
    }

    #[test]
    fn test_decode_ldh_a_ind_imm8() {
        let mut bus = MemoryBus::new(vec![0; 32768]);
        bus.write(0x0001, 0x20);

        let (instr, bytes) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0xF0);
        match instr {
            Instruction::LdhAIndImm8 { address } => {
                assert_eq!(address, 0x20);
            }
            _ => panic!("Expected LdhAIndImm8"),
        }
        assert_eq!(bytes, 2);
    }

    #[test]
    fn test_decode_ldh_ind_c_a() {
        let bus = MemoryBus::new(vec![0; 32768]);

        let (instr, _) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0xE2);
        assert_eq!(instr, Instruction::LdhIndCA);

        let (instr, _) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0xF2);
        assert_eq!(instr, Instruction::LdhAC);
    }

    #[test]
    fn test_decode_arithmetic_imm8() {
        let mut bus = MemoryBus::new(vec![0; 32768]);
        bus.write(0x0001, 0x42);

        // ADD A, d8
        let (instr, bytes) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0xC6);
        match instr {
            Instruction::AddAImm8 { value } => {
                assert_eq!(value, 0x42);
            }
            _ => panic!("Expected AddAImm8"),
        }
        assert_eq!(bytes, 2);

        // ADC A, d8
        let (instr, bytes) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0xCE);
        match instr {
            Instruction::AdcAImm8 { value } => {
                assert_eq!(value, 0x42);
            }
            _ => panic!("Expected AdcAImm8"),
        }

        // SUB A, d8
        let (instr, bytes) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0xD6);
        match instr {
            Instruction::SubAImm8 { value } => {
                assert_eq!(value, 0x42);
            }
            _ => panic!("Expected SubAImm8"),
        }

        // SBC A, d8
        let (instr, bytes) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0xDE);
        match instr {
            Instruction::SbcAImm8 { value } => {
                assert_eq!(value, 0x42);
            }
            _ => panic!("Expected SbcAImm8"),
        }

        // AND A, d8
        let (instr, bytes) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0xE6);
        match instr {
            Instruction::AndAImm8 { value } => {
                assert_eq!(value, 0x42);
            }
            _ => panic!("Expected AndAImm8"),
        }

        // XOR A, d8
        let (instr, bytes) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0xEE);
        match instr {
            Instruction::XorAImm8 { value } => {
                assert_eq!(value, 0x42);
            }
            _ => panic!("Expected XorAImm8"),
        }

        // OR A, d8
        let (instr, bytes) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0xF6);
        match instr {
            Instruction::OrAImm8 { value } => {
                assert_eq!(value, 0x42);
            }
            _ => panic!("Expected OrAImm8"),
        }

        // CP A, d8
        let (instr, bytes) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0xFE);
        match instr {
            Instruction::CpAImm8 { value } => {
                assert_eq!(value, 0x42);
            }
            _ => panic!("Expected CpAImm8"),
        }
    }

    #[test]
    fn test_decode_sp_instructions() {
        let mut bus = MemoryBus::new(vec![0; 32768]);
        bus.write(0x0001, 0x05);

        // ADD SP, d8
        let (instr, bytes) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0xE8);
        match instr {
            Instruction::AddSpImm8 { value } => {
                assert_eq!(value, 5);
            }
            _ => panic!("Expected AddSpImm8"),
        }
        assert_eq!(bytes, 2);

        // LD (HL), SP
        let (instr, bytes) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0xF8);
        match instr {
            Instruction::LdHlSpImm8 { value } => {
                assert_eq!(value, 5);
            }
            _ => panic!("Expected LdHlSpImm8"),
        }
        assert_eq!(bytes, 2);

        // LD SP, HL
        let (instr, _) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0xF9);
        assert_eq!(instr, Instruction::LdSpHl);
    }

    #[test]
    fn test_decode_control_instructions() {
        let bus = MemoryBus::new(vec![0; 32768]);

        let (instr, _) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0x10);
        assert_eq!(instr, Instruction::STOP);

        let (instr, _) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0xF3);
        assert_eq!(instr, Instruction::DI);

        let (instr, _) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0xFB);
        assert_eq!(instr, Instruction::EI);
    }

    #[test]
    fn test_decode_cb_prefix() {
        let mut bus = MemoryBus::new(vec![0; 32768]);
        bus.write(0x0001, 0x00); // RLC B

        let (instr, bytes) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0xCB);
        match instr {
            Instruction::CB { cb_instr } => {
                assert_eq!(cb_instr, crate::cpu::instructions::CBInstruction::RLCR8 { reg: R8Register::B });
            }
            _ => panic!("Expected CB instruction"),
        }
        assert_eq!(bytes, 2);
    }

    #[test]
    fn test_decode_all_cb_prefix_opcodes() {
        let mut bus = MemoryBus::new(vec![0; 32768]);

        for cb_opcode in 0x00..=0xFF {
            bus.write(0x0001, cb_opcode);

            let (instr, bytes) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0xCB);

            match instr {
                Instruction::CB { cb_instr } => {
                    // Just verify it doesn't panic and returns a valid instruction
                    let _ = cb_instr;
                }
                _ => panic!("Expected CB instruction for cb_opcode={:02X}", cb_opcode),
            }
            assert_eq!(bytes, 2);
        }
    }

    #[test]
    fn test_decode_nop_for_invalid_opcodes() {
        let bus = MemoryBus::new(vec![0; 32768]);

        // Test some invalid opcodes that should decode to NOP
        let (instr, bytes) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0xD3);
        assert_eq!(instr, Instruction::NOP);
        assert_eq!(bytes, 1);

        let (instr, bytes) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0xDB);
        assert_eq!(instr, Instruction::LdhAC);
        assert_eq!(bytes, 1);

        let (instr, bytes) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0xDC);
        assert_eq!(bytes, 3); // CALL C, a16
    }

    #[test]
    fn test_decode_ld_ind_r16_mem_modes() {
        let bus = MemoryBus::new(vec![0; 32768]);

        // LD (HL), A
        let (instr, _) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0x32);
        assert_eq!(instr, Instruction::LdIndR16A { src: R16Mem::HLMinus });

        // LD A, (HL)
        let (instr, _) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0x3A);
        assert_eq!(instr, Instruction::LdAIndR16 { dest: R16Mem::HLMinus });

        // LD (HL+), A
        let (instr, _) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0x22);
        assert_eq!(instr, Instruction::LdIndR16A { src: R16Mem::HLPlus });

        // LD A, (HL+)
        let (instr, _) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0x2A);
        assert_eq!(instr, Instruction::LdAIndR16 { dest: R16Mem::HLPlus });
    }

    #[test]
    fn test_decode_ldh_a_c_opcode_0xf2() {
        let bus = MemoryBus::new(vec![0; 32768]);

        let (instr, _) = decode_instruction(&crate::cpu::CPUState::new(), &bus, 0x0000, 0xF2);
        assert_eq!(instr, Instruction::LdhAC);
    }
}
