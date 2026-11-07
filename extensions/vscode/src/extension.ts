import * as vscode from 'vscode';

/**
 * Entry point of the emu86 VS Code extension.
 *
 * The debugger itself lives in the `emu86-dap` binary; this extension only
 * contributes the language, wires the adapter and exposes commands.
 */
export function activate(context: vscode.ExtensionContext): void {
  context.subscriptions.push(
    vscode.commands.registerCommand('emu86.about', () => {
      void vscode.window.showInformationMessage(
        'emu86 0.0.0 — pre-alpha. Assembler, emulator and debugger are not implemented yet.',
      );
    }),
  );
}

export function deactivate(): void {
  // Nothing to clean up yet.
}
