/// GameBoy CPU implementation
///
/// The CPU is based on the SM83, a 8-bit CPU compatible with GBZ80.

use crate::memory::MemoryBus;
use crate::cpu::{CPUState};
use crate::cpu::instructions::{Instruction, R8Register, R16Register, R16Mem, Condition, CBInstruction};
use crate::cpu::registers::Flags;

/// GameBoy CPU
#[derive(Debug)]
pub struct CPU {
    state: CPUState,
    cycles: u32, // Total cycles executed
    halted: bool,
    stop_halt: bool,
}

impl CPU {
    /// Create a new CPU
    pub fn new() -> Self {
        CPU {
            state: CPUState::new(),
            cycles: 0,
            halted: false,
            stop_halt: false,
        }
    }

    /// Reset the CPU to power-on state
    pub fn reset(&mut self) {
        self.state.registers.pc = 0x0100;
        self.state.registers.sp = 0xFFFE;
        self.state.registers.af = 0x01B0;
        self.state.registers.bc = 0x0013;
        self.state.registers.de = 0x00D8;
        self.state.registers.hl = 0x014D;
        self.state.ime = false;
        self.halted = false;
        self.stop_halt = false;
        self.cycles = 0;
    }

    /// Get current CPU state
    pub fn state(&self) -> &CPUState {
        &self.state
    }

    /// Get mutable CPU state
    pub fn state_mut(&mut self) -> &mut CPUState {
        &mut self.state
    }

    /// Get total cycles executed
    pub fn cycles(&self) -> u32 {
        self.cycles
    }

    /// Execute one instruction
    pub fn execute(&mut self, bus: &mut MemoryBus) -> u32 {
        if self.halted {
            // While halted, only interrupt handling consumes cycles
            // Return 1 cycle for HALT
            self.cycles += 1;
            return 1;
        }

        // Read opcode at PC
        let pc = self.state.registers.pc;
        let opcode = bus.read(pc);

        // Decode instruction
        let (instruction, opcode_bytes) = self.decode_instruction(opcode, bus, pc);

        // Execute instruction
        let cycles = self.execute_instruction(instruction, bus);

        // Update PC
        self.state.registers.pc += opcode_bytes as u16;

        // Update cycle count
        self.cycles += cycles as u32;

        cycles as u32
    }

    /// Decode an instruction from the opcode
    fn decode_instruction(&self, opcode: u8, bus: &MemoryBus, pc: u16) -> (Instruction, u8) {
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
                // Read the CB opcode byte
                let cb_opcode = bus.read(pc + 1);
                let cb_instr = CBInstruction::from_byte(opcode, cb_opcode);
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
                let address = bus.read(pc + 1);
                (Instruction::LdhIndImm8A { address }, 2)
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
                let address = bus.read(pc + 1);
                (Instruction::LdhAIndImm8 { address }, 2)
            }
            0xF1 => (Instruction::PopR16 { reg: R16Register::HL }, 1),
            0xF2 => (Instruction::LdhAC, 1),
            0xF3 => (Instruction::DI, 1),
            0xF5 => (Instruction::PushR16 { reg: R16Register::HL }, 1),
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

    /// Execute an instruction and return cycles taken
    fn execute_instruction(&mut self, instruction: Instruction, bus: &mut MemoryBus) -> u8 {
        match instruction {
            Instruction::NOP => 1,

            Instruction::LdR16Imm16 { dest, value } => {
                match dest {
                    R16Register::BC => self.state.registers.bc = value,
                    R16Register::DE => self.state.registers.de = value,
                    R16Register::HL => self.state.registers.hl = value,
                    R16Register::SP => self.state.registers.sp = value,
                }
                3
            }

            Instruction::LdIndR16A { src } => {
                let address = match src {
                    R16Mem::BC => self.state.registers.bc,
                    R16Mem::DE => self.state.registers.de,
                    _ => 0,
                };
                bus.write(address, self.state.registers.a());
                2
            }

            Instruction::LdAIndR16 { dest } => {
                let address = match dest {
                    R16Mem::BC => self.state.registers.bc,
                    R16Mem::DE => self.state.registers.de,
                    _ => 0,
                };
                self.state.registers.set_a(bus.read(address));
                2
            }

            Instruction::LdIndImm16Sp { address } => {
                bus.write(address, (self.state.registers.sp & 0x00FF) as u8);
                bus.write(address + 1, (self.state.registers.sp >> 8) as u8);
                5
            }

            Instruction::IncR16 { reg } => {
                match reg {
                    R16Register::BC => self.state.registers.bc = self.state.registers.bc.wrapping_add(1),
                    R16Register::DE => self.state.registers.de = self.state.registers.de.wrapping_add(1),
                    R16Register::HL => self.state.registers.hl = self.state.registers.hl.wrapping_add(1),
                    R16Register::SP => self.state.registers.sp = self.state.registers.sp.wrapping_add(1),
                }
                2
            }

            Instruction::DecR16 { reg } => {
                match reg {
                    R16Register::BC => self.state.registers.bc = self.state.registers.bc.wrapping_sub(1),
                    R16Register::DE => self.state.registers.de = self.state.registers.de.wrapping_sub(1),
                    R16Register::HL => self.state.registers.hl = self.state.registers.hl.wrapping_sub(1),
                    R16Register::SP => self.state.registers.sp = self.state.registers.sp.wrapping_sub(1),
                }
                2
            }

            Instruction::AddHlR16 { reg } => {
                let hl = self.state.registers.hl as u32;
                let val = match reg {
                    R16Register::BC => self.state.registers.bc as u32,
                    R16Register::DE => self.state.registers.de as u32,
                    R16Register::HL => self.state.registers.hl as u32,
                    R16Register::SP => self.state.registers.sp as u32,
                };
                let result = hl.wrapping_add(val);

                // Set flags
                let mut flags = self.state.registers.f();
                flags.set_zero(false);
                flags.set_subtraction(false);
                flags.set_half_carry((hl & 0x0FFF) + (val & 0x0FFF) > 0x0FFF);
                flags.set_carry(result > 0xFFFF);
                self.state.registers.set_f(flags);

                self.state.registers.hl = result as u16;
                2
            }

            Instruction::IncR8 { reg } => {
                let val = self.get_r8(reg, bus);
                self.set_r8(reg, val.wrapping_add(1), bus);
                // Update flags
                let mut flags = self.state.registers.f();
                let val = self.get_r8(reg, bus);
                flags.set_zero(val == 0);
                flags.set_subtraction(false);
                flags.set_half_carry((val & 0x0F) == 0x0F);
                self.state.registers.set_f(flags);
                1
            }

            Instruction::DecR8 { reg } => {
                let val = self.get_r8(reg, bus);
                self.set_r8(reg, val.wrapping_sub(1), bus);
                // Update flags
                let mut flags = self.state.registers.f();
                let val = self.get_r8(reg, bus);
                flags.set_zero(val == 0);
                flags.set_subtraction(true);
                flags.set_half_carry((val & 0x0F) == 0x00);
                self.state.registers.set_f(flags);
                1
            }

            Instruction::LdR8Imm8 { dest, value } => {
                self.set_r8(dest, value, bus);
                2
            }

            Instruction::RLCA => {
                let a = self.state.registers.a();
                let new_a = a.rotate_left(1);
                let mut flags = self.state.registers.f();
                flags.set_zero(false);
                flags.set_subtraction(false);
                flags.set_half_carry(false);
                flags.set_carry((a & 0x80) != 0);
                self.state.registers.set_a(new_a);
                self.state.registers.set_f(flags);
                1
            }

            Instruction::RRCA => {
                let a = self.state.registers.a();
                let new_a = a.rotate_right(1);
                let mut flags = self.state.registers.f();
                flags.set_zero(false);
                flags.set_subtraction(false);
                flags.set_half_carry(false);
                flags.set_carry((a & 0x01) != 0);
                self.state.registers.set_a(new_a);
                self.state.registers.set_f(flags);
                1
            }

            Instruction::RLA => {
                let a = self.state.registers.a();
                let carry = if self.state.registers.f().is_carry() { 1 } else { 0 };
                let new_a = (a << 1) | carry;
                let mut flags = self.state.registers.f();
                flags.set_zero(false);
                flags.set_subtraction(false);
                flags.set_half_carry(false);
                flags.set_carry((a & 0x80) != 0);
                self.state.registers.set_a(new_a);
                self.state.registers.set_f(flags);
                1
            }

            Instruction::RRA => {
                let a = self.state.registers.a();
                let carry = if self.state.registers.f().is_carry() { 0x80 } else { 0 };
                let new_a = (a >> 1) | carry;
                let mut flags = self.state.registers.f();
                flags.set_zero(false);
                flags.set_subtraction(false);
                flags.set_half_carry(false);
                flags.set_carry((a & 0x01) != 0);
                self.state.registers.set_a(new_a);
                self.state.registers.set_f(flags);
                1
            }

            Instruction::DAA => {
                // TODO: Implement DAA
                1
            }

            Instruction::CPL => {
                let a = self.state.registers.a();
                self.state.registers.set_a(!a);
                let mut flags = self.state.registers.f();
                flags.set_subtraction(true);
                flags.set_half_carry(true);
                self.state.registers.set_f(flags);
                1
            }

            Instruction::SCF => {
                let mut flags = self.state.registers.f();
                flags.set_zero(false);
                flags.set_subtraction(false);
                flags.set_half_carry(false);
                flags.set_carry(true);
                self.state.registers.set_f(flags);
                1
            }

            Instruction::CCF => {
                let mut flags = self.state.registers.f();
                flags.set_zero(false);
                flags.set_subtraction(false);
                flags.set_half_carry(false);
                flags.set_carry(!flags.is_carry());
                self.state.registers.set_f(flags);
                1
            }

            Instruction::JrImm8 { offset } => {
                self.state.registers.pc = self.state.registers.pc.wrapping_add(offset as i16 as u16);
                3
            }

            Instruction::JrCondImm8 { cond, offset } => {
                let should_jump = self.condition_met(cond);
                self.state.registers.pc = self.state.registers.pc.wrapping_add(offset as i16 as u16);
                if should_jump { 3 } else { 2 }
            }

            Instruction::STOP => {
                self.stop_halt = true;
                1
            }

            Instruction::HALT => {
                self.halted = true;
                1
            }

            Instruction::LdR8R8 { dest, src } => {
                self.set_r8(dest, self.get_r8(src, bus), bus);
                1
            }

            Instruction::AddAR8 { reg } => {
                let val = self.get_r8(reg, bus);
                let a = self.state.registers.a();
                let result = a.wrapping_add(val);
                self.state.registers.set_a(result);

                let mut flags = self.state.registers.f();
                flags.set_zero(result == 0);
                flags.set_subtraction(false);
                flags.set_half_carry((a & 0x0F) + (val & 0x0F) > 0x0F);
                flags.set_carry(a as u16 + val as u16 > 0xFF);
                self.state.registers.set_f(flags);
                1
            }

            Instruction::AdcAR8 { reg } => {
                let val = self.get_r8(reg, bus);
                let a = self.state.registers.a();

                // Carry input (0 or 1)
                let carry_in = if self.state.registers.f().is_carry() { 1 } else { 0 };

                // Perform 16-bit addition to detect carry
                let sum = a as u16 + val as u16 + carry_in as u16;
                let result = sum as u8;

                // Compute half-carry (carry from bit 3)
                let half_carry =
                    ((a & 0x0F) + (val & 0x0F) + carry_in) > 0x0F;

                // Set result
                self.state.registers.set_a(result);

                // Set flags explicitly
                let mut flags = Flags::new();
                flags.set_zero(result == 0);
                flags.set_subtraction(false);
                flags.set_half_carry(half_carry);
                flags.set_carry(sum > 0xFF);

                self.state.registers.set_f(flags);

                1 // cycles (ADC A, r8 = 1 machine cycle)
            }

            Instruction::SubAR8 { reg } => {
                let val = self.get_r8(reg, bus);
                let a = self.state.registers.a();
                let result = a.wrapping_sub(val);
                self.state.registers.set_a(result);

                let mut flags = self.state.registers.f();
                flags.set_zero(result == 0);
                flags.set_subtraction(true);
                flags.set_half_carry((a & 0x0F) < (val & 0x0F));
                flags.set_carry(a < val);
                self.state.registers.set_f(flags);
                1
            }

            Instruction::SbcAR8 { reg } => {
                let val = self.get_r8(reg, bus);
                let a = self.state.registers.a();
                let carry = if self.state.registers.f().is_carry() { 1 } else { 0 };
                let result = a.wrapping_sub(val).wrapping_sub(carry);
                self.state.registers.set_a(result);

                let mut flags = self.state.registers.f();
                flags.set_zero(result == 0);
                flags.set_subtraction(true);
                let carry_flag = (a & 0x0F) < (val & 0x0F) + carry;
                flags.set_half_carry(carry_flag);
                flags.set_carry(a < val + carry);
                self.state.registers.set_f(flags);
                1
            }

            Instruction::AndAR8 { reg } => {
                let val = self.get_r8(reg, bus);
                let result = self.state.registers.a() & val;
                self.state.registers.set_a(result);

                let mut flags = self.state.registers.f();
                flags.set_zero(result == 0);
                flags.set_subtraction(false);
                flags.set_half_carry(true);
                flags.set_carry(false);
                self.state.registers.set_f(flags);
                1
            }

            Instruction::XorAR8 { reg } => {
                let val = self.get_r8(reg, bus);
                let result = self.state.registers.a() ^ val;
                self.state.registers.set_a(result);

                let mut flags = self.state.registers.f();
                flags.set_zero(result == 0);
                flags.set_subtraction(false);
                flags.set_half_carry(false);
                flags.set_carry(false);
                self.state.registers.set_f(flags);
                1
            }

            Instruction::OrAR8 { reg } => {
                let val = self.get_r8(reg, bus);
                let result = self.state.registers.a() | val;
                self.state.registers.set_a(result);

                let mut flags = self.state.registers.f();
                flags.set_zero(result == 0);
                flags.set_subtraction(false);
                flags.set_half_carry(false);
                flags.set_carry(false);
                self.state.registers.set_f(flags);
                1
            }

            Instruction::CpAR8 { reg } => {
                let val = self.get_r8(reg, bus);
                let a = self.state.registers.a();
                let result = a.wrapping_sub(val);

                let mut flags = self.state.registers.f();
                flags.set_zero(result == 0);
                flags.set_subtraction(true);
                flags.set_half_carry((a & 0x0F) < (val & 0x0F));
                flags.set_carry(a < val);
                self.state.registers.set_f(flags);
                1
            }

            Instruction::AddAImm8 { value } => {
                let a = self.state.registers.a();
                let result = a.wrapping_add(value);
                self.state.registers.set_a(result);

                let mut flags = self.state.registers.f();
                flags.set_zero(result == 0);
                flags.set_subtraction(false);
                flags.set_half_carry((a & 0x0F) + (value & 0x0F) > 0x0F);
                flags.set_carry(a as u16 + value as u16 > 0xFF);
                self.state.registers.set_f(flags);
                2
            }

            Instruction::AdcAImm8 { value } => {
                let a = self.state.registers.a();

                // Read carry before modifying flags
                let carry = if self.state.registers.f().is_carry() { 1 } else { 0 };

                // Compute result
                let result = a.wrapping_add(value).wrapping_add(carry);

                // Compute flags
                let half_carry = ((a & 0x0F) + (value & 0x0F) + carry) > 0x0F;
                let carry_flag = (a as u16 + value as u16 + carry as u16) > 0xFF;

                // Write back A
                self.state.registers.set_a(result);

                // Build a new Flags object and write it back
                let mut flags = Flags::new();
                flags.set_zero(result == 0);
                flags.set_subtraction(false);
                flags.set_half_carry(half_carry);
                flags.set_carry(carry_flag);

                self.state.registers.set_f(flags);

                2 // cycles
            }

            Instruction::SubAImm8 { value } => {
                let a = self.state.registers.a();
                let result = a.wrapping_sub(value);
                self.state.registers.set_a(result);

                let mut flags = self.state.registers.f();
                flags.set_zero(result == 0);
                flags.set_subtraction(true);
                flags.set_half_carry((a & 0x0F) < (value & 0x0F));
                flags.set_carry(a < value);
                self.state.registers.set_f(flags);
                2
            }

            Instruction::SbcAImm8 { value } => {
                let a = self.state.registers.a();
                let carry = if self.state.registers.f().is_carry() { 1 } else { 0 };
                let result = a.wrapping_sub(value).wrapping_sub(carry);
                self.state.registers.set_a(result);

                let mut flags = self.state.registers.f();
                flags.set_zero(result == 0);
                flags.set_subtraction(true);
                let carry_flag = (a & 0x0F) < (value & 0x0F) + carry;
                flags.set_half_carry(carry_flag);
                flags.set_carry(a < value + carry);
                self.state.registers.set_f(flags);
                2
            }

            Instruction::AndAImm8 { value } => {
                let result = self.state.registers.a() & value;
                self.state.registers.set_a(result);

                let mut flags = self.state.registers.f();
                flags.set_zero(result == 0);
                flags.set_subtraction(false);
                flags.set_half_carry(true);
                flags.set_carry(false);
                self.state.registers.set_f(flags);
                2
            }

            Instruction::XorAImm8 { value } => {
                let result = self.state.registers.a() ^ value;
                self.state.registers.set_a(result);

                let mut flags = self.state.registers.f();
                flags.set_zero(result == 0);
                flags.set_subtraction(false);
                flags.set_half_carry(false);
                flags.set_carry(false);
                self.state.registers.set_f(flags);
                2
            }

            Instruction::OrAImm8 { value } => {
                let result = self.state.registers.a() | value;
                self.state.registers.set_a(result);

                let mut flags = self.state.registers.f();
                flags.set_zero(result == 0);
                flags.set_subtraction(false);
                flags.set_half_carry(false);
                flags.set_carry(false);
                self.state.registers.set_f(flags);
                2
            }

            Instruction::CpAImm8 { value } => {
                let a = self.state.registers.a();
                let result = a.wrapping_sub(value);

                let mut flags = self.state.registers.f();
                flags.set_zero(result == 0);
                flags.set_subtraction(true);
                flags.set_half_carry((a & 0x0F) < (value & 0x0F));
                flags.set_carry(a < value);
                self.state.registers.set_f(flags);
                2
            }

            Instruction::RetCond { cond } => {
                if self.condition_met(cond) {
                    let low = bus.read(self.state.registers.sp) as u8;
                    self.state.registers.sp = self.state.registers.sp.wrapping_add(1);
                    let high = bus.read(self.state.registers.sp) as u8;
                    self.state.registers.sp = self.state.registers.sp.wrapping_add(1);
                    let address = (high as u16) << 8 | low as u16;
                    self.state.registers.pc = address;
                    5
                } else {
                    2
                }
            }

            Instruction::RET => {
                let low = bus.read(self.state.registers.sp) as u8;
                self.state.registers.sp = self.state.registers.sp.wrapping_add(1);
                let high = bus.read(self.state.registers.sp) as u8;
                self.state.registers.sp = self.state.registers.sp.wrapping_add(1);
                let address = (high as u16) << 8 | low as u16;
                self.state.registers.pc = address;
                4
            }

            Instruction::RETI => {
                // RETI pops PC and enables interrupts
                let low = bus.read(self.state.registers.sp) as u8;
                self.state.registers.sp = self.state.registers.sp.wrapping_add(1);
                let high = bus.read(self.state.registers.sp) as u8;
                self.state.registers.sp = self.state.registers.sp.wrapping_add(1);
                let address = (high as u16) << 8 | low as u16;
                self.state.registers.pc = address;
                self.state.ime = true;
                4
            }

            Instruction::JpCondImm16 { cond, address } => {
                if self.condition_met(cond) {
                    self.state.registers.pc = address;
                }
                3
            }

            Instruction::JpImm16 { address } => {
                self.state.registers.pc = address;
                4
            }

            Instruction::JpHl => {
                self.state.registers.pc = self.state.registers.hl;
                1
            }

            Instruction::CallCondImm16 { cond, address } => {
                if self.condition_met(cond) {
                    let sp = self.state.registers.sp;
                    bus.write(sp.wrapping_sub(1), (self.state.registers.pc >> 8) as u8);
                    bus.write(sp.wrapping_sub(2), (self.state.registers.pc & 0x00FF) as u8);
                    self.state.registers.sp = sp.wrapping_sub(2);
                    self.state.registers.pc = address;
                    6
                } else {
                    3
                }
            }

            Instruction::CallImm16 { address } => {
                let sp = self.state.registers.sp;
                bus.write(sp.wrapping_sub(1), (self.state.registers.pc >> 8) as u8);
                bus.write(sp.wrapping_sub(2), (self.state.registers.pc & 0x00FF) as u8);
                self.state.registers.sp = sp.wrapping_sub(2);
                self.state.registers.pc = address;
                6
            }

            Instruction::RST { target } => {
                let sp = self.state.registers.sp;
                bus.write(sp.wrapping_sub(1), (self.state.registers.pc >> 8) as u8);
                bus.write(sp.wrapping_sub(2), (self.state.registers.pc & 0x00FF) as u8);
                self.state.registers.sp = sp.wrapping_sub(2);
                self.state.registers.pc = target as u16;
                4
            }

            Instruction::PopR16 { reg } => {
                let low = bus.read(self.state.registers.sp) as u8;
                self.state.registers.sp = self.state.registers.sp.wrapping_add(1);
                let high = bus.read(self.state.registers.sp) as u8;
                self.state.registers.sp = self.state.registers.sp.wrapping_add(1);
                let value = (high as u16) << 8 | low as u16;
                match reg {
                    R16Register::BC => self.state.registers.bc = value,
                    R16Register::DE => self.state.registers.de = value,
                    R16Register::HL => self.state.registers.hl = value,
                    R16Register::SP => self.state.registers.sp = value,
                }
                3
            }

            Instruction::PushR16 { reg } => {
                let sp = self.state.registers.sp;
                let value = match reg {
                    R16Register::BC => self.state.registers.bc,
                    R16Register::DE => self.state.registers.de,
                    R16Register::HL => self.state.registers.hl,
                    R16Register::SP => self.state.registers.sp,
                };
                bus.write(sp.wrapping_sub(2), (value & 0x00FF) as u8);
                self.state.registers.sp = sp.wrapping_sub(2);
                5
            }

            Instruction::LdhIndCA => {
                let address = 0xFF00 | (self.state.registers.c() as u16);
                bus.write(address, self.state.registers.a());
                2
            }

            Instruction::LdhIndImm8A { address } => {
                let address = 0xFF00 | (address as u16);
                bus.write(address, self.state.registers.a());
                3
            }

            Instruction::LdIndImm16A { address } => {
                bus.write(address, self.state.registers.a());
                4
            }

            Instruction::LdhAC => {
                let address = 0xFF00 | (self.state.registers.c() as u16);
                self.state.registers.set_a(bus.read(address));
                2
            }

            Instruction::LdhAIndImm8 { address } => {
                let address = 0xFF00 | (address as u16);
                self.state.registers.set_a(bus.read(address));
                3
            }

            Instruction::LdAIndImm16 { address } => {
                self.state.registers.set_a(bus.read(address));
                4
            }

            Instruction::AddSpImm8 { value } => {
                let sp = self.state.registers.sp as i16;
                let offset = value as i16;
                let result = sp.wrapping_add(offset) as u16;

                // Update flags
                let mut flags = self.state.registers.f();
                flags.set_zero(false);
                flags.set_subtraction(false);
                flags.set_half_carry(((sp as u16 & 0x0F) + (offset as u16 & 0x0F)) > 0x0F);
                flags.set_carry(((sp as u16 & 0xFF) + (offset as u16 & 0xFF)) > 0xFF);
                self.state.registers.set_f(flags);

                self.state.registers.sp = result;
                4
            }

            Instruction::LdHlSpImm8 { value } => {
                let sp = self.state.registers.sp as i16;
                let offset = value as i16;
                let result = sp.wrapping_add(offset) as u16;

                self.state.registers.hl = result;

                // Update flags
                let mut flags = self.state.registers.f();
                flags.set_zero(false);
                flags.set_subtraction(false);
                flags.set_half_carry(((self.state.registers.sp as u16 & 0x0F) + (value as u16 & 0x0F)) > 0x0F);
                flags.set_carry(((self.state.registers.sp as u16 & 0xFF) + (value as u16 & 0xFF)) > 0xFF);
                self.state.registers.set_f(flags);
                3
            }

            Instruction::LdSpHl => {
                self.state.registers.sp = self.state.registers.hl;
                2
            }

            Instruction::DI => {
                self.state.ime = false;
                1
            }

            Instruction::EI => {
                self.state.ime = true;
                1
            }

            Instruction::CB { cb_instr } => {
                self.execute_cb_instruction(cb_instr, bus)
            }
        }
    }

    /// Execute a CB-prefixed instruction
    fn execute_cb_instruction(&mut self, cb_instr: CBInstruction, bus: &mut MemoryBus) -> u8 {
        match cb_instr {
            CBInstruction::RLCR8 { reg } => {
                let val = self.get_r8(reg, bus);
                let result = val.rotate_left(1);
                self.set_r8(reg, result, bus);
                let mut flags = self.state.registers.f();
                flags.set_zero(false);
                flags.set_subtraction(false);
                flags.set_half_carry(false);
                flags.set_carry((val & 0x80) != 0);
                self.state.registers.set_f(flags);
                2
            }
            CBInstruction::RRCR8 { reg } => {
                let val = self.get_r8(reg, bus);
                let result = val.rotate_right(1);
                self.set_r8(reg, result, bus);
                let mut flags = self.state.registers.f();
                flags.set_zero(false);
                flags.set_subtraction(false);
                flags.set_half_carry(false);
                flags.set_carry((val & 0x01) != 0);
                self.state.registers.set_f(flags);
                2
            }
            CBInstruction::RLR8 { reg } => {
                let val = self.get_r8(reg, bus);
                let carry = if self.state.registers.f().is_carry() { 1 } else { 0 };
                let result = (val << 1) | carry;
                self.set_r8(reg, result, bus);
                let mut flags = self.state.registers.f();
                flags.set_zero(false);
                flags.set_subtraction(false);
                flags.set_half_carry(false);
                flags.set_carry((val & 0x80) != 0);
                self.state.registers.set_f(flags);
                2
            }
            CBInstruction::RRR8 { reg } => {
                let val = self.get_r8(reg, bus);
                let carry = if self.state.registers.f().is_carry() { 0x80 } else { 0 };
                let result = (val >> 1) | carry;
                self.set_r8(reg, result, bus);
                let mut flags = self.state.registers.f();
                flags.set_zero(false);
                flags.set_subtraction(false);
                flags.set_half_carry(false);
                flags.set_carry((val & 0x01) != 0);
                self.state.registers.set_f(flags);
                2
            }
            CBInstruction::SLAR8 { reg } => {
                let val = self.get_r8(reg, bus);
                let result = (val << 1) | ((val & 0x80) >> 7);
                self.set_r8(reg, result, bus);
                let mut flags = self.state.registers.f();
                flags.set_zero(result == 0);
                flags.set_subtraction(false);
                flags.set_half_carry(false);
                flags.set_carry((val & 0x80) != 0);
                self.state.registers.set_f(flags);
                2
            }
            CBInstruction::SRAR8 { reg } => {
                let val = self.get_r8(reg, bus);
                let result = (val as i8 >> 1) as u8;
                self.set_r8(reg, result, bus);
                let mut flags = self.state.registers.f();
                flags.set_zero(result == 0);
                flags.set_subtraction(false);
                flags.set_half_carry(false);
                flags.set_carry((val & 0x01) != 0);
                self.state.registers.set_f(flags);
                2
            }
            CBInstruction::SWAPR8 { reg } => {
                let val = self.get_r8(reg, bus);
                let result = (val << 4) | (val >> 4);
                self.set_r8(reg, result, bus);
                let mut flags = self.state.registers.f();
                flags.set_zero(result == 0);
                flags.set_subtraction(false);
                flags.set_half_carry(false);
                flags.set_carry(false);
                self.state.registers.set_f(flags);
                2
            }
            CBInstruction::SRLR8 { reg } => {
                let val = self.get_r8(reg, bus);
                let result = val >> 1;
                self.set_r8(reg, result, bus);
                let mut flags = self.state.registers.f();
                flags.set_zero(result == 0);
                flags.set_subtraction(false);
                flags.set_half_carry(false);
                flags.set_carry((val & 0x01) != 0);
                self.state.registers.set_f(flags);
                2
            }
            CBInstruction::BITBR8 { bit, reg } => {
                let val = self.get_r8(reg, bus);
                let mut flags = self.state.registers.f();
                flags.set_zero((val & (1 << bit)) == 0);
                flags.set_subtraction(false);
                flags.set_half_carry(true);
                // Flags N and C unchanged, Z set if bit not set
                self.state.registers.set_f(flags);
                2
            }
            CBInstruction::RESBR8 { bit, reg } => {
                let val = self.get_r8(reg, bus);
                let result = val & !(1 << bit);
                self.set_r8(reg, result, bus);
                2
            }
            CBInstruction::SETBR8 { bit, reg } => {
                let val = self.get_r8(reg, bus);
                let result = val | (1 << bit);
                self.set_r8(reg, result, bus);
                2
            }
        }
    }

    /// Get value of 8-bit register (or memory at HL)
    fn get_r8(&self, reg: R8Register, bus: &MemoryBus) -> u8 {
        match reg {
            R8Register::B => self.state.registers.b(),
            R8Register::C => self.state.registers.c(),
            R8Register::D => self.state.registers.d(),
            R8Register::E => self.state.registers.e(),
            R8Register::H => self.state.registers.h(),
            R8Register::L => self.state.registers.l(),
            R8Register::HL => {
                let hl = self.state.registers.hl;
                bus.read(hl)
            }
            R8Register::A => self.state.registers.a(),
        }
    }

    /// Set value of 8-bit register (or memory at HL)
    fn set_r8(&mut self, reg: R8Register, value: u8, bus: &mut MemoryBus) {
        match reg {
            R8Register::B => self.state.registers.set_b(value),
            R8Register::C => self.state.registers.set_c(value),
            R8Register::D => self.state.registers.set_d(value),
            R8Register::E => self.state.registers.set_e(value),
            R8Register::H => self.state.registers.set_h(value),
            R8Register::L => self.state.registers.set_l(value),
            R8Register::HL => {
                let hl = self.state.registers.hl;
                bus.write(hl, value);
            }
            R8Register::A => self.state.registers.set_a(value),
        }
    }

    /// Check if condition is met
    fn condition_met(&self, cond: Condition) -> bool {
        let flags = self.state.registers.f();
        match cond {
            Condition::NZ => !flags.is_zero(),
            Condition::Z => flags.is_zero(),
            Condition::NC => !flags.is_carry(),
            Condition::C => flags.is_carry(),
        }
    }
}

impl Default for CPU {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu::Registers;

    #[test]
    fn test_cpu_reset() {
        let mut cpu = CPU::new();
        cpu.reset();
        assert_eq!(cpu.state.registers.pc, 0x0100); // GameBoy boot ROM entry point
        assert_eq!(cpu.state.registers.sp, 0xFFFE);
    }

    // Helper function to set up CPU for instruction test
    fn setup_cpu_for_test(initial_regs: Registers, initial_mem: Vec<(u16, u8)>) -> (CPU, MemoryBus) {
        let mut cpu = CPU::new();
        cpu.state.registers = initial_regs;
        cpu.state.ime = false;

        let mut rom = vec![0; 65536];
        for (addr, val) in initial_mem {
            if (addr as usize) < rom.len() {
                rom[addr as usize] = val;
            }
        }

        let bus = MemoryBus::new(rom);

        (cpu, bus)
    }

    #[test]
    fn test_nop() {
        let mut cpu = CPU::new();
        cpu.state.registers.pc = 0x0000;
        let mut bus = MemoryBus::new(vec![0x00, 0x00]); // NOP instruction
        let cycles = cpu.execute(&mut bus);
        assert_eq!(cycles, 1);
        assert_eq!(cpu.state.registers.pc, 1); // PC should increment by 1
    }

    #[test]
    fn test_ld_r16_imm16_bc() {
        let mut cpu = CPU::new();
        cpu.state.registers.pc = 0x0000;
        let rom = vec![0x01, 0x34, 0x12]; // LD BC, $1234
        let mut bus = MemoryBus::new(rom);
        let cycles = cpu.execute(&mut bus);
        assert_eq!(cycles, 3);
        assert_eq!(cpu.state.registers.bc, 0x1234);
    }

    #[test]
    fn test_ld_r16_imm16_de() {
        let mut cpu = CPU::new();
        cpu.state.registers.pc = 0x0000;
        let rom = vec![0x11, 0x56, 0x78]; // LD DE, $7856
        let mut bus = MemoryBus::new(rom);
        let cycles = cpu.execute(&mut bus);
        assert_eq!(cycles, 3);
        assert_eq!(cpu.state.registers.de, 0x7856);
    }

    #[test]
    fn test_ld_r16_imm16_hl() {
        let mut cpu = CPU::new();
        cpu.state.registers.pc = 0x0000;
        let rom = vec![0x21, 0x9A, 0xBC]; // LD HL, $BC9A
        let mut bus = MemoryBus::new(rom);
        let cycles = cpu.execute(&mut bus);
        assert_eq!(cycles, 3);
        assert_eq!(cpu.state.registers.hl, 0xBC9A);
    }

    #[test]
    fn test_ld_r16_imm16_sp() {
        let mut cpu = CPU::new();
        cpu.state.registers.pc = 0x0000;
        let rom = vec![0x31, 0x00, 0xC0]; // LD SP, $C000
        let mut bus = MemoryBus::new(rom);
        let cycles = cpu.execute(&mut bus);
        assert_eq!(cycles, 3);
        assert_eq!(cpu.state.registers.sp, 0xC000);
    }

    #[test]
    fn test_ld_ind_r16_a_bc() {
        let mut cpu = CPU::new();
        cpu.state.registers.bc = 0xC000;
        cpu.state.registers.set_a(0xAB);
        cpu.state.registers.pc = 0x0000;
        let rom = vec![0x02]; // LD [BC], A
        let mut bus = MemoryBus::new(rom);
        let cycles = cpu.execute(&mut bus);
        assert_eq!(cycles, 2);
        assert_eq!(bus.read(0xC000), 0xAB);
    }

    #[test]
    fn test_ld_ind_r16_a_de() {
        let mut cpu = CPU::new();
        cpu.state.registers.de = 0xD000;
        cpu.state.registers.set_a(0xCD);
        cpu.state.registers.pc = 0x0000;
        let rom = vec![0x12]; // LD [DE], A
        let mut bus = MemoryBus::new(rom);
        let cycles = cpu.execute(&mut bus);
        assert_eq!(cycles, 2);
        assert_eq!(bus.read(0xD000), 0xCD);
    }

    #[test]
    fn test_ld_a_ind_r16_bc() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0x0A]); // LD A, [BC]
        cpu.state.registers.bc = 0xE000;
        bus.write(0xE000, 0xEF);
        cpu.state.registers.pc = 0x0000;
        let cycles = cpu.execute(&mut bus);
        assert_eq!(cycles, 2);
        assert_eq!(cpu.state.registers.a(), 0xEF);
    }

    #[test]
    fn test_inc_r16_bc() {
        let mut cpu = CPU::new();
        cpu.state.registers.bc = 0x1234;
        cpu.state.registers.pc = 0x0000;
        let rom = vec![0x03]; // INC BC
        let mut bus = MemoryBus::new(rom);
        let cycles = cpu.execute(&mut bus);
        assert_eq!(cycles, 1);
        assert_eq!(cpu.state.registers.bc, 0x1235);
    }

    #[test]
    fn test_dec_r16_bc() {
        let mut cpu = CPU::new();
        cpu.state.registers.bc = 0x1234;
        cpu.state.registers.pc = 0x0000;
        let rom = vec![0x0B]; // DEC BC
        let mut bus = MemoryBus::new(rom);
        let cycles = cpu.execute(&mut bus);
        assert_eq!(cycles, 1);
        assert_eq!(cpu.state.registers.bc, 0x1233);
    }

    #[test]
    fn test_inc_r8_b() {
        let mut cpu = CPU::new();
        cpu.state.registers.set_b(0x42);
        cpu.state.registers.pc = 0x0000;
        let rom = vec![0x04]; // INC B
        let mut bus = MemoryBus::new(rom);
        let cycles = cpu.execute(&mut bus);
        assert_eq!(cycles, 1);
        assert_eq!(cpu.state.registers.b(), 0x43);
        // Check zero flag is not set
        assert!(!cpu.state.registers.f().is_zero());
    }

    #[test]
    fn test_inc_r8_b_zero_flag() {
        let mut cpu = CPU::new();
        cpu.state.registers.set_b(0xFF);
        cpu.state.registers.pc = 0x0000;
        let rom = vec![0x04]; // INC B
        let mut bus = MemoryBus::new(rom);
        let cycles = cpu.execute(&mut bus);
        assert_eq!(cycles, 1);
        assert_eq!(cpu.state.registers.b(), 0x00);
        assert!(cpu.state.registers.f().is_zero());
    }

    #[test]
    fn test_dec_r8_b() {
        let mut cpu = CPU::new();
        cpu.state.registers.set_b(0x42);
        cpu.state.registers.pc = 0x0000;
        let rom = vec![0x05]; // DEC B
        let mut bus = MemoryBus::new(rom);
        let cycles = cpu.execute(&mut bus);
        assert_eq!(cycles, 1);
        assert_eq!(cpu.state.registers.b(), 0x41);
        assert!(cpu.state.registers.f().is_subtraction());
    }

    #[test]
    fn test_ld_r8_imm8_b() {
        let mut cpu = CPU::new();
        cpu.state.registers.pc = 0x0000;
        let rom = vec![0x06, 0x42]; // LD B, $42
        let mut bus = MemoryBus::new(rom);
        let cycles = cpu.execute(&mut bus);
        assert_eq!(cycles, 2);
        assert_eq!(cpu.state.registers.b(), 0x42);
    }

    #[test]
    fn test_rlca() {
        let mut cpu = CPU::new();
        cpu.state.registers.set_a(0b10110010); // 0xB2
        cpu.state.registers.pc = 0x0000;
        let rom = vec![0x07]; // RLCA
        let mut bus = MemoryBus::new(rom);
        let cycles = cpu.execute(&mut bus);
        assert_eq!(cycles, 1);
        // Rotate left: 0b01100101 with carry out = 1
        assert_eq!(cpu.state.registers.a(), 0b01100101);
        assert!(cpu.state.registers.f().is_carry());
        assert!(!cpu.state.registers.f().is_zero());
    }

    #[test]
    fn test_rrca() {
        let mut cpu = CPU::new();
        cpu.state.registers.set_a(0b10110010); // 0xB2
        cpu.state.registers.pc = 0x0000;
        let rom = vec![0x0F]; // RRCA
        let mut bus = MemoryBus::new(rom);
        let cycles = cpu.execute(&mut bus);
        assert_eq!(cycles, 1);
        // Rotate right: 0b01011001 with carry out = 0
        assert_eq!(cpu.state.registers.a(), 0b01011001);
        assert!(!cpu.state.registers.f().is_carry());
    }

    #[test]
    fn test_rla() {
        let mut cpu = CPU::new();
        cpu.state.registers.set_a(0b10110010); // 0xB2
        cpu.state.registers.pc = 0x0000;
        let rom = vec![0x17]; // RLA
        let mut bus = MemoryBus::new(rom);
        let cycles = cpu.execute(&mut bus);
        assert_eq!(cycles, 1);
        // Rotate left through carry: 0b01100101 with carry out = 1
        assert_eq!(cpu.state.registers.a(), 0b01100101);
        assert!(cpu.state.registers.f().is_carry());
    }

    #[test]
    fn test_rra() {
        let mut cpu = CPU::new();
        cpu.state.registers.set_a(0b10110010); // 0xB2
        cpu.state.registers.pc = 0x0000;
        let rom = vec![0x1F]; // RRA
        let mut bus = MemoryBus::new(rom);
        let cycles = cpu.execute(&mut bus);
        assert_eq!(cycles, 1);
        // Rotate right through carry: 0b11011001 with carry out = 0
        assert_eq!(cpu.state.registers.a(), 0b11011001);
        assert!(!cpu.state.registers.f().is_carry());
    }

    #[test]
    fn test_jr_imm8() {
        let mut cpu = CPU::new();
        cpu.state.registers.pc = 0x0000;
        let rom = vec![0x18, 0x02]; // JR +2
        let mut bus = MemoryBus::new(rom);
        let cycles = cpu.execute(&mut bus);
        assert_eq!(cycles, 2);
        // PC should be at 0x0003 after the JR (PC was 0 before execute, now at 3)
        assert_eq!(cpu.state.registers.pc, 3);
    }

    #[test]
    fn test_jr_backward() {
        let mut cpu = CPU::new();
        cpu.state.registers.pc = 0x0002;
        let rom = vec![0x18, 0xFE]; // JR -2 (back to 0x0001)
        let mut bus = MemoryBus::new(rom);
        let cycles = cpu.execute(&mut bus);
        assert_eq!(cycles, 2);
        assert_eq!(cpu.state.registers.pc, 0x0000);
    }

    #[test]
    fn test_jr_cond_nz_taken() {
        let mut cpu = CPU::new();
        cpu.state.registers.pc = 0x0000;
        let rom = vec![0x20, 0x02]; // JR NZ, +2
        let mut bus = MemoryBus::new(rom);
        let cycles = cpu.execute(&mut bus);
        assert_eq!(cycles, 2);
        assert_eq!(cpu.state.registers.pc, 3);
    }

    #[test]
    fn test_jr_cond_nz_not_taken() {
        let mut cpu = CPU::new();
        cpu.state.registers.pc = 0x0000;
        let mut flags = cpu.state.registers.f(); // copy current flags
        flags.set_zero(true);                     // set Z
        cpu.state.registers.set_f(flags);        // write back
        let rom = vec![0x20, 0x02]; // JR NZ, +2 (but Z is set, so not taken)
        let mut bus = MemoryBus::new(rom);
        let cycles = cpu.execute(&mut bus);
        assert_eq!(cycles, 2); // 2 cycles for conditional not taken
        assert_eq!(cpu.state.registers.pc, 2); // Only increment by 2 (opcode + offset)
    }

    #[test]
    fn test_jr_cond_z_taken() {
        let mut cpu = CPU::new();
        cpu.state.registers.pc = 0x0000;
        let mut flags = cpu.state.registers.f(); // copy current flags
        flags.set_zero(true);                     // set Z
        cpu.state.registers.set_f(flags);        // write back
        let rom = vec![0x28, 0x02]; // JR Z, +2
        let mut bus = MemoryBus::new(rom);
        let cycles = cpu.execute(&mut bus);
        assert_eq!(cycles, 2);
        assert_eq!(cpu.state.registers.pc, 3);
    }

    #[test]
    fn test_jp_imm16() {
        let mut cpu = CPU::new();
        cpu.state.registers.pc = 0x0000;
        let rom = vec![0xC3, 0x50, 0x80]; // JP $8050
        let mut bus = MemoryBus::new(rom);
        let cycles = cpu.execute(&mut bus);
        assert_eq!(cycles, 4);
        assert_eq!(cpu.state.registers.pc, 0x8050);
    }

    #[test]
    fn test_call_imm16() {
        let mut cpu = CPU::new();
        cpu.state.registers.pc = 0x0000;
        cpu.state.registers.sp = 0xFFFE;
        let rom = vec![0xCD, 0x50, 0x80]; // CALL $8050
        let mut bus = MemoryBus::new(rom);
        let cycles = cpu.execute(&mut bus);
        assert_eq!(cycles, 6);
        assert_eq!(cpu.state.registers.pc, 0x8050);
        // Check stack has return address
        assert_eq!(bus.read(0xFFFC), 0x03); // High byte
        assert_eq!(bus.read(0xFFFD), 0x00); // Low byte
        assert_eq!(cpu.state.registers.sp, 0xFFFC);
    }

    #[test]
    fn test_ret() {
        let mut cpu = CPU::new();
        cpu.state.registers.pc = 0x0000;
        cpu.state.registers.sp = 0xFFFC;
        let rom = vec![0xC9]; // RET
        let mut bus = MemoryBus::new(rom);
        // Set up return address on stack
        bus.write(0xFFFC, 0x12);
        bus.write(0xFFFD, 0x34);
        let cycles = cpu.execute(&mut bus);
        assert_eq!(cycles, 4);
        assert_eq!(cpu.state.registers.pc, 0x3412);
        assert_eq!(cpu.state.registers.sp, 0xFFFE);
    }

    #[test]
    fn test_push_pop_bc() {
        let mut cpu = CPU::new();
        cpu.state.registers.bc = 0xABCD;
        cpu.state.registers.pc = 0x0000;
        cpu.state.registers.sp = 0xFFFE;
        let rom = vec![0xC5, 0xC1]; // PUSH BC, POP BC
        let mut bus = MemoryBus::new(rom);
        let cycles_push = cpu.execute(&mut bus);
        assert_eq!(cycles_push, 1);
        assert_eq!(bus.read(0xFFFC), 0xCD);
        assert_eq!(bus.read(0xFFFD), 0xAB);
        assert_eq!(cpu.state.registers.sp, 0xFFFC);

        let cycles_pop = cpu.execute(&mut bus);
        assert_eq!(cycles_pop, 3);
        assert_eq!(cpu.state.registers.bc, 0xABCD);
        assert_eq!(cpu.state.registers.sp, 0xFFFE);
    }

    #[test]
    fn test_add_hl_bc() {
        let mut cpu = CPU::new();
        cpu.state.registers.hl = 0x1234;
        cpu.state.registers.bc = 0x5678;
        cpu.state.registers.pc = 0x0000;
        let rom = vec![0x09]; // ADD HL, BC
        let mut bus = MemoryBus::new(rom);
        let cycles = cpu.execute(&mut bus);
        assert_eq!(cycles, 2);
        assert_eq!(cpu.state.registers.hl, 0x68AC);
        assert!(!cpu.state.registers.f().is_subtraction());
        assert!(!cpu.state.registers.f().is_carry());
    }

    #[test]
    fn test_add_hl_bc_carry() {
        let mut cpu = CPU::new();
        cpu.state.registers.hl = 0xFFFF;
        cpu.state.registers.bc = 0x0001;
        cpu.state.registers.pc = 0x0000;
        let rom = vec![0x09]; // ADD HL, BC
        let mut bus = MemoryBus::new(rom);
        let cycles = cpu.execute(&mut bus);
        assert_eq!(cycles, 2);
        assert_eq!(cpu.state.registers.hl, 0x0000);
        assert!(cpu.state.registers.f().is_carry());
    }

    #[test]
    fn test_add_hl_bc_half_carry() {
        let mut cpu = CPU::new();
        cpu.state.registers.hl = 0x0FFF;
        cpu.state.registers.bc = 0x0001;
        cpu.state.registers.pc = 0x0000;
        let rom = vec![0x09]; // ADD HL, BC
        let mut bus = MemoryBus::new(rom);
        let cycles = cpu.execute(&mut bus);
        assert_eq!(cycles, 2);
        assert_eq!(cpu.state.registers.hl, 0x1000);
        assert!(cpu.state.registers.f().is_half_carry());
    }

    #[test]
    fn test_add_a_r8() {
        let mut cpu = CPU::new();
        cpu.state.registers.set_a(0x10);
        cpu.state.registers.set_b(0x20);
        cpu.state.registers.pc = 0x0000;
        let rom = vec![0x80]; // ADD A, B
        let mut bus = MemoryBus::new(rom);
        let cycles = cpu.execute(&mut bus);
        assert_eq!(cycles, 1);
        assert_eq!(cpu.state.registers.a(), 0x30);
        assert!(!cpu.state.registers.f().is_carry());
        assert!(!cpu.state.registers.f().is_zero());
    }

    #[test]
    fn test_add_a_r8_carry() {
        let mut cpu = CPU::new();
        cpu.state.registers.set_a(0x80);
        cpu.state.registers.set_b(0x80);
        cpu.state.registers.pc = 0x0000;
        let rom = vec![0x80]; // ADD A, B
        let mut bus = MemoryBus::new(rom);
        let cycles = cpu.execute(&mut bus);
        assert_eq!(cycles, 1);
        assert_eq!(cpu.state.registers.a(), 0x00);
        assert!(cpu.state.registers.f().is_carry());
        assert!(cpu.state.registers.f().is_zero());
    }

    #[test]
    fn test_add_a_r8_half_carry() {
        let mut cpu = CPU::new();
        cpu.state.registers.set_a(0x0F);
        cpu.state.registers.set_b(0x01);
        cpu.state.registers.pc = 0x0000;
        let rom = vec![0x80]; // ADD A, B
        let mut bus = MemoryBus::new(rom);
        let cycles = cpu.execute(&mut bus);
        assert_eq!(cycles, 1);
        assert_eq!(cpu.state.registers.a(), 0x10);
        assert!(cpu.state.registers.f().is_half_carry());
    }

    #[test]
    fn test_adc_a_r8_with_carry() {
        let mut cpu = CPU::new();
        cpu.state.registers.set_a(0x10);
        cpu.state.registers.set_b(0x20);
        cpu.state.registers.pc = 0x0000;
        let mut flags = cpu.state.registers.f(); // copy current flags
        flags.set_carry(true);                    // modify the copy
        cpu.state.registers.set_f(flags);         // write it back into AF
        let rom = vec![0x88]; // ADC A, B
        let mut bus = MemoryBus::new(rom);
        let cycles = cpu.execute(&mut bus);
        assert_eq!(cycles, 1);
        assert_eq!(cpu.state.registers.a(), 0x31); // 0x10 + 0x20 + 1
    }

    #[test]
    fn test_sub_a_r8() {
        let mut cpu = CPU::new();
        cpu.state.registers.set_a(0x30);
        cpu.state.registers.set_b(0x10);
        cpu.state.registers.pc = 0x0000;
        let rom = vec![0x90]; // SUB A, B
        let mut bus = MemoryBus::new(rom);
        let cycles = cpu.execute(&mut bus);
        assert_eq!(cycles, 1);
        assert_eq!(cpu.state.registers.a(), 0x20);
        assert!(cpu.state.registers.f().is_subtraction());
        assert!(!cpu.state.registers.f().is_carry());
    }

    #[test]
    fn test_sub_a_r8_borrow() {
        let mut cpu = CPU::new();
        cpu.state.registers.set_a(0x10);
        cpu.state.registers.set_b(0x30);
        cpu.state.registers.pc = 0x0000;
        let rom = vec![0x90]; // SUB A, B
        let mut bus = MemoryBus::new(rom);
        let cycles = cpu.execute(&mut bus);
        assert_eq!(cycles, 1);
        assert_eq!(cpu.state.registers.a(), 0xE0);
        assert!(cpu.state.registers.f().is_carry());
    }

    #[test]
    fn test_sbc_a_r8_with_carry() {
        let mut cpu = CPU::new();
        cpu.state.registers.set_a(0x30);
        cpu.state.registers.set_b(0x10);
        cpu.state.registers.pc = 0x0000;
        cpu.state.registers.f().set_carry(true);
        let rom = vec![0x98]; // SBC A, B
        let mut bus = MemoryBus::new(rom);
        let cycles = cpu.execute(&mut bus);
        assert_eq!(cycles, 1);
        assert_eq!(cpu.state.registers.a(), 0x1F); // 0x30 - 0x10 - 1
    }

    #[test]
    fn test_and_a_r8() {
        let mut cpu = CPU::new();
        cpu.state.registers.set_a(0b11110000);
        cpu.state.registers.set_b(0b10101010);
        cpu.state.registers.pc = 0x0000;
        let rom = vec![0xA0]; // AND A, B
        let mut bus = MemoryBus::new(rom);
        let cycles = cpu.execute(&mut bus);
        assert_eq!(cycles, 1);
        assert_eq!(cpu.state.registers.a(), 0b10100000);
        assert!(cpu.state.registers.f().is_half_carry());
        assert!(!cpu.state.registers.f().is_carry());
        assert!(!cpu.state.registers.f().is_zero());
    }

    #[test]
    fn test_and_a_r8_zero() {
        let mut cpu = CPU::new();
        cpu.state.registers.set_a(0xFF);
        cpu.state.registers.set_b(0x00);
        cpu.state.registers.pc = 0x0000;
        let rom = vec![0xA0]; // AND A, B
        let mut bus = MemoryBus::new(rom);
        let cycles = cpu.execute(&mut bus);
        assert_eq!(cycles, 1);
        assert_eq!(cpu.state.registers.a(), 0x00);
        assert!(cpu.state.registers.f().is_zero());
    }

    #[test]
    fn test_or_a_r8() {
        let mut cpu = CPU::new();
        cpu.state.registers.set_a(0b11110000);
        cpu.state.registers.set_b(0b10101010);
        cpu.state.registers.pc = 0x0000;
        let rom = vec![0xB0]; // OR A, B
        let mut bus = MemoryBus::new(rom);
        let cycles = cpu.execute(&mut bus);
        assert_eq!(cycles, 1);
        assert_eq!(cpu.state.registers.a(), 0b11111010);
        assert!(!cpu.state.registers.f().is_half_carry());
        assert!(!cpu.state.registers.f().is_carry());
        assert!(!cpu.state.registers.f().is_zero());
    }

    #[test]
    fn test_xor_a_r8() {
        let mut cpu = CPU::new();
        cpu.state.registers.set_a(0b11110000);
        cpu.state.registers.set_b(0b10101010);
        cpu.state.registers.pc = 0x0000;
        let rom = vec![0xA8]; // XOR A, B
        let mut bus = MemoryBus::new(rom);
        let cycles = cpu.execute(&mut bus);
        assert_eq!(cycles, 1);
        assert_eq!(cpu.state.registers.a(), 0b01011010);
    }

    #[test]
    fn test_cp_a_r8() {
        let mut cpu = CPU::new();
        cpu.state.registers.set_a(0x30);
        cpu.state.registers.set_b(0x10);
        cpu.state.registers.pc = 0x0000;
        let rom = vec![0xB8]; // CP A, B
        let mut bus = MemoryBus::new(rom);
        let cycles = cpu.execute(&mut bus);
        assert_eq!(cycles, 1);
        // CP should not change A
        assert_eq!(cpu.state.registers.a(), 0x30);
        // But flags should be set as if subtraction
        assert!(cpu.state.registers.f().is_subtraction());
        assert!(!cpu.state.registers.f().is_carry());
    }

    #[test]
    fn test_cp_a_r8_equal() {
        let mut cpu = CPU::new();
        cpu.state.registers.set_a(0x42);
        cpu.state.registers.set_b(0x42);
        cpu.state.registers.pc = 0x0000;
        let rom = vec![0xB8]; // CP A, B
        let mut bus = MemoryBus::new(rom);
        let cycles = cpu.execute(&mut bus);
        assert_eq!(cycles, 1);
        assert_eq!(cpu.state.registers.a(), 0x42);
        assert!(cpu.state.registers.f().is_zero());
    }

    #[test]
    fn test_add_a_imm8() {
        let mut cpu = CPU::new();
        cpu.state.registers.set_a(0x10);
        cpu.state.registers.pc = 0x0000;
        let rom = vec![0xC6, 0x20]; // ADD A, $20
        let mut bus = MemoryBus::new(rom);
        let cycles = cpu.execute(&mut bus);
        assert_eq!(cycles, 2);
        assert_eq!(cpu.state.registers.a(), 0x30);
    }

    #[test]
    fn test_adc_a_imm8() {
        let mut cpu = CPU::new();
        cpu.state.registers.set_a(0x10);
        let mut flags = cpu.state.registers.f(); // copy current flags
        flags.set_carry(true);                    // modify the copy
        cpu.state.registers.set_f(flags);         // write it back into AF
        cpu.state.registers.pc = 0x0000;
        let rom = vec![0xCE, 0x20]; // ADC A, $20
        let mut bus = MemoryBus::new(rom);
        let cycles = cpu.execute(&mut bus);
        assert_eq!(cycles, 2);
        assert_eq!(cpu.state.registers.a(), 0x31);
    }

    #[test]
    fn test_sub_a_imm8() {
        let mut cpu = CPU::new();
        cpu.state.registers.set_a(0x30);
        cpu.state.registers.pc = 0x0000;
        let rom = vec![0xD6, 0x10]; // SUB A, $10
        let mut bus = MemoryBus::new(rom);
        let cycles = cpu.execute(&mut bus);
        assert_eq!(cycles, 2);
        assert_eq!(cpu.state.registers.a(), 0x20);
    }

    #[test]
    fn test_sbc_a_imm8() {
        let mut cpu = CPU::new();
        cpu.state.registers.set_a(0x30);
        cpu.state.registers.f().set_carry(true);
        cpu.state.registers.pc = 0x0000;
        let rom = vec![0xDE, 0x10]; // SBC A, $10
        let mut bus = MemoryBus::new(rom);
        let cycles = cpu.execute(&mut bus);
        assert_eq!(cycles, 2);
        assert_eq!(cpu.state.registers.a(), 0x1F);
    }

    #[test]
    fn test_and_a_imm8() {
        let mut cpu = CPU::new();
        cpu.state.registers.set_a(0b11110000);
        cpu.state.registers.pc = 0x0000;
        let rom = vec![0xE6, 0b10101010]; // AND A, $AA
        let mut bus = MemoryBus::new(rom);
        let cycles = cpu.execute(&mut bus);
        assert_eq!(cycles, 2);
        assert_eq!(cpu.state.registers.a(), 0b10100000);
    }

    #[test]
    fn test_xor_a_imm8() {
        let mut cpu = CPU::new();
        cpu.state.registers.set_a(0b11110000);
        cpu.state.registers.pc = 0x0000;
        let rom = vec![0xEE, 0b10101010]; // XOR A, $AA
        let mut bus = MemoryBus::new(rom);
        let cycles = cpu.execute(&mut bus);
        assert_eq!(cycles, 2);
        assert_eq!(cpu.state.registers.a(), 0b01011010);
    }

    #[test]
    fn test_or_a_imm8() {
        let mut cpu = CPU::new();
        cpu.state.registers.set_a(0b11110000);
        cpu.state.registers.pc = 0x0000;
        let rom = vec![0xF6, 0b10101010]; // OR A, $AA
        let mut bus = MemoryBus::new(rom);
        let cycles = cpu.execute(&mut bus);
        assert_eq!(cycles, 2);
        assert_eq!(cpu.state.registers.a(), 0b11111010);
    }

    #[test]
    fn test_cp_a_imm8() {
        let mut cpu = CPU::new();
        cpu.state.registers.set_a(0x30);
        cpu.state.registers.pc = 0x0000;
        let rom = vec![0xFE, 0x10]; // CP A, $10
        let mut bus = MemoryBus::new(rom);
        let cycles = cpu.execute(&mut bus);
        assert_eq!(cycles, 2);
        assert_eq!(cpu.state.registers.a(), 0x30);
        assert!(cpu.state.registers.f().is_subtraction());
    }

    #[test]
    fn test_ld_r8_r8_b_to_c() {
        let mut cpu = CPU::new();
        cpu.state.registers.set_b(0x42);
        cpu.state.registers.pc = 0x0000;
        let rom = vec![0x01 << 3 | 0x01]; // LD C, B (0x09)
        let mut bus = MemoryBus::new(rom);
        let cycles = cpu.execute(&mut bus);
        assert_eq!(cycles, 1);
        assert_eq!(cpu.state.registers.c(), 0x42);
    }

    #[test]
    fn test_cpl() {
        let mut cpu = CPU::new();
        cpu.state.registers.set_a(0b10101010);
        cpu.state.registers.pc = 0x0000;
        let rom = vec![0x2F]; // CPL
        let mut bus = MemoryBus::new(rom);
        let cycles = cpu.execute(&mut bus);
        assert_eq!(cycles, 1);
        assert_eq!(cpu.state.registers.a(), 0b01010101);
        assert!(cpu.state.registers.f().is_subtraction());
        assert!(cpu.state.registers.f().is_half_carry());
    }

    #[test]
    fn test_scf() {
        let mut cpu = CPU::new();
        cpu.state.registers.pc = 0x0000;
        let rom = vec![0x37]; // SCF
        let mut bus = MemoryBus::new(rom);
        let cycles = cpu.execute(&mut bus);
        assert_eq!(cycles, 1);
        assert!(cpu.state.registers.f().is_carry());
        assert!(!cpu.state.registers.f().is_subtraction());
    }

    #[test]
    fn test_ccf() {
        let mut cpu = CPU::new();
        cpu.state.registers.pc = 0x0000;
        let rom = vec![0x3F]; // CCF
        let mut bus = MemoryBus::new(rom);
        let cycles = cpu.execute(&mut bus);
        assert_eq!(cycles, 1);
        // Should toggle carry flag
        assert!(cpu.state.registers.f().is_carry());
    }

    #[test]
    fn test_ccf_with_carry() {
        let mut cpu = CPU::new();
        cpu.state.registers.f().set_carry(true);
        cpu.state.registers.pc = 0x0000;
        let rom = vec![0x3F]; // CCF
        let mut bus = MemoryBus::new(rom);
        let cycles = cpu.execute(&mut bus);
        assert_eq!(cycles, 1);
        assert!(!cpu.state.registers.f().is_carry());
    }

    #[test]
    fn test_daa() {
        let mut cpu = CPU::new();
        cpu.state.registers.set_a(0x12);
        cpu.state.registers.pc = 0x0000;
        let rom = vec![0x27]; // DAA
        let mut bus = MemoryBus::new(rom);
        let cycles = cpu.execute(&mut bus);
        assert_eq!(cycles, 1);
        // DAA without carry should just return value
        assert_eq!(cpu.state.registers.a(), 0x12);
    }

    #[test]
    fn test_di() {
        let mut cpu = CPU::new();
        cpu.state.ime = true;
        cpu.state.registers.pc = 0x0000;
        let rom = vec![0xF3]; // DI
        let mut bus = MemoryBus::new(rom);
        let cycles = cpu.execute(&mut bus);
        assert_eq!(cycles, 1);
        assert!(!cpu.state.ime);
    }

    #[test]
    fn test_ei() {
        let mut cpu = CPU::new();
        cpu.state.ime = false;
        cpu.state.registers.pc = 0x0000;
        let rom = vec![0xFB]; // EI
        let mut bus = MemoryBus::new(rom);
        let cycles = cpu.execute(&mut bus);
        assert_eq!(cycles, 1);
        assert!(cpu.state.ime);
    }

    #[test]
    fn test_ldh_ind_imm8_a() {
        let mut cpu = CPU::new();
        cpu.state.registers.set_a(0xAB);
        cpu.state.registers.pc = 0x0000;
        let rom = vec![0xE0, 0x30]; // LDH [$30], A
        let mut bus = MemoryBus::new(rom);
        let cycles = cpu.execute(&mut bus);
        assert_eq!(cycles, 2);
        assert_eq!(bus.read(0xFF30), 0xAB);
    }

    #[test]
    fn test_ldh_a_ind_imm8() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0xF0, 0x30]); // LDH A, [$30]
        bus.write(0xFF30, 0xCD);
        cpu.state.registers.pc = 0x0000;
        let cycles = cpu.execute(&mut bus);
        assert_eq!(cycles, 2);
        assert_eq!(cpu.state.registers.a(), 0xCD);
    }

    #[test]
    fn test_ld_ind_imm16_a() {
        let mut cpu = CPU::new();
        cpu.state.registers.set_a(0xAB);
        cpu.state.registers.pc = 0x0000;
        let rom = vec![0xEA, 0x34, 0x12]; // LD [$1234], A
        let mut bus = MemoryBus::new(rom);
        let cycles = cpu.execute(&mut bus);
        assert_eq!(cycles, 4);
        assert_eq!(bus.read(0x1234), 0xAB);
    }

    #[test]
    fn test_ld_a_ind_imm16() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0xFA, 0x34, 0x12]); // LD A, [$1234]
        bus.write(0x1234, 0xCD);
        cpu.state.registers.pc = 0x0000;
        let cycles = cpu.execute(&mut bus);
        assert_eq!(cycles, 4);
        assert_eq!(cpu.state.registers.a(), 0xCD);
    }

    #[test]
    fn test_add_sp_imm8() {
        let mut cpu = CPU::new();
        cpu.state.registers.sp = 0xFF00;
        cpu.state.registers.pc = 0x0000;
        let rom = vec![0xE8, 0x10]; // ADD SP, $10
        let mut bus = MemoryBus::new(rom);
        let cycles = cpu.execute(&mut bus);
        assert_eq!(cycles, 4);
        assert_eq!(cpu.state.registers.sp, 0xFF10);
        assert!(!cpu.state.registers.f().is_carry());
    }

    #[test]
    fn test_ld_hl_sp_imm8() {
        let mut cpu = CPU::new();
        cpu.state.registers.sp = 0xFF00;
        cpu.state.registers.pc = 0x0000;
        let rom = vec![0xF8, 0x10]; // LD HL, SP+$10
        let mut bus = MemoryBus::new(rom);
        let cycles = cpu.execute(&mut bus);
        assert_eq!(cycles, 3);
        assert_eq!(cpu.state.registers.hl, 0xFF10);
    }

    #[test]
    fn test_ld_sp_hl() {
        let mut cpu = CPU::new();
        cpu.state.registers.hl = 0xC000;
        cpu.state.registers.pc = 0x0000;
        let rom = vec![0xF9]; // LD SP, HL
        let mut bus = MemoryBus::new(rom);
        let cycles = cpu.execute(&mut bus);
        assert_eq!(cycles, 2);
        assert_eq!(cpu.state.registers.sp, 0xC000);
    }

    #[test]
    fn test_jp_hl() {
        let mut cpu = CPU::new();
        cpu.state.registers.hl = 0x8000;
        cpu.state.registers.pc = 0x0000;
        let rom = vec![0xE9]; // JP HL
        let mut bus = MemoryBus::new(rom);
        let cycles = cpu.execute(&mut bus);
        assert_eq!(cycles, 1);
        assert_eq!(cpu.state.registers.pc, 0x8000);
    }

    #[test]
    fn test_jp_cond_nz_taken() {
        let mut cpu = CPU::new();
        cpu.state.registers.pc = 0x0000;
        let rom = vec![0xC2, 0x50, 0x80]; // JP NZ, $8050
        let mut bus = MemoryBus::new(rom);
        let cycles = cpu.execute(&mut bus);
        assert_eq!(cycles, 3);
        assert_eq!(cpu.state.registers.pc, 0x8050);
    }

    #[test]
    fn test_jp_cond_nz_not_taken() {
        let mut cpu = CPU::new();
        cpu.state.registers.pc = 0x0000;
        let mut flags = cpu.state.registers.f(); // copy current flags
        flags.set_zero(true);                     // set Z
        cpu.state.registers.set_f(flags);        // write back
        let rom = vec![0xC2, 0x50, 0x80]; // JP NZ, $8050 (but Z is set)
        let mut bus = MemoryBus::new(rom);
        let cycles = cpu.execute(&mut bus);
        assert_eq!(cycles, 3);
        assert_eq!(cpu.state.registers.pc, 0x0003); // Not jumped, so PC = 3
    }

    #[test]
    fn test_jp_cond_z_taken() {
        let mut cpu = CPU::new();
        cpu.state.registers.pc = 0x0000;
        let mut flags = cpu.state.registers.f(); // copy current flags
        flags.set_zero(true);                     // set Z
        cpu.state.registers.set_f(flags);        // write back
        let rom = vec![0xCA, 0x50, 0x80]; // JP Z, $8050
        let mut bus = MemoryBus::new(rom);
        let cycles = cpu.execute(&mut bus);
        assert_eq!(cycles, 3);
        assert_eq!(cpu.state.registers.pc, 0x8050);
    }

    #[test]
    fn test_jp_cond_nc_taken() {
        let mut cpu = CPU::new();
        cpu.state.registers.pc = 0x0000;
        let rom = vec![0xD2, 0x50, 0x80]; // JP NC, $8050
        let mut bus = MemoryBus::new(rom);
        let cycles = cpu.execute(&mut bus);
        assert_eq!(cycles, 3);
        assert_eq!(cpu.state.registers.pc, 0x8050);
    }

    #[test]
    fn test_jp_cond_c_taken() {
        let mut cpu = CPU::new();
        cpu.state.registers.pc = 0x0000;
        cpu.state.registers.f().set_carry(true);
        let rom = vec![0xDA, 0x50, 0x80]; // JP C, $8050
        let mut bus = MemoryBus::new(rom);
        let cycles = cpu.execute(&mut bus);
        assert_eq!(cycles, 3);
        assert_eq!(cpu.state.registers.pc, 0x8050);
    }

    #[test]
    fn test_call_cond_nz_taken() {
        let mut cpu = CPU::new();
        cpu.state.registers.pc = 0x0000;
        cpu.state.registers.sp = 0xFFFE;
        let rom = vec![0xC4, 0x50, 0x80]; // CALL NZ, $8050
        let mut bus = MemoryBus::new(rom);
        let cycles = cpu.execute(&mut bus);
        assert_eq!(cycles, 6);
        assert_eq!(cpu.state.registers.pc, 0x8050);
    }

    #[test]
    fn test_call_cond_nz_not_taken() {
        let mut cpu = CPU::new();
        cpu.state.registers.pc = 0x0000;
        let mut flags = cpu.state.registers.f(); // copy current flags
        flags.set_zero(true);                     // set Z
        cpu.state.registers.set_f(flags);        // write back
        let rom = vec![0xC4, 0x50, 0x80]; // CALL NZ, $8050 (Z is set)
        let mut bus = MemoryBus::new(rom);
        let cycles = cpu.execute(&mut bus);
        assert_eq!(cycles, 3);
        assert_eq!(cpu.state.registers.pc, 0x0003);
    }

    #[test]
    fn test_call_cond_z_taken() {
        let mut cpu = CPU::new();
        cpu.state.registers.pc = 0x0000;
        cpu.state.registers.sp = 0xFFFE;
        let mut flags = cpu.state.registers.f(); // copy current flags
        flags.set_zero(true);                     // set Z
        cpu.state.registers.set_f(flags);        // write back
        let rom = vec![0xCC, 0x50, 0x80]; // CALL Z, $8050
        let mut bus = MemoryBus::new(rom);
        let cycles = cpu.execute(&mut bus);
        assert_eq!(cycles, 6);
        assert_eq!(cpu.state.registers.pc, 0x8050);
    }

    #[test]
    fn test_rst() {
        let mut cpu = CPU::new();
        cpu.state.registers.pc = 0x0000;
        cpu.state.registers.sp = 0xFFFE;
        let rom = vec![0xC7]; // RST $00
        let mut bus = MemoryBus::new(rom);
        let cycles = cpu.execute(&mut bus);
        assert_eq!(cycles, 4);
        assert_eq!(cpu.state.registers.pc, 0x0000);
        assert_eq!(cpu.state.registers.sp, 0xFFFC);
    }

    #[test]
    fn test_rst_08() {
        let mut cpu = CPU::new();
        cpu.state.registers.pc = 0x0000;
        cpu.state.registers.sp = 0xFFFE;
        let rom = vec![0xCF]; // RST $08
        let mut bus = MemoryBus::new(rom);
        let cycles = cpu.execute(&mut bus);
        assert_eq!(cycles, 4);
        assert_eq!(cpu.state.registers.pc, 0x0008);
    }

    #[test]
    fn test_ret_cond_nz_taken() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0xC0]); // RET NZ
        cpu.state.registers.pc = 0x0000;
        cpu.state.registers.sp = 0xFFFC;
        bus.write(0xFFFC, 0x50);
        bus.write(0xFFFD, 0x80);
        let cycles = cpu.execute(&mut bus);
        assert_eq!(cycles, 5);
        assert_eq!(cpu.state.registers.pc, 0x8050);
    }

    #[test]
    fn test_ret_cond_nz_not_taken() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0xC0]); // RET NZ (Z is set, so not taken)
        cpu.state.registers.pc = 0x0000;
        cpu.state.registers.sp = 0xFFFC;
        let mut flags = cpu.state.registers.f(); // copy current flags
        flags.set_zero(true);                     // set Z
        cpu.state.registers.set_f(flags);        // write back
        bus.write(0xFFFC, 0x50);
        bus.write(0xFFFD, 0x80);
        let cycles = cpu.execute(&mut bus);
        assert_eq!(cycles, 2);
        assert_eq!(cpu.state.registers.pc, 0x0001);
    }

    #[test]
    fn test_reti() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0xD9]); // RETI
        cpu.state.ime = false;
        cpu.state.registers.pc = 0x0000;
        cpu.state.registers.sp = 0xFFFC;
        bus.write(0xFFFC, 0x50);
        bus.write(0xFFFD, 0x80);
        let cycles = cpu.execute(&mut bus);
        assert_eq!(cycles, 4);
        assert_eq!(cpu.state.registers.pc, 0x8050);
        assert!(cpu.state.ime);
    }

    #[test]
    fn test_halt() {
        let mut cpu = CPU::new();
        cpu.state.registers.pc = 0x0000;
        let rom = vec![0x76]; // HALT
        let mut bus = MemoryBus::new(rom);
        let cycles = cpu.execute(&mut bus);
        assert_eq!(cycles, 1);
        assert!(cpu.halted);
    }

    #[test]
    fn test_halt_with_interrupt() {
        let mut cpu = CPU::new();
        cpu.state.registers.pc = 0x0000;
        cpu.state.ime = true;
        cpu.state.registers.set_a(0x11);
        cpu.state.registers.set_b(0x22);
        let rom = vec![0x76, 0x00]; // HALT, then NOP
        let mut bus = MemoryBus::new(rom);
        // Trigger interrupt after HALT
        cpu.execute(&mut bus);
        assert!(cpu.halted);
        // After interrupt, CPU should continue
        cpu.state.ime = false; // Clear for next execute
        let cycles = cpu.execute(&mut bus);
        assert!(!cpu.halted);
        assert_eq!(cpu.state.registers.pc, 2);
        assert_eq!(cycles, 1); // Just 1 cycle for NOP after HALT
    }

    #[test]
    fn test_stop() {
        let mut cpu = CPU::new();
        cpu.state.registers.pc = 0x0000;
        let rom = vec![0x10]; // STOP
        let mut bus = MemoryBus::new(rom);
        let cycles = cpu.execute(&mut bus);
        assert_eq!(cycles, 1);
        assert!(cpu.stop_halt);
    }

    #[test]
    fn test_cycles_across_instructions() {
        let mut cpu = CPU::new();
        cpu.state.registers.pc = 0x0000;
        // NOP (1) + LD BC, $1234 (3) + ADD A, B (1)
        let rom = vec![0x00, 0x01, 0x34, 0x12, 0x80];
        let mut bus = MemoryBus::new(rom);
        let total_cycles = cpu.execute(&mut bus) + cpu.execute(&mut bus) + cpu.execute(&mut bus);
        assert_eq!(total_cycles, 1 + 3 + 1);
    }

    #[test]
    fn test_cpu_state_getters() {
        let cpu = CPU::new();
        assert_eq!(cpu.cycles(), 0);
        assert!(!cpu.state().ime);
    }

    #[test]
    fn test_cpu_state_mut() {
        let mut cpu = CPU::new();
        cpu.state_mut().ime = true;
        assert!(cpu.state().ime);
    }
}
