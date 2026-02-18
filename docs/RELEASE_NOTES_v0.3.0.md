# Snask v0.3.0 — First Stable Release

This release marks Snask **v0.3.0** as the first stable baseline.

## Highlights
- **SPS (Snask Project System)** with `snask.snif` manifests, dependency install flow, and `snask.lock`.
- **SNIF (Snask Interchange Format)**: strict parsing, typed literals, references, and safe big-integer tagging.
- **Import-only native APIs**: low-level native functions are reserved for libraries, improving boundaries and ergonomics.
- **Runtime integrations**: GUI (GTK3), SQLite, multithreading, HTTP, filesystem utilities.
- **English-only tooling output**: compiler/runtime messaging standardized for universal use.

## Compatibility notes
- SPS manifests use `snask.snif` by default.
- `snask.toml` is supported as a deprecated fallback for migration.

## Docs
See:
- `docs/STATUS.md`
- `docs/ROADMAP.md`
- `docs/SPS.md`
- `docs/snif/spec.md`

