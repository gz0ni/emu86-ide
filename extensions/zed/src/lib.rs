//! Zed extension for emu86.
//!
//! The extension only wires editor-side pieces: the tree-sitter grammar and
//! the debug adapter declaration. Everything else lives in the native
//! `emu86-dap` and `emu86-ui` binaries.

use zed_extension_api as zed;

struct Emu86Extension;

impl zed::Extension for Emu86Extension {
    fn new() -> Self {
        Self
    }
}

zed::register_extension!(Emu86Extension);
