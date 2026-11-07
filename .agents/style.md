# Style

Writing rules for everything the agent produces: answers to the user, commit messages, code comments, documentation. Follow these exactly — no exceptions. If a rule is missing, propose adding it here instead of improvising.

## Language

- Communication with the user: Russian.
- Code identifiers, file names, commit messages, and repo docs: English.
- UI text in the app: Russian.

## Punctuation (Russian text)

- Long dash is `—` (em dash) with spaces around it: `текст — текст`.
- Hyphen only inside words: `какой-то`.
- Quotes: Russian guillemets `«...»`; nested quotes use `„...“`.
- Ellipsis: `…`, not three dots.
- Digits with units: `5 МБ`, `30 Гц` (space between number and unit).

## Tone

- Short answers: no filler, no fluff, no polite preambles.
- Answer directly; if asked a question, answer it first, explain after.
- Do not summarize what was done unless asked.

## Code

- No comments unless explicitly requested.
- Keep changes minimal; match existing patterns.
- Never log or print secrets, keys, or tokens.

## Commits

- Short imperative subject line (`Add decoder for mov`, not `Added` or `adding...`).
- Lowercase, no trailing period, English only.

---

## Project additions

- Technical terms in Russian text keep their original spelling: `эмулятор`, `ассемблер`, but `DAP`, `JWasm`, `tree-sitter`, `WASM` stay in Latin script.
- File and directory references are written as inline code paths: `crates/emu86-core/src/memory.rs`.
- Numbers of tests, crates and milestones are written as digits: `14 тестов`, `4 крейта`, `M1`.
