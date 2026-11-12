//! Control-flow, interrupt and flag-control instructions.

use crate::cpu::Cpu;
use crate::decoder::ModRm;
use crate::flags::Flags;
use crate::registers::Reg8;
use crate::StepError;

pub(super) fn jump_short(cpu: &mut Cpu) -> Result<(), StepError> {
    let displacement = cpu.fetch8() as i8 as i16;
    cpu.regs.ip = cpu.regs.ip.wrapping_add(displacement as u16);
    Ok(())
}

pub(super) fn jump_near(cpu: &mut Cpu) -> Result<(), StepError> {
    let displacement = cpu.fetch16();
    cpu.regs.ip = cpu.regs.ip.wrapping_add(displacement);
    Ok(())
}

pub(super) fn jump_far(cpu: &mut Cpu) -> Result<(), StepError> {
    let offset = cpu.fetch16();
    let segment = cpu.fetch16();
    cpu.regs.ip = offset;
    cpu.regs.cs = segment;
    Ok(())
}

pub(super) fn conditional_jump(cpu: &mut Cpu, opcode: u8) -> Result<(), StepError> {
    let displacement = cpu.fetch8() as i8 as i16;
    if condition(cpu, opcode) {
        cpu.regs.ip = cpu.regs.ip.wrapping_add(displacement as u16);
    }
    Ok(())
}

fn condition(cpu: &Cpu, opcode: u8) -> bool {
    let flags = cpu.regs.flags;
    let cf = flags.contains(Flags::CF);
    let zf = flags.contains(Flags::ZF);
    let sf = flags.contains(Flags::SF);
    let of = flags.contains(Flags::OF);
    let pf = flags.contains(Flags::PF);

    match opcode & 0x0F {
        0x0 => of,
        0x1 => !of,
        0x2 => cf,
        0x3 => !cf,
        0x4 => zf,
        0x5 => !zf,
        0x6 => cf || zf,
        0x7 => !cf && !zf,
        0x8 => sf,
        0x9 => !sf,
        0xA => pf,
        0xB => !pf,
        0xC => sf != of,
        0xD => sf == of,
        0xE => zf || sf != of,
        _ => !zf && sf == of,
    }
}

pub(super) fn loop_instruction(cpu: &mut Cpu, opcode: u8) -> Result<(), StepError> {
    let displacement = cpu.fetch8() as i8 as i16;
    cpu.regs.cx = cpu.regs.cx.wrapping_sub(1);
    let zf = cpu.regs.flags.contains(Flags::ZF);
    let jump = match opcode {
        0xE0 => cpu.regs.cx != 0 && !zf,
        0xE1 => cpu.regs.cx != 0 && zf,
        0xE2 => cpu.regs.cx != 0,
        _ => unreachable!(),
    };
    if jump {
        cpu.regs.ip = cpu.regs.ip.wrapping_add(displacement as u16);
    }
    Ok(())
}

pub(super) fn jcxz(cpu: &mut Cpu) -> Result<(), StepError> {
    let displacement = cpu.fetch8() as i8 as i16;
    if cpu.regs.cx == 0 {
        cpu.regs.ip = cpu.regs.ip.wrapping_add(displacement as u16);
    }
    Ok(())
}

pub(super) fn call_near(cpu: &mut Cpu) -> Result<(), StepError> {
    let displacement = cpu.fetch16();
    cpu.push16(cpu.regs.ip);
    cpu.regs.ip = cpu.regs.ip.wrapping_add(displacement);
    Ok(())
}

pub(super) fn call_far(cpu: &mut Cpu) -> Result<(), StepError> {
    let offset = cpu.fetch16();
    let segment = cpu.fetch16();
    cpu.push16(cpu.regs.cs);
    cpu.push16(cpu.regs.ip);
    cpu.regs.ip = offset;
    cpu.regs.cs = segment;
    Ok(())
}

pub(super) fn call_near_indirect(cpu: &mut Cpu, modrm: ModRm) {
    let target = modrm.operand16().read(cpu);
    cpu.push16(cpu.regs.ip);
    cpu.regs.ip = target;
}

pub(super) fn call_far_indirect(cpu: &mut Cpu, modrm: ModRm) -> Result<(), StepError> {
    let (offset, segment) = read_far_pointer(cpu, modrm)?;
    cpu.push16(cpu.regs.cs);
    cpu.push16(cpu.regs.ip);
    cpu.regs.ip = offset;
    cpu.regs.cs = segment;
    Ok(())
}

pub(super) fn jump_near_indirect(cpu: &mut Cpu, modrm: ModRm) {
    cpu.regs.ip = modrm.operand16().read(cpu);
}

pub(super) fn jump_far_indirect(cpu: &mut Cpu, modrm: ModRm) -> Result<(), StepError> {
    let (offset, segment) = read_far_pointer(cpu, modrm)?;
    cpu.regs.ip = offset;
    cpu.regs.cs = segment;
    Ok(())
}

fn read_far_pointer(cpu: &Cpu, modrm: ModRm) -> Result<(u16, u16), StepError> {
    let memory = modrm.memory().ok_or(StepError::InvalidOpcode(0xFF))?;
    let segment = cpu.regs.get(memory.segment.register());
    let offset = cpu.mem.read16(segment, memory.offset);
    let target_segment = cpu.mem.read16(segment, memory.offset.wrapping_add(2));
    Ok((offset, target_segment))
}

pub(super) fn ret(cpu: &mut Cpu, opcode: u8) -> Result<(), StepError> {
    match opcode {
        0xC2 => {
            let adjustment = cpu.fetch16();
            cpu.regs.ip = cpu.pop16();
            cpu.regs.sp = cpu.regs.sp.wrapping_add(adjustment);
        }
        0xC3 => cpu.regs.ip = cpu.pop16(),
        0xCA => {
            let adjustment = cpu.fetch16();
            cpu.regs.ip = cpu.pop16();
            cpu.regs.cs = cpu.pop16();
            cpu.regs.sp = cpu.regs.sp.wrapping_add(adjustment);
        }
        0xCB => {
            cpu.regs.ip = cpu.pop16();
            cpu.regs.cs = cpu.pop16();
        }
        _ => unreachable!(),
    }
    Ok(())
}

pub(super) fn int3(cpu: &mut Cpu) -> Result<(), StepError> {
    interrupt(cpu, 3);
    Ok(())
}

pub(super) fn int_imm(cpu: &mut Cpu) -> Result<(), StepError> {
    let vector = cpu.fetch8();
    interrupt(cpu, vector);
    Ok(())
}

pub(super) fn into(cpu: &mut Cpu) -> Result<(), StepError> {
    if cpu.regs.flags.contains(Flags::OF) {
        interrupt(cpu, 4);
    }
    Ok(())
}

fn interrupt(cpu: &mut Cpu, vector: u8) {
    if vector == 0x20 || (vector == 0x21 && cpu.regs.get8(Reg8::Ah) == 0x4C) {
        cpu.halted = true;
        return;
    }

    cpu.push16(cpu.regs.flags.bits());
    cpu.push16(cpu.regs.cs);
    cpu.push16(cpu.regs.ip);
    cpu.regs.flags.set(Flags::IF, false);
    cpu.regs.flags.set(Flags::TF, false);

    let address = u16::from(vector) * 4;
    cpu.regs.ip = cpu.mem.read16(0, address);
    cpu.regs.cs = cpu.mem.read16(0, address.wrapping_add(2));
}

pub(super) fn iret(cpu: &mut Cpu) -> Result<(), StepError> {
    cpu.regs.ip = cpu.pop16();
    cpu.regs.cs = cpu.pop16();
    cpu.regs.flags = Flags::from_bits(cpu.pop16());
    Ok(())
}

pub(super) fn hlt(cpu: &mut Cpu) -> Result<(), StepError> {
    cpu.regs.ip = cpu.regs.ip.wrapping_sub(1);
    cpu.halted = true;
    Ok(())
}

pub(super) fn clc(cpu: &mut Cpu) -> Result<(), StepError> {
    cpu.regs.flags.set(Flags::CF, false);
    Ok(())
}

pub(super) fn stc(cpu: &mut Cpu) -> Result<(), StepError> {
    cpu.regs.flags.set(Flags::CF, true);
    Ok(())
}

pub(super) fn cmc(cpu: &mut Cpu) -> Result<(), StepError> {
    let carry = cpu.regs.flags.contains(Flags::CF);
    cpu.regs.flags.set(Flags::CF, !carry);
    Ok(())
}

pub(super) fn cld(cpu: &mut Cpu) -> Result<(), StepError> {
    cpu.regs.flags.set(Flags::DF, false);
    Ok(())
}

pub(super) fn std(cpu: &mut Cpu) -> Result<(), StepError> {
    cpu.regs.flags.set(Flags::DF, true);
    Ok(())
}

pub(super) fn cli(cpu: &mut Cpu) -> Result<(), StepError> {
    cpu.regs.flags.set(Flags::IF, false);
    Ok(())
}

pub(super) fn sti(cpu: &mut Cpu) -> Result<(), StepError> {
    cpu.regs.flags.set(Flags::IF, true);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu_with_code;

    #[test]
    fn short_and_near_jumps() {
        let mut cpu = cpu_with_code(&[0xEB, 0x02]);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.ip, 0x0104);

        let mut cpu = cpu_with_code(&[0xEB, 0xFE]);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.ip, 0x0100);

        let mut cpu = cpu_with_code(&[0xE9, 0x00, 0x01]);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.ip, 0x0203);

        let mut cpu = cpu_with_code(&[0xEA, 0x34, 0x12, 0x78, 0x56]);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.ip, 0x1234);
        assert_eq!(cpu.regs.cs, 0x5678);
    }

    #[test]
    fn conditional_jumps_cover_every_condition() {
        let cases = [
            (0x70, Flags::OF, 0),
            (0x71, 0, Flags::OF),
            (0x72, Flags::CF, 0),
            (0x73, 0, Flags::CF),
            (0x74, Flags::ZF, 0),
            (0x75, 0, Flags::ZF),
            (0x76, Flags::CF, 0),
            (0x77, 0, Flags::CF),
            (0x78, Flags::SF, 0),
            (0x79, 0, Flags::SF),
            (0x7A, Flags::PF, 0),
            (0x7B, 0, Flags::PF),
            (0x7C, Flags::SF, Flags::SF | Flags::OF),
            (0x7D, 0, Flags::SF),
            (0x7E, Flags::ZF, 0),
            (0x7F, 0, Flags::ZF),
        ];

        for (opcode, taken_bits, not_taken_bits) in cases {
            let mut cpu = cpu_with_code(&[opcode, 0x02]);
            cpu.regs.flags = Flags::from_bits(taken_bits);
            cpu.step().unwrap();
            assert_eq!(cpu.regs.ip, 0x0104, "opcode {opcode:02X} should jump");

            let mut cpu = cpu_with_code(&[opcode, 0x02]);
            cpu.regs.flags = Flags::from_bits(not_taken_bits);
            cpu.step().unwrap();
            assert_eq!(
                cpu.regs.ip, 0x0102,
                "opcode {opcode:02X} should fall through"
            );
        }
    }

    #[test]
    fn loop_variants_and_jcxz() {
        let mut cpu = cpu_with_code(&[0xE2, 0xFE]);
        cpu.regs.cx = 2;
        cpu.step().unwrap();
        assert_eq!(cpu.regs.cx, 1);
        assert_eq!(cpu.regs.ip, 0x0100);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.cx, 0);
        assert_eq!(cpu.regs.ip, 0x0102);

        let mut cpu = cpu_with_code(&[0xE1, 0xFE]);
        cpu.regs.cx = 2;
        cpu.regs.flags.set(Flags::ZF, true);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.cx, 1);
        assert_eq!(cpu.regs.ip, 0x0100);

        let mut cpu = cpu_with_code(&[0xE1, 0xFE]);
        cpu.regs.cx = 2;
        cpu.step().unwrap();
        assert_eq!(cpu.regs.cx, 1);
        assert_eq!(cpu.regs.ip, 0x0102);

        let mut cpu = cpu_with_code(&[0xE0, 0xFE]);
        cpu.regs.cx = 2;
        cpu.step().unwrap();
        assert_eq!(cpu.regs.cx, 1);
        assert_eq!(cpu.regs.ip, 0x0100);

        let mut cpu = cpu_with_code(&[0xE0, 0xFE]);
        cpu.regs.cx = 2;
        cpu.regs.flags.set(Flags::ZF, true);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.cx, 1);
        assert_eq!(cpu.regs.ip, 0x0102);

        let mut cpu = cpu_with_code(&[0xE2, 0x02]);
        cpu.regs.cx = 0;
        cpu.step().unwrap();
        assert_eq!(cpu.regs.cx, 0xFFFF);
        assert_eq!(cpu.regs.ip, 0x0104);

        let mut cpu = cpu_with_code(&[0xE3, 0x02]);
        cpu.regs.cx = 0;
        cpu.step().unwrap();
        assert_eq!(cpu.regs.cx, 0);
        assert_eq!(cpu.regs.ip, 0x0104);

        let mut cpu = cpu_with_code(&[0xE3, 0x02]);
        cpu.regs.cx = 1;
        cpu.step().unwrap();
        assert_eq!(cpu.regs.cx, 1);
        assert_eq!(cpu.regs.ip, 0x0102);
    }

    #[test]
    fn near_call_and_ret_use_the_stack() {
        let mut cpu = cpu_with_code(&[0xE8, 0xFD, 0x00]);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.ip, 0x0200);
        assert_eq!(cpu.regs.sp, 0xFFFE);
        assert_eq!(cpu.mem.read16(cpu.regs.ss, 0xFFFE), 0x0103);

        let mut cpu = cpu_with_code(&[0xC3]);
        cpu.regs.sp = 0xFFFC;
        cpu.mem.write16(cpu.regs.ss, 0xFFFC, 0x1234);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.ip, 0x1234);
        assert_eq!(cpu.regs.sp, 0xFFFE);

        let mut cpu = cpu_with_code(&[0xC2, 0x04, 0x00]);
        cpu.regs.sp = 0xFFFC;
        cpu.mem.write16(cpu.regs.ss, 0xFFFC, 0x1234);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.ip, 0x1234);
        assert_eq!(cpu.regs.sp, 0x0002);
    }

    #[test]
    fn far_call_and_far_returns() {
        let mut cpu = cpu_with_code(&[0x9A, 0x34, 0x12, 0x78, 0x56]);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.ip, 0x1234);
        assert_eq!(cpu.regs.cs, 0x5678);
        assert_eq!(cpu.regs.sp, 0xFFFC);
        assert_eq!(cpu.mem.read16(cpu.regs.ss, 0xFFFC), 0x0105);
        assert_eq!(cpu.mem.read16(cpu.regs.ss, 0xFFFE), 0x0000);

        let mut cpu = cpu_with_code(&[0xCB]);
        cpu.regs.sp = 0xFFF8;
        cpu.mem.write16(cpu.regs.ss, 0xFFF8, 0x1234);
        cpu.mem.write16(cpu.regs.ss, 0xFFFA, 0x5678);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.ip, 0x1234);
        assert_eq!(cpu.regs.cs, 0x5678);
        assert_eq!(cpu.regs.sp, 0xFFFC);

        let mut cpu = cpu_with_code(&[0xCA, 0x04, 0x00]);
        cpu.regs.sp = 0xFFF8;
        cpu.mem.write16(cpu.regs.ss, 0xFFF8, 0x1234);
        cpu.mem.write16(cpu.regs.ss, 0xFFFA, 0x5678);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.ip, 0x1234);
        assert_eq!(cpu.regs.cs, 0x5678);
        assert_eq!(cpu.regs.sp, 0x0000);
    }

    #[test]
    fn indirect_calls_and_jumps() {
        let mut cpu = cpu_with_code(&[0xFF, 0xD0]);
        cpu.regs.ax = 0x0200;
        cpu.step().unwrap();
        assert_eq!(cpu.regs.ip, 0x0200);
        assert_eq!(cpu.regs.sp, 0xFFFE);
        assert_eq!(cpu.mem.read16(cpu.regs.ss, 0xFFFE), 0x0102);

        let mut cpu = cpu_with_code(&[0xFF, 0xE0]);
        cpu.regs.ax = 0x0300;
        cpu.step().unwrap();
        assert_eq!(cpu.regs.ip, 0x0300);
        assert_eq!(cpu.regs.sp, 0x0000);

        let mut cpu = cpu_with_code(&[0xFF, 0x1F]);
        cpu.regs.bx = 0x0200;
        cpu.mem.write16(0, 0x0200, 0x1234);
        cpu.mem.write16(0, 0x0202, 0x5678);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.ip, 0x1234);
        assert_eq!(cpu.regs.cs, 0x5678);
        assert_eq!(cpu.regs.sp, 0xFFFC);

        let mut cpu = cpu_with_code(&[0xFF, 0x2F]);
        cpu.regs.bx = 0x0200;
        cpu.mem.write16(0, 0x0200, 0x1234);
        cpu.mem.write16(0, 0x0202, 0x5678);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.ip, 0x1234);
        assert_eq!(cpu.regs.cs, 0x5678);
        assert_eq!(cpu.regs.sp, 0x0000);

        let mut cpu = cpu_with_code(&[0xFF, 0xD8]);
        assert_eq!(cpu.step(), Err(StepError::InvalidOpcode(0xFF)));

        let mut cpu = cpu_with_code(&[0xFF, 0xE8]);
        assert_eq!(cpu.step(), Err(StepError::InvalidOpcode(0xFF)));
    }

    #[test]
    fn interrupts_push_state_and_load_the_vector() {
        let mut cpu = cpu_with_code(&[0xCD, 0x21]);
        cpu.regs.sp = 0x1000;
        cpu.regs.flags.set(Flags::CF, true);
        cpu.regs.flags.set(Flags::IF, true);
        cpu.regs.flags.set(Flags::TF, true);
        cpu.mem.write16(0, 0x21 * 4, 0x1234);
        cpu.mem.write16(0, 0x21 * 4 + 2, 0x5678);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.ip, 0x1234);
        assert_eq!(cpu.regs.cs, 0x5678);
        assert!(!cpu.regs.flags.contains(Flags::IF));
        assert!(!cpu.regs.flags.contains(Flags::TF));
        assert!(cpu.regs.flags.contains(Flags::CF));
        assert_eq!(cpu.regs.sp, 0x0FFA);
        assert_eq!(cpu.mem.read16(cpu.regs.ss, 0x0FFA), 0x0102);
        assert_eq!(cpu.mem.read16(cpu.regs.ss, 0x0FFC), 0x0000);
        assert_eq!(
            cpu.mem.read16(cpu.regs.ss, 0x0FFE),
            Flags::power_on().bits() | Flags::CF | Flags::IF | Flags::TF
        );
    }

    #[test]
    fn iret_restores_ip_cs_and_flags() {
        let mut cpu = cpu_with_code(&[0xCF]);
        cpu.regs.sp = 0x0FFA;
        cpu.mem.write16(cpu.regs.ss, 0x0FFA, 0x1234);
        cpu.mem.write16(cpu.regs.ss, 0x0FFC, 0x5678);
        cpu.mem.write16(cpu.regs.ss, 0x0FFE, 0x0001);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.ip, 0x1234);
        assert_eq!(cpu.regs.cs, 0x5678);
        assert_eq!(cpu.regs.flags, Flags::from_bits(0x0001));
        assert_eq!(cpu.regs.sp, 0x1000);
    }

    #[test]
    fn int3_and_into_use_their_vectors() {
        let mut cpu = cpu_with_code(&[0xCC]);
        cpu.mem.write16(0, 3 * 4, 0x1234);
        cpu.mem.write16(0, 3 * 4 + 2, 0x5678);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.ip, 0x1234);
        assert_eq!(cpu.regs.cs, 0x5678);
        assert_eq!(cpu.mem.read16(cpu.regs.ss, 0xFFFA), 0x0101);

        let mut cpu = cpu_with_code(&[0xCE]);
        cpu.regs.flags.set(Flags::OF, true);
        cpu.mem.write16(0, 4 * 4, 0x2345);
        cpu.mem.write16(0, 4 * 4 + 2, 0x6789);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.ip, 0x2345);
        assert_eq!(cpu.regs.cs, 0x6789);

        let mut cpu = cpu_with_code(&[0xCE]);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.ip, 0x0101);
        assert_eq!(cpu.regs.sp, 0x0000);
    }

    #[test]
    fn dos_termination_vectors_halt_the_machine() {
        let mut cpu = cpu_with_code(&[0xCD, 0x20]);
        cpu.step().unwrap();
        assert!(cpu.halted);
        assert_eq!(cpu.regs.sp, 0x0000);

        let mut cpu = cpu_with_code(&[0xCD, 0x21]);
        cpu.regs.set8(Reg8::Ah, 0x4C);
        cpu.step().unwrap();
        assert!(cpu.halted);
        assert_eq!(cpu.regs.sp, 0x0000);

        let mut cpu = cpu_with_code(&[0xCD, 0x21]);
        cpu.regs.set8(Reg8::Ah, 0x09);
        cpu.step().unwrap();
        assert!(!cpu.halted);
        assert_eq!(cpu.regs.ip, 0x0000);
        assert_eq!(cpu.regs.cs, 0x0000);
        assert_eq!(cpu.regs.sp, 0xFFFA);
    }

    #[test]
    fn hlt_stops_on_the_opcode_even_with_a_prefix_or_ip_wraparound() {
        for (start, code, expected) in [
            (0x0100, &[0xF4][..], 0x0100),
            (0x0100, &[0x26, 0xF4][..], 0x0101),
            (0xFFFF, &[0xF4][..], 0xFFFF),
            (0xFFFF, &[0x26, 0xF4][..], 0x0000),
        ] {
            let mut cpu = Cpu::new();
            cpu.regs.ip = start;
            cpu.regs.flags = Flags(0x0202);
            for (index, &byte) in code.iter().enumerate() {
                cpu.mem.write8(0, start.wrapping_add(index as u16), byte);
            }
            let memory = cpu.mem.as_slice().to_vec();
            cpu.step().unwrap();
            assert!(cpu.halted);
            assert_eq!(cpu.regs.ip, expected);
            assert_eq!(cpu.regs.flags.bits(), 0x0202);
            assert_eq!(cpu.mem.as_slice(), memory);
        }
    }

    #[test]
    fn stepping_a_halted_cpu_preserves_its_state() {
        let mut cpu = cpu_with_code(&[0xF4, 0xB8, 0x34, 0x12]);
        cpu.step().unwrap();
        cpu.mem.write8(cpu.regs.cs, cpu.regs.ip, 0xB8);
        cpu.mem
            .write8(cpu.regs.cs, cpu.regs.ip.wrapping_add(1), 0x78);
        cpu.mem
            .write8(cpu.regs.cs, cpu.regs.ip.wrapping_add(2), 0x56);
        let registers = cpu.regs;
        let memory = cpu.mem.as_slice().to_vec();
        for _ in 0..2 {
            cpu.step().unwrap();
            assert!(cpu.halted);
            assert_eq!(cpu.regs, registers);
            assert_eq!(cpu.mem.as_slice(), memory);
        }
    }

    #[test]
    fn hlt_and_flag_control_instructions() {
        let mut cpu = cpu_with_code(&[0xF4]);
        cpu.step().unwrap();
        assert!(cpu.halted);

        let mut cpu = cpu_with_code(&[0xF9, 0xF8, 0xF5]);
        cpu.step().unwrap();
        assert!(cpu.regs.flags.contains(Flags::CF));
        cpu.step().unwrap();
        assert!(!cpu.regs.flags.contains(Flags::CF));
        cpu.step().unwrap();
        assert!(cpu.regs.flags.contains(Flags::CF));

        let mut cpu = cpu_with_code(&[0xFD, 0xFC]);
        cpu.step().unwrap();
        assert!(cpu.regs.flags.contains(Flags::DF));
        cpu.step().unwrap();
        assert!(!cpu.regs.flags.contains(Flags::DF));

        let mut cpu = cpu_with_code(&[0xFB, 0xFA]);
        cpu.step().unwrap();
        assert!(cpu.regs.flags.contains(Flags::IF));
        cpu.step().unwrap();
        assert!(!cpu.regs.flags.contains(Flags::IF));
    }
}
