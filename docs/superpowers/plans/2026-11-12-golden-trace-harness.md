# Golden Trace Harness Implementation Plan

> Execute the steps in this session; use a separate reviewer after verification.

**Goal:** Finish M1.5 infrastructure without claiming reference compatibility before a real emu8086 capture exists.

**Architecture:** Keep replay, parsing and file access in the integration test, outside the CPU library. Import manually recorded states with the PowerShell helper. Both consumers enforce the documented snapshot schema.

**Tech Stack:** Rust 2021, standard library only, PowerShell.

**Spec:** `docs/ROADMAP.md` M1 and `tests/golden/README.md`.

## Constraints

- No dependencies in `emu86-core`; no files, DAP or UI in its library.
- Preserve existing uncommitted decoder work; do not commit or create a PR.
- Synthetic fixtures verify the harness and are never reference captures.
- M1 remains open until a real trace passes and deferred instructions are resolved.

## Task 1: Validate and replay snapshots

**Files:** `crates/emu86-core/tests/golden.rs`.

**Interfaces:** `replay(program: &[u8], trace: &str, final_state: &str) -> Result<(), String>`.

- [x] Add tests for JSON whitespace, duplicate keys, invalid structure, wrong types and out-of-range registers. Use the existing `snapshot_json` fixture; malformed input such as `valid.replace("\"ip\":256", "\"ip\":256,\"ip\":257")` must fail.
- [x] Run `cargo test -p emu86-core --test golden`; confirm the missing validation fails.
- [x] Replace substring lookup with a schema parser for objects, unsigned integers and booleans; accept JSON whitespace and reject duplicate/unknown fields, trailing tokens and malformed separators.
- [x] Test instruction sequencing with hand-derived `NOP`/`HLT` states, empty traces, premature halt, opcode errors, register/flag/memory corruption and the image-size limit.
- [x] Run the focused tests and keep file operations outside the library.

## Task 2: Validate imported captures

**Files:** `tools/capture-trace.ps1`, `tools/tests/capture-trace.tests.ps1`.

**Interfaces:** Existing `-Name`, `-Program`, `-Trace`, `-Final`, optional `-Source`, `-GoldenRoot` and `-SkipChecksumCheck` arguments.

- [x] Run the importer from a dependency-free PowerShell test using temporary input files. Valid captures must install; fractional/string/null numbers, multiple final states, unsafe names, oversized images and existing destinations must fail before copying.
- [x] Run `pwsh -NoProfile -File tools/tests/capture-trace.tests.ps1` and observe missing validation failures.
- [x] Validate exact schema and numeric types before casts, normalize JSON output, enforce exactly one final state, check image size unconditionally and reject overwriting an existing capture.
- [x] Run the script tests and replay the installed synthetic fixture with the Rust harness.

## Task 3: Document and verify

**Files:** `tests/golden/README.md`, `tools/README.md`, `.agents/todo.md`, `.agents/project.md`, `.agents/commands.md`, `.agents/architecture.md`, `.agents/gotchas.md`, `.agents/bugs.md`, `.agents/project-decisions.md`, `AGENTS.md`.

- [x] Document pre-instruction trace rows, final state, initial memory, fixed physical checksum window, binary input and manual reference provenance.
- [x] Split M1.5 into verified infrastructure and pending real capture; record discovered issues and decisions.
- [x] Run `cargo fmt --all`, `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, and the PowerShell helper tests.
- [x] Request a focused independent review, resolve actionable findings, and repeat affected checks.
