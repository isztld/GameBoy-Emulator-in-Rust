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

**Typo**: `is_pallete_number()` — misspelled, should be `is_palette_number()`.

OAM data lives in `MemoryBus::oam`; `VideoController::render_scanline` reads it directly from there.

## Renderer (rendering.rs)
- `tiles: [Tile; 384]` — cached tile data (not currently kept in sync with VRAM writes; rendering reads VRAM directly from `MemoryBus`).
- `decode_tile_row` and `decode_bitplanes` do the same thing — code duplication. `decode_tile_row` is used in old code paths; `decode_bitplanes` is the newer inline version. One should be removed.
- Palette buffers `bg_palette`, `obj_palette_0`, `obj_palette_1` are `[u8; 4]` shade arrays (not raw register values).
- `apply_palette(palette_reg, idx)` maps a 2-bit colour index through a GB palette byte.
- `render_bg_scanline`, `render_window_scanline`, `render_sprites` are called in order by `render_scanline` in `video.rs`.

## Refactoring opportunities
1. **`decode_tile_row` duplication** — remove `decode_tile_row` (only used in dead/old code) and keep `decode_bitplanes`. Or rename `decode_bitplanes` to `decode_tile_row` and delete the original.
2. **`tiles` array is stale** — the `Renderer::tiles` array is never written from VRAM. Rendering reads VRAM directly from `MemoryBus`. Either keep the array as a proper cache (updated on VRAM writes) or remove it entirely.
3. **OAM typo** — `is_pallete_number` → `is_palette_number`.
4. **`oam_dma_active` / `oam_dma_address` on `VideoController`** — DMA is actually managed by `MemoryBus`. These fields on the PPU side appear unused or redundant.
5. **`Lcdc` bit accessor names** — `bg_tile_map_display()` (bit 0) vs `tile_map_select()` (bit 3) vs `tile_data_select()` (bit 4) naming is inconsistent with the Pan Docs convention. Consider renaming to match standard naming.
