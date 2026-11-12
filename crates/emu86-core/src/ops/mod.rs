//! Opcode dispatch.

mod alu;
mod arith;
mod control;
mod data;
mod logic;
mod shift;
mod string;

use crate::cpu::{Cpu, Repeat};
use crate::decoder::Segment;
use crate::StepError;

pub(crate) fn execute(
    cpu: &mut Cpu,
    opcode: u8,
    segment_override: Option<Segment>,
    repeat: Option<Repeat>,
) -> Result<(), StepError> {
    match opcode {
        0x00..=0x05
        | 0x08..=0x0D
        | 0x10..=0x15
        | 0x18..=0x1D
        | 0x20..=0x25
        | 0x28..=0x2D
        | 0x30..=0x35
        | 0x38..=0x3D => alu::binary(cpu, opcode, segment_override),
        0x27 => arith::daa(cpu),
        0x2F => arith::das(cpu),
        0x37 => arith::aaa(cpu),
        0x3F => arith::aas(cpu),
        0x40..=0x4F => arith::inc_dec_reg(cpu, opcode),
        0x70..=0x7F => control::conditional_jump(cpu, opcode),
        0x80..=0x83 => arith::group_80(cpu, opcode, segment_override),
        0x84 | 0x85 => logic::test_rm_reg(cpu, opcode, segment_override),
        0x88..=0x8B => data::mov_rm_reg(cpu, opcode, segment_override),
        0x8C | 0x8E => data::mov_segment(cpu, opcode, segment_override),
        0xA0..=0xA3 => data::mov_moffs(cpu, opcode, segment_override),
        0xA4..=0xA7 | 0xAA..=0xAF => string::execute(cpu, opcode, segment_override, repeat),
        0xA8 | 0xA9 => logic::test_accumulator(cpu, opcode),
        0xB0..=0xB7 => data::mov_imm8(cpu, opcode),
        0xB8..=0xBF => data::mov_imm16(cpu, opcode),
        0xC6 | 0xC7 => data::mov_imm_rm(cpu, opcode, segment_override),
        0xD0..=0xD3 => shift::group_d0(cpu, opcode, segment_override),
        0xD4 => arith::aam(cpu),
        0xD5 => arith::aad(cpu),
        0xF6 | 0xF7 => arith::group_f6(cpu, opcode, segment_override),
        0xFE => arith::group_fe(cpu, segment_override),
        0xFF => arith::group_ff(cpu, segment_override),
        0x86 | 0x87 => data::xchg_rm_reg(cpu, opcode, segment_override),
        0x90..=0x97 => data::xchg_ax(cpu, opcode),
        0x50..=0x57 => data::push_reg(cpu, opcode),
        0x06 | 0x0E | 0x16 | 0x1E => data::push_segment(cpu, opcode),
        0x58..=0x5F => data::pop_reg(cpu, opcode),
        0x07 | 0x0F | 0x17 | 0x1F => data::pop_segment(cpu, opcode),
        0x8F => data::pop_rm(cpu, segment_override),
        0x9C => data::pushf(cpu),
        0x9D => data::popf(cpu),
        0x9E => data::sahf(cpu),
        0x9F => data::lahf(cpu),
        0x8D => data::lea(cpu, segment_override),
        0xC4 | 0xC5 => data::load_far_pointer(cpu, opcode, segment_override),
        0xD7 => data::xlat(cpu, segment_override),
        0x98 => data::cbw(cpu),
        0x99 => data::cwd(cpu),
        0x9A => control::call_far(cpu),
        0xC2 | 0xC3 | 0xCA | 0xCB => control::ret(cpu, opcode),
        0xCC => control::int3(cpu),
        0xCD => control::int_imm(cpu),
        0xCE => control::into(cpu),
        0xCF => control::iret(cpu),
        0xE0..=0xE2 => control::loop_instruction(cpu, opcode),
        0xE3 => control::jcxz(cpu),
        0xE8 => control::call_near(cpu),
        0xE9 => control::jump_near(cpu),
        0xEA => control::jump_far(cpu),
        0xEB => control::jump_short(cpu),
        0xF4 => control::hlt(cpu),
        0xF5 => control::cmc(cpu),
        0xF8 => control::clc(cpu),
        0xF9 => control::stc(cpu),
        0xFA => control::cli(cpu),
        0xFB => control::sti(cpu),
        0xFC => control::cld(cpu),
        0xFD => control::std(cpu),
        _ if is_invalid_opcode(opcode) => Err(StepError::InvalidOpcode(opcode)),
        _ => Err(StepError::Unimplemented),
    }
}

const fn is_invalid_opcode(opcode: u8) -> bool {
    matches!(opcode, 0x60..=0x6F | 0xC0 | 0xC1 | 0xC8 | 0xC9 | 0xF1)
}
