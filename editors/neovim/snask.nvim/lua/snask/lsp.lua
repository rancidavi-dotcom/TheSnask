local config = require("snask.config")
local util = require("snask.util")

local M = {}

local function setup_diagnostics()
  local sign = vim.fn.sign_define
  local icons = {
    Error = "✘",
    Warn = "⚠",
    Info = "ℹ",
    Hint = "➤",
  }

  for severity, icon in pairs(icons) do
    local name = "DiagnosticSign" .. severity
    pcall(sign, name, { text = icon, texthl = name })
  end

  vim.diagnostic.config({
    virtual_text = { prefix = "●" },
    signs = true,
    underline = true,
    update_in_insert = false,
    severity_sort = true,
    float = {
      focusable = false,
      style = "minimal",
      border = "rounded",
      source = "always",
      header = "",
      prefix = "",
    },
  })
end

local function supports_method(client, method)
  if client.supports_method then
    return client:supports_method(method)
  end
  return client.server_capabilities and client.server_capabilities[method]
end

function M.on_attach(client, bufnr)
  if config.options.lsp.semantic_tokens == false and client.server_capabilities then
    client.server_capabilities.semanticTokensProvider = nil
  end

  if config.options.keymaps.enable then
    local opts = { buffer = bufnr, silent = true }
    vim.keymap.set("n", "K", vim.lsp.buf.hover, opts)
    vim.keymap.set("n", "gd", vim.lsp.buf.definition, opts)
    vim.keymap.set("n", "<leader>ca", vim.lsp.buf.code_action, opts)
  end

  if supports_method(client, "textDocument/documentSymbol") then
    vim.bo[bufnr].tagfunc = "v:lua.vim.lsp.tagfunc"
  end
end

function M.start(bufnr)
  bufnr = bufnr or 0
  if not config.options.lsp.enable then
    return
  end

  setup_diagnostics()

  local cmd = config.options.lsp.cmd
  if not util.executable(cmd) then
    if not vim.b[bufnr].snask_lsp_missing_notified then
      vim.b[bufnr].snask_lsp_missing_notified = true
      util.notify("snask-lsp nao encontrado. Compile com `cargo build --release --bin snask-lsp`.", vim.log.levels.WARN)
    end
    return
  end

  local file = vim.api.nvim_buf_get_name(bufnr)
  local root = util.root_dir(file)

  for _, client in ipairs(vim.lsp.get_active_clients({ bufnr = bufnr })) do
    if client.name == "snask-lsp" then
      return
    end
  end

  vim.lsp.start({
    name = "snask-lsp",
    cmd = cmd,
    root_dir = root,
    on_attach = M.on_attach,
    capabilities = vim.lsp.protocol.make_client_capabilities(),
  }, { bufnr = bufnr })
end

return M
