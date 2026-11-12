//! Shift and rotate instructions (group 2).

use crate::cpu::Cpu;
use crate::decoder::{ModRm, Segment};
use crate::flags::Flags;
use crate::ops::alu::{self, Width};
use crate::registers::Reg8;
use crate::StepError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ShiftOp {
    Rol,
    Ror,
    Rcl,
    Rcr,
    Shl,
    Shr,
    Sar,
}

impl ShiftOp {
    const fn from_encoding(value: u8) -> Self {
        match value & 0x07 {
            0 => Self::Rol,
            1 => Self::Ror,
            2 => Self::Rcl,
            3 => Self::Rcr,
            4 | 6 => Self::Shl,
            5 => Self::Shr,
            _ => Self::Sar,
        }
    }

    const fn is_rotate(self) -> bool {
        matches!(self, Self::Rol | Self::Ror | Self::Rcl | Self::Rcr)
    }
}

pub(super) fn group_d0(
    cpu: &mut Cpu,
    opcode: u8,
    segment_override: Option<Segment>,
) -> Result<(), StepError> {
    let modrm = ModRm::decode(cpu, segment_override);
    let width = match opcode {
        0xD0 | 0xD2 => Width::Byte,
        0xD1 | 0xD3 => Width::Word,
        _ => unreachable!(),
    };
    let count = match opcode {
        0xD0 | 0xD1 => 1,
        0xD2 | 0xD3 => cpu.regs.get8(Reg8::Cl),
        _ => unreachable!(),
    };

    if count == 0 {
        return Ok(());
    }

    let op = ShiftOp::from_encoding(modrm.reg);
    let value = alu::read_rm(cpu, modrm, width);
    let result = apply(cpu, op, value, count, width);
    alu::write_rm(cpu, modrm, width, result);
    Ok(())
}

fn apply(cpu: &mut Cpu, op: ShiftOp, value: u16, count: u8, width: Width) -> u16 {
    let mask = width.mask();
    let sign = width.sign();
    let bits = match width {
        Width::Byte => 8,
        Width::Word => 16,
    };
    let original = value & mask;
    let mut value = original;
    let mut carry = cpu.regs.flags.contains(Flags::CF);

    for _ in 0..count {
        match op {
            ShiftOp::Rol => {
                carry = value & sign != 0;
                value = ((value << 1) | u16::from(carry)) & mask;
            }
            ShiftOp::Ror => {
                carry = value & 1 != 0;
                value = (value >> 1) | (u16::from(carry) << (bits - 1));
            }
            ShiftOp::Rcl => {
                let next = value & sign != 0;
                value = ((value << 1) | u16::from(carry)) & mask;
                carry = next;
            }
            ShiftOp::Rcr => {
                let next = value & 1 != 0;
                value = (value >> 1) | (u16::from(carry) << (bits - 1));
                carry = next;
            }
            ShiftOp::Shl => {
                carry = value & sign != 0;
                value = (value << 1) & mask;
            }
            ShiftOp::Shr => {
                carry = value & 1 != 0;
                value >>= 1;
            }
            ShiftOp::Sar => {
                carry = value & 1 != 0;
                value = match width {
                    Width::Byte => u16::from(((value as u8 as i8) >> 1) as u8),
                    Width::Word => ((value as i16) >> 1) as u16,
                };
            }
        }
    }

    cpu.regs.flags.set(Flags::CF, carry);

    if count == 1 {
        let overflow = match op {
            ShiftOp::Rol | ShiftOp::Rcl | ShiftOp::Shl => (value & sign != 0) ^ carry,
            ShiftOp::Ror | ShiftOp::Rcr => (value & sign != 0) ^ (value & (sign >> 1) != 0),
            ShiftOp::Shr => original & sign != 0,
            ShiftOp::Sar => false,
        };
        cpu.regs.flags.set(Flags::OF, overflow);
    }

    if !op.is_rotate() {
        cpu.regs.flags.set(Flags::SF, value & sign != 0);
        cpu.regs.flags.set(Flags::ZF, value == 0);
        cpu.regs.flags.set(Flags::PF, alu::parity(value as u8));
    }

    value
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu_with_code;

    #[test]
    fn rol_and_ror_byte() {
        let mut cpu = cpu_with_code(&[0xD0, 0xC0]);
        cpu.regs.set8(Reg8::Al, 0x81);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.get8(Reg8::Al), 0x03);
        assert!(cpu.regs.flags.contains(Flags::CF));
        assert!(cpu.regs.flags.contains(Flags::OF));

        let mut cpu = cpu_with_code(&[0xD0, 0xC8]);
        cpu.regs.set8(Reg8::Al, 0x01);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.get8(Reg8::Al), 0x80);
        assert!(cpu.regs.flags.contains(Flags::CF));
        assert!(cpu.regs.flags.contains(Flags::OF));
    }

    #[test]
    fn rcl_and_rcr_use_the_carry_flag() {
        let mut cpu = cpu_with_code(&[0xD0, 0xD0]);
        cpu.regs.set8(Reg8::Al, 0x80);
        cpu.regs.flags.set(Flags::CF, true);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.get8(Reg8::Al), 0x01);
        assert!(cpu.regs.flags.contains(Flags::CF));
        assert!(cpu.regs.flags.contains(Flags::OF));

        let mut cpu = cpu_with_code(&[0xD0, 0xD8]);
        cpu.regs.set8(Reg8::Al, 0x01);
        cpu.regs.flags.set(Flags::CF, true);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.get8(Reg8::Al), 0x80);
        assert!(cpu.regs.flags.contains(Flags::CF));
        assert!(cpu.regs.flags.contains(Flags::OF));
    }

    #[test]
    fn shl_shr_and_sar_byte() {
        let mut cpu = cpu_with_code(&[0xD0, 0xE0]);
        cpu.regs.set8(Reg8::Al, 0x80);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.get8(Reg8::Al), 0x00);
        assert!(cpu.regs.flags.contains(Flags::CF));
        assert!(cpu.regs.flags.contains(Flags::ZF));
        assert!(cpu.regs.flags.contains(Flags::PF));
        assert!(cpu.regs.flags.contains(Flags::OF));

        let mut cpu = cpu_with_code(&[0xD0, 0xE8]);
        cpu.regs.set8(Reg8::Al, 0x01);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.get8(Reg8::Al), 0x00);
        assert!(cpu.regs.flags.contains(Flags::CF));
        assert!(cpu.regs.flags.contains(Flags::ZF));
        assert!(!cpu.regs.flags.contains(Flags::OF));

        let mut cpu = cpu_with_code(&[0xD0, 0xF8]);
        cpu.regs.set8(Reg8::Al, 0x81);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.get8(Reg8::Al), 0xC0);
        assert!(cpu.regs.flags.contains(Flags::CF));
        assert!(cpu.regs.flags.contains(Flags::SF));
        assert!(!cpu.regs.flags.contains(Flags::OF));
    }

    #[test]
    fn selector_six_is_shl() {
        let mut cpu = cpu_with_code(&[0xD0, 0xF0]);
        cpu.regs.set8(Reg8::Al, 0x01);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.get8(Reg8::Al), 0x02);
    }

    #[test]
    fn count_zero_leaves_value_and_flags_alone() {
        let mut cpu = cpu_with_code(&[0xD2, 0xE0]);
        cpu.regs.set8(Reg8::Al, 0x81);
        cpu.regs.set8(Reg8::Cl, 0x00);
        cpu.regs.flags.set(Flags::CF, true);
        cpu.regs.flags.set(Flags::OF, true);
        cpu.regs.flags.set(Flags::ZF, true);
        let flags = cpu.regs.flags;
        cpu.step().unwrap();
        assert_eq!(cpu.regs.get8(Reg8::Al), 0x81);
        assert_eq!(cpu.regs.flags, flags);
    }

    #[test]
    fn shift_by_cl_and_word_forms() {
        let mut cpu = cpu_with_code(&[0xD2, 0xE0]);
        cpu.regs.set8(Reg8::Al, 0x81);
        cpu.regs.set8(Reg8::Cl, 0x02);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.get8(Reg8::Al), 0x04);
        assert!(!cpu.regs.flags.contains(Flags::CF));

        let mut cpu = cpu_with_code(&[0xD1, 0xE0]);
        cpu.regs.ax = 0x8000;
        cpu.step().unwrap();
        assert_eq!(cpu.regs.ax, 0x0000);
        assert!(cpu.regs.flags.contains(Flags::CF));
        assert!(cpu.regs.flags.contains(Flags::ZF));
        assert!(cpu.regs.flags.contains(Flags::OF));

        let mut cpu = cpu_with_code(&[0xD1, 0xF8]);
        cpu.regs.ax = 0x8001;
        cpu.step().unwrap();
        assert_eq!(cpu.regs.ax, 0xC000);
        assert!(cpu.regs.flags.contains(Flags::CF));
        assert!(cpu.regs.flags.contains(Flags::SF));
        assert!(!cpu.regs.flags.contains(Flags::OF));
    }

    #[test]
    fn rotates_preserve_szp_and_shifts_preserve_af() {
        let mut cpu = cpu_with_code(&[0xD0, 0xC0]);
        cpu.regs.set8(Reg8::Al, 0x01);
        cpu.regs.flags.set(Flags::ZF, true);
        cpu.regs.flags.set(Flags::SF, true);
        cpu.regs.flags.set(Flags::PF, true);
        cpu.step().unwrap();
        assert!(cpu.regs.flags.contains(Flags::ZF));
        assert!(cpu.regs.flags.contains(Flags::SF));
        assert!(cpu.regs.flags.contains(Flags::PF));

        let mut cpu = cpu_with_code(&[0xD0, 0xE0]);
        cpu.regs.set8(Reg8::Al, 0x01);
        cpu.regs.flags.set(Flags::AF, true);
        cpu.step().unwrap();
        assert!(cpu.regs.flags.contains(Flags::AF));
    }

    #[test]
    fn group_d0_operates_on_memory() {
        let mut cpu = cpu_with_code(&[0xD0, 0x07]);
        cpu.regs.bx = 0x0200;
        cpu.mem.write8(0, 0x0200, 0x81);
        cpu.step().unwrap();
        assert_eq!(cpu.mem.read8(0, 0x0200), 0x03);
    }

    #[test]
    fn shift_immediate_opcodes_are_invalid_on_the_8086() {
        let mut cpu = cpu_with_code(&[0xC0, 0xC0, 0x01]);
        assert_eq!(cpu.step(), Err(StepError::InvalidOpcode(0xC0)));

        let mut cpu = cpu_with_code(&[0xC1, 0xC0, 0x01]);
        assert_eq!(cpu.step(), Err(StepError::InvalidOpcode(0xC1)));
    }
}
