# Runtime Nativo do Snask v0.4.1-alpha

O runtime nativo e a camada de suporte chamada pelo LLVM gerado pelo compilador. Ele nao transforma Snask em C; ele fornece funcoes de baixo nivel para strings, IO, memoria, GUI experimental, SQLite, OM e perfil `systems`.

## Representacao de valores

Historicamente o runtime usa `SnaskValue` para partes dinamicas. O compilador atual tambem baixa tipos nativos conhecidos diretamente para inteiros, floats e ponteiros LLVM quando possivel.

```c
typedef struct {
    double tag;
    double num;
    void* ptr;
} SnaskValue;
```

## Convencao interna

Funcoes antigas do runtime seguem padrao com ponteiro de saida:

```text
s_func(out, arg1, arg2, ...)
```

Funcoes novas de systems programming podem operar diretamente com tipos nativos quando o codegen conhece a assinatura.

## Modulos principais

### Objetos e colecoes

- `s_alloc_obj`
- `s_get_member`
- `s_set_member`
- `s_len`
- `s_is_nil`

### OM-Snask-System

- registro de zonas;
- arenas;
- recursos nativos;
- cleanup deterministico;
- extracao interna de ponteiro de recurso para chamadas ABI.

### IO e tempo

- `s_print`
- `s_println`
- `s_time`
- concatenacao/string helpers.

Em perfil `baremetal`, partes como `print` devem ser bloqueadas quando nao houver std runtime.

### SQLite, HTTP e GUI

Existem como runtime experimental/parcial. Nao devem ser tratados como API final da linguagem.

### Systems/NES

O runtime tambem expoe memoria crua e helpers de inteiros para `--profile systems`, como `mem_alloc_zero`, `mem_read_u8`, `mem_write_u8`, conversoes `as_u8`/`as_u16` e helpers de overflow/flags.

## Relacao com OM

Destrutores e ponteiros crus nao devem virar API publica segura. O codegen chama funcoes internas para registrar e limpar recursos; o usuario escreve Snask.

## Status

Parcial/experimental. Este documento descreve a arquitetura atual e deve acompanhar mudancas em `src/runtime/` e `src/llvm_generator.rs`.
