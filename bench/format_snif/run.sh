#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"
OUT_DIR="$ROOT_DIR/bench/format_snif/out"
TARGET_MB="${TARGET_MB:-100}"
RUNS="${RUNS:-7}"

mkdir -p "$OUT_DIR"

echo "[format] building tools..."
cargo build --release

echo "[format] generating dataset (${TARGET_MB}MB target)..."
"$ROOT_DIR/target/release/snif-dataset-gen" --target-mb "$TARGET_MB" --out-dir "$OUT_DIR"

echo "[format] running benchmark (runs=$RUNS)..."
"$ROOT_DIR/target/release/snif-format-bench" --dir "$OUT_DIR" --runs "$RUNS"

echo "[format] OK: $OUT_DIR/report.md"

