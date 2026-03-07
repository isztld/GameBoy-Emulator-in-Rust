/// Rotate/shift instruction executors
use crate::cpu::CPUState;

/// Execute RLCA
pub fn exec_rlca(cpu_state: &mut CPUState) -> u32 {
    let a = cpu_state.registers.a();
    let new_a = a.rotate_left(1);
    cpu_state.registers.set_a(new_a);
    cpu_state.registers.modify_f(|f| f.set_carry((a & 0x80) != 0));
    cpu_state.registers.modify_f(|f| f.set_zero(false));
    cpu_state.registers.modify_f(|f| f.set_subtraction(false));
    cpu_state.registers.modify_f(|f| f.set_half_carry(false));
    1
}

/// Execute RRCA
pub fn exec_rrca(cpu_state: &mut CPUState) -> u32 {
    let a = cpu_state.registers.a();
    let new_a = a.rotate_right(1);
    cpu_state.registers.set_a(new_a);
    cpu_state.registers.modify_f(|f| f.set_carry((a & 0x01) != 0));
    cpu_state.registers.modify_f(|f| f.set_zero(false));
    cpu_state.registers.modify_f(|f| f.set_subtraction(false));
    cpu_state.registers.modify_f(|f| f.set_half_carry(false));
    1
}

/// Execute RLA
pub fn exec_rla(cpu_state: &mut CPUState) -> u32 {
    let a = cpu_state.registers.a();
    let old_c = cpu_state.registers.f().is_carry() as u8;
    let new_a = (a << 1) | old_c;
    cpu_state.registers.set_a(new_a);
    cpu_state.registers.modify_f(|f| f.set_carry((a & 0x80) != 0));
    cpu_state.registers.modify_f(|f| f.set_zero(false));
    cpu_state.registers.modify_f(|f| f.set_subtraction(false));
    cpu_state.registers.modify_f(|f| f.set_half_carry(false));
    1
}

/// Execute RRA
pub fn exec_rra(cpu_state: &mut CPUState) -> u32 {
    let a = cpu_state.registers.a();
    let old_c = cpu_state.registers.f().is_carry() as u8;
    let new_a = (a >> 1) | (old_c << 7);
    cpu_state.registers.set_a(new_a);
    cpu_state.registers.modify_f(|f| f.set_carry((a & 0x01) != 0));
    cpu_state.registers.modify_f(|f| f.set_zero(false));
    cpu_state.registers.modify_f(|f| f.set_subtraction(false));
    cpu_state.registers.modify_f(|f| f.set_half_carry(false));
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
    cpu_state.registers.modify_f(|f| f.set_zero(new_a == 0));
    cpu_state.registers.modify_f(|f| f.set_half_carry(false));
    cpu_state.registers.modify_f(|f| f.set_carry(new_carry));
    1
}

/// Execute CPL
pub fn exec_cpl(cpu_state: &mut CPUState) -> u32 {
    let a = cpu_state.registers.a();
    cpu_state.registers.set_a(!a);
    cpu_state.registers.modify_f(|f| f.set_subtraction(true));
    cpu_state.registers.modify_f(|f| f.set_half_carry(true));
    1
}

/// Execute SCF
pub fn exec_scf(cpu_state: &mut CPUState) -> u32 {
    cpu_state.registers.modify_f(|f| f.set_carry(true));
    cpu_state.registers.modify_f(|f| f.set_subtraction(false));
    cpu_state.registers.modify_f(|f| f.set_half_carry(false));
    1
}

/// Execute CCF
pub fn exec_ccf(cpu_state: &mut CPUState) -> u32 {
    let carry = cpu_state.registers.f().is_carry();
    cpu_state.registers.modify_f(|f| f.set_carry(!carry));
    cpu_state.registers.modify_f(|f| f.set_subtraction(false));
    cpu_state.registers.modify_f(|f| f.set_half_carry(false));
    1
}

#[cfg(test)]
#[path = "rotate_shift_tests.rs"]
mod tests;
