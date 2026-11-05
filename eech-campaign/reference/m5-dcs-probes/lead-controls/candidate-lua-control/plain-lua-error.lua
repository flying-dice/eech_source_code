local function mark(s) io.stdout:write(s .. '\n'); io.stdout:flush() end
mark('BEFORE_PLAIN_LUA_ERROR')
local ok, err = pcall(error, 'compat expected error')
mark('AFTER_PLAIN_LUA_ERROR=' .. tostring(ok))
assert(not ok and tostring(err):find('compat expected error', 1, true))
mark('PLAIN_LUA_ERROR=PASS')
