# Gotchas

Traps and pitfalls the agent has stepped on in this project. Log a gotcha immediately whenever something does not build, run, or pass — right after the incident, while context is fresh, do not wait for session end. Check this file before repeating similar work. One entry per trap.

## Format

```
### <Short trap title> (YYYY-MM-DD)

- What happened / what was wrong
- Root cause
- How to avoid next time
```

---

<!-- New entries go here, newest first. -->

### HLT advances past the reference stopping address (2026-11-12)

- What happened: the real-capture regression failed with `final state: ip 0x010A != 0x0109` after HLT.
- Root cause: instruction fetch increments IP before the halt handler, while emu8086 4.08 displays and exports the stopping opcode's address.
- Avoid: verify the stopping address, prefixed instructions and repeated stepping against the reference before updating halt behavior.
- Additional regression tests failed on ordinary stopping IP and on `Cpu::step` executing a new MOV while `halted` was already set; the step entry point needs an early halt check as well as the HLT address fix.
- After the fix, two synthetic replay tests still expected IP after HLT and failed. Update those hand-derived halt expectations to the observed stopping opcode address. The PowerShell synthetic capture ends after NOP and needs no change.

### Real reference initialization exposed replay assumptions (2026-11-12)

- What happened: new replay tests failed because raw `0202` flags became `F202`, relocated dump memory was ignored and invalid dump sizes were accepted.
- Root cause: replay always normalized flags and assumed a zero-filled image at physical `0x0100`.
- Avoid: preserve recorded flags and validate optional low-memory initialization before replay; keep synthetic fallback behavior covered separately.
- Importer regression initially failed because `-InitialMemory` was not supported; validate and copy the optional dump in the importer as well as Rust replay.

### emu8086 rejects the harness load address (2026-11-12)

- What happened: emu8086 4.08 refused physical `0x0100` and reported its free-memory range as `0050:0000` through `A000:0000`.
- Root cause: the harness's fixed image address overlaps the reference emulator's reserved low memory.
- Avoid: preserve real reference initialization and extend replay initialization rather than relocating recorded states or fabricating checksums.

### emu8086 UI geometry requires a fresh screenshot (2026-11-12)

- What happened: a menu click after a text-only observation failed with `coordinate input geometry is unavailable`.
- Root cause: the legacy window did not expose usable geometry in that observation.
- Avoid: refresh window selection, activate the target and capture a screenshot before coordinate input.

### Python defaults produced non-UTF-8 punctuation (2026-11-12)

- What happened: review found replacement glyphs for two em dashes in `tools/README.md`.
- Root cause: shell Python wrote its Windows default encoding instead of UTF-8.
- Avoid: always pass `encoding='utf-8'` and `newline='\n'` when writing repository text from Python; verify decoded output after writing.

### Shell writes to agent context are protected (2026-11-12)

- What happened: Python raised `PermissionError` while updating `.agents/todo.md` from the shell.
- Root cause: this environment exposes agent context as read-only to shell processes.
- Avoid: use the structured patch tool for authorized repository context updates; it supports these edits without shell escalation.

### Git index writes require elevated sandbox access (2026-11-12)

- What happened: authorized `git add` failed creating `.git/index.lock` with `Permission denied`.
- Root cause: the sandbox exposes `.git` as read-only to ordinary shell commands.
- Avoid: use the approval-reviewed elevated execution path for user-authorized staging, commits and pushes; do not change repository permissions.

### Untracked files are absent from ordinary diff checks (2026-11-12)

- What happened: `git diff --cached --check` found an extra blank line at the end of the new PowerShell test after ordinary `git diff --check` had passed.
- Root cause: ordinary diffs do not include untracked files.
- Avoid: inspect the staged diff with `git diff --cached --check` before committing newly added files.

### Capture importer coerced invalid JSON numbers and used a signed FNV literal (2026-11-12)

- What happened: importer regression tests accepted strings, nulls, booleans and fractional registers; the valid import failed converting `0x811C9DC5` to `UInt32`.
- Root cause: casts were used as validation, and PowerShell interprets `0x811C9DC5` and `0xFFFFFFFF` as negative `Int32` values. The signed mask also caused overflow on the next checksum attempt. Duplicate fields, unsafe names and repeated final states were not checked either.
- Avoid: validate JSON token kinds and ranges before conversion, use decimal `2166136261` for the FNV offset and `4294967295` for its mask, and validate all inputs before creating the destination.

### Golden harness validation gaps exposed by regression tests (2026-11-12)

- What happened: new harness tests failed on whitespace, duplicate JSON keys and an extra halted trace row. Review exposed a second failing case: Unicode whitespace after an object was accepted. Cargo also warned that incremental-cache hard links fell back to copies.
- Root cause: substring field lookup did not parse JSON structure, replay accepted a post-instruction halt state as an extra pre-instruction row, and `str::trim` accepts more than JSON whitespace. The cache warning is an environment limitation, not a test failure.
- Avoid: parse the snapshot schema, reject halted trace rows and test malformed captures before accepting reference data. Judge cache warnings separately from test exit codes.

### INT stack layout and the default SP trip up test expectations (2026-11-07)

- What happened: two control-flow tests failed because they expected the interrupted `IP` at `SS:FFFE` and `SP=0x0FFA`.
- Root cause: `INT` pushes `FLAGS`, then `CS`, then `IP`, so with the default `SP=0` the saved `IP` lands at `SS:FFFA` and `SP` ends at `FFFA`; a test that started from `SP=0x1000` correctly ended at `0x0FFA`.
- Avoid: when asserting interrupt stack contents, remember the push order (`FLAGS` highest address, `IP` lowest) and compute `SP` from the starting value with wrapping, not by copying the `SP=0x1000` case.

### 8086 shift immediates and REP are not 186+ forms (2026-11-07)

- What happened: `C0`/`C1` look like natural shift-immediate opcodes, and `REP` used to be consumed and discarded by the prefix loop.
- Root cause: 8086 group 2 only has `D0`-`D3`; immediate-count shifts arrived with the 186. String repeat needs the prefix state, not just consumption.
- Avoid: keep `C0`/`C1` in `is_invalid_opcode`; pass `Repeat` through `ops::execute` to `ops::string`, and apply the ZF stop condition only to `CMPS`/`SCAS`.

### `Memory` breaks `#[derive(Debug)]` on `Cpu` (2026-11-07)

- `cargo build` failed with `Memory doesn't implement Debug` while `Cpu` derived `Debug`.
- Root cause: `Memory` intentionally had no `Debug`, and a derived parent requires it.
- Avoid: `Memory` has a hand-written `Debug` that prints only the size. Never derive `Debug` on `Memory` — a 1 MiB array in a log line is unreadable.

### `cargo fmt --check` fails after writing code by hand (2026-11-07)

- CI runs `cargo fmt --all -- --check` first; a one-line `assert!` in `memory.rs` was long enough to be reformatted.
- Root cause: hand-written formatting drifted from `rustfmt.toml`.
- Avoid: run `cargo fmt --all` before any verification run, not after it.

### npm 12 blocks install scripts, so the CodeGraph engine never downloads (2026-11-07)

- `npm i -g @astudioplus/codegraph-mcp` finished with exit 0, but `codegraph-mcp` failed with `Binary not found: ...codegraph-server-win32-x64.exe`.
- Root cause: npm 12 blocks `postinstall` scripts unless the package is allowed (`npm warn install-scripts`), and the engine is fetched by that script.
- Avoid: after installing any package that downloads a binary in `postinstall`, run `codegraph-mcp-fetch-engine` explicitly (or `npm install -g --allow-scripts=<pkg>`). Check for the engine before trusting an exit code.

### PowerShell shows `NativeCommandError` for normal cargo output (2026-11-07)

- Every `cargo` invocation printed a red `NativeCommandError` block even on success.
- Root cause: cargo writes progress to stderr and PowerShell treats stderr as an error record.
- Avoid: judge by `$LASTEXITCODE`, not by the colour. Pipe through `2>&1 | Select-Object` when the noise is distracting.
