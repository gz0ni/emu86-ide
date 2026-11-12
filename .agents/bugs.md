# Bugs

Known bugs found in this project and how they were fixed, so they do not reappear. Logged at the end of every session. Status: `open` / `fixed` / `won't fix`.

## Format

```
### <Short bug title> (YYYY-MM-DD) — status

- Symptom
- Root cause
- Fix applied
- How to prevent recurrence
```

---

<!-- New entries go here, newest first. -->

### HLT reference IP and halt-state observability (2026-11-12) — open

- Symptom: an exploratory HLT step in emu8086 4.08 reports a halted emulator but keeps IP at `0109h`; the complete saved state is identical to the preceding running state. The core advances IP after fetching HLT.
- Root cause: reference halt handling and its debugger/export representation need separate investigation.
- Fix: none yet. The first accepted golden capture is explicitly bounded before HLT, with 3 executed instructions and a running final state.
- Prevent: capture halt behavior through both the debugger and a state export before choosing compatibility semantics; do not invent a halted export or mask IP differences.

### Replay discarded reference initialization (2026-11-12) — fixed

- Symptom: reference low memory and raw `0202h` flags could not replay under the fixed zero-memory and normalized-flags assumptions.
- Root cause: the initial synthetic harness assumed physical `0x0100` loading and used `Flags::from_bits` for captured state.
- Fix: optional validated low-64-KiB memory initialization and exact raw flags in the integration harness, plus matching importer support. Core power-on defaults are unchanged.
- Prevent: relocated-memory, raw-flags, malformed-dump and importer-to-replay regression tests.

### Golden capture validation and checksum errors (2026-11-12) — fixed

- Symptom: replay rejected whitespace before JSON colons but accepted duplicate fields and an extra halted trace row; the importer accepted invalid numeric types and failed calculating the initial checksum.
- Root cause: substring parsing ignored structure, the trace boundary was ambiguous, numeric casts coerced invalid values, and signed PowerShell hexadecimal literals broke FNV arithmetic.
- Fix: schema-aware parsing with JSON whitespace, strict field/type/range checks in both consumers, pre-instruction rows only, exactly one final state, unsigned FNV constants, image-size validation and no destination overwrites.
- Prevent: malformed-capture regression cases, memory/flag divergence checks and a PowerShell-import-to-Rust-replay integration test using independently calculated checksum values. Real reference compatibility still needs a capture.

### RCL/RCR ignored the incoming carry (2026-11-07) — fixed

- Symptom: `RCL AL,1` with `AL=0x80`, `CF=1` produced `AL=0x00` instead of `AL=0x01`.
- Root cause: the shift loop seeded its carry variable with `false` instead of the current `CF`, so the first rotation used a zero carry.
- Fix: seed the loop carry from `CF`; `ROL`/`ROR`/`SHL`/`SHR`/`SAR` overwrite it before reading, so only `RCL`/`RCR` are affected.
- Prevent: add a regression test that sets `CF` before every rotate-through-carry form, and keep the loop carry initialized from architectural state.

### DAA/DAS evaluated the high-digit condition after adjustment (2026-11-07) — fixed

- Symptom: `DAS` with `AL=0xA0`, `AF=1`, `CF=0` produced `AL=0x9A`, `CF=0` instead of `AL=0x3A`, `CF=1`; `DAS` with `AL=0x03`, `AF=1`, `CF=0` produced `AL=0x9D` instead of `AL=0xFD`.
- Root cause: the high-digit condition used the adjusted `AL` and the updated `CF` instead of the original `AL` and incoming `CF` required by the Intel pseudocode (`old_AL > 99H || old_CF`), and `DAA` did not clear `CF` when the condition was false.
- Fix: `DAA`/`DAS` now save `old_AL`, `old_CF` and `old_AF`, evaluate the high-digit condition against the saved values, `DAA` clears `CF` when it does not adjust, and `DAS` keeps the borrow from the low-digit adjustment.
- Prevent: transcribe adjustment pseudocode literally, including its `ELSE` branches, and add regression tests for incoming `CF`/`AF`, not only the common BCD cases.

### Word accesses used `offset + 1` across segment boundaries (2026-11-07) — fixed

- Symptom: `Memory::read16`/`write16` at `segment:FFFF` fetched the high byte from `segment:0000` instead of the next linear byte.
- Root cause: the code incremented the 16-bit offset and translated it again, mixing offset space with linear address space.
- Fix: translate once, then read/write `(physical + 1) & (MEMORY_SIZE - 1)`; tests cover the 64 KiB and 1 MiB wrap points.
- Prevent: treat offset and linear address as different spaces; test both wrap boundaries for every multi-byte access.

### `Cpu` could not derive `Debug` (2026-11-07) — fixed

- Symptom: `error[E0277]: Memory doesn't implement Debug` in `crates/emu86-core/src/lib.rs`.
- Root cause: `Memory` had no `Debug` implementation while `Cpu` derived it.
- Fix: hand-written `Debug` for `Memory` that reports only the buffer size.
- Prevent: any type embedded in `Cpu` must implement `Debug`, `Clone` and `Default`; add them at definition time, not after the first compile error.
