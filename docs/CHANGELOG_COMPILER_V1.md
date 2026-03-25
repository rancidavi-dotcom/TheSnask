# Snask Compiler Changelog (v1.x)

This file documents major compiler architecture changes and notable additions.

## v0.3.0
- SPS MVP (manifest + deps + deterministic lockfile).
- Runtime: SQLite integration, multithreading (pthread), GUI (GTK3) support.
- Language ergonomics: improved indentation handling and better diagnostics (English output).
- Libraries: import-only native APIs (reserved native symbols must be accessed via packages).

