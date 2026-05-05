# Benchmarks de tamanho: Snask vs C convencional

Este benchmark compara executaveis dinamicamente linkados. C convencional aqui significa:

```bash
gcc file.c -o app
```

sem `-Os`, sem `-s`, sem LTO e sem script de linker.

## Variantes Snask

- `--tiny`
- `--release-size`
- `--extreme` quando aplicavel no ambiente

## Rodar

```bash
cargo build --release
./target/release/snask setup
./bench/run.sh
cat bench/out/report.md
```

## Resultado atual

A fonte de verdade e sempre `bench/out/report.md`. Este arquivo explica o que esta sendo medido e evita fixar numeros que podem envelhecer.

## Por que importa

Snask tenta entregar linguagem mais humana com binarios nativos pequenos. O ganho vem de perfis oficiais, fatias de runtime e linkagem com remocao de codigo morto, nao de truque manual por app.
