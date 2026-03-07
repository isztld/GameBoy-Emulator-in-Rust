# src/cpu/ — SM83 CPU

## Module map
| File | Purpose |
|------|---------|
| `mod.rs` | Public re-exports |
| `registers.rs` | `Flags`, `Registers`, `CPUState` |
| `instructions.rs` | `Instruction` enum and operand enums |
| `decode.rs` | Opcode → `Instruction` decoder |
| `cpu.rs` | `CPU` struct — interrupt handling, fetch-decode-execute loop |
| `exec.rs` | Dispatcher: `Instruction` → exec submodule functions |
| `exec/` | Per-group execution helpers (see exec/CLAUDE.md) |
| `testing.rs` | JSON CPU test infrastructure |
| `cycle_validation.rs` | Cycle-count regression tests |

## CPU lifecycle
`CPU::execute(bus, tick)` does one of:
1. **Interrupt service** — if `IME && (IE & IF & 0x1F) != 0`: push PC, jump to vector, consume 5 M-cycles, clear IF bit.
2. **Promote `ime_pending`** — EI sets `ime_pending`; on the cycle after EI, `ime` becomes true.
3. **Spin (HALT/STOP)** — calls `tick` once, returns 1 M-cycle. Timer/PPU still advance.
4. **Fetch-decode-execute** — reads opcode at PC+1 (PC already advanced by `decode`), calls `execute_instruction`.

Interrupt vectors: VBlank=0x40, STAT=0x48, Timer=0x50, Serial=0x58, Joypad=0x60.

## Registers / CPUState
- `Registers` stores `af`, `bc`, `de`, `hl`, `sp`, `pc` as `u16` pairs.
- `Flags` is a `u8` newtype; lower nibble is always zero (enforced by `Flags::set`).
- `CPUState` wraps `Registers` + `ime`/`ime_pending`/`halted`/`stopped`.
- **Soundness issue**: `Registers::f_mut()` uses an `unsafe` raw-pointer cast to return `&mut Flags`. This is UB because `Flags` is not laid out at the `f` byte offset within `Registers`. Prefer a method that mutates `self.af` in-place, or add a real `f: Flags` field separate from `af`.
- `halted`/`stopped` exist on both `CPU` (the real state) and `CPUState` (snapshot). Only `CPU`'s fields drive behaviour; `CPUState::halted`/`stopped` are stale and misleading — consider removing them.

## decode.rs
- Returns `(Instruction, len_in_bytes)`. Length is used by `System::step` for the CPU log only (the decoder already advanced PC internally).
- CB prefix: the sub-opcode byte is fetched inside `decode_instruction` at `pc+1`; the returned length is 2.
- `_cpu_state` parameter is unused — kept for future use (e.g., HALT bug detection).

## instructions.rs
- `Instruction` is `Copy` — keep it that way; executors pass it by value.
- `InstructionFormat` enum is defined but **never used** anywhere. It is dead code and can be deleted.
- `R16Stk` and `R16Register` overlap (same registers, different context). They are kept separate because `PUSH/POP` use `AF` while most other instructions use `SP`.

## testing.rs
- Loads JSON files from `GameboyCPUTests/` (one JSON array per opcode file).
- Uses `MemoryBus::new` in `flat_mode = true` so test RAM is fully writable.
- `run_test_case` applies initial state, executes one instruction, then compares all registers and RAM bytes to `final_state`.

## cycle_validation.rs
- Contains `#[cfg(test)]` only — not compiled in release builds.
- Authoritative cycle tables from `gb-test-roms instr_timing.s`. Tests every non-illegal opcode.
- Conditional instructions are tested with the condition *not taken* (minimum cycle count).

## Refactoring opportunities
1. Remove `InstructionFormat` enum (dead code).
2. Fix `Registers::f_mut()` unsafe cast — unsound.
3. Remove `CPUState::halted` and `CPUState::stopped` — they are never kept in sync with `CPU::halted`/`CPU::stopped`.
4. `decode_instruction` takes `_cpu_state` but ignores it — remove the parameter or use it for HALT-bug emulation.
