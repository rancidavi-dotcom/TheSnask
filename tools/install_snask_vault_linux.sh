#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APP_DIR="$ROOT_DIR/apps/snask_vault"

if ! command -v snask >/dev/null 2>&1; then
  echo "snask not found in PATH. Install Snask first."
  exit 1
fi

mkdir -p "$HOME/.local/bin" \
         "$HOME/.local/share/applications" \
         "$HOME/.local/share/icons/hicolor/scalable/apps"

echo "[1/3] Building Snask Vault..."
snask build "$APP_DIR/main.snask" >/dev/null || true

if [[ ! -f "$APP_DIR/main" ]]; then
  echo "Build failed: expected binary not found at $APP_DIR/main"
  exit 1
fi

echo "[2/3] Installing binary + desktop entry..."
install -m 0755 "$APP_DIR/main" "$HOME/.local/bin/snask-vault"
install -m 0644 "$APP_DIR/snask-vault.desktop" "$HOME/.local/share/applications/snask-vault.desktop"
install -m 0644 "$APP_DIR/assets/snask-vault.svg" "$HOME/.local/share/icons/hicolor/scalable/apps/snask-vault.svg"

if command -v update-desktop-database >/dev/null 2>&1; then
  update-desktop-database "$HOME/.local/share/applications" >/dev/null 2>&1 || true
fi

echo "[3/3] Done."
echo "Run: snask-vault"
echo "Or open your app launcher and search: Snask Vault"
