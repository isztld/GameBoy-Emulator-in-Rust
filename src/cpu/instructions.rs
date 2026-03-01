/// GameBoy CPU Instruction Set
///
/// The GameBoy SM83 CPU has a CISC instruction set with variable-length instructions.
/// Instructions are grouped into blocks based on the first opcode byte.
///
/// Opcode blocks:
/// - Block 0: 00-3F (Control, loads, arithmetic)
/// - Block 1: 40-7F (8-bit register-to-register loads)
/// - Block 2: 80-BF (8-bit arithmetic)
/// - Block 3: C0-FF (Jumps, calls, returns, stack operations)
/// - CB prefix: CBB0-CBFF (Rotate, shift, bit operations)

/// Instruction formats
#[derive(Debug, Clone, Copy)]
pub enum InstructionFormat {
    None,      // 1 byte, no operands
    Imm8,      // 1 byte + 1 byte immediate
    Imm16,     // 1 byte + 2 bytes immediate (little-endian)
    R8,        // 1 byte + r8 operand (encoded in instruction)
    R16,       // 1 byte + r16 operand
    R16Mem,    // 1 byte + r16mem operand (bc, de, hl+, hl-)
    Cond,      // 1 byte + cond operand (nz, z, nc, c)
    R16Stk,    // 1 byte + r16stk operand (bc, de, hl, af)
}

/// CPU Instruction
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Instruction {
    // Block 0 instructions
    NOP,
    LdR16Imm16 { dest: R16Register, value: u16 },
    LdIndR16A { src: R16Mem },
    LdAIndR16 { dest: R16Mem },
    LdIndImm16Sp { address: u16 },
    IncR16 { reg: R16Register },
    DecR16 { reg: R16Register },
    AddHlR16 { reg: R16Register },
    IncR8 { reg: R8Register },
    DecR8 { reg: R8Register },
    LdR8Imm8 { dest: R8Register, value: u8 },
    RLCA,
    RRCA,
    RLA,
    RRA,
    DAA,
    CPL,
    SCF,
    CCF,
    JrImm8 { offset: i8 },
    JrCondImm8 { cond: Condition, offset: i8 },
    STOP,
    HALT,

    // Block 1 instructions
    LdR8R8 { dest: R8Register, src: R8Register },

    // Block 2 instructions
    AddAR8 { reg: R8Register },
    AdcAR8 { reg: R8Register },
    SubAR8 { reg: R8Register },
    SbcAR8 { reg: R8Register },
    AndAR8 { reg: R8Register },
    XorAR8 { reg: R8Register },
    OrAR8 { reg: R8Register },
    CpAR8 { reg: R8Register },

    // Block 3 instructions
    AddAImm8 { value: u8 },
    AdcAImm8 { value: u8 },
    SubAImm8 { value: u8 },
    SbcAImm8 { value: u8 },
    AndAImm8 { value: u8 },
    XorAImm8 { value: u8 },
    OrAImm8 { value: u8 },
    CpAImm8 { value: u8 },
    RetCond { cond: Condition },
    RET,
    RETI,
    JpCondImm16 { cond: Condition, address: u16 },
    JpImm16 { address: u16 },
    JpHl,
    CallCondImm16 { cond: Condition, address: u16 },
    CallImm16 { address: u16 },
    RST { target: u8 },
    PopR16 { reg: R16Register },
    PushR16 { reg: R16Register },
    LdhIndCA,
    LdhIndImm8A { address: u8 },
    LdIndImm16A { address: u16 },
    LdhAC,
    LdhAIndImm8 { address: u8 },
    LdAIndImm16 { address: u16 },
    AddSpImm8 { value: i8 },
    LdHlSpImm8 { value: i8 },
    LdSpHl,
    DI,
    EI,

    // CB-prefixed instructions
    CB { cb_instr: CBInstruction },
}

/// R8Register: 8-bit registers
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum R8Register {
    B,
    C,
    D,
    E,
    H,
    L,
    HL,
    A,
}

impl R8Register {
    pub fn from_byte(byte: u8) -> Self {
        match byte & 0x07 {
            0x00 => R8Register::B,
            0x01 => R8Register::C,
            0x02 => R8Register::D,
            0x03 => R8Register::E,
            0x04 => R8Register::H,
            0x05 => R8Register::L,
            0x06 => R8Register::HL,
            0x07 => R8Register::A,
            _ => unreachable!(),
        }
    }

    pub fn to_byte(&self) -> u8 {
        match self {
            R8Register::B => 0x00,
            R8Register::C => 0x01,
            R8Register::D => 0x02,
            R8Register::E => 0x03,
            R8Register::H => 0x04,
            R8Register::L => 0x05,
            R8Register::HL => 0x06,
            R8Register::A => 0x07,
        }
    }
}

/// R16Register: 16-bit registers
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum R16Register {
    BC,
    DE,
    HL,
    SP,
    AF,
}

impl R16Register {
    pub fn from_byte(byte: u8) -> Self {
        match byte & 0x30 {
            0x00 => R16Register::BC,
            0x10 => R16Register::DE,
            0x20 => R16Register::HL,
            0x30 => R16Register::SP,
            _ => unreachable!(),
        }
    }

    pub fn to_byte(&self) -> u8 {
        match self {
            R16Register::BC => 0x00,
            R16Register::DE => 0x10,
            R16Register::HL => 0x20,
            R16Register::SP => 0x30,
            R16Register::AF => 0x30, // Same encoding as SP in some contexts
        }
    }
}

/// R16Mem: 16-bit memory operands for LD instructions
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum R16Mem {
    BC,
    DE,
    HLPlus, // HL++
    HLMinus, // HL--
}

impl R16Mem {
    pub fn from_byte(byte: u8) -> Self {
        match (byte >> 4) & 0x03 {
            0x00 => R16Mem::BC,
            0x01 => R16Mem::DE,
            0x02 => R16Mem::HLPlus,
            0x03 => R16Mem::HLMinus,
            _ => unreachable!(),
        }
    }
}

/// Condition codes for conditional instructions
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Condition {
    NZ, // Not zero
    Z,  // Zero
    NC, // Not carry
    C,  // Carry
}

impl Condition {
    pub fn from_byte(byte: u8) -> Self {
        match (byte >> 3) & 0x03 {
            0x00 => Condition::NZ,
            0x01 => Condition::Z,
            0x02 => Condition::NC,
            0x03 => Condition::C,
            _ => unreachable!(),
        }
    }

    pub fn to_byte(&self) -> u8 {
        match self {
            Condition::NZ => 0x00,
            Condition::Z => 0x08,
            Condition::NC => 0x10,
            Condition::C => 0x18,
        }
    }
}

/// R16Stk: 16-bit stack registers
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum R16Stk {
    BC,
    DE,
    HL,
    AF,
}

impl R16Stk {
    pub fn from_byte(byte: u8) -> Self {
        match (byte >> 4) & 0x03 {
            0x00 => R16Stk::BC,
            0x01 => R16Stk::DE,
            0x02 => R16Stk::HL,
            0x03 => R16Stk::AF,
            _ => unreachable!(),
        }
    }

    pub fn to_byte(&self) -> u8 {
        match self {
            R16Stk::BC => 0x00,
            R16Stk::DE => 0x10,
            R16Stk::HL => 0x20,
            R16Stk::AF => 0x30,
        }
    }
}

/// CB-prefixed instructions (rotate, shift, bit operations)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CBInstruction {
    RLCR8 { reg: R8Register },
    RRCR8 { reg: R8Register },
    RLR8 { reg: R8Register },
    RRR8 { reg: R8Register },
    SLAR8 { reg: R8Register },
    SRAR8 { reg: R8Register },
    SWAPR8 { reg: R8Register },
    SRLR8 { reg: R8Register },
    BITBR8 { bit: u8, reg: R8Register },
    RESBR8 { bit: u8, reg: R8Register },
    SETBR8 { bit: u8, reg: R8Register },
}

impl CBInstruction {
    pub fn from_byte(_opcode: u8, cb_opcode: u8) -> Self {
        let reg = R8Register::from_byte(cb_opcode);
        match cb_opcode & 0xF8 {
            0x00 => CBInstruction::RLCR8 { reg },
            0x08 => CBInstruction::RRCR8 { reg },
            0x10 => CBInstruction::RLR8 { reg },
            0x18 => CBInstruction::RRR8 { reg },
            0x20 => CBInstruction::SLAR8 { reg },
            0x28 => CBInstruction::SRAR8 { reg },
            0x30 => CBInstruction::SWAPR8 { reg },
            0x38 => CBInstruction::SRLR8 { reg },
            _ => {
                // BIT (0x40-0x7F), RES (0x80-0xBF), SET (0xC0-0xFF)
                // All have D7-D5 = 0b01x or 0b1xx, so we check D7 and D6
                let is_bit = (cb_opcode & 0xC0) == 0x40; // D7=0, D6=1
                let is_res = (cb_opcode & 0xC0) == 0x80; // D7=1, D6=0
                let is_set = (cb_opcode & 0xC0) == 0xC0; // D7=1, D6=1
                let bit = (cb_opcode >> 3) & 0x07;

                if is_bit {
                    CBInstruction::BITBR8 { bit, reg }
                } else if is_res {
                    CBInstruction::RESBR8 { bit, reg }
                } else if is_set {
                    CBInstruction::SETBR8 { bit, reg }
                } else {
                    panic!("Invalid CB instruction: {:02X}", cb_opcode);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_r8_register() {
        assert_eq!(R8Register::from_byte(0x00), R8Register::B);
        assert_eq!(R8Register::from_byte(0x07), R8Register::A);
        assert_eq!(R8Register::from_byte(0x46), R8Register::HL);
    }

    #[test]
    fn test_r8_register_to_byte() {
        assert_eq!(R8Register::B.to_byte(), 0x00);
        assert_eq!(R8Register::C.to_byte(), 0x01);
        assert_eq!(R8Register::D.to_byte(), 0x02);
        assert_eq!(R8Register::E.to_byte(), 0x03);
        assert_eq!(R8Register::H.to_byte(), 0x04);
        assert_eq!(R8Register::L.to_byte(), 0x05);
        assert_eq!(R8Register::HL.to_byte(), 0x06);
        assert_eq!(R8Register::A.to_byte(), 0x07);
    }

    #[test]
    fn test_r8_register_from_all_encoding_positions() {
        // Each register appears in multiple positions in the opcode
        // D7-D6 D5-D4 D2-D0 encode the register
        // For LD r8, r8: dddddd = dddddd (source and dest)

        // Test B register (ddd=000)
        assert_eq!(R8Register::from_byte(0x00), R8Register::B); // 00000000
        assert_eq!(R8Register::from_byte(0x40), R8Register::B); // 01000000 (LD B,B)
        assert_eq!(R8Register::from_byte(0x80), R8Register::B); // 10000000 (ADD A,B)
        assert_eq!(R8Register::from_byte(0xC0), R8Register::B); // 11000000 (RET NZ)

        // Test C register (ddd=001)
        assert_eq!(R8Register::from_byte(0x01), R8Register::C);
        assert_eq!(R8Register::from_byte(0x41), R8Register::C);
        assert_eq!(R8Register::from_byte(0x81), R8Register::C);
        assert_eq!(R8Register::from_byte(0xC1), R8Register::C);

        // Test A register (ddd=111)
        assert_eq!(R8Register::from_byte(0x07), R8Register::A);
        assert_eq!(R8Register::from_byte(0x47), R8Register::A);
        assert_eq!(R8Register::from_byte(0x87), R8Register::A);
        assert_eq!(R8Register::from_byte(0xC7), R8Register::A);
    }

    #[test]
    fn test_r16_register() {
        assert_eq!(R16Register::from_byte(0x01), R16Register::BC);
        assert_eq!(R16Register::from_byte(0x11), R16Register::DE);
        assert_eq!(R16Register::from_byte(0x21), R16Register::HL);
        assert_eq!(R16Register::from_byte(0x31), R16Register::SP);
    }

    #[test]
    fn test_r16_register_to_byte() {
        assert_eq!(R16Register::BC.to_byte(), 0x00);
        assert_eq!(R16Register::DE.to_byte(), 0x10);
        assert_eq!(R16Register::HL.to_byte(), 0x20);
        assert_eq!(R16Register::SP.to_byte(), 0x30);
    }

    #[test]
    fn test_r16_register_from_all_encoding_positions() {
        // LD r16,imm16 uses dd bits for register
        assert_eq!(R16Register::from_byte(0x01), R16Register::BC); // 00000001
        assert_eq!(R16Register::from_byte(0x11), R16Register::DE); // 00010001
        assert_eq!(R16Register::from_byte(0x21), R16Register::HL); // 00100001
        assert_eq!(R16Register::from_byte(0x31), R16Register::SP); // 00110001
    }

    #[test]
    fn test_r16mem() {
        assert_eq!(R16Mem::from_byte(0x00), R16Mem::BC);
        assert_eq!(R16Mem::from_byte(0x10), R16Mem::DE);
        assert_eq!(R16Mem::from_byte(0x20), R16Mem::HLPlus);
        assert_eq!(R16Mem::from_byte(0x30), R16Mem::HLMinus);
    }

    #[test]
    fn test_condition() {
        assert_eq!(Condition::from_byte(0xC0), Condition::NZ); // 11000000
        assert_eq!(Condition::from_byte(0xC8), Condition::Z);  // 11001000
        assert_eq!(Condition::from_byte(0xD0), Condition::NC); // 11010000
        assert_eq!(Condition::from_byte(0xD8), Condition::C);  // 11011000
    }

    #[test]
    fn test_condition_to_byte() {
        assert_eq!(Condition::NZ.to_byte(), 0x00);
        assert_eq!(Condition::Z.to_byte(), 0x08);
        assert_eq!(Condition::NC.to_byte(), 0x10);
        assert_eq!(Condition::C.to_byte(), 0x18);
    }

    #[test]
    fn test_r16stk() {
        assert_eq!(R16Stk::from_byte(0xC1), R16Stk::BC); // 11000001 (POP BC)
        assert_eq!(R16Stk::from_byte(0xD1), R16Stk::DE); // 11010001 (POP DE)
        assert_eq!(R16Stk::from_byte(0xE1), R16Stk::HL); // 11100001 (POP HL)
        assert_eq!(R16Stk::from_byte(0xF1), R16Stk::AF); // 11110001 (POP AF)
    }

    #[test]
    fn test_r16stk_to_byte() {
        assert_eq!(R16Stk::BC.to_byte(), 0x00);
        assert_eq!(R16Stk::DE.to_byte(), 0x10);
        assert_eq!(R16Stk::HL.to_byte(), 0x20);
        assert_eq!(R16Stk::AF.to_byte(), 0x30);
    }

    #[test]
    fn test_cb_instruction() {
        // RLC instructions
        match CBInstruction::from_byte(0xCB, 0x00) {
            CBInstruction::RLCR8 { reg } => assert_eq!(reg, R8Register::B),
            _ => panic!("Expected RLCR8"),
        }
        match CBInstruction::from_byte(0xCB, 0x07) {
            CBInstruction::RLCR8 { reg } => assert_eq!(reg, R8Register::A),
            _ => panic!("Expected RLCR8"),
        }

        // RRC instructions
        match CBInstruction::from_byte(0xCB, 0x08) {
            CBInstruction::RRCR8 { reg } => assert_eq!(reg, R8Register::B),
            _ => panic!("Expected RRCR8"),
        }
        match CBInstruction::from_byte(0xCB, 0x0F) {
            CBInstruction::RRCR8 { reg } => assert_eq!(reg, R8Register::A),
            _ => panic!("Expected RRCR8"),
        }

        // RL instructions
        match CBInstruction::from_byte(0xCB, 0x10) {
            CBInstruction::RLR8 { reg } => assert_eq!(reg, R8Register::B),
            _ => panic!("Expected RLR8"),
        }
        match CBInstruction::from_byte(0xCB, 0x17) {
            CBInstruction::RLR8 { reg } => assert_eq!(reg, R8Register::A),
            _ => panic!("Expected RLR8"),
        }

        // RR instructions
        match CBInstruction::from_byte(0xCB, 0x18) {
            CBInstruction::RRR8 { reg } => assert_eq!(reg, R8Register::B),
            _ => panic!("Expected RRR8"),
        }
        match CBInstruction::from_byte(0xCB, 0x1F) {
            CBInstruction::RRR8 { reg } => assert_eq!(reg, R8Register::A),
            _ => panic!("Expected RRR8"),
        }

        // SLA instructions
        match CBInstruction::from_byte(0xCB, 0x20) {
            CBInstruction::SLAR8 { reg } => assert_eq!(reg, R8Register::B),
            _ => panic!("Expected SLAR8"),
        }
        match CBInstruction::from_byte(0xCB, 0x27) {
            CBInstruction::SLAR8 { reg } => assert_eq!(reg, R8Register::A),
            _ => panic!("Expected SLAR8"),
        }

        // SRA instructions
        match CBInstruction::from_byte(0xCB, 0x28) {
            CBInstruction::SRAR8 { reg } => assert_eq!(reg, R8Register::B),
            _ => panic!("Expected SRAR8"),
        }
        match CBInstruction::from_byte(0xCB, 0x2F) {
            CBInstruction::SRAR8 { reg } => assert_eq!(reg, R8Register::A),
            _ => panic!("Expected SRAR8"),
        }

        // SWAP instructions
        match CBInstruction::from_byte(0xCB, 0x30) {
            CBInstruction::SWAPR8 { reg } => assert_eq!(reg, R8Register::B),
            _ => panic!("Expected SWAPR8"),
        }
        match CBInstruction::from_byte(0xCB, 0x37) {
            CBInstruction::SWAPR8 { reg } => assert_eq!(reg, R8Register::A),
            _ => panic!("Expected SWAPR8"),
        }

        // SRL instructions
        match CBInstruction::from_byte(0xCB, 0x38) {
            CBInstruction::SRLR8 { reg } => assert_eq!(reg, R8Register::B),
            _ => panic!("Expected SRLR8"),
        }
        match CBInstruction::from_byte(0xCB, 0x3F) {
            CBInstruction::SRLR8 { reg } => assert_eq!(reg, R8Register::A),
            _ => panic!("Expected SRLR8"),
        }

        // BIT instructions
        match CBInstruction::from_byte(0xCB, 0x40) {
            CBInstruction::BITBR8 { bit, reg } => {
                assert_eq!(bit, 0);
                assert_eq!(reg, R8Register::B);
            }
            _ => panic!("Expected BITBR8"),
        }
        match CBInstruction::from_byte(0xCB, 0x47) {
            CBInstruction::BITBR8 { bit, reg } => {
                assert_eq!(bit, 0);
                assert_eq!(reg, R8Register::A);
            }
            _ => panic!("Expected BITBR8"),
        }

        // RES instructions
        match CBInstruction::from_byte(0xCB, 0x80) {
            CBInstruction::RESBR8 { bit, reg } => {
                assert_eq!(bit, 0);
                assert_eq!(reg, R8Register::B);
            }
            _ => panic!("Expected RESBR8"),
        }

        // SET instructions
        match CBInstruction::from_byte(0xCB, 0xC0) {
            CBInstruction::SETBR8 { bit, reg } => {
                assert_eq!(bit, 0);
                assert_eq!(reg, R8Register::B);
            }
            _ => panic!("Expected SETBR8"),
        }
    }

    // ==================== CB INSTRUCTION TESTS ====================

    #[test]
    fn test_cb_all_rotate_shift_instructions() {
        // Test all rotate/shift instructions with each register
        for reg_byte in 0x00..=0x07 {
            let reg = R8Register::from_byte(reg_byte);

            // RLC
            match CBInstruction::from_byte(0xCB, reg_byte) {
                CBInstruction::RLCR8 { reg: r } => assert_eq!(r, reg),
                _ => panic!("Expected RLC for reg_byte={:02X}", reg_byte),
            }

            // RRC
            match CBInstruction::from_byte(0xCB, reg_byte | 0x08) {
                CBInstruction::RRCR8 { reg: r } => assert_eq!(r, reg),
                _ => panic!("Expected RRC for reg_byte={:02X}", reg_byte),
            }

            // RL
            match CBInstruction::from_byte(0xCB, reg_byte | 0x10) {
                CBInstruction::RLR8 { reg: r } => assert_eq!(r, reg),
                _ => panic!("Expected RL for reg_byte={:02X}", reg_byte),
            }

            // RR
            match CBInstruction::from_byte(0xCB, reg_byte | 0x18) {
                CBInstruction::RRR8 { reg: r } => assert_eq!(r, reg),
                _ => panic!("Expected RR for reg_byte={:02X}", reg_byte),
            }

            // SLA
            match CBInstruction::from_byte(0xCB, reg_byte | 0x20) {
                CBInstruction::SLAR8 { reg: r } => assert_eq!(r, reg),
                _ => panic!("Expected SLA for reg_byte={:02X}", reg_byte),
            }

            // SRA
            match CBInstruction::from_byte(0xCB, reg_byte | 0x28) {
                CBInstruction::SRAR8 { reg: r } => assert_eq!(r, reg),
                _ => panic!("Expected SRA for reg_byte={:02X}", reg_byte),
            }

            // SWAP
            match CBInstruction::from_byte(0xCB, reg_byte | 0x30) {
                CBInstruction::SWAPR8 { reg: r } => assert_eq!(r, reg),
                _ => panic!("Expected SWAP for reg_byte={:02X}", reg_byte),
            }

            // SRL
            match CBInstruction::from_byte(0xCB, reg_byte | 0x38) {
                CBInstruction::SRLR8 { reg: r } => assert_eq!(r, reg),
                _ => panic!("Expected SRL for reg_byte={:02X}", reg_byte),
            }
        }
    }

    #[test]
    fn test_cb_all_bit_instructions() {
        // Test BIT instructions with all bits (0-7) and all registers
        // BIT opcodes are 0x40-0x7F
        for bit in 0..8 {
            for reg_byte in 0x00..=0x07 {
                let reg = R8Register::from_byte(reg_byte);

                match CBInstruction::from_byte(0xCB, 0x40 | (bit << 3) | reg_byte) {
                    CBInstruction::BITBR8 { bit: b, reg: r } => {
                        assert_eq!(b, bit);
                        assert_eq!(r, reg);
                    }
                    _ => panic!("Expected BIT for bit={}, reg_byte={:02X}", bit, reg_byte),
                }
            }
        }
    }

    #[test]
    fn test_cb_all_res_instructions() {
        // Test RES instructions with all bits (0-7) and all registers
        for bit in 0..8 {
            for reg_byte in 0x00..=0x07 {
                let reg = R8Register::from_byte(reg_byte);

                match CBInstruction::from_byte(0xCB, 0x80 | (bit << 3) | reg_byte) {
                    CBInstruction::RESBR8 { bit: b, reg: r } => {
                        assert_eq!(b, bit);
                        assert_eq!(r, reg);
                    }
                    _ => panic!("Expected RES for bit={}, reg_byte={:02X}", bit, reg_byte),
                }
            }
        }
    }

    #[test]
    fn test_cb_all_set_instructions() {
        // Test SET instructions with all bits (0-7) and all registers
        for bit in 0..8 {
            for reg_byte in 0x00..=0x07 {
                let reg = R8Register::from_byte(reg_byte);

                match CBInstruction::from_byte(0xCB, 0xC0 | (bit << 3) | reg_byte) {
                    CBInstruction::SETBR8 { bit: b, reg: r } => {
                        assert_eq!(b, bit);
                        assert_eq!(r, reg);
                    }
                    _ => panic!("Expected SET for bit={}, reg_byte={:02X}", bit, reg_byte),
                }
            }
        }
    }
}
