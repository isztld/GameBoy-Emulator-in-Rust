# src/memory/ — Memory Management

## Module map
| File | Purpose |
|------|---------|
| `mod.rs` | Re-exports `MemoryBus`, `MemoryBankController`, `MbcType`, `MbcConfig` |
| `mmu.rs` | `MemoryBus` — full 64 KiB address space implementation |
| `mbc.rs` | `MemoryBankController` — MBC1/2/3/5/HuC1/HuC3 bank switching |

## MemoryBus (mmu.rs)
### Memory map
```
0000-3FFF  ROM bank 0 (always mapped)
4000-7FFF  ROM bank N (switched by MBC)
8000-9FFF  VRAM (bus.vram)
A000-BFFF  External/cartridge RAM (bus.external_ram)
C000-DFFF  WRAM (bus.wram, 8 KiB)
E000-FDFF  Echo RAM → mirrors C000-DDFF
FE00-FE9F  OAM (bus.oam)
FEA0-FEFF  Unusable (returns 0xFF on read)
FF00-FF7F  I/O registers (bus.io, index = addr & 0x7F)
FF80-FFFE  HRAM (bus.hram)
FFFF       IE register (bus.ie)
```

### flat_mode
When `flat_mode = true`, all reads/writes address `bus.rom` as a 64 KiB flat array, bypassing all memory-mapped regions. Used exclusively by the CPU JSON test harness. Normal emulation must never set this.

### I/O register conventions
- `bus.io` is a `[u8; 128]` array. Index = `address & 0x7F`.
- Timer registers: written via `write_io` which sets `timer_div_reset` / `timer_tma_write` / `timer_tac_write` flags. `System::step` drains these into `Timer` after each instruction.
- PPU registers (LCDC, STAT, LY, SCY, SCX, WY, WX, DMA, BGP, OBP0, OBP1): written to `bus.io`; PPU reads them back via `mmu.read`.
- Joypad: `update_joypad_io()` encodes `joypad_action`/`joypad_dpad` into `io[0x00]`.
- Serial (0xFF01/0xFF02): `write_io` at 0xFF01 calls `MemoryBus::write_serial_byte` when bit 7 of SC (0xFF02) is set.

### OAM DMA
- Writing 0xFF46 triggers DMA: the bus immediately copies 160 bytes from `src<<8` into `oam`. `oam_dma_cycles_remaining` is set to 160.
- `advance_dma(n)` decrements the counter; while non-zero, CPU reads outside HRAM return 0xFF.
- `ppu.handle_oam_dma` is called once per step but the copy is already complete.

### Global serial log file — known design issue
`SERIAL_LOG_FILE` is a `static Mutex<Option<...>>`. This means only one `MemoryBus` instance can log serial output at a time. Refactoring to an instance field requires plumbing the file through more of the codebase.

## MemoryBankController (mbc.rs)
- Supported: None (ROM-only), MBC1, MBC2, MBC3, MBC5, MBC6, MBC7, HuC1, HuC3.
- MBC6/7/HuC3 detection is stubbed — banking logic is not implemented for them.
- MBC3 RTC registers are not implemented.
- MBC5 uses a 9-bit bank number (`mbc5_rom_bank_low` + `mbc5_rom_bank_high`).

## Refactoring opportunities
1. **Serial log should be instance state** — move `SERIAL_LOG_FILE` static into `MemoryBus` as `serial_log_file: Option<Arc<Mutex<File>>>`.
2. **Timer register writes are double-buffered unnecessarily** — `timer_tima_write` is stored on the bus but then immediately discarded in `System::step` (comment says TIMA is synced live via `io[0x05]`). Remove `timer_tima_write` field.
3. **MBC6/7/HuC3 stubs** — identified in header parsing but fall back to no-banking. Document or implement.
4. **`oam_dma_active` / `oam_dma_address` on VideoController** — partially duplicates DMA state held on `MemoryBus`. Consolidate ownership.
