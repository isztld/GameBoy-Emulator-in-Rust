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
    let a = cpu_state.registers.a();
    let sub = cpu_state.registers.f().is_subtraction();
    let half_carry = cpu_state.registers.f().is_half_carry();
    let carry = cpu_state.registers.f().is_carry();

    let mut adjust: u8 = 0;
    let mut new_carry = carry;

    if !sub {
        if half_carry || (a & 0x0F) > 9 {
            adjust |= 0x06;
        }
        if carry || a > 0x99 {
            adjust |= 0x60;
            new_carry = true;
        }
    } else {
        if half_carry {
            adjust |= 0x06;
        }
        if carry {
            adjust |= 0x60;
        }
        // new_carry is unchanged for subtract
    }

    let new_a = if !sub {
        a.wrapping_add(adjust)
    } else {
        a.wrapping_sub(adjust)
    };

    cpu_state.registers.set_a(new_a);
    cpu_state.registers.f_mut().set_zero(new_a == 0);
    cpu_state.registers.f_mut().set_half_carry(false);
    cpu_state.registers.f_mut().set_carry(new_carry);
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

    // -----------------------------------------------------------------------
    // RLCA
    // -----------------------------------------------------------------------

    #[test]
    fn test_rlca_with_bit7_set() {
        let mut cpu = init_cpu_state();
        cpu.registers.set_a(0b11001111);
        assert_eq!(exec_rlca(&mut cpu), 1);
        assert_eq!(cpu.registers.a(), 0b10011111);
        assert!(cpu.registers.f().is_carry());
        assert!(!cpu.registers.f().is_zero());
        assert!(!cpu.registers.f().is_subtraction());
        assert!(!cpu.registers.f().is_half_carry());
    }

    #[test]
    fn test_rlca_with_bit7_clear() {
        let mut cpu = init_cpu_state();
        cpu.registers.set_a(0b01001111);
        exec_rlca(&mut cpu);
        assert_eq!(cpu.registers.a(), 0b10011110);
        assert!(!cpu.registers.f().is_carry());
    }

    #[test]
    fn test_rlca_clears_zero_flag() {
        // RLCA always clears zero, even if A was 0x00 before (result 0x00 after rotate)
        // 0x00 rotated is still 0x00, but zero flag must be 0.
        let mut cpu = init_cpu_state();
        cpu.registers.set_a(0x00);
        cpu.registers.f_mut().set_zero(true);
        exec_rlca(&mut cpu);
        assert!(!cpu.registers.f().is_zero());
    }

    // -----------------------------------------------------------------------
    // RRCA
    // -----------------------------------------------------------------------

    #[test]
    fn test_rrca_with_bit0_set() {
        let mut cpu = init_cpu_state();
        cpu.registers.set_a(0b11001111);
        assert_eq!(exec_rrca(&mut cpu), 1);
        assert_eq!(cpu.registers.a(), 0b11100111);
        assert!(cpu.registers.f().is_carry());
        assert!(!cpu.registers.f().is_zero());
        assert!(!cpu.registers.f().is_subtraction());
        assert!(!cpu.registers.f().is_half_carry());
    }

    #[test]
    fn test_rrca_with_bit0_clear() {
        let mut cpu = init_cpu_state();
        cpu.registers.set_a(0b11001110);
        exec_rrca(&mut cpu);
        assert_eq!(cpu.registers.a(), 0b01100111);
        assert!(!cpu.registers.f().is_carry());
    }

    // -----------------------------------------------------------------------
    // RLA
    // -----------------------------------------------------------------------

    #[test]
    fn test_rla_carry_in_zero_carry_out_one() {
        let mut cpu = init_cpu_state();
        cpu.registers.set_a(0b11001111);
        cpu.registers.f_mut().set_carry(false);
        assert_eq!(exec_rla(&mut cpu), 1);
        // bit 7 was 1 → carry out = 1; carry in = 0 → bit 0 = 0
        assert_eq!(cpu.registers.a(), 0b10011110);
        assert!(cpu.registers.f().is_carry());
        assert!(!cpu.registers.f().is_zero());
        assert!(!cpu.registers.f().is_subtraction());
        assert!(!cpu.registers.f().is_half_carry());
    }

    #[test]
    fn test_rla_carry_in_one_carry_out_zero() {
        let mut cpu = init_cpu_state();
        cpu.registers.set_a(0b01001111);
        cpu.registers.f_mut().set_carry(true);
        exec_rla(&mut cpu);
        // bit 7 was 0 → carry out = 0; carry in = 1 → bit 0 = 1
        assert_eq!(cpu.registers.a(), 0b10011111);
        assert!(!cpu.registers.f().is_carry()); // carry out is 0, not 1
    }

    #[test]
    fn test_rla_carry_in_one_carry_out_one() {
        let mut cpu = init_cpu_state();
        cpu.registers.set_a(0b11001111);
        cpu.registers.f_mut().set_carry(true);
        exec_rla(&mut cpu);
        assert_eq!(cpu.registers.a(), 0b10011111);
        assert!(cpu.registers.f().is_carry());
    }

    // -----------------------------------------------------------------------
    // RRA
    // -----------------------------------------------------------------------

    #[test]
    fn test_rra_carry_in_zero_carry_out_one() {
        let mut cpu = init_cpu_state();
        cpu.registers.set_a(0b11001111);
        cpu.registers.f_mut().set_carry(false);
        assert_eq!(exec_rra(&mut cpu), 1);
        // bit 0 was 1 → carry out = 1; carry in = 0 → bit 7 = 0
        assert_eq!(cpu.registers.a(), 0b01100111);
        assert!(cpu.registers.f().is_carry());
        assert!(!cpu.registers.f().is_zero());
        assert!(!cpu.registers.f().is_subtraction());
        assert!(!cpu.registers.f().is_half_carry());
    }

    #[test]
    fn test_rra_carry_in_one_carry_out_zero() {
        let mut cpu = init_cpu_state();
        cpu.registers.set_a(0b01001110);
        cpu.registers.f_mut().set_carry(true);
        exec_rra(&mut cpu);
        // bit 0 was 0 → carry out = 0; carry in = 1 → bit 7 = 1
        assert_eq!(cpu.registers.a(), 0b10100111);
        assert!(!cpu.registers.f().is_carry());
    }

    #[test]
    fn test_rra_carry_in_one_carry_out_one() {
        let mut cpu = init_cpu_state();
        cpu.registers.set_a(0b01001111);
        cpu.registers.f_mut().set_carry(true);
        exec_rra(&mut cpu);
        assert_eq!(cpu.registers.a(), 0b10100111);
        assert!(cpu.registers.f().is_carry());
    }

    // -----------------------------------------------------------------------
    // DAA
    // -----------------------------------------------------------------------

    #[test]
    fn test_daa_add_valid_bcd_no_adjustment() {
        // 0x12 is already valid BCD after an add — no correction needed
        let mut cpu = init_cpu_state();
        cpu.registers.set_a(0x12);
        assert_eq!(exec_daa(&mut cpu), 1);
        assert_eq!(cpu.registers.a(), 0x12);
        assert!(!cpu.registers.f().is_zero());
        assert!(!cpu.registers.f().is_carry());
        assert!(!cpu.registers.f().is_half_carry());
    }

    #[test]
    fn test_daa_add_lower_nibble_overflow() {
        // 0x0A in lower nibble → add 0x06
        let mut cpu = init_cpu_state();
        cpu.registers.set_a(0x1A);
        exec_daa(&mut cpu);
        assert_eq!(cpu.registers.a(), 0x20);
        assert!(!cpu.registers.f().is_carry());
    }

    #[test]
    fn test_daa_add_both_nibbles_overflow() {
        // 0x9A: lower nibble > 9 AND value > 0x99 → add 0x06 + 0x60 = 0x66
        // 0x9A + 0x66 = 0x100, wraps to 0x00, carry set
        let mut cpu = init_cpu_state();
        cpu.registers.set_a(0x9A);
        exec_daa(&mut cpu);
        assert_eq!(cpu.registers.a(), 0x00);
        assert!(cpu.registers.f().is_zero());
        assert!(cpu.registers.f().is_carry());
        assert!(!cpu.registers.f().is_half_carry());
    }

    #[test]
    fn test_daa_add_upper_nibble_overflow_via_carry() {
        // Carry flag set from previous add → upper correction
        let mut cpu = init_cpu_state();
        cpu.registers.set_a(0x12);
        cpu.registers.f_mut().set_carry(true);
        exec_daa(&mut cpu);
        // 0x12 + 0x60 = 0x72, carry preserved
        assert_eq!(cpu.registers.a(), 0x72);
        assert!(cpu.registers.f().is_carry());
    }

    #[test]
    fn test_daa_add_half_carry_flag() {
        // Half-carry set → lower correction regardless of nibble value
        let mut cpu = init_cpu_state();
        cpu.registers.set_a(0x10);
        cpu.registers.f_mut().set_half_carry(true);
        exec_daa(&mut cpu);
        assert_eq!(cpu.registers.a(), 0x16);
        assert!(!cpu.registers.f().is_half_carry()); // always cleared by DAA
    }

    #[test]
    fn test_daa_sub_valid_bcd_no_adjustment() {
        let mut cpu = init_cpu_state();
        cpu.registers.set_a(0x12);
        cpu.registers.f_mut().set_subtraction(true);
        exec_daa(&mut cpu);
        assert_eq!(cpu.registers.a(), 0x12);
        assert!(!cpu.registers.f().is_carry()); // carry unchanged for sub
    }

    #[test]
    fn test_daa_sub_half_carry_adjustment() {
        // Half-carry set after SUB → subtract 0x06
        let mut cpu = init_cpu_state();
        cpu.registers.set_a(0x20);
        cpu.registers.f_mut().set_subtraction(true);
        cpu.registers.f_mut().set_half_carry(true);
        exec_daa(&mut cpu);
        assert_eq!(cpu.registers.a(), 0x1A);
        assert!(!cpu.registers.f().is_half_carry());
    }

    #[test]
    fn test_daa_sub_carry_adjustment() {
        // Carry set after SUB → subtract 0x60
        let mut cpu = init_cpu_state();
        cpu.registers.set_a(0x72);
        cpu.registers.f_mut().set_subtraction(true);
        cpu.registers.f_mut().set_carry(true);
        exec_daa(&mut cpu);
        assert_eq!(cpu.registers.a(), 0x12);
        assert!(cpu.registers.f().is_carry()); // carry preserved in sub case
    }

    // -----------------------------------------------------------------------
    // CPL
    // -----------------------------------------------------------------------

    #[test]
    fn test_cpl_all_ones() {
        let mut cpu = init_cpu_state();
        cpu.registers.set_a(0xFF);
        assert_eq!(exec_cpl(&mut cpu), 1);
        assert_eq!(cpu.registers.a(), 0x00);
        assert!(cpu.registers.f().is_subtraction());
        assert!(cpu.registers.f().is_half_carry());
    }

    #[test]
    fn test_cpl_all_zeros() {
        let mut cpu = init_cpu_state();
        cpu.registers.set_a(0x00);
        exec_cpl(&mut cpu);
        assert_eq!(cpu.registers.a(), 0xFF);
        assert!(cpu.registers.f().is_subtraction());
        assert!(cpu.registers.f().is_half_carry());
    }

    #[test]
    fn test_cpl_preserves_zero_and_carry() {
        let mut cpu = init_cpu_state();
        cpu.registers.set_a(0xAA);
        cpu.registers.f_mut().set_zero(true);
        cpu.registers.f_mut().set_carry(true);
        exec_cpl(&mut cpu);
        assert_eq!(cpu.registers.a(), 0x55);
        assert!(cpu.registers.f().is_zero());  // preserved
        assert!(cpu.registers.f().is_carry()); // preserved
    }

    // -----------------------------------------------------------------------
    // SCF / CCF
    // -----------------------------------------------------------------------

    #[test]
    fn test_scf() {
        let mut cpu = init_cpu_state();
        cpu.registers.f_mut().set_carry(false);
        assert_eq!(exec_scf(&mut cpu), 1);
        assert!(cpu.registers.f().is_carry());
        assert!(!cpu.registers.f().is_subtraction());
        assert!(!cpu.registers.f().is_half_carry());
    }

    #[test]
    fn test_scf_already_set() {
        // SCF sets carry unconditionally
        let mut cpu = init_cpu_state();
        cpu.registers.f_mut().set_carry(true);
        exec_scf(&mut cpu);
        assert!(cpu.registers.f().is_carry());
    }

    #[test]
    fn test_ccf_flip_to_set() {
        let mut cpu = init_cpu_state();
        cpu.registers.f_mut().set_carry(false);
        assert_eq!(exec_ccf(&mut cpu), 1);
        assert!(cpu.registers.f().is_carry());
        assert!(!cpu.registers.f().is_subtraction());
        assert!(!cpu.registers.f().is_half_carry());
    }

    #[test]
    fn test_ccf_flip_to_clear() {
        let mut cpu = init_cpu_state();
        cpu.registers.f_mut().set_carry(true);
        exec_ccf(&mut cpu);
        assert!(!cpu.registers.f().is_carry());
    }
}
