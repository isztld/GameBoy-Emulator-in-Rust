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
        // MBC1 supports up to 2 MiB, MBC5 for larger
        let mbc_type = if rom_size <= 32768 {
            MbcType::None
        } else if rom_size < 1048576 {
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
        } else if size < 1048576 {
            MbcType::MBC1
        } else {
            MbcType::MBC5
        };
    }

    /// Read from ROM (unbanked region, 0x0000-0x3FFF)
    pub fn read_rom(&self, address: u16) -> u8 {
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
    fn test_mbc_create_with_different_rom_sizes() {
        // 16 KiB ROM - no MBC needed
        let c = MemoryBankController::new(16384);
        assert_eq!(c.config.mbc_type, MbcType::None);

        // 32 KiB ROM - no MBC needed
        let c = MemoryBankController::new(32768);
        assert_eq!(c.config.mbc_type, MbcType::None);

        // 64 KiB ROM - MBC1
        let c = MemoryBankController::new(65536);
        assert_eq!(c.config.mbc_type, MbcType::MBC1);

        // 128 KiB ROM - MBC1
        let c = MemoryBankController::new(131072);
        assert_eq!(c.config.mbc_type, MbcType::MBC1);

        // 256 KiB ROM - MBC1
        let c = MemoryBankController::new(262144);
        assert_eq!(c.config.mbc_type, MbcType::MBC1);

        // 512 KiB ROM - MBC1
        let c = MemoryBankController::new(524288);
        assert_eq!(c.config.mbc_type, MbcType::MBC1);

        // 1 MiB ROM - MBC5
        let c = MemoryBankController::new(1048576);
        assert_eq!(c.config.mbc_type, MbcType::MBC5);

        // 2 MiB ROM - MBC5
        let c = MemoryBankController::new(2097152);
        assert_eq!(c.config.mbc_type, MbcType::MBC5);

        // 4 MiB ROM - MBC5
        let c = MemoryBankController::new(4194304);
        assert_eq!(c.config.mbc_type, MbcType::MBC5);

        // 8 MiB ROM - MBC5
        let c = MemoryBankController::new(8388608);
        assert_eq!(c.config.mbc_type, MbcType::MBC5);
    }

    #[test]
    fn test_mbc_set_rom() {
        let mut controller = MemoryBankController::new(32768);
        assert_eq!(controller.config.mbc_type, MbcType::None);

        // Set larger ROM - should change MBC type
        controller.set_rom(vec![0; 65536]);
        assert_eq!(controller.config.mbc_type, MbcType::MBC1);

        // Set even larger ROM
        controller.set_rom(vec![0; 1048576]);
        assert_eq!(controller.config.mbc_type, MbcType::MBC5);
    }

    #[test]
    fn test_mbc1_ram_enable() {
        let mut controller = MemoryBankController::new(65536);

        // Enable RAM (write 0x0A to 0x0000-0x1FFF)
        controller.mbc1_write(0x0000, 0x0A);
        assert!(controller.ram_enabled);

        // Disable RAM (write other value)
        controller.mbc1_write(0x0000, 0x00);
        assert!(!controller.ram_enabled);
    }

    #[test]
    fn test_mbc1_rom_bank_switching() {
        let mut controller = MemoryBankController::new(65536);

        // Set ROM bank to 1
        controller.mbc1_write(0x2000, 0x01);
        assert_eq!(controller.rom_bank, 1);

        // Set ROM bank to 31 (max for MBC1)
        controller.mbc1_write(0x2000, 0x1F);
        assert_eq!(controller.rom_bank, 31);

        // Bank 0 maps to bank 1
        controller.mbc1_write(0x2000, 0x00);
        assert_eq!(controller.rom_bank, 1);
    }

    #[test]
    fn test_mbc1_ram_bank_switching() {
        let mut controller = MemoryBankController::new(65536);
        controller.rom_mode = false; // Simple mode

        // Set RAM bank to 0
        controller.mbc1_write(0x4000, 0x00);
        assert_eq!(controller.ram_bank, 0);

        // Set RAM bank to 3 (max for MBC1 with RAM)
        controller.mbc1_write(0x4000, 0x03);
        assert_eq!(controller.ram_bank, 3);
    }

    #[test]
    fn test_mbc1_mode_switching() {
        let mut controller = MemoryBankController::new(1048576); // 1 MiB

        // Mode 0: RAM bank mode
        controller.rom_mode = false;
        controller.rom_bank = 0x10;
        controller.ram_bank = 0x00;

        // Switch to mode 1 (ROM banking mode)
        controller.mbc1_write(0x6000, 0x01);
        assert!(controller.rom_mode);

        // In mode 1, upper 2 bits go into ROM bank
        controller.mbc1_write(0x4000, 0x01); // Set upper bit
        assert_eq!(controller.rom_bank, 0x30); // 0x10 | (0x01 << 5)

        // Switch back to mode 0
        controller.mbc1_write(0x6000, 0x00);
        assert!(!controller.rom_mode);
    }

    #[test]
    fn test_mbc1_read_rom() {
        let mut controller = MemoryBankController::new(65536 * 2); // 128 KiB
        controller.rom = vec![0x00; 131072];
        // Fill bank 1 with 0x55
        for i in 0..16384 {
            controller.rom[16384 + i] = 0x55;
        }
        // Fill bank 2 with 0xAA
        for i in 0..16384 {
            controller.rom[32768 + i] = 0xAA;
        }

        controller.rom_bank = 1;
        assert_eq!(controller.read_rom_banked(0x4000), 0x55);
        assert_eq!(controller.read_rom_banked(0x7FFF), 0x55);

        controller.rom_bank = 2;
        assert_eq!(controller.read_rom_banked(0x4000), 0xAA);
        assert_eq!(controller.read_rom_banked(0x7FFF), 0xAA);
    }

    #[test]
    fn test_mbc1_read_rom_out_of_bounds() {
        let controller = MemoryBankController::new(32768); // 32 KiB

        // Should return 0xFF for addresses outside ROM
        // For 32KB ROM, addresses 0x8000+ are out of bounds
        assert_eq!(controller.read_rom(0x8000), 0xFF);
        assert_eq!(controller.read_rom(0xFFFF), 0xFF);
    }

    #[test]
    fn test_mbc1_read_rom_banked_edge_cases() {
        let controller = MemoryBankController::new(65536);

        // Out of range addresses should return 0xFF
        assert_eq!(controller.read_rom_banked(0x3FFF), 0xFF);
        assert_eq!(controller.read_rom_banked(0x8000), 0xFF);
    }

    #[test]
    fn test_mbc1_large_rom_mode() {
        let mut controller = MemoryBankController::new(2097152); // 2 MiB

        // Mode 1 with large ROM
        controller.rom_mode = true;
        controller.rom_bank = 0x10;

        // Set upper ROM bits
        controller.mbc1_write(0x4000, 0x01);
        assert_eq!(controller.rom_bank, 0x30); // 0x10 | (0x01 << 5)
    }

    #[test]
    fn test_mbc2_create() {
        let controller = MemoryBankController::new(65536);
        // MBC2 is for ROMs up to 512 KiB with built-in RAM
        // We'll use MBC1 for now as it's more common
        assert_eq!(controller.config.mbc_type, MbcType::MBC1);
    }

    #[test]
    fn test_mbc2_ram_enable() {
        let mut controller = MemoryBankController::new(65536);
        controller.config.mbc_type = MbcType::MBC2;

        // MBC2 RAM enable is at address 0x0000-0x3FFF
        // When bit 8 is 0, write controls RAM
        controller.mbc2_write(0x0000, 0x0A);
        assert!(controller.mbc2_ram_enabled);

        controller.mbc2_write(0x0000, 0x00);
        assert!(!controller.mbc2_ram_enabled);
    }

    #[test]
    fn test_mbc2_rom_bank() {
        let mut controller = MemoryBankController::new(65536);
        controller.config.mbc_type = MbcType::MBC2;

        // MBC2 ROM bank select at address 0x0100-0x1FF
        controller.mbc2_write(0x0100, 0x05);
        assert_eq!(controller.mbc2_rom_bank, 5);

        controller.mbc2_write(0x0100, 0x0F); // Max for MBC2
        assert_eq!(controller.mbc2_rom_bank, 15);
    }

    #[test]
    fn test_mbc2_bank_zero_maps_to_one() {
        let mut controller = MemoryBankController::new(65536);
        controller.config.mbc_type = MbcType::MBC2;

        // Bank 0 should map to bank 1
        controller.mbc2_write(0x0100, 0x00);
        assert_eq!(controller.mbc2_rom_bank, 1);
    }

    #[test]
    fn test_mbc3_ram_enable() {
        let mut controller = MemoryBankController::new(65536);
        controller.config.mbc_type = MbcType::MBC3;

        // MBC3 RAM enable
        controller.mbc3_write(0x0000, 0x0A);
        assert!(controller.ram_enabled);

        controller.mbc3_write(0x0000, 0x00);
        assert!(!controller.ram_enabled);
    }

    #[test]
    fn test_mbc3_rom_bank() {
        let mut controller = MemoryBankController::new(65536);
        controller.config.mbc_type = MbcType::MBC3;

        // MBC3 ROM bank (7 bits)
        controller.mbc3_write(0x2000, 0x01);
        assert_eq!(controller.rom_bank, 1);

        controller.mbc3_write(0x2000, 0x7F); // Max for MBC3
        assert_eq!(controller.rom_bank, 127);
    }

    #[test]
    fn test_mbc3_ram_bank() {
        let mut controller = MemoryBankController::new(65536);
        controller.config.mbc_type = MbcType::MBC3;

        // MBC3 RAM bank select
        controller.mbc3_write(0x4000, 0x00);
        assert_eq!(controller.ram_bank, 0);

        controller.mbc3_write(0x4000, 0x07);
        assert_eq!(controller.ram_bank, 7);
    }

    #[test]
    fn test_mbc5_rom_bank() {
        let mut controller = MemoryBankController::new(65536);
        controller.config.mbc_type = MbcType::MBC5;

        // MBC5 ROM bank low
        controller.mbc5_write(0x2000, 0x34);
        assert_eq!(controller.mbc5_rom_bank_low, 0x34);

        // MBC5 ROM bank high
        controller.mbc5_write(0x3000, 0x01);
        assert_eq!(controller.mbc5_rom_bank_high, 0x01);
    }

    #[test]
    fn test_mbc5_ram_bank() {
        let mut controller = MemoryBankController::new(65536);
        controller.config.mbc_type = MbcType::MBC5;

        // MBC5 RAM bank
        controller.mbc5_write(0x4000, 0x00);
        assert_eq!(controller.ram_bank, 0);

        controller.mbc5_write(0x4000, 0x0F);
        assert_eq!(controller.ram_bank, 15);
    }

    #[test]
    fn test_mbc5_ram_enable() {
        let mut controller = MemoryBankController::new(65536);
        controller.config.mbc_type = MbcType::MBC5;

        // MBC5 RAM enable
        controller.mbc5_write(0x0000, 0x0A);
        assert!(controller.ram_enabled);

        controller.mbc5_write(0x0000, 0x00);
        assert!(!controller.ram_enabled);
    }

    #[test]
    fn test_mbc5_large_rom_banks() {
        let mut controller = MemoryBankController::new(65536);
        controller.config.mbc_type = MbcType::MBC5;

        // Test large ROM bank numbers
        controller.mbc5_write(0x2000, 0xFF); // Low byte
        controller.mbc5_write(0x3000, 0x01); // High bit
        // Full bank = (1 << 8) | 0xFF = 511
    }

    #[test]
    fn test_get_rom_bank() {
        let mut controller = MemoryBankController::new(65536);
        controller.rom_bank = 0x10;
        assert_eq!(controller.get_rom_bank(), 0x10);
    }

    #[test]
    fn test_get_ram_bank() {
        let mut controller = MemoryBankController::new(65536);
        controller.ram_bank = 0x03;
        assert_eq!(controller.get_ram_bank(), 0x03);
    }

    #[test]
    fn test_is_ram_enabled() {
        let mut controller = MemoryBankController::new(65536);
        assert!(!controller.is_ram_enabled());
        controller.ram_enabled = true;
        assert!(controller.is_ram_enabled());
    }

    #[test]
    fn test_is_rom_mode() {
        let mut controller = MemoryBankController::new(65536);
        controller.rom_mode = false;
        assert!(!controller.is_rom_mode());
        controller.rom_mode = true;
        assert!(controller.is_rom_mode());
    }
}
