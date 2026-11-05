-- control: errors raised by lua.dll's own C, no native module or host callback in the error path
local function mark(s) io.stdout:write(s .. '\n'); io.stdout:flush() end
mark('BEFORE_PLAIN_ERRORS')
for i = 1, 100 do
  local ok, err = pcall(error, 'plain lua error')
  assert(not ok and tostring(err):find('plain lua error', 1, true), tostring(err))
  ok, err = pcall(string.rep)
  assert(not ok and tostring(err):find('bad argument', 1, true), tostring(err))
end
mark('PLAIN_100_ERRORS=PASS')
