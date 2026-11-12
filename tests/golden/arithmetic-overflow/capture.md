# Arithmetic Overflow Reference Capture

## Source and Environment

- Reference: installed emu8086 4.08, Windows, captured on 2026-11-12.
- `program.asm` is original code written for this capture. It uses only 8086 instructions.
- Assemble with the reference IDE's F5 action. Its `program.bin_` output is retained byte-for-byte as `program.bin`: `B8 FF 7F 05 01 00 A3 00 02 F4`.
- Binary directives load at `0100:0100` (physical `0x1100`), with `CS=DS=ES=SS=0100h`, `IP=0100h`, `SP=FFFEh`. Other registers are observed as zero. Initial raw flags are `0202h`.
- The reference refuses a load at physical `0x0100`. The successful capture loads at physical `0x1100`; physical `0x1000` is zero-filled. The dump preserves vector data, default memory contents and the reference's padding after the image (`NOP` bytes through `0x111D`, then `HLT` at `0x111E`). No firmware routine or bundled example was added to the repository.

## Recording Procedure

1. Open `program.asm` in emu8086 and assemble with F5. Check the registers and disassembly against the entry point above.
2. Open the flags window. Save the initial state with **file > save the emulator's state**. Copy the resulting `emulator.state` to a temporary `step-0.state` before saving another state.
3. Click **single step** once, save and copy `step-1.state`. Repeat for `step-2.state` and `step-3.state`. Observe each register and flag change in the debugger.
4. Stop the accepted trace before executing `HLT`. The 3 trace rows describe states before `MOV AX`, `ADD AX` and `MOV [0200h],AX`. `final.json` is the running state after the memory write, at `IP=0109h`.
5. Extract memory and register data from these reference exports, calculate every checksum from its corresponding reference RAM, then import with `tools/capture-trace.ps1 -Name arithmetic-overflow -Program program.bin -Source program.asm -Trace trace.jsonl -Final final.json -InitialMemory initial-memory.bin`.

For this installed build, each state file was 1114393 bytes. Raw RAM starts at file offset zero: bytes visible at physical `0x1100` match the assembled image, and the memory write changes file offset `0x1201` from `00` to `80`. The low 65536 bytes of `step-0.state` become `initial-memory.bin`. Firmware code at high memory and all other state-file contents are excluded from the committed dump.

The register block at file offset `0x10FFF6` contains 14 little-endian words in this order: `AX BX CX DX DS ES SI DI BP CS IP SS SP FLAGS`. This layout was checked against the visible register and flags windows and consecutive exports: the first step changes only AX and IP, the second changes AX, IP and FLAGS, and the third changes IP and the destination memory byte. These offsets are an observed 4.08 export layout, not a portable state-file contract.

Equivalent extraction for one state, using Python's standard library:

```python
import struct
from pathlib import Path

data = Path("step-0.state").read_bytes()
assert len(data) == 1114393
names = "ax bx cx dx ds es si di bp cs ip ss sp flags".split()
registers = dict(zip(names, struct.unpack_from("<14H", data, 0x10FFF6)))
checksum = 2166136261
for byte in data[0x100:0x10000]:
    checksum = ((checksum ^ byte) * 16777619) & 0xFFFFFFFF
Path("initial-memory.bin").write_bytes(data[:0x10000])
```

Use each step's own exported bytes for its checksum; write the initial dump only from step 0. No emu86 execution output was used to generate reference states or hashes.

## Observed States

All unlisted registers retain their initial values; all accepted snapshots have `halted=false`.

| Export | IP | AX | FLAGS | Word at physical 0x1200 | FNV-1a |
| --- | --- | --- | --- | --- | --- |
| 0 | 0100 | 0000 | 0202 | 0000 | 84F529A5 |
| 1 | 0103 | 7FFF | 0202 | 0000 | 84F529A5 |
| 2 | 0106 | 8000 | 0A96 | 0000 | 84F529A5 |
| 3 | 0109 | 8000 | 0A96 | 8000 | EB7EDC25 |

The second instruction sets OF, SF, AF and PF; CF and ZF remain clear, and IF remains set. The checksum window remains physical `[0x0100, 0x10000)`.

An additional exploratory HLT step showed the message “the emulator is halted” while IP stayed at `0109h`; its saved state was identical to export 3. That export has no distinguishable halt marker. HLT is outside this bounded capture and was recorded for a separate compatibility investigation; no final state was fabricated for it.

The follow-up investigation and halt regression are documented in [HLT reference observations](../hlt-observations.md). The bounded data files above remain unchanged.

## Integrity

SHA-256 values of the exact files used during extraction:

| File | SHA-256 |
| --- | --- |
| program.bin | 2cf909cb7145233647db2e3361def9606857d85f450f9190c597aeb5be1cd49c |
| initial-memory.bin | 71a95aa5bfbfdf37a92f0b81167cdc96c68b023e32b1c13bfb1ca85f334bf2e0 |
| step-0.state | f67411f388143b25664d37e8e6018ea3f1d3b5ccf93efc6078705d670a5c8bb6 |
| step-1.state | 150e60e682bf7b4a4e22522b079016a14d0c404705d8875c80a93faa0001e3bc |
| step-2.state | ba5b91e6a7058cef5a19e505709b183d72d6ed53430b38b18c698e0762c24fe0 |
| step-3.state | 09271c62fe04abf1795671e89129ec3a567ab5b4217ae1146903d472de32e1b2 |

Complete proprietary state files are temporary local inputs and are not part of the repository. A fresh capture may have different unrelated RAM or path metadata; preserve and checksum its actual memory rather than forcing these hashes.
