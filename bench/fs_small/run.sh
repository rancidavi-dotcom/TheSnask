#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"
OUT_DIR="$ROOT_DIR/bench/fs_small/out"
mkdir -p "$OUT_DIR"

N_FILES="${N_FILES:-100000}"
RUNS="${RUNS:-5}"

SNASK_BIN="${SNASK_BIN:-$ROOT_DIR/target/release/snask}"
if [[ ! -x "$SNASK_BIN" ]]; then
  echo "snask binary not found at: $SNASK_BIN" >&2
  echo "build it first: cargo build --release" >&2
  exit 1
fi

echo "[fs_small] building binaries..."
pushd "$ROOT_DIR/bench/fs_small" >/dev/null

if command -v gcc >/dev/null 2>&1; then
  gcc -O2 c_fs_small.c -o "$OUT_DIR/c_fs_small"
else
  echo "[fs_small] WARN: gcc not found; skipping C" >&2
fi

if command -v go >/dev/null 2>&1; then
  go build -o "$OUT_DIR/go_fs_small" go_fs_small.go
else
  echo "[fs_small] WARN: go not found; skipping Go" >&2
fi

"$SNASK_BIN" build snask_fs_small.snask --release-size --min-runtime >/dev/null
mv -f "$ROOT_DIR/bench/fs_small/snask_fs_small" "$OUT_DIR/snask_fs_small"

popd >/dev/null

measure_one() {
  local name="$1"; shift
  local -a cmd=("$@")
  local workdir="$OUT_DIR/work_${name}"
  rm -rf "$workdir"
  mkdir -p "$workdir"

  local t_out="$OUT_DIR/${name}_time.txt"
  local prog_out="$OUT_DIR/${name}_out.txt"
  : >"$t_out"
  : >"$prog_out"

  if [[ "$name" == "snask" ]]; then
    BENCH_DIR="$workdir" BENCH_N="$N_FILES" /usr/bin/time -v "${cmd[@]}" >"$prog_out" 2>"$t_out"
  else
    /usr/bin/time -v "${cmd[@]}" "$workdir" "$N_FILES" >"$prog_out" 2>"$t_out"
  fi

  local count
  count=$(rg -o "[0-9]+" "$prog_out" | tail -n 1 || true)
  if [[ -z "$count" ]]; then
    count="0"
  fi
  local elapsed
  elapsed=$(rg -n "^\\s*Elapsed \\(wall clock\\) time" "$t_out" | head -n1 | sed 's/.*: //' || true)
  local maxrss
  maxrss=$(rg -n "^\\s*Maximum resident set size" "$t_out" | head -n1 | sed 's/.*: //' || true)

  local sec
  sec=$(python3 - <<PY
s = "${elapsed}".strip()
def to_sec(s):
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

  echo "$name,$count,$sec,$maxrss"
}

echo "[fs_small] running benchmark (N_FILES=$N_FILES, RUNS=$RUNS)..."
csv="$OUT_DIR/results.csv"
echo "name,count,sec,maxrss_kb" >"$csv"

run_many() {
  local name="$1"; shift
  local -a cmd=("$@")
  for _ in $(seq 1 "$RUNS"); do
    measure_one "$name" "${cmd[@]}" >>"$csv"
  done
}

if [[ -x "$OUT_DIR/c_fs_small" ]]; then run_many "c" "$OUT_DIR/c_fs_small"; fi
if [[ -x "$OUT_DIR/go_fs_small" ]]; then run_many "go" "$OUT_DIR/go_fs_small"; fi
run_many "snask" "$OUT_DIR/snask_fs_small"
run_many "python" python3 "$ROOT_DIR/bench/fs_small/py_fs_small.py"
run_many "node" node "$ROOT_DIR/bench/fs_small/node_fs_small.js"

OUT_DIR="$OUT_DIR" N_FILES="$N_FILES" python3 - <<'PY'
import csv, os, pathlib, statistics

out_dir = pathlib.Path(os.environ["OUT_DIR"]).resolve()
n_files = int(os.environ["N_FILES"])
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
report.append("# Many small files — create + list + delete\n\n")
report.append(f"N_FILES: {n_files}\n\n")
report.append("| Lang | Count (median) | Wall sec (median) | Ops/s (files/sec) | Peak RSS MiB (median) | N |\n")
report.append("| --- | ---:| ---:| ---:| ---:| ---:|\n")

for name, rs in sorted(by.items()):
  count_vals=[int(r["count"]) for r in rs if r["count"].isdigit()]
  sec_vals=[float(r["sec"]) for r in rs if r["sec"]]
  rss_vals=[int(r["maxrss_kb"]) for r in rs if r["maxrss_kb"].isdigit()]
  c=med(sorted(count_vals))
  s=med(sorted(sec_vals))
  rss=med(sorted(rss_vals))
  n=len(rs)
  ops = (n_files/s) if s and s>0 else 0.0
  rss_mib = (rss/1024.0) if rss else 0.0
  report.append(f"| `{name}` | {c} | {s:.6f} | {ops:.0f} | {rss_mib:.1f} | {n} |\n")

(out_dir/"report.md").write_text("".join(report))
print("OK:", out_dir/"report.md")
PY

echo "[fs_small] OK: $OUT_DIR/report.md"
