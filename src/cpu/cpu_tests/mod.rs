    pub(super) use super::*;
    pub(super) use crate::memory::MemoryBus;
    pub(super) use crate::cpu::instructions::{Instruction, R8Register, R16Register, R16Mem, Condition, CBInstruction};

    pub(super) fn noop_tick(_: &mut [u8; 128]) {}

    #[test]
    fn test_cpu_create() {
        let cpu = CPU::new();
        assert_eq!(cpu.state.registers.pc, 0x0100);
        assert_eq!(cpu.state.registers.sp, 0xFFFE);
    }

    #[test]
    fn test_cpu_reset() {
        let mut cpu = CPU::new();
        cpu.state.registers.pc = 0x1234;
        cpu.reset();
        assert_eq!(cpu.state.registers.pc, 0x0100);
    }

    #[test]
    fn test_cpu_cycles() {
        let cpu = CPU::new();
        assert_eq!(cpu.cycles(), 0);
    }

    #[test]
    fn test_state_getters() {
        let cpu = CPU::new();
        assert_eq!(cpu.state().registers.pc, 0x0100);
        assert!(!cpu.state().ime);
    }

    #[test]
    fn test_state_mut() {
        let mut cpu = CPU::new();
        cpu.state_mut().ime = true;
        assert!(cpu.state().ime);
    }

    mod data_transfer;
    mod alu_r8;
    mod alu_misc;
    mod jumps;
    mod stack;
    mod cb;
    mod integration;
