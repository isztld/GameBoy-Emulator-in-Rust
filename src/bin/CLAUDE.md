# src/bin/ — Binary crates

## lcd_display.rs — primary display binary
The interactive emulator window. Uses **wgpu** (Metal backend on macOS) + **Dear ImGui** (via `imgui-wgpu` + `imgui-winit-support`).

### Architecture
```
EventLoop (winit)
  └─ AppHandler
       ├─ AppWindow  — wgpu device, queue, surface, swap chain
       ├─ ImguiState — imgui context, wgpu renderer, GB texture ID
       └─ EmulatorState — System, SharedFrameBuffer, FPS tracking, audio stream + volume
```

### Frame pacing
- `GB_FRAME_DURATION = 16_742_706 ns` (~16.74 ms, 59.73 Hz)
- `EmulatorState::last_emu_advance` tracks when the last emulator frame was advanced.
- Each `WindowEvent::RedrawRequested` checks elapsed time; if `>= GB_FRAME_DURATION`, calls `system.run_frame()` then uploads the frame buffer to a wgpu texture.

### Rendering pipeline
1. `system.run_frame()` runs the emulator until VBlank (`system.frame_complete`).
2. Frame buffer pixels (`[u32; 23040]`) are uploaded to `gb_texture` via `wgpu::Queue::write_texture`.
3. ImGui renders the GB texture in a window (`"GameBoy"`) and an info panel (`"Info"`) showing FPS, frame count, audio controls, and emulator status.

### Audio pipeline
- `system.get_audio_buffer()` returns `Arc<Mutex<VecDeque<(f32,f32)>>>` filled by `AudioProcessor::clock()` at 44100 Hz.
- `setup_audio_stream(buffer, volume)` opens the default cpal output device and drains the buffer in its callback, applying the software volume multiplier.
- Software volume is an `Arc<AtomicU32>` holding f32 bits (lock-free read in the audio callback).
- `EmulatorState` holds `_audio_stream: Option<cpal::Stream>` (keeps the stream alive) and `volume: Arc<AtomicU32>`.

### Audio controls (Info panel)
- **Volume slider** — `##vol` slider (0.0 – 1.0), width fills the panel. Moving the slider automatically un-mutes.
- **Mute checkbox** — `Mute` checkbox. When muted, writes `0.0` to the atomic volume so the callback produces silence without losing the slider position.
- Both controls live in `EmulatorState::{volume, muted}`; the atomic is updated every frame after the ImGui closure.

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

