# Snask Guide (Developer Track) — v0.3.0

This guide is a consolidated reference for Snask as a batteries-included platform language for fast desktop and tooling apps: installation → language → packages → SPS → SNIF → GUI.

## 1) Install / update
```bash
curl -fsSL https://raw.githubusercontent.com/rancidavi-dotcom/TheSnask/main/install.sh | bash
snask --version
```

## 2) Program structure
Snask programs typically define:
```snask
class main
    fun start()
        print("Hello");
```

## 3) Indentation
Blocks are indentation-based. Keep indentation consistent throughout the file.

## 4) Imports and namespaces
```snask
import "json";
import "snif";

class main
    fun start()
        let v = snif::parse_value("{ a: 1, }"); // SNIF example
```

## 5) SPS projects
```bash
snask init
snask build
```

Manifest: `snask.snif` (see `docs/SPS.md`).

## 6) SNIF (Snask Interchange Format)
SNIF is Snask’s configuration/interchange format (see `docs/snif/spec.md`).

## 7) Desktop + tooling mindset
Snask is designed around building “real” apps:
- Desktop GUIs (GTK on Linux) via `import "gui"`.
- Developer tools (CLIs, automation, packaging) via SPS and the package ecosystem.
- Built-in primitives (SNIF, HTTP, SQLite, threads) exposed via import-only libraries.
