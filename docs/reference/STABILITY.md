# Politica de Estabilidade v0.4.1-alpha

Snask ainda esta em fase `alpha`. A politica abaixo existe para evitar que documentacao, exemplos e marketing prometam mais do que o compilador entrega.

## Niveis

### Estavel

Funciona de ponta a ponta, tem testes e deve evitar quebras sem necessidade.

### Parcial

Existe e pode ser usado em cenarios controlados, mas ainda tem lacunas de semantica, type system, runtime ou diagnostico.

### Experimental

Existe para evolucao rapida. Pode mudar entre commits e releases alpha.

### Planejada

E direcao de projeto. Nao deve aparecer como exemplo `snask` compilavel.

## Classificacao resumida atual

Estavel:

- `class main` com `fun start`;
- `let`, `mut`, `const` basicos;
- `if`, `else`, `while`;
- strings interpoladas;
- build AOT via LLVM;
- SNIF core e formatter.

Parcial:

- funcoes com type system ainda incompleto;
- classes de usuario e heranca;
- listas/dicionarios;
- SPS e packages;
- runtime nativo amplo;
- LSP.

Experimental:

- OM-Snask-System avancado;
- `import_c_om`;
- GUI;
- SQLite;
- `@unsafe`;
- perfil `systems` e memoria crua;
- emulador NES em `apps/nes_emulator`.

Planejada:

- borrow checking formal;
- escape analysis forte;
- structs C universais;
- callbacks C seguros;
- perfil `baremetal` completo;
- API GUI madura de alto nivel.

## Regra para docs

Blocos marcados como `snask` devem compilar ou representar erro intencional claramente explicado. Sintaxe futura deve ficar em bloco `text` ou ser marcada como conceitual.
