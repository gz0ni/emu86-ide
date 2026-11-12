# HLT Reference Observations

Reference: installed emu8086 4.08 on Windows, 2026-11-12. These observations resolve the HLT follow-up from `arithmetic-overflow/capture.md` without changing that bounded capture.

## Ordinary HLT

The original arithmetic program places HLT at `0100:0109`. After executing it, the debugger reports `the emulator is halted`, while both its register display and exported register block retain `IP=0109h`. The complete export is identical to the preceding running state's export, whose SHA-256 is `09271c62fe04abf1795671e89129ec3a567ab5b4217ae1146903d472de32e1b2`.

The integration regression reuses the exact image, initial memory and 3 recorded running states from that capture. It appends the recorded pre-HLT state and compares against the same register, flags and memory values after HLT, with `halted=true` taken from the explicit debugger message. That boolean is a manual observation, not a field inferred from the binary export. Before the fix, replay failed with `final state: ip 0x010A != 0x0109`.

## Prefixed HLT and Repeated Stepping

The second original probe used the same binary directives and initial registers as the arithmetic program, with this body:

```asm
org 100h
nop
db 26h, 0F4h
mov ax, 1234h
```

The exact assembled bytes were `90 26 F4 B8 34 12`, loaded at physical `0x1100`. `26` is the ES segment prefix and `F4` is HLT. The reference debugger presents and steps that prefix separately. Its observed IP sequence was `0100h` before NOP, `0101h` before the prefix, `0102h` before HLT and `0102h` after HLT. AX remained zero, and raw FLAGS remained `0202h`. The exported register block confirmed `CS=0100h`, `IP=0102h`, `AX=0000h`, `FLAGS=0202h`.

An attempted additional step prompted `program terminated. reload`. After declining reload, the reference reported halt again and retained the same registers and memory; the following `MOV` did not execute. State exports before and after that additional step were byte-identical, with SHA-256 `6f7244d2b9ff600038140a0176cbf754ac5fe39ee768afacd4c994b3605695fa`.

The temporary complete state files are excluded from the repository. Extraction used the same observed layout documented in the arithmetic capture: RAM starts at offset zero, and 14 little-endian register words start at `0x10FFF6`.

## Compatibility Choice

HLT leaves the core's IP on its `F4` opcode, after any consumed prefixes. `Cpu::step` does nothing while `halted` is set; resuming requires the caller to clear that state explicitly. DOS termination shortcuts retain their existing IP behavior. Prefixes remain part of one core instruction; this change does not reproduce the reference debugger's separate prefix UI steps.

Unit tests cover ordinary and prefixed HLT, 16-bit IP wraparound, preserved flags and memory, and repeated stepping after the stopping byte has been replaced with a MOV. Replacing that byte ensures a missing halt guard cannot pass merely by executing HLT repeatedly.
