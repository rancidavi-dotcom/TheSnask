local M = {}

M.defaults = {
  snask = "snask",
  lsp = {
    enable = true,
    cmd = { "snask-lsp" },
    semantic_tokens = true,
    filetypes = { "snask", "snif" },
  },
  build = {
    profile = nil,
    output_dir = nil,
  },
  keymaps = {
    enable = true,
  },
}

M.options = vim.deepcopy(M.defaults)

local function merge(dst, src)
  if type(src) ~= "table" then
    return dst
  end
  return vim.tbl_deep_extend("force", dst, src)
end

function M.setup(opts)
  M.options = merge(vim.deepcopy(M.defaults), opts or {})
  return M.options
end

return M

