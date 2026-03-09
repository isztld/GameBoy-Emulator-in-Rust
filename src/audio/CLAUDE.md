# src/audio/ — Audio Processing Unit

## Module map
| File | Purpose |
|------|---------|
| `mod.rs` | Re-exports `AudioProcessor` |
| `apu.rs` | `AudioProcessor`, `AudioOutput` |
| `channels.rs` | `SquareChannel`, `WaveChannel`, `NoiseChannel` |

## AudioProcessor (apu.rs)
Concrete channel fields — no trait objects:
```rust
pub struct AudioProcessor {
    pub ch1: SquareChannel,   // square + sweep
    pub ch2: SquareChannel,   // square
    pub ch3: WaveChannel,     // wave RAM
    pub ch4: NoiseChannel,    // LFSR noise
    pub enabled: bool,        // NR52 bit 7
    pub master_volume: u8,    // NR50
    pub panning: u8,          // NR51
    sample_timer: f64,
    pub audio_buffer: Arc<Mutex<VecDeque<(f32, f32)>>>,
    frame_seq_timer: u32,
    frame_seq_step: u8,
    hp_prev_l/r, hp_out_l/r: f32,  // high-pass filter state
}
```

### clock() — called once per M-cycle from System::step tick closure
- Clocks all four channels (period timers).
- Frame sequencer: fires every 2048 M-cycles (512 Hz), 8 steps:
  - Steps 0,2,4,6: length counters (256 Hz)
  - Steps 2,6: CH1 frequency sweep (128 Hz)
  - Step 7: volume envelopes (64 Hz)
- Calls `accumulate_sample()` every ~23.77 M-cycles to push a 44100 Hz stereo sample.

### accumulate_sample() / mix()
- High-pass filter (α=0.998943) removes DC offset before pushing to buffer.
- `audio_buffer: Arc<Mutex<VecDeque<(f32,f32)>>>` capped at 8192 entries (~0.18 s).
- `System::get_audio_buffer()` clones the Arc for the `lcd_display` cpal callback.
- NR51 panning: SO2=left (bits 7-4), SO1=right (bits 3-0). Each side sums channels, divides by 4, scales by SO vol/7.

### NR52 read-back
`AudioProcessor::nr52_value()` returns `0x70 | (master<<7) | (ch4..ch1 enable bits)`.
`System::step` writes this to `mmu.io[0x26]` after each instruction so CPU reads are live.

### Power-off (NR52 bit 7 written 0)
All channel registers reset; **wave RAM is preserved** (DMG hardware behaviour).

### NR register decoding (write_audio_register)
Raw value is always forwarded to the APU. `MemoryBus::write_io` also mirrors a **masked**
read-back value into `bus.io[]` — write-only bits read as 1, open-bus bits read as 1.

| Address | Register | APU handler | Read mask |
|---------|----------|-------------|-----------|
| 0xFF10 | NR10 | `ch1.write_sweep` | `(val & 0x7F) \| 0x80` |
| 0xFF11 | NR11 | `ch1.write_duty_length` | `(val & 0xC0) \| 0x3F` |
| 0xFF12 | NR12 | `ch1.write_envelope` | `val` |
| 0xFF13 | NR13 | `ch1.write_freq_lo` | `0xFF` (write-only) |
| 0xFF14 | NR14 | `ch1.write_freq_hi` + trigger | `(val & 0x40) \| 0xBF` |
| 0xFF16 | NR21 | `ch2.write_duty_length` | `(val & 0xC0) \| 0x3F` |
| 0xFF17 | NR22 | `ch2.write_envelope` | `val` |
| 0xFF18 | NR23 | `ch2.write_freq_lo` | `0xFF` (write-only) |
| 0xFF19 | NR24 | `ch2.write_freq_hi` + trigger | `(val & 0x40) \| 0xBF` |
| 0xFF1A | NR30 | `ch3.write_dac_enable` | `(val & 0x80) \| 0x7F` |
| 0xFF1B | NR31 | `ch3.write_length` | `0xFF` (write-only) |
| 0xFF1C | NR32 | `ch3.write_volume` | `(val & 0x60) \| 0x9F` |
| 0xFF1D | NR33 | `ch3.write_freq_lo` | `0xFF` (write-only) |
| 0xFF1E | NR34 | `ch3.write_freq_hi` + trigger | `(val & 0x40) \| 0xBF` |
| 0xFF20 | NR41 | `ch4.write_length` | `0xFF` (write-only) |
| 0xFF21 | NR42 | `ch4.write_envelope` | `val` |
| 0xFF22 | NR43 | `ch4.write_poly` | `val` |
| 0xFF23 | NR44 | `ch4.write_trigger` | `(val & 0x40) \| 0xBF` |
| 0xFF24 | NR50 | `master_volume` | `val` |
| 0xFF25 | NR51 | `panning` | `val` |
| 0xFF26 | NR52 | `enabled`; power-off resets channels | `(val & 0x80) \| 0x70` (ch bits via `nr52_value()`) |
| 0xFF30–0xFF3F | Wave RAM | `ch3.write_wave_byte(index, val)` | `val` |

## Channel types (channels.rs)

### SquareChannel (CH1 and CH2)
- Period timer: reloads `(2048 - frequency)` M-cycles; advances 8-step duty position.
- Duty table: `[0,0,0,0,0,0,0,1]` / `[1,0,0,0,0,0,0,1]` / `[1,0,0,0,1,1,1,1]` / `[0,1,1,1,1,1,1,0]`.
- Volume envelope: initial volume + add/subtract direction + period (0=frozen).
- Length counter: 6-bit, disables channel when it expires (if length-enable set).
- CH1 sweep: shadow register, overflow check on trigger and on each sweep clock.

### WaveChannel (CH3)
- Period timer: reloads `(2048 - frequency) / 2` M-cycles; advances 32-nibble wave RAM cursor.
- Wave RAM: 16 bytes (`pattern: [u8;16]`), initialised to DMG boot-ROM power-on state.
  `[0x84,0x40,0x43,0xAA,0x2D,0x78,0x92,0x3C,0x60,0x59,0xAD,0xA1,0x0C,0xE2,0xF3,0x44]`
- Volume shift: 0=mute, 1=100%, 2=50%, 3=25% (right-shift of 4-bit sample).
- Length counter: 8-bit (256 steps).

### NoiseChannel (CH4)
- 15-bit LFSR; 7-bit width mode available (NR43 bit 3 sets bit 6 alongside bit 14).
- Period timer: `NOISE_DIVISORS[divisor_code] << clock_shift` M-cycles.
  `NOISE_DIVISORS = [2,4,8,12,16,20,24,28]` (hardware T-cycle table ÷ 4).
- Volume envelope: same as square channels.

## Known limitations
- No wave RAM access conflict when CH3 is active (CPU writes go directly to `pattern[]`).
- Length-counter extra-clock edge case on NRx4 write mid-frame-sequencer step not implemented.
- Sweep negate-used kill flag not tracked (obscure Blargg edge case).
