# Snask Libraries (v0.3.0)

Snask uses a module system with **namespaces by default**.

After importing a library, you call functions using `lib::function()`.

Exception: `prelude` is designed to be imported and used **without prefixes** (ergonomics).

## Import
```snask
import "json";
import "snif";
import "os";
import "gui"; // legacy
import "snask_gtk"; // recommended
```

## Common libraries
- `json` — JSON parse/stringify and helpers.
- `snif` — SNIF (Snask Interchange Format): comments, trailing commas, typed literals, references, big-int tagging.
- `os` — OS helpers (env, cwd, random, etc.).
- `sfs` — filesystem/path helpers.
- `requests` — HTTP helpers.
- `sqlite` — SQLite bindings (library-only).
- `gui` — GTK3 GUI bindings (library-only). *(Legacy name; still supported.)*
- `snask_gtk` — **Snask_GTK** GUI bindings (recommended name).
- `snask_gtk_layout` — layout helpers for `snask_gtk`.
- `snask_gtk_widgets` — higher-level widgets for `snask_gtk`.
- `snask_gtk_app` — app scaffolding for `snask_gtk`.
- `snask_skia` — **experimental** Skia-based 2D graphics API (scaffolding).
- `log` — logging helpers.
- `prelude` — short helpers (`assert`, `println`, etc.).

## Important: import-only natives
Low-level native functions (`gui_*`, `sqlite_*`, `snif_*`, etc.) are reserved for libraries and cannot be called directly from apps. Always `import` the package and call `pkg::...`.

## Philosophy (batteries-included)
Snask aims to keep the language core small and stable, while shipping “batteries” as libraries with a unified distribution workflow (SPS + registry + lockfile).
