# Benchmarks de I/O

Compara Snask contra C/Go/Node/Python em um workload simples de escrita e leitura/validacao de tamanho.

## Fonte de verdade

```text
bench/io/out/report.md
```

## Rodar

```bash
SIZE_MB=256 RUNS=7 ./bench/io/run.sh
cat bench/io/out/report.md
```

## Nota

A superficie atual de I/O do Snask ainda e parcial. Melhorias futuras em streaming podem mudar os resultados.
