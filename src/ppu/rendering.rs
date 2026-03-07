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

    /// Map a raw colour index (0-3) through a GB palette register to a shade (0-3).
    /// Palette register layout: bits 1-0 = shade for index 0, bits 3-2 = index 1, etc.
    #[inline]
    fn apply_palette(palette: u8, idx: u8) -> u8 {
        (palette >> (idx * 2)) & 0x03
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
    /// `window_line` is the PPU's internal window line counter (separate from LY).
    /// It increments only on scanlines where the window is actually visible, so it
    /// correctly handles mid-frame window enable/disable and WY changes.
    pub fn render_window(
        &self,
        bus: &MemoryBus,
        scanline_y: u8,
        lcdc: &Lcdc,
        wx: u8,
        wy: u8,
        window_line: u8,
        out: &mut [u8; 160],
    ) {
        // Window must be enabled and scanline must be at or below window Y.
        if !lcdc.window_display() || scanline_y < wy {
            return;
        }

        // wx=167+ hides the window entirely (screen x = wx-7 ≥ 160).
        if wx > 166 {
            return;
        }

        // Window uses bit 6 of LCDC (window_tile_map_select), not bit 3 (bg tile map).
        let tile_map_vram_base: usize = if lcdc.window_tile_map_select() { 0x1C00 } else { 0x1800 };
        let tile_data_select = lcdc.tile_data_select();

        // Use the internal window line counter, not scanline_y - wy.
        let window_y = window_line as usize;
        let tile_row = window_y % 8;
        let tile_map_row = (window_y / 8) % 32;

        // Cache the decoded pixel row
        let mut cached_tile_x = usize::MAX;
        let mut cached_row = [0u8; 8];

        // wx ≥ 7: window's left edge is at screen x = wx-7.
        // wx < 7: window is clipped on the left; rendering starts at screen x=0,
        //         but the first (7-wx) window pixels are skipped.
        let (window_start_x, window_pixel_offset) = if wx >= 7 {
            ((wx as usize) - 7, 0usize)
        } else {
            (0usize, (7 - wx) as usize)
        };

        for screen_x in window_start_x..SCREEN_WIDTH {
            let window_x = (screen_x - window_start_x) + window_pixel_offset;
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
    /// `out_palette[x]` is true when the sprite should use OBP1 (flag bit 4 set).
    /// Lower OAM X value wins when sprites overlap; ties broken by lower OAM index.
    /// Uses direct VRAM access; no heap allocation.
    pub fn render_sprites(
        &self,
        bus: &MemoryBus,
        oam_bytes: &[u8; 160],
        scanline_y: u8,
        lcdc: &Lcdc,
        out: &mut [u8; 160],
        out_priority: &mut [bool; 160],
        out_palette: &mut [bool; 160],
    ) {
        if !lcdc.obj_display() {
            return;
        }

        let height = lcdc.obj_size(); // 8 or 16
        let tile_data_select = true; // sprites always use 0x8000-base addressing

        // DMG priority: the sprite with the smallest OAM X value wins at each pixel.
        // Ties are broken by lower OAM index (naturally handled by iterating 0→39
        // and only overwriting with strictly-smaller X).
        let mut pixel_x = [u8::MAX; 160];

        // Hardware selects only the first 10 Y-matching sprites per scanline.
        // Sprites with X=0 or X≥168 still count toward this limit but are not drawn.
        let mut sprite_count = 0usize;

        for i in 0..40usize {
            if sprite_count >= 10 {
                break;
            }

            let base = i * 4;
            let sprite_y = oam_bytes[base];
            let sprite_x = oam_bytes[base + 1];
            let raw_tile  = oam_bytes[base + 2];
            let flags     = oam_bytes[base + 3];

            // GB OAM: sprite_y encodes screen_top + 16; sprite_x encodes screen_left + 8.
            // Y=0 and Y≥160 are never on any visible scanline — skip without counting.
            if sprite_y == 0 || sprite_y >= 160 {
                continue;
            }

            // Use signed arithmetic so partially top-clipped sprites (sprite_y < 16)
            // get the correct tile row instead of saturating to row 0.
            let screen_top    = sprite_y as i16 - 16;
            let screen_bottom = screen_top + height as i16;

            if (scanline_y as i16) < screen_top || (scanline_y as i16) >= screen_bottom {
                continue; // Not in Y range — does not count toward limit
            }

            // This sprite is Y-selected; counts toward the 10-per-scanline limit.
            sprite_count += 1;

            // X=0 or X≥168 hides the sprite but still counts toward the limit (handled above).
            if sprite_x == 0 || sprite_x >= 168 {
                continue;
            }

            // Correct tile row within the sprite tile, handling partial top clip.
            let mut tile_row = (scanline_y as i16 - screen_top) as usize;
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
                if screen_x < 160 && row[px] != 0 && sprite_x < pixel_x[screen_x] {
                    out[screen_x] = row[px];
                    out_priority[screen_x] = flags & 0x80 != 0;
                    out_palette[screen_x]  = flags & 0x10 != 0; // bit 4: 0=OBP0, 1=OBP1
                    pixel_x[screen_x] = sprite_x;
                }
            }
        }
    }

    /// Render a complete scanline to the frame buffer.
    /// Combines background, window, and sprite rendering for a single scanline.
    /// `window_line` is the PPU's internal window line counter (caller must increment it).
    /// `bgp`, `obp0`, `obp1` are the current palette register values from io[0x47..=0x49].
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
        window_line: u8,
        bgp: u8,
        obp0: u8,
        obp1: u8,
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
            self.render_window(bus, scanline_y, lcdc, wx, wy, window_line, &mut window_pixels);
        }

        // Sprite overlay — zero means "no sprite pixel here"
        let mut sprite_pixels   = [0u8; 160];
        let mut sprite_priority = [false; 160];
        let mut sprite_palette  = [false; 160]; // false=OBP0, true=OBP1
        self.render_sprites(bus, oam_bytes, scanline_y, lcdc,
                            &mut sprite_pixels, &mut sprite_priority, &mut sprite_palette);

        let y = scanline_y as usize;
        for x in 0..SCREEN_WIDTH {
            // Raw (pre-palette) BG+Window index for priority comparisons.
            let raw_bg_win = if window_pixels[x] != 0 { window_pixels[x] } else { bg_pixels[x] };
            // BG+Window colour after BGP mapping.
            let bg_win_color = Self::apply_palette(bgp, raw_bg_win);

            // Sprite compositing:
            // - index 0 is always transparent (never enters this branch).
            // - OAM bit 7 = 0 (priority): sprite is above BG/window.
            // - OAM bit 7 = 1 (behind BG): sprite only shows where raw BG/window index = 0.
            let color = if sprite_pixels[x] != 0 && (!sprite_priority[x] || raw_bg_win == 0) {
                let obp = if sprite_palette[x] { obp1 } else { obp0 };
                Self::apply_palette(obp, sprite_pixels[x])
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
        bgp: u8,
        obp0: u8,
        obp1: u8,
    ) {
        let mut window_line = 0u8;
        let window_enable = lcdc.bg_tile_map_display() && lcdc.window_display() && wx < 167;
        for y in 0..SCREEN_HEIGHT {
            let scanline_y = y as u8;
            self.render_scanline(frame_buffer, bus, scanline_y, lcdc,
                                 scroll_x, scroll_y, wx, wy, oam_bytes,
                                 window_line, bgp, obp0, obp1);
            if window_enable && scanline_y >= wy {
                window_line = window_line.wrapping_add(1);
            }
        }
        frame_buffer.mark_frame_ready();
    }
}

impl Default for Renderer {
    fn default() -> Self {
        Self::new()
    }
}
