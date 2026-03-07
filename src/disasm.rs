/// Simple disassembler for GameBoy CPU (SM83)

use crate::memory::MemoryBus as GameBoyMemoryBus;

#[derive(Debug, Clone)]
pub struct DisassembledInstruction {
    pub address: u16,
    pub bytes: Vec<u8>,
    pub mnemonic: String,
    pub operand_str: String,
}

/// Register names for SM83 CPU
const REG_NAMES: [&str; 8] = ["B", "C", "D", "E", "H", "L", "(HL)", "A"];

/// Get register name by index
fn reg_name(index: usize) -> &'static str {
    REG_NAMES[index & 7]
}

/// Disassemble a single instruction starting at the given address
pub fn disasm_one(bus: &impl MemoryRead, address: u16) -> DisassembledInstruction {
    let opcode = bus.read_byte(address);

    let (bytes, mnemonic, operand_str) = match opcode {
        // 8-bit load group
        0x06 => load_r_d8(bus, address, "B"),
        0x0E => load_r_d8(bus, address, "C"),
        0x16 => load_r_d8(bus, address, "D"),
        0x1E => load_r_d8(bus, address, "E"),
        0x26 => load_r_d8(bus, address, "H"),
        0x2E => load_r_d8(bus, address, "L"),
        0x36 => load_hl_d8(bus, address),
        0x3E => load_r_d8(bus, address, "A"),

        // 16-bit load group
        0x01 => load_rp_dd(bus, address, "BC"),
        0x11 => load_rp_dd(bus, address, "DE"),
        0x21 => load_rp_dd(bus, address, "HL"),
        0x31 => load_rp_dd(bus, address, "SP"),

        // Storage operations
        0x02 => (vec![0x02], "LD".to_string(), "BC[A], A".to_string()),
        0x0A => (vec![0x0A], "LD".to_string(), "A, (BC)".to_string()),
        0x12 => (vec![0x12], "LD".to_string(), "DE[A], A".to_string()),
        0x1A => (vec![0x1A], "LD".to_string(), "A, (DE)".to_string()),
        0x22 => (vec![0x22], "LD".to_string(), "HL+, A".to_string()),
        0x2A => (vec![0x2A], "LD".to_string(), "A, HL+".to_string()),
        0x32 => (vec![0x32], "LD".to_string(), "HL-, A".to_string()),
        0x3A => (vec![0x3A], "LD".to_string(), "A, HL-".to_string()),

        0xE0 => ldh_a_d8(bus, address),
        0xF0 => ldh_d8_a(bus, address),

        0xEA => ld_abus_a(bus, address),
        0xFA => ld_a_abus(bus, address),

        0xE2 => (vec![0xE2], "LD".to_string(), "(C), A".to_string()),
        0xF2 => (vec![0xF2], "LD".to_string(), "A, (C)".to_string()),
        0xE5 => (vec![0xE5], "PUSH".to_string(), "HL".to_string()),
        0xC5 => (vec![0xC5], "PUSH".to_string(), "BC".to_string()),
        0xD5 => (vec![0xD5], "PUSH".to_string(), "DE".to_string()),
        0xF5 => (vec![0xF5], "PUSH".to_string(), "AF".to_string()),
        0xC1 => (vec![0xC1], "POP".to_string(), "BC".to_string()),
        0xE1 => (vec![0xE1], "POP".to_string(), "HL".to_string()),
        0xD1 => (vec![0xD1], "POP".to_string(), "DE".to_string()),
        0xF1 => (vec![0xF1], "POP".to_string(), "AF".to_string()),

        // Increment/Decrement 8-bit
        0x04 => (vec![0x04], "INC".to_string(), "B".to_string()),
        0x05 => (vec![0x05], "DEC".to_string(), "B".to_string()),
        0x0C => (vec![0x0C], "INC".to_string(), "C".to_string()),
        0x0D => (vec![0x0D], "DEC".to_string(), "C".to_string()),
        0x14 => (vec![0x14], "INC".to_string(), "D".to_string()),
        0x15 => (vec![0x15], "DEC".to_string(), "D".to_string()),
        0x1C => (vec![0x1C], "INC".to_string(), "E".to_string()),
        0x1D => (vec![0x1D], "DEC".to_string(), "E".to_string()),
        0x24 => (vec![0x24], "INC".to_string(), "H".to_string()),
        0x25 => (vec![0x25], "DEC".to_string(), "H".to_string()),
        0x2C => (vec![0x2C], "INC".to_string(), "L".to_string()),
        0x2D => (vec![0x2D], "DEC".to_string(), "L".to_string()),
        0x3C => (vec![0x3C], "INC".to_string(), "A".to_string()),
        0x3D => (vec![0x3D], "DEC".to_string(), "A".to_string()),
        0x34 => (vec![0x34], "INC".to_string(), "(HL)".to_string()),
        0x35 => (vec![0x35], "DEC".to_string(), "(HL)".to_string()),

        // Increment/Decrement 16-bit
        0x03 => (vec![0x03], "INC".to_string(), "BC".to_string()),
        0x0B => (vec![0x0B], "DEC".to_string(), "BC".to_string()),
        0x13 => (vec![0x13], "INC".to_string(), "DE".to_string()),
        0x1B => (vec![0x1B], "DEC".to_string(), "DE".to_string()),
        0x23 => (vec![0x23], "INC".to_string(), "HL".to_string()),
        0x2B => (vec![0x2B], "DEC".to_string(), "HL".to_string()),
        0x33 => (vec![0x33], "INC".to_string(), "SP".to_string()),
        0x3B => (vec![0x3B], "DEC".to_string(), "SP".to_string()),

        // ALU - 8-bit register
        0x80..=0x87 => alu_r(opcode, "ADD"),
        0x88..=0x8F => alu_r(opcode, "ADC"),
        0x90..=0x97 => alu_r(opcode, "SUB"),
        0x98..=0x9F => alu_r(opcode, "SBC"),
        0xA0..=0xA7 => alu_r(opcode, "AND"),
        0xA8..=0xAF => alu_r(opcode, "XOR"),
        0xB0..=0xB7 => alu_r(opcode, "OR"),
        0xB8..=0xBF => alu_r(opcode, "CP"),

        // ALU - immediate
        0xC6 => alu_d8(bus, address, "ADD"),
        0xD6 => alu_d8(bus, address, "SUB"),
        0xE6 => alu_d8(bus, address, "AND"),
        0xF6 => alu_d8(bus, address, "OR"),
        0xCE => alu_d8(bus, address, "ADC"),
        0xDE => alu_d8(bus, address, "SBC"),
        0xFE => alu_d8(bus, address, "CP"),

        // ALU - single operand
        0x3F => (vec![0x3F], "CCF".to_string(), "".to_string()),
        0x2F => (vec![0x2F], "CPL".to_string(), "".to_string()),
        0x37 => (vec![0x37], "SCF".to_string(), "".to_string()),
        0x07 => (vec![0x07], "RLCA".to_string(), "".to_string()),
        0x0F => (vec![0x0F], "RRCA".to_string(), "".to_string()),
        0x17 => (vec![0x17], "RLA".to_string(), "".to_string()),
        0x1F => (vec![0x1F], "RRA".to_string(), "".to_string()),
        0x27 => (vec![0x27], "DAA".to_string(), "".to_string()),

        // Jumps
        0xC3 => jp_ccc_im16(bus, address, "".to_string()),
        0xC2 => jp_ccc_im16(bus, address, "NZ".to_string()),
        0xCA => jp_ccc_im16(bus, address, "Z".to_string()),
        0xD2 => jp_ccc_im16(bus, address, "NC".to_string()),
        0xDA => jp_ccc_im16(bus, address, "C".to_string()),
        0xE9 => (vec![0xE9], "JP".to_string(), "HL".to_string()),
        0x18 => jr_ccc_rel(bus, address, "".to_string()),
        0x20 => jr_ccc_rel(bus, address, "NZ".to_string()),
        0x28 => jr_ccc_rel(bus, address, "Z".to_string()),
        0x30 => jr_ccc_rel(bus, address, "NC".to_string()),
        0x38 => jr_ccc_rel(bus, address, "C".to_string()),

        // Calls and Returns
        0xCD => call_im16(bus, address),
        0xC4 => call_ccc_im16(bus, address, "NZ".to_string()),
        0xCC => call_ccc_im16(bus, address, "Z".to_string()),
        0xD4 => call_ccc_im16(bus, address, "NC".to_string()),
        0xDC => call_ccc_im16(bus, address, "C".to_string()),
        0xC9 => (vec![0xC9], "RET".to_string(), "".to_string()),
        0xC0 => (vec![0xC0], "RET".to_string(), "NZ".to_string()),
        0xC8 => (vec![0xC8], "RET".to_string(), "Z".to_string()),
        0xD0 => (vec![0xD0], "RET".to_string(), "NC".to_string()),
        0xD8 => (vec![0xD8], "RET".to_string(), "C".to_string()),
        0xD9 => (vec![0xD9], "RETI".to_string(), "".to_string()),

        // Restart (RST)
        0xC7 => rst(bus, address, 0x00),
        0xCF => rst(bus, address, 0x08),
        0xD7 => rst(bus, address, 0x10),
        0xDF => rst(bus, address, 0x18),
        0xE7 => rst(bus, address, 0x20),
        0xEF => rst(bus, address, 0x28),
        0xF7 => rst(bus, address, 0x30),
        0xFF => rst(bus, address, 0x38),

        // CPU control
        0x00 => (vec![0x00], "NOP".to_string(), "".to_string()),
        0x76 => (vec![0x76], "HALT".to_string(), "".to_string()),
        0x10 => stop(bus, address),
        0xF3 => (vec![0xF3], "DI".to_string(), "".to_string()),
        0xFB => (vec![0xFB], "EI".to_string(), "".to_string()),

        // Add HL
        0x09 => add_hl_rp("BC"),
        0x19 => add_hl_rp("DE"),
        0x29 => add_hl_rp("HL"),
        0x39 => add_hl_rp("SP"),

        // SP offset
        0xE8 => sp_offset(bus, address),
        0xF8 => sp_offset_hl(bus, address),

        // CB prefix instructions
        0xCB => cb_prefix(bus, address),

        // Unknown
        0xDD | 0xED | 0xFD => (vec![opcode], "UNIMPLEMENTED".to_string(), format!("{:02X}", opcode)),

        // Default
        _ => (vec![opcode], "DB".to_string(), format!("{:02X}", opcode)),
    };

    DisassembledInstruction {
        address,
        bytes,
        mnemonic,
        operand_str,
    }
}

// Helper functions for instruction formatting

fn load_r_d8(bus: &impl MemoryRead, address: u16, reg: &str) -> (Vec<u8>, String, String) {
    let value = bus.read_byte(address + 1);
    (vec![bus.read_byte(address), value], "LD".to_string(), format!("{}, {:02X}", reg, value))
}

fn load_hl_d8(bus: &impl MemoryRead, address: u16) -> (Vec<u8>, String, String) {
    let value = bus.read_byte(address + 1);
    (vec![bus.read_byte(address), value], "LD".to_string(), format!("(HL), {:02X}", value))
}

fn load_rp_dd(bus: &impl MemoryRead, address: u16, reg: &str) -> (Vec<u8>, String, String) {
    let lo = bus.read_byte(address + 1);
    let hi = bus.read_byte(address + 2);
    let value = (hi as u16) << 8 | lo as u16;
    (vec![bus.read_byte(address), lo, hi], "LD".to_string(), format!("{}, {:04X}", reg, value))
}

fn ldh_a_d8(bus: &impl MemoryRead, address: u16) -> (Vec<u8>, String, String) {
    let value = bus.read_byte(address + 1);
    (vec![bus.read_byte(address), value], "LDH".to_string(), format!("({:02X}), A", value))
}

fn ldh_d8_a(bus: &impl MemoryRead, address: u16) -> (Vec<u8>, String, String) {
    let value = bus.read_byte(address + 1);
    (vec![bus.read_byte(address), value], "LDH".to_string(), format!("{:02X}, A", value))
}

fn ld_abus_a(bus: &impl MemoryRead, address: u16) -> (Vec<u8>, String, String) {
    let lo = bus.read_byte(address + 1);
    let hi = bus.read_byte(address + 2);
    let addr = (hi as u16) << 8 | lo as u16;
    (vec![bus.read_byte(address), lo, hi], "LD".to_string(), format!("({:04X}), A", addr))
}

fn ld_a_abus(bus: &impl MemoryRead, address: u16) -> (Vec<u8>, String, String) {
    let lo = bus.read_byte(address + 1);
    let hi = bus.read_byte(address + 2);
    let addr = (hi as u16) << 8 | lo as u16;
    (vec![bus.read_byte(address), lo, hi], "LD".to_string(), format!("A, {:04X}", addr))
}

fn alu_r(opcode: u8, mnemonic: &str) -> (Vec<u8>, String, String) {
    let reg = reg_name(opcode as usize & 7);
    (vec![opcode], mnemonic.to_string(), format!("A, {}", reg))
}

fn alu_d8(bus: &impl MemoryRead, address: u16, mnemonic: &str) -> (Vec<u8>, String, String) {
    let value = bus.read_byte(address + 1);
    (vec![bus.read_byte(address), value], mnemonic.to_string(), format!("A, {:02X}", value))
}

fn jp_ccc_im16(bus: &impl MemoryRead, address: u16, ccc: String) -> (Vec<u8>, String, String) {
    let lo = bus.read_byte(address + 1);
    let hi = bus.read_byte(address + 2);
    let addr = (hi as u16) << 8 | lo as u16;
    let bytes = vec![bus.read_byte(address), lo, hi];
    if ccc.is_empty() {
        (bytes, "JP".to_string(), format!("{:04X}", addr))
    } else {
        (bytes, "JP".to_string(), format!("{}, {:04X}", ccc, addr))
    }
}

fn jr_ccc_rel(bus: &impl MemoryRead, address: u16, ccc: String) -> (Vec<u8>, String, String) {
    let offset = bus.read_byte(address + 1) as i8;
    let target = (address as i16 + 2 + (offset as i16)) as u16;
    let bytes = vec![bus.read_byte(address), bus.read_byte(address + 1)];
    if ccc.is_empty() {
        (bytes, "JR".to_string(), format!("#{:04X}", target))
    } else {
        (bytes, "JR".to_string(), format!("{}, #{:04X}", ccc, target))
    }
}

fn call_im16(bus: &impl MemoryRead, address: u16) -> (Vec<u8>, String, String) {
    let lo = bus.read_byte(address + 1);
    let hi = bus.read_byte(address + 2);
    let addr = (hi as u16) << 8 | lo as u16;
    (vec![bus.read_byte(address), lo, hi], "CALL".to_string(), format!("{:04X}", addr))
}

fn call_ccc_im16(bus: &impl MemoryRead, address: u16, ccc: String) -> (Vec<u8>, String, String) {
    let lo = bus.read_byte(address + 1);
    let hi = bus.read_byte(address + 2);
    let addr = (hi as u16) << 8 | lo as u16;
    (vec![bus.read_byte(address), lo, hi], "CALL".to_string(), format!("{}, {:04X}", ccc, addr))
}

fn rst(bus: &impl MemoryRead, address: u16, vector: u8) -> (Vec<u8>, String, String) {
    (vec![bus.read_byte(address)], "RST".to_string(), format!("{:02X}", vector))
}

fn stop(bus: &impl MemoryRead, address: u16) -> (Vec<u8>, String, String) {
    let second = bus.read_byte(address + 1);
    (vec![bus.read_byte(address), second], "STOP".to_string(), "".to_string())
}

fn add_hl_rp(reg: &str) -> (Vec<u8>, String, String) {
    (vec![0x09], "ADD".to_string(), format!("HL, {}", reg))
}

fn sp_offset(bus: &impl MemoryRead, address: u16) -> (Vec<u8>, String, String) {
    let value = bus.read_byte(address + 1) as i8;
    let target = (address as i16 + 2 + (value as i16)) as u16;
    (vec![bus.read_byte(address), bus.read_byte(address + 1)], "ADD".to_string(), format!("SP, #{:04X}", target))
}

fn sp_offset_hl(bus: &impl MemoryRead, address: u16) -> (Vec<u8>, String, String) {
    let value = bus.read_byte(address + 1) as i8;
    (vec![bus.read_byte(address), bus.read_byte(address + 1)], "LD".to_string(), format!("HL, SP+#{:02X}", value))
}

fn cb_prefix(bus: &impl MemoryRead, address: u16) -> (Vec<u8>, String, String) {
    let cb_opcode = bus.read_byte(address + 1);
    let reg = reg_name(cb_opcode as usize & 7);
    let (op_name, operand) = match cb_opcode {
        0x00..=0x07 => ("RLC", format!("{}", reg)),
        0x08..=0x0F => ("RRC", format!("{}", reg)),
        0x10..=0x17 => ("RL", format!("{}", reg)),
        0x18..=0x1F => ("RR", format!("{}", reg)),
        0x20..=0x27 => ("SLA", format!("{}", reg)),
        0x28..=0x2F => ("SRA", format!("{}", reg)),
        0x30..=0x37 => ("SWAP", format!("{}", reg)),
        0x38..=0x3F => ("SRL", format!("{}", reg)),
        0x40..=0x7F => {
            let bit = (cb_opcode >> 3) & 0x07;
            ("BIT", format!("{}, {}", bit, reg))
        }
        0x80..=0xBF => {
            let bit = ((cb_opcode - 0x80) >> 3) & 0x07;
            ("RES", format!("{}, {}", bit, reg))
        }
        0xC0..=0xFF => {
            let bit = ((cb_opcode - 0xC0) >> 3) & 0x07;
            ("SET", format!("{}, {}", bit, reg))
        }
    };
    (vec![bus.read_byte(address), cb_opcode], op_name.to_string(), operand)
}

/// Trait for memory read operations
pub trait MemoryRead {
    fn read_byte(&self, address: u16) -> u8;
}

impl MemoryRead for GameBoyMemoryBus {
    fn read_byte(&self, address: u16) -> u8 {
        self.read(address)
    }
}

/// Disassemble a region of memory
pub fn disasm_region(bus: &impl MemoryRead, start: u16, count: usize) -> Vec<DisassembledInstruction> {
    let mut result = Vec::with_capacity(count);
    let mut addr = start;

    for _ in 0..count {
        let instr = disasm_one(bus, addr);
        let instr_bytes = instr.bytes.len();
        if instr_bytes == 0 {
            // Defensive guard: prevent infinite loop if instruction has zero bytes
            break;
        }
        result.push(instr);
        addr = addr.wrapping_add(instr_bytes as u16);
    }

    result
}

#[cfg(test)]
mod tests {
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
}
