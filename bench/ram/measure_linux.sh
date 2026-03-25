#!/usr/bin/env bash
set -euo pipefail

# Measure RAM of a process tree on Linux using /proc/*/smaps_rollup.
# Outputs a JSON line to stdout.
#
# Usage:
#   ./bench/ram/measure_linux.sh --name snask_vault -- sleep 5
#   ./bench/ram/measure_linux.sh --name electron_min -- npm start

NAME=""
WAIT_SECS=3

while [[ $# -gt 0 ]]; do
  case "$1" in
    --name) NAME="$2"; shift 2;;
    --wait) WAIT_SECS="$2"; shift 2;;
    --) shift; break;;
    *) echo "unknown arg: $1" >&2; exit 2;;
  esac
done

if [[ -z "$NAME" ]]; then
  echo "--name is required" >&2
  exit 2
fi

if [[ $# -lt 1 ]]; then
  echo "command is required" >&2
  exit 2
fi

cmd=("$@")

# Start process in its own process group so we can kill everything.
setsid "${cmd[@]}" >/tmp/ram_${NAME}_stdout.txt 2>/tmp/ram_${NAME}_stderr.txt &
PID=$!

sleep "$WAIT_SECS"

if [[ ! -d "/proc/$PID" ]]; then
  echo "process exited early: $NAME" >&2
  exit 1
fi

# Collect descendants (including root)
collect_pids() {
  local root="$1"
  local pids=("$root")
  local i=0
  while [[ $i -lt ${#pids[@]} ]]; do
    local p="${pids[$i]}"
    local kids
    kids=$(pgrep -P "$p" || true)
    if [[ -n "$kids" ]]; then
      while read -r k; do
        [[ -n "$k" ]] && pids+=("$k")
      done <<<"$kids"
    fi
    i=$((i+1))
  done
  printf "%s\n" "${pids[@]}" | sort -n | uniq
}

pids=$(collect_pids "$PID")
count=$(wc -l <<<"$pids" | tr -d ' ')

sum_kb_field() {
  local field="$1"
  local total=0
  while read -r p; do
    local f="/proc/$p/smaps_rollup"
    if [[ -r "$f" ]]; then
      local v
      v=$(awk -v key="$field" '$1==key":" {print $2}' "$f" 2>/dev/null || true)
      [[ -n "$v" ]] && total=$((total+v))
    fi
  done <<<"$pids"
  echo "$total"
}

rss_kb=$(sum_kb_field "Rss")
pss_kb=$(sum_kb_field "Pss")
priv_clean_kb=$(sum_kb_field "Private_Clean")
priv_dirty_kb=$(sum_kb_field "Private_Dirty")
uss_kb=$((priv_clean_kb + priv_dirty_kb))

# record some info
now=$(date -Iseconds)

# best-effort command line (root)
cmdline=$(tr '\0' ' ' </proc/$PID/cmdline 2>/dev/null | sed 's/[[:space:]]\+$//' || true)

# kill process group
kill -TERM -"$PID" 2>/dev/null || true
sleep 0.5
kill -KILL -"$PID" 2>/dev/null || true

# JSON line (no jq needed)
printf '{"date":"%s","name":"%s","pid":%s,"proc_count":%s,"rss_kb":%s,"pss_kb":%s,"uss_kb":%s,"cmdline":"%s"}\n' \
  "$now" \
  "$NAME" \
  "$PID" \
  "$count" \
  "$rss_kb" \
  "$pss_kb" \
  "$uss_kb" \
  "${cmdline//"/\\"}"
