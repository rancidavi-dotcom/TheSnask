# 📘 Guia Definitivo da Linguagem Snask (v0.2.1)

**Bem-vindo ao Snask!** Este guia consolidado fornece tudo o que você precisa saber para dominar a linguagem, desde a instalação até o desenvolvimento de sistemas de alto desempenho utilizando o backend LLVM.

---

## 📑 Índice

1. [O que é Snask?](#1-o-que-é-snask)
2. [Configuração e Build](#2-configuração-e-build)
3. [Fundamentos da Linguagem](#3-fundamentos-da-linguagem)
4. [Estruturas de Controle](#4-estruturas-de-controle)
5. [Funções e Modularização](#5-funções-e-modularização)
6. [Biblioteca Padrão e Runtime Nativo (C)](#7-biblioteca-padrão-e-runtime-nativo-c)
7. [Arquitetura e Performance](#8-arquitetura-e-performance)

---

## 1. O que é Snask?

**Snask** é uma linguagem de programação focada em **performance extrema** e **simplicidade**. Utiliza um **compilador nativo baseado em LLVM 18**, combinando a facilidade de linguagens de script com a velocidade bruta do C/C++.

---

## 2. Configuração e Build

### Pré-requisitos
- **Rust** (compilador Snask).
- **LLVM 18** e **Clang 18** (backend de geração de código).

### Build do Compilador
```bash
cargo build --release
```

---

## 3. Fundamentos da Linguagem

### Variáveis
| Palavra-chave | Propósito | Exemplo |
| :--- | :--- | :--- |
| `let` | **Imutável** (Otimizado). | `let nome = "Davi";` |
| `mut` | **Mutável**. | `mut contador = 0;` |

### Tipos de Dados
- **Num**: Números de ponto flutuante 64-bit (IEEE 754).
- **Str**: Cadeias de caracteres seguras.
- **Bool**: `true` ou `false`.
- **Nil**: Ausência de valor.

---

## 4. Estruturas de Controle

### Condicionais
```snask
if nota >= 7.0 {
    print("Aprovado!");
} else {
    print("Reprovado.");
}
```

### Loops
```snask
mut i = 0;
while i < 5 {
    print("Contagem:", i);
    i = i + 1;
}
```

---

## 5. Funções e Modularização

### Definição
Funções utilizam a palavra-chave `fun`. Elas são isoladas em namespaces internos (`f_`) para evitar conflitos com o sistema.

```snask
fun somar(a, b) {
    return a + b;
}
print(somar(10, 5));
```

### Importação
O Snask usa injeção direta de código para módulos.
```snask
import "utils"
saudar("Davi");
```

---

## 6. Biblioteca Padrão e Runtime Nativo (C)

O Snask utiliza um runtime em C altamente otimizado para I/O e memória.

### Sistema de Arquivos (SFS)
- `sfs_read(path)`: Lê arquivos.
- `sfs_write(path, content)`: Escreve arquivos (com auto-flush).
- `sfs_exists(path)`: Verifica existência.

### Utilidades
- `s_time()`: Timestamp atual.
- `s_sleep(ms)`: Pausa a execução.
- `s_abs(n)`: Valor absoluto.

---

## 7. Arquitetura e Performance

O Snask v0.2.1 utiliza uma struct de valor alinhada em **64 bits**:
- **Tag**: 8 bytes (double)
- **Data**: 8 bytes (double)
- **Pointer**: 8 bytes (ptr)

Isso garante que a comunicação entre o LLVM e o Runtime C seja livre de erros de alinhamento e falhas de segmentação.

---
*Documentação técnica atualizada em 16 de fevereiro de 2026.*
