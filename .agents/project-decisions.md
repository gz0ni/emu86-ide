# Project Decisions

Recorded decisions and the reasoning behind them, so future work does not silently contradict them. Add an entry whenever a non-trivial choice is made. Latest first.

## Format

```
### <Decision title> (YYYY-MM-DD)

- Decision: <what was chosen, one sentence>
- Reason: <why this and not the alternatives>
- Alternatives considered: <what was rejected and why>
- Consequences: <what this decision affects>
```

---

<!-- New entries go here, newest first. -->

### Preserve real reference memory and flags (2026-11-12)

- Decision: allow an optional 65536-byte `initial-memory.bin`, validate the exact program at the first physical CS:IP within the existing checksum window, and initialize replay flags from the raw recorded word without normalization.
- Reason: emu8086 4.08 rejects physical `0x0100`, initializes low memory and starts with `0202h` flags. Reference data must remain unchanged.
- Alternatives considered: relocate snapshots, replace memory hashes with emulator output, normalize reference flags or include a complete proprietary state file (all rejected because they lose reference fidelity or copy excluded content).
- Consequences: no core-library changes or dependencies; only low 64 KiB is initialized, with the remainder zero. The first real trace covers 3 instructions and ends before HLT; full M1 acceptance remains open for deferred instructions and the observed HLT question. This extends the earlier synthetic replay convention below.

### M1 golden replay remains test infrastructure (2026-11-12)

- Decision: replay flat images at physical `0x0100` with zero-filled memory, initialize registers and flags from the first pre-instruction snapshot, execute one instruction per row and compare one separate final state. Check memory with FNV-1a over physical `[0x0100, 0x10000)`.
- Reason: this provides deterministic CPU regression checks before DOS loaders and device initialization exist, while keeping file parsing out of the editor-agnostic library and adding no Rust dependencies.
- Alternatives considered: add serde to the core (forbidden dependency), fabricate reference traces from emu86 output (does not prove compatibility), automate the unavailable reference debugger (requires a separate capture workflow).
- Consequences: `capture-trace.ps1` is a PowerShell 7 importer of manually recorded snapshots, not a debugger recorder. Strictly validate and normalize captures, never overwrite existing directories, and keep M1 open until a real emu8086 trace passes. Memory outside the checksum window, initial memory dumps and screen comparison remain unsupported.

### Stage 4 control flow, interrupts and flag control (2026-11-07)

- Decision: `ops/control.rs` owns short/near/far `JMP`, direct and indirect `CALL`, `RET`/`RETF`, all 16 conditional jumps, `LOOP`/`LOOPE`/`LOOPNE`/`JCXZ`, `INT3`/`INT imm8`/`INTO`/`IRET`, `HLT` and the flag-control instructions. `INT` pushes `FLAGS`, `CS` and `IP` before clearing `IF`/`TF`, then loads `CS:IP` from the IVT at segment 0. `INT 20h` and `INT 21h` with `AH=4Ch` set `halted` without pushing. Near/far indirect pointers are read before the stack is modified; register operands are invalid for the far forms.
- Reason: matches 8086 execution order and gives the DAP a working halt path before DOS handlers exist. Pushing before clearing keeps the saved flags faithful to the interrupted program.
- Alternatives considered: leaving `INT` unimplemented until M2 (blocks program termination tests), pushing after clearing `IF`/`TF` (wrong saved state), treating register operands as far pointers (no such 8086 encoding).
- Consequences: `group_ff` selectors `/2`-`/5` delegate to control helpers; `INT 20h`/`INT 21h AH=4Ch` are temporary halt shortcuts that M2 must replace with real DOS handlers; `IRET` restores the full flags word including `IF`/`TF`.

### Stage 3 shifts, rotates and string semantics (2026-11-07)

- Decision: group 2 (`D0`-`D3`) implements `ROL`, `ROR`, `RCL`, `RCR`, `SHL`/`SAL`, `SHR`, `SAR`; selector `/6` is treated as `SAL` on the 8086, `C0`/`C1` are invalid, `CL` is used unmasked (0-255), count zero leaves all flags unchanged, rotates leave SF/ZF/PF alone, shifts leave AF alone, and OF is updated only for count 1. String instructions use DS:SI (overridable) and ES:DI (fixed), `DF` chooses the step, and `F3`/`F2` repeat while CX is non-zero, with the ZF condition applied only to `CMPS`/`SCAS`.
- Reason: matches the documented 8086 group 2 and string behaviour; treating `/6` as `SAL` matches the original decoder, and leaving undefined flags untouched keeps golden-trace comparisons stable.
- Alternatives considered: rejecting `/6` (would diverge from the 8086), masking `CL` to 5 bits (a 186+ behaviour), applying the ZF condition to `MOVS`/`STOS`/`LODS` (wrong).
- Consequences: `Cpu::step` carries `Repeat` from the prefix loop into the string handler; later control-flow work can reuse the prefix plumbing, and shift helpers stay local to `ops/shift.rs`.

### Stage 2 flag conventions and shared ALU helpers (2026-11-07)

- Decision: `ops/alu.rs` owns `Width`, `BinaryOp`, the flag helpers and the eight binary ALU forms; `ops/arith.rs` and `ops/logic.rs` reuse them. Logic clears CF/OF/AF; `MUL`/`IMUL` set only CF/OF; `DIV`/`IDIV` leave flags unchanged; BCD sets its defined flags and leaves the undefined ones unchanged; `AAM` with base 0 returns `DivideError`, `AAD` with base 0 does not.
- Reason: one flag implementation prevents arithmetic and logic from drifting apart, and the conventions match the documented 8086 behaviour while keeping undefined flags stable for debugger comparisons.
- Alternatives considered: duplicate flag code per handler (drift risk), set every flag on every instruction (would contradict the "undefined" bits and break golden traces).
- Consequences: stage 2 tests assert both defined flags and the preservation of undefined ones; later shift/rotate and string groups can reuse `Width` and `alu::read_rm`/`write_rm`.

### Decoder dispatches directly from opcode bytes (2026-11-07)

- Decision: `Cpu::step` matches the opcode byte and calls handlers in `ops/`; M1 has no public instruction AST.
- Reason: state-level tests and golden traces only need execution, and an AST would be a large enum with no consumer until `disassemble` in M4.
- Alternatives considered: decode into `Instruction` first (more isolated tests, but duplicates operand plumbing), table-driven decoder (harder to read around 8086 quirks).
- Consequences: `decoder.rs` exposes ModR/M and operands inside the crate; M4 may add an AST and reuse `ModRm` for disassembly.

### Step errors distinguish invalid opcodes, divide errors and unimplemented instructions (2026-11-07)

- Decision: `StepError::{InvalidOpcode(u8), DivideError, Unimplemented}`.
- Reason: tests and the DAP layer must tell a non-8086 opcode from an instruction that is merely not written yet, and from a division that will become `INT 0`.
- Alternatives considered: one opaque `Unimplemented` (hides progress, makes invalid-opcode tests impossible), `String` errors (violates the error-enum rule).
- Consequences: `Unimplemented` is temporary and must disappear when M1 closes; `DivideError` becomes `INT 0` in M2.

### INT and HLT execute in M1 (2026-11-07)

- Decision: M1 implements the real interrupt vector fetch and `HLT` termination, not just their decoding.
- Reason: control-flow tests and the debugger need `INT` and `HLT` to change machine state; BIOS/DOS handlers stay in M2.
- Alternatives considered: leave them `Unimplemented` until M2 (M1 cannot claim the full instruction set), terminate in the adapter (leaks CPU semantics out of `emu86-core`).
- Consequences: M2 adds the interrupt handler table and termination services on top of the vector mechanism.

### Rust as the implementation language (2026-11-07)

- Decision: `emu86-core`, `emu86-asm`, `emu86-dap` and `emu86-ui` are Rust; TypeScript only for the VS Code extension.
- Reason: one toolchain for three platforms, native binaries that editors can spawn, and a WASM build path for the Zed extension.
- Alternatives considered: C++ (poor portability, no WASM for Zed), Go (no Zed extension path), C# (heavy distribution off Windows), TypeScript core (slow stepping, extra runtime for the window).
- Consequences: `egui`/`eframe` will pull a large dependency tree at M2; `emu86-core` stays dependency-free. See `docs/adr/0004-rust-implementation.md`.

### Project name and repository naming (2026-11-07)

- Decision: the product is `emu86`, the repository is `gz0ni/emu86-ide`, the VS Code extension id is `gz0ni.emu86`.
- Reason: `emu86` is free on crates.io, npm and the VS Code Marketplace; the `-ide` suffix avoids the crowded GitHub name space (`Emu86Dev/Emu86`, `mfld-fr/emu86`) and states what the project is.
- Alternatives considered: `emu8086-*` (someone else's trademark), `asm86` (too generic), plain `emu86` on GitHub (collides with existing 8086 emulators, hurts discoverability).
- Consequences: crate names `emu86-core` / `emu86-asm` / `emu86-dap` / `emu86-ui` are reserved by name only, not yet published.

### Zed is the primary editor target (2026-11-07)

- Decision: the Zed extension is milestone M5, the VS Code extension is M6.
- Reason: the maintainer works in Zed day to day, so Zed is where the tool gets used and dogfooded; VS Code follows once the DAP surface is stable.
- Alternatives considered: VS Code first (larger audience, easier packaging because the VSIX can bundle binaries), both in parallel (splits attention before the core works).
- Consequences: the Zed install step for the native binaries must be solved early, since Zed cannot ship them inside the extension. See `docs/ROADMAP.md`.

### Distribution differs per editor (2026-11-07)

- Decision: the VS Code VSIX bundles `emu86-dap`, `emu86-ui` and JWasm per platform; the Zed extension ships no binaries and documents an install step.
- Reason: VS Code extensions may spawn processes and package platform-specific VSIX builds; Zed extensions are WASM and cannot spawn or download anything.
- Alternatives considered: a single global install for both editors (worse VS Code UX), a self-downloading installer started by the adapter (needs its own trust and signing story before M5).
- Consequences: two packaging pipelines; the README must stay honest about the extra step on Zed; `tools/package-vscode.ps1` is still unwritten.

### `AGENTS.md` lives in the repository root, not in `.agents/` (2026-11-07)

- Decision: the agent entry point is `./AGENTS.md`; every other context file lives in `.agents/`.
- Reason: `AGENTS.md` is auto-loaded from the repository root by OpenCode, Codex and Claude Code, and the links use repo-relative paths (`.agents/project.md`), which would break from inside `.agents/`.
- Alternatives considered: `AGENTS.md` inside `.agents/` with adjusted links (duplicated content, easy to drift), both files (two sources of truth).
- Consequences: this deviates from the `project-init` skill template on purpose; see `.agents/agent-config.md`.

### Telemetry is disabled in this repository (2026-11-07)

- Decision: all MCP configuration passes `CODEGRAPH_TELEMETRY=off`.
- Reason: the code-graph server reports usage to PostHog by default; the project does not send third-party analytics from a developer machine or CI.
- Alternatives considered: leaving the default on (no benefit for a single maintainer, adds a surprising outbound connection).
- Consequences: verified by watching `~/.codegraph/telemetry-queue.jsonl` stay unchanged after a run. Any new MCP server must be checked for the same.

### CodeGraph MCP for code navigation (2026-11-07)

- Decision: index the repository with `codegraph-ai/CodeGraph` and expose it over MCP to OpenCode and Claude Code.
- Reason: structural questions (callers, definitions, impact) cost several `grep` round-trips; a code graph answers them in one call and gets cheaper as the workspace grows.
- Alternatives considered: `CodeGraphContext` (needs Neo4j; its embedded backends do not install on Windows), Serena with rust-analyzer (stronger for Rust but a different, larger setup), no MCP at all (viable today, will hurt past ~10k lines).
- Consequences: one global npm install plus a downloaded native engine; `CODEGRAPH_TELEMETRY=off` in both configs; `.codegraph/` ignored by git; Codex deferred. See `AGENTS.md` for the `codegraph_*` before `grep` rule.

### Monorepo with two editor extensions (2026-11-07)

- Decision: one repository holds four crates and both editor extensions.
- Reason: the core, the adapter and both extensions must be versioned together; a protocol change is a single commit.
- Alternatives considered: separate repositories for Zed and VS Code (five-repo version matrix, cross-repo breaking changes), organization-level split (premature for a single maintainer).
- Consequences: VSIX is packaged from `extensions/vscode`; the Zed registry entry points at `extensions/zed` by path.

### Companion window instead of per-editor UI (2026-11-07)

- Decision: the 80x25 screen, hex dump and keyboard live in one native `emu86-ui` process.
- Reason: Zed extensions cannot render panels or spawn processes; VS Code extensions cannot create OS windows. A shared native window is the only way to get identical behaviour in both editors.
- Alternatives considered: VS Code webview only (no Zed parity), ANSI output into the debug console (no colours, no cursor, no keyboard).
- Consequences: two processes per session; the window also works standalone. See `docs/adr/0001-dap-and-companion-ui.md`.

### External MASM-compatible assembler, not a home-grown one (2026-11-07)

- Decision: invoke JWasm as a child process; own only preprocessing, listing parsing and loading.
- Reason: the MASM dialect used by lab manuals is weeks of work and a permanent divergence risk.
- Alternatives considered: writing an assembler (2–3 weeks, compatibility gamble), NASM/YASM (wrong dialect), Keystone (no MASM directives).
- Consequences: users install one extra binary; the VS Code extension can bundle it, the Zed extension cannot. JWasm is never vendored or linked. See `docs/adr/0002-jwasm-as-external-binary.md`.

### MIT license (2026-11-07)

- Decision: MIT for the whole repository.
- Reason: maximum adoption, compatible with GPLv3 projects, matches the license already declared in the repository.
- Alternatives considered: dual `MIT OR Apache-2.0` (adds a patent grant at the cost of a second file and ambiguity for students).
- Consequences: no explicit patent grant; relicensing to dual is possible later without breaking compatibility. See `docs/adr/0003-mit-license.md`.

### Register editing through plain DAP scopes (2026-11-07)

- Decision: expose registers, flags and segments as ordinary DAP variables.
- Reason: `setVariable` then works in both editors without custom UI, and the editors' own debug panels become the register window.
- Alternatives considered: custom DAP requests and a bespoke register editor (duplicated per editor).
- Consequences: variable names must be stable (`AX`, `CF`, `DS`); memory is handled separately through `readMemory`/`writeMemory`.
