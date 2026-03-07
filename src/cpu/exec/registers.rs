/// Register/pair operations executors
use crate::memory::MemoryBus;
use crate::cpu::CPUState;
use crate::cpu::instructions::{R16Register, R8Register};
use crate::cpu::exec::register_utils::{r16, set_r16, get_r8, set_r8};

/// Execute INC r16
pub fn exec_inc_r16(cpu_state: &mut CPUState, reg: R16Register, io: &mut [u8; 128], tick: &mut dyn FnMut(&mut [u8; 128])) -> u32 {
    let val = r16(&cpu_state.registers, reg);
    set_r16(&mut cpu_state.registers, reg, val.wrapping_add(1));
    tick(io);
    2
}

/// Execute DEC r16
pub fn exec_dec_r16(cpu_state: &mut CPUState, reg: R16Register, io: &mut [u8; 128], tick: &mut dyn FnMut(&mut [u8; 128])) -> u32 {
    let val = r16(&cpu_state.registers, reg);
    set_r16(&mut cpu_state.registers, reg, val.wrapping_sub(1));
    tick(io);
    2
}

/// Execute INC r8
pub fn exec_inc_r8(cpu_state: &mut CPUState, bus: &mut MemoryBus, reg: R8Register, tick: &mut dyn FnMut(&mut [u8; 128])) -> u32 {
    let old = get_r8(&cpu_state.registers, bus, reg);
    if reg == R8Register::HL {
        tick(&mut bus.io);
    }
    let new_val = old.wrapping_add(1);
    set_r8(&mut cpu_state.registers, bus, reg, new_val);
    if reg == R8Register::HL {
        tick(&mut bus.io);
    }
    cpu_state.registers.modify_f(|f| f.set_zero(new_val == 0));
    cpu_state.registers.modify_f(|f| f.set_subtraction(false));
    cpu_state.registers.modify_f(|f| f.set_half_carry((old & 0x0F) == 0x0F));
    match reg {
        R8Register::HL => 3,
        _ => 1,
    }
}

/// Execute DEC r8
pub fn exec_dec_r8(cpu_state: &mut CPUState, bus: &mut MemoryBus, reg: R8Register, tick: &mut dyn FnMut(&mut [u8; 128])) -> u32 {
    let old = get_r8(&cpu_state.registers, bus, reg);
    if reg == R8Register::HL {
        tick(&mut bus.io);
    }
    let new_val = old.wrapping_sub(1);
    set_r8(&mut cpu_state.registers, bus, reg, new_val);
    if reg == R8Register::HL {
        tick(&mut bus.io);
    }
    cpu_state.registers.modify_f(|f| f.set_zero(new_val == 0));
    cpu_state.registers.modify_f(|f| f.set_subtraction(true));
    cpu_state.registers.modify_f(|f| f.set_half_carry((old & 0x0F) == 0x00));
    match reg {
        R8Register::HL => 3,
        _ => 1,
    }
}

/// Execute ADD HL, r16
pub fn exec_add_hl_r16(cpu_state: &mut CPUState, reg: R16Register, io: &mut [u8; 128], tick: &mut dyn FnMut(&mut [u8; 128])) -> u32 {
    let hl = r16(&cpu_state.registers, R16Register::HL) as u32;
    let add = r16(&cpu_state.registers, reg) as u32;
    let result = hl.wrapping_add(add);
    set_r16(&mut cpu_state.registers, R16Register::HL, result as u16);
    cpu_state.registers.modify_f(|f| f.set_half_carry((hl & 0x0FFF) + (add & 0x0FFF) > 0x0FFF));
    cpu_state.registers.modify_f(|f| f.set_subtraction(false));
    cpu_state.registers.modify_f(|f| f.set_carry(result > 0xFFFF));
    tick(io);
    2
}

#[cfg(test)]
#[path = "registers_tests.rs"]
mod tests;
