--- @class slack.Client
local client = {
  --- @type slack.ClientImplementation
  implementation = require("slack.client.rust"),
}

function client.set_implementation(callback)
  client.implementation = require("slack_client")
  client.implementation.init_logging(vim.fn.stdpath("log"))
  client.register("work", "")
  local ok, result = client.implementation.init_runtime("work")

  callback(ok, result)
end

function client.register(profile, token)
  return client.implementation.register(profile, token)
end

function client.conversations(limit)
  return client.implementation.conversations(limit)
end

return client
