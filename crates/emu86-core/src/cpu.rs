//! CPU state, instruction fetch and the step loop.

use crate::decoder::Segment;
use crate::memory::Memory;
use crate::ops;
use crate::registers::Registers;
use crate::StepError;

/// The repeat prefix attached to a string instruction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Repeat {
    /// `REP` / `REPE`, encoded as `F3`.
    WhileZero,
    /// `REPNE`, encoded as `F2`.
    WhileNotZero,
}

/// The emulated machine: CPU state plus memory.
#[derive(Debug, Clone, Default)]
pub struct Cpu {
    /// Architectural registers.
    pub regs: Registers,
    /// 1 MiB of addressable memory.
    pub mem: Memory,
    /// Set when the program terminated (INT 20h, INT 21h AH=4Ch, or HLT).
    pub halted: bool,
}

impl Cpu {
    /// Creates a machine in its power-on state.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Executes exactly one instruction and updates the machine state.
    ///
    /// # Errors
    ///
    /// Returns [`StepError::InvalidOpcode`] for opcodes that do not exist on
    /// the 8086, [`StepError::DivideError`] when a division cannot be
    /// represented, and [`StepError::Unimplemented`] for instructions the
    /// decoder does not handle yet.
    pub fn step(&mut self) -> Result<(), StepError> {
        if self.halted {
            return Ok(());
        }

        let mut segment_override = None;
        let mut repeat = None;

        loop {
            let opcode = self.fetch8();
            match opcode {
                0x26 => segment_override = Some(Segment::Es),
                0x2E => segment_override = Some(Segment::Cs),
                0x36 => segment_override = Some(Segment::Ss),
                0x3E => segment_override = Some(Segment::Ds),
                0xF2 => repeat = Some(Repeat::WhileNotZero),
                0xF3 => repeat = Some(Repeat::WhileZero),
                0xF0 => {}
                _ => return ops::execute(self, opcode, segment_override, repeat),
            }
        }
    }

    pub(crate) fn fetch8(&mut self) -> u8 {
        let value = self.mem.read8(self.regs.cs, self.regs.ip);
        self.regs.ip = self.regs.ip.wrapping_add(1);
        value
    }

    pub(crate) fn fetch16(&mut self) -> u16 {
        let low = self.fetch8();
        let high = self.fetch8();
        u16::from_le_bytes([low, high])
    }

    pub(crate) fn push16(&mut self, value: u16) {
        self.regs.sp = self.regs.sp.wrapping_sub(2);
        self.mem.write16(self.regs.ss, self.regs.sp, value);
    }

    pub(crate) fn pop16(&mut self) -> u16 {
        let value = self.mem.read16(self.regs.ss, self.regs.sp);
        self.regs.sp = self.regs.sp.wrapping_add(2);
        value
    }
}
