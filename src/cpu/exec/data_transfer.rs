/// Data transfer instruction executors
use crate::memory::MemoryBus;
use crate::cpu::CPUState;
use crate::cpu::instructions::R16Register;
use crate::cpu::exec::register_utils::{set_r16, get_r8, set_r8};

/// Execute LD r16, d16
pub fn exec_ld_r16_imm16(cpu_state: &mut CPUState, dest: R16Register, value: u16, io: &mut [u8; 128], tick: &mut dyn FnMut(&mut [u8; 128])) -> u32 {
    set_r16(&mut cpu_state.registers, dest, value);
    tick(io);
    tick(io);
    3
}

/// Execute LD (r16), A
pub fn exec_ld_ind_r16_a(cpu_state: &mut CPUState, src: crate::cpu::instructions::R16Mem, bus: &mut MemoryBus, tick: &mut dyn FnMut(&mut [u8; 128])) -> u32 {
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
    tick(&mut bus.io);
    2
}

/// Execute LD A, (r16)
pub fn exec_ld_a_ind_r16(cpu_state: &mut CPUState, dest: crate::cpu::instructions::R16Mem, bus: &mut MemoryBus, tick: &mut dyn FnMut(&mut [u8; 128])) -> u32 {
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
    tick(&mut bus.io);
    2
}

/// Execute LD (a16), SP
pub fn exec_ld_ind_imm16_sp(cpu_state: &mut CPUState, address: u16, bus: &mut MemoryBus, tick: &mut dyn FnMut(&mut [u8; 128])) -> u32 {
    let sp = cpu_state.registers.sp;
    let lo = (sp & 0xFF) as u8;
    let hi = (sp >> 8) as u8;

    tick(&mut bus.io);
    tick(&mut bus.io);
    bus.write(address, lo);                // low byte first
    tick(&mut bus.io);
    bus.write(address.wrapping_add(1), hi); // high byte second
    tick(&mut bus.io);
    5
}

/// Execute LD (a16), A
pub fn exec_ld_ind_imm16_a(cpu_state: &mut CPUState, address: u16, bus: &mut MemoryBus, tick: &mut dyn FnMut(&mut [u8; 128])) -> u32 {
    tick(&mut bus.io);
    tick(&mut bus.io);
    bus.write(address, cpu_state.registers.a());
    tick(&mut bus.io);
    4
}

/// Execute LD A, (a16)
pub fn exec_ld_a_ind_imm16(cpu_state: &mut CPUState, address: u16, bus: &mut MemoryBus, tick: &mut dyn FnMut(&mut [u8; 128])) -> u32 {
    tick(&mut bus.io);
    tick(&mut bus.io);
    cpu_state.registers.set_a(bus.read(address));
    tick(&mut bus.io);
    4
}

/// Execute LD r8, d8
pub fn exec_ld_r8_imm8(
    cpu_state: &mut CPUState,
    bus: &mut MemoryBus,
    dest: crate::cpu::instructions::R8Register,
    value: u8,
    tick: &mut dyn FnMut(&mut [u8; 128]),
) -> u32 {
    match dest {
        crate::cpu::instructions::R8Register::HL => {
            tick(&mut bus.io); // simulated imm read
            set_r8(&mut cpu_state.registers, bus, dest, value);
            tick(&mut bus.io); // write to (HL)
            3
        }
        _ => {
            tick(&mut bus.io); // simulated imm read
            set_r8(&mut cpu_state.registers, bus, dest, value);
            2
        }
    }
}

/// Execute LD r8, r8
pub fn exec_ld_r8_r8(
    cpu_state: &mut CPUState,
    bus: &mut MemoryBus,
    dest: crate::cpu::instructions::R8Register,
    src: crate::cpu::instructions::R8Register,
    tick: &mut dyn FnMut(&mut [u8; 128]),
) -> u32 {
    let val = get_r8(&cpu_state.registers, bus, src);
    if src == crate::cpu::instructions::R8Register::HL {
        tick(&mut bus.io);
    }
    set_r8(&mut cpu_state.registers, bus, dest, val);
    if dest == crate::cpu::instructions::R8Register::HL {
        tick(&mut bus.io);
    }
    match (dest, src) {
        // LD (HL), r8 or LD r8, (HL) — 2 machine cycles
        (crate::cpu::instructions::R8Register::HL, _)
        | (_, crate::cpu::instructions::R8Register::HL) => 2,
        // LD r8, r8 — 1 machine cycle
        _ => 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::MemoryBus;
    use crate::cpu::instructions::{R8Register, R16Register, R16Mem};

    fn make_bus() -> MemoryBus {
        MemoryBus::new(vec![0u8; 32768])
    }

    fn init_cpu_state() -> CPUState {
        let mut cpu = CPUState::new();
        cpu.registers.f_mut().set_zero(false);
        cpu.registers.f_mut().set_subtraction(false);
        cpu.registers.f_mut().set_half_carry(false);
        cpu.registers.f_mut().set_carry(false);
        cpu
    }

    fn noop_tick(_: &mut [u8; 128]) {}

    // -----------------------------------------------------------------------
    // LD r16, d16
    // -----------------------------------------------------------------------

    #[test]
    fn test_ld_r16_imm16() {
        let mut cpu = init_cpu_state();
        assert_eq!(exec_ld_r16_imm16(&mut cpu, R16Register::BC, 0x1234, &mut [0u8; 128], &mut noop_tick), 3);
        assert_eq!(cpu.registers.bc, 0x1234);
    }

    #[test]
    fn test_ld_r16_imm16_all_registers() {
        let mut cpu = init_cpu_state();
        exec_ld_r16_imm16(&mut cpu, R16Register::DE, 0x5678, &mut [0u8; 128], &mut noop_tick);
        assert_eq!(cpu.registers.de, 0x5678);
        exec_ld_r16_imm16(&mut cpu, R16Register::HL, 0x9ABC, &mut [0u8; 128], &mut noop_tick);
        assert_eq!(cpu.registers.hl, 0x9ABC);
        exec_ld_r16_imm16(&mut cpu, R16Register::SP, 0xFFFE, &mut [0u8; 128], &mut noop_tick);
        assert_eq!(cpu.registers.sp, 0xFFFE);
    }

    // -----------------------------------------------------------------------
    // LD (r16), A
    // -----------------------------------------------------------------------

    #[test]
    fn test_ld_ind_r16_a_bc() {
        let mut cpu = init_cpu_state();
        let mut bus = make_bus();
        cpu.registers.bc = 0xC000;
        cpu.registers.set_a(0xAB);
        assert_eq!(exec_ld_ind_r16_a(&mut cpu, R16Mem::BC, &mut bus, &mut noop_tick), 2);
        assert_eq!(bus.read(0xC000), 0xAB);
    }

    #[test]
    fn test_ld_ind_r16_a_de() {
        let mut cpu = init_cpu_state();
        let mut bus = make_bus();
        cpu.registers.de = 0xC000;
        cpu.registers.set_a(0xCD);
        assert_eq!(exec_ld_ind_r16_a(&mut cpu, R16Mem::DE, &mut bus, &mut noop_tick), 2);
        assert_eq!(bus.read(0xC000), 0xCD);
    }

    #[test]
    fn test_ld_ind_r16_a_hl_plus() {
        let mut cpu = init_cpu_state();
        let mut bus = make_bus();
        cpu.registers.hl = 0xC000;
        cpu.registers.set_a(0xEF);
        assert_eq!(exec_ld_ind_r16_a(&mut cpu, R16Mem::HLPlus, &mut bus, &mut noop_tick), 2);
        assert_eq!(bus.read(0xC000), 0xEF); // written before increment
        assert_eq!(cpu.registers.hl, 0xC001);
    }

    #[test]
    fn test_ld_ind_r16_a_hl_minus() {
        let mut cpu = init_cpu_state();
        let mut bus = make_bus();
        cpu.registers.hl = 0xC000;
        cpu.registers.set_a(0x12);
        assert_eq!(exec_ld_ind_r16_a(&mut cpu, R16Mem::HLMinus, &mut bus, &mut noop_tick), 2);
        assert_eq!(bus.read(0xC000), 0x12); // written at original HL before decrement
        assert_eq!(cpu.registers.hl, 0xBFFF);
    }

    // -----------------------------------------------------------------------
    // LD A, (r16)
    // -----------------------------------------------------------------------

    #[test]
    fn test_ld_a_ind_r16_bc() {
        let mut cpu = init_cpu_state();
        let mut bus = make_bus();
        cpu.registers.bc = 0xC000;
        bus.write(0xC000, 0xAB);
        assert_eq!(exec_ld_a_ind_r16(&mut cpu, R16Mem::BC, &mut bus, &mut noop_tick), 2);
        assert_eq!(cpu.registers.a(), 0xAB);
    }

    #[test]
    fn test_ld_a_ind_r16_de() {
        let mut cpu = init_cpu_state();
        let mut bus = make_bus();
        cpu.registers.de = 0xC000;
        bus.write(0xC000, 0xCD);
        assert_eq!(exec_ld_a_ind_r16(&mut cpu, R16Mem::DE, &mut bus, &mut noop_tick), 2);
        assert_eq!(cpu.registers.a(), 0xCD);
    }

    #[test]
    fn test_ld_a_ind_r16_hl_plus() {
        let mut cpu = init_cpu_state();
        let mut bus = make_bus();
        cpu.registers.hl = 0xC000;
        bus.write(0xC000, 0xEF);
        assert_eq!(exec_ld_a_ind_r16(&mut cpu, R16Mem::HLPlus, &mut bus, &mut noop_tick), 2);
        assert_eq!(cpu.registers.a(), 0xEF);
        assert_eq!(cpu.registers.hl, 0xC001);
    }

    #[test]
    fn test_ld_a_ind_r16_hl_minus() {
        let mut cpu = init_cpu_state();
        let mut bus = make_bus();
        cpu.registers.hl = 0xC000;
        bus.write(0xC000, 0x12);
        assert_eq!(exec_ld_a_ind_r16(&mut cpu, R16Mem::HLMinus, &mut bus, &mut noop_tick), 2);
        assert_eq!(cpu.registers.a(), 0x12);
        assert_eq!(cpu.registers.hl, 0xBFFF);
    }

    // -----------------------------------------------------------------------
    // LD (a16), SP
    // -----------------------------------------------------------------------

    #[test]
    fn test_ld_ind_imm16_sp() {
        let mut cpu = init_cpu_state();
        let mut bus = make_bus();
        cpu.registers.sp = 0xABCD;
        assert_eq!(exec_ld_ind_imm16_sp(&mut cpu, 0xC000, &mut bus, &mut noop_tick), 5);
        assert_eq!(bus.read(0xC000), 0xCD); // low byte
        assert_eq!(bus.read(0xC001), 0xAB); // high byte
    }

    // -----------------------------------------------------------------------
    // LD (a16), A  /  LD A, (a16)
    // -----------------------------------------------------------------------

    #[test]
    fn test_ld_ind_imm16_a() {
        let mut cpu = init_cpu_state();
        let mut bus = make_bus();
        cpu.registers.set_a(0xAB);
        assert_eq!(exec_ld_ind_imm16_a(&mut cpu, 0xC000, &mut bus, &mut noop_tick), 4);
        assert_eq!(bus.read(0xC000), 0xAB);
    }

    #[test]
    fn test_ld_a_ind_imm16() {
        let mut cpu = init_cpu_state();
        let mut bus = make_bus();
        bus.write(0xC000, 0xAB);
        assert_eq!(exec_ld_a_ind_imm16(&mut cpu, 0xC000, &mut bus, &mut noop_tick), 4);
        assert_eq!(cpu.registers.a(), 0xAB);
    }

    // -----------------------------------------------------------------------
    // LD r8, d8
    // -----------------------------------------------------------------------

    #[test]
    fn test_ld_r8_imm8_register() {
        let mut cpu = init_cpu_state();
        let mut bus = make_bus();
        assert_eq!(exec_ld_r8_imm8(&mut cpu, &mut bus, R8Register::B, 0xAB, &mut noop_tick), 2);
        assert_eq!(cpu.registers.b(), 0xAB);
    }

    #[test]
    fn test_ld_r8_imm8_hl_indirect() {
        // LD (HL), n8 takes 3 machine cycles.
        let mut cpu = init_cpu_state();
        let mut bus = make_bus();
        cpu.registers.hl = 0xC000;
        assert_eq!(exec_ld_r8_imm8(&mut cpu, &mut bus, R8Register::HL, 0x55, &mut noop_tick), 3);
        assert_eq!(bus.read(0xC000), 0x55);
    }

    #[test]
    fn test_ld_r8_imm8_all_registers() {
        let mut cpu = init_cpu_state();
        let mut bus = make_bus();
        exec_ld_r8_imm8(&mut cpu, &mut bus, R8Register::C, 0xCD, &mut noop_tick);
        assert_eq!(cpu.registers.c(), 0xCD);
        exec_ld_r8_imm8(&mut cpu, &mut bus, R8Register::D, 0xEF, &mut noop_tick);
        assert_eq!(cpu.registers.d(), 0xEF);
        exec_ld_r8_imm8(&mut cpu, &mut bus, R8Register::E, 0x12, &mut noop_tick);
        assert_eq!(cpu.registers.e(), 0x12);
        exec_ld_r8_imm8(&mut cpu, &mut bus, R8Register::H, 0x34, &mut noop_tick);
        assert_eq!(cpu.registers.h(), 0x34);
        exec_ld_r8_imm8(&mut cpu, &mut bus, R8Register::L, 0x56, &mut noop_tick);
        assert_eq!(cpu.registers.l(), 0x56);
        exec_ld_r8_imm8(&mut cpu, &mut bus, R8Register::A, 0x78, &mut noop_tick);
        assert_eq!(cpu.registers.a(), 0x78);
    }

    // -----------------------------------------------------------------------
    // LD r8, r8
    // -----------------------------------------------------------------------

    #[test]
    fn test_ld_r8_r8_register_to_register() {
        let mut cpu = init_cpu_state();
        let mut bus = make_bus();
        cpu.registers.set_b(0xAB);
        assert_eq!(exec_ld_r8_r8(&mut cpu, &mut bus, R8Register::C, R8Register::B, &mut noop_tick), 1);
        assert_eq!(cpu.registers.c(), 0xAB);
        assert_eq!(cpu.registers.b(), 0xAB); // source unchanged
    }

    #[test]
    fn test_ld_r8_r8_from_hl_indirect() {
        // LD r8, (HL) — 2 machine cycles
        let mut cpu = init_cpu_state();
        let mut bus = make_bus();
        cpu.registers.hl = 0xC000;
        bus.write(0xC000, 0x42);
        assert_eq!(exec_ld_r8_r8(&mut cpu, &mut bus, R8Register::A, R8Register::HL, &mut noop_tick), 2);
        assert_eq!(cpu.registers.a(), 0x42);
    }

    #[test]
    fn test_ld_r8_r8_to_hl_indirect() {
        // LD (HL), r8 — 2 machine cycles
        let mut cpu = init_cpu_state();
        let mut bus = make_bus();
        cpu.registers.hl = 0xC000;
        cpu.registers.set_b(0x42);
        assert_eq!(exec_ld_r8_r8(&mut cpu, &mut bus, R8Register::HL, R8Register::B, &mut noop_tick), 2);
        assert_eq!(bus.read(0xC000), 0x42);
    }
}
