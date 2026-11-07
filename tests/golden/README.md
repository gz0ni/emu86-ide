# Golden traces

Behavioural tests captured from a real emu8086 install.

## Why

The 8086 has enough edge cases (flag results on `AF`, divide errors,
`rep movs` with `CX = 0`, wrapped addresses) that unit tests written from the
manual will not catch every divergence. We compare against the reference
implementation instead of guessing.

## Format

One directory per program:

```
tests/golden/<program>/
  program.asm
  trace.jsonl        one JSON object per executed instruction
  final.json         registers, flags, memory checksum and screen cells
```

`trace.jsonl` line:

```json
{"ip": 256, "cs": 0, "regs": {"ax": 0, "bx": 0, "cx": 0, "dx": 0,
 "sp": 65534, "bp": 0, "si": 0, "di": 0, "ds": 0, "es": 0, "ss": 0},
 "flags": 61714, "mem_checksum": 123456}
```

## Capturing

Captures are produced manually on Windows (native emu8086) or under Wine and
committed here. The recording helper lives in `tools/`.
