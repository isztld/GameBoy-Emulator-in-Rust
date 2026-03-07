    use super::*;
    use crate::memory::MemoryBus;
    use crate::cpu::instructions::R8Register;

    // Helper to initialize CPU state with specific A value
    fn init_cpu_state(a: u8) -> CPUState {
        let mut cpu = CPUState::new();
        cpu.registers.set_a(a);
        cpu.registers.modify_f(|f| f.set_zero(false));
        cpu.registers.modify_f(|f| f.set_subtraction(false));
        cpu.registers.modify_f(|f| f.set_half_carry(false));
        cpu.registers.modify_f(|f| f.set_carry(false));
        cpu
    }

    fn noop_tick(_: &mut [u8; 128]) {}

    #[test]
    fn test_add_a_r8_no_carry() {
        let mut cpu = init_cpu_state(0x10);
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.registers.set_b(0x20);

        let cycles = exec_add_a_r8(&mut cpu, &mut bus, R8Register::B, &mut noop_tick);

        assert_eq!(cycles, 1);
        assert_eq!(cpu.registers.a(), 0x30);
        assert!(!cpu.registers.f().is_zero());
        assert!(!cpu.registers.f().is_subtraction());
        assert!(!cpu.registers.f().is_half_carry());
        assert!(!cpu.registers.f().is_carry());
    }

    #[test]
    fn test_add_a_r8_zero_result() {
        let mut cpu = init_cpu_state(0x00);
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.registers.set_b(0x00);

        let cycles = exec_add_a_r8(&mut cpu, &mut bus, R8Register::B, &mut noop_tick);

        assert_eq!(cycles, 1);
        assert_eq!(cpu.registers.a(), 0x00);
        assert!(cpu.registers.f().is_zero());
        assert!(!cpu.registers.f().is_subtraction());
        assert!(!cpu.registers.f().is_half_carry());
        assert!(!cpu.registers.f().is_carry());
    }

    #[test]
    fn test_add_a_r8_carry() {
        let mut cpu = init_cpu_state(0xFF);
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.registers.set_b(0x01);

        let cycles = exec_add_a_r8(&mut cpu, &mut bus, R8Register::B, &mut noop_tick);

        assert_eq!(cycles, 1);
        assert_eq!(cpu.registers.a(), 0x00);
        assert!(cpu.registers.f().is_carry());
        assert!(cpu.registers.f().is_half_carry());
    }

    #[test]
    fn test_add_a_r8_half_carry() {
        let mut cpu = init_cpu_state(0x0F);
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.registers.set_b(0x01);

        let cycles = exec_add_a_r8(&mut cpu, &mut bus, R8Register::B, &mut noop_tick);

        assert_eq!(cycles, 1);
        assert_eq!(cpu.registers.a(), 0x10);
        assert!(cpu.registers.f().is_half_carry());
        assert!(!cpu.registers.f().is_carry());
    }

    #[test]
    fn test_adc_a_r8_no_carry() {
        let mut cpu = init_cpu_state(0x10);
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.registers.set_b(0x20);

        let cycles = exec_adc_a_r8(&mut cpu, &mut bus, R8Register::B, &mut noop_tick);

        assert_eq!(cycles, 1);
        assert_eq!(cpu.registers.a(), 0x30);
        assert!(!cpu.registers.f().is_carry());
    }

    #[test]
    fn test_adc_a_r8_with_carry() {
        let mut cpu = init_cpu_state(0x10);
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.registers.set_b(0x20);
        cpu.registers.modify_f(|f| f.set_carry(true));

        let cycles = exec_adc_a_r8(&mut cpu, &mut bus, R8Register::B, &mut noop_tick);

        assert_eq!(cycles, 1);
        assert_eq!(cpu.registers.a(), 0x31); // 10 + 20 + 1
    }

    #[test]
    fn test_sub_a_r8() {
        let mut cpu = init_cpu_state(0x50);
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.registers.set_b(0x30);

        let cycles = exec_sub_a_r8(&mut cpu, &mut bus, R8Register::B, &mut noop_tick);

        assert_eq!(cycles, 1);
        assert_eq!(cpu.registers.a(), 0x20);
        assert!(cpu.registers.f().is_subtraction());
        assert!(!cpu.registers.f().is_carry());
    }

    #[test]
    fn test_sub_a_r8_carry() {
        let mut cpu = init_cpu_state(0x10);
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.registers.set_b(0x20);

        let cycles = exec_sub_a_r8(&mut cpu, &mut bus, R8Register::B, &mut noop_tick);

        assert_eq!(cycles, 1);
        assert_eq!(cpu.registers.a(), 0xF0);
        assert!(cpu.registers.f().is_carry());
    }

    #[test]
    fn test_sbc_a_r8() {
        let mut cpu = init_cpu_state(0x50);
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.registers.set_b(0x30);
        cpu.registers.modify_f(|f| f.set_carry(true)); // set initial carry

        let cycles = exec_sbc_a_r8(&mut cpu, &mut bus, R8Register::B, &mut noop_tick);

        assert_eq!(cycles, 1);
        assert_eq!(cpu.registers.a(), 0x1F); // 50 - 30 - 1
    }

    #[test]
    fn test_sbc_a_r8_with_carry() {
        let mut cpu = init_cpu_state(0x50);
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.registers.set_b(0x30);
        cpu.registers.modify_f(|f| f.set_carry(true));

        let cycles = exec_sbc_a_r8(&mut cpu, &mut bus, R8Register::B, &mut noop_tick);

        assert_eq!(cycles, 1);
        assert_eq!(cpu.registers.a(), 0x1F); // 50 - 30 - 2
    }

    #[test]
    fn test_and_a_r8() {
        let mut cpu = init_cpu_state(0xFF);
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.registers.set_b(0x0F);

        let cycles = exec_and_a_r8(&mut cpu, &mut bus, R8Register::B, &mut noop_tick);

        assert_eq!(cycles, 1);
        assert_eq!(cpu.registers.a(), 0x0F);
        assert!(!cpu.registers.f().is_zero());
        assert!(!cpu.registers.f().is_subtraction());
        assert!(cpu.registers.f().is_half_carry());
        assert!(!cpu.registers.f().is_carry());
    }

    #[test]
    fn test_and_a_r8_zero() {
        let mut cpu = init_cpu_state(0xFF);
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.registers.set_b(0x00);

        let cycles = exec_and_a_r8(&mut cpu, &mut bus, R8Register::B, &mut noop_tick);

        assert_eq!(cycles, 1);
        assert_eq!(cpu.registers.a(), 0x00);
        assert!(cpu.registers.f().is_zero());
    }

    #[test]
    fn test_xor_a_r8() {
        let mut cpu = init_cpu_state(0xFF);
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.registers.set_b(0x0F);

        let cycles = exec_xor_a_r8(&mut cpu, &mut bus, R8Register::B, &mut noop_tick);

        assert_eq!(cycles, 1);
        assert_eq!(cpu.registers.a(), 0xF0);
        assert!(!cpu.registers.f().is_zero());
    }

    #[test]
    fn test_xor_a_r8_same() {
        let mut cpu = init_cpu_state(0x5A);
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.registers.set_b(0x5A);

        let cycles = exec_xor_a_r8(&mut cpu, &mut bus, R8Register::B, &mut noop_tick);

        assert_eq!(cycles, 1);
        assert_eq!(cpu.registers.a(), 0x00);
        assert!(cpu.registers.f().is_zero());
    }

    #[test]
    fn test_or_a_r8() {
        let mut cpu = init_cpu_state(0xF0);
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.registers.set_b(0x0F);

        let cycles = exec_or_a_r8(&mut cpu, &mut bus, R8Register::B, &mut noop_tick);

        assert_eq!(cycles, 1);
        assert_eq!(cpu.registers.a(), 0xFF);
        assert!(!cpu.registers.f().is_zero());
    }

    #[test]
    fn test_or_a_r8_zero() {
        let mut cpu = init_cpu_state(0x00);
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.registers.set_b(0x00);

        let cycles = exec_or_a_r8(&mut cpu, &mut bus, R8Register::B, &mut noop_tick);

        assert_eq!(cycles, 1);
        assert_eq!(cpu.registers.a(), 0x00);
        assert!(cpu.registers.f().is_zero());
    }

    #[test]
    fn test_cp_a_r8_equal() {
        let mut cpu = init_cpu_state(0x50);
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.registers.set_b(0x50);

        let cycles = exec_cp_a_r8(&mut cpu, &mut bus, R8Register::B, &mut noop_tick);

        assert_eq!(cycles, 1);
        assert!(cpu.registers.f().is_zero());
        assert!(cpu.registers.f().is_subtraction());
        assert!(!cpu.registers.f().is_carry());
        // A should not be modified
        assert_eq!(cpu.registers.a(), 0x50);
    }

    #[test]
    fn test_cp_a_r8_less() {
        let mut cpu = init_cpu_state(0x30);
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.registers.set_b(0x50);

        let cycles = exec_cp_a_r8(&mut cpu, &mut bus, R8Register::B, &mut noop_tick);

        assert_eq!(cycles, 1);
        assert!(!cpu.registers.f().is_zero());
        assert!(cpu.registers.f().is_carry());
    }

    #[test]
    fn test_cp_a_r8_negative() {
        let mut cpu = init_cpu_state(0x10);
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.registers.set_b(0x20);

        let cycles = exec_cp_a_r8(&mut cpu, &mut bus, R8Register::B, &mut noop_tick);

        assert_eq!(cycles, 1);
        // In signed comparison, 0x10 < 0x20, so carry should be set
        assert!(cpu.registers.f().is_carry());
    }

    #[test]
    fn test_add_a_imm8() {
        let mut cpu = init_cpu_state(0x10);

        let cycles = exec_add_a_imm8(&mut cpu, 0x20, &mut [0u8; 128], &mut noop_tick);

        assert_eq!(cycles, 2);
        assert_eq!(cpu.registers.a(), 0x30);
        assert!(!cpu.registers.f().is_carry());
    }

    #[test]
    fn test_adc_a_imm8() {
        let mut cpu = init_cpu_state(0x10);

        let cycles = exec_adc_a_imm8(&mut cpu, 0x20, &mut [0u8; 128], &mut noop_tick);

        assert_eq!(cycles, 2);
        assert_eq!(cpu.registers.a(), 0x30);
    }

    #[test]
    fn test_adc_a_imm8_with_carry() {
        let mut cpu = init_cpu_state(0x10);
        cpu.registers.modify_f(|f| f.set_carry(true));

        let cycles = exec_adc_a_imm8(&mut cpu, 0x20, &mut [0u8; 128], &mut noop_tick);

        assert_eq!(cycles, 2);
        assert_eq!(cpu.registers.a(), 0x31);
    }

    #[test]
    fn test_sub_a_imm8() {
        let mut cpu = init_cpu_state(0x50);

        let cycles = exec_sub_a_imm8(&mut cpu, 0x30, &mut [0u8; 128], &mut noop_tick);

        assert_eq!(cycles, 2);
        assert_eq!(cpu.registers.a(), 0x20);
        assert!(cpu.registers.f().is_subtraction());
    }

    #[test]
    fn test_sbc_a_imm8() {
        let mut cpu = init_cpu_state(0x50);

        let cycles = exec_sbc_a_imm8(&mut cpu, 0x30, &mut [0u8; 128], &mut noop_tick);

        assert_eq!(cycles, 2);
        assert_eq!(cpu.registers.a(), 0x20);
    }

    #[test]
    fn test_and_a_imm8() {
        let mut cpu = init_cpu_state(0xFF);

        let cycles = exec_and_a_imm8(&mut cpu, 0x0F, &mut [0u8; 128], &mut noop_tick);

        assert_eq!(cycles, 2);
        assert_eq!(cpu.registers.a(), 0x0F);
        assert!(cpu.registers.f().is_half_carry());
    }

    #[test]
    fn test_xor_a_imm8() {
        let mut cpu = init_cpu_state(0xFF);

        let cycles = exec_xor_a_imm8(&mut cpu, 0x0F, &mut [0u8; 128], &mut noop_tick);

        assert_eq!(cycles, 2);
        assert_eq!(cpu.registers.a(), 0xF0);
    }

    #[test]
    fn test_or_a_imm8() {
        let mut cpu = init_cpu_state(0xF0);

        let cycles = exec_or_a_imm8(&mut cpu, 0x0F, &mut [0u8; 128], &mut noop_tick);

        assert_eq!(cycles, 2);
        assert_eq!(cpu.registers.a(), 0xFF);
    }

    #[test]
    fn test_cp_a_imm8() {
        let mut cpu = init_cpu_state(0x50);

        let cycles = exec_cp_a_imm8(&mut cpu, 0x50, &mut [0u8; 128], &mut noop_tick);

        assert_eq!(cycles, 2);
        assert!(cpu.registers.f().is_zero());
        assert!(cpu.registers.f().is_subtraction());
    }
