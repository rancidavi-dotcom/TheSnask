if vim.b.did_snask_ftplugin then
  return
end
vim.b.did_snask_ftplugin = true

vim.bo.commentstring = "// %s"
vim.bo.comments = "s1:/*,mb:*,ex:*/,://"
vim.bo.expandtab = true
vim.bo.shiftwidth = 4
vim.bo.softtabstop = 4
vim.bo.tabstop = 4
vim.bo.textwidth = 100
vim.bo.suffixesadd = ".snask"
vim.bo.includeexpr = "v:lua.require'snask.include'.includeexpr(v:fname)"
vim.bo.include = [[^\s*\%(import\|from\)\s\+["']\?\zs[-_./A-Za-z0-9:]\+]]

pcall(function()
  vim.opt_local.path:append(vim.fn.getcwd() .. "/src")
  vim.opt_local.path:append(vim.fn.getcwd() .. "/apps")
  vim.opt_local.path:append(vim.fn.getcwd() .. "/examples")
end)

require("snask").on_ft()

