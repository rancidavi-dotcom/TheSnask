# Snask

**Snask** is a batteries-included platform language for building fast desktop and tooling applications with a unified ecosystem.

It is compiled (LLVM 18) and ships with a first-party runtime + package workflow designed for real-world apps.

## Install (Linux)

Install or update Snask with a single command:

```bash
curl -fsSL https://raw.githubusercontent.com/rancidavi-dotcom/TheSnask/main/install.sh | bash
```

## Highlights (v0.3.0)

| Feature | Description |
| --- | --- |
| Native compilation | Produces native executables via LLVM (no interpreter). |
| Indentation-based blocks | Blocks are defined by spaces (Python-like). |
| SPS (Project System) | `snask.snif`, dependencies, lockfile, and `snask build` without arguments. |
| Unified ecosystem | Import-only native APIs + official packages (stable boundaries). |
| Desktop-first | GTK-based GUI stack via the `gui` package (Linux target). |
| Tooling-first | Strong CLI workflow (`init/add/remove/build/run/setup`) for projects and packages. |
| Native runtime | Builtins for IO, HTTP, JSON, SNIF, GUI (GTK), SQLite, threads, and more. |
| SQLite + threads | Native integrations for data access and multithreading. |

## Documentation

See `docs/`:
- `docs/GUIA_SNASK.md` — Full language guide.
- `docs/APRENDA_SNASK.md` — Beginner tutorial.
- `docs/BIBLIOTECAS_SNASK.md` — Packages and modules.
- `docs/SPS.md` — SPS manifest (`snask.snif`) + dependency flow.
- `docs/snif/spec.md` — SNIF (Snask Interchange Format) specification.
- `docs/ROADMAP.md` — Phased roadmap (priorities + milestones).
- `docs/STATUS.md` — Current stability status (stable/beta/experimental).

## Product direction (roadmap-style)
If Snask’s purpose is “batteries-included desktop + tooling”, the highest-impact areas are:
- **Standard app primitives**: config (SNIF), logging, paths/fs, networking, process management.
- **First-class GUI DX**: widgets/layout helpers, theming, packaging.
- **Reliable builds**: deterministic deps/lockfile, better diagnostics, predictable project layout.

## Current status (v0.3.0)
Stable:
- Core language syntax
- Base runtime
- SPS core (build/run/install)
- SNIF core parsing/spec
- Package resolution

Beta:
- GUI toolkit
- SQLite bindings
- Threading
- Packaging / dist tooling
- Store app

Experimental:
- Advanced diagnostics
- Deterministic builds hardening
- Advanced optimizations
- Plugins / future extensions

## Status

Snask is actively evolving; breaking changes may occur between versions. Issues and contributions are welcome.

## License

MIT License.
