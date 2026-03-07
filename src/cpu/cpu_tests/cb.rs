use super::*;

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
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

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
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

    assert_eq!(cpu.state.registers.a(), 0b11011000); // 0xD8
    assert!(cpu.state.registers.f().is_carry());
}

#[test]
fn test_rlr8_a() {
    let mut cpu = CPU::new();
    let mut bus = MemoryBus::new(vec![0; 32768]);
    cpu.state.registers.set_a(0b10110001);
    cpu.state.registers.modify_f(|f| f.set_carry(true));

    let instruction = crate::cpu::instructions::Instruction::CB {
        cb_instr: CBInstruction::RLR8 {
            reg: R8Register::A,
        },
    };
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

    assert_eq!(cpu.state.registers.a(), 0b01100011);
    assert!(cpu.state.registers.f().is_carry());
}

#[test]
fn test_rrr8_a() {
    let mut cpu = CPU::new();
    let mut bus = MemoryBus::new(vec![0; 32768]);
    cpu.state.registers.set_a(0b10110001);
    cpu.state.registers.modify_f(|f| f.set_carry(true));

    let instruction = crate::cpu::instructions::Instruction::CB {
        cb_instr: CBInstruction::RRR8 {
            reg: R8Register::A,
        },
    };
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

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
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

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
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

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
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

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
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

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
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

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
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

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
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

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
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

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
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

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
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

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
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

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
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

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
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

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
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

    assert_eq!(cpu.state.registers.a(), 0b10000000);
}

