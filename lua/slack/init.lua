local M = {
  is_open = false,
}
function M.open()
  local client = require("kubectl.client")
  client.set_implementation(function(ok, result)
    print(ok, result)
  end)
end

function M.close()
  print("closing")
end

function M.toggle()
  if M.is_open then
    M.close()
    M.is_open = false
  else
    vim.cmd("tabnew")
    M.open()
    M.is_open = true
  end
end

return M
