//! Arithmetic flag helpers shared by the arithmetic and logic groups.

use crate::cpu::Cpu;
use crate::decoder::{ModRm, Segment};
use crate::flags::Flags;
use crate::registers::Reg8;
use crate::StepError;

/// Operand width of an ALU operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Width {
    Byte,
    Word,
}

impl Width {
    pub(super) const fn mask(self) -> u16 {
        match self {
            Self::Byte => 0x00FF,
            Self::Word => 0xFFFF,
        }
    }

    pub(super) const fn sign(self) -> u16 {
        match self {
            Self::Byte => 0x0080,
            Self::Word => 0x8000,
        }
    }
}

pub(super) fn add(
    flags: &mut Flags,
    destination: u16,
    source: u16,
    carry: bool,
    width: Width,
) -> u16 {
    let mask = width.mask();
    let sign = width.sign();
    let destination = destination & mask;
    let source = source & mask;
    let carry = u16::from(carry);
    let sum = u32::from(destination) + u32::from(source) + u32::from(carry);
    let result = (sum as u16) & mask;

    flags.set(Flags::CF, sum > u32::from(mask));
    flags.set(Flags::ZF, result == 0);
    flags.set(Flags::SF, result & sign != 0);
    flags.set(Flags::PF, parity(result as u8));
    flags.set(
        Flags::AF,
        (destination & 0x000F) + (source & 0x000F) + carry > 0x000F,
    );
    flags.set(
        Flags::OF,
        (destination ^ result) & (source ^ result) & sign != 0,
    );

    result
}

pub(super) fn sub(
    flags: &mut Flags,
    destination: u16,
    source: u16,
    borrow: bool,
    width: Width,
) -> u16 {
    let mask = width.mask();
    let sign = width.sign();
    let destination = destination & mask;
    let source = source & mask;
    let borrow = u16::from(borrow);
    let result = destination.wrapping_sub(source).wrapping_sub(borrow) & mask;

    flags.set(
        Flags::CF,
        u32::from(destination) < u32::from(source) + u32::from(borrow),
    );
    flags.set(Flags::ZF, result == 0);
    flags.set(Flags::SF, result & sign != 0);
    flags.set(Flags::PF, parity(result as u8));
    flags.set(
        Flags::AF,
        (destination & 0x000F) < (source & 0x000F) + borrow,
    );
    flags.set(
        Flags::OF,
        (destination ^ source) & (destination ^ result) & sign != 0,
    );

    result
}

pub(super) fn logic(flags: &mut Flags, result: u16, width: Width) {
    let result = result & width.mask();

    flags.set(Flags::CF, false);
    flags.set(Flags::OF, false);
    flags.set(Flags::AF, false);
    flags.set(Flags::ZF, result == 0);
    flags.set(Flags::SF, result & width.sign() != 0);
    flags.set(Flags::PF, parity(result as u8));
}

pub(super) fn increment(flags: &mut Flags, value: u16, width: Width) -> u16 {
    let carry = flags.contains(Flags::CF);
    let result = add(flags, value, 1, false, width);
    flags.set(Flags::CF, carry);
    result
}

pub(super) fn decrement(flags: &mut Flags, value: u16, width: Width) -> u16 {
    let carry = flags.contains(Flags::CF);
    let result = sub(flags, value, 1, false, width);
    flags.set(Flags::CF, carry);
    result
}

pub(super) fn negate(flags: &mut Flags, value: u16, width: Width) -> u16 {
    sub(flags, 0, value, false, width)
}

pub(super) fn parity(value: u8) -> bool {
    value.count_ones() % 2 == 0
}

pub(super) fn set_szp(flags: &mut Flags, value: u8) {
    flags.set(Flags::SF, value & 0x80 != 0);
    flags.set(Flags::ZF, value == 0);
    flags.set(Flags::PF, parity(value));
}

/// The eight ALU operations encoded in bits 5-3 of an opcode byte and in the
/// `reg` field of group 1.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum BinaryOp {
    Add,
    Or,
    Adc,
    Sbb,
    And,
    Sub,
    Xor,
    Cmp,
}

impl BinaryOp {
    pub(super) const fn from_encoding(value: u8) -> Self {
        match value & 0x07 {
            0 => Self::Add,
            1 => Self::Or,
            2 => Self::Adc,
            3 => Self::Sbb,
            4 => Self::And,
            5 => Self::Sub,
            6 => Self::Xor,
            _ => Self::Cmp,
        }
    }

    pub(super) const fn writes_destination(self) -> bool {
        !matches!(self, Self::Cmp)
    }
}

pub(super) fn apply(
    flags: &mut Flags,
    op: BinaryOp,
    destination: u16,
    source: u16,
    width: Width,
) -> u16 {
    match op {
        BinaryOp::Add => add(flags, destination, source, false, width),
        BinaryOp::Adc => {
            let carry = flags.contains(Flags::CF);
            add(flags, destination, source, carry, width)
        }
        BinaryOp::Sub | BinaryOp::Cmp => sub(flags, destination, source, false, width),
        BinaryOp::Sbb => {
            let borrow = flags.contains(Flags::CF);
            sub(flags, destination, source, borrow, width)
        }
        BinaryOp::And => {
            let result = destination & source & width.mask();
            logic(flags, result, width);
            result
        }
        BinaryOp::Or => {
            let result = (destination | source) & width.mask();
            logic(flags, result, width);
            result
        }
        BinaryOp::Xor => {
            let result = (destination ^ source) & width.mask();
            logic(flags, result, width);
            result
        }
    }
}

pub(super) fn read_rm(cpu: &Cpu, modrm: ModRm, width: Width) -> u16 {
    match width {
        Width::Byte => u16::from(modrm.operand8().read(cpu)),
        Width::Word => modrm.operand16().read(cpu),
    }
}

pub(super) fn write_rm(cpu: &mut Cpu, modrm: ModRm, width: Width, value: u16) {
    match width {
        Width::Byte => modrm.operand8().write(cpu, value as u8),
        Width::Word => modrm.operand16().write(cpu, value),
    }
}

pub(super) fn binary(
    cpu: &mut Cpu,
    opcode: u8,
    segment_override: Option<Segment>,
) -> Result<(), StepError> {
    let op = BinaryOp::from_encoding(opcode >> 3);

    match opcode & 0x07 {
        0 => {
            let modrm = ModRm::decode(cpu, segment_override);
            let destination = modrm.operand8();
            let left = u16::from(destination.read(cpu));
            let right = u16::from(cpu.regs.get8(modrm.reg8()));
            let result = apply(&mut cpu.regs.flags, op, left, right, Width::Byte);
            if op.writes_destination() {
                destination.write(cpu, result as u8);
            }
        }
        1 => {
            let modrm = ModRm::decode(cpu, segment_override);
            let destination = modrm.operand16();
            let left = destination.read(cpu);
            let right = cpu.regs.get(modrm.reg16());
            let result = apply(&mut cpu.regs.flags, op, left, right, Width::Word);
            if op.writes_destination() {
                destination.write(cpu, result);
            }
        }
        2 => {
            let modrm = ModRm::decode(cpu, segment_override);
            let left = u16::from(cpu.regs.get8(modrm.reg8()));
            let right = u16::from(modrm.operand8().read(cpu));
            let result = apply(&mut cpu.regs.flags, op, left, right, Width::Byte);
            if op.writes_destination() {
                cpu.regs.set8(modrm.reg8(), result as u8);
            }
        }
        3 => {
            let modrm = ModRm::decode(cpu, segment_override);
            let left = cpu.regs.get(modrm.reg16());
            let right = modrm.operand16().read(cpu);
            let result = apply(&mut cpu.regs.flags, op, left, right, Width::Word);
            if op.writes_destination() {
                cpu.regs.set(modrm.reg16(), result);
            }
        }
        4 => {
            let right = u16::from(cpu.fetch8());
            let left = u16::from(cpu.regs.get8(Reg8::Al));
            let result = apply(&mut cpu.regs.flags, op, left, right, Width::Byte);
            if op.writes_destination() {
                cpu.regs.set8(Reg8::Al, result as u8);
            }
        }
        5 => {
            let right = cpu.fetch16();
            let result = apply(&mut cpu.regs.flags, op, cpu.regs.ax, right, Width::Word);
            if op.writes_destination() {
                cpu.regs.ax = result;
            }
        }
        _ => unreachable!(),
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu_with_code;

    #[test]
    fn parity_counts_the_low_byte() {
        assert!(parity(0x00));
        assert!(!parity(0x01));
        assert!(parity(0x03));
        assert!(!parity(0x80));
        assert!(parity(0xFF));
    }

    #[test]
    fn byte_add_sets_carry_and_overflow() {
        let mut flags = Flags::power_on();
        assert_eq!(add(&mut flags, 0x7F, 0x01, false, Width::Byte), 0x80);
        assert!(!flags.contains(Flags::CF));
        assert!(flags.contains(Flags::OF));
        assert!(flags.contains(Flags::SF));
        assert!(flags.contains(Flags::AF));
        assert!(!flags.contains(Flags::ZF));
        assert!(!flags.contains(Flags::PF));

        let mut flags = Flags::power_on();
        assert_eq!(add(&mut flags, 0xFF, 0x01, false, Width::Byte), 0x00);
        assert!(flags.contains(Flags::CF));
        assert!(flags.contains(Flags::ZF));
        assert!(flags.contains(Flags::PF));
    }

    #[test]
    fn word_add_uses_the_word_sign_bit() {
        let mut flags = Flags::power_on();
        assert_eq!(add(&mut flags, 0x7FFF, 0x0001, false, Width::Word), 0x8000);
        assert!(!flags.contains(Flags::CF));
        assert!(flags.contains(Flags::OF));
        assert!(flags.contains(Flags::SF));
    }

    #[test]
    fn subtract_sets_borrow_and_overflow() {
        let mut flags = Flags::power_on();
        assert_eq!(sub(&mut flags, 0x00, 0x01, false, Width::Byte), 0xFF);
        assert!(flags.contains(Flags::CF));
        assert!(flags.contains(Flags::AF));
        assert!(flags.contains(Flags::SF));
        assert!(!flags.contains(Flags::OF));
        assert!(flags.contains(Flags::PF));

        let mut flags = Flags::power_on();
        assert_eq!(sub(&mut flags, 0x80, 0x01, false, Width::Byte), 0x7F);
        assert!(!flags.contains(Flags::CF));
        assert!(flags.contains(Flags::OF));
        assert!(flags.contains(Flags::AF));
    }

    #[test]
    fn increment_and_decrement_preserve_carry() {
        let mut flags = Flags::power_on();
        flags.set(Flags::CF, true);
        assert_eq!(increment(&mut flags, 0xFFFF, Width::Word), 0x0000);
        assert!(flags.contains(Flags::CF));
        assert!(flags.contains(Flags::ZF));

        assert_eq!(decrement(&mut flags, 0x0000, Width::Word), 0xFFFF);
        assert!(flags.contains(Flags::CF));
        assert!(flags.contains(Flags::SF));
        assert!(flags.contains(Flags::AF));
    }

    #[test]
    fn negate_sets_carry_only_for_non_zero() {
        let mut flags = Flags::power_on();
        assert_eq!(negate(&mut flags, 0x00, Width::Byte), 0x00);
        assert!(!flags.contains(Flags::CF));
        assert!(flags.contains(Flags::ZF));

        let mut flags = Flags::power_on();
        assert_eq!(negate(&mut flags, 0x80, Width::Byte), 0x80);
        assert!(flags.contains(Flags::CF));
        assert!(flags.contains(Flags::OF));
        assert!(!flags.contains(Flags::AF));
    }

    #[test]
    fn logic_clears_carry_overflow_and_auxiliary_carry() {
        let mut flags = Flags::power_on();
        flags.set(Flags::CF, true);
        flags.set(Flags::OF, true);
        flags.set(Flags::AF, true);
        logic(&mut flags, 0x00, Width::Byte);
        assert!(!flags.contains(Flags::CF));
        assert!(!flags.contains(Flags::OF));
        assert!(!flags.contains(Flags::AF));
        assert!(flags.contains(Flags::ZF));
        assert!(flags.contains(Flags::PF));
    }

    #[test]
    fn add_forms_cover_registers_memory_and_immediates() {
        let mut cpu = cpu_with_code(&[0x00, 0x07]);
        cpu.regs.bx = 0x0200;
        cpu.regs.set8(Reg8::Al, 0x05);
        cpu.mem.write8(0, 0x0200, 0x03);
        cpu.step().unwrap();
        assert_eq!(cpu.mem.read8(0, 0x0200), 0x08);

        let mut cpu = cpu_with_code(&[0x01, 0x07]);
        cpu.regs.bx = 0x0200;
        cpu.regs.ax = 0x1234;
        cpu.mem.write16(0, 0x0200, 0x1111);
        cpu.step().unwrap();
        assert_eq!(cpu.mem.read16(0, 0x0200), 0x2345);

        let mut cpu = cpu_with_code(&[0x02, 0x07]);
        cpu.regs.bx = 0x0200;
        cpu.regs.set8(Reg8::Al, 0x05);
        cpu.mem.write8(0, 0x0200, 0x03);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.get8(Reg8::Al), 0x08);

        let mut cpu = cpu_with_code(&[0x03, 0x07]);
        cpu.regs.bx = 0x0200;
        cpu.regs.ax = 0x1234;
        cpu.mem.write16(0, 0x0200, 0x1111);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.ax, 0x2345);

        let mut cpu = cpu_with_code(&[0x04, 0x01]);
        cpu.regs.set8(Reg8::Al, 0x7F);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.get8(Reg8::Al), 0x80);
        assert!(cpu.regs.flags.contains(Flags::OF));

        let mut cpu = cpu_with_code(&[0x05, 0x01, 0x00]);
        cpu.regs.ax = 0xFFFF;
        cpu.step().unwrap();
        assert_eq!(cpu.regs.ax, 0x0000);
        assert!(cpu.regs.flags.contains(Flags::CF));
        assert!(cpu.regs.flags.contains(Flags::ZF));
    }

    #[test]
    fn adc_and_sbb_use_the_carry_flag() {
        let mut cpu = cpu_with_code(&[0x14, 0x01]);
        cpu.regs.set8(Reg8::Al, 0x00);
        cpu.regs.flags.set(Flags::CF, true);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.get8(Reg8::Al), 0x02);

        let mut cpu = cpu_with_code(&[0x1C, 0x01]);
        cpu.regs.set8(Reg8::Al, 0x02);
        cpu.regs.flags.set(Flags::CF, true);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.get8(Reg8::Al), 0x00);
        assert!(cpu.regs.flags.contains(Flags::ZF));
    }

    #[test]
    fn cmp_sets_flags_without_writing_the_destination() {
        let mut cpu = cpu_with_code(&[0x3C, 0x10]);
        cpu.regs.set8(Reg8::Al, 0x10);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.get8(Reg8::Al), 0x10);
        assert!(cpu.regs.flags.contains(Flags::ZF));
        assert!(!cpu.regs.flags.contains(Flags::CF));

        let mut cpu = cpu_with_code(&[0x3D, 0x00, 0x20]);
        cpu.regs.ax = 0x1000;
        cpu.step().unwrap();
        assert_eq!(cpu.regs.ax, 0x1000);
        assert!(cpu.regs.flags.contains(Flags::CF));
    }

    #[test]
    fn logic_forms_clear_carry_overflow_and_auxiliary_carry() {
        let mut cpu = cpu_with_code(&[0x24, 0x0F]);
        cpu.regs.set8(Reg8::Al, 0xF0);
        cpu.regs.flags.set(Flags::CF, true);
        cpu.regs.flags.set(Flags::OF, true);
        cpu.regs.flags.set(Flags::AF, true);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.get8(Reg8::Al), 0x00);
        assert!(!cpu.regs.flags.contains(Flags::CF));
        assert!(!cpu.regs.flags.contains(Flags::OF));
        assert!(!cpu.regs.flags.contains(Flags::AF));
        assert!(cpu.regs.flags.contains(Flags::ZF));

        let mut cpu = cpu_with_code(&[0x0C, 0x0F]);
        cpu.regs.set8(Reg8::Al, 0xF0);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.get8(Reg8::Al), 0xFF);

        let mut cpu = cpu_with_code(&[0x34, 0xFF]);
        cpu.regs.set8(Reg8::Al, 0x0F);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.get8(Reg8::Al), 0xF0);
    }

    #[test]
    fn binary_forms_respect_segment_overrides() {
        let mut cpu = cpu_with_code(&[0x26, 0x02, 0x07]);
        cpu.regs.es = 0x1000;
        cpu.regs.bx = 0x0200;
        cpu.regs.set8(Reg8::Al, 0x01);
        cpu.mem.write8(0x1000, 0x0200, 0x41);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.get8(Reg8::Al), 0x42);
    }
}
