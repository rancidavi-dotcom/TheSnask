# Snask Benchmarks (size-first)

This folder contains **size-focused** benchmarks to compare Snask against **conventional C (gcc defaults)**.

Benchmarks:
- `cli_hello`: print a single line and exit
- `cli_io`: write/read a small file and validate
- `cli_full`: “real CLI app” size benchmark (subcommands + SNIF config parsing)
- `gui_min` (optional): minimal GTK window + quit

Rules (anti-cheat):
- **C conventional** means `gcc file.c -o out` with no size flags.
- Snask is allowed to use its official profiles (`--tiny`, `--release-size`, and future `--ultra-tiny`).

Run:
```bash
./bench/run.sh
```

Outputs:
- `bench/out/report.md`
- built binaries in `bench/out/`
