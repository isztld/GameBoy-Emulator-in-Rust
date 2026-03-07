    use super::*;
    use crate::memory::MemoryBus;
    use crate::cpu::instructions::R16Register;

    fn init_cpu_state() -> CPUState {
        let mut cpu = CPUState::new();
        cpu.registers.modify_f(|f| f.set_zero(false));
        cpu.registers.modify_f(|f| f.set_subtraction(false));
        cpu.registers.modify_f(|f| f.set_half_carry(false));
        cpu.registers.modify_f(|f| f.set_carry(false));
        cpu.ime = false;
        cpu
    }

    fn noop_tick(_: &mut [u8; 128]) {}

    #[test]
    fn test_ret() {
        let mut cpu = init_cpu_state();
        cpu.registers.sp = 0xFFFC;
        let mut bus = MemoryBus::new(vec![0; 32768]);
        bus.write(0xFFFC, 0x00); // low byte
        bus.write(0xFFFD, 0x80); // high byte

        let cycles = exec_ret(&mut cpu, &mut bus, &mut noop_tick);

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

        let cycles = exec_reti(&mut cpu, &mut bus, &mut noop_tick);

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

        let cycles = exec_pop_r16(&mut cpu, R16Register::BC, &mut bus, &mut noop_tick);

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

        let cycles = exec_pop_r16(&mut cpu, R16Register::AF, &mut bus, &mut noop_tick);

        assert_eq!(cycles, 3);
        assert_eq!(cpu.registers.af, 0x1230); // lower nibble of F always 0
    }

    #[test]
    fn test_push_r16() {
        let mut cpu = init_cpu_state();
        cpu.registers.sp = 0xFFFE;
        cpu.registers.bc = 0x1234;
        let mut bus = MemoryBus::new(vec![0; 32768]);

        let cycles = exec_push_r16(&mut cpu, R16Register::BC, &mut bus, &mut noop_tick);

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

        exec_push_r16(&mut cpu, R16Register::BC, &mut bus, &mut noop_tick);
        cpu.registers.bc = 0x0000;
        exec_pop_r16(&mut cpu, R16Register::BC, &mut bus, &mut noop_tick);

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

        exec_rst(&mut cpu_state, 0x08, &mut bus, &mut noop_tick);

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

        exec_rst(&mut cpu_state, 0x08, &mut bus, &mut noop_tick);
        assert_eq!(cpu_state.registers.pc, 0x0008);

        exec_ret(&mut cpu_state, &mut bus, &mut noop_tick);
        assert_eq!(cpu_state.registers.pc, 0x1001); // back to instruction after RST
        assert_eq!(cpu_state.registers.sp, 0xC000);
    }

    #[test]
    fn test_add_sp_imm8_positive() {
        let mut cpu = init_cpu_state();
        cpu.registers.sp = 0xFF00;

        let cycles = exec_add_sp_imm8(&mut cpu, 10, &mut [0u8; 128], &mut noop_tick);

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

        let cycles = exec_add_sp_imm8(&mut cpu, -1, &mut [0u8; 128], &mut noop_tick);

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

        let cycles = exec_add_sp_imm8(&mut cpu, 1, &mut [0u8; 128], &mut noop_tick);

        assert_eq!(cycles, 4);
        assert_eq!(cpu.registers.sp, 0x0010);
        assert!(cpu.registers.f().is_half_carry());
        assert!(!cpu.registers.f().is_carry());
    }

    #[test]
    fn test_add_sp_imm8_carry() {
        let mut cpu = init_cpu_state();
        cpu.registers.sp = 0x00FF;

        let cycles = exec_add_sp_imm8(&mut cpu, 1, &mut [0u8; 128], &mut noop_tick);

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

        exec_add_sp_imm8(&mut cpu, -16, &mut [0u8; 128], &mut noop_tick);

        assert_eq!(cpu.registers.sp, 0xFEF0);
        assert!(!cpu.registers.f().is_carry());
        assert!(!cpu.registers.f().is_half_carry());
    }

    #[test]
    fn test_ld_hl_sp_imm8_positive() {
        let mut cpu = init_cpu_state();
        cpu.registers.sp = 0xFF00;

        let cycles = exec_ld_hl_sp_imm8(&mut cpu, 10, &mut [0u8; 128], &mut noop_tick);

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

        let cycles = exec_ld_hl_sp_imm8(&mut cpu, -10, &mut [0u8; 128], &mut noop_tick);

        assert_eq!(cycles, 3);
        assert_eq!(cpu.registers.hl, 0xFEF6);
        assert_eq!(cpu.registers.sp, 0xFF00); // SP must not change
    }

    #[test]
    fn test_ld_sp_hl() {
        let mut cpu = init_cpu_state();
        cpu.registers.hl = 0x8000;

        let cycles = exec_ld_sp_hl(&mut cpu, &mut [0u8; 128], &mut noop_tick);

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

        let cycles = exec_ldh_ind_c_a(&mut cpu, &mut bus, &mut noop_tick);

        assert_eq!(cycles, 2);
        assert_eq!(bus.read(0xFF10), 0x42);
    }

    #[test]
    fn test_ldh_a_c() {
        let mut cpu = init_cpu_state();
        cpu.registers.set_c(0x10);
        let mut bus = MemoryBus::new(vec![0; 32768]);
        bus.write(0xFF10, 0xAB);

        let cycles = exec_ldh_a_c(&mut cpu, &mut bus, &mut noop_tick);

        assert_eq!(cycles, 2);
        assert_eq!(cpu.registers.a(), 0xAB);
    }

    #[test]
    fn test_ldh_ind_imm8_a() {
        let mut cpu = init_cpu_state();
        cpu.registers.set_a(0x77);
        let mut bus = MemoryBus::new(vec![0; 32768]);

        let cycles = exec_ldh_ind_imm8_a(&mut cpu, 0x10, &mut bus, &mut noop_tick);

        assert_eq!(cycles, 3);
        assert_eq!(bus.read(0xFF10), 0x77);
    }

    #[test]
    fn test_ldh_a_ind_imm8() {
        let mut cpu = init_cpu_state();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        bus.write(0xFF10, 0xAB);

        let cycles = exec_ldh_a_ind_imm8(&mut cpu, 0x10, &mut bus, &mut noop_tick);

        assert_eq!(cycles, 3);
        assert_eq!(cpu.registers.a(), 0xAB);
    }
