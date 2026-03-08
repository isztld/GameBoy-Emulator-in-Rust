# src/display/ — Display output

## Module map
| File | Purpose |
|------|---------|
| `mod.rs` | Re-exports public API |
| `frame_buffer.rs` | `FrameBuffer`, `SharedFrameBuffer`, screen constants |

## frame_buffer.rs
### Constants
- `SCREEN_WIDTH = 160`, `SCREEN_HEIGHT = 144` — DMG native resolution
- `FRAME_BUFFER_SIZE = 160 * 144 = 23040` pixels

### FrameBuffer
- `pixels: [u32; 23040]` — RGBA packed (0xAARRGGBB).
- `set_pixel(x, y, color: u8)` — maps GB shade 0-3 through `color_to_rgba` to an RGBA value.
- `mark_frame_ready` / `clear_frame_ready` — flag for the display loop.

### SharedFrameBuffer
```rust
pub type SharedFrameBuffer = Arc<Mutex<FrameBuffer>>;
pub fn create_shared_frame_buffer() -> SharedFrameBuffer;
```
The PPU holds a `SharedFrameBuffer` and writes scanlines into it. The display binary reads from it on every VBlank.

