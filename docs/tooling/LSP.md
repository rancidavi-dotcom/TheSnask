# Snask LSP

Servidor Language Server Protocol do Snask para editores.

## Recursos atuais

- diagnosticos de sintaxe;
- diagnosticos semanticos basicos;
- hover basico;
- completion de palavras-chave/snippets;
- base para goto definition, code actions e semantic tokens.

## Build

```bash
cargo build --release
```

Binario:

```text
target/release/snask-lsp
```

## Teste no VS Code

Sem extensao dedicada, use o LSP Inspector.

1. Rode o setup auxiliar quando existir no ambiente:

```bash
tools/dev/setup_lsp_live_test.sh
```

2. No VS Code:

- instale `octref.lsp-inspector-webview`;
- rode `LSP Inspector: Start Server`;
- transporte: `stdio`;
- comando: `target/release/snask-lsp`.

## Configuracao manual

Se houver uma extensao Snask local, configure para iniciar:

```bash
target/release/snask-lsp
```

Para Neovim, use o plugin oficial local em `editors/neovim/snask.nvim`. Veja `docs/tooling/NEOVIM.md`.

## Status

`parcial`. Parser errors ja carregam linha/coluna. Alguns erros semanticos ainda podem aparecer com span incompleto e precisam evoluir junto com `docs/reference/HUMANE_DIAGNOSTICS.md`.
