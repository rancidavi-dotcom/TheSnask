# Benchmarks SNIF

Objetivo: medir SNIF como formato de dados/configuracao, independente da linguagem Snask.

## Compara

- JSON
- TOML
- YAML
- CBOR
- MessagePack
- SNIF

## Mede

- parse;
- serializacao canonica;
- pico de RSS quando disponivel.

## Rodar

```bash
./bench/format_snif/run.sh
cat bench/format_snif/out/report.md
```
