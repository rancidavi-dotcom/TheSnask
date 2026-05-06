# Extensao Neovim do Snask

O plugin fica em:

```text
editors/neovim/snask.nvim
```

Ele e pensado como extensao completa de desenvolvimento Snask para Neovim:

- filetype para `.snask`, `.snif` e `.om.snif`;
- syntax highlighting sem dependencia externa;
- indentacao de blocos;
- integracao LSP com `snask-lsp`;
- quickfix para build;
- comandos de toolchain;
- snippets;
- healthcheck.

## Instalar localmente com lazy.nvim

```lua
{
  dir = "/home/davidev/Desktop/TheSnask/editors/neovim/snask.nvim",
  ft = { "snask", "snif" },
  config = function()
    require("snask").setup({
      lsp = {
        cmd = { "snask-lsp" },
      },
    })
  end,
}
```

## Instalar com pack nativo

```bash
mkdir -p ~/.local/share/nvim/site/pack/snask/start
ln -s /home/davidev/Desktop/TheSnask/editors/neovim/snask.nvim \
  ~/.local/share/nvim/site/pack/snask/start/snask.nvim
```

O plugin faz setup automatico por padrao. Para desativar:

```lua
vim.g.snask_nvim_auto_setup = 0
require("snask").setup()
```

## LSP

Compile e instale o servidor:

```bash
cargo build --release --bin snask-lsp
mkdir -p ~/.snask/bin
cp target/release/snask-lsp ~/.snask/bin/snask-lsp
```

Depois:

```vim
:checkhealth snask
```

## Comandos

```vim
:SnaskBuild
:SnaskBuild --profile systems
:SnaskRun
:SnaskDoctor
:SnaskSetup
:SnaskExplain S1001
:SnaskOmScan sqlite3.h --lib sqlite3
:SnaskLspRestart
```

## Atalhos padrao

Em buffers `.snask`:

- `<leader>sb`: build;
- `<leader>sr`: run;
- `<leader>sd`: doctor;
- `<leader>se`: explain;
- `K`: hover;
- `gd`: definition;
- `<leader>ca`: code action.

## Status

O plugin ja funciona com syntax regex e LSP. Tree-sitter ainda depende de um parser Snask dedicado; por isso o plugin inclui apenas queries iniciais, prontas para quando o parser existir.

