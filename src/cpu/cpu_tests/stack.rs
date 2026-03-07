use super::*;

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
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

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
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

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
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

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
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

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
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

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
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

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
    execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

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
    execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

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
    execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

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
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

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
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

    assert_eq!(cpu.state.registers.sp, 0xC000);
}

// ==================== CONTROL INSTRUCTIONS ====================

#[test]
fn test_di() {
    let mut cpu = CPU::new();
    let mut bus = MemoryBus::new(vec![0; 32768]);
    cpu.state.ime = true;

    let instruction = crate::cpu::instructions::Instruction::DI;
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

    assert!(!cpu.state.ime);
}

#[test]
fn test_ei() {
    let mut cpu = CPU::new();
    let mut bus = MemoryBus::new(vec![0; 32768]);
    cpu.state.ime = false;

    let instruction = crate::cpu::instructions::Instruction::EI;
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

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
    let cycles = crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

    assert_eq!(cycles, 1);
    assert_eq!(cpu.state.registers.pc, 0x1000); // PC unchanged
}

#[test]
fn test_stop() {
    let mut cpu = CPU::new();
    let mut bus = MemoryBus::new(vec![0; 32768]);

    let instruction = crate::cpu::instructions::Instruction::STOP;
    let cycles = crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

    assert_eq!(cycles, 1);
}

