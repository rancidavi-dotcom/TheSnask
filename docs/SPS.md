# SPS (Snask Project System) — v1 (MVP)

SPS is Snask’s official project system (manifest + dependencies + lockfile). It is a core part of Snask being a batteries-included platform language.

## 1) Create a project
```bash
snask init
```

Creates:
- `snask.snif`
- `main.snask` (default entry)

## 2) Manifest: `snask.snif`
Example:
```snif
{
  package: { name: "my_app", version: "0.1.0", entry: "main.snask", },
  dependencies: { json: "*", },
  build: { opt_level: 2, },
}
```

Fields:
- `package.name` (required)
- `package.version` (required)
- `package.entry` (default: `main.snask`)
- `build.opt_level` (0..3, default: 2)
- `dependencies` (map: `name -> version`, where `*` means “any” for now)

## 3) Build / run without a file
Inside an SPS project:
```bash
snask build
snask run
```

You can still build a file directly:
```bash
snask build other.snask
snask run other.snask
```

## 4) Dependencies
```bash
snask add json
snask remove json
```

## 5) Lockfile: `snask.lock`
On `snask build`, SPS writes a deterministic lockfile (version + sha256) to make builds reproducible.
