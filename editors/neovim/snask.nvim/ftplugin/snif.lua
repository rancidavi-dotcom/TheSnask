if vim.b.did_snif_ftplugin then
  return
end
vim.b.did_snif_ftplugin = true

vim.bo.commentstring = "# %s"
vim.bo.expandtab = true
vim.bo.shiftwidth = 2
vim.bo.softtabstop = 2
vim.bo.tabstop = 2
vim.bo.textwidth = 100

require("snask").on_ft()

