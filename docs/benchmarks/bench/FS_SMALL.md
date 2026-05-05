# Muitos arquivos pequenos: Snask vs C/Go/Node/Python

Workload:

1. Criar `N` arquivos pequenos de 1 KiB em um diretorio limpo.
2. Listar entradas do diretorio.
3. Apagar os arquivos.

## Metricas

- tempo de parede;
- ops/s por fase e total;
- pico de RSS via `/usr/bin/time -v`.

## Rodar

```bash
N_FILES=100000 RUNS=5 ./bench/fs_small/run.sh
cat bench/fs_small/out/report.md
```

## Observacoes

Este benchmark depende muito de disco, filesystem, mount options e page cache. O runner compila Snask com perfil pequeno para evitar puxar subsistemas desnecessarios.
