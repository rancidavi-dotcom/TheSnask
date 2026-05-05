# I/O throughput e pico de RAM

Benchmark de escrita de arquivo em blocos de 1 MiB, seguido de `stat()` para validar tamanho.

## Metricas

- tempo de parede;
- pico de RSS;
- throughput aproximado.

## Rodar

```bash
SIZE_MB=256 RUNS=7 ./bench/io/run.sh
cat bench/io/out/report.md
```

## Observacao

O objetivo e medir throughput de escrita em page cache. Nao e benchmark de durabilidade com `fsync`.
