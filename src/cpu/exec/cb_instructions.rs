/// CB-prefixed instruction executors
use crate::memory::MemoryBus;
use crate::cpu::CPUState;
use crate::cpu::instructions::CBInstruction;
use crate::cpu::instructions::R8Register;
use crate::cpu::exec::register_utils::{get_r8, set_r8};

/// Execute a CB-prefixed instruction
pub fn execute_cb(cpu_state: &mut CPUState, bus: &mut MemoryBus, cb_instr: CBInstruction, tick: &mut dyn FnMut(&mut [u8; 128])) -> u32 {
    match cb_instr {
        CBInstruction::RLCR8 { reg } => exec_rlcr8(cpu_state, bus, reg, tick),
        CBInstruction::RRCR8 { reg } => exec_rrcr8(cpu_state, bus, reg, tick),
        CBInstruction::RLR8 { reg } => exec_rlr8(cpu_state, bus, reg, tick),
        CBInstruction::RRR8 { reg } => exec_rrr8(cpu_state, bus, reg, tick),
        CBInstruction::SLAR8 { reg } => exec_slar8(cpu_state, bus, reg, tick),
        CBInstruction::SRAR8 { reg } => exec_srar8(cpu_state, bus, reg, tick),
        CBInstruction::SWAPR8 { reg } => exec_swapr8(cpu_state, bus, reg, tick),
        CBInstruction::SRLR8 { reg } => exec_srlr8(cpu_state, bus, reg, tick),
        CBInstruction::BITBR8 { bit, reg } => exec_bitbr8(cpu_state, bus, bit, reg, tick),
        CBInstruction::RESBR8 { bit, reg } => exec_resbr8(cpu_state, bus, bit, reg, tick),
        CBInstruction::SETBR8 { bit, reg } => exec_setbr8(cpu_state, bus, bit, reg, tick),
    }
}

/// Execute RLC r8
pub fn exec_rlcr8(cpu_state: &mut CPUState, bus: &mut MemoryBus, reg: R8Register, tick: &mut dyn FnMut(&mut [u8; 128])) -> u32 {
    tick(&mut bus.io); // CB opcode byte read
    let val = get_r8(&cpu_state.registers, bus, reg);
    if reg == R8Register::HL { tick(&mut bus.io); } // HL read
    let new_val = val.rotate_left(1);
    set_r8(&mut cpu_state.registers, bus, reg, new_val);
    if reg == R8Register::HL { tick(&mut bus.io); } // HL write
    cpu_state.registers.modify_f(|f| f.set_carry((val & 0x80) != 0));
    cpu_state.registers.modify_f(|f| f.set_zero(new_val == 0));
    cpu_state.registers.modify_f(|f| f.set_subtraction(false));
    cpu_state.registers.modify_f(|f| f.set_half_carry(false));
    if reg == R8Register::HL { 4 } else { 2 }
}

/// Execute RRC r8
pub fn exec_rrcr8(cpu_state: &mut CPUState, bus: &mut MemoryBus, reg: R8Register, tick: &mut dyn FnMut(&mut [u8; 128])) -> u32 {
    tick(&mut bus.io); // CB opcode byte read
    let val = get_r8(&cpu_state.registers, bus, reg);
    if reg == R8Register::HL { tick(&mut bus.io); } // HL read
    let new_val = val.rotate_right(1);
    set_r8(&mut cpu_state.registers, bus, reg, new_val);
    if reg == R8Register::HL { tick(&mut bus.io); } // HL write
    cpu_state.registers.modify_f(|f| f.set_carry((val & 0x01) != 0));
    cpu_state.registers.modify_f(|f| f.set_zero(new_val == 0));
    cpu_state.registers.modify_f(|f| f.set_subtraction(false));
    cpu_state.registers.modify_f(|f| f.set_half_carry(false));
    if reg == R8Register::HL { 4 } else { 2 }
}

/// Execute RL r8
pub fn exec_rlr8(cpu_state: &mut CPUState, bus: &mut MemoryBus, reg: R8Register, tick: &mut dyn FnMut(&mut [u8; 128])) -> u32 {
    tick(&mut bus.io); // CB opcode byte read
    let val = get_r8(&cpu_state.registers, bus, reg);
    if reg == R8Register::HL { tick(&mut bus.io); } // HL read
    let old_c = cpu_state.registers.f().is_carry() as u8;
    let new_val = (val << 1) | old_c;
    set_r8(&mut cpu_state.registers, bus, reg, new_val);
    if reg == R8Register::HL { tick(&mut bus.io); } // HL write
    cpu_state.registers.modify_f(|f| f.set_carry((val & 0x80) != 0));
    cpu_state.registers.modify_f(|f| f.set_zero(new_val == 0));
    cpu_state.registers.modify_f(|f| f.set_subtraction(false));
    cpu_state.registers.modify_f(|f| f.set_half_carry(false));
    if reg == R8Register::HL { 4 } else { 2 }
}

/// Execute RR r8
pub fn exec_rrr8(cpu_state: &mut CPUState, bus: &mut MemoryBus, reg: R8Register, tick: &mut dyn FnMut(&mut [u8; 128])) -> u32 {
    tick(&mut bus.io); // CB opcode byte read
    let val = get_r8(&cpu_state.registers, bus, reg);
    if reg == R8Register::HL { tick(&mut bus.io); } // HL read
    let old_c = cpu_state.registers.f().is_carry() as u8;
    let new_val = (val >> 1) | (old_c << 7);
    set_r8(&mut cpu_state.registers, bus, reg, new_val);
    if reg == R8Register::HL { tick(&mut bus.io); } // HL write
    cpu_state.registers.modify_f(|f| f.set_carry((val & 0x01) != 0));
    cpu_state.registers.modify_f(|f| f.set_zero(new_val == 0));
    cpu_state.registers.modify_f(|f| f.set_subtraction(false));
    cpu_state.registers.modify_f(|f| f.set_half_carry(false));
    if reg == R8Register::HL { 4 } else { 2 }
}

/// Execute SLA r8
pub fn exec_slar8(cpu_state: &mut CPUState, bus: &mut MemoryBus, reg: R8Register, tick: &mut dyn FnMut(&mut [u8; 128])) -> u32 {
    tick(&mut bus.io); // CB opcode byte read
    let val = get_r8(&cpu_state.registers, bus, reg);
    if reg == R8Register::HL { tick(&mut bus.io); } // HL read
    let new_val = val << 1;
    set_r8(&mut cpu_state.registers, bus, reg, new_val);
    if reg == R8Register::HL { tick(&mut bus.io); } // HL write
    cpu_state.registers.modify_f(|f| f.set_carry((val & 0x80) != 0));
    cpu_state.registers.modify_f(|f| f.set_zero(new_val == 0));
    cpu_state.registers.modify_f(|f| f.set_subtraction(false));
    cpu_state.registers.modify_f(|f| f.set_half_carry(false));
    if reg == R8Register::HL { 4 } else { 2 }
}

/// Execute SRA r8
pub fn exec_srar8(cpu_state: &mut CPUState, bus: &mut MemoryBus, reg: R8Register, tick: &mut dyn FnMut(&mut [u8; 128])) -> u32 {
    tick(&mut bus.io); // CB opcode byte read
    let val = get_r8(&cpu_state.registers, bus, reg);
    if reg == R8Register::HL { tick(&mut bus.io); } // HL read
    let new_val = (val as i8 >> 1) as u8;
    set_r8(&mut cpu_state.registers, bus, reg, new_val);
    if reg == R8Register::HL { tick(&mut bus.io); } // HL write
    cpu_state.registers.modify_f(|f| f.set_carry((val & 0x01) != 0));
    cpu_state.registers.modify_f(|f| f.set_zero(new_val == 0));
    cpu_state.registers.modify_f(|f| f.set_subtraction(false));
    cpu_state.registers.modify_f(|f| f.set_half_carry(false));
    if reg == R8Register::HL { 4 } else { 2 }
}

/// Execute SWAP r8
pub fn exec_swapr8(cpu_state: &mut CPUState, bus: &mut MemoryBus, reg: R8Register, tick: &mut dyn FnMut(&mut [u8; 128])) -> u32 {
    tick(&mut bus.io); // CB opcode byte read
    let val = get_r8(&cpu_state.registers, bus, reg);
    if reg == R8Register::HL { tick(&mut bus.io); } // HL read
    let new_val = (val >> 4) | (val << 4);
    set_r8(&mut cpu_state.registers, bus, reg, new_val);
    if reg == R8Register::HL { tick(&mut bus.io); } // HL write
    cpu_state.registers.modify_f(|f| f.set_zero(new_val == 0));
    cpu_state.registers.modify_f(|f| f.set_subtraction(false));
    cpu_state.registers.modify_f(|f| f.set_half_carry(false));
    cpu_state.registers.modify_f(|f| f.set_carry(false));
    if reg == R8Register::HL { 4 } else { 2 }
}

/// Execute SRL r8
pub fn exec_srlr8(cpu_state: &mut CPUState, bus: &mut MemoryBus, reg: R8Register, tick: &mut dyn FnMut(&mut [u8; 128])) -> u32 {
    tick(&mut bus.io); // CB opcode byte read
    let val = get_r8(&cpu_state.registers, bus, reg);
    if reg == R8Register::HL { tick(&mut bus.io); } // HL read
    let new_val = val >> 1;
    set_r8(&mut cpu_state.registers, bus, reg, new_val);
    if reg == R8Register::HL { tick(&mut bus.io); } // HL write
    cpu_state.registers.modify_f(|f| f.set_carry((val & 0x01) != 0));
    cpu_state.registers.modify_f(|f| f.set_zero(new_val == 0));
    cpu_state.registers.modify_f(|f| f.set_subtraction(false));
    cpu_state.registers.modify_f(|f| f.set_half_carry(false));
    if reg == R8Register::HL { 4 } else { 2 }
}

/// Execute BIT b, r8
pub fn exec_bitbr8(cpu_state: &mut CPUState, bus: &mut MemoryBus, bit: u8, reg: R8Register, tick: &mut dyn FnMut(&mut [u8; 128])) -> u32 {
    tick(&mut bus.io); // CB opcode byte read
    let val = get_r8(&cpu_state.registers, bus, reg);
    if reg == R8Register::HL { tick(&mut bus.io); } // HL read (no write for BIT)
    cpu_state.registers.modify_f(|f| f.set_zero(((val >> bit) & 1) == 0));
    cpu_state.registers.modify_f(|f| f.set_subtraction(false));
    cpu_state.registers.modify_f(|f| f.set_half_carry(true));
    if reg == R8Register::HL { 3 } else { 2 }
}

/// Execute RES b, r8
pub fn exec_resbr8(cpu_state: &mut CPUState, bus: &mut MemoryBus, bit: u8, reg: R8Register, tick: &mut dyn FnMut(&mut [u8; 128])) -> u32 {
    tick(&mut bus.io); // CB opcode byte read
    let mut val = get_r8(&cpu_state.registers, bus, reg);
    if reg == R8Register::HL { tick(&mut bus.io); } // HL read
    val &= !(1 << bit);
    set_r8(&mut cpu_state.registers, bus, reg, val);
    if reg == R8Register::HL { tick(&mut bus.io); } // HL write
    if reg == R8Register::HL { 4 } else { 2 }
}

/// Execute SET b, r8
pub fn exec_setbr8(cpu_state: &mut CPUState, bus: &mut MemoryBus, bit: u8, reg: R8Register, tick: &mut dyn FnMut(&mut [u8; 128])) -> u32 {
    tick(&mut bus.io); // CB opcode byte read
    let mut val = get_r8(&cpu_state.registers, bus, reg);
    if reg == R8Register::HL { tick(&mut bus.io); } // HL read
    val |= 1 << bit;
    set_r8(&mut cpu_state.registers, bus, reg, val);
    if reg == R8Register::HL { tick(&mut bus.io); } // HL write
    if reg == R8Register::HL { 4 } else { 2 }
}

#[cfg(test)]
#[path = "cb_instructions_tests.rs"]
mod tests;
