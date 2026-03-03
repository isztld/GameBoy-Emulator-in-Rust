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
    // Store low byte at the lower address (sp‑2) and high byte at the higher address (sp‑1)
    bus.write(sp.wrapping_sub(2), (value & 0x00FF) as u8);   // low byte at sp‑2
    bus.write(sp.wrapping_sub(1), (value >> 8) as u8);       // high byte at sp‑1
    cpu_state.registers.sp = sp.wrapping_sub(2);
    4
}

/// Execute RST n
pub fn exec_rst(cpu_state: &mut CPUState, target: u8, bus: &mut MemoryBus) -> u32 {
    let sp = cpu_state.registers.sp;
    let return_pc = cpu_state.registers.pc; // PC already pre-advanced by CPU::execute
    bus.write(sp.wrapping_sub(1), (return_pc >> 8) as u8);
    bus.write(sp.wrapping_sub(2), (return_pc & 0xFF) as u8);
    cpu_state.registers.sp = sp.wrapping_sub(2);
    cpu_state.registers.pc = target as u16;
    4
}


/// Execute ADD SP, r8
pub fn exec_add_sp_imm8(cpu_state: &mut CPUState, value: i8) -> u32 {
    let sp = cpu_state.registers.sp;
    // Flags are computed on the unsigned byte addition — the offset's raw
    // bit pattern is treated as an unsigned byte for H and C flag purposes.
    let rhs_byte = value as u8 as u16;
    let result = sp.wrapping_add(value as i16 as u16);
    cpu_state.registers.sp = result;
    cpu_state.registers.f_mut().set_zero(false);
    cpu_state.registers.f_mut().set_subtraction(false);
    cpu_state.registers.f_mut().set_half_carry((sp & 0x0F) + (rhs_byte & 0x0F) > 0x0F);
    cpu_state.registers.f_mut().set_carry((sp & 0xFF) + (rhs_byte & 0xFF) > 0xFF);
    4
}

/// Execute LD (HL), SP
pub fn exec_ld_hl_sp_imm8(cpu_state: &mut CPUState, value: i8) -> u32 {
    let sp = cpu_state.registers.sp;
    let rhs_byte = value as u8 as u16;
    let result = sp.wrapping_add(value as i16 as u16);
    cpu_state.registers.hl = result;
    cpu_state.registers.f_mut().set_zero(false);
    cpu_state.registers.f_mut().set_subtraction(false);
    cpu_state.registers.f_mut().set_half_carry((sp & 0x0F) + (rhs_byte & 0x0F) > 0x0F);
    cpu_state.registers.f_mut().set_carry((sp & 0xFF) + (rhs_byte & 0xFF) > 0xFF);
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
/// IME is not enabled immediately — it takes effect after the following instruction.
pub fn exec_ei(cpu_state: &mut CPUState) -> u32 {
    cpu_state.ime_pending = true;
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
        bus.write(0xFFFC, 0x34); // low byte (F — lower 4 bits must read back as 0)
        bus.write(0xFFFD, 0x12); // high byte (A)

        let cycles = exec_pop_r16(&mut cpu, R16Register::AF, &mut bus);

        assert_eq!(cycles, 3);
        assert_eq!(cpu.registers.af, 0x1230); // lower nibble of F always 0
    }

    #[test]
    fn test_push_r16() {
        let mut cpu = init_cpu_state();
        cpu.registers.sp = 0xFFFE;
        cpu.registers.bc = 0x1234;
        let mut bus = MemoryBus::new(vec![0; 32768]);

        let cycles = exec_push_r16(&mut cpu, R16Register::BC, &mut bus);

        assert_eq!(cycles, 4);
        assert_eq!(cpu.registers.sp, 0xFFFC);
        assert_eq!(bus.read(0xFFFD), 0x12); // high byte at sp-1
        assert_eq!(bus.read(0xFFFC), 0x34); // low byte at sp-2
    }

    #[test]
    fn test_push_pop_roundtrip() {
        // PUSH followed by POP must restore the original value.
        let mut cpu = init_cpu_state();
        cpu.registers.sp = 0xFFFE;
        cpu.registers.bc = 0xABCD;
        let mut bus = MemoryBus::new(vec![0; 32768]);

        exec_push_r16(&mut cpu, R16Register::BC, &mut bus);
        cpu.registers.bc = 0x0000;
        exec_pop_r16(&mut cpu, R16Register::BC, &mut bus);

        assert_eq!(cpu.registers.bc, 0xABCD);
        assert_eq!(cpu.registers.sp, 0xFFFE);
    }

    #[test]
    fn test_rst() {
        let mut cpu_state = CPUState::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        // Simulate CPU::execute pre-advancing PC past the 1-byte RST opcode.
        cpu_state.registers.pc = 0x1001;
        cpu_state.registers.sp = 0xC000;

        exec_rst(&mut cpu_state, 0x08, &mut bus);

        assert_eq!(cpu_state.registers.pc, 0x0008);
        assert_eq!(cpu_state.registers.sp, 0xBFFE);
        assert_eq!(bus.read(0xBFFF), 0x10); // return address high byte
        assert_eq!(bus.read(0xBFFE), 0x01); // return address low byte
    }

    #[test]
    fn test_rst_ret_roundtrip() {
        let mut cpu_state = CPUState::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        // Pre-advanced PC: RST at 0x1000, next instruction at 0x1001.
        cpu_state.registers.pc = 0x1001;
        cpu_state.registers.sp = 0xC000;

        exec_rst(&mut cpu_state, 0x08, &mut bus);
        assert_eq!(cpu_state.registers.pc, 0x0008);

        exec_ret(&mut cpu_state, &mut bus);
        assert_eq!(cpu_state.registers.pc, 0x1001); // back to instruction after RST
        assert_eq!(cpu_state.registers.sp, 0xC000);
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
        assert!(!cpu.registers.f().is_half_carry());
        assert!(!cpu.registers.f().is_carry());
    }

    #[test]
    fn test_add_sp_imm8_negative() {
        // SP=0xFF1F, offset=-1 (rhs_byte=0xFF)
        // half-carry: (0x0F + 0x0F) = 0x1E > 0x0F → set
        // carry:      (0x1F + 0xFF) = 0x11E > 0xFF → set
        let mut cpu = init_cpu_state();
        cpu.registers.sp = 0xFF1F;

        let cycles = exec_add_sp_imm8(&mut cpu, -1);

        assert_eq!(cycles, 4);
        assert_eq!(cpu.registers.sp, 0xFF1E);
        assert!(!cpu.registers.f().is_zero());
        assert!(!cpu.registers.f().is_subtraction());
        assert!(cpu.registers.f().is_half_carry());
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
        assert!(!cpu.registers.f().is_carry());
    }

    #[test]
    fn test_add_sp_imm8_carry() {
        let mut cpu = init_cpu_state();
        cpu.registers.sp = 0x00FF;

        let cycles = exec_add_sp_imm8(&mut cpu, 1);

        assert_eq!(cycles, 4);
        assert_eq!(cpu.registers.sp, 0x0100);
        assert!(cpu.registers.f().is_carry());
        assert!(cpu.registers.f().is_half_carry());
    }

    #[test]
    fn test_add_sp_imm8_no_carry_for_clean_negative() {
        // SP=0xFF00, offset=-16 (rhs_byte=0xF0)
        // half-carry: (0x00 + 0x00) = 0x00, not > 0x0F → clear
        // carry:      (0x00 + 0xF0) = 0xF0, not > 0xFF → clear
        let mut cpu = init_cpu_state();
        cpu.registers.sp = 0xFF00;

        exec_add_sp_imm8(&mut cpu, -16);

        assert_eq!(cpu.registers.sp, 0xFEF0);
        assert!(!cpu.registers.f().is_carry());
        assert!(!cpu.registers.f().is_half_carry());
    }

    #[test]
    fn test_ld_hl_sp_imm8_positive() {
        let mut cpu = init_cpu_state();
        cpu.registers.sp = 0xFF00;

        let cycles = exec_ld_hl_sp_imm8(&mut cpu, 10);

        assert_eq!(cycles, 3);
        assert_eq!(cpu.registers.hl, 0xFF0A);
        assert_eq!(cpu.registers.sp, 0xFF00); // SP must not change
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
        assert_eq!(cpu.registers.sp, 0xFF00); // SP must not change
    }

    #[test]
    fn test_ld_sp_hl() {
        let mut cpu = init_cpu_state();
        cpu.registers.hl = 0x8000;

        let cycles = exec_ld_sp_hl(&mut cpu);

        assert_eq!(cycles, 2);
        assert_eq!(cpu.registers.sp, 0x8000);
        assert_eq!(cpu.registers.hl, 0x8000); // HL must not change
    }

    #[test]
    fn test_di() {
        let mut cpu = init_cpu_state();
        cpu.ime = true;
        assert_eq!(exec_di(&mut cpu), 1);
        assert!(!cpu.ime);
    }

    #[test]
    fn test_ei() {
        let mut cpu = init_cpu_state();
        cpu.ime = false;
        assert_eq!(exec_ei(&mut cpu), 1);
        // EI does not enable IME immediately; it sets ime_pending for a 1-instruction delay.
        assert!(!cpu.ime);
        assert!(cpu.ime_pending);
    }

    #[test]
    fn test_ldh_ind_c_a() {
        let mut cpu = init_cpu_state();
        cpu.registers.set_a(0x42);
        cpu.registers.set_c(0x10);
        let mut bus = MemoryBus::new(vec![0; 32768]);

        let cycles = exec_ldh_ind_c_a(&mut cpu, &mut bus);

        assert_eq!(cycles, 2);
        assert_eq!(bus.read(0xFF10), 0x42);
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
        cpu.registers.set_a(0x77);
        let mut bus = MemoryBus::new(vec![0; 32768]);

        let cycles = exec_ldh_ind_imm8_a(&mut cpu, 0x10, &mut bus);

        assert_eq!(cycles, 3);
        assert_eq!(bus.read(0xFF10), 0x77);
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
