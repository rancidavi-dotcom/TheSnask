local config = require("snask.config")
local util = require("snask.util")

local M = {}

local function flatten_args(args)
  local out = {}
  for _, arg in ipairs(args or {}) do
    if arg and arg ~= "" then
      table.insert(out, arg)
    end
  end
  return out
end

function M.run(args, opts)
  opts = opts or {}
  local command = opts.command or config.options.snask
  local cwd = opts.cwd or util.root_dir()
  local stdout = {}
  local stderr = {}

  if not util.executable(command) then
    util.notify("Executavel nao encontrado: " .. tostring(command), vim.log.levels.ERROR)
    return
  end

  vim.fn.jobstart(vim.list_extend({ command }, flatten_args(args)), {
    cwd = cwd,
    stdout_buffered = true,
    stderr_buffered = true,
    on_stdout = function(_, data)
      if data then
        vim.list_extend(stdout, data)
      end
    end,
    on_stderr = function(_, data)
      if data then
        vim.list_extend(stderr, data)
      end
    end,
    on_exit = function(_, code)
      vim.schedule(function()
        local all = {}
        vim.list_extend(all, stdout)
        vim.list_extend(all, stderr)

        local qf = util.qf_from_lines(all)
        if #qf > 0 then
          vim.fn.setqflist(qf, "r")
          vim.cmd("copen")
        end

        if opts.open_output or code ~= 0 then
          util.open_scratch("Snask Output", all, "snask-output")
        end

        if opts.on_exit then
          opts.on_exit(code, all)
        elseif code == 0 then
          util.notify("Comando Snask finalizado.")
        else
          util.notify("Comando Snask falhou com codigo " .. code, vim.log.levels.ERROR)
        end
      end)
    end,
  })
end

return M

