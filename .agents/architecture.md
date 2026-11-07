# Architecture

<!-- Living map of the codebase: entry points, layers, key modules, data flow. Update when structure changes. -->

## Entry Point

Four independent artifacts, no single binary:

- `crates/emu86-dap/src/main.rs` — the only process editors talk to; stdio DAP.
- `crates/emu86-ui/src/main.rs` — companion window, standalone or attached to a session.
- `extensions/vscode/src/extension.ts` — `activate()` registers `emu86.about`; will spawn the adapter.
- `extensions/zed/src/lib.rs` — WASM extension implementing `zed::Extension`.

## Core Layer

### emu86-core

- responsibility: machine state only — registers, flags, 1 MiB memory, later the instruction decoder.
- key files: `lib.rs` (Cpu, StepError), `registers.rs` (Reg16, Reg8, Registers), `flags.rs` (Flags), `memory.rs` (Memory, physical translation).
- notable details: `#![forbid(unsafe_code)]`; `Memory::physical` wraps at 1 MiB like a real 8086; `Flags::power_on()` sets reserved bits `0xF002`; `Memory` has a hand-written `Debug` that prints only the size, because the default would dump 1 MiB.
- rule: this crate must never learn about DAP, files or UI.

### emu86-asm

- responsibility: everything around the external assembler.
- key files: `lib.rs` (AssembleOptions, Artifact, AssembleError, `assemble()`), `sourcemap.rs` (SourceMap, SourceLocation).
- notable details: `assemble()` currently returns `AssembleError::NotImplemented`. `SourceMap::lookup` uses `partition_point` and must be preceded by `sort()`.

### emu86-dap

- responsibility: DAP framing and mapping machine state to scopes/variables.
- notable details: stub — prints version to stderr and exits. Planned requests listed in `docs/SPEC.md` §6.

### emu86-ui

- responsibility: 80x25 text screen, hex dump, keyboard.
- notable details: stub. Planned stack `egui`/`eframe`; not added yet to keep the workspace dependency-free and offline-buildable.

## Feature Layer

### Editor extensions

- `extensions/vscode`: `package.json` contributes language `asm86` (`.asm`, `.inc`), TextMate grammar `source.asm86`, command `emu86.about`. `tsconfig` outputs to `out/`.
- `extensions/zed`: `extension.toml` declares id `emu86`; grammar and debug adapter sections are commented out until the grammar repository exists.
- Both editors are expected to consume the same DAP server; see `docs/adr/0001-dap-and-companion-ui.md`.

## Known Gaps

- `Cpu::step` returns `StepError::Unimplemented`.
- No instruction decoder, no loaders (.com, MZ .exe), no interrupts, no video.
- `emu86-asm::assemble` does nothing.
- DAP server and companion window are placeholders.
- `extensions/zed` has no tree-sitter grammar and is excluded from the workspace, so `cargo build --workspace` does not cover it.
- No golden traces captured yet.

## Roadmap

1. M1: decoder and flags — see `docs/ROADMAP.md`.
2. M2: loaders, interrupts, text video.
3. M3: assembler pipeline and source map from listing.
4. M4–M6: DAP server, VS Code extension, Zed extension.
