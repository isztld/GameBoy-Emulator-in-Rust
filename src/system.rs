//! GameBoy system module
//!
//! The System struct ties all components together:
//! - CPU
//! - Memory (MMU/MBC)
//! - PPU
//! - APU
//! - Timer
//! - Interrupts
//! - Input

pub use crate::cpu::{CPU, CPUState};
pub use crate::memory::MemoryBus;
pub use crate::ppu::video::VideoController;
pub use crate::audio::apu::AudioProcessor;
pub use crate::timer::Timer;
pub use crate::interrupt::InterruptController;
pub use crate::input::joypad::Joypad;

/// GameBoy System
pub struct System {
    pub cpu: CPU,
    pub mmu: MemoryBus,
    pub ppu: VideoController,
    pub apu: AudioProcessor,
    pub timer: Timer,
    pub interrupt: InterruptController,
    pub joypad: Joypad,
    pub running: bool,
    pub frame_complete: bool,
    pub total_cycles: u64,
    pub max_instructions: u64,
}

impl System {
    /// Create a new GameBoy system with the given ROM
    pub fn new(rom_data: Vec<u8>) -> Self {
        let mut system = System {
            cpu: CPU::new(),
            mmu: MemoryBus::new(rom_data),
            ppu: VideoController::new(),
            apu: AudioProcessor::new(),
            timer: Timer::new(),
            interrupt: InterruptController::new(),
            joypad: Joypad::new(),
            running: false,
            frame_complete: false,
            total_cycles: 0,
            max_instructions: 100000, // Run for max 100000 instructions (enough for CPU test)
        };
        system.reset(); // Initialize CPU registers
        system
    }

    /// Reset the system to power-on state
    pub fn reset(&mut self) {
        self.cpu.reset();
        self.timer.reset();
        self.interrupt = InterruptController::new();
        self.frame_complete = false;
    }

    /// Run the system for one CPU instruction
    pub fn step(&mut self) {
        // Check if we've exceeded max instructions
        if self.total_cycles >= self.max_instructions {
            self.running = false;
            return;
        }

        // Execute CPU instruction
        let cycles = self.cpu.execute(&mut self.mmu);

        // Log instruction
        let pc = self.cpu.state().registers.pc;
        let a = self.cpu.state().registers.a();
        let f = self.cpu.state().registers.f();
        let b = self.cpu.state().registers.b();
        let c = self.cpu.state().registers.c();
        let d = self.cpu.state().registers.d();
        let e = self.cpu.state().registers.e();
        let h = self.cpu.state().registers.h();
        let l = self.cpu.state().registers.l();
        let sp = self.cpu.state().registers.sp;
        println!("PC=${:04X} A:${:02X} F:{:02X} BC:${:04X} DE:${:04X} HL:${:04X} SP:${:04X} CYCLES:{}",
            pc, a, f.get(), (b as u16) << 8 | c as u16, (d as u16) << 8 | e as u16, (h as u16) << 8 | l as u16, sp, cycles);

        // Update timer (DIV increments every 4 cycles at 16384 Hz)
        for _ in 0..cycles {
            self.timer.increment_div();
        }

        // Update PPU
        for _ in 0..cycles {
            self.ppu.update(&mut self.mmu);
        }

        // Update audio (clocked at 2x CPU frequency)
        for _ in 0..(cycles * 2) {
            self.apu.clock();
        }

        // Update timer
        for _ in 0..cycles {
            self.timer.clock();
        }

        // Update cycle count
        self.total_cycles += cycles as u64;

        // Check for pending interrupts
        self.check_interrupts();
    }

    /// Check for pending interrupts and handle them
    fn check_interrupts(&mut self) {
        if self.interrupt.has_pending() {
            if self.cpu.state().ime {
                if let Some(vector) = self.interrupt.get_pending_vector() {
                    // Handle interrupt
                    self.cpu.state_mut().ime = false;
                    self.interrupt.acknowledge(vector);

                    // Push PC to stack and jump to vector
                    let sp = self.cpu.state().registers.sp;
                    let pc = self.cpu.state().registers.pc;

                    // Push high byte
                    self.mmu.write(sp.wrapping_sub(1), (pc >> 8) as u8);
                    // Push low byte
                    self.mmu.write(sp.wrapping_sub(2), (pc & 0x00FF) as u8);

                    // Update SP and PC
                    self.cpu.state_mut().registers.sp = sp.wrapping_sub(2);
                    self.cpu.state_mut().registers.pc = vector;

                    // V-Blank interrupt triggers frame complete
                    if vector == 0x40 {
                        self.frame_complete = true;
                    }
                }
            }
        }
    }

    /// Run until a frame is complete
    pub fn run_frame(&mut self) {
        self.frame_complete = false;
        while !self.frame_complete && self.running {
            self.step();
        }
    }

    /// Start the system
    pub fn start(&mut self) {
        self.running = true;
    }

    /// Stop the system
    pub fn stop(&mut self) {
        self.running = false;
    }

    /// Check if system is running
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Get current CPU state
    pub fn cpu_state(&self) -> &CPUState {
        self.cpu.state()
    }

    /// Get audio output
    pub fn get_audio_output(&self) -> crate::audio::apu::AudioOutput {
        self.apu.get_output()
    }
}

impl Default for System {
    fn default() -> Self {
        System::new(vec![0; 32768]) // Default 32 KiB ROM
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_create() {
        let rom = vec![0; 32768];
        let system = System::new(rom);
        assert!(!system.is_running());
        assert_eq!(system.cpu_state().registers.pc, 0x0100); // PC should be 0x0100 after reset in System::new
    }

    #[test]
    fn test_system_reset() {
        let mut system = System::new(vec![0; 32768]);
        system.cpu.state_mut().registers.pc = 0x1234;
        system.reset();
        // After reset, PC should be 0x0100 (from CPU::reset)
        assert_eq!(system.cpu_state().registers.pc, 0x0100);
    }
}
