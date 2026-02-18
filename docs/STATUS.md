# Snask Status (v0.3.1)

This document describes the current stability status of Snask features and the immediate focus for Phase 1.

## Current release
- **Current:** v0.3.1

## Stability levels
- **Stable:** expected to work reliably; breaking changes avoided.
- **Beta:** usable but may change; edge cases and API polish still in progress.
- **Experimental:** may be incomplete or change frequently; expect rough edges.

## Stable
- **Core language syntax**
- **Base runtime**
- **SPS core** (`init/build/run/install/uninstall/update`, manifest + entry resolution)
- **SNIF core** (strict parsing + spec)
- **Package resolution** (registry + install flow)

## Beta
- **GUI toolkit**
- **SQLite bindings**
- **Threading**
- **Packaging / dist tooling**
- **Store app**

## Experimental
- **Advanced diagnostics** (multi-span, rich hints, “did you mean”)
- **Deterministic builds hardening** (lock semantics, offline mode guarantees)
- **Advanced optimizations**
- **Plugins / future extensions**

## Phase 1 focus (v0.3 → v0.4)
Priority is **stability + error UX**, not new features:
1) **Core stability policy** (what is stable vs beta vs experimental; compatibility rules)
2) **Diagnostics baseline** (actionable errors with “how to fix” hints)
