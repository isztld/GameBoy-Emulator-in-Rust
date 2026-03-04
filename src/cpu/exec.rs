/// CPU execution module
///
/// This module contains the instruction execution implementation.

mod data_transfer;
mod registers;
mod alu;
mod rotate_shift;
mod jump_call;
mod stack;
mod cb_instructions;
mod register_utils;

use crate::memory::MemoryBus;
use crate::cpu::CPUState;
use crate::cpu::instructions::Instruction;

/// Execute a single instruction and return cycles taken
pub fn execute_instruction(cpu_state: &mut CPUState, bus: &mut MemoryBus, instruction: Instruction, tick: &mut dyn FnMut(&mut [u8; 128])) -> u32 {
    match instruction {
        // Data transfer instructions
        Instruction::NOP => 1,
        Instruction::LdR16Imm16 { dest, value } => data_transfer::exec_ld_r16_imm16(cpu_state, dest, value, &mut bus.io, tick),
        Instruction::LdIndR16A { src } => data_transfer::exec_ld_ind_r16_a(cpu_state, src, bus, tick),
        Instruction::LdAIndR16 { dest } => data_transfer::exec_ld_a_ind_r16(cpu_state, dest, bus, tick),
        Instruction::LdIndImm16Sp { address } => data_transfer::exec_ld_ind_imm16_sp(cpu_state, address, bus, tick),
        Instruction::LdIndImm16A { address } => data_transfer::exec_ld_ind_imm16_a(cpu_state, address, bus, tick),
        Instruction::LdAIndImm16 { address } => data_transfer::exec_ld_a_ind_imm16(cpu_state, address, bus, tick),
        Instruction::LdhIndImm8A { address } => stack::exec_ldh_ind_imm8_a(cpu_state, address, bus, tick),
        Instruction::LdhAIndImm8 { address } => stack::exec_ldh_a_ind_imm8(cpu_state, address, bus, tick),
        Instruction::LdhIndCA => stack::exec_ldh_ind_c_a(cpu_state, bus, tick),
        Instruction::LdhAC => stack::exec_ldh_a_c(cpu_state, bus, tick),
        Instruction::LdR8Imm8 { dest, value } => data_transfer::exec_ld_r8_imm8(cpu_state, bus, dest, value, tick),
        Instruction::LdR8R8 { dest, src } => data_transfer::exec_ld_r8_r8(cpu_state, bus, dest, src, tick),

        // Register operations
        Instruction::IncR16 { reg } => registers::exec_inc_r16(cpu_state, reg, &mut bus.io, tick),
        Instruction::DecR16 { reg } => registers::exec_dec_r16(cpu_state, reg, &mut bus.io, tick),
        Instruction::IncR8 { reg } => registers::exec_inc_r8(cpu_state, bus, reg, tick),
        Instruction::DecR8 { reg } => registers::exec_dec_r8(cpu_state, bus, reg, tick),
        Instruction::AddHlR16 { reg } => registers::exec_add_hl_r16(cpu_state, reg, &mut bus.io, tick),

        // ALU instructions
        Instruction::AddAR8 { reg } => alu::exec_add_a_r8(cpu_state, bus, reg, tick),
        Instruction::AdcAR8 { reg } => alu::exec_adc_a_r8(cpu_state, bus, reg, tick),
        Instruction::SubAR8 { reg } => alu::exec_sub_a_r8(cpu_state, bus, reg, tick),
        Instruction::SbcAR8 { reg } => alu::exec_sbc_a_r8(cpu_state, bus, reg, tick),
        Instruction::AndAR8 { reg } => alu::exec_and_a_r8(cpu_state, bus, reg, tick),
        Instruction::XorAR8 { reg } => alu::exec_xor_a_r8(cpu_state, bus, reg, tick),
        Instruction::OrAR8 { reg } => alu::exec_or_a_r8(cpu_state, bus, reg, tick),
        Instruction::CpAR8 { reg } => alu::exec_cp_a_r8(cpu_state, bus, reg, tick),
        Instruction::AddAImm8 { value } => alu::exec_add_a_imm8(cpu_state, value, &mut bus.io, tick),
        Instruction::AdcAImm8 { value } => alu::exec_adc_a_imm8(cpu_state, value, &mut bus.io, tick),
        Instruction::SubAImm8 { value } => alu::exec_sub_a_imm8(cpu_state, value, &mut bus.io, tick),
        Instruction::SbcAImm8 { value } => alu::exec_sbc_a_imm8(cpu_state, value, &mut bus.io, tick),
        Instruction::AndAImm8 { value } => alu::exec_and_a_imm8(cpu_state, value, &mut bus.io, tick),
        Instruction::XorAImm8 { value } => alu::exec_xor_a_imm8(cpu_state, value, &mut bus.io, tick),
        Instruction::OrAImm8 { value } => alu::exec_or_a_imm8(cpu_state, value, &mut bus.io, tick),
        Instruction::CpAImm8 { value } => alu::exec_cp_a_imm8(cpu_state, value, &mut bus.io, tick),

        // Rotate/shift instructions
        Instruction::RLCA => rotate_shift::exec_rlca(cpu_state),
        Instruction::RRCA => rotate_shift::exec_rrca(cpu_state),
        Instruction::RLA => rotate_shift::exec_rla(cpu_state),
        Instruction::RRA => rotate_shift::exec_rra(cpu_state),
        Instruction::DAA => rotate_shift::exec_daa(cpu_state),
        Instruction::CPL => rotate_shift::exec_cpl(cpu_state),
        Instruction::SCF => rotate_shift::exec_scf(cpu_state),
        Instruction::CCF => rotate_shift::exec_ccf(cpu_state),

        // Jump and call instructions
        Instruction::JrImm8 { offset } => jump_call::exec_jr_imm8(cpu_state, offset, &mut bus.io, tick),
        Instruction::JrCondImm8 { cond, offset } => jump_call::exec_jr_cond_imm8(cpu_state, cond, offset, &mut bus.io, tick),
        Instruction::JpCondImm16 { cond, address } => jump_call::exec_jp_cond_imm16(cpu_state, cond, address, &mut bus.io, tick),
        Instruction::JpImm16 { address } => jump_call::exec_jp_imm16(cpu_state, address, &mut bus.io, tick),
        Instruction::JpHl => jump_call::exec_jp_hl(cpu_state),
        Instruction::CallCondImm16 { cond, address } => jump_call::exec_call_cond_imm16(cpu_state, cond, address, bus, tick),
        Instruction::CallImm16 { address } => jump_call::exec_call_imm16(cpu_state, address, bus, tick),
        Instruction::RetCond { cond } => jump_call::exec_ret_cond(cpu_state, cond, bus, tick),

        // Stack instructions
        Instruction::RET => stack::exec_ret(cpu_state, bus, tick),
        Instruction::RETI => stack::exec_reti(cpu_state, bus, tick),
        Instruction::PopR16 { reg } => stack::exec_pop_r16(cpu_state, reg, bus, tick),
        Instruction::PushR16 { reg } => stack::exec_push_r16(cpu_state, reg, bus, tick),
        Instruction::RST { target } => stack::exec_rst(cpu_state, target, bus, tick),

        // I/O instructions
        Instruction::AddSpImm8 { value } => stack::exec_add_sp_imm8(cpu_state, value, &mut bus.io, tick),
        Instruction::LdHlSpImm8 { value } => stack::exec_ld_hl_sp_imm8(cpu_state, value, &mut bus.io, tick),
        Instruction::LdSpHl => stack::exec_ld_sp_hl(cpu_state, &mut bus.io, tick),

        // Control instructions
        Instruction::STOP => 1,
        Instruction::HALT => 1,
        Instruction::DI => stack::exec_di(cpu_state),
        Instruction::EI => stack::exec_ei(cpu_state),

        // CB-prefixed instructions
        Instruction::CB { cb_instr } => cb_instructions::execute_cb(cpu_state, bus, cb_instr, tick),
    }
}
