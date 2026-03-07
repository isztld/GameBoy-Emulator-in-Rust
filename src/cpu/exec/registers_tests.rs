    use super::*;
    use crate::memory::MemoryBus;
    use crate::cpu::instructions::{R8Register, R16Register};

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
    // INC r16 / DEC r16
    // -----------------------------------------------------------------------

    #[test]
    fn test_inc_r16() {
        let mut cpu = init_cpu_state();
        cpu.registers.bc = 0x1234;
        assert_eq!(exec_inc_r16(&mut cpu, R16Register::BC, &mut [0u8; 128], &mut noop_tick), 2);
        assert_eq!(cpu.registers.bc, 0x1235);
    }

    #[test]
    fn test_inc_r16_wrap() {
        let mut cpu = init_cpu_state();
        cpu.registers.bc = 0xFFFF;
        assert_eq!(exec_inc_r16(&mut cpu, R16Register::BC, &mut [0u8; 128], &mut noop_tick), 2);
        assert_eq!(cpu.registers.bc, 0x0000);
    }

    #[test]
    fn test_dec_r16() {
        let mut cpu = init_cpu_state();
        cpu.registers.bc = 0x1234;
        assert_eq!(exec_dec_r16(&mut cpu, R16Register::BC, &mut [0u8; 128], &mut noop_tick), 2);
        assert_eq!(cpu.registers.bc, 0x1233);
    }

    #[test]
    fn test_dec_r16_wrap() {
        let mut cpu = init_cpu_state();
        cpu.registers.bc = 0x0000;
        assert_eq!(exec_dec_r16(&mut cpu, R16Register::BC, &mut [0u8; 128], &mut noop_tick), 2);
        assert_eq!(cpu.registers.bc, 0xFFFF);
    }

    // INC/DEC r16 do not affect flags
    #[test]
    fn test_inc_dec_r16_flags_unchanged() {
        let mut cpu = init_cpu_state();
        cpu.registers.modify_f(|f| f.set_zero(true));
        cpu.registers.modify_f(|f| f.set_carry(true));
        cpu.registers.hl = 0x1000;
        exec_inc_r16(&mut cpu, R16Register::HL, &mut [0u8; 128], &mut noop_tick);
        assert!(cpu.registers.f().is_zero());
        assert!(cpu.registers.f().is_carry());
    }

    // -----------------------------------------------------------------------
    // INC r8
    // -----------------------------------------------------------------------

    #[test]
    fn test_inc_r8_basic() {
        let mut cpu = init_cpu_state();
        let mut bus = make_bus();
        cpu.registers.set_b(0x10);
        assert_eq!(exec_inc_r8(&mut cpu, &mut bus, R8Register::B, &mut noop_tick), 1);
        assert_eq!(cpu.registers.b(), 0x11);
        assert!(!cpu.registers.f().is_zero());
        assert!(!cpu.registers.f().is_subtraction());
        assert!(!cpu.registers.f().is_half_carry()); // 0x10 + 1: no nibble overflow
    }

    #[test]
    fn test_inc_r8_half_carry() {
        // 0x0F + 1 → nibble overflow → half-carry set
        let mut cpu = init_cpu_state();
        let mut bus = make_bus();
        cpu.registers.set_b(0x0F);
        exec_inc_r8(&mut cpu, &mut bus, R8Register::B, &mut noop_tick);
        assert_eq!(cpu.registers.b(), 0x10);
        assert!(cpu.registers.f().is_half_carry());
        assert!(!cpu.registers.f().is_zero());
    }

    #[test]
    fn test_inc_r8_wrap_to_zero() {
        // 0xFF + 1 → 0x00; zero flag set, half-carry set
        let mut cpu = init_cpu_state();
        let mut bus = make_bus();
        cpu.registers.set_b(0xFF);
        exec_inc_r8(&mut cpu, &mut bus, R8Register::B, &mut noop_tick);
        assert_eq!(cpu.registers.b(), 0x00);
        assert!(cpu.registers.f().is_zero());
        assert!(cpu.registers.f().is_half_carry());
    }

    #[test]
    fn test_inc_r8_hl_indirect() {
        // INC (HL) takes 3 machine cycles
        let mut cpu = init_cpu_state();
        let mut bus = make_bus();
        cpu.registers.hl = 0xC000;
        bus.write(0xC000, 0x41);
        assert_eq!(exec_inc_r8(&mut cpu, &mut bus, R8Register::HL, &mut noop_tick), 3);
        assert_eq!(bus.read(0xC000), 0x42);
    }

    // -----------------------------------------------------------------------
    // DEC r8
    // -----------------------------------------------------------------------

    #[test]
    fn test_dec_r8_basic() {
        let mut cpu = init_cpu_state();
        let mut bus = make_bus();
        cpu.registers.set_b(0x10);
        assert_eq!(exec_dec_r8(&mut cpu, &mut bus, R8Register::B, &mut noop_tick), 1);
        assert_eq!(cpu.registers.b(), 0x0F);
        assert!(!cpu.registers.f().is_zero());
        assert!(cpu.registers.f().is_subtraction());
        // 0x10 & 0x0F == 0x00 → half-carry set (borrow from nibble)
        assert!(cpu.registers.f().is_half_carry());
    }

    #[test]
    fn test_dec_r8_to_zero() {
        let mut cpu = init_cpu_state();
        let mut bus = make_bus();
        cpu.registers.set_b(0x01);
        exec_dec_r8(&mut cpu, &mut bus, R8Register::B, &mut noop_tick);
        assert_eq!(cpu.registers.b(), 0x00);
        assert!(cpu.registers.f().is_zero());
        assert!(cpu.registers.f().is_subtraction());
        assert!(!cpu.registers.f().is_half_carry()); // 0x01 & 0x0F != 0
    }

    #[test]
    fn test_dec_r8_wrap_from_zero() {
        // 0x00 - 1 → 0xFF; zero clear, half-carry set (borrow), subtraction set
        let mut cpu = init_cpu_state();
        let mut bus = make_bus();
        cpu.registers.set_b(0x00);
        exec_dec_r8(&mut cpu, &mut bus, R8Register::B, &mut noop_tick);
        assert_eq!(cpu.registers.b(), 0xFF);
        assert!(!cpu.registers.f().is_zero());
        assert!(cpu.registers.f().is_subtraction());
        assert!(cpu.registers.f().is_half_carry()); // 0x00 & 0x0F == 0
    }

    #[test]
    fn test_dec_r8_hl_indirect() {
        // DEC (HL) takes 3 machine cycles
        let mut cpu = init_cpu_state();
        let mut bus = make_bus();
        cpu.registers.hl = 0xC000;
        bus.write(0xC000, 0x43);
        assert_eq!(exec_dec_r8(&mut cpu, &mut bus, R8Register::HL, &mut noop_tick), 3);
        assert_eq!(bus.read(0xC000), 0x42);
    }

    // -----------------------------------------------------------------------
    // ADD HL, r16
    // -----------------------------------------------------------------------

    #[test]
    fn test_add_hl_r16_no_carry() {
        let mut cpu = init_cpu_state();
        cpu.registers.hl = 0x1000;
        cpu.registers.bc = 0x2000;
        assert_eq!(exec_add_hl_r16(&mut cpu, R16Register::BC, &mut [0u8; 128], &mut noop_tick), 2);
        assert_eq!(cpu.registers.hl, 0x3000);
        assert!(!cpu.registers.f().is_half_carry());
        assert!(!cpu.registers.f().is_carry());
        assert!(!cpu.registers.f().is_subtraction());
    }

    #[test]
    fn test_add_hl_r16_carry() {
        let mut cpu = init_cpu_state();
        cpu.registers.hl = 0xFFFF;
        cpu.registers.bc = 0x0001;
        exec_add_hl_r16(&mut cpu, R16Register::BC, &mut [0u8; 128], &mut noop_tick);
        assert_eq!(cpu.registers.hl, 0x0000);
        assert!(cpu.registers.f().is_carry());
    }

    #[test]
    fn test_add_hl_r16_half_carry() {
        let mut cpu = init_cpu_state();
        cpu.registers.hl = 0x0FFF;
        cpu.registers.bc = 0x0001;
        exec_add_hl_r16(&mut cpu, R16Register::BC, &mut [0u8; 128], &mut noop_tick);
        assert_eq!(cpu.registers.hl, 0x1000);
        assert!(cpu.registers.f().is_half_carry());
        assert!(!cpu.registers.f().is_carry());
    }

    #[test]
    fn test_add_hl_r16_zero_flag_unaffected() {
        // ADD HL, r16 does not modify the zero flag
        let mut cpu = init_cpu_state();
        cpu.registers.modify_f(|f| f.set_zero(true));
        cpu.registers.hl = 0x0001;
        cpu.registers.de = 0x0001;
        exec_add_hl_r16(&mut cpu, R16Register::DE, &mut [0u8; 128], &mut noop_tick);
        assert!(cpu.registers.f().is_zero(), "zero flag must be preserved");
    }

    #[test]
    fn test_add_hl_r16_with_sp() {
        let mut cpu = init_cpu_state();
        cpu.registers.hl = 0x1000;
        cpu.registers.sp = 0x2000;
        exec_add_hl_r16(&mut cpu, R16Register::SP, &mut [0u8; 128], &mut noop_tick);
        assert_eq!(cpu.registers.hl, 0x3000);
    }

    #[test]
    fn test_add_hl_hl() {
        // ADD HL, HL — HL is both source and destination
        let mut cpu = init_cpu_state();
        cpu.registers.hl = 0x1000;
        exec_add_hl_r16(&mut cpu, R16Register::HL, &mut [0u8; 128], &mut noop_tick);
        assert_eq!(cpu.registers.hl, 0x2000);
    }
