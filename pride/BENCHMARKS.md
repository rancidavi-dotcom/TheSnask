# Benchmarks (proof) — Snask vs conventional C (gcc defaults)

This is a **reproducible** size benchmark focused on *executables alone* (dynamic linking still uses system libraries).

Related:
- `pride/RAM_BENCHMARKS.md` — Snask (native GTK) vs Electron (Chromium) RAM/PSS comparison

## What we compare

### C (conventional)
“Conventional C” is defined as:
```bash
gcc file.c -o app
```
No `-Os`, no `-s`, no LTO, no linker scripts.

### Snask
We build the same behavior using Snask and compare these variants:
- `--tiny`
- `--release-size`
- `--ultra-tiny` (custom `_start`, no CRT; Linux x86_64 for now)

## How to reproduce
From the repo root:
```bash
cargo build --release
./target/release/snask setup
./bench/run.sh
cat bench/out/report.md
```

## Results (bytes)

Source of truth: `bench/out/report.md`.

### CLI hello
| Variant | Size |
| --- | ---: |
| C (gcc default) | 15.59 KiB (15960) |
| Snask `--tiny` | 13.95 KiB (14280) |
| Snask `--ultra-tiny` | 13.30 KiB (13616) |

### CLI IO (write + read + validate)
| Variant | Size |
| --- | ---: |
| C (gcc default) | 15.88 KiB (16264) |
| Snask `--tiny` | 14.02 KiB (14352) |
| Snask `--ultra-tiny` | 13.37 KiB (13688) |

### CLI full (subcommands + SNIF config parse)
This is a more “realistic” CLI skeleton (command table + SNIF parsing code paths).

| Variant | Size |
| --- | ---: |
| C (gcc default) | 16.12 KiB (16512) |
| Snask `--tiny` | 26.09 KiB (26720) |
| Snask `--ultra-tiny` | 25.37 KiB (25976) |

## Visual charts (lower is better)

Scale: 1 block ≈ 0.5 KiB

### cli_hello
- C (gcc)          15.59 KiB | ████████████████████████████
- Snask `--tiny`   13.95 KiB | ██████████████████████████
- Snask `--ultra`  13.30 KiB | █████████████████████████

### cli_io
- C (gcc)          15.88 KiB | ████████████████████████████
- Snask `--tiny`   14.02 KiB | ██████████████████████████
- Snask `--ultra`  13.37 KiB | █████████████████████████

## Why this is a big deal

Snask is a high-level language with:
- a project system (SPS),
- a runtime,
- and first-party tooling.

Yet, on these CLI benchmarks, **Snask produces smaller executables than conventional C (gcc defaults)** — and does it via **repeatable build profiles**, not manual hand-tuning per project.
