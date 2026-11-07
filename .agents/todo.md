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

These cannot be started until the lab manuals are available.

- [ ] Collect lab manuals and sample `.asm` files; close the TBD list in `docs/SPEC.md`.
- [ ] Decide `.com` only or full `.model small` with an MZ `.exe` loader.
- [ ] Decide whether macros and conditional assembly are needed (changes assembler scope by weeks).
- [ ] Confirm which emu8086 virtual devices are actually used in the course.
- [ ] Decide whether the assembler stays external forever or is replaced later (see `docs/adr/0002-jwasm-as-external-binary.md`).

## Open — Implementation

- [ ] M1 — 8086 decoder, flag semantics, per-instruction tests.
- [ ] Create the tree-sitter grammar repository for Zed (M5 comes before M6).
- [ ] Write `tools/capture-trace.ps1` for golden traces.
- [ ] Write `tools/package-vscode.ps1` for per-platform VSIX packaging.
- [ ] Add `serde`/`serde_json` when the DAP server starts (keep `emu86-core` dependency-free).
- [ ] Add `egui`/`eframe` for `emu86-ui` at M2 (deliberately deferred).
- [ ] Add CodeGraph MCP config for Codex CLI (`~/.codex/config.toml`) — deferred by the user.

## In Progress

Nothing yet.

## Done

- [x] CodeGraph MCP wired up for OpenCode and Claude Code; telemetry off; verified in OpenCode — 42 tools (2026-11-07).
- [x] Record all decisions: ADR 0004 plus six journal entries — naming, Zed-first, distribution, `AGENTS.md` placement, telemetry, CodeGraph (2026-11-07).
- [x] M0 — workspace, four crates, docs, ADRs, CI, editor extension skeletons (2026-11-07).
- [x] Scaffold `.agents/` context set (2026-11-07).
