# Snask — Pride Folder

This folder is a curated set of **proof points** and “wins” that show why Snask is worth paying attention to.

## Conquests (quick links)
- **Size battle (Snask vs conventional C / gcc defaults):** `pride/BENCHMARKS.md`
- **RAM battle (Snask native GTK vs Electron minimal):** `pride/RAM_BENCHMARKS.md`
- **FS battle (100k small files):** `pride/FS_SMALL_BENCHMARKS.md`

Contents:
- `pride/WHY_SNASK_IS_FAST_AND_SMALL.md` — plain-English explanation of how Snask gets tiny native binaries
- `pride/BENCHMARKS.md` — reproducible benchmark results (Snask vs conventional C) + charts
- `pride/RAM_BENCHMARKS.md` — reproducible RAM benchmarks (Snask vs Electron minimal)
- `pride/STARTUP_BENCHMARKS.md` — reproducible cold-start benchmarks (Snask vs other languages)
- `pride/SNIF_BENCHMARKS.md` — reproducible SNIF vs other formats benchmark (config, 100MB)
- `pride/IO_BENCHMARKS.md` — reproducible I/O throughput + peak RAM benchmark (Snask vs C/Go/Node/Python)
- `pride/FS_SMALL_BENCHMARKS.md` — many small files benchmark (create + list + delete)

How to reproduce the benchmark numbers:
```bash
cargo build --release
./target/release/snask setup
./bench/run.sh
cat bench/out/report.md
```
