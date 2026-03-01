/// OAM (Object Attribute Memory) for GameBoy PPU
///
/// OAM stores sprite attributes: position, tile, and flags.

/// Sprite attribute entry (4 bytes)
#[derive(Debug, Clone, Copy)]
pub struct OamEntry {
    pub y: u8,        // Y position (0-159, 0x90 = sprite off-screen)
    pub x: u8,        // X position (0-167, 0x90 = sprite off-screen)
    pub tile: u8,     // Tile number (0-255)
    pub flags: u8,    // Flags (d, p, x, y)
}

impl OamEntry {
    pub fn new(y: u8, x: u8, tile: u8, flags: u8) -> Self {
        OamEntry { y, x, tile, flags }
    }

    pub fn from_bytes(bytes: [u8; 4]) -> Self {
        OamEntry {
            y: bytes[0],
            x: bytes[1],
            tile: bytes[2],
            flags: bytes[3],
        }
    }

    pub fn to_bytes(&self) -> [u8; 4] {
        [self.y, self.x, self.tile, self.flags]
    }

    // Flags bit definitions
    pub fn is_pallete_number(&self) -> bool {
        self.flags & 0x10 != 0
    }

    pub fn is_x_flip(&self) -> bool {
        self.flags & 0x20 != 0
    }

    pub fn is_y_flip(&self) -> bool {
        self.flags & 0x40 != 0
    }

    pub fn is_priority(&self) -> bool {
        self.flags & 0x80 != 0
    }
}

/// OAM structure (40 sprites x 4 bytes = 160 bytes)
#[derive(Debug)]
pub struct OAM {
    pub entries: [OamEntry; 40],
}

impl OAM {
    pub fn new() -> Self {
        OAM {
            entries: [OamEntry::new(0, 0, 0, 0); 40],
        }
    }

    /// Read from OAM entry
    pub fn read(&self, address: u16) -> u8 {
        let offset = (address - 0xFE00) as usize;
        let entry_idx = offset / 4;
        let byte_idx = offset % 4;

        match self.entries.get(entry_idx) {
            Some(entry) => {
                match byte_idx {
                    0 => entry.y,
                    1 => entry.x,
                    2 => entry.tile,
                    3 => entry.flags,
                    _ => 0xFF,
                }
            }
            None => 0xFF,
        }
    }

    /// Write to OAM entry
    pub fn write(&mut self, address: u16, value: u8) {
        let offset = (address - 0xFE00) as usize;
        let entry_idx = offset / 4;
        let byte_idx = offset % 4;

        if let Some(entry) = self.entries.get_mut(entry_idx) {
            match byte_idx {
                0 => entry.y = value,
                1 => entry.x = value,
                2 => entry.tile = value,
                3 => entry.flags = value,
                _ => {}
            }
        }
    }

    /// Clear all entries
    pub fn clear(&mut self) {
        for entry in &mut self.entries {
            *entry = OamEntry::new(0, 0, 0, 0);
        }
    }

    /// Get visible sprites for a given scanline
    /// Returns up to 10 visible sprites
    pub fn get_visible_sprites(&self, scanline_y: u8) -> Vec<&OamEntry> {
        let mut visible = Vec::new();

        for entry in &self.entries {
            // Sprite is on this scanline if:
            // - Y position <= scanline_y < Y position + height
            // - Sprite is not off-screen (X and Y > 0x90)
            let height = 8; // Default size, 16 for CGB
            let y = entry.y;
            if y < 0x90 && scanline_y >= y && scanline_y < y + height {
                visible.push(entry);
                if visible.len() >= 10 {
                    break; // Max 10 sprites per scanline
                }
            }
        }

        visible
    }
}

impl Default for OAM {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oam_entry_flags() {
        let entry = OamEntry::new(100, 100, 0, 0xFF);
        assert!(entry.is_pallete_number());
        assert!(entry.is_x_flip());
        assert!(entry.is_y_flip());
        assert!(entry.is_priority());
    }

    #[test]
    fn test_oam_read_write() {
        let mut oam = OAM::new();
        oam.write(0xFE00, 0x90); // Y position
        oam.write(0xFE01, 0x80); // X position
        oam.write(0xFE02, 0x00); // Tile
        oam.write(0xFE03, 0x00); // Flags

        assert_eq!(oam.read(0xFE00), 0x90);
        assert_eq!(oam.read(0xFE01), 0x80);
        assert_eq!(oam.read(0xFE02), 0x00);
        assert_eq!(oam.read(0xFE03), 0x00);
    }
}
