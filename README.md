# 🐍 Snask: Compilador Nativo de Alto Desempenho

[![Versão](https://img.shields.io/badge/Versão-v0.2.1-blue.svg)](https://github.com/rancidavi-dotcom/TheSnask)
[![Compilador](https://img.shields.io/badge/Backend-LLVM%2018-orange.svg)](https://llvm.org/)
[![Construído com](https://img.shields.io/badge/Construído%20com-Rust-red.svg)](https://www.rust-lang.org/)

**Snask** é uma linguagem de programação focada em **velocidade bruta** e **sintaxe intuitiva**. Através de um compilador baseado em **LLVM 18**, o Snask gera binários nativos otimizados, eliminando o overhead de interpretação e garantindo performance de nível C/C++.

## 🚀 Destaques da v0.2.1

A versão atual marca a estabilidade do ecossistema Snask, unindo o poder do Rust no frontend com a eficiência do LLVM no backend.

| Recurso | Descrição |
| :--- | :--- |
| **⚙️ LLVM Backend** | Geração de IR otimizado e linkagem nativa via Clang. |
| **📦 Namespaces** | Módulos organizados com sintaxe `modulo::funcao()`. |
| **🌐 Full Web** | Biblioteca `requests` nativa para GET, POST, PUT, DELETE e PATCH. |
| **📂 SFS (File System)** | Manipulação de arquivos veloz integrada ao Runtime C. |
| **🛡️ Memory Safe** | Structs alinhadas em 64-bit para comunicação estável entre LLVM e C. |

## 📦 Instalação e Build

### Pré-requisitos
- **Rust** (Cargo)
- **LLVM 18** e **Clang 18** (Disponíveis via `apt install llvm-18 clang-18` no Ubuntu/Pop!_OS)

### Compilando o Compilador Snask
```bash
git clone https://github.com/rancidavi-dotcom/TheSnask
cd TheSnask
cargo build --release
```

## 🛠️ Começando

O Snask compila arquivos `.snask` diretamente para executáveis do sistema.

### Seu primeiro programa (`hello.snask`)
```snask
import "requests"

print("Iniciando Snask...");
let res = requests::get("https://google.com");
print("Tamanho da página:", s_len(res));
```

### Compilar e Rodar
```bash
./target/release/snask build hello.snask
./hello
```

## 📚 Documentação Oficial

Explore os guias detalhados na pasta `docs/`:

1.  **[Guia Geral](docs/GUIA_SNASK.md)**: Referência técnica completa.
2.  **[Aprenda Snask](docs/APRENDA_SNASK.md)**: Tutorial passo a passo para iniciantes.
3.  **[Bibliotecas e Módulos](docs/BIBLIOTECAS_SNASK.md)**: Como usar `requests`, `sfs` e `utils`.

## 🗺️ Roadmap v0.3.0
- **SQLite ORM**: Integração nativa de banco de dados no compilador.
- **Multithreading**: Suporte a execução paralela nativa.
- **Cross-Compilation**: Build fácil para Windows e macOS a partir do Linux.
- **Garbage Collection**: Gerenciamento automático de memória para strings dinâmicas.

## 📄 Licença
Distribuído sob a **Licença MIT**.

---
*Mantido com ⚡ por [rancidavi-dotcom](https://github.com/rancidavi-dotcom)*
