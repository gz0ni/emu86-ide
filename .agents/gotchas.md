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
