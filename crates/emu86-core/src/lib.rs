//! Intel 8086 CPU core.
//!
//! Registers, 1 MiB of memory, flags and (eventually) full instruction
//! execution. The crate is editor-agnostic: it knows nothing about DAP, the
//! screen window or any other UI. Everything a debugger needs is exposed as
//! plain data plus small, predictable methods, so the DAP layer can read and
//! write state without reaching into instruction decoding.
//!
//! # Design notes
//!
//! * Memory is a flat 1 MiB array. Real-mode addresses are formed as
//!   `segment * 16 + offset` and wrapped at 1 MiB.
//! * Speed is not a goal. Correctness and a fully observable machine state
//!   are, because golden-trace tests compare emu86 against emu8086 step by
//!   step.

#![forbid(unsafe_code)]

pub mod flags;
pub mod memory;
pub mod registers;

pub use flags::Flags;
pub use memory::{Memory, MEMORY_SIZE};
pub use registers::{Reg16, Reg8, Registers};

/// Load address used by emu8086-style `.COM` programs (the PSP is placed
/// right before it).
pub const COM_LOAD_ADDRESS: u16 = 0x0100;

/// Result of executing a single instruction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepError {
    /// The decoder does not know this opcode (yet).
    Unimplemented,
}

impl core::fmt::Display for StepError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Unimplemented => f.write_str("instruction not implemented yet"),
        }
    }
}

impl std::error::Error for StepError {}

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
    /// Returns [`StepError::Unimplemented`] while the decoder is still missing.
    pub fn step(&mut self) -> Result<(), StepError> {
        Err(StepError::Unimplemented)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fresh_machine_has_power_on_flags_and_no_halt() {
        let cpu = Cpu::new();
        assert_eq!(cpu.regs.flags, Flags::power_on());
        assert!(!cpu.halted);
    }
}
