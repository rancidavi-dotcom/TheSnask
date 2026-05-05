# Pride: provas e vitorias do Snask

Esta pasta reune benchmarks e argumentos tecnicos que mostram onde Snask ja tem resultados interessantes.

## Indice

- `docs/benchmarks/pride/BENCHMARKS.md`: tamanho de binario contra C convencional.
- `docs/benchmarks/pride/RAM_BENCHMARKS.md`: RAM/PSS de GUI nativa contra Electron minimo.
- `docs/benchmarks/pride/STARTUP_BENCHMARKS.md`: startup a frio.
- `docs/benchmarks/pride/SNIF_BENCHMARKS.md`: SNIF contra formatos comuns.
- `docs/benchmarks/pride/IO_BENCHMARKS.md`: I/O e pico de RAM.
- `docs/benchmarks/pride/FS_SMALL_BENCHMARKS.md`: muitos arquivos pequenos.
- `docs/benchmarks/pride/WHY_SNASK_IS_FAST_AND_SMALL.md`: explicacao tecnica dos binarios pequenos.

## Reproduzir

```bash
cargo build --release
./target/release/snask setup
./bench/run.sh
cat bench/out/report.md
```

## Regra

Numeros aqui sao evidencias, nao slogans eternos. Se o ambiente muda, rode de novo.
