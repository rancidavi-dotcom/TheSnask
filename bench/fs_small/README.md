# Many small files (create + list + delete) — Snask vs C vs Go vs Node vs Python

Workload:
1) Create `N` small files (1 KiB each) in a fresh directory
2) List directory entries (count)
3) Delete all files

Metrics:
- wall time (seconds)
- ops/s (N / wall) per phase + total
- peak RSS (KiB) via `/usr/bin/time -v`

Defaults:
- `N=100000` (`N_FILES=100000`)
- runs: `RUNS=5`

Run:
```bash
N_FILES=100000 RUNS=5 ./bench/fs_small/run.sh
cat bench/fs_small/out/report.md
```

Notes:
- This is a filesystem-heavy benchmark; results depend strongly on disk type and mount options.
- We avoid `fsync` to measure typical tooling workloads (pagecache + metadata).
- Snask is built with `snask build --release-size --min-runtime` to avoid linking heavy subsystems for a pure-CLI workload.
