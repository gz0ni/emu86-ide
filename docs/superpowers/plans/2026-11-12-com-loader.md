# COM Loader Implementation Plan

**Goal:** Load a flat COM image into a fresh CPU and execute it with the existing decoder.

**Architecture:** Add `Cpu::from_com(image: &[u8]) -> Result<Self, LoadError>` in `loader.rs`. Build a fresh machine at the specification's fixed PSP segment `0100h`, copy the image at offset `0100h`, and reserve the initial zero return word at `FFFEh`. Keep filesystem access outside the core.

**Tech Stack:** Rust 2021, standard library only, Rust 1.75 minimum.

**Spec:** `docs/SPEC.md` section 4 and `docs/ARCHITECTURE.md` debug-session data flow.

## Constraints

- Keep the core editor-agnostic and dependency-free; do not add comments, commits or external assets.
- Implement a minimal PSP: `INT 20h` at offset 0, exclusive allocation end `1100h` at offset 2, and an empty command tail at offsets `80h`/`81h`.
- Initialize `CS=DS=ES=SS=0100h`, `IP=0100h`, `SP=FFFEh`; use existing power-on defaults for other registers and flags. Reference-specific initial flags remain unverified.
- Accept 1 through `FEFEh` bytes. Reject larger images before copying so the initial stack word cannot overwrite code.
- Keep M1 open for deferred instructions and M2 open for MZ, interrupts and devices.

## Task 1: Load and execute COM images

**Files:** Create `crates/emu86-core/src/loader.rs`; export `LoadError` from `crates/emu86-core/src/lib.rs`; update architecture, status and agent checklist.

**Interface:** `LoadError::{EmptyImage, ImageTooLarge(usize)}` with `Display` and `std::error::Error`; `Cpu::from_com(image: &[u8]) -> Result<Self, LoadError>`.

- [x] Add the API skeleton and tests for fresh registers, exact image placement, minimal PSP fields, empty/oversized input and the largest valid image. The skeleton returns `Err(LoadError::EmptyImage)` so valid inputs fail behaviorally.

```rust
let cpu = Cpu::from_com(&[0xB8, 0x34, 0x12, 0xF4]).unwrap();
assert_eq!((cpu.regs.cs, cpu.regs.ip, cpu.regs.sp), (0x0100, 0x0100, 0xFFFE));
assert_eq!(&cpu.mem.as_slice()[0x1100..0x1104], &[0xB8, 0x34, 0x12, 0xF4]);
```

- [x] Run `cargo test -p emu86-core loader::tests` and confirm failures come from the missing successful load and size validation.
- [x] Validate `image.is_empty()` and `image.len() > 0xFEFE`, then initialize the fresh machine and copy bytes using `Memory::load_linear(0x1100, image)`. Write PSP fields and the zero return word through existing memory methods.
- [x] Run loader tests, including a literal byte program that stores arithmetic results through DS, uses PUSH/POP, and halts; verify `INT 20h`, `INT 21h AH=4Ch`, and `RET` through the PSP terminate within a bounded number of steps.
- [x] Run fmt, clippy and all workspace tests; request independent read-only review using `requesting-code-review`, resolve findings and update project status.

**Results:** 139 core unit tests (6 new loader tests), 3 assembler unit tests and 10 golden integration tests pass. fmt and clippy pass. Independent read-only review found no actionable issues. Cargo still reports the known incremental-cache hard-link fallback warning. No production bugs were discovered; the test accessor correction and expected red cycle are logged in `.agents/gotchas.md`.

## Reference

The Microsoft Press [Advanced MS-DOS Programming](https://www.pcjs.org/documents/books/mspl13/msdos/advdos/) COM startup description and format comparison reserve 256 PSP bytes and 2 stack bytes. The [MS-DOS Encyclopedia PSP layout](https://www.pcjs.org/documents/books/mspl13/msdos/encyclopedia/appendix-h/) defines the initialized fields. The fixed segment and deterministic remaining state are project conventions, not a captured emu8086 initialization.
