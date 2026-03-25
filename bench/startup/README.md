# Cold start benchmark (CLI)

This benchmark measures **CLI cold start** across languages on Linux:

- **TTFB (time to first byte)**: how fast the program can produce its first stdout byte.
- **Wall time**: total runtime as measured by `/usr/bin/time`.

Why both?
- TTFB captures “how fast the tool feels” (prompt responsiveness).
- Wall time captures end-to-end overhead.

## What we compare
- **Snask**: `--release-size` and `--ultra-tiny`
- **C**: `gcc file.c -o app` (defaults)
- **Go**: `go build` (defaults)
- **Python**: `python3 script.py`
- **Node**: `node script.js`

All programs do the same thing: print `ok` and exit.

## Run
From repo root:

```bash
./bench/startup/run.sh
cat bench/startup/out/report.md
```

## Notes
- This benchmark is best-effort “realistic cold-ish” start.
  - The kernel page cache, CPU frequency scaling (turbo), background daemons, and scheduler jitter **will** affect results.
  - Below ~1–2 ms, differences of tens of microseconds are often within noise unless you collect lots of samples + interleave runs.
- The runner supports **interleaved A/B runs** and reports p50/p95 plus a bootstrap CI for the difference (optional).

### Recommended “most realistic possible” setup
- Keep your normal desktop running (don’t isolate too aggressively), but:
  - close heavy apps and stop large downloads
  - run on AC power
  - avoid thermal throttling
- If you want “most stable numbers” (less realistic, more lab-like):
  - set CPU governor to performance
  - disable turbo (optional)
  - pin to one core (the scripts do this via `taskset`)
