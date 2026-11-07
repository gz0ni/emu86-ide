# emu86

An open-source Intel 8086 toolchain and debugger for modern editors: **Zed** and **VS Code**.

`emu86` takes the useful part of emu8086 — the assembler, the emulator and the debugger — and drops the editor. You write assembly in the editor you already know, press `F5`, and get real registers, real memory, real stepping and the emulated text screen.

> **Status: pre-alpha.** The workspace, crate skeleton and documentation land first. Nothing executes instructions yet.

## Why

- emu8086 is Windows-only, closed source and looks like 2003.
- University lab manuals are written against MASM/TASM syntax, so a plain NASM setup does not work.
- Zed and VS Code already provide editing, git, tabs, search and a debug UI. Rebuilding that is wasted effort.

## What it does

| Piece | What you get |
|---|---|
| Assembler | MASM-compatible dialect (`.model`, `assume`, `proc`, `dup`, macros) via an external JWasm process |
| Emulator | Intel 8086 CPU, 1 MiB of memory, DOS/BIOS interrupt subset, text-mode video |
| Debugger | Breakpoints, step/step-over, register and memory inspection, register editing, watch |
| Screen | 80x25 text screen with colours and keyboard input in a separate companion window |
| Editors | Zed (primary) and VS Code, both through the same Debug Adapter Protocol server |

## Repository layout

```
crates/emu86-core   CPU, memory, flags, devices           (no I/O, no UI)
crates/emu86-asm    preprocessing, JWasm driver, source map
crates/emu86-dap    Debug Adapter Protocol server
crates/emu86-ui     companion window: screen, hex dump, keyboard
extensions/vscode   VS Code extension (grammar, debug factory, webview)
extensions/zed      Zed extension (tree-sitter grammar, adapter schema)
docs/               spec, architecture, roadmap, ADRs
examples/           sample programs in the lab dialect
tests/golden/       reference traces captured from real emu8086
```

## Build

```sh
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
```

No external dependencies yet, so the workspace builds offline.

## License

MIT. See [LICENSE](LICENSE).
