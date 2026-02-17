# Snask

[![Versão](https://img.shields.io/badge/Versão-v0.3.0-blue.svg)](https://github.com/rancidavi-dotcom/TheSnask)
[![Compilador](https://img.shields.io/badge/Backend-LLVM%2018-orange.svg)](https://llvm.org/)
[![Construído com](https://img.shields.io/badge/Construído%20com-Rust-red.svg)](https://www.rust-lang.org/)

**Snask** é uma linguagem compilada (LLVM 18) focada em **binários nativos**, **identação obrigatória** e uma experiência simples para construir projetos e bibliotecas.

## Instalação (Linux)

Instale/atualize o SNask com um único comando:

```bash
curl -fsSL https://raw.githubusercontent.com/rancidavi-dotcom/TheSnask/main/install.sh | bash
```

## Destaques (v0.3.0)

A v0.3.0 adiciona base de banco (SQLite) no runtime, multithreading nativo (pthread), cross-compilation por alvo e GC simples para strings/buffers.

| Recurso | Descrição |
| :--- | :--- |
| **Compilação nativa** | Gera executáveis nativos via LLVM (sem interpretador). |
| **Identação obrigatória** | Blocos definidos por espaços (estilo Python). |
| **SPS (Project System)** | `snask.toml`, dependências, lockfile e build sem argumentos. |
| **Runtime nativo** | Módulos/builtins para IO, JSON/Sjson, GUI (GTK) e mais. |
| **SQLite + threads** | Integrações nativas para dados e paralelismo. |

## Documentação

Explore os guias detalhados na pasta `docs/`:

1.  **[Guia Geral](docs/GUIA_SNASK.md)**: Referência técnica completa.
2.  **[Aprenda Snask](docs/APRENDA_SNASK.md)**: Tutorial passo a passo para iniciantes.
3.  **[Bibliotecas e Módulos](docs/BIBLIOTECAS_SNASK.md)**: Como usar `requests`, `sfs` e `utils`.
4.  **[SPS (Snask Project System)](docs/SPS.md)**: Manifesto `snask.toml` + `snask build/run` sem argumentos.

## Status

- Snask é um projeto em evolução; mudanças podem ocorrer entre versões.
- Issues e contribuições são bem-vindas.

## Licença
Distribuído sob a **Licença MIT**.

---
Mantido por [rancidavi-dotcom](https://github.com/rancidavi-dotcom).
