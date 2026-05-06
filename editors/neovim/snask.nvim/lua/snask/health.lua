local M = {}

local health = vim.health or {}

local function ok(msg)
  if health.ok then
    health.ok(msg)
  else
    health.report_ok(msg)
  end
end

local function warn(msg)
  if health.warn then
    health.warn(msg)
  else
    health.report_warn(msg)
  end
end

local function error(msg)
  if health.error then
    health.error(msg)
  else
    health.report_error(msg)
  end
end

local function executable(name)
  return vim.fn.executable(name) == 1
end

function M.check()
  if health.start then
    health.start("snask.nvim")
  else
    health.report_start("snask.nvim")
  end

  if vim.fn.has("nvim-0.9") == 1 then
    ok("Neovim >= 0.9")
  else
    warn("Neovim 0.9+ recomendado")
  end

  if executable("snask") then
    ok("snask encontrado no PATH")
  else
    error("snask nao encontrado no PATH")
  end

  if executable("snask-lsp") then
    ok("snask-lsp encontrado no PATH")
  else
    warn("snask-lsp nao encontrado. LSP nao inicia sem ele.")
  end

  if vim.fn.exists(":SnaskBuild") == 2 then
    ok("Comandos Snask registrados")
  else
    warn("Comandos Snask ainda nao registrados; chame require('snask').setup()")
  end
end

return M
