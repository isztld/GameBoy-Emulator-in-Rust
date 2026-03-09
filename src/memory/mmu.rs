/// Memory Management Unit (MMU) for GameBoy
///
/// The GameBoy has a 16-bit address bus, allowing access to 64 KiB of address space.
/// Memory map:
/// - 0000-3FFF: 16 KiB ROM Bank 0
/// - 4000-7FFF: 16 KiB ROM Bank 1-NN (switchable via MBC)
/// - 8000-9FFF: 8 KiB Video RAM (VRAM)
/// - A000-BFFF: 8 KiB External RAM (from cartridge)
/// - C000-CFFF: 4 KiB Work RAM (WRAM)
/// - D000-DFEF: 4 KiB Work RAM (bankable on CGB)
/// - E000-FDFF: Echo RAM (mirror of C000-DDFF)
/// - FE00-FE9F: Object Attribute Memory (OAM)
/// - FEA0-FEFF: Not usable
/// - FF00-FF7F: I/O Registers
/// - FF80-FFFE: High RAM (HRAM)
/// - FFFF: Interrupt Enable (IE) register

use crate::memory::mbc::MemoryBankController;
use std::io::Write;
use std::sync::{Arc, Mutex};

/// Memory bus for the GameBoy
#[derive(Debug)]
pub struct MemoryBus {
    pub(crate) rom: Vec<u8>,
    pub(crate) vram: [u8; 8192],
    pub(crate) external_ram: [u8; 8192],
    pub(crate) wram: [u8; 8192],
    pub(crate) hram: [u8; 127],
    pub(crate) oam: [u8; 160],
    pub(crate) io: [u8; 128],
    /// When true, all reads and writes go directly to `rom` as a flat 64 KiB
    /// array, bypassing all memory-mapped regions and the MBC.  Used by the
    /// CPU test harness.
    pub flat_mode: bool,
    pub(crate) ie: u8,
    pub(crate) mbc: MemoryBankController,

    // Pending timer register writes — set by write_io and drained by System::step.
    pub(crate) timer_div_reset: bool,
    pub(crate) timer_tma_write: Option<u8>,
    pub(crate) timer_tac_write: Option<u8>,

    // Pending APU register writes — set by write_io and drained by System::step.
    pub(crate) apu_writes: Vec<(u16, u8)>,

    /// Action button states (1=pressed): bit0=A, bit1=B, bit2=Select, bit3=Start
    pub(crate) joypad_action: u8,
    /// D-pad states (1=pressed): bit0=Right, bit1=Left, bit2=Up, bit3=Down
    pub(crate) joypad_dpad: u8,

    /// Remaining M-cycles until the active OAM DMA transfer completes.
    pub(crate) oam_dma_cycles_remaining: u8,

    /// Optional file for serial output logging.  Set after construction to
    /// redirect serial bytes to a file instead of stdout.
    pub serial_log_file: Option<Arc<Mutex<std::fs::File>>>,
}

impl MemoryBus {
    /// Write a character to the serial log or stdout if not configured.
    /// Does NOT append a newline — callers write raw characters.
    fn write_serial_byte(&self, byte: u8) {
        if let Some(ref file) = self.serial_log_file {
            let mut f = file.lock().unwrap();
            f.write_all(&[byte]).ok(); // write raw byte
            f.flush().ok();
        } else {
            // Fallback to stdout: only print printable ASCII or space
            if byte.is_ascii_graphic() || byte == b' ' {
                print!("{}", byte as char);
            } else if byte == b'\n' {
                print!("\n");
            } else if byte == b'\r' {
                print!("\r");
            } else {
                // Show non-printable as hex in brackets
                print!("[{:02X}]", byte);
            }
            std::io::stdout().flush().ok();
        }
    }

    pub fn new(rom_data: Vec<u8>) -> Self {
        // In MemoryBus::new, after rom_data is validated to be at least 0x148 bytes:
        let header_byte = rom_data.get(0x0147).copied().unwrap_or(0x00);
        let mbc = MemoryBankController::new(rom_data.len(), header_byte);

        let mut io = [0u8; 128];
        io[0x00] = 0xCF; // P1/JOYP
        io[0x04] = 0x00; // DIV
        io[0x07] = 0xF8; // TAC
        io[0x0F] = 0xE0; // IF - bit 0 = 0 (no pending interrupts), bits 5-7 = 1 (open bus)
        io[0x10] = 0x80; // NR10
        io[0x11] = 0xBF; // NR11
        io[0x12] = 0xF3; // NR12
        io[0x13] = 0xFF; // NR13
        io[0x14] = 0xBF; // NR14
        io[0x15] = 0xFF; // unused (open bus)
        io[0x16] = 0x3F; // NR21
        io[0x17] = 0x00; // NR22
        io[0x18] = 0xFF; // NR23
        io[0x19] = 0xBF; // NR24
        io[0x1A] = 0x7F; // NR30
        io[0x1B] = 0xFF; // NR31
        io[0x1C] = 0x9F; // NR32
        io[0x1D] = 0xFF; // NR33
        io[0x1E] = 0xBF; // NR34
        io[0x1F] = 0xFF; // unused (open bus)
        io[0x20] = 0xFF; // NR41
        io[0x21] = 0x00; // NR42
        io[0x22] = 0x00; // NR43
        io[0x23] = 0xBF; // NR44
        io[0x24] = 0x77; // NR50
        io[0x25] = 0xF3; // NR51
        io[0x26] = 0xF1; // NR52
        io[0x27..=0x2F].fill(0xFF); // unused (open bus)
        // Wave RAM power-on state (DMG, matching WaveChannel::new())
        let wave_init = [0x84u8, 0x40, 0x43, 0xAA, 0x2D, 0x78, 0x92, 0x3C,
                         0x60, 0x59, 0xAD, 0xA1, 0x0C, 0xE2, 0xF3, 0x44];
        io[0x30..0x40].copy_from_slice(&wave_init);
        io[0x40] = 0x91; // LCDC
        // STAT: bits 3-6 writable by software; start in mode 1 (VBlank) with no
        // interrupt-select bits armed so no spurious STAT IRQ fires immediately.
        io[0x41] = 0x01; // STAT — mode 1, no interrupt selects set
        io[0x44] = 0x00; // LY
        io[0x45] = 0x00; // LYC
        io[0x46] = 0xFF; // DMA
        io[0x47] = 0xFC; // BGP  (DMG power-on default)
        io[0x48] = 0xFF; // OBP0 (DMG power-on default)
        io[0x49] = 0xFF; // OBP1 (DMG power-on default)
        io[0x4A] = 0x00; // WY
        io[0x4B] = 0x00; // WX

        MemoryBus {
            rom: rom_data,
            vram: [0u8; 8192],
            external_ram: [0u8; 8192],
            wram: [0u8; 8192],
            hram: [0u8; 127],
            oam: [0u8; 160],
            io,
            ie: 0x00,
            mbc,
            flat_mode: false,
            timer_div_reset: false,
            timer_tma_write: None,
            timer_tac_write: None,
            joypad_action: 0,
            joypad_dpad: 0,
            oam_dma_cycles_remaining: 0,
            serial_log_file: None,
            apu_writes: Vec::new(),
        }
    }

    /// Read a byte from memory.
    pub fn read(&self, address: u16) -> u8 {
        if self.flat_mode {
            return self.rom.get(address as usize).copied().unwrap_or(0xFF);
        }

        // During OAM DMA the CPU may only access HRAM ($FF80–$FFFE).
        // All other reads return $FF (DMG hardware behaviour).
        if self.oam_dma_cycles_remaining > 0 {
            match address {
                0xFF80..=0xFFFE => return self.hram[(address - 0xFF80) as usize],
                0xFFFF           => return self.ie,
                _                => return 0xFF,
            }
        }

        match address {
            // ROM bank 0 — always mapped directly.
            0x0000..=0x3FFF => {
                let idx = self.mbc.rom_bank0_offset() + address as usize;
                self.rom.get(idx).copied().unwrap_or(0xFF)
            }
            // ROM bank 1-NN — offset determined by MBC.
            0x4000..=0x7FFF => {
                let bank_offset = self.mbc.rom_bank_offset(); // byte offset of the active bank
                let idx = bank_offset + (address - 0x4000) as usize;
                self.rom.get(idx).copied().unwrap_or(0xFF)
            }
            // VRAM is inaccessible during Mode 3 (pixel transfer) when the LCD is on.
            0x8000..=0x9FFF => {
                if self.io[0x40] & 0x80 != 0 && self.io[0x41] & 0x03 == 0x03 {
                    0xFF
                } else {
                    self.vram[(address - 0x8000) as usize]
                }
            }
            0xA000..=0xBFFF => self.external_ram[(address - 0xA000) as usize],
            0xC000..=0xCFFF => self.wram[(address - 0xC000) as usize],
            0xD000..=0xDFFF => self.wram[(address - 0xC000) as usize],
            // Echo RAM mirrors C000-DDFF (not all the way to DFFF).
            0xE000..=0xFDFF => self.wram[(address - 0xE000) as usize],
            // OAM is inaccessible during Modes 2-3 (OAM scan + pixel transfer) when LCD is on.
            0xFE00..=0xFE9F => {
                let mode = self.io[0x41] & 0x03;
                if self.io[0x40] & 0x80 != 0 && mode >= 0x02 {
                    0xFF
                } else {
                    self.oam[(address - 0xFE00) as usize]
                }
            }
            // Unusable region — return high nibble of low address byte, repeated.
            0xFEA0..=0xFEFF => self.read_fea0(address),
            0xFF00..=0xFF7F => self.io[(address - 0xFF00) as usize],
            0xFF80..=0xFFFE => self.hram[(address - 0xFF80) as usize],
            0xFFFF => self.ie,
        }
    }

    /// Write a byte to memory.
    pub fn write(&mut self, address: u16, value: u8) {
        if self.flat_mode {
            if let Some(slot) = self.rom.get_mut(address as usize) {
                *slot = value;
            }
            return;
        }

        // During OAM DMA the CPU may only access HRAM; all other writes are ignored.
        if self.oam_dma_cycles_remaining > 0 {
            match address {
                0xFF80..=0xFFFE => { self.hram[(address - 0xFF80) as usize] = value; }
                0xFFFF           => { self.ie = value; }
                _                => {}
            }
            return;
        }

        match address {
            // ROM area — MBC intercepts control writes; ROM itself is read-only.
            0x0000..=0x7FFF => {
                self.mbc.write_rom_control(address, value);
            }
            // VRAM writes ignored during Mode 3 when LCD is on.
            0x8000..=0x9FFF => {
                if self.io[0x40] & 0x80 == 0 || self.io[0x41] & 0x03 != 0x03 {
                    self.vram[(address - 0x8000) as usize] = value;
                }
            }
            0xA000..=0xBFFF => self.external_ram[(address - 0xA000) as usize] = value,
            0xC000..=0xCFFF => self.wram[(address - 0xC000) as usize] = value,
            0xD000..=0xDFFF => self.wram[(address - 0xC000) as usize] = value,
            // Echo RAM mirrors C000-DDFF.
            0xE000..=0xFDFF => self.wram[(address - 0xE000) as usize] = value,
            // OAM writes ignored during Modes 2-3 when LCD is on.
            0xFE00..=0xFE9F => {
                let mode = self.io[0x41] & 0x03;
                if self.io[0x40] & 0x80 == 0 || mode < 0x02 {
                    self.oam[(address - 0xFE00) as usize] = value;
                }
            }
            0xFEA0..=0xFEFF => {} // Unusable — writes ignored.
            0xFF00..=0xFF7F => self.write_io(address, value),
            0xFF80..=0xFFFE => self.hram[(address - 0xFF80) as usize] = value,
            0xFFFF => self.ie = value,
        }
    }

    /// Unusable region 0xFEA0–0xFEFF.
    /// Returns the high nibble of the low address byte mirrored into both nibbles.
    fn read_fea0(&self, address: u16) -> u8 {
        let nibble = ((address & 0x00F0) >> 4) as u8;
        (nibble << 4) | nibble
    }

    /// Handle writes to I/O registers (0xFF00–0xFF7F).
    fn write_io(&mut self, address: u16, value: u8) {
        let offset = (address - 0xFF00) as usize;
        match offset {
            0x00 => {
                // P1/JOYP: bits 4-5 writable (select lines); bits 6-7 preserved; bits 0-3 read-only (inputs).
                self.io[offset] = (self.io[offset] & 0xCF) | (value & 0x30);
                self.update_joypad_io();
            }
            0x01 => {
                // SB — serial transfer data.
                self.io[offset] = value;
            }
            0x02 => {
                // SC — serial transfer control.
                // Bit 7 = transfer start, bit 0 = clock select (1=internal, 0=external).
                // Only auto-complete transfers driven by the internal clock (bit 0 = 1).
                // External-clock transfers wait for a remote device and never complete
                // in a standalone emulator — triggering the interrupt would make games
                // think a link-cable partner responded.
                let start          = value & 0x80 != 0;
                let internal_clock = value & 0x01 != 0;
                if start && internal_clock {
                    // Complete instantly: output the byte, fill SB with 0xFF (no device),
                    // clear the transfer-start bit, and fire the serial interrupt.
                    let data = self.io[0x01];
                    self.write_serial_byte(data);
                    self.io[0x01] = 0xFF; // received byte from absent partner
                    self.io[offset] = value & 0x7F; // clear bit 7 (transfer done)
                    self.io[0x0F] = 0xE0 | ((self.io[0x0F] | 0x08) & 0x1F);
                } else {
                    // External clock or no start: just store SC as-is (bit 7 stays set,
                    // transfer is pending and will never complete without a real partner).
                    self.io[offset] = value;
                }
            }
            0x04 => {
                // DIV — any write resets the divider to 0.
                self.io[offset] = 0x00;
                self.timer_div_reset = true;
            }
            0x05 => {
                self.io[offset] = value; // TIMA
            }
            0x06 => {
                self.io[offset] = value; // TMA
                self.timer_tma_write = Some(value);
            }
            0x07 => {
                // TAC — only bits 0-2 are defined.
                self.io[offset] = value & 0x07;
                self.timer_tac_write = Some(value & 0x07);
            }
            0x0F => {
                // IF: bits 0-4 are the interrupt flags; bits 5-7 are open bus and always read 1.
                self.io[offset] = 0xE0 | (value & 0x1F);
            }
            0x10..=0x14 | 0x16..=0x19 | 0x1A..=0x1E | 0x20..=0x26 => {
                // Audio registers — queue for APU; mirror into io[] with DMG open-bus masking.
                // Write-only bits always read 1; trigger bit always reads 0; unused bits read 1.
                let read_val = match offset {
                    0x10 => (value & 0x7F) | 0x80, // NR10: bit 7 open
                    0x11 => (value & 0xC0) | 0x3F, // NR11: length write-only → lower 6 read 1
                    0x12 => value,                  // NR12: fully readable
                    0x13 => 0xFF,                   // NR13: freq LSB write-only
                    0x14 => (value & 0x40) | 0xBF, // NR14: only length-enable readable
                    0x16 => (value & 0xC0) | 0x3F, // NR21
                    0x17 => value,                  // NR22
                    0x18 => 0xFF,                   // NR23: write-only
                    0x19 => (value & 0x40) | 0xBF, // NR24
                    0x1A => (value & 0x80) | 0x7F, // NR30: only DAC enable readable
                    0x1B => 0xFF,                   // NR31: write-only
                    0x1C => (value & 0x60) | 0x9F, // NR32: only volume code readable
                    0x1D => 0xFF,                   // NR33: write-only
                    0x1E => (value & 0x40) | 0xBF, // NR34
                    0x20 => 0xFF,                   // NR41: write-only
                    0x21 => value,                  // NR42
                    0x22 => value,                  // NR43
                    0x23 => (value & 0x40) | 0xBF, // NR44
                    0x24 => value,                  // NR50
                    0x25 => value,                  // NR51
                    0x26 => (value & 0x80) | 0x70, // NR52: bit 7 + open-bus bits 6-4
                    _    => 0xFF,
                };
                self.io[offset] = read_val;
                // APU power-off: reset all NR10-NR51 register readback values to
                // their "zero-written" state (open-bus bits remain 1, channel bits 0).
                if offset == 0x26 && value & 0x80 == 0 {
                    self.io[0x10] = 0x80;
                    self.io[0x11] = 0x3F;
                    self.io[0x12] = 0x00;
                    self.io[0x13] = 0xFF;
                    self.io[0x14] = 0xBF;
                    // 0x15 unchanged (open bus, always 0xFF)
                    self.io[0x16] = 0x3F;
                    self.io[0x17] = 0x00;
                    self.io[0x18] = 0xFF;
                    self.io[0x19] = 0xBF;
                    self.io[0x1A] = 0x7F;
                    self.io[0x1B] = 0xFF;
                    self.io[0x1C] = 0x9F;
                    self.io[0x1D] = 0xFF;
                    self.io[0x1E] = 0xBF;
                    // 0x1F unchanged (open bus, always 0xFF)
                    self.io[0x20] = 0xFF;
                    self.io[0x21] = 0x00;
                    self.io[0x22] = 0x00;
                    self.io[0x23] = 0xBF;
                    self.io[0x24] = 0x00;
                    self.io[0x25] = 0x00;
                }
                self.apu_writes.push((address, value));
            }
            0x30..=0x3F => {
                // Wave RAM — queue for APU; also mirror into io[] for read-back.
                self.io[offset] = value;
                self.apu_writes.push((address, value));
            }
            0x40 => self.io[offset] = value, // LCDC
            0x41 => {
                // STAT — bits 3-6 are writable (interrupt-select + LYC enable).
                // Bits 0-2 (mode / LYC flag) are read-only.
                self.io[offset] = (self.io[offset] & 0x07) | (value & 0x78);
            }
            0x42 => self.io[offset] = value, // SCY
            0x43 => self.io[offset] = value, // SCX
            0x44 => {}                        // LY — read-only, writes ignored.
            0x45 => self.io[offset] = value, // LYC
            0x46 => self.start_oam_dma(value),
            0x47 | 0x48 | 0x49 => self.io[offset] = value, // BGP, OBP0, OBP1
            0x4A => self.io[offset] = value, // WY
            0x4B => self.io[offset] = value, // WX
            0x4D => self.io[offset] = value, // KEY1 (CGB speed switch)
            0x4F => self.io[offset] = value & 0x01, // VBK (CGB VRAM bank)
            0x50 => self.io[offset] = value, // Boot ROM disable
            0x51..=0x55 => self.io[offset] = value, // HDMA (CGB)
            0x56 => self.io[offset] = value & 0x3F, // RP (CGB IR port)
            0x68..=0x6B => self.io[offset] = value, // CGB palettes
            0x6C => self.io[offset] = value & 0x01, // OPRI (CGB)
            0x70 => self.io[offset] = value & 0x07, // SVBK (CGB WRAM bank)
            0x76 | 0x77 => {}                        // PCM12/34 — read-only.
            _ => {}                                  // Unmapped — ignore.
        }
    }

    /// Perform an OAM DMA transfer.
    ///
    /// Copies 160 bytes from `source_page << 8` into OAM and starts the 160-cycle
    /// DMA window during which the CPU may only access HRAM.
    /// The source is read through the bus so all normal address-decode rules apply.
    /// We snapshot the relevant source bytes first to avoid borrow-checker issues
    /// and to match hardware behaviour (DMA reads the bus before OAM is written).
    fn start_oam_dma(&mut self, source_page: u8) {
        self.io[0x46] = source_page;
        let source_base = (source_page as u16) << 8;

        // Collect source bytes before mutating OAM.
        let mut buf = [0u8; 160];
        for (i, byte) in buf.iter_mut().enumerate() {
            *byte = self.read(source_base + i as u16);
        }
        self.oam.copy_from_slice(&buf);

        // The transfer takes 160 M-cycles.  Restrict CPU memory access until the
        // counter reaches zero (decremented by advance_dma() each M-cycle).
        self.oam_dma_cycles_remaining = 160;
    }

    /// Advance the OAM DMA cycle counter by `cycles` M-cycles.
    /// Called once per CPU instruction step from the system loop.
    pub fn advance_dma(&mut self, cycles: u32) {
        self.oam_dma_cycles_remaining =
            self.oam_dma_cycles_remaining.saturating_sub(cycles as u8);
    }

    /// Return a reference to the full ROM slice.
    pub fn get_rom(&self) -> &[u8] {
        &self.rom
    }

    /// Recompute the lower 4 bits of io[0x00] from the current button states
    /// and the select lines (bits 4-5). Must be called after any button state
    /// change or after a write to 0xFF00.
    pub fn update_joypad_io(&mut self) {
        let select = self.io[0x00] & 0x30;
        let p14 = select & 0x10 == 0; // bit 4 low → d-pad selected
        let p15 = select & 0x20 == 0; // bit 5 low → action buttons selected

        // GB convention: 0 = pressed on the wire, so invert the pressed bits.
        let dpad_lines   = (!self.joypad_dpad)   & 0x0F;
        let action_lines = (!self.joypad_action) & 0x0F;

        let input_lines = match (p14, p15) {
            (true,  true)  => dpad_lines & action_lines,
            (true,  false) => dpad_lines,
            (false, true)  => action_lines,
            (false, false) => 0x0F, // neither group selected → all high
        };

        self.io[0x00] = (self.io[0x00] & 0xF0) | (input_lines & 0x0F);
    }

    /// Update the LY I/O register from PPU value
    /// This is needed because the PPU updates its internal ly counter,
    /// but the MMU's io array needs to reflect this for CPU reads.
    pub fn update_ly(&mut self, ly: u8) {
        self.io[68] = ly; // 0xFF44 - 0xFF00 = 68
    }

    /// Update STAT bits 0-2 (mode + LYC-match) from the PPU.
    /// Bits 3-6 (interrupt selects written by software) are preserved.
    /// Bit 7 is always 1 per hardware spec.
    pub fn update_ppu_stat(&mut self, stat: u8) {
        self.io[0x41] = 0x80 | (self.io[0x41] & 0x78) | (stat & 0x07);
    }
}

#[cfg(test)]
#[path = "mmu_tests.rs"]
mod tests;
