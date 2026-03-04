/// Video Controller for GameBoy PPU
///
/// The Video Controller handles VRAM access, LCD control,
/// and display generation.

use crate::memory::MemoryBus;
use crate::ppu::rendering::Renderer;
use crate::display::{SharedFrameBuffer, create_shared_frame_buffer};
use std::sync::Arc;

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

    /// BG Tile Map Display Select: bit 3 (0=0x9800, 1=0x9C00)
    pub fn tile_map_select(&self) -> bool {
        self.0 & 0x08 != 0
    }

    /// BG & Window Tile Data Select: bit 4 (0=0x8800 signed, 1=0x8000 unsigned)
    pub fn tile_data_select(&self) -> bool {
        self.0 & 0x10 != 0
    }

    /// BG & Window Display Enable: bit 0
    pub fn bg_tile_map_display(&self) -> bool {
        self.0 & 0x01 != 0
    }

    /// OBJ (Sprite) Size: bit 2 (0=8x8, 1=8x16)
    pub fn obj_size(&self) -> usize {
        if self.0 & 0x04 != 0 { 16 } else { 8 }
    }

    /// OBJ Display Enable: bit 1
    pub fn obj_display(&self) -> bool {
        self.0 & 0x02 != 0
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
    // Rendering components
    pub renderer: Renderer,
    pub frame_buffer: SharedFrameBuffer,
    /// Set to true when PixelTransfer→HBlank transition occurs;
    /// system.step() renders the scanline then clears this flag.
    pub scanline_ready: bool,
    /// Set to true on the first cycle of VBlank entry (edge-triggered).
    /// Cleared by system.step() after it sets frame_complete.
    pub vblank_entered: bool,
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
            renderer: Renderer::new(),
            frame_buffer: create_shared_frame_buffer(),
            scanline_ready: false,
            vblank_entered: false,
        }
    }

    /// Create a new VideoController with a shared frame buffer
    pub fn with_frame_buffer(frame_buffer: SharedFrameBuffer) -> Self {
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
            renderer: Renderer::new(),
            frame_buffer,
            scanline_ready: false,
            vblank_entered: false,
        }
    }

    /// Advance the PPU state machine by one machine cycle using only the I/O
    /// register array (bus.io).  This is the hot path called once per M-cycle
    /// during instruction execution; OAM DMA is handled separately via
    /// `update()` which still takes the full bus.
    ///
    /// Writes IF bit 0 (VBlank) directly into io[0x0F] and keeps LY (io[0x44])
    /// and STAT mode bits (io[0x41]) up to date.
    pub fn tick_io(&mut self, io: &mut [u8; 128]) {
        self.advance_mode(io);
        self.update_stat();
        // Sync LY and STAT bits 0-2 to the I/O array so the CPU sees current
        // values on every subsequent bus read within the same instruction.
        io[0x44] = self.ly;
        io[0x41] = 0x80 | (io[0x41] & 0x78) | (self.stat & 0x07);
    }

    /// Advance the PPU by one machine cycle, including OAM DMA.
    /// This is equivalent to calling `tick_io(&mut bus.io)` followed by an
    /// OAM DMA check.  Used by tests and any call-site that has full bus
    /// access; System::step() uses `tick_io` directly inside the tick closure
    /// and calls `handle_oam_dma` separately.
    pub fn update(&mut self, bus: &mut MemoryBus) {
        if self.oam_dma_active {
            self.perform_oam_dma(bus);
        }
        self.tick_io(&mut bus.io);
    }

    /// Perform any pending OAM DMA transfer without advancing the state
    /// machine.  Called from System::step() after the per-cycle tick closure
    /// has already run `tick_io()`.
    pub fn handle_oam_dma(&mut self, bus: &mut MemoryBus) {
        if self.oam_dma_active {
            self.perform_oam_dma(bus);
        }
    }

    /// Inner mode state-machine step shared by tick_io and update.
    fn advance_mode(&mut self, io: &mut [u8; 128]) {
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
                    self.mode_clock = 0;
                    self.mode = PpuMode::HBlank;
                    self.scanline_ready = true;
                }
            }
            PpuMode::HBlank => {
                self.mode_clock += 1;
                if self.mode_clock >= 51 {
                    self.ly += 1;
                    self.mode_clock = 0;
                    if self.ly >= 144 {
                        self.mode = PpuMode::VBlank;
                        self.vblank_entered = true;
                        // Set VBlank interrupt (bit 0 of IF).
                        io[0x0F] = 0xE0 | ((io[0x0F] | 0x01) & 0x1F);
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
                        // Do NOT clear IF bit 0 here — the CPU clears it when
                        // servicing the interrupt.
                    }
                }
            }
        }
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

    /// Get the frame buffer
    pub fn get_frame_buffer(&self) -> SharedFrameBuffer {
        Arc::clone(&self.frame_buffer)
    }

    /// Render the current scanline to the frame buffer
    pub fn render_scanline(&self, bus: &MemoryBus) {
        if !self.lcdc.is_enabled() {
            return;
        }

        let mut fb = self.frame_buffer.lock().unwrap();

        // Get control registers from memory
        let scy = self.scy;
        let scx = self.scx;

        // Render this scanline
        self.renderer.render_scanline(
            &mut fb,
            bus,
            self.ly,
            &self.lcdc,
            scx,
            scy,
            &bus.oam,
        );

        // After the last visible scanline, mark the frame as ready for display.
        if self.ly == 143 {
            fb.mark_frame_ready();
        }
    }

    /// Render a complete frame to the frame buffer
    pub fn render_frame(&self, bus: &MemoryBus) {
        if !self.lcdc.is_enabled() {
            return;
        }

        let mut fb = self.frame_buffer.lock().unwrap();
        fb.clear();

        // Render all 144 scanlines
        for y in 0..144 {
            self.renderer.render_scanline(
                &mut fb,
                bus,
                y as u8,
                &self.lcdc,
                self.scx,
                self.scy,
                &bus.oam,
            );
        }

        fb.mark_frame_ready();
    }
}

#[cfg(test)]
mod tests {
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
}
