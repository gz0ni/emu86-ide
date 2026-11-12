# TODO

Task checklist. The agent maintains this file: creates items, marks them done, keeps it honest. Work from here — do not lose tasks between sessions.

## Format

- `[ ]` pending task — add a due date or a note if needed
- `[x]` done task — keep for history, do not delete immediately
- `[~]` in progress — who or what is working on it

## Rules

- Add a task the moment it is discovered, even if vague.
- Mark `[x]` only when actually verified done (checks passed), not by intent.
- Review this file at the end of every session: close finished items, add follow-ups.

---

## Blocked On Input

Labs 5 and 6 have been reviewed; further manuals and examples are still needed.

- [ ] Collect the remaining lab manuals and sample `.asm` files; close the TBD list in `docs/SPEC.md`.
- [ ] Decide `.com` only or full `.model small` with an MZ `.exe` loader.
- [ ] Decide whether macros and conditional assembly are needed (changes assembler scope by weeks).
- [ ] Confirm which emu8086 virtual devices are actually used in the course.
- [ ] Implement `IN`/`OUT` (and decide on `WAIT`/`ESC`) once the virtual-device list is known.
- [ ] Decide whether the assembler stays external forever or is replaced later (see `docs/adr/0002-jwasm-as-external-binary.md`).

## Open — Implementation

- [ ] Assemble and run both lab 5 sample forms as COM programs; verify stepping, register and memory state and the reported `RHIGHT=0096h`, `RLOW=3000h` result.
- [ ] Check assembler diagnostics and source locations for the lab 5 malformed examples: missing separator, operand size mismatch, undefined symbol and invalid mnemonic.
- [ ] Add lab 6 acceptance cases for signed and unsigned byte and word arithmetic, overflow and division, including program-generated `00h`/`FFh` error codes in memory.
- [x] Resolve HLT compatibility: verified ordinary/prefixed stopping IP and repeated stepping in emu8086 4.08; HLT keeps IP on F4 and halted steps preserve state, with unit and real-capture regression coverage (2026-11-12).
- [ ] Create the tree-sitter grammar repository for Zed (M5 comes before M6).
- [x] Finish `tools/capture-trace.ps1` as a validated importer for manually recorded golden traces (2026-11-12).
- [ ] Write `tools/package-vscode.ps1` for per-platform VSIX packaging.
- [ ] Add `serde`/`serde_json` when the DAP server starts (keep `emu86-core` dependency-free).
- [ ] Add `egui`/`eframe` for `emu86-ui` at M2 (deliberately deferred).
- [ ] Add CodeGraph MCP config for Codex CLI (`~/.codex/config.toml`) — deferred by the user.

## In Progress

- [~] M1 — 8086 decoder, flag semantics, per-instruction tests.
  - [x] M1.1 — decoder skeleton, ModR/M, effective addresses, segment overrides and the data movement group (2026-11-07).
  - [x] M1.2 — arithmetic and logic: add/adc/sub/sbb/cmp/inc/dec/neg/not/test/and/or/xor, mul/imul/div/idiv, BCD (2026-11-07).
  - [x] M1.3 — shifts, rotates, string operations with REP/REPNE and DF (2026-11-07).
  - [x] M1.4 — control flow, INT/INT3/INTO/IRET, HLT and flag-control instructions (2026-11-07).
  - [x] M1.5a — golden-trace replay harness, validated capture importer and synthetic integration tests (2026-11-12).
  - [x] M1.5b — captured and replayed the first real emu8086 4.08 trace: 3 instructions with raw flags and low-memory initialization; the HLT follow-up is resolved, while M1 acceptance remains open for deferred instructions (2026-11-12).

## Done

- [x] CodeGraph MCP wired up for OpenCode and Claude Code; telemetry off; verified in OpenCode — 42 tools (2026-11-07).
- [x] Record all decisions: ADR 0004 plus six journal entries — naming, Zed-first, distribution, `AGENTS.md` placement, telemetry, CodeGraph (2026-11-07).
- [x] M0 — workspace, four crates, docs, ADRs, CI, editor extension skeletons (2026-11-07).
- [x] Scaffold `.agents/` context set (2026-11-07).
