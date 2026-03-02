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
    // RST is 1 byte, so return address is PC + 1 (next instruction after RST)
    // Stack grows down, so we write low byte first (at sp-1), then high byte (at sp-2)
    let return_pc = cpu_state.registers.pc + 1;
    bus.write(sp.wrapping_sub(1), (return_pc & 0x00FF) as u8); // low byte
    bus.write(sp.wrapping_sub(2), (return_pc >> 8) as u8);     // high byte
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
        cpu.ime = false;
        cpu
    }

    #[test]
    fn test_ret() {
        let mut cpu = init_cpu_state();
        cpu.registers.sp = 0xFFFC;
        let mut bus = MemoryBus::new(vec![0; 32768]);
        bus.write(0xFFFC, 0x00); // low byte
        bus.write(0xFFFD, 0x80); // high byte

        let cycles = exec_ret(&mut cpu, &mut bus);

        assert_eq!(cycles, 4);
        assert_eq!(cpu.registers.pc, 0x8000);
        assert_eq!(cpu.registers.sp, 0xFFFE);
    }

    #[test]
    fn test_reti() {
        let mut cpu = init_cpu_state();
        cpu.ime = false;
        cpu.registers.sp = 0xFFFC;
        let mut bus = MemoryBus::new(vec![0; 32768]);
        bus.write(0xFFFC, 0x00); // low byte
        bus.write(0xFFFD, 0x80); // high byte

        let cycles = exec_reti(&mut cpu, &mut bus);

        assert_eq!(cycles, 4);
        assert_eq!(cpu.registers.pc, 0x8000);
        assert_eq!(cpu.registers.sp, 0xFFFE);
        assert!(cpu.ime);
    }

    #[test]
    fn test_pop_r16() {
        let mut cpu = init_cpu_state();
        cpu.registers.sp = 0xFFFC;
        let mut bus = MemoryBus::new(vec![0; 32768]);
        bus.write(0xFFFC, 0x34); // low byte
        bus.write(0xFFFD, 0x12); // high byte

        let cycles = exec_pop_r16(&mut cpu, R16Register::BC, &mut bus);

        assert_eq!(cycles, 3);
        assert_eq!(cpu.registers.bc, 0x1234);
        assert_eq!(cpu.registers.sp, 0xFFFE);
    }

    #[test]
    fn test_pop_r16_af() {
        let mut cpu = init_cpu_state();
        cpu.registers.sp = 0xFFFC;
        let mut bus = MemoryBus::new(vec![0; 32768]);
        bus.write(0xFFFC, 0x34); // low byte
        bus.write(0xFFFD, 0x12); // high byte

        let cycles = exec_pop_r16(&mut cpu, R16Register::AF, &mut bus);

        assert_eq!(cycles, 3);
        // AF should have lower 4 bits of F cleared
        assert_eq!(cpu.registers.af, 0x1230);
    }

    #[test]
    fn test_push_r16() {
        let mut cpu = init_cpu_state();
        cpu.registers.sp = 0xFFFE;
        cpu.registers.bc = 0x1234;
        let mut bus = MemoryBus::new(vec![0; 32768]);

        let cycles = exec_push_r16(&mut cpu, R16Register::BC, &mut bus);

        assert_eq!(cycles, 5);
        assert_eq!(bus.read(0xFFFD), 0x34); // low byte
        assert_eq!(bus.read(0xFFFC), 0x12); // high byte
        assert_eq!(cpu.registers.sp, 0xFFFC);
    }

    #[test]
    fn test_rst() {
        let mut cpu = init_cpu_state();
        cpu.registers.sp = 0xFFFE;
        cpu.registers.pc = 0x1000;
        let mut bus = MemoryBus::new(vec![0; 32768]);

        let cycles = exec_rst(&mut cpu, 0x08, &mut bus);

        assert_eq!(cycles, 4);
        assert_eq!(cpu.registers.pc, 0x0008);
        // Return address should be 0x1001 (PC + 1 after RST)
        assert_eq!(bus.read(0xFFFD), 0x01);
        assert_eq!(bus.read(0xFFFC), 0x10);
        assert_eq!(cpu.registers.sp, 0xFFFC);
    }

    #[test]
    fn test_add_sp_imm8_positive() {
        let mut cpu = init_cpu_state();
        cpu.registers.sp = 0xFF00;

        let cycles = exec_add_sp_imm8(&mut cpu, 10);

        assert_eq!(cycles, 4);
        assert_eq!(cpu.registers.sp, 0xFF0A);
        assert!(!cpu.registers.f().is_zero());
        assert!(!cpu.registers.f().is_subtraction());
        assert!(!cpu.registers.f().is_carry());
    }

    #[test]
    fn test_add_sp_imm8_negative() {
        let mut cpu = init_cpu_state();
        cpu.registers.sp = 0xFF00;

        let cycles = exec_add_sp_imm8(&mut cpu, -10);

        assert_eq!(cycles, 4);
        assert_eq!(cpu.registers.sp, 0xFEF6);
        assert!(!cpu.registers.f().is_zero());
        assert!(!cpu.registers.f().is_subtraction());
        // Should have carry for negative offset
        assert!(cpu.registers.f().is_carry());
    }

    #[test]
    fn test_add_sp_imm8_half_carry() {
        let mut cpu = init_cpu_state();
        cpu.registers.sp = 0x000F;

        let cycles = exec_add_sp_imm8(&mut cpu, 1);

        assert_eq!(cycles, 4);
        assert_eq!(cpu.registers.sp, 0x0010);
        assert!(cpu.registers.f().is_half_carry());
    }

    #[test]
    fn test_add_sp_imm8_carry() {
        let mut cpu = init_cpu_state();
        cpu.registers.sp = 0x00FF;

        let cycles = exec_add_sp_imm8(&mut cpu, 1);

        assert_eq!(cycles, 4);
        assert_eq!(cpu.registers.sp, 0x0100);
        assert!(cpu.registers.f().is_carry());
    }

    #[test]
    fn test_ld_hl_sp_imm8_positive() {
        let mut cpu = init_cpu_state();
        cpu.registers.sp = 0xFF00;

        let cycles = exec_ld_hl_sp_imm8(&mut cpu, 10);

        assert_eq!(cycles, 3);
        assert_eq!(cpu.registers.hl, 0xFF0A);
        assert!(!cpu.registers.f().is_zero());
        assert!(!cpu.registers.f().is_subtraction());
    }

    #[test]
    fn test_ld_hl_sp_imm8_negative() {
        let mut cpu = init_cpu_state();
        cpu.registers.sp = 0xFF00;

        let cycles = exec_ld_hl_sp_imm8(&mut cpu, -10);

        assert_eq!(cycles, 3);
        assert_eq!(cpu.registers.hl, 0xFEF6);
    }

    #[test]
    fn test_ld_sp_hl() {
        let mut cpu = init_cpu_state();
        cpu.registers.hl = 0x8000;

        let cycles = exec_ld_sp_hl(&mut cpu);

        assert_eq!(cycles, 2);
        assert_eq!(cpu.registers.sp, 0x8000);
    }

    #[test]
    fn test_di() {
        let mut cpu = init_cpu_state();
        cpu.ime = true;

        let cycles = exec_di(&mut cpu);

        assert_eq!(cycles, 1);
        assert!(!cpu.ime);
    }

    #[test]
    fn test_ei() {
        let mut cpu = init_cpu_state();
        cpu.ime = false;

        let cycles = exec_ei(&mut cpu);

        assert_eq!(cycles, 1);
        assert!(cpu.ime);
    }

    #[test]
    fn test_ldh_ind_c_a() {
        let mut cpu = init_cpu_state();
        cpu.registers.set_c(0x10);
        let mut bus = MemoryBus::new(vec![0; 32768]);

        let cycles = exec_ldh_ind_c_a(&mut cpu, &mut bus);

        assert_eq!(cycles, 2);
        assert_eq!(bus.read(0xFF10), cpu.registers.a());
    }

    #[test]
    fn test_ldh_a_c() {
        let mut cpu = init_cpu_state();
        cpu.registers.set_c(0x10);
        let mut bus = MemoryBus::new(vec![0; 32768]);
        bus.write(0xFF10, 0xAB);

        let cycles = exec_ldh_a_c(&mut cpu, &mut bus);

        assert_eq!(cycles, 2);
        assert_eq!(cpu.registers.a(), 0xAB);
    }

    #[test]
    fn test_ldh_ind_imm8_a() {
        let mut cpu = init_cpu_state();
        let mut bus = MemoryBus::new(vec![0; 32768]);

        let cycles = exec_ldh_ind_imm8_a(&mut cpu, 0x10, &mut bus);

        assert_eq!(cycles, 3);
        assert_eq!(bus.read(0xFF10), cpu.registers.a());
    }

    #[test]
    fn test_ldh_a_ind_imm8() {
        let mut cpu = init_cpu_state();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        bus.write(0xFF10, 0xAB);

        let cycles = exec_ldh_a_ind_imm8(&mut cpu, 0x10, &mut bus);

        assert_eq!(cycles, 3);
        assert_eq!(cpu.registers.a(), 0xAB);
    }
}
