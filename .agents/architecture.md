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

- responsibility: machine state and instruction execution — registers, flags, 1 MiB memory, decoder.
- key files: `cpu.rs` (`Cpu`, instruction fetch, prefix loop, `step`, `Repeat`), `decoder.rs` (ModR/M, operands, effective addresses), `ops/mod.rs` (opcode dispatch), `ops/alu.rs` (shared flag helpers and the eight binary ALU forms), `ops/arith.rs` (group 1/3/4/5, BCD, AAM/AAD), `ops/logic.rs` (TEST and NOT), `ops/shift.rs` (group 2 shifts and rotates), `ops/string.rs` (string instructions and REP), `ops/control.rs` (jumps, calls, returns, loops, interrupts and flag control), `ops/data.rs` (data movement), `lib.rs` (`StepError`, re-exports), `registers.rs`, `flags.rs`, `memory.rs`.
- notable details: `#![forbid(unsafe_code)]`; `Memory::physical` wraps at 1 MiB like a real 8086; `read16`/`write16` advance the linear address, so words crossing a 64 KiB segment boundary are correct; `Flags::power_on()` sets reserved bits `0xF002`; `Memory` has a hand-written `Debug` that prints only the size, because the default would dump 1 MiB. `Cpu::step` consumes segment/REP/LOCK prefixes, passes `REP`/`REPNE` to the string handler, returns `InvalidOpcode` for non-8086 opcodes and `Unimplemented` for valid instructions not written yet. `ops/alu.rs` owns `Width`, `BinaryOp`, `add`/`sub`/`logic`/`increment`/`decrement`/`negate` and the parity helper, so arithmetic and logic handlers share one flag implementation. `ops/control.rs` pushes `FLAGS`/`CS`/`IP` before clearing `IF`/`TF`, reads vectors from the IVT at segment 0, and halts on `INT 20h` and `INT 21h` with `AH=4Ch` instead of vectoring.
- rule: this crate must never learn about DAP, files or UI.
- halt behavior: HLT restores IP to its F4 opcode after any consumed prefixes; `Cpu::step` returns without mutation while `halted` is set. DOS termination shortcut IP behavior is unchanged. Hardware wake-up is not implemented.
- COM loader: `loader.rs` adds `Cpu::from_com(&[u8]) -> Result<Cpu, LoadError>` and public `LoadError::{EmptyImage, ImageTooLarge(usize)}`. It constructs fresh state, copies at `0100:0100`, initializes a minimal PSP and shared segment registers, and reserves a zero return word at `0100:FFFE`. It accepts 1–65278 bytes, has no file I/O and preserves the core's default flags convention.

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

## Golden Replay Infrastructure

- `crates/emu86-core/tests/golden.rs` loads flat images at physical `0x0100`, or initializes physical `[0, 0x10000)` from an optional exact 65536-byte `initial-memory.bin`. With a dump, the nonempty image must match memory at the first physical CS:IP and fit the checksum window. It initializes registers and raw flags from the first pre-instruction snapshot, steps once per trace row and checks final state. Its schema parser and file access stay outside the core library.
- State comparison covers all registers, the full unnormalized flags word, halt state and FNV-1a over physical `[0x0100, 0x10000)`. No screen comparison or initialization above the low 64 KiB yet.
- `tools/capture-trace.ps1` validates and imports manually prepared reference snapshots with PowerShell 7, optionally accepting `-InitialMemory`. `tools/tests/capture-trace.tests.ps1` verifies rejected inputs and replays imported synthetic captures with and without dumps using `EMU86_GOLDEN_ROOT`.
- `tests/golden/arithmetic-overflow/` is the first real emu8086 4.08 capture: 3 instructions ending before HLT. Its provenance documents reference state extraction and actual memory checksums. A separate regression extends those recorded states through observed HLT; `tests/golden/hlt-observations.md` documents ordinary/prefixed probes and repeated stepping. Deferred instructions still keep M1 open.

## Feature Layer

### Editor extensions

- `extensions/vscode`: `package.json` contributes language `asm86` (`.asm`, `.inc`), TextMate grammar `source.asm86`, command `emu86.about`. `tsconfig` outputs to `out/`.
- `extensions/zed`: `extension.toml` declares id `emu86`; grammar and debug adapter sections are commented out until the grammar repository exists.
- Both editors are expected to consume the same DAP server; see `docs/adr/0001-dap-and-companion-ui.md`.

## Known Gaps

- Decoder covers prefixes, data movement, arithmetic, logic, BCD, the group 2 shift/rotate forms, the group 3/4/5 ALU forms, string instructions with `REP`/`REPE`/`REPNE`, control flow (jumps, calls, returns, loops) and `INT`/`IRET`/`HLT`/flag control.
- No MZ .exe loader, full DOS PSP, BIOS/DOS interrupt handlers, video or keyboard. COM loading from bytes works with a minimal PSP; arguments, environment, handles and reference startup-state verification remain open.
- `IN`/`OUT`, `WAIT` and the 8087 `ESC` opcodes return `Unimplemented`; port I/O depends on the virtual-device list.
- `emu86-asm::assemble` does nothing.
- DAP server and companion window are placeholders.
- `extensions/zed` has no tree-sitter grammar and is excluded from the workspace, so `cargo build --workspace` does not cover it.
- The first golden trace and HLT follow-up cover only register loading, signed overflow, a memory write and the stopping state. Other instruction groups still need real reference captures.

## Roadmap

1. M1: decoder and flags — see `docs/ROADMAP.md`.
2. M2: loaders, interrupts, text video.
3. M3: assembler pipeline and source map from listing.
4. M4–M6: DAP server, VS Code extension, Zed extension.
