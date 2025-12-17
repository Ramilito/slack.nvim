local M = {
  is_open = false,
}
function M.open()
  local client = require("slack.client")
  client.set_implementation(function(ok, result)

    -- if not result.ok then
    --   vim.notify("Slack register failed: " .. result.error, vim.log.levels.ERROR)
    -- else
    --   print(
    --     ("Connected to %s (%s) as %s (%s)"):format(
    --       result.team,
    --       result.team_id,
    --       result.user or "<unknown>",
    --       result.user_id
    --     )
    --   )
    -- end
    --
    -- -- Later, just re-use the stored token
    -- local ping = slack_native.test_connection("work")
    -- if not ping.ok then
    --   vim.notify("Slack connection failed: " .. ping.error, vim.log.levels.ERROR)
    -- else
    --   print("Slack connection OK for team " .. ping.team)
    -- end
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
