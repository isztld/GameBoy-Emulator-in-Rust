/// ALU instruction executors
use crate::memory::MemoryBus;
use crate::cpu::CPUState;
use crate::cpu::instructions::R8Register;
use crate::cpu::exec::register_utils::get_r8;

/// Execute ADD A, r8
pub fn exec_add_a_r8(cpu_state: &mut CPUState, bus: &mut MemoryBus, reg: R8Register) -> u32 {
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

/// Execute ADC A, r8
pub fn exec_adc_a_r8(cpu_state: &mut CPUState, bus: &mut MemoryBus, reg: R8Register) -> u32 {
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

/// Execute SUB A, r8
pub fn exec_sub_a_r8(cpu_state: &mut CPUState, bus: &mut MemoryBus, reg: R8Register) -> u32 {
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

/// Execute SBC A, r8
pub fn exec_sbc_a_r8(cpu_state: &mut CPUState, bus: &mut MemoryBus, reg: R8Register) -> u32 {
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

/// Execute AND A, r8
pub fn exec_and_a_r8(cpu_state: &mut CPUState, bus: &mut MemoryBus, reg: R8Register) -> u32 {
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

/// Execute XOR A, r8
pub fn exec_xor_a_r8(cpu_state: &mut CPUState, bus: &mut MemoryBus, reg: R8Register) -> u32 {
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

/// Execute OR A, r8
pub fn exec_or_a_r8(cpu_state: &mut CPUState, bus: &mut MemoryBus, reg: R8Register) -> u32 {
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

/// Execute CP A, r8
pub fn exec_cp_a_r8(cpu_state: &mut CPUState, bus: &mut MemoryBus, reg: R8Register) -> u32 {
    let val = get_r8(&cpu_state.registers, bus, reg);
    let a = cpu_state.registers.a();
    let result = a.wrapping_sub(val);
    cpu_state.registers.f_mut().set_zero(result == 0);
    cpu_state.registers.f_mut().set_subtraction(true);
    cpu_state.registers.f_mut().set_carry(a < val);
    cpu_state.registers.f_mut().set_half_carry((a as i8) < (val as i8));
    1
}

/// Execute ADD A, d8
pub fn exec_add_a_imm8(cpu_state: &mut CPUState, value: u8) -> u32 {
    let a = cpu_state.registers.a();
    let result = a.wrapping_add(value);
    cpu_state.registers.set_a(result);
    cpu_state.registers.f_mut().set_zero(result == 0);
    cpu_state.registers.f_mut().set_subtraction(false);
    cpu_state.registers.f_mut().set_half_carry((a & 0x0F) + (value & 0x0F) > 0x0F);
    cpu_state.registers.f_mut().set_carry(result < a);
    2
}

/// Execute ADC A, d8
pub fn exec_adc_a_imm8(cpu_state: &mut CPUState, value: u8) -> u32 {
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

/// Execute SUB A, d8
pub fn exec_sub_a_imm8(cpu_state: &mut CPUState, value: u8) -> u32 {
    let a = cpu_state.registers.a();
    let result = a.wrapping_sub(value);
    cpu_state.registers.set_a(result);
    cpu_state.registers.f_mut().set_zero(result == 0);
    cpu_state.registers.f_mut().set_subtraction(true);
    cpu_state.registers.f_mut().set_carry(a < value);
    cpu_state.registers.f_mut().set_half_carry((a as i8) < (value as i8));
    2
}

/// Execute SBC A, d8
pub fn exec_sbc_a_imm8(cpu_state: &mut CPUState, value: u8) -> u32 {
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

/// Execute AND A, d8
pub fn exec_and_a_imm8(cpu_state: &mut CPUState, value: u8) -> u32 {
    let a = cpu_state.registers.a();
    let result = a & value;
    cpu_state.registers.set_a(result);
    cpu_state.registers.f_mut().set_zero(result == 0);
    cpu_state.registers.f_mut().set_subtraction(false);
    cpu_state.registers.f_mut().set_half_carry(true);
    cpu_state.registers.f_mut().set_carry(false);
    2
}

/// Execute XOR A, d8
pub fn exec_xor_a_imm8(cpu_state: &mut CPUState, value: u8) -> u32 {
    let a = cpu_state.registers.a();
    let result = a ^ value;
    cpu_state.registers.set_a(result);
    cpu_state.registers.f_mut().set_zero(result == 0);
    cpu_state.registers.f_mut().set_subtraction(false);
    cpu_state.registers.f_mut().set_half_carry(false);
    cpu_state.registers.f_mut().set_carry(false);
    2
}

/// Execute OR A, d8
pub fn exec_or_a_imm8(cpu_state: &mut CPUState, value: u8) -> u32 {
    let a = cpu_state.registers.a();
    let result = a | value;
    cpu_state.registers.set_a(result);
    cpu_state.registers.f_mut().set_zero(result == 0);
    cpu_state.registers.f_mut().set_subtraction(false);
    cpu_state.registers.f_mut().set_half_carry(false);
    cpu_state.registers.f_mut().set_carry(false);
    2
}

/// Execute CP A, d8
pub fn exec_cp_a_imm8(cpu_state: &mut CPUState, value: u8) -> u32 {
    let a = cpu_state.registers.a();
    let result = a.wrapping_sub(value);
    cpu_state.registers.f_mut().set_zero(result == 0);
    cpu_state.registers.f_mut().set_subtraction(true);
    cpu_state.registers.f_mut().set_carry(a < value);
    cpu_state.registers.f_mut().set_half_carry((a as i8) < (value as i8));
    2
}
