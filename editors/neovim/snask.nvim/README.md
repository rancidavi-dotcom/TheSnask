# snask.nvim

Plugin Neovim oficial do Snask.

Ele fornece:

- deteccao de filetype para `.snask`, `.snif` e `.om.snif`;
- syntax highlighting Vimscript sem dependencia externa;
- integracao LSP com `snask-lsp`;
- comandos de build, run, setup, doctor, explain e OM scan;
- indentacao basica consciente de blocos Snask;
- snippets JSON para engines compativeis com VS Code/LuaSnip;
- healthcheck com `:checkhealth snask`;
- query inicial de highlights para quando um parser Tree-sitter Snask existir.

## Instalacao com lazy.nvim

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

Enquanto o plugin nao estiver publicado, use `dir`. Depois ele pode virar:

```lua
{
  "rancidavi-dotcom/snask.nvim",
  config = function()
    require("snask").setup()
  end,
}
```

## Instalacao com pack nativo

```bash
mkdir -p ~/.local/share/nvim/site/pack/snask/start
ln -s /home/davidev/Desktop/TheSnask/editors/neovim/snask.nvim \
  ~/.local/share/nvim/site/pack/snask/start/snask.nvim
```

No `init.lua`:

```lua
require("snask").setup()
```

## Requisitos

- Neovim 0.9+ recomendado;
- `snask` no `PATH`;
- `snask-lsp` no `PATH` para LSP completo.

Build do LSP:

```bash
cargo build --release --bin snask-lsp
cp target/release/snask-lsp ~/.snask/bin/snask-lsp
```

## Comandos

| Comando | Acao |
| --- | --- |
| `:SnaskBuild` | compila o arquivo atual |
| `:SnaskBuild --profile systems` | compila com argumentos extras |
| `:SnaskRun` | compila e roda o arquivo atual |
| `:SnaskDoctor` | roda `snask doctor` |
| `:SnaskSetup` | roda `snask setup` |
| `:SnaskExplain S1001` | abre explicacao de diagnostico |
| `:SnaskOmScan sqlite3.h --lib sqlite3` | roda scanner OM |
| `:SnaskLspRestart` | reinicia clientes LSP Snask |

Os comandos abrem a quickfix quando encontram mensagens no formato `arquivo:linha:coluna`.

## Configuracao

```lua
require("snask").setup({
  snask = "snask",
  lsp = {
    enable = true,
    cmd = { "snask-lsp" },
    semantic_tokens = true,
  },
  build = {
    profile = nil,
    output_dir = nil,
  },
  keymaps = {
    enable = true,
  },
})
```

## Keymaps padrao

Aplicados apenas em buffers Snask:

- `<leader>sb`: build;
- `<leader>sr`: run;
- `<leader>sd`: doctor;
- `<leader>se`: explain do codigo sob cursor;
- `K`: hover LSP quando houver cliente ativo;
- `gd`: goto definition;
- `<leader>ca`: code action.

