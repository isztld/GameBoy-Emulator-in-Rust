# src/audio/ — Audio Processing Unit

## Module map
| File | Purpose |
|------|---------|
| `mod.rs` | Re-exports `AudioProcessor`, channel types |
| `apu.rs` | `AudioProcessor`, `AudioChannel` trait, `AudioOutput` |
| `channels.rs` | `SquareChannel`, `WaveChannel`, `NoiseChannel`, `Channel` enum |

## Current status: skeleton / not implemented
The APU is structurally present but produces no audio:
- `AudioProcessor::enabled` starts `false`; `get_output` returns silence until enabled.
- Channel `clock()` methods return `false` (no output change) and produce no waveform.
- No I/O register writes are routed to the APU — NR10-NR52 are not decoded.
- `wave_pattern` buffer exists but is never read.

## AudioChannel trait (apu.rs)
```rust
pub trait AudioChannel {
    fn clock(&mut self) -> bool; // true if output changed
    fn get_output(&self) -> f32;
    fn is_enabled(&self) -> bool;
    fn reset(&mut self);
}
```
All four channels are `Box<dyn AudioChannel>` stored in `AudioProcessor::channels`.

## AudioOutput (apu.rs)
`AudioOutput { left: f32, right: f32 }` — stereo mix, normalised to `[0.0, 1.0]`. `NR51` (output_select) controls which channels go to which side; this logic is stubbed.

## Channel types (channels.rs)
- `SquareChannel` — fields: `enabled`, `frequency`, `duty_cycle`, `volume`. No timer, no envelope, no sweep.
- `WaveChannel` — fields: `enabled`, `frequency`, `volume`, `position`. No wave RAM read.
- `NoiseChannel` — fields: `enabled`, `volume`, `lfsr`. No LFSR stepping.
- `Channel` enum (`Square1`, `Square2`, `Wave`, `Noise`) — used only for identification; not currently referenced.

## What needs implementing
Priority order for a functional APU:
1. Route `MemoryBus::write_io` for 0xFF10-0xFF3F into the APU.
2. Implement the frame sequencer (512 Hz clock derived from the timer).
3. Implement SquareChannel with frequency timer, duty table, and volume envelope.
4. Implement NoiseChannel with LFSR and envelope.
5. Implement WaveChannel reading from wave RAM.
6. Implement NR50/NR51 mixing and master enable (NR52).
7. Integrate with an audio output backend (e.g., `cpal`).

## Refactoring opportunities
1. `AudioProcessor` stores `wave_pattern: [u8; 32]` but it is never read or written. It belongs on `WaveChannel`.
2. `SquareChannel` duplicates the `is_enabled()` method in both the `AudioChannel` impl and as a direct method — pick one.
3. `Channel` enum is unused. Remove or use it to index into `AudioProcessor::channels`.
4. `AudioProcessor::clock` is called at "2x CPU frequency (8388 Hz)" per the comment, but the actual call site in `System::step::tick` fires once per M-cycle (1 MHz). The comment is wrong.
