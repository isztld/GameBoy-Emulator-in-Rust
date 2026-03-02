/// Rotate/shift instruction executors
use crate::cpu::CPUState;

/// Execute RLCA
pub fn exec_rlca(cpu_state: &mut CPUState) -> u32 {
    let a = cpu_state.registers.a();
    let new_a = a.rotate_left(1);
    cpu_state.registers.set_a(new_a);
    cpu_state.registers.f_mut().set_carry((a & 0x80) != 0);
    cpu_state.registers.f_mut().set_zero(false);
    cpu_state.registers.f_mut().set_subtraction(false);
    cpu_state.registers.f_mut().set_half_carry(false);
    1
}

/// Execute RRCA
pub fn exec_rrca(cpu_state: &mut CPUState) -> u32 {
    let a = cpu_state.registers.a();
    let new_a = a.rotate_right(1);
    cpu_state.registers.set_a(new_a);
    cpu_state.registers.f_mut().set_carry((a & 0x01) != 0);
    cpu_state.registers.f_mut().set_zero(false);
    cpu_state.registers.f_mut().set_subtraction(false);
    cpu_state.registers.f_mut().set_half_carry(false);
    1
}

/// Execute RLA
pub fn exec_rla(cpu_state: &mut CPUState) -> u32 {
    let a = cpu_state.registers.a();
    let old_c = cpu_state.registers.f().is_carry() as u8;
    let new_a = (a << 1) | old_c;
    cpu_state.registers.set_a(new_a);
    cpu_state.registers.f_mut().set_carry((a & 0x80) != 0);
    cpu_state.registers.f_mut().set_zero(false);
    cpu_state.registers.f_mut().set_subtraction(false);
    cpu_state.registers.f_mut().set_half_carry(false);
    1
}

/// Execute RRA
pub fn exec_rra(cpu_state: &mut CPUState) -> u32 {
    let a = cpu_state.registers.a();
    let old_c = cpu_state.registers.f().is_carry() as u8;
    let new_a = (a >> 1) | (old_c << 7);
    cpu_state.registers.set_a(new_a);
    cpu_state.registers.f_mut().set_carry((a & 0x01) != 0);
    cpu_state.registers.f_mut().set_zero(false);
    cpu_state.registers.f_mut().set_subtraction(false);
    cpu_state.registers.f_mut().set_half_carry(false);
    1
}

/// Execute DAA
pub fn exec_daa(cpu_state: &mut CPUState) -> u32 {
    let mut a = cpu_state.registers.a();
    let mut adjust = 0;
    let mut carry = cpu_state.registers.f().is_carry();
    let f = cpu_state.registers.f();

    if !f.is_subtraction() {
        // ADD case
        if f.is_half_carry() || (a & 0x0F) > 9 {
            adjust += 0x06;
        }
        if carry || a > 0x99 {
            adjust += 0x60;
            carry = true;
        }
        a = a.wrapping_add(adjust);
    } else {
        // SUB case
        if f.is_half_carry() {
            adjust += 0x06;
        }
        if carry {
            adjust += 0x60;
        }
        a = a.wrapping_sub(adjust);
    }

    cpu_state.registers.set_a(a);

    let f_mut = cpu_state.registers.f_mut();
    f_mut.set_zero(a == 0);
    f_mut.set_half_carry(false);

    // Only update carry if it was an ADD
    if !f.is_subtraction() {
        f_mut.set_carry(carry);
    }

    1
}

/// Execute CPL
pub fn exec_cpl(cpu_state: &mut CPUState) -> u32 {
    let a = cpu_state.registers.a();
    cpu_state.registers.set_a(!a);
    cpu_state.registers.f_mut().set_subtraction(true);
    cpu_state.registers.f_mut().set_half_carry(true);
    1
}

/// Execute SCF
pub fn exec_scf(cpu_state: &mut CPUState) -> u32 {
    cpu_state.registers.f_mut().set_carry(true);
    cpu_state.registers.f_mut().set_subtraction(false);
    cpu_state.registers.f_mut().set_half_carry(false);
    1
}

/// Execute CCF
pub fn exec_ccf(cpu_state: &mut CPUState) -> u32 {
    let carry = cpu_state.registers.f().is_carry();
    cpu_state.registers.f_mut().set_carry(!carry);
    cpu_state.registers.f_mut().set_subtraction(false);
    cpu_state.registers.f_mut().set_half_carry(false);
    1
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init_cpu_state() -> CPUState {
        let mut cpu = CPUState::new();
        cpu.registers.set_a(0x00);
        cpu.registers.f_mut().set_zero(false);
        cpu.registers.f_mut().set_subtraction(false);
        cpu.registers.f_mut().set_half_carry(false);
        cpu.registers.f_mut().set_carry(false);
        cpu
    }

    #[test]
    fn test_rlca() {
        let mut cpu = init_cpu_state();
        cpu.registers.set_a(0b11001111);

        let cycles = exec_rlca(&mut cpu);

        assert_eq!(cycles, 1);
        // Rotate left: 0b11001111 -> 0b10011111
        assert_eq!(cpu.registers.a(), 0b10011111);
        // Old bit 7 (1) went to carry
        assert!(cpu.registers.f().is_carry());
        assert!(!cpu.registers.f().is_zero());
        assert!(!cpu.registers.f().is_subtraction());
        assert!(!cpu.registers.f().is_half_carry());
    }

    #[test]
    fn test_rlca_carry_from_zero() {
        let mut cpu = init_cpu_state();
        cpu.registers.set_a(0b01001111);

        let cycles = exec_rlca(&mut cpu);

        assert_eq!(cycles, 1);
        // Rotate left: 0b01001111 -> 0b10011110
        assert_eq!(cpu.registers.a(), 0b10011110);
        // Old bit 7 (0) went to carry
        assert!(!cpu.registers.f().is_carry());
    }

    #[test]
    fn test_rrca() {
        let mut cpu = init_cpu_state();
        cpu.registers.set_a(0b11001111);

        let cycles = exec_rrca(&mut cpu);

        assert_eq!(cycles, 1);
        // Rotate right: 0b11001111 -> 0b11100111
        assert_eq!(cpu.registers.a(), 0b11100111);
        // Old bit 0 (1) went to carry
        assert!(cpu.registers.f().is_carry());
    }

    #[test]
    fn test_rrca_carry_from_zero() {
        let mut cpu = init_cpu_state();
        cpu.registers.set_a(0b11001110);

        let cycles = exec_rrca(&mut cpu);

        assert_eq!(cycles, 1);
        // Rotate right: 0b11001110 -> 0b01100111
        assert_eq!(cpu.registers.a(), 0b01100111);
        // Old bit 0 (0) went to carry
        assert!(!cpu.registers.f().is_carry());
    }

    #[test]
    fn test_rla() {
        let mut cpu = init_cpu_state();
        cpu.registers.set_a(0b11001111);
        cpu.registers.f_mut().set_carry(false);

        let cycles = exec_rla(&mut cpu);

        assert_eq!(cycles, 1);
        // Shift left, carry in: 0b11001111 << 1 = 0b10011110, carry out = 1
        assert_eq!(cpu.registers.a(), 0b10011110);
        assert!(cpu.registers.f().is_carry());
    }

    #[test]
    fn test_rla_with_carry() {
        let mut cpu = init_cpu_state();
        cpu.registers.set_a(0b01001111);
        cpu.registers.f_mut().set_carry(true);

        let cycles = exec_rla(&mut cpu);

        assert_eq!(cycles, 1);
        // Shift left with carry in: 0b01001111 << 1 = 0b10011110, OR carry = 0b10011111
        assert_eq!(cpu.registers.a(), 0b10011111);
        assert!(cpu.registers.f().is_carry());
    }

    #[test]
    fn test_rra() {
        let mut cpu = init_cpu_state();
        cpu.registers.set_a(0b11001111);
        cpu.registers.f_mut().set_carry(false);

        let cycles = exec_rra(&mut cpu);

        assert_eq!(cycles, 1);
        // Shift right, carry in: 0b11001111 >> 1 = 0b01100111, carry out = 1
        assert_eq!(cpu.registers.a(), 0b01100111);
        assert!(cpu.registers.f().is_carry());
    }

    #[test]
    fn test_rra_with_carry() {
        let mut cpu = init_cpu_state();
        cpu.registers.set_a(0b01001111);
        cpu.registers.f_mut().set_carry(true);

        let cycles = exec_rra(&mut cpu);

        assert_eq!(cycles, 1);
        // Shift right with carry in: 0b01001111 >> 1 = 0b00100111, OR (carry << 7) = 0b10100111
        assert_eq!(cpu.registers.a(), 0b10100111);
        assert!(cpu.registers.f().is_carry());
    }

    #[test]
    fn test_daa_add_positive() {
        let mut cpu = init_cpu_state();
        cpu.registers.set_a(0x12);
        cpu.registers.f_mut().set_subtraction(false);
        cpu.registers.f_mut().set_carry(false);

        let cycles = exec_daa(&mut cpu);

        assert_eq!(cycles, 1);
        // 0x12 is already valid BCD, no adjustment needed
        assert_eq!(cpu.registers.a(), 0x12);
        assert!(!cpu.registers.f().is_zero());
    }

    #[test]
    fn test_daa_add_adjust_lower_nibble() {
        let mut cpu = init_cpu_state();
        cpu.registers.set_a(0x1A);
        cpu.registers.f_mut().set_subtraction(false);
        cpu.registers.f_mut().set_carry(false);
        cpu.registers.f_mut().set_half_carry(false);

        let cycles = exec_daa(&mut cpu);

        assert_eq!(cycles, 1);
        // 0x1A > 9, so add 0x06 -> 0x20
        assert_eq!(cpu.registers.a(), 0x20);
    }

    #[test]
    fn test_daa_add_adjust_upper_nibble() {
        let mut cpu = init_cpu_state();
        cpu.registers.set_a(0x9A);
        cpu.registers.f_mut().set_subtraction(false);
        cpu.registers.f_mut().set_carry(false);

        let cycles = exec_daa(&mut cpu);

        assert_eq!(cycles, 1);
        // 0x9A > 0x99, so add 0x60 -> 0xFA
        assert_eq!(cpu.registers.a(), 0xFA);
    }

    #[test]
    fn test_daa_sub() {
        let mut cpu = init_cpu_state();
        cpu.registers.set_a(0x12);
        cpu.registers.f_mut().set_subtraction(true);
        cpu.registers.f_mut().set_carry(false);

        let cycles = exec_daa(&mut cpu);

        assert_eq!(cycles, 1);
        // 0x12 is already valid BCD, no adjustment needed
        assert_eq!(cpu.registers.a(), 0x12);
        // Carry should remain unchanged for subtract
        assert!(!cpu.registers.f().is_carry());
    }

    #[test]
    fn test_cpl() {
        let mut cpu = init_cpu_state();
        cpu.registers.set_a(0xFF);

        let cycles = exec_cpl(&mut cpu);

        assert_eq!(cycles, 1);
        assert_eq!(cpu.registers.a(), 0x00);
        assert!(cpu.registers.f().is_subtraction());
        assert!(cpu.registers.f().is_half_carry());
    }

    #[test]
    fn test_cpl_inverted() {
        let mut cpu = init_cpu_state();
        cpu.registers.set_a(0x00);

        let cycles = exec_cpl(&mut cpu);

        assert_eq!(cycles, 1);
        assert_eq!(cpu.registers.a(), 0xFF);
    }

    #[test]
    fn test_scf() {
        let mut cpu = init_cpu_state();
        // Carry should be false initially

        let cycles = exec_scf(&mut cpu);

        assert_eq!(cycles, 1);
        assert!(cpu.registers.f().is_carry());
        assert!(!cpu.registers.f().is_subtraction());
        assert!(!cpu.registers.f().is_half_carry());
    }

    #[test]
    fn test_ccf() {
        let mut cpu = init_cpu_state();
        cpu.registers.f_mut().set_carry(false);

        let cycles = exec_ccf(&mut cpu);

        assert_eq!(cycles, 1);
        assert!(cpu.registers.f().is_carry());
        assert!(!cpu.registers.f().is_subtraction());
        assert!(!cpu.registers.f().is_half_carry());
    }

    #[test]
    fn test_ccf_invert() {
        let mut cpu = init_cpu_state();
        cpu.registers.f_mut().set_carry(true);

        let cycles = exec_ccf(&mut cpu);

        assert_eq!(cycles, 1);
        assert!(!cpu.registers.f().is_carry());
    }
}
