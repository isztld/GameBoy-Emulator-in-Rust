    use super::*;
    use crate::memory::MemoryBus as GameBoyMemoryBus;

    fn make_bus(size: usize) -> GameBoyMemoryBus {
        let mut bus = GameBoyMemoryBus::new(vec![0u8; size]);
        bus.flat_mode = true;
        bus
    }

    #[test]
    fn test_disasm_nop() {
        let mut bus = make_bus(32768);
        bus.rom[0x0100] = 0x00; // NOP
        let instr = disasm_one(&bus, 0x0100);
        assert_eq!(instr.mnemonic, "NOP");
        assert_eq!(instr.bytes.len(), 1);
    }

    #[test]
    fn test_disasm_ld_bc_imm16() {
        let mut bus = make_bus(32768);
        bus.rom[0x0100] = 0x01; // LD BC, d16
        bus.rom[0x0101] = 0x34;
        bus.rom[0x0102] = 0x12;
        let instr = disasm_one(&bus, 0x0100);
        assert_eq!(instr.mnemonic, "LD");
        assert_eq!(instr.operand_str, "BC, 1234");
        assert_eq!(instr.bytes.len(), 3);
    }

    #[test]
    fn test_disasm_jr() {
        let mut bus = make_bus(32768);
        bus.rom[0x0100] = 0x18; // JR r8
        bus.rom[0x0101] = 0x05; // offset +5
        let instr = disasm_one(&bus, 0x0100);
        assert_eq!(instr.mnemonic, "JR");
    }

    #[test]
    fn test_disasm_halt() {
        let mut bus = make_bus(32768);
        bus.rom[0x0100] = 0x76; // HALT
        let instr = disasm_one(&bus, 0x0100);
        assert_eq!(instr.mnemonic, "HALT");
        assert_eq!(instr.operand_str, "");
    }

    #[test]
    fn test_disasm_cb_prefix() {
        let mut bus = make_bus(32768);
        bus.rom[0x0100] = 0xCB;
        bus.rom[0x0101] = 0x00; // RLC B
        let instr = disasm_one(&bus, 0x0100);
        assert_eq!(instr.mnemonic, "RLC");
        assert_eq!(instr.operand_str, "B");
        assert_eq!(instr.bytes.len(), 2);
    }

    #[test]
    fn test_disasm_add_hl() {
        let mut bus = make_bus(32768);
        bus.rom[0x0100] = 0x29; // ADD HL, HL
        let instr = disasm_one(&bus, 0x0100);
        assert_eq!(instr.mnemonic, "ADD");
        assert_eq!(instr.operand_str, "HL, HL");
    }

    #[test]
    fn test_disasm_ldh() {
        let mut bus = make_bus(32768);
        bus.rom[0x0100] = 0xE0; // LDH (n), A
        bus.rom[0x0101] = 0xFF;
        let instr = disasm_one(&bus, 0x0100);
        assert_eq!(instr.mnemonic, "LDH");
        assert_eq!(instr.operand_str, "(FF), A");
    }

    #[test]
    fn test_disasm_region() {
        let mut bus = make_bus(32768);
        bus.rom[0x0100] = 0x00; // NOP
        bus.rom[0x0101] = 0x00; // NOP
        bus.rom[0x0102] = 0x76; // HALT

        let instructions = disasm_region(&bus, 0x0100, 3);
        assert_eq!(instructions.len(), 3);
        assert_eq!(instructions[0].mnemonic, "NOP");
        assert_eq!(instructions[1].mnemonic, "NOP");
        assert_eq!(instructions[2].mnemonic, "HALT");
    }
