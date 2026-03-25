# Many small files benchmark (create + list + delete)

This benchmark measures filesystem operations for **many small files**:

- Create `N_FILES` small files (1 KiB each)
- List the directory (count entries)
- Delete the files

Metrics:
- **Ops/s** = `N_FILES / wall_seconds`
- **Peak RSS** from `/usr/bin/time -v`

## How to run

```bash
cargo build --release
./target/release/snask setup

# Full run (as used for the pride numbers)
N_FILES=100000 RUNS=5 ./bench/fs_small/run.sh
cat bench/fs_small/out/report.md

# Quick sanity run
N_FILES=5000 RUNS=3 ./bench/fs_small/run.sh
```

Note:
- The Snask variant is built with `snask build --release-size --min-runtime` (the runner script does this automatically).

## Latest result (example)

From `bench/fs_small/out/report.md`:

| Lang | Count (median) | Wall sec (median) | Ops/s (files/sec) | Peak RSS MiB (median) | N |
| --- | ---:| ---:| ---:| ---:| ---:|
| `c` | 100000 | 8.600000 | 11628 | 1.4 | 5 |
| `go` | 100000 | 9.400000 | 10638 | 19.8 | 5 |
| `node` | 100000 | 9.210000 | 10858 | 78.3 | 5 |
| `python` | 100000 | 11.590000 | 8628 | 17.0 | 5 |
| `snask` | 100000 | 7.740000 | 12920 | 1.4 | 5 |
