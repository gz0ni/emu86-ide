# Agent Skills

Repo-scoped skills live under `.agents/skills/*/SKILL.md` and are discoverable by skill `name` and `description`; full instructions load only when a task matches.

## Available Repo Skills

None yet.

Candidates worth adding when the workflow stabilises:

- `8086-instruction` — add a decoder entry, flag semantics and tests for one instruction group without diverging from emu8086.
- `capture-trace` — record a golden trace from a real emu8086 install.

## Authoring Notes

- Add new repeatable workflows as `.agents/skills/<skill-name>/SKILL.md`.
- Use lowercase hyphenated skill names.
- Start skill descriptions with `Use when...`.
- Keep `SKILL.md` lean; link to `.agents/*.md` for large reference material.
