# 📘 Guia Completo do Snask (Trilha do Desenvolvedor) — v0.3.0

Este documento é uma trilha completa para você dominar o Snask: **instalação → linguagem → módulos → web → autenticação → boas práticas**.

- Tutorial rápido: `docs/APRENDA_SNASK.md`
- Referência de módulos: `docs/BIBLIOTECAS_SNASK.md`
- Projetos (SPS): `docs/SPS.md`

---

## 📑 Índice (Trilha)

1. [O que é Snask (e o que não é)](#1-o-que-é-snask-e-o-que-não-é)
2. [Ferramentas: build, run, setup](#2-ferramentas-build-run-setup)
3. [Primeiro programa](#3-primeiro-programa)
4. [Sintaxe essencial](#4-sintaxe-essencial)
5. [Tipos e valores (modelo atual)](#5-tipos-e-valores-modelo-atual)
6. [Controle de fluxo](#6-controle-de-fluxo)
7. [Funções (estilo e padrões)](#7-funções-estilo-e-padrões)
8. [POO: classes, propriedades e métodos](#8-poo-classes-propriedades-e-métodos)
9. [Módulos e bibliotecas (import e namespace)](#9-módulos-e-bibliotecas-import-e-namespace)
10. [I/O e sistema: “equivalente ao stdio.h”](#10-io-e-sistema-equivalente-ao-stdioh)
11. [JSON de verdade: parse/stringify + arquivos](#11-json-de-verdade-parsestringify--arquivos)
12. [HTTP simples: requests](#12-http-simples-requests)
13. [Web server: Blaze](#13-web-server-blaze)
14. [Autenticação: Blaze Auth](#14-autenticação-blaze-auth)
15. [Estrutura de projeto recomendada](#15-estrutura-de-projeto-recomendada)
16. [Debug e troubleshooting](#16-debug-e-troubleshooting)
17. [Limitações atuais e próximos passos](#17-limitações-atuais-e-próximos-passos)

---

## 1. O que é Snask (e o que não é)

**Snask** é uma linguagem **compilada** focada em performance, com sintaxe por **indentação** e orientação a objetos. O compilador gera binários nativos via **LLVM 18**.

O Snask **não** é:
- um interpretador (você não “executa o .snask diretamente”)
- um “C com headers” (você não inclui `stdio.h` no código Snask)

O Snask **é**:
- um compilador + um runtime nativo em C (`runtime.o`)
- um ecossistema de **módulos `.snask`** (bibliotecas)

---

## 2. Ferramentas: build, run, setup

Comandos principais do CLI:

- `snask build arquivo.snask` → compila e gera um binário `./arquivo`
- `snask run arquivo.snask` → atalho que faz **build + executa** `./arquivo`
- `snask setup` → (re)gera `~/.snask/lib/runtime.o` e instala o CLI no `PATH`

Pré-requisitos (Linux):
- Rust (para compilar o compilador)
- LLVM 18 + Clang 18 (para gerar/linkar binários)

---

## 3. Primeiro programa

Todo programa Snask precisa ter:
- `class main`
- `fun start()`

Exemplo (`hello.snask`):
```snask
class main
    fun start()
        print("Olá, Snask!");
        let x = 10;
        print("x * 5 =", x * 5);
```

Compilar e rodar:
```bash
snask build hello.snask
./hello
```

---

## 4. Sintaxe essencial

### 4.1 Indentação
Blocos são definidos por indentação (estilo Python).

### 4.2 Variáveis: `let` e `mut`
- `let` cria variável imutável
- `mut` cria variável mutável

```snask
let nome = "Davi";
mut idade = 25;
idade = idade + 1;
```

### 4.3 Operadores úteis (ergonomia)
Lógicos:
- `and`, `or`, `not`

Atribuição com açúcar:
- `+=`, `-=`, `*=`, `/=`

Exemplo:
```snask
mut x = 1;
x += 2;
if (x == 3) and not false
    print("ok");
```

### 4.4 Comentários
Use `//` para comentário de linha.

---

## 5. Tipos e valores (modelo atual)

O runtime atual trabalha com estes valores principais:
- `num` (número, representado como float internamente)
- `str` (string)
- `bool` (`true/false`)
- `nil`
- `obj` (objeto — usado para instâncias de `class` e também para objetos/arrays JSON parseados)

Checagens úteis (nativas):
- `is_nil(x)`
- `is_str(x)`
- `is_obj(x)`

---

## 6. Controle de fluxo

### 6.1 `if` / `else`
```snask
if 10 > 5
    print("maior");
else
    print("menor");
```

### 6.2 `while`
```snask
mut i = 0;
while i < 3
    print("i:", i);
    i = i + 1;
```

---

## 7. Funções (estilo e padrões)

Funções são declaradas com `fun` e podem retornar com `return`.

```snask
fun somar(a, b)
    return a + b;
```

Padrões recomendados:
- prefira funções pequenas e retornos explícitos
- evite “estado global” em módulos

---

## 8. POO: classes, propriedades e métodos

Uma `class` define propriedades (normalmente `let`) e métodos com `fun`.

```snask
class Pessoa
    let nome = "Davi";
    let idade = 25;

class main
    fun start()
        let p = Pessoa();
        print(p.nome, p.idade);
```

---

## 9. Módulos e bibliotecas (import e namespace)

Importe módulos com:
```snask
import "json";
import "os";
```

E use sempre o namespace:
```snask
let obj = json::new_object();
os::write_json_pretty("data.json", obj);
```

Exceção: `prelude` é um módulo “ergonômico” pensado para ser importado e usado sem prefixo:
```snask
import "prelude";

class main
    fun start()
        println("hello");
        assert(1 + 1 == 2, "math");
```

O compilador procura módulos:
1) localmente (`./nome.snask`)
2) em `~/.snask/packages/nome.snask`

---

## 10. I/O e sistema: “equivalente ao stdio.h”

O “equivalente ao `stdio.h`” no Snask é:

- stdout: `print(...)`
- arquivos: `sfs::*` ou `os::*`

Exemplo usando `os`:
```snask
import "os";

class main
    fun start()
        os::write_file("log.txt", "oi");
        os::append_file("log.txt", "\\nmais");
        print(os::read_file("log.txt"));
```

---

## 11. JSON de verdade: parse/stringify + arquivos

```snask
import "json";
import "os";

class main
    fun start()
        let o = json::new_object();
        json::set(o, "name", "davi");
        os::write_json_pretty("user.json", o);

        let x = os::read_json("user.json");
        print("name:", json::get(x, "name"));
```

---

## 12. HTTP simples: requests

```snask
import "requests";

class main
    fun start()
        let body = requests::get("https://example.com");
        print(body);
```

---

## 13. Web server: Blaze

O Blaze permite responder rotas de 2 formas:

### 13.1 Rotas estáticas
```snask
import "blaze";

class main
    fun start()
        let routes = blaze::new();
        blaze::get(routes, "/", blaze::resp_text("ok"));
        blaze::run(8080, routes);
```

### 13.2 Handlers (dinâmico) com query/cookie/body
Você registra um **handler por nome** e o runtime chama sua função:
```snask
import "blaze";

class main
    fun start()
        let routes = blaze::new();
        blaze::handler_get(routes, "/hello", "hello_handler");
        blaze::run(8080, routes);

fun hello_handler(method, path, query, body, cookie)
    let name = blaze::qs_get(query, "name");
    if is_nil(name)
        return blaze::bad_request();
    return blaze::resp_text("Olá " + name);
```

---

## 14. Autenticação: Blaze Auth

O `blaze_auth` fornece:
- storage local (users/sessions em JSON)
- hash/verify nativo (demo)
- response com `Set-Cookie: sid=...`

Exemplo real (pronto no repo): `blaze_auth_system.snask`

Testando com curl (exemplo):
```bash
./blaze_auth_system
curl "http://127.0.0.1:8080/register?user=alice&pass=123"
curl -i -c /tmp/cj "http://127.0.0.1:8080/login?user=alice&pass=123"
curl -i -b /tmp/cj "http://127.0.0.1:8080/me"
```

---

## 15. Estrutura de projeto recomendada

Para um app web:
```
app.snask
routes.snask
models.snask
```

E bibliotecas locais:
```
blaze_app_helpers.snask
```

---

## 16. Debug e troubleshooting

### 16.1 “Undefined reference” na linkagem
Normalmente significa que:
- o runtime (`~/.snask/lib/runtime.o`) está desatualizado, ou
- você chamou uma builtin que não existe no runtime atual.

Solução:
```bash
snask setup
```
ou recompile manualmente o runtime se estiver desenvolvendo o compilador.

### 16.2 Strings com escapes
Snask suporta escapes comuns em strings:
- `\\n`, `\\r`, `\\t`, `\\\"`, `\\\\`
- `\\uXXXX` (unicode)

Exemplo:
```snask
let s = "linha1\\nlinha2\\t\\\"q\\\"\\\\";
print(s);
```

Para JSON, ainda é válido preferir `json::new_object()` + `json::set(...)` para evitar strings gigantes.

---

## 17. Limitações atuais e próximos passos

Limitações comuns do modelo atual:
- modelo de “obj” ainda é simples (sem tipos fortes)
- JSON arrays são representados como “obj” com keys `"0..n-1"`
- handlers web ainda são minimalistas (sem roteamento avançado, sem middleware)

Próximos passos típicos:
- cookies mais completos (SameSite/Secure)
- parsing de `application/x-www-form-urlencoded` e JSON body em handlers
- hash de senha forte (bcrypt/argon2 nativo)

---

*Guia atualizado em 17 de fevereiro de 2026 (Snask v0.3.0).*
