    use super::*;
    use crate::memory::MemoryBus;
    use crate::cpu::instructions::{R8Register, R16Register, R16Mem};

    fn make_bus() -> MemoryBus {
        MemoryBus::new(vec![0u8; 32768])
    }

    fn init_cpu_state() -> CPUState {
        let mut cpu = CPUState::new();
        cpu.registers.modify_f(|f| f.set_zero(false));
        cpu.registers.modify_f(|f| f.set_subtraction(false));
        cpu.registers.modify_f(|f| f.set_half_carry(false));
        cpu.registers.modify_f(|f| f.set_carry(false));
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
