/// GameBoy CPU implementation
///
/// The CPU is based on the SM83, a 8-bit CPU compatible with GBZ80.

use crate::memory::MemoryBus;
use crate::cpu::{CPUState};
use crate::cpu::instructions::{Instruction, R8Register, R16Register, R16Mem, Condition};

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
        self.state.registers.pc = 0x0000;
        self.state.registers.sp = 0xFFFE;
        self.state.registers.af = 0x0000;
        self.state.registers.bc = 0x0000;
        self.state.registers.de = 0x0000;
        self.state.registers.hl = 0x0000;
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
            0x00 => (Instruction::NOP, 1),
            0x01 => {
                let low = bus.read(pc + 1) as u8;
                let high = bus.read(pc + 2) as u8;
                let value = (high as u16) << 8 | low as u16;
                (Instruction::LD_R16_IMM16 { dest: R16Register::BC, value }, 3)
            }
            0x02 => (Instruction::LD_IND_R16_A { src: R16Mem::BC }, 1),
            0x03 => (Instruction::INC_R16 { reg: R16Register::BC }, 1),
            0x04 => (Instruction::INC_R8 { reg: R8Register::B }, 1),
            0x05 => (Instruction::DEC_R8 { reg: R8Register::B }, 1),
            0x06 => {
                let value = bus.read(pc + 1);
                (Instruction::LD_R8_IMM8 { dest: R8Register::B, value }, 2)
            }
            0x07 => (Instruction::RLCA, 1),
            0x08 => {
                let low = bus.read(pc + 1) as u8;
                let high = bus.read(pc + 2) as u8;
                let address = (high as u16) << 8 | low as u16;
                (Instruction::LD_IND_IMM16_SP { address }, 3)
            }
            0x09 => (Instruction::ADD_HL_R16 { reg: R16Register::BC }, 1),
            0x0A => (Instruction::LD_A_IND_R16 { dest: R16Mem::BC }, 1),
            0x0B => (Instruction::DEC_R16 { reg: R16Register::BC }, 1),
            0x0C => (Instruction::INC_R8 { reg: R8Register::C }, 1),
            0x0D => (Instruction::DEC_R8 { reg: R8Register::C }, 1),
            0x0E => {
                let value = bus.read(pc + 1);
                (Instruction::LD_R8_IMM8 { dest: R8Register::C, value }, 2)
            }
            0x0F => (Instruction::RRCA, 1),

            // Block 1: 8-bit register loads (40-7F)
            0x40..=0x7F => {
                let reg_src = R8Register::from_byte(opcode);
                let reg_dest = R8Register::from_byte((opcode & 0x07) | ((opcode >> 3) & 0x07));
                (Instruction::LD_R8_R8 { dest: reg_dest, src: reg_src }, 1)
            }

            // Block 2: 8-bit arithmetic (80-BF)
            0x80..=0xBF => {
                let reg = R8Register::from_byte(opcode);
                match opcode & 0xC7 {
                    0x80 => (Instruction::ADD_A_R8 { reg }, 1),
                    0x88 => (Instruction::ADC_A_R8 { reg }, 1),
                    0x90 => (Instruction::SUB_A_R8 { reg }, 1),
                    0x98 => (Instruction::SBC_A_R8 { reg }, 1),
                    0xA0 => (Instruction::AND_A_R8 { reg }, 1),
                    0xA8 => (Instruction::XOR_A_R8 { reg }, 1),
                    0xB0 => (Instruction::OR_A_R8 { reg }, 1),
                    0xB8 => (Instruction::CP_A_R8 { reg }, 1),
                    _ => (Instruction::NOP, 1),
                }
            }

            // Block 3: Jumps, calls, returns (C0-FF)
            0xC0 => (Instruction::RET_COND { cond: Condition::NZ }, 1),
            0xC1 => (Instruction::POP_R16 { reg: R16Register::BC }, 1),
            0xC2 => {
                let low = bus.read(pc + 1) as u8;
                let high = bus.read(pc + 2) as u8;
                let address = (high as u16) << 8 | low as u16;
                (Instruction::JP_COND_IMM16 { cond: Condition::NZ, address }, 3)
            }
            0xC3 => {
                let low = bus.read(pc + 1) as u8;
                let high = bus.read(pc + 2) as u8;
                let address = (high as u16) << 8 | low as u16;
                (Instruction::JP_IMM16 { address }, 3)
            }
            0xC4 => {
                let low = bus.read(pc + 1) as u8;
                let high = bus.read(pc + 2) as u8;
                let address = (high as u16) << 8 | low as u16;
                (Instruction::CALL_COND_IMM16 { cond: Condition::NZ, address }, 3)
            }
            0xC5 => (Instruction::PUSH_R16 { reg: R16Register::BC }, 1),
            0xC6 => {
                let value = bus.read(pc + 1);
                (Instruction::ADD_A_IMM8 { value }, 2)
            }
            0xC7 => (Instruction::RST { target: 0x00 }, 1),
            0xC8 => (Instruction::RET_COND { cond: Condition::Z }, 1),
            0xC9 => (Instruction::RET, 1),
            0xCA => {
                let low = bus.read(pc + 1) as u8;
                let high = bus.read(pc + 2) as u8;
                let address = (high as u16) << 8 | low as u16;
                (Instruction::JP_COND_IMM16 { cond: Condition::Z, address }, 3)
            }
            0xCB => {
                let cb_opcode = bus.read(pc + 1);
                let cb_instr = crate::cpu::instructions::CBInstruction::from_byte(opcode, cb_opcode);
                // CB instructions are handled separately
                // For now, return NOP with 2 bytes
                (Instruction::NOP, 2)
            }
            0xCC => {
                let low = bus.read(pc + 1) as u8;
                let high = bus.read(pc + 2) as u8;
                let address = (high as u16) << 8 | low as u16;
                (Instruction::CALL_COND_IMM16 { cond: Condition::Z, address }, 3)
            }
            0xCD => {
                let low = bus.read(pc + 1) as u8;
                let high = bus.read(pc + 2) as u8;
                let address = (high as u16) << 8 | low as u16;
                (Instruction::CALL_IMM16 { address }, 3)
            }
            0xCE => {
                let value = bus.read(pc + 1);
                (Instruction::ADC_A_IMM8 { value }, 2)
            }
            0xCF => (Instruction::RST { target: 0x08 }, 1),
            0xD0 => (Instruction::RET_COND { cond: Condition::NC }, 1),
            0xD1 => (Instruction::POP_R16 { reg: R16Register::DE }, 1),
            0xD2 => {
                let low = bus.read(pc + 1) as u8;
                let high = bus.read(pc + 2) as u8;
                let address = (high as u16) << 8 | low as u16;
                (Instruction::JP_COND_IMM16 { cond: Condition::NC, address }, 3)
            }
            0xD4 => {
                let low = bus.read(pc + 1) as u8;
                let high = bus.read(pc + 2) as u8;
                let address = (high as u16) << 8 | low as u16;
                (Instruction::CALL_COND_IMM16 { cond: Condition::NC, address }, 3)
            }
            0xD5 => (Instruction::PUSH_R16 { reg: R16Register::DE }, 1),
            0xD6 => {
                let value = bus.read(pc + 1);
                (Instruction::SUB_A_IMM8 { value }, 2)
            }
            0xD7 => (Instruction::RST { target: 0x10 }, 1),
            0xD8 => (Instruction::RET_COND { cond: Condition::C }, 1),
            0xD9 => (Instruction::RETI, 1),
            0xDA => {
                let low = bus.read(pc + 1) as u8;
                let high = bus.read(pc + 2) as u8;
                let address = (high as u16) << 8 | low as u16;
                (Instruction::JP_COND_IMM16 { cond: Condition::C, address }, 3)
            }
            0xDC => {
                let low = bus.read(pc + 1) as u8;
                let high = bus.read(pc + 2) as u8;
                let address = (high as u16) << 8 | low as u16;
                (Instruction::CALL_COND_IMM16 { cond: Condition::C, address }, 3)
            }
            0xDE => {
                let value = bus.read(pc + 1);
                (Instruction::SBC_A_IMM8 { value }, 2)
            }
            0xDF => (Instruction::RST { target: 0x18 }, 1),
            0xE0 => {
                let address = bus.read(pc + 1);
                (Instruction::LDH_IND_IMM8_A { address }, 2)
            }
            0xE1 => (Instruction::POP_R16 { reg: R16Register::HL }, 1),
            0xE2 => (Instruction::LDH_IND_C_A, 1),
            0xE5 => (Instruction::PUSH_R16 { reg: R16Register::HL }, 1),
            0xE6 => {
                let value = bus.read(pc + 1);
                (Instruction::AND_A_IMM8 { value }, 2)
            }
            0xE7 => (Instruction::RST { target: 0x20 }, 1),
            0xE8 => {
                let value = bus.read(pc + 1) as i8;
                (Instruction::ADD_SP_IMM8 { value }, 2)
            }
            0xE9 => (Instruction::JP_HL, 1),
            0xEA => {
                let low = bus.read(pc + 1) as u8;
                let high = bus.read(pc + 2) as u8;
                let address = (high as u16) << 8 | low as u16;
                (Instruction::LD_IND_IMM16_A { address }, 3)
            }
            0xEE => {
                let value = bus.read(pc + 1);
                (Instruction::XOR_A_IMM8 { value }, 2)
            }
            0xEF => (Instruction::RST { target: 0x28 }, 1),
            0xF0 => {
                let address = bus.read(pc + 1);
                (Instruction::LDH_A_IND_IMM8 { address }, 2)
            }
            0xF1 => (Instruction::POP_R16 { reg: R16Register::HL }, 1),
            0xF2 => (Instruction::LDH_A_IND_C, 1),
            0xF3 => (Instruction::DI, 1),
            0xF5 => (Instruction::PUSH_R16 { reg: R16Register::HL }, 1),
            0xF6 => {
                let value = bus.read(pc + 1);
                (Instruction::OR_A_IMM8 { value }, 2)
            }
            0xF7 => (Instruction::RST { target: 0x30 }, 1),
            0xF8 => {
                let value = bus.read(pc + 1) as i8;
                (Instruction::LD_HL_SP_IMM8 { value }, 2)
            }
            0xF9 => (Instruction::LD_SP_HL, 1),
            0xFA => {
                let low = bus.read(pc + 1) as u8;
                let high = bus.read(pc + 2) as u8;
                let address = (high as u16) << 8 | low as u16;
                (Instruction::LD_A_IND_IMM16 { address }, 3)
            }
            0xFB => (Instruction::EI, 1),
            0xFE => {
                let value = bus.read(pc + 1);
                (Instruction::CP_A_IMM8 { value }, 2)
            }
            0xFF => (Instruction::RST { target: 0x38 }, 1),

            _ => (Instruction::NOP, 1),
        }
    }

    /// Execute an instruction and return cycles taken
    fn execute_instruction(&mut self, instruction: Instruction, bus: &mut MemoryBus) -> u8 {
        match instruction {
            Instruction::NOP => 1,

            Instruction::LD_R16_IMM16 { dest, value } => {
                match dest {
                    R16Register::BC => self.state.registers.bc = value,
                    R16Register::DE => self.state.registers.de = value,
                    R16Register::HL => self.state.registers.hl = value,
                    R16Register::SP => self.state.registers.sp = value,
                }
                3
            }

            Instruction::LD_IND_R16_A { src } => {
                let address = match src {
                    R16Mem::BC => self.state.registers.bc,
                    R16Mem::DE => self.state.registers.de,
                    _ => 0,
                };
                bus.write(address, self.state.registers.a());
                2
            }

            Instruction::LD_A_IND_R16 { dest } => {
                let address = match dest {
                    R16Mem::BC => self.state.registers.bc,
                    R16Mem::DE => self.state.registers.de,
                    _ => 0,
                };
                self.state.registers.set_a(bus.read(address));
                2
            }

            Instruction::LD_IND_IMM16_SP { address } => {
                bus.write(address, (self.state.registers.sp & 0x00FF) as u8);
                bus.write(address + 1, (self.state.registers.sp >> 8) as u8);
                5
            }

            Instruction::INC_R16 { reg } => {
                match reg {
                    R16Register::BC => self.state.registers.bc = self.state.registers.bc.wrapping_add(1),
                    R16Register::DE => self.state.registers.de = self.state.registers.de.wrapping_add(1),
                    R16Register::HL => self.state.registers.hl = self.state.registers.hl.wrapping_add(1),
                    R16Register::SP => self.state.registers.sp = self.state.registers.sp.wrapping_add(1),
                }
                2
            }

            Instruction::DEC_R16 { reg } => {
                match reg {
                    R16Register::BC => self.state.registers.bc = self.state.registers.bc.wrapping_sub(1),
                    R16Register::DE => self.state.registers.de = self.state.registers.de.wrapping_sub(1),
                    R16Register::HL => self.state.registers.hl = self.state.registers.hl.wrapping_sub(1),
                    R16Register::SP => self.state.registers.sp = self.state.registers.sp.wrapping_sub(1),
                }
                2
            }

            Instruction::ADD_HL_R16 { reg } => {
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

            Instruction::INC_R8 { reg } => {
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

            Instruction::DEC_R8 { reg } => {
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

            Instruction::LD_R8_IMM8 { dest, value } => {
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

            Instruction::JR_IMM8 { offset } => {
                self.state.registers.pc = self.state.registers.pc.wrapping_add(offset as i16 as u16);
                3
            }

            Instruction::JR_COND_IMM8 { cond, offset } => {
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

            Instruction::LD_R8_R8 { dest, src } => {
                self.set_r8(dest, self.get_r8(src, bus), bus);
                1
            }

            Instruction::ADD_A_R8 { reg } => {
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

            Instruction::ADC_A_R8 { reg } => {
                let val = self.get_r8(reg, bus);
                let a = self.state.registers.a();
                let carry = if self.state.registers.f().is_carry() { 1 } else { 0 };
                let result = a.wrapping_add(val).wrapping_add(carry);
                self.state.registers.set_a(result);

                let mut flags = self.state.registers.f();
                flags.set_zero(result == 0);
                flags.set_subtraction(false);
                let carry_flag = (a & 0x0F) + (val & 0x0F) + carry > 0x0F;
                flags.set_half_carry(carry_flag);
                flags.set_carry(a as u16 + val as u16 + carry as u16 > 0xFF);
                self.state.registers.set_f(flags);
                1
            }

            Instruction::SUB_A_R8 { reg } => {
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

            Instruction::SBC_A_R8 { reg } => {
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

            Instruction::AND_A_R8 { reg } => {
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

            Instruction::XOR_A_R8 { reg } => {
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

            Instruction::OR_A_R8 { reg } => {
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

            Instruction::CP_A_R8 { reg } => {
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

            Instruction::ADD_A_IMM8 { value } => {
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

            Instruction::ADC_A_IMM8 { value } => {
                let a = self.state.registers.a();
                let carry = if self.state.registers.f().is_carry() { 1 } else { 0 };
                let result = a.wrapping_add(value).wrapping_add(carry);
                self.state.registers.set_a(result);

                let mut flags = self.state.registers.f();
                flags.set_zero(result == 0);
                flags.set_subtraction(false);
                let carry_flag = (a & 0x0F) + (value & 0x0F) + carry > 0x0F;
                flags.set_half_carry(carry_flag);
                flags.set_carry(a as u16 + value as u16 + carry as u16 > 0xFF);
                self.state.registers.set_f(flags);
                2
            }

            Instruction::SUB_A_IMM8 { value } => {
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

            Instruction::SBC_A_IMM8 { value } => {
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

            Instruction::AND_A_IMM8 { value } => {
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

            Instruction::XOR_A_IMM8 { value } => {
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

            Instruction::OR_A_IMM8 { value } => {
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

            Instruction::CP_A_IMM8 { value } => {
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

            Instruction::RET_COND { cond } => {
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

            Instruction::JP_COND_IMM16 { cond, address } => {
                if self.condition_met(cond) {
                    self.state.registers.pc = address;
                }
                3
            }

            Instruction::JP_IMM16 { address } => {
                self.state.registers.pc = address;
                4
            }

            Instruction::JP_HL => {
                self.state.registers.pc = self.state.registers.hl;
                1
            }

            Instruction::CALL_COND_IMM16 { cond, address } => {
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

            Instruction::CALL_IMM16 { address } => {
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

            Instruction::POP_R16 { reg } => {
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

            Instruction::PUSH_R16 { reg } => {
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

            Instruction::LDH_IND_C_A => {
                let address = 0xFF00 | (self.state.registers.c() as u16);
                bus.write(address, self.state.registers.a());
                2
            }

            Instruction::LDH_IND_IMM8_A { address } => {
                let address = 0xFF00 | (address as u16);
                bus.write(address, self.state.registers.a());
                3
            }

            Instruction::LD_IND_IMM16_A { address } => {
                bus.write(address, self.state.registers.a());
                4
            }

            Instruction::LDH_A_IND_C => {
                let address = 0xFF00 | (self.state.registers.c() as u16);
                self.state.registers.set_a(bus.read(address));
                2
            }

            Instruction::LDH_A_IND_IMM8 { address } => {
                let address = 0xFF00 | (address as u16);
                self.state.registers.set_a(bus.read(address));
                3
            }

            Instruction::LD_A_IND_IMM16 { address } => {
                self.state.registers.set_a(bus.read(address));
                4
            }

            Instruction::ADD_SP_IMM8 { value } => {
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

            Instruction::LD_HL_SP_IMM8 { value } => {
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

            Instruction::LD_SP_HL => {
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

            // CB instructions handled separately
            _ => 1,
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

    #[test]
    fn test_cpu_reset() {
        let mut cpu = CPU::new();
        cpu.reset();
        assert_eq!(cpu.state.registers.pc, 0x0000);
        assert_eq!(cpu.state.registers.sp, 0xFFFE);
    }
}
