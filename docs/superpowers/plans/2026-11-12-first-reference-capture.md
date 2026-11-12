# First Reference Capture Plan

**Goal:** Record and replay an original program in the installed emu8086 4.08.

**Architecture:** Keep capture initialization and file parsing in the Rust integration harness. Preserve actual reference memory and flags, and validate the same inputs in the PowerShell importer. Do not change core-library CPU semantics for this bounded capture.

**Tech Stack:** Rust 2021 standard library, PowerShell 7, Python standard library for temporary state extraction, installed emu8086 4.08.

**Spec:** `tests/golden/README.md` and `docs/ROADMAP.md` M1.

**Scope:** Reference data and capture provenance. Change the harness only if actual reference initialization requires it. Keep M1 open for deferred instructions.

**Observed requirements:** emu8086 refuses physical `0x0100`. Preserve a raw initial low-64-KiB memory dump and the exact flags word. Keep BIOS code, emulator binaries and complete proprietary state files out of the repository. Compare the same physical checksum window as before.

## Replay and Import Support

**Files:** `crates/emu86-core/tests/golden.rs`, `tools/capture-trace.ps1`, `tools/tests/capture-trace.tests.ps1`. Capture files and provenance live in `tests/golden/arithmetic-overflow/`; project status lives in `AGENTS.md` and `.agents/*.md`.

- Add failing tests for an optional `initial-memory.bin` containing exactly 65536 bytes, with the exact program image at the first snapshot's physical `CS:IP`. Reject truncated dumps, mismatched program bytes and images outside the checksum window. Preserve zero-filled `0x0100` loading for existing synthetic captures.
- Initialize replay memory from this dump when present and initialize flags from the raw recorded word. Keep full-word comparisons, including undefined bits, unchanged.
- Add `-InitialMemory` to the importer with matching checks before creating a destination. Copy the dump only after validation. Test invalid inputs and replay a synthetic imported dump.
- Verify the real saved-state RAM layout against the visible debugger and compare successive exports to distinguish memory from register metadata. Retain only low-memory reference state and original program data.

## Tasks

- [x] Assemble an original flat program at physical `0x1100` with explicit initial registers after observing the reference's refusal of `0x0100`. Inspect the reference debugger's memory export and flag display.
- [x] Record pre-instruction states and a separate final state. Calculate checksums from reference memory dumps. Preserve the exact image and capture procedure under `tests/golden/` through the existing importer.
- [x] Replay the capture, run fmt, clippy, workspace tests and importer tests, request independent review, and update project status.

## Verification Results

- Replay/importer regression tests failed before implementation and passed afterward.
- 131 core unit tests, 3 assembler unit tests and 9 golden integration tests pass; PowerShell importer integration checks pass.
- fmt, clippy and diff whitespace checks pass. Cargo reports the previously documented incremental-cache hard-link fallback.
- Independent review found one provenance error about residual memory at `0x1000`; the dump shows zeros, and the description was corrected. No replay/importer implementation findings.
- M1.5b is complete with a bounded 3-instruction real trace. HLT observability/IP behavior is tracked separately; M1 remains open.

**Validation:** `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `pwsh -NoProfile -File tools/tests/capture-trace.tests.ps1`.
