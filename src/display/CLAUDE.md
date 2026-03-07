# src/display/ — Display output

## Module map
| File | Purpose |
|------|---------|
| `mod.rs` | Re-exports public API |
| `frame_buffer.rs` | `FrameBuffer`, `SharedFrameBuffer`, screen constants |
| `metal_renderer.rs` | `MetalRenderer` — Metal device init stub |

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

## metal_renderer.rs
A stub that creates a Metal `Device` and `CommandQueue` but does **not** render to screen. It is never used by the main rendering path (which uses `lcd_display.rs` / wgpu). It is kept as a reference for a future native Metal integration but is effectively dead code.

## Refactoring opportunities
1. **`MetalRenderer` is dead code** — the display is done via wgpu in `bin/lcd_display.rs`. Consider either implementing `MetalRenderer` properly or removing it.
2. **`color_to_rgba` is private** — the GB shade-to-RGBA mapping is useful for testing. Consider exposing it or making the palette configurable (e.g., allow custom colour palettes beyond the default 4-shade greens).
3. **`FrameBuffer::pixels` is a fixed-size array** — `[u32; FRAME_BUFFER_SIZE]` requires `FRAME_BUFFER_SIZE` to be a const. This is fine for DMG but would need restructuring to support CGB (same size) or other resolutions.
