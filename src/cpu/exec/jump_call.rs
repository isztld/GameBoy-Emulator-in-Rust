/// Jump and call instruction executors
use crate::memory::MemoryBus;
use crate::cpu::CPUState;
use crate::cpu::instructions::Condition;

/// Execute JR r8
pub fn exec_jr_imm8(cpu_state: &mut CPUState, offset: i8) -> u32 {
    cpu_state.registers.pc = cpu_state.registers.pc.wrapping_add(offset as i32 as u16);
    3
}

/// Execute JR cc, r8
pub fn exec_jr_cond_imm8(cpu_state: &mut CPUState, cond: Condition, offset: i8) -> u32 {
    let jump = cond_condition(cpu_state, cond);
    if jump {
        cpu_state.registers.pc = cpu_state.registers.pc.wrapping_add(offset as i32 as u16);
        3
    } else {
        2
    }
}

/// Execute JP cc, a16
pub fn exec_jp_cond_imm16(cpu_state: &mut CPUState, cond: Condition, address: u16) -> u32 {
    if cond_condition(cpu_state, cond) {
        cpu_state.registers.pc = address;
        4
    } else {
        3
    }
}

/// Execute JP a16
pub fn exec_jp_imm16(cpu_state: &mut CPUState, address: u16) -> u32 {
    cpu_state.registers.pc = address;
    4
}

/// Execute JP (HL)
pub fn exec_jp_hl(cpu_state: &mut CPUState) -> u32 {
    cpu_state.registers.pc = cpu_state.registers.hl;
    1
}

/// Execute CALL cc, a16
pub fn exec_call_cond_imm16(cpu_state: &mut CPUState, cond: Condition, address: u16, bus: &mut MemoryBus) -> u32 {
    if cond_condition(cpu_state, cond) {
        let sp = cpu_state.registers.sp;
        // CALL is 3 bytes, so return address is PC + 3 (next instruction after CALL)
        let return_pc = cpu_state.registers.pc + 3;
        bus.write(sp.wrapping_sub(1), (return_pc >> 8) as u8);
        bus.write(sp.wrapping_sub(2), (return_pc & 0x00FF) as u8);
        cpu_state.registers.sp = sp.wrapping_sub(2);
        cpu_state.registers.pc = address;
        6
    } else {
        3
    }
}

/// Execute CALL a16
pub fn exec_call_imm16(cpu_state: &mut CPUState, address: u16, bus: &mut MemoryBus) -> u32 {
    let sp = cpu_state.registers.sp;
    // CALL is 3 bytes, so return address is PC + 3 (next instruction after CALL)
    let return_pc = cpu_state.registers.pc + 3;
    bus.write(sp.wrapping_sub(1), (return_pc >> 8) as u8);
    bus.write(sp.wrapping_sub(2), (return_pc & 0x00FF) as u8);
    cpu_state.registers.sp = sp.wrapping_sub(2);
    cpu_state.registers.pc = address;
    6
}

/// Execute RET cc
pub fn exec_ret_cond(cpu_state: &mut CPUState, cond: Condition, bus: &mut MemoryBus) -> u32 {
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

/// Condition check helper
fn cond_condition(cpu_state: &CPUState, cond: Condition) -> bool {
    match cond {
        Condition::NZ => !cpu_state.registers.f().is_zero(),
        Condition::Z => cpu_state.registers.f().is_zero(),
        Condition::NC => !cpu_state.registers.f().is_carry(),
        Condition::C => cpu_state.registers.f().is_carry(),
    }
}
