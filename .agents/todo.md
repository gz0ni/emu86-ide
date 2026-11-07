# TODO

Task checklist. The agent maintains this file: creates items, marks them done, keeps it honest. Work from here — do not lose tasks between sessions.

## Format

- `[ ]` pending task — add due date or note if needed
- `[x]` done task — keep for history, do not delete immediately
- `[~]` in progress — who/what is working on it

## Rules

- Add a task the moment it is discovered, even if vague.
- Mark `[x]` only when actually verified done (checks passed), not by intent.
- Review this file at the end of every session: close finished items, add follow-ups.

---

## Open

- [ ] Collect lab manuals and sample `.asm` files; close the TBD list in `docs/SPEC.md`.
- [ ] Decide whether the manuals need macros and conditional assembly (affects assembler scope).
- [ ] Decide `.com` only or full `.model small` with MZ `.exe` loader.
- [ ] Verify emu8086 virtual devices actually used in the course.
- [ ] Create the tree-sitter grammar repository for Zed (M5, primary editor
      target — before the VS Code extension).
- [ ] Add CodeGraph MCP config for Codex CLI (`~/.codex/config.toml`) — deferred
      by the user.
- [ ] Add `serde`/`serde_json` when the DAP server starts (keep `emu86-core` dependency-free).
- [ ] Write `tools/capture-trace.ps1` for golden traces.

## In Progress

- [~] M1 — 8086 decoder, flags semantics, per-instruction tests.

## Done

- [x] CodeGraph MCP wired up for OpenCode and Claude Code; telemetry off; verified in OpenCode (2026-11-07).
- [x] M0 — workspace, four crates, docs, ADRs, CI, editor extension skeletons (2026-11-07).
- [x] Scaffold `.agents/` context set (2026-11-07).
