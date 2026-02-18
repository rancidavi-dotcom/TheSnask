# Files and Features (v0.3.0)

This document describes how to use Snask v0.3.0 features related to:
- GC (simple runtime GC for internal strings/buffers)
- SQLite runtime integration
- Cross-compilation (MVP) via `--target`

Snask is compiled (produces a native binary). `snask setup` prepares the native runtime used by the linker.

## Prerequisites (Linux)
- `cargo`
- `clang-18`, `llc-18`
- `gcc`
- `pkg-config`

Optional:
- SQLite headers: `libsqlite3-dev`
- GTK3 headers: `libgtk-3-dev`

Then:
```bash
snask setup
```

## GC (runtime)
Snask v0.3.0 tracks many internal allocations (strings/buffers) and frees them at process exit.

Notes:
- This is not a full incremental/compacting GC.
- Long-running processes can still grow memory if they allocate continuously.

## SQLite
SQLite is available via the `sqlite` package (import-only).

## Cross-compilation
Use:
```bash
snask build --target <llvm-triple>
```

