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
    pub ie: u8, // Interrupt Enable (FFFF)
    pub mbc: MemoryBankController,
    pub cgb_mode: bool, // CGB mode enabled
}

// Global serial log file for output (using Arc<Mutex<File>> for sharing)
static SERIAL_LOG_FILE: Mutex<Option<Arc<Mutex<std::fs::File>>>> = Mutex::new(None);

impl MemoryBus {
    /// Set the serial log file for console output
    pub fn set_serial_log_file(file: Option<Arc<Mutex<std::fs::File>>>) {
        let mut log_file = SERIAL_LOG_FILE.lock().unwrap();
        *log_file = file;
    }

    /// Write a character to the serial log or stdout if not configured
    pub fn write_serial_char(c: char) {
        let log_file = SERIAL_LOG_FILE.lock().unwrap();
        if let Some(ref file) = *log_file {
            let mut f = file.lock().unwrap();
            f.write_all(c.to_string().as_bytes()).ok();
            f.write_all(b"\n").ok();
            f.flush().ok();
        }
    }

    pub fn new(rom_data: Vec<u8>) -> Self {
        let mbc = MemoryBankController::new(rom_data.len());
        // Note: MBC will have an empty ROM, the actual ROM data is stored in MemoryBus

        // Initialize I/O registers to power-on state
        let mut io = [0u8; 128];
        io[0x00] = 0xCF; // P1/JOYP
        io[0x04] = 0x00; // DIV
        io[0x07] = 0xF8; // TAC
        io[0x0F] = 0xE1; // IF
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
        io[0x41] = 0x85; // STAT
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
        }
    }

    /// Read a byte from memory
    pub fn read(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x3FFF => self.rom[address as usize],
            0x4000..=0x7FFF => self.rom[address as usize],
            0x8000..=0x9FFF => self.vram[(address - 0x8000) as usize],
            0xA000..=0xBFFF => self.external_ram[(address - 0xA000) as usize],
            0xC000..=0xCFFF => self.wram[(address - 0xC000) as usize],
            0xD000..=0xDFFF => self.wram[(address - 0xC000) as usize], // WRAM bank 1
            0xE000..=0xFDFF => {
                // Echo RAM - mirror of C000-DDFF
                self.wram[(address - 0xE000) as usize]
            }
            0xFE00..=0xFE9F => self.oam[(address - 0xFE00) as usize],
            0xFEA0..=0xFEFF => self.read_fea0(address),
            0xFF00..=0xFF7F => self.io[(address - 0xFF00) as usize],
            0xFF80..=0xFFFE => self.hram[(address - 0xFF80) as usize],
            0xFFFF => self.ie,
        }
    }

    /// Write a byte to memory
    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x3FFF => {
                // For No MBC, write directly to ROM (allows testing)
                // For MBC, write to MBC for control
                if self.mbc.is_none() {
                    let index = address as usize;
                    if index < self.rom.len() {
                        self.rom[index] = value;
                    }
                } else {
                    self.mbc.write_rom_control(address, value);
                }
            }
            0x4000..=0x7FFF => self.mbc.write_rom_banked(address, value),
            0x8000..=0x9FFF => self.vram[(address - 0x8000) as usize] = value,
            0xA000..=0xBFFF => self.external_ram[(address - 0xA000) as usize] = value,
            0xC000..=0xCFFF => self.wram[(address - 0xC000) as usize] = value,
            0xD000..=0xDFFF => self.wram[(address - 0xC000) as usize] = value, // WRAM bank 1
            0xE000..=0xFDFF => {
                // Echo RAM
                self.wram[(address - 0xE000) as usize] = value;
            }
            0xFE00..=0xFE9F => self.oam[(address - 0xFE00) as usize] = value,
            0xFF00..=0xFF7F => self.write_io(address, value),
            0xFF80..=0xFFFE => self.hram[(address - 0xFF80) as usize] = value,
            0xFFFF => self.ie = value,
            _ => {} // Ignore writes to invalid addresses
        }
    }

    /// Read from FEA0-FEFF range
    fn read_fea0(&self, address: u16) -> u8 {
        // Not usable area
        // On DMG: returns $FF when OAM blocked, $00 otherwise
        // On CGB: returns high nibble of lower address byte twice
        ((address & 0x00F0) >> 4) as u8
    }

    /// Write to I/O registers
    fn write_io(&mut self, address: u16, value: u8) {
        let offset = (address - 0xFF00) as usize;
        match offset {
            0x00 => {
                // P1/JOYP - Joypad
                // Only writes to select which buttons to read
                self.io[offset] = value & 0xF0;
            }
            0x01 => {
                // SB - Serial transfer data
                self.io[offset] = value;
            }
            0x02 => {
                // SC - Serial transfer control
                // Bit 7 is transfer start (auto-cleared when transfer completes)
                self.io[offset] = value & 0x7F; // Clear bit 7 (transfer complete)
                // If transfer was requested (bit 7 was set), output the data
                if value & 0x80 != 0 {
                    let data = self.io[0x01];
                    eprintln!("DEBUG: Serial output character: {:?} (0x{:02X})", data as char, data);
                    // Output character to serial log file if enabled
                    MemoryBus::write_serial_char(data as char);
                }
            }
            0x04 => {
                // DIV - Divider register (write resets to 0)
                self.io[offset] = 0x00;
            }
            0x05 => {
                // TIMA - Timer counter
                self.io[offset] = value;
            }
            0x06 => {
                // TMA - Timer modulo
                self.io[offset] = value;
            }
            0x07 => {
                // TAC - Timer control
                self.io[offset] = value & 0x07;
            }
            0x0F => {
                // IF - Interrupt flag
                self.io[offset] = value & 0x1F;
            }
            0x10..=0x14 | 0x16..=0x19 | 0x1B..=0x1E | 0x20..=0x23 | 0x26 => {
                // Audio registers - some are write-only
                self.io[offset] = value;
            }
            0x15 | 0x1F | 0x24 | 0x25 => {
                // Audio registers that read back written values
                self.io[offset] = value;
            }
            0x40 => {
                // LCDC - LCD control
                self.io[offset] = value;
            }
            0x41 => {
                // STAT - LCD status
                self.io[offset] = value & 0x87; // Only bits 0-2 and 6 are writable
            }
            0x42 => {
                // SCY - Scroll Y
                self.io[offset] = value;
            }
            0x43 => {
                // SCX - Scroll X
                self.io[offset] = value;
            }
            0x44 => {
                // LY - LCD Y coordinate (read-only)
                // Writing has no effect
            }
            0x45 => {
                // LYC - LY compare
                self.io[offset] = value;
            }
            0x46 => {
                // DMA - OAM DMA
                self.start_oam_dma(value);
            }
            0x47 | 0x48 | 0x49 => {
                // Palettes
                self.io[offset] = value;
            }
            0x4A => {
                // WY - Window Y
                self.io[offset] = value;
            }
            0x4B => {
                // WX - Window X
                self.io[offset] = value;
            }
            0x4D => {
                // KEY1 - Speed control (CGB)
                self.io[offset] = value;
            }
            0x4F => {
                // VBK - VRAM bank select (CGB)
                self.io[offset] = value & 0x01;
            }
            0x50 => {
                // BANK - Boot ROM mapping control
                self.io[offset] = value;
            }
            0x51..=0x55 => {
                // HDMA - VRAM DMA (CGB)
                self.io[offset] = value;
            }
            0x56 => {
                // RP - Infrared port (CGB)
                self.io[offset] = value & 0x3F;
            }
            0x68..=0x6B => {
                // CGB palettes
                self.io[offset] = value;
            }
            0x6C => {
                // OPRI - Object priority (CGB)
                self.io[offset] = value & 0x01;
            }
            0x70 => {
                // SVBK - WRAM bank select (CGB)
                self.io[offset] = value & 0x07;
            }
            0x76 | 0x77 => {
                // PCM - Audio outputs (CGB, read-only)
                // Writing has no effect
            }
            _ => {}
        }
    }

    /// Start OAM DMA transfer
    fn start_oam_dma(&mut self, source: u8) {
        let source_base = (source as u16) << 8;
        // DMA transfer copies 160 bytes from source to OAM
        for i in 0..160 {
            let addr = source_base + i as u16;
            let value = self.read(addr);
            self.oam[i] = value;
        }
    }

    /// Get ROM data
    pub fn get_rom(&self) -> &[u8] {
        &self.rom
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_memory_bus_create() {
        let rom = vec![0; 32768]; // 32 KiB ROM
        let bus = MemoryBus::new(rom);

        // Check initial I/O values
        assert_eq!(bus.io[0x00], 0xCF); // P1
        assert_eq!(bus.io[0x40], 0x91); // LCDC
        assert_eq!(bus.io[0x70], 0x00); // Not set yet (CGB only)
    }

    #[test]
    fn test_read_write() {
        let mut rom = vec![0; 32768];
        rom[0x100] = 0x01; // Test ROM
        let mut bus = MemoryBus::new(rom);

        // Test ROM read
        assert_eq!(bus.read(0x100), 0x01);

        // Test RAM write and read
        bus.write(0xC000, 0xAB);
        assert_eq!(bus.read(0xC000), 0xAB);

        // Test OAM
        bus.write(0xFE00, 0x10);
        assert_eq!(bus.read(0xFE00), 0x10);
    }

    #[test]
    fn test_vram_read_write() {
        let mut bus = MemoryBus::new(vec![0; 32768]);

        // Test VRAM write and read
        bus.write(0x8000, 0xAB);
        assert_eq!(bus.read(0x8000), 0xAB);
        assert_eq!(bus.vram[0], 0xAB);

        bus.write(0x9FFF, 0xCD);
        assert_eq!(bus.read(0x9FFF), 0xCD);
        assert_eq!(bus.vram[8191], 0xCD);
    }

    #[test]
    fn test_external_ram_read_write() {
        let mut bus = MemoryBus::new(vec![0; 32768]);

        // Test external RAM write and read
        bus.write(0xA000, 0xEF);
        assert_eq!(bus.read(0xA000), 0xEF);
        assert_eq!(bus.external_ram[0], 0xEF);

        bus.write(0xBFFF, 0x12);
        assert_eq!(bus.read(0xBFFF), 0x12);
        assert_eq!(bus.external_ram[8191], 0x12);
    }

    #[test]
    fn test_wram_read_write() {
        let mut bus = MemoryBus::new(vec![0; 32768]);

        // Test WRAM bank 0
        bus.write(0xC000, 0x34);
        assert_eq!(bus.read(0xC000), 0x34);
        assert_eq!(bus.wram[0], 0x34);

        // Test WRAM bank 1 (D000-DFFF)
        bus.write(0xD000, 0x56);
        assert_eq!(bus.read(0xD000), 0x56);
        assert_eq!(bus.wram[4096], 0x56);
    }

    #[test]
    fn test_echo_ram() {
        let mut bus = MemoryBus::new(vec![0; 32768]);

        // Echo RAM (E000-FDFF) mirrors C000-DDFF
        bus.write(0xC000, 0x78);
        assert_eq!(bus.read(0xE000), 0x78); // Echo RAM should read same

        bus.write(0xE000, 0x9A);
        assert_eq!(bus.read(0xC000), 0x9A); // Writing to echo RAM also writes to original
        assert_eq!(bus.wram[0], 0x9A);
    }

    #[test]
    fn test_hram_read_write() {
        let mut bus = MemoryBus::new(vec![0; 32768]);

        // Test HRAM (FF80-FFFE)
        bus.write(0xFF80, 0xBC);
        assert_eq!(bus.read(0xFF80), 0xBC);
        assert_eq!(bus.hram[0], 0xBC);

        bus.write(0xFFFE, 0xDE);
        assert_eq!(bus.read(0xFFFE), 0xDE);
        assert_eq!(bus.hram[126], 0xDE);
    }

    #[test]
    fn test_ie_register() {
        let mut bus = MemoryBus::new(vec![0; 32768]);

        // Test IE register at FFFF
        bus.write(0xFFFF, 0x3F);
        assert_eq!(bus.read(0xFFFF), 0x3F);
        assert_eq!(bus.ie, 0x3F);

        bus.write(0xFFFF, 0x00);
        assert_eq!(bus.read(0xFFFF), 0x00);
    }

    #[test]
    fn test_io_registers_initial_state() {
        let bus = MemoryBus::new(vec![0; 32768]);

        assert_eq!(bus.io[0x00], 0xCF); // P1/JOYP
        assert_eq!(bus.io[0x04], 0x00); // DIV
        assert_eq!(bus.io[0x07], 0xF8); // TAC
        assert_eq!(bus.io[0x0F], 0xE1); // IF
        assert_eq!(bus.io[0x10], 0x80); // NR10
        assert_eq!(bus.io[0x11], 0xBF); // NR11
        assert_eq!(bus.io[0x12], 0xF3); // NR12
        assert_eq!(bus.io[0x13], 0xFF); // NR13
        assert_eq!(bus.io[0x14], 0xBF); // NR14
        assert_eq!(bus.io[0x16], 0x3F); // NR21
        assert_eq!(bus.io[0x17], 0x00); // NR22
        assert_eq!(bus.io[0x18], 0xFF); // NR23
        assert_eq!(bus.io[0x19], 0xBF); // NR24
        assert_eq!(bus.io[0x1A], 0x7F); // NR30
        assert_eq!(bus.io[0x1B], 0xFF); // NR31
        assert_eq!(bus.io[0x1C], 0x9F); // NR32
        assert_eq!(bus.io[0x1D], 0xFF); // NR33
        assert_eq!(bus.io[0x1E], 0xBF); // NR34
        assert_eq!(bus.io[0x20], 0xFF); // NR41
        assert_eq!(bus.io[0x21], 0x00); // NR42
        assert_eq!(bus.io[0x22], 0x00); // NR43
        assert_eq!(bus.io[0x23], 0xBF); // NR44
        assert_eq!(bus.io[0x24], 0x77); // NR50
        assert_eq!(bus.io[0x25], 0xF3); // NR51
        assert_eq!(bus.io[0x26], 0xF1); // NR52
        assert_eq!(bus.io[0x40], 0x91); // LCDC
        assert_eq!(bus.io[0x41], 0x85); // STAT
        assert_eq!(bus.io[0x44], 0x00); // LY
        assert_eq!(bus.io[0x45], 0x00); // LYC
        assert_eq!(bus.io[0x46], 0xFF); // DMA
        assert_eq!(bus.io[0x47], 0xFC); // BGP
        assert_eq!(bus.io[0x4A], 0x00); // WY
        assert_eq!(bus.io[0x4B], 0x00); // WX
    }

    #[test]
    fn test_io_register_write() {
        let mut bus = MemoryBus::new(vec![0; 32768]);

        // Write to LCDC
        bus.write(0xFF40, 0x91);
        assert_eq!(bus.io[0x40], 0x91);

        // Write to SCY
        bus.write(0xFF42, 0x10);
        assert_eq!(bus.io[0x42], 0x10);

        // Write to SCX
        bus.write(0xFF43, 0x20);
        assert_eq!(bus.io[0x43], 0x20);

        // Write to LYC
        bus.write(0xFF45, 0x30);
        assert_eq!(bus.io[0x45], 0x30);
    }

    #[test]
    fn test_io_register_read_only() {
        let mut bus = MemoryBus::new(vec![0; 32768]);

        // LY is read-only
        let initial_ly = bus.io[0x44];
        bus.write(0xFF44, 0xFF); // Should have no effect
        assert_eq!(bus.read(0xFF44), initial_ly);
    }

    #[test]
    fn test_oam_dma() {
        // Use 64KB ROM to support source address in ROM range
        let mut rom = vec![0; 65536];
        // Set up source data for OAM DMA at address 0x8000 (VRAM, but we can use ROM range)
        // Use 0x1000 which is in ROM range (0x0000-0x7FFF)
        rom[0x1000] = 0x01; // Sprite Y
        rom[0x1001] = 0x10; // Sprite X
        rom[0x1002] = 0x02; // Tile
        rom[0x1003] = 0x03; // Attributes
        for i in 4..160 {
            rom[0x1000 + i] = (i as u8) & 0xFF;
        }

        let mut bus = MemoryBus::new(rom);

        // Start OAM DMA from $10 (upper byte of 0x1000)
        bus.write(0xFF46, 0x10);

        // Check OAM was loaded
        assert_eq!(bus.oam[0], 0x01);
        assert_eq!(bus.oam[1], 0x10);
        assert_eq!(bus.oam[2], 0x02);
        assert_eq!(bus.oam[3], 0x03);
    }

    #[test]
    fn test_oam_read_write() {
        let mut bus = MemoryBus::new(vec![0; 32768]);

        // Write to OAM
        bus.write(0xFE00, 0x50); // Y position
        assert_eq!(bus.read(0xFE00), 0x50);

        bus.write(0xFE9F, 0x9F); // Last OAM entry
        assert_eq!(bus.read(0xFE9F), 0x9F);
    }

    #[test]
    fn test_fea0_feff_range() {
        let bus = MemoryBus::new(vec![0; 32768]);

        // FEA0-FEFF returns high nibble of lower address byte
        assert_eq!(bus.read(0xFEA0), 0x0A);
        assert_eq!(bus.read(0xFEB0), 0x0B);
        assert_eq!(bus.read(0xFEC0), 0x0C);
        assert_eq!(bus.read(0xFED0), 0x0D);
        assert_eq!(bus.read(0xFEE0), 0x0E);
        assert_eq!(bus.read(0xFEF0), 0x0F);
    }

    #[test]
    fn test_memory_map_boundaries() {
        let bus = MemoryBus::new(vec![0; 32768]);

        // ROM Bank 0
        assert_eq!(bus.read(0x0000), 0x00);
        assert_eq!(bus.read(0x3FFF), 0x00);

        // ROM Bank 1 (same for now since no MBC)
        assert_eq!(bus.read(0x4000), 0x00);
        assert_eq!(bus.read(0x7FFF), 0x00);

        // VRAM boundary
        assert_eq!(bus.read(0x8000), 0x00);
        assert_eq!(bus.read(0x9FFF), 0x00);

        // External RAM
        assert_eq!(bus.read(0xA000), 0x00);
        assert_eq!(bus.read(0xBFFF), 0x00);

        // WRAM
        assert_eq!(bus.read(0xC000), 0x00);
        assert_eq!(bus.read(0xCFFF), 0x00);
        assert_eq!(bus.read(0xD000), 0x00);
        assert_eq!(bus.read(0xDFFF), 0x00);

        // Echo RAM
        assert_eq!(bus.read(0xE000), 0x00);
        assert_eq!(bus.read(0xFDFF), 0x00);

        // OAM
        assert_eq!(bus.read(0xFE00), 0x00);
        assert_eq!(bus.read(0xFE9F), 0x00);

        // I/O
        assert_eq!(bus.read(0xFF00), 0xCF);
        assert_eq!(bus.read(0xFF7F), 0x00);

        // HRAM
        assert_eq!(bus.read(0xFF80), 0x00);
        assert_eq!(bus.read(0xFFFE), 0x00);

        // IE
        assert_eq!(bus.read(0xFFFF), 0x00);
    }

    #[test]
    fn test_memory_overflow_ignores() {
        let mut bus = MemoryBus::new(vec![0; 32768]);

        // Writes to invalid addresses should be ignored
        // Using wrapping_sub and wrapping_add to avoid compile-time overflow
        bus.write(0u16.wrapping_sub(1), 0xFF); // Should not panic (wraps to 0xFFFF)
        bus.write(0xFFFFu16.wrapping_add(1), 0xFF); // Should not panic (wraps to 0x0000)
    }

    #[test]
    fn test_serial_io() {
        let mut bus = MemoryBus::new(vec![0; 32768]);

        // SB (serial data)
        bus.write(0xFF01, 0x48); // 'H'
        assert_eq!(bus.read(0xFF01), 0x48);

        // SC (serial control)
        bus.write(0xFF02, 0x80); // Start transfer
        assert_eq!(bus.read(0xFF02) & 0x7F, 0x00); // Bit 7 should be cleared
    }

    #[test]
    fn test_divider_register() {
        let mut bus = MemoryBus::new(vec![0; 32768]);

        // DIV register resets to 0 on write
        bus.write(0xFF04, 0xFF);
        assert_eq!(bus.read(0xFF04), 0x00);
    }

    #[test]
    fn test_timer_registers() {
        let mut bus = MemoryBus::new(vec![0; 32768]);

        // TIMA
        bus.write(0xFF05, 0x42);
        assert_eq!(bus.read(0xFF05), 0x42);

        // TMA
        bus.write(0xFF06, 0x24);
        assert_eq!(bus.read(0xFF06), 0x24);

        // TAC - only lower 3 bits writable
        bus.write(0xFF07, 0xFF);
        assert_eq!(bus.read(0xFF07), 0x07); // Only bits 0-2 preserved
    }

    #[test]
    fn test_interrupt_flag_register() {
        let mut bus = MemoryBus::new(vec![0; 32768]);

        // IF register - only bits 0-4 writable
        bus.write(0xFF0F, 0xFF);
        assert_eq!(bus.read(0xFF0F), 0x1F); // Only bits 0-4 preserved
    }

    #[test]
    fn test_joypad_register() {
        let mut bus = MemoryBus::new(vec![0; 32768]);

        // P1 register - only upper bits writable
        bus.write(0xFF00, 0xF0);
        assert_eq!(bus.read(0xFF00), 0xF0);

        bus.write(0xFF00, 0x0F);
        // Lower bits are always 0 on write (only upper 4 bits are writable)
        assert_eq!(bus.io[0x00] & 0x0F, 0x00);
    }

    #[test]
    fn test_get_rom() {
        let rom_data = vec![0x11, 0x22, 0x33, 0x44];
        let bus = MemoryBus::new(rom_data.clone());

        assert_eq!(bus.get_rom(), rom_data.as_slice());
    }

    #[test]
    fn test_cgb_mode_flag() {
        let bus = MemoryBus::new(vec![0; 32768]);
        assert!(!bus.cgb_mode);

        let mut bus = MemoryBus::new(vec![0; 32768]);
        bus.cgb_mode = true;
        assert!(bus.cgb_mode);
    }
}
