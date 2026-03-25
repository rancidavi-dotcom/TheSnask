# SNIF vs JSON/TOML/YAML/CBOR/MsgPack (config, 100MB) — parse + canon

This benchmark compares **SNIF** against common data formats using the same Rust harness:
- JSON (text)
- TOML (text)
- YAML (text)
- CBOR (binary)
- MessagePack (binary)

We measure:
- parse time
- canonical serialize time (“canon”)
- total time
- peak RSS (best-effort via `getrusage`)

## Run
From repo root:

```bash
cargo build --release
./target/release/snif-dataset-gen --target-mb 100 --out-dir bench/format_snif/out
./target/release/snif-format-bench --dir bench/format_snif/out --runs 7
cat bench/format_snif/out/report.md
```

Or use the helper:

```bash
./bench/format_snif/run.sh
```

