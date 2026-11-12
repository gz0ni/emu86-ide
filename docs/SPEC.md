# emu86 — functional specification

Status: draft. Sections marked **TBD** need input from the actual lab manuals
and from emu8086 behaviour captured on a real install.

Lab manuals 5 and 6 and the lab 5 sample were reviewed. They are only the
first available assembly assignments; later manuals may add requirements.

## 1. Goal

Replace emu8086 as the tool used to write, assemble, run and debug 8086
assembly for university coursework, while keeping the source dialect and the
observable behaviour of emu8086.

Out of scope: emulating anything past the 8086 (no 186/286/386 additions), no
protected mode, no Windows/DOS binaries beyond the documented interrupt subset.

## 2. Source dialect

Must compile the programs from the lab manuals unchanged.

Required:

- [ ] `.model small|tiny`, `.stack <n>`, `.data`, `.code`
- [ ] `segment` / `ends`, `assume`
- [ ] `org 100h`
- [ ] `proc` / `endp`, `end <entry>`
- [ ] Data directives: `db`, `dw`, `dd`, `dup`, `?`, string literals
- [ ] `equ`, `=` constants
- [ ] `offset`, `byte ptr`, `word ptr`, `ptr`
- [ ] `@data`, `$` (current location counter)
- [ ] Labels: named, local (`@@`, `@F`, `@B`)
- [ ] Jump size selection: short / near automatically
- [ ] `include`

**TBD:** whether the manuals use macros (`macro`/`endm`), conditional
assembly (`if`/`else`/`endif`), `struct`, `record`.

Lab 5 uses a COM-style program with `segment`/`ends`, `assume`, `org 100h`,
`equ`, `db`, `dw`, `?`, forward-referenced variables and `end <entry>`.
Its emulator-oriented variant omits the segment declarations. Both forms
need compatibility checks against the assembler pipeline.

## 3. Assembler front-end

- External MASM-compatible assembler (JWasm) runs as a child process.
- emu8086-specific `#`-directives are expanded by a preprocessor before
  JWasm sees the file.
  **TBD:** exact list used by the manuals (`#make_COM#`, `#make_EXE#`,
  `#startup`, `#include`, ...).
- Output: flat image for `.com`, MZ image for `.exe`.
- Output: a source map (`offset -> file:line:column`) parsed from the
  assembler listing.

## 4. Machine

- CPU: full 8086 instruction set, including string operations with `rep`,
  segment overrides, `int`/`iret`, divide errors.
- Flags: all nine documented flags plus the reserved bits.
- Memory: 1 MiB, `segment * 16 + offset`, wrapped at 1 MiB.
- Loader:
  - `.com`: PSP at `0100:0000`, code at `CS = DS = ES = SS = PSP`, `IP = 0100h`.
  - `.exe`: MZ header, relocation table, separate `CS`/`DS`, `SS:SP` from the
    header.
- Termination: `INT 20h`, `INT 21h AH=4Ch`, `HLT`.

## 5. Interrupts and devices

- [ ] `INT 21h`: `01h`, `02h`, `09h`, `0Ah`, `4Ch` (file I/O **TBD**)
- [ ] `INT 10h`: `02h`, `06h`, `0Eh` (more **TBD**)
- [ ] `INT 16h`: `00h`, `01h`
- [ ] `INT 1Ah`: tick counter
- [ ] Text video: 80x25, attribute byte per cell, hardware cursor, scrolling
- [ ] Keyboard: blocking and non-blocking reads
- [ ] Port I/O hooks for emu8086 virtual devices (**TBD:** which devices are
      actually used in the course)

Labs 5 and 6 do not identify virtual devices; later labs may still use them.

## 6. Debugger

Exposed over DAP so both editors get it for free:

- breakpoints (address based, mapped from source lines),
- `continue`, `next`, `stepIn`, `stepOut`, `pause`,
- `scopes`/`variables`: CPU, Flags, Segments, Stack, user symbols,
- `setVariable`: edit registers, flags and memory-backed variables,
- `readMemory`/`writeMemory`,
- `disassemble`,
- custom request for the companion window: screen matrix, cursor, keyboard.

## 7. Non-goals

- Reimplementing the emu8086 GUI.
- Copying emu8086 code, help text or example programs.
- Cycle-accurate timing.

## 8. Compatibility testing

Golden traces captured from real emu8086 under Windows/Wine:

1. run a program under emu8086 with single-stepping instrumentation,
2. dump registers, flags and a memory checksum after every instruction,
3. replay the same program in `emu86-core` and compare.

The trace corpus lives in `tests/golden/`.

Initial acceptance cases from labs 5 and 6:

- Assemble and run the lab 5 COM-style sample, including its simplified
  variant, and inspect registers, flags and memory after each instruction.
  For the supplied values the reported final words are `RHIGHT=0096h` and
  `RLOW=3000h`; this is a report-derived expectation, not a captured trace.
- Verify diagnostics with source locations for a missing separator, operand
  size mismatch, undefined symbol and invalid mnemonic.
- Exercise signed and unsigned byte and word arithmetic, including overflow,
  multiplication and division, with results stored in memory. Lab 6 programs
  produce a byte error code (`00h` for success, `FFh` for an error); that code
  is program logic, not an automatic CPU or assembler feature.
