# Project Context

emu86 is an open-source Intel 8086 toolchain and debugger that replaces emu8086 for university coursework while keeping its source dialect and observable behaviour. It provides a MASM-compatible assembler front-end, an 8086 emulator with DOS/BIOS interrupt subset, and a Debug Adapter Protocol server so that Zed and VS Code supply the editor and the debug UI. The emulated 80x25 text screen, hex dump and keyboard input live in a separate native companion window shared by both editors.

## Platforms

- Windows (primary, university machines)
- Linux
- macOS

Editors: Zed (primary target) and VS Code.

## Toolchain

- Rust, edition 2021, `rust-version = 1.75`, workspace with four crates.
- `rust-toolchain.toml` pins channel `stable` with `rustfmt` and `clippy`.
- No external Rust dependencies yet — the workspace builds offline.
- VS Code extension: TypeScript 5.4, Node 20, compiled with `tsc`.
- Zed extension: WASM crate against `zed_extension_api`.
- CI: GitHub Actions, matrix over `ubuntu-latest`, `windows-latest`, `macos-latest`.
- External tool (not vendored): JWasm, a MASM-compatible assembler invoked as a child process.

## Package / Repository

- `name:` emu86
- `version:` 0.0.0 (pre-alpha)
- `org:` gz0ni — https://github.com/gz0ni/emu86-ide
- Crates: `emu86-core`, `emu86-asm`, `emu86-dap`, `emu86-ui`
- VS Code publisher: `gz0ni`; extension id `gz0ni.emu86`

## Dependencies

- Rust workspace: none.
- Planned: `egui`/`eframe` for the companion window, `serde`/`serde_json` for DAP.
- External binaries: JWasm (assembler). VS Code extension may bundle it; the Zed extension cannot and needs a documented install step.
- No dependency may be added to `emu86-core`.

## Design Basis

- [docs/SPEC.md](../docs/SPEC.md): functional specification, source dialect, interrupts, non-goals.
- [docs/ARCHITECTURE.md](../docs/ARCHITECTURE.md): components and data flow.
- [docs/ROADMAP.md](../docs/ROADMAP.md): milestones M0–M7 with acceptance criteria.
- [docs/adr/](../docs/adr/): accepted architecture decisions.

## Status

Milestone: **M0 done, M1 not started.** See `docs/ROADMAP.md` for the milestone list and `.agents/todo.md` for the live checklist.

### Done and verified

- Workspace of four crates builds offline with zero external dependencies.
- 14 unit tests pass: register halves, flag bits, segment translation and 1 MiB wraparound, little-endian words, source-map lookup.
- `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` all green on Windows.
- CI matrix configured for Windows, macOS and Linux, plus a VS Code typecheck job.
- Documentation: spec, architecture, roadmap, four ADRs.
- Editor skeletons: VS Code grammar and manifest (valid, typechecks), Zed `extension.toml` and WASM crate skeleton (excluded from the workspace).
- Agent context: `AGENTS.md` plus the full `.agents/` set.
- CodeGraph MCP connected in OpenCode and Claude Code, telemetry off.

### Not done

- No instruction decoder: `Cpu::step` returns `StepError::Unimplemented`.
- No loaders (`.com`, MZ `.exe`), no PSP, no interrupts, no video, no keyboard.
- `emu86_asm::assemble` returns `AssembleError::NotImplemented`.
- `emu86-dap` and `emu86-ui` are placeholders that print a version and exit.
- Zed extension has no tree-sitter grammar and declares no debug adapter yet.
- No golden traces captured from emu8086.

### Open questions

- `.com` only, or full `.model small` with an MZ loader.
- Macros and conditional assembly in the lab corpus.
- Which virtual devices (ports) the course actually uses.

