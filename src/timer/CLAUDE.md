# src/timer/ — Timer hardware

## Module map
| File | Purpose |
|------|---------|
| `mod.rs` | Re-exports `Timer` |
| `timer.rs` | `Timer`, `TAC` — DIV and TIMA implementation |

## Timer registers
| Address | Name | Description |
|---------|------|-------------|
| FF04 | DIV | Divider — increments at 16384 Hz (every 64 M-cycles) |
| FF05 | TIMA | Timer counter — increments at rate set by TAC |
| FF06 | TMA | Timer modulo — reloaded into TIMA on overflow |
| FF07 | TAC | Timer control — enable bit + clock select |

## Tick model
`Timer::tick(io: &mut [u8; 128])` is called once per M-cycle (from the tick closure in `System::step`). It:
1. Increments `div_counter`; when it hits `DIV_PERIOD` (64), increments `div` and reflects it to `io[0x04]`.
2. If `tac.enabled`: increments `tima_counter`; when it hits `tac.tima_period()`, increments `tima`. If `tima` overflows: loads `tma`, sets `io[0x05] = tma`, requests timer interrupt via `io[0x0F] |= 0x04`.

## Write protocol
`MemoryBus::write_io` does **not** write timer registers directly into `Timer`; instead it sets flags on `MemoryBus`:
- `timer_div_reset = true` → `System::step` calls `timer.write_div()` which zeros `div` and `div_counter`.
- `timer_tma_write = Some(v)` → `timer.write_tma(v)`.
- `timer_tac_write = Some(v)` → `timer.write_tac(v)`.
- `timer_tima_write` — stored but immediately discarded by `System::step` (TIMA is synced live through `io[0x05]`).

## TAC
`TAC::tima_period()` returns M-cycle period per TIMA increment:
- CS=0: 256 M-cycles (4096 Hz)
- CS=1: 4 M-cycles (262144 Hz)
- CS=2: 16 M-cycles (65536 Hz)
- CS=3: 64 M-cycles (16384 Hz)

## Refactoring opportunities
1. **`timer_tima_write` field is dead** — it is set by `write_io` and then unconditionally discarded in `System::step` with `let _ = self.mmu.timer_tima_write.take()`. Remove the field.
2. **TIMA overflow delay** — the hardware has a 1-machine-cycle delay between TIMA overflow and TMA reload/interrupt. This is not modelled; it may cause off-by-one failures in some games.
3. **DIV obscure behaviour** — writing any value to DIV resets the internal 16-bit counter (not just the 8-bit register). The current model resets only the 8-bit `div` and its 6-bit `div_counter` (64-cycle period). This is close but not exact for all DIV edge cases.
