# 🛠️ Snask Compiler: Evolução para Arquitetura AOT (v1.0.0-alpha)

Este documento detalha a transição técnica do Snask de um interpretador puramente baseado em AST para um **Compilador Antecipada (Ahead-of-Time)** utilizando transpilação para C (Samurai Path).

## 🚀 Resumo da Mudança
O Snask agora é capaz de gerar binários nativos de alto desempenho. Ao invés de avaliar a árvore de sintaxe (AST) em tempo de execução, o compilador traduz o código Snask para código C intermediário e utiliza o `gcc` (GNU Compiler Collection) para gerar o executável final.

## 🏗️ Nova Arquitetura do Compilador

### 1. `c_generator.rs` (O Coração do Compilador)
Foi implementado um novo módulo de backend que percorre a AST e emite código C equivalente.
- **Runtime Intrínseco:** O gerador injeta um "Micro-Runtime" em C que define a estrutura `SnaskValue` e a `SnaskType` (enum), permitindo que o C entenda a natureza dinâmica do Snask.
- **Memory Safety (Base):** Utiliza-se de tipos primitivos de C e strings literais para garantir overhead mínimo.
- **Dynamic Dispatch Simulado:** O `print_value` em C funciona como um dispatcher para os tipos internos, garantindo que `print(x)` funcione independente do tipo da variável.

### 2. Fluxo de Compilação (`snask build`)
O comando `build` introduz o seguinte pipeline:
1. **Frontend (Parser):** Transforma o código `.snask` em AST.
2. **Analysis (Semantic):** Valida tipos, escopo e constantes antes da geração.
3. **Backend (C Generator):** Traduz a AST validada em um arquivo `temp_snask.c`.
4. **Linkagem (GCC):** O compilador invoca o GCC: `gcc temp_snask.c -o <output>`.
5. **Cleanup:** Remove artefatos temporários, deixando apenas o binário nativo.

## 📊 Ganhos de Performance
- **Execução:** Praticamente idêntica ao código C escrito à mão.
- **Startup:** Zero latência de interpretação. O binário é carregado diretamente pelo SO.
- **Portabilidade:** O código gerado é C padrão (C99), permitindo compilação em diversas arquiteturas.

## 🛠️ Comandos Adicionados

| Comando | Descrição |
| :--- | :--- |
| `snask build <file>.snask` | Gera o executável nativo. |
| `snask build <file>.snask -o <name>` | Gera o executável com nome customizado. |

## 🧩 Suporte Atual no Backend C
- [x] Declaração de Variáveis (`let`, `mut`)
- [x] Atribuições Dinâmicas
- [x] Operações Aritméticas e Comparação
- [x] Estruturas Condicionais (`if`, `elif`, `else`)
- [x] Loops de Controle (`while`)
- [x] Impressão de Valores (`print`)

## 🗺️ Próximos Passos (Roadmap)
1. **Function Mapping:** Mapear `fun` do Snask para funções reais em C.
2. **Heap Management:** Implementar Garbage Collection ou Reference Counting para listas e dicionários dinâmicos em C.
3. **Stdlib Linkage:** Vincular as 70+ funções da Stdlib (em Rust) como funções externas no binário C.

---

# 🚀 Snask Compiler: Evolução para Backend LLVM (v0.2.1)

Recentemente, o Snask ultrapassou a barreira da transpilação para C e adotou o **LLVM (Low Level Virtual Machine)** como seu backend principal. Esta mudança coloca o Snask no mesmo patamar de linguagens modernas como Rust, Swift e Clang.

## 🏗️ Nova Arquitetura LLVM

### 1. `llvm_generator.rs`
Substituindo a emissão de texto C por geração direta de **LLVM IR (Intermediate Representation)** utilizando a biblioteca `inkwell`.
- **Tipagem Forte no Backend:** Utilização de structs LLVM para representar o `SnaskValue` de forma nativa.
- **Otimizações de Compilação:** O código agora passa por todas as passagens de otimização do LLVM (O2, O3).

### 2. Integração com `runtime.o`
Ao invés de emitir todo o código em um arquivo, o compilador agora vincula (linka) o código gerado com um arquivo de runtime pré-compilado em C (`runtime.o`), resultando em binários menores e mais rápidos.

### 3. Pipeline de Build Atualizado
1. **Frontend:** Parser Rust.
2. **IR Gen:** `llvm_generator.rs` gera o código `.ll`.
3. **Assemble:** `llc-18` transforma `.ll` em `.o` (object file).
4. **Link:** `clang-18` realiza a linkagem final com o runtime e bibliotecas do sistema.

---
*Documentação atualizada em 16 de fevereiro de 2026.*
