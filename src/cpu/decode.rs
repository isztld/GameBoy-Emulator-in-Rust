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
        0x40..=0x7F => {
            let reg_src = R8Register::from_byte(opcode);
            let reg_dest = R8Register::from_byte((opcode & 0x07) | ((opcode >> 3) & 0x07));
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
