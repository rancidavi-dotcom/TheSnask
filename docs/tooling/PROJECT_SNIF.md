# Configuracao de Projeto: snask.snif

`snask.snif` e o manifesto do SPS, o Snask Project System. Ele descreve metadados, arquivo principal, dependencias e scripts do projeto.

## Manifesto minimo

```snif
[project]
name = "hello_snask"
version = "0.4.1-alpha"
main = "src/main.snask"
```

## Dependencias

```snif
[dependencies]
logger = "1.0.0"
```

## Build

```snif
[build]
profile = "humane"
target = "native"
strip = false
```

## Scripts

Use comandos que existem no CLI atual:

```snif
[scripts]
test = "snask build tests/test_all.snask --output tests/test_all && ./tests/test_all"
bench = "snask build bench.snask --profile systems --output bench_app && ./bench_app"
```

## Comandos relacionados

```bash
snask init meu_app
snask build
snask run
snask add pacote
snask remove pacote
snask snif fmt snask.snif
```

## Estado

SPS esta `parcial`: ja e util para projetos simples, mas workspaces, resolucao avancada e estabilidade de registry ainda precisam amadurecer.
