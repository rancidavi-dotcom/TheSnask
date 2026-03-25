# SNIF benchmarks (format-only): SNIF vs JSON/TOML/YAML/CBOR/MsgPack

Goal: benchmark SNIF as a **data/config format**, independent of the Snask language.

We compare SNIF against common formats:
- JSON
- TOML
- YAML
- CBOR
- MessagePack

Dataset choice:
- **Config-like** structure (nested objects, maps, strings, arrays)
- Target size: **~100MB** JSON baseline

What we measure:
- parse time
- canonical serialize (“canon”) time
- peak RSS (best-effort per format run)

Source of truth:
- `bench/format_snif/out/report.md`

How to reproduce:
```bash
./bench/format_snif/run.sh
cat bench/format_snif/out/report.md
```

Notes:
- SNIF canon uses Snask’s canonical SNIF formatter (`snif_fmt::format_snif`).
- Some formats (YAML) do not have a universal canonical form; this benchmark uses their stable Rust serialization for the same in-memory data.

