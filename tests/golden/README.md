# Golden traces

Reference captures from a real emu8086 install. No reference captures are committed yet. Synthetic tests verify the replay infrastructure; they do not establish CPU compatibility or close M1.

## Layout

One directory per program:

```text
tests/golden/<program>/
  program.asm       optional original source
  program.com       exact flat image, or program.bin
  trace.jsonl       one snapshot BEFORE each executed instruction
  final.json        exactly one snapshot AFTER the last instruction
  capture.md        reference version and manual capture procedure
```

`program.com` is preferred when both binary names exist. Keep only one image in a capture. The test loads it at physical `0x0100`; this is an M1 replay convention, not a DOS loader. Images larger than `65280` bytes are rejected.

## Snapshot schema

Each nonblank line is one JSON object. All values are unsigned decimal integers except `regs` (an object) and `halted` (a boolean):

```json
{"ip":256,"cs":0,"regs":{"ax":0,"bx":0,"cx":0,"dx":0,"sp":65534,"bp":0,"si":0,"di":0,"ds":0,"es":0,"ss":0},"flags":61442,"mem_checksum":123456,"halted":false}
```

The checksum above is illustrative. All fields are required except `halted`, which defaults to `false`. Register values, `ip`, `cs` and `flags` must fit `u16`; `mem_checksum` must fit `u32`. Keys are case-sensitive and use the literal ASCII spelling shown above. JSON whitespace and any field order are accepted. Unknown or duplicate fields, arrays, nulls, strings, negative/fractional/exponent numbers and trailing content are rejected. `final.json` uses the same schema, with one object on one line for the importer.

A trace row always describes a running machine (`halted: false`). For N rows, replay executes exactly N instructions and compares the resulting state with `final.json`. Do not append the final state to `trace.jsonl`. Set `halted: true` in `final.json` after `HLT` or a supported DOS termination shortcut. A bounded partial execution may end with `halted: false`.

The first row initializes registers and flags only. All physical memory starts at zero, then the flat binary is loaded. The first row's checksum must match that memory. Flags are compared as a full word, including the 8086 reserved bits fixed by `Flags::from_bits`; undefined flag differences are reported rather than masked silently.

## Memory checksum

`mem_checksum` is FNV-1a 32-bit over physical addresses `0x0100` through `0xFFFF` inclusive, in ascending order. Start with `2166136261`; for each byte, XOR it into the hash, multiply by `16777619`, then keep the low 32 bits. Include code, data, stack and unused bytes in this window. It is a fixed physical window, not a segment-relative range. Changes outside it, screen cells and full initial memory dumps are not supported yet.

Calculate every recorded checksum from the reference machine's actual memory. Do not substitute hashes from emu86 execution. If reference initialization differs inside the window, record the difference and extend the harness before importing; do not rewrite reference data to make tests pass. PSP/BIOS bytes below `0x0100` are outside the checksum, but instructions that read them still require future memory-initialization support.

## Manual capture and import

1. Run a small original 8086-only flat program in emu8086 on Windows or under Wine. Keep the exact assembled binary and original source. Do not copy emu8086 examples or assets.
2. Record every register, the flags word and a reference memory checksum before each instruction; record one final state after the last instruction. Preserve the reference execution order, including REP as one CPU instruction.
3. Save those states as JSONL and JSON using the schema above. Record emu8086 version, platform, assembly options, initial state, memory-dump/checksum method and stopping condition in `capture.md`. The helper imports manually prepared captures; it does not automate emu8086 or extract debugger state.
4. Import from PowerShell 7:

```powershell
pwsh -NoProfile -File tools/capture-trace.ps1 -Name arithmetic `
  -Program C:/captures/program.com -Source C:/captures/program.asm `
  -Trace C:/captures/trace.jsonl -Final C:/captures/final.json
```

5. Add `capture.md` to the new directory and run `cargo test -p emu86-core --test golden -- --nocapture`.

The importer validates the schema, image size and initial checksum before creating a new capture directory. It normalizes JSON to UTF-8 without a BOM and never overwrites an existing capture. `-SkipChecksumCheck` only bypasses the importer's initial-memory check for investigation; Rust replay still checks the checksum. `-GoldenRoot` selects another output directory.

## Running tests

```powershell
cargo test -p emu86-core --test golden -- --nocapture
pwsh -NoProfile -File tools/tests/capture-trace.tests.ps1
```

The PowerShell tests install a temporary synthetic capture and replay it with Rust. They require PowerShell 7 and Cargo; no Pester module or external assembler is needed. Temporary captures are removed afterwards.

To replay captures outside the repository, set `EMU86_GOLDEN_ROOT` to their parent directory before running the Rust test. An explicit root containing no traces fails. The default repository directory may be empty during M1 development and reports that no reference traces were found; that passing discovery test does not satisfy M1 acceptance.
