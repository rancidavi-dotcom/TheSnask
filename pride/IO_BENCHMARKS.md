# I/O Benchmarks (throughput + peak RAM)

Goal: compare Snask against common languages on a realistic “write + read back” workload, including **peak RAM**.

Source of truth:
- `bench/io/out/report.md`

What we do:
- write a file by appending 1 MiB chunks repeatedly (`SIZE_MB`)
- read it back and count bytes

Metrics:
- wall time (median)
- throughput (MiB/s)
- peak RSS (median), from `/usr/bin/time -v`

Reproduce:
```bash
SIZE_MB=256 RUNS=7 ./bench/io/run.sh
cat bench/io/out/report.md
```

Notes:
- Today Snask reads the file into a single string via `sfs::read`, so its peak RAM includes the file size.
  This benchmark reflects the current standard library surface; improving streaming I/O would improve Snask here.

