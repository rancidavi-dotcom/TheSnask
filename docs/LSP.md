# Snask LSP (Language Server Protocol)

Snask ships an LSP server to provide editor features like:
- diagnostics (syntax + basic semantic)
- hover (basic)
- completion (basic snippets/keywords)

## Build

```bash
cargo build --release
```

Binary:
- `target/release/snask-lsp`

## Live Testing (VS Code, real-time)

You can test `snask-lsp` live in VS Code without a dedicated Snask extension by using Microsoft's **LSP Inspector**.

1) Build:
- `tools/dev/setup_lsp_live_test.sh`

2) In VS Code:
- Install extension: `octref.lsp-inspector-webview`
- Command Palette → `LSP Inspector: Start Server`
- Transport: `stdio`
- Command: `target/release/snask-lsp`

## VS Code (manual wiring)

If you have an existing Snask VS Code extension, configure it to launch:

```bash
target/release/snask-lsp
```

over stdio.

## Notes

- Diagnostics are best-effort today:
  - parser errors include line/column
  - semantic errors currently do not have spans, so they show at the top of the file
