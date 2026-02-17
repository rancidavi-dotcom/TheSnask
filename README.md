# 🐍 Snask: Compilador Nativo de Alto Desempenho

[![Versão](https://img.shields.io/badge/Versão-v0.2.2-blue.svg)](https://github.com/rancidavi-dotcom/TheSnask)
[![Compilador](https://img.shields.io/badge/Backend-LLVM%2018-orange.svg)](https://llvm.org/)
[![Construído com](https://img.shields.io/badge/Construído%20com-Rust-red.svg)](https://www.rust-lang.org/)

**Snask** é uma linguagem de programação focada em **velocidade bruta**, **identação obrigatória** e **POO Nativa**. Através de um compilador baseado em **LLVM 18**, o Snask gera binários nativos otimizados, eliminando o overhead de interpretação.

## 🚀 Instalação Rápida (Linux)

Instale o SNask v0.2.2 com um único comando:

```bash
curl -fsSL https://raw.githubusercontent.com/rancidavi-dotcom/TheSnask/main/install.sh | bash
```

## 🛠️ Destaques da v0.2.2

A versão 0.2.2 traz o suporte real a Objetos e uma estrutura de código moderna e organizada.

| Recurso | Descrição |
| :--- | :--- |
| **🧬 POO Real** | Classes, métodos com `self` e instanciação dinâmica. |
| **📐 Identação** | Blocos de código definidos por espaços (estilo Python). |
| **⚙️ Pure Compiled** | Motor 100% LLVM, sem overhead de interpretador. |
| **🚀 Auto-Update** | Comando `snask update` para manter a linguagem atualizada. |
| **📂 Global PATH** | Instalador configura o sistema para uso global. |

## 📦 Começando

Todo programa Snask deve ter uma `class main` com um método `start()`.

### Seu primeiro programa (`hello.snask`)
```snask
class main
    fun start()
        print("Olá, Snask v0.2.2!");
        let x = 10;
        print("Resultado:", x * 5);
```

### Compilar e Rodar
```bash
snask run hello.snask
```

## 📚 Documentação Oficial

Explore os guias detalhados na pasta `docs/`:

1.  **[Guia Geral](docs/GUIA_SNASK.md)**: Referência técnica completa.
2.  **[Aprenda Snask](docs/APRENDA_SNASK.md)**: Tutorial passo a passo para iniciantes.
3.  **[Bibliotecas e Módulos](docs/BIBLIOTECAS_SNASK.md)**: Como usar `requests`, `sfs` e `utils`.
4.  **[SPS (Snask Project System)](docs/SPS.md)**: Manifesto `snask.toml` + `snask build/run` sem argumentos.

## 🗺️ Roadmap v0.3.0
- **SQLite ORM**: Integração nativa de banco de dados no compilador.
- **Multithreading**: Suporte a execução paralela nativa.
- **Cross-Compilation**: Build fácil para Windows e macOS a partir do Linux.
- **Garbage Collection**: Gerenciamento automático de memória para strings dinâmicas.

## 📄 Licença
Distribuído sob a **Licença MIT**.

---
*Mantido com ⚡ por [rancidavi-dotcom](https://github.com/rancidavi-dotcom)*
