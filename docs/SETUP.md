# Setup on a new machine

Everything needed to go from `git clone` to a green build. Written for Windows;
the same commands work on macOS and Linux with the platform's package manager.

## 1. Required

| Tool | Version | Why |
|---|---|---|
| Rust (rustup) | stable, 1.75+ | all four crates; `rust-toolchain.toml` pins the channel |
| Git | any recent | repository |
| Node.js | 20+ (26 tested) | VS Code extension and CodeGraph |
| npm | 10+ (12 tested) | global packages |

JWasm is **not** required yet: nothing calls the assembler until M3. When it is,
it stays an external binary and is never vendored.

## 2. Rust

```bash
rustup toolchain install stable --component rustfmt --component clippy
cargo build --workspace
cargo test --workspace
```

## 3. VS Code extension (optional until M6)

```bash
cd extensions/vscode
npm install
npm run compile
```

## 4. CodeGraph MCP (agent tooling)

```bash
npm install -g @astudioplus/codegraph-mcp
```

npm 12 blocks `postinstall` scripts by default, so the native engine is **not**
downloaded by the install itself. Fetch it explicitly:

```bash
codegraph-mcp-fetch-engine
```

Then check that the server answers:

```bash
opencode mcp list        # expect: codegraph  connected
```

Project configuration lives in `opencode.json` (OpenCode) and `.mcp.json`
(Claude Code); both set `CODEGRAPH_TELEMETRY=off`. Codex CLI configuration is
not written yet — see `.agents/todo.md`.

Index data lives in `~/.codegraph/`, not in the repository.

## 5. Verification

Run before claiming anything works:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Windows note: `cargo` writes progress to stderr and PowerShell renders it as
`NativeCommandError`. Judge by `$LASTEXITCODE`, not by the colour.

## 6. What to read first

1. `AGENTS.md` — entry point, hard rules, current state.
2. `.agents/todo.md` — what is done, what is open, what is blocked.
3. `docs/SPEC.md` — functional scope.
4. `docs/ROADMAP.md` — milestone order and acceptance criteria.
5. `.agents/project-decisions.md` and `docs/adr/` — why things are the way they are.
