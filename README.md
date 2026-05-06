# Linguagem de Programacao Snask v0.4.1-alpha

Snask e uma linguagem compilada AOT para binarios nativos via LLVM. O objetivo atual e unir uma superficie humana para apps, CLI e DX com uma base de sistemas capaz de rodar runtimes, emuladores e interop nativa sem transformar o usuario em gerenciador manual de memoria.

> Snask deve continuar simples para comecar, mas forte o bastante para chegar perto da maquina quando voce pede.

## O que e verdade hoje

- Snask compila para LLVM IR e gera binario nativo. Nao e transpilador para C.
- O perfil padrao e `humane`, com runtime completo e diagnosticos humanos.
- O perfil `systems` existe para codigo baixo nivel, emuladores e memoria crua controlada.
- O OM-Snask-System unifica zonas, arenas, stack/heap, recursos nativos e contratos deduzidos de headers C.
- `@unsafe` existe para regioes onde o programador assume memoria manual ou chamadas restritas.
- Ha runtime nativo com IO, GUI experimental, SQLite, SNIF e funcoes de systems programming.
- O app `apps/nes_emulator` ja executa uma ROM real NROM de NES em Snask puro como laboratorio do perfil `systems`.

## Instalacao rapida

```bash
git clone https://github.com/rancidavi-dotcom/TheSnask.git
cd TheSnask
./install.sh
```

Ou direto por `curl`:

```bash
curl -fsSL https://raw.githubusercontent.com/rancidavi-dotcom/TheSnask/main/install.sh | bash
```

O instalador detecta Arch, Debian/Ubuntu, Fedora, openSUSE e Alpine, instala as dependencias quando possivel, encontra LLVM 18 e coloca o binario em `~/.snask/bin/snask`. Veja [Instalacao Linux](docs/tooling/INSTALLATION.md) para detalhes, variaveis de escape e correcao de erros do `llvm-sys`.

## Hello World atual

```snask
class main {
    fun start() {
        print("Ola, Snask!\n")
    }
}
```

Build:

```bash
./target/debug/snask build hello.snask --output hello
./hello
```

Sem `--profile`, o compilador usa `humane` por padrao.

## Perfis principais

| Perfil | Uso | Observacao |
| --- | --- | --- |
| `humane` | apps, CLI, aprendizado, runtime completo | padrao quando nenhum perfil e informado |
| `systems` | emuladores, parsers binarios, memoria crua controlada | mantem Snask, OM e diagnosticos |
| `baremetal` | kernels/embedded futuros | partes de std/runtime como `print` podem nao existir |

Perfis de tamanho continuam existindo como flags de build: `--release-size`, `--min-runtime`, `--tiny` e `--extreme`.

## Documentos importantes

- [Indice completo da documentacao](docs/INDEX.md)
- [Aprender Snask](docs/reference/LEARN_SNASK.md)
- [Referencia da Linguagem](docs/reference/LANGUAGE_REFERENCE.md)
- [Status real das features](docs/reference/FEATURE_STATUS.md)
- [OM-Snask-System](docs/systems/OM_SNASK_SYSTEM.md)
- [Diagnosticos humanos](docs/reference/HUMANE_DIAGNOSTICS.md)
- [Fundacao systems/NES](docs/systems/SNASK_NES_SYSTEMS_FOUNDATION.md)
- [Arquitetura](docs/systems/ARCHITECTURE.md)

## Benchmark e orgulho tecnico

Os documentos de performance ficam em `docs/benchmarks/` e devem ser tratados como snapshots reproduziveis, nao como promessa absoluta para toda maquina.

```bash
cargo build --release
./target/release/snask setup
./bench/run.sh
cat bench/out/report.md
```

## Estado do projeto

Snask ainda e `alpha`. A linguagem ja tem partes reais e impressionantes, mas a documentacao agora separa claramente:

- `estavel`: pode ser usado com confianca relativa;
- `parcial`: funciona, mas tem limites importantes;
- `experimental`: existe, mas a API/semantica ainda pode mudar;
- `planejada`: direcao de design, nao promessa pronta.

Use `docs/reference/FEATURE_STATUS.md` como fonte de verdade antes de depender de uma feature.

## Licenca

MIT.
