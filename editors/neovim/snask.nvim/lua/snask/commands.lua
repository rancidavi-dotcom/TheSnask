local config = require("snask.config")
local jobs = require("snask.jobs")
local util = require("snask.util")

local M = {}

local function split_args(s)
  if not s or s == "" then
    return {}
  end
  return vim.split(s, "%s+", { trimempty = true })
end

local function current_file_or_warn()
  local file = util.current_file()
  if not file then
    util.notify("Salve o buffer antes de rodar Snask.", vim.log.levels.WARN)
    return nil
  end
  return file
end

function M.build(extra)
  local file = current_file_or_warn()
  if not file then
    return
  end

  local args = { "build", file }
  if config.options.build.profile then
    vim.list_extend(args, { "--profile", config.options.build.profile })
  end
  vim.list_extend(args, split_args(extra))
  jobs.run(args)
end

function M.run(extra)
  local file = current_file_or_warn()
  if not file then
    return
  end

  local output = vim.fn.tempname() .. "-snask-run"
  local args = { "build", file, "--output", output }
  if config.options.build.profile then
    vim.list_extend(args, { "--profile", config.options.build.profile })
  end
  vim.list_extend(args, split_args(extra))

  jobs.run(args, {
    on_exit = function(code)
      if code ~= 0 then
        util.notify("Build falhou; execucao cancelada.", vim.log.levels.ERROR)
        return
      end
      vim.fn.jobstart({ output }, {
        stdout_buffered = true,
        stderr_buffered = true,
        on_stdout = function(_, data)
          if data and #data > 1 then
            util.open_scratch("Snask Run", data, "snask-output")
          end
        end,
        on_stderr = function(_, data)
          if data and #data > 1 then
            util.open_scratch("Snask Run Errors", data, "snask-output")
          end
        end,
      })
    end,
  })
end

function M.doctor()
  jobs.run({ "doctor" }, { open_output = true })
end

function M.setup_toolchain(extra)
  local args = { "setup" }
  vim.list_extend(args, split_args(extra))
  jobs.run(args, { open_output = true })
end

function M.explain(code)
  if not code or code == "" then
    code = vim.fn.expand("<cword>")
  end
  jobs.run({ "explain", code }, { open_output = true })
end

function M.om_scan(extra)
  local args = { "om", "scan" }
  vim.list_extend(args, split_args(extra))
  jobs.run(args, { open_output = true })
end

function M.lsp_restart()
  for _, client in ipairs(vim.lsp.get_active_clients({ bufnr = 0 })) do
    if client.name == "snask-lsp" then
      client.stop()
    end
  end
  vim.defer_fn(function()
    require("snask.lsp").start(0)
  end, 150)
end

function M.create()
  vim.api.nvim_create_user_command("SnaskBuild", function(params)
    M.build(params.args)
  end, { nargs = "*", force = true })

  vim.api.nvim_create_user_command("SnaskRun", function(params)
    M.run(params.args)
  end, { nargs = "*", force = true })

  vim.api.nvim_create_user_command("SnaskDoctor", function()
    M.doctor()
  end, { force = true })

  vim.api.nvim_create_user_command("SnaskSetup", function(params)
    M.setup_toolchain(params.args)
  end, { nargs = "*", force = true })

  vim.api.nvim_create_user_command("SnaskExplain", function(params)
    M.explain(params.args)
  end, { nargs = "?", force = true })

  vim.api.nvim_create_user_command("SnaskOmScan", function(params)
    M.om_scan(params.args)
  end, { nargs = "*", force = true })

  vim.api.nvim_create_user_command("SnaskLspRestart", function()
    M.lsp_restart()
  end, { force = true })
end

return M
