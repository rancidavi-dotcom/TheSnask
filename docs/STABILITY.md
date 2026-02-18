# Stability Policy (v0.3.0 baseline; current v0.3.1)

Snask is evolving quickly, but it must remain predictable for real desktop and tooling projects.

This document defines what **Stable / Beta / Experimental** mean and how compatibility is handled.

## Stability levels

### Stable
- Expected to work reliably.
- Breaking changes are avoided.
- Changes are allowed only for bug fixes and security fixes, and should be backwards compatible.

### Beta
- Usable, but APIs may change.
- Breaking changes may happen between minor releases.
- Edge cases and performance may still be in progress.

### Experimental
- Incomplete or under heavy iteration.
- Breaking changes can happen at any time.
- Intended for early adopters and internal testing.

## Current classification (v0.3.1)
Stable:
- Core language syntax
- Base runtime
- SPS core (`init/build/run/install/uninstall/update`, manifest + entry resolution)
- SNIF core parsing/spec
- Package resolution (registry + install flow)

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

## Compatibility rules
- **Patch releases**: must not intentionally break Stable features.
- **Minor releases**: may adjust Beta features; Stable features should still be kept compatible.
- **Experimental**: no compatibility guarantee.

## “Language core” vs “batteries”
- The **language core** should stay small and stable.
- “Batteries” (GUI, SQLite, HTTP, SNIF utilities, etc.) ship as **libraries** with import-only native boundaries.
