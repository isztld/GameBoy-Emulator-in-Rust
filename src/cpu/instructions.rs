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
    LD_R16_IMM16 { dest: R16Register, value: u16 },
    LD_IND_R16_A { src: R16Mem },
    LD_A_IND_R16 { dest: R16Mem },
    LD_IND_IMM16_SP { address: u16 },
    INC_R16 { reg: R16Register },
    DEC_R16 { reg: R16Register },
    ADD_HL_R16 { reg: R16Register },
    INC_R8 { reg: R8Register },
    DEC_R8 { reg: R8Register },
    LD_R8_IMM8 { dest: R8Register, value: u8 },
    RLCA,
    RRCA,
    RLA,
    RRA,
    DAA,
    CPL,
    SCF,
    CCF,
    JR_IMM8 { offset: i8 },
    JR_COND_IMM8 { cond: Condition, offset: i8 },
    STOP,
    HALT,

    // Block 1 instructions
    LD_R8_R8 { dest: R8Register, src: R8Register },

    // Block 2 instructions
    ADD_A_R8 { reg: R8Register },
    ADC_A_R8 { reg: R8Register },
    SUB_A_R8 { reg: R8Register },
    SBC_A_R8 { reg: R8Register },
    AND_A_R8 { reg: R8Register },
    XOR_A_R8 { reg: R8Register },
    OR_A_R8 { reg: R8Register },
    CP_A_R8 { reg: R8Register },

    // Block 3 instructions
    ADD_A_IMM8 { value: u8 },
    ADC_A_IMM8 { value: u8 },
    SUB_A_IMM8 { value: u8 },
    SBC_A_IMM8 { value: u8 },
    AND_A_IMM8 { value: u8 },
    XOR_A_IMM8 { value: u8 },
    OR_A_IMM8 { value: u8 },
    CP_A_IMM8 { value: u8 },
    RET_COND { cond: Condition },
    RET,
    RETI,
    JP_COND_IMM16 { cond: Condition, address: u16 },
    JP_IMM16 { address: u16 },
    JP_HL,
    CALL_COND_IMM16 { cond: Condition, address: u16 },
    CALL_IMM16 { address: u16 },
    RST { target: u8 },
    POP_R16 { reg: R16Register },
    PUSH_R16 { reg: R16Register },
    LDH_IND_C_A,
    LDH_IND_IMM8_A { address: u8 },
    LD_IND_IMM16_A { address: u16 },
    LDH_A_IND_C,
    LDH_A_IND_IMM8 { address: u8 },
    LD_A_IND_IMM16 { address: u16 },
    ADD_SP_IMM8 { value: i8 },
    LD_HL_SP_IMM8 { value: i8 },
    LD_SP_HL,
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
        match (byte >> 4) & 0x03 {
            0x00 => R16Register::BC,
            0x01 => R16Register::DE,
            0x02 => R16Register::HL,
            0x03 => R16Register::SP,
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
    RLC_R8 { reg: R8Register },
    RRC_R8 { reg: R8Register },
    RL_R8 { reg: R8Register },
    RR_R8 { reg: R8Register },
    SLA_R8 { reg: R8Register },
    SRA_R8 { reg: R8Register },
    SWAP_R8 { reg: R8Register },
    SRL_R8 { reg: R8Register },
    BIT_B_R8 { bit: u8, reg: R8Register },
    RES_B_R8 { bit: u8, reg: R8Register },
    SET_B_R8 { bit: u8, reg: R8Register },
}

impl CBInstruction {
    pub fn from_byte(_opcode: u8, cb_opcode: u8) -> Self {
        let reg = R8Register::from_byte(cb_opcode);
        match cb_opcode & 0xC7 {
            0x00 => CBInstruction::RLC_R8 { reg },
            0x01 => CBInstruction::RRC_R8 { reg },
            0x02 => CBInstruction::RL_R8 { reg },
            0x03 => CBInstruction::RR_R8 { reg },
            0x04 => CBInstruction::SLA_R8 { reg },
            0x05 => CBInstruction::SRA_R8 { reg },
            0x06 => CBInstruction::SWAP_R8 { reg },
            0x07 => CBInstruction::SRL_R8 { reg },
            0x40 => CBInstruction::BIT_B_R8 { bit: (cb_opcode >> 4) & 0x07, reg },
            0x80 => CBInstruction::RES_B_R8 { bit: (cb_opcode >> 4) & 0x07, reg },
            0xC0 => CBInstruction::SET_B_R8 { bit: (cb_opcode >> 4) & 0x07, reg },
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
        assert_eq!(R16Register::from_byte(0x31), R16Register::DE);
        assert_eq!(R16Register::from_byte(0x61), R16Register::HL);
        assert_eq!(R16Register::from_byte(0x71), R16Register::SP);
    }
}
