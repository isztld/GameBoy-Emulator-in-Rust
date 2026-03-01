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
#[derive(Debug, Clone, Copy)]
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
        }
    }
}

/// R16Mem: 16-bit memory operands for LD instructions
#[derive(Debug, Clone, Copy)]
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
#[derive(Debug, Clone, Copy)]
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
#[derive(Debug, Clone, Copy)]
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
#[derive(Debug, Clone, Copy)]
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
        match cb_opcode & 0xC7 {
            0x00 => CBInstruction::RLCR8 { reg },
            0x01 => CBInstruction::RRCR8 { reg },
            0x02 => CBInstruction::RLR8 { reg },
            0x03 => CBInstruction::RRR8 { reg },
            0x04 => CBInstruction::SLAR8 { reg },
            0x05 => CBInstruction::SRAR8 { reg },
            0x06 => CBInstruction::SWAPR8 { reg },
            0x07 => CBInstruction::SRLR8 { reg },
            0x40 => CBInstruction::BITBR8 { bit: (cb_opcode >> 4) & 0x07, reg },
            0x80 => CBInstruction::RESBR8 { bit: (cb_opcode >> 4) & 0x07, reg },
            0xC0 => CBInstruction::SETBR8 { bit: (cb_opcode >> 4) & 0x07, reg },
            _ => panic!("Invalid CB instruction: {:02X}", cb_opcode),
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
    fn test_r16_register() {
        assert_eq!(R16Register::from_byte(0x01), R16Register::BC);
        assert_eq!(R16Register::from_byte(0x11), R16Register::DE);
        assert_eq!(R16Register::from_byte(0x21), R16Register::HL);
        assert_eq!(R16Register::from_byte(0x31), R16Register::SP);
    }
}
