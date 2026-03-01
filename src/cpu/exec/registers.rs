/// Register/pair operations executors
use crate::memory::MemoryBus;
use crate::cpu::CPUState;
use crate::cpu::instructions::{R16Register, R8Register};
use crate::cpu::exec::register_utils::{r16, set_r16, get_r8, set_r8};

/// Execute INC r16
pub fn exec_inc_r16(cpu_state: &mut CPUState, reg: R16Register) -> u32 {
    let val = r16(&cpu_state.registers, reg);
    set_r16(&mut cpu_state.registers, reg, val + 1);
    2
}

/// Execute DEC r16
pub fn exec_dec_r16(cpu_state: &mut CPUState, reg: R16Register) -> u32 {
    let val = r16(&cpu_state.registers, reg);
    set_r16(&mut cpu_state.registers, reg, val - 1);
    2
}

/// Execute INC r8
pub fn exec_inc_r8(cpu_state: &mut CPUState, bus: &mut MemoryBus, reg: R8Register) -> u32 {
    let new_val = get_r8(&cpu_state.registers, bus, reg).wrapping_add(1);
    set_r8(&mut cpu_state.registers, bus, reg, new_val);
    cpu_state.registers.f_mut().set_zero(new_val == 0);
    cpu_state.registers.f_mut().set_subtraction(false);
    1
}

/// Execute DEC r8
pub fn exec_dec_r8(cpu_state: &mut CPUState, bus: &mut MemoryBus, reg: R8Register) -> u32 {
    let new_val = get_r8(&cpu_state.registers, bus, reg).wrapping_sub(1);
    set_r8(&mut cpu_state.registers, bus, reg, new_val);
    cpu_state.registers.f_mut().set_zero(new_val == 0);
    cpu_state.registers.f_mut().set_subtraction(true);
    1
}

/// Execute ADD HL, r16
pub fn exec_add_hl_r16(cpu_state: &mut CPUState, reg: R16Register) -> u32 {
    let hl = r16(&cpu_state.registers, R16Register::HL) as u32;
    let add = r16(&cpu_state.registers, reg) as u32;
    let result = hl.wrapping_add(add);
    set_r16(&mut cpu_state.registers, R16Register::HL, result as u16);
    cpu_state.registers.f_mut().set_half_carry((hl & 0x0FFF) + (add & 0x0FFF) > 0x0FFF);
    cpu_state.registers.f_mut().set_subtraction(false);
    cpu_state.registers.f_mut().set_carry(result > 0xFFFF);
    2
}
