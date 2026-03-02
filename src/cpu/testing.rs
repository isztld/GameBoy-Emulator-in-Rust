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
    // First, load the ROM with the test instruction bytes
    // The name field contains hex bytes like "cd a5 d0"
    let name_parts: Vec<&str> = test.name.split_whitespace().collect();
    let mut rom = vec![0u8; 65536]; // Full 64KB ROM

    // Parse the instruction bytes from name
    let mut pc_offset = 0;
    for part in &name_parts {
        if let Ok(byte) = u8::from_str_radix(part, 16) {
            rom[test.initial.pc as usize + pc_offset] = byte;
            pc_offset += 1;
        }
    }

    // Create a new bus with the ROM
    let mut bus = MemoryBus::new(rom);

    // Restore initial RAM values
    for &(addr, val) in &test.initial.ram {
        bus.write(addr, val);
    }

    // Create initial CPU state
    let initial_state = create_cpu_state(&test.initial);

    // Create CPU with initial state
    let mut cpu = CPU::new();
    cpu.state_mut().registers = initial_state.registers;
    cpu.state_mut().ime = initial_state.ime;
    cpu.state_mut().halted = initial_state.halted;
    cpu.state_mut().stopped = initial_state.stopped;

    // Execute until we reach target PC
    let target_pc = test.final_state.pc;

    while cpu.state().registers.pc != target_pc {
        let pc = cpu.state().registers.pc;
        let opcode = bus.read(pc);

        // Decode and execute
        let (instruction, opcode_bytes) = decode_instruction(cpu.state(), &bus, pc, opcode);

        // Advance PC before execution (matching the real CPU behavior)
        cpu.state_mut().registers.pc = pc.wrapping_add(opcode_bytes as u16);

        // Execute the instruction
        execute_instruction(cpu.state_mut(), &mut bus, instruction);

        // Safety limit to prevent infinite loops
        if cpu.cycles() > 10000 {
            return Err(format!("Test exceeded max cycles (PC={:04X}, target={:04X})",
                cpu.state().registers.pc, target_pc));
        }
    }

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
    let mut passed = 0;
    let mut failures = Vec::new();

    for (i, test) in tests.iter().enumerate() {
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
