use super::*;

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
    let cycles1 = cpu.execute(&mut bus, &mut noop_tick);
    assert_eq!(cycles1, 2);
    assert_eq!(cpu.cycles(), 2);
    assert_eq!(cpu.state.registers.a(), 0x10);

    // Execute ADD A, 0x20
    let cycles2 = cpu.execute(&mut bus, &mut noop_tick);
    assert_eq!(cycles2, 2);
    assert_eq!(cpu.cycles(), 4);
    assert_eq!(cpu.state.registers.a(), 0x30);

    // Execute RET (returns to 0x0100 which loops back)
    let cycles3 = cpu.execute(&mut bus, &mut noop_tick);
    assert_eq!(cycles3, 4);
}

#[test]
fn test_cpu_halt() {
    let mut cpu = CPU::new();
    let mut bus = MemoryBus::new(vec![0; 32768]);
    cpu.halted = true;

    // When halted, CPU should return 1 cycle
    let cycles = cpu.execute(&mut bus, &mut noop_tick);
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
