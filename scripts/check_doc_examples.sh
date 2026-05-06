#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SNASK_BIN="${SNASK_BIN:-$ROOT/target/debug/snask}"
OUT_DIR="${TMPDIR:-/tmp}/snask-doc-examples"

mkdir -p "$OUT_DIR"

if [[ ! -x "$SNASK_BIN" ]]; then
  cargo build --manifest-path "$ROOT/Cargo.toml"
fi

check() {
  local profile="$1"
  local file="$2"
  local out="$OUT_DIR/$(basename "$file" .snask)"
  echo "doc example: $file [$profile]"
  "$SNASK_BIN" build "$ROOT/$file" --profile "$profile" --output "$out"
}

check humane docs/examples/reference/io_hello.snask
check systems docs/examples/reference/systems_bits.snask
check systems docs/examples/reference/systems_memory.snask
check humane docs/examples/reference/sfs_basic.snask
check humane docs/examples/reference/json_basic.snask
check humane docs/examples/reference/gui_minimal.snask
