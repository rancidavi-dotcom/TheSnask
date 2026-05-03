# Snask Feature Status

Este documento registra o status real das features da linguagem com base no compilador, analisador semântico, codegen, runtime e testes atuais do repositório.

Objetivo:
- Separar o que já existe de verdade do que ainda é parcial ou planejado.
- Evitar que documentação e marketing ultrapassem a implementação.
- Dar base para roadmap, releases e migrações.

Legenda:
- `estavel`: existe, funciona de ponta a ponta e pode ser usado normalmente.
- `parcial`: existe, mas ainda tem limitações relevantes ou semântica incompleta.
- `experimental`: existe, mas ainda não deve ser tratado como compromisso forte da linguagem.
- `planejada`: citada no projeto, mas ainda não é uma feature consolidada no compilador.

## Core da Linguagem

| Feature | Status | Observações |
| --- | --- | --- |
| `let`, `mut`, `const` | `estavel` | Parsing, semântica básica e codegen presentes. |
| Funções | `parcial` | Parâmetros, retorno e chamadas funcionam; o analisador agora valida tipos desconhecidos, chamadas variádicas básicas e missing return em funções tipadas, mas ainda há bastante `Any`. |
| Classes | `parcial` | Existem no parser e codegen, mas o type system ainda não modela instâncias de forma forte. |
| Herança | `parcial` | Funciona via expansão de membros e agora também tem resolução semântica básica de membros herdados. |
| `if` / `elif` / `else` | `estavel` | Parsing, semântica e codegen presentes. |
| `while` | `estavel` | Parsing, semântica e codegen presentes. |
| `for in` | `estavel` | Lowering LLVM implementado e validado em execução real para listas, dicionários e strings. |
| `return` | `estavel` | Validado semânticamente. |
| Listas e dicionários | `parcial` | O analisador infere tipo homogêneo básico de elementos/chaves/valores e aceita anotações públicas `list<T>` / `dict<K, V>` para coleções. |
| Acesso por índice | `parcial` | O analisador propaga tipo de elemento/valor em listas e dicionários homogêneos, incluindo coleções anotadas com `list<T>` / `dict<K, V>`. |
| Acesso por propriedade | `parcial` | Instâncias nominais já resolvem propriedades e métodos básicos, incluindo herança simples. |
| Operadores aritméticos | `estavel` | Com coerção numérica básica. |
| Igualdade / comparação | `parcial` | Existe, mas ainda precisa semântica mais formal. |
| Strings interpoladas | `estavel` | Suportadas no parser. |
| Blocos por indentação | `estavel` | Base da sintaxe atual. |
| Blocos com `{}` | `estavel` | Suportados junto com indentação. |
| `;` opcional por newline | `estavel` | O parser aceita newline como terminador. |

## Type System

| Feature | Status | Observações |
| --- | --- | --- |
| Tipos primitivos (`int`, `float`, `str`, `bool`) | `estavel` | Funcionam no parser e na semântica básica. |
| `list` e `dict` como tipos | `parcial` | Existem nas formas abertas `list` / `dict` e parametrizadas `list<T>` / `dict<K, V>`. |
| Tipos exatos de sistema (`u8`, `i32`, `i64`, `ptr`) | `experimental` | Existem no enum de tipos, ainda sem modelo completo de linguagem. |
| Tipos de função | `parcial` | Existem internamente, mas sem ergonomia e cobertura amplas. |
| Inferência local | `parcial` | Melhorou para classes, coleções homogêneas e alguns fluxos de indexação/iteração, mas ainda cai em `Any` em várias superfícies. |
| Tipos nominais de instância | `parcial` | `new Class(...)` agora produz `Type::User(Class)` no analisador semântico, mas o type system ainda não cobre tudo. |
| Generics | `parcial` | Existem apenas para coleções (`list<T>` e `dict<K, V>`); ainda não há generics definidos por usuário, constraints ou inferência avançada. |
| `enum` / ADTs | `planejada` | Ainda não existem no AST principal. |
| Traits / interfaces | `planejada` | Ainda não existem. |
| `Option` / `Result` | `planejada` | Ainda não existem como tipos da linguagem. |

## Orchestrated Memory

| Feature | Status | Observações |
| --- | --- | --- |
| `new stack` / `new heap` / `new arena` | `experimental` | A sintaxe existe e participa do codegen; semanticamente `new Class(...)` já produz tipo nominal básico. |
| `promote` | `experimental` | A sintaxe existe, mas a checagem semântica ainda é muito rasa. |
| `zone` | `experimental` | Existe no parser e no AST, mas ainda não possui formalismo estático completo. |
| `scope` | `experimental` | Hoje funciona mais como delimitação sintática do que como contrato forte de memória. |
| `entangle` | `experimental` | Existe sintaticamente, mas ainda não está validado de forma séria. |
| OM-Snask-System / Auto-OM para C | `experimental` | Já deduz contratos a partir de headers C, emite chamadas nativas LLVM, registra recursos opacos em zonas e aceita `.om.snif` como patch. Validado com stdio.h e SDL2 simples. Ver `docs/OM_SNASK_SYSTEM.md`. |
| Escape analysis | `planejada` | Ainda não está consolidada no analisador semântico. |
| Borrow checking | `planejada` | Ainda não existe de forma real no compilador. |
| Zone depth formal | `planejada` | Ainda não existe como regra completa e testada. |

## Modulos, Build e Packages

| Feature | Status | Observações |
| --- | --- | --- |
| `import "mod"` | `estavel` | Resolução básica funciona. |
| `from ... import ...` | `parcial` | Existe, mas ainda com semântica simples de arquivo. |
| SPS / `snask.snif` | `parcial` | Já existe e é útil, mas ainda precisa amadurecer como build system completo. |
| Lockfile | `parcial` | Existe, mas ainda precisa endurecer reprodutibilidade e semântica de resolução. |
| Packages globais | `parcial` | Funcionam, mas o sistema de pacotes ainda é inicial. |
| Workspaces multi-package | `planejada` | Ainda não é parte madura da toolchain. |

## Tooling

| Feature | Status | Observações |
| --- | --- | --- |
| Diagnósticos do parser | `parcial` | Já têm códigos e spans, mas ainda precisam cobertura maior. |
| Snask Humane Diagnostics | `experimental` | Erros do parser/semântica agora têm códigos públicos curtos, snippet com caret, help/notes e `snask explain`. Ver `docs/HUMANE_DIAGNOSTICS.md`. |
| Diagnósticos semânticos | `parcial` | Existem, agora renderizados de forma mais humana, mas ainda dependem demais de `Any` e precisam mais sugestões contextuais. |
| LSP | `parcial` | Completion, hover, goto definition, code actions e semantic tokens já existem. |
| Formatter SNIF | `estavel` | Existe e já possui testes próprios. |
| Formatter da linguagem principal | `planejada` | Ainda não existe como ferramenta madura. |
| Linter oficial | `planejada` | Ainda não existe. |
| Debugger story | `planejada` | Ainda não existe. |
| `snask test` oficial | `planejada` | Ainda não existe como subcomando maduro da linguagem. |

## Runtime e Interop

| Feature | Status | Observações |
| --- | --- | --- |
| Runtime C nativo | `parcial` | Existe e é grande, mas ainda precisa modularização e contratos mais claros. |
| Runtime mínimo / tiny | `experimental` | Existe, mas ainda precisa endurecimento. |
| `import_c_om` | `experimental` | Importa headers C, gera contrato OM em memória e permite chamadas Snask para símbolos C seguros. Ainda não cobre callbacks, structs complexas e ownership ambíguo de forma universal. |
| HTTP / JSON / IO / OS nativos | `parcial` | Há superfície funcional, mas ainda com semântica e tipagem fracas. |
| GUI | `experimental` | Existe, mas ainda não deve ser tratado como parte madura do núcleo. |
| SQLite | `experimental` | Existe como integração, ainda não como pilar estável. |
| FFI formal | `planejada` | Ainda não há contrato de linguagem bem definido para isso. |

## Regras Para Atualizar Este Documento

- Só promover uma feature para `estavel` quando houver:
  - implementação de ponta a ponta
  - testes adequados
  - comportamento documentado
  - semântica razoavelmente fechada
- Se a doc prometer algo que o compilador ainda não garante, o status deve ser `parcial`, `experimental` ou `planejada`.
- Este documento deve ser atualizado no mesmo PR que mudar o status real de uma feature.
