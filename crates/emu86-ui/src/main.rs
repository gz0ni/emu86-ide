//! `emu86-ui` — companion window.
//!
//! Renders the parts of the machine that editors cannot show on their own:
//!
//! * the 80x25 text screen (video memory at `B800:0000`) with colours and the
//!   hardware cursor,
//! * a navigable hex dump of the 1 MiB address space,
//! * keyboard input routed to `INT 16h` / `INT 21h`.
//!
//! The window is shared by the Zed and VS Code integrations so the experience
//! is identical in both editors. It also runs standalone (`emu86-ui prog.com`)
//! for people who just want to run a lab program from a terminal.
//!
//! Planned stack: `egui`/`eframe` over a local socket to the DAP session.
//! For now this is a placeholder binary; adding a GUI toolkit is a separate
//! change so the workspace stays dependency-free and builds offline.

fn main() {
    eprintln!(
        "emu86-ui {} — companion window stub, no UI implemented yet",
        env!("CARGO_PKG_VERSION")
    );
}
