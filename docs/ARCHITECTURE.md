# Architecture

## Components

```
                    ┌──────────────────────────────┐
   Zed              │  extensions/zed (WASM)       │
   ──────────────►  │  tree-sitter grammar         │
                    │  debug adapter declaration   │
                    └──────────────┬───────────────┘
                                   │ spawns (Zed)
                    ┌──────────────▼───────────────┐
   VS Code          │  emu86-dap                   │
   ──────────────►  │  stdio DAP server            │
   extension spawns │  ┌────────────────────────┐  │
                    │  │ emu86-core             │  │
                    │  │  CPU, memory, devices  │  │
                    │  └────────────────────────┘  │
                    │  ┌────────────────────────┐  │
                    │  │ emu86-asm              │  │
                    │  │  preprocess + JWasm    │  │
                    │  └────────────────────────┘  │
                    └──────────────┬───────────────┘
                                   │ local socket
                    ┌──────────────▼───────────────┐
                    │  emu86-ui                    │
                    │  80x25 screen, hex dump,     │
                    │  keyboard                    │
                    └──────────────────────────────┘
```

## Why one DAP server and not two extensions

The Debug Adapter Protocol is implemented natively by both editors. One
adapter means one implementation of registers, memory, stepping and
breakpoints, and both editors render it with their own debug UI.

## Why a separate window for the screen

Zed extensions are WASM modules in a sandbox: no process spawning, no custom
panels, no network. VS Code extensions cannot open OS windows. A small native
companion process is therefore the only way to get the same screen experience
in both editors, and it doubles as a standalone runner:

```
emu86-ui prog.com        # run without an editor
emu86-ui --attach 127.0.0.1:7000
```

## Data flow of a debug session

1. Editor reads `launch.json` / Zed debug config: program path, working dir.
2. Editor spawns `emu86-dap`, sends `initialize`.
3. `launch` request: `emu86-asm` assembles the source, returns image + source
   map; `emu86-core` loads the image and builds the PSP.
4. Editor sets breakpoints (`setBreakpoints` with source lines); adapter maps
   them to addresses via the source map.
5. `emu86-ui` is spawned and connects to the adapter's local socket.
6. Stepping: adapter executes instructions until the source line changes
   (for `next`/`stepIn`), then emits `stopped`.
7. `scopes`/`variables` render registers as plain variables, so `setVariable`
   edits work without custom UI.

## Crate boundaries

| Crate | May depend on | Must not know about |
|---|---|---|
| `emu86-core` | nothing | DAP, files, UI, clipboard, screen window |
| `emu86-asm` | `emu86-core` | DAP, UI |
| `emu86-dap` | both | egui, editor-specific APIs |
| `emu86-ui` | `emu86-core` | DAP framing, editors |

`emu86-core` exposes state through plain data structures so the adapter can
snapshot, diff and restore machine state without touching the instruction
decoder.

## Versioning

All crates in the workspace share one version. Editor extensions carry the
same version and pin a minimum core version in their manifest.
