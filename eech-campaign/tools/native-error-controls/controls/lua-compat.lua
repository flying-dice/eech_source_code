print('COMPAT_HOST_LUA=' .. _VERSION)
local dc = require('eech_dc')
assert(type(dc.name) == 'string' and type(dc.boot) == 'function')
print('COMPAT_MODULE=' .. dc.name)
for i = 1, 100 do
  local ok, err = pcall(dc.boot, 'not a table')
  assert(not ok, 'invalid config unexpectedly accepted')
  assert(tostring(err):find('table', 1, true), tostring(err))
  if i == 1 then print('COMPAT_EXPECTED_ERROR=' .. tostring(err)) end
end
collectgarbage('collect')
print('COMPAT_100_ERROR_CALLBACKS=PASS')
