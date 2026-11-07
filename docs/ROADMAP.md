# Roadmap

Each milestone has an acceptance criterion that can be checked mechanically.

## M0 — bootstrap (done)

Workspace, crates, docs, CI green on Windows, macOS and Linux.

## M1 — CPU core

- decoder for the full 8086 instruction set,
- flags with 8086 semantics,
- `Memory` and address translation already in place.

**Done when:** `emu86-core` passes the per-instruction unit tests and the
first golden trace from `tests/golden/`.

## M2 — execution environment

- `.com` and MZ `.exe` loaders, PSP, initial segment registers,
- text video memory and cursor,
- `INT 21h` / `INT 10h` / `INT 16h` subset,
- termination via `INT 20h`, `INT 21h AH=4Ch`, `HLT`.

**Done when:** `examples/hello.asm` prints through the emulated screen and
`examples/` programs from the lab manuals run to completion.

## M3 — assembler pipeline

- emu8086 preprocessor for `#`-directives,
- JWasm driver with `-bin` / `-Fl`,
- listing parser producing a source map,
- diagnostics with file, line and message.

**Done when:** every sample program assembles and the source map resolves each
address to a line.

## M4 — DAP server

- `initialize`, `launch`, `configurationDone`, `disconnect`,
- `setBreakpoints`, `continue`, `pause`, `next`, `stepIn`, `stepOut`,
- `scopes`, `variables`, `setVariable`, `evaluate`,
- `readMemory`, `writeMemory`, `disassemble`,
- `stopped`, `terminated`, `output` events.

**Done when:** a raw DAP client can step through `examples/hello.asm` and edit
`AX` from a `setVariable` request.

## M5 — Zed extension

Primary editor target.

- tree-sitter grammar,
- debug adapter declaration,
- documented binary installation path (the extension itself cannot ship a
  native binary).

**Done when:** `F5` assembles, runs, stops on a breakpoint and shows registers,
with the companion window for the screen.

## M6 — VS Code extension

- grammar, language configuration, snippets,
- debug adapter factory and launch configuration,
- problem matcher for assembler diagnostics,
- optional webview screen.

**Done when:** the same `F5` flow works in VS Code and both editors share one
DAP server.

## M7 — parity

- macros and conditional assembly if the manuals need them,
- watch/symbols view,
- animation mode,
- emu8086 virtual devices on ports,
- snippet pack and in-editor help.

**Done when:** the whole lab corpus runs unchanged.

## Testing strategy

Unit tests live next to the code. Behavioural tests replay golden traces
captured from emu8086. No test may depend on wall-clock timing.
