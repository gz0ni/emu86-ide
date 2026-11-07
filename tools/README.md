# tools

Development helpers. None of this ships to users.

Planned:

- `capture-trace.ps1` — record golden traces from a real emu8086 install
  (registers and flags after every instruction).
- `fetch-jwasm.ps1` / `fetch-jwasm.sh` — download the assembler binary used
  locally and in CI so it never lands in the repository.
- `package-vscode.ps1` — build the VSIX for each target platform with the
  native binaries bundled.
