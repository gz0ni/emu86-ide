# AGENTS.md

Agent entry point for this repository. Keep this file small; detailed context lives in `.agents/*.md` and repo-scoped skills live in `.agents/skills/*/SKILL.md`.

Communication with the user: Russian. Code, commits, and documentation: English.

## Start Here

Read these files before making changes:

- [.agents/project.md](.agents/project.md): project overview, platforms, toolchain, dependencies, status.
- [.agents/commands.md](.agents/commands.md): build, test, lint and packaging commands.
- [.agents/rules.md](.agents/rules.md): coding, testing, and workflow conventions.
- [.agents/style.md](.agents/style.md): writing style for all text you produce.
- [.agents/todo.md](.agents/todo.md): current task checklist — read it, work from it, update it.

Read these only when the task touches their area:

- [.agents/architecture.md](.agents/architecture.md): crate map, data flow, known gaps.
- [.agents/gotchas.md](.agents/gotchas.md): traps already stepped on — check before repeating work.
- [.agents/bugs.md](.agents/bugs.md): known bugs and their fixes.
- [.agents/project-decisions.md](.agents/project-decisions.md): why things are done this way.
- [.agents/models.md](.agents/models.md): model catalog and routing.
- [.agents/skills.md](.agents/skills.md): index of repo-scoped skills.

## Current State

Read this before planning any work; it is the shortest path to "what is left".

- **Milestone:** M1 (8086 decoder) in progress — stages 1 (decoder skeleton and data movement), 2 (arithmetic, logic, flags, BCD), 3 (shifts, rotates, strings with REP) and 4 (control flow, interrupts, flag control) done.
- **Works:** workspace builds offline with zero dependencies; 131 unit tests pass; `Cpu::step` decodes prefixes, ModR/M, data movement, arithmetic/logic, shift/rotate, string and control-flow groups, including `INT`/`IRET` and flag-control instructions; fmt, clippy and test are green; CI matrix configured for Windows, macOS and Linux.
- **Golden traces:** replay infrastructure and the validated manual-capture importer work; 6 harness tests pass. No real emu8086 trace has been recorded yet, so M1 acceptance remains open.
- **Does not work yet:** `IN`/`OUT`, `WAIT` and `ESC` still return `Unimplemented`; no loaders, BIOS/DOS interrupt handlers, video or keyboard; `emu86_asm::assemble` is a stub; `emu86-dap` and `emu86-ui` only print a version; the Zed extension has no grammar and no debug adapter.
- **Blocked on input:** lab manuals — they decide `.com` versus MZ `.exe`, macros, and which virtual devices matter.
- **Live checklist:** [.agents/todo.md](.agents/todo.md). **Detailed status:** [.agents/project.md](.agents/project.md).
- **New machine:** [docs/SETUP.md](docs/SETUP.md).

## Highest Priority Rules

- Read `.agents/*.md` before starting work; follow them.
- Verify with the project's checks (see `.agents/commands.md`) before claiming work is complete.
- Keep changes minimal and follow existing patterns; this is a Rust workspace with four crates.
- Do not add comments unless requested.
- Do not commit, push, or create PRs without explicit user request.
- UI text: Russian; code identifiers, commits, docs: English.
- All text you write must follow `.agents/style.md`.

## Code Navigation

For structural questions — who calls X, where Y is defined, what Z imports, what breaks when W changes — query CodeGraph first (the `codegraph_*` MCP tools), not `grep`. Use `grep` and whole-file reads only for exact text, or when CodeGraph returned nothing useful.

## Project-Specific Hard Rules

- `emu86-core` must stay editor-agnostic: no DAP, no files, no UI.
- Never vendor or link the external assembler (JWasm). It is a child process only — see `.agents/project-decisions.md`.
- Do not copy code, help text or example programs from emu8086. Behaviour may be reimplemented, assets may not.
- The 8086 instruction set only. No 186/286/386 additions, no protected mode.
- Nothing merges with a failing `cargo fmt --all -- --check` or `cargo clippy --workspace --all-targets -- -D warnings`.

## Agent Maintenance Duties

You maintain these files yourself — no user prompting needed:

- **Whenever something does not build/run/pass**: log the gotcha immediately in `.agents/gotchas.md`.
- **End of every session**: append new bugs to `.agents/bugs.md`, record decisions in `.agents/project-decisions.md`.
- **Every task**: update `.agents/todo.md` — create items, mark done, keep it honest.
- **When project facts change**: update `project.md`, `commands.md`, `architecture.md`, `rules.md`.

## Repo Skills

Use repo skills from `.agents/skills/` when a task matches their descriptions. See [.agents/skills.md](.agents/skills.md).
