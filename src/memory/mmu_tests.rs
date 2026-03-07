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

        let temp_path = std::env::temp_dir()
            .join(format!("gb_serial_{}.txt", std::process::id()));

        let file = OpenOptions::new()
            .write(true).create(true).truncate(true)
            .open(&temp_path)
            .expect("open temp file");

        let mut bus = make_bus(32768);
        bus.serial_log_file = Some(Arc::new(Mutex::new(file)));

        for ch in [b'H', b'i'] {
            bus.write(0xFF01, ch);
            bus.write(0xFF02, 0x81);
        }

        // Flush and close by dropping the log file reference.
        bus.serial_log_file = None;

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
