/// Data transfer instruction executors
use crate::memory::MemoryBus;
use crate::cpu::CPUState;
use crate::cpu::instructions::R16Register;
use crate::cpu::exec::register_utils::{set_r16, get_r8, set_r8};

/// Execute LD r16, d16
pub fn exec_ld_r16_imm16(cpu_state: &mut CPUState, dest: R16Register, value: u16) -> u32 {
    set_r16(&mut cpu_state.registers, dest, value);
    3
}

/// Execute LD (r16), A
pub fn exec_ld_ind_r16_a(cpu_state: &mut CPUState, src: crate::cpu::instructions::R16Mem, bus: &mut MemoryBus) -> u32 {
    let addr = match src {
        crate::cpu::instructions::R16Mem::BC => cpu_state.registers.bc,
        crate::cpu::instructions::R16Mem::DE => cpu_state.registers.de,
        crate::cpu::instructions::R16Mem::HLPlus => {
            let addr = cpu_state.registers.hl;
            cpu_state.registers.hl = cpu_state.registers.hl.wrapping_add(1);
            addr
        }
        crate::cpu::instructions::R16Mem::HLMinus => {
            let addr = cpu_state.registers.hl;
            cpu_state.registers.hl = cpu_state.registers.hl.wrapping_sub(1);
            addr
        }
    };
    bus.write(addr, cpu_state.registers.a());
    2
}

/// Execute LD A, (r16)
pub fn exec_ld_a_ind_r16(cpu_state: &mut CPUState, dest: crate::cpu::instructions::R16Mem, bus: &mut MemoryBus) -> u32 {
    let addr = match dest {
        crate::cpu::instructions::R16Mem::BC => cpu_state.registers.bc,
        crate::cpu::instructions::R16Mem::DE => cpu_state.registers.de,
        crate::cpu::instructions::R16Mem::HLPlus => {
            let addr = cpu_state.registers.hl;
            cpu_state.registers.hl = cpu_state.registers.hl.wrapping_add(1);
            addr
        }
        crate::cpu::instructions::R16Mem::HLMinus => {
            let addr = cpu_state.registers.hl;
            cpu_state.registers.hl = cpu_state.registers.hl.wrapping_sub(1);
            addr
        }
    };
    cpu_state.registers.set_a(bus.read(addr));
    2
}

/// Execute LD (a16), SP
pub fn exec_ld_ind_imm16_sp(cpu_state: &mut CPUState, address: u16, bus: &mut MemoryBus) -> u32 {
    let sp = cpu_state.registers.sp;
    let lo = (sp & 0xFF) as u8;
    let hi = (sp >> 8) as u8;

    bus.write(address, lo);                // low byte first
    bus.write(address.wrapping_add(1), hi); // high byte second
    5
}

/// Execute LD (a16), A
pub fn exec_ld_ind_imm16_a(cpu_state: &mut CPUState, address: u16, bus: &mut MemoryBus) -> u32 {
    bus.write(address, cpu_state.registers.a());
    4
}

/// Execute LD A, (a16)
pub fn exec_ld_a_ind_imm16(cpu_state: &mut CPUState, address: u16, bus: &mut MemoryBus) -> u32 {
    cpu_state.registers.set_a(bus.read(address));
    4
}

/// Execute LD r8, d8
pub fn exec_ld_r8_imm8(cpu_state: &mut CPUState, bus: &mut MemoryBus, dest: crate::cpu::instructions::R8Register, value: u8) -> u32 {
    set_r8(&mut cpu_state.registers, bus, dest, value);
    2
}

/// Execute LD r8, r8
pub fn exec_ld_r8_r8(cpu_state: &mut CPUState, bus: &mut MemoryBus, dest: crate::cpu::instructions::R8Register, src: crate::cpu::instructions::R8Register) -> u32 {
    let val = get_r8(&cpu_state.registers, bus, src);
    set_r8(&mut cpu_state.registers, bus, dest, val);
    1
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::MemoryBus;
    use crate::cpu::instructions::{R8Register, R16Register, R16Mem};

    fn init_cpu_state() -> CPUState {
        let mut cpu = CPUState::new();
        cpu.registers.f_mut().set_zero(false);
        cpu.registers.f_mut().set_subtraction(false);
        cpu.registers.f_mut().set_half_carry(false);
        cpu.registers.f_mut().set_carry(false);
        cpu
    }

    #[test]
    fn test_ld_r16_imm16() {
        let mut cpu = init_cpu_state();
        let _bus = MemoryBus::new(vec![0; 32768]);

        let cycles = exec_ld_r16_imm16(&mut cpu, R16Register::BC, 0x1234);

        assert_eq!(cycles, 3);
        assert_eq!(cpu.registers.bc, 0x1234);
    }

    #[test]
    fn test_ld_r16_imm16_all_registers() {
        let mut cpu = init_cpu_state();
        let _bus = MemoryBus::new(vec![0; 32768]);

        exec_ld_r16_imm16(&mut cpu, R16Register::DE, 0x5678);
        assert_eq!(cpu.registers.de, 0x5678);

        exec_ld_r16_imm16(&mut cpu, R16Register::HL, 0x9ABC);
        assert_eq!(cpu.registers.hl, 0x9ABC);

        exec_ld_r16_imm16(&mut cpu, R16Register::SP, 0xFFFE);
        assert_eq!(cpu.registers.sp, 0xFFFE);
    }

    #[test]
    fn test_ld_ind_r16_a_bc() {
        let mut cpu = init_cpu_state();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.registers.bc = 0xC000;
        cpu.registers.set_a(0xAB);

        let cycles = exec_ld_ind_r16_a(&mut cpu, R16Mem::BC, &mut bus);

        assert_eq!(cycles, 2);
        assert_eq!(bus.read(0xC000), 0xAB);
    }

    #[test]
    fn test_ld_ind_r16_a_de() {
        let mut cpu = init_cpu_state();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.registers.de = 0xC000;
        cpu.registers.set_a(0xCD);

        let cycles = exec_ld_ind_r16_a(&mut cpu, R16Mem::DE, &mut bus);

        assert_eq!(cycles, 2);
        assert_eq!(bus.read(0xC000), 0xCD);
    }

    #[test]
    fn test_ld_ind_r16_a_hl_plus() {
        let mut cpu = init_cpu_state();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.registers.hl = 0xC000;
        cpu.registers.set_a(0xEF);

        let cycles = exec_ld_ind_r16_a(&mut cpu, R16Mem::HLPlus, &mut bus);

        assert_eq!(cycles, 2);
        assert_eq!(bus.read(0xC000), 0xEF);
        assert_eq!(cpu.registers.hl, 0xC001); // HL should be incremented
    }

    #[test]
    fn test_ld_ind_r16_a_hl_minus() {
        let mut cpu = init_cpu_state();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.registers.hl = 0xC000;
        cpu.registers.set_a(0x12);

        let cycles = exec_ld_ind_r16_a(&mut cpu, R16Mem::HLMinus, &mut bus);

        assert_eq!(cycles, 2);
        assert_eq!(bus.read(0xBFFF), 0x12);
        assert_eq!(cpu.registers.hl, 0xBFFF); // HL should be decremented
    }

    #[test]
    fn test_ld_a_ind_r16_bc() {
        let mut cpu = init_cpu_state();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.registers.bc = 0xC000;
        bus.write(0xC000, 0xAB);

        let cycles = exec_ld_a_ind_r16(&mut cpu, R16Mem::BC, &mut bus);

        assert_eq!(cycles, 2);
        assert_eq!(cpu.registers.a(), 0xAB);
    }

    #[test]
    fn test_ld_a_ind_r16_de() {
        let mut cpu = init_cpu_state();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.registers.de = 0xC000;
        bus.write(0xC000, 0xCD);

        let cycles = exec_ld_a_ind_r16(&mut cpu, R16Mem::DE, &mut bus);

        assert_eq!(cycles, 2);
        assert_eq!(cpu.registers.a(), 0xCD);
    }

    #[test]
    fn test_ld_a_ind_r16_hl_plus() {
        let mut cpu = init_cpu_state();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.registers.hl = 0xC000;
        bus.write(0xC000, 0xEF);

        let cycles = exec_ld_a_ind_r16(&mut cpu, R16Mem::HLPlus, &mut bus);

        assert_eq!(cycles, 2);
        assert_eq!(cpu.registers.a(), 0xEF);
        assert_eq!(cpu.registers.hl, 0xC001);
    }

    #[test]
    fn test_ld_a_ind_r16_hl_minus() {
        let mut cpu = init_cpu_state();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.registers.hl = 0xC000;
        bus.write(0xC000, 0x12);

        let cycles = exec_ld_a_ind_r16(&mut cpu, R16Mem::HLMinus, &mut bus);

        assert_eq!(cycles, 2);
        assert_eq!(cpu.registers.a(), 0x12);
        assert_eq!(cpu.registers.hl, 0xBFFF);
    }

    #[test]
    fn test_ld_ind_imm16_sp() {
        let mut cpu = init_cpu_state();
        let mut bus = MemoryBus::new(vec![0; 0x10000]);
        // ensure MBC is None or disabled for tests
        //bus.mbc = MemoryBankController::none();
        cpu.registers.sp = 0xABCD;

        let cycles = exec_ld_ind_imm16_sp(&mut cpu, 0xC000, &mut bus);

        assert_eq!(cycles, 5);
        // Low byte first, then high byte
        assert_eq!(bus.read(0xC000), 0xCD); // low byte
        assert_eq!(bus.read(0xC001), 0xAB); // high byte
    }

    #[test]
    fn test_ld_ind_imm16_a() {
        let mut cpu = init_cpu_state();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.registers.set_a(0xAB);

        let cycles = exec_ld_ind_imm16_a(&mut cpu, 0xC000, &mut bus);

        assert_eq!(cycles, 4);
        assert_eq!(bus.read(0xC000), 0xAB);
    }

    #[test]
    fn test_ld_a_ind_imm16() {
        let mut cpu = init_cpu_state();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        bus.write(0xC000, 0xAB);

        let cycles = exec_ld_a_ind_imm16(&mut cpu, 0xC000, &mut bus);

        assert_eq!(cycles, 4);
        assert_eq!(cpu.registers.a(), 0xAB);
    }

    #[test]
    fn test_ld_r8_imm8() {
        let mut cpu = init_cpu_state();
        let mut bus = MemoryBus::new(vec![0; 32768]);

        let cycles = exec_ld_r8_imm8(&mut cpu, &mut bus, R8Register::B, 0xAB);

        assert_eq!(cycles, 2);
        assert_eq!(cpu.registers.b(), 0xAB);
    }

    #[test]
    fn test_ld_r8_imm8_all_registers() {
        let mut cpu = init_cpu_state();
        let mut bus = MemoryBus::new(vec![0; 32768]);

        exec_ld_r8_imm8(&mut cpu, &mut bus, R8Register::C, 0xCD);
        assert_eq!(cpu.registers.c(), 0xCD);

        exec_ld_r8_imm8(&mut cpu, &mut bus, R8Register::D, 0xEF);
        assert_eq!(cpu.registers.d(), 0xEF);

        exec_ld_r8_imm8(&mut cpu, &mut bus, R8Register::E, 0x12);
        assert_eq!(cpu.registers.e(), 0x12);

        exec_ld_r8_imm8(&mut cpu, &mut bus, R8Register::H, 0x34);
        assert_eq!(cpu.registers.h(), 0x34);

        exec_ld_r8_imm8(&mut cpu, &mut bus, R8Register::L, 0x56);
        assert_eq!(cpu.registers.l(), 0x56);

        exec_ld_r8_imm8(&mut cpu, &mut bus, R8Register::A, 0x78);
        assert_eq!(cpu.registers.a(), 0x78);
    }

    #[test]
    fn test_ld_r8_r8() {
        let mut cpu = init_cpu_state();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.registers.set_b(0xAB);

        let cycles = exec_ld_r8_r8(&mut cpu, &mut bus, R8Register::C, R8Register::B);

        assert_eq!(cycles, 1);
        assert_eq!(cpu.registers.c(), 0xAB);
    }

    #[test]
    fn test_ld_r8_r8_hl_memory() {
        let mut cpu = init_cpu_state();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.registers.hl = 0xC000;
        bus.write(0xC000, 0x42);
        cpu.registers.set_a(0x00);

        // Load from HL memory into A
        let cycles = exec_ld_r8_r8(&mut cpu, &mut bus, R8Register::A, R8Register::HL);

        assert_eq!(cycles, 1);
        assert_eq!(cpu.registers.a(), 0x42);
    }

    #[test]
    fn test_ld_r8_r8_to_hl_memory() {
        let mut cpu = init_cpu_state();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.registers.hl = 0xC000;
        cpu.registers.set_b(0x42);

        // Load B into HL memory
        let cycles = exec_ld_r8_r8(&mut cpu, &mut bus, R8Register::HL, R8Register::B);

        assert_eq!(cycles, 1);
        assert_eq!(bus.read(0xC000), 0x42);
    }
}
