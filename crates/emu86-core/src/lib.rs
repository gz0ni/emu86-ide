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

mod cpu;
mod decoder;
pub mod flags;
mod loader;
pub mod memory;
mod ops;
pub mod registers;

pub use cpu::Cpu;
pub use flags::Flags;
pub use loader::LoadError;
pub use memory::{Memory, MEMORY_SIZE};
pub use registers::{Reg16, Reg8, Registers};

/// Load address used by emu8086-style `.COM` programs (the PSP is placed
/// right before it).
pub const COM_LOAD_ADDRESS: u16 = 0x0100;

/// Result of executing a single instruction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepError {
    /// The opcode byte does not exist on the 8086.
    InvalidOpcode(u8),
    /// Division overflow or division by zero.
    DivideError,
    /// The instruction is valid but not implemented yet.
    Unimplemented,
}

impl core::fmt::Display for StepError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidOpcode(opcode) => write!(f, "invalid opcode 0x{opcode:02X}"),
            Self::DivideError => f.write_str("divide error"),
            Self::Unimplemented => f.write_str("instruction not implemented yet"),
        }
    }
}

impl std::error::Error for StepError {}

#[cfg(test)]
pub(crate) fn cpu_with_code(code: &[u8]) -> Cpu {
    cpu_with_code_at(0x0100, code)
}

#[cfg(test)]
pub(crate) fn cpu_with_code_at(address: usize, code: &[u8]) -> Cpu {
    let mut cpu = Cpu::new();
    cpu.regs.ip = address as u16;
    cpu.mem.load_linear(address, code);
    cpu
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
