use super::*;

// ==================== REGISTER INSTRUCTIONS ====================

#[test]
fn test_inc_r16_bc() {
    let mut cpu = CPU::new();
    let mut bus = MemoryBus::new(vec![0; 32768]);
    cpu.state.registers.bc = 0x1234;

    let instruction = crate::cpu::instructions::Instruction::IncR16 {
        reg: crate::cpu::instructions::R16Register::BC,
    };
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

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
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

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
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

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
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

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
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

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
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

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
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

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
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

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
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

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
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

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
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

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
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

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
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

    assert_eq!(cpu.state.registers.a(), 0b11011000); // 0xD8
    assert!(cpu.state.registers.f().is_carry()); // LSB was 1
    assert!(!cpu.state.registers.f().is_zero());
}

#[test]
fn test_rla() {
    let mut cpu = CPU::new();
    let mut bus = MemoryBus::new(vec![0; 32768]);
    cpu.state.registers.set_a(0b10110001); // 0xB1
    cpu.state.registers.modify_f(|f| f.set_carry(true));

    let instruction = crate::cpu::instructions::Instruction::RLA;
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

    assert_eq!(cpu.state.registers.a(), 0b01100011); // 0x63
    assert!(cpu.state.registers.f().is_carry()); // MSB was 1
}

#[test]
fn test_rra() {
    let mut cpu = CPU::new();
    let mut bus = MemoryBus::new(vec![0; 32768]);
    cpu.state.registers.set_a(0b10110001); // 0xB1
    cpu.state.registers.modify_f(|f| f.set_carry(true));

    let instruction = crate::cpu::instructions::Instruction::RRA;
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

    assert_eq!(cpu.state.registers.a(), 0b11011000); // 0xD8
    assert!(cpu.state.registers.f().is_carry()); // LSB was 1
}

#[test]
fn test_daa() {
    let mut cpu = CPU::new();
    let mut bus = MemoryBus::new(vec![0; 32768]);
    cpu.state.registers.set_a(0x19); // 19 in BCD
    cpu.state.registers.modify_f(|f| f.set(0));

    let instruction = crate::cpu::instructions::Instruction::DAA;
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

    // 0x19 has both nibbles <= 9, so no adjustment needed
    assert_eq!(cpu.state.registers.a(), 0x19);
}

#[test]
fn daa_half_carry_adjust() {
    let mut cpu = CPU::new();
    let mut bus = MemoryBus::new(vec![0; 32768]);
    cpu.state.registers.set_a(0x0A);
    cpu.state.registers.modify_f(|f| {
        f.set_half_carry(true);
        f.set_subtraction(false);
        f.set_carry(false);
    });

    let instruction = crate::cpu::instructions::Instruction::DAA;
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

    assert_eq!(cpu.state.registers.a(), 0x10);
}

#[test]
fn daa_full_carry_adjust() {
    let mut cpu = CPU::new();
    let mut bus = MemoryBus::new(vec![0; 32768]);
    cpu.state.registers.set_a(0x9A);
    cpu.state.registers.modify_f(|f| {
        f.set_subtraction(false);
        f.set_half_carry(false);
        f.set_carry(false);
    });

    let instruction = crate::cpu::instructions::Instruction::DAA;
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

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
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

    assert_eq!(cpu.state.registers.a(), 0xAA);
    assert!(cpu.state.registers.f().is_subtraction());
    assert!(cpu.state.registers.f().is_half_carry());
}

#[test]
fn test_scf() {
    let mut cpu = CPU::new();
    let mut bus = MemoryBus::new(vec![0; 32768]);
    cpu.state.registers.modify_f(|f| f.set_carry(false));

    let instruction = crate::cpu::instructions::Instruction::SCF;
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

    assert!(cpu.state.registers.f().is_carry());
    assert!(!cpu.state.registers.f().is_subtraction());
}

#[test]
fn test_ccf() {
    let mut cpu = CPU::new();
    let mut bus = MemoryBus::new(vec![0; 32768]);
    cpu.state.registers.modify_f(|f| f.set_carry(false));

    let instruction = crate::cpu::instructions::Instruction::CCF;
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

    assert!(cpu.state.registers.f().is_carry()); // Flip carry

    cpu.state.registers.modify_f(|f| f.set_carry(true));
    let instruction = crate::cpu::instructions::Instruction::CCF;
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

    assert!(!cpu.state.registers.f().is_carry()); // Flip carry
}

