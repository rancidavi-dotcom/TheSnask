# Benchmark de startup a frio

Objetivo: mostrar que CLIs Snask podem iniciar com latencia proxima de C quando usam perfis enxutos.

## Mede

- TTFB: tempo ate primeiro byte em stdout.
- tempo total de parede.

## Rodar

```bash
./bench/startup/run.sh
cat bench/startup/out/report.md
```

## Nota

Resultados de milissegundos dependem de scheduler, turbo boost, cache e carga do sistema. Use muitas repeticoes antes de tirar conclusoes.
