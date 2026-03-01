/// Stack instruction executors
use crate::memory::MemoryBus;
use crate::cpu::CPUState;
use crate::cpu::instructions::R16Register;
use crate::cpu::exec::register_utils::r16;

/// Execute RET
pub fn exec_ret(cpu_state: &mut CPUState, bus: &mut MemoryBus) -> u32 {
    let sp = cpu_state.registers.sp;
    let low = bus.read(sp);
    let high = bus.read(sp.wrapping_add(1));
    cpu_state.registers.sp = sp.wrapping_add(2);
    cpu_state.registers.pc = ((high as u16) << 8) | (low as u16);
    4
}

/// Execute RETI
pub fn exec_reti(cpu_state: &mut CPUState, bus: &mut MemoryBus) -> u32 {
    let sp = cpu_state.registers.sp;
    let low = bus.read(sp);
    let high = bus.read(sp.wrapping_add(1));
    cpu_state.registers.sp = sp.wrapping_add(2);
    cpu_state.registers.pc = ((high as u16) << 8) | (low as u16);
    cpu_state.ime = true;
    4
}

/// Execute POP r16
pub fn exec_pop_r16(cpu_state: &mut CPUState, reg: R16Register, bus: &mut MemoryBus) -> u32 {
    let sp = cpu_state.registers.sp;
    let low = bus.read(sp);
    let high = bus.read(sp.wrapping_add(1));
    cpu_state.registers.sp = sp.wrapping_add(2);
    let value = ((high as u16) << 8) | (low as u16);
    crate::cpu::exec::register_utils::set_r16(&mut cpu_state.registers, reg, value);
    3
}

/// Execute PUSH r16
pub fn exec_push_r16(cpu_state: &mut CPUState, reg: R16Register, bus: &mut MemoryBus) -> u32 {
    let sp = cpu_state.registers.sp;
    let value = r16(&cpu_state.registers, reg);
    bus.write(sp.wrapping_sub(1), (value >> 8) as u8);
    bus.write(sp.wrapping_sub(2), (value & 0x00FF) as u8);
    cpu_state.registers.sp = sp.wrapping_sub(2);
    5
}

/// Execute RST n
pub fn exec_rst(cpu_state: &mut CPUState, target: u8, bus: &mut MemoryBus) -> u32 {
    let sp = cpu_state.registers.sp;
    bus.write(sp.wrapping_sub(1), (cpu_state.registers.pc >> 8) as u8);
    bus.write(sp.wrapping_sub(2), (cpu_state.registers.pc & 0x00FF) as u8);
    cpu_state.registers.sp = sp.wrapping_sub(2);
    cpu_state.registers.pc = target as u16;
    4
}

/// Execute ADD SP, r8
pub fn exec_add_sp_imm8(cpu_state: &mut CPUState, value: i8) -> u32 {
    let sp = cpu_state.registers.sp;
    let result = sp.wrapping_add(value as u16);
    cpu_state.registers.sp = result;
    cpu_state.registers.f_mut().set_zero(false);
    cpu_state.registers.f_mut().set_subtraction(false);
    // Half carry: carry from bit 3 to bit 4
    cpu_state.registers.f_mut().set_half_carry((sp & 0x0F) + (value as u16 & 0x0F) > 0x0F);
    // Carry: set when there's a borrow from bit 15 (for negative values)
    // For ADD SP, r8 with negative value: carry is set when lower byte underflows
    let result_i16 = ((sp as i16).wrapping_add(value as i16)) as u16;
    cpu_state.registers.f_mut().set_carry(result_i16 < sp);
    4
}

/// Execute LD (HL), SP
pub fn exec_ld_hl_sp_imm8(cpu_state: &mut CPUState, value: i8) -> u32 {
    let sp = cpu_state.registers.sp as i32;
    let result = sp.wrapping_add(value as i32) as u16;
    cpu_state.registers.hl = result;
    cpu_state.registers.f_mut().set_zero(false);
    cpu_state.registers.f_mut().set_subtraction(false);
    cpu_state.registers.f_mut().set_half_carry((sp & 0x0F) + (value as i32 & 0x0F) > 0x0F);
    cpu_state.registers.f_mut().set_carry((sp & 0xFF) + (value as i32 & 0xFF) > 0xFF);
    3
}

/// Execute LD SP, HL
pub fn exec_ld_sp_hl(cpu_state: &mut CPUState) -> u32 {
    cpu_state.registers.sp = cpu_state.registers.hl;
    2
}

/// Execute DI
pub fn exec_di(cpu_state: &mut CPUState) -> u32 {
    cpu_state.ime = false;
    1
}

/// Execute EI
pub fn exec_ei(cpu_state: &mut CPUState) -> u32 {
    cpu_state.ime = true;
    1
}

/// Execute LDH (C), A
pub fn exec_ldh_ind_c_a(cpu_state: &mut CPUState, bus: &mut MemoryBus) -> u32 {
    bus.write(0xFF00u16.wrapping_add(cpu_state.registers.c() as u16), cpu_state.registers.a());
    2
}

/// Execute LDH A, (C)
pub fn exec_ldh_a_c(cpu_state: &mut CPUState, bus: &mut MemoryBus) -> u32 {
    cpu_state.registers.set_a(bus.read(0xFF00u16.wrapping_add(cpu_state.registers.c() as u16)));
    2
}

/// Execute LDH (a8), A
pub fn exec_ldh_ind_imm8_a(cpu_state: &mut CPUState, address: u8, bus: &mut MemoryBus) -> u32 {
    bus.write(0xFF00u16.wrapping_add(address as u16), cpu_state.registers.a());
    3
}

/// Execute LDH A, (a8)
pub fn exec_ldh_a_ind_imm8(cpu_state: &mut CPUState, address: u8, bus: &mut MemoryBus) -> u32 {
    cpu_state.registers.set_a(bus.read(0xFF00u16.wrapping_add(address as u16)));
    3
}
