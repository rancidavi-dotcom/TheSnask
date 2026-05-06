if vim.g.loaded_snask_nvim == 1 then
  return
end
vim.g.loaded_snask_nvim = 1

if vim.g.snask_nvim_auto_setup == 0 then
  return
end

require("snask").setup(vim.g.snask_nvim or {})
