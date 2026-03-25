#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"
OUT_DIR="$ROOT_DIR/bench/io/out"
mkdir -p "$OUT_DIR"

SIZE_MB="${SIZE_MB:-256}"
RUNS="${RUNS:-7}"

SNASK_BIN="${SNASK_BIN:-$ROOT_DIR/target/release/snask}"
if [[ ! -x "$SNASK_BIN" ]]; then
  echo "snask binary not found at: $SNASK_BIN" >&2
  echo "build it first: cargo build --release" >&2
  exit 1
fi

echo "[io] building binaries..."
pushd "$ROOT_DIR/bench/io" >/dev/null

if command -v gcc >/dev/null 2>&1; then
  gcc -O2 c_io.c -o "$OUT_DIR/c_io"
else
  echo "[io] WARN: gcc not found; skipping C" >&2
fi

if command -v go >/dev/null 2>&1; then
  go build -o "$OUT_DIR/go_io" go_io.go
else
  echo "[io] WARN: go not found; skipping Go" >&2
fi

"$SNASK_BIN" build snask_io.snask --release-size >/dev/null
mv -f "$ROOT_DIR/bench/io/snask_io" "$OUT_DIR/snask_io"

popd >/dev/null

measure_one() {
  local name="$1"; shift
  local -a cmd=("$@")
  local file_path="$OUT_DIR/data_${name}.bin"

  # Ensure clean
  rm -f "$file_path"

  # /usr/bin/time -v writes to stderr
  local t_out="$OUT_DIR/${name}_time.txt"
  local prog_out="$OUT_DIR/${name}_out.txt"
  : >"$t_out"
  : >"$prog_out"

  if [[ "$name" == "snask" ]]; then
    BENCH_PATH="$file_path" BENCH_SIZE_MB="$SIZE_MB" /usr/bin/time -v "${cmd[@]}" >"$prog_out" 2>"$t_out" || true
  else
    /usr/bin/time -v "${cmd[@]}" "$file_path" "$SIZE_MB" >"$prog_out" 2>"$t_out" || true
  fi

  local bytes
  bytes=$(rg -o "[0-9]+" "$prog_out" | tail -n 1 || true)
  local elapsed
  elapsed=$(rg -n "^\\s*Elapsed \\(wall clock\\) time" "$t_out" | head -n1 | sed 's/.*: //' || true)
  local maxrss
  maxrss=$(rg -n "^\\s*Maximum resident set size" "$t_out" | head -n1 | sed 's/.*: //' || true)

  # Parse elapsed as seconds (best-effort).
  local sec
  sec=$(python3 - <<PY
import sys
s = "${elapsed}"
def to_sec(s):
  s=s.strip()
  if not s: return 0.0
  parts = s.split(':')
  if len(parts)==3:
    h,m,ss = parts
    return float(h)*3600 + float(m)*60 + float(ss)
  if len(parts)==2:
    m,ss = parts
    return float(m)*60 + float(ss)
  return float(s)
print("{:.6f}".format(to_sec(s)))
PY
)

  echo "$name,$bytes,$sec,$maxrss"
}

echo "[io] running benchmark (SIZE_MB=$SIZE_MB, RUNS=$RUNS)..."

csv="$OUT_DIR/results.csv"
echo "name,bytes,sec,maxrss_kb" >"$csv"

run_many() {
  local name="$1"; shift
  local -a cmd=("$@")
  for _ in $(seq 1 "$RUNS"); do
    measure_one "$name" "${cmd[@]}" >>"$csv"
  done
}

if [[ -x "$OUT_DIR/c_io" ]]; then run_many "c" "$OUT_DIR/c_io"; fi
if [[ -x "$OUT_DIR/go_io" ]]; then run_many "go" "$OUT_DIR/go_io"; fi
run_many "snask" "$OUT_DIR/snask_io"
run_many "python" python3 "$ROOT_DIR/bench/io/py_io.py"
run_many "node" node "$ROOT_DIR/bench/io/node_io.js"

OUT_DIR="$OUT_DIR" python3 - <<'PY'
import csv, os, pathlib

out_dir = pathlib.Path(os.environ["OUT_DIR"]).resolve()
rows = list(csv.DictReader((out_dir/"results.csv").open()))

def med(xs):
  xs=[x for x in xs if x is not None]
  xs.sort()
  return xs[len(xs)//2] if xs else None

by={}
for r in rows:
  name=r["name"]
  by.setdefault(name, []).append(r)

report=[]
report.append("# I/O throughput + peak RAM (write+read)\n\n")
report.append(f"Size: {rows[0]['bytes'] if rows else '?'} bytes target per run (SIZE_MB env controls size)\n\n")
report.append("| Lang | Bytes (median) | Wall sec (median) | Throughput MiB/s | Peak RSS MiB (median) | N |\n")
report.append("| --- | ---:| ---:| ---:| ---:| ---:|\n")

for name, rs in sorted(by.items()):
  bytes_vals=[int(r["bytes"]) for r in rs if r["bytes"].isdigit()]
  sec_vals=[float(r["sec"]) for r in rs if r["sec"]]
  rss_vals=[int(r["maxrss_kb"]) for r in rs if r["maxrss_kb"].isdigit()]
  b=med(sorted(bytes_vals))
  s=med(sorted(sec_vals))
  rss=med(sorted(rss_vals))
  n=len(rs)
  mib = b/(1024*1024) if b else 0.0
  thr = (mib/s) if s and s>0 else 0.0
  rss_mib = (rss/1024.0) if rss else 0.0
  report.append(f"| `{name}` | {b} | {s:.6f} | {thr:.2f} | {rss_mib:.1f} | {n} |\n")

(out_dir/"report.md").write_text("".join(report))
print("OK:", out_dir/"report.md")
PY

echo "[io] OK: $OUT_DIR/report.md"
