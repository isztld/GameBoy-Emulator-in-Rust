/// Memory Bank Controller (MBC) implementations
///
/// The GameBoy uses MBC chips to expand the addressable memory beyond
/// the 64 KiB address space. Different MBC types support different
/// ROM and RAM sizes.

/// MBC Type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MbcType {
    None,        // No MBC, direct ROM mapping
    MBC1,        // MBC1
    MBC2,        // MBC2
    MBC3,        // MBC3 (with RTC)
    MBC5,        // MBC5
    MBC6,        // MBC6
    MBC7,        // MBC7
    HuC1,        // HuC1
    HuC3,        // HuC3
}

/// MBC Configuration
#[derive(Debug, Clone)]
pub struct MbcConfig {
    pub mbc_type: MbcType,
    pub rom_size: usize,
    pub ram_size: usize,
}

/// Memory Bank Controller
#[derive(Debug)]
pub struct MemoryBankController {
    config: MbcConfig,
    rom: Vec<u8>,

    // MBC1/MBC3/MBC5 registers
    rom_bank: u8,
    ram_bank: u8,
    rom_mode: bool, // For MBC1: 0=simple, 1=advanced

    // MBC2 specific
    mbc2_ram_enabled: bool,
    mbc2_rom_bank: u8,

    // MBC5 specific
    mbc5_rom_bank_low: u8,
    mbc5_rom_bank_high: u8,

    // RAM enable
    ram_enabled: bool,
}

impl MemoryBankController {
    /// Create a new MBC based on ROM size
    pub fn new(rom_size: usize) -> Self {
        // Determine MBC type based on ROM size
        // For now, use MBC1 for ROMs > 32 KiB
        let mbc_type = if rom_size <= 32768 {
            MbcType::None
        } else if rom_size <= 1048576 {
            MbcType::MBC1
        } else {
            MbcType::MBC5
        };

        MemoryBankController {
            config: MbcConfig {
                mbc_type,
                rom_size,
                ram_size: 0,
            },
            rom: vec![0; rom_size],
            rom_bank: 1, // Default to bank 1 for MBC
            ram_bank: 0,
            rom_mode: false,
            mbc2_ram_enabled: false,
            mbc2_rom_bank: 1,
            mbc5_rom_bank_low: 0,
            mbc5_rom_bank_high: 0,
            ram_enabled: false,
        }
    }

    /// Set the ROM data
    pub fn set_rom(&mut self, data: Vec<u8>) {
        let size = data.len();
        self.rom = data;
        self.config.rom_size = size;

        // Reconfigure MBC type based on actual ROM size
        self.config.mbc_type = if size <= 32768 {
            MbcType::None
        } else if size <= 1048576 {
            MbcType::MBC1
        } else {
            MbcType::MBC5
        };
    }

    /// Read from ROM (unbanked region, 0x0000-0x3FFF)
    pub fn read_rom(&self, address: u16) -> u8 {
        if address >= 0x4000 {
            return 0xFF; // Out of range
        }

        let index = address as usize;
        if index < self.rom.len() {
            self.rom[index]
        } else {
            0xFF
        }
    }

    /// Read from ROM (banked region, 0x4000-0x7FFF)
    pub fn read_rom_banked(&self, address: u16) -> u8 {
        if address < 0x4000 || address >= 0x8000 {
            return 0xFF; // Out of range
        }

        let offset = (address - 0x4000) as usize;
        let bank_offset = (self.rom_bank as usize & 0x7F) * 16384;
        let index = bank_offset + offset;

        if index < self.rom.len() {
            self.rom[index]
        } else {
            0xFF
        }
    }

    /// Write to ROM control region (0x0000-0x7FFF)
    pub fn write_rom_control(&mut self, address: u16, value: u8) {
        if self.config.mbc_type == MbcType::None {
            return;
        }

        match self.config.mbc_type {
            MbcType::MBC1 => self.mbc1_write(address, value),
            MbcType::MBC2 => self.mbc2_write(address, value),
            MbcType::MBC3 => self.mbc3_write(address, value),
            MbcType::MBC5 => self.mbc5_write(address, value),
            _ => {}
        }
    }

    /// Write to banked ROM region (0x4000-0x7FFF)
    pub fn write_rom_banked(&mut self, _address: u16, _value: u8) {
        // Reading from this region is always ROM
    }

    // MBC1 implementation
    fn mbc1_write(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x1FFF => {
                // RAM enable/write protect
                if (value & 0x0F) == 0x0A {
                    self.ram_enabled = true;
                } else {
                    self.ram_enabled = false;
                }
            }
            0x2000..=0x3FFF => {
                // ROM bank number (5 bits)
                self.rom_bank = (value & 0x1F) as u8;
                if self.rom_bank == 0 {
                    self.rom_bank = 1; // Bank 0 maps to bank 1
                }
            }
            0x4000..=0x5FFF => {
                // RAM bank number or upper ROM bits (2 bits)
                if self.config.rom_size > 524288 {
                    // Large ROM: upper 2 bits of ROM bank
                    self.rom_bank = (self.rom_bank & 0x1F) | ((value as u8 & 0x03) << 5);
                } else {
                    // RAM bank number
                    self.ram_bank = value as u8 & 0x03;
                }
            }
            0x6000..=0x7FFF => {
                // Banking mode select
                self.rom_mode = (value & 0x01) != 0;
            }
            _ => {}
        }
    }

    // MBC2 implementation
    fn mbc2_write(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x3FFF => {
                // RAM enable (bit 8 of address selects function)
                if (address & 0x0100) == 0 {
                    // RAM control
                    if (value & 0x0F) == 0x0A {
                        self.mbc2_ram_enabled = true;
                    } else {
                        self.mbc2_ram_enabled = false;
                    }
                } else {
                    // ROM bank select
                    self.mbc2_rom_bank = (value & 0x0F) as u8;
                    if self.mbc2_rom_bank == 0 {
                        self.mbc2_rom_bank = 1;
                    }
                }
            }
            _ => {}
        }
    }

    // MBC3 implementation
    fn mbc3_write(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x1FFF => {
                // RAM/RTC enable
                if (value & 0x0F) == 0x0A {
                    self.ram_enabled = true;
                } else {
                    self.ram_enabled = false;
                }
            }
            0x2000..=0x3FFF => {
                // ROM bank number (7 bits)
                self.rom_bank = value as u8 & 0x7F;
                if self.rom_bank == 0 {
                    self.rom_bank = 1;
                }
            }
            0x4000..=0x5FFF => {
                // RAM bank number or RTC register select
                if value <= 7 {
                    self.ram_bank = value as u8;
                }
                // RTC registers 0x8-0xC not implemented yet
            }
            0x6000..=0x7FFF => {
                // Latch clock data (RTC)
                // Not implemented yet
            }
            _ => {}
        }
    }

    // MBC5 implementation
    fn mbc5_write(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x1FFF => {
                // RAM enable
                if (value & 0x0F) == 0x0A {
                    self.ram_enabled = true;
                } else {
                    self.ram_enabled = false;
                }
            }
            0x2000..=0x2FFF => {
                // 8 LSB of ROM bank number
                self.mbc5_rom_bank_low = value as u8;
            }
            0x3000..=0x3FFF => {
                // 9th bit of ROM bank number
                self.mbc5_rom_bank_high = value as u8 & 0x01;
            }
            0x4000..=0x5FFF => {
                // RAM bank number
                self.ram_bank = value as u8 & 0x0F;
            }
            _ => {}
        }
    }

    /// Get current ROM bank
    pub fn get_rom_bank(&self) -> u8 {
        self.rom_bank
    }

    /// Get current RAM bank
    pub fn get_ram_bank(&self) -> u8 {
        self.ram_bank
    }

    /// Check if RAM is enabled
    pub fn is_ram_enabled(&self) -> bool {
        self.ram_enabled
    }

    /// Check if ROM mode is enabled (MBC1)
    pub fn is_rom_mode(&self) -> bool {
        self.rom_mode
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mbc_create() {
        let controller = MemoryBankController::new(32768);
        assert_eq!(controller.config.mbc_type, MbcType::None);

        let controller = MemoryBankController::new(65536);
        assert_eq!(controller.config.mbc_type, MbcType::MBC1);

        let controller = MemoryBankController::new(1048576);
        assert_eq!(controller.config.mbc_type, MbcType::MBC5);
    }

    #[test]
    fn test_mbc1_bank_switching() {
        let mut controller = MemoryBankController::new(65536);

        // Write to ROM bank register
        controller.mbc1_write(0x2000, 0x05);
        assert_eq!(controller.rom_bank, 5);

        // Bank 0 maps to bank 1
        controller.mbc1_write(0x2000, 0x00);
        assert_eq!(controller.rom_bank, 1);
    }

    #[test]
    fn test_mbc1_mode_switching() {
        let mut controller = MemoryBankController::new(1048576); // 1 MiB

        // Mode 0
        controller.rom_mode = false;
        controller.rom_bank = 0x10;

        // Mode 1 enables upper bits
        controller.mbc1_write(0x6000, 0x01);
        assert!(controller.rom_mode);

        // Now upper bits affect ROM bank
        controller.mbc1_write(0x4000, 0x01); // Set upper bit
        assert_eq!(controller.rom_bank, 0x30); // 0x10 | (0x01 << 5)
    }
}
