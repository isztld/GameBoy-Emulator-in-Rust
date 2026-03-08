# src/input/ — Input handling

## Module map
| File | Purpose |
|------|---------|
| `mod.rs` | Re-exports `Button` |
| `joypad.rs` | `Button` enum |

## How input works
Input is implemented on `MemoryBus` directly:
- `MemoryBus::joypad_action` — bitmask of pressed action buttons (A/B/Select/Start)
- `MemoryBus::joypad_dpad` — bitmask of pressed D-pad buttons (Right/Left/Up/Down)
- `MemoryBus::update_joypad_io()` — encodes action/dpad into `io[0x00]` (P1/JOYP) respecting the select bits

`System::press_button` / `System::release_button` write to these `MemoryBus` fields.

## Button enum (joypad.rs)
```rust
pub enum Button { A, B, Select, Start, Right, Left, Up, Down }
```
This enum is the public API used by `System::press_button` / `System::release_button` and by `lcd_display.rs` for key mapping.

## P1/JOYP register format (0xFF00)
```
Bit 5: Select action buttons (0=selected)
Bit 4: Select D-pad buttons (0=selected)
Bit 3: Down / Start  (0=pressed)
Bit 2: Up   / Select (0=pressed)
Bit 1: Left / B      (0=pressed)
Bit 0: Right/ A      (0=pressed)
```
Note: bits are **active-low** — 0 means pressed.

## Joypad interrupt
When a button is pressed (transition from released to pressed), IF bit 4 should be set to request a joypad interrupt. This is **not currently implemented** — `update_joypad_io()` updates the P1 register but does not set the interrupt flag.

## Known limitations
- **Joypad interrupt** — IF bit 4 is not set in `update_joypad_io()` on new button presses. This could be implemented by comparing the old vs new P1 value and setting `io[0x0F] |= 0x10` on a falling edge.
