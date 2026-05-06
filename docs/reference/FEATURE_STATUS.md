# Status Real das Features Snask v0.4.1-alpha

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
| Funções tipadas simples | `estavel` | Parâmetros tipados, retorno tipado, chamada, aridade, tipo de argumento e missing return em funções tipadas são validados por testes semânticos. |
| Funções dinâmicas/avançadas | `parcial` | Funções sem anotação ainda podem cair em `Any`; ainda não há overloads, generics de função ou closures maduras. |
| Classes nominais simples | `estavel` | `new Class()`, anotação com nome de classe, propriedades declaradas e métodos como membros são reconhecidos semanticamente. |
| Classes avançadas | `parcial` | Ainda faltam visibilidade, construtores formais, inicialização obrigatória, imutabilidade de propriedades no modelo de classe e ergonomia OOP completa. |
| Herança simples | `estavel` | Membros herdados são resolvidos e subtipo filho -> pai é aceito em anotações e chamadas; pai -> filho é rejeitado. |
| Herança avançada | `parcial` | Ainda não há interfaces, múltipla herança, override checking, `super` ou regras formais de inicialização herdada. |
| `if` / `elif` / `else` | `estavel` | Parsing, semântica e codegen presentes. |
| `while` | `estavel` | Parsing, semântica e codegen presentes. |
| `for in` | `estavel` | Lowering LLVM implementado e validado em execução real para listas, dicionários e strings. |
| `return` | `estavel` | Validado semânticamente. |
| Listas e dicionários homogêneos | `estavel` | Literais homogêneos inferem `list<T>` / `dict<K, V>` básicos e anotações públicas são validadas. |
| Coleções avançadas | `parcial` | Ainda faltam APIs de coleção fortemente tipadas, coleções heterogêneas formais, iteradores customizados e generics definidos por usuário. |
| Acesso por índice em `str`, `list<T>` e `dict<K, V>` | `estavel` | O analisador valida tipo de índice/chave e propaga tipo de elemento/valor para atribuições e leituras. |
| Acesso por propriedade nominal | `estavel` | Instâncias nominais resolvem propriedades e métodos básicos, incluindo membros herdados. |
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
| `list` e `dict` como tipos | `estavel` | Formas abertas `list` / `dict` e parametrizadas `list<T>` / `dict<K, V>` são parseadas e validadas para coleções homogêneas. |
| Tipos exatos de sistema (`u8`, `i32`, `i64`, `ptr`) | `parcial` | Existem no parser, enum de tipos, semântica e builtins de systems; ainda precisam fechar layout/ABI completo para todos os usos. |
| Tipos de função | `parcial` | Existem internamente, mas sem ergonomia e cobertura amplas. |
| Inferência local | `parcial` | Melhorou para classes, coleções homogêneas e alguns fluxos de indexação/iteração, mas ainda cai em `Any` em várias superfícies. |
| Tipos nominais de instância | `estavel` | `new Class(...)` produz `Type::User(Class)`, aceita anotação nominal e respeita subtipo filho -> pai em herança simples. |
| Generics de coleção | `estavel` | `list<T>` e `dict<K, V>` têm parsing e checagem semântica para literais, indexação e atribuição. |
| Generics definidos por usuário | `planejada` | Ainda não há generics em classes/funções, constraints ou inferência avançada. |
| `enum` / ADTs | `planejada` | Ainda não existem no AST principal. |
| Traits / interfaces | `planejada` | Ainda não existem. |
| `Option` / `Result` | `planejada` | Ainda não existem como tipos da linguagem. |

## OM-Snask-System

| Feature | Status | Observações |
| --- | --- | --- |
| `new stack` / `new heap` / `new arena` | `experimental` | A sintaxe existe e participa do codegen; semanticamente `new Class(...)` já produz tipo nominal básico. |
| `promote` | `experimental` | A sintaxe existe, mas a checagem semântica ainda é muito rasa. |
| `zone` | `experimental` | Existe no parser e no AST, mas ainda não possui formalismo estático completo. |
| `scope` | `experimental` | Hoje funciona mais como delimitação sintática do que como contrato forte de memória. |
| `entangle` | `experimental` | Existe sintaticamente, mas ainda não está validado de forma séria. |
| `@unsafe` para chamadas restritas | `estavel` | O analisador bloqueia funções nativas internas e memória crua fora de `@unsafe`, e libera a região mínima explicitamente marcada. |
| C interop pelo OM-Snask-System | `experimental` | `import_c_om` é uma porta do mesmo sistema: deduz contratos a partir de headers C, emite chamadas nativas LLVM, registra recursos opacos em zonas e aceita `.om.snif` como patch. Validado com stdio.h e SDL2 simples. Ver `docs/systems/OM_SNASK_SYSTEM.md`. |
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
| Diagnósticos do parser | `estavel` | Erros recuperáveis têm código, span, mensagem humana, snippet e limite de recuperação testado. |
| Snask Humane Diagnostics | `parcial` | Formato público com código curto, snippet com caret, help/notes e `snask explain` já existe; ainda faltam traduzir mensagens restantes e cobrir linker/OM. |
| Diagnósticos semânticos | `parcial` | Existem, agora renderizados de forma mais humana, mas ainda dependem demais de `Any` e precisam mais sugestões contextuais. |
| LSP | `parcial` | Completion, hover, goto definition, code actions, SNIF formatter/schema e semantic tokens já existem; ainda falta suíte LSP automatizada forte. |
| Plugin Neovim | `experimental` | Existe em `editors/neovim/snask.nvim` com filetype, syntax, indent, comandos, LSP e healthcheck; ainda precisa uso real antes de estabilizar API. |
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
| Primitivas systems/NES | `estavel` | Conversões inteiras, bits/flags, overflow/carry/borrow e memória crua com gate `@unsafe` têm testes semânticos e base documentada. |
| `import_c_om` | `experimental` | Importa headers C, gera contrato OM em memória e permite chamadas Snask para símbolos C seguros. Ainda não cobre callbacks, structs complexas e ownership ambíguo de forma universal. |
| HTTP / JSON / IO / OS nativos | `parcial` | Há superfície funcional, mas ainda com semântica e tipagem fracas. |
| GUI | `experimental` | Existe via runtime nativo e `snaskgui`, mas ainda não deve ser tratado como parte madura do núcleo. |
| Emulador NES em Snask | `experimental` | `apps/nes_emulator` executa ROM NROM real e serve como laboratório do perfil `systems`; ainda não é emulador NES completo. |
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

## Nota de revisao v0.4.1-alpha

A documentacao foi normalizada para PT-BR e os exemplos compilaveis foram separados de exemplos conceituais. Quando uma feature estiver descrita como plano, ela nao deve aparecer como bloco `snask` sem aviso.
