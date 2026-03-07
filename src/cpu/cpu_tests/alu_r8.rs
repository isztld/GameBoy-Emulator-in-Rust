use super::*;

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
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

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
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

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
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

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
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

    assert_eq!(cpu.state.registers.a(), 0x10);
    assert!(cpu.state.registers.f().is_half_carry());
}

#[test]
fn test_adc_a_r8_no_carry() {
    let mut cpu = CPU::new();
    let mut bus = MemoryBus::new(vec![0; 32768]);
    cpu.state.registers.set_a(0x10);
    cpu.state.registers.set_b(0x20);
    cpu.state.registers.modify_f(|f| f.set_carry(false)); // Clear carry from reset

    let instruction = crate::cpu::instructions::Instruction::AdcAR8 {
        reg: crate::cpu::instructions::R8Register::B,
    };
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

    assert_eq!(cpu.state.registers.a(), 0x30);
}

#[test]
fn test_adc_a_r8_with_carry() {
    let mut cpu = CPU::new();
    let mut bus = MemoryBus::new(vec![0; 32768]);
    cpu.state.registers.set_a(0x10);
    cpu.state.registers.set_b(0x20);
    cpu.state.registers.modify_f(|f| f.set_carry(true));

    let instruction = crate::cpu::instructions::Instruction::AdcAR8 {
        reg: crate::cpu::instructions::R8Register::B,
    };
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

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
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

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
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

    assert_eq!(cpu.state.registers.a(), 0xE0);
    assert!(cpu.state.registers.f().is_carry());
}

#[test]
fn test_sbc_a_r8_no_carry() {
    let mut cpu = CPU::new();
    let mut bus = MemoryBus::new(vec![0; 32768]);
    cpu.state.registers.set_a(0x30);
    cpu.state.registers.set_b(0x10);
    cpu.state.registers.modify_f(|f| f.set(0)); // clear all flags

    let instruction = crate::cpu::instructions::Instruction::SbcAR8 {
        reg: crate::cpu::instructions::R8Register::B,
    };
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

    assert_eq!(cpu.state.registers.a(), 0x20);
}

#[test]
fn test_sbc_a_r8_with_carry() {
    let mut cpu = CPU::new();
    let mut bus = MemoryBus::new(vec![0; 32768]);
    cpu.state.registers.set_a(0x30);
    cpu.state.registers.set_b(0x10);
    cpu.state.registers.modify_f(|f| f.set_carry(true));

    let instruction = crate::cpu::instructions::Instruction::SbcAR8 {
        reg: crate::cpu::instructions::R8Register::B,
    };
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

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
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

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
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

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
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

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
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

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
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

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
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

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
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

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
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

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
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

    assert_eq!(cpu.state.registers.a(), 0x42); // A unchanged
    assert!(cpu.state.registers.f().is_zero());
}

#[test]
fn test_add_a_imm8() {
    let mut cpu = CPU::new();
    let mut bus = MemoryBus::new(vec![0; 32768]);
    cpu.state.registers.set_a(0x10);

    let instruction = crate::cpu::instructions::Instruction::AddAImm8 { value: 0x20 };
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

    assert_eq!(cpu.state.registers.a(), 0x30);
}

#[test]
fn test_adc_a_imm8() {
    let mut cpu = CPU::new();
    let mut bus = MemoryBus::new(vec![0; 32768]);
    cpu.state.registers.set_a(0x10);
    cpu.state.registers.modify_f(|f| f.set_carry(true));

    let instruction = crate::cpu::instructions::Instruction::AdcAImm8 { value: 0x20 };
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

    assert_eq!(cpu.state.registers.a(), 0x31); // 0x10 + 0x20 + 1
}

#[test]
fn test_sub_a_imm8() {
    let mut cpu = CPU::new();
    let mut bus = MemoryBus::new(vec![0; 32768]);
    cpu.state.registers.set_a(0x30);

    let instruction = crate::cpu::instructions::Instruction::SubAImm8 { value: 0x10 };
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

    assert_eq!(cpu.state.registers.a(), 0x20);
    assert!(cpu.state.registers.f().is_subtraction());
}

#[test]
fn test_sbc_a_imm8() {
    let mut cpu = CPU::new();
    let mut bus = MemoryBus::new(vec![0; 32768]);
    cpu.state.registers.set_a(0x30);
    cpu.state.registers.modify_f(|f| f.set(0)); // clear all flags

    let instruction = crate::cpu::instructions::Instruction::SbcAImm8 { value: 0x10 };
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

    assert_eq!(cpu.state.registers.a(), 0x20);
}

#[test]
fn test_and_a_imm8() {
    let mut cpu = CPU::new();
    let mut bus = MemoryBus::new(vec![0; 32768]);
    cpu.state.registers.set_a(0xFF);

    let instruction = crate::cpu::instructions::Instruction::AndAImm8 { value: 0x0F };
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

    assert_eq!(cpu.state.registers.a(), 0x0F);
}

#[test]
fn test_xor_a_imm8() {
    let mut cpu = CPU::new();
    let mut bus = MemoryBus::new(vec![0; 32768]);
    cpu.state.registers.set_a(0xFF);

    let instruction = crate::cpu::instructions::Instruction::XorAImm8 { value: 0x0F };
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

    assert_eq!(cpu.state.registers.a(), 0xF0);
}

#[test]
fn test_or_a_imm8() {
    let mut cpu = CPU::new();
    let mut bus = MemoryBus::new(vec![0; 32768]);
    cpu.state.registers.set_a(0xF0);

    let instruction = crate::cpu::instructions::Instruction::OrAImm8 { value: 0x0F };
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

    assert_eq!(cpu.state.registers.a(), 0xFF);
}

#[test]
fn test_cp_a_imm8() {
    let mut cpu = CPU::new();
    let mut bus = MemoryBus::new(vec![0; 32768]);
    cpu.state.registers.set_a(0x30);

    let instruction = crate::cpu::instructions::Instruction::CpAImm8 { value: 0x10 };
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

    assert_eq!(cpu.state.registers.a(), 0x30); // A unchanged
    assert!(!cpu.state.registers.f().is_carry());
}

