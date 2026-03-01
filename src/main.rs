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

use std::env;
use std::fs;
use std::path::Path;

use system::System;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <rom_file>", args[0]);
        std::process::exit(1);
    }

    let rom_path = &args[1];

    // Load ROM
    let rom_data = match load_rom(rom_path) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Failed to load ROM '{}': {}", rom_path, e);
            std::process::exit(1);
        }
    };

    println!("Loaded ROM: {} bytes", rom_data.len());

    // Create system
    let mut system = System::new(rom_data);

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
