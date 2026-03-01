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
pub fn execute_instruction(cpu_state: &mut CPUState, bus: &mut MemoryBus, instruction: Instruction) -> u32 {
    match instruction {
        // Data transfer instructions
        Instruction::NOP => 1,
        Instruction::LdR16Imm16 { dest, value } => data_transfer::exec_ld_r16_imm16(cpu_state, dest, value),
        Instruction::LdIndR16A { src } => data_transfer::exec_ld_ind_r16_a(cpu_state, src, bus),
        Instruction::LdAIndR16 { dest } => data_transfer::exec_ld_a_ind_r16(cpu_state, dest, bus),
        Instruction::LdIndImm16Sp { address } => data_transfer::exec_ld_ind_imm16_sp(cpu_state, address, bus),
        Instruction::LdIndImm16A { address } => data_transfer::exec_ld_ind_imm16_a(cpu_state, address, bus),
        Instruction::LdAIndImm16 { address } => data_transfer::exec_ld_a_ind_imm16(cpu_state, address, bus),
        Instruction::LdhIndImm8A { address } => stack::exec_ldh_ind_imm8_a(cpu_state, address, bus),
        Instruction::LdhAIndImm8 { address } => stack::exec_ldh_a_ind_imm8(cpu_state, address, bus),
        Instruction::LdhIndCA => stack::exec_ldh_ind_c_a(cpu_state, bus),
        Instruction::LdhAC => stack::exec_ldh_a_c(cpu_state, bus),
        Instruction::LdR8Imm8 { dest, value } => data_transfer::exec_ld_r8_imm8(cpu_state, bus, dest, value),
        Instruction::LdR8R8 { dest, src } => data_transfer::exec_ld_r8_r8(cpu_state, bus, dest, src),

        // Register operations
        Instruction::IncR16 { reg } => registers::exec_inc_r16(cpu_state, reg),
        Instruction::DecR16 { reg } => registers::exec_dec_r16(cpu_state, reg),
        Instruction::IncR8 { reg } => registers::exec_inc_r8(cpu_state, bus, reg),
        Instruction::DecR8 { reg } => registers::exec_dec_r8(cpu_state, bus, reg),
        Instruction::AddHlR16 { reg } => registers::exec_add_hl_r16(cpu_state, reg),

        // ALU instructions
        Instruction::AddAR8 { reg } => alu::exec_add_a_r8(cpu_state, bus, reg),
        Instruction::AdcAR8 { reg } => alu::exec_adc_a_r8(cpu_state, bus, reg),
        Instruction::SubAR8 { reg } => alu::exec_sub_a_r8(cpu_state, bus, reg),
        Instruction::SbcAR8 { reg } => alu::exec_sbc_a_r8(cpu_state, bus, reg),
        Instruction::AndAR8 { reg } => alu::exec_and_a_r8(cpu_state, bus, reg),
        Instruction::XorAR8 { reg } => alu::exec_xor_a_r8(cpu_state, bus, reg),
        Instruction::OrAR8 { reg } => alu::exec_or_a_r8(cpu_state, bus, reg),
        Instruction::CpAR8 { reg } => alu::exec_cp_a_r8(cpu_state, bus, reg),
        Instruction::AddAImm8 { value } => alu::exec_add_a_imm8(cpu_state, value),
        Instruction::AdcAImm8 { value } => alu::exec_adc_a_imm8(cpu_state, value),
        Instruction::SubAImm8 { value } => alu::exec_sub_a_imm8(cpu_state, value),
        Instruction::SbcAImm8 { value } => alu::exec_sbc_a_imm8(cpu_state, value),
        Instruction::AndAImm8 { value } => alu::exec_and_a_imm8(cpu_state, value),
        Instruction::XorAImm8 { value } => alu::exec_xor_a_imm8(cpu_state, value),
        Instruction::OrAImm8 { value } => alu::exec_or_a_imm8(cpu_state, value),
        Instruction::CpAImm8 { value } => alu::exec_cp_a_imm8(cpu_state, value),

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
        Instruction::JrImm8 { offset } => jump_call::exec_jr_imm8(cpu_state, offset),
        Instruction::JrCondImm8 { cond, offset } => jump_call::exec_jr_cond_imm8(cpu_state, cond, offset),
        Instruction::JpCondImm16 { cond, address } => jump_call::exec_jp_cond_imm16(cpu_state, cond, address),
        Instruction::JpImm16 { address } => jump_call::exec_jp_imm16(cpu_state, address),
        Instruction::JpHl => jump_call::exec_jp_hl(cpu_state),
        Instruction::CallCondImm16 { cond, address } => jump_call::exec_call_cond_imm16(cpu_state, cond, address, bus),
        Instruction::CallImm16 { address } => jump_call::exec_call_imm16(cpu_state, address, bus),
        Instruction::RetCond { cond } => jump_call::exec_ret_cond(cpu_state, cond, bus),

        // Stack instructions
        Instruction::RET => stack::exec_ret(cpu_state, bus),
        Instruction::RETI => stack::exec_reti(cpu_state, bus),
        Instruction::PopR16 { reg } => stack::exec_pop_r16(cpu_state, reg, bus),
        Instruction::PushR16 { reg } => stack::exec_push_r16(cpu_state, reg, bus),
        Instruction::RST { target } => stack::exec_rst(cpu_state, target, bus),

        // I/O instructions
        Instruction::AddSpImm8 { value } => stack::exec_add_sp_imm8(cpu_state, value),
        Instruction::LdHlSpImm8 { value } => stack::exec_ld_hl_sp_imm8(cpu_state, value),
        Instruction::LdSpHl => stack::exec_ld_sp_hl(cpu_state),

        // Control instructions
        Instruction::STOP => 1,
        Instruction::HALT => 1,
        Instruction::DI => stack::exec_di(cpu_state),
        Instruction::EI => stack::exec_ei(cpu_state),

        // CB-prefixed instructions
        Instruction::CB { cb_instr } => cb_instructions::execute_cb(cpu_state, bus, cb_instr),
    }
}
