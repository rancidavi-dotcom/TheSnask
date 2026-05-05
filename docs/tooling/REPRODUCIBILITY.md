# Guia de Reprodutibilidade v0.4.1-alpha

Este guia explica como reproduzir os benchmarks e evitar conclusoes baseadas em numeros soltos.

## Preparacao

Requisitos:

- Rust stable;
- LLVM compatvel com o projeto;
- GCC ou Clang para linkagem;
- Python 3 para alguns benchmarks auxiliares.

```bash
rustup update stable
cargo build --release
./target/release/snask setup
```

## Rodar todos os benchmarks principais

```bash
./bench/run.sh
cat bench/out/report.md
```

## Rodar casos especificos

```bash
./bench/startup/run.sh
cat bench/startup/out/report.md
```

```bash
SIZE_MB=256 RUNS=7 ./bench/io/run.sh
cat bench/io/out/report.md
```

```bash
N_FILES=100000 RUNS=5 ./bench/fs_small/run.sh
cat bench/fs_small/out/report.md
```

```bash
./bench/format_snif/run.sh
cat bench/format_snif/out/report.md
```

## Interpretacao

- Compare rodadas no mesmo ambiente.
- Startup e filesystem sofrem muito com ruido do sistema.
- RAM varia com distro, tema GTK, versao de libs e desktop.
- Use os reports gerados como fonte de verdade.
