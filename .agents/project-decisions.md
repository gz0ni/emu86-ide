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
