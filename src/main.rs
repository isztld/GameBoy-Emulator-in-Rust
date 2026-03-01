//! GameBoy Emulator in Rust
//!
//! This emulator implements the GameBoy (DMG-01) hardware,
//! including the SM83 CPU, memory management, PPU, APU,
//! and all standard peripherals.

pub mod cpu;
pub mod memory;
pub mod ppu;
pub mod audio;
pub mod timer;
pub mod interrupt;
pub mod input;
pub mod system;
pub mod config;

use std::env;
use std::fs;
use std::path::Path;

use system::System;
use config::EmulatorFlags;

fn parse_flags() -> (EmulatorFlags, String) {
    let mut flags = EmulatorFlags::default();
    let args: Vec<String> = env::args().collect();

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
            "--help" | "-h" => {
                println!("Usage: {} [options] <rom_file>", args[0]);
                println!("\nOptions:");
                println!("  --cpu-log [file]      Enable CPU instruction logging (default: cpu_log.txt)");
                println!("  --serial-log [file]   Enable serial output logging (default: serial_log.txt)");
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

    if i >= args.len() {
        eprintln!("Usage: {} <rom_file>", args[0]);
        std::process::exit(1);
    }

    let rom_path = args[i].clone();
    (flags, rom_path)
}

fn main() {
    let (flags, rom_path) = parse_flags();

    // Load ROM
    let rom_data = match load_rom(&rom_path) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Failed to load ROM '{}': {}", rom_path, e);
            std::process::exit(1);
        }
    };

    println!("Loaded ROM: {} bytes", rom_data.len());
    println!("CPU logging: {} (-> {})", flags.log_cpu, flags.log_cpu_file);
    println!("Serial logging: {} (-> {})", flags.log_serial, flags.log_serial_file);

    // Create system with logging enabled
    let mut system = System::new(rom_data, flags);

    // Run the system
    system.start();

    // Main emulation loop
    while system.is_running() {
        system.step();

        // Check for frame completion (for display output)
        if system.frame_complete {
            // In a real emulator, you would render the frame here
            // For now, just reset the flag
            system.frame_complete = false;
        }
    }
}

/// Load a ROM file into a vector of bytes
fn load_rom(path: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let path = Path::new(path);
    let data = fs::read(path)?;
    Ok(data)
}
