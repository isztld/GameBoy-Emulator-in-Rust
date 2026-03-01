/// Data transfer instruction executors
use crate::memory::MemoryBus;
use crate::cpu::CPUState;
use crate::cpu::instructions::R16Register;
use crate::cpu::exec::register_utils::{set_r16, get_r8, set_r8};

/// Execute LD r16, d16
pub fn exec_ld_r16_imm16(cpu_state: &mut CPUState, dest: R16Register, value: u16) -> u32 {
    set_r16(&mut cpu_state.registers, dest, value);
    3
}

/// Execute LD (r16), A
pub fn exec_ld_ind_r16_a(cpu_state: &mut CPUState, src: crate::cpu::instructions::R16Mem, bus: &mut MemoryBus) -> u32 {
    let addr = match src {
        crate::cpu::instructions::R16Mem::BC => cpu_state.registers.bc,
        crate::cpu::instructions::R16Mem::DE => cpu_state.registers.de,
        crate::cpu::instructions::R16Mem::HLPlus => {
            let addr = cpu_state.registers.hl;
            cpu_state.registers.hl = cpu_state.registers.hl.wrapping_add(1);
            addr
        }
        crate::cpu::instructions::R16Mem::HLMinus => {
            let addr = cpu_state.registers.hl;
            cpu_state.registers.hl = cpu_state.registers.hl.wrapping_sub(1);
            addr
        }
    };
    bus.write(addr, cpu_state.registers.a());
    2
}

/// Execute LD A, (r16)
pub fn exec_ld_a_ind_r16(cpu_state: &mut CPUState, dest: crate::cpu::instructions::R16Mem, bus: &mut MemoryBus) -> u32 {
    let addr = match dest {
        crate::cpu::instructions::R16Mem::BC => cpu_state.registers.bc,
        crate::cpu::instructions::R16Mem::DE => cpu_state.registers.de,
        crate::cpu::instructions::R16Mem::HLPlus => {
            let addr = cpu_state.registers.hl;
            cpu_state.registers.hl = cpu_state.registers.hl.wrapping_add(1);
            addr
        }
        crate::cpu::instructions::R16Mem::HLMinus => {
            let addr = cpu_state.registers.hl;
            cpu_state.registers.hl = cpu_state.registers.hl.wrapping_sub(1);
            addr
        }
    };
    cpu_state.registers.set_a(bus.read(addr));
    2
}

/// Execute LD (a16), SP
pub fn exec_ld_ind_imm16_sp(cpu_state: &mut CPUState, address: u16, bus: &mut MemoryBus) -> u32 {
    bus.write(address, (cpu_state.registers.sp >> 8) as u8);
    bus.write(address.wrapping_add(1), (cpu_state.registers.sp & 0xFF) as u8);
    5
}

/// Execute LD (a16), A
pub fn exec_ld_ind_imm16_a(cpu_state: &mut CPUState, address: u16, bus: &mut MemoryBus) -> u32 {
    bus.write(address, cpu_state.registers.a());
    4
}

/// Execute LD A, (a16)
pub fn exec_ld_a_ind_imm16(cpu_state: &mut CPUState, address: u16, bus: &mut MemoryBus) -> u32 {
    cpu_state.registers.set_a(bus.read(address));
    4
}

/// Execute LD r8, d8
pub fn exec_ld_r8_imm8(cpu_state: &mut CPUState, bus: &mut MemoryBus, dest: crate::cpu::instructions::R8Register, value: u8) -> u32 {
    set_r8(&mut cpu_state.registers, bus, dest, value);
    2
}

/// Execute LD r8, r8
pub fn exec_ld_r8_r8(cpu_state: &mut CPUState, bus: &mut MemoryBus, dest: crate::cpu::instructions::R8Register, src: crate::cpu::instructions::R8Register) -> u32 {
    let val = get_r8(&cpu_state.registers, bus, src);
    set_r8(&mut cpu_state.registers, bus, dest, val);
    1
}
