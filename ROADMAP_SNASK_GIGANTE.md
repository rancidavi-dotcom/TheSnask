# Snask Roadmap: De Experimento a Linguagem Grande

Este roadmap define o que precisa ser construído para a Snask evoluir de linguagem experimental para uma linguagem séria, robusta e grande no nível de maturidade técnica de ecossistemas como C, Rust, Zig, Go ou Swift.

Escopo deste documento:
- Foca em linguagem, compilador, runtime, tooling, distribuição e confiabilidade.
- Não depende de comunidade grande nem de muitas libs para começar.
- Assume que performance continua importante, mas nunca acima de corretude.

Principios:
- A especificação precisa mandar no compilador.
- O compilador precisa ser previsível antes de ser esperto.
- O modelo de memória precisa ser formal antes de ser vendido como diferencial.
- Tooling e testes são parte da linguagem, não extras.
- Benchmarks sem corretude não contam como maturidade.

## Estado Atual

Hoje a Snask já tem base promissora:
- Parser, AST, análise semântica e geração LLVM.
- Runtime nativo em C.
- Sistema de projeto e manifest.
- LSP inicial.
- Benchmarks e exemplos reais.

Mas ainda faltam pilares de linguagem grande:
- Type system fraco e muito dependente de `Any`.
- Modelo de memória ainda mais conceitual do que formal.
- Inconsistências entre documentação, testes e implementação.
- Pouca cobertura automatizada para garantir evolução segura.
- Ferramentas de DX ainda incompletas.

## Meta Final

A Snask deve chegar em um ponto onde:
- O type system consegue expressar programas grandes sem cair em dinamismo excessivo.
- O modelo OM tem regras estáticas claras, auditáveis e testadas.
- O compilador rejeita programas inválidos com diagnósticos bons e consistentes.
- O runtime é pequeno, previsível e confiável.
- O tooling permite escrever, refatorar, depurar e distribuir software grande.
- A especificação é estável o suficiente para terceiros implementarem partes do ecossistema.

## Fase 0: Estabilizacao do Nucleo Atual

Objetivo:
- Parar de expandir superfície e fechar as rachaduras do que já existe.

Entregas:
- Corrigir testes quebrados do parser e tornar `cargo test` verde por padrão. -feito-
- Alinhar docs com o comportamento real da linguagem. -feito-
- Criar tabela oficial de features: `planejada`, `experimental`, `estavel`, `descontinuada`. -feito-
- Eliminar comportamentos ambíguos entre parser, analisador semântico e codegen.
- Definir uma política de compatibilidade para releases.
- Congelar temporariamente novas features grandes até a base ficar confiável.

Trabalho tecnico:
- Revisar semicolons opcionais, recuperação de erro, spans e mensagens diagnósticas.
- Auditar todos os pontos em que a documentação promete mais do que o código entrega.
- Criar testes mínimos por feature já existente.
- Mapear tudo que retorna `Any` e classificar: aceitável, provisório, bug, dívida.
- Alinhar a validação de entrypoint com o codegen: `class main` agora precisa ter ao menos um método, e a regra foi coberta por testes. -feito-

Criterio de pronto:
- `cargo test` verde.
- Docs principais sem contradições grosseiras.
- Lista pública do que realmente existe na linguagem.

## Fase 1: Especificacao Formal da Linguagem

Objetivo:
- Transformar Snask de implementação experimental em linguagem definida.

Entregas:
- Especificação lexical.
- Especificação sintática.
- Especificação semântica.
- Especificação do modelo de tipos.
- Especificação do modelo de memória OM.
- Especificação de módulos, imports, manifest, build e linking.

Trabalho tecnico:
- Definir oficialmente gramática e precedência de operadores.
- Definir scoping, shadowing, mutabilidade e regras de resolução de nomes.
- Definir igualdade, coerções, promoção numérica e truthiness.
- Definir semântica precisa de `zone`, `scope`, `promote`, `entangle`, `new stack`, `new arena`, `new heap`.
- Definir o que é UB, erro de compilação, erro de runtime e comportamento definido.

Criterio de pronto:
- É possível responder qualquer dúvida da linguagem apontando para spec, não para código ad hoc.

## Fase 2: Type System de Verdade

Objetivo:
- Sair do modelo excessivamente dinâmico e construir garantias fortes.

Entregas prioritárias:
- Tipos nominais reais para classes e structs.
- Tipos de instância em vez de `Any` para `new`.
- Tipagem de propriedades e métodos.
- Tipagem de listas e dicionários com parâmetros.
- Melhor inferência local.
- Melhor checagem de retorno e chamadas.

Progresso inicial desta fase:
- Tipos nominais básicos para classes e instâncias de `new` já foram introduzidos no analisador semântico, incluindo type annotations com nomes de classe e resolução semântica básica de propriedades/métodos. -feito-
- Resolução de membros herdados e validação tipada de `property assignment` também já foram introduzidas no analisador semântico, com testes cobrindo acesso herdado e atribuição incompatível. -feito-
- O analisador também já faz inferência homogênea básica para listas e dicionários, propagando tipos em `for`, indexação e index assignment. -feito-
- O analisador agora também valida tipos de usuário desconhecidos, chamadas variádicas básicas, `new` de classe inexistente e missing return em funções com retorno tipado. -feito-

Features alvo:
- `list<T>`
- `dict<K, V>`
- `option<T>`
- `result<T, E>`
- tipos de função mais completos
- aliases de tipo
- `struct` separado de `class`
- `enum` e ADTs

Depois disso:
- generics
- constraints
- traits ou interfaces
- associated methods
- type aliases parametrizados

Trabalho tecnico:
- Reduzir gradualmente uso de `Any`.
- Introduzir tabela de tipos de usuário.
- Tipar `self`, propriedades, construtores e dispatch.
- Tipar acesso por índice e propriedade.
- Definir conversões implícitas permitidas e proibir o resto.

Criterio de pronto:
- Programas médios conseguem ser escritos quase sem `Any`.
- Erros de tipo relevantes aparecem no compile-time.

## Fase 3: Modelo OM Formal e Confiavel

Objetivo:
- Fazer o OM-Snask-System ser uma tecnologia séria, não só uma ideia boa.

Entregas:
- Escape analysis real.
- Regras de lifetime/ownership por zona.
- Checagem de retorno seguro entre zonas.
- Modelo claro para aliasing.
- Regras para promoção manual e auto-promotion.
- Diagnósticos específicos para violações de memória.

Trabalho tecnico:
- Definir `zone_depth` real no analisador semântico.
- Rastrear origem, destino e tempo de vida de valores.
- Identificar referências que escapam.
- Impedir uso-after-reset e vazamento semântico entre arenas.
- Delimitar o que `unsafe` pode furar.

Muito importante:
- Se OM quiser competir com Rust em confiança, precisa de formalismo.
- Se quiser competir com C em controle, precisa de previsibilidade.
- Sem isso, vai parecer apenas um runtime customizado com sintaxe amigável.

Criterio de pronto:
- Há exemplos de erros de memória pegos em compile-time.
- O whitepaper bate com a implementação.

## Fase 4: Superficie de Linguagem para Software Grande

Objetivo:
- Dar capacidade expressiva suficiente para projetos reais.

Features prioritárias:
- `match`
- `break`
- `continue`
- `defer`
- `enum`
- `struct`
- closures/lambdas maduras
- métodos estáticos
- visibilidade `pub` e privada
- namespaces reais
- imports seletivos robustos
- pattern matching
- destructuring

Features importantes depois:
- interfaces/traits
- overload controlado ou dispatch explícito
- annotations/attributes
- compile-time flags melhores
- macros simples ou metaprogramação restrita

Nao fazer cedo demais:
- features mágicas que pioram previsibilidade
- sintaxe demais sem semântica consolidada

Criterio de pronto:
- Dá para escrever CLI, servidor, app desktop e ferramentas de build sem sentir falta de construções básicas.

## Fase 5: Erros, Efeitos e Seguranca Semantica

Objetivo:
- Tornar falha parte do design da linguagem.

Entregas:
- `option<T>` e `result<T, E>` nativos.
- `try` ou operador de propagação.
- panics bem definidos.
- diferenças formais entre erro recuperável e falha fatal.
- contratos para APIs nativas.

Trabalho tecnico:
- Proibir uso solto de `nil` como solução universal.
- Tirar fluxo de erro implícito baseado em convenção.
- Melhorar checagem de caminhos de retorno.
- Introduzir no LSP sugestões de tratamento de erro.

Criterio de pronto:
- APIs novas conseguem expressar sucesso/falha sem depender de documentação informal.

## Fase 6: Compilador e Backend de Producao

Objetivo:
- Fazer o compilador aguentar codebase grande com confiança.

Entregas:
- pipeline mais modular
- IR interna mais bem definida
- lowering semântico separado de codegen LLVM
- infraestrutura de otimizações próprias onde fizer sentido
- builds incrementais
- cache de compilação
- compilação paralela de módulos

Trabalho tecnico:
- separar melhor front-end, middle-end e backend
- reduzir lógica semântica espalhada
- evitar decisões de codegen baseadas em heurística implícita
- preparar terreno para multi-backend no futuro
- Implementar lowering LLVM de `for in` com execução real validada em listas, dicionários e strings. -feito-

Possível futuro:
- LLVM continua backend principal
- C backend ou bytecode backend apenas se houver motivo real

Criterio de pronto:
- O compilador cresce sem virar um arquivo monolítico difícil de manter.

## Fase 7: Runtime Pequeno, Modular e Auditavel

Objetivo:
- Ter runtime nativo confiável, separável e bem versionado.

Entregas:
- modularização real do runtime
- ABI interna documentada
- separação clara entre runtime core e módulos pesados
- testes dedicados do runtime C
- sanitizers no CI
- benchmark com correctness checks

Trabalho tecnico:
- definir contrato estável entre LLVM IR e runtime
- auditar memória, strings, objetos, JSON, IO, GUI e HTTP
- reduzir duplicação entre `runtime_old.c`, `runtime_nano.c` e módulos
- definir quais partes são core, optional e legacy

Criterio de pronto:
- O runtime pode evoluir sem quebrar tudo silenciosamente.

## Fase 8: Modulos, Packages e Build System de Verdade

Objetivo:
- Fazer SPS e packages sustentarem projetos grandes e reproduzíveis.

Entregas:
- resolução de dependência mais robusta
- lockfile realmente confiável
- version constraints formais
- namespaces de package
- suporte melhor a workspace multi-package
- builds reproduzíveis
- publicação de package com validação forte

Trabalho tecnico:
- melhorar semântica de `import`
- definir API pública versus interna
- evitar colisão de nomes entre módulos
- validar integridade e compatibilidade de packages
- preparar suporte a toolchain pinning

Criterio de pronto:
- Projetos com vários módulos e packages não viram bagunça de arquivo.

## Fase 9: Tooling de Linguagem Grande

Objetivo:
- Dar experiência de desenvolvimento que acompanhe a ambição da linguagem.

Entregas:
- formatter oficial estável
- linter oficial
- LSP com rename
- LSP com references
- LSP com document symbols
- LSP com workspace symbols
- LSP com inlay hints
- LSP com code actions melhores
- debugger story
- profiler story

Ferramentas desejadas:
- `snask fmt`
- `snask check`
- `snask test`
- `snask doc`
- `snask bench`
- `snask doctor`

Criterio de pronto:
- Escrever Snask em projeto grande não exige adivinhar comportamento do compilador.

## Fase 10: Testes, QA e Confiabilidade Industrial

Objetivo:
- Construir confiança acumulada.

Entregas:
- suíte grande de parser tests
- suíte grande de semântica
- golden tests de diagnósticos
- testes de integração de build
- testes de runtime
- fuzzing de parser
- fuzzing de manifest/SNIF
- testes diferenciais
- benchmarks reprodutíveis com validação de resultado

Trabalho tecnico:
- criar diretórios de fixtures por feature
- medir cobertura útil, não só cobertura percentual
- CI com Linux e, quando possível, macOS e Windows
- sanitizers, ASan, UBSan, valgrind em pipelines específicos

Criterio de pronto:
- Quebrar feature importante sem o CI perceber vira exceção rara.

## Fase 11: Interop, FFI e Plataforma

Objetivo:
- Permitir que Snask converse com o mundo real com segurança.

Entregas:
- FFI C documentada
- regras para tipos compatíveis
- `unsafe` formal
- linking estático e dinâmico com contrato claro
- suporte de plataforma melhor definido

Depois:
- chamadas para libs do sistema
- embedding da runtime
- geração de bindings

Criterio de pronto:
- Integrar com C deixa de ser gambiarra e vira feature confiável.

## Fase 12: Distribuicao, Reprodutibilidade e Releases

Objetivo:
- Fazer a linguagem ser instalável, versionável e previsível.

Entregas:
- canais de release
- nightly, beta, stable
- changelog rigoroso
- policy de breaking changes
- toolchain pinning
- reproducible builds
- binários oficiais por plataforma

Criterio de pronto:
- Usuário sabe exatamente o que esperar ao atualizar.

## Fase 13: Documentacao de Nivel Profissional

Objetivo:
- Fazer aprendizado e referência escalarem junto com a linguagem.

Entregas:
- book oficial
- language reference confiável
- guia de OM
- guia de FFI
- guia de package publishing
- guia de performance
- guia de debugging
- migration guides por versão

Importante:
- docs devem refletir o código de release, não intenções futuras.

Criterio de pronto:
- Nova pessoa consegue aprender Snask sem depender do autor para explicar lacunas.

## Fase 14: 1.0

Snask só deve chamar 1.0 quando tiver:
- spec estável
- type system suficientemente forte
- OM formalizado
- `cargo test` e CI confiáveis
- runtime auditado
- formatter e LSP bons
- pacotes e build reproduzíveis
- política de compatibilidade
- documentação séria

Nao chamar 1.0 antes disso:
- mesmo que benchmarks estejam ótimos
- mesmo que a sintaxe esteja bonita
- mesmo que existam demos impressionantes

## Ordem Recomendada de Execucao

Ordem certa:
1. Fase 0
2. Fase 1
3. Fase 2
4. Fase 3
5. Fase 10 em paralelo com 2 e 3
6. Fase 4
7. Fase 5
8. Fase 6
9. Fase 7
10. Fase 8
11. Fase 9
12. Fase 11
13. Fase 12
14. Fase 13
15. Fase 14

Porque:
- Sem base estável, novas features só aumentam dívida.
- Sem spec, o compilador vira coleção de exceções.
- Sem type system e OM formais, Snask não terá identidade técnica forte.
- Sem QA, cada avanço custa mais do que entrega.

## Prioridade Imediata dos Proximos 90 Dias

Se fosse para focar no que mais muda o jogo agora:
1. Fazer a suíte passar e ampliar testes do parser, semântica e imports.
2. Criar documento oficial de status das features reais da linguagem.
3. Formalizar o plano do type system v1.
4. Formalizar o plano do OM checker v1.
5. Introduzir tipos nominais reais para classes/instâncias.
6. Reduzir `Any` nas áreas mais críticas.
7. Melhorar diagnósticos e LSP básico.

## Riscos Que Podem Matar a Evolucao

- Continuar adicionando sintaxe sem semântica forte.
- Vender OM como diferencial sem checagem estática real.
- Aceitar contradições entre docs e compilador por muito tempo.
- Crescer runtime e stdlib antes de consolidar o núcleo.
- Basear reputação só em benchmark.
- Não criar disciplina de release e compatibilidade.

## Sinal de Que a Snask Esta no Caminho Certo

Você vai saber que a Snask entrou no caminho de linguagem grande quando:
- o compilador começar a recusar classes inteiras de bugs cedo
- o código da linguagem ficar menos dependente de `Any`
- as docs virarem fonte de verdade
- refactors ficarem seguros
- exemplos grandes começarem a parecer normais, não demos especiais
- contributors conseguirem evoluir o projeto sem conversar com o autor toda hora

## Resumo Executivo

O caminho para a Snask virar linguagem gigante nao e:
- só adicionar mais keywords
- só bater benchmark
- só criar frameworks

O caminho real e:
- especificacao
- type system
- modelo de memoria formal
- confiabilidade
- tooling
- distribuicao

Se esses pilares forem construidos na ordem certa, a Snask pode ter identidade propria de verdade.
