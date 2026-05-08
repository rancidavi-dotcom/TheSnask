local config = require("snask.config")

local M = {}

function M.setup(opts)
  config.setup(opts)
  require("snask.commands").create()

  vim.api.nvim_create_autocmd("FileType", {
    group = vim.api.nvim_create_augroup("snask_nvim_lsp", { clear = true }),
    pattern = { "snask", "snif" },
    callback = function(args)
      require("snask").on_ft(args.buf)
    end,
  })
end

function M.on_ft(bufnr)
  bufnr = bufnr or vim.api.nvim_get_current_buf()
  local ft = vim.bo[bufnr].filetype
  if config.options.keymaps.enable and (ft == "snask" or ft == "snif") then
    local opts = { buffer = bufnr, silent = true }
    vim.keymap.set("n", "<leader>sb", "<cmd>SnaskBuild<cr>", opts)
    vim.keymap.set("n", "<leader>sr", "<cmd>SnaskRun<cr>", opts)
    vim.keymap.set("n", "<leader>sd", "<cmd>SnaskDoctor<cr>", opts)
    vim.keymap.set("n", "<leader>se", "<cmd>SnaskExplain<cr>", opts)
    vim.keymap.set("n", "<leader>sf", "<cmd>SnaskFormat<cr>", opts)
  end

  require("snask.lsp").start(bufnr)
end

return M
