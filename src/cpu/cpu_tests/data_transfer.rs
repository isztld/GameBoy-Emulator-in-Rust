use super::*;

// ==================== DATA TRANSFER INSTRUCTIONS ====================

#[test]
fn test_ld_r16_imm16_bc() {
    let mut cpu = CPU::new();
    let mut bus = MemoryBus::new(vec![0; 32768]);
    let instruction = crate::cpu::instructions::Instruction::LdR16Imm16 {
        dest: crate::cpu::instructions::R16Register::BC,
        value: 0xABCD,
    };
    let cycles = crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);
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
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);
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
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);
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
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);
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
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);
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
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);
    assert_eq!(cpu.state.registers.c(), 0x12);

    // LD D, C
    let instruction = crate::cpu::instructions::Instruction::LdR8R8 {
        dest: crate::cpu::instructions::R8Register::D,
        src: crate::cpu::instructions::R8Register::C,
    };
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);
    assert_eq!(cpu.state.registers.d(), 0x12);

    // LD E, D
    let instruction = crate::cpu::instructions::Instruction::LdR8R8 {
        dest: crate::cpu::instructions::R8Register::E,
        src: crate::cpu::instructions::R8Register::D,
    };
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);
    assert_eq!(cpu.state.registers.e(), 0x12);

    // LD H, E
    let instruction = crate::cpu::instructions::Instruction::LdR8R8 {
        dest: crate::cpu::instructions::R8Register::H,
        src: crate::cpu::instructions::R8Register::E,
    };
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);
    assert_eq!(cpu.state.registers.h(), 0x12);

    // LD L, H
    let instruction = crate::cpu::instructions::Instruction::LdR8R8 {
        dest: crate::cpu::instructions::R8Register::L,
        src: crate::cpu::instructions::R8Register::H,
    };
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);
    assert_eq!(cpu.state.registers.l(), 0x12);

    // LD A, L
    let instruction = crate::cpu::instructions::Instruction::LdR8R8 {
        dest: crate::cpu::instructions::R8Register::A,
        src: crate::cpu::instructions::R8Register::L,
    };
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);
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
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

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
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

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
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

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
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

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
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

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
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

    assert_eq!(cpu.state.registers.a(), 0x66);
    assert_eq!(cpu.state.registers.hl, 0xA001);
}

#[test]
fn test_ld_ind_imm16_sp() {
    let mut cpu = CPU::new();
    let mut bus = MemoryBus::new(vec![0; 32768]);
    cpu.state.registers.sp = 0xC000;

    let instruction = Instruction::LdIndImm16Sp { address: 0xD000 };
    execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

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
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

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
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

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
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

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
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

    assert_eq!(cpu.state.registers.a(), 0xAA);
}

#[test]
fn test_ldh_ind_c_a() {
    let mut cpu = CPU::new();
    let mut bus = MemoryBus::new(vec![0; 32768]);
    cpu.state.registers.set_c(0x01); // Use 0xFF01 (SB) which stores full value
    cpu.state.registers.set_a(0xBB);

    let instruction = crate::cpu::instructions::Instruction::LdhIndCA;
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

    assert_eq!(bus.read(0xFF01), 0xBB);
}

#[test]
fn test_ldh_a_c() {
    let mut cpu = CPU::new();
    let mut bus = MemoryBus::new(vec![0; 32768]);

    cpu.state.registers.set_c(0x01); // Use 0xFF01 (SB) which stores full value
    bus.write(0xFF01, 0xCC);

    let instruction = crate::cpu::instructions::Instruction::LdhAC;
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

    assert_eq!(cpu.state.registers.a(), 0xCC);
}

