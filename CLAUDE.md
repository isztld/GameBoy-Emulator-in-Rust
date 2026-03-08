# GameBoy (DMG-01) Emulator — Project Root

## Overview
A GameBoy (DMG-01) emulator written in Rust targeting macOS. Despite the repo name (`osx_rust_gba_emu`), this emulates the original DMG GameBoy, not the GBA. The SM83 CPU, full memory map, PPU, APU, Timer, and Joypad are all modelled.

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

