-- control: the host's own native callback (eech-world host.log), no campaign module loaded
local function mark(s) io.stdout:write(s .. '\n'); io.stdout:flush() end
mark('BEFORE_HOST_LOG_GOOD')
host.log('host-log control: good call')
mark('AFTER_HOST_LOG_GOOD')
mark('BEFORE_HOST_LOG_BAD')
for i = 1, 100 do
  local ok, err = pcall(host.log, {})
  if i == 1 then mark('AFTER_HOST_LOG_BAD=' .. tostring(ok)); mark('HOST_LOG_ERROR=' .. tostring(err)) end
  assert(not ok and tostring(err):find('string', 1, true), tostring(err))
end
mark('HOST_LOG_100_ERRORS=PASS')
mark('BEFORE_GC'); collectgarbage('collect'); mark('AFTER_GC')
