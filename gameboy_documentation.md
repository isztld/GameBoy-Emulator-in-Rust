# Game Boy Emulator Documentation

This comprehensive document contains all the information needed to build a working Game Boy emulator, based on detailed analysis of the Game Boy hardware documentation.

---

## Table of Contents

1. [Hardware Overview](#hardware-overview)
2. [CPU Architecture](#cpu-architecture)
3. [Memory Map](#memory-map)
4. [Interrupt System](#interrupt-system)
5. [Video Display Unit (PPU)](#video-display-unit-ppu)
6. [Audio Processing Unit (APU)](#audio-processing-unit-apu)
7. [Input/Output Registers](#io-registers)
8. [Cartridge Hardware](#cartridge-hardware)
9. [Super Game Boy (SGB)](#super-game-boy-sgb)
10. [Game Boy Color (CGB)](#game-boy-color-cgb)
11. [Power Management](#power-management)
12. [Implementation Checklist](#implementation-checklist)

---

## Hardware Overview

The Game Boy is an 8-bit handheld game console released by Nintendo in 1989. The system evolved through several hardware revisions:

| Model | CPU Clock | Work RAM | Video RAM | Colors |
|-------|-----------|----------|-----------|--------|
| DMG (Original) | 4.194304 MHz | 8 KiB | 8 KiB | 4 shades |
| MGB (Pocket) | 4.194304 MHz | 8 KiB | 8 KiB | 4 shades |
| SGB | ~4.2955 MHz (NTSC) | 8 KiB | 8 KiB | 32768 (via SNES) |
| CGB (Color) | Up to 8.388608 MHz | 32 KiB | 16 KiB | 32768 |

### Key Specifications

- **CPU**: 8-bit SM83 (8080-like) core
- **Master Clock**: 4.194304 MHz (DMG/MGB/CGB normal speed)
- **System Clock**: 1/4 of master clock = 1.048576 MHz
- **Screen Resolution**: 160 × 144 pixels
- **Refresh Rate**: ~59.73 Hz (NTSC) / ~50 Hz (PAL)
- **Horizontal Sync**: 9.198 kHz
- **Video RAM**: 8 KiB (DMG/MGB/SGB), 16 KiB (CGB)
- **Work RAM**: 8 KiB (DMG/MGB/SGB), 32 KiB (CGB)
- **Audio**: 4 channels (2 pulse, 1 wave, 1 noise)

---

## CPU Architecture

### Registers

The Game Boy uses a modified 8080/Z80-style register set:

| 16-bit | High | Low | Function |
|--------|------|-----|----------|
| AF | A | F | Accumulator & Flags |
| BC | B | C | General purpose |
| DE | D | E | General purpose |
| HL | H | L | General purpose |
| SP | - | - | Stack Pointer |
| PC | - | - | Program Counter |

### Flags Register (F)

| Bit | Name | Description |
|-----|------|-------------|
| 7 | Z | Zero flag (set if result is zero) |
| 6 | N | Subtraction flag (BCD) |
| 5 | H | Half Carry flag (BCD) |
| 4 | C | Carry flag |
| 3-0 | - | Unused (always 0) |

### Instruction Set Overview

The Game Boy's SM83 processor has a variable-length instruction set:

**Block 0 (00-3F):**
- `nop`, `halt`
- 16-bit load operations (`ld r16, imm16`)
- 16-bit arithmetic (`inc r16`, `dec r16`, `add hl, r16`)
- 8-bit arithmetic (`inc r8`, `dec r8`)
- Rotate/shift operations (`rlca`, `rrca`, `rla`, `rra`, `daa`, `cpl`, `scf`, `ccf`)
- Relative jumps (`jr`, `jr cond`)
- `stop`

**Block 1 (80-BF):**
- 8-bit register-to-register transfers (`ld r8, r8`)

**Block 2 (C0-FF):**
- 8-bit arithmetic with accumulator (`add a, r8`, `adc a, r8`, `sub a, r8`, etc.)
- 8-bit arithmetic with immediate (`add a, imm8`, etc.)
- Return and call operations (`ret`, `reti`, `call`, `rst`)
- Stack operations (`pop`, `push`)
- I/O operations (`ldh`, `ld [imm16], a`, etc.)
- SP arithmetic (`add sp, imm8`, `ld hl, sp+imm8`, `ld sp, hl`)
- Interrupt control (`di`, `ei`)

**CB-prefixed instructions:**
- Rotate/shift (`rlc`, `rrc`, `rl`, `rr`, `sla`, `sra`, `swap`, `srl`)
- Bit operations (`bit`, `res`, `set`)

### CPU Timing

- **M-cycle**: Machine cycle = 4 T-cycles (4.194 MHz / 4 = 1.048576 MHz)
- **T-cycle**: Time cycle = 1/4.194304 MHz ≈ 238 ns
- Most instructions: 1-4 M-cycles
- `halt`: Waits for interrupt (power-saving)
- `stop`: Enters low-power standby mode

### CPU Special Notes

- **halt bug**: When `halt` is executed with `IME=0` and an interrupt is pending, PC fails to increment properly
- **invalid opcodes**: $D3, $DB, $DD, $E3, $E4, $EB, $EC, $ED, $F4, $FC, $FD lock the CPU
- **stop instruction**: Can trigger speed switch on CGB when combined with KEY1 register

---

## Memory Map

The Game Boy has a 16-bit address bus (64 KiB addressable):

| Range | Size | Description |
|-------|------|-------------|
| $0000-$3FFF | 16 KiB | ROM Bank 0 (cartridge) |
| $4000-$7FFF | 16 KiB | ROM Bank 01-NN (cartridge, banked) |
| $8000-$9FFF | 8 KiB | Video RAM (VRAM) |
| $A000-$BFFF | 8 KiB | External RAM (cartridge, banked if MBC) |
| $C000-$CFFF | 4 KiB | Work RAM (WRAM) Bank 0 |
| $D000-$DFFF | 4 KiB | Work RAM (WRAM) Bank 1-7 (CGB only) |
| $E000-$FDFF | 2 KiB | Echo RAM (mirror of C000-DDFF) |
| $FE00-$FE9F | 160 bytes | Object Attribute Memory (OAM) |
| $FEA0-$FEFF | 96 bytes | Not Usable (returns $FF or hardware-dependent) |
| $FF00-$FF7F | 128 bytes | I/O Registers |
| $FF80-$FFFE | 127 bytes | High RAM (HRAM) |
| $FFFF | 1 byte | Interrupt Enable (IE) register |

### VRAM Layout

**Bank 0:**
- $8000-$8FFF: 128 tiles (first 2 blocks)
- $9000-$97FF: 128 tiles (third block)
- $9800-$9BFF: Tile map 0 (32×32 entries)
- $9C00-$9FFF: Tile map 1 (32×32 entries)

**Bank 1 (CGB only):**
- $8000-$8FFF: Tiles (same layout as bank 0)
- $9800-$9BFF: Attribute map 0
- $9C00-$9FFF: Attribute map 1

### Tile Data Format

Each tile is 16 bytes (8×8 pixels, 2 bits per pixel):

```
Byte 0-1:  Row 0 (LSB, MSB)
Byte 2-3:  Row 1 (LSB, MSB)
...
Byte 14-15: Row 7 (LSB, MSB)
```

### Tile Mapping

Two addressing modes controlled by LCDC.4:

- **$8000 mode**: Tiles 0-127 in block 0, 128-255 in block 1 (unsigned)
- **$8800 mode**: Tiles 0-127 in block 2, -128 to -1 in block 1 (signed)

### Echo RAM

- $E000-$FDFF mirrors $C000-$DDFF
- Only lower 13 bits of address used
- Nintendo prohibits use of this area

### HRAM (High RAM)

- $FF80-$FFFE: 127 bytes of fast RAM
- Often used for stack in interrupt handlers

---

## Interrupt System

### Interrupt Vector Addresses

| Address | Interrupt | Trigger |
|---------|-----------|---------|
| $0040 | VBlank | PPU enters VBlank (LY=144) |
| $0048 | LCD STAT | STAT interrupt condition met |
| $0050 | Timer | TIMA overflows |
| $0058 | Serial | Serial transfer complete |
| $0060 | Joypad | P10-P13 transitions from high to low |

### Interrupt Registers

**$FFFF - IE (Interrupt Enable):**

| Bit | Name | Description |
|-----|------|-------------|
| 0 | VBlank | VBlank interrupt enable |
| 1 | LCD | STAT interrupt enable |
| 2 | Timer | Timer interrupt enable |
| 3 | Serial | Serial interrupt enable |
| 4 | Joypad | Joypad interrupt enable |
| 7-5 | - | Unused |

**$FF0F - IF (Interrupt Flag):**

Same bit layout as IE. Set by hardware when interrupt condition occurs.

### Interrupt Handling

1. CPU checks `IE & IF` (both must be non-zero)
2. `IME` (Interrupt Master Enable) must be set
3. If multiple interrupts pending, highest priority services first:
   - VBlank (highest) → LCD → Timer → Serial → Joypad (lowest)
4. `IME` is cleared, PC pushed to stack
5. CPU jumps to interrupt vector address
6. Handler executes, typically ends with `reti` (restores IME)

### IME (Interrupt Master Enable)

- Set by `ei`, cleared by `di`
- Cleared when interrupt handler starts
- Set by `reti`
- **Note**: `ei` effect is delayed by one instruction

---

## Video Display Unit (PPU)

### PPU Modes

| Mode | Duration | Action | Accessible Memory |
|------|----------|--------|-------------------|
| 0 (HBlank) | 376 - mode 3 | Wait until end of scanline | VRAM, OAM |
| 1 (VBlank) | 4560 dots (10 lines) | Wait until next frame | VRAM, OAM |
| 2 (OAM Scan) | 80 dots | Search for sprites | VRAM, palettes |
| 3 (Pixel Transfer) | 172-289 dots | Draw pixels | None |

### PPU Timing

- **Line duration**: 456 dots (1.44 ms at 59.73 Hz)
- **Frame duration**: 70224 dots (16.74 ms at 59.73 Hz)
- **Total lines**: 154 (0-153)
- **Visible lines**: 0-143 (144 lines)
- **VBlank lines**: 144-153 (10 lines)

### PPU Registers

**$FF40 - LCDC (LCD Control):**

| Bit | Name | Description |
|-----|------|-------------|
| 7 | LCD Enable | 0=Off, 1=On |
| 6 | Window Tile Map | 0=$9800, 1=$9C00 |
| 5 | Window Enable | 0=Off, 1=On |
| 4 | BG/Window Tile Data | 0=$8800, 1=$8000 |
| 3 | BG Tile Map | 0=$9800, 1=$9C00 |
| 2 | OBJ Size | 0=8×8, 1=8×16 |
| 1 | OBJ Enable | 0=Off, 1=On |
| 0 | BG/Window Enable | DMG: 0=Off, 1=On; CGB: priority control |

**$FF41 - STAT (LCD Status):**

| Bit | Name | Description |
|-----|------|-------------|
| 6 | LYC Int Select | 1=Enable LYC=LY interrupt |
| 5 | Mode 2 Int Select | 1=Enable mode 2 interrupt |
| 4 | Mode 1 Int Select | 1=Enable mode 1 interrupt |
| 3 | Mode 0 Int Select | 1=Enable mode 0 interrupt |
| 2 | LYC == LY | 1=LYC matches LY |
| 1-0 | PPU Mode | Current PPU mode (0-3) |

**$FF42 - SCY (Scroll Y):** Viewport Y position (0-255)

**$FF43 - SCX (Scroll X):** Viewport X position (0-255)

**$FF44 - LY (LCD Y):** Current scanline (0-153)

**$FF45 - LYC:** LY compare value (triggers STAT interrupt when equal to LY)

**$FF4A - WY:** Window Y position (0-143)

**$FF4B - WX:** Window X position minus 7 (0-166, actual X = WX+7)

### Background Rendering

- 256×256 pixel tile map (32×32 tiles)
- 8×8 pixel tiles with 2 bits per pixel
- 4 color indices (0-3)
- Scrollable via SCY/SCX registers
- Two tile maps selectable via LCDC

### Window Rendering

- Superimposed on background
- Non-scrollable (starts at tile map origin)
- Position controlled by WY/WX
- Only visible when LCDC.5 set and (DMG: LCDC.0 also set)

### Sprite (OBJ) Rendering

**OAM Entry Format (4 bytes per sprite):**

| Byte | Description |
|------|-------------|
| 0 | Y position (Y+16 = screen position) |
| 1 | X position (X+8 = screen position) |
| 2 | Tile index (0-255) |
| 3 | Attributes/Flags |

**Sprite Attributes:**

| Bit | Name | DMG | CGB |
|-----|------|-----|-----|
| 7 | Priority | 0=OBJ over BG, 1=BG over OBJ (1-3 only) |
| 6 | Y flip | 0=Normal, 1=Vertical flip |
| 5 | X flip | 0=Normal, 1=Horizontal flip |
| 4 | Palette | 0=OBP0, 1=OBP1 |
| 3 | VRAM Bank | - | 0=Bank 0, 1=Bank 1 |
| 2-0 | Palette | - | 0-7 (OBP0-7) |

**Sprite Constraints:**
- Max 40 sprites on screen
- Max 10 sprites per scanline
- 8×8 or 8×16 pixel size (LCDC.2)
- 160×144 pixel visible area

### Sprite Priority

**DMG:** Lower X coordinate = higher priority. Ties broken by OAM order (earlier = higher priority).

**CGB:** Earlier OAM entry = higher priority.

### Rendering Algorithm

1. **Mode 2 (OAM Scan)**: Search 40 sprites for those overlapping current scanline
2. **Mode 3 (Pixel Transfer)**:
   - Fetch background tiles and render
   - Fetch and render sprites
   - Mix sprite and background pixels
3. **Mode 0 (HBlank)**: Wait for end of scanline
4. **Mode 1 (VBlank)**: Wait for next frame

### Mid-frame LCDC Changes

LCDC bits can be modified mid-frame:
- LCDC.1 (OBJ Enable): Toggles object visibility
- LCDC.5 (Window Enable): Toggles window visibility
- LCDC.6 (Window Tile Map): Switches window tile map

### LCD Enable Timing

**Critical:** LCD must only be disabled during VBlank (LY=144-153). Disabling during active display may permanently damage the hardware (burn-in).

---

## Audio Processing Unit (APU)

### APU Architecture

The Game Boy has 4 sound channels:

| Channel | Type | Features |
|---------|------|----------|
| CH1 | Pulse (Square) | Period sweep |
| CH2 | Pulse (Square) | No sweep |
| CH3 | Wave | User-defined waveform |
| CH4 | Noise | Pseudo-random LFSR |

### Global Controls

**$FF26 - NR52 (Audio Master Control):**

| Bit | Name | Description |
|-----|------|-------------|
| 7 | Audio On/Off | 0=Off, 1=On |
| 3 | CH4 On? | Status (read-only) |
| 2 | CH3 On? | Status (read-only) |
| 1 | CH2 On? | Status (read-only) |
| 0 | CH1 On? | Status (read-only) |

**$FF25 - NR51 (Sound Panning):**

| Bit | Name | Description |
|-----|------|-------------|
| 7 | CH4 Left | 1=Enable |
| 6 | CH3 Left | 1=Enable |
| 5 | CH2 Left | 1=Enable |
| 4 | CH1 Left | 1=Enable |
| 3 | CH4 Right | 1=Enable |
| 2 | CH3 Right | 1=Enable |
| 1 | CH2 Right | 1=Enable |
| 0 | CH1 Right | 1=Enable |

**$FF24 - NR50 (Master Volume & VIN):**

| Bits | Name | Description |
|------|------|-------------|
| 7 | VIN Left | VIN panning |
| 6-4 | Left Volume | 0-7 (1-8 scaled) |
| 3 | VIN Right | VIN panning |
| 2-0 | Right Volume | 0-7 (1-8 scaled) |

### Channel 1: Pulse with Sweep

**$FF10 - NR10 (Sweep):**

| Bits | Name | Description |
|------|------|-------------|
| 6-4 | Pace | Sweep rate (128 Hz ticks) |
| 3 | Direction | 0=Add, 1=Subtract |
| 2-0 | Step | Number of steps |

**$FF11 - NR11 (Duty & Length):**

| Bits | Name | Description |
|------|------|-------------|
| 7-6 | Duty Cycle | 00=12.5%, 01=25%, 10=50%, 11=75% |
| 5-0 | Initial Length | 64-length (length timer) |

**$FF12 - NR12 (Envelope):**

| Bits | Name | Description |
|------|------|-------------|
| 7-4 | Initial Volume | 0-15 |
| 3 | Direction | 0=Decrease, 1=Increase |
| 2-0 | Sweep Pace | Envelope rate |

**$FF13 - NR13 (Period Low):** Period value bits 7-0

**$FF14 - NR14 (Period High & Control):**

| Bits | Name | Description |
|------|------|-------------|
| 7 | Trigger | 1=Trigger channel |
| 6 | Length Enable | 1=Enable length timer |
| 2-0 | Period | Period value bits 10-8 |

### Channel 2: Pulse (No Sweep)

Identical to CH1 except no NR10 register.

**$FF16 - NR21** → NR11 equivalent
**$FF17 - NR22** → NR12 equivalent
**$FF18 - NR23** → NR13 equivalent
**$FF19 - NR24** → NR14 equivalent

### Channel 3: Wave Output

**$FF1A - NR30 (DAC Enable):**

| Bit | Name | Description |
|-----|------|-------------|
| 7 | DAC On/Off | 0=Off, 1=On |

**$FF1B - NR31 (Length):** Length timer initial value

**$FF1C - NR32 (Output Level):**

| Bits | Name | Description |
|------|------|-------------|
| 6-5 | Output Level | 00=Mute, 01=100%, 10=50%, 11=25% |

**$FF1D - NR33 (Period Low):** Period value bits 7-0

**$FF1E - NR34 (Period High & Control):**

| Bits | Name | Description |
|------|------|-------------|
| 7 | Trigger | 1=Trigger channel |
| 6 | Length Enable | 1=Enable length timer |
| 2-0 | Period | Period value bits 10-8 |

**$FF30-$FF3F - Wave RAM:** 16 bytes (32 samples, 4-bit each)

### Channel 4: Noise

**$FF20 - NR41 (Length):** Length timer initial value

**$FF21 - NR42 (Envelope):** Same format as NR12

**$FF22 - NR43 (Frequency & Randomness):**

| Bits | Name | Description |
|------|------|-------------|
| 7-4 | Clock Shift | LFSR clock divider |
| 3 | LFSR Width | 0=15-bit, 1=7-bit |
| 2-0 | Clock Divider | Frequency divisor |

**$FF23 - NR44 (Control):**

| Bits | Name | Description |
|------|------|-------------|
| 7 | Trigger | 1=Trigger channel |
| 6 | Length Enable | 1=Enable length timer |

---

## IO Registers

### Joypad ($FF00 - P1/JOYP)

| Bit | Name | Description |
|-----|------|-------------|
| 7-6 | - | Unused (read as 1) |
| 5 | Select Buttons | 0=Buttons, 1=Direction |
| 4 | Select Direction | 0=Direction, 1=Buttons |
| 3 | Start | 0=Pressed |
| 2 | Select | 0=Pressed |
| 1 | B | 0=Pressed |
| 0 | A | 0=Pressed |

### Serial Transfer ($FF01 - SB, $FF02 - SC)

**SB:** Data to transfer

**SC:**

| Bit | Name | Description |
|-----|------|-------------|
| 7 | Transfer Start | 1=Start transfer |
| 1 | Clock Speed | CGB only: 1=High speed |
| 0 | Clock Select | 0=External (slave), 1=Internal (master) |

### Timer ($FF04 - DIV, $FF05 - TIMA, $FF06 - TMA, $FF07 - TAC)

**DIV:** Divider register (incremented at 16384 Hz)

**TIMA:** Timer counter

**TMA:** Timer modulo (loaded when TIMA overflows)

**TAC:**

| Bits | Name | Description |
|------|------|-------------|
| 2 | Enable | 0=Disable, 1=Enable |
| 1-0 | Clock Select | See table in Timer docs |

### OAM DMA ($FF46 - DMA)

Writing to this register starts a 160-byte DMA transfer:
- Source: $XX00-$XX9F (XX = written value)
- Destination: $FE00-$FE9F (OAM)
- Duration: 160 M-cycles

### VRAM DMA ($FF51-$FF55 - HDMA1-5, CGB only)

**HDMA1-2:** Source address (bits 15-4, lower 4 bits ignored)

**HDMA3-4:** Destination address (bits 15-4, lower 4 bits ignored)

**HDMA5:**

| Bits | Name | Description |
|------|------|-------------|
| 7 | Transfer Mode | 0=General purpose, 1=HBlank |
| 6-0 | Length | (Length+1) × $10 bytes |

### CGB Registers

**$FF4C - KEY0 (CPU Mode):**
- Bit 2: DMG compatibility mode

**$FF4D - KEY1 (Speed Switch):**
- Bit 7: Current speed (0=Normal, 1=Double)
- Bit 0: Switch armed (write 1 to prepare switch)

**$FF4F - VBK (VRAM Bank):**
- Bit 0: 0=Bank 0, 1=Bank 1

**$FF70 - SVBK (WRAM Bank):**
- Bits 2-0: Bank number (1-7, 0 maps bank 1)

**$FF68-$FF6B - Palette registers:** CGB palette access

---

## Cartridge Hardware

### Cartridge Header ($0100-$014F)

**$0100-$0103:** Entry point (usually `nop` + `jp`)

**$0104-$0133:** Nintendo logo (must match exactly)

**$0134-$0143:** Game title (16 chars, ASCII)

**$0143:** CGB flag ($80=dual compatibility, $C0=CGB only)

**$0144-$0145:** New licensee code (ASCII, e.g., "01"=Nintendo)

**$0146:** SGB flag ($03=SGB compatible)

**$0147:** Cartridge type (MBC selection)

**$0148:** ROM size (2^N × 32 KiB)

**$0149:** RAM size (0, 8, 32, 128 KiB)

**$014A:** Destination code ($00=Japan, $01=Overseas)

**$014B:** Old licensee code ($33=use new code instead)

**$014D:** Header checksum (computed from $0134-$014C)

**$014E-$014F:** Global checksum (sum of ROM except these bytes)

### Cartridge Types

| Code | Type |
|------|------|
| $00 | ROM ONLY |
| $01 | MBC1 |
| $02 | MBC1+RAM |
| $03 | MBC1+RAM+BATTERY |
| $05 | MBC2 |
| $06 | MBC2+BATTERY |
| $08 | ROM+RAM |
| $09 | ROM+RAM+BATTERY |
| $11 | MBC3 |
| $12 | MBC3+RAM |
| $13 | MBC3+RAM+BATTERY |
| $19 | MBC5 |
| $1A | MBC5+RAM |
| $1B | MBC5+RAM+BATTERY |
| $1C | MBC5+RUMBLE |
| $1D | MBC5+RUMBLE+RAM |
| $1E | MBC5+RUMBLE+RAM+BATTERY |
| $22 | MBC7+SENSOR+RUMBLE+RAM+BATTERY |
| $FC | POCKET CAMERA |
| $FD | BANDAI TAMA5 |
| $FE | HuC3 |
| $FF | HuC1+RAM+BATTERY |

### MBC1 Memory Bank Controller

**Registers:**

**$0000-$1FFF:** RAM Enable (write $A to enable)

**$2000-$3FFF:** ROM Bank Number (5 bits, $01-$1F)

**$4000-$5FFF:** RAM Bank / Upper ROM bits (2 bits)

**$6000-$7FFF:** Banking Mode Select (0=Simple, 1=Advanced)

**Addressing:**

- $0000-$3FFF: ROM Bank 0 (mode 0) or banked (mode 1)
- $4000-$7FFF: ROM Bank (selected by $2000-$3FFF)
- $A000-$BFFF: RAM Bank (selected by $4000-$5FFF)

### MBC5 Memory Bank Controller

**Registers:**

**$0000-$1FFF:** RAM Enable (write $A to enable)

**$2000-$2FFF:** ROM Bank Number (bits 7-0)

**$3000-$3FFF:** ROM Bank Number (bit 8)

**$4000-$5FFF:** RAM Bank Number (0-15)

**Features:**
- Supports up to 8 MiB ROM
- Supports up to 128 KiB RAM
- Guaranteed to work with CGB double speed mode

### MBC3 with RTC

**Additional RTC Registers:**

**$08-$0C:** RTC registers (when RAM bank selected)

| Register | Name | Description |
|----------|------|-------------|
| $08 | S | Seconds (0-59) |
| $09 | M | Minutes (0-59) |
| $0A | H | Hours (0-23) |
| $0B | DL | Day counter low |
| $0C | DH | Day counter high + flags |

**Latch clock:** Write $00 then $01 to $6000-$7FFF

### HuC1

**Registers:**

**$0000-$1FFF:** IR/RAM Select (write $E to select IR mode)

**$2000-$3FFF:** ROM Bank Number

**$4000-$5FFF:** RAM Bank Select

**$A000-$BFFF:** RAM or IR register (depending on select)

### HuC-3

Similar to HuC1 but with:
- Real-time clock
- Speaker output
- More complex IR protocol

### MBC7

**Features:**
- 2-axis accelerometer (ADXL202E)
- 256-byte EEPROM (93LC56)
- 2-wire serial interface

**EEPROM Commands:**
- READ: 10xAAAAAAAb
- WRITE: 01xAAAAAAAb
- EWEN: 0011xxxxxxb
- EWDS: 0000xxxxxxb
- ERASE: 11xAAAAAAAb

---

## Super Game Boy (SGB)

### SGB Hardware

The Super Game Boy is an adapter cartridge for SNES that allows Game Boy games to run on a TV.

### SGB Command Packets

**Packet Format:**
1. Start pulse (JOYP bits 4-5 both low)
2. Header byte: (Command × 8) + Length
3. Data bytes (up to 15)
4. Stop bit (0)

**Maximum:** 7 packets (111 data bytes)

### SGB Commands

**Palette Commands:**
- $00-$03: Set palettes 0-3
- $0A: Set palette indirect
- $0B: Transfer palette data

**Attribute Commands:**
- $04: Block area designation
- $05: Line area designation
- $06: Divide area designation
- $07: 1CHR area designation
- $15: Transfer attributes
- $16: Set attributes

**Transfer Commands:**
- $13: Character font data
- $14: Screen color data
- $18: SNES sprite mode

**System Commands:**
- $0C: Attraction enable
- $0D: Test enable
- $0E: Icon enable
- $0F-$10: SNES WRAM transfer
- $11: Multiple controllers request
- $12: Set SNES program counter
- $17: Game Boy window mask
- $19: Palette priority

### SGB Detection

- **C register after reset:** $14 = SGB/SGB2
- **A register after reset:** $01 = DMG/SGB, $FF = MGB/SGB2
- **Cartridge header:** $0146 must be $03 for SGB games

---

## Game Boy Color (CGB)

### CGB Hardware Features

- Up to 8.388608 MHz CPU clock (double speed mode)
- 32 KiB WRAM (8 banks of 4 KiB)
- 16 KiB VRAM (2 banks of 8 KiB)
- 32768 colors (15-bit RGB)
- Speed switching between normal and double speed

### CGB Registers

**$FF4C - KEY0 (CPU Mode):**
- Bit 2: DMG compatibility mode
- Locks after boot

**$FF4D - KEY1 (Speed Switch):**
- Bit 7: Current speed (0=Normal, 1=Double)
- Bit 0: Switch armed

**$FF4F - VBK (VRAM Bank):**
- Bit 0: 0=Bank 0, 1=Bank 1

**$FF51-$FF55 - HDMA1-5 (VRAM DMA):**
- Source/destination addresses
- Transfer length and mode (general purpose or HBlank)

**$FF68-$FF6B - CGB Palettes:**
- BCPS/BCPD: Background palette
- OCPS/OCPD: OBJ palette

**$FF70 - SVBK (WRAM Bank):**
- 1-7: Bank number (0 maps bank 1)

### CGB Speed Switch

1. Check KEY1 bit 7 for current speed
2. Set KEY1 bit 0 to arm switch
3. Execute `stop` instruction
4. CPU speed changes after `stop`

**Note:** During speed switch, CPU stops for 2050 M-cycles. VRAM/OAM locking is frozen.

### CGB Color Palettes

**Palette Memory:** 64 bytes each for BG and OBJ
- 8 palettes × 4 colors × 2 bytes = 64 bytes
- Each color: RGB555 format (15-bit)

**Access:**
1. Set BCPS/OCPS address (auto-increment on write)
2. Write color data to BCPD/OCPD

**RGB555 Format:**
```
Bit 14-10: Red (0-31)
Bit 9-5: Green (0-31)
Bit 4-0: Blue (0-31)
```

### CGB Compatibility Mode

When running DMG games on CGB:
- Boot ROM auto-colorizes based on title checksum
- Uses CGB palettes but indices match DMG shades
- OBJ uses OBP0/OBP1 which index into CGB palettes

### CGB Detection

- **A register after reset:** $11 = CGB/AGB
- **B register bit 0:** 0 = CGB, 1 = GBA

---

## Power Management

### HALT Instruction

- Pauses CPU until interrupt enabled in IE
- Reduces power consumption significantly
- Common use: Wait for VBlank

**Example:**
```asm
wait_vblank:
    halt
    cp [vblank_flag], 0
    jr z, wait_vblank
    ld [vblank_flag], 0
```

### STOP Instruction

- Enters very low-power standby mode
- Terminated by any button press
- On CGB: Can trigger speed switch

**Warning:** Never disable LCD outside VBlank - may damage hardware!

### Power Consumption Tips

1. Use `halt` when possible (5-50% power savings)
2. Disable audio when not needed (NR52 bit 7 = 0)
3. Use normal speed on CGB when possible
4. Write tight assembly code
5. Use `stop` for extended idle periods

---

## Implementation Checklist

### Phase 1: Core CPU

- [ ] SM83 CPU with all registers
- [ ] Instruction decoder
- [ ] All 256 opcodes implemented
- [ ] CB-prefixed instructions
- [ ] Interrupt system (IME, IE, IF)
- [ ] Timing (M-cycles, T-cycles)

### Phase 2: Memory System

- [ ] 16-bit address bus
- [ ] ROM bank 0 always accessible
- [ ] MMU for memory mapping
- [ ] Echo RAM behavior
- [ ] HRAM implementation

### Phase 3: PPU

- [ ] PPU modes (0-3)
- [ ] LCD status register
- [ ] SCY/SCX scrolling
- [ ] Tile rendering
- [ ] Sprite rendering
- [ ] Window rendering
- [ ] Palette selection (DMG)
- [ ] Mid-frame LCDC changes

### Phase 4: APU

- [ ] NR52 master control
- [ ] NR51 panning
- [ ] NR50 master volume
- [ ] CH1-2: Pulse channels with envelope
- [ ] CH3: Wave channel with waveform RAM
- [ ] CH4: Noise channel with LFSR

### Phase 5: IO System

- [ ] Joypad matrix reading
- [ ] Serial transfer (link cable)
- [ ] DIV/TIMA/TMA/TAC timers
- [ ] OAM DMA ($FF46)

### Phase 6: Cartridge

- [ ] Cartridge header parsing
- [ ] MBC1 support
- [ ] MBC3 with RTC support
- [ ] MBC5 support
- [ ] Cartridge RAM (SRAM)
- [ ] Battery-backed save

### Phase 7: CGB Support

- [ ] CGB detection
- [ ] WRAM bank switching
- [ ] VRAM bank switching
- [ ] CGB palette system
- [ ] HDMA support
- [ ] Double speed mode

### Phase 8: SGB Support (Optional)

- [ ] SGB command packet parsing
- [ ] SGB command handling
- [ ] SNES communication protocol

---

## Testing

### Test ROMs

Use these test ROMs to verify implementation:

1. **blargg's test ROMs:** https://github.com/retrio/gb-test-roms
   - cpu_instrs
   - mem_timing
   - instr_timing
   - interrupt_time

2. **Mooneye-GB tests:** https://github.com/Gekkio/mooneye-gb
   - acceptance tests
   - gbapi tests
   - hardware tests

### Common Issues

1. **Timing errors:** Verify M-cycle counts
2. **Interrupt timing:** Check delay before handler call
3. **OAM corruption:** Verify access during PPU modes
4. **VRAM access:** Check STAT mode bits before access
5. **LCD enable:** Must only disable during VBlank

---

## References

- [Game Boy Development Guide](https://gbdev.io/)
- [Game Boy Opcode Table](https://gbdev.io/gb-opcodes/optables/)
- [Game Boy Register Reference](https://gbdev.io/gb registers/)
- [Game Boy Hardware Reference](https://gbdev.io/gb-hardware/)
- [Mooneye-GB tests](https://github.com/Gekkio/mooneye-gb)
- [Game Boy Programming Manual](https://www.pentacom.jp/pentacom/bitlib/manuais/gbprogram.pdf)

---

*This documentation was compiled from the pandoc documentation suite and is intended to provide all the information needed to build a functional Game Boy emulator.*
