//! Data movement instructions.

use crate::cpu::Cpu;
use crate::decoder::{reg16_from_encoding, ModRm, Operand16, Segment};
use crate::flags::Flags;
use crate::registers::{Reg16, Reg8};
use crate::StepError;

pub(super) fn mov_rm_reg(
    cpu: &mut Cpu,
    opcode: u8,
    segment_override: Option<Segment>,
) -> Result<(), StepError> {
    let modrm = ModRm::decode(cpu, segment_override);

    match opcode {
        0x88 => {
            let value = cpu.regs.get8(modrm.reg8());
            modrm.operand8().write(cpu, value);
        }
        0x89 => {
            let value = cpu.regs.get(modrm.reg16());
            modrm.operand16().write(cpu, value);
        }
        0x8A => {
            let value = modrm.operand8().read(cpu);
            cpu.regs.set8(modrm.reg8(), value);
        }
        0x8B => {
            let value = modrm.operand16().read(cpu);
            cpu.regs.set(modrm.reg16(), value);
        }
        _ => unreachable!(),
    }

    Ok(())
}

pub(super) fn mov_segment(
    cpu: &mut Cpu,
    opcode: u8,
    segment_override: Option<Segment>,
) -> Result<(), StepError> {
    let modrm = ModRm::decode(cpu, segment_override);
    let segment = Segment::from_encoding(modrm.reg).ok_or(StepError::InvalidOpcode(opcode))?;

    match opcode {
        0x8C => {
            let value = cpu.regs.get(segment.register());
            modrm.operand16().write(cpu, value);
        }
        0x8E => {
            if segment == Segment::Cs {
                return Err(StepError::InvalidOpcode(opcode));
            }
            let value = modrm.operand16().read(cpu);
            cpu.regs.set(segment.register(), value);
        }
        _ => unreachable!(),
    }

    Ok(())
}

pub(super) fn mov_moffs(
    cpu: &mut Cpu,
    opcode: u8,
    segment_override: Option<Segment>,
) -> Result<(), StepError> {
    let offset = cpu.fetch16();
    let segment = segment_override.unwrap_or(Segment::Ds);
    let segment_value = cpu.regs.get(segment.register());

    match opcode {
        0xA0 => {
            let value = cpu.mem.read8(segment_value, offset);
            cpu.regs.set8(Reg8::Al, value);
        }
        0xA1 => {
            let value = cpu.mem.read16(segment_value, offset);
            cpu.regs.ax = value;
        }
        0xA2 => cpu
            .mem
            .write8(segment_value, offset, cpu.regs.get8(Reg8::Al)),
        0xA3 => cpu.mem.write16(segment_value, offset, cpu.regs.ax),
        _ => unreachable!(),
    }

    Ok(())
}

pub(super) fn mov_imm8(cpu: &mut Cpu, opcode: u8) -> Result<(), StepError> {
    let reg = Reg8::ALL[(opcode & 0x07) as usize];
    let value = cpu.fetch8();
    cpu.regs.set8(reg, value);
    Ok(())
}

pub(super) fn mov_imm16(cpu: &mut Cpu, opcode: u8) -> Result<(), StepError> {
    let reg = reg16_from_encoding(opcode & 0x07);
    let value = cpu.fetch16();
    cpu.regs.set(reg, value);
    Ok(())
}

pub(super) fn mov_imm_rm(
    cpu: &mut Cpu,
    opcode: u8,
    segment_override: Option<Segment>,
) -> Result<(), StepError> {
    let modrm = ModRm::decode(cpu, segment_override);
    if modrm.reg != 0 {
        return Err(StepError::InvalidOpcode(opcode));
    }

    match opcode {
        0xC6 => {
            let value = cpu.fetch8();
            modrm.operand8().write(cpu, value);
        }
        0xC7 => {
            let value = cpu.fetch16();
            modrm.operand16().write(cpu, value);
        }
        _ => unreachable!(),
    }

    Ok(())
}

pub(super) fn xchg_rm_reg(
    cpu: &mut Cpu,
    opcode: u8,
    segment_override: Option<Segment>,
) -> Result<(), StepError> {
    let modrm = ModRm::decode(cpu, segment_override);

    match opcode {
        0x86 => {
            let operand = modrm.operand8();
            let register = modrm.reg8();
            let left = operand.read(cpu);
            let right = cpu.regs.get8(register);
            operand.write(cpu, right);
            cpu.regs.set8(register, left);
        }
        0x87 => {
            let operand = modrm.operand16();
            let register = modrm.reg16();
            let left = operand.read(cpu);
            let right = cpu.regs.get(register);
            operand.write(cpu, right);
            cpu.regs.set(register, left);
        }
        _ => unreachable!(),
    }

    Ok(())
}

pub(super) fn xchg_ax(cpu: &mut Cpu, opcode: u8) -> Result<(), StepError> {
    let register = reg16_from_encoding(opcode & 0x07);
    let value = cpu.regs.ax;
    cpu.regs.ax = cpu.regs.get(register);
    cpu.regs.set(register, value);
    Ok(())
}

pub(super) fn push_reg(cpu: &mut Cpu, opcode: u8) -> Result<(), StepError> {
    let register = reg16_from_encoding(opcode & 0x07);
    let value = if register == Reg16::Sp {
        cpu.regs.sp.wrapping_sub(2)
    } else {
        cpu.regs.get(register)
    };
    cpu.push16(value);
    Ok(())
}

pub(super) fn push_segment(cpu: &mut Cpu, opcode: u8) -> Result<(), StepError> {
    let segment = match opcode {
        0x06 => Segment::Es,
        0x0E => Segment::Cs,
        0x16 => Segment::Ss,
        0x1E => Segment::Ds,
        _ => unreachable!(),
    };
    let value = cpu.regs.get(segment.register());
    cpu.push16(value);
    Ok(())
}

pub(super) fn pop_reg(cpu: &mut Cpu, opcode: u8) -> Result<(), StepError> {
    let register = reg16_from_encoding(opcode & 0x07);
    let value = cpu.pop16();
    cpu.regs.set(register, value);
    Ok(())
}

pub(super) fn pop_segment(cpu: &mut Cpu, opcode: u8) -> Result<(), StepError> {
    let segment = match opcode {
        0x07 => Segment::Es,
        0x0F => Segment::Cs,
        0x17 => Segment::Ss,
        0x1F => Segment::Ds,
        _ => unreachable!(),
    };
    let value = cpu.pop16();
    cpu.regs.set(segment.register(), value);
    Ok(())
}

pub(super) fn pop_rm(cpu: &mut Cpu, segment_override: Option<Segment>) -> Result<(), StepError> {
    let modrm = ModRm::decode(cpu, segment_override);
    if modrm.reg != 0 {
        return Err(StepError::InvalidOpcode(0x8F));
    }

    let operand = modrm.operand16();
    let value = cpu.pop16();
    operand.write(cpu, value);
    Ok(())
}

pub(super) fn pushf(cpu: &mut Cpu) -> Result<(), StepError> {
    let value = cpu.regs.flags.bits();
    cpu.push16(value);
    Ok(())
}

pub(super) fn popf(cpu: &mut Cpu) -> Result<(), StepError> {
    let value = cpu.pop16();
    cpu.regs.flags = Flags::from_bits(value);
    Ok(())
}

pub(super) fn sahf(cpu: &mut Cpu) -> Result<(), StepError> {
    let value = u16::from(cpu.regs.get8(Reg8::Ah)) & 0xD5;
    cpu.regs.flags = Flags::from_bits((cpu.regs.flags.bits() & 0xFF00) | value);
    Ok(())
}

pub(super) fn lahf(cpu: &mut Cpu) -> Result<(), StepError> {
    let value = cpu.regs.flags.bits() as u8;
    cpu.regs.set8(Reg8::Ah, value);
    Ok(())
}

pub(super) fn lea(cpu: &mut Cpu, segment_override: Option<Segment>) -> Result<(), StepError> {
    let modrm = ModRm::decode(cpu, segment_override);
    let memory = modrm.memory().ok_or(StepError::InvalidOpcode(0x8D))?;
    cpu.regs.set(modrm.reg16(), memory.offset);
    Ok(())
}

pub(super) fn load_far_pointer(
    cpu: &mut Cpu,
    opcode: u8,
    segment_override: Option<Segment>,
) -> Result<(), StepError> {
    let modrm = ModRm::decode(cpu, segment_override);
    let memory = modrm.memory().ok_or(StepError::InvalidOpcode(opcode))?;
    let source_segment = cpu.regs.get(memory.segment.register());
    let offset = cpu.mem.read16(source_segment, memory.offset);
    let segment = cpu
        .mem
        .read16(source_segment, memory.offset.wrapping_add(2));

    cpu.regs.set(modrm.reg16(), offset);
    match opcode {
        0xC4 => cpu.regs.es = segment,
        0xC5 => cpu.regs.ds = segment,
        _ => unreachable!(),
    }

    Ok(())
}

pub(super) fn xlat(cpu: &mut Cpu, segment_override: Option<Segment>) -> Result<(), StepError> {
    let segment = segment_override.unwrap_or(Segment::Ds);
    let offset = cpu.regs.bx.wrapping_add(u16::from(cpu.regs.get8(Reg8::Al)));
    let value = cpu.mem.read8(cpu.regs.get(segment.register()), offset);
    cpu.regs.set8(Reg8::Al, value);
    Ok(())
}

pub(super) fn cbw(cpu: &mut Cpu) -> Result<(), StepError> {
    let value = cpu.regs.get8(Reg8::Al) as i8 as i16 as u16;
    cpu.regs.ax = value;
    Ok(())
}

pub(super) fn cwd(cpu: &mut Cpu) -> Result<(), StepError> {
    cpu.regs.dx = if cpu.regs.ax & 0x8000 != 0 { 0xFFFF } else { 0 };
    Ok(())
}

pub(super) fn push_operand16(cpu: &mut Cpu, operand: Operand16) {
    let value = match operand {
        Operand16::Reg(Reg16::Sp) => cpu.regs.sp.wrapping_sub(2),
        _ => operand.read(cpu),
    };
    cpu.push16(value);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu_with_code;
    use crate::cpu_with_code_at;

    #[test]
    fn mov_register_to_register_16() {
        let mut cpu = cpu_with_code(&[0x89, 0xD8]);
        cpu.regs.bx = 0x1234;
        cpu.step().unwrap();
        assert_eq!(cpu.regs.ax, 0x1234);
        assert_eq!(cpu.regs.ip, 0x0102);
    }

    #[test]
    fn mov_memory_to_register_16() {
        let mut cpu = cpu_with_code(&[0x8B, 0x07]);
        cpu.regs.bx = 0x0200;
        cpu.mem.write16(0, 0x0200, 0xBEEF);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.ax, 0xBEEF);
    }

    #[test]
    fn mov_register_to_memory_8() {
        let mut cpu = cpu_with_code(&[0x88, 0x07]);
        cpu.regs.ax = 0x1234;
        cpu.regs.bx = 0x0200;
        cpu.step().unwrap();
        assert_eq!(cpu.mem.read8(0, 0x0200), 0x34);
    }

    #[test]
    fn mov_high_byte_register() {
        let mut cpu = cpu_with_code(&[0x88, 0xDC]);
        cpu.regs.ax = 0xAA00;
        cpu.regs.bx = 0x0042;
        cpu.step().unwrap();
        assert_eq!(cpu.regs.ax, 0x4200);
    }

    #[test]
    fn mov_immediate_to_registers() {
        let mut cpu = cpu_with_code(&[0xB0, 0x7F, 0xBB, 0xEF, 0xBE]);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.get8(Reg8::Al), 0x7F);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.bx, 0xBEEF);
    }

    #[test]
    fn mov_immediate_to_memory() {
        let mut cpu = cpu_with_code(&[0xC6, 0x07, 0x12, 0xC7, 0x47, 0x02, 0x34, 0x12]);
        cpu.regs.bx = 0x0200;
        cpu.step().unwrap();
        assert_eq!(cpu.mem.read8(0, 0x0200), 0x12);
        cpu.step().unwrap();
        assert_eq!(cpu.mem.read16(0, 0x0202), 0x1234);
    }

    #[test]
    fn mov_moffs_round_trip() {
        let mut cpu = cpu_with_code(&[0xA2, 0x00, 0x02, 0xA0, 0x00, 0x02]);
        cpu.regs.set8(Reg8::Al, 0x5A);
        cpu.step().unwrap();
        assert_eq!(cpu.mem.read8(0, 0x0200), 0x5A);
        cpu.regs.set8(Reg8::Al, 0);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.get8(Reg8::Al), 0x5A);

        let mut cpu = cpu_with_code(&[0xA3, 0x00, 0x02, 0xA1, 0x00, 0x02]);
        cpu.regs.ax = 0xBEEF;
        cpu.step().unwrap();
        assert_eq!(cpu.mem.read16(0, 0x0200), 0xBEEF);
        cpu.regs.ax = 0;
        cpu.step().unwrap();
        assert_eq!(cpu.regs.ax, 0xBEEF);
    }

    #[test]
    fn mov_segment_registers() {
        let mut cpu = cpu_with_code(&[0x8C, 0xD8, 0x8E, 0xD8]);
        cpu.regs.ds = 0x1234;
        cpu.step().unwrap();
        assert_eq!(cpu.regs.ax, 0x1234);
        cpu.regs.ax = 0x5678;
        cpu.step().unwrap();
        assert_eq!(cpu.regs.ds, 0x5678);
    }

    #[test]
    fn mov_to_cs_is_invalid() {
        let mut cpu = cpu_with_code(&[0x8E, 0xC8]);
        assert_eq!(cpu.step(), Err(StepError::InvalidOpcode(0x8E)));
    }

    #[test]
    fn xchg_register_and_memory() {
        let mut cpu = cpu_with_code(&[0x87, 0x07]);
        cpu.regs.bx = 0x0200;
        cpu.regs.ax = 0x1111;
        cpu.mem.write16(0, 0x0200, 0x2222);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.ax, 0x2222);
        assert_eq!(cpu.mem.read16(0, 0x0200), 0x1111);

        let mut cpu = cpu_with_code(&[0x86, 0x07]);
        cpu.regs.bx = 0x0200;
        cpu.regs.set8(Reg8::Al, 0x11);
        cpu.mem.write8(0, 0x0200, 0x22);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.get8(Reg8::Al), 0x22);
        assert_eq!(cpu.mem.read8(0, 0x0200), 0x11);
    }

    #[test]
    fn xchg_ax_short_forms() {
        let mut cpu = cpu_with_code(&[0x93, 0x90]);
        cpu.regs.ax = 0x1111;
        cpu.regs.bx = 0x2222;
        cpu.step().unwrap();
        assert_eq!(cpu.regs.ax, 0x2222);
        assert_eq!(cpu.regs.bx, 0x1111);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.ax, 0x2222);
    }

    #[test]
    fn push_and_pop_registers() {
        let mut cpu = cpu_with_code(&[0x50, 0x5B]);
        cpu.regs.ss = 0;
        cpu.regs.sp = 0x1000;
        cpu.regs.ax = 0xBEEF;
        cpu.step().unwrap();
        assert_eq!(cpu.regs.sp, 0x0FFE);
        assert_eq!(cpu.mem.read16(0, 0x0FFE), 0xBEEF);
        cpu.regs.ax = 0;
        cpu.step().unwrap();
        assert_eq!(cpu.regs.bx, 0xBEEF);
        assert_eq!(cpu.regs.sp, 0x1000);
    }

    #[test]
    fn push_sp_pushes_decremented_value() {
        let mut cpu = cpu_with_code(&[0x54]);
        cpu.regs.ss = 0;
        cpu.regs.sp = 0x1000;
        cpu.step().unwrap();
        assert_eq!(cpu.regs.sp, 0x0FFE);
        assert_eq!(cpu.mem.read16(0, 0x0FFE), 0x0FFE);
    }

    #[test]
    fn pop_sp_loads_popped_value() {
        let mut cpu = cpu_with_code(&[0x5C]);
        cpu.regs.ss = 0;
        cpu.regs.sp = 0x0FFE;
        cpu.mem.write16(0, 0x0FFE, 0x2000);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.sp, 0x2000);
    }

    #[test]
    fn push_and_pop_segments() {
        for (push, pop, register) in [(0x06, 0x07, Reg16::Es), (0x1E, 0x1F, Reg16::Ds)] {
            let mut cpu = cpu_with_code(&[push, pop]);
            cpu.regs.ss = 0;
            cpu.regs.sp = 0x1000;
            cpu.regs.set(register, 0x1234);
            cpu.step().unwrap();
            cpu.regs.set(register, 0);
            cpu.step().unwrap();
            assert_eq!(cpu.regs.get(register), 0x1234);
        }
    }

    #[test]
    fn pushf_and_popf() {
        let mut cpu = cpu_with_code(&[0x9C, 0x9D]);
        cpu.regs.ss = 0;
        cpu.regs.sp = 0x1000;
        cpu.regs.flags.set(Flags::CF, true);
        cpu.regs.flags.set(Flags::ZF, true);
        let bits = cpu.regs.flags.bits();
        cpu.step().unwrap();
        assert_eq!(cpu.mem.read16(0, 0x0FFE), bits);
        cpu.regs.flags = Flags::power_on();
        cpu.step().unwrap();
        assert_eq!(cpu.regs.flags.bits(), bits);
    }

    #[test]
    fn lahf_and_sahf() {
        let mut cpu = cpu_with_code(&[0x9F, 0x9E]);
        cpu.regs.flags.set(Flags::CF, true);
        cpu.regs.flags.set(Flags::SF, true);
        let low = cpu.regs.flags.bits() as u8;
        cpu.step().unwrap();
        assert_eq!(cpu.regs.get8(Reg8::Ah), low);
        cpu.regs.flags = Flags::power_on();
        cpu.step().unwrap();
        assert!(cpu.regs.flags.contains(Flags::CF));
        assert!(cpu.regs.flags.contains(Flags::SF));
        assert!(!cpu.regs.flags.contains(Flags::ZF));
    }

    #[test]
    fn lea_uses_offset_only() {
        let mut cpu = cpu_with_code(&[0x8D, 0x47, 0x10]);
        cpu.regs.bx = 0x0200;
        cpu.regs.ds = 0x1000;
        cpu.step().unwrap();
        assert_eq!(cpu.regs.ax, 0x0210);
    }

    #[test]
    fn lea_register_operand_is_invalid() {
        let mut cpu = cpu_with_code(&[0x8D, 0xC0]);
        assert_eq!(cpu.step(), Err(StepError::InvalidOpcode(0x8D)));
    }

    #[test]
    fn lds_and_les() {
        let mut cpu = cpu_with_code(&[0xC5, 0x07]);
        cpu.regs.bx = 0x0200;
        cpu.mem.write16(0, 0x0200, 0x3412);
        cpu.mem.write16(0, 0x0202, 0x5678);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.ax, 0x3412);
        assert_eq!(cpu.regs.ds, 0x5678);

        let mut cpu = cpu_with_code(&[0xC4, 0x1C]);
        cpu.regs.si = 0x0300;
        cpu.mem.write16(0, 0x0300, 0x1111);
        cpu.mem.write16(0, 0x0302, 0x2222);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.bx, 0x1111);
        assert_eq!(cpu.regs.es, 0x2222);
    }

    #[test]
    fn xlat_uses_al_as_index() {
        let mut cpu = cpu_with_code(&[0xD7]);
        cpu.regs.bx = 0x0200;
        cpu.regs.set8(Reg8::Al, 0x05);
        cpu.mem.write8(0, 0x0205, 0xAB);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.get8(Reg8::Al), 0xAB);
    }

    #[test]
    fn cbw_and_cwd() {
        let mut cpu = cpu_with_code(&[0x98]);
        cpu.regs.set8(Reg8::Al, 0x80);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.ax, 0xFF80);

        let mut cpu = cpu_with_code(&[0x99]);
        cpu.regs.ax = 0x8000;
        cpu.step().unwrap();
        assert_eq!(cpu.regs.dx, 0xFFFF);

        let mut cpu = cpu_with_code(&[0x99]);
        cpu.regs.ax = 0x7FFF;
        cpu.step().unwrap();
        assert_eq!(cpu.regs.dx, 0);
    }

    #[test]
    fn segment_override_applies_to_memory_operand() {
        let mut cpu = cpu_with_code(&[0x26, 0x8A, 0x02]);
        cpu.regs.es = 0x1000;
        cpu.regs.ss = 0x2000;
        cpu.regs.bp = 0x0010;
        cpu.regs.si = 0x0005;
        cpu.mem.write8(0x1000, 0x0015, 0x42);
        cpu.mem.write8(0x2000, 0x0015, 0x43);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.get8(Reg8::Al), 0x42);
    }

    #[test]
    fn default_segment_for_bp_is_ss() {
        let mut cpu = cpu_with_code(&[0x8A, 0x02]);
        cpu.regs.es = 0x1000;
        cpu.regs.ss = 0x2000;
        cpu.regs.bp = 0x0010;
        cpu.regs.si = 0x0005;
        cpu.mem.write8(0x1000, 0x0015, 0x42);
        cpu.mem.write8(0x2000, 0x0015, 0x43);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.get8(Reg8::Al), 0x43);
    }

    #[test]
    fn direct_address_and_displacements() {
        let mut cpu = cpu_with_code(&[0x8A, 0x06, 0x00, 0x02]);
        cpu.mem.write8(0, 0x0200, 0x55);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.get8(Reg8::Al), 0x55);

        let mut cpu = cpu_with_code(&[0x8A, 0x47, 0xFF]);
        cpu.regs.bx = 0x0200;
        cpu.mem.write8(0, 0x01FF, 0x99);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.get8(Reg8::Al), 0x99);

        let mut cpu = cpu_with_code(&[0x8A, 0x87, 0x00, 0x02]);
        cpu.regs.bx = 0x0100;
        cpu.mem.write8(0, 0x0300, 0x77);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.get8(Reg8::Al), 0x77);
    }

    #[test]
    fn instruction_fetch_wraps_at_one_megabyte() {
        let mut cpu = cpu_with_code_at(0xFFFF, &[0xB0]);
        cpu.mem.write8(0, 0x0000, 0x7F);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.get8(Reg8::Al), 0x7F);
        assert_eq!(cpu.regs.ip, 0x0001);
    }

    #[test]
    fn invalid_and_unimplemented_opcodes() {
        let mut cpu = cpu_with_code(&[0x60]);
        assert_eq!(cpu.step(), Err(StepError::InvalidOpcode(0x60)));

        let mut cpu = cpu_with_code(&[0xE4]);
        assert_eq!(cpu.step(), Err(StepError::Unimplemented));

        let mut cpu = cpu_with_code(&[0xC6, 0x0F, 0x12]);
        assert_eq!(cpu.step(), Err(StepError::InvalidOpcode(0xC6)));

        let mut cpu = cpu_with_code(&[0x8F, 0x0F]);
        assert_eq!(cpu.step(), Err(StepError::InvalidOpcode(0x8F)));

        let mut cpu = cpu_with_code(&[0xFF, 0x3F]);
        assert_eq!(cpu.step(), Err(StepError::InvalidOpcode(0xFF)));
    }

    #[test]
    fn push_and_pop_memory_through_groups() {
        let mut cpu = cpu_with_code(&[0xFF, 0x37, 0x8F, 0x07]);
        cpu.regs.ss = 0;
        cpu.regs.sp = 0x1000;
        cpu.regs.bx = 0x0200;
        cpu.mem.write16(0, 0x0200, 0xBEEF);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.sp, 0x0FFE);
        assert_eq!(cpu.mem.read16(0, 0x0FFE), 0xBEEF);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.sp, 0x1000);
        assert_eq!(cpu.mem.read16(0, 0x0200), 0xBEEF);
    }

    #[test]
    fn push_sp_through_group_ff() {
        let mut cpu = cpu_with_code(&[0xFF, 0xF4]);
        cpu.regs.ss = 0;
        cpu.regs.sp = 0x1000;
        cpu.step().unwrap();
        assert_eq!(cpu.mem.read16(0, 0x0FFE), 0x0FFE);
    }

    #[test]
    fn xlat_respects_segment_override() {
        let mut cpu = cpu_with_code(&[0x26, 0xD7]);
        cpu.regs.es = 0x1000;
        cpu.regs.set8(Reg8::Al, 0x05);
        cpu.mem.write8(0x1000, 0x0005, 0xCD);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.get8(Reg8::Al), 0xCD);
    }

    #[test]
    fn lds_respects_segment_override() {
        let mut cpu = cpu_with_code(&[0x26, 0xC5, 0x07]);
        cpu.regs.es = 0x1000;
        cpu.regs.bx = 0x0200;
        cpu.mem.write16(0x1000, 0x0200, 0x3412);
        cpu.mem.write16(0x1000, 0x0202, 0x5678);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.ax, 0x3412);
        assert_eq!(cpu.regs.ds, 0x5678);
    }

    #[test]
    fn sahf_preserves_high_flags() {
        let mut cpu = cpu_with_code(&[0x9E]);
        cpu.regs.flags.set(Flags::IF, true);
        cpu.regs.flags.set(Flags::DF, true);
        cpu.regs.flags.set(Flags::OF, true);
        cpu.regs.set8(Reg8::Ah, 0x01);
        cpu.step().unwrap();
        assert!(cpu.regs.flags.contains(Flags::CF));
        assert!(cpu.regs.flags.contains(Flags::IF));
        assert!(cpu.regs.flags.contains(Flags::DF));
        assert!(cpu.regs.flags.contains(Flags::OF));
    }

    #[test]
    fn popf_restores_reserved_bits() {
        let mut cpu = cpu_with_code(&[0x9D]);
        cpu.regs.ss = 0;
        cpu.regs.sp = 0x1000;
        cpu.regs.flags = Flags(0xFFFF);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.flags.bits(), Flags::RESERVED);
    }

    #[test]
    fn data_movement_preserves_flags() {
        let cases: &[&[u8]] = &[
            &[0x89, 0xD8],
            &[0x87, 0xD8],
            &[0x93],
            &[0x50],
            &[0x58],
            &[0x06],
            &[0x07],
            &[0x9C],
            &[0x8D, 0x47, 0x10],
            &[0xC5, 0x07],
            &[0xC4, 0x1C],
            &[0xD7],
            &[0x98],
            &[0x99],
            &[0x9F],
            &[0x90],
        ];

        for code in cases {
            let mut cpu = cpu_with_code(code);
            cpu.regs.flags.set(Flags::CF, true);
            cpu.regs.flags.set(Flags::ZF, true);
            let flags = cpu.regs.flags;
            cpu.step().unwrap();
            assert_eq!(cpu.regs.flags, flags, "flags changed by {code:02X?}");
        }
    }
}
