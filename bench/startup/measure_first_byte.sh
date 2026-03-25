#!/usr/bin/env bash
set -euo pipefail

# Measure "time to first stdout byte" (TTFB) in milliseconds.
# Usage:
#   ./bench/startup/measure_first_byte.sh -- <cmd...>

if [[ "${1:-}" != "--" ]]; then
  echo "usage: $0 -- <cmd...>" >&2
  exit 2
fi
shift

if [[ $# -lt 1 ]]; then
  echo "command required" >&2
  exit 2
fi

python3 - "$@" <<'PY'
import os, subprocess, sys, time

cmd = sys.argv[1:]
if not cmd:
  print("command required", file=sys.stderr)
  sys.exit(2)

start = time.perf_counter_ns()
proc = subprocess.Popen(cmd, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL)
try:
  b = proc.stdout.read(1) if proc.stdout is not None else b""
finally:
  # Best-effort: don't leave stragglers if something goes wrong.
  try: proc.kill()
  except Exception: pass

end = time.perf_counter_ns()

if not b:
  sys.exit(1)

ms = (end - start) / 1_000_000.0
print(f"{ms:.10f}")
PY
