# RAM Benchmarks (Snask vs Electron)

Goal: validate the claim that Snask native GUI apps can use **far less RAM** than Electron-style GUI apps, even for “minimal UI”.

Source of truth:
- `bench/ram/out/report.md`

Key metric:
- **PSS** (Proportional Set Size), because Electron launches a **process tree** and shared pages must be accounted for fairly.

## Result (latest snapshot)

Snask (GTK) vs Electron (Chromium) for a minimal “one window” app:
- Snask Vault: **~10.7 MiB PSS** (1 process)
- Electron minimal: **~270 MiB PSS** (7 processes)

Repro steps:
- `bench/ram/out/report.md`

