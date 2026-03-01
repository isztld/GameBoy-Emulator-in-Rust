/// Rendering module for GameBoy PPU
///
/// Handles tile rendering, window display, and sprite rendering.

use crate::memory::MemoryBus;
use crate::ppu::oam::OAM;
use crate::ppu::video::Lcdc;

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

    /// Render a background scanline
    /// Returns 160 pixel values for the scanline
    pub fn render_background(
        &self,
        bus: &MemoryBus,
        scanline_y: u8,
        lcdc: &Lcdc,
        scroll_x: u8,
        scroll_y: u8,
    ) -> [u8; 160] {
        let mut pixels = [0; 160];

        if !lcdc.is_enabled() {
            return pixels;
        }

        // Determine tile map base address
        let tile_map_base = if lcdc.tile_map_select() { 0x9C00 } else { 0x9800 };

        // Determine tile data base (used for signed tile indices)
        let _tile_data_base = if lcdc.tile_data_select() { 0x8000 } else { 0x9000 };

        let bg_y = (scanline_y as i32 + scroll_y as i32) as u32;
        let bg_x_start = scroll_x as i32;

        for screen_x in 0..160 {
            let x = (bg_x_start + screen_x as i32) as u32;
            let tile_x = (x / 8) % 32;
            let tile_y = (bg_y / 8) % 32;

            let tile_map_offset = tile_map_base as u32 + tile_y * 32 + tile_x;
            let tile_index = bus.read(tile_map_offset as u16) as i8;

            let tile_idx = if lcdc.tile_data_select() {
                // Signed tile indices for 8000-7FFF
                if tile_index >= 0 {
                    tile_index as u16
                } else {
                    256 + (tile_index as u16)
                }
            } else {
                tile_index as u16
            };

            let tile_row = (bg_y % 8) as usize;
            let tile = match self.get_tile(tile_idx as usize) {
                Some(t) => t,
                None => continue,
            };

            let pixel_row = self.decode_tile_row(tile, tile_row);
            let pixel_x = (x % 8) as usize;

            pixels[screen_x as usize] = pixel_row[pixel_x];
        }

        pixels
    }

    /// Render sprites for a scanline
    /// Returns sprite pixels and their positions
    pub fn render_sprites(
        &self,
        oam: &OAM,
        scanline_y: u8,
        lcdc: &Lcdc,
    ) -> Vec<(usize, u8)> {
        let mut sprites = Vec::new();
        let height = lcdc.obj_size();

        for entry in &oam.entries {
            let y = entry.y;
            let x = entry.x;

            // Check if sprite is on this scanline
            if x < 0x90 && y < 0x90 && scanline_y >= y && scanline_y < y + height as u8 {
                let tile_row = (scanline_y - y) as usize;
                let tile = self.get_tile(entry.tile as usize);

                if let Some(t) = tile {
                    let pixel_row = self.decode_tile_row(t, tile_row);

                    for i in 0..8 {
                        let pixel_x = x as usize + i;
                        if pixel_x < 160 {
                            let pixel_val = pixel_row[i];
                            if pixel_val != 0 {
                                sprites.push((pixel_x, pixel_val));
                            }
                        }
                    }
                }
            }
        }

        sprites
    }
}

impl Default for Renderer {
    fn default() -> Self {
        Self::new()
    }
}
