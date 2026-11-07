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
  translation and 1 MiB wraparound, little-endian words, source-map lookup.
- `tests/golden/` will hold traces captured from a real emu8086 install;
  format documented in `tests/golden/README.md`, not populated yet.
