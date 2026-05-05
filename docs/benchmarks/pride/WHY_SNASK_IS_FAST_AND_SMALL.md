# Por que Snask pode ser pequeno e rapido

Snask nao tenta ganhar tamanho por abandonar a linguagem. Ele usa uma pipeline controlada.

## 1. LLVM e linker consciente de tamanho

O compilador sabe quais partes do runtime foram usadas e passa flags para permitir remocao de codigo morto.

## 2. Fatias de runtime

Perfis pequenos evitam puxar GUI, SQLite, HTTP e partes pesadas quando o programa e apenas CLI.

## 3. Build honesto

Se o programa usa GUI ou SQLite, o binario cresce. Snask nao esconde dependencia pesada.

## 4. Comparacao justa

Os benchmarks comparam contra C convencional, nao contra C manualmente tunado com todos os truques de tamanho.

## 5. Reproducao

Veja:

- `docs/benchmarks/pride/BENCHMARKS.md`
- `docs/benchmarks/bench/README_BENCH.md`
- `docs/tooling/REPRODUCIBILITY.md`
