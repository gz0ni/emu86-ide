# tools

Development helpers. None of this ships to users.

## Golden capture importer

`capture-trace.ps1` validates and installs manually recorded emu8086 snapshots. It requires PowerShell 7 (including `System.Text.Json` and UTF-8 without BOM support). It does not control emu8086 or record its UI automatically.

See `tests/golden/README.md` for the snapshot schema, memory checksum, capture procedure and import command. Existing capture directories are never overwritten.

Run importer and Rust replay integration checks with:

```powershell
pwsh -NoProfile -File tools/tests/capture-trace.tests.ps1
```

The tests use temporary synthetic inputs, check rejected captures and replay an imported image. They require Cargo but no external packages.

## Planned

- `fetch-jwasm.ps1` / `fetch-jwasm.sh` — download the assembler binary used locally and in CI so it never lands in the repository.
- `package-vscode.ps1` — build the VSIX for each target platform with the native binaries bundled.
