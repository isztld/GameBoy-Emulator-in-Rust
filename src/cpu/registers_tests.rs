    use super::*;

    #[test]
    fn test_flags_initial_state() {
        let flags = Flags::new();
        // All flags should be 0 initially
        assert!(!flags.is_zero());
        assert!(!flags.is_subtraction());
        assert!(!flags.is_half_carry());
        assert!(!flags.is_carry());
    }

    #[test]
    fn test_flags_set_and_clear() {
        let mut flags = Flags::new();

        // Test Zero flag
        flags.set_zero(true);
        assert!(flags.is_zero());
        flags.set_zero(false);
        assert!(!flags.is_zero());

        // Test Subtraction flag
        flags.set_subtraction(true);
        assert!(flags.is_subtraction());
        flags.set_subtraction(false);
        assert!(!flags.is_subtraction());

        // Test Half Carry flag
        flags.set_half_carry(true);
        assert!(flags.is_half_carry());
        flags.set_half_carry(false);
        assert!(!flags.is_half_carry());

        // Test Carry flag
        flags.set_carry(true);
        assert!(flags.is_carry());
        flags.set_carry(false);
        assert!(!flags.is_carry());
    }

    #[test]
    fn test_flags_get_set() {
        let mut flags = Flags::new();

        flags.set_zero(true);
        flags.set_carry(true);

        // Z and C set
        assert_eq!(flags.get(), Flags::Z | Flags::C);

        // Overwrite with all bits set
        flags.set(0xFF);

        // Lower nibble must be zero (hardware behavior)
        assert_eq!(flags.get(), 0xF0);

        assert!(flags.is_zero());
        assert!(flags.is_subtraction());
        assert!(flags.is_half_carry());
        assert!(flags.is_carry());
    }

    #[test]
    fn test_flags_constants() {
        assert_eq!(Flags::Z, 0x80);
        assert_eq!(Flags::N, 0x40);
        assert_eq!(Flags::H, 0x20);
        assert_eq!(Flags::C, 0x10);
    }

    #[test]
    fn test_registers_new() {
        let regs = Registers::new();
        assert_eq!(regs.af, 0x0000);
        assert_eq!(regs.bc, 0x0000);
        assert_eq!(regs.de, 0x0000);
        assert_eq!(regs.hl, 0x0000);
        assert_eq!(regs.sp, 0xFFFE);
        assert_eq!(regs.pc, 0x0000);
    }

    #[test]
    fn test_registers_a_f() {
        let mut regs = Registers::new();

        // Test A register
        regs.set_a(0xAB);
        assert_eq!(regs.a(), 0xAB);

        // Test F register
        let mut flags = Flags::new();
        flags.set_zero(true);
        flags.set_carry(true);
        regs.set_f(flags);
        assert!(regs.f().is_zero());
        assert!(regs.f().is_carry());

        // Verify AF register is properly composed (A=0xAB, F with Z and C set = 0x90)
        assert_eq!(regs.af, 0xAB90);
    }

    #[test]
    fn test_registers_bc() {
        let mut regs = Registers::new();

        regs.set_b(0xCD);
        regs.set_c(0xEF);
        assert_eq!(regs.b(), 0xCD);
        assert_eq!(regs.c(), 0xEF);
        assert_eq!(regs.bc, 0xCDEF);

        // Test setting full 16-bit value
        regs.bc = 0x1234;
        assert_eq!(regs.b(), 0x12);
        assert_eq!(regs.c(), 0x34);
    }

    #[test]
    fn test_registers_de() {
        let mut regs = Registers::new();

        regs.set_d(0x56);
        regs.set_e(0x78);
        assert_eq!(regs.d(), 0x56);
        assert_eq!(regs.e(), 0x78);
        assert_eq!(regs.de, 0x5678);
    }

    #[test]
    fn test_registers_hl() {
        let mut regs = Registers::new();

        regs.set_h(0x9A);
        regs.set_l(0xBC);
        assert_eq!(regs.h(), 0x9A);
        assert_eq!(regs.l(), 0xBC);
        assert_eq!(regs.hl, 0x9ABC);
    }

    #[test]
    fn test_registers_sp() {
        let mut regs = Registers::new();
        assert_eq!(regs.sp, 0xFFFE);

        regs.sp = 0xC000;
        assert_eq!(regs.sp, 0xC000);
    }

    #[test]
    fn test_registers_pc() {
        let mut regs = Registers::new();
        assert_eq!(regs.pc, 0x0000);

        regs.pc = 0x0100;
        assert_eq!(regs.pc, 0x0100);
    }

    #[test]
    fn test_all_registers_together() {
        let mut regs = Registers::new();
        regs.af = 0x1234;
        regs.bc = 0x5678;
        regs.de = 0x9ABC;
        regs.hl = 0xDEF0;
        regs.sp = 0xFF00;
        regs.pc = 0x0100;

        assert_eq!(regs.a(), 0x12);
        assert_eq!(regs.f().get(), 0x30); // F register only uses upper 4 bits (hardware behavior)
        assert_eq!(regs.b(), 0x56);
        assert_eq!(regs.c(), 0x78);
        assert_eq!(regs.d(), 0x9A);
        assert_eq!(regs.e(), 0xBC);
        assert_eq!(regs.h(), 0xDE);
        assert_eq!(regs.l(), 0xF0);
        assert_eq!(regs.sp, 0xFF00);
        assert_eq!(regs.pc, 0x0100);
    }

    #[test]
    fn test_cpu_state() {
        let state = CPUState::new();
        assert_eq!(state.registers.pc, 0x0000);
        assert!(!state.ime);
    }

    #[test]
    fn test_registers_default() {
        let regs = Registers::default();
        // After reset, PC should be 0x0100 for GameBoy boot
        // But Registers::new() initializes to 0, CPU::reset sets PC to 0x0100
        assert_eq!(regs.pc, 0x0000);
        assert_eq!(regs.sp, 0xFFFE);
    }
