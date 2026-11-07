//! `emu86-dap` — Debug Adapter Protocol server.
//!
//! The adapter talks stdio with the editor and exposes the machine state the
//! way both Zed and VS Code expect it:
//!
//! * `scopes` / `variables` — registers, flags, segments and the stack,
//! * `setVariable` — register editing from the debugger panel,
//! * `readMemory` / `writeMemory` — the memory window,
//! * `disassemble` — the disassembly view,
//! * `next` / `stepIn` / `stepOut`, breakpoints, `evaluate`.
//!
//! Nothing of that is implemented yet; this binary is a placeholder that keeps
//! the workspace, CI and packaging wiring honest.

fn main() {
    eprintln!(
        "emu86-dap {} — DAP server stub, no protocol implemented yet",
        env!("CARGO_PKG_VERSION")
    );
}
