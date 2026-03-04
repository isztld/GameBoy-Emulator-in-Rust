/// Jump and call instruction executors
use crate::memory::MemoryBus;
use crate::cpu::CPUState;
use crate::cpu::instructions::Condition;

/// Execute JR r8
pub fn exec_jr_imm8(cpu_state: &mut CPUState, offset: i8, io: &mut [u8; 128], tick: &mut dyn FnMut(&mut [u8; 128])) -> u32 {
    cpu_state.registers.pc = cpu_state.registers.pc.wrapping_add(offset as i32 as u16);
    tick(io); // simulated offset read
    tick(io); // internal delay
    3
}

/// Execute JR cc, r8
pub fn exec_jr_cond_imm8(cpu_state: &mut CPUState, cond: Condition, offset: i8, io: &mut [u8; 128], tick: &mut dyn FnMut(&mut [u8; 128])) -> u32 {
    let jump = cond_condition(cpu_state, cond);
    tick(io); // offset read always happens
    if jump {
        cpu_state.registers.pc = cpu_state.registers.pc.wrapping_add(offset as i32 as u16);
        tick(io); // internal delay only if taken
        3
    } else {
        2
    }
}

/// Execute JP cc, a16
pub fn exec_jp_cond_imm16(cpu_state: &mut CPUState, cond: Condition, address: u16, io: &mut [u8; 128], tick: &mut dyn FnMut(&mut [u8; 128])) -> u32 {
    if cond_condition(cpu_state, cond) {
        cpu_state.registers.pc = address;
        tick(io); // addr low read
        tick(io); // addr high read
        tick(io); // internal delay
        4
    } else {
        tick(io); // addr low read
        tick(io); // addr high read
        3
    }
}

/// Execute JP a16
pub fn exec_jp_imm16(cpu_state: &mut CPUState, address: u16, io: &mut [u8; 128], tick: &mut dyn FnMut(&mut [u8; 128])) -> u32 {
    cpu_state.registers.pc = address;
    tick(io); // addr low read
    tick(io); // addr high read
    tick(io); // internal delay
    4
}

/// Execute JP (HL)
pub fn exec_jp_hl(cpu_state: &mut CPUState) -> u32 {
    cpu_state.registers.pc = cpu_state.registers.hl;
    1
}

/// Execute CALL cc, a16
pub fn exec_call_cond_imm16(cpu_state: &mut CPUState, cond: Condition, address: u16, bus: &mut MemoryBus, tick: &mut dyn FnMut(&mut [u8; 128])) -> u32 {
    if cond_condition(cpu_state, cond) {
        tick(&mut bus.io); // simulated addr low read
        tick(&mut bus.io); // simulated addr high read
        let sp = cpu_state.registers.sp;
        // PC has already been advanced past the instruction's operand bytes by the
        // caller, so it holds the correct return address (the byte after CALL).
        let return_pc = cpu_state.registers.pc;
        tick(&mut bus.io); // internal delay before writes
        bus.write(sp.wrapping_sub(1), (return_pc >> 8) as u8);     // high byte
        tick(&mut bus.io);
        bus.write(sp.wrapping_sub(2), (return_pc & 0x00FF) as u8); // low byte
        tick(&mut bus.io);
        cpu_state.registers.sp = sp.wrapping_sub(2);
        cpu_state.registers.pc = address;
        6
    } else {
        tick(&mut bus.io); // simulated addr low read
        tick(&mut bus.io); // simulated addr high read
        3
    }
}

/// Execute CALL a16
pub fn exec_call_imm16(cpu_state: &mut CPUState, address: u16, bus: &mut MemoryBus, tick: &mut dyn FnMut(&mut [u8; 128])) -> u32 {
    tick(&mut bus.io); // simulated addr low read
    tick(&mut bus.io); // simulated addr high read
    let sp = cpu_state.registers.sp;
    // PC has already been advanced past the instruction's operand bytes by the
    // caller, so it holds the correct return address (the byte after CALL).
    let return_pc = cpu_state.registers.pc;
    tick(&mut bus.io); // internal delay before writes
    bus.write(sp.wrapping_sub(1), (return_pc >> 8) as u8);     // high byte
    tick(&mut bus.io);
    bus.write(sp.wrapping_sub(2), (return_pc & 0x00FF) as u8); // low byte
    tick(&mut bus.io);
    cpu_state.registers.sp = sp.wrapping_sub(2);
    cpu_state.registers.pc = address;
    6
}

/// Execute RET cc
pub fn exec_ret_cond(cpu_state: &mut CPUState, cond: Condition, bus: &mut MemoryBus, tick: &mut dyn FnMut(&mut [u8; 128])) -> u32 {
    tick(&mut bus.io); // condition check (M-cycle 2) — paid by both taken and not-taken
    if cond_condition(cpu_state, cond) {
        let sp = cpu_state.registers.sp;
        let low = bus.read(sp);
        tick(&mut bus.io); // read SP low
        let high = bus.read(sp.wrapping_add(1));
        tick(&mut bus.io); // read SP high
        cpu_state.registers.sp = sp.wrapping_add(2);
        cpu_state.registers.pc = ((high as u16) << 8) | (low as u16);
        tick(&mut bus.io); // internal delay
        5
    } else {
        2
    }
}

/// Condition check helper
fn cond_condition(cpu_state: &CPUState, cond: Condition) -> bool {
    match cond {
        Condition::NZ => !cpu_state.registers.f().is_zero(),
        Condition::Z => cpu_state.registers.f().is_zero(),
        Condition::NC => !cpu_state.registers.f().is_carry(),
        Condition::C => cpu_state.registers.f().is_carry(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::MemoryBus;
    use crate::cpu::instructions::Condition;

    fn init_cpu_state() -> CPUState {
        let mut cpu = CPUState::new();
        cpu.registers.f_mut().set_zero(false);
        cpu.registers.f_mut().set_subtraction(false);
        cpu.registers.f_mut().set_half_carry(false);
        cpu.registers.f_mut().set_carry(false);
        cpu
    }

    fn noop_tick(_: &mut [u8; 128]) {}

    #[test]
    fn test_jr_imm8_positive() {
        let mut cpu = init_cpu_state();
        cpu.registers.pc = 0x1000;

        let cycles = exec_jr_imm8(&mut cpu, 5, &mut [0u8; 128], &mut noop_tick);

        assert_eq!(cycles, 3);
        assert_eq!(cpu.registers.pc, 0x1005);
    }

    #[test]
    fn test_jr_imm8_negative() {
        let mut cpu = init_cpu_state();
        cpu.registers.pc = 0x1000;

        let cycles = exec_jr_imm8(&mut cpu, -5, &mut [0u8; 128], &mut noop_tick);

        assert_eq!(cycles, 3);
        assert_eq!(cpu.registers.pc, 0x0FFB);
    }

    #[test]
    fn test_jr_imm8_wrap() {
        let mut cpu = init_cpu_state();
        cpu.registers.pc = 0x1000;

        let cycles = exec_jr_imm8(&mut cpu, 127, &mut [0u8; 128], &mut noop_tick);

        assert_eq!(cycles, 3);
        assert_eq!(cpu.registers.pc, 0x107F);
    }

    #[test]
    fn test_jr_cond_jump_taken() {
        let mut cpu = init_cpu_state();
        cpu.registers.pc = 0x1000;
        cpu.registers.f_mut().set_zero(false);

        let cycles = exec_jr_cond_imm8(&mut cpu, Condition::NZ, 5, &mut [0u8; 128], &mut noop_tick);

        assert_eq!(cycles, 3);
        assert_eq!(cpu.registers.pc, 0x1005);
    }

    #[test]
    fn test_jr_cond_jump_not_taken() {
        let mut cpu = init_cpu_state();
        cpu.registers.pc = 0x1000;
        cpu.registers.f_mut().set_zero(true);

        let cycles = exec_jr_cond_imm8(&mut cpu, Condition::NZ, 5, &mut [0u8; 128], &mut noop_tick);

        assert_eq!(cycles, 2);
        assert_eq!(cpu.registers.pc, 0x1000);
    }

    #[test]
    fn test_jp_cond_imm16_taken() {
        let mut cpu = init_cpu_state();
        cpu.registers.f_mut().set_zero(false);

        let cycles = exec_jp_cond_imm16(&mut cpu, Condition::NZ, 0x8000, &mut [0u8; 128], &mut noop_tick);

        assert_eq!(cycles, 4);
        assert_eq!(cpu.registers.pc, 0x8000);
    }

    #[test]
    fn test_jp_cond_imm16_not_taken() {
        let mut cpu = init_cpu_state();
        cpu.registers.f_mut().set_zero(true);

        let cycles = exec_jp_cond_imm16(&mut cpu, Condition::NZ, 0x8000, &mut [0u8; 128], &mut noop_tick);

        assert_eq!(cycles, 3);
        assert_eq!(cpu.registers.pc, 0x0000);
    }

    #[test]
    fn test_jp_imm16() {
        let mut cpu = init_cpu_state();

        let cycles = exec_jp_imm16(&mut cpu, 0x8000, &mut [0u8; 128], &mut noop_tick);

        assert_eq!(cycles, 4);
        assert_eq!(cpu.registers.pc, 0x8000);
    }

    #[test]
    fn test_jp_hl() {
        let mut cpu = init_cpu_state();
        cpu.registers.hl = 0x8000;

        let cycles = exec_jp_hl(&mut cpu);

        assert_eq!(cycles, 1);
        assert_eq!(cpu.registers.pc, 0x8000);
    }

    #[test]
    fn test_call_cond_imm16_taken() {
        let mut cpu = init_cpu_state();
        cpu.registers.sp = 0xFFFE;
        cpu.registers.pc = 0x1003;
        cpu.registers.f_mut().set_zero(false);
        let mut bus = MemoryBus::new(vec![0; 32768]);

        let cycles = exec_call_cond_imm16(&mut cpu, Condition::NZ, 0x8000, &mut bus, &mut noop_tick);

        assert_eq!(cycles, 6);
        assert_eq!(cpu.registers.pc, 0x8000);
        // Return address should be 0x1003 (PC + 3 after CALL)
        assert_eq!(bus.read(0xFFFD), 0x10); // high byte
        assert_eq!(bus.read(0xFFFC), 0x03); // low byte
        assert_eq!(cpu.registers.sp, 0xFFFC);
    }

    #[test]
    fn test_call_cond_imm16_not_taken() {
        let mut cpu = init_cpu_state();
        cpu.registers.sp = 0xFFFE;
        cpu.registers.pc = 0x1000;
        cpu.registers.f_mut().set_zero(true);
        let mut bus = MemoryBus::new(vec![0; 32768]);

        let cycles = exec_call_cond_imm16(&mut cpu, Condition::NZ, 0x8000, &mut bus, &mut noop_tick);

        assert_eq!(cycles, 3);
        assert_eq!(cpu.registers.pc, 0x1000);
        assert_eq!(cpu.registers.sp, 0xFFFE);
    }

    #[test]
    fn test_call_imm16() {
        let mut cpu = init_cpu_state();
        cpu.registers.sp = 0xFFFE;
        cpu.registers.pc = 0x1003;
        let mut bus = MemoryBus::new(vec![0; 32768]);

        let cycles = exec_call_imm16(&mut cpu, 0x8000, &mut bus, &mut noop_tick);

        assert_eq!(cycles, 6);
        assert_eq!(cpu.registers.pc, 0x8000);
        assert_eq!(bus.read(0xFFFD), 0x10);
        assert_eq!(bus.read(0xFFFC), 0x03);
        assert_eq!(cpu.registers.sp, 0xFFFC);
    }

    #[test]
    fn test_ret_cond_taken() {
        let mut cpu = init_cpu_state();
        cpu.registers.sp = 0xFFFC;
        cpu.registers.pc = 0x0000;
        let mut bus = MemoryBus::new(vec![0; 32768]);
        bus.write(0xFFFC, 0x00); // low byte
        bus.write(0xFFFD, 0x80); // high byte
        cpu.registers.f_mut().set_zero(false);

        let cycles = exec_ret_cond(&mut cpu, Condition::NZ, &mut bus, &mut noop_tick);

        assert_eq!(cycles, 5);
        assert_eq!(cpu.registers.pc, 0x8000);
        assert_eq!(cpu.registers.sp, 0xFFFE);
    }

    #[test]
    fn test_ret_cond_not_taken() {
        let mut cpu = init_cpu_state();
        cpu.registers.sp = 0xFFFC;
        cpu.registers.pc = 0x0000;
        let mut bus = MemoryBus::new(vec![0; 32768]);
        bus.write(0xFFFC, 0x00); // low byte
        bus.write(0xFFFD, 0x80); // high byte
        cpu.registers.f_mut().set_zero(true);

        let cycles = exec_ret_cond(&mut cpu, Condition::NZ, &mut bus, &mut noop_tick);

        assert_eq!(cycles, 2);
        assert_eq!(cpu.registers.pc, 0x0000);
        assert_eq!(cpu.registers.sp, 0xFFFC);
    }

    #[test]
    fn test_cond_nz_true() {
        let mut cpu = init_cpu_state();
        cpu.registers.f_mut().set_zero(false);

        assert!(cond_condition(&cpu, Condition::NZ));
    }

    #[test]
    fn test_cond_nz_false() {
        let mut cpu = init_cpu_state();
        cpu.registers.f_mut().set_zero(true);

        assert!(!cond_condition(&cpu, Condition::NZ));
    }

    #[test]
    fn test_cond_z_true() {
        let mut cpu = init_cpu_state();
        cpu.registers.f_mut().set_zero(true);

        assert!(cond_condition(&cpu, Condition::Z));
    }

    #[test]
    fn test_cond_z_false() {
        let mut cpu = init_cpu_state();
        cpu.registers.f_mut().set_zero(false);

        assert!(!cond_condition(&cpu, Condition::Z));
    }

    #[test]
    fn test_cond_nc_true() {
        let mut cpu = init_cpu_state();
        cpu.registers.f_mut().set_carry(false);

        assert!(cond_condition(&cpu, Condition::NC));
    }

    #[test]
    fn test_cond_nc_false() {
        let mut cpu = init_cpu_state();
        cpu.registers.f_mut().set_carry(true);

        assert!(!cond_condition(&cpu, Condition::NC));
    }

    #[test]
    fn test_cond_c_true() {
        let mut cpu = init_cpu_state();
        cpu.registers.f_mut().set_carry(true);

        assert!(cond_condition(&cpu, Condition::C));
    }

    #[test]
    fn test_cond_c_false() {
        let mut cpu = init_cpu_state();
        cpu.registers.f_mut().set_carry(false);

        assert!(!cond_condition(&cpu, Condition::C));
    }
}
