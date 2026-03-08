# src/audio/ — Audio Processing Unit

## Module map
| File | Purpose |
|------|---------|
| `mod.rs` | Re-exports `AudioProcessor` |
| `apu.rs` | `AudioProcessor`, `AudioChannel` trait, `AudioOutput` |
| `channels.rs` | `SquareChannel`, `WaveChannel`, `NoiseChannel` |

## Current status: functional skeleton
The APU is fully wired. Channels have correct period timers, frame sequencer, and register decoding. Audio is output via `cpal` in `lcd_display`. Games must write NR52 bit 7 to enable master audio before any sound is heard (correct DMG behaviour).

## AudioProcessor (apu.rs)
Concrete channel fields — no `Box<dyn AudioChannel>`:
```rust
pub struct AudioProcessor {
    pub ch1: SquareChannel,   // square + sweep
    pub ch2: SquareChannel,   // square
    pub ch3: WaveChannel,     // wave RAM
    pub ch4: NoiseChannel,    // LFSR noise
    pub enabled: bool,        // NR52 bit 7
    pub master_volume: u8,    // NR50
    pub panning: u8,          // NR51
    sample_timer: f32,
    pub audio_buffer: Arc<Mutex<VecDeque<(f32, f32)>>>,
    frame_seq_timer: u32,
    frame_seq_step: u8,
}
```

### clock() — called once per M-cycle from System::step tick closure
- Clocks all four channels (period timers).
- Advances frame sequencer every 8192 M-cycles (128 Hz):
  - Steps 0,2,4,6: length counters
  - Steps 2,6: CH1 frequency sweep
  - Step 7: volume envelopes
- Calls `accumulate_sample()` every ~23.77 M-cycles to push a 44100 Hz stereo sample.

### Sample buffer
`audio_buffer: Arc<Mutex<VecDeque<(f32,f32)>>>` — capped at 8192 entries (~0.18 s).
`System::get_audio_buffer()` clones the Arc for use by the display binary.

### NR register decoding (write_audio_register)
| Address | Register | Routed to |
|---------|----------|-----------|
| 0xFF10 | NR10 | `ch1.write_sweep` |
| 0xFF11 | NR11 | `ch1.write_duty_length` |
| 0xFF12 | NR12 | `ch1.write_envelope` |
| 0xFF13 | NR13 | `ch1.write_freq_lo` |
| 0xFF14 | NR14 | `ch1.write_freq_hi` (+ trigger) |
| 0xFF16 | NR21 | `ch2.write_duty_length` |
| 0xFF17 | NR22 | `ch2.write_envelope` |
| 0xFF18 | NR23 | `ch2.write_freq_lo` |
| 0xFF19 | NR24 | `ch2.write_freq_hi` (+ trigger) |
| 0xFF1A | NR30 | `ch3.write_dac_enable` |
| 0xFF1B | NR31 | `ch3.write_length` |
| 0xFF1C | NR32 | `ch3.write_volume` |
| 0xFF1D | NR33 | `ch3.write_freq_lo` |
| 0xFF1E | NR34 | `ch3.write_freq_hi` (+ trigger) |
| 0xFF20 | NR41 | `ch4.write_length` |
| 0xFF21 | NR42 | `ch4.write_envelope` |
| 0xFF22 | NR43 | `ch4.write_poly` |
| 0xFF23 | NR44 | `ch4.write_trigger` |
| 0xFF24 | NR50 | `master_volume` |
| 0xFF25 | NR51 | `panning` |
| 0xFF26 | NR52 | `enabled`; power-off resets all channels |
| 0xFF30–0xFF3F | Wave RAM | `ch3.write_wave_byte(index, val)` |

### Mixing (mix())
NR51 panning bits select which channels feed SO2 (left) and SO1 (right).
Each side sums up to 4 channel outputs (each 0.0–1.0), divides by 4, multiplies by SO vol/7.

## Channel types (channels.rs)

### SquareChannel (CH1 and CH2)
- Period timer: decrements each M-cycle; reloads `(2048 - frequency)` and advances duty position.
- Duty table: 4 patterns × 8 steps (`[0,0,0,0,0,0,0,1]`, `[1,0,0,0,0,0,0,1]`, `[1,0,0,0,1,1,1,1]`, `[0,1,1,1,1,1,1,0]`).
- Volume envelope: clocked at 64 Hz by frame sequencer step 7.
- Length counter: clocked at 256 Hz by frame sequencer steps 0,2,4,6.
- CH1 frequency sweep: clocked at 128 Hz by frame sequencer steps 2,6; overflow disables channel.

### WaveChannel (CH3)
- Period timer: reloads `(2048 - frequency) * 2`; advances 32-position wave RAM cursor.
- Wave RAM: 16 bytes (`pattern: [u8;16]`), each byte = 2 nibbles = 2 4-bit samples.
- Volume shift: 0=mute, 1=100%, 2=50%, 3=25%.

### NoiseChannel (CH4)
- 15-bit LFSR; optional 7-bit width mode (NR43 bit 3).
- Period timer: `NOISE_DIVISORS[divisor_code] << clock_shift` M-cycles.
  `NOISE_DIVISORS = [8,16,32,48,64,80,96,112]`.
- Volume envelope same as square channels.

## Known limitations
- No APU read-back (NR52 channel status bits always 0).
- No obscure power-on state for wave RAM.
