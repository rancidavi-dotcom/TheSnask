#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT="$ROOT/bench/out"
mkdir -p "$OUT"

SNASK_BIN="${SNASK_BIN:-$ROOT/target/release/snask}"

ts() { date -Iseconds; }
have() { command -v "$1" >/dev/null 2>&1; }

bytes() {
  # linux stat
  stat -c%s "$1"
}

human() {
  python3 - "$1" <<'PY'
import sys
n=int(sys.argv[1])
for unit in ["B","KiB","MiB","GiB"]:
  if n<1024 or unit=="GiB":
    print(f"{n:.2f} {unit}" if unit!="B" else f"{n} B")
    break
  n/=1024
PY
}

size_breakdown() {
  local bin="$1"
  if have llvm-size-18; then
    llvm-size-18 "$bin"
  elif have size; then
    size "$bin"
  else
    echo "(no size tool available)"
  fi
}

build_c() {
  local src="$1" out="$2"
  gcc "$src" -o "$out"
}

build_snask() {
  local src="$1" out="$2" profile="$3"
  case "$profile" in
    tiny) "$SNASK_BIN" build --tiny "$src" --output "$out" 1>&2 ;;
    release-size) "$SNASK_BIN" build --release-size "$src" --output "$out" 1>&2 ;;
    ultra-tiny)
      # Future: implement `--ultra-tiny`. For now, fallback to `--tiny`.
      if "$SNASK_BIN" build --help | rg -q -- "--ultra-tiny"; then
        "$SNASK_BIN" build --ultra-tiny "$src" --output "$out" 1>&2
      else
        "$SNASK_BIN" build --tiny "$src" --output "$out" 1>&2
      fi
      ;;
    *) echo "unknown profile: $profile" >&2; exit 2 ;;
  esac
}

build_snask_min_runtime() {
  local src="$1" out="$2" profile="$3"
  case "$profile" in
    tiny) "$SNASK_BIN" build --tiny --min-runtime "$src" --output "$out" 1>&2 ;;
    release-size) "$SNASK_BIN" build --release-size --min-runtime "$src" --output "$out" 1>&2 ;;
    ultra-tiny)
      if "$SNASK_BIN" build --help | rg -q -- "--ultra-tiny"; then
        "$SNASK_BIN" build --ultra-tiny --min-runtime "$src" --output "$out" 1>&2
      else
        "$SNASK_BIN" build --tiny --min-runtime "$src" --output "$out" 1>&2
      fi
      ;;
    *) echo "unknown profile: $profile" >&2; exit 2 ;;
  esac
}

bench_one() {
  local name="$1" c_src="$2" sn_src="$3"

  local c_out="$OUT/${name}_c"
  local sn_tiny="$OUT/${name}_snask_tiny"
  local sn_rel="$OUT/${name}_snask_release_size"
  local sn_ultra="$OUT/${name}_snask_ultra_tiny"

  echo "[build] $name (C conventional gcc)" >&2
  build_c "$c_src" "$c_out"

  echo "[build] $name (Snask tiny)" >&2
  if [[ "$name" == "cli_full" ]]; then build_snask_min_runtime "$sn_src" "$sn_tiny" "tiny"; else build_snask "$sn_src" "$sn_tiny" "tiny"; fi

  echo "[build] $name (Snask release-size)" >&2
  if [[ "$name" == "cli_full" ]]; then build_snask_min_runtime "$sn_src" "$sn_rel" "release-size"; else build_snask "$sn_src" "$sn_rel" "release-size"; fi

  echo "[build] $name (Snask ultra-tiny)" >&2
  if [[ "$name" == "cli_full" ]]; then build_snask_min_runtime "$sn_src" "$sn_ultra" "ultra-tiny"; else build_snask "$sn_src" "$sn_ultra" "ultra-tiny"; fi

  echo "[run] $name" >&2
  "$c_out" >/dev/null
  "$sn_tiny" >/dev/null
  "$sn_rel" >/dev/null
  "$sn_ultra" >/dev/null

  printf "%s|%s|%s\n" "$name" "$c_out" "$(bytes "$c_out")"
  printf "%s|%s|%s\n" "$name" "$sn_tiny" "$(bytes "$sn_tiny")"
  printf "%s|%s|%s\n" "$name" "$sn_rel" "$(bytes "$sn_rel")"
  printf "%s|%s|%s\n" "$name" "$sn_ultra" "$(bytes "$sn_ultra")"
}

REPORT="$OUT/report.md"
{
  echo "# Snask Bench Report"
  echo
  echo "- date: $(ts)"
  echo "- snask: $("$SNASK_BIN" --version 2>/dev/null || echo unknown)"
  echo "- gcc: $(gcc --version | head -n1)"
  echo
  echo "## Results (bytes)"
  echo
  echo "| bench | variant | bytes | human |"
  echo "| --- | --- | ---: | --- |"
} >"$REPORT"

tmp="$OUT/_rows.txt"
: >"$tmp"

bench_one "cli_hello" "$ROOT/bench/cli_hello/hello.c" "$ROOT/bench/cli_hello/hello.snask" >>"$tmp"
bench_one "cli_io" "$ROOT/bench/cli_io/io.c" "$ROOT/bench/cli_io/io.snask" >>"$tmp"
bench_one "cli_full" "$ROOT/bench/cli_full/cli_full.c" "$ROOT/bench/cli_full/cli_full.snask" >>"$tmp"

while IFS='|' read -r bench path b; do
  [[ "$b" =~ ^[0-9]+$ ]] || continue
  var="$(basename "$path")"
  printf "| %s | %s | %s | %s |\n" "$bench" "$var" "$b" "$(human "$b")" >>"$REPORT"
done <"$tmp"

{
  echo
  echo "## Size breakdown"
  echo
  for bin in "$OUT"/*_c "$OUT"/*_snask_tiny "$OUT"/*_snask_release_size "$OUT"/*_snask_ultra_tiny; do
    echo "### $(basename "$bin")"
    echo
    echo '```'
    size_breakdown "$bin"
    echo '```'
    echo
  done
} >>"$REPORT"

echo "OK: $REPORT"
