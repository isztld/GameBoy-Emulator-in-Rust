/// Video Controller for GameBoy PPU
///
/// The Video Controller handles VRAM access, LCD control,
/// and display generation.

use crate::memory::MemoryBus;

/// PPU Mode
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PpuMode {
    HBlank,       // Horizontal blanking
    VBlank,       // Vertical blanking
    OamScan,      // Scanning OAM
    PixelTransfer, // Transferring pixel data
}

/// LCD Control Register (LCDC)
#[derive(Debug, Clone, Copy)]
pub struct Lcdc(u8);

impl Lcdc {
    pub fn new(value: u8) -> Self {
        Lcdc(value)
    }

    pub fn is_enabled(&self) -> bool {
        self.0 & 0x80 != 0
    }

    pub fn window_tile_map_select(&self) -> bool {
        self.0 & 0x40 != 0
    }

    pub fn window_display(&self) -> bool {
        self.0 & 0x20 != 0
    }

    pub fn tile_map_select(&self) -> bool {
        self.0 & 0x10 != 0
    }

    pub fn tile_data_select(&self) -> bool {
        self.0 & 0x08 != 0
    }

    pub fn bg_tile_map_display(&self) -> bool {
        self.0 & 0x04 != 0
    }

    pub fn obj_size(&self) -> usize {
        if self.0 & 0x02 != 0 { 16 } else { 8 }
    }

    pub fn obj_display(&self) -> bool {
        self.0 & 0x01 != 0
    }
}

/// Video Controller
#[derive(Debug)]
pub struct VideoController {
    pub mode: PpuMode,
    pub mode_clock: u32, // Clock cycles in current mode
    pub ly: u8,          // LCD Y coordinate (0-153)
    pub lyc: u8,         // LY compare
    pub lcdc: Lcdc,      // LCD control
    pub stat: u8,        // LCD status
    pub scy: u8,         // Scroll Y
    pub scx: u8,         // Scroll X
    pub wy: u8,          // Window Y
    pub wx: u8,          // Window X
    pub dma: u8,         // OAM DMA source
    pub oam_dma_active: bool,
    pub oam_dma_address: u16,
}

impl VideoController {
    pub fn new() -> Self {
        VideoController {
            mode: PpuMode::OamScan,
            mode_clock: 0,
            ly: 0,
            lyc: 0,
            lcdc: Lcdc::new(0x91),
            stat: 0x85,
            scy: 0,
            scx: 0,
            wy: 0,
            wx: 0,
            dma: 0,
            oam_dma_active: false,
            oam_dma_address: 0,
        }
    }

    /// Update PPU mode and state
    pub fn update(&mut self, bus: &mut MemoryBus) {
        // Handle OAM DMA
        if self.oam_dma_active {
            self.perform_oam_dma(bus);
        }

        // Update mode based on clock cycles
        // Total mode cycle counts:
        // - OamScan: 20 cycles
        // - PixelTransfer: 43 cycles
        // - HBlank: 51 cycles
        // - VBlank: 114 cycles
        match self.mode {
            PpuMode::OamScan => {
                self.mode_clock += 1;
                if self.mode_clock >= 20 {
                    self.mode = PpuMode::PixelTransfer;
                    self.mode_clock = 0;
                }
            }
            PpuMode::PixelTransfer => {
                self.mode_clock += 1;
                if self.mode_clock >= 43 {
                    self.ly += 1;
                    self.mode_clock = 0;
                    if self.ly >= 144 {
                        self.mode = PpuMode::VBlank;
                    } else {
                        self.mode = PpuMode::HBlank;
                    }
                }
            }
            PpuMode::HBlank => {
                self.mode_clock += 1;
                if self.mode_clock >= 51 {
                    self.ly += 1;
                    self.mode_clock = 0;
                    if self.ly >= 153 {
                        self.mode = PpuMode::OamScan;
                        self.ly = 0;
                    } else {
                        self.mode = PpuMode::OamScan;
                    }
                }
            }
            PpuMode::VBlank => {
                self.mode_clock += 1;
                if self.mode_clock >= 114 {
                    self.ly += 1;
                    self.mode_clock = 0;
                    if self.ly > 153 {
                        self.ly = 0;
                        self.mode = PpuMode::OamScan;
                    }
                }
            }
        }

        // Update STAT register
        self.update_stat();
    }

    fn update_stat(&mut self) {
        // Set mode bits (bits 0-1)
        self.stat &= !0x03;
        self.stat |= match self.mode {
            PpuMode::HBlank => 0x00,
            PpuMode::VBlank => 0x01,
            PpuMode::OamScan => 0x02,
            PpuMode::PixelTransfer => 0x03,
        };

        // Check LYC comparison
        if self.ly == self.lyc {
            self.stat |= 0x04; // Set LYC flag
        } else {
            self.stat &= !0x04;
        }
    }

    fn perform_oam_dma(&mut self, bus: &mut MemoryBus) {
        let source_base = (self.dma as u16) << 8;
        for i in 0..160 {
            let addr = source_base + i as u16;
            let value = bus.read(addr);
            bus.write(0xFE00 + i as u16, value);
        }
        self.oam_dma_active = false;
    }

    /// Read from LCD status register (FF41)
    pub fn read_stat(&self) -> u8 {
        self.stat
    }

    /// Write to LCD status register (FF41)
    pub fn write_stat(&mut self, value: u8) {
        // Only bits 3-6 are writable
        self.stat = (self.stat & 0x07) | (value & 0xF8);
    }

    /// Check if LYC matches LY
    pub fn lyc_matches(&self) -> bool {
        self.ly == self.lyc
    }

    /// Request OAM DMA
    pub fn start_oam_dma(&mut self, dma_source: u8) {
        self.dma = dma_source;
        self.oam_dma_active = true;
    }

    /// Get current LY (Y coordinate)
    pub fn get_ly(&self) -> u8 {
        self.ly
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lcdc_flags() {
        let lcdc = Lcdc::new(0xF5);
        assert!(lcdc.is_enabled());
        assert!(lcdc.window_tile_map_select());
        assert!(lcdc.window_display());
        assert!(lcdc.tile_map_select());
        assert!(lcdc.bg_tile_map_display());
        assert_eq!(lcdc.obj_size(), 8);
        assert!(lcdc.obj_display());
    }
}
