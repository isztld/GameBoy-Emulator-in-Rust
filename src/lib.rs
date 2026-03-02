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
pub mod input;
pub mod system;
pub mod config;
pub mod disasm;

pub use system::System;
pub use config::EmulatorFlags;
pub use cpu::{testing, CPU, execute_instruction};

// Re-export testing functions for convenience
pub use cpu::testing::{load_tests_from_dir, run_test_case, run_all_tests};
