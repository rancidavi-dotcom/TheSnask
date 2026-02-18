# Snask Roadmap (platform language)

Snask’s goal:
> “Snask is a batteries-included platform language for building fast desktop and tooling applications with a unified ecosystem.”

This file turns the goal into an execution-oriented roadmap: phases, priorities, and milestones.

## Guiding principles
- **Minimal surface area**: prefer fewer features that are well specified.
- **Determinism**: builds and package installs must be reproducible.
- **Strict boundaries**: the language core stays small; “batteries” ship as packages.
- **Desktop + tooling first**: prioritize GUI and CLI developer experience.

## Pillars (non-negotiable)
1) **SPS + SNIF**: project identity, dependency flow, lockfile, and a strict manifest format.
2) **GUI stack**: Linux desktop apps are a primary target.
3) **Tooling**: first-class CLI workflows (create/build/run/package/publish).

## Phases and milestones

### Phase 1 — Foundation (v0.3 → v0.4)
Goal: stability and predictability.
- **Core stability policy** (document what is stable vs experimental).
- **SNIF as a stable spec** (strict grammar + error messages + examples).
- **SPS polish**:
  - `snask.snif` as the default manifest,
  - lockfile by sha256 (already present) + clear error UX,
  - offline-friendly cache behavior.
- **Diagnostics baseline**: actionable errors (line/col + “how to fix”).

Exit criteria:
- A new user can `snask init` → `snask build` reliably on Linux.
- Projects build the same way across machines when `snask.lock` is present.

Implementation tracker: see `docs/PHASE1_CHECKLIST.md`.

### Phase 2 — Developer Experience (v0.5 → v0.7)
Goal: productivity and “pleasant to use”.
- **Higher-level GUI toolkit** on top of `gui`:
  - forms/layout helpers,
  - standard widgets (dialogs, lists/tables),
  - theming presets (CSS packs, dark/light),
  - state/update helpers (reduce global state patterns).
- **CLI framework package** (args/subcommands/help generation).
- **SNIF tooling**:
  - formatter (`snif fmt`),
  - validator/linter (`snif check`).
- **Packaging MVP** in CLI (Linux):
  - `.deb` + `.AppImage`,
  - desktop entry + icons.

Exit criteria:
- Building a small desktop app (calculator/installer) feels “high-level”.
- Producing a distributable Linux artifact is one command.

### Phase 3 — Platform (v0.8 → v1.0)
Goal: ecosystem maturity and long-term maintainability.
- **Deterministic builds hardening**:
  - stronger lock semantics,
  - dependency integrity enforcement,
  - offline builds from cache.
- **Registry evolution** beyond a single file (index/search/pagination).
- **Package scaffolding** (`snask lib new`) + CI checks for publish.
- **Data layer**:
  - SQLite ORM improvements (migrations, query builder).
- **Performance**:
  - streaming decode for JSON/SNIF,
  - profiling and hotspots.

## Feature backlog (organized by domain)

### Desktop app DX (GUI)
- **Higher-level GUI toolkit** on top of `gui`:
  - declarative layout helpers (rows/columns/forms),
  - standard widgets (menus, tabs, table/list, dialogs),
  - theming API (CSS presets, dark/light, icon packs),
  - state/update helpers (avoid manual global state patterns).
- **App packaging command** in SPS/CLI:
  - Linux: `.deb` + `.AppImage`,
  - Desktop entry + icon installation,
  - versioned release artifacts.

### Tooling DX (CLI apps)
- **CLI framework package** (arg parsing, subcommands, help generation).
- **Process API** (spawn, pipe, capture output, timeouts).
- **Filesystem/path polish** (copy/move/glob, temp dirs, atomic write).

### Data + config
- **SNIF tooling**:
  - formatter (`snif fmt`),
  - validator/linter (`snif check`),
  - schema-lite conventions (typed tags + required keys).
- **SQLite ORM improvements** (query builder, migrations).

### Reliability and performance
- **Better diagnostics** (actionable hints, multi-span errors).
- **Deterministic builds**:
  - stronger lockfile semantics,
  - dependency integrity by sha256,
  - offline builds from cache.
- **Streaming decode** for JSON/SNIF in runtime (large payloads).

### Ecosystem and distribution
- **Package scaffolding** (`snask lib new` templates + CI checks).
- **Package metadata standardization** (required fields, icons, categories).
- **Registry evolution** beyond a single file (indexing, search, pagination).
