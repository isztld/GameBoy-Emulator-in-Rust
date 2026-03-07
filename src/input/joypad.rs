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

