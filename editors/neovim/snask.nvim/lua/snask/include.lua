local M = {}

function M.includeexpr(fname)
  if not fname or fname == "" then
    return fname
  end

  local clean = fname:gsub('^["\']', ""):gsub('["\']$', "")
  clean = clean:gsub("::", "/")

  if clean:match("%.snask$") or clean:match("%.snif$") then
    return clean
  end

  return clean .. ".snask"
end

return M

