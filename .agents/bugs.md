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

### `Cpu` could not derive `Debug` (2026-11-07) — fixed

- Symptom: `error[E0277]: Memory doesn't implement Debug` in `crates/emu86-core/src/lib.rs`.
- Root cause: `Memory` had no `Debug` implementation while `Cpu` derived it.
- Fix: hand-written `Debug` for `Memory` that reports only the buffer size.
- Prevent: any type embedded in `Cpu` must implement `Debug`, `Clone` and `Default`; add them at definition time, not after the first compile error.
