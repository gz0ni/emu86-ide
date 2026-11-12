//! String instructions and their repeat prefixes.

use crate::cpu::{Cpu, Repeat};
use crate::decoder::Segment;
use crate::flags::Flags;
use crate::ops::alu::{self, Width};
use crate::registers::Reg8;
use crate::StepError;

pub(super) fn execute(
    cpu: &mut Cpu,
    opcode: u8,
    segment_override: Option<Segment>,
    repeat: Option<Repeat>,
) -> Result<(), StepError> {
    let width = if opcode & 1 == 0 {
        Width::Byte
    } else {
        Width::Word
    };
    let conditional = matches!(opcode, 0xA6 | 0xA7 | 0xAE | 0xAF);
    let repeated = repeat.is_some();

    loop {
        if repeated && cpu.regs.cx == 0 {
            break;
        }

        match opcode {
            0xA4 | 0xA5 => move_string(cpu, segment_override, width),
            0xA6 | 0xA7 => compare_string(cpu, segment_override, width),
            0xAA | 0xAB => store_string(cpu, width),
            0xAC | 0xAD => load_string(cpu, segment_override, width),
            0xAE | 0xAF => scan_string(cpu, width),
            _ => unreachable!(),
        }

        if !repeated {
            break;
        }

        cpu.regs.cx = cpu.regs.cx.wrapping_sub(1);

        if conditional {
            let zero = cpu.regs.flags.contains(Flags::ZF);
            match repeat {
                Some(Repeat::WhileZero) if !zero => break,
                Some(Repeat::WhileNotZero) if zero => break,
                _ => {}
            }
        }
    }

    Ok(())
}

fn move_string(cpu: &mut Cpu, segment_override: Option<Segment>, width: Width) {
    let value = read_source(cpu, segment_override, width);
    write_destination(cpu, width, value);
    advance_both(cpu, width);
}

fn compare_string(cpu: &mut Cpu, segment_override: Option<Segment>, width: Width) {
    let source = read_source(cpu, segment_override, width);
    let destination = read_destination(cpu, width);
    alu::sub(&mut cpu.regs.flags, source, destination, false, width);
    advance_both(cpu, width);
}

fn store_string(cpu: &mut Cpu, width: Width) {
    let value = match width {
        Width::Byte => u16::from(cpu.regs.get8(Reg8::Al)),
        Width::Word => cpu.regs.ax,
    };
    write_destination(cpu, width, value);
    advance_di(cpu, width);
}

fn load_string(cpu: &mut Cpu, segment_override: Option<Segment>, width: Width) {
    let value = read_source(cpu, segment_override, width);
    match width {
        Width::Byte => cpu.regs.set8(Reg8::Al, value as u8),
        Width::Word => cpu.regs.ax = value,
    }
    advance_si(cpu, width);
}

fn scan_string(cpu: &mut Cpu, width: Width) {
    let accumulator = match width {
        Width::Byte => u16::from(cpu.regs.get8(Reg8::Al)),
        Width::Word => cpu.regs.ax,
    };
    let destination = read_destination(cpu, width);
    alu::sub(&mut cpu.regs.flags, accumulator, destination, false, width);
    advance_di(cpu, width);
}

fn read_source(cpu: &Cpu, segment_override: Option<Segment>, width: Width) -> u16 {
    let segment = cpu
        .regs
        .get(segment_override.unwrap_or(Segment::Ds).register());
    match width {
        Width::Byte => u16::from(cpu.mem.read8(segment, cpu.regs.si)),
        Width::Word => cpu.mem.read16(segment, cpu.regs.si),
    }
}

fn read_destination(cpu: &Cpu, width: Width) -> u16 {
    match width {
        Width::Byte => u16::from(cpu.mem.read8(cpu.regs.es, cpu.regs.di)),
        Width::Word => cpu.mem.read16(cpu.regs.es, cpu.regs.di),
    }
}

fn write_destination(cpu: &mut Cpu, width: Width, value: u16) {
    match width {
        Width::Byte => cpu.mem.write8(cpu.regs.es, cpu.regs.di, value as u8),
        Width::Word => cpu.mem.write16(cpu.regs.es, cpu.regs.di, value),
    }
}

fn delta(cpu: &Cpu, width: Width) -> u16 {
    let step = match width {
        Width::Byte => 1,
        Width::Word => 2,
    };
    if cpu.regs.flags.contains(Flags::DF) {
        0u16.wrapping_sub(step)
    } else {
        step
    }
}

fn advance_both(cpu: &mut Cpu, width: Width) {
    let delta = delta(cpu, width);
    cpu.regs.si = cpu.regs.si.wrapping_add(delta);
    cpu.regs.di = cpu.regs.di.wrapping_add(delta);
}

fn advance_si(cpu: &mut Cpu, width: Width) {
    let delta = delta(cpu, width);
    cpu.regs.si = cpu.regs.si.wrapping_add(delta);
}

fn advance_di(cpu: &mut Cpu, width: Width) {
    let delta = delta(cpu, width);
    cpu.regs.di = cpu.regs.di.wrapping_add(delta);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu_with_code;

    #[test]
    fn movsb_and_movsw_copy_and_advance() {
        let mut cpu = cpu_with_code(&[0xA4]);
        cpu.regs.si = 0x0300;
        cpu.regs.di = 0x0400;
        cpu.mem.write8(0, 0x0300, 0xAB);
        cpu.step().unwrap();
        assert_eq!(cpu.mem.read8(cpu.regs.es, 0x0400), 0xAB);
        assert_eq!(cpu.regs.si, 0x0301);
        assert_eq!(cpu.regs.di, 0x0401);

        let mut cpu = cpu_with_code(&[0xA5]);
        cpu.regs.si = 0x0300;
        cpu.regs.di = 0x0400;
        cpu.mem.write16(0, 0x0300, 0x1234);
        cpu.step().unwrap();
        assert_eq!(cpu.mem.read16(cpu.regs.es, 0x0400), 0x1234);
        assert_eq!(cpu.regs.si, 0x0302);
        assert_eq!(cpu.regs.di, 0x0402);
    }

    #[test]
    fn direction_flag_reverses_the_step() {
        let mut cpu = cpu_with_code(&[0xA4]);
        cpu.regs.si = 0x0300;
        cpu.regs.di = 0x0400;
        cpu.regs.flags.set(Flags::DF, true);
        cpu.mem.write8(0, 0x0300, 0x7F);
        cpu.step().unwrap();
        assert_eq!(cpu.mem.read8(cpu.regs.es, 0x0400), 0x7F);
        assert_eq!(cpu.regs.si, 0x02FF);
        assert_eq!(cpu.regs.di, 0x03FF);
    }

    #[test]
    fn stos_and_lods_use_the_accumulator() {
        let mut cpu = cpu_with_code(&[0xAA]);
        cpu.regs.di = 0x0400;
        cpu.regs.set8(Reg8::Al, 0x55);
        cpu.step().unwrap();
        assert_eq!(cpu.mem.read8(cpu.regs.es, 0x0400), 0x55);
        assert_eq!(cpu.regs.di, 0x0401);

        let mut cpu = cpu_with_code(&[0xAB]);
        cpu.regs.di = 0x0400;
        cpu.regs.ax = 0x1234;
        cpu.step().unwrap();
        assert_eq!(cpu.mem.read16(cpu.regs.es, 0x0400), 0x1234);
        assert_eq!(cpu.regs.di, 0x0402);

        let mut cpu = cpu_with_code(&[0xAC]);
        cpu.regs.si = 0x0300;
        cpu.mem.write8(0, 0x0300, 0x66);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.get8(Reg8::Al), 0x66);
        assert_eq!(cpu.regs.si, 0x0301);

        let mut cpu = cpu_with_code(&[0xAD]);
        cpu.regs.si = 0x0300;
        cpu.mem.write16(0, 0x0300, 0x4321);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.ax, 0x4321);
        assert_eq!(cpu.regs.si, 0x0302);
    }

    #[test]
    fn cmps_and_scas_set_flags() {
        let mut cpu = cpu_with_code(&[0xA6]);
        cpu.regs.si = 0x0300;
        cpu.regs.di = 0x0400;
        cpu.mem.write8(0, 0x0300, 0x10);
        cpu.mem.write8(cpu.regs.es, 0x0400, 0x10);
        cpu.step().unwrap();
        assert!(cpu.regs.flags.contains(Flags::ZF));
        assert!(!cpu.regs.flags.contains(Flags::CF));

        let mut cpu = cpu_with_code(&[0xA6]);
        cpu.regs.si = 0x0300;
        cpu.regs.di = 0x0400;
        cpu.mem.write8(0, 0x0300, 0x10);
        cpu.mem.write8(cpu.regs.es, 0x0400, 0x20);
        cpu.step().unwrap();
        assert!(!cpu.regs.flags.contains(Flags::ZF));
        assert!(cpu.regs.flags.contains(Flags::CF));

        let mut cpu = cpu_with_code(&[0xAE]);
        cpu.regs.di = 0x0400;
        cpu.regs.set8(Reg8::Al, 0x10);
        cpu.mem.write8(cpu.regs.es, 0x0400, 0x10);
        cpu.step().unwrap();
        assert!(cpu.regs.flags.contains(Flags::ZF));
        assert_eq!(cpu.regs.di, 0x0401);
    }

    #[test]
    fn rep_movsb_copies_the_whole_string() {
        let mut cpu = cpu_with_code(&[0xF3, 0xA4]);
        cpu.regs.cx = 3;
        cpu.regs.si = 0x0300;
        cpu.regs.di = 0x0400;
        cpu.mem.load_linear(0x0300, &[1, 2, 3]);
        cpu.step().unwrap();
        assert_eq!(cpu.mem.read8(cpu.regs.es, 0x0400), 1);
        assert_eq!(cpu.mem.read8(cpu.regs.es, 0x0401), 2);
        assert_eq!(cpu.mem.read8(cpu.regs.es, 0x0402), 3);
        assert_eq!(cpu.regs.cx, 0);
        assert_eq!(cpu.regs.si, 0x0303);
        assert_eq!(cpu.regs.di, 0x0403);
    }

    #[test]
    fn rep_with_zero_cx_does_nothing() {
        let mut cpu = cpu_with_code(&[0xF3, 0xA4]);
        cpu.regs.cx = 0;
        cpu.regs.si = 0x0300;
        cpu.regs.di = 0x0400;
        cpu.step().unwrap();
        assert_eq!(cpu.regs.si, 0x0300);
        assert_eq!(cpu.regs.di, 0x0400);
        assert_eq!(cpu.mem.read8(cpu.regs.es, 0x0400), 0);
    }

    #[test]
    fn repe_cmpsb_stops_on_mismatch() {
        let mut cpu = cpu_with_code(&[0xF3, 0xA6]);
        cpu.regs.cx = 4;
        cpu.regs.si = 0x0300;
        cpu.regs.di = 0x0400;
        cpu.mem.load_linear(0x0300, b"ABCD");
        cpu.mem.load_linear(0x0400, b"ABXD");
        cpu.step().unwrap();
        assert_eq!(cpu.regs.cx, 1);
        assert_eq!(cpu.regs.si, 0x0303);
        assert_eq!(cpu.regs.di, 0x0403);
        assert!(!cpu.regs.flags.contains(Flags::ZF));
    }

    #[test]
    fn repne_scasb_stops_on_match() {
        let mut cpu = cpu_with_code(&[0xF2, 0xAE]);
        cpu.regs.cx = 5;
        cpu.regs.di = 0x0400;
        cpu.regs.set8(Reg8::Al, 0x00);
        cpu.mem.load_linear(0x0400, &[1, 2, 0, 4, 5]);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.cx, 2);
        assert_eq!(cpu.regs.di, 0x0403);
        assert!(cpu.regs.flags.contains(Flags::ZF));
    }

    #[test]
    fn rep_stosw_and_lodsb_repeat() {
        let mut cpu = cpu_with_code(&[0xF3, 0xAB]);
        cpu.regs.cx = 2;
        cpu.regs.di = 0x0400;
        cpu.regs.ax = 0x1234;
        cpu.step().unwrap();
        assert_eq!(cpu.mem.read16(cpu.regs.es, 0x0400), 0x1234);
        assert_eq!(cpu.mem.read16(cpu.regs.es, 0x0402), 0x1234);
        assert_eq!(cpu.regs.di, 0x0404);
        assert_eq!(cpu.regs.cx, 0);

        let mut cpu = cpu_with_code(&[0xF3, 0xAC]);
        cpu.regs.cx = 3;
        cpu.regs.si = 0x0300;
        cpu.mem.load_linear(0x0300, &[7, 8, 9]);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.get8(Reg8::Al), 9);
        assert_eq!(cpu.regs.si, 0x0303);
        assert_eq!(cpu.regs.cx, 0);
    }

    #[test]
    fn repne_movsb_repeats_unconditionally() {
        let mut cpu = cpu_with_code(&[0xF2, 0xA4]);
        cpu.regs.cx = 2;
        cpu.regs.si = 0x0300;
        cpu.regs.di = 0x0400;
        cpu.mem.load_linear(0x0300, &[4, 5]);
        cpu.step().unwrap();
        assert_eq!(cpu.mem.read8(cpu.regs.es, 0x0400), 4);
        assert_eq!(cpu.mem.read8(cpu.regs.es, 0x0401), 5);
        assert_eq!(cpu.regs.cx, 0);
    }

    #[test]
    fn segment_override_applies_to_the_source_only() {
        let mut cpu = cpu_with_code(&[0x26, 0xA4]);
        cpu.regs.ds = 0x1000;
        cpu.regs.es = 0x2000;
        cpu.regs.si = 0x0300;
        cpu.regs.di = 0x0400;
        cpu.mem.write8(0x1000, 0x0300, 0x11);
        cpu.mem.write8(0x2000, 0x0300, 0x22);
        cpu.step().unwrap();
        assert_eq!(cpu.mem.read8(0x2000, 0x0400), 0x22);
        assert_eq!(cpu.regs.si, 0x0301);
        assert_eq!(cpu.regs.di, 0x0401);
    }

    #[test]
    fn rep_prefix_on_a_non_string_instruction_is_ignored() {
        let mut cpu = cpu_with_code(&[0xF3, 0x90]);
        cpu.step().unwrap();
        assert_eq!(cpu.regs.ip, 0x0102);
    }
}
