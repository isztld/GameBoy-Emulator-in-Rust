    use super::*;
    use crate::memory::MemoryBus;
    use crate::cpu::instructions::R8Register;

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
    fn test_rlcr8() {
        let mut cpu = init_cpu_state();
        cpu.registers.set_b(0b11001111);
        let mut bus = MemoryBus::new(vec![0; 32768]);

        let cycles = exec_rlcr8(&mut cpu, &mut bus, R8Register::B, &mut noop_tick);

        assert_eq!(cycles, 2);
        // Rotate left: 0b11001111 -> 0b10011111
        assert_eq!(cpu.registers.b(), 0b10011111);
        // Old bit 7 (1) went to carry
        assert!(cpu.registers.f().is_carry());
        assert!(!cpu.registers.f().is_zero());
    }

    #[test]
    fn test_rlcr8_zero() {
        let mut cpu = init_cpu_state();
        cpu.registers.set_b(0b00000001);
        let mut bus = MemoryBus::new(vec![0; 32768]);

        let cycles = exec_rlcr8(&mut cpu, &mut bus, R8Register::B, &mut noop_tick);

        assert_eq!(cycles, 2);
        // Rotate left: 0b00000001 -> 0b00000010
        assert_eq!(cpu.registers.b(), 0b00000010);
        // Old bit 7 (0) went to carry
        assert!(!cpu.registers.f().is_carry());
    }

    #[test]
    fn test_rlcr8_zero_result() {
        let mut cpu = init_cpu_state();
        cpu.registers.set_b(0b00000000);
        let mut bus = MemoryBus::new(vec![0; 32768]);

        let cycles = exec_rlcr8(&mut cpu, &mut bus, R8Register::B, &mut noop_tick);

        assert_eq!(cycles, 2);
        // Result is still 0
        assert!(cpu.registers.f().is_zero());
    }

    #[test]
    fn test_rrcr8() {
        let mut cpu = init_cpu_state();
        cpu.registers.set_b(0b11001111);
        let mut bus = MemoryBus::new(vec![0; 32768]);

        let cycles = exec_rrcr8(&mut cpu, &mut bus, R8Register::B, &mut noop_tick);

        assert_eq!(cycles, 2);
        // Rotate right: 0b11001111 -> 0b11100111
        assert_eq!(cpu.registers.b(), 0b11100111);
        // Old bit 0 (1) went to carry
        assert!(cpu.registers.f().is_carry());
    }

    #[test]
    fn test_rlr8() {
        let mut cpu = init_cpu_state();
        cpu.registers.set_b(0b11001111);
        cpu.registers.modify_f(|f| f.set_carry(false));
        let mut bus = MemoryBus::new(vec![0; 32768]);

        let cycles = exec_rlr8(&mut cpu, &mut bus, R8Register::B, &mut noop_tick);

        assert_eq!(cycles, 2);
        // Shift left with carry in: 0b11001111 << 1 = 0b10011110, carry out = 1
        assert_eq!(cpu.registers.b(), 0b10011110);
        assert!(cpu.registers.f().is_carry());
    }

    #[test]
    fn test_rlr8_with_carry() {
        let mut cpu = init_cpu_state();
        cpu.registers.set_b(0b01001111);
        cpu.registers.modify_f(|f| f.set_carry(true));
        let mut bus = MemoryBus::new(vec![0; 32768]);

        let cycles = exec_rlr8(&mut cpu, &mut bus, R8Register::B, &mut noop_tick);

        assert_eq!(cycles, 2);
        // Shift left with carry in: 0b01001111 << 1 = 0b10011110, OR carry = 0b10011111
        assert_eq!(cpu.registers.b(), 0b10011111);
        assert!(!cpu.registers.f().is_carry());
    }

    #[test]
    fn test_rrr8() {
        let mut cpu = init_cpu_state();
        cpu.registers.set_b(0b11001111);
        cpu.registers.modify_f(|f| f.set_carry(false));
        let mut bus = MemoryBus::new(vec![0; 32768]);

        let cycles = exec_rrr8(&mut cpu, &mut bus, R8Register::B, &mut noop_tick);

        assert_eq!(cycles, 2);
        // Shift right with carry in: 0b11001111 >> 1 = 0b01100111, carry out = 1
        assert_eq!(cpu.registers.b(), 0b01100111);
        assert!(cpu.registers.f().is_carry());
    }

    #[test]
    fn test_rrr8_with_carry() {
        let mut cpu = init_cpu_state();
        cpu.registers.set_b(0b01001111);
        cpu.registers.modify_f(|f| f.set_carry(true));
        let mut bus = MemoryBus::new(vec![0; 32768]);

        let cycles = exec_rrr8(&mut cpu, &mut bus, R8Register::B, &mut noop_tick);

        assert_eq!(cycles, 2);
        // Shift right with carry in: 0b01001111 >> 1 = 0b00100111, OR (carry << 7) = 0b10100111
        assert_eq!(cpu.registers.b(), 0b10100111);
        assert!(cpu.registers.f().is_carry());
    }

    #[test]
    fn test_slar8() {
        let mut cpu = init_cpu_state();
        cpu.registers.set_b(0b11001111);
        let mut bus = MemoryBus::new(vec![0; 32768]);

        let cycles = exec_slar8(&mut cpu, &mut bus, R8Register::B, &mut noop_tick);

        assert_eq!(cycles, 2);
        // Shift left: 0b11001111 << 1 = 0b10011110, carry out = 1
        assert_eq!(cpu.registers.b(), 0b10011110);
        assert!(cpu.registers.f().is_carry());
    }

    #[test]
    fn test_srar8() {
        let mut cpu = init_cpu_state();
        cpu.registers.set_b(0b11001111);
        let mut bus = MemoryBus::new(vec![0; 32768]);

        let cycles = exec_srar8(&mut cpu, &mut bus, R8Register::B, &mut noop_tick);

        assert_eq!(cycles, 2);
        // Arithmetic shift right (sign-extended): 0b11001111 >> 1 = 0b11100111
        assert_eq!(cpu.registers.b(), 0b11100111);
        assert!(cpu.registers.f().is_carry()); // Old bit 0
    }

    #[test]
    fn test_srar8_positive() {
        let mut cpu = init_cpu_state();
        cpu.registers.set_b(0b01001111);
        let mut bus = MemoryBus::new(vec![0; 32768]);

        let cycles = exec_srar8(&mut cpu, &mut bus, R8Register::B, &mut noop_tick);

        assert_eq!(cycles, 2);
        // Arithmetic shift right: 0b01001111 >> 1 = 0b00100111
        assert_eq!(cpu.registers.b(), 0b00100111);
    }

    #[test]
    fn test_swapr8() {
        let mut cpu = init_cpu_state();
        cpu.registers.set_b(0x5A);
        let mut bus = MemoryBus::new(vec![0; 32768]);

        let cycles = exec_swapr8(&mut cpu, &mut bus, R8Register::B, &mut noop_tick);

        assert_eq!(cycles, 2);
        // Swap nibbles: 0x5A -> 0xA5
        assert_eq!(cpu.registers.b(), 0xA5);
        assert!(!cpu.registers.f().is_zero());
        assert!(!cpu.registers.f().is_half_carry());
        assert!(!cpu.registers.f().is_carry());
    }

    #[test]
    fn test_swapr8_zero() {
        let mut cpu = init_cpu_state();
        cpu.registers.set_b(0x00);
        let mut bus = MemoryBus::new(vec![0; 32768]);

        let cycles = exec_swapr8(&mut cpu, &mut bus, R8Register::B, &mut noop_tick);

        assert_eq!(cycles, 2);
        assert!(cpu.registers.f().is_zero());
    }

    #[test]
    fn test_srlr8() {
        let mut cpu = init_cpu_state();
        cpu.registers.set_b(0b11001111);
        let mut bus = MemoryBus::new(vec![0; 32768]);

        let cycles = exec_srlr8(&mut cpu, &mut bus, R8Register::B, &mut noop_tick);

        assert_eq!(cycles, 2);
        // Logical shift right: 0b11001111 >> 1 = 0b01100111
        assert_eq!(cpu.registers.b(), 0b01100111);
        assert!(cpu.registers.f().is_carry()); // Old bit 0
    }

    #[test]
    fn test_bitbr8_bit_set() {
        let mut cpu = init_cpu_state();
        cpu.registers.set_b(0b10101010);
        let mut bus = MemoryBus::new(vec![0; 32768]);

        let cycles = exec_bitbr8(&mut cpu, &mut bus, 1, R8Register::B, &mut noop_tick);

        assert_eq!(cycles, 2);
        // Bit 1 is 1
        assert!(!cpu.registers.f().is_zero());
        assert!(!cpu.registers.f().is_subtraction());
        assert!(cpu.registers.f().is_half_carry());
    }

    #[test]
    fn test_bitbr8_bit_clear() {
        let mut cpu = init_cpu_state();
        cpu.registers.set_b(0b10101010);
        let mut bus = MemoryBus::new(vec![0; 32768]);

        let cycles = exec_bitbr8(&mut cpu, &mut bus, 0, R8Register::B, &mut noop_tick);

        assert_eq!(cycles, 2);
        // Bit 0 is 0
        assert!(cpu.registers.f().is_zero());
    }

    #[test]
    fn test_bitbr8_all_bits() {
        let mut cpu = init_cpu_state();
        cpu.registers.set_b(0xFF);
        let mut bus = MemoryBus::new(vec![0; 32768]);

        for bit in 0..8 {
            let cycles = exec_bitbr8(&mut cpu, &mut bus, bit, R8Register::B, &mut noop_tick);
            assert_eq!(cycles, 2);
            assert!(!cpu.registers.f().is_zero(), "Bit {} should be set", bit);
        }
    }

    #[test]
    fn test_bitbr8_preserves_carry() {
        let mut cpu = init_cpu_state();
        cpu.registers.set_b(0b00000001);

        // Explicitly set carry before BIT
        cpu.registers.modify_f(|f| f.set_carry(true));

        let mut bus = MemoryBus::new(vec![0; 32768]);

        let cycles = exec_bitbr8(&mut cpu, &mut bus, 0, R8Register::B, &mut noop_tick);

        assert_eq!(cycles, 2);

        // Bit 0 is set → Z = 0
        assert!(!cpu.registers.f().is_zero());

        // BIT must:
        // - Clear N
        // - Set H
        // - Leave C unchanged
        assert!(!cpu.registers.f().is_subtraction());
        assert!(cpu.registers.f().is_half_carry());

        assert!(cpu.registers.f().is_carry());
    }

    #[test]
    fn test_resbr8() {
        let mut cpu = init_cpu_state();
        cpu.registers.set_b(0b11111111);
        let mut bus = MemoryBus::new(vec![0; 32768]);

        let cycles = exec_resbr8(&mut cpu, &mut bus, 3, R8Register::B, &mut noop_tick);

        assert_eq!(cycles, 2);
        // Clear bit 3: 0b11111111 -> 0b11110111
        assert_eq!(cpu.registers.b(), 0b11110111);
    }

    #[test]
    fn test_resbr8_multiple_bits() {
        let mut cpu = init_cpu_state();
        cpu.registers.set_b(0b11111111);
        let mut bus = MemoryBus::new(vec![0; 32768]);

        exec_resbr8(&mut cpu, &mut bus, 0, R8Register::B, &mut noop_tick);
        exec_resbr8(&mut cpu, &mut bus, 2, R8Register::B, &mut noop_tick);
        exec_resbr8(&mut cpu, &mut bus, 4, R8Register::B, &mut noop_tick);
        exec_resbr8(&mut cpu, &mut bus, 6, R8Register::B, &mut noop_tick);

        assert_eq!(cpu.registers.b(), 0b10101010);
    }

    #[test]
    fn test_setbr8() {
        let mut cpu = init_cpu_state();
        cpu.registers.set_b(0b00000000);
        let mut bus = MemoryBus::new(vec![0; 32768]);

        let cycles = exec_setbr8(&mut cpu, &mut bus, 3, R8Register::B, &mut noop_tick);

        assert_eq!(cycles, 2);
        // Set bit 3: 0b00000000 -> 0b00001000
        assert_eq!(cpu.registers.b(), 0b00001000);
    }

    #[test]
    fn test_setbr8_multiple_bits() {
        let mut cpu = init_cpu_state();
        cpu.registers.set_b(0b00000000);
        let mut bus = MemoryBus::new(vec![0; 32768]);

        exec_setbr8(&mut cpu, &mut bus, 0, R8Register::B, &mut noop_tick);
        exec_setbr8(&mut cpu, &mut bus, 2, R8Register::B, &mut noop_tick);
        exec_setbr8(&mut cpu, &mut bus, 4, R8Register::B, &mut noop_tick);
        exec_setbr8(&mut cpu, &mut bus, 6, R8Register::B, &mut noop_tick);

        // Set bits 0, 2, 4, 6: 0b00000000 -> 0b01010101
        assert_eq!(cpu.registers.b(), 0b01010101);
    }
