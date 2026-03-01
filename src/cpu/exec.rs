/// CPU execution module
///
/// This module contains the instruction execution implementation.

use crate::memory::MemoryBus;
use crate::cpu::CPUState;
use crate::cpu::instructions::{Instruction, R8Register, R16Register};

/// Execute a single instruction and return cycles taken
pub fn execute_instruction(cpu_state: &mut CPUState, bus: &mut MemoryBus, instruction: Instruction) -> u32 {
    match instruction {
        // Block 0 instructions
        Instruction::NOP => 1,
        Instruction::LdR16Imm16 { dest, value } => {
            set_r16(&mut cpu_state.registers, dest, value);
            3
        }
        Instruction::LdIndR16A { src } => {
            let addr = r16mem_address(&mut cpu_state.registers, src);
            bus.write(addr, cpu_state.registers.a());
            2
        }
        Instruction::LdAIndR16 { dest } => {
            let addr = r16mem_address(&mut cpu_state.registers, dest);
            cpu_state.registers.set_a(bus.read(addr));
            2
        }
        Instruction::LdIndImm16Sp { address } => {
            bus.write(address, (cpu_state.registers.sp >> 8) as u8);
            bus.write(address.wrapping_add(1), (cpu_state.registers.sp & 0xFF) as u8);
            5
        }
        Instruction::IncR16 { reg } => {
            let val = r16(&cpu_state.registers, reg);
            set_r16(&mut cpu_state.registers, reg, val + 1);
            2
        }
        Instruction::DecR16 { reg } => {
            let val = r16(&cpu_state.registers, reg);
            set_r16(&mut cpu_state.registers, reg, val - 1);
            2
        }
        Instruction::AddHlR16 { reg } => {
            let hl = r16(&cpu_state.registers, R16Register::HL) as u32;
            let add = r16(&cpu_state.registers, reg) as u32;
            let result = hl.wrapping_add(add);
            set_r16(&mut cpu_state.registers, R16Register::HL, result as u16);
            cpu_state.registers.f_mut().set_half_carry((hl & 0x0FFF) + (add & 0x0FFF) > 0x0FFF);
            cpu_state.registers.f_mut().set_subtraction(false);
            cpu_state.registers.f_mut().set_carry(result > 0xFFFF);
            2
        }
        Instruction::IncR8 { reg } => {
            let new_val = get_r8(&cpu_state.registers, bus, reg).wrapping_add(1);
            set_r8(&mut cpu_state.registers, bus, reg, new_val);
            cpu_state.registers.f_mut().set_zero(new_val == 0);
            cpu_state.registers.f_mut().set_subtraction(false);
            1
        }
        Instruction::DecR8 { reg } => {
            let new_val = get_r8(&cpu_state.registers, bus, reg).wrapping_sub(1);
            set_r8(&mut cpu_state.registers, bus, reg, new_val);
            cpu_state.registers.f_mut().set_zero(new_val == 0);
            cpu_state.registers.f_mut().set_subtraction(true);
            1
        }
        Instruction::LdR8Imm8 { dest, value } => {
            set_r8(&mut cpu_state.registers, bus, dest, value);
            2
        }
        Instruction::RLCA => {
            let a = cpu_state.registers.a();
            let new_a = a.rotate_left(1);
            cpu_state.registers.set_a(new_a);
            cpu_state.registers.f_mut().set_carry((a & 0x80) != 0);
            cpu_state.registers.f_mut().set_zero(false);
            cpu_state.registers.f_mut().set_subtraction(false);
            cpu_state.registers.f_mut().set_half_carry(false);
            1
        }
        Instruction::RRCA => {
            let a = cpu_state.registers.a();
            let new_a = a.rotate_right(1);
            cpu_state.registers.set_a(new_a);
            cpu_state.registers.f_mut().set_carry((a & 0x01) != 0);
            cpu_state.registers.f_mut().set_zero(false);
            cpu_state.registers.f_mut().set_subtraction(false);
            cpu_state.registers.f_mut().set_half_carry(false);
            1
        }
        Instruction::RLA => {
            let a = cpu_state.registers.a();
            let old_c = cpu_state.registers.f().is_carry() as u8;
            let new_a = (a << 1) | old_c;
            cpu_state.registers.set_a(new_a);
            cpu_state.registers.f_mut().set_carry((a & 0x80) != 0);
            cpu_state.registers.f_mut().set_zero(false);
            cpu_state.registers.f_mut().set_subtraction(false);
            cpu_state.registers.f_mut().set_half_carry(false);
            1
        }
        Instruction::RRA => {
            let a = cpu_state.registers.a();
            let old_c = cpu_state.registers.f().is_carry() as u8;
            let new_a = (a >> 1) | (old_c << 7);
            cpu_state.registers.set_a(new_a);
            cpu_state.registers.f_mut().set_carry((a & 0x01) != 0);
            cpu_state.registers.f_mut().set_zero(false);
            cpu_state.registers.f_mut().set_subtraction(false);
            cpu_state.registers.f_mut().set_half_carry(false);
            1
        }
        Instruction::DAA => {
            let mut a = cpu_state.registers.a();
            let mut adjust = 0;
            if cpu_state.registers.f().is_carry() { adjust |= 0x60; }
            if cpu_state.registers.f().is_half_carry() { adjust |= 0x06; }
            if !cpu_state.registers.f().is_subtraction() {
                if (a & 0x0F) > 9 { adjust |= 0x06; }
                if (a & 0xF0) > 0x90 { adjust |= 0x60; }
            }
            a = a.wrapping_add(adjust);
            cpu_state.registers.set_a(a);
            if (adjust & 0x60) != 0 { cpu_state.registers.f_mut().set_carry(true); }
            1
        }
        Instruction::CPL => {
            let a = cpu_state.registers.a();
            cpu_state.registers.set_a(!a);
            cpu_state.registers.f_mut().set_subtraction(true);
            cpu_state.registers.f_mut().set_half_carry(true);
            1
        }
        Instruction::SCF => {
            cpu_state.registers.f_mut().set_carry(true);
            cpu_state.registers.f_mut().set_subtraction(false);
            cpu_state.registers.f_mut().set_half_carry(false);
            1
        }
        Instruction::CCF => {
            let carry = cpu_state.registers.f().is_carry();
            cpu_state.registers.f_mut().set_carry(!carry);
            cpu_state.registers.f_mut().set_subtraction(false);
            cpu_state.registers.f_mut().set_half_carry(false);
            1
        }
        Instruction::JrImm8 { offset } => {
            cpu_state.registers.pc = cpu_state.registers.pc.wrapping_add(offset as i32 as u16);
            3
        }
        Instruction::JrCondImm8 { cond, offset } => {
            let jump = cond_condition(cpu_state, cond);
            if jump {
                cpu_state.registers.pc = cpu_state.registers.pc.wrapping_add(offset as i32 as u16);
                3
            } else {
                2
            }
        }
        Instruction::STOP => 1,
        Instruction::HALT => 1,
        Instruction::LdR8R8 { dest, src } => {
            let val = get_r8(&cpu_state.registers, bus, src);
            set_r8(&mut cpu_state.registers, bus, dest, val);
            1
        }
        Instruction::AddAR8 { reg } => {
            let val = get_r8(&cpu_state.registers, bus, reg);
            let a = cpu_state.registers.a();
            let result = a.wrapping_add(val);
            cpu_state.registers.set_a(result);
            cpu_state.registers.f_mut().set_zero(result == 0);
            cpu_state.registers.f_mut().set_subtraction(false);
            cpu_state.registers.f_mut().set_half_carry((a & 0x0F) + (val & 0x0F) > 0x0F);
            cpu_state.registers.f_mut().set_carry(result < a);
            1
        }
        Instruction::AdcAR8 { reg } => {
            let val = get_r8(&cpu_state.registers, bus, reg);
            let a = cpu_state.registers.a();
            let old_c = cpu_state.registers.f().is_carry() as u8;
            let result = a.wrapping_add(val).wrapping_add(old_c);
            cpu_state.registers.set_a(result);
            cpu_state.registers.f_mut().set_zero(result == 0);
            cpu_state.registers.f_mut().set_subtraction(false);
            cpu_state.registers.f_mut().set_half_carry((a & 0x0F) + (val & 0x0F) + old_c as u8 > 0x0F);
            cpu_state.registers.f_mut().set_carry(result < a);
            1
        }
        Instruction::SubAR8 { reg } => {
            let val = get_r8(&cpu_state.registers, bus, reg);
            let a = cpu_state.registers.a();
            let result = a.wrapping_sub(val);
            cpu_state.registers.set_a(result);
            cpu_state.registers.f_mut().set_zero(result == 0);
            cpu_state.registers.f_mut().set_subtraction(true);
            cpu_state.registers.f_mut().set_carry(a < val);
            cpu_state.registers.f_mut().set_half_carry((a as i8) < (val as i8));
            1
        }
        Instruction::SbcAR8 { reg } => {
            let val = get_r8(&cpu_state.registers, bus, reg);
            let a = cpu_state.registers.a();
            let old_c = cpu_state.registers.f().is_carry() as u8;
            let result = a.wrapping_sub(val).wrapping_sub(old_c);
            cpu_state.registers.set_a(result);
            cpu_state.registers.f_mut().set_zero(result == 0);
            cpu_state.registers.f_mut().set_subtraction(true);
            cpu_state.registers.f_mut().set_carry(a < val.wrapping_add(old_c));
            cpu_state.registers.f_mut().set_half_carry((a as i8) < (val.wrapping_add(old_c) as i8));
            1
        }
        Instruction::AndAR8 { reg } => {
            let val = get_r8(&cpu_state.registers, bus, reg);
            let a = cpu_state.registers.a();
            let result = a & val;
            cpu_state.registers.set_a(result);
            cpu_state.registers.f_mut().set_zero(result == 0);
            cpu_state.registers.f_mut().set_subtraction(false);
            cpu_state.registers.f_mut().set_half_carry(true);
            cpu_state.registers.f_mut().set_carry(false);
            1
        }
        Instruction::XorAR8 { reg } => {
            let val = get_r8(&cpu_state.registers, bus, reg);
            let a = cpu_state.registers.a();
            let result = a ^ val;
            cpu_state.registers.set_a(result);
            cpu_state.registers.f_mut().set_zero(result == 0);
            cpu_state.registers.f_mut().set_subtraction(false);
            cpu_state.registers.f_mut().set_half_carry(false);
            cpu_state.registers.f_mut().set_carry(false);
            1
        }
        Instruction::OrAR8 { reg } => {
            let val = get_r8(&cpu_state.registers, bus, reg);
            let a = cpu_state.registers.a();
            let result = a | val;
            cpu_state.registers.set_a(result);
            cpu_state.registers.f_mut().set_zero(result == 0);
            cpu_state.registers.f_mut().set_subtraction(false);
            cpu_state.registers.f_mut().set_half_carry(false);
            cpu_state.registers.f_mut().set_carry(false);
            1
        }
        Instruction::CpAR8 { reg } => {
            let val = get_r8(&cpu_state.registers, bus, reg);
            let a = cpu_state.registers.a();
            let result = a.wrapping_sub(val);
            cpu_state.registers.f_mut().set_zero(result == 0);
            cpu_state.registers.f_mut().set_subtraction(true);
            cpu_state.registers.f_mut().set_carry(a < val);
            cpu_state.registers.f_mut().set_half_carry((a as i8) < (val as i8));
            1
        }
        Instruction::AddAImm8 { value } => {
            let a = cpu_state.registers.a();
            let result = a.wrapping_add(value);
            cpu_state.registers.set_a(result);
            cpu_state.registers.f_mut().set_zero(result == 0);
            cpu_state.registers.f_mut().set_subtraction(false);
            cpu_state.registers.f_mut().set_half_carry((a & 0x0F) + (value & 0x0F) > 0x0F);
            cpu_state.registers.f_mut().set_carry(result < a);
            2
        }
        Instruction::AdcAImm8 { value } => {
            let a = cpu_state.registers.a();
            let old_c = cpu_state.registers.f().is_carry() as u8;
            let result = a.wrapping_add(value).wrapping_add(old_c);
            cpu_state.registers.set_a(result);
            cpu_state.registers.f_mut().set_zero(result == 0);
            cpu_state.registers.f_mut().set_subtraction(false);
            cpu_state.registers.f_mut().set_half_carry((a & 0x0F) + (value & 0x0F) + old_c as u8 > 0x0F);
            cpu_state.registers.f_mut().set_carry(result < a);
            2
        }
        Instruction::SubAImm8 { value } => {
            let a = cpu_state.registers.a();
            let result = a.wrapping_sub(value);
            cpu_state.registers.set_a(result);
            cpu_state.registers.f_mut().set_zero(result == 0);
            cpu_state.registers.f_mut().set_subtraction(true);
            cpu_state.registers.f_mut().set_carry(a < value);
            cpu_state.registers.f_mut().set_half_carry((a as i8) < (value as i8));
            2
        }
        Instruction::SbcAImm8 { value } => {
            let a = cpu_state.registers.a();
            let old_c = cpu_state.registers.f().is_carry() as u8;
            let result = a.wrapping_sub(value).wrapping_sub(old_c);
            cpu_state.registers.set_a(result);
            cpu_state.registers.f_mut().set_zero(result == 0);
            cpu_state.registers.f_mut().set_subtraction(true);
            cpu_state.registers.f_mut().set_carry(a < value.wrapping_add(old_c));
            cpu_state.registers.f_mut().set_half_carry((a as i8) < (value.wrapping_add(old_c) as i8));
            2
        }
        Instruction::AndAImm8 { value } => {
            let a = cpu_state.registers.a();
            let result = a & value;
            cpu_state.registers.set_a(result);
            cpu_state.registers.f_mut().set_zero(result == 0);
            cpu_state.registers.f_mut().set_subtraction(false);
            cpu_state.registers.f_mut().set_half_carry(true);
            cpu_state.registers.f_mut().set_carry(false);
            2
        }
        Instruction::XorAImm8 { value } => {
            let a = cpu_state.registers.a();
            let result = a ^ value;
            cpu_state.registers.set_a(result);
            cpu_state.registers.f_mut().set_zero(result == 0);
            cpu_state.registers.f_mut().set_subtraction(false);
            cpu_state.registers.f_mut().set_half_carry(false);
            cpu_state.registers.f_mut().set_carry(false);
            2
        }
        Instruction::OrAImm8 { value } => {
            let a = cpu_state.registers.a();
            let result = a | value;
            cpu_state.registers.set_a(result);
            cpu_state.registers.f_mut().set_zero(result == 0);
            cpu_state.registers.f_mut().set_subtraction(false);
            cpu_state.registers.f_mut().set_half_carry(false);
            cpu_state.registers.f_mut().set_carry(false);
            2
        }
        Instruction::CpAImm8 { value } => {
            let a = cpu_state.registers.a();
            let result = a.wrapping_sub(value);
            cpu_state.registers.f_mut().set_zero(result == 0);
            cpu_state.registers.f_mut().set_subtraction(true);
            cpu_state.registers.f_mut().set_carry(a < value);
            cpu_state.registers.f_mut().set_half_carry((a as i8) < (value as i8));
            2
        }
        Instruction::RetCond { cond } => {
            if cond_condition(cpu_state, cond) {
                let sp = cpu_state.registers.sp;
                let low = bus.read(sp);
                let high = bus.read(sp.wrapping_add(1));
                cpu_state.registers.sp = sp.wrapping_add(2);
                cpu_state.registers.pc = ((high as u16) << 8) | (low as u16);
                5
            } else {
                2
            }
        }
        Instruction::RET => {
            let sp = cpu_state.registers.sp;
            let low = bus.read(sp);
            let high = bus.read(sp.wrapping_add(1));
            cpu_state.registers.sp = sp.wrapping_add(2);
            cpu_state.registers.pc = ((high as u16) << 8) | (low as u16);
            4
        }
        Instruction::RETI => {
            let sp = cpu_state.registers.sp;
            let low = bus.read(sp);
            let high = bus.read(sp.wrapping_add(1));
            cpu_state.registers.sp = sp.wrapping_add(2);
            cpu_state.registers.pc = ((high as u16) << 8) | (low as u16);
            cpu_state.ime = true;
            4
        }
        Instruction::JpCondImm16 { cond, address } => {
            if cond_condition(cpu_state, cond) {
                cpu_state.registers.pc = address;
                4
            } else {
                3
            }
        }
        Instruction::JpImm16 { address } => {
            cpu_state.registers.pc = address;
            4
        }
        Instruction::JpHl => {
            cpu_state.registers.pc = cpu_state.registers.hl;
            1
        }
        Instruction::CallCondImm16 { cond, address } => {
            if cond_condition(cpu_state, cond) {
                let sp = cpu_state.registers.sp;
                bus.write(sp.wrapping_sub(1), (cpu_state.registers.pc >> 8) as u8);
                bus.write(sp.wrapping_sub(2), (cpu_state.registers.pc & 0x00FF) as u8);
                cpu_state.registers.sp = sp.wrapping_sub(2);
                cpu_state.registers.pc = address;
                6
            } else {
                3
            }
        }
        Instruction::CallImm16 { address } => {
            let sp = cpu_state.registers.sp;
            bus.write(sp.wrapping_sub(1), (cpu_state.registers.pc >> 8) as u8);
            bus.write(sp.wrapping_sub(2), (cpu_state.registers.pc & 0x00FF) as u8);
            cpu_state.registers.sp = sp.wrapping_sub(2);
            cpu_state.registers.pc = address;
            6
        }
        Instruction::RST { target } => {
            let sp = cpu_state.registers.sp;
            bus.write(sp.wrapping_sub(1), (cpu_state.registers.pc >> 8) as u8);
            bus.write(sp.wrapping_sub(2), (cpu_state.registers.pc & 0x00FF) as u8);
            cpu_state.registers.sp = sp.wrapping_sub(2);
            cpu_state.registers.pc = target as u16;
            4
        }
        Instruction::PopR16 { reg } => {
            let sp = cpu_state.registers.sp;
            let low = bus.read(sp);
            let high = bus.read(sp.wrapping_add(1));
            cpu_state.registers.sp = sp.wrapping_add(2);
            let value = ((high as u16) << 8) | (low as u16);
            set_r16(&mut cpu_state.registers, reg, value);
            3
        }
        Instruction::PushR16 { reg } => {
            let sp = cpu_state.registers.sp;
            let value = r16(&cpu_state.registers, reg);
            bus.write(sp.wrapping_sub(1), (value >> 8) as u8);
            bus.write(sp.wrapping_sub(2), (value & 0x00FF) as u8);
            cpu_state.registers.sp = sp.wrapping_sub(2);
            5
        }
        Instruction::LdhIndCA => {
            bus.write(0xFF00u16.wrapping_add(cpu_state.registers.c() as u16), cpu_state.registers.a());
            2
        }
        Instruction::LdhIndImm8A { address } => {
            bus.write(0xFF00u16.wrapping_add(address as u16), cpu_state.registers.a());
            3
        }
        Instruction::LdIndImm16A { address } => {
            bus.write(address, cpu_state.registers.a());
            4
        }
        Instruction::LdhAC => {
            cpu_state.registers.set_a(bus.read(0xFF00u16.wrapping_add(cpu_state.registers.c() as u16)));
            2
        }
        Instruction::LdhAIndImm8 { address } => {
            cpu_state.registers.set_a(bus.read(0xFF00u16.wrapping_add(address as u16)));
            3
        }
        Instruction::LdAIndImm16 { address } => {
            cpu_state.registers.set_a(bus.read(address));
            4
        }
        Instruction::AddSpImm8 { value } => {
            let sp = cpu_state.registers.sp as i32;
            let result = sp.wrapping_add(value as i32) as u16;
            cpu_state.registers.sp = result;
            cpu_state.registers.f_mut().set_zero(false);
            cpu_state.registers.f_mut().set_subtraction(false);
            cpu_state.registers.f_mut().set_half_carry((sp & 0x0F) + (value as i32 & 0x0F) > 0x0F);
            cpu_state.registers.f_mut().set_carry((sp & 0xFF) + (value as i32 & 0xFF) > 0xFF);
            4
        }
        Instruction::LdHlSpImm8 { value } => {
            let sp = cpu_state.registers.sp as i32;
            let result = sp.wrapping_add(value as i32) as u16;
            cpu_state.registers.hl = result;
            cpu_state.registers.f_mut().set_zero(false);
            cpu_state.registers.f_mut().set_subtraction(false);
            cpu_state.registers.f_mut().set_half_carry((sp & 0x0F) + (value as i32 & 0x0F) > 0x0F);
            cpu_state.registers.f_mut().set_carry((sp & 0xFF) + (value as i32 & 0xFF) > 0xFF);
            3
        }
        Instruction::LdSpHl => {
            cpu_state.registers.sp = cpu_state.registers.hl;
            2
        }
        Instruction::DI => {
            cpu_state.ime = false;
            1
        }
        Instruction::EI => {
            cpu_state.ime = true;
            1
        }
        Instruction::CB { cb_instr } => {
            execute_cb(cpu_state, bus, cb_instr)
        }
    }
}

fn r16mem_address(registers: &mut crate::cpu::registers::Registers, mem: crate::cpu::instructions::R16Mem) -> u16 {
    match mem {
        crate::cpu::instructions::R16Mem::BC => registers.bc,
        crate::cpu::instructions::R16Mem::DE => registers.de,
        crate::cpu::instructions::R16Mem::HLPlus => {
            let addr = registers.hl;
            registers.hl = registers.hl.wrapping_add(1);
            addr
        }
        crate::cpu::instructions::R16Mem::HLMinus => {
            let addr = registers.hl;
            registers.hl = registers.hl.wrapping_sub(1);
            addr
        }
    }
}

fn r16(registers: &crate::cpu::registers::Registers, reg: crate::cpu::instructions::R16Register) -> u16 {
    match reg {
        crate::cpu::instructions::R16Register::BC => registers.bc,
        crate::cpu::instructions::R16Register::DE => registers.de,
        crate::cpu::instructions::R16Register::HL => registers.hl,
        crate::cpu::instructions::R16Register::SP => registers.sp,
        crate::cpu::instructions::R16Register::AF => registers.af,
    }
}

fn set_r16(registers: &mut crate::cpu::registers::Registers, reg: crate::cpu::instructions::R16Register, value: u16) {
    match reg {
        crate::cpu::instructions::R16Register::BC => registers.bc = value,
        crate::cpu::instructions::R16Register::DE => registers.de = value,
        crate::cpu::instructions::R16Register::HL => registers.hl = value,
        crate::cpu::instructions::R16Register::SP => registers.sp = value,
        crate::cpu::instructions::R16Register::AF => registers.af = value,
    }
}

fn get_r8(registers: &crate::cpu::registers::Registers, bus: &mut MemoryBus, reg: R8Register) -> u8 {
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

fn set_r8(registers: &mut crate::cpu::registers::Registers, bus: &mut MemoryBus, reg: R8Register, value: u8) {
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

fn cond_condition(cpu_state: &CPUState, cond: crate::cpu::instructions::Condition) -> bool {
    match cond {
        crate::cpu::instructions::Condition::NZ => !cpu_state.registers.f().is_zero(),
        crate::cpu::instructions::Condition::Z => cpu_state.registers.f().is_zero(),
        crate::cpu::instructions::Condition::NC => !cpu_state.registers.f().is_carry(),
        crate::cpu::instructions::Condition::C => cpu_state.registers.f().is_carry(),
    }
}

fn execute_cb(cpu_state: &mut CPUState, bus: &mut MemoryBus, cb_instr: crate::cpu::instructions::CBInstruction) -> u32 {
    match cb_instr {
        crate::cpu::instructions::CBInstruction::RLCR8 { reg } => {
            let val = get_r8(&cpu_state.registers, bus, reg);
            let new_val = val.rotate_left(1);
            set_r8(&mut cpu_state.registers, bus, reg, new_val);
            cpu_state.registers.f_mut().set_carry((val & 0x80) != 0);
            cpu_state.registers.f_mut().set_zero(new_val == 0);
            cpu_state.registers.f_mut().set_subtraction(false);
            cpu_state.registers.f_mut().set_half_carry(false);
            2
        }
        crate::cpu::instructions::CBInstruction::RRCR8 { reg } => {
            let val = get_r8(&cpu_state.registers, bus, reg);
            let new_val = val.rotate_right(1);
            set_r8(&mut cpu_state.registers, bus, reg, new_val);
            cpu_state.registers.f_mut().set_carry((val & 0x01) != 0);
            cpu_state.registers.f_mut().set_zero(new_val == 0);
            cpu_state.registers.f_mut().set_subtraction(false);
            cpu_state.registers.f_mut().set_half_carry(false);
            2
        }
        crate::cpu::instructions::CBInstruction::RLR8 { reg } => {
            let val = get_r8(&cpu_state.registers, bus, reg);
            let old_c = cpu_state.registers.f().is_carry() as u8;
            let new_val = (val << 1) | old_c;
            set_r8(&mut cpu_state.registers, bus, reg, new_val);
            cpu_state.registers.f_mut().set_carry((val & 0x80) != 0);
            cpu_state.registers.f_mut().set_zero(new_val == 0);
            cpu_state.registers.f_mut().set_subtraction(false);
            cpu_state.registers.f_mut().set_half_carry(false);
            2
        }
        crate::cpu::instructions::CBInstruction::RRR8 { reg } => {
            let val = get_r8(&cpu_state.registers, bus, reg);
            let old_c = cpu_state.registers.f().is_carry() as u8;
            let new_val = (val >> 1) | (old_c << 7);
            set_r8(&mut cpu_state.registers, bus, reg, new_val);
            cpu_state.registers.f_mut().set_carry((val & 0x01) != 0);
            cpu_state.registers.f_mut().set_zero(new_val == 0);
            cpu_state.registers.f_mut().set_subtraction(false);
            cpu_state.registers.f_mut().set_half_carry(false);
            2
        }
        crate::cpu::instructions::CBInstruction::SLAR8 { reg } => {
            let val = get_r8(&cpu_state.registers, bus, reg);
            let new_val = val << 1;
            set_r8(&mut cpu_state.registers, bus, reg, new_val);
            cpu_state.registers.f_mut().set_carry((val & 0x80) != 0);
            cpu_state.registers.f_mut().set_zero(new_val == 0);
            cpu_state.registers.f_mut().set_subtraction(false);
            cpu_state.registers.f_mut().set_half_carry(false);
            2
        }
        crate::cpu::instructions::CBInstruction::SRAR8 { reg } => {
            let val = get_r8(&cpu_state.registers, bus, reg);
            let new_val = (val as i8 >> 1) as u8;
            set_r8(&mut cpu_state.registers, bus, reg, new_val);
            cpu_state.registers.f_mut().set_carry((val & 0x01) != 0);
            cpu_state.registers.f_mut().set_zero(new_val == 0);
            cpu_state.registers.f_mut().set_subtraction(false);
            cpu_state.registers.f_mut().set_half_carry(false);
            2
        }
        crate::cpu::instructions::CBInstruction::SWAPR8 { reg } => {
            let val = get_r8(&cpu_state.registers, bus, reg);
            let new_val = (val >> 4) | (val << 4);
            set_r8(&mut cpu_state.registers, bus, reg, new_val);
            cpu_state.registers.f_mut().set_zero(new_val == 0);
            cpu_state.registers.f_mut().set_subtraction(false);
            cpu_state.registers.f_mut().set_half_carry(false);
            cpu_state.registers.f_mut().set_carry(false);
            2
        }
        crate::cpu::instructions::CBInstruction::SRLR8 { reg } => {
            let val = get_r8(&cpu_state.registers, bus, reg);
            let new_val = val >> 1;
            set_r8(&mut cpu_state.registers, bus, reg, new_val);
            cpu_state.registers.f_mut().set_carry((val & 0x01) != 0);
            cpu_state.registers.f_mut().set_zero(new_val == 0);
            cpu_state.registers.f_mut().set_subtraction(false);
            cpu_state.registers.f_mut().set_half_carry(false);
            2
        }
        crate::cpu::instructions::CBInstruction::BITBR8 { bit, reg } => {
            let val = get_r8(&cpu_state.registers, bus, reg);
            cpu_state.registers.f_mut().set_zero(((val >> bit) & 1) == 0);
            cpu_state.registers.f_mut().set_subtraction(false);
            cpu_state.registers.f_mut().set_half_carry(true);
            2
        }
        crate::cpu::instructions::CBInstruction::RESBR8 { bit, reg } => {
            let mut val = get_r8(&cpu_state.registers, bus, reg);
            val &= !(1 << bit);
            set_r8(&mut cpu_state.registers, bus, reg, val);
            2
        }
        crate::cpu::instructions::CBInstruction::SETBR8 { bit, reg } => {
            let mut val = get_r8(&cpu_state.registers, bus, reg);
            val |= 1 << bit;
            set_r8(&mut cpu_state.registers, bus, reg, val);
            2
        }
    }
}
