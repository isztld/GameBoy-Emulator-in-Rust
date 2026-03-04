/// Rendering module for GameBoy PPU
///
/// Handles tile rendering, window display, and sprite rendering.

use crate::memory::MemoryBus;
use crate::ppu::video::Lcdc;
use crate::display::{FrameBuffer, SCREEN_WIDTH, SCREEN_HEIGHT};

/// Tile data (16 bytes per tile for 8x8 monochrome)
#[derive(Debug, Clone, Copy)]
pub struct Tile {
    pub data: [u8; 16],
}

impl Tile {
    pub fn new() -> Self {
        Tile { data: [0; 16] }
    }

    pub fn from_bytes(bytes: [u8; 16]) -> Self {
        Tile { data: bytes }
    }
}

/// Renderer state
#[derive(Debug)]
pub struct Renderer {
    pub tiles: [Tile; 384], // 384 tiles (64 KB / 16 bytes per tile)
    pub bg_palette: [u8; 4], // BGP palette
    pub obj_palette_0: [u8; 4], // Object palette 0
    pub obj_palette_1: [u8; 4], // Object palette 1
}

impl Renderer {
    pub fn new() -> Self {
        Renderer {
            tiles: [Tile::new(); 384],
            bg_palette: [0xFC; 4], // Power-on default
            obj_palette_0: [0xFF; 4],
            obj_palette_1: [0xFF; 4],
        }
    }

    /// Get a tile from VRAM
    pub fn get_tile(&self, index: usize) -> Option<&Tile> {
        self.tiles.get(index)
    }

    /// Set a tile in VRAM
    pub fn set_tile(&mut self, index: usize, tile: Tile) {
        if index < self.tiles.len() {
            self.tiles[index] = tile;
        }
    }

    /// Decode a tile row into pixel values
    /// Returns 8 pixel values (0-3) for the given row
    pub fn decode_tile_row(&self, tile: &Tile, row: usize) -> [u8; 8] {
        if row >= 8 {
            return [0; 8];
        }

        let mut pixels = [0; 8];

        let lsb = tile.data[row * 2];
        let msb = tile.data[row * 2 + 1];

        for i in 0..8 {
            let bit = 7 - i;
            let l = (lsb >> bit) & 1;
            let m = (msb >> bit) & 1;
            pixels[i] = (m << 1) | l;
        }

        pixels
    }

    /// Decode two bitplane bytes into 8 pixel colour indices (0-3).
    fn decode_bitplanes(lsb: u8, msb: u8) -> [u8; 8] {
        let mut row = [0u8; 8];
        for i in 0..8usize {
            let bit = 7 - i;
            let lo = (lsb >> bit) & 1;
            let hi = (msb >> bit) & 1;
            row[i] = (hi << 1) | lo;
        }
        row
    }

    /// Return the VRAM byte-offset of the first byte of a tile's row data.
    /// Handles both addressing modes:
    ///   - tile_data_select=true  → 0x8000 base, unsigned index 0-255
    ///   - tile_data_select=false → 0x9000 base, signed index -128..127
    fn tile_row_vram_offset(raw_index: u8, tile_row: usize, tile_data_select: bool) -> usize {
        let tile_base: usize = if tile_data_select {
            (raw_index as usize) * 16
        } else {
            let signed = raw_index as i8;
            (0x1000i32 + signed as i32 * 16) as usize
        };
        tile_base + tile_row * 2
    }

    /// Render a background scanline.
    /// Reads tile map and tile data directly from `bus.vram` to avoid the
    /// per-byte `bus.read()` dispatch overhead and the stale tile-cache.
    /// Decodes each tile row once and caches it for the 8 pixels it covers.
    pub fn render_background(
        &self,
        bus: &MemoryBus,
        scanline_y: u8,
        lcdc: &Lcdc,
        scroll_x: u8,
        scroll_y: u8,
    ) -> [u8; 160] {
        let mut pixels = [0u8; 160];

        if !lcdc.is_enabled() {
            return pixels;
        }

        // Tile map lives in VRAM; base offset relative to start of vram (0x8000).
        let tile_map_vram_base: usize = if lcdc.tile_map_select() { 0x1C00 } else { 0x1800 };
        let tile_data_select = lcdc.tile_data_select();

        let bg_y = scanline_y.wrapping_add(scroll_y) as usize;
        let tile_row = bg_y % 8;
        let tile_map_row = (bg_y / 8) % 32;

        // Cache the decoded pixel row for the current tile so we only call
        // decode_bitplanes once per 8-pixel tile span instead of per pixel.
        let mut cached_tile_x = usize::MAX;
        let mut cached_row = [0u8; 8];

        for screen_x in 0..160usize {
            let bg_x = (screen_x + scroll_x as usize) & 0xFF;
            let tile_map_col = (bg_x / 8) % 32;

            if tile_map_col != cached_tile_x {
                let map_offset = tile_map_vram_base + tile_map_row * 32 + tile_map_col;
                let raw_index = bus.vram[map_offset];
                let data_offset = Self::tile_row_vram_offset(raw_index, tile_row, tile_data_select);
                let lsb = bus.vram[data_offset];
                let msb = bus.vram[data_offset + 1];
                cached_row = Self::decode_bitplanes(lsb, msb);
                cached_tile_x = tile_map_col;
            }

            pixels[screen_x] = cached_row[bg_x % 8];
        }

        pixels
    }

    /// Render the window overlay for a scanline.
    /// Window pixels are non-zero where the window should be drawn.
    /// Window rendering is similar to background but starts from window position.
    pub fn render_window(
        &self,
        bus: &MemoryBus,
        scanline_y: u8,
        lcdc: &Lcdc,
        wx: u8,
        wy: u8,
        out: &mut [u8; 160],
    ) {
        // Window must be enabled and scanline must be at or below window Y
        if !lcdc.window_display() || scanline_y < wy {
            return;
        }

        // Window X position: the left edge of the window is at screen X = wx - 7.
        // wx < 7 means the window is fully off-screen to the left; wx=7 is valid
        // and places the window starting at screen x=0.
        if wx < 7 {
            return;
        }

        // Window uses bit 6 of LCDC (window_tile_map_select), not bit 3 (bg tile map).
        let tile_map_vram_base: usize = if lcdc.window_tile_map_select() { 0x1C00 } else { 0x1800 };
        let tile_data_select = lcdc.tile_data_select();

        // Window scanline within the window (0-159)
        let window_y = (scanline_y - wy) as usize;
        let tile_row = window_y % 8;
        let tile_map_row = (window_y / 8) % 32;

        // Cache the decoded pixel row
        let mut cached_tile_x = usize::MAX;
        let mut cached_row = [0u8; 8];

        // Window starts at screen X = wx - 7
        let window_start_x = (wx as usize) - 7;

        for screen_x in window_start_x..SCREEN_WIDTH {
            let window_x = screen_x - window_start_x;
            let tile_map_col = (window_x / 8) % 32;

            if tile_map_col != cached_tile_x {
                let map_offset = tile_map_vram_base + tile_map_row * 32 + tile_map_col;
                let raw_index = bus.vram[map_offset];
                let data_offset = Self::tile_row_vram_offset(raw_index, tile_row, tile_data_select);
                let lsb = bus.vram[data_offset];
                let msb = bus.vram[data_offset + 1];
                cached_row = Self::decode_bitplanes(lsb, msb);
                cached_tile_x = tile_map_col;
            }

            out[screen_x] = cached_row[window_x % 8];
        }
    }

    /// Render sprites for a scanline into a fixed-size overlay buffer.
    /// `out[x]` is non-zero where a sprite pixel should be drawn.
    /// `out_priority[x]` is true when OAM flag bit 7 is set (sprite behind BG).
    /// Lower OAM index wins when sprites overlap (first sprite in OAM has priority).
    /// Uses direct VRAM access; no heap allocation.
    pub fn render_sprites(
        &self,
        bus: &MemoryBus,
        oam_bytes: &[u8; 160],
        scanline_y: u8,
        lcdc: &Lcdc,
        out: &mut [u8; 160],
        out_priority: &mut [bool; 160],
    ) {
        if !lcdc.obj_display() {
            return;
        }

        let height = lcdc.obj_size(); // 8 or 16
        let tile_data_select = true; // sprites always use 0x8000-base addressing

        for i in 0..40usize {
            let base = i * 4;
            let sprite_y = oam_bytes[base];
            let sprite_x = oam_bytes[base + 1];
            let raw_tile  = oam_bytes[base + 2];
            let flags     = oam_bytes[base + 3];

            // GB OAM: sprite_y is the bottom of the sprite + 16, sprite_x is left + 8.
            // A sprite with sprite_y=0 or sprite_x=0 is hidden.
            if sprite_y == 0 || sprite_x == 0 || sprite_y >= 160 || sprite_x >= 168 {
                continue;
            }

            let screen_top = sprite_y.saturating_sub(16);
            let screen_bottom = screen_top + height as u8;

            if scanline_y < screen_top || scanline_y >= screen_bottom {
                continue;
            }

            let mut tile_row = (scanline_y - screen_top) as usize;
            let tile_index = if height == 16 {
                if flags & 0x40 != 0 { tile_row = 15 - tile_row; } // Y-flip
                if tile_row < 8 { raw_tile & 0xFE } else { tile_row -= 8; raw_tile | 0x01 }
            } else {
                if flags & 0x40 != 0 { tile_row = 7 - tile_row; } // Y-flip
                raw_tile
            };

            let data_offset = Self::tile_row_vram_offset(tile_index, tile_row, tile_data_select);
            let lsb = bus.vram[data_offset];
            let msb = bus.vram[data_offset + 1];
            let mut row = Self::decode_bitplanes(lsb, msb);

            if flags & 0x20 != 0 { row.reverse(); } // X-flip

            for px in 0..8usize {
                let screen_x = (sprite_x as usize + px).wrapping_sub(8);
                // Only write if the pixel is opaque and no earlier sprite already
                // claimed this position (lower OAM index has priority on DMG).
                if screen_x < 160 && row[px] != 0 && out[screen_x] == 0 {
                    out[screen_x] = row[px];
                    out_priority[screen_x] = flags & 0x80 != 0;
                }
            }
        }
    }

    /// Render a complete scanline to the frame buffer.
    /// Combines background, window, and sprite rendering for a single scanline.
    pub fn render_scanline(
        &self,
        frame_buffer: &mut FrameBuffer,
        bus: &MemoryBus,
        scanline_y: u8,
        lcdc: &Lcdc,
        scroll_x: u8,
        scroll_y: u8,
        wx: u8,
        wy: u8,
        oam_bytes: &[u8; 160],
    ) {
        if !lcdc.is_enabled() {
            return;
        }

        // When LCDC bit 0 is clear the BG and Window are disabled (all white/0).
        let bg_pixels = if lcdc.bg_tile_map_display() {
            self.render_background(bus, scanline_y, lcdc, scroll_x, scroll_y)
        } else {
            [0u8; 160]
        };

        // Window overlay — zero means "no window pixel here"
        let mut window_pixels = [0u8; 160];
        if lcdc.bg_tile_map_display() {
            self.render_window(bus, scanline_y, lcdc, wx, wy, &mut window_pixels);
        }

        // Sprite overlay — zero means "no sprite pixel here"
        let mut sprite_pixels = [0u8; 160];
        // true = OAM bit 7 set, sprite renders behind BG/window color indices 1-3
        let mut sprite_priority = [false; 160];
        self.render_sprites(bus, oam_bytes, scanline_y, lcdc, &mut sprite_pixels, &mut sprite_priority);

        let y = scanline_y as usize;
        for x in 0..SCREEN_WIDTH {
            // Resolve the BG+Window color for this pixel (window on top of BG).
            let bg_win_color = if window_pixels[x] != 0 { window_pixels[x] } else { bg_pixels[x] };

            // Sprite compositing:
            // - color 0 is always transparent.
            // - OAM bit 7 = 0 (priority): sprite is above BG/window.
            // - OAM bit 7 = 1 (behind BG): sprite only shows where BG/window = 0.
            let color = if sprite_pixels[x] != 0 && (!sprite_priority[x] || bg_win_color == 0) {
                sprite_pixels[x]
            } else {
                bg_win_color
            };
            frame_buffer.set_pixel(x, y, color);
        }
    }

    /// Render the entire frame to a frame buffer
    /// This is a simplified version that renders all 144 scanlines
    pub fn render_frame(
        &self,
        frame_buffer: &mut FrameBuffer,
        bus: &MemoryBus,
        lcdc: &Lcdc,
        scroll_x: u8,
        scroll_y: u8,
        wx: u8,
        wy: u8,
        oam_bytes: &[u8; 160],
    ) {
        for y in 0..SCREEN_HEIGHT {
            self.render_scanline(frame_buffer, bus, y as u8, lcdc, scroll_x, scroll_y, wx, wy, oam_bytes);
        }
        frame_buffer.mark_frame_ready();
    }
}

impl Default for Renderer {
    fn default() -> Self {
        Self::new()
    }
}
