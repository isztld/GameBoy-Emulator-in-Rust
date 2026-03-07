    use super::*;

    #[test]
    fn test_lcdc_flags() {
        // 0xF5 = 1111_0101
        // bit7=1 bit6=1 bit5=1 bit4=1 bit3=0 bit2=1 bit1=0 bit0=1
        let lcdc = Lcdc::new(0xF5);
        assert!(lcdc.is_enabled());           // bit 7
        assert!(lcdc.window_tile_map_select()); // bit 6
        assert!(lcdc.window_display());        // bit 5
        assert!(lcdc.tile_data_select());      // bit 4
        assert!(!lcdc.tile_map_select());      // bit 3
        assert_eq!(lcdc.obj_size(), 16);       // bit 2
        assert!(!lcdc.obj_display());          // bit 1
        assert!(lcdc.bg_tile_map_display());   // bit 0
    }

    #[test]
    fn test_vblank_interrupt_triggered_on_entry() {
        let mut video = VideoController::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);

        // Start in HBlank mode with LY=143 and most of the cycles done
        // so next update will complete HBlank, increment LY to 144, and enter VBlank
        video.mode = PpuMode::HBlank;
        video.ly = 143;
        video.mode_clock = 50; // Almost done with HBlank

        // Run one more cycle to complete HBlank and enter VBlank
        video.update(&mut bus);

        // Should now be in VBlank mode with LY=144
        assert_eq!(video.mode, PpuMode::VBlank, "Should enter VBlank mode");
        assert_eq!(video.ly, 144);

        // VBlank interrupt bit (bit 0) should be set in IF register
        let if_val = bus.read(0xFF0F);
        assert_eq!(if_val & 0x01, 0x01, "VBlank interrupt bit should be set");
    }

    #[test]
    fn test_vblank_interrupt_persists_through_vblank() {
        let mut video = VideoController::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);

        // Set initial state to VBlank with IF bit set
        bus.write(0xFF0F, 0x01); // Set VBlank interrupt

        // Manually set mode to VBlank
        video.mode = PpuMode::VBlank;
        video.ly = 153; // Just before returning to OamScan

        // Run enough cycles to exit VBlank
        for _ in 0..114 {
            video.update(&mut bus);
        }
        assert_eq!(video.mode, PpuMode::OamScan);
        assert_eq!(video.ly, 0);

        // VBlank interrupt bit should persist (CPU clears it when servicing the interrupt)
        let if_val = bus.read(0xFF0F);
        assert_eq!(if_val & 0x01, 0x01, "VBlank interrupt bit should persist until CPU clears it");
    }

    #[test]
    fn test_full_vblank_duration() {
        let mut video = VideoController::new();
        let mut bus = MemoryBus::new(vec![0; 32768]);

        // Start in HBlank mode with LY=143, about to enter VBlank
        video.mode = PpuMode::HBlank;
        video.ly = 143;
        video.mode_clock = 50;

        // Run to enter VBlank
        video.update(&mut bus);
        assert_eq!(video.mode, PpuMode::VBlank);
        assert_eq!(video.ly, 144);

        // Simulate full VBlank duration: 10 scanlines (144-153) at 114 cycles each = 1140 cycles
        for _ in 144..=153 {
            for _ in 0..114 {
                video.update(&mut bus);
            }
        }
        assert_eq!(video.mode, PpuMode::OamScan);
        assert_eq!(video.ly, 0);

        // VBlank interrupt should still be set (not cleared by PPU)
        let if_val = bus.read(0xFF0F);
        assert_eq!(if_val & 0x01, 0x01, "VBlank interrupt should persist after VBlank ends");
    }
