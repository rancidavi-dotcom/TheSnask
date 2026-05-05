# Benchmark SNIF vs JSON/TOML/YAML/CBOR/MsgPack

Compara SNIF como formato de dados/configuracao contra formatos comuns usando o mesmo harness Rust.

## Mede

- tempo de parse;
- tempo de serializacao canonica;
- tempo total;
- pico de RSS quando disponivel.

## Rodar

```bash
cargo build --release
./target/release/snif-dataset-gen --target-mb 100 --out-dir bench/format_snif/out
./target/release/snif-format-bench --dir bench/format_snif/out --runs 7
cat bench/format_snif/out/report.md
```

Ou:

```bash
./bench/format_snif/run.sh
```
