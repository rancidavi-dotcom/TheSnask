# Skia backend (experimental)

Snask currently ships `snask_skia` with a **Cairo fallback** (enabled when GTK3 is enabled in the runtime).

To use a **real Skia backend**, the runtime must be compiled with `SNASK_SKIA`.

## Install Skia (one command)

Snask can install Skia as an optional component:

```bash
snask install-optional skia
```

This downloads Skia (via Chromium depot_tools), builds it, and generates a `skia.pc` file under `~/.snask/optional/pkgconfig/`.

## Enable Skia via `snask setup`

`snask setup` will try to detect Skia using:

```bash
pkg-config --cflags skia
pkg-config --libs skia
```

If it succeeds, setup will:
- compile the runtime with `-DSNASK_SKIA`
- add Skia link flags to `~/.snask/lib/runtime.linkargs`

If it fails, Snask keeps working and `snask_skia` runs on Cairo (fallback).

Note: `snask setup` automatically includes `~/.snask/optional/pkgconfig` in `PKG_CONFIG_PATH` if it exists.

## Installing Skia SDK (Linux)

Skia is not a default system dependency.

Recommended approaches:
- Provide a `skia.pc` file for `pkg-config` that points to your Skia build output and headers.
- Or install a system package that provides `pkg-config` metadata for Skia (rare).

## Current status

The runtime supports **both backends**:
- default: Cairo
- optional: real Skia (when installed)

To opt into real Skia in a program, set at top-level:

```snask
import "snask_skia";
const USE_SKIA = 1;
```

Snask will automatically enable the real backend before `main::start()` when available.
