# Snask Low-Level Plan

## Identidade

Snask deve ser uma linguagem de sistemas com superficie humana por padrao.

```text
Low-level core.
Humane surface.
OM by default.
Unsafe by choice.
```

O objetivo nao e remover o que ja existe. O objetivo e fazer a fundacao da linguagem ser forte o bastante para apps, engines, emuladores, runtimes e kernels, mantendo a experiencia inicial amigavel.

## Tres perfis oficiais

### `humane`

Perfil padrao.

Para:

- iniciantes;
- apps;
- CLI tools;
- GUI futura;
- uso normal de bibliotecas;
- OM-Snask-System completo;
- diagnosticos humanos.

### `systems`

Perfil de baixo nivel com runtime ainda disponivel.

Para:

- emuladores de CPU;
- engines;
- parsers binarios;
- runtimes;
- bancos de dados;
- bibliotecas nativas;
- interop C mais direta.

Este perfil deve adicionar tipos de maquina, layout explicito, ponteiros sob `@unsafe`, intrinsecos e controle de ABI, sem abandonar OM.

### `baremetal`

Perfil freestanding.

Para:

- kernels;
- bootloaders;
- embedded;
- codigo sem libc obrigatoria;
- runtime minimo ou ausente;
- linker script e entrypoint customizado.

Neste perfil, `print`, filesystem, GUI, SDL2 e outras partes de std/runtime nao existem por padrao.

## Regra principal

```text
O compilador entende maquina.
A superficie continua humana.
```

Toda feature high-level deve baixar para uma base low-level clara. Todo acesso perigoso deve passar por `@unsafe`.

## Ordem de implementacao

1. Reconhecer perfis oficiais no toolchain: `humane`, `systems`, `baremetal`.
2. Preservar perfis existentes de build/tamanho: `dev`, `release`, `release-size`, `tiny`, `extreme`.
3. Adicionar tipos nativos: `u8`, `u16`, `u32`, `u64`, `i8`, `i16`, `i32`, `i64`, `usize`, `isize`, `f32`, `f64`, `bool`, `never`.
4. Garantir lowering LLVM sem boxing para tipos nativos conhecidos.
5. Adicionar operadores de sistemas: bitwise, shifts, overflow explicito.
6. Adicionar `ptr<T>`, `rawptr`, `null`, `addr_of`.
7. Adicionar `raw.alloc`, `raw.free`, `ptr.read`, `ptr.write`, `ptr.offset`, `ptr.cast` somente dentro de `@unsafe`.
8. Adicionar `struct repr(C)`, `repr(packed)`, `align(N)`.
9. Adicionar `sizeof<T>()`, `alignof<T>()`, `offsetof<T>("field")`.
10. Adicionar arrays fixos `[T; N]` e slices `slice<T>`.
11. Adicionar `extern "C" fun` e calling conventions.
12. Evoluir Auto-OM para structs, enums, bitflags, out parameters, ponteiro duplo e callbacks simples.
13. Dividir runtime em camadas: core, OM, std, GUI.
14. Adicionar `no_std`, `no_runtime`, `entry`, target freestanding e linker script.
15. Adicionar intrinsecos: volatile, atomics, memcpy/memset, portas x86, halt, barriers.
16. Criar demos oficiais: emulador 6502 e kernel hello world.

## Filosofia de seguranca

Snask nao impede tocar a maquina. Ele exige que isso seja explicito.

```snask
@unsafe:
    let p = raw.alloc(64)
    ptr.write<u8>(p, 255)
    raw.free(p)
```

Fora de `@unsafe`, ponteiros crus, memoria manual e chamadas nativas restritas devem gerar diagnosticos humanos.

## Resultado esperado

Snask deve conseguir escrever:

- app com GUI;
- jogo com SDL2;
- emulador de CPU;
- biblioteca nativa;
- runtime proprio;
- kernel experimental.

Sempre com:

```text
Snask puro.
Compilado via LLVM.
OM-Snask-System por padrao.
@unsafe quando necessario.
Diagnosticos humanos sempre.
```
