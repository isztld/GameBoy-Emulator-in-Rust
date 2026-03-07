# GameBoy (DMG-01) Emulator — Project Root

## Overview
A GameBoy (DMG-01) emulator written in Rust targeting macOS. Despite the repo name (`osx_rust_gba_emu`), this emulates the original DMG GameBoy, not the GBA. The SM83 CPU, full memory map, PPU, APU skeleton, Timer, and Joypad are all modelled.

## Crate layout
- **Library crate** (`src/lib.rs`) — re-exports everything; used by the `lcd_display` binary.
- **Binary `src/main.rs`** — CLI runner: ROM loading, CPU JSON test mode, disassembler mode, headless loop.
- **Binary `src/bin/lcd_display.rs`** — full wgpu + Dear ImGui display window; the primary way to run ROMs interactively.

## Build & run
```sh
cargo run --bin gb_emu -- path/to/rom.gb          # headless (no display)
cargo run --bin lcd_display -- path/to/rom.gb     # windowed display
cargo run --bin gb_emu -- --cpu-json-test path/to/GameboyCPUTests/
cargo run --bin gb_emu -- --disasm path/to/rom.gb
```

## Key design decisions
- **Tick closure** — `CPU::execute` accepts `tick: &mut dyn FnMut(&mut [u8; 128])` called once per M-cycle. Timer and PPU advance via the shared I/O slice (`bus.io`) without needing a full bus borrow inside the closure.
- **Timer authority** — `Timer` owns all timer state. `MemoryBus` queues writes (`timer_div_reset`, `timer_tma_write`, etc.) which `System::step` drains after each instruction.
- **Split borrows** — `System::step` destructures `self` to allow independent borrows of `cpu`, `mmu`, `ppu`, `timer`, and `apu` simultaneously.
- **Scanline rendering** — PPU sets `scanline_ready` on HBlank entry; `System::step` calls `render_scanline` then clears the flag.

## Known issues / refactoring opportunities
1. **Duplicated module declarations** — `main.rs` and `lib.rs` both declare all modules (`pub mod cpu; pub mod display;` etc.). Only `lib.rs` should declare them; `main.rs` should use the library crate.
2. **Global serial log static** — `MemoryBus::SERIAL_LOG_FILE` is a `static Mutex` (noted as known limitation in `system.rs`). Should be an instance field on `MemoryBus`.
3. **`Joypad` struct is dead code** — button state is managed directly on `MemoryBus::joypad_action`/`joypad_dpad`. The `Joypad` struct in `input/joypad.rs` has stub implementations and is instantiated in `System` but never used.
4. **Audio not implemented** — `AudioProcessor::enabled` starts `false`; channels produce no output. The APU clockin `system.step` advances nothing meaningful.
5. **`disasm::MemoryBus` is deprecated** — a thin ROM wrapper that should be removed; callers should use `GameBoyMemoryBus` directly.
