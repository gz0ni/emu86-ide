# Rules

<!-- Coding, testing, and workflow conventions specific to this project. Static rules live here; the agent updates them when conventions change. -->

## Code Style

- `cargo fmt` with the repository `rustfmt.toml` (`max_width = 100`, edition 2021). Never hand-format against it.
- `cargo clippy --workspace --all-targets -- -D warnings` must stay clean. Warnings are errors.
- `#![forbid(unsafe_code)]` in every crate that can have it.
- Public items carry doc comments; internal comments only where the reasoning is not obvious from the code.
- Naming: `Reg16::Ax`, `Memory::read16`, `Cpu::step` — Rust API guidelines style, no abbreviations beyond the architectural ones (`Reg`, `Mem`, `Cpu`).
- Errors are enums with `Display` + `std::error::Error`, not `String`.

## Languages

- User-facing UI text: Russian.
- Identifiers, file names, commits, docs: English.

## Design

- One DAP server for both editors; no editor-specific logic in `emu86-core` or `emu86-dap`.
- The companion window is the only place that renders pixels; editors never render the screen.
- Do not invent parallel abstractions: extend `Cpu`, `Memory`, `SourceMap` instead of wrapping them.
- Instruction semantics follow the Intel 8086 datasheet, and where emu8086 differs, emu8086 wins — that is the compatibility target.

## State and Persistence

- No persistence yet. Machine state lives in `Cpu` and is created per debug session.
- `Cpu` derives `Clone`, so the adapter can snapshot state for stepping back later.

## Testing

- `cargo test --workspace`. Unit tests live in `#[cfg(test)] mod tests` next to the code.
- Every new instruction needs at least one test: normal case, flag results, and one edge case (carry out, sign, overflow or wraparound).
- Behavioural compatibility is tested against golden traces from `tests/golden/`; no test may depend on wall-clock time.

## Generated and External Files

- `Cargo.lock` is committed.
- `target/`, `node_modules/`, `out/`, `dist/`, `*.vsix` are ignored.
- Never vendor JWasm or any part of emu8086 into this repository.
- `extensions/zed` is excluded from the Cargo workspace; do not add it to `members`.

## Code Navigation

- Structural questions (callers, definitions, imports, impact of a change) go to the CodeGraph MCP tools first: `codegraph_*`.
- `grep`, `glob` and whole-file reads are for exact text and for verifying what CodeGraph reported.
- CodeGraph is configured in `opencode.json` (OpenCode) and `.mcp.json` (Claude Code). Both pass `CODEGRAPH_TELEMETRY=off`.
- The index is local state: `.codegraph/` is ignored by git.

## Git

- Do not commit, push, create PRs, or run destructive git operations without explicit user request.
- Before any commit (if requested): check `git status`, `git diff`, `git log --oneline -10`; stage only intended files.
- Commit subject: short imperative English, lowercase, no trailing period.
