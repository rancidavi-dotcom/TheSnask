# Snask Docs Site

Este diretorio contem o site estatico oficial da documentacao Snask.

Ele foi pensado para GitHub Pages e Vercel: nao precisa de Node, bundler, gerador estatico
ou etapa de build. O workflow `.github/workflows/pages.yml` publica exatamente o
conteudo de `docs/site`.

## Funcionalidades do Site

- Tema claro/escuro automatico (segue o sistema) com alternancia manual
- Layout responsivo com menu hamburger em dispositivos moveis
- Navegacao lateral com destaque da pagina ativa
- Botoes de copiar codigo com feedback visual
- Busca de funcoes na pagina de indice
- Design moderno com variaveis CSS e transicoes suaves

## Estrutura

- `index.html`: entrada principal da documentacao.
- `learn/`: guias para aprender e criar projetos.
- `reference/`: referencia da linguagem, tipos, runtime/builtins, diagnosticos e status real.
- `reference/functions/`: paginas individuais geradas para funcoes e builtins.
- `systems/`: OM-Snask-System, profile systems, low-level e interop C.
- `tooling/`: instalacao, CLI, SPS/SNIF, LSP e Neovim.
- `examples/`: exemplos pequenos e verificaveis.
- `assets/`: CSS e JavaScript compartilhados.
- `vercel.json`: configuracao de deploy para Vercel (clean URLs).

## Como testar localmente

```bash
cd docs/site
python3 -m http.server 8080
```

Abra `http://localhost:8080`.

## Como regenerar a referencia de funcoes

```bash
scripts/generate_docs_reference.py
```

## Como validar exemplos da documentacao

```bash
scripts/check_doc_examples.sh
```

Exemplos que abrem GUI ficam fora do fluxo padrao. Para inclui-los:

```bash
SNASK_DOC_GUI=1 scripts/check_doc_examples.sh
```

## Regras editoriais

- Escreva em portugues claro e consistente.
- Separe o que esta pronto do que e parcial ou experimental.
- Prefira exemplos pequenos que ainda compilam no Snask atual.
- Nao prometa features futuras como se ja fossem estaveis.
- Quando uma pagina resumir algo tecnico, mantenha link para o Markdown fonte em
  `docs/`.

## Publicacao

Pushes para `main` que alteram `docs/site/**` ou `.github/workflows/pages.yml`
acionam o deploy automatico do GitHub Pages.
