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

impl MemoryBus {
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
            0x0000..=0x3FFF => self.mbc.write_rom_control(address, value),
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
        ((address & 0x0F00) >> 8) as u8
    }

    /// Write to I/O registers
    fn write_io(&mut self, address: u16, value: u8) {
        let offset = (address - 0xFF00) as usize;
        match offset {
            0x00 => {
                // P1/JOYP - Joypad
                // Only writes to select which buttons to read
                self.io[offset] = (self.io[offset] & 0x0F) | (value & 0xF0);
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
                    // Output character to stdout
                    print!("{}", data as char);
                    std::io::stdout().flush().ok();
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
}
