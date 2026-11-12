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

Milestone: **M1 in progress (stages 1–4, golden-trace infrastructure, the first real reference capture and HLT compatibility done; deferred instructions remain open).** See `docs/ROADMAP.md` for the milestone list and `.agents/todo.md` for the live checklist.

### Done and verified

- Workspace of four crates builds offline with zero external dependencies.
- Instruction decoder foundation: ModR/M, effective addresses, segment overrides, prefix loop, and the data movement group (`MOV`, `XCHG`, `PUSH`, `POP`, `PUSHF`, `POPF`, `LAHF`, `SAHF`, `LEA`, `LDS`, `LES`, `XLAT`, `CBW`, `CWD`, `NOP`).
- Arithmetic and logic: `ADD`, `ADC`, `SUB`, `SBB`, `CMP`, `INC`, `DEC`, `NEG`, `NOT`, `TEST`, `AND`, `OR`, `XOR`, `MUL`, `IMUL`, `DIV`, `IDIV`, `DAA`, `DAS`, `AAA`, `AAS`, `AAM`, `AAD`, including group 1/3/4/5 forms and flag semantics.
- Shifts and rotates: `ROL`, `ROR`, `RCL`, `RCR`, `SHL`/`SAL`, `SHR`, `SAR` through `D0`-`D3`, including `CL` counts, count zero, and the 8086-only rejection of `C0`/`C1`.
- String instructions: `MOVS`, `CMPS`, `STOS`, `LODS`, `SCAS` with byte/word forms, `DF` direction, segment override on the source, and `REP`/`REPE`/`REPNE`.
- Control flow: short/near/far `JMP`, direct and indirect near/far `CALL`, `RET`/`RETF` with immediates, all 16 conditional jumps, `LOOP`/`LOOPE`/`LOOPNE`/`JCXZ`.
- Interrupts and machine control: `INT3`, `INT imm8`, `INTO`, `IRET`, `HLT`, plus `CLC`/`STC`/`CMC`, `CLD`/`STD` and `CLI`/`STI`.
- `Cpu::step` distinguishes `InvalidOpcode`, `DivideError` and `Unimplemented`; the prefix loop records `REP`/`REPNE` and passes it to the string handler.
- 133 core unit tests pass: register halves, flag bits, segment translation and 1 MiB wraparound, little-endian words and word accesses across segment boundaries, ModR/M and effective addresses, data movement and stack semantics, every ALU form, multiply/divide edge cases, BCD adjustment and decimal conversion, shift/rotate flags, string movement and repeat conditions, every conditional jump, loop variants, calls/returns, interrupts, flag-control instructions, HLT stopping IP and halted-step state preservation. Another 3 assembler unit tests cover source-map lookup and the assembler stub.
- `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` all green on Windows.
- 10 golden-harness integration tests pass, covering schema validation, instruction sequencing, register/flag/memory divergence, raw reference flags, initial memory dumps, the reference HLT stopping state and synthetic replay. PowerShell importer tests also pass, including replay of imported synthetic captures with and without initial memory.
- The first real emu8086 4.08 capture, `tests/golden/arithmetic-overflow/`, replays 3 instructions through register loading, signed overflow and a memory write. It preserves raw flags and the captured low 64 KiB; the accepted trace stops before HLT.
- HLT compatibility is verified against ordinary and prefixed reference probes: IP stays on the F4 opcode, and stepping a halted core does not mutate registers, flags or memory. The debugger supplies the observed halt marker; the reference export itself has none.
- CI matrix configured for Windows, macOS and Linux, plus a VS Code typecheck job.
- Documentation: spec, architecture, roadmap, four ADRs.
- Editor skeletons: VS Code grammar and manifest (valid, typechecks), Zed `extension.toml` and WASM crate skeleton (excluded from the workspace).
- Agent context: `AGENTS.md` plus the full `.agents/` set.
- CodeGraph MCP connected in OpenCode and Claude Code, telemetry off.

### Not done

- No loaders (`.com`, MZ `.exe`), no PSP, no BIOS/DOS interrupt handlers, no video, no keyboard.
- Interrupts load `CS:IP` from the IVT, but no handler bodies exist yet; `INT 20h` and `INT 21h` with `AH=4Ch` halt the machine instead of vectoring.
- `IN`/`OUT`, `WAIT` and the 8087 `ESC` opcodes still return `Unimplemented`; port I/O is blocked on the virtual-device list.
- `emu86_asm::assemble` returns `AssembleError::NotImplemented`.
- `emu86-dap` and `emu86-ui` are placeholders that print a version and exit.
- Zed extension has no tree-sitter grammar and declares no debug adapter yet.
- The reference capture and HLT follow-up do not establish compatibility for the full instruction set. Hardware interrupt wake-up and timing remain outside the current core.

### Open questions

- `.com` only, or full `.model small` with an MZ loader.
- Macros and conditional assembly in the lab corpus.
- Which virtual devices (ports) the course actually uses.

