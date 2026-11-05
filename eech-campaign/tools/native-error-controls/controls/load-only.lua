local function mark(s) io.stdout:write(s .. '\n'); io.stdout:flush() end
mark('BEFORE_REQUIRE')
local ok, dc = pcall(require, 'eech_dc')
mark('AFTER_REQUIRE=' .. tostring(ok))
if not ok then mark(tostring(dc)); error(dc) end
mark('MODULE=' .. tostring(dc.name))
mark('BEFORE_GC'); collectgarbage('collect'); mark('AFTER_GC')
