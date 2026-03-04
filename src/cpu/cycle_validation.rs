/// Cycle count validation tests
///
/// Validates that every instruction returns the correct machine-cycle count
/// by comparing against the authoritative tables from gb-test-roms instr_timing.s.
///
/// For conditional instructions (JR cc, JP cc, CALL cc, RET cc) the table holds
/// the "not-taken" (minimum) count, so flags are arranged so the condition fails.

#[cfg(test)]
mod tests {
    use crate::cpu::CPUState;
    use crate::cpu::decode::decode_instruction;
    use crate::cpu::exec::execute_instruction;
    use crate::memory::MemoryBus;

    // Authoritative cycle counts from gb-test-roms instr_timing.s (op_times table).
    // 0 means the opcode is illegal / not timed (STOP, HALT, CB-prefix, undefined).
    #[rustfmt::skip]
    const NORMAL_TIMES: [u8; 256] = [
        // 0x00-0x0F
        1,3,2,2,1,1,2,1,5,2,2,2,1,1,2,1,
        // 0x10-0x1F
        0,3,2,2,1,1,2,1,3,2,2,2,1,1,2,1,
        // 0x20-0x2F
        2,3,2,2,1,1,2,1,2,2,2,2,1,1,2,1,
        // 0x30-0x3F
        2,3,2,2,3,3,3,1,2,2,2,2,1,1,2,1,
        // 0x40-0x4F
        1,1,1,1,1,1,2,1,1,1,1,1,1,1,2,1,
        // 0x50-0x5F
        1,1,1,1,1,1,2,1,1,1,1,1,1,1,2,1,
        // 0x60-0x6F
        1,1,1,1,1,1,2,1,1,1,1,1,1,1,2,1,
        // 0x70-0x7F  (0x76 = HALT → 0)
        2,2,2,2,2,2,0,2,1,1,1,1,1,1,2,1,
        // 0x80-0x8F
        1,1,1,1,1,1,2,1,1,1,1,1,1,1,2,1,
        // 0x90-0x9F
        1,1,1,1,1,1,2,1,1,1,1,1,1,1,2,1,
        // 0xA0-0xAF
        1,1,1,1,1,1,2,1,1,1,1,1,1,1,2,1,
        // 0xB0-0xBF
        1,1,1,1,1,1,2,1,1,1,1,1,1,1,2,1,
        // 0xC0-0xCF  (0xCB = CB prefix → 0)
        2,3,3,4,3,4,2,4,2,4,3,0,3,6,2,4,
        // 0xD0-0xDF  (0xD3,0xDB,0xDD → 0)
        2,3,3,0,3,4,2,4,2,4,3,0,3,0,2,4,
        // 0xE0-0xEF  (0xE3,0xE4,0xEB,0xEC,0xED → 0)
        3,3,2,0,0,4,2,4,4,1,4,0,0,0,2,4,
        // 0xF0-0xFF  (0xF4,0xFC,0xFD → 0)
        3,3,2,1,0,4,2,4,3,2,4,1,0,0,2,4,
    ];

    // Authoritative cycle counts for CB-prefixed instructions (cb_op_times table).
    #[rustfmt::skip]
    const CB_TIMES: [u8; 256] = [
        // 0xCB00-0xCB0F
        2,2,2,2,2,2,4,2,2,2,2,2,2,2,4,2,
        // 0xCB10-0xCB1F
        2,2,2,2,2,2,4,2,2,2,2,2,2,2,4,2,
        // 0xCB20-0xCB2F
        2,2,2,2,2,2,4,2,2,2,2,2,2,2,4,2,
        // 0xCB30-0xCB3F
        2,2,2,2,2,2,4,2,2,2,2,2,2,2,4,2,
        // 0xCB40-0xCB4F  (BIT)
        2,2,2,2,2,2,3,2,2,2,2,2,2,2,3,2,
        // 0xCB50-0xCB5F
        2,2,2,2,2,2,3,2,2,2,2,2,2,2,3,2,
        // 0xCB60-0xCB6F
        2,2,2,2,2,2,3,2,2,2,2,2,2,2,3,2,
        // 0xCB70-0xCB7F
        2,2,2,2,2,2,3,2,2,2,2,2,2,2,3,2,
        // 0xCB80-0xCB8F  (RES)
        2,2,2,2,2,2,4,2,2,2,2,2,2,2,4,2,
        // 0xCB90-0xCB9F
        2,2,2,2,2,2,4,2,2,2,2,2,2,2,4,2,
        // 0xCBA0-0xCBAF
        2,2,2,2,2,2,4,2,2,2,2,2,2,2,4,2,
        // 0xCBB0-0xCBBF
        2,2,2,2,2,2,4,2,2,2,2,2,2,2,4,2,
        // 0xCBC0-0xCBCF  (SET)
        2,2,2,2,2,2,4,2,2,2,2,2,2,2,4,2,
        // 0xCBD0-0xCBDF
        2,2,2,2,2,2,4,2,2,2,2,2,2,2,4,2,
        // 0xCBE0-0xCBEF
        2,2,2,2,2,2,4,2,2,2,2,2,2,2,4,2,
        // 0xCBF0-0xCBFF
        2,2,2,2,2,2,4,2,2,2,2,2,2,2,4,2,
    ];

    /// Build a 64 KiB flat-mode bus with `opcode` at address 0 (followed by zeros).
    fn make_bus(opcode: u8) -> MemoryBus {
        let mut mem = vec![0u8; 65536];
        mem[0] = opcode;
        let mut bus = MemoryBus::new(mem);
        bus.flat_mode = true;
        bus
    }

    /// Build a 64 KiB flat-mode bus with a CB-prefixed opcode at address 0.
    fn make_cb_bus(cb_opcode: u8) -> MemoryBus {
        let mut mem = vec![0u8; 65536];
        mem[0] = 0xCB;
        mem[1] = cb_opcode;
        let mut bus = MemoryBus::new(mem);
        bus.flat_mode = true;
        bus
    }

    /// Build a default CPUState with sensible register values for execution.
    fn make_cpu() -> CPUState {
        let mut cpu = CPUState::new();
        // Keep SP at the default 0xFFFE from Registers::new().
        // HL/BC/DE point into the middle of flat RAM so (HL) reads/writes land safely.
        cpu.registers.hl = 0x8000;
        cpu.registers.bc = 0x8002;
        cpu.registers.de = 0x8004;
        cpu
    }

    /// Set CPU flags so that the given conditional opcode does NOT branch.
    /// This ensures `execute_instruction` returns the minimum (not-taken) cycle count,
    /// which is what the authoritative table records.
    ///
    /// Flag layout in AF (u16):  high byte = A,  low byte = F
    ///   Bit 7 of F (= bit 7 of AF low byte = AF bit 7) : Z flag  → AF |= 0x0080
    ///   Bit 4 of F (= bit 4 of AF low byte = AF bit 4) : C flag  → AF |= 0x0010
    fn set_not_taken_flags(cpu: &mut CPUState, opcode: u8) {
        match opcode {
            // NZ condition: NOT taken when Z = 1
            0x20 | 0xC0 | 0xC2 | 0xC4 => cpu.registers.af |= 0x0080,
            // NC condition: NOT taken when C = 1
            0x30 | 0xD0 | 0xD2 | 0xD4 => cpu.registers.af |= 0x0010,
            // Z condition:  NOT taken when Z = 0  (default — no change needed)
            // C condition:  NOT taken when C = 0  (default — no change needed)
            _ => {}
        }
    }

    fn noop_tick(_: &mut [u8; 128]) {}

    #[test]
    fn test_normal_opcode_cycles() {
        let mut errors: Vec<String> = Vec::new();

        for opcode in 0u8..=255 {
            let expected = NORMAL_TIMES[opcode as usize];
            if expected == 0 {
                continue; // illegal / HALT / STOP / CB-prefix — not tested here
            }

            let mut bus = make_bus(opcode);
            let mut cpu = make_cpu();
            set_not_taken_flags(&mut cpu, opcode);

            let (instruction, _) = decode_instruction(&cpu, &bus, 0x0000, opcode);
            let cycles = execute_instruction(&mut cpu, &mut bus, instruction, &mut noop_tick);

            if cycles != expected as u32 {
                errors.push(format!(
                    "  0x{:02X}: expected {} M-cycles, got {}",
                    opcode, expected, cycles
                ));
            }
        }

        if !errors.is_empty() {
            panic!(
                "{} cycle-count mismatch(es) in normal opcodes:\n{}",
                errors.len(),
                errors.join("\n")
            );
        }
    }

    #[test]
    fn test_cb_opcode_cycles() {
        let mut errors: Vec<String> = Vec::new();

        for cb_opcode in 0u8..=255 {
            let expected = CB_TIMES[cb_opcode as usize];
            if expected == 0 {
                continue;
            }

            let mut bus = make_cb_bus(cb_opcode);
            let mut cpu = make_cpu();

            let (instruction, _) = decode_instruction(&cpu, &bus, 0x0000, 0xCB);
            let cycles = execute_instruction(&mut cpu, &mut bus, instruction, &mut noop_tick);

            if cycles != expected as u32 {
                errors.push(format!(
                    "  0xCB{:02X}: expected {} M-cycles, got {}",
                    cb_opcode, expected, cycles
                ));
            }
        }

        if !errors.is_empty() {
            panic!(
                "{} cycle-count mismatch(es) in CB-prefixed opcodes:\n{}",
                errors.len(),
                errors.join("\n")
            );
        }
    }
}
