local M = {}

function M.executable(cmd)
  if type(cmd) == "table" then
    cmd = cmd[1]
  end
  return cmd and vim.fn.executable(cmd) == 1
end

function M.root_dir(fname)
  fname = fname or vim.api.nvim_buf_get_name(0)
  local markers = {
    "snask.snif",
    ".snask",
    ".git",
    "Cargo.toml",
  }

  local found = vim.fs.find(markers, {
    upward = true,
    path = fname ~= "" and vim.fs.dirname(fname) or vim.loop.cwd(),
  })[1]

  if found then
    return vim.fs.dirname(found)
  end

  if fname ~= "" then
    return vim.fs.dirname(fname)
  end

  return vim.loop.cwd()
end

function M.current_file()
  local file = vim.api.nvim_buf_get_name(0)
  if file == "" then
    return nil
  end
  return file
end

function M.qf_from_lines(lines)
  local items = {}
  local pattern = "^([^:]+):(%d+):(%d+):%s*(.*)$"
  for _, line in ipairs(lines or {}) do
    local file, lnum, col, text = line:match(pattern)
    if file then
      table.insert(items, {
        filename = file,
        lnum = tonumber(lnum),
        col = tonumber(col),
        text = text,
      })
    end
  end
  return items
end

function M.notify(msg, level)
  vim.notify(msg, level or vim.log.levels.INFO, { title = "Snask" })
end

function M.open_scratch(name, lines, filetype)
  vim.cmd("botright split")
  local buf = vim.api.nvim_create_buf(false, true)
  vim.api.nvim_win_set_buf(0, buf)
  vim.api.nvim_buf_set_name(buf, name)
  vim.bo[buf].buftype = "nofile"
  vim.bo[buf].bufhidden = "wipe"
  vim.bo[buf].swapfile = false
  vim.bo[buf].filetype = filetype or "text"
  vim.api.nvim_buf_set_lines(buf, 0, -1, false, lines or {})
end

return M

