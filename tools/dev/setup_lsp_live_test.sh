#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

echo "[snask] building snask-lsp (release)..."
cargo build --release

LSP_BIN="$ROOT_DIR/target/release/snask-lsp"
if [[ ! -x "$LSP_BIN" ]]; then
  echo "[snask] error: expected LSP binary at: $LSP_BIN" >&2
  exit 1
fi

echo
echo "[snask] OK: $LSP_BIN"
echo
echo "Live test in VS Code (no custom extension needed):"
echo "1) Install the VS Code extension: 'LSP Inspector' (publisher: octref)."
echo "   - CLI: code --install-extension octref.lsp-inspector-webview"
echo "2) Open your Snask workspace folder in VS Code."
echo "3) Open Command Palette and run: 'LSP Inspector: Start Server'."
echo "4) Choose 'stdio' and set the command to:"
echo "   $LSP_BIN"
echo
echo "What you should see working:"
echo "- diagnostics (errors) updating as you type"
echo "- hover"
echo "- go to definition"
echo "- completion"
echo "- semantic tokens"
echo "- code actions (quick fixes)"
