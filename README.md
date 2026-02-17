# 🐍 Snask: A Linguagem de Alto Desempenho para Sistemas Modernos

[![Versão](https://img.shields.io/badge/Versão-v0.2.1-blue.svg)](https://github.com/Davivilasdev/Snask)
[![Compilador](https://img.shields.io/badge/Backend-LLVM%2018-orange.svg)](https://llvm.org/)
[![Construído com](https://img.shields.io/badge/Construído%20com-Rust-red.svg)](https://www.rust-lang.org/)

**Snask** é uma linguagem de programação focada em **performance extrema** e **simplicidade**. Evoluindo de um interpretador dinâmico para um **compilador nativo baseado em LLVM**, o Snask combina a facilidade de linguagens de script com a velocidade bruta do C/C++.

## 🚀 O que há de novo na v0.2.1?

A versão `v0.2.1` marca a transição definitiva para a infraestrutura **LLVM**, permitindo que o código Snask seja compilado diretamente para binários nativos de alto desempenho.

| Recurso | Descrição |
| :--- | :--- |
| **⚙️ Backend LLVM 18** | Geração de código de máquina otimizado usando a mesma tecnologia do Clang e Swift. |
| **📦 Sistema de Pacotes** | Gerenciador integrado para baixar e gerenciar bibliotecas externas via `registry.json`. |
| **🗄️ SQL & Crypto** | Suporte inicial para SQLite, BCrypt e UUIDs (em transição para o compilador). |
| **🛠️ Native Runtime** | Runtime escrito em C para garantir máxima eficiência em operações de I/O e memória. |
| **🎨 Semântica Avançada** | Analisador semântico rigoroso que previne erros antes mesmo da compilação. |

## 📦 Instalação e Build

O Snask requer o **Rust** e as ferramentas do **LLVM 18** instaladas no sistema.

### Compilando o Compilador

```bash
cargo build --release
```

### Compilando seu primeiro programa Snask

O Snask transforma arquivos `.snask` em executáveis nativos:

```bash
# Compilar para binário nativo
./target/release/snask build meu_codigo.snask

# Executar o programa resultante
./meu_codigo
```

## 📖 A Linguagem Snask

### Tipagem e Variáveis
O Snask utiliza um sistema de tipos híbrido, otimizado para o backend LLVM.

```snask
let nome = "Snask";       // Imutável
mut contador = 0;         // Mutável
const VERSAO = "0.2.1";   // Constante
```

### Funções Nativas (Built-ins)
O Snask já vem com uma biblioteca padrão potente integrada diretamente ao executável final:

*   **Matemática:** `s_abs()`, `s_max()`, `s_min()`, `sqrt()`.
*   **Strings:** `s_len()`, `s_upper()`, `s_concat()`.
*   **Sistema de Arquivos (SFS):** `sfs_read()`, `sfs_write()`, `sfs_exists()`, `sfs_delete()`.
*   **Sistema:** `s_time()`, `s_sleep()`.

### Exemplo de Código
```snask
fun saudacao(nome: str) {
    print("Olá, " + nome + "!");
}

saudacao("Mundo Snask");

if sfs_exists("config.txt") {
    let config = sfs_read("config.txt");
    print("Configuração carregada: " + config);
}
```

## 🛠️ Roadmap v0.3.0
*   **Full Web Integration:** Migração do módulo HTTP (`reqwest`/`rouille`) para o backend LLVM.
*   **Database ORM:** Integração nativa de SQLite no compilador.
*   **Garbage Collection:** Gerenciamento de memória aprimorado para strings e objetos complexos.
*   **Cross-Compilation:** Suporte para compilação cruzada (Windows/Linux/macOS).

## 📄 Documentação e Guia
Para aprender a programar em Snask, consulte o nosso **[Guia de Uso Completo](docs/GUIA_USO.md)**.

## 📄 Licença
Distribuído sob a **Licença MIT**.

---
*Desenvolvido com ⚡ por Davivilasdev*
