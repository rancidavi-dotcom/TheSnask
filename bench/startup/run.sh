#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"
OUT_DIR="$ROOT_DIR/bench/startup/out"
mkdir -p "$OUT_DIR"

RUNS="${RUNS:-25}"
WARMUP="${WARMUP:-3}"
INTERLEAVE="${INTERLEAVE:-1}"   # 1 = A/B/A/B order, 0 = run each target in blocks
TASKSET_CPU="${TASKSET_CPU:-0}"
BOOTSTRAP="${BOOTSTRAP:-1}"    # 1 = compute bootstrap CI for Snask vs C delta

need() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "missing required tool: $1" >&2
    exit 1
  fi
}

need python3

SNASK_BIN="${SNASK_BIN:-$ROOT_DIR/target/release/snask}"
if [[ ! -x "$SNASK_BIN" ]]; then
  echo "snask binary not found at: $SNASK_BIN" >&2
  echo "build it first: cargo build --release" >&2
  exit 1
fi

echo "[startup] building binaries..."

pushd "$ROOT_DIR/bench/startup" >/dev/null

# C
if command -v gcc >/dev/null 2>&1; then
  gcc c_ok.c -o "$OUT_DIR/c_ok"
else
  echo "[startup] WARN: gcc not found; skipping C" >&2
fi

# Go
if command -v go >/dev/null 2>&1; then
  go build -o "$OUT_DIR/go_ok" go_ok.go
else
  echo "[startup] WARN: go not found; skipping Go" >&2
fi

# Snask
"$SNASK_BIN" build snask_ok.snask --release-size >/dev/null
mv -f "$ROOT_DIR/bench/startup/snask_ok" "$OUT_DIR/snask_ok_release_size"

"$SNASK_BIN" build snask_ok.snask --ultra-tiny >/dev/null
mv -f "$ROOT_DIR/bench/startup/snask_ok" "$OUT_DIR/snask_ok_ultra_tiny"

popd >/dev/null

measure_wall_ms() {
  local out_file="$1"; shift
  local cmd=("$@")
  /usr/bin/time -f '%e' "${cmd[@]}" >/dev/null 2>&1 | true
}

measure_wall_ms_one() {
  local cmd=("$@")
  local start_ns end_ns
  start_ns=$(date +%s%N)
  "${cmd[@]}" >/dev/null 2>&1 || true
  end_ns=$(date +%s%N)
  python3 - <<PY
ns = int("${end_ns}") - int("${start_ns}")
ms = ns / 1_000_000.0
print(f"{ms:.10f}")
PY
}

bench_one() {
  local name="$1"; shift
  local -a cmd=("$@")

  echo "[startup] bench: $name"

  local ttfb_file="$OUT_DIR/${name}_ttfb_ms.txt"
  local wall_file="$OUT_DIR/${name}_wall_ms.txt"
  : >"$ttfb_file"
  : >"$wall_file"

  # warmup
  for _ in $(seq 1 "$WARMUP"); do
    "$ROOT_DIR/bench/startup/measure_first_byte.sh" -- "${cmd[@]}" >/dev/null || true
    measure_wall_ms_one "${cmd[@]}" >/dev/null || true
  done

  for _ in $(seq 1 "$RUNS"); do
    "$ROOT_DIR/bench/startup/measure_first_byte.sh" -- "${cmd[@]}" >>"$ttfb_file"
    measure_wall_ms_one "${cmd[@]}" >>"$wall_file"
  done
}

bench_pair_interleaved() {
  local a_name="$1"; shift
  local a_bin="$1"; shift
  local b_name="$1"; shift
  local b_bin="$1"; shift

  local a_ttfb="$OUT_DIR/${a_name}_ttfb_ms.txt"
  local a_wall="$OUT_DIR/${a_name}_wall_ms.txt"
  local b_ttfb="$OUT_DIR/${b_name}_ttfb_ms.txt"
  local b_wall="$OUT_DIR/${b_name}_wall_ms.txt"

  : >"$a_ttfb"; : >"$a_wall"; : >"$b_ttfb"; : >"$b_wall"

  echo "[startup] warmup: ${a_name} vs ${b_name}"
  for _ in $(seq 1 "$WARMUP"); do
    "$ROOT_DIR/bench/startup/measure_first_byte.sh" -- taskset -c "$TASKSET_CPU" "$a_bin" >/dev/null || true
    "$ROOT_DIR/bench/startup/measure_first_byte.sh" -- taskset -c "$TASKSET_CPU" "$b_bin" >/dev/null || true
  done

  echo "[startup] interleaved bench: ${a_name} vs ${b_name} (N=$RUNS)"
  for _ in $(seq 1 "$RUNS"); do
    "$ROOT_DIR/bench/startup/measure_first_byte.sh" -- taskset -c "$TASKSET_CPU" "$a_bin" >>"$a_ttfb"
    measure_wall_ms_one taskset -c "$TASKSET_CPU" "$a_bin" >>"$a_wall"

    "$ROOT_DIR/bench/startup/measure_first_byte.sh" -- taskset -c "$TASKSET_CPU" "$b_bin" >>"$b_ttfb"
    measure_wall_ms_one taskset -c "$TASKSET_CPU" "$b_bin" >>"$b_wall"
  done
}

echo "[startup] running benchmarks (INTERLEAVE=$INTERLEAVE, CPU=$TASKSET_CPU)..."

if [[ "$INTERLEAVE" == "1" && -x "$OUT_DIR/c_ok" ]]; then
  # High-fidelity comparison: Snask ultra-tiny vs C, interleaved on the same core.
  bench_pair_interleaved "c" "$OUT_DIR/c_ok" "snask_ultra_tiny" "$OUT_DIR/snask_ok_ultra_tiny"
else
  # Block mode (fallback)
  if [[ -x "$OUT_DIR/c_ok" ]]; then bench_one "c" taskset -c "$TASKSET_CPU" "$OUT_DIR/c_ok"; fi
  bench_one "snask_ultra_tiny" taskset -c "$TASKSET_CPU" "$OUT_DIR/snask_ok_ultra_tiny"
fi

# Others (not interleaved by default)
if [[ -x "$OUT_DIR/go_ok" ]]; then bench_one "go" taskset -c "$TASKSET_CPU" "$OUT_DIR/go_ok"; fi
bench_one "snask_release_size" taskset -c "$TASKSET_CPU" "$OUT_DIR/snask_ok_release_size"
bench_one "python" taskset -c "$TASKSET_CPU" python3 "$ROOT_DIR/bench/startup/py_ok.py"
bench_one "node" taskset -c "$TASKSET_CPU" node "$ROOT_DIR/bench/startup/node_ok.js"

echo "[startup] generating report..."

OUT_DIR="$OUT_DIR" BOOTSTRAP="$BOOTSTRAP" python3 - <<'PY'
import os, statistics, pathlib
import random

out = pathlib.Path(os.environ["OUT_DIR"]).resolve()
out.mkdir(parents=True, exist_ok=True)
do_bootstrap = os.environ.get("BOOTSTRAP","1") == "1"

def load_nums(p: pathlib.Path):
  vals=[]
  for line in p.read_text().splitlines():
    line=line.strip()
    if not line: continue
    try: vals.append(float(line))
    except: pass
  return vals

def pct(vals, p):
  if not vals: return None
  xs=sorted(vals)
  k=(len(xs)-1)*p
  f=int(k)
  c=min(f+1,len(xs)-1)
  if f==c: return xs[f]
  return xs[f]*(c-k)+xs[c]*(k-f)

def summary(vals):
  vals=list(vals)
  vals.sort()
  return {
    "n": len(vals),
    "p50": statistics.median(vals),
    "p95": pct(vals, 0.95),
    "mean": statistics.fmean(vals),
    "stdev": statistics.pstdev(vals) if len(vals) > 1 else 0.0,
    "min": vals[0],
    "max": vals[-1],
  }

rows=[]
for base in sorted(out.glob("*_ttfb_ms.txt")):
  name = base.name.replace("_ttfb_ms.txt","")
  ttfb = load_nums(base)
  wall = load_nums(out / f"{name}_wall_ms.txt")
  if not ttfb or not wall:
    continue
  rows.append((name, summary(ttfb), summary(wall)))

def human_ms(ms): return f"{ms:.10f} ms"

report = []
report.append("# Cold start (CLI) — report\n")
report.append("Runs (measured): p50/p95 + mean/stdev per target.\n")
report.append("\n")
report.append("| Target | TTFB p50 | TTFB p95 | Wall p50 | Wall p95 | N |\n")
report.append("| --- | ---:| ---:| ---:| ---:| ---:|\n")
for name, t, w in rows:
  report.append(
    f"| `{name}` | {human_ms(t['p50'])} | {human_ms(t['p95'])} | {human_ms(w['p50'])} | {human_ms(w['p95'])} | {t['n']} |\n"
  )

if do_bootstrap:
  c = load_nums(out / "c_ttfb_ms.txt") if (out / "c_ttfb_ms.txt").exists() else []
  s = load_nums(out / "snask_ultra_tiny_ttfb_ms.txt") if (out / "snask_ultra_tiny_ttfb_ms.txt").exists() else []
  if c and s:
    # Bootstrap CI for delta in medians: (snask - c), in milliseconds.
    iters = 5000
    deltas=[]
    rng = random.Random(0)
    for _ in range(iters):
      cb = [rng.choice(c) for _ in range(len(c))]
      sb = [rng.choice(s) for _ in range(len(s))]
      deltas.append(statistics.median(sb) - statistics.median(cb))
    deltas.sort()
    lo = deltas[int(0.025*iters)]
    hi = deltas[int(0.975*iters)]
    report.append("\n")
    report.append("## Snask vs C (TTFB) — bootstrap CI\n\n")
    report.append("Delta definition: `median(snask_ultra_tiny) - median(c)` (ms).\n\n")
    report.append(f"- 95% CI: [{lo:.10f} ms, {hi:.10f} ms]\n")

(out / "report.md").write_text("".join(report))
print("OK:", out / "report.md")
PY

echo "[startup] OK: $OUT_DIR/report.md"
