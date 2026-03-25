#!/usr/bin/env bash
set -euo pipefail

# Cold-start profiling (Linux) for C vs Snask ultra-tiny.
# Produces artifacts in bench/startup/out/profile/.
#
# Usage:
#   ./bench/startup/run.sh
#   ./bench/startup/profile.sh
#
# Optional env:
#   PERF_RUNS=50
#   TASKSET_CPU=0

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"
OUT_DIR="$ROOT_DIR/bench/startup/out"
PROF_DIR="$OUT_DIR/profile"
mkdir -p "$PROF_DIR"

SNASK_BIN="$OUT_DIR/snask_ok_ultra_tiny"
C_BIN="$OUT_DIR/c_ok"

if [[ ! -x "$SNASK_BIN" ]]; then
  echo "missing: $SNASK_BIN" >&2
  echo "run: ./bench/startup/run.sh" >&2
  exit 1
fi
if [[ ! -x "$C_BIN" ]]; then
  echo "missing: $C_BIN" >&2
  echo "run: ./bench/startup/run.sh (with gcc installed)" >&2
  exit 1
fi

PERF_RUNS="${PERF_RUNS:-50}"
TASKSET_CPU="${TASKSET_CPU:-0}"

run_cmd() {
  local name="$1"; shift
  local bin="$1"; shift
  local -a cmd=("$@")

  echo "[profile] $name"

  # ldd snapshot
  (ldd "$bin" || true) >"$PROF_DIR/${name}_ldd.txt"

  # loader stats
  (env LD_DEBUG=statistics "${cmd[@]}" >/dev/null) 2>"$PROF_DIR/${name}_ld_debug_statistics.txt" || true

  # syscall summary (follow forks/threads, count only)
  if command -v strace >/dev/null 2>&1; then
    strace -f -c -o "$PROF_DIR/${name}_strace_c.txt" "${cmd[@]}" >/dev/null 2>&1 || true
  else
    echo "WARN: strace not installed" >"$PROF_DIR/${name}_strace_c.txt"
  fi

  # perf stat (repeat; capture stderr)
  if command -v perf >/dev/null 2>&1; then
    perf stat -r "$PERF_RUNS" -- taskset -c "$TASKSET_CPU" "${cmd[@]}" >/dev/null 2>"$PROF_DIR/${name}_perf_stat.txt" || true
  else
    echo "WARN: perf not installed" >"$PROF_DIR/${name}_perf_stat.txt"
  fi
}

run_cmd "c" "$C_BIN" taskset -c "$TASKSET_CPU" "$C_BIN"
run_cmd "snask_ultra_tiny" "$SNASK_BIN" taskset -c "$TASKSET_CPU" "$SNASK_BIN"

PROF_DIR="$PROF_DIR" python3 - <<'PY'
import pathlib, re

prof = pathlib.Path(__import__("os").environ["PROF_DIR"]).resolve()
prof.mkdir(parents=True, exist_ok=True)

def read(p):
  try:
    return p.read_text(errors="replace")
  except FileNotFoundError:
    return ""

def parse_ld_debug(txt: str):
  # best-effort: extract last "total time" line if present
  lines = [l.strip() for l in txt.splitlines() if l.strip()]
  # keep a small tail for report
  return "\n".join(lines[-20:])

def parse_strace_c(txt: str):
  # keep top syscall rows (strace -c table)
  lines = txt.splitlines()
  # find header line containing "% time"
  out=[]
  for i,l in enumerate(lines):
    if "% time" in l and "seconds" in l:
      out = lines[i:i+15]
      break
  if not out:
    out = lines[-15:]
  return "\n".join(out).strip()

def parse_perf_stat(txt: str):
  # keep key lines
  keep=[]
  for l in txt.splitlines():
    if any(k in l for k in ["seconds time elapsed","cycles","instructions","task-clock","context-switches","cpu-migrations","page-faults"]):
      keep.append(l)
  return "\n".join(keep[-20:]).strip()

report=[]
report.append("# Cold start profiling — C vs Snask (ultra-tiny)\n\n")
report.append("Artifacts are stored in `bench/startup/out/profile/`.\n\n")
for name in ["c","snask_ultra_tiny"]:
  report.append(f"## `{name}`\n\n")
  report.append("### ldd\n\n```text\n")
  report.append(read(prof / f"{name}_ldd.txt").strip() + "\n")
  report.append("```\n\n")

  report.append("### LD_DEBUG=statistics (tail)\n\n```text\n")
  report.append(parse_ld_debug(read(prof / f"{name}_ld_debug_statistics.txt")) + "\n")
  report.append("```\n\n")

  report.append("### strace -c (top)\n\n```text\n")
  report.append(parse_strace_c(read(prof / f"{name}_strace_c.txt")) + "\n")
  report.append("```\n\n")

  report.append("### perf stat (selected)\n\n```text\n")
  report.append(parse_perf_stat(read(prof / f"{name}_perf_stat.txt")) + "\n")
  report.append("```\n\n")

(prof / "report.md").write_text("".join(report))
print("OK:", prof / "report.md")
PY

echo "[profile] OK: $PROF_DIR/report.md"
