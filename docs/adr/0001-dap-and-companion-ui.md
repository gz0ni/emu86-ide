# ADR 0001 — One DAP server plus a native companion window

Date: 2026-11-07
Status: accepted

## Context

We want the emulator, assembler and debugger to work in both Zed and VS Code
without maintaining two independent UIs.

Constraints found while evaluating the editors:

- VS Code extensions cannot create OS-level windows, only `WebviewPanel` tabs.
- Zed extensions are WASM modules: no process spawning, no custom panels, no
  network access. Zed does ship a DAP client and renders `variables`,
  `watch`, `call stack` and `console` for whatever the adapter reports.
- Zed has no built-in hex memory viewer.

## Decision

1. All debugger functionality is exposed over **DAP** from one Rust binary
   (`emu86-dap`). Registers, flags, segments and the stack are reported as
   scopes/variables, which makes `setVariable` register editing work in both
   editors with zero custom UI.
2. Memory is exposed through the DAP `readMemory`/`writeMemory` requests.
3. Everything DAP cannot render — the 80x25 text screen, keyboard input and a
   navigable hex dump — lives in **one native companion window**
   (`emu86-ui`), launched by the adapter and connected over a local socket.
   The same window runs standalone from a terminal.

## Consequences

+ One UI codebase, identical behaviour in both editors.
+ Zed gets a first-class experience without abusing the WASM sandbox.
+ The companion window is usable without any editor.
- Two processes per session instead of one.
- VS Code could have rendered the screen in a webview; we deliberately do not
  duplicate it (optional webview may be added later, sharing the same data).
