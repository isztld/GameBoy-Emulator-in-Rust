/// GameBoy CPU implementation
///
/// The CPU is based on the SM83, a 8-bit CPU compatible with GBZ80.

use crate::memory::MemoryBus;
use crate::cpu::{CPUState};
use crate::cpu::instructions::{Instruction};
use crate::cpu::decode::decode_instruction;
use crate::cpu::exec::execute_instruction;

/// GameBoy CPU
#[derive(Debug)]
pub struct CPU {
    state: CPUState,
    cycles: u32, // Total cycles executed
    halted: bool,
    stop_halt: bool,
}

impl CPU {
    /// Create a new CPU
    pub fn new() -> Self {
        let mut cpu = CPU {
            state: CPUState::new(),
            cycles: 0,
            halted: false,
            stop_halt: false,
        };
        cpu.reset();
        cpu
    }

    /// Reset the CPU to power-on state
    pub fn reset(&mut self) {
        self.state.registers.pc = 0x0100;
        self.state.registers.sp = 0xFFFE;
        self.state.registers.af = 0x01B0;
        self.state.registers.bc = 0x0013;
        self.state.registers.de = 0x00D8;
        self.state.registers.hl = 0x014D;
        self.state.ime = false;
        self.halted = false;
        self.stop_halt = false;
        self.cycles = 0;
    }

    /// Get current CPU state
    pub fn state(&self) -> &CPUState {
        &self.state
    }

    /// Get mutable CPU state
    pub fn state_mut(&mut self) -> &mut CPUState {
        &mut self.state
    }

    /// Get total cycles executed
    pub fn cycles(&self) -> u32 {
        self.cycles
    }

    /// Execute one instruction
    pub fn execute(&mut self, bus: &mut MemoryBus) -> u32 {
        if self.halted {
            // While halted, only interrupt handling consumes cycles
            // Return 1 cycle for HALT
            self.cycles += 1;
            return 1;
        }

        // Read opcode at PC
        let pc = self.state.registers.pc;
        let opcode = bus.read(pc);

        // Decode instruction
        let (instruction, opcode_bytes) = decode_instruction(&self.state, bus, pc, opcode);

        // Execute instruction
        let cycles = execute_instruction(&mut self.state, bus, instruction);

        // Update cycle count
        self.cycles += cycles as u32;

        // For instructions that modify PC themselves (jumps, calls, returns, RST),
        // we don't add opcode_bytes. The instruction's execute_instruction handles it.
        // For other instructions, we add opcode_bytes to advance PC.
        match instruction {
            Instruction::JrImm8 { .. }
            | Instruction::JrCondImm8 { .. }
            | Instruction::JpImm16 { .. }
            | Instruction::JpCondImm16 { .. }
            | Instruction::JpHl
            | Instruction::CallImm16 { .. }
            | Instruction::CallCondImm16 { .. }
            | Instruction::RET
            | Instruction::RetCond { .. }
            | Instruction::RETI
            | Instruction::RST { .. }
            | Instruction::LdHlSpImm8 { .. }
            | Instruction::LdSpHl => {
                // PC already set by execute_instruction, nothing to do
            }
            _ => {
                self.state.registers.pc += opcode_bytes as u16;
            }
        }

        cycles as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
