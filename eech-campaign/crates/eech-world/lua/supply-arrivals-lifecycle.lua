-- Failure and re-entry contract for the public SUPPLY arrival callback.
local args = host.args
local root = assert (args.root, "root=<installation root>")
local case = assert (args.case, "case=missing|malformed|error|reentry")
-- reject an unknown case before the module loads or the installation is touched, so a
-- mistyped case cannot run no branch and still report success
local cases = { missing = true, malformed = true, error = true, reentry = true }
assert (cases[case], "unknown case '" .. tostring (case) .. "': case=missing|malformed|error|reentry")
local dc = require ("eech_dc")
dc.prepare_installation (root)

local function config (callback)
	return {
		install_root = root, map = "..\\common\\maps\\map5", campaign_directory = "camp01", campaign = "LEBANON.CHC",
		gunship = "apache", seed = 1, bind_mobile = host.bind_mobile, move_vehicle = host.move_vehicle, observe_loss = host.observe_loss, supply_reached = callback, line_of_sight = host.line_of_sight,
	}
end

local function expect_error (name, call, text)
	local ok, message = pcall (call)
	assert (not ok and tostring (message):find (text, 1, true), name .. ": " .. tostring (message))
	print (name .. "\tok\t" .. tostring (message):gsub ("[\r\n]+", " "))
end

local function frames_until_error (engine, text)
	for _ = 1, 36000 do
		local ok, message = pcall (engine.frame, engine, 100)
		if not ok then
			assert (tostring (message):find (text, 1, true), tostring (message))
			print ("callback_failure\tok\t" .. tostring (message):gsub ("[\r\n]+", " "))
			return
		end
	end
	error ("SUPPLY arrival callback was not invoked within one simulated hour")
end

if case == "missing" then
	expect_error ("missing", function () dc.boot (config (nil)) end, "missing host supply_reached callback")
	local engine = dc.boot (config (host.supply_reached))
	assert (#engine:members () > 0)
	print ("boot_after\tok")
elseif case == "malformed" or case == "error" then
	local callback = case == "malformed" and function () return nil end or function () error ("host arrival observation failed") end
	local engine = dc.boot (config (callback))
	frames_until_error (engine, case == "malformed" and "supply_reached must return a boolean or a result table, got nil" or "host arrival observation failed")
	expect_error ("poisoned", function () engine:objects () end, "engine is poisoned")
elseif case == "reentry" then
	local engine
	local checked = false
	local function callback (descriptor)
		if not checked then
			local calls = {
				members = function () engine:members () end,
				objects = function () engine:objects () end,
				supply_tasks = function () engine:supply_tasks () end,
				roster_stats = function () engine:roster_stats () end,
				clock = function () engine:clock () end,
				diagnostics = function () engine:diagnostics () end,
				frame = function () engine:frame (100) end,
				prepare = function () dc.prepare_installation (root) end,
				boot = function () dc.boot (config (host.supply_reached)) end,
			}
			for name, call in pairs (calls) do expect_error ("reentry_" .. name, call, "not valid in the engine's state") end
			checked = true
		end
		return host.supply_reached (descriptor)
	end
	engine = dc.boot (config (callback))
	for _ = 1, 36000 do
		engine:frame (100)
		if checked then break end
	end
	assert (checked, "SUPPLY arrival callback was not invoked within one simulated hour")
	engine:frame (100)
	assert (#engine:supply_tasks () > 0)
	print ("continued\tok")
end
print ("result\tok")
