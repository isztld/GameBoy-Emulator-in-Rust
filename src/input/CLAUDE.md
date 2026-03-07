# src/input/ — Input handling

## Module map
| File | Purpose |
|------|---------|
| `mod.rs` | Re-exports `Joypad` |
| `joypad.rs` | `Button` enum, `Joypad` struct (stub) |

## Current status: Joypad struct is dead code
Input is **actually** implemented on `MemoryBus` directly:
- `MemoryBus::joypad_action` — bitmask of pressed action buttons (A/B/Select/Start)
- `MemoryBus::joypad_dpad` — bitmask of pressed D-pad buttons (Right/Left/Up/Down)
- `MemoryBus::update_joypad_io()` — encodes action/dpad into `io[0x00]` (P1/JOYP) respecting the select bits

`System::press_button` / `System::release_button` write to these `MemoryBus` fields.

The `Joypad` struct in `joypad.rs` has stub `press()` / `release()` methods that do nothing and a `get_input()` that always returns `0x0F`. It is instantiated on `System` but never called.

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

## Refactoring opportunities
1. **Delete or implement `Joypad`** — the struct is unused. Either delete it entirely (keeping only the `Button` enum), or implement it properly and route `System::press_button` through it instead of directly into `MemoryBus`.
2. **Joypad interrupt** — implement IF bit 4 set in `update_joypad_io()` when a new button press is detected (compare old vs new P1 value).
3. **`Joypad::get_input()` always returns `0x0F`** — misleading stub. If the struct is kept, this should actually read the current button state.
