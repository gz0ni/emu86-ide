# ADR 0002 — MASM-compatible assembler as an external process

Date: 2026-11-07
Status: accepted

## Context

Lab manuals are written for MASM/TASM syntax: `.model small`, `assume @data`,
`proc`/`endp`, `dup`, macros, conditional assembly. Writing a compatible
assembler is the single most expensive part of the project (weeks) and the
most likely to diverge from emu8086 behaviour.

NASM/YASM do not accept that dialect. Keystone supports 16-bit encoding but
not MASM directives.

## Decision

Use **JWasm**, an open-source MASM-compatible assembler, invoked as a child
process. This crate owns only:

- the preprocessing of emu8086-specific `#`-directives,
- process invocation and error parsing,
- listing parsing into a source map,
- the `.com` and MZ `.exe` loaders.

JWasm is **not** vendored into the repository and **not** linked. It is
resolved from `PATH` or from a user-configured path.

## Consequences

+ Weeks of assembler work avoided.
+ Real compatibility with the manuals from day one.
+ License isolation: a separate process, no linking, no combined work.
- Users must install one extra binary; the VS Code extension can bundle it,
  the Zed extension cannot and needs a documented install step.
- We depend on JWasm's dialect quirks; divergences are handled by the
  preprocessor and by bug reports upstream.
- A native assembler can replace JWasm later behind the same
  `emu86_asm::assemble` API.
