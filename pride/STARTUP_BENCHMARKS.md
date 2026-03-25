# Cold start benchmarks (Snask vs other languages)

Goal: show that Snask can feel “instant” for CLI tools by achieving **C-like cold start** times when built with size-focused profiles (especially `--ultra-tiny`).

Source of truth:
- `bench/startup/out/report.md`

What we measure:
- **TTFB** (time to first byte): how fast the program produces its first stdout byte.
- **Wall time**: end-to-end runtime.

Methodology highlights:
- Snask vs C is run in an **interleaved A/B** order on the same CPU core to reduce cache/warmup bias.
- Report includes **p50/p95** and a **bootstrap CI** for the Snask-vs-C delta (so we don’t overclaim microsecond differences).

## Latest snapshot

From `bench/startup/out/report.md` (N=300):
- `snask_ultra_tiny` TTFB p50: **~3.32 ms**
- `c` (gcc defaults) TTFB p50: **~3.37 ms**
- `python` TTFB p50: **~27 ms**
- `node` TTFB p50: **~31 ms**

Reproduce:
```bash
./bench/startup/run.sh
cat bench/startup/out/report.md
```

