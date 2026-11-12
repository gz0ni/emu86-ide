# Commands

All commands verified on Windows, PowerShell 7, Rust 1.97.1.

## Development

```bash
cargo build --workspace
```

No dependencies, so no install step. Fresh machine: see `docs/SETUP.md`.

`extensions/zed` is excluded from the workspace on purpose (it needs the WASM
target and `zed_extension_api`); do not add it to `members`.

## Verification

Run before claiming work is done:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Windows note: `cargo` writes progress to stderr and PowerShell renders it as
`NativeCommandError`. That is not a failure — check `$LASTEXITCODE`.

## Build / Package

```bash
cargo build --workspace --release
```

VS Code extension:

```bash
cd extensions/vscode
npm install
npm run compile     # tsc -p .
npx @vscode/vsce package
```

Zed extension: built by Zed itself from `extensions/zed`; publishing goes
through the Zed extension registry and needs a public repository.

## CI

`.github/workflows/ci.yml`, triggered on push to `main` and on pull requests.

- `rust` job, matrix `ubuntu-latest` / `windows-latest` / `macos-latest`:
  fmt check, clippy with `-D warnings`, tests, release build.
- `vscode-extension` job, `ubuntu-latest`: `npm ci || npm install`, typecheck.

## Tests

- Unit tests live next to the code in `#[cfg(test)] mod tests`.
- Coverage today: register halves (AH/AL), flag bit handling, segment
  translation and 1 MiB wraparound, little-endian words and word accesses
  across segment boundaries, source-map lookup, ModR/M and effective
  addresses, the data movement instruction group, every binary ALU form,
  multiply/divide edge cases, BCD adjustment and decimal conversion,
  shift/rotate flags and counts, string movement and repeat conditions,
  every conditional jump, loop variants, calls and returns, interrupts and
  flag-control instructions, HLT stopping addresses and halted-step state preservation.
  COM loading tests cover the minimal PSP, initial registers, image boundaries,
  arithmetic/memory/stack execution and termination through HLT, DOS shortcuts and RET.

Run only COM loader tests: `cargo test -p emu86-core loader::tests`.

## Golden trace infrastructure

```powershell
cargo test -p emu86-core --test golden -- --nocapture
pwsh -NoProfile -File tools/tests/capture-trace.tests.ps1
```

The PowerShell checks require PowerShell 7 and Cargo, with no Pester dependency. They import temporary synthetic captures with and without memory dumps and replay them through Rust. The default Rust replay also checks the real `arithmetic-overflow` capture from emu8086 4.08. See `tests/golden/README.md` for manual recording and import instructions; `-InitialMemory` passes an exact 65536-byte low-memory dump to the importer. `EMU86_GOLDEN_ROOT` overrides the trace directory for Rust replay; an explicit empty root fails.
- `tests/golden/arithmetic-overflow/` holds the first real capture, with 3 instructions and a running final state before HLT; format and provenance are documented alongside it.
