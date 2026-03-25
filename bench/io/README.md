# I/O throughput + peak RAM (Snask vs C vs Go vs Node vs Python)

This benchmark writes a file by appending **1 MiB chunks** repeatedly, then **stats** the file size.

Metrics:
- **Wall time** (seconds)
- **Peak RSS** (KiB) via `/usr/bin/time -v` “Maximum resident set size”

Defaults:
- Size: 256 MiB (`SIZE_MB=256`)
- Runs: 7 (`RUNS=7`)

## Run
From repo root:

```bash
./bench/io/run.sh
cat bench/io/out/report.md
```

## Notes / fairness
- This is a **write throughput** benchmark (write + `stat()` size).
- We do not verify file contents; the goal is “how fast can you write a blob to disk/pagecache”.
