# HLT Compatibility Plan

**Goal:** Match emu8086 4.08 stopping IP and prevent execution after a halted CPU is stepped again.

**Architecture:** Preserve the stopping opcode address in the HLT handler. Make `Cpu::step` return without mutation while halted. Keep the reference's raw register/flags/memory observations in integration tests and provenance, outside the core library.

**Tech Stack:** Rust 2021 standard library, existing golden replay and PowerShell importer, installed emu8086 4.08.

**Spec:** `.agents/rules.md` reference compatibility and `.agents/todo.md` HLT follow-up.

## Diagnosis

- [x] Reproduce the ordinary HLT discrepancy using the existing real capture: `cargo test -p emu86-core --test golden hlt_preserves_the_reference_instruction_pointer -- --nocapture` fails with `ip 0x010A != 0x0109`.
- [x] Run an original `NOP; ES: HLT; MOV AX,1234h` probe in the reference. Its debugger steps the prefix separately, then stops with IP on the F4 byte at `0102h`; the exported register block agrees.
- [x] Attempt another reference step, decline program reload, and export again. IP remains `0102h`, AX remains zero and the complete exports are identical. Full state files remain ignored local inputs.

## Implementation

**Files:** `crates/emu86-core/src/cpu.rs`, `crates/emu86-core/src/ops/control.rs`, `crates/emu86-core/tests/golden.rs`, reference capture provenance and `.agents/*.md` status.

- [x] Add failing unit tests for stopping IP, prefix handling, IP wraparound and repeated stepping without register/flag/memory mutation.
- [x] Restore IP to the HLT opcode with wrapping subtraction; return early from `Cpu::step` when halted. Do not change DOS termination IP behavior or instruction-prefix grouping.
- [x] Update affected synthetic halt expectations, record the reference probe and verify the real-capture regression.
- [x] Run fmt, clippy, workspace tests and importer tests; request independent review and update the checklist.

**Results:** 133 core unit tests, 3 assembler unit tests and 10 golden integration tests pass. fmt, clippy and PowerShell importer checks pass. Independent review found no actionable issues. The existing Cargo incremental-cache hard-link fallback remains an environment warning.

## Constraints

- No new dependencies, comments, firmware assets or proprietary state files.
- A halt marker comes from the explicit reference debugger message; the reference export itself has no such field.
- No changes to the prior bounded arithmetic capture's data. Keep M1 open for deferred instructions.
