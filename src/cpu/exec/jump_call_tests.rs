    use super::*;
    use crate::memory::MemoryBus;
    use crate::cpu::instructions::Condition;

    fn init_cpu_state() -> CPUState {
        let mut cpu = CPUState::new();
        cpu.registers.modify_f(|f| f.set_zero(false));
        cpu.registers.modify_f(|f| f.set_subtraction(false));
        cpu.registers.modify_f(|f| f.set_half_carry(false));
        cpu.registers.modify_f(|f| f.set_carry(false));
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
        cpu.registers.modify_f(|f| f.set_zero(false));

        let cycles = exec_jr_cond_imm8(&mut cpu, Condition::NZ, 5, &mut [0u8; 128], &mut noop_tick);

        assert_eq!(cycles, 3);
        assert_eq!(cpu.registers.pc, 0x1005);
    }

    #[test]
    fn test_jr_cond_jump_not_taken() {
        let mut cpu = init_cpu_state();
        cpu.registers.pc = 0x1000;
        cpu.registers.modify_f(|f| f.set_zero(true));

        let cycles = exec_jr_cond_imm8(&mut cpu, Condition::NZ, 5, &mut [0u8; 128], &mut noop_tick);

        assert_eq!(cycles, 2);
        assert_eq!(cpu.registers.pc, 0x1000);
    }

    #[test]
    fn test_jp_cond_imm16_taken() {
        let mut cpu = init_cpu_state();
        cpu.registers.modify_f(|f| f.set_zero(false));

        let cycles = exec_jp_cond_imm16(&mut cpu, Condition::NZ, 0x8000, &mut [0u8; 128], &mut noop_tick);

        assert_eq!(cycles, 4);
        assert_eq!(cpu.registers.pc, 0x8000);
    }

    #[test]
    fn test_jp_cond_imm16_not_taken() {
        let mut cpu = init_cpu_state();
        cpu.registers.modify_f(|f| f.set_zero(true));

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
        cpu.registers.modify_f(|f| f.set_zero(false));
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
        cpu.registers.modify_f(|f| f.set_zero(true));
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
        cpu.registers.modify_f(|f| f.set_zero(false));

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
        cpu.registers.modify_f(|f| f.set_zero(true));

        let cycles = exec_ret_cond(&mut cpu, Condition::NZ, &mut bus, &mut noop_tick);

        assert_eq!(cycles, 2);
        assert_eq!(cpu.registers.pc, 0x0000);
        assert_eq!(cpu.registers.sp, 0xFFFC);
    }

    #[test]
    fn test_cond_nz_true() {
        let mut cpu = init_cpu_state();
        cpu.registers.modify_f(|f| f.set_zero(false));

        assert!(cond_condition(&cpu, Condition::NZ));
    }

    #[test]
    fn test_cond_nz_false() {
        let mut cpu = init_cpu_state();
        cpu.registers.modify_f(|f| f.set_zero(true));

        assert!(!cond_condition(&cpu, Condition::NZ));
    }

    #[test]
    fn test_cond_z_true() {
        let mut cpu = init_cpu_state();
        cpu.registers.modify_f(|f| f.set_zero(true));

        assert!(cond_condition(&cpu, Condition::Z));
    }

    #[test]
    fn test_cond_z_false() {
        let mut cpu = init_cpu_state();
        cpu.registers.modify_f(|f| f.set_zero(false));

        assert!(!cond_condition(&cpu, Condition::Z));
    }

    #[test]
    fn test_cond_nc_true() {
        let mut cpu = init_cpu_state();
        cpu.registers.modify_f(|f| f.set_carry(false));

        assert!(cond_condition(&cpu, Condition::NC));
    }

    #[test]
    fn test_cond_nc_false() {
        let mut cpu = init_cpu_state();
        cpu.registers.modify_f(|f| f.set_carry(true));

        assert!(!cond_condition(&cpu, Condition::NC));
    }

    #[test]
    fn test_cond_c_true() {
        let mut cpu = init_cpu_state();
        cpu.registers.modify_f(|f| f.set_carry(true));

        assert!(cond_condition(&cpu, Condition::C));
    }

    #[test]
    fn test_cond_c_false() {
        let mut cpu = init_cpu_state();
        cpu.registers.modify_f(|f| f.set_carry(false));

        assert!(!cond_condition(&cpu, Condition::C));
    }
