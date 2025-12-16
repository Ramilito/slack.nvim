--- @class slack.Client
local client = {
  --- @type slack.ClientImplementation
  implementation = require("slack.client.rust"),
}

function client.set_implementation(callback)
  client.implementation = require("slack_client")
  client.implementation.init_logging(vim.fn.stdpath("log"))
  local ok, result = client.implementation.init_runtime()
  callback(ok, result)
end

function client.register_session_token(token, profile)
  return client.implementation.register_session_token(token, profile)
end

return client
