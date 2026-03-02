/// GameBoy CPU implementation
///
/// The CPU is based on the SM83, a 8-bit CPU compatible with GBZ80.

use crate::memory::MemoryBus;
use crate::cpu::CPUState;
use crate::cpu::instructions::Instruction;
use crate::cpu::decode::decode_instruction;
use crate::cpu::exec::execute_instruction;

/// Interrupt vectors (address jumped to when servicing each interrupt).
const INT_VBLANK:  u16 = 0x0040;
const INT_STAT:    u16 = 0x0048;
const INT_TIMER:   u16 = 0x0050;
const INT_SERIAL:  u16 = 0x0058;
const INT_JOYPAD:  u16 = 0x0060;

/// Bit masks for the IE/IF registers.
const INT_BIT_VBLANK:  u8 = 1 << 0;
const INT_BIT_STAT:    u8 = 1 << 1;
const INT_BIT_TIMER:   u8 = 1 << 2;
const INT_BIT_SERIAL:  u8 = 1 << 3;
const INT_BIT_JOYPAD:  u8 = 1 << 4;

#[derive(Debug)]
pub struct CPU {
    state: CPUState,
    /// Total T-cycles executed (u64 avoids overflow at 4 MHz).
    cycles: u64,
    halted: bool,
    stopped: bool,
}

impl CPU {
    pub fn new() -> Self {
        let mut cpu = CPU {
            state: CPUState::new(),
            cycles: 0,
            halted: false,
            stopped: false,
        };
        cpu.reset();
        cpu
    }

    pub fn reset(&mut self) {
        self.state.registers.pc = 0x0100;
        self.state.registers.sp = 0xFFFE;
        self.state.registers.af = 0x01B0;
        self.state.registers.bc = 0x0013;
        self.state.registers.de = 0x00D8;
        self.state.registers.hl = 0x014D;
        self.state.ime = false;
        self.state.ime_pending = false;
        self.halted = false;
        self.stopped = false;
        self.cycles = 0;
    }

    pub fn state(&self) -> &CPUState { &self.state }
    pub fn state_mut(&mut self) -> &mut CPUState { &mut self.state }
    pub fn cycles(&self) -> u64 { self.cycles }

    /// Execute one step: service a pending interrupt OR execute one instruction.
    /// Returns the number of machine cycles consumed (1 machine cycle = 4 T-cycles).
    pub fn execute(&mut self, bus: &mut MemoryBus) -> u32 {
        // --- Interrupt check ---------------------------------------------------
        // An interrupt can un-halt the CPU regardless of IME.
        let pending = bus.ie & bus.read(0xFF0F) & 0x1F;
        if pending != 0 {
            self.halted = false; // always wake from HALT on any pending interrupt
        }

        if self.state.ime && pending != 0 {
            let cycles = self.service_interrupt(bus, pending);
            self.cycles += cycles as u64;
            return cycles;
        }

        // --- EI delay: promote ime_pending → ime ----------------------------------
        // EI enables interrupts after the *following* instruction, so we promote
        // the pending flag here (after the interrupt check) so that the NEXT call
        // to execute() will see ime=true when it checks for pending interrupts.
        if self.state.ime_pending {
            self.state.ime = true;
            self.state.ime_pending = false;
        }

        // --- HALT / STOP ------------------------------------------------------
        if self.halted || self.stopped {
            // Spin for 1 machine cycle, waiting for an interrupt to arrive.
            self.cycles += 1;
            return 1;
        }

        // --- Normal instruction fetch / decode / execute ----------------------
        let pc = self.state.registers.pc;
        let opcode = bus.read(pc);
        let (instruction, opcode_bytes) = decode_instruction(&self.state, bus, pc, opcode);

        // Advance PC past this instruction before executing, so that relative
        // jumps and calls that read PC (e.g. RST return address) see the correct
        // "next instruction" address.  Jump/call/return instructions will
        // overwrite PC themselves.
        self.state.registers.pc = pc.wrapping_add(opcode_bytes as u16);

        let cycles: u32 = match instruction {
            Instruction::HALT => {
                self.halted = true;
                1
            }
            Instruction::STOP => {
                self.stopped = true;
                1
            }
            instr => execute_instruction(&mut self.state, bus, instr),
        };

        self.cycles += cycles as u64;
        cycles
    }

    /// Service the highest-priority pending interrupt.
    /// Clears the interrupt bit in IF, disables IME, and pushes PC onto the stack.
    /// Returns the number of machine cycles consumed (5).
    fn service_interrupt(&mut self, bus: &mut MemoryBus, pending: u8) -> u32 {
        self.state.ime = false;

        // Find the highest-priority interrupt (lowest bit number).
        let (bit, vector) = if pending & INT_BIT_VBLANK != 0 {
            (INT_BIT_VBLANK, INT_VBLANK)
        } else if pending & INT_BIT_STAT != 0 {
            (INT_BIT_STAT, INT_STAT)
        } else if pending & INT_BIT_TIMER != 0 {
            (INT_BIT_TIMER, INT_TIMER)
        } else if pending & INT_BIT_SERIAL != 0 {
            (INT_BIT_SERIAL, INT_SERIAL)
        } else {
            (INT_BIT_JOYPAD, INT_JOYPAD)
        };

        // Clear the IF bit for this interrupt.
        let if_val = bus.read(0xFF0F);
        bus.write(0xFF0F, if_val & !bit);

        // Push current PC onto the stack (2 machine cycles).
        let pc = self.state.registers.pc;
        let sp = self.state.registers.sp;
        bus.write(sp.wrapping_sub(1), (pc >> 8) as u8);
        bus.write(sp.wrapping_sub(2), (pc & 0xFF) as u8);
        self.state.registers.sp = sp.wrapping_sub(2);

        // Jump to interrupt vector.
        self.state.registers.pc = vector;

        // ISR dispatch takes 5 machine cycles total.
        5
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::MemoryBus;
    use crate::cpu::instructions::{R8Register, CBInstruction};

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

    // ==================== DATA TRANSFER INSTRUCTIONS ====================

    #[test]
    fn test_ld_r16_imm16_bc() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        let instruction = crate::cpu::instructions::Instruction::LdR16Imm16 {
            dest: crate::cpu::instructions::R16Register::BC,
            value: 0xABCD,
        };
        let cycles = crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);
        assert_eq!(cycles, 3);
        assert_eq!(cpu.state.registers.bc, 0xABCD);
    }

    #[test]
    fn test_ld_r16_imm16_de() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        let instruction = crate::cpu::instructions::Instruction::LdR16Imm16 {
            dest: crate::cpu::instructions::R16Register::DE,
            value: 0x1234,
        };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);
        assert_eq!(cpu.state.registers.de, 0x1234);
    }

    #[test]
    fn test_ld_r16_imm16_hl() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        let instruction = crate::cpu::instructions::Instruction::LdR16Imm16 {
            dest: crate::cpu::instructions::R16Register::HL,
            value: 0x5678,
        };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);
        assert_eq!(cpu.state.registers.hl, 0x5678);
    }

    #[test]
    fn test_ld_r16_imm16_sp() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        let instruction = crate::cpu::instructions::Instruction::LdR16Imm16 {
            dest: crate::cpu::instructions::R16Register::SP,
            value: 0xC000,
        };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);
        assert_eq!(cpu.state.registers.sp, 0xC000);
    }

    #[test]
    fn test_ld_r8_imm8() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        let instruction = crate::cpu::instructions::Instruction::LdR8Imm8 {
            dest: crate::cpu::instructions::R8Register::A,
            value: 0x42,
        };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);
        assert_eq!(cpu.state.registers.a(), 0x42);
    }

    #[test]
    fn test_ld_r8_r8_all_combinations() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);

        // Set B = 0x12
        cpu.state.registers.set_b(0x12);

        // LD C, B
        let instruction = crate::cpu::instructions::Instruction::LdR8R8 {
            dest: crate::cpu::instructions::R8Register::C,
            src: crate::cpu::instructions::R8Register::B,
        };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);
        assert_eq!(cpu.state.registers.c(), 0x12);

        // LD D, C
        let instruction = crate::cpu::instructions::Instruction::LdR8R8 {
            dest: crate::cpu::instructions::R8Register::D,
            src: crate::cpu::instructions::R8Register::C,
        };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);
        assert_eq!(cpu.state.registers.d(), 0x12);

        // LD E, D
        let instruction = crate::cpu::instructions::Instruction::LdR8R8 {
            dest: crate::cpu::instructions::R8Register::E,
            src: crate::cpu::instructions::R8Register::D,
        };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);
        assert_eq!(cpu.state.registers.e(), 0x12);

        // LD H, E
        let instruction = crate::cpu::instructions::Instruction::LdR8R8 {
            dest: crate::cpu::instructions::R8Register::H,
            src: crate::cpu::instructions::R8Register::E,
        };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);
        assert_eq!(cpu.state.registers.h(), 0x12);

        // LD L, H
        let instruction = crate::cpu::instructions::Instruction::LdR8R8 {
            dest: crate::cpu::instructions::R8Register::L,
            src: crate::cpu::instructions::R8Register::H,
        };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);
        assert_eq!(cpu.state.registers.l(), 0x12);

        // LD A, L
        let instruction = crate::cpu::instructions::Instruction::LdR8R8 {
            dest: crate::cpu::instructions::R8Register::A,
            src: crate::cpu::instructions::R8Register::L,
        };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);
        assert_eq!(cpu.state.registers.a(), 0x12);
    }

    #[test]
    fn test_ld_ind_r16_a_bc() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.bc = 0x8000;
        cpu.state.registers.set_a(0xAB);

        let instruction = crate::cpu::instructions::Instruction::LdIndR16A {
            src: crate::cpu::instructions::R16Mem::BC,
        };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(bus.read(0x8000), 0xAB);
    }

    #[test]
    fn test_ld_ind_r16_a_de() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.de = 0x9000;
        cpu.state.registers.set_a(0xCD);

        let instruction = crate::cpu::instructions::Instruction::LdIndR16A {
            src: crate::cpu::instructions::R16Mem::DE,
        };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(bus.read(0x9000), 0xCD);
    }

    #[test]
    fn test_ld_ind_r16_a_hl_plus() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.hl = 0xA000;
        cpu.state.registers.set_a(0xEF);

        let instruction = crate::cpu::instructions::Instruction::LdIndR16A {
            src: crate::cpu::instructions::R16Mem::HLPlus,
        };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(bus.read(0xA000), 0xEF);
        assert_eq!(cpu.state.registers.hl, 0xA001); // HL incremented
    }

    #[test]
    fn test_ld_ind_r16_a_hl_minus() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.hl = 0xA000;
        cpu.state.registers.set_a(0xEF);

        let instruction = crate::cpu::instructions::Instruction::LdIndR16A {
            src: crate::cpu::instructions::R16Mem::HLMinus,
        };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(bus.read(0xA000), 0xEF);
        assert_eq!(cpu.state.registers.hl, 0x9FFF); // HL decremented
    }

    #[test]
    fn test_ld_a_ind_r16_bc() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.bc = 0x8000;
        bus.write(0x8000, 0x55);

        let instruction = crate::cpu::instructions::Instruction::LdAIndR16 {
            dest: crate::cpu::instructions::R16Mem::BC,
        };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cpu.state.registers.a(), 0x55);
    }

    #[test]
    fn test_ld_a_ind_r16_hl_plus() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.hl = 0xA000;
        bus.write(0xA000, 0x66);

        let instruction = crate::cpu::instructions::Instruction::LdAIndR16 {
            dest: crate::cpu::instructions::R16Mem::HLPlus,
        };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cpu.state.registers.a(), 0x66);
        assert_eq!(cpu.state.registers.hl, 0xA001);
    }

    #[test]
    fn test_ld_ind_imm16_sp() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.sp = 0xC000;

        let instruction = Instruction::LdIndImm16Sp { address: 0xD000 };
        execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(bus.read(0xD000), 0x00); // SP low byte at address
        assert_eq!(bus.read(0xD001), 0xC0); // SP high byte at address + 1
        assert_eq!(cpu.state.registers.sp, 0xC000); // SP itself unchanged
    }

    #[test]
    fn test_ld_ind_imm16_a() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.set_a(0x77);

        let instruction = crate::cpu::instructions::Instruction::LdIndImm16A {
            address: 0xC000,
        };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(bus.read(0xC000), 0x77);
    }

    #[test]
    fn test_ld_a_ind_imm16() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        bus.write(0xC000, 0x88);

        let instruction = crate::cpu::instructions::Instruction::LdAIndImm16 {
            address: 0xC000,
        };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cpu.state.registers.a(), 0x88);
    }

    #[test]
    fn test_ldh_ind_imm8_a() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.set_a(0x99);

        let instruction = crate::cpu::instructions::Instruction::LdhIndImm8A {
            address: 0x10,
        };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(bus.read(0xFF10), 0x99);
    }

    #[test]
    fn test_ldh_a_ind_imm8() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        bus.write(0xFF10, 0xAA);

        let instruction = crate::cpu::instructions::Instruction::LdhAIndImm8 {
            address: 0x10,
        };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cpu.state.registers.a(), 0xAA);
    }

    #[test]
    fn test_ldh_ind_c_a() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.set_c(0x01); // Use 0xFF01 (SB) which stores full value
        cpu.state.registers.set_a(0xBB);

        let instruction = crate::cpu::instructions::Instruction::LdhIndCA;
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(bus.read(0xFF01), 0xBB);
    }

    #[test]
    fn test_ldh_a_c() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);

        cpu.state.registers.set_c(0x01); // Use 0xFF01 (SB) which stores full value
        bus.write(0xFF01, 0xCC);

        let instruction = crate::cpu::instructions::Instruction::LdhAC;
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cpu.state.registers.a(), 0xCC);
    }

    // ==================== ARITHMETIC INSTRUCTIONS ====================

    #[test]
    fn test_add_a_r8() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.set_a(0x10);
        cpu.state.registers.set_b(0x20);

        let instruction = crate::cpu::instructions::Instruction::AddAR8 {
            reg: crate::cpu::instructions::R8Register::B,
        };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cpu.state.registers.a(), 0x30);
        assert!(!cpu.state.registers.f().is_zero());
        assert!(!cpu.state.registers.f().is_carry());
        assert!(!cpu.state.registers.f().is_half_carry());
    }

    #[test]
    fn test_add_a_r8_zero() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.set_a(0x00);
        cpu.state.registers.set_b(0x00);

        let instruction = crate::cpu::instructions::Instruction::AddAR8 {
            reg: crate::cpu::instructions::R8Register::B,
        };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cpu.state.registers.a(), 0x00);
        assert!(cpu.state.registers.f().is_zero());
    }

    #[test]
    fn test_add_a_r8_carry() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.set_a(0xFF);
        cpu.state.registers.set_b(0x01);

        let instruction = crate::cpu::instructions::Instruction::AddAR8 {
            reg: crate::cpu::instructions::R8Register::B,
        };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cpu.state.registers.a(), 0x00);
        assert!(cpu.state.registers.f().is_carry());
        assert!(cpu.state.registers.f().is_zero()); // Result is 0, so Z flag is set
    }

    #[test]
    fn test_add_a_r8_half_carry() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.set_a(0x0F);
        cpu.state.registers.set_b(0x01);

        let instruction = crate::cpu::instructions::Instruction::AddAR8 {
            reg: crate::cpu::instructions::R8Register::B,
        };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cpu.state.registers.a(), 0x10);
        assert!(cpu.state.registers.f().is_half_carry());
    }

    #[test]
    fn test_adc_a_r8_no_carry() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.set_a(0x10);
        cpu.state.registers.set_b(0x20);
        cpu.state.registers.f_mut().set_carry(false); // Clear carry from reset

        let instruction = crate::cpu::instructions::Instruction::AdcAR8 {
            reg: crate::cpu::instructions::R8Register::B,
        };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cpu.state.registers.a(), 0x30);
    }

    #[test]
    fn test_adc_a_r8_with_carry() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.set_a(0x10);
        cpu.state.registers.set_b(0x20);
        cpu.state.registers.f_mut().set_carry(true);

        let instruction = crate::cpu::instructions::Instruction::AdcAR8 {
            reg: crate::cpu::instructions::R8Register::B,
        };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cpu.state.registers.a(), 0x31); // 0x10 + 0x20 + 1
    }

    #[test]
    fn test_sub_a_r8() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.set_a(0x30);
        cpu.state.registers.set_b(0x10);

        let instruction = crate::cpu::instructions::Instruction::SubAR8 {
            reg: crate::cpu::instructions::R8Register::B,
        };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cpu.state.registers.a(), 0x20);
        assert!(cpu.state.registers.f().is_subtraction());
    }

    #[test]
    fn test_sub_a_r8_carry() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.set_a(0x10);
        cpu.state.registers.set_b(0x30);

        let instruction = crate::cpu::instructions::Instruction::SubAR8 {
            reg: crate::cpu::instructions::R8Register::B,
        };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cpu.state.registers.a(), 0xE0);
        assert!(cpu.state.registers.f().is_carry());
    }

    #[test]
    fn test_sbc_a_r8_no_carry() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.set_a(0x30);
        cpu.state.registers.set_b(0x10);
        cpu.state.registers.f_mut().set(0); // clear all flags

        let instruction = crate::cpu::instructions::Instruction::SbcAR8 {
            reg: crate::cpu::instructions::R8Register::B,
        };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cpu.state.registers.a(), 0x20);
    }

    #[test]
    fn test_sbc_a_r8_with_carry() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.set_a(0x30);
        cpu.state.registers.set_b(0x10);
        cpu.state.registers.f_mut().set_carry(true);

        let instruction = crate::cpu::instructions::Instruction::SbcAR8 {
            reg: crate::cpu::instructions::R8Register::B,
        };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cpu.state.registers.a(), 0x1F); // 0x30 - 0x10 - 1
    }

    #[test]
    fn test_and_a_r8() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.set_a(0xFF);
        cpu.state.registers.set_b(0x0F);

        let instruction = crate::cpu::instructions::Instruction::AndAR8 {
            reg: crate::cpu::instructions::R8Register::B,
        };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cpu.state.registers.a(), 0x0F);
        assert!(cpu.state.registers.f().is_half_carry());
        assert!(!cpu.state.registers.f().is_zero());
    }

    #[test]
    fn test_and_a_r8_zero() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.set_a(0xFF);
        cpu.state.registers.set_b(0x00);

        let instruction = crate::cpu::instructions::Instruction::AndAR8 {
            reg: crate::cpu::instructions::R8Register::B,
        };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cpu.state.registers.a(), 0x00);
        assert!(cpu.state.registers.f().is_zero());
    }

    #[test]
    fn test_xor_a_r8() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.set_a(0xFF);
        cpu.state.registers.set_b(0x0F);

        let instruction = crate::cpu::instructions::Instruction::XorAR8 {
            reg: crate::cpu::instructions::R8Register::B,
        };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cpu.state.registers.a(), 0xF0);
        assert!(!cpu.state.registers.f().is_zero());
    }

    #[test]
    fn test_xor_a_r8_same() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.set_a(0x55);
        cpu.state.registers.set_b(0x55);

        let instruction = crate::cpu::instructions::Instruction::XorAR8 {
            reg: crate::cpu::instructions::R8Register::B,
        };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cpu.state.registers.a(), 0x00);
        assert!(cpu.state.registers.f().is_zero());
    }

    #[test]
    fn test_or_a_r8() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.set_a(0xF0);
        cpu.state.registers.set_b(0x0F);

        let instruction = crate::cpu::instructions::Instruction::OrAR8 {
            reg: crate::cpu::instructions::R8Register::B,
        };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cpu.state.registers.a(), 0xFF);
        assert!(!cpu.state.registers.f().is_zero());
    }

    #[test]
    fn test_or_a_r8_zero() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.set_a(0x00);
        cpu.state.registers.set_b(0x00);

        let instruction = crate::cpu::instructions::Instruction::OrAR8 {
            reg: crate::cpu::instructions::R8Register::B,
        };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cpu.state.registers.a(), 0x00);
        assert!(cpu.state.registers.f().is_zero());
    }

    #[test]
    fn test_cp_a_r8_no_borrow() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.set_a(0x30);
        cpu.state.registers.set_b(0x10);

        let instruction = crate::cpu::instructions::Instruction::CpAR8 {
            reg: crate::cpu::instructions::R8Register::B,
        };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cpu.state.registers.a(), 0x30); // A unchanged
        assert!(!cpu.state.registers.f().is_carry());
        assert!(!cpu.state.registers.f().is_zero());
    }

    #[test]
    fn test_cp_a_r8_borrow() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.set_a(0x10);
        cpu.state.registers.set_b(0x30);

        let instruction = crate::cpu::instructions::Instruction::CpAR8 {
            reg: crate::cpu::instructions::R8Register::B,
        };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cpu.state.registers.a(), 0x10); // A unchanged
        assert!(cpu.state.registers.f().is_carry());
    }

    #[test]
    fn test_cp_a_r8_equal() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.set_a(0x42);
        cpu.state.registers.set_b(0x42);

        let instruction = crate::cpu::instructions::Instruction::CpAR8 {
            reg: crate::cpu::instructions::R8Register::B,
        };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cpu.state.registers.a(), 0x42); // A unchanged
        assert!(cpu.state.registers.f().is_zero());
    }

    #[test]
    fn test_add_a_imm8() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.set_a(0x10);

        let instruction = crate::cpu::instructions::Instruction::AddAImm8 { value: 0x20 };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cpu.state.registers.a(), 0x30);
    }

    #[test]
    fn test_adc_a_imm8() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.set_a(0x10);
        cpu.state.registers.f_mut().set_carry(true);

        let instruction = crate::cpu::instructions::Instruction::AdcAImm8 { value: 0x20 };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cpu.state.registers.a(), 0x31); // 0x10 + 0x20 + 1
    }

    #[test]
    fn test_sub_a_imm8() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.set_a(0x30);

        let instruction = crate::cpu::instructions::Instruction::SubAImm8 { value: 0x10 };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cpu.state.registers.a(), 0x20);
        assert!(cpu.state.registers.f().is_subtraction());
    }

    #[test]
    fn test_sbc_a_imm8() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.set_a(0x30);
        cpu.state.registers.f_mut().set(0); // clear all flags

        let instruction = crate::cpu::instructions::Instruction::SbcAImm8 { value: 0x10 };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cpu.state.registers.a(), 0x20);
    }

    #[test]
    fn test_and_a_imm8() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.set_a(0xFF);

        let instruction = crate::cpu::instructions::Instruction::AndAImm8 { value: 0x0F };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cpu.state.registers.a(), 0x0F);
    }

    #[test]
    fn test_xor_a_imm8() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.set_a(0xFF);

        let instruction = crate::cpu::instructions::Instruction::XorAImm8 { value: 0x0F };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cpu.state.registers.a(), 0xF0);
    }

    #[test]
    fn test_or_a_imm8() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.set_a(0xF0);

        let instruction = crate::cpu::instructions::Instruction::OrAImm8 { value: 0x0F };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cpu.state.registers.a(), 0xFF);
    }

    #[test]
    fn test_cp_a_imm8() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.set_a(0x30);

        let instruction = crate::cpu::instructions::Instruction::CpAImm8 { value: 0x10 };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cpu.state.registers.a(), 0x30); // A unchanged
        assert!(!cpu.state.registers.f().is_carry());
    }

    // ==================== REGISTER INSTRUCTIONS ====================

    #[test]
    fn test_inc_r16_bc() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.bc = 0x1234;

        let instruction = crate::cpu::instructions::Instruction::IncR16 {
            reg: crate::cpu::instructions::R16Register::BC,
        };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cpu.state.registers.bc, 0x1235);
    }

    #[test]
    fn test_inc_r16_sp() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.sp = 0xC000;

        let instruction = crate::cpu::instructions::Instruction::IncR16 {
            reg: crate::cpu::instructions::R16Register::SP,
        };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cpu.state.registers.sp, 0xC001);
    }

    #[test]
    fn test_dec_r16_bc() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.bc = 0x1234;

        let instruction = crate::cpu::instructions::Instruction::DecR16 {
            reg: crate::cpu::instructions::R16Register::BC,
        };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cpu.state.registers.bc, 0x1233);
    }

    #[test]
    fn test_inc_r8() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.set_a(0x0F);

        let instruction = crate::cpu::instructions::Instruction::IncR8 {
            reg: crate::cpu::instructions::R8Register::A,
        };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cpu.state.registers.a(), 0x10);
        assert!(!cpu.state.registers.f().is_zero());
    }

    #[test]
    fn test_inc_r8_zero() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.set_a(0xFF);

        let instruction = crate::cpu::instructions::Instruction::IncR8 {
            reg: crate::cpu::instructions::R8Register::A,
        };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cpu.state.registers.a(), 0x00);
        assert!(cpu.state.registers.f().is_zero());
    }

    #[test]
    fn test_dec_r8() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.set_a(0x10);

        let instruction = crate::cpu::instructions::Instruction::DecR8 {
            reg: crate::cpu::instructions::R8Register::A,
        };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cpu.state.registers.a(), 0x0F);
        assert!(!cpu.state.registers.f().is_zero());
        assert!(cpu.state.registers.f().is_subtraction());
    }

    #[test]
    fn test_inc_hl_memory() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.hl = 0xC000;
        bus.write(0xC000, 0x42);

        let instruction = crate::cpu::instructions::Instruction::IncR8 {
            reg: crate::cpu::instructions::R8Register::HL,
        };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(bus.read(0xC000), 0x43);
    }

    #[test]
    fn test_dec_hl_memory() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.hl = 0xC000;
        bus.write(0xC000, 0x42);

        let instruction = crate::cpu::instructions::Instruction::DecR8 {
            reg: crate::cpu::instructions::R8Register::HL,
        };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(bus.read(0xC000), 0x41);
    }

    #[test]
    fn test_add_hl_r16_bc() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.hl = 0x1234;
        cpu.state.registers.bc = 0x0010;

        let instruction = crate::cpu::instructions::Instruction::AddHlR16 {
            reg: crate::cpu::instructions::R16Register::BC,
        };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cpu.state.registers.hl, 0x1244);
        assert!(!cpu.state.registers.f().is_carry());
        assert!(!cpu.state.registers.f().is_half_carry());
    }

    #[test]
    fn test_add_hl_r16_carry() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.hl = 0xFFFF;
        cpu.state.registers.bc = 0x0001;

        let instruction = crate::cpu::instructions::Instruction::AddHlR16 {
            reg: crate::cpu::instructions::R16Register::BC,
        };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cpu.state.registers.hl, 0x0000);
        assert!(cpu.state.registers.f().is_carry());
    }

    #[test]
    fn test_add_hl_r16_half_carry() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.hl = 0x0FFF;
        cpu.state.registers.bc = 0x0001;

        let instruction = crate::cpu::instructions::Instruction::AddHlR16 {
            reg: crate::cpu::instructions::R16Register::BC,
        };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cpu.state.registers.hl, 0x1000);
        assert!(cpu.state.registers.f().is_half_carry());
    }

    // ==================== ROTATE/SHIFT INSTRUCTIONS ====================

    #[test]
    fn test_rlca() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.set_a(0b10110001); // 0xB1

        let instruction = crate::cpu::instructions::Instruction::RLCA;
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cpu.state.registers.a(), 0b01100011); // 0x63
        assert!(cpu.state.registers.f().is_carry()); // MSB was 1
        assert!(!cpu.state.registers.f().is_zero());
    }

    #[test]
    fn test_rrca() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.set_a(0b10110001); // 0xB1

        let instruction = crate::cpu::instructions::Instruction::RRCA;
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cpu.state.registers.a(), 0b11011000); // 0xD8
        assert!(cpu.state.registers.f().is_carry()); // LSB was 1
        assert!(!cpu.state.registers.f().is_zero());
    }

    #[test]
    fn test_rla() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.set_a(0b10110001); // 0xB1
        cpu.state.registers.f_mut().set_carry(true);

        let instruction = crate::cpu::instructions::Instruction::RLA;
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cpu.state.registers.a(), 0b01100011); // 0x63
        assert!(cpu.state.registers.f().is_carry()); // MSB was 1
    }

    #[test]
    fn test_rra() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.set_a(0b10110001); // 0xB1
        cpu.state.registers.f_mut().set_carry(true);

        let instruction = crate::cpu::instructions::Instruction::RRA;
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cpu.state.registers.a(), 0b11011000); // 0xD8
        assert!(cpu.state.registers.f().is_carry()); // LSB was 1
    }

    #[test]
    fn test_daa() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.set_a(0x19); // 19 in BCD
        cpu.state.registers.f_mut().set(0);

        let instruction = crate::cpu::instructions::Instruction::DAA;
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        // 0x19 has both nibbles <= 9, so no adjustment needed
        assert_eq!(cpu.state.registers.a(), 0x19);
    }

    #[test]
    fn daa_half_carry_adjust() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.set_a(0x0A);
        let f = cpu.state.registers.f_mut();
        f.set_half_carry(true);
        f.set_subtraction(false);
        f.set_carry(false);

        let instruction = crate::cpu::instructions::Instruction::DAA;
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cpu.state.registers.a(), 0x10);
    }

    #[test]
    fn daa_full_carry_adjust() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.set_a(0x9A);
        let f = cpu.state.registers.f_mut();
        f.set_subtraction(false);
        f.set_half_carry(false);
        f.set_carry(false);

        let instruction = crate::cpu::instructions::Instruction::DAA;
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cpu.state.registers.a(), 0x00);
        assert!(cpu.state.registers.f().is_carry());
        assert!(cpu.state.registers.f().is_zero());
    }

    #[test]
    fn test_cpl() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.set_a(0x55);

        let instruction = crate::cpu::instructions::Instruction::CPL;
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cpu.state.registers.a(), 0xAA);
        assert!(cpu.state.registers.f().is_subtraction());
        assert!(cpu.state.registers.f().is_half_carry());
    }

    #[test]
    fn test_scf() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.f_mut().set_carry(false);

        let instruction = crate::cpu::instructions::Instruction::SCF;
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert!(cpu.state.registers.f().is_carry());
        assert!(!cpu.state.registers.f().is_subtraction());
    }

    #[test]
    fn test_ccf() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.f_mut().set_carry(false);

        let instruction = crate::cpu::instructions::Instruction::CCF;
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert!(cpu.state.registers.f().is_carry()); // Flip carry

        cpu.state.registers.f_mut().set_carry(true);
        let instruction = crate::cpu::instructions::Instruction::CCF;
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert!(!cpu.state.registers.f().is_carry()); // Flip carry
    }

    // ==================== JUMP/CONTROL FLOW INSTRUCTIONS ====================

    #[test]
    fn test_jr_imm8() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.pc = 0x1000;

        let instruction = crate::cpu::instructions::Instruction::JrImm8 { offset: 0x05 };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cpu.state.registers.pc, 0x1005);
    }

    #[test]
    fn test_jr_imm8_negative() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.pc = 0x1005;

        let instruction = crate::cpu::instructions::Instruction::JrImm8 { offset: -0x03 };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cpu.state.registers.pc, 0x1002);
    }

    #[test]
    fn test_jr_cond_nz_taken() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.pc = 0x1000;
        cpu.state.registers.f_mut().set_zero(false); // Not zero

        let instruction = crate::cpu::instructions::Instruction::JrCondImm8 {
            cond: crate::cpu::instructions::Condition::NZ,
            offset: 0x05,
        };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cpu.state.registers.pc, 0x1005);
    }

    #[test]
    fn test_jr_cond_nz_not_taken() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.pc = 0x1000;
        cpu.state.registers.f_mut().set_zero(true); // Zero

        let instruction = crate::cpu::instructions::Instruction::JrCondImm8 {
            cond: crate::cpu::instructions::Condition::NZ,
            offset: 0x05,
        };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cpu.state.registers.pc, 0x1000); // Not taken
    }

    #[test]
    fn test_jr_cond_z_taken() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.pc = 0x1000;
        cpu.state.registers.f_mut().set_zero(true); // Zero

        let instruction = crate::cpu::instructions::Instruction::JrCondImm8 {
            cond: crate::cpu::instructions::Condition::Z,
            offset: 0x05,
        };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cpu.state.registers.pc, 0x1005);
    }

    #[test]
    fn test_jr_cond_nc_taken() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.pc = 0x1000;
        cpu.state.registers.f_mut().set_carry(false); // Not carry

        let instruction = crate::cpu::instructions::Instruction::JrCondImm8 {
            cond: crate::cpu::instructions::Condition::NC,
            offset: 0x05,
        };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cpu.state.registers.pc, 0x1005);
    }

    #[test]
    fn test_jp_imm16() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.pc = 0x1000;

        let instruction = crate::cpu::instructions::Instruction::JpImm16 {
            address: 0x2000,
        };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cpu.state.registers.pc, 0x2000);
    }

    #[test]
    fn test_jp_cond_nz_taken() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.pc = 0x1000;
        cpu.state.registers.f_mut().set_zero(false);

        let instruction = crate::cpu::instructions::Instruction::JpCondImm16 {
            cond: crate::cpu::instructions::Condition::NZ,
            address: 0x2000,
        };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cpu.state.registers.pc, 0x2000);
    }

    #[test]
    fn test_jp_cond_z_taken() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.pc = 0x1000;
        cpu.state.registers.f_mut().set_zero(true);

        let instruction = crate::cpu::instructions::Instruction::JpCondImm16 {
            cond: crate::cpu::instructions::Condition::Z,
            address: 0x2000,
        };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cpu.state.registers.pc, 0x2000);
    }

    #[test]
    fn test_jp_cond_nc_taken() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.pc = 0x1000;
        cpu.state.registers.f_mut().set_carry(false);

        let instruction = crate::cpu::instructions::Instruction::JpCondImm16 {
            cond: crate::cpu::instructions::Condition::NC,
            address: 0x2000,
        };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cpu.state.registers.pc, 0x2000);
    }

    #[test]
    fn test_jp_cond_c_taken() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.pc = 0x1000;
        cpu.state.registers.f_mut().set_carry(true);

        let instruction = crate::cpu::instructions::Instruction::JpCondImm16 {
            cond: crate::cpu::instructions::Condition::C,
            address: 0x2000,
        };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cpu.state.registers.pc, 0x2000);
    }

    #[test]
    fn test_jp_hl() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.hl = 0x2000;

        let instruction = crate::cpu::instructions::Instruction::JpHl;
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cpu.state.registers.pc, 0x2000);
    }

    #[test]
    fn test_call_imm16() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.pc = 0x1000;
        cpu.state.registers.sp = 0xC000;

        let instruction = crate::cpu::instructions::Instruction::CallImm16 {
            address: 0x2000,
        };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cpu.state.registers.pc, 0x2000);    // Jumped to target
        assert_eq!(cpu.state.registers.sp, 0xBFFE);    // SP decreased by 2
        assert_eq!(bus.read(0xBFFF), 0x10);            // Return address low byte
        assert_eq!(bus.read(0xBFFE), 0x03);            // Return address high byte
    }

    #[test]
    fn test_call_cond_nz_taken() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.pc = 0x1000;
        cpu.state.registers.sp = 0xC000;
        cpu.state.registers.f_mut().set_zero(false); // Z = 0

        let instruction = crate::cpu::instructions::Instruction::CallCondImm16 {
            cond: crate::cpu::instructions::Condition::NZ,
            address: 0x2000,
        };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cpu.state.registers.pc, 0x2000);    // Jump taken
        assert_eq!(cpu.state.registers.sp, 0xBFFE);    // SP decreased by 2
        assert_eq!(bus.read(0xBFFF), 0x10); // High byte
        assert_eq!(bus.read(0xBFFE), 0x03); // Low byte
    }

    #[test]
    fn test_call_cond_nz_not_taken() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.pc = 0x1000;
        cpu.state.registers.sp = 0xC000;
        cpu.state.registers.f_mut().set_zero(true);

        let instruction = crate::cpu::instructions::Instruction::CallCondImm16 {
            cond: crate::cpu::instructions::Condition::NZ,
            address: 0x2000,
        };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cpu.state.registers.pc, 0x1000); // Not called
        assert_eq!(cpu.state.registers.sp, 0xC000); // SP unchanged
    }

    #[test]
    fn test_call_cond_c_taken() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.pc = 0x1000;
        cpu.state.registers.sp = 0xC000;
        cpu.state.registers.f_mut().set_carry(true);

        let instruction = crate::cpu::instructions::Instruction::CallCondImm16 {
            cond: crate::cpu::instructions::Condition::C,
            address: 0x2000,
        };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cpu.state.registers.pc, 0x2000);
    }

    #[test]
    fn test_ret() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.pc = 0x1000;
        cpu.state.registers.sp = 0xBFFC;
        bus.write(0xBFFC, 0x02); // low byte
        bus.write(0xBFFD, 0x20); // high byte

        let instruction = crate::cpu::instructions::Instruction::RET;
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cpu.state.registers.pc, 0x2002); // PC set to return address
        assert_eq!(cpu.state.registers.sp, 0xBFFE); // SP incremented by 2
    }

    #[test]
    fn test_ret_cond_nz_taken() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.pc = 0x1000;
        cpu.state.registers.sp = 0xBFFC;
        bus.write(0xBFFC, 0x02); // low byte
        bus.write(0xBFFD, 0x20); // high byte
        cpu.state.registers.f_mut().set_zero(false); // Z = 0

        let instruction = crate::cpu::instructions::Instruction::RetCond {
            cond: crate::cpu::instructions::Condition::NZ,
        };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cpu.state.registers.pc, 0x2002); // RET taken
        assert_eq!(cpu.state.registers.sp, 0xBFFE); // SP incremented by 2
    }

    #[test]
    fn test_ret_cond_nz_not_taken() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.pc = 0x1000;
        cpu.state.registers.sp = 0xBFFC;
        cpu.state.registers.f_mut().set_zero(true);

        let instruction = crate::cpu::instructions::Instruction::RetCond {
            cond: crate::cpu::instructions::Condition::NZ,
        };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cpu.state.registers.pc, 0x1000);
        assert_eq!(cpu.state.registers.sp, 0xBFFC); // Not changed
    }

    #[test]
    fn test_reti() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.pc = 0x1000;
        cpu.state.registers.sp = 0xBFFC;
        bus.write(0xBFFC, 0x02);
        bus.write(0xBFFD, 0x20);
        cpu.state.ime = false;

        let instruction = crate::cpu::instructions::Instruction::RETI;
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cpu.state.registers.pc, 0x2002);
        assert!(cpu.state.ime); // IME set by RETI
    }

    #[test]
    fn test_rst() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.pc = 0x1000; // return address (PC at time of RST execution)
        cpu.state.registers.sp = 0xC000;

        let instruction = Instruction::RST { target: 0x08 };
        execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cpu.state.registers.pc, 0x0008);     // jumped to target
        assert_eq!(cpu.state.registers.sp, 0xBFFE);     // SP decremented by 2
        assert_eq!(bus.read(0xBFFF), 0x10);             // return address high byte
        assert_eq!(bus.read(0xBFFE), 0x00);             // return address low byte
    }

    // ==================== STACK INSTRUCTIONS ====================

    #[test]
    fn test_push_r16_bc() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.bc = 0x1234;
        cpu.state.registers.sp = 0xC000;

        let instruction = crate::cpu::instructions::Instruction::PushR16 {
            reg: crate::cpu::instructions::R16Register::BC,
        };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cpu.state.registers.sp, 0xBFFE);      // SP decremented by 2
        assert_eq!(bus.read(0xBFFE), 0x34);              // Low byte (C)
        assert_eq!(bus.read(0xBFFF), 0x12);              // High byte (B)
    }

    #[test]
    fn test_push_r16_de() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.de = 0x5678;
        cpu.state.registers.sp = 0xC000;

        let instruction = crate::cpu::instructions::Instruction::PushR16 {
            reg: crate::cpu::instructions::R16Register::DE,
        };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cpu.state.registers.sp, 0xBFFE); // SP decremented by 2
        assert_eq!(bus.read(0xBFFE), 0x78);         // Low byte (E)
        assert_eq!(bus.read(0xBFFF), 0x56);         // High byte (D)
    }

    #[test]
    fn test_pop_r16_bc() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.sp = 0xBFFE;
        bus.write(0xBFFE, 0x12);
        bus.write(0xBFFF, 0x34);

        let instruction = crate::cpu::instructions::Instruction::PopR16 {
            reg: crate::cpu::instructions::R16Register::BC,
        };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cpu.state.registers.bc, 0x3412);
        assert_eq!(cpu.state.registers.sp, 0xC000);
    }

    #[test]
    fn test_pop_r16_af() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.sp = 0xBFFE;
        bus.write(0xBFFE, 0xAB); // F (lower nibble ignored)
        bus.write(0xBFFF, 0x00); // A

        let instruction = crate::cpu::instructions::Instruction::PopR16 {
            reg: crate::cpu::instructions::R16Register::AF,
        };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cpu.state.registers.af, 0x00A0); // lower nibble of F is always zero
        assert_eq!(cpu.state.registers.sp, 0xC000); // SP incremented
    }

    // ==================== SP/PC INSTRUCTIONS ====================

    #[test]
    fn test_add_sp_imm8_positive() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.sp = 0xC000;

        let instruction = crate::cpu::instructions::Instruction::AddSpImm8 {
            value: 0x05,
        };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cpu.state.registers.sp, 0xC005);
        assert!(!cpu.state.registers.f().is_zero());
        assert!(!cpu.state.registers.f().is_carry());
    }

    #[test]
    fn test_add_sp_imm8_negative() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.sp = 0xC000;

        let instruction = crate::cpu::instructions::Instruction::AddSpImm8 {
            value: -0x05,
        };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cpu.state.registers.sp, 0xBFFB);
    }

    #[test]
    fn test_add_sp_imm8_negative_no_carry() {
        // SP=0xC000, offset=-1 (0xFF unsigned).
        // Byte addition: 0x00 + 0xFF = 0xFF, no carry, no half-carry.
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.sp = 0xC000;

        let instruction = Instruction::AddSpImm8 { value: -1i8 };
        execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cpu.state.registers.sp, 0xBFFF);
        assert!(!cpu.state.registers.f().is_carry());
        assert!(!cpu.state.registers.f().is_half_carry());
        assert!(!cpu.state.registers.f().is_zero());
        assert!(!cpu.state.registers.f().is_subtraction());
    }

    #[test]
    fn test_add_sp_imm8_carry() {
        // SP=0xC001, offset=+127 (0x7F unsigned).
        // Byte addition: 0x01 + 0x7F = 0x80, no carry, half-carry set (0x1 + 0xF > 0xF).
        // Use SP=0xC0FF, offset=+1: 0xFF + 0x01 = 0x100, carry set.
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.sp = 0xC0FF;

        let instruction = Instruction::AddSpImm8 { value: 1i8 };
        execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cpu.state.registers.sp, 0xC100);
        assert!(cpu.state.registers.f().is_carry());
        assert!(cpu.state.registers.f().is_half_carry()); // 0xF + 0x1 > 0xF
        assert!(!cpu.state.registers.f().is_zero());
        assert!(!cpu.state.registers.f().is_subtraction());
    }

    #[test]
    fn test_add_sp_imm8_half_carry_only() {
        // SP=0xC00F, offset=+1: byte addition 0x0F + 0x01 = 0x10.
        // Half-carry set, no carry.
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.sp = 0xC00F;

        let instruction = Instruction::AddSpImm8 { value: 1i8 };
        execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cpu.state.registers.sp, 0xC010);
        assert!(!cpu.state.registers.f().is_carry());
        assert!(cpu.state.registers.f().is_half_carry());
    }

    #[test]
    fn test_ld_hl_sp_imm8() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.sp = 0xC005;

        let instruction = crate::cpu::instructions::Instruction::LdHlSpImm8 {
            value: 0x0A,
        };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cpu.state.registers.hl, 0xC00F);
        assert!(!cpu.state.registers.f().is_zero());
        assert!(!cpu.state.registers.f().is_carry());
    }

    #[test]
    fn test_ld_sp_hl() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.hl = 0xC000;

        let instruction = crate::cpu::instructions::Instruction::LdSpHl;
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cpu.state.registers.sp, 0xC000);
    }

    // ==================== CONTROL INSTRUCTIONS ====================

    #[test]
    fn test_di() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.ime = true;

        let instruction = crate::cpu::instructions::Instruction::DI;
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert!(!cpu.state.ime);
    }

    #[test]
    fn test_ei() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.ime = false;

        let instruction = crate::cpu::instructions::Instruction::EI;
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        // EI does not enable IME immediately; ime_pending is set for a 1-instruction delay.
        assert!(!cpu.state.ime);
        assert!(cpu.state.ime_pending);
    }

    #[test]
    fn test_nop() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.pc = 0x1000;

        let instruction = crate::cpu::instructions::Instruction::NOP;
        let cycles = crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cycles, 1);
        assert_eq!(cpu.state.registers.pc, 0x1000); // PC unchanged
    }

    #[test]
    fn test_stop() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);

        let instruction = crate::cpu::instructions::Instruction::STOP;
        let cycles = crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cycles, 1);
    }

    // ==================== CB PREFIXED INSTRUCTIONS ====================

    #[test]
    fn test_rlcr8_b() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.set_b(0b10110001); // 0xB1

        let instruction = crate::cpu::instructions::Instruction::CB {
            cb_instr: crate::cpu::instructions::CBInstruction::RLCR8 {
                reg: crate::cpu::instructions::R8Register::B,
            },
        };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cpu.state.registers.b(), 0b01100011); // 0x63
        assert!(cpu.state.registers.f().is_carry());
        assert!(!cpu.state.registers.f().is_zero());
    }

    #[test]
    fn test_rrcr8_a() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.set_a(0b10110001); // 0xB1

        let instruction = crate::cpu::instructions::Instruction::CB {
            cb_instr: crate::cpu::instructions::CBInstruction::RRCR8 {
                reg: crate::cpu::instructions::R8Register::A,
            },
        };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cpu.state.registers.a(), 0b11011000); // 0xD8
        assert!(cpu.state.registers.f().is_carry());
    }

    #[test]
    fn test_rlr8_a() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.set_a(0b10110001);
        cpu.state.registers.f_mut().set_carry(true);

        let instruction = crate::cpu::instructions::Instruction::CB {
            cb_instr: CBInstruction::RLR8 {
                reg: R8Register::A,
            },
        };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cpu.state.registers.a(), 0b01100011);
        assert!(cpu.state.registers.f().is_carry());
    }

    #[test]
    fn test_rrr8_a() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.set_a(0b10110001);
        cpu.state.registers.f_mut().set_carry(true);

        let instruction = crate::cpu::instructions::Instruction::CB {
            cb_instr: CBInstruction::RRR8 {
                reg: R8Register::A,
            },
        };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cpu.state.registers.a(), 0b11011000);
        assert!(cpu.state.registers.f().is_carry());
    }

    #[test]
    fn test_slar8_a() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.set_a(0b01000000);

        let instruction = crate::cpu::instructions::Instruction::CB {
            cb_instr: CBInstruction::SLAR8 {
                reg: R8Register::A,
            },
        };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cpu.state.registers.a(), 0b10000000);
        assert!(!cpu.state.registers.f().is_carry());
    }

    #[test]
    fn test_srar8_a() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.set_a(0b11000000);

        let instruction = crate::cpu::instructions::Instruction::CB {
            cb_instr: CBInstruction::SRAR8 {
                reg: R8Register::A,
            },
        };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cpu.state.registers.a(), 0b11100000); // Sign extended
        assert!(!cpu.state.registers.f().is_carry()); // LSB was 0
    }

    #[test]
    fn test_swapr8_a() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.set_a(0x12);

        let instruction = crate::cpu::instructions::Instruction::CB {
            cb_instr: CBInstruction::SWAPR8 {
                reg: R8Register::A,
            },
        };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cpu.state.registers.a(), 0x21);
        assert!(!cpu.state.registers.f().is_zero());
    }

    #[test]
    fn test_swapr8_a_zero() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.set_a(0x00);

        let instruction = crate::cpu::instructions::Instruction::CB {
            cb_instr: CBInstruction::SWAPR8 {
                reg: R8Register::A,
            },
        };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cpu.state.registers.a(), 0x00);
        assert!(cpu.state.registers.f().is_zero());
    }

    #[test]
    fn test_srlr8_a() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.set_a(0b01000000);

        let instruction = crate::cpu::instructions::Instruction::CB {
            cb_instr: CBInstruction::SRLR8 {
                reg: R8Register::A,
            },
        };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cpu.state.registers.a(), 0b00100000);
        assert!(!cpu.state.registers.f().is_carry()); // LSB was 0
    }

    #[test]
    fn test_srlr8_a_lsb() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.set_a(0b00000001);

        let instruction = crate::cpu::instructions::Instruction::CB {
            cb_instr: CBInstruction::SRLR8 {
                reg: R8Register::A,
            },
        };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cpu.state.registers.a(), 0b00000000);
        assert!(cpu.state.registers.f().is_carry()); // LSB was 1
        assert!(cpu.state.registers.f().is_zero());
    }

    #[test]
    fn test_bitbr8_bit0_set() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.set_a(0b00000001);

        let instruction = crate::cpu::instructions::Instruction::CB {
            cb_instr: crate::cpu::instructions::CBInstruction::BITBR8 {
                bit: 0,
                reg: crate::cpu::instructions::R8Register::A,
            },
        };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert!(!cpu.state.registers.f().is_zero()); // Bit 0 is set
        assert!(cpu.state.registers.f().is_half_carry());
    }

    #[test]
    fn test_bitbr8_bit0_clear() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.set_a(0b00000000);

        let instruction = crate::cpu::instructions::Instruction::CB {
            cb_instr: crate::cpu::instructions::CBInstruction::BITBR8 {
                bit: 0,
                reg: crate::cpu::instructions::R8Register::A,
            },
        };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert!(cpu.state.registers.f().is_zero()); // Bit 0 is clear
        assert!(cpu.state.registers.f().is_half_carry());
    }

    #[test]
    fn test_bitbr8_bit7_set() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.set_a(0b10000000);

        let instruction = crate::cpu::instructions::Instruction::CB {
            cb_instr: crate::cpu::instructions::CBInstruction::BITBR8 {
                bit: 7,
                reg: crate::cpu::instructions::R8Register::A,
            },
        };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert!(!cpu.state.registers.f().is_zero()); // Bit 7 is set
    }

    #[test]
    fn test_bitbr8_hl_memory() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.hl = 0xC000;
        bus.write(0xC000, 0b00001000); // Bit 3 set

        let instruction = crate::cpu::instructions::Instruction::CB {
            cb_instr: crate::cpu::instructions::CBInstruction::BITBR8 {
                bit: 3,
                reg: crate::cpu::instructions::R8Register::HL,
            },
        };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert!(!cpu.state.registers.f().is_zero()); // Bit 3 is set
    }

    #[test]
    fn test_resbr8_b_bit0() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.set_b(0b11111111);

        let instruction = crate::cpu::instructions::Instruction::CB {
            cb_instr: crate::cpu::instructions::CBInstruction::RESBR8 {
                bit: 0,
                reg: crate::cpu::instructions::R8Register::B,
            },
        };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cpu.state.registers.b(), 0b11111110);
    }

    #[test]
    fn test_resbr8_a_bit7() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.set_a(0b11111111);

        let instruction = crate::cpu::instructions::Instruction::CB {
            cb_instr: crate::cpu::instructions::CBInstruction::RESBR8 {
                bit: 7,
                reg: crate::cpu::instructions::R8Register::A,
            },
        };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cpu.state.registers.a(), 0b01111111);
    }

    #[test]
    fn test_setbr8_b_bit0() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.set_b(0b00000000);

        let instruction = crate::cpu::instructions::Instruction::CB {
            cb_instr: crate::cpu::instructions::CBInstruction::SETBR8 {
                bit: 0,
                reg: crate::cpu::instructions::R8Register::B,
            },
        };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cpu.state.registers.b(), 0b00000001);
    }

    #[test]
    fn test_setbr8_a_bit7() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.state.registers.set_a(0b00000000);

        let instruction = crate::cpu::instructions::Instruction::CB {
            cb_instr: crate::cpu::instructions::CBInstruction::SETBR8 {
                bit: 7,
                reg: crate::cpu::instructions::R8Register::A,
            },
        };
        crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction);

        assert_eq!(cpu.state.registers.a(), 0b10000000);
    }

    // ==================== INSTRUCTION EXECUTION INTEGRATION TESTS ====================

    #[test]
    fn test_cpu_execute_sequence() {
        let mut cpu = CPU::new();
        let mut rom = vec![0; 32768];

        // Program:
        // 0x100: LD A, 0x10
        // 0x102: ADD A, 0x20
        // 0x104: RET (0xC9)
        rom[0x100] = 0x3E; // LD A, d8
        rom[0x101] = 0x10;
        rom[0x102] = 0xC6; // ADD A, d8
        rom[0x103] = 0x20;
        rom[0x104] = 0xC9; // RET

        let mut bus = MemoryBus::new(rom);

        // Execute LD A, 0x10
        let cycles1 = cpu.execute(&mut bus);
        assert_eq!(cycles1, 2);
        assert_eq!(cpu.cycles(), 2);
        assert_eq!(cpu.state.registers.a(), 0x10);

        // Execute ADD A, 0x20
        let cycles2 = cpu.execute(&mut bus);
        assert_eq!(cycles2, 2);
        assert_eq!(cpu.cycles(), 4);
        assert_eq!(cpu.state.registers.a(), 0x30);

        // Execute RET (returns to 0x0100 which loops back)
        let cycles3 = cpu.execute(&mut bus);
        assert_eq!(cycles3, 4);
    }

    #[test]
    fn test_cpu_halt() {
        let mut cpu = CPU::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);
        cpu.halted = true;

        // When halted, CPU should return 1 cycle
        let cycles = cpu.execute(&mut bus);
        assert_eq!(cycles, 1);
        assert_eq!(cpu.cycles(), 1);
    }

    #[test]
    fn test_cpu_reset_initializes_registers() {
        let mut cpu = CPU::new();
        // Modify all registers
        cpu.state.registers.af = 0xDEAD;
        cpu.state.registers.bc = 0xBEEF;
        cpu.state.registers.de = 0xCAFE;
        cpu.state.registers.hl = 0xFACE;
        cpu.state.registers.sp = 0x0000;
        cpu.state.registers.pc = 0x0000;
        cpu.state.ime = true;
        cpu.halted = true;

        cpu.reset();

        assert_eq!(cpu.state.registers.pc, 0x0100);
        assert_eq!(cpu.state.registers.sp, 0xFFFE);
        assert_eq!(cpu.state.registers.af, 0x01B0);
        assert_eq!(cpu.state.registers.bc, 0x0013);
        assert_eq!(cpu.state.registers.de, 0x00D8);
        assert_eq!(cpu.state.registers.hl, 0x014D);
        assert!(!cpu.state.ime);
        assert!(!cpu.halted);
    }
}
