# Snask Humane Diagnostics

Sistema de diagnosticos humanos do Snask para erros de parser, semantica e toolchain.

## Objetivo

- erro curto por padrao;
- codigo publico estavel;
- linha e coluna exatas;
- snippet com caret;
- ajuda acionavel;
- explicacao longa via `snask explain`;
- nada de dumps internos Rust na experiencia normal.

## Formato

```text
error[S1002]: missing closing `)`
 --> hello.snask:3:22
  |
3 |         print("Hello"
  |                      ^ expected `)` here
note: unclosed '(' opened at 3:14
help: You probably missed a closing ')'.
```

As mensagens ainda podem estar em ingles em alguns pontos do compilador, mas o formato publico ja e o alvo correto.

## Faixas de codigo

| Faixa | Area |
| --- | --- |
| `S1000`-`S1999` | parser e sintaxe |
| `S2000`-`S2999` | semantica e tipos |
| `S9000`-`S9999` | perfis, build e politica de toolchain |

## Exemplos

Erro de parenteses:

```text
class main {
    fun start() {
        print("Hello"
    }
}
```

Erro de tipo:

```text
class main {
    fun start() {
        let age: int = "18"
    }
}
```

Erro de nome:

```text
class main {
    fun start() {
        let message = "Hello"
        print(mesage)
    }
}
```

## `snask explain`

```bash
snask explain S1002
snask explain S2002
```

## Arquivos principais

- `src/diagnostics.rs`
- `src/compiler.rs`
- `src/semantic_analyzer.rs`
- `src/explain.rs`
- `src/main.rs`

## Testes uteis

```bash
cargo test humane -- --nocapture
cargo test explain -- --nocapture
cargo check
```

## Roadmap

- traduzir mensagens restantes;
- JSON diagnostics para LSP;
- spans secundarios melhores;
- fix-its aplicaveis;
- codigos publicos para OM e linker.
