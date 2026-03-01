# GameBoy Emulator in Rust

A GameBoy (DMG-01) emulator written in Rust, implementing the SM83 CPU, memory management, PPU, APU, and all standard peripherals.

## Features

- **CPU**: SM83 (GBZ80-compatible) instruction set
- **Memory**: Full memory map including VRAM, WRAM, OAM, I/O registers, and MBC support
- **PPU**: LCD controller with mode tracking (OamScan, PixelTransfer, HBlank, VBlank)
- **APU**: 4-channel audio processor (pulse, wave, noise, stereo)
- **Timer**: DIV, TIMA, TMA, TAC registers
- **Interrupts**: V-Blank, LCDC STAT, Timer, Serial IO, Joypad

## Building

```bash
cargo build --release
```

## Usage

```bash
cargo run --release -- <rom_file> [options]
```

## Command-Line Flags

| Flag | Description | Default |
|------|-------------|---------|
| `--cpu-log [file]` | Enable CPU instruction logging to the specified file | `cpu_log.txt` |
| `--serial-log [file]` | Enable serial output (console) logging to the specified file | `serial_log.txt` |
| `--help`, `-h` | Show help message and exit | - |

## Examples

Run a ROM with CPU logging enabled:

```bash
cargo run --release -- game.gb --cpu-log
```

Run with both CPU and serial output logging:

```bash
cargo run --release -- game.gb --cpu-log cpu.log --serial-log serial.log
```

Show help:

```bash
cargo run --release -- --help
```

## Logging Behavior

### CPU Instruction Log (`--cpu-log`)

When enabled, the emulator writes each executed instruction to the log file in the following format:

```
PC=$1234 A:$00 F:00 BC:$0013 DE:$00D8 HL:$014D SP:$FFFE CYCLES:4
```

Fields:
- `PC`: Program counter (16-bit hex)
- `A`: Accumulator (8-bit hex)
- `F`: Flags register (8-bit hex)
- `BC`, `DE`, `HL`: Register pairs (16-bit hex)
- `SP`: Stack pointer (16-bit hex)
- `CYCLES`: CPU cycles consumed by the instruction

### Serial Output Log (`--serial-log`)

When enabled, all GameBoy serial output (typically written to stdout via the SC register) is captured to this file instead of the terminal. This includes any character output from ROMs that use the serial interface for debug output.

## Memory Map

| Address Range | Size | Description |
|---------------|------|-------------|
| 0000-3FFF | 16 KiB | ROM Bank 0 |
| 4000-7FFF | 16 KiB | ROM Bank 1-NN (switchable via MBC) |
| 8000-9FFF | 8 KiB | Video RAM (VRAM) |
| A000-BFFF | 8 KiB | External RAM (from cartridge) |
| C000-CFFF | 4 KiB | Work RAM (WRAM) |
| D000-DFFF | 4 KiB | Work RAM (bankable on CGB) |
| E000-FDFF | 8 KiB | Echo RAM (mirror of C000-DDFF) |
| FE00-FE9F | 160 B | Object Attribute Memory (OAM) |
| FF00-FF7F | 128 B | I/O Registers |
| FF80-FFFE | 127 B | High RAM (HRAM) |
| FFFF | 1 B | Interrupt Enable (IE) |

## Project Structure

```
src/
├── audio/          # APU implementation
│   ├── apu.rs      # Audio processor
│   └── channels.rs # Audio channels
├── cpu/            # CPU implementation
│   ├── cpu.rs      # CPU struct and execution
│   ├── decode.rs   # Instruction decoding
│   ├── exec/       # Instruction execution
│   ├── instructions.rs
│   └── registers.rs
├── input/          # Input handling
│   └── joypad.rs
├── interrupt/      # Interrupt controller
├── memory/         # Memory management
│   ├── mbc.rs      # Memory Bank Controller
│   └── mmu.rs      # Memory Bus Unit
├── ppu/            # Picture Processing Unit
│   ├── oam.rs
│   ├── rendering.rs
│   └── video.rs
├── system.rs       # Main system controller
├── timer.rs        # Timer module
└── main.rs         # Entry point
```

## License

MIT
