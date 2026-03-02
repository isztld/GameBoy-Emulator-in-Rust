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
    cpu_state.registers.f_mut().set_half_carry(((a & 0xF) + (val & 0xF) + old_c) > 0xF);
    cpu_state.registers.f_mut().set_carry((a as u16) + (val as u16) + (old_c as u16) > 0xFF);
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
    cpu_state.registers.f_mut().set_half_carry((a & 0xF) < (val & 0xF));
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

    let borrow = val as u16 + old_c as u16;
    cpu_state.registers.f_mut().set_carry((a as u16) < borrow);
    cpu_state.registers.f_mut().set_half_carry((a as u32 & 0xF) < (val as u32 & 0xF) + old_c as u32);
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
    cpu_state.registers.f_mut().set_half_carry((a & 0x0F) < (val & 0x0F));
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
    cpu_state.registers.f_mut().set_carry((a as u16) + (value as u16) + (old_c as u16) > 0xFF);
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
    cpu_state.registers.f_mut().set_half_carry((a & 0x0F) < (value & 0x0F));
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
    cpu_state.registers.f_mut().set_half_carry((a as u32 & 0xF) < (value as u32 & 0xF) + old_c as u32);
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
    cpu_state.registers.f_mut().set_half_carry((a & 0x0F) < (value & 0x0F));
    2
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::MemoryBus;
    use crate::cpu::instructions::R8Register;

    // Helper to initialize CPU state with specific A value
    fn init_cpu_state(a: u8) -> CPUState {
        let mut cpu = CPUState::new();
        cpu.registers.set_a(a);
        cpu.registers.f_mut().set_zero(false);
        cpu.registers.f_mut().set_subtraction(false);
        cpu.registers.f_mut().set_half_carry(false);
        cpu.registers.f_mut().set_carry(false);
        cpu
    }

    #[test]
    fn test_add_a_r8_no_carry() {
        let mut cpu = init_cpu_state(0x10);
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.registers.set_b(0x20);

        let cycles = exec_add_a_r8(&mut cpu, &mut bus, R8Register::B);

        assert_eq!(cycles, 1);
        assert_eq!(cpu.registers.a(), 0x30);
        assert!(!cpu.registers.f().is_zero());
        assert!(!cpu.registers.f().is_subtraction());
        assert!(!cpu.registers.f().is_half_carry());
        assert!(!cpu.registers.f().is_carry());
    }

    #[test]
    fn test_add_a_r8_zero_result() {
        let mut cpu = init_cpu_state(0x00);
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.registers.set_b(0x00);

        let cycles = exec_add_a_r8(&mut cpu, &mut bus, R8Register::B);

        assert_eq!(cycles, 1);
        assert_eq!(cpu.registers.a(), 0x00);
        assert!(cpu.registers.f().is_zero());
        assert!(!cpu.registers.f().is_subtraction());
        assert!(!cpu.registers.f().is_half_carry());
        assert!(!cpu.registers.f().is_carry());
    }

    #[test]
    fn test_add_a_r8_carry() {
        let mut cpu = init_cpu_state(0xFF);
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.registers.set_b(0x01);

        let cycles = exec_add_a_r8(&mut cpu, &mut bus, R8Register::B);

        assert_eq!(cycles, 1);
        assert_eq!(cpu.registers.a(), 0x00);
        assert!(cpu.registers.f().is_carry());
        assert!(cpu.registers.f().is_half_carry());
    }

    #[test]
    fn test_add_a_r8_half_carry() {
        let mut cpu = init_cpu_state(0x0F);
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.registers.set_b(0x01);

        let cycles = exec_add_a_r8(&mut cpu, &mut bus, R8Register::B);

        assert_eq!(cycles, 1);
        assert_eq!(cpu.registers.a(), 0x10);
        assert!(cpu.registers.f().is_half_carry());
        assert!(!cpu.registers.f().is_carry());
    }

    #[test]
    fn test_adc_a_r8_no_carry() {
        let mut cpu = init_cpu_state(0x10);
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.registers.set_b(0x20);

        let cycles = exec_adc_a_r8(&mut cpu, &mut bus, R8Register::B);

        assert_eq!(cycles, 1);
        assert_eq!(cpu.registers.a(), 0x30);
        assert!(!cpu.registers.f().is_carry());
    }

    #[test]
    fn test_adc_a_r8_with_carry() {
        let mut cpu = init_cpu_state(0x10);
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.registers.set_b(0x20);
        cpu.registers.f_mut().set_carry(true);

        let cycles = exec_adc_a_r8(&mut cpu, &mut bus, R8Register::B);

        assert_eq!(cycles, 1);
        assert_eq!(cpu.registers.a(), 0x31); // 10 + 20 + 1
    }

    #[test]
    fn test_sub_a_r8() {
        let mut cpu = init_cpu_state(0x50);
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.registers.set_b(0x30);

        let cycles = exec_sub_a_r8(&mut cpu, &mut bus, R8Register::B);

        assert_eq!(cycles, 1);
        assert_eq!(cpu.registers.a(), 0x20);
        assert!(cpu.registers.f().is_subtraction());
        assert!(!cpu.registers.f().is_carry());
    }

    #[test]
    fn test_sub_a_r8_carry() {
        let mut cpu = init_cpu_state(0x10);
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.registers.set_b(0x20);

        let cycles = exec_sub_a_r8(&mut cpu, &mut bus, R8Register::B);

        assert_eq!(cycles, 1);
        assert_eq!(cpu.registers.a(), 0xF0);
        assert!(cpu.registers.f().is_carry());
    }

    #[test]
    fn test_sbc_a_r8() {
        let mut cpu = init_cpu_state(0x50);
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.registers.set_b(0x30);
        cpu.registers.f_mut().set_carry(true); // set initial carry

        let cycles = exec_sbc_a_r8(&mut cpu, &mut bus, R8Register::B);

        assert_eq!(cycles, 1);
        assert_eq!(cpu.registers.a(), 0x1F); // 50 - 30 - 1
    }

    #[test]
    fn test_sbc_a_r8_with_carry() {
        let mut cpu = init_cpu_state(0x50);
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.registers.set_b(0x30);
        cpu.registers.f_mut().set_carry(true);

        let cycles = exec_sbc_a_r8(&mut cpu, &mut bus, R8Register::B);

        assert_eq!(cycles, 1);
        assert_eq!(cpu.registers.a(), 0x1F); // 50 - 30 - 2
    }

    #[test]
    fn test_and_a_r8() {
        let mut cpu = init_cpu_state(0xFF);
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.registers.set_b(0x0F);

        let cycles = exec_and_a_r8(&mut cpu, &mut bus, R8Register::B);

        assert_eq!(cycles, 1);
        assert_eq!(cpu.registers.a(), 0x0F);
        assert!(!cpu.registers.f().is_zero());
        assert!(!cpu.registers.f().is_subtraction());
        assert!(cpu.registers.f().is_half_carry());
        assert!(!cpu.registers.f().is_carry());
    }

    #[test]
    fn test_and_a_r8_zero() {
        let mut cpu = init_cpu_state(0xFF);
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.registers.set_b(0x00);

        let cycles = exec_and_a_r8(&mut cpu, &mut bus, R8Register::B);

        assert_eq!(cycles, 1);
        assert_eq!(cpu.registers.a(), 0x00);
        assert!(cpu.registers.f().is_zero());
    }

    #[test]
    fn test_xor_a_r8() {
        let mut cpu = init_cpu_state(0xFF);
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.registers.set_b(0x0F);

        let cycles = exec_xor_a_r8(&mut cpu, &mut bus, R8Register::B);

        assert_eq!(cycles, 1);
        assert_eq!(cpu.registers.a(), 0xF0);
        assert!(!cpu.registers.f().is_zero());
    }

    #[test]
    fn test_xor_a_r8_same() {
        let mut cpu = init_cpu_state(0x5A);
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.registers.set_b(0x5A);

        let cycles = exec_xor_a_r8(&mut cpu, &mut bus, R8Register::B);

        assert_eq!(cycles, 1);
        assert_eq!(cpu.registers.a(), 0x00);
        assert!(cpu.registers.f().is_zero());
    }

    #[test]
    fn test_or_a_r8() {
        let mut cpu = init_cpu_state(0xF0);
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.registers.set_b(0x0F);

        let cycles = exec_or_a_r8(&mut cpu, &mut bus, R8Register::B);

        assert_eq!(cycles, 1);
        assert_eq!(cpu.registers.a(), 0xFF);
        assert!(!cpu.registers.f().is_zero());
    }

    #[test]
    fn test_or_a_r8_zero() {
        let mut cpu = init_cpu_state(0x00);
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.registers.set_b(0x00);

        let cycles = exec_or_a_r8(&mut cpu, &mut bus, R8Register::B);

        assert_eq!(cycles, 1);
        assert_eq!(cpu.registers.a(), 0x00);
        assert!(cpu.registers.f().is_zero());
    }

    #[test]
    fn test_cp_a_r8_equal() {
        let mut cpu = init_cpu_state(0x50);
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.registers.set_b(0x50);

        let cycles = exec_cp_a_r8(&mut cpu, &mut bus, R8Register::B);

        assert_eq!(cycles, 1);
        assert!(cpu.registers.f().is_zero());
        assert!(cpu.registers.f().is_subtraction());
        assert!(!cpu.registers.f().is_carry());
        // A should not be modified
        assert_eq!(cpu.registers.a(), 0x50);
    }

    #[test]
    fn test_cp_a_r8_less() {
        let mut cpu = init_cpu_state(0x30);
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.registers.set_b(0x50);

        let cycles = exec_cp_a_r8(&mut cpu, &mut bus, R8Register::B);

        assert_eq!(cycles, 1);
        assert!(!cpu.registers.f().is_zero());
        assert!(cpu.registers.f().is_carry());
    }

    #[test]
    fn test_cp_a_r8_negative() {
        let mut cpu = init_cpu_state(0x10);
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.registers.set_b(0x20);

        let cycles = exec_cp_a_r8(&mut cpu, &mut bus, R8Register::B);

        assert_eq!(cycles, 1);
        // In signed comparison, 0x10 < 0x20, so carry should be set
        assert!(cpu.registers.f().is_carry());
    }

    #[test]
    fn test_add_a_imm8() {
        let mut cpu = init_cpu_state(0x10);

        let cycles = exec_add_a_imm8(&mut cpu, 0x20);

        assert_eq!(cycles, 2);
        assert_eq!(cpu.registers.a(), 0x30);
        assert!(!cpu.registers.f().is_carry());
    }

    #[test]
    fn test_adc_a_imm8() {
        let mut cpu = init_cpu_state(0x10);

        let cycles = exec_adc_a_imm8(&mut cpu, 0x20);

        assert_eq!(cycles, 2);
        assert_eq!(cpu.registers.a(), 0x30);
    }

    #[test]
    fn test_adc_a_imm8_with_carry() {
        let mut cpu = init_cpu_state(0x10);
        cpu.registers.f_mut().set_carry(true);

        let cycles = exec_adc_a_imm8(&mut cpu, 0x20);

        assert_eq!(cycles, 2);
        assert_eq!(cpu.registers.a(), 0x31);
    }

    #[test]
    fn test_sub_a_imm8() {
        let mut cpu = init_cpu_state(0x50);

        let cycles = exec_sub_a_imm8(&mut cpu, 0x30);

        assert_eq!(cycles, 2);
        assert_eq!(cpu.registers.a(), 0x20);
        assert!(cpu.registers.f().is_subtraction());
    }

    #[test]
    fn test_sbc_a_imm8() {
        let mut cpu = init_cpu_state(0x50);

        let cycles = exec_sbc_a_imm8(&mut cpu, 0x30);

        assert_eq!(cycles, 2);
        assert_eq!(cpu.registers.a(), 0x20);
    }

    #[test]
    fn test_and_a_imm8() {
        let mut cpu = init_cpu_state(0xFF);

        let cycles = exec_and_a_imm8(&mut cpu, 0x0F);

        assert_eq!(cycles, 2);
        assert_eq!(cpu.registers.a(), 0x0F);
        assert!(cpu.registers.f().is_half_carry());
    }

    #[test]
    fn test_xor_a_imm8() {
        let mut cpu = init_cpu_state(0xFF);

        let cycles = exec_xor_a_imm8(&mut cpu, 0x0F);

        assert_eq!(cycles, 2);
        assert_eq!(cpu.registers.a(), 0xF0);
    }

    #[test]
    fn test_or_a_imm8() {
        let mut cpu = init_cpu_state(0xF0);

        let cycles = exec_or_a_imm8(&mut cpu, 0x0F);

        assert_eq!(cycles, 2);
        assert_eq!(cpu.registers.a(), 0xFF);
    }

    #[test]
    fn test_cp_a_imm8() {
        let mut cpu = init_cpu_state(0x50);

        let cycles = exec_cp_a_imm8(&mut cpu, 0x50);

        assert_eq!(cycles, 2);
        assert!(cpu.registers.f().is_zero());
        assert!(cpu.registers.f().is_subtraction());
    }
}
