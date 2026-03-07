//! GameBoy Emulator in Rust
//!
//! This emulator implements the GameBoy (DMG-01) hardware,
//! including the SM83 CPU, memory management, PPU, APU,
//! and all standard peripherals.

use std::env;
use std::fs;
use std::path::Path;

use gb_emu::{System, EmulatorFlags, disasm};
use gb_emu::memory::MemoryBus as MemoryBusAlias;

fn parse_flags() -> (EmulatorFlags, String, bool) {
    let mut flags = EmulatorFlags::default();
    let args: Vec<String> = env::args().collect();

    let mut disassemble = false;
    let mut i = 1; // Skip program name
    while i < args.len() {
        match args[i].as_str() {
            "--cpu-log" => {
                flags.log_cpu = true;
                if i + 1 < args.len() && !args[i + 1].starts_with("--") {
                    i += 1;
                    flags.log_cpu_file = args[i].clone();
                }
            }
            "--serial-log" => {
                flags.log_serial = true;
                if i + 1 < args.len() && !args[i + 1].starts_with("--") {
                    i += 1;
                    flags.log_serial_file = args[i].clone();
                }
            }
            "--cpu-json-test" => {
                flags.cpu_json_test = true;
                if i + 1 < args.len() && !args[i + 1].starts_with("--") {
                    i += 1;
                    flags.cpu_json_test_dir = Some(args[i].clone());
                } else {
                    eprintln!("--cpu-json-test requires string path to GameboyCPUTests/");
                    std::process::exit(1);
                }
            }
            "--disasm" => {
                disassemble = true;
            }
            "--cycle-limit" => {
                if i + 1 < args.len() && !args[i + 1].starts_with("--") {
                    i += 1;
                    flags.cycle_limit = Some(args[i].parse::<u64>().expect("Invalid cycle limit value"));
                } else {
                    eprintln!("--cycle-limit requires a numeric argument");
                    std::process::exit(1);
                }
            }
            "--help" | "-h" => {
                println!("Usage: {} [options] <rom_file>", args[0]);
                println!("\nOptions:");
                println!("  --cpu-log [file]      Enable CPU instruction logging (default: cpu_log.txt)");
                println!("  --serial-log [file]   Enable serial output logging (default: serial_log.txt)");
                println!("  --disasm              Disassemble ROM instead of running");
                println!("  --cycle-limit <n>     Set maximum cycles to execute");
                println!("  --help, -h            Show this help message");
                std::process::exit(0);
            }
            _ if !args[i].starts_with("--") => break,
            _ => {
                eprintln!("Unknown option: {}", args[i]);
                std::eprintln!("Use --help for usage information");
                std::process::exit(1);
            }
        }
        i += 1;
    }

    if flags.cpu_json_test {
        return (flags, "".to_string(), disassemble)
    }

    if i >= args.len() {
        eprintln!("Usage: {} <rom_file>", args[0]);
        std::process::exit(1);
    }

    let rom_path = args[i].clone();
    (flags, rom_path, disassemble)
}

fn main() {
    let (flags, rom_path, disassemble) = parse_flags();

    if flags.cpu_json_test && let Some(cpu_json_test_dir) = &flags.cpu_json_test_dir {
        use gb_emu::run_all_tests;

        println!("Loading tests from: {}", cpu_json_test_dir);
        let start_time = std::time::Instant::now();

        let (passed, failed, failures) = run_all_tests(cpu_json_test_dir);

        let duration = start_time.elapsed();
        println!("\n=== Test Results ===");
        println!("Total: {}", passed + failed);
        println!("Passed: {}", passed);
        println!("Failed: {}", failed);
        println!("Time: {:.2?}", duration);

        if failed > 0 {
            println!("\n=== Failures ===");
            for failure in failures.iter().take(20) {
                println!("{}", failure);
            }
            if failures.len() > 20 {
                println!("... and {} more failures", failures.len() - 20);
            }
            std::process::exit(1);
        } else {
            println!("\nAll tests passed!");
            std::process::exit(0);
        }
    }else if flags.cpu_json_test {
        eprintln!("cpu_json_test_dir was not set");
        std::process::exit(1);
    }

    // Load ROM
    let rom_data = match load_rom(&rom_path) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Failed to load ROM '{}': {}", rom_path, e);
            std::process::exit(1);
        }
    };

    if disassemble {
        // Disassemble mode
        let bus = MemoryBusAlias::new(rom_data);
        let instructions = disasm::disasm_region(&bus, 0x0100, 50);
        println!("Disassembly starting at 0x0100:");
        for instr in instructions {
            let hex: Vec<String> = instr.bytes.iter().map(|b| format!("{:02X}", b)).collect();
            println!("${:04X} {:12} {:8} {}", instr.address, hex.join(" "), instr.mnemonic, instr.operand_str);
        }
        return;
    }

    println!("Loaded ROM: {} bytes", rom_data.len());
    if flags.log_cpu {
        println!("CPU logging enabled (-> {})", flags.log_cpu_file);
    }
    if flags.log_serial {
        println!("Serial logging enabled (-> {})", flags.log_serial_file);
    }

    // Create system with logging enabled
    let mut system = System::new(rom_data, flags);

    // Run the system
    system.start();

    // Main emulation loop
    while system.is_running() {
        system.step();

        // Check for frame completion (for display output).
        let _ = system.take_frame_complete();
    }
}

/// Load a ROM file into a vector of bytes
fn load_rom(path: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let path = Path::new(path);
    let data = fs::read(path)?;
    Ok(data)
}
