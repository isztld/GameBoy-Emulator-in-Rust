/// GameBoy CPU implementation
///
/// The CPU is based on the SM83, a 8-bit CPU compatible with GBZ80.

use crate::memory::MemoryBus;
use crate::cpu::CPUState;
use crate::cpu::instructions::Instruction;
use crate::cpu::decode::decode_instruction;
use crate::cpu::exec::execute_instruction;

/// Interrupt vectors (address jumped to when servicing each interrupt).
const INT_VBLANK:  u16 = 0x0040;
const INT_STAT:    u16 = 0x0048;
const INT_TIMER:   u16 = 0x0050;
const INT_SERIAL:  u16 = 0x0058;
const INT_JOYPAD:  u16 = 0x0060;

/// Bit masks for the IE/IF registers.
const INT_BIT_VBLANK:  u8 = 1 << 0;
const INT_BIT_STAT:    u8 = 1 << 1;
const INT_BIT_TIMER:   u8 = 1 << 2;
const INT_BIT_SERIAL:  u8 = 1 << 3;
const INT_BIT_JOYPAD:  u8 = 1 << 4;

#[derive(Debug)]
pub struct CPU {
    state: CPUState,
    /// Total T-cycles executed (u64 avoids overflow at 4 MHz).
    cycles: u64,
    halted: bool,
    stopped: bool,
}

impl CPU {
    pub fn new() -> Self {
        let mut cpu = CPU {
            state: CPUState::new(),
            cycles: 0,
            halted: false,
            stopped: false,
        };
        cpu.reset();
        cpu
    }

    pub fn reset(&mut self) {
        self.state.registers.pc = 0x0100;
        self.state.registers.sp = 0xFFFE;
        self.state.registers.af = 0x01B0;
        self.state.registers.bc = 0x0013;
        self.state.registers.de = 0x00D8;
        self.state.registers.hl = 0x014D;
        self.state.ime = false;
        self.state.ime_pending = false;
        self.halted = false;
        self.stopped = false;
        self.cycles = 0;
    }

    pub fn state(&self) -> &CPUState { &self.state }
    pub fn state_mut(&mut self) -> &mut CPUState { &mut self.state }
    pub fn cycles(&self) -> u64 { self.cycles }
    /// True when the CPU is spinning in HALT or STOP — no instruction is executed this cycle.
    pub fn is_spinning(&self) -> bool { self.halted || self.stopped }

    /// Execute one step: service a pending interrupt OR execute one instruction.
    /// Returns the number of machine cycles consumed (1 machine cycle = 4 T-cycles).
    pub fn execute(&mut self, bus: &mut MemoryBus, tick: &mut dyn FnMut(&mut [u8; 128])) -> u32 {
        // --- Interrupt check ---------------------------------------------------
        // An interrupt can un-halt the CPU regardless of IME.
        let pending = bus.ie & bus.read(0xFF0F) & 0x1F;
        if pending != 0 {
            self.halted = false; // always wake from HALT on any pending interrupt
        }

        if self.state.ime && pending != 0 {
            let cycles = self.service_interrupt(bus, pending, tick);
            self.cycles += cycles as u64;
            return cycles;
        }

        // --- EI delay: promote ime_pending → ime ----------------------------------
        // EI enables interrupts after the *following* instruction, so we promote
        // the pending flag here (after the interrupt check) so that the NEXT call
        // to execute() will see ime=true when it checks for pending interrupts.
        if self.state.ime_pending {
            self.state.ime = true;
            self.state.ime_pending = false;
        }

        // --- HALT / STOP ------------------------------------------------------
        if self.halted || self.stopped {
            // Spin for 1 machine cycle, waiting for an interrupt to arrive.
            // The tick must fire so that the timer (and PPU) advance during HALT;
            // without it the timer interrupt would never arrive to wake the CPU.
            tick(&mut bus.io);
            self.cycles += 1;
            return 1;
        }

        // --- Normal instruction fetch / decode / execute ----------------------
        let pc = self.state.registers.pc;
        let opcode = bus.read(pc);
        // M-cycle 1: opcode fetch.  Every instruction pays this cost; the exec
        // sub-functions only tick for the *additional* M-cycles (immediate reads,
        // memory accesses, internal delays), so we must tick once here.
        tick(&mut bus.io);
        let (instruction, opcode_bytes) = decode_instruction(&self.state, bus, pc, opcode);

        // Advance PC past this instruction before executing, so that relative
        // jumps and calls that read PC (e.g. RST return address) see the correct
        // "next instruction" address.  Jump/call/return instructions will
        // overwrite PC themselves.
        self.state.registers.pc = pc.wrapping_add(opcode_bytes as u16);

        let cycles: u32 = match instruction {
            Instruction::HALT => {
                self.halted = true;
                1
            }
            Instruction::STOP => {
                // KEY1 (0xFF4D) bit 0 = CGB double-speed switch request.
                // On CGB, STOP after writing KEY1=1 performs the speed switch
                // and resumes immediately.  On DMG the write is ignored, but
                // some ROMs (including combined Blargg tests) hit this path
                // due to CGB-detection code that runs unconditionally.
                // Treat a pending speed-switch as a no-op rather than halting
                // forever (we don't support CGB double-speed).
                let key1 = bus.read(0xFF4D);
                if key1 & 0x01 != 0 {
                    // Clear the speed-switch request and continue (CGB compat).
                    bus.write(0xFF4D, key1 & !0x01);
                } else {
                    self.stopped = true;
                }
                1
            }
            instr => execute_instruction(&mut self.state, bus, instr, tick),
        };

        self.cycles += cycles as u64;
        cycles
    }

    /// Service the highest-priority pending interrupt.
    /// Clears the interrupt bit in IF, disables IME, and pushes PC onto the stack.
    /// Returns the number of machine cycles consumed (5).
    fn service_interrupt(&mut self, bus: &mut MemoryBus, pending: u8, tick: &mut dyn FnMut(&mut [u8; 128])) -> u32 {
        self.state.ime = false;

        // Find the highest-priority interrupt (lowest bit number).
        let (bit, vector) = if pending & INT_BIT_VBLANK != 0 {
            (INT_BIT_VBLANK, INT_VBLANK)
        } else if pending & INT_BIT_STAT != 0 {
            (INT_BIT_STAT, INT_STAT)
        } else if pending & INT_BIT_TIMER != 0 {
            (INT_BIT_TIMER, INT_TIMER)
        } else if pending & INT_BIT_SERIAL != 0 {
            (INT_BIT_SERIAL, INT_SERIAL)
        } else {
            (INT_BIT_JOYPAD, INT_JOYPAD)
        };

        // Clear the IF bit for this interrupt.
        let if_val = bus.read(0xFF0F);
        bus.write(0xFF0F, if_val & !bit);

        // Tick peripherals for each of the 5 M-cycles of interrupt dispatch:
        // 2 internal NOP cycles, push PC high, push PC low, set PC.
        tick(&mut bus.io); // M1: internal
        tick(&mut bus.io); // M2: internal
        let pc = self.state.registers.pc;
        let sp = self.state.registers.sp;
        bus.write(sp.wrapping_sub(1), (pc >> 8) as u8);
        tick(&mut bus.io); // M3: push PC high
        bus.write(sp.wrapping_sub(2), (pc & 0xFF) as u8);
        tick(&mut bus.io); // M4: push PC low
        self.state.registers.sp = sp.wrapping_sub(2);
        self.state.registers.pc = vector;
        tick(&mut bus.io); // M5: set PC

        5
    }
}

#[cfg(test)]
#[path = "cpu_tests.rs"]
mod tests;
