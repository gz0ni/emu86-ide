# Agent Configuration Model

<!-- Placement rules for agent surfaces in this repo. Keep stable team conventions here, link from AGENTS.md. -->

## Surfaces

- `AGENTS.md`: auto-loaded repository entry point. Keep it small; reserve for always-on rules, routing, and high-priority expectations.
- `.agents/*.md`: human- and agent-readable reference docs linked from `AGENTS.md`.
- `.agents/skills/*/SKILL.md`: repo-scoped skills, loaded only when a task matches.
- Global user-level skills live outside the repo and are loaded automatically; repo copies should not duplicate them.

## Placement Rules

- Put stable conventions in `AGENTS.md` only when every task must see them.
- Put detailed explanations in `.agents/*.md` and link them from `AGENTS.md`.
- Put reusable task workflows in `.agents/skills/<skill-name>/SKILL.md`.
- Put mechanical enforcement in linters, tests, or CI — do not rely on prose when tooling can enforce the rule.
- Keep user-specific preferences out of the repository.

## Project Enforcement Map

| Rule | Enforced by |
|---|---|
| formatting | `cargo fmt --all -- --check` in CI |
| lint cleanliness | `cargo clippy --workspace --all-targets -- -D warnings` |
| no unsafe code | `#![forbid(unsafe_code)]` in crate roots |
| 8086-only scope | review; `StepError::Unimplemented` for unknown opcodes |
| no vendored assembler | `.vscodeignore`/`tools` + review; JWasm resolved from `PATH` |
| editor-agnostic core | crate dependency rules in `docs/ARCHITECTURE.md` |

## Project Deviation From The `project-init` Template

- The entry point `AGENTS.md` sits in the repository root, not in `.agents/`. Rationale: it is auto-loaded from the root by OpenCode, Codex and Claude Code, and all links are repo-relative (`.agents/project.md`). See `.agents/project-decisions.md`.
- Everything else follows the template: journals and static references live in `.agents/`.

## Skill Authoring Rules

- Lowercase hyphenated names.
- Descriptions start with `Use when...` and describe trigger conditions.
- Keep `SKILL.md` lean; link to `.agents/*.md` for reference material.
