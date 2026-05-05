# Benchmark de muitos arquivos pequenos

Mede criar, listar e apagar muitos arquivos pequenos.

## Workload

- criar `N_FILES` arquivos de 1 KiB;
- contar entradas no diretorio;
- apagar tudo.

## Rodar

```bash
N_FILES=100000 RUNS=5 ./bench/fs_small/run.sh
cat bench/fs_small/out/report.md
```

## Metrica

- ops/s;
- tempo de parede;
- pico de RSS.

## Nota

Filesystem e page cache influenciam muito. Compare apenas rodadas feitas no mesmo ambiente.
