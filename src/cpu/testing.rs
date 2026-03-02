/// Test runner for Gameboy CPU tests
///
/// This module loads and executes the official CPU test suite from the
/// GameboyCPUTests directory.

use crate::cpu::{CPUState, CPU};
use crate::memory::MemoryBus;
use crate::cpu::decode::decode_instruction;
use crate::cpu::exec::execute_instruction;

use serde::Deserialize;
use std::fs;

/// Represents a test case from the JSON file
#[derive(Debug, Deserialize)]
pub struct TestCase {
    pub name: String,
    pub initial: TestState,
    #[serde(rename = "final")]
    pub final_state: TestState,
    pub cycles: Vec<Option<MemoryTransaction>>,
}

/// Represents the initial/final state of registers and RAM
#[derive(Debug, Deserialize)]
pub struct TestState {
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub f: u8,
    pub h: u8,
    pub l: u8,
    pub pc: u16,
    pub sp: u16,
    pub ram: Vec<(u16, u8)>,
}

/// Represents a memory transaction (read/write)
#[derive(Debug, Deserialize, Clone)]
pub struct MemoryTransaction {
    pub address: u16,
    pub value: u8,
    #[serde(rename = "type")]
    pub txn_type: String,
}

/// Load all test cases from a directory
pub fn load_tests_from_dir(dir: &str) -> Vec<TestCase> {
    let mut tests = Vec::new();

    for entry in fs::read_dir(dir).expect("Failed to read test directory") {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();

        if path.extension().map_or(false, |ext| ext == "json") {
            let content = fs::read_to_string(&path).expect("Failed to read test file");
            let file_tests: Vec<TestCase> = serde_json::from_str(&content)
                .expect("Failed to parse test JSON");

            tests.extend(file_tests);
        }
    }

    tests
}

/// Run a single test case and return whether it passed
pub fn run_test_case(test: &TestCase) -> Result<(), String> {
    // Pre-populate ROM data so that writes to ROM-mapped addresses (0x0000–0x7FFF)
    // are visible to reads. bus.write() in that range hits the MBC control path and
    // never modifies self.rom, so we must seed the ROM vector directly.
    let mut rom_data = vec![0u8; 65536];
    for &(addr, val) in &test.initial.ram {
        if addr < 0x8000 {
            rom_data[addr as usize] = val;
        }
    }
    let mut bus = MemoryBus::new(rom_data);

    // Write non-ROM initial values through the normal bus path.
    for &(addr, val) in &test.initial.ram {
        if addr >= 0x8000 {
            bus.write(addr, val);
        }
    }

    // Create initial CPU state
    let initial_state = create_cpu_state(&test.initial);

    // Create CPU with initial state
    let mut cpu = CPU::new();
    cpu.state_mut().registers = initial_state.registers;
    cpu.state_mut().ime = initial_state.ime;
    cpu.state_mut().halted = initial_state.halted;
    cpu.state_mut().stopped = initial_state.stopped;

    eprintln!("  Initial PC: {:04X}, A={:02X}, B={:02X}, C={:02X}, D={:02X}, E={:02X}, H={:02X}, L={:02X}",
        cpu.state().registers.pc,
        cpu.state().registers.a(),
        cpu.state().registers.b(),
        cpu.state().registers.c(),
        cpu.state().registers.d(),
        cpu.state().registers.e(),
        cpu.state().registers.h(),
        cpu.state().registers.l());

    // Get the target PC from the test
    let _target_pc = test.final_state.pc;

    // The SM83 tests use a decode-execute-prefetch loop: initial.pc is already past the
    // opcode byte (the opcode was pre-fetched at initial.pc - 1).  Operands start at
    // initial.pc and the last M-cycle is always a prefetch of the next instruction that
    // reads from [PC] and increments PC by 1.
    let pc = cpu.state().registers.pc; // = initial.pc (already past the opcode)
    let opcode = bus.read(pc.wrapping_sub(1)); // opcode is at initial.pc - 1

    eprintln!("  PC={:04X}, opcode={:02X}", pc, opcode);

    // Decode: pass pc-1 so operand reads land at initial.pc, initial.pc+1, etc.
    let (instruction, opcode_bytes) = decode_instruction(cpu.state(), &bus, pc.wrapping_sub(1), opcode);
    eprintln!("  instruction={:?}, opcode_bytes={}", instruction, opcode_bytes);

    // Advance PC past the operand bytes only (the opcode byte is already consumed).
    // For a 1-byte instruction opcode_bytes == 1, so this leaves PC == initial.pc.
    cpu.state_mut().registers.pc = pc.wrapping_add(opcode_bytes as u16 - 1);

    // Execute the instruction (jumps/calls override PC to their target).
    execute_instruction(cpu.state_mut(), &mut bus, instruction);

    // Prefetch cycle: every instruction's final M-cycle reads the next opcode byte,
    // incrementing PC by 1 regardless of whether a jump was taken.
    cpu.state_mut().registers.pc = cpu.state().registers.pc.wrapping_add(1);

    eprintln!("  Final PC: {:04X}, A={:02X}, B={:02X}, C={:02X}, D={:02X}, E={:02X}, H={:02X}, L={:02X}",
        cpu.state().registers.pc,
        cpu.state().registers.a(),
        cpu.state().registers.b(),
        cpu.state().registers.c(),
        cpu.state().registers.d(),
        cpu.state().registers.e(),
        cpu.state().registers.h(),
        cpu.state().registers.l());

    // Verify final state
    verify_state(test, &bus, cpu.state())
}

/// Create a CPUState from TestState
fn create_cpu_state(state: &TestState) -> CPUState {
    let mut cpu = CPUState::new();
    cpu.registers.set_a(state.a);
    cpu.registers.set_b(state.b);
    cpu.registers.set_c(state.c);
    cpu.registers.set_d(state.d);
    cpu.registers.set_e(state.e);
    // F register - only upper 4 bits are used
    let f_val = (state.f & 0xF0) as u16;
    cpu.registers.af = (cpu.registers.af & 0xFF00) | f_val;
    cpu.registers.set_h(state.h);
    cpu.registers.set_l(state.l);
    cpu.registers.pc = state.pc;
    cpu.registers.sp = state.sp;
    cpu
}

/// Verify that the actual state matches the expected final state
fn verify_state(test: &TestCase, bus: &MemoryBus, cpu_state: &CPUState) -> Result<(), String> {
    // Check A register
    if cpu_state.registers.a() != test.final_state.a {
        return Err(format!("A mismatch: actual={:02X}, expected={:02X}",
            cpu_state.registers.a(), test.final_state.a));
    }

    // Check B register
    if cpu_state.registers.b() != test.final_state.b {
        return Err(format!("B mismatch: actual={:02X}, expected={:02X}",
            cpu_state.registers.b(), test.final_state.b));
    }

    // Check C register
    if cpu_state.registers.c() != test.final_state.c {
        return Err(format!("C mismatch: actual={:02X}, expected={:02X}",
            cpu_state.registers.c(), test.final_state.c));
    }

    // Check D register
    if cpu_state.registers.d() != test.final_state.d {
        return Err(format!("D mismatch: actual={:02X}, expected={:02X}",
            cpu_state.registers.d(), test.final_state.d));
    }

    // Check E register
    if cpu_state.registers.e() != test.final_state.e {
        return Err(format!("E mismatch: actual={:02X}, expected={:02X}",
            cpu_state.registers.e(), test.final_state.e));
    }

    // Check H register
    if cpu_state.registers.h() != test.final_state.h {
        return Err(format!("H mismatch: actual={:02X}, expected={:02X}",
            cpu_state.registers.h(), test.final_state.h));
    }

    // Check L register
    if cpu_state.registers.l() != test.final_state.l {
        return Err(format!("L mismatch: actual={:02X}, expected={:02X}",
            cpu_state.registers.l(), test.final_state.l));
    }

    // Check F register (only upper 4 bits are valid)
    let actual_f = cpu_state.registers.f().get() & 0xF0;
    let expected_f = test.final_state.f & 0xF0;
    if actual_f != expected_f {
        return Err(format!("F mismatch: actual={:02X}, expected={:02X}",
            actual_f, expected_f));
    }

    // Check SP
    if cpu_state.registers.sp != test.final_state.sp {
        return Err(format!("SP mismatch: actual={:04X}, expected={:04X}",
            cpu_state.registers.sp, test.final_state.sp));
    }

    // Check PC
    if cpu_state.registers.pc != test.final_state.pc {
        return Err(format!("PC mismatch: actual={:04X}, expected={:04X}",
            cpu_state.registers.pc, test.final_state.pc));
    }

    // Check RAM changes
    for &(addr, expected_val) in &test.final_state.ram {
        let actual_val = bus.read(addr);
        if actual_val != expected_val {
            return Err(format!("RAM[{:04X}] mismatch: actual={:02X}, expected={:02X}",
                addr, actual_val, expected_val));
        }
    }

    Ok(())
}

/// Run all tests from a directory and return results
pub fn run_all_tests(dir: &str) -> (usize, usize, Vec<String>) {
    let tests = load_tests_from_dir(dir);
    let total = tests.len();
    eprintln!("Loaded {} tests", total);
    let mut passed = 0;
    let mut failures = Vec::new();

    for (i, test) in tests.iter().enumerate() {
        eprintln!("[{:02}] Running test: {}", i, test.name);
        match run_test_case(test) {
            Ok(()) => passed += 1,
            Err(msg) => {
                failures.push(format!("Test {}: {} - {}", i, test.name, msg));
            }
        }

        if (i + 1) % 100 == 0 {
            eprintln!("Progress: {}/{} tests completed", i + 1, total);
        }
    }

    (passed, total - passed, failures)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_tests() {
        // Get the directory where the test file is located
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
        let test_dir = format!("{}/GameboyCPUTests", manifest_dir);

        let tests = load_tests_from_dir(&test_dir);
        assert!(tests.len() > 0, "Should load at least one test");
    }

    #[test]
    fn test_run_nop() {
        // NOP instruction - just advance PC by 1
        let test = TestCase {
            name: "00".to_string(),
            initial: TestState {
                a: 0, b: 0, c: 0, d: 0, e: 0, f: 0, h: 0, l: 0,
                pc: 0x100, sp: 0xFFFE,
                ram: vec![],
            },
            final_state: TestState {
                a: 0, b: 0, c: 0, d: 0, e: 0, f: 0, h: 0, l: 0,
                pc: 0x101, sp: 0xFFFE,
                ram: vec![],
            },
            cycles: vec![None],
        };

        // This should pass
        let result = run_test_case(&test);
        assert!(result.is_ok(), "NOP test should pass: {:?}", result);
    }
}
