# Benchmarks do Snask

Esta pasta contem benchmarks reproduziveis usados para comparar tamanho, startup, I/O e formatos de dados.

## Casos

- `cli_hello`: imprime uma linha e sai.
- `cli_io`: escreve/le um arquivo pequeno e valida tamanho.
- `cli_full`: CLI um pouco mais realista com subcomandos e parse SNIF.
- `format_snif`: SNIF contra JSON/TOML/YAML/CBOR/MsgPack.
- `startup`: tempo de partida a frio.
- `fs_small`: muitos arquivos pequenos.

## Rodar

```bash
cargo build --release
./target/release/snask setup
./bench/run.sh
```

Saidas principais:

- `bench/out/report.md`
- `bench/*/out/report.md`
- binarios em `bench/out/`

## Regra de honestidade

Os benchmarks sao snapshots reproduziveis. Resultado de filesystem, RAM e startup varia com kernel, desktop, disco, CPU e cache.
