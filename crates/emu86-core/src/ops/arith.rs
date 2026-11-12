//! Arithmetic instructions: group 1, 3, 4 and 5 forms, BCD adjustment,
//! AAM and AAD.

use crate::cpu::Cpu;
use crate::decoder::{reg16_from_encoding, ModRm, Segment};
use crate::flags::Flags;
use crate::ops::alu::{self, BinaryOp, Width};
use crate::ops::{control, data};
use crate::registers::Reg8;
use crate::StepError;

pub(super) fn inc_dec_reg(cpu: &mut Cpu, opcode: u8) -> Result<(), StepError> {
    let register = reg16_from_encoding(opcode & 0x07);
    let value = cpu.regs.get(register);
    let result = if opcode & 0x08 == 0 {
        alu::increment(&mut cpu.regs.flags, value, Width::Word)
    } else {
        alu::decrement(&mut cpu.regs.flags, value, Width::Word)
    };
    cpu.regs.set(register, result);
    Ok(())
}

pub(super) fn group_80(
    cpu: &mut Cpu,
    opcode: u8,
    segment_override: Option<Segment>,
) -> Result<(), StepError> {
    let modrm = ModRm::decode(cpu, segment_override);
    let op = BinaryOp::from_encoding(modrm.reg);
    let width = match opcode {
        0x80 | 0x82 => Width::Byte,
        0x81 | 0x83 => Width::Word,
        _ => unreachable!(),
    };
    let immediate = match opcode {
        0x80 | 0x82 => u16::from(cpu.fetch8()),
        0x81 => cpu.fetch16(),
        0x83 => cpu.fetch8() as i8 as i16 as u16,
        _ => unreachable!(),
    };

    let destination = alu::read_rm(cpu, modrm, width);
    let result = alu::apply(&mut cpu.regs.flags, op, destination, immediate, width);
    if op.writes_destination() {
        alu::write_rm(cpu, modrm, width, result);
    }

    Ok(())
}

pub(super) fn group_f6(
    cpu: &mut Cpu,
    opcode: u8,
    segment_override: Option<Segment>,
) -> Result<(), StepError> {
    let modrm = ModRm::decode(cpu, segment_override);
    let width = if opcode == 0xF6 {
        Width::Byte
    } else {
        Width::Word
    };

    match modrm.reg {
        0 => {
            let immediate = match width {
                Width::Byte => u16::from(cpu.fetch8()),
                Width::Word => cpu.fetch16(),
            };
            let value = alu::read_rm(cpu, modrm, width);
            alu::logic(&mut cpu.regs.flags, value & immediate, width);
        }
        2 => super::logic::not(cpu, modrm, width),
        3 => {
            let value = alu::read_rm(cpu, modrm, width);
            let result = alu::negate(&mut cpu.regs.flags, value, width);
            alu::write_rm(cpu, modrm, width, result);
        }
        4 => multiply(cpu, modrm, width, false),
        5 => multiply(cpu, modrm, width, true),
        6 => divide(cpu, modrm, width, false)?,
        7 => divide(cpu, modrm, width, true)?,
        _ => return Err(StepError::InvalidOpcode(opcode)),
    }

    Ok(())
}

fn multiply(cpu: &mut Cpu, modrm: ModRm, width: Width, signed: bool) {
    let multiplier = alu::read_rm(cpu, modrm, width);

    let overflow = match width {
        Width::Byte => {
            let multiplicand = cpu.regs.get8(Reg8::Al);
            let multiplier = multiplier as u8;
            if signed {
                let product = i16::from(multiplicand as i8) * i16::from(multiplier as i8);
                cpu.regs.ax = product as u16;
                product != i16::from(product as i8)
            } else {
                let product = u16::from(multiplicand) * u16::from(multiplier);
                cpu.regs.ax = product;
                product > u16::from(u8::MAX)
            }
        }
        Width::Word => {
            let multiplicand = cpu.regs.ax;
            if signed {
                let product = i32::from(multiplicand as i16) * i32::from(multiplier as i16);
                cpu.regs.ax = product as u16;
                cpu.regs.dx = (product as u32 >> 16) as u16;
                product != i32::from(product as i16)
            } else {
                let product = u32::from(multiplicand) * u32::from(multiplier);
                cpu.regs.ax = product as u16;
                cpu.regs.dx = (product >> 16) as u16;
                product > u32::from(u16::MAX)
            }
        }
    };

    cpu.regs.flags.set(Flags::CF, overflow);
    cpu.regs.flags.set(Flags::OF, overflow);
}

fn divide(cpu: &mut Cpu, modrm: ModRm, width: Width, signed: bool) -> Result<(), StepError> {
    let divisor = alu::read_rm(cpu, modrm, width);

    match width {
        Width::Byte => {
            let divisor = divisor as u8;
            if divisor == 0 {
                return Err(StepError::DivideError);
            }

            let dividend = cpu.regs.ax;
            let (quotient, remainder) = if signed {
                let dividend = dividend as i16;
                let divisor = i16::from(divisor as i8);
                if dividend == i16::MIN && divisor == -1 {
                    return Err(StepError::DivideError);
                }
                let quotient = dividend / divisor;
                if quotient < i16::from(i8::MIN) || quotient > i16::from(i8::MAX) {
                    return Err(StepError::DivideError);
                }
                (quotient as u8, (dividend % divisor) as u8)
            } else {
                let quotient = dividend / u16::from(divisor);
                if quotient > u16::from(u8::MAX) {
                    return Err(StepError::DivideError);
                }
                (quotient as u8, (dividend % u16::from(divisor)) as u8)
            };

            cpu.regs.set8(Reg8::Al, quotient);
            cpu.regs.set8(Reg8::Ah, remainder);
        }
        Width::Word => {
            if divisor == 0 {
                return Err(StepError::DivideError);
            }

            let dividend = (u32::from(cpu.regs.dx) << 16) | u32::from(cpu.regs.ax);
            let (quotient, remainder) = if signed {
                let dividend = dividend as i32;
                let divisor = i32::from(divisor as i16);
                if dividend == i32::MIN && divisor == -1 {
                    return Err(StepError::DivideError);
                }
                let quotient = dividend / divisor;
                if quotient < i32::from(i16::MIN) || quotient > i32::from(i16::MAX) {
                    return Err(StepError::DivideError);
                }
                (quotient as i16 as u16, (dividend % divisor) as i16 as u16)
            } else {
                let quotient = dividend / u32::from(divisor);
                if quotient > u32::from(u16::MAX) {
                    return Err(StepError::DivideError);
                }
                (quotient as u16, (dividend % u32::from(divisor)) as u16)
            };

            cpu.regs.ax = quotient;
            cpu.regs.dx = remainder;
        }
    }

    Ok(())
}

pub(super) fn group_fe(cpu: &mut Cpu, segment_override: Option<Segment>) -> Result<(), StepError> {
    let modrm = ModRm::decode(cpu, segment_override);
    let value = u16::from(modrm.operand8().read(cpu));
    let result = match modrm.reg {
        0 => alu::increment(&mut cpu.regs.flags, value, Width::Byte),
        1 => alu::decrement(&mut cpu.regs.flags, value, Width::Byte),
        _ => return Err(StepError::InvalidOpcode(0xFE)),
    };
    modrm.operand8().write(cpu, result as u8);
    Ok(())
}

pub(super) fn group_ff(cpu: &mut Cpu, segment_override: Option<Segment>) -> Result<(), StepError> {
    let modrm = ModRm::decode(cpu, segment_override);

    match modrm.reg {
        0 => {
            let value = alu::read_rm(cpu, modrm, Width::Word);
            let result = alu::increment(&mut cpu.regs.flags, value, Width::Word);
            alu::write_rm(cpu, modrm, Width::Word, result);
            Ok(())
        }
        1 => {
            let value = alu::read_rm(cpu, modrm, Width::Word);
            let result = alu::decrement(&mut cpu.regs.flags, value, Width::Word);
            alu::write_rm(cpu, modrm, Width::Word, result);
            Ok(())
        }
        2 => {
            control::call_near_indirect(cpu, modrm);
            Ok(())
        }
        3 => control::call_far_indirect(cpu, modrm),
        4 => {
            control::jump_near_indirect(cpu, modrm);
            Ok(())
        }
        5 => control::jump_far_indirect(cpu, modrm),
        6 => {
            data::push_operand16(cpu, modrm.operand16());
            Ok(())
        }
        7 => Err(StepError::InvalidOpcode(0xFF)),
        _ => Err(StepError::Unimplemented),
    }
}

pub(super) fn daa(cpu: &mut Cpu) -> Result<(), StepError> {
    let old_al = cpu.regs.get8(Reg8::Al);
    let old_cf = cpu.regs.flags.contains(Flags::CF);
    let old_af = cpu.regs.flags.contains(Flags::AF);
    let mut al = old_al;

    let af = if (al & 0x0F) > 9 || old_af {
        al = al.wrapping_add(0x06);
        true
    } else {
        false
    };

    let cf = old_al > 0x99 || old_cf;
    if cf {
        al = al.wrapping_add(0x60);
    }

    cpu.regs.set8(Reg8::Al, al);
    cpu.regs.flags.set(Flags::CF, cf);
    cpu.regs.flags.set(Flags::AF, af);
    alu::set_szp(&mut cpu.regs.flags, al);
    Ok(())
}

pub(super) fn das(cpu: &mut Cpu) -> Result<(), StepError> {
    let old_al = cpu.regs.get8(Reg8::Al);
    let old_cf = cpu.regs.flags.contains(Flags::CF);
    let old_af = cpu.regs.flags.contains(Flags::AF);
    let mut al = old_al;
    let mut cf = false;

    let af = if (al & 0x0F) > 9 || old_af {
        let (adjusted, borrow) = al.overflowing_sub(0x06);
        al = adjusted;
        cf = old_cf || borrow;
        true
    } else {
        false
    };

    if old_al > 0x99 || old_cf {
        al = al.wrapping_sub(0x60);
        cf = true;
    }

    cpu.regs.set8(Reg8::Al, al);
    cpu.regs.flags.set(Flags::CF, cf);
    cpu.regs.flags.set(Flags::AF, af);
    alu::set_szp(&mut cpu.regs.flags, al);
    Ok(())
}

pub(super) fn aaa(cpu: &mut Cpu) -> Result<(), StepError> {
    let al = cpu.regs.get8(Reg8::Al);
    let adjust = (al & 0x0F) > 9 || cpu.regs.flags.contains(Flags::AF);

    if adjust {
        cpu.regs.ax = cpu.regs.ax.wrapping_add(0x0106);
    }

    cpu.regs.flags.set(Flags::AF, adjust);
    cpu.regs.flags.set(Flags::CF, adjust);
    let al = cpu.regs.get8(Reg8::Al) & 0x0F;
    cpu.regs.set8(Reg8::Al, al);
    Ok(())
}

pub(super) fn aas(cpu: &mut Cpu) -> Result<(), StepError> {
    let al = cpu.regs.get8(Reg8::Al);
    let adjust = (al & 0x0F) > 9 || cpu.regs.flags.contains(Flags::AF);

    if adjust {
        cpu.regs.ax = cpu.regs.ax.wrapping_sub(0x0106);
    }

    cpu.regs.flags.set(Flags::AF, adjust);
    cpu.regs.flags.set(Flags::CF, adjust);
    let al = cpu.regs.get8(Reg8::Al) & 0x0F;
    cpu.regs.set8(Reg8::Al, al);
    Ok(())
}

pub(super) fn aam(cpu: &mut Cpu) -> Result<(), StepError> {
    let base = cpu.fetch8();
    if base == 0 {
        return Err(StepError::DivideError);
    }

    let al = cpu.regs.get8(Reg8::Al);
    cpu.regs.set8(Reg8::Ah, al / base);
    let al = al % base;
    cpu.regs.set8(Reg8::Al, al);
    alu::set_szp(&mut cpu.regs.flags, al);
    Ok(())
}

pub(super) fn aad(cpu: &mut Cpu) -> Result<(), StepError> {
    let base = cpu.fetch8();
    let al = cpu.regs.get8(Reg8::Al);
    let ah = cpu.regs.get8(Reg8::Ah);
    let result = al.wrapping_add(ah.wrapping_mul(base));

    cpu.regs.set8(Reg8::Ah, 0);
    cpu.regs.set8(Reg8::Al, result);
    alu::set_szp(&mut cpu.regs.flags, result);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu_with_code;

    #[test]
    fn inc_and_dec_register_forms_preserve_carry() {
        let mut cpu = cpu_with_code(&[0x40, 0x48]);
        cpu.regs.ax = 0xFFFF;
        cpu.regs.flags.set(Flags::CF, true);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.ax, 0x0000);
        assert!(cpu.regs.flags.contains(Flags::CF));
        assert!(cpu.regs.flags.contains(Flags::ZF));

        cpu.regs.ax = 0x0000;
        cpu.step().unwrap();
        assert_eq!(cpu.regs.ax, 0xFFFF);
        assert!(cpu.regs.flags.contains(Flags::CF));
        assert!(cpu.regs.flags.contains(Flags::SF));
    }

    #[test]
    fn group_80_handles_every_operation() {
        let cases = [
            (0, 0x0F, 0x01, 0x10),
            (1, 0x0F, 0x01, 0x0F),
            (2, 0x0F, 0x01, 0x10),
            (3, 0x0F, 0x01, 0x0E),
            (4, 0x0F, 0x01, 0x01),
            (5, 0x0F, 0x01, 0x0E),
            (6, 0x0F, 0x01, 0x0E),
        ];

        for (selector, value, immediate, expected) in cases {
            let modrm = 0xC0 | (selector << 3);
            let mut cpu = cpu_with_code(&[0x80, modrm, immediate]);
            cpu.regs.set8(Reg8::Al, value);
            cpu.step().unwrap();
            assert_eq!(cpu.regs.get8(Reg8::Al), expected, "selector {selector}");
        }
    }

    #[test]
    fn group_80_cmp_and_adc_use_carry() {
        let mut cpu = cpu_with_code(&[0x80, 0xF8, 0x10]);
        cpu.regs.set8(Reg8::Al, 0x10);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.get8(Reg8::Al), 0x10);
        assert!(cpu.regs.flags.contains(Flags::ZF));
        assert!(!cpu.regs.flags.contains(Flags::CF));

        let mut cpu = cpu_with_code(&[0x80, 0xD0, 0x01]);
        cpu.regs.set8(Reg8::Al, 0x00);
        cpu.regs.flags.set(Flags::CF, true);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.get8(Reg8::Al), 0x02);
    }

    #[test]
    fn group_81_and_83_use_word_operands() {
        let mut cpu = cpu_with_code(&[0x81, 0xC0, 0x34, 0x12]);
        cpu.regs.ax = 0x1111;
        cpu.step().unwrap();
        assert_eq!(cpu.regs.ax, 0x2345);

        let mut cpu = cpu_with_code(&[0x83, 0xC0, 0xFF]);
        cpu.regs.ax = 0x0002;
        cpu.step().unwrap();
        assert_eq!(cpu.regs.ax, 0x0001);

        let mut cpu = cpu_with_code(&[0x83, 0xC0, 0x80]);
        cpu.regs.ax = 0x0100;
        cpu.step().unwrap();
        assert_eq!(cpu.regs.ax, 0x0080);
    }

    #[test]
    fn group_82_is_an_alias_of_80() {
        let mut cpu = cpu_with_code(&[0x82, 0xC0, 0x01]);
        cpu.regs.set8(Reg8::Al, 0x41);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.get8(Reg8::Al), 0x42);
    }

    #[test]
    fn group_fe_increments_and_decrements_bytes() {
        let mut cpu = cpu_with_code(&[0xFE, 0xC0, 0xFE, 0xC8]);
        cpu.regs.set8(Reg8::Al, 0xFF);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.get8(Reg8::Al), 0x00);
        assert!(cpu.regs.flags.contains(Flags::ZF));

        cpu.step().unwrap();
        assert_eq!(cpu.regs.get8(Reg8::Al), 0xFF);
        assert!(cpu.regs.flags.contains(Flags::SF));

        let mut cpu = cpu_with_code(&[0xFE, 0xD0]);
        assert_eq!(cpu.step(), Err(StepError::InvalidOpcode(0xFE)));
    }

    #[test]
    fn group_ff_increments_and_decrements_words() {
        let mut cpu = cpu_with_code(&[0xFF, 0xC0, 0xFF, 0xC8]);
        cpu.regs.ax = 0xFFFF;
        cpu.step().unwrap();
        assert_eq!(cpu.regs.ax, 0x0000);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.ax, 0xFFFF);
    }

    #[test]
    fn group_f6_test_sets_flags_without_writing() {
        let mut cpu = cpu_with_code(&[0xF6, 0xC0, 0x0F]);
        cpu.regs.set8(Reg8::Al, 0xF0);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.get8(Reg8::Al), 0xF0);
        assert!(cpu.regs.flags.contains(Flags::ZF));
        assert!(!cpu.regs.flags.contains(Flags::CF));

        let mut cpu = cpu_with_code(&[0xF7, 0xC0, 0x00, 0x80]);
        cpu.regs.ax = 0x8000;
        cpu.step().unwrap();
        assert_eq!(cpu.regs.ax, 0x8000);
        assert!(cpu.regs.flags.contains(Flags::SF));
        assert!(!cpu.regs.flags.contains(Flags::ZF));
    }

    #[test]
    fn group_f6_not_inverts_without_changing_flags() {
        let mut cpu = cpu_with_code(&[0xF6, 0xD0]);
        cpu.regs.set8(Reg8::Al, 0x0F);
        cpu.regs.flags.set(Flags::CF, true);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.get8(Reg8::Al), 0xF0);
        assert!(cpu.regs.flags.contains(Flags::CF));

        let mut cpu = cpu_with_code(&[0xF7, 0xD0]);
        cpu.regs.ax = 0x00FF;
        cpu.step().unwrap();
        assert_eq!(cpu.regs.ax, 0xFF00);
    }

    #[test]
    fn group_f6_neg_sets_carry_for_non_zero() {
        let mut cpu = cpu_with_code(&[0xF6, 0xD8]);
        cpu.regs.set8(Reg8::Al, 0x01);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.get8(Reg8::Al), 0xFF);
        assert!(cpu.regs.flags.contains(Flags::CF));

        let mut cpu = cpu_with_code(&[0xF6, 0xD8]);
        cpu.regs.set8(Reg8::Al, 0x00);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.get8(Reg8::Al), 0x00);
        assert!(!cpu.regs.flags.contains(Flags::CF));
    }

    #[test]
    fn group_f6_selector_one_is_invalid() {
        let mut cpu = cpu_with_code(&[0xF6, 0xC8]);
        assert_eq!(cpu.step(), Err(StepError::InvalidOpcode(0xF6)));

        let mut cpu = cpu_with_code(&[0xF7, 0xC8]);
        assert_eq!(cpu.step(), Err(StepError::InvalidOpcode(0xF7)));
    }

    #[test]
    fn mul_unsigned_byte_and_word() {
        let mut cpu = cpu_with_code(&[0xF6, 0xE3]);
        cpu.regs.set8(Reg8::Al, 0x10);
        cpu.regs.set8(Reg8::Bl, 0x10);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.ax, 0x0100);
        assert!(cpu.regs.flags.contains(Flags::CF));
        assert!(cpu.regs.flags.contains(Flags::OF));

        let mut cpu = cpu_with_code(&[0xF7, 0xE3]);
        cpu.regs.ax = 0x0100;
        cpu.regs.bx = 0x0010;
        cpu.step().unwrap();
        assert_eq!(cpu.regs.ax, 0x1000);
        assert_eq!(cpu.regs.dx, 0x0000);
        assert!(!cpu.regs.flags.contains(Flags::CF));

        let mut cpu = cpu_with_code(&[0xF7, 0xE3]);
        cpu.regs.ax = 0x1000;
        cpu.regs.bx = 0x0010;
        cpu.step().unwrap();
        assert_eq!(cpu.regs.ax, 0x0000);
        assert_eq!(cpu.regs.dx, 0x0001);
        assert!(cpu.regs.flags.contains(Flags::CF));
    }

    #[test]
    fn imul_signed_byte_and_word() {
        let mut cpu = cpu_with_code(&[0xF6, 0xEB]);
        cpu.regs.set8(Reg8::Al, 0xFF);
        cpu.regs.set8(Reg8::Bl, 0x02);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.ax, 0xFFFE);
        assert!(!cpu.regs.flags.contains(Flags::CF));

        let mut cpu = cpu_with_code(&[0xF6, 0xEB]);
        cpu.regs.set8(Reg8::Al, 0x40);
        cpu.regs.set8(Reg8::Bl, 0x02);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.ax, 0x0080);
        assert!(cpu.regs.flags.contains(Flags::CF));

        let mut cpu = cpu_with_code(&[0xF7, 0xEB]);
        cpu.regs.ax = 0xFFFF;
        cpu.regs.bx = 0x0002;
        cpu.step().unwrap();
        assert_eq!(cpu.regs.ax, 0xFFFE);
        assert_eq!(cpu.regs.dx, 0xFFFF);
        assert!(!cpu.regs.flags.contains(Flags::CF));

        let mut cpu = cpu_with_code(&[0xF7, 0xEB]);
        cpu.regs.ax = 0x4000;
        cpu.regs.bx = 0x0002;
        cpu.step().unwrap();
        assert_eq!(cpu.regs.ax, 0x8000);
        assert_eq!(cpu.regs.dx, 0x0000);
        assert!(cpu.regs.flags.contains(Flags::CF));
    }

    #[test]
    fn mul_and_imul_leave_other_flags_alone() {
        let mut cpu = cpu_with_code(&[0xF6, 0xE3]);
        cpu.regs.flags.set(Flags::ZF, true);
        cpu.regs.flags.set(Flags::SF, true);
        cpu.regs.set8(Reg8::Al, 0x02);
        cpu.regs.set8(Reg8::Bl, 0x03);
        cpu.step().unwrap();
        assert!(cpu.regs.flags.contains(Flags::ZF));
        assert!(cpu.regs.flags.contains(Flags::SF));
        assert!(!cpu.regs.flags.contains(Flags::CF));
        assert!(!cpu.regs.flags.contains(Flags::OF));
    }

    #[test]
    fn div_unsigned_byte_and_word() {
        let mut cpu = cpu_with_code(&[0xF6, 0xF3]);
        cpu.regs.ax = 0x0010;
        cpu.regs.set8(Reg8::Bl, 0x03);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.get8(Reg8::Al), 0x05);
        assert_eq!(cpu.regs.get8(Reg8::Ah), 0x01);

        let mut cpu = cpu_with_code(&[0xF7, 0xF3]);
        cpu.regs.dx = 0x0001;
        cpu.regs.ax = 0x0000;
        cpu.regs.bx = 0x0002;
        cpu.step().unwrap();
        assert_eq!(cpu.regs.ax, 0x8000);
        assert_eq!(cpu.regs.dx, 0x0000);
    }

    #[test]
    fn div_reports_divide_errors() {
        let mut cpu = cpu_with_code(&[0xF6, 0xF3]);
        cpu.regs.set8(Reg8::Bl, 0x00);
        assert_eq!(cpu.step(), Err(StepError::DivideError));

        let mut cpu = cpu_with_code(&[0xF6, 0xF3]);
        cpu.regs.ax = 0x0100;
        cpu.regs.set8(Reg8::Bl, 0x01);
        assert_eq!(cpu.step(), Err(StepError::DivideError));

        let mut cpu = cpu_with_code(&[0xF7, 0xF3]);
        cpu.regs.dx = 0x0001;
        cpu.regs.ax = 0x0000;
        cpu.regs.bx = 0x0001;
        assert_eq!(cpu.step(), Err(StepError::DivideError));
    }

    #[test]
    fn idiv_signed_byte_and_word() {
        let mut cpu = cpu_with_code(&[0xF6, 0xFB]);
        cpu.regs.ax = 0xFFF9;
        cpu.regs.set8(Reg8::Bl, 0x02);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.get8(Reg8::Al), 0xFD);
        assert_eq!(cpu.regs.get8(Reg8::Ah), 0xFF);

        let mut cpu = cpu_with_code(&[0xF7, 0xFB]);
        cpu.regs.dx = 0xFFFF;
        cpu.regs.ax = 0xFFF9;
        cpu.regs.bx = 0x0002;
        cpu.step().unwrap();
        assert_eq!(cpu.regs.ax, 0xFFFD);
        assert_eq!(cpu.regs.dx, 0xFFFF);
    }

    #[test]
    fn idiv_reports_overflow_and_divide_by_zero() {
        let mut cpu = cpu_with_code(&[0xF6, 0xFB]);
        cpu.regs.ax = 0x8000;
        cpu.regs.set8(Reg8::Bl, 0xFF);
        assert_eq!(cpu.step(), Err(StepError::DivideError));

        let mut cpu = cpu_with_code(&[0xF6, 0xFB]);
        cpu.regs.ax = 0x0080;
        cpu.regs.set8(Reg8::Bl, 0x01);
        assert_eq!(cpu.step(), Err(StepError::DivideError));

        let mut cpu = cpu_with_code(&[0xF7, 0xFB]);
        cpu.regs.dx = 0x8000;
        cpu.regs.ax = 0x0000;
        cpu.regs.bx = 0xFFFF;
        assert_eq!(cpu.step(), Err(StepError::DivideError));

        let mut cpu = cpu_with_code(&[0xF7, 0xFB]);
        cpu.regs.dx = 0x0000;
        cpu.regs.ax = 0x8000;
        cpu.regs.bx = 0x0001;
        assert_eq!(cpu.step(), Err(StepError::DivideError));

        let mut cpu = cpu_with_code(&[0xF6, 0xFB]);
        cpu.regs.set8(Reg8::Bl, 0x00);
        assert_eq!(cpu.step(), Err(StepError::DivideError));
    }

    #[test]
    fn div_leaves_flags_unchanged() {
        let mut cpu = cpu_with_code(&[0xF6, 0xF3]);
        cpu.regs.ax = 0x0010;
        cpu.regs.set8(Reg8::Bl, 0x03);
        cpu.regs.flags.set(Flags::CF, true);
        cpu.regs.flags.set(Flags::ZF, true);
        let flags = cpu.regs.flags;
        cpu.step().unwrap();
        assert_eq!(cpu.regs.flags, flags);
    }

    #[test]
    fn daa_adjusts_bcd_after_addition() {
        let mut cpu = cpu_with_code(&[0x27]);
        cpu.regs.set8(Reg8::Al, 0x9A);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.get8(Reg8::Al), 0x00);
        assert!(cpu.regs.flags.contains(Flags::CF));
        assert!(cpu.regs.flags.contains(Flags::AF));
        assert!(cpu.regs.flags.contains(Flags::ZF));

        let mut cpu = cpu_with_code(&[0x27]);
        cpu.regs.set8(Reg8::Al, 0x15);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.get8(Reg8::Al), 0x15);
        assert!(!cpu.regs.flags.contains(Flags::CF));
        assert!(!cpu.regs.flags.contains(Flags::AF));
    }

    #[test]
    fn das_adjusts_bcd_after_subtraction() {
        let mut cpu = cpu_with_code(&[0x2F]);
        cpu.regs.set8(Reg8::Al, 0x0D);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.get8(Reg8::Al), 0x07);
        assert!(!cpu.regs.flags.contains(Flags::CF));
        assert!(cpu.regs.flags.contains(Flags::AF));

        let mut cpu = cpu_with_code(&[0x2F]);
        cpu.regs.set8(Reg8::Al, 0xFB);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.get8(Reg8::Al), 0x95);
        assert!(cpu.regs.flags.contains(Flags::CF));
    }

    #[test]
    fn aaa_and_aas_adjust_ascii_after_arithmetic() {
        let mut cpu = cpu_with_code(&[0x37]);
        cpu.regs.ax = 0x000F;
        cpu.step().unwrap();
        assert_eq!(cpu.regs.ax, 0x0105);
        assert!(cpu.regs.flags.contains(Flags::AF));
        assert!(cpu.regs.flags.contains(Flags::CF));

        let mut cpu = cpu_with_code(&[0x37]);
        cpu.regs.ax = 0x0005;
        cpu.step().unwrap();
        assert_eq!(cpu.regs.ax, 0x0005);
        assert!(!cpu.regs.flags.contains(Flags::AF));
        assert!(!cpu.regs.flags.contains(Flags::CF));

        let mut cpu = cpu_with_code(&[0x3F]);
        cpu.regs.ax = 0x020F;
        cpu.step().unwrap();
        assert_eq!(cpu.regs.ax, 0x0109);
        assert!(cpu.regs.flags.contains(Flags::AF));
        assert!(cpu.regs.flags.contains(Flags::CF));

        let mut cpu = cpu_with_code(&[0x3F]);
        cpu.regs.ax = 0x0205;
        cpu.step().unwrap();
        assert_eq!(cpu.regs.ax, 0x0205);
        assert!(!cpu.regs.flags.contains(Flags::AF));
        assert!(!cpu.regs.flags.contains(Flags::CF));
    }

    #[test]
    fn aam_and_aad_convert_between_bcd_and_binary() {
        let mut cpu = cpu_with_code(&[0xD4, 0x0A]);
        cpu.regs.set8(Reg8::Al, 0x2F);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.get8(Reg8::Ah), 0x04);
        assert_eq!(cpu.regs.get8(Reg8::Al), 0x07);
        assert!(!cpu.regs.flags.contains(Flags::ZF));

        let mut cpu = cpu_with_code(&[0xD5, 0x0A]);
        cpu.regs.set8(Reg8::Ah, 0x04);
        cpu.regs.set8(Reg8::Al, 0x07);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.get8(Reg8::Ah), 0x00);
        assert_eq!(cpu.regs.get8(Reg8::Al), 0x2F);

        let mut cpu = cpu_with_code(&[0xD4, 0x00]);
        cpu.regs.set8(Reg8::Al, 0x2F);
        assert_eq!(cpu.step(), Err(StepError::DivideError));

        let mut cpu = cpu_with_code(&[0xD5, 0x00]);
        cpu.regs.set8(Reg8::Ah, 0x04);
        cpu.regs.set8(Reg8::Al, 0x07);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.get8(Reg8::Ah), 0x00);
        assert_eq!(cpu.regs.get8(Reg8::Al), 0x07);
    }

    #[test]
    fn aam_sets_zero_flag_for_zero_result() {
        let mut cpu = cpu_with_code(&[0xD4, 0x0A]);
        cpu.regs.set8(Reg8::Al, 0x00);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.get8(Reg8::Ah), 0x00);
        assert_eq!(cpu.regs.get8(Reg8::Al), 0x00);
        assert!(cpu.regs.flags.contains(Flags::ZF));
    }

    #[test]
    fn adc_and_sbb_word_forms() {
        let mut cpu = cpu_with_code(&[0x15, 0x01, 0x00]);
        cpu.regs.ax = 0xFFFF;
        cpu.regs.flags.set(Flags::CF, true);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.ax, 0x0001);
        assert!(cpu.regs.flags.contains(Flags::CF));

        let mut cpu = cpu_with_code(&[0x1D, 0x01, 0x00]);
        cpu.regs.ax = 0x0000;
        cpu.regs.flags.set(Flags::CF, true);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.ax, 0xFFFE);
        assert!(cpu.regs.flags.contains(Flags::CF));
    }

    #[test]
    fn groups_operate_on_memory_operands() {
        let mut cpu = cpu_with_code(&[0x80, 0x07, 0x01]);
        cpu.regs.bx = 0x0200;
        cpu.mem.write8(0, 0x0200, 0x0F);
        cpu.step().unwrap();
        assert_eq!(cpu.mem.read8(0, 0x0200), 0x10);

        let mut cpu = cpu_with_code(&[0xF6, 0x27]);
        cpu.regs.bx = 0x0200;
        cpu.regs.set8(Reg8::Al, 0x10);
        cpu.mem.write8(0, 0x0200, 0x10);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.ax, 0x0100);

        let mut cpu = cpu_with_code(&[0xFE, 0x07]);
        cpu.regs.bx = 0x0200;
        cpu.mem.write8(0, 0x0200, 0xFF);
        cpu.step().unwrap();
        assert_eq!(cpu.mem.read8(0, 0x0200), 0x00);

        let mut cpu = cpu_with_code(&[0xFF, 0x07]);
        cpu.regs.bx = 0x0200;
        cpu.mem.write16(0, 0x0200, 0xFFFF);
        cpu.step().unwrap();
        assert_eq!(cpu.mem.read16(0, 0x0200), 0x0000);
    }

    #[test]
    fn group_ff_rejects_register_far_pointers_and_selector_seven() {
        let mut cpu = cpu_with_code(&[0xFF, 0xDF]);
        assert_eq!(cpu.step(), Err(StepError::InvalidOpcode(0xFF)));

        let mut cpu = cpu_with_code(&[0xFF, 0x3F]);
        assert_eq!(cpu.step(), Err(StepError::InvalidOpcode(0xFF)));
    }

    #[test]
    fn daa_and_das_honor_incoming_carry() {
        let mut cpu = cpu_with_code(&[0x27]);
        cpu.regs.set8(Reg8::Al, 0x00);
        cpu.regs.flags.set(Flags::CF, true);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.get8(Reg8::Al), 0x60);
        assert!(cpu.regs.flags.contains(Flags::CF));

        let mut cpu = cpu_with_code(&[0x2F]);
        cpu.regs.set8(Reg8::Al, 0x00);
        cpu.regs.flags.set(Flags::CF, true);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.get8(Reg8::Al), 0xA0);
        assert!(cpu.regs.flags.contains(Flags::CF));
    }

    #[test]
    fn daa_and_das_use_the_original_al_and_carry() {
        let mut cpu = cpu_with_code(&[0x2F]);
        cpu.regs.set8(Reg8::Al, 0xA0);
        cpu.regs.flags.set(Flags::AF, true);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.get8(Reg8::Al), 0x3A);
        assert!(cpu.regs.flags.contains(Flags::CF));
        assert!(cpu.regs.flags.contains(Flags::AF));

        let mut cpu = cpu_with_code(&[0x27]);
        cpu.regs.set8(Reg8::Al, 0x94);
        cpu.regs.flags.set(Flags::AF, true);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.get8(Reg8::Al), 0x9A);
        assert!(!cpu.regs.flags.contains(Flags::CF));
        assert!(cpu.regs.flags.contains(Flags::AF));
    }

    #[test]
    fn das_keeps_the_low_digit_borrow_in_carry() {
        let mut cpu = cpu_with_code(&[0x2F]);
        cpu.regs.set8(Reg8::Al, 0x03);
        cpu.regs.flags.set(Flags::AF, true);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.get8(Reg8::Al), 0xFD);
        assert!(cpu.regs.flags.contains(Flags::CF));
        assert!(cpu.regs.flags.contains(Flags::AF));
    }
}
