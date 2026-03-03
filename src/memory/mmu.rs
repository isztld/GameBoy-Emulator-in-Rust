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
    pub rom: Vec<u8>,
    pub vram: [u8; 8192], // 8 KiB VRAM
    pub external_ram: [u8; 8192], // 8 KiB external RAM
    pub wram: [u8; 8192], // 8 KiB WRAM (4+4)
    pub hram: [u8; 127], // 127 bytes HRAM (FF80-FFFE)
    pub oam: [u8; 160], // 160 bytes OAM (FE00-FE9F)
    pub io: [u8; 128], // I/O registers (FF00-FF7F)
    /// When true, all reads and writes go directly to `rom` as a flat 64 KiB
    /// array, bypassing all memory-mapped regions and the MBC.  Used by the
    /// CPU test harness, which assumes a fully-writable 64 KiB address space.
    pub flat_mode: bool,
    pub ie: u8, // Interrupt Enable (FFFF)
    pub mbc: MemoryBankController,
    pub cgb_mode: bool, // CGB mode enabled

    // Pending timer register writes — set by write_io and drained by System::step
    // into the Timer struct, which is the authoritative source for timer state.
    pub timer_div_reset: bool,
    pub timer_tima_write: Option<u8>,
    pub timer_tma_write: Option<u8>,
    pub timer_tac_write: Option<u8>,
}

// Global serial log file for output (using Arc<Mutex<File>> for sharing)
static SERIAL_LOG_FILE: Mutex<Option<Arc<Mutex<std::fs::File>>>> = Mutex::new(None);

impl MemoryBus {
    /// Set the serial log file for console output
    pub fn set_serial_log_file(file: Option<Arc<Mutex<std::fs::File>>>) {
        let mut log_file = SERIAL_LOG_FILE.lock().unwrap();
        *log_file = file;
    }

    /// Write a character to the serial log or stdout if not configured.
    /// Does NOT append a newline — callers write raw characters.
    pub fn write_serial_byte(byte: u8) {
        let log_file = SERIAL_LOG_FILE.lock().unwrap();
        if let Some(ref file) = *log_file {
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
        io[0x16] = 0x3F; // NR21
        io[0x17] = 0x00; // NR22
        io[0x18] = 0xFF; // NR23
        io[0x19] = 0xBF; // NR24
        io[0x1A] = 0x7F; // NR30
        io[0x1B] = 0xFF; // NR31
        io[0x1C] = 0x9F; // NR32
        io[0x1D] = 0xFF; // NR33
        io[0x1E] = 0xBF; // NR34
        io[0x20] = 0xFF; // NR41
        io[0x21] = 0x00; // NR42
        io[0x22] = 0x00; // NR43
        io[0x23] = 0xBF; // NR44
        io[0x24] = 0x77; // NR50
        io[0x25] = 0xF3; // NR51
        io[0x26] = 0xF1; // NR52
        io[0x40] = 0x91; // LCDC
        // STAT: bits 3-6 writable by software; start in mode 1 (VBlank) with no
        // interrupt-select bits armed so no spurious STAT IRQ fires immediately.
        io[0x41] = 0x01; // STAT — mode 1, no interrupt selects set
        io[0x44] = 0x00; // LY
        io[0x45] = 0x00; // LYC
        io[0x46] = 0xFF; // DMA
        io[0x47] = 0xFC; // BGP
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
            cgb_mode: false,
            flat_mode: false,
            timer_div_reset: false,
            timer_tima_write: None,
            timer_tma_write: None,
            timer_tac_write: None,
        }
    }

    /// Read a byte from memory.
    pub fn read(&self, address: u16) -> u8 {
        if self.flat_mode {
            return self.rom.get(address as usize).copied().unwrap_or(0xFF);
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
            0x8000..=0x9FFF => self.vram[(address - 0x8000) as usize],
            0xA000..=0xBFFF => self.external_ram[(address - 0xA000) as usize],
            0xC000..=0xCFFF => self.wram[(address - 0xC000) as usize],
            0xD000..=0xDFFF => self.wram[(address - 0xC000) as usize],
            // Echo RAM mirrors C000-DDFF (not all the way to DFFF).
            0xE000..=0xFDFF => self.wram[(address - 0xE000) as usize],
            0xFE00..=0xFE9F => self.oam[(address - 0xFE00) as usize],
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
        match address {
            // ROM area — MBC intercepts control writes; ROM itself is read-only.
            0x0000..=0x7FFF => {
                self.mbc.write_rom_control(address, value);
            }
            0x8000..=0x9FFF => self.vram[(address - 0x8000) as usize] = value,
            0xA000..=0xBFFF => self.external_ram[(address - 0xA000) as usize] = value,
            0xC000..=0xCFFF => self.wram[(address - 0xC000) as usize] = value,
            0xD000..=0xDFFF => self.wram[(address - 0xC000) as usize] = value,
            // Echo RAM mirrors C000-DDFF.
            0xE000..=0xFDFF => self.wram[(address - 0xE000) as usize] = value,
            0xFE00..=0xFE9F => self.oam[(address - 0xFE00) as usize] = value,
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
            }
            0x01 => {
                // SB — serial transfer data.
                self.io[offset] = value;
            }
            0x02 => {
                // SC — serial transfer control.
                // Bit 7 = transfer start; clear it immediately (transfer is "instant" here).
                self.io[offset] = value & 0x7F;
                if value & 0x80 != 0 {
                    let data = self.io[0x01];
                    MemoryBus::write_serial_byte(data as u8);
                    // Fire serial transfer complete interrupt (IF bit 3)
                    self.io[0x0F] = 0xE0 | ((self.io[0x0F] | 0x08) & 0x1F);
                }
            }
            0x04 => {
                // DIV — any write resets the divider to 0.
                self.io[offset] = 0x00;
                self.timer_div_reset = true;
            }
            0x05 => {
                self.io[offset] = value; // TIMA
                self.timer_tima_write = Some(value);
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
                // Audio registers.
                self.io[offset] = value;
            }
            0x30..=0x3F => {
                // Wave RAM.
                self.io[offset] = value;
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
    /// Copies 160 bytes from `source_page << 8` into OAM.
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
    }

    /// Return a reference to the full ROM slice.
    pub fn get_rom(&self) -> &[u8] {
        &self.rom
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
mod tests {
    use super::*;

    fn make_bus(size: usize) -> MemoryBus {
        MemoryBus::new(vec![0u8; size])
    }

    // -----------------------------------------------------------------------
    // Construction
    // -----------------------------------------------------------------------

    #[test]
    fn test_memory_bus_create() {
        let bus = make_bus(32768);
        assert_eq!(bus.read(0xFF00), 0xCF); // P1
        assert_eq!(bus.read(0xFF40), 0x91); // LCDC
        assert_eq!(bus.read(0xFF70), 0x00); // SVBK (CGB only, starts 0)
    }

    #[test]
    fn test_io_registers_initial_state() {
        let bus = make_bus(32768);

        assert_eq!(bus.read(0xFF00), 0xCF); // P1/JOYP
        assert_eq!(bus.read(0xFF04), 0x00); // DIV
        assert_eq!(bus.read(0xFF07), 0xF8); // TAC
        assert_eq!(bus.read(0xFF0F), 0xE0); // IF
        assert_eq!(bus.read(0xFF10), 0x80); // NR10
        assert_eq!(bus.read(0xFF11), 0xBF); // NR11
        assert_eq!(bus.read(0xFF12), 0xF3); // NR12
        assert_eq!(bus.read(0xFF13), 0xFF); // NR13
        assert_eq!(bus.read(0xFF14), 0xBF); // NR14
        assert_eq!(bus.read(0xFF16), 0x3F); // NR21
        assert_eq!(bus.read(0xFF17), 0x00); // NR22
        assert_eq!(bus.read(0xFF18), 0xFF); // NR23
        assert_eq!(bus.read(0xFF19), 0xBF); // NR24
        assert_eq!(bus.read(0xFF1A), 0x7F); // NR30
        assert_eq!(bus.read(0xFF1B), 0xFF); // NR31
        assert_eq!(bus.read(0xFF1C), 0x9F); // NR32
        assert_eq!(bus.read(0xFF1D), 0xFF); // NR33
        assert_eq!(bus.read(0xFF1E), 0xBF); // NR34
        assert_eq!(bus.read(0xFF20), 0xFF); // NR41
        assert_eq!(bus.read(0xFF21), 0x00); // NR42
        assert_eq!(bus.read(0xFF22), 0x00); // NR43
        assert_eq!(bus.read(0xFF23), 0xBF); // NR44
        assert_eq!(bus.read(0xFF24), 0x77); // NR50
        assert_eq!(bus.read(0xFF25), 0xF3); // NR51
        assert_eq!(bus.read(0xFF26), 0xF1); // NR52
        assert_eq!(bus.read(0xFF40), 0x91); // LCDC
        // STAT initial value: mode 1, no interrupt selects armed
        assert_eq!(bus.read(0xFF41), 0x01); // STAT
        assert_eq!(bus.read(0xFF44), 0x00); // LY
        assert_eq!(bus.read(0xFF45), 0x00); // LYC
        assert_eq!(bus.read(0xFF46), 0xFF); // DMA
        assert_eq!(bus.read(0xFF47), 0xFC); // BGP
        assert_eq!(bus.read(0xFF4A), 0x00); // WY
        assert_eq!(bus.read(0xFF4B), 0x00); // WX
    }

    // -----------------------------------------------------------------------
    // ROM
    // -----------------------------------------------------------------------

    #[test]
    fn test_rom_bank0_read() {
        let mut rom = vec![0u8; 32768];
        rom[0x0100] = 0xAB;
        rom[0x3FFF] = 0xCD;
        let bus = MemoryBus::new(rom);
        assert_eq!(bus.read(0x0100), 0xAB);
        assert_eq!(bus.read(0x3FFF), 0xCD);
    }

    #[test]
    fn test_rom_is_read_only() {
        let mut rom = vec![0u8; 32768];
        rom[0x0100] = 0x42;
        let mut bus = MemoryBus::new(rom);
        bus.write(0x0100, 0xFF); // must be ignored
        assert_eq!(bus.read(0x0100), 0x42);
    }

    #[test]
    fn test_get_rom() {
        let rom_data = vec![0x11u8, 0x22, 0x33, 0x44];
        let bus = MemoryBus::new(rom_data.clone());
        assert_eq!(bus.get_rom(), rom_data.as_slice());
    }

    // -----------------------------------------------------------------------
    // VRAM
    // -----------------------------------------------------------------------

    #[test]
    fn test_vram_read_write() {
        let mut bus = make_bus(32768);
        bus.write(0x8000, 0xAB);
        assert_eq!(bus.read(0x8000), 0xAB);
        bus.write(0x9FFF, 0xCD);
        assert_eq!(bus.read(0x9FFF), 0xCD);
    }

    // -----------------------------------------------------------------------
    // External RAM
    // -----------------------------------------------------------------------

    #[test]
    fn test_external_ram_read_write() {
        let mut bus = make_bus(32768);
        bus.write(0xA000, 0xEF);
        assert_eq!(bus.read(0xA000), 0xEF);
        bus.write(0xBFFF, 0x12);
        assert_eq!(bus.read(0xBFFF), 0x12);
    }

    // -----------------------------------------------------------------------
    // WRAM
    // -----------------------------------------------------------------------

    #[test]
    fn test_wram_read_write() {
        let mut bus = make_bus(32768);
        bus.write(0xC000, 0x34);
        assert_eq!(bus.read(0xC000), 0x34);
        bus.write(0xD000, 0x56);
        assert_eq!(bus.read(0xD000), 0x56);
    }

    #[test]
    fn test_echo_ram_mirrors_wram() {
        let mut bus = make_bus(32768);

        // Write to WRAM, read back through echo window
        bus.write(0xC000, 0x78);
        assert_eq!(bus.read(0xE000), 0x78);

        // Write through echo window, read back from WRAM
        bus.write(0xE001, 0x9A);
        assert_eq!(bus.read(0xC001), 0x9A);
    }

    #[test]
    fn test_echo_ram_upper_boundary() {
        // 0xFDFF is the last echo address; maps to WRAM offset 0x1DFF
        let mut bus = make_bus(32768);
        bus.write(0xFDFF, 0x55);
        assert_eq!(bus.read(0xFDFF), 0x55);
        assert_eq!(bus.read(0xDDFF), 0x55);
    }

    // -----------------------------------------------------------------------
    // OAM
    // -----------------------------------------------------------------------

    #[test]
    fn test_oam_read_write() {
        let mut bus = make_bus(32768);
        bus.write(0xFE00, 0x50);
        assert_eq!(bus.read(0xFE00), 0x50);
        bus.write(0xFE9F, 0x9F);
        assert_eq!(bus.read(0xFE9F), 0x9F);
    }

    #[test]
    fn test_oam_dma() {
        // 32 KiB ROM with header byte 0x00 (no MBC); source at 0x1000 (bank 0)
        let mut rom = vec![0u8; 32768];
        for i in 0..160usize {
            rom[0x1000 + i] = i as u8;
        }
        let mut bus = MemoryBus::new(rom);

        bus.write(0xFF46, 0x10); // DMA from 0x1000

        for i in 0..160usize {
            assert_eq!(bus.oam[i], i as u8, "OAM[{i}] mismatch");
        }
    }

    // -----------------------------------------------------------------------
    // Unusable region
    // -----------------------------------------------------------------------

    #[test]
    fn test_fea0_feff_read() {
        let bus = make_bus(32768);
        // Returns high nibble duplicated in both halves (e.g., 0xFEA0 -> 0xAA)
        assert_eq!(bus.read(0xFEA0), 0xAA);
        assert_eq!(bus.read(0xFEB0), 0xBB);
        assert_eq!(bus.read(0xFEC0), 0xCC);
        assert_eq!(bus.read(0xFED0), 0xDD);
        assert_eq!(bus.read(0xFEE0), 0xEE);
        assert_eq!(bus.read(0xFEF0), 0xFF);
    }

    #[test]
    fn test_fea0_feff_write_ignored() {
        let mut bus = make_bus(32768);
        let before = bus.read(0xFEA0);
        bus.write(0xFEA0, 0xFF);
        assert_eq!(bus.read(0xFEA0), before);
    }

    // -----------------------------------------------------------------------
    // HRAM
    // -----------------------------------------------------------------------

    #[test]
    fn test_hram_read_write() {
        let mut bus = make_bus(32768);
        bus.write(0xFF80, 0xBC);
        assert_eq!(bus.read(0xFF80), 0xBC);
        bus.write(0xFFFE, 0xDE);
        assert_eq!(bus.read(0xFFFE), 0xDE);
    }

    // -----------------------------------------------------------------------
    // IE register
    // -----------------------------------------------------------------------

    #[test]
    fn test_ie_register() {
        let mut bus = make_bus(32768);
        bus.write(0xFFFF, 0x1F);
        assert_eq!(bus.read(0xFFFF), 0x1F);
        bus.write(0xFFFF, 0x00);
        assert_eq!(bus.read(0xFFFF), 0x00);
    }

    // -----------------------------------------------------------------------
    // I/O register behaviour
    // -----------------------------------------------------------------------

    #[test]
    fn test_lcdc_scx_scy_lyc_writable() {
        let mut bus = make_bus(32768);
        bus.write(0xFF40, 0x80); assert_eq!(bus.read(0xFF40), 0x80); // LCDC
        bus.write(0xFF42, 0x10); assert_eq!(bus.read(0xFF42), 0x10); // SCY
        bus.write(0xFF43, 0x20); assert_eq!(bus.read(0xFF43), 0x20); // SCX
        bus.write(0xFF45, 0x30); assert_eq!(bus.read(0xFF45), 0x30); // LYC
    }

    #[test]
    fn test_ly_is_read_only() {
        let mut bus = make_bus(32768);
        let before = bus.read(0xFF44);
        bus.write(0xFF44, 0xFF);
        assert_eq!(bus.read(0xFF44), before);
    }

    #[test]
    fn test_stat_writable_bits() {
        let mut bus = make_bus(32768);
        // Bits 3-6 are writable; bits 0-2 (mode/LYC flag) are read-only.
        // Start with mode bits = 0x01 (from init), write 0xFF, expect
        // read-only bits preserved and writable bits updated.
        bus.write(0xFF41, 0xFF);
        let stat = bus.read(0xFF41);
        assert_eq!(stat & 0x07, 0x01, "mode bits must be read-only");
        assert_eq!(stat & 0x78, 0x78, "interrupt-select bits must be writable");
    }

    #[test]
    fn test_divider_resets_on_write() {
        let mut bus = make_bus(32768);
        bus.write(0xFF04, 0xFF);
        assert_eq!(bus.read(0xFF04), 0x00);
    }

    #[test]
    fn test_timer_registers() {
        let mut bus = make_bus(32768);
        bus.write(0xFF05, 0x42); assert_eq!(bus.read(0xFF05), 0x42); // TIMA
        bus.write(0xFF06, 0x24); assert_eq!(bus.read(0xFF06), 0x24); // TMA
        bus.write(0xFF07, 0xFF); assert_eq!(bus.read(0xFF07), 0x07); // TAC: only bits 0-2
    }

    #[test]
    fn test_interrupt_flag_register() {
        let mut bus = make_bus(32768);
        bus.write(0xFF0F, 0xFF);
        // Only bits 0-4 are writable; bits 5-7 are open bus and always read 1.
        assert_eq!(bus.read(0xFF0F), 0xFF); // 0xE0 | 0x1F = 0xFF
    }

    #[test]
    fn test_interrupt_flag_upper_bits_always_one() {
        let mut bus = make_bus(32768);
        bus.write(0xFF0F, 0x00);
        assert_eq!(bus.read(0xFF0F) & 0xE0, 0xE0, "IF bits 5-7 must always read 1");
    }

    #[test]
    fn test_joypad_select_bits_writable() {
        let mut bus = make_bus(32768);

        // Initial value is 0xCF: bits 6-7 set (unused/open bus), bits 4-5 set (no selection),
        // bits 0-3 set (inputs pulled high, no buttons pressed).

        // Select action buttons (clear bit 5, set bit 4)
        bus.write(0xFF00, 0x20);
        let p1 = bus.read(0xFF00);
        assert_eq!(p1 & 0x30, 0x20, "select bits must reflect write");
        assert_eq!(p1 & 0x0F, 0x0F, "input lines must remain high (unpressed)");
        // Bits 6-7 are open bus and should be preserved from initial value.
        assert_eq!(p1 & 0xC0, 0xC0, "bits 6-7 must be preserved");

        // Select direction buttons (clear bit 4, set bit 5)
        bus.write(0xFF00, 0x10);
        let p1 = bus.read(0xFF00);
        assert_eq!(p1 & 0x30, 0x10);
        assert_eq!(p1 & 0x0F, 0x0F);
        assert_eq!(p1 & 0xC0, 0xC0);
    }

    // -----------------------------------------------------------------------
    // Serial
    // -----------------------------------------------------------------------

    #[test]
    fn test_serial_sb_readable() {
        let mut bus = make_bus(32768);
        bus.write(0xFF01, 0x48);
        assert_eq!(bus.read(0xFF01), 0x48);
    }

    #[test]
    fn test_serial_sc_clears_bit7_after_transfer() {
        let mut bus = make_bus(32768);
        bus.write(0xFF01, 0x41);
        bus.write(0xFF02, 0x81); // start transfer, internal clock
        assert_eq!(bus.read(0xFF02) & 0x80, 0x00);
    }

    #[test]
    fn test_serial_output_to_file() {
        use std::fs::OpenOptions;
        use std::sync::{Arc, Mutex};

        // Use a unique filename per test invocation to reduce (but not eliminate)
        // races on the global static. A proper fix requires making the log file
        // an instance field on MemoryBus.
        let temp_path = std::env::temp_dir()
            .join(format!("gb_serial_{}.txt", std::process::id()));

        let file = OpenOptions::new()
            .write(true).create(true).truncate(true)
            .open(&temp_path)
            .expect("open temp file");

        MemoryBus::set_serial_log_file(Some(Arc::new(Mutex::new(file))));

        let mut bus = make_bus(32768);
        for ch in [b'H', b'i'] {
            bus.write(0xFF01, ch);
            bus.write(0xFF02, 0x81);
        }

        // Flush and close by dropping the reference before reading.
        MemoryBus::set_serial_log_file(None);

        let content = std::fs::read_to_string(&temp_path).expect("read temp file");
        let _ = std::fs::remove_file(&temp_path);

        assert_eq!(content, "Hi");
    }

    // -----------------------------------------------------------------------
    // CGB flag
    // -----------------------------------------------------------------------

    #[test]
    fn test_cgb_mode_defaults_false() {
        assert!(!make_bus(32768).cgb_mode);
    }
}
