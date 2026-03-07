/// Data transfer instruction executors
use crate::memory::MemoryBus;
use crate::cpu::CPUState;
use crate::cpu::instructions::R16Register;
use crate::cpu::exec::register_utils::{set_r16, get_r8, set_r8};

/// Execute LD r16, d16
pub fn exec_ld_r16_imm16(cpu_state: &mut CPUState, dest: R16Register, value: u16, io: &mut [u8; 128], tick: &mut dyn FnMut(&mut [u8; 128])) -> u32 {
    set_r16(&mut cpu_state.registers, dest, value);
    tick(io);
    tick(io);
    3
}

/// Execute LD (r16), A
pub fn exec_ld_ind_r16_a(cpu_state: &mut CPUState, src: crate::cpu::instructions::R16Mem, bus: &mut MemoryBus, tick: &mut dyn FnMut(&mut [u8; 128])) -> u32 {
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
    tick(&mut bus.io);
    2
}

/// Execute LD A, (r16)
pub fn exec_ld_a_ind_r16(cpu_state: &mut CPUState, dest: crate::cpu::instructions::R16Mem, bus: &mut MemoryBus, tick: &mut dyn FnMut(&mut [u8; 128])) -> u32 {
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
    tick(&mut bus.io);
    2
}

/// Execute LD (a16), SP
pub fn exec_ld_ind_imm16_sp(cpu_state: &mut CPUState, address: u16, bus: &mut MemoryBus, tick: &mut dyn FnMut(&mut [u8; 128])) -> u32 {
    let sp = cpu_state.registers.sp;
    let lo = (sp & 0xFF) as u8;
    let hi = (sp >> 8) as u8;

    tick(&mut bus.io);
    tick(&mut bus.io);
    bus.write(address, lo);                // low byte first
    tick(&mut bus.io);
    bus.write(address.wrapping_add(1), hi); // high byte second
    tick(&mut bus.io);
    5
}

/// Execute LD (a16), A
pub fn exec_ld_ind_imm16_a(cpu_state: &mut CPUState, address: u16, bus: &mut MemoryBus, tick: &mut dyn FnMut(&mut [u8; 128])) -> u32 {
    tick(&mut bus.io);
    tick(&mut bus.io);
    bus.write(address, cpu_state.registers.a());
    tick(&mut bus.io);
    4
}

/// Execute LD A, (a16)
pub fn exec_ld_a_ind_imm16(cpu_state: &mut CPUState, address: u16, bus: &mut MemoryBus, tick: &mut dyn FnMut(&mut [u8; 128])) -> u32 {
    tick(&mut bus.io);
    tick(&mut bus.io);
    cpu_state.registers.set_a(bus.read(address));
    tick(&mut bus.io);
    4
}

/// Execute LD r8, d8
pub fn exec_ld_r8_imm8(
    cpu_state: &mut CPUState,
    bus: &mut MemoryBus,
    dest: crate::cpu::instructions::R8Register,
    value: u8,
    tick: &mut dyn FnMut(&mut [u8; 128]),
) -> u32 {
    match dest {
        crate::cpu::instructions::R8Register::HL => {
            tick(&mut bus.io); // simulated imm read
            set_r8(&mut cpu_state.registers, bus, dest, value);
            tick(&mut bus.io); // write to (HL)
            3
        }
        _ => {
            tick(&mut bus.io); // simulated imm read
            set_r8(&mut cpu_state.registers, bus, dest, value);
            2
        }
    }
}

/// Execute LD r8, r8
pub fn exec_ld_r8_r8(
    cpu_state: &mut CPUState,
    bus: &mut MemoryBus,
    dest: crate::cpu::instructions::R8Register,
    src: crate::cpu::instructions::R8Register,
    tick: &mut dyn FnMut(&mut [u8; 128]),
) -> u32 {
    let val = get_r8(&cpu_state.registers, bus, src);
    if src == crate::cpu::instructions::R8Register::HL {
        tick(&mut bus.io);
    }
    set_r8(&mut cpu_state.registers, bus, dest, val);
    if dest == crate::cpu::instructions::R8Register::HL {
        tick(&mut bus.io);
    }
    match (dest, src) {
        // LD (HL), r8 or LD r8, (HL) — 2 machine cycles
        (crate::cpu::instructions::R8Register::HL, _)
        | (_, crate::cpu::instructions::R8Register::HL) => 2,
        // LD r8, r8 — 1 machine cycle
        _ => 1,
    }
}

#[cfg(test)]
#[path = "data_transfer_tests.rs"]
mod tests;
