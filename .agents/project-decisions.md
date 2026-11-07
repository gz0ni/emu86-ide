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
