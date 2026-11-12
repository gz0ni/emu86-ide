//! ModR/M decoding and real-mode effective addresses.

use crate::cpu::Cpu;
use crate::registers::{Reg16, Reg8};

/// Segment register selector for memory operands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Segment {
    Es,
    Cs,
    Ss,
    Ds,
}

impl Segment {
    pub(crate) const fn register(self) -> Reg16 {
        match self {
            Self::Es => Reg16::Es,
            Self::Cs => Reg16::Cs,
            Self::Ss => Reg16::Ss,
            Self::Ds => Reg16::Ds,
        }
    }

    pub(crate) const fn from_encoding(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Es),
            1 => Some(Self::Cs),
            2 => Some(Self::Ss),
            3 => Some(Self::Ds),
            _ => None,
        }
    }
}

/// A decoded memory operand with its segment already resolved.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct MemOperand {
    pub segment: Segment,
    pub offset: u16,
}

impl MemOperand {
    fn read8(self, cpu: &Cpu) -> u8 {
        cpu.mem
            .read8(cpu.regs.get(self.segment.register()), self.offset)
    }

    fn write8(self, cpu: &mut Cpu, value: u8) {
        let segment = cpu.regs.get(self.segment.register());
        cpu.mem.write8(segment, self.offset, value);
    }

    fn read16(self, cpu: &Cpu) -> u16 {
        cpu.mem
            .read16(cpu.regs.get(self.segment.register()), self.offset)
    }

    fn write16(self, cpu: &mut Cpu, value: u16) {
        let segment = cpu.regs.get(self.segment.register());
        cpu.mem.write16(segment, self.offset, value);
    }
}

/// An 8-bit general-purpose operand.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Operand8 {
    Reg(Reg8),
    Mem(MemOperand),
}

impl Operand8 {
    pub(crate) fn read(self, cpu: &Cpu) -> u8 {
        match self {
            Self::Reg(reg) => cpu.regs.get8(reg),
            Self::Mem(mem) => mem.read8(cpu),
        }
    }

    pub(crate) fn write(self, cpu: &mut Cpu, value: u8) {
        match self {
            Self::Reg(reg) => cpu.regs.set8(reg, value),
            Self::Mem(mem) => mem.write8(cpu, value),
        }
    }
}

/// A 16-bit general-purpose operand.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Operand16 {
    Reg(Reg16),
    Mem(MemOperand),
}

impl Operand16 {
    pub(crate) fn read(self, cpu: &Cpu) -> u16 {
        match self {
            Self::Reg(reg) => cpu.regs.get(reg),
            Self::Mem(mem) => mem.read16(cpu),
        }
    }

    pub(crate) fn write(self, cpu: &mut Cpu, value: u16) {
        match self {
            Self::Reg(reg) => cpu.regs.set(reg, value),
            Self::Mem(mem) => mem.write16(cpu, value),
        }
    }
}

/// The `r/m` half of a ModR/M byte.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Rm {
    Reg(u8),
    Mem(MemOperand),
}

/// A decoded ModR/M byte.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ModRm {
    pub reg: u8,
    pub rm: Rm,
}

impl ModRm {
    pub(crate) fn decode(cpu: &mut Cpu, segment_override: Option<Segment>) -> Self {
        let byte = cpu.fetch8();
        let mode = byte >> 6;
        let reg = (byte >> 3) & 0x07;
        let rm = byte & 0x07;

        if mode == 0b11 {
            return Self {
                reg,
                rm: Rm::Reg(rm),
            };
        }

        let (mut offset, default_segment) = match rm {
            0 => (cpu.regs.bx.wrapping_add(cpu.regs.si), Segment::Ds),
            1 => (cpu.regs.bx.wrapping_add(cpu.regs.di), Segment::Ds),
            2 => (cpu.regs.bp.wrapping_add(cpu.regs.si), Segment::Ss),
            3 => (cpu.regs.bp.wrapping_add(cpu.regs.di), Segment::Ss),
            4 => (cpu.regs.si, Segment::Ds),
            5 => (cpu.regs.di, Segment::Ds),
            6 if mode == 0 => (cpu.fetch16(), Segment::Ds),
            6 => (cpu.regs.bp, Segment::Ss),
            7 => (cpu.regs.bx, Segment::Ds),
            _ => unreachable!(),
        };

        match mode {
            1 => {
                let displacement = cpu.fetch8() as i8 as i16 as u16;
                offset = offset.wrapping_add(displacement);
            }
            2 => offset = offset.wrapping_add(cpu.fetch16()),
            _ => {}
        }

        let segment = segment_override.unwrap_or(default_segment);
        Self {
            reg,
            rm: Rm::Mem(MemOperand { segment, offset }),
        }
    }

    pub(crate) fn reg8(self) -> Reg8 {
        Reg8::ALL[self.reg as usize]
    }

    pub(crate) fn reg16(self) -> Reg16 {
        reg16_from_encoding(self.reg)
    }

    pub(crate) fn operand8(self) -> Operand8 {
        match self.rm {
            Rm::Reg(index) => Operand8::Reg(Reg8::ALL[index as usize]),
            Rm::Mem(mem) => Operand8::Mem(mem),
        }
    }

    pub(crate) fn operand16(self) -> Operand16 {
        match self.rm {
            Rm::Reg(index) => Operand16::Reg(reg16_from_encoding(index)),
            Rm::Mem(mem) => Operand16::Mem(mem),
        }
    }

    pub(crate) fn memory(self) -> Option<MemOperand> {
        match self.rm {
            Rm::Mem(mem) => Some(mem),
            Rm::Reg(_) => None,
        }
    }
}

pub(crate) const fn reg16_from_encoding(value: u8) -> Reg16 {
    match value & 0x07 {
        0 => Reg16::Ax,
        1 => Reg16::Cx,
        2 => Reg16::Dx,
        3 => Reg16::Bx,
        4 => Reg16::Sp,
        5 => Reg16::Bp,
        6 => Reg16::Si,
        _ => Reg16::Di,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu_with_code_at;

    fn decode(byte: u8, segment_override: Option<Segment>) -> (Cpu, ModRm) {
        let mut cpu = cpu_with_code_at(0x0100, &[byte]);
        let modrm = ModRm::decode(&mut cpu, segment_override);
        (cpu, modrm)
    }

    #[test]
    fn register_modrm_maps_encoding_order() {
        let (_, modrm) = decode(0xD8, None);
        assert_eq!(modrm.reg, 3);
        assert_eq!(modrm.reg8(), Reg8::Bl);
        assert_eq!(modrm.reg16(), Reg16::Bx);
        assert_eq!(modrm.operand8(), Operand8::Reg(Reg8::Al));
        assert_eq!(modrm.operand16(), Operand16::Reg(Reg16::Ax));
    }

    #[test]
    fn every_rm_register_maps_to_architecture_order() {
        let expected16 = [
            Reg16::Ax,
            Reg16::Cx,
            Reg16::Dx,
            Reg16::Bx,
            Reg16::Sp,
            Reg16::Bp,
            Reg16::Si,
            Reg16::Di,
        ];

        for index in 0..8u8 {
            let (_, modrm) = decode(0xC0 | index, None);
            assert_eq!(
                modrm.operand16(),
                Operand16::Reg(expected16[index as usize])
            );
            assert_eq!(modrm.operand8(), Operand8::Reg(Reg8::ALL[index as usize]));
        }
    }

    #[test]
    fn effective_addresses_and_default_segments() {
        let cases = [
            (0x00, 0x1010, Segment::Ds),
            (0x01, 0x1020, Segment::Ds),
            (0x02, 0x2010, Segment::Ss),
            (0x03, 0x2020, Segment::Ss),
            (0x04, 0x0010, Segment::Ds),
            (0x05, 0x0020, Segment::Ds),
            (0x07, 0x1000, Segment::Ds),
        ];

        for (byte, expected_offset, expected_segment) in cases {
            let mut cpu = cpu_with_code_at(0x0100, &[byte]);
            cpu.regs.bx = 0x1000;
            cpu.regs.si = 0x0010;
            cpu.regs.di = 0x0020;
            cpu.regs.bp = 0x2000;
            let modrm = ModRm::decode(&mut cpu, None);
            let memory = modrm.memory().expect("memory operand");
            assert_eq!(memory.offset, expected_offset);
            assert_eq!(memory.segment, expected_segment);
        }
    }

    #[test]
    fn direct_address_uses_disp16() {
        let mut cpu = cpu_with_code_at(0x0100, &[0x06, 0x44, 0x33]);
        let modrm = ModRm::decode(&mut cpu, None);
        let memory = modrm.memory().expect("memory operand");
        assert_eq!(memory.offset, 0x3344);
        assert_eq!(memory.segment, Segment::Ds);
    }

    #[test]
    fn segment_override_wins() {
        let mut cpu = cpu_with_code_at(0x0100, &[0x02]);
        cpu.regs.bp = 0x2000;
        cpu.regs.si = 0x0010;
        let modrm = ModRm::decode(&mut cpu, Some(Segment::Es));
        let memory = modrm.memory().expect("memory operand");
        assert_eq!(memory.offset, 0x2010);
        assert_eq!(memory.segment, Segment::Es);
    }

    #[test]
    fn disp8_is_sign_extended() {
        let mut cpu = cpu_with_code_at(0x0100, &[0x40, 0xFF]);
        cpu.regs.bx = 0x1000;
        cpu.regs.si = 0x0010;
        let modrm = ModRm::decode(&mut cpu, None);
        let memory = modrm.memory().expect("memory operand");
        assert_eq!(memory.offset, 0x100F);
    }

    #[test]
    fn disp16_is_added_to_the_base() {
        let mut cpu = cpu_with_code_at(0x0100, &[0x80, 0x00, 0x02]);
        cpu.regs.bx = 0x1000;
        cpu.regs.si = 0x0010;
        let modrm = ModRm::decode(&mut cpu, None);
        let memory = modrm.memory().expect("memory operand");
        assert_eq!(memory.offset, 0x1210);
    }
}
