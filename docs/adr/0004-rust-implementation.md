# ADR 0004 — Rust for the core, the adapter and the companion window

Date: 2026-11-07
Status: accepted

## Context

The project ships in four shapes:

- a CPU core that must be fast enough for stepping and animation,
- a DAP server launched as a child process by both editors,
- a companion window with a text screen and a hex dump,
- a Zed extension, which is compiled to WASM and sandboxed.

The team is one developer on Windows, and the workspace must build on
Windows, macOS and Linux from a clean checkout.

## Decision

Use Rust for `emu86-core`, `emu86-asm`, `emu86-dap` and `emu86-ui`, edition
2021, `#![forbid(unsafe_code)]`. The Zed extension is also Rust
(`zed_extension_api`), compiled to WASM by Zed itself. TypeScript is used only
for the VS Code extension, because the VS Code API is JavaScript-only.

## Alternatives considered

- **C++** — portability is work, dependency management is work, and the same
  toolchain cannot produce the Zed WASM extension.
- **Go** — simple cross-compilation, but no practical path to a Zed extension
  and a GC in a stepping loop.
- **C#/.NET** — excellent tooling on Windows, heavier distribution on macOS
  and Linux, no WASM target for Zed.
- **TypeScript/Node for the core** — the whole toolchain could live in npm,
  but stepping through 1 MiB of memory and a full decoder in JS is slower and
  the companion window would need an extra runtime.

## Consequences

+ One toolchain, one `cargo build --workspace`, three target platforms.
+ `emu86-core` compiles to WASM, so a future browser or Zed-hosted runner is
  possible without a rewrite.
+ Memory safety and a small binary; `forbid(unsafe_code)` removes a class of
  emulator bugs.
- Slower iteration than TypeScript for UI work; the companion window still
  needs `egui`/`eframe`, which is a large dependency tree and is therefore not
  added until M2.
- Contributors must know Rust; the VS Code extension remains the only
  TypeScript surface.
