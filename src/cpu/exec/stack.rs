/// Stack instruction executors
use crate::memory::MemoryBus;
use crate::cpu::CPUState;
use crate::cpu::instructions::R16Register;
use crate::cpu::exec::register_utils::r16;

/// Execute RET
pub fn exec_ret(cpu_state: &mut CPUState, bus: &mut MemoryBus, tick: &mut dyn FnMut(&mut [u8; 128])) -> u32 {
    let sp = cpu_state.registers.sp;
    let low = bus.read(sp);
    tick(&mut bus.io);
    let high = bus.read(sp.wrapping_add(1));
    tick(&mut bus.io);
    cpu_state.registers.sp = sp.wrapping_add(2);
    cpu_state.registers.pc = ((high as u16) << 8) | (low as u16);
    tick(&mut bus.io); // internal delay
    4
}

/// Execute RETI
pub fn exec_reti(cpu_state: &mut CPUState, bus: &mut MemoryBus, tick: &mut dyn FnMut(&mut [u8; 128])) -> u32 {
    let sp = cpu_state.registers.sp;
    let low = bus.read(sp);
    tick(&mut bus.io);
    let high = bus.read(sp.wrapping_add(1));
    tick(&mut bus.io);
    cpu_state.registers.sp = sp.wrapping_add(2);
    cpu_state.registers.pc = ((high as u16) << 8) | (low as u16);
    cpu_state.ime = true;
    tick(&mut bus.io); // internal delay
    4
}

/// Execute POP r16
pub fn exec_pop_r16(cpu_state: &mut CPUState, reg: R16Register, bus: &mut MemoryBus, tick: &mut dyn FnMut(&mut [u8; 128])) -> u32 {
    let sp = cpu_state.registers.sp;
    let low = bus.read(sp);
    tick(&mut bus.io);
    let high = bus.read(sp.wrapping_add(1));
    tick(&mut bus.io);
    cpu_state.registers.sp = sp.wrapping_add(2);
    let value = ((high as u16) << 8) | (low as u16);
    crate::cpu::exec::register_utils::set_r16(&mut cpu_state.registers, reg, value);
    3
}

/// Execute PUSH r16
pub fn exec_push_r16(cpu_state: &mut CPUState, reg: R16Register, bus: &mut MemoryBus, tick: &mut dyn FnMut(&mut [u8; 128])) -> u32 {
    let sp = cpu_state.registers.sp;
    let value = r16(&cpu_state.registers, reg);
    tick(&mut bus.io); // internal delay before writes
    // Store low byte at the lower address (sp‑2) and high byte at the higher address (sp‑1)
    bus.write(sp.wrapping_sub(1), (value >> 8) as u8);       // high byte at sp‑1
    tick(&mut bus.io);
    bus.write(sp.wrapping_sub(2), (value & 0x00FF) as u8);   // low byte at sp‑2
    tick(&mut bus.io);
    cpu_state.registers.sp = sp.wrapping_sub(2);
    4
}

/// Execute RST n
pub fn exec_rst(cpu_state: &mut CPUState, target: u8, bus: &mut MemoryBus, tick: &mut dyn FnMut(&mut [u8; 128])) -> u32 {
    let sp = cpu_state.registers.sp;
    let return_pc = cpu_state.registers.pc; // PC already pre-advanced by CPU::execute
    tick(&mut bus.io); // internal delay before writes
    bus.write(sp.wrapping_sub(1), (return_pc >> 8) as u8);
    tick(&mut bus.io);
    bus.write(sp.wrapping_sub(2), (return_pc & 0xFF) as u8);
    tick(&mut bus.io);
    cpu_state.registers.sp = sp.wrapping_sub(2);
    cpu_state.registers.pc = target as u16;
    4
}


/// Execute ADD SP, r8
pub fn exec_add_sp_imm8(cpu_state: &mut CPUState, value: i8, io: &mut [u8; 128], tick: &mut dyn FnMut(&mut [u8; 128])) -> u32 {
    let sp = cpu_state.registers.sp;
    // Flags are computed on the unsigned byte addition — the offset's raw
    // bit pattern is treated as an unsigned byte for H and C flag purposes.
    let rhs_byte = value as u8 as u16;
    let result = sp.wrapping_add(value as i16 as u16);
    cpu_state.registers.sp = result;
    cpu_state.registers.modify_f(|f| f.set_zero(false));
    cpu_state.registers.modify_f(|f| f.set_subtraction(false));
    cpu_state.registers.modify_f(|f| f.set_half_carry((sp & 0x0F) + (rhs_byte & 0x0F) > 0x0F));
    cpu_state.registers.modify_f(|f| f.set_carry((sp & 0xFF) + (rhs_byte & 0xFF) > 0xFF));
    tick(io); // simulated imm read
    tick(io); // internal delay 1
    tick(io); // internal delay 2
    4
}

/// Execute LD (HL), SP
pub fn exec_ld_hl_sp_imm8(cpu_state: &mut CPUState, value: i8, io: &mut [u8; 128], tick: &mut dyn FnMut(&mut [u8; 128])) -> u32 {
    let sp = cpu_state.registers.sp;
    let rhs_byte = value as u8 as u16;
    let result = sp.wrapping_add(value as i16 as u16);
    cpu_state.registers.hl = result;
    cpu_state.registers.modify_f(|f| f.set_zero(false));
    cpu_state.registers.modify_f(|f| f.set_subtraction(false));
    cpu_state.registers.modify_f(|f| f.set_half_carry((sp & 0x0F) + (rhs_byte & 0x0F) > 0x0F));
    cpu_state.registers.modify_f(|f| f.set_carry((sp & 0xFF) + (rhs_byte & 0xFF) > 0xFF));
    tick(io); // simulated imm read
    tick(io); // internal delay
    3
}

/// Execute LD SP, HL
pub fn exec_ld_sp_hl(cpu_state: &mut CPUState, io: &mut [u8; 128], tick: &mut dyn FnMut(&mut [u8; 128])) -> u32 {
    cpu_state.registers.sp = cpu_state.registers.hl;
    tick(io); // internal cycle
    2
}

/// Execute DI
pub fn exec_di(cpu_state: &mut CPUState) -> u32 {
    cpu_state.ime = false;
    1
}

/// Execute EI
/// IME is not enabled immediately — it takes effect after the following instruction.
pub fn exec_ei(cpu_state: &mut CPUState) -> u32 {
    cpu_state.ime_pending = true;
    1
}

/// Execute LDH (C), A
pub fn exec_ldh_ind_c_a(cpu_state: &mut CPUState, bus: &mut MemoryBus, tick: &mut dyn FnMut(&mut [u8; 128])) -> u32 {
    bus.write(0xFF00u16.wrapping_add(cpu_state.registers.c() as u16), cpu_state.registers.a());
    tick(&mut bus.io);
    2
}

/// Execute LDH A, (C)
pub fn exec_ldh_a_c(cpu_state: &mut CPUState, bus: &mut MemoryBus, tick: &mut dyn FnMut(&mut [u8; 128])) -> u32 {
    cpu_state.registers.set_a(bus.read(0xFF00u16.wrapping_add(cpu_state.registers.c() as u16)));
    tick(&mut bus.io);
    2
}

/// Execute LDH (a8), A
pub fn exec_ldh_ind_imm8_a(cpu_state: &mut CPUState, address: u8, bus: &mut MemoryBus, tick: &mut dyn FnMut(&mut [u8; 128])) -> u32 {
    tick(&mut bus.io); // simulated imm read
    bus.write(0xFF00u16.wrapping_add(address as u16), cpu_state.registers.a());
    tick(&mut bus.io);
    3
}

/// Execute LDH A, (a8)
pub fn exec_ldh_a_ind_imm8(cpu_state: &mut CPUState, address: u8, bus: &mut MemoryBus, tick: &mut dyn FnMut(&mut [u8; 128])) -> u32 {
    tick(&mut bus.io); // simulated imm read
    cpu_state.registers.set_a(bus.read(0xFF00u16.wrapping_add(address as u16)));
    tick(&mut bus.io);
    3
}

#[cfg(test)]
#[path = "stack_tests.rs"]
mod tests;
