use super::*;

// ==================== JUMP/CONTROL FLOW INSTRUCTIONS ====================

#[test]
fn test_jr_imm8() {
    let mut cpu = CPU::new();
    let mut bus = MemoryBus::new(vec![0; 32768]);
    cpu.state.registers.pc = 0x1000;

    let instruction = crate::cpu::instructions::Instruction::JrImm8 { offset: 0x05 };
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

    assert_eq!(cpu.state.registers.pc, 0x1005);
}

#[test]
fn test_jr_imm8_negative() {
    let mut cpu = CPU::new();
    let mut bus = MemoryBus::new(vec![0; 32768]);
    cpu.state.registers.pc = 0x1005;

    let instruction = crate::cpu::instructions::Instruction::JrImm8 { offset: -0x03 };
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

    assert_eq!(cpu.state.registers.pc, 0x1002);
}

#[test]
fn test_jr_cond_nz_taken() {
    let mut cpu = CPU::new();
    let mut bus = MemoryBus::new(vec![0; 32768]);
    cpu.state.registers.pc = 0x1000;
    cpu.state.registers.modify_f(|f| f.set_zero(false)); // Not zero

    let instruction = crate::cpu::instructions::Instruction::JrCondImm8 {
        cond: crate::cpu::instructions::Condition::NZ,
        offset: 0x05,
    };
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

    assert_eq!(cpu.state.registers.pc, 0x1005);
}

#[test]
fn test_jr_cond_nz_not_taken() {
    let mut cpu = CPU::new();
    let mut bus = MemoryBus::new(vec![0; 32768]);
    cpu.state.registers.pc = 0x1000;
    cpu.state.registers.modify_f(|f| f.set_zero(true)); // Zero

    let instruction = crate::cpu::instructions::Instruction::JrCondImm8 {
        cond: crate::cpu::instructions::Condition::NZ,
        offset: 0x05,
    };
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

    assert_eq!(cpu.state.registers.pc, 0x1000); // Not taken
}

#[test]
fn test_jr_cond_z_taken() {
    let mut cpu = CPU::new();
    let mut bus = MemoryBus::new(vec![0; 32768]);
    cpu.state.registers.pc = 0x1000;
    cpu.state.registers.modify_f(|f| f.set_zero(true)); // Zero

    let instruction = crate::cpu::instructions::Instruction::JrCondImm8 {
        cond: crate::cpu::instructions::Condition::Z,
        offset: 0x05,
    };
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

    assert_eq!(cpu.state.registers.pc, 0x1005);
}

#[test]
fn test_jr_cond_nc_taken() {
    let mut cpu = CPU::new();
    let mut bus = MemoryBus::new(vec![0; 32768]);
    cpu.state.registers.pc = 0x1000;
    cpu.state.registers.modify_f(|f| f.set_carry(false)); // Not carry

    let instruction = crate::cpu::instructions::Instruction::JrCondImm8 {
        cond: crate::cpu::instructions::Condition::NC,
        offset: 0x05,
    };
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

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
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

    assert_eq!(cpu.state.registers.pc, 0x2000);
}

#[test]
fn test_jp_cond_nz_taken() {
    let mut cpu = CPU::new();
    let mut bus = MemoryBus::new(vec![0; 32768]);
    cpu.state.registers.pc = 0x1000;
    cpu.state.registers.modify_f(|f| f.set_zero(false));

    let instruction = crate::cpu::instructions::Instruction::JpCondImm16 {
        cond: crate::cpu::instructions::Condition::NZ,
        address: 0x2000,
    };
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

    assert_eq!(cpu.state.registers.pc, 0x2000);
}

#[test]
fn test_jp_cond_z_taken() {
    let mut cpu = CPU::new();
    let mut bus = MemoryBus::new(vec![0; 32768]);
    cpu.state.registers.pc = 0x1000;
    cpu.state.registers.modify_f(|f| f.set_zero(true));

    let instruction = crate::cpu::instructions::Instruction::JpCondImm16 {
        cond: crate::cpu::instructions::Condition::Z,
        address: 0x2000,
    };
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

    assert_eq!(cpu.state.registers.pc, 0x2000);
}

#[test]
fn test_jp_cond_nc_taken() {
    let mut cpu = CPU::new();
    let mut bus = MemoryBus::new(vec![0; 32768]);
    cpu.state.registers.pc = 0x1000;
    cpu.state.registers.modify_f(|f| f.set_carry(false));

    let instruction = crate::cpu::instructions::Instruction::JpCondImm16 {
        cond: crate::cpu::instructions::Condition::NC,
        address: 0x2000,
    };
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

    assert_eq!(cpu.state.registers.pc, 0x2000);
}

#[test]
fn test_jp_cond_c_taken() {
    let mut cpu = CPU::new();
    let mut bus = MemoryBus::new(vec![0; 32768]);
    cpu.state.registers.pc = 0x1000;
    cpu.state.registers.modify_f(|f| f.set_carry(true));

    let instruction = crate::cpu::instructions::Instruction::JpCondImm16 {
        cond: crate::cpu::instructions::Condition::C,
        address: 0x2000,
    };
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

    assert_eq!(cpu.state.registers.pc, 0x2000);
}

#[test]
fn test_jp_hl() {
    let mut cpu = CPU::new();
    let mut bus = MemoryBus::new(vec![0; 32768]);
    cpu.state.registers.hl = 0x2000;

    let instruction = crate::cpu::instructions::Instruction::JpHl;
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

    assert_eq!(cpu.state.registers.pc, 0x2000);
}

#[test]
fn test_call_imm16() {
    let mut cpu = CPU::new();
    let mut bus = MemoryBus::new(vec![0; 32768]);
    cpu.state.registers.pc = 0x1003;
    cpu.state.registers.sp = 0xC000;

    let instruction = crate::cpu::instructions::Instruction::CallImm16 {
        address: 0x2000,
    };
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

    assert_eq!(cpu.state.registers.pc, 0x2000);    // Jumped to target
    assert_eq!(cpu.state.registers.sp, 0xBFFE);    // SP decreased by 2
    assert_eq!(bus.read(0xBFFF), 0x10);            // Return address low byte
    assert_eq!(bus.read(0xBFFE), 0x03);            // Return address high byte
}

#[test]
fn test_call_cond_nz_taken() {
    let mut cpu = CPU::new();
    let mut bus = MemoryBus::new(vec![0; 32768]);
    cpu.state.registers.pc = 0x1003;
    cpu.state.registers.sp = 0xC000;
    cpu.state.registers.modify_f(|f| f.set_zero(false)); // Z = 0

    let instruction = crate::cpu::instructions::Instruction::CallCondImm16 {
        cond: crate::cpu::instructions::Condition::NZ,
        address: 0x2000,
    };
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

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
    cpu.state.registers.modify_f(|f| f.set_zero(true));

    let instruction = crate::cpu::instructions::Instruction::CallCondImm16 {
        cond: crate::cpu::instructions::Condition::NZ,
        address: 0x2000,
    };
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

    assert_eq!(cpu.state.registers.pc, 0x1000); // Not called
    assert_eq!(cpu.state.registers.sp, 0xC000); // SP unchanged
}

#[test]
fn test_call_cond_c_taken() {
    let mut cpu = CPU::new();
    let mut bus = MemoryBus::new(vec![0; 32768]);
    cpu.state.registers.pc = 0x1000;
    cpu.state.registers.sp = 0xC000;
    cpu.state.registers.modify_f(|f| f.set_carry(true));

    let instruction = crate::cpu::instructions::Instruction::CallCondImm16 {
        cond: crate::cpu::instructions::Condition::C,
        address: 0x2000,
    };
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

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
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

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
    cpu.state.registers.modify_f(|f| f.set_zero(false)); // Z = 0

    let instruction = crate::cpu::instructions::Instruction::RetCond {
        cond: crate::cpu::instructions::Condition::NZ,
    };
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

    assert_eq!(cpu.state.registers.pc, 0x2002); // RET taken
    assert_eq!(cpu.state.registers.sp, 0xBFFE); // SP incremented by 2
}

#[test]
fn test_ret_cond_nz_not_taken() {
    let mut cpu = CPU::new();
    let mut bus = MemoryBus::new(vec![0; 32768]);
    cpu.state.registers.pc = 0x1000;
    cpu.state.registers.sp = 0xBFFC;
    cpu.state.registers.modify_f(|f| f.set_zero(true));

    let instruction = crate::cpu::instructions::Instruction::RetCond {
        cond: crate::cpu::instructions::Condition::NZ,
    };
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

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
    crate::cpu::exec::execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

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
    execute_instruction(&mut cpu.state, &mut bus, instruction, &mut noop_tick);

    assert_eq!(cpu.state.registers.pc, 0x0008);     // jumped to target
    assert_eq!(cpu.state.registers.sp, 0xBFFE);     // SP decremented by 2
    assert_eq!(bus.read(0xBFFF), 0x10);             // return address high byte
    assert_eq!(bus.read(0xBFFE), 0x00);             // return address low byte
}

