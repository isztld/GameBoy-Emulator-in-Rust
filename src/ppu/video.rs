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
    pub(crate) mode: PpuMode,
    pub(crate) mode_clock: u32, // Clock cycles in current mode
    pub(crate) ly: u8,          // LCD Y coordinate (0-153)
    pub(crate) lyc: u8,         // LY compare
    pub(crate) lcdc: Lcdc,      // LCD control
    pub(crate) stat: u8,        // LCD status
    pub(crate) scy: u8,         // Scroll Y
    pub(crate) scx: u8,         // Scroll X
    pub(crate) wy: u8,          // Window Y
    pub(crate) wx: u8,          // Window X
    // Rendering components
    pub(crate) renderer: Renderer,
    pub(crate) frame_buffer: SharedFrameBuffer,
    /// Set to true when PixelTransfer→HBlank transition occurs;
    /// system.step() renders the scanline then clears this flag.
    pub(crate) scanline_ready: bool,
    /// Set to true on the first cycle of VBlank entry (edge-triggered).
    /// Cleared by system.step() after it sets frame_complete.
    pub(crate) vblank_entered: bool,
    /// Internal window line counter. Increments once per scanline where the
    /// window is actually rendered. Reset to 0 at the start of each frame
    /// (when LY wraps to 0). Using LY-WY instead of this causes wrong tiles
    /// whenever the window is toggled or WY changes mid-frame.
    pub(crate) window_line: u8,
    /// Tracks whether the LCD was enabled on the previous tick, so we can
    /// detect the 1→0 edge (LCD just disabled) and blank the frame buffer.
    pub(crate) lcd_was_enabled: bool,
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
            renderer: Renderer::new(),
            frame_buffer: create_shared_frame_buffer(),
            scanline_ready: false,
            vblank_entered: false,
            window_line: 0,
            lcd_was_enabled: true,
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
            renderer: Renderer::new(),
            frame_buffer,
            scanline_ready: false,
            vblank_entered: false,
            window_line: 0,
            lcd_was_enabled: true,
        }
    }

    /// Advance the PPU state machine by one machine cycle using only the I/O
    /// register array (bus.io).  This is the hot path called once per M-cycle
    /// during instruction execution; OAM DMA is handled separately via
    /// `update()` which still takes the full bus.
    ///
    /// Writes IF bit 0 (VBlank) directly into io[0x0F] and keeps LY (io[0x44])
    /// and STAT mode bits (io[0x41]) up to date.
    ///
    /// Also syncs LCDC/SCX/SCY from the I/O array so rendering sees the latest
    /// values written by the CPU (critical for games that modify scroll regs
    /// during VBlank to animate the next frame).
    pub fn tick_io(&mut self, io: &mut [u8; 128]) {
        // Sync scroll/LCDC registers from bus.io[] — the CPU writes these
        // directly to the I/O array, and they must be reflected in the PPU.
        self.lcdc = Lcdc::new(io[0x40]);
        self.scy  = io[0x42];
        self.scx  = io[0x43];
        self.lyc  = io[0x45]; // LYC must be synced so LY==LYC comparison is correct
        self.wy   = io[0x4A]; // Window Y
        self.wx   = io[0x4B]; // Window X

        // Detect LCD enable/disable edges.
        let lcd_just_disabled = self.lcd_was_enabled && !self.lcdc.is_enabled();
        let lcd_just_enabled  = !self.lcd_was_enabled && self.lcdc.is_enabled();
        self.lcd_was_enabled = self.lcdc.is_enabled();

        // When the LCD is disabled (LCDC bit 7 = 0) the PPU halts: LY is forced
        // to 0, mode is set to HBlank (mode 0), and the mode clock is cleared.
        // The PPU must NOT advance its state machine while disabled.
        if !self.lcdc.is_enabled() {
            self.ly = 0;
            self.mode = PpuMode::HBlank;
            self.mode_clock = 0;
            // Still update STAT and sync the I/O registers so the CPU sees
            // consistent values (LY=0, mode=0) while the LCD is off.
            self.update_stat();
            io[0x44] = self.ly;
            io[0x41] = 0x80 | (io[0x41] & 0x78) | (self.stat & 0x07);
            // On the first cycle after LCD disable, fill the frame buffer with
            // white and signal a completed frame so the display shows a blank screen
            // instead of retaining the last rendered image.
            if lcd_just_disabled {
                let mut fb = self.frame_buffer.lock().unwrap();
                fb.pixels.fill(0xFFFFFFFF_u32);
                fb.mark_frame_ready();
                drop(fb);
                self.vblank_entered = true;
            }
            return;
        }

        // On the 0→1 edge (LCD just turned on): reset the PPU to the beginning
        // of line 0 in OAM-scan mode.  Hardware behaviour: the first line after
        // LCD-on takes 110 M-cycles (not 114) before LY increments.  Modelling
        // this as mode_clock=1 (one M-cycle already elapsed) means OAM scan
        // runs for 19 ticks instead of 20, giving 19+43+51 = 113 total ticks;
        // advance_mode() increments mode_clock immediately, so LY becomes 1 at
        // tick 112 from the LCD-on event — exactly what hardware does.
        if lcd_just_enabled {
            self.ly = 0;
            self.mode = PpuMode::OamScan;
            self.mode_clock = 1;
            self.window_line = 0;
        }

        self.advance_mode(io);
        self.update_stat();
        // Sync LY and STAT bits 0-2 to the I/O array so the CPU sees current
        // values on every subsequent bus read within the same instruction.
        io[0x44] = self.ly;
        io[0x41] = 0x80 | (io[0x41] & 0x78) | (self.stat & 0x07);
    }

    /// Advance the PPU by one machine cycle.
    /// Used by tests and any call-site that has full bus access;
    /// System::step() uses `tick_io` directly inside the tick closure.
    pub fn update(&mut self, bus: &mut MemoryBus) {
        self.tick_io(&mut bus.io);
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
                    // HBlank STAT interrupt (STAT bit 3).
                    if io[0x41] & 0x08 != 0 {
                        io[0x0F] = 0xE0 | ((io[0x0F] | 0x02) & 0x1F);
                    }
                }
            }
            PpuMode::HBlank => {
                self.mode_clock += 1;
                if self.mode_clock >= 51 {
                    self.ly += 1;
                    self.mode_clock = 0;

                    // LYC=LY STAT interrupt (STAT bit 6) on every LY update.
                    if self.ly == self.lyc && io[0x41] & 0x40 != 0 {
                        io[0x0F] = 0xE0 | ((io[0x0F] | 0x02) & 0x1F);
                    }

                    if self.ly >= 144 {
                        self.mode = PpuMode::VBlank;
                        self.vblank_entered = true;
                        // VBlank interrupt (IF bit 0).
                        io[0x0F] = 0xE0 | ((io[0x0F] | 0x01) & 0x1F);
                        // VBlank STAT interrupt (STAT bit 4).
                        if io[0x41] & 0x10 != 0 {
                            io[0x0F] = 0xE0 | ((io[0x0F] | 0x02) & 0x1F);
                        }
                    } else {
                        self.mode = PpuMode::OamScan;
                        // OAM-scan STAT interrupt (STAT bit 5).
                        if io[0x41] & 0x20 != 0 {
                            io[0x0F] = 0xE0 | ((io[0x0F] | 0x02) & 0x1F);
                        }
                    }
                }
            }
            PpuMode::VBlank => {
                self.mode_clock += 1;
                if self.mode_clock >= 114 {
                    self.ly += 1;
                    self.mode_clock = 0;

                    // LYC=LY check during VBlank lines 144–153.
                    if self.ly == self.lyc && io[0x41] & 0x40 != 0 {
                        io[0x0F] = 0xE0 | ((io[0x0F] | 0x02) & 0x1F);
                    }

                    if self.ly > 153 {
                        self.ly = 0;
                        self.window_line = 0; // Reset window line counter for the new frame.
                        self.mode = PpuMode::OamScan;
                        // OAM-scan STAT interrupt for the first line of the new frame.
                        if io[0x41] & 0x20 != 0 {
                            io[0x0F] = 0xE0 | ((io[0x0F] | 0x02) & 0x1F);
                        }
                        // LYC=LY check for LY=0.
                        if self.lyc == 0 && io[0x41] & 0x40 != 0 {
                            io[0x0F] = 0xE0 | ((io[0x0F] | 0x02) & 0x1F);
                        }
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

    /// Get current LY (Y coordinate)
    pub fn get_ly(&self) -> u8 {
        self.ly
    }

    /// Get the frame buffer
    pub fn get_frame_buffer(&self) -> SharedFrameBuffer {
        Arc::clone(&self.frame_buffer)
    }

    /// Render the current scanline to the frame buffer.
    /// Must be &mut self because we update the window line counter.
    pub fn render_scanline(&mut self, bus: &MemoryBus) {
        if !self.lcdc.is_enabled() {
            return;
        }

        let scanline_y = self.ly;
        let scy = self.scy;
        let scx = self.scx;
        let wy  = self.wy;
        let wx  = self.wx;

        // Read palette registers directly from the I/O array.
        let bgp  = bus.io[0x47];
        let obp0 = bus.io[0x48];
        let obp1 = bus.io[0x49];

        let mut fb = self.frame_buffer.lock().unwrap();

        self.renderer.render_scanline(
            &mut fb,
            bus,
            scanline_y,
            &self.lcdc,
            scx,
            scy,
            wx,
            wy,
            &bus.oam,
            self.window_line,
            bgp,
            obp0,
            obp1,
        );

        // The window line counter increments every scanline on which the window
        // is actually rendered (visible, enabled, and within bounds).
        // wx=167+ hides the window (screen x = wx-7 ≥ 160); wx<7 is clipped but visible.
        let window_visible = self.lcdc.bg_tile_map_display()
            && self.lcdc.window_display()
            && scanline_y >= wy
            && wx < 167;
        if window_visible {
            self.window_line = self.window_line.wrapping_add(1);
        }

        // After the last visible scanline, mark the frame as ready for display.
        if scanline_y == 143 {
            fb.mark_frame_ready();
        }
    }

}

#[cfg(test)]
#[path = "video_tests.rs"]
mod tests;
