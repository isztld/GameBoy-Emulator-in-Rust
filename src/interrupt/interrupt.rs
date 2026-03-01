/// Interrupt controller for GameBoy
///
/// Handles interrupt enable (IE), interrupt flags (IF),
/// and interrupt priority.

/// Interrupt flags register
/// Bit 0: V-Blank (0x40)
/// Bit 1: LCD STAT (0x48)
/// Bit 2: Timer (0x50)
/// Bit 3: Serial (0x58)
/// Bit 4: Joypad (0x60)

#[derive(Debug, Clone, Copy)]
pub struct InterruptFlags(u8);

impl InterruptFlags {
    pub fn new() -> Self {
        InterruptFlags(0)
    }

    pub fn vblank(&self) -> bool {
        self.0 & 0x01 != 0
    }

    pub fn set_vblank(&mut self, value: bool) {
        if value {
            self.0 |= 0x01;
        } else {
            self.0 &= !0x01;
        }
    }

    pub fn lcd_stat(&self) -> bool {
        self.0 & 0x02 != 0
    }

    pub fn set_lcd_stat(&mut self, value: bool) {
        if value {
            self.0 |= 0x02;
        } else {
            self.0 &= !0x02;
        }
    }

    pub fn timer(&self) -> bool {
        self.0 & 0x04 != 0
    }

    pub fn set_timer(&mut self, value: bool) {
        if value {
            self.0 |= 0x04;
        } else {
            self.0 &= !0x04;
        }
    }

    pub fn serial(&self) -> bool {
        self.0 & 0x08 != 0
    }

    pub fn set_serial(&mut self, value: bool) {
        if value {
            self.0 |= 0x08;
        } else {
            self.0 &= !0x08;
        }
    }

    pub fn joypad(&self) -> bool {
        self.0 & 0x10 != 0
    }

    pub fn set_joypad(&mut self, value: bool) {
        if value {
            self.0 |= 0x10;
        } else {
            self.0 &= !0x10;
        }
    }

    pub fn get(&self) -> u8 {
        self.0
    }

    pub fn set(&mut self, value: u8) {
        self.0 = value & 0x1F; // Only bits 0-4 are used
    }

    pub fn or(&mut self, value: u8) {
        self.0 |= value & 0x1F;
    }
}

impl Default for InterruptFlags {
    fn default() -> Self {
        Self::new()
    }
}

/// Interrupt controller
#[derive(Debug)]
pub struct InterruptController {
    pub ie: u8,        // Interrupt enable
    pub if_flags: InterruptFlags, // Interrupt flags
    pub ime: bool,     // Interrupt master enable
}

impl InterruptController {
    pub fn new() -> Self {
        InterruptController {
            ie: 0x00,
            if_flags: InterruptFlags::new(),
            ime: false,
        }
    }

    /// Check if any interrupts are enabled and pending
    pub fn has_pending(&self) -> bool {
        if !self.ime {
            return false;
        }
        let enabled = self.ie & 0x1F;
        let pending = self.if_flags.get() & enabled;
        pending != 0
    }

    /// Get the highest priority pending interrupt vector
    /// Returns the interrupt vector address or None
    pub fn get_pending_vector(&self) -> Option<u16> {
        if !self.ime {
            return None;
        }

        let enabled = self.ie & 0x1F;
        let pending = self.if_flags.get() & enabled;

        if pending == 0 {
            return None;
        }

        // Return the highest priority pending interrupt
        // Priority order (highest to lowest): VBlank, LCD STAT, Timer, Serial, Joypad
        if pending & 0x01 != 0 {
            Some(0x40) // V-Blank
        } else if pending & 0x02 != 0 {
            Some(0x48) // LCD STAT
        } else if pending & 0x04 != 0 {
            Some(0x50) // Timer
        } else if pending & 0x08 != 0 {
            Some(0x58) // Serial
        } else if pending & 0x10 != 0 {
            Some(0x60) // Joypad
        } else {
            None
        }
    }

    /// Acknowledge an interrupt (clear flag and disable IE)
    pub fn acknowledge(&mut self, vector: u16) {
        let bit = match vector {
            0x40 => 0x01, // V-Blank
            0x48 => 0x02, // LCD STAT
            0x50 => 0x04, // Timer
            0x58 => 0x08, // Serial
            0x60 => 0x10, // Joypad
            _ => return,
        };
        self.if_flags.set(self.if_flags.get() & !bit);
    }

    /// Enable/disable interrupts
    pub fn enable(&mut self) {
        self.ime = true;
    }

    pub fn disable(&mut self) {
        self.ime = false;
    }

    /// Write to IE register
    pub fn write_ie(&mut self, value: u8) {
        self.ie = value & 0x1F;
    }

    /// Write to IF register
    pub fn write_if(&mut self, value: u8) {
        self.if_flags.set(value & 0x1F);
    }

    /// Set an interrupt flag
    pub fn set_flag(&mut self, flag: u8) {
        self.if_flags.or(flag & 0x1F);
    }

    /// Get IE register
    pub fn get_ie(&self) -> u8 {
        self.ie
    }

    /// Get IF register
    pub fn get_if(&self) -> u8 {
        self.if_flags.get()
    }
}

impl Default for InterruptController {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interrupt_flags() {
        let mut flags = InterruptFlags::new();
        assert!(!flags.vblank());
        assert!(!flags.timer());

        flags.set_vblank(true);
        assert!(flags.vblank());
        assert_eq!(flags.get(), 0x01);

        flags.set_timer(true);
        assert!(flags.timer());
        assert_eq!(flags.get(), 0x05);
    }

    #[test]
    fn test_pending_interrupts() {
        let mut ic = InterruptController::new();
        ic.ie = 0x05; // Enable VBlank and Timer
        ic.if_flags.set(0x05); // Set both flags

        assert!(ic.has_pending());

        let vector = ic.get_pending_vector();
        assert_eq!(vector, Some(0x40)); // VBlank has priority
    }
}
