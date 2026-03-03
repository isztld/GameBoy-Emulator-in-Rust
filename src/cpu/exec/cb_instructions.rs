/// CB-prefixed instruction executors
use crate::memory::MemoryBus;
use crate::cpu::CPUState;
use crate::cpu::instructions::CBInstruction;
use crate::cpu::instructions::R8Register;
use crate::cpu::exec::register_utils::{get_r8, set_r8};

/// Execute a CB-prefixed instruction
pub fn execute_cb(cpu_state: &mut CPUState, bus: &mut MemoryBus, cb_instr: CBInstruction) -> u32 {
    match cb_instr {
        CBInstruction::RLCR8 { reg } => exec_rlcr8(cpu_state, bus, reg),
        CBInstruction::RRCR8 { reg } => exec_rrcr8(cpu_state, bus, reg),
        CBInstruction::RLR8 { reg } => exec_rlr8(cpu_state, bus, reg),
        CBInstruction::RRR8 { reg } => exec_rrr8(cpu_state, bus, reg),
        CBInstruction::SLAR8 { reg } => exec_slar8(cpu_state, bus, reg),
        CBInstruction::SRAR8 { reg } => exec_srar8(cpu_state, bus, reg),
        CBInstruction::SWAPR8 { reg } => exec_swapr8(cpu_state, bus, reg),
        CBInstruction::SRLR8 { reg } => exec_srlr8(cpu_state, bus, reg),
        CBInstruction::BITBR8 { bit, reg } => exec_bitbr8(cpu_state, bus, bit, reg),
        CBInstruction::RESBR8 { bit, reg } => exec_resbr8(cpu_state, bus, bit, reg),
        CBInstruction::SETBR8 { bit, reg } => exec_setbr8(cpu_state, bus, bit, reg),
    }
}

/// Execute RLC r8
pub fn exec_rlcr8(cpu_state: &mut CPUState, bus: &mut MemoryBus, reg: R8Register) -> u32 {
    let val = get_r8(&cpu_state.registers, bus, reg);
    let new_val = val.rotate_left(1);
    set_r8(&mut cpu_state.registers, bus, reg, new_val);
    cpu_state.registers.f_mut().set_carry((val & 0x80) != 0);
    cpu_state.registers.f_mut().set_zero(new_val == 0);
    cpu_state.registers.f_mut().set_subtraction(false);
    cpu_state.registers.f_mut().set_half_carry(false);
    if reg == R8Register::HL { 4 } else { 2 }
}

/// Execute RRC r8
pub fn exec_rrcr8(cpu_state: &mut CPUState, bus: &mut MemoryBus, reg: R8Register) -> u32 {
    let val = get_r8(&cpu_state.registers, bus, reg);
    let new_val = val.rotate_right(1);
    set_r8(&mut cpu_state.registers, bus, reg, new_val);
    cpu_state.registers.f_mut().set_carry((val & 0x01) != 0);
    cpu_state.registers.f_mut().set_zero(new_val == 0);
    cpu_state.registers.f_mut().set_subtraction(false);
    cpu_state.registers.f_mut().set_half_carry(false);
    if reg == R8Register::HL { 4 } else { 2 }
}

/// Execute RL r8
pub fn exec_rlr8(cpu_state: &mut CPUState, bus: &mut MemoryBus, reg: R8Register) -> u32 {
    let val = get_r8(&cpu_state.registers, bus, reg);
    let old_c = cpu_state.registers.f().is_carry() as u8;
    let new_val = (val << 1) | old_c;
    set_r8(&mut cpu_state.registers, bus, reg, new_val);
    cpu_state.registers.f_mut().set_carry((val & 0x80) != 0);
    cpu_state.registers.f_mut().set_zero(new_val == 0);
    cpu_state.registers.f_mut().set_subtraction(false);
    cpu_state.registers.f_mut().set_half_carry(false);
    if reg == R8Register::HL { 4 } else { 2 }
}

/// Execute RR r8
pub fn exec_rrr8(cpu_state: &mut CPUState, bus: &mut MemoryBus, reg: R8Register) -> u32 {
    let val = get_r8(&cpu_state.registers, bus, reg);
    let old_c = cpu_state.registers.f().is_carry() as u8;
    let new_val = (val >> 1) | (old_c << 7);
    set_r8(&mut cpu_state.registers, bus, reg, new_val);
    cpu_state.registers.f_mut().set_carry((val & 0x01) != 0);
    cpu_state.registers.f_mut().set_zero(new_val == 0);
    cpu_state.registers.f_mut().set_subtraction(false);
    cpu_state.registers.f_mut().set_half_carry(false);
    if reg == R8Register::HL { 4 } else { 2 }
}

/// Execute SLA r8
pub fn exec_slar8(cpu_state: &mut CPUState, bus: &mut MemoryBus, reg: R8Register) -> u32 {
    let val = get_r8(&cpu_state.registers, bus, reg);
    let new_val = val << 1;
    set_r8(&mut cpu_state.registers, bus, reg, new_val);
    cpu_state.registers.f_mut().set_carry((val & 0x80) != 0);
    cpu_state.registers.f_mut().set_zero(new_val == 0);
    cpu_state.registers.f_mut().set_subtraction(false);
    cpu_state.registers.f_mut().set_half_carry(false);
    if reg == R8Register::HL { 4 } else { 2 }
}

/// Execute SRA r8
pub fn exec_srar8(cpu_state: &mut CPUState, bus: &mut MemoryBus, reg: R8Register) -> u32 {
    let val = get_r8(&cpu_state.registers, bus, reg);
    let new_val = (val as i8 >> 1) as u8;
    set_r8(&mut cpu_state.registers, bus, reg, new_val);
    cpu_state.registers.f_mut().set_carry((val & 0x01) != 0);
    cpu_state.registers.f_mut().set_zero(new_val == 0);
    cpu_state.registers.f_mut().set_subtraction(false);
    cpu_state.registers.f_mut().set_half_carry(false);
    if reg == R8Register::HL { 4 } else { 2 }
}

/// Execute SWAP r8
pub fn exec_swapr8(cpu_state: &mut CPUState, bus: &mut MemoryBus, reg: R8Register) -> u32 {
    let val = get_r8(&cpu_state.registers, bus, reg);
    let new_val = (val >> 4) | (val << 4);
    set_r8(&mut cpu_state.registers, bus, reg, new_val);
    cpu_state.registers.f_mut().set_zero(new_val == 0);
    cpu_state.registers.f_mut().set_subtraction(false);
    cpu_state.registers.f_mut().set_half_carry(false);
    cpu_state.registers.f_mut().set_carry(false);
    if reg == R8Register::HL { 4 } else { 2 }
}

/// Execute SRL r8
pub fn exec_srlr8(cpu_state: &mut CPUState, bus: &mut MemoryBus, reg: R8Register) -> u32 {
    let val = get_r8(&cpu_state.registers, bus, reg);
    let new_val = val >> 1;
    set_r8(&mut cpu_state.registers, bus, reg, new_val);
    cpu_state.registers.f_mut().set_carry((val & 0x01) != 0);
    cpu_state.registers.f_mut().set_zero(new_val == 0);
    cpu_state.registers.f_mut().set_subtraction(false);
    cpu_state.registers.f_mut().set_half_carry(false);
    if reg == R8Register::HL { 4 } else { 2 }
}

/// Execute BIT b, r8
pub fn exec_bitbr8(cpu_state: &mut CPUState, _bus: &mut MemoryBus, bit: u8, reg: R8Register) -> u32 {
    let val = get_r8(&cpu_state.registers, _bus, reg);
    cpu_state.registers.f_mut().set_zero(((val >> bit) & 1) == 0);
    cpu_state.registers.f_mut().set_subtraction(false);
    cpu_state.registers.f_mut().set_half_carry(true);
    if reg == R8Register::HL { 3 } else { 2 }
}

/// Execute RES b, r8
pub fn exec_resbr8(cpu_state: &mut CPUState, bus: &mut MemoryBus, bit: u8, reg: R8Register) -> u32 {
    let mut val = get_r8(&cpu_state.registers, bus, reg);
    val &= !(1 << bit);
    set_r8(&mut cpu_state.registers, bus, reg, val);
    if reg == R8Register::HL { 4 } else { 2 }
}

/// Execute SET b, r8
pub fn exec_setbr8(cpu_state: &mut CPUState, bus: &mut MemoryBus, bit: u8, reg: R8Register) -> u32 {
    let mut val = get_r8(&cpu_state.registers, bus, reg);
    val |= 1 << bit;
    set_r8(&mut cpu_state.registers, bus, reg, val);
    if reg == R8Register::HL { 4 } else { 2 }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::MemoryBus;
    use crate::cpu::instructions::R8Register;

    fn init_cpu_state() -> CPUState {
        let mut cpu = CPUState::new();
        cpu.registers.f_mut().set_zero(false);
        cpu.registers.f_mut().set_subtraction(false);
        cpu.registers.f_mut().set_half_carry(false);
        cpu.registers.f_mut().set_carry(false);
        cpu
    }

    #[test]
    fn test_rlcr8() {
        let mut cpu = init_cpu_state();
        cpu.registers.set_b(0b11001111);
        let mut bus = MemoryBus::new(vec![0; 32768]);

        let cycles = exec_rlcr8(&mut cpu, &mut bus, R8Register::B);

        assert_eq!(cycles, 2);
        // Rotate left: 0b11001111 -> 0b10011111
        assert_eq!(cpu.registers.b(), 0b10011111);
        // Old bit 7 (1) went to carry
        assert!(cpu.registers.f().is_carry());
        assert!(!cpu.registers.f().is_zero());
    }

    #[test]
    fn test_rlcr8_zero() {
        let mut cpu = init_cpu_state();
        cpu.registers.set_b(0b00000001);
        let mut bus = MemoryBus::new(vec![0; 32768]);

        let cycles = exec_rlcr8(&mut cpu, &mut bus, R8Register::B);

        assert_eq!(cycles, 2);
        // Rotate left: 0b00000001 -> 0b00000010
        assert_eq!(cpu.registers.b(), 0b00000010);
        // Old bit 7 (0) went to carry
        assert!(!cpu.registers.f().is_carry());
    }

    #[test]
    fn test_rlcr8_zero_result() {
        let mut cpu = init_cpu_state();
        cpu.registers.set_b(0b00000000);
        let mut bus = MemoryBus::new(vec![0; 32768]);

        let cycles = exec_rlcr8(&mut cpu, &mut bus, R8Register::B);

        assert_eq!(cycles, 2);
        // Result is still 0
        assert!(cpu.registers.f().is_zero());
    }

    #[test]
    fn test_rrcr8() {
        let mut cpu = init_cpu_state();
        cpu.registers.set_b(0b11001111);
        let mut bus = MemoryBus::new(vec![0; 32768]);

        let cycles = exec_rrcr8(&mut cpu, &mut bus, R8Register::B);

        assert_eq!(cycles, 2);
        // Rotate right: 0b11001111 -> 0b11100111
        assert_eq!(cpu.registers.b(), 0b11100111);
        // Old bit 0 (1) went to carry
        assert!(cpu.registers.f().is_carry());
    }

    #[test]
    fn test_rlr8() {
        let mut cpu = init_cpu_state();
        cpu.registers.set_b(0b11001111);
        cpu.registers.f_mut().set_carry(false);
        let mut bus = MemoryBus::new(vec![0; 32768]);

        let cycles = exec_rlr8(&mut cpu, &mut bus, R8Register::B);

        assert_eq!(cycles, 2);
        // Shift left with carry in: 0b11001111 << 1 = 0b10011110, carry out = 1
        assert_eq!(cpu.registers.b(), 0b10011110);
        assert!(cpu.registers.f().is_carry());
    }

    #[test]
    fn test_rlr8_with_carry() {
        let mut cpu = init_cpu_state();
        cpu.registers.set_b(0b01001111);
        cpu.registers.f_mut().set_carry(true);
        let mut bus = MemoryBus::new(vec![0; 32768]);

        let cycles = exec_rlr8(&mut cpu, &mut bus, R8Register::B);

        assert_eq!(cycles, 2);
        // Shift left with carry in: 0b01001111 << 1 = 0b10011110, OR carry = 0b10011111
        assert_eq!(cpu.registers.b(), 0b10011111);
        assert!(!cpu.registers.f().is_carry());
    }

    #[test]
    fn test_rrr8() {
        let mut cpu = init_cpu_state();
        cpu.registers.set_b(0b11001111);
        cpu.registers.f_mut().set_carry(false);
        let mut bus = MemoryBus::new(vec![0; 32768]);

        let cycles = exec_rrr8(&mut cpu, &mut bus, R8Register::B);

        assert_eq!(cycles, 2);
        // Shift right with carry in: 0b11001111 >> 1 = 0b01100111, carry out = 1
        assert_eq!(cpu.registers.b(), 0b01100111);
        assert!(cpu.registers.f().is_carry());
    }

    #[test]
    fn test_rrr8_with_carry() {
        let mut cpu = init_cpu_state();
        cpu.registers.set_b(0b01001111);
        cpu.registers.f_mut().set_carry(true);
        let mut bus = MemoryBus::new(vec![0; 32768]);

        let cycles = exec_rrr8(&mut cpu, &mut bus, R8Register::B);

        assert_eq!(cycles, 2);
        // Shift right with carry in: 0b01001111 >> 1 = 0b00100111, OR (carry << 7) = 0b10100111
        assert_eq!(cpu.registers.b(), 0b10100111);
        assert!(cpu.registers.f().is_carry());
    }

    #[test]
    fn test_slar8() {
        let mut cpu = init_cpu_state();
        cpu.registers.set_b(0b11001111);
        let mut bus = MemoryBus::new(vec![0; 32768]);

        let cycles = exec_slar8(&mut cpu, &mut bus, R8Register::B);

        assert_eq!(cycles, 2);
        // Shift left: 0b11001111 << 1 = 0b10011110, carry out = 1
        assert_eq!(cpu.registers.b(), 0b10011110);
        assert!(cpu.registers.f().is_carry());
    }

    #[test]
    fn test_srar8() {
        let mut cpu = init_cpu_state();
        cpu.registers.set_b(0b11001111);
        let mut bus = MemoryBus::new(vec![0; 32768]);

        let cycles = exec_srar8(&mut cpu, &mut bus, R8Register::B);

        assert_eq!(cycles, 2);
        // Arithmetic shift right (sign-extended): 0b11001111 >> 1 = 0b11100111
        assert_eq!(cpu.registers.b(), 0b11100111);
        assert!(cpu.registers.f().is_carry()); // Old bit 0
    }

    #[test]
    fn test_srar8_positive() {
        let mut cpu = init_cpu_state();
        cpu.registers.set_b(0b01001111);
        let mut bus = MemoryBus::new(vec![0; 32768]);

        let cycles = exec_srar8(&mut cpu, &mut bus, R8Register::B);

        assert_eq!(cycles, 2);
        // Arithmetic shift right: 0b01001111 >> 1 = 0b00100111
        assert_eq!(cpu.registers.b(), 0b00100111);
    }

    #[test]
    fn test_swapr8() {
        let mut cpu = init_cpu_state();
        cpu.registers.set_b(0x5A);
        let mut bus = MemoryBus::new(vec![0; 32768]);

        let cycles = exec_swapr8(&mut cpu, &mut bus, R8Register::B);

        assert_eq!(cycles, 2);
        // Swap nibbles: 0x5A -> 0xA5
        assert_eq!(cpu.registers.b(), 0xA5);
        assert!(!cpu.registers.f().is_zero());
        assert!(!cpu.registers.f().is_half_carry());
        assert!(!cpu.registers.f().is_carry());
    }

    #[test]
    fn test_swapr8_zero() {
        let mut cpu = init_cpu_state();
        cpu.registers.set_b(0x00);
        let mut bus = MemoryBus::new(vec![0; 32768]);

        let cycles = exec_swapr8(&mut cpu, &mut bus, R8Register::B);

        assert_eq!(cycles, 2);
        assert!(cpu.registers.f().is_zero());
    }

    #[test]
    fn test_srlr8() {
        let mut cpu = init_cpu_state();
        cpu.registers.set_b(0b11001111);
        let mut bus = MemoryBus::new(vec![0; 32768]);

        let cycles = exec_srlr8(&mut cpu, &mut bus, R8Register::B);

        assert_eq!(cycles, 2);
        // Logical shift right: 0b11001111 >> 1 = 0b01100111
        assert_eq!(cpu.registers.b(), 0b01100111);
        assert!(cpu.registers.f().is_carry()); // Old bit 0
    }

    #[test]
    fn test_bitbr8_bit_set() {
        let mut cpu = init_cpu_state();
        cpu.registers.set_b(0b10101010);
        let mut bus = MemoryBus::new(vec![0; 32768]);

        let cycles = exec_bitbr8(&mut cpu, &mut bus, 1, R8Register::B);

        assert_eq!(cycles, 2);
        // Bit 1 is 1
        assert!(!cpu.registers.f().is_zero());
        assert!(!cpu.registers.f().is_subtraction());
        assert!(cpu.registers.f().is_half_carry());
    }

    #[test]
    fn test_bitbr8_bit_clear() {
        let mut cpu = init_cpu_state();
        cpu.registers.set_b(0b10101010);
        let mut bus = MemoryBus::new(vec![0; 32768]);

        let cycles = exec_bitbr8(&mut cpu, &mut bus, 0, R8Register::B);

        assert_eq!(cycles, 2);
        // Bit 0 is 0
        assert!(cpu.registers.f().is_zero());
    }

    #[test]
    fn test_bitbr8_all_bits() {
        let mut cpu = init_cpu_state();
        cpu.registers.set_b(0xFF);
        let mut bus = MemoryBus::new(vec![0; 32768]);

        for bit in 0..8 {
            let cycles = exec_bitbr8(&mut cpu, &mut bus, bit, R8Register::B);
            assert_eq!(cycles, 2);
            assert!(!cpu.registers.f().is_zero(), "Bit {} should be set", bit);
        }
    }

    #[test]
    fn test_bitbr8_preserves_carry() {
        let mut cpu = init_cpu_state();
        cpu.registers.set_b(0b00000001);

        // Explicitly set carry before BIT
        cpu.registers.f_mut().set_carry(true);

        let mut bus = MemoryBus::new(vec![0; 32768]);

        let cycles = exec_bitbr8(&mut cpu, &mut bus, 0, R8Register::B);

        assert_eq!(cycles, 2);

        // Bit 0 is set → Z = 0
        assert!(!cpu.registers.f().is_zero());

        // BIT must:
        // - Clear N
        // - Set H
        // - Leave C unchanged
        assert!(!cpu.registers.f().is_subtraction());
        assert!(cpu.registers.f().is_half_carry());

        assert!(cpu.registers.f().is_carry());
    }

    #[test]
    fn test_resbr8() {
        let mut cpu = init_cpu_state();
        cpu.registers.set_b(0b11111111);
        let mut bus = MemoryBus::new(vec![0; 32768]);

        let cycles = exec_resbr8(&mut cpu, &mut bus, 3, R8Register::B);

        assert_eq!(cycles, 2);
        // Clear bit 3: 0b11111111 -> 0b11110111
        assert_eq!(cpu.registers.b(), 0b11110111);
    }

    #[test]
    fn test_resbr8_multiple_bits() {
        let mut cpu = init_cpu_state();
        cpu.registers.set_b(0b11111111);
        let mut bus = MemoryBus::new(vec![0; 32768]);

        exec_resbr8(&mut cpu, &mut bus, 0, R8Register::B);
        exec_resbr8(&mut cpu, &mut bus, 2, R8Register::B);
        exec_resbr8(&mut cpu, &mut bus, 4, R8Register::B);
        exec_resbr8(&mut cpu, &mut bus, 6, R8Register::B);

        assert_eq!(cpu.registers.b(), 0b10101010);
    }

    #[test]
    fn test_setbr8() {
        let mut cpu = init_cpu_state();
        cpu.registers.set_b(0b00000000);
        let mut bus = MemoryBus::new(vec![0; 32768]);

        let cycles = exec_setbr8(&mut cpu, &mut bus, 3, R8Register::B);

        assert_eq!(cycles, 2);
        // Set bit 3: 0b00000000 -> 0b00001000
        assert_eq!(cpu.registers.b(), 0b00001000);
    }

    #[test]
    fn test_setbr8_multiple_bits() {
        let mut cpu = init_cpu_state();
        cpu.registers.set_b(0b00000000);
        let mut bus = MemoryBus::new(vec![0; 32768]);

        exec_setbr8(&mut cpu, &mut bus, 0, R8Register::B);
        exec_setbr8(&mut cpu, &mut bus, 2, R8Register::B);
        exec_setbr8(&mut cpu, &mut bus, 4, R8Register::B);
        exec_setbr8(&mut cpu, &mut bus, 6, R8Register::B);

        // Set bits 0, 2, 4, 6: 0b00000000 -> 0b01010101
        assert_eq!(cpu.registers.b(), 0b01010101);
    }
}
