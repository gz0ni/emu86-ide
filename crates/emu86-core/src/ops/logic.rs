//! Logic instructions: TEST and NOT.

use crate::cpu::Cpu;
use crate::decoder::{ModRm, Segment};
use crate::ops::alu::{self, Width};
use crate::registers::Reg8;
use crate::StepError;

pub(super) fn not(cpu: &mut Cpu, modrm: ModRm, width: Width) {
    let value = alu::read_rm(cpu, modrm, width);
    alu::write_rm(cpu, modrm, width, !value & width.mask());
}

pub(super) fn test_rm_reg(
    cpu: &mut Cpu,
    opcode: u8,
    segment_override: Option<Segment>,
) -> Result<(), StepError> {
    let modrm = ModRm::decode(cpu, segment_override);

    match opcode {
        0x84 => {
            let value = u16::from(modrm.operand8().read(cpu));
            let mask = u16::from(cpu.regs.get8(modrm.reg8()));
            alu::logic(&mut cpu.regs.flags, value & mask, Width::Byte);
        }
        0x85 => {
            let value = modrm.operand16().read(cpu);
            let mask = cpu.regs.get(modrm.reg16());
            alu::logic(&mut cpu.regs.flags, value & mask, Width::Word);
        }
        _ => unreachable!(),
    }

    Ok(())
}

pub(super) fn test_accumulator(cpu: &mut Cpu, opcode: u8) -> Result<(), StepError> {
    match opcode {
        0xA8 => {
            let value = u16::from(cpu.fetch8());
            let accumulator = u16::from(cpu.regs.get8(Reg8::Al));
            alu::logic(&mut cpu.regs.flags, accumulator & value, Width::Byte);
        }
        0xA9 => {
            let value = cpu.fetch16();
            alu::logic(&mut cpu.regs.flags, cpu.regs.ax & value, Width::Word);
        }
        _ => unreachable!(),
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu_with_code;
    use crate::flags::Flags;

    #[test]
    fn test_rm_reg_sets_flags_without_writing() {
        let mut cpu = cpu_with_code(&[0x84, 0xC8]);
        cpu.regs.set8(Reg8::Al, 0x0F);
        cpu.regs.set8(Reg8::Cl, 0xF0);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.get8(Reg8::Al), 0x0F);
        assert_eq!(cpu.regs.get8(Reg8::Cl), 0xF0);
        assert!(cpu.regs.flags.contains(Flags::ZF));
        assert!(!cpu.regs.flags.contains(Flags::CF));

        let mut cpu = cpu_with_code(&[0x85, 0xC8]);
        cpu.regs.ax = 0x8000;
        cpu.regs.cx = 0x8000;
        cpu.step().unwrap();
        assert_eq!(cpu.regs.ax, 0x8000);
        assert_eq!(cpu.regs.cx, 0x8000);
        assert!(cpu.regs.flags.contains(Flags::SF));
        assert!(!cpu.regs.flags.contains(Flags::ZF));
    }

    #[test]
    fn test_accumulator_immediates() {
        let mut cpu = cpu_with_code(&[0xA8, 0x0F]);
        cpu.regs.set8(Reg8::Al, 0xF0);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.get8(Reg8::Al), 0xF0);
        assert!(cpu.regs.flags.contains(Flags::ZF));

        let mut cpu = cpu_with_code(&[0xA9, 0x00, 0x80]);
        cpu.regs.ax = 0x8000;
        cpu.step().unwrap();
        assert_eq!(cpu.regs.ax, 0x8000);
        assert!(cpu.regs.flags.contains(Flags::SF));
        assert!(!cpu.regs.flags.contains(Flags::ZF));
    }

    #[test]
    fn test_clears_carry_overflow_and_auxiliary_carry() {
        let mut cpu = cpu_with_code(&[0x84, 0xC8]);
        cpu.regs.flags.set(Flags::CF, true);
        cpu.regs.flags.set(Flags::OF, true);
        cpu.regs.flags.set(Flags::AF, true);
        cpu.regs.set8(Reg8::Al, 0x01);
        cpu.regs.set8(Reg8::Cl, 0x01);
        cpu.step().unwrap();
        assert!(!cpu.regs.flags.contains(Flags::CF));
        assert!(!cpu.regs.flags.contains(Flags::OF));
        assert!(!cpu.regs.flags.contains(Flags::AF));
    }
}
