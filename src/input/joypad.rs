/// Joypad input handling
///
/// The GameBoy joypad has 8 buttons:
/// - A, B, Select, Start
/// - Right, Left, Up, Down

/// Button state
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Button {
    A,
    B,
    Select,
    Start,
    Right,
    Left,
    Up,
    Down,
}

/// Joypad input state
#[derive(Debug)]
pub struct Joypad {
    /// Bit 7: Unused
    /// Bit 6: Unused
    /// Bit 5: P15 - Button selection (0=selected)
    /// Bit 4: P14 - Direction selection (0=selected)
    /// Bit 3: P13 - Input data (0=pressed)
    /// Bit 2: P12 - Input data (0=pressed)
    /// Bit 1: P11 - Input data (0=pressed)
    /// Bit 0: P10 - Input data (0=pressed)
    pub state: u8,
}

impl Joypad {
    pub fn new() -> Self {
        Joypad { state: 0x00 }
    }

    /// Get the current input state
    /// Returns a byte where each bit represents a button
    /// Bit 0-3: Direction buttons (Down, Up, Left, Right)
    /// Bit 4-7: Action buttons (Start, Select, B, A)
    pub fn get_input(&self) -> u8 {
        // In DMG mode, returns $0F when all buttons released
        0x0F
    }

    /// Press a button
    pub fn press(&mut self, button: Button) {
        // This would set the appropriate bit
        // Implementation depends on how buttons are selected
        let _ = button;
    }

    /// Release a button
    pub fn release(&mut self, button: Button) {
        // This would clear the appropriate bit
        let _ = button;
    }

    /// Check if a button is pressed
    pub fn is_pressed(&self, button: Button) -> bool {
        // Returns true if button is pressed
        let _ = button;
        false
    }

    /// Write to joypad register
    /// Writing to $FF00 selects which buttons to monitor
    pub fn write(&mut self, value: u8) {
        self.state = value & 0x30; // Only bits 4-5 are used for selection
    }

    /// Read from joypad register
    /// Returns button states based on current selection
    pub fn read(&self) -> u8 {
        // Return $0F when no buttons pressed
        // The actual value depends on which buttons are selected
        0x0F
    }
}

impl Default for Joypad {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_joypad_default() {
        let joypad = Joypad::new();
        assert_eq!(joypad.state, 0x00);
    }

    #[test]
    fn test_joypad_write() {
        let mut joypad = Joypad::new();
        joypad.write(0x20); // Select direction buttons
        assert_eq!(joypad.state, 0x20);
    }
}
