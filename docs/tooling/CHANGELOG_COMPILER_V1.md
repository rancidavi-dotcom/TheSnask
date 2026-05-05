# Changelog do Compilador Snask

Este arquivo registra mudancas grandes de arquitetura. Para status detalhado, use `docs/reference/FEATURE_STATUS.md`.

## v0.4.1-alpha

- Perfil padrao `humane` quando nenhum `--profile` e informado.
- Perfil `systems` usado como base para memoria crua e emulador NES.
- OM e OM-Snask-System consolidados como um unico sistema nos documentos.
- `@unsafe` documentado como fronteira explicita para memoria manual/chamadas restritas.
- Diagnosticos humanos com codigos e `snask explain`.
- Fundacoes low-level: inteiros de largura fixa, helpers de bits/flags, memoria crua e operacoes deterministicas.
- `apps/nes_emulator` usa Snask puro como laboratorio real de CPU/PPU/input.

## v0.3.x

- SPS MVP: manifesto, deps e lockfile deterministico.
- Runtime: SQLite, threading, GUI GTK3 experimental.
- SNIF: parser, formatter e benchmarks.
- LSP inicial.
