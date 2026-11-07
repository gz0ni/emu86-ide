//! Assembler front-end.
//!
//! The heavy lifting is delegated to an external MASM-compatible assembler
//! (JWasm): it understands the dialect used by university lab manuals
//! (`.model`, `assume`, `proc`, `dup`, macros, conditional assembly). This
//! crate owns the parts JWasm does not know about:
//!
//! * a preprocessor for emu8086-specific `#`-directives,
//! * invocation of the assembler binary,
//! * parsing of the listing file into a [`SourceMap`],
//! * a minimal MZ (`.EXE`) loader for `.model small` programs.
//!
//! See `docs/adr/0002-jwasm-as-external-binary.md` for why the assembler is a
//! separate process rather than a linked library.

#![forbid(unsafe_code)]

pub mod sourcemap;

use std::path::PathBuf;

pub use sourcemap::{SourceLocation, SourceMap};

/// Options for a single assembly run.
#[derive(Debug, Clone, Default)]
pub struct AssembleOptions {
    /// Path to the assembler binary. Falls back to `jwasm` on `PATH`.
    pub jwasm: Option<PathBuf>,
    /// Produce a flat `.COM` image instead of an MZ `.EXE`.
    pub com: bool,
}

/// Successful assembly result.
#[derive(Debug, Clone, Default)]
pub struct Artifact {
    /// Raw machine code to load into the emulated machine.
    pub image: Vec<u8>,
    /// Address-to-source mapping for source level debugging.
    pub source_map: SourceMap,
}

/// Everything that can go wrong while assembling.
#[derive(Debug)]
pub enum AssembleError {
    /// The assembler pipeline is not wired up yet.
    NotImplemented,
    /// The assembler binary could not be started.
    Spawn(std::io::Error),
    /// The assembler reported errors.
    Assembler(String),
}

impl core::fmt::Display for AssembleError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NotImplemented => f.write_str("assembler is not implemented yet"),
            Self::Spawn(err) => write!(f, "failed to start assembler: {err}"),
            Self::Assembler(msg) => write!(f, "assembler failed: {msg}"),
        }
    }
}

impl std::error::Error for AssembleError {}

/// Assembles `source` into a loadable image plus a source map.
///
/// # Errors
///
/// Returns [`AssembleError::NotImplemented`] until the JWasm pipeline lands.
pub fn assemble(_source: &str, _options: &AssembleOptions) -> Result<Artifact, AssembleError> {
    Err(AssembleError::NotImplemented)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assemble_reports_not_implemented_for_now() {
        let err = assemble("mov ax, 4C00h\n", &AssembleOptions::default()).unwrap_err();
        assert!(matches!(err, AssembleError::NotImplemented));
    }
}
