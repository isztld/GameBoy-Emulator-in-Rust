# src/cpu/exec/ — Instruction executors

## Module map
| File | Instruction group |
|------|-------------------|
| `register_utils.rs` | `get_r8`, `set_r8`, `r16`, `set_r16` — shared helpers |
| `alu.rs` | ADD, ADC, SUB, SBC, AND, XOR, OR, CP (register and immediate forms) |
| `rotate_shift.rs` | RLCA, RRCA, RLA, RRA, DAA, CPL, SCF, CCF |
| `cb_instructions.rs` | All 0xCB-prefixed instructions: RLC, RRC, RL, RR, SLA, SRA, SWAP, SRL, BIT, RES, SET |
| `data_transfer.rs` | LD r8/r16, indirect LD/ST, LdIndImm16Sp |
| `stack.rs` | PUSH/POP, LDH, SP-relative loads |
| `jump_call.rs` | JR, JP, CALL, RET, RETI, RST |
| `registers.rs` | INC/DEC r8, INC/DEC r16, ADD HL,r16 |

## Calling convention
All exec functions have this signature:
```rust
fn exec_*(cpu_state: &mut CPUState, bus: &mut MemoryBus, ..., tick: &mut dyn FnMut(&mut [u8; 128])) -> u32
```
- Returns **M-cycle count** (not T-cycles).
- `tick` must be called exactly once per M-cycle *beyond the first* (the first M-cycle is the fetch, handled by `CPU::execute`). For most 1-cycle instructions, `tick` is never called from within the executor.
- For `(HL)` operands: call `tick` after the memory read, before the result is written back.

## register_utils.rs
- `get_r8(registers, bus, reg)` — reads from register or `bus.read(HL)`.
- `set_r8(registers, bus, reg, value)` — writes to register or `bus.write(HL, value)`.
- These are free functions (not methods) to avoid borrow conflicts when the caller holds other references.
- `set_r16` for `AF` masks the lower nibble to zero (`value & 0xFFF0`), matching hardware.

## alu.rs
- ALU ops that read `(HL)` check `reg == R8Register::HL` and emit one extra tick.
- Carry detection: ADD uses `result < a` (wrapping); multi-byte carries use widened arithmetic.
- Half-carry: `(a & 0xF) + (val & 0xF) > 0xF` for addition; `(a & 0xF) < (val & 0xF)` for subtraction.

## cb_instructions.rs
- `(HL)` variants cost 4 M-cycles (2 extra ticks: one for read, one for write-back).
- BIT instructions do **not** write back — they set Z/N/H flags only. Cost is 3 M-cycles for `(HL)`.
- SWAP sets all flags to zero except Z.

## jump_call.rs
- CALL: push PC (2 ticks for the two SP-1/SP-2 writes) then jump. Total 6 M-cycles.
- RET: pop PC (2 ticks for reads) + 1 tick delay = 4 M-cycles. RETI also re-enables IME.
- Conditional variants: on condition-not-taken, only the fetch cycle fires (2 M-cycles for JR/JP, 3 for CALL, 2 for RET).
- RST: behaves like CALL to a fixed vector.

