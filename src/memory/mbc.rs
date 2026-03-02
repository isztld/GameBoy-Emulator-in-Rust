/// Memory Bank Controller (MBC) implementations
///
/// The GameBoy uses MBC chips to expand the addressable memory beyond
/// the 64 KiB address space. Different MBC types support different
/// ROM and RAM sizes.

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

impl MbcType {
    /// Decode MBC type from cartridge header byte 0x0147.
    pub fn from_header_byte(byte: u8) -> Self {
        match byte {
            0x00 | 0x08 | 0x09 => MbcType::None,
            0x01..=0x03 => MbcType::MBC1,
            0x05 | 0x06 => MbcType::MBC2,
            0x0F..=0x13 => MbcType::MBC3,
            0x19..=0x1E => MbcType::MBC5,
            0x20 => MbcType::MBC6,
            0x22 => MbcType::MBC7,
            0xFE => MbcType::HuC3,
            0xFF => MbcType::HuC1,
            _ => MbcType::None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MbcConfig {
    pub mbc_type: MbcType,
    pub rom_size: usize,
    pub ram_size: usize,
}

#[derive(Debug)]
pub struct MemoryBankController {
    config: MbcConfig,

    // MBC1/MBC3 — 7-bit (MBC3) or 5+2-bit (MBC1) rom bank
    rom_bank: u16,
    ram_bank: u8,
    rom_mode: bool, // MBC1: false=simple (default), true=advanced
    ram_enabled: bool,

    // MBC5 — 9-bit bank stored as low + high
    mbc5_rom_bank_low: u8,
    mbc5_rom_bank_high: u8, // only bit 0 used

    // MBC2 — 4-bit bank, separate enable flag
    mbc2_rom_bank: u8,
    mbc2_ram_enabled: bool,
}

impl MemoryBankController {
    /// Create a new MBC.  `rom_data` is the full ROM image; `header_byte` is
    /// cartridge header byte 0x0147 which identifies the mapper type.
    pub fn new(rom_data_len: usize, header_byte: u8) -> Self {
        let mbc_type = MbcType::from_header_byte(header_byte);
        MemoryBankController {
            config: MbcConfig {
                mbc_type,
                rom_size: rom_data_len,
                ram_size: 0,
            },
            rom_bank: 1,
            ram_bank: 0,
            rom_mode: false,
            ram_enabled: false,
            mbc5_rom_bank_low: 1,
            mbc5_rom_bank_high: 0,
            mbc2_rom_bank: 1,
            mbc2_ram_enabled: false,
        }
    }

    // -----------------------------------------------------------------------
    // Bank-offset helpers (used by MemoryBus to index into its ROM slice)
    // -----------------------------------------------------------------------

    /// Byte offset for the unbanked window (0x0000–0x3FFF).
    ///
    /// For MBC1 in advanced (ROM) mode the upper 2 register bits also apply to
    /// this window, giving access to banks 0x00, 0x20, 0x40, 0x60.
    pub fn rom_bank0_offset(&self) -> usize {
        match self.config.mbc_type {
            MbcType::MBC1 if self.rom_mode => {
                // Upper 2 bits select which 1 MiB block; bank 0 of that block.
                let upper = (self.rom_bank & 0x60) as usize;
                upper * 0x4000
            }
            _ => 0,
        }
    }

    /// Byte offset for the banked window (0x4000–0x7FFF).
    pub fn rom_bank_offset(&self) -> usize {
        let bank = self.active_rom_bank() as usize;
        bank * 0x4000
    }

    /// Assemble the active ROM bank number from the individual MBC registers.
    fn active_rom_bank(&self) -> u16 {
        match self.config.mbc_type {
            MbcType::None => 1,
            MbcType::MBC1 => {
                // 5-bit lower number | 2-bit upper number
                // Banks 0x00, 0x20, 0x40, 0x60 redirect to 0x01, 0x21, 0x41, 0x61.
                let lower = self.rom_bank & 0x1F;
                let upper = self.rom_bank & 0x60;
                let bank = upper | lower;
                if bank & 0x1F == 0 { bank | 1 } else { bank }
            }
            MbcType::MBC2 => self.mbc2_rom_bank as u16,
            MbcType::MBC3 => self.rom_bank & 0x7F,
            MbcType::MBC5 => {
                (self.mbc5_rom_bank_low as u16) | ((self.mbc5_rom_bank_high as u16 & 0x01) << 8)
            }
            _ => 1,
        }
    }

    // -----------------------------------------------------------------------
    // Bus-facing write handler
    // -----------------------------------------------------------------------

    /// Called for any write in 0x0000–0x7FFF.  ROM is read-only; these writes
    /// are mapper control registers only.
    pub fn write_rom_control(&mut self, address: u16, value: u8) {
        match self.config.mbc_type {
            MbcType::None => {} // No mapper — writes are silently ignored.
            MbcType::MBC1 => self.mbc1_write(address, value),
            MbcType::MBC2 => self.mbc2_write(address, value),
            MbcType::MBC3 => self.mbc3_write(address, value),
            MbcType::MBC5 => self.mbc5_write(address, value),
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // Per-mapper write logic
    // -----------------------------------------------------------------------

    fn mbc1_write(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x1FFF => {
                self.ram_enabled = (value & 0x0F) == 0x0A;
            }
            0x2000..=0x3FFF => {
                // 5-bit lower ROM bank.  Zero remaps to 1 (handled in active_rom_bank).
                let lower = value & 0x1F;
                self.rom_bank = (self.rom_bank & 0x60) | lower as u16;
            }
            0x4000..=0x5FFF => {
                // 2-bit secondary register: upper ROM bits or RAM bank.
                let bits = (value as u16 & 0x03) << 5;
                self.rom_bank = (self.rom_bank & 0x1F) | bits;
                self.ram_bank = value & 0x03;
            }
            0x6000..=0x7FFF => {
                self.rom_mode = (value & 0x01) != 0;
            }
            _ => {}
        }
    }

    fn mbc2_write(&mut self, address: u16, value: u8) {
        // Only the 0x0000–0x3FFF range is used; bit 8 of the address
        // distinguishes RAM-enable (0) from ROM-bank-select (1).
        if address > 0x3FFF {
            return;
        }
        if address & 0x0100 == 0 {
            self.mbc2_ram_enabled = (value & 0x0F) == 0x0A;
        } else {
            self.mbc2_rom_bank = value & 0x0F;
            if self.mbc2_rom_bank == 0 {
                self.mbc2_rom_bank = 1;
            }
        }
    }

    fn mbc3_write(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x1FFF => {
                self.ram_enabled = (value & 0x0F) == 0x0A;
            }
            0x2000..=0x3FFF => {
                self.rom_bank = (value & 0x7F) as u16;
                if self.rom_bank == 0 {
                    self.rom_bank = 1;
                }
            }
            0x4000..=0x5FFF => {
                if value <= 0x07 {
                    self.ram_bank = value & 0x07;
                }
                // RTC register select (0x08–0x0C) — not yet implemented.
            }
            0x6000..=0x7FFF => {
                // Latch clock data — not yet implemented.
            }
            _ => {}
        }
    }

    fn mbc5_write(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x1FFF => {
                self.ram_enabled = (value & 0x0F) == 0x0A;
            }
            0x2000..=0x2FFF => {
                // Low 8 bits of ROM bank (MBC5 allows bank 0 directly).
                self.mbc5_rom_bank_low = value;
            }
            0x3000..=0x3FFF => {
                // Bit 0 = 9th bit of ROM bank.
                self.mbc5_rom_bank_high = value & 0x01;
            }
            0x4000..=0x5FFF => {
                self.ram_bank = value & 0x0F;
            }
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // Accessors
    // -----------------------------------------------------------------------

    pub fn get_rom_bank(&self) -> u16 {
        self.active_rom_bank()
    }

    pub fn get_ram_bank(&self) -> u8 {
        self.ram_bank
    }

    pub fn is_ram_enabled(&self) -> bool {
        match self.config.mbc_type {
            MbcType::MBC2 => self.mbc2_ram_enabled,
            _ => self.ram_enabled,
        }
    }

    pub fn is_rom_mode(&self) -> bool {
        self.rom_mode
    }

    pub fn is_none(&self) -> bool {
        self.config.mbc_type == MbcType::None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ------------------------------------------------------------------
    // Construction
    // ------------------------------------------------------------------

    #[test]
    fn test_mbc_type_from_header_byte() {
        assert_eq!(MbcType::from_header_byte(0x00), MbcType::None);
        assert_eq!(MbcType::from_header_byte(0x01), MbcType::MBC1);
        assert_eq!(MbcType::from_header_byte(0x02), MbcType::MBC1);
        assert_eq!(MbcType::from_header_byte(0x03), MbcType::MBC1);
        assert_eq!(MbcType::from_header_byte(0x05), MbcType::MBC2);
        assert_eq!(MbcType::from_header_byte(0x06), MbcType::MBC2);
        assert_eq!(MbcType::from_header_byte(0x0F), MbcType::MBC3);
        assert_eq!(MbcType::from_header_byte(0x11), MbcType::MBC3);
        assert_eq!(MbcType::from_header_byte(0x13), MbcType::MBC3);
        assert_eq!(MbcType::from_header_byte(0x19), MbcType::MBC5);
        assert_eq!(MbcType::from_header_byte(0x1E), MbcType::MBC5);
        assert_eq!(MbcType::from_header_byte(0x20), MbcType::MBC6);
        assert_eq!(MbcType::from_header_byte(0x22), MbcType::MBC7);
        assert_eq!(MbcType::from_header_byte(0xFE), MbcType::HuC3);
        assert_eq!(MbcType::from_header_byte(0xFF), MbcType::HuC1);
        assert_eq!(MbcType::from_header_byte(0xAB), MbcType::None); // unknown
    }

    #[test]
    fn test_new_records_size_and_type() {
        let c = MemoryBankController::new(32768, 0x00);
        assert_eq!(c.config.mbc_type, MbcType::None);
        assert_eq!(c.config.rom_size, 32768);

        let c = MemoryBankController::new(131072, 0x01);
        assert_eq!(c.config.mbc_type, MbcType::MBC1);
        assert_eq!(c.config.rom_size, 131072);

        let c = MemoryBankController::new(1048576, 0x19);
        assert_eq!(c.config.mbc_type, MbcType::MBC5);
        assert_eq!(c.config.rom_size, 1048576);
    }

    // ------------------------------------------------------------------
    // No MBC
    // ------------------------------------------------------------------

    #[test]
    fn test_no_mbc_bank0_offset_is_always_zero() {
        let c = MemoryBankController::new(32768, 0x00);
        assert_eq!(c.rom_bank0_offset(), 0);
        assert_eq!(c.rom_bank_offset(), 0x4000); // bank 1
    }

    #[test]
    fn test_no_mbc_ignores_writes() {
        let mut c = MemoryBankController::new(32768, 0x00);
        let before = c.rom_bank_offset();
        c.write_rom_control(0x2000, 0x05);
        assert_eq!(c.rom_bank_offset(), before);
    }

    // ------------------------------------------------------------------
    // MBC1
    // ------------------------------------------------------------------

    #[test]
    fn test_mbc1_ram_enable() {
        let mut c = MemoryBankController::new(131072, 0x01);
        assert!(!c.is_ram_enabled());

        c.write_rom_control(0x0000, 0x0A);
        assert!(c.is_ram_enabled());

        c.write_rom_control(0x1FFF, 0x00);
        assert!(!c.is_ram_enabled());
    }

    #[test]
    fn test_mbc1_rom_bank_switching() {
        let mut c = MemoryBankController::new(131072, 0x01);

        // Bank 1 is default
        assert_eq!(c.rom_bank_offset(), 1 * 0x4000);

        // Switch to bank 3
        c.write_rom_control(0x2000, 0x03);
        assert_eq!(c.rom_bank_offset(), 3 * 0x4000);

        // Switch to bank 31
        c.write_rom_control(0x2000, 0x1F);
        assert_eq!(c.rom_bank_offset(), 31 * 0x4000);
    }

    #[test]
    fn test_mbc1_bank_zero_remaps_to_one() {
        let mut c = MemoryBankController::new(131072, 0x01);
        c.write_rom_control(0x2000, 0x00);
        assert_eq!(c.rom_bank_offset(), 1 * 0x4000);
    }

    #[test]
    fn test_mbc1_bank_0x20_remaps_to_0x21() {
        let mut c = MemoryBankController::new(1048576, 0x01); // needs large ROM for upper bits
        // Set upper 2 bits to 0x01 (=> 0x20 base), lower 5 bits to 0
        c.write_rom_control(0x4000, 0x01); // upper bits
        c.write_rom_control(0x2000, 0x00); // lower bits = 0 → remaps to 1
        assert_eq!(c.rom_bank_offset(), 0x21 * 0x4000);
    }

    #[test]
    fn test_mbc1_ram_bank_switching() {
        let mut c = MemoryBankController::new(131072, 0x01);

        c.write_rom_control(0x4000, 0x02);
        assert_eq!(c.get_ram_bank(), 2);

        c.write_rom_control(0x4000, 0x03);
        assert_eq!(c.get_ram_bank(), 3);
    }

    #[test]
    fn test_mbc1_mode_switching() {
        let mut c = MemoryBankController::new(131072, 0x01);
        assert!(!c.is_rom_mode());

        c.write_rom_control(0x6000, 0x01);
        assert!(c.is_rom_mode());

        c.write_rom_control(0x6000, 0x00);
        assert!(!c.is_rom_mode());
    }

    #[test]
    fn test_mbc1_advanced_mode_bank0_window() {
        // In advanced (ROM) mode, upper 2 bits apply to the bank-0 window too.
        let mut c = MemoryBankController::new(1048576, 0x01);
        c.write_rom_control(0x6000, 0x01); // advanced mode
        c.write_rom_control(0x4000, 0x01); // upper bits = 01 → bank-0 window starts at 0x20*0x4000

        assert_eq!(c.rom_bank0_offset(), 0x20 * 0x4000);
    }

    #[test]
    fn test_mbc1_simple_mode_bank0_window_always_zero() {
        let mut c = MemoryBankController::new(1048576, 0x01);
        c.write_rom_control(0x6000, 0x00); // simple mode
        c.write_rom_control(0x4000, 0x03); // upper bits set, but ignored for window 0

        assert_eq!(c.rom_bank0_offset(), 0);
    }

    // ------------------------------------------------------------------
    // MBC2
    // ------------------------------------------------------------------

    #[test]
    fn test_mbc2_ram_enable() {
        let mut c = MemoryBankController::new(131072, 0x05);

        // Bit 8 of address = 0 → RAM control
        c.write_rom_control(0x0000, 0x0A);
        assert!(c.is_ram_enabled());

        c.write_rom_control(0x0000, 0x00);
        assert!(!c.is_ram_enabled());
    }

    #[test]
    fn test_mbc2_rom_bank_select() {
        let mut c = MemoryBankController::new(131072, 0x05);

        // Bit 8 of address = 1 → ROM bank select
        c.write_rom_control(0x0100, 0x05);
        assert_eq!(c.rom_bank_offset(), 5 * 0x4000);

        c.write_rom_control(0x0100, 0x0F);
        assert_eq!(c.rom_bank_offset(), 15 * 0x4000);
    }

    #[test]
    fn test_mbc2_bank_zero_remaps_to_one() {
        let mut c = MemoryBankController::new(131072, 0x05);
        c.write_rom_control(0x0100, 0x00);
        assert_eq!(c.rom_bank_offset(), 1 * 0x4000);
    }

    // ------------------------------------------------------------------
    // MBC3
    // ------------------------------------------------------------------

    #[test]
    fn test_mbc3_ram_enable() {
        let mut c = MemoryBankController::new(131072, 0x11);

        c.write_rom_control(0x0000, 0x0A);
        assert!(c.is_ram_enabled());

        c.write_rom_control(0x0000, 0x00);
        assert!(!c.is_ram_enabled());
    }

    #[test]
    fn test_mbc3_rom_bank_switching() {
        let mut c = MemoryBankController::new(131072, 0x11);

        c.write_rom_control(0x2000, 0x01);
        assert_eq!(c.rom_bank_offset(), 1 * 0x4000);

        c.write_rom_control(0x2000, 0x7F);
        assert_eq!(c.rom_bank_offset(), 0x7F * 0x4000);
    }

    #[test]
    fn test_mbc3_bank_zero_remaps_to_one() {
        let mut c = MemoryBankController::new(131072, 0x11);
        c.write_rom_control(0x2000, 0x00);
        assert_eq!(c.rom_bank_offset(), 1 * 0x4000);
    }

    #[test]
    fn test_mbc3_ram_bank_switching() {
        let mut c = MemoryBankController::new(131072, 0x11);

        c.write_rom_control(0x4000, 0x00);
        assert_eq!(c.get_ram_bank(), 0);

        c.write_rom_control(0x4000, 0x07);
        assert_eq!(c.get_ram_bank(), 7);
    }

    // ------------------------------------------------------------------
    // MBC5
    // ------------------------------------------------------------------

    #[test]
    fn test_mbc5_ram_enable() {
        let mut c = MemoryBankController::new(1048576, 0x19);

        c.write_rom_control(0x0000, 0x0A);
        assert!(c.is_ram_enabled());

        c.write_rom_control(0x0000, 0x00);
        assert!(!c.is_ram_enabled());
    }

    #[test]
    fn test_mbc5_rom_bank_switching() {
        let mut c = MemoryBankController::new(1048576, 0x19);

        // 8-bit low byte only
        c.write_rom_control(0x2000, 0x42);
        assert_eq!(c.rom_bank_offset(), 0x42 * 0x4000);
    }

    #[test]
    fn test_mbc5_9bit_rom_bank() {
        let mut c = MemoryBankController::new(1048576, 0x19);

        c.write_rom_control(0x2000, 0xFF); // low 8 bits
        c.write_rom_control(0x3000, 0x01); // 9th bit
        // bank = (1 << 8) | 0xFF = 511
        assert_eq!(c.rom_bank_offset(), 511 * 0x4000);
    }

    #[test]
    fn test_mbc5_bank_zero_is_valid() {
        // Unlike MBC1/2/3, MBC5 can directly address bank 0 in the banked window.
        let mut c = MemoryBankController::new(1048576, 0x19);
        c.write_rom_control(0x2000, 0x00);
        c.write_rom_control(0x3000, 0x00);
        assert_eq!(c.rom_bank_offset(), 0);
    }

    #[test]
    fn test_mbc5_ram_bank_switching() {
        let mut c = MemoryBankController::new(1048576, 0x19);

        c.write_rom_control(0x4000, 0x00);
        assert_eq!(c.get_ram_bank(), 0);

        c.write_rom_control(0x4000, 0x0F);
        assert_eq!(c.get_ram_bank(), 15);
    }
}
