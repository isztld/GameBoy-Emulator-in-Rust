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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::MemoryBus;
    use crate::cpu::instructions::R16Register;

    fn init_cpu_state() -> CPUState {
        let mut cpu = CPUState::new();
        cpu.registers.f_mut().set_zero(false);
        cpu.registers.f_mut().set_subtraction(false);
        cpu.registers.f_mut().set_half_carry(false);
        cpu.registers.f_mut().set_carry(false);
        cpu
    }

    #[test]
    fn test_inc_r16() {
        let mut cpu = init_cpu_state();
        cpu.registers.bc = 0x1234;

        let cycles = exec_inc_r16(&mut cpu, R16Register::BC);

        assert_eq!(cycles, 2);
        assert_eq!(cpu.registers.bc, 0x1235);
    }

    #[test]
    fn test_inc_r16_wrap() {
        let mut cpu = init_cpu_state();
        cpu.registers.bc = 0xFFFF;

        let cycles = exec_inc_r16(&mut cpu, R16Register::BC);

        assert_eq!(cycles, 2);
        assert_eq!(cpu.registers.bc, 0x0000);
    }

    #[test]
    fn test_dec_r16() {
        let mut cpu = init_cpu_state();
        cpu.registers.bc = 0x1234;

        let cycles = exec_dec_r16(&mut cpu, R16Register::BC);

        assert_eq!(cycles, 2);
        assert_eq!(cpu.registers.bc, 0x1233);
    }

    #[test]
    fn test_dec_r16_wrap() {
        let mut cpu = init_cpu_state();
        cpu.registers.bc = 0x0000;

        let cycles = exec_dec_r16(&mut cpu, R16Register::BC);

        assert_eq!(cycles, 2);
        assert_eq!(cpu.registers.bc, 0xFFFF);
    }

    #[test]
    fn test_inc_r8() {
        let mut cpu = init_cpu_state();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.registers.set_b(0x10);

        let cycles = exec_inc_r8(&mut cpu, &mut bus, R8Register::B);

        assert_eq!(cycles, 1);
        assert_eq!(cpu.registers.b(), 0x11);
        assert!(!cpu.registers.f().is_zero());
        assert!(!cpu.registers.f().is_subtraction());
    }

    #[test]
    fn test_inc_r8_zero() {
        let mut cpu = init_cpu_state();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.registers.set_b(0xFF);

        let cycles = exec_inc_r8(&mut cpu, &mut bus, R8Register::B);

        assert_eq!(cycles, 1);
        assert_eq!(cpu.registers.b(), 0x00);
        assert!(cpu.registers.f().is_zero());
    }

    #[test]
    fn test_dec_r8() {
        let mut cpu = init_cpu_state();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.registers.set_b(0x10);

        let cycles = exec_dec_r8(&mut cpu, &mut bus, R8Register::B);

        assert_eq!(cycles, 1);
        assert_eq!(cpu.registers.b(), 0x0F);
        assert!(!cpu.registers.f().is_zero());
        assert!(cpu.registers.f().is_subtraction());
    }

    #[test]
    fn test_dec_r8_zero() {
        let mut cpu = init_cpu_state();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.registers.set_b(0x00);

        let cycles = exec_dec_r8(&mut cpu, &mut bus, R8Register::B);

        assert_eq!(cycles, 1);
        assert_eq!(cpu.registers.b(), 0xFF);
        assert!(cpu.registers.f().is_zero());
        assert!(cpu.registers.f().is_subtraction());
    }

    #[test]
    fn test_add_hl_r16_no_carry() {
        let mut cpu = init_cpu_state();
        let _bus = MemoryBus::new(vec![0; 32768]);
        cpu.registers.hl = 0x1000;
        cpu.registers.bc = 0x2000;

        let cycles = exec_add_hl_r16(&mut cpu, R16Register::BC);

        assert_eq!(cycles, 2);
        assert_eq!(cpu.registers.hl, 0x3000);
        assert!(!cpu.registers.f().is_half_carry());
        assert!(!cpu.registers.f().is_carry());
        assert!(!cpu.registers.f().is_subtraction());
    }

    #[test]
    fn test_add_hl_r16_with_carry() {
        let mut cpu = init_cpu_state();
        cpu.registers.hl = 0xFFFF;
        cpu.registers.bc = 0x0001;

        let cycles = exec_add_hl_r16(&mut cpu, R16Register::BC);

        assert_eq!(cycles, 2);
        assert_eq!(cpu.registers.hl, 0x0000);
        assert!(cpu.registers.f().is_carry());
    }

    #[test]
    fn test_add_hl_r16_half_carry() {
        let mut cpu = init_cpu_state();
        cpu.registers.hl = 0x0FFF;
        cpu.registers.bc = 0x0001;

        let cycles = exec_add_hl_r16(&mut cpu, R16Register::BC);

        assert_eq!(cycles, 2);
        assert_eq!(cpu.registers.hl, 0x1000);
        assert!(cpu.registers.f().is_half_carry());
    }

    #[test]
    fn test_add_hl_r16_different_registers() {
        let mut cpu = init_cpu_state();
        cpu.registers.hl = 0x1000;
        cpu.registers.de = 0x2000;

        let cycles = exec_add_hl_r16(&mut cpu, R16Register::DE);

        assert_eq!(cycles, 2);
        assert_eq!(cpu.registers.hl, 0x3000);

        cpu.registers.hl = 0x1000;
        cpu.registers.sp = 0x2000;

        let cycles = exec_add_hl_r16(&mut cpu, R16Register::SP);

        assert_eq!(cycles, 2);
        assert_eq!(cpu.registers.hl, 0x3000);
    }
}
