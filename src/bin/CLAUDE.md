# src/bin/ — Binary crates

## lcd_display.rs — primary display binary
The interactive emulator window. Uses **wgpu** (Metal backend on macOS) + **Dear ImGui** (via `imgui-wgpu` + `imgui-winit-support`).

### Architecture
```
EventLoop (winit)
  └─ AppHandler
       ├─ AppWindow  — wgpu device, queue, surface, swap chain
       ├─ ImguiState — imgui context, wgpu renderer, GB texture ID
       └─ EmulatorState — System, SharedFrameBuffer, FPS tracking
```

### Frame pacing
- `GB_FRAME_DURATION = 16_742_706 ns` (~16.74 ms, 59.73 Hz)
- `EmulatorState::last_emu_advance` tracks when the last emulator frame was advanced.
- Each `WindowEvent::RedrawRequested` checks elapsed time; if `>= GB_FRAME_DURATION`, calls `system.run_frame()` then uploads the frame buffer to a wgpu texture.

### Rendering pipeline
1. `system.run_frame()` runs the emulator until VBlank (`system.frame_complete`).
2. Frame buffer pixels (`[u32; 23040]`) are uploaded to `gb_texture` via `wgpu::Queue::write_texture`.
3. ImGui renders the GB texture in a window (`"GameBoy"`) and an info panel (`"Info"`) showing FPS, frame count, and CPU state.

### Key mapping (keyboard → Button)
| Key | Button |
|-----|--------|
| Z | A |
| X | B |
| Enter | Start |
| Backspace | Select |
| Arrow keys | D-pad |
| Escape | Quit |

### Window layout
- GB screen: `160 * 3 = 480` × `144 * 3 = 432` pixels (scale factor `SCALE = 3`)
- Info panel: `INFO_PANEL_WIDTH = 220` px wide, same height as GB window
- Title bar height: `TITLE_BAR_H = 19` px (ImGui default)
- Total window: `(480 + 220) × (432 + 19)` logical pixels

### Running
```sh
cargo run --bin lcd_display -- path/to/rom.gb
cargo run --bin lcd_display -- --cpu-log path/to/rom.gb
```
All `EmulatorFlags` from `src/config.rs` are supported (same flag parsing as the `gb_emu` binary).

## Refactoring opportunities
1. **Flag parsing duplicated** — both `main.rs` and `lcd_display.rs` have separate `parse_flags()` functions. Extract into a shared helper in `config.rs` or as a library function.
2. **`AppHandler` is split across `resumed` / `window_event`** — the wgpu `Surface` is created in `resumed` but used in `window_event`. The `AppWindow` struct could be restructured to hold an `Option<Surface>` more cleanly.
3. **Frame buffer upload allocates on every frame** — `wgpu::Queue::write_texture` is called with a raw byte slice derived from `pixels`. Pre-allocating a `Vec<u8>` scratch buffer and reusing it would avoid repeated allocation.
4. **CPU info panel reads registers after `run_frame()`** — registers show the state *after* the frame, which may be in the middle of an instruction. This is fine for debugging but labelling it as "end-of-frame state" would be clearer.
