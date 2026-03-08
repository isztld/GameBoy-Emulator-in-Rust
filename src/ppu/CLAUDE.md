# src/ppu/ — Pixel Processing Unit

## Module map
| File | Purpose |
|------|---------|
| `mod.rs` | Re-exports `VideoController`, `OAM`, `Renderer` |
| `video.rs` | `VideoController` — PPU state machine, mode transitions, LCDC/STAT |
| `oam.rs` | `OamEntry`, `OAM` — sprite attribute parsing |
| `rendering.rs` | `Renderer` — tile decode, BG/Window/Sprite scanline rendering |

## PPU state machine (video.rs)
Four modes cycle per frame:

```
OamScan (80 M-cycles) → PixelTransfer (172 M-cycles) → HBlank (204 M-cycles)
                                                                   ↓ (repeat 144 times)
                         VBlank (4560 M-cycles, LY 144-153)       ↑
```

- `tick_io(io)` is called once per M-cycle from `System::step`'s tick closure. It advances `mode_clock` and handles mode transitions.
- On `PixelTransfer → HBlank`: sets `scanline_ready = true`. `System::step` then calls `render_scanline`.
- On `LY 143 → 144`: sets `vblank_entered = true` (edge-triggered). `System::step` sets `frame_complete` and clears the flag.
- LY 153 → 0 resets `window_line` to 0 for the next frame.

### STAT interrupt sources
Bit 3 (HBlank), bit 4 (VBlank), bit 5 (OAM), bit 6 (LYC=LY) of STAT can each trigger STAT interrupt (IF bit 1) on rising edge. Implemented in `tick_io`.

### window_line
An internal counter that increments once per scanline on which the window is actually drawn (not simply LY-WY). This is required because the window can be toggled or WY can change mid-frame.

## OAM (oam.rs)
`OamEntry` holds sprite Y, X, tile, and flags. Key flag bits:
- bit 7: BG-over-OBJ priority
- bit 6: Y-flip
- bit 5: X-flip
- bit 4: palette (OBP0 or OBP1)

OAM data lives in `MemoryBus::oam`; `VideoController::render_scanline` reads it directly from there.

## Renderer (rendering.rs)
- `decode_bitplanes(lsb, msb)` decodes two bitplane bytes into 8 pixel colour indices (0-3). All tile rendering reads VRAM directly from `MemoryBus`.
- Palette buffers `bg_palette`, `obj_palette_0`, `obj_palette_1` are `[u8; 4]` shade arrays (not raw register values).
- `apply_palette(palette_reg, idx)` maps a 2-bit colour index through a GB palette byte.
- `render_background`, `render_window`, `render_sprites` are called in order by `render_scanline` in `video.rs`.

