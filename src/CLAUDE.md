# src/ — Library root

## Files
| File | Purpose |
|------|---------|
| `lib.rs` | Crate root; re-exports public API from all modules |
| `main.rs` | CLI binary — flag parsing, ROM loading, headless loop, `--cpu-json-test`, `--disasm` |
| `system.rs` | `System` struct — integrates CPU + MMU + PPU + APU + Timer + Joypad |
| `config.rs` | `EmulatorFlags` — configuration passed from CLI to `System::new` |
| `disasm.rs` | SM83 disassembler; standalone, does not require CPU state |

## system.rs — critical integration points
- `System::step` is the single-cycle entrypoint. Call order matters:
  1. `cpu.execute(mmu, &mut tick)` — runs one instruction; tick drives Timer and PPU per M-cycle
  2. `mmu.advance_dma` — decrements OAM DMA window counter
  3. `ppu.handle_oam_dma` — copies data during DMA
  4. `ppu.render_scanline` if `scanline_ready`
  5. `mmu.update_ly` / `update_ppu_stat` — sync PPU state to I/O
  6. Drain timer register writes (`timer_div_reset`, etc.)
  7. Check `ppu.vblank_entered` → set `frame_complete`

- `System::press_button` / `release_button` write directly to `mmu.joypad_action` / `mmu.joypad_dpad` and call `mmu.update_joypad_io()`. The `Joypad` struct field on `System` is unused.

## config.rs
`EmulatorFlags` is `Clone`; all fields have sensible defaults. If adding a new flag, add it here and parse it in `main.rs::parse_flags`.

## disasm.rs
- Implements `MemoryRead` trait on `MemoryBus`. The disassembler calls `bus.read(addr)` for opcode bytes.
- `disasm_one` / `disasm_region` are self-contained and safe to call without CPU state.

