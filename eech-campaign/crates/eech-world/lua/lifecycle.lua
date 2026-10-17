-- The module's lifecycle and failure behaviour, observed through the public
-- path (host Lua -> require ("eech_dc")). One case per process: EECH is global
-- state, booted at most once per process, so every case starts from a fresh
-- process (tools/lifecycle-windows.ps1 runs them all and checks the results).
--
-- host.args: root (a prepared retail installation), scenario (lebanon_retail or
--            georgia_retail), case (below)
--
-- Each step prints one line to stdout: <step> TAB ok|error TAB <first line of the message>

local args = host.args
local root = assert (args.root, "root=<installation root>")
-- the retail campaigns, as campaign.lua boots them
local scenarios = {
	lebanon_retail = { map = "..\\common\\maps\\map5", campaign = "LEBANON.CHC" },
	georgia_retail = { map = "..\\common\\maps\\map3", campaign = "GEORGIA.CHC" },
}
local scenario = assert (scenarios[args.scenario or "lebanon_retail"], "scenario=lebanon_retail|georgia_retail")

local function config (overrides)
	local c = { install_root = root, map = scenario.map, campaign_directory = "camp01", campaign = scenario.campaign, gunship = "apache", seed = 1 }
	for k, v in pairs (overrides or {}) do
		if v == false then c[k] = nil else c[k] = v end
	end
	return c
end

local function step (name, f, ...)
	local ok, result = pcall (f, ...)
	local message = ok and "" or (tostring (result):match ("^[^\n]*") or "")
	print (string.format ("%s\t%s\t%s", name, ok and "ok" or "error", message))
	io.stdout:flush ()
	return ok, result
end

local dc = require ("eech_dc")
step ("load", function () assert (type (dc.boot) == "function") end)
step ("prepare", dc.prepare_installation, root)

local function frames (engine, n)
	for _ = 1, n do engine:frame (100) end
end

local cases = {}

-- one boot per process: a second boot is refused and the first engine carries on
function cases.boot_twice ()
	local _, engine = step ("boot", dc.boot, config ())
	step ("frames", frames, engine, 50)
	step ("boot_again", dc.boot, config ())
	step ("frames_after", frames, engine, 50)
	step ("clock", function () return engine:clock () end)
end

-- the binding checks its arguments before the engine sees them
function cases.bad_arguments ()
	step ("boot_unknown_gunship", dc.boot, config { gunship = "tiger" })
	step ("boot_no_install_root", dc.boot, config { install_root = false })
	step ("boot_no_map", dc.boot, config { map = false })
	step ("boot_no_campaign", dc.boot, config { campaign = false })
	step ("boot_not_a_table", dc.boot, "lebanon")
	local _, engine = step ("boot", dc.boot, config ())
	step ("frame_negative", function () engine:frame (-1) end)
	step ("frame_not_a_number", function () engine:frame ("soon") end)
	step ("frame_zero", function () engine:frame (0) end)
	step ("frames_after", frames, engine, 50)
end

-- inputs the engine itself cannot use
function cases.missing_root ()
	step ("boot_missing_root", dc.boot, config { install_root = root .. "/does-not-exist" })
	step ("boot_after", dc.boot, config ())
end

function cases.missing_map ()
	step ("boot_missing_map", dc.boot, config { map = "..\\common\\maps\\map99" })
	step ("boot_after", dc.boot, config ())
end

function cases.missing_campaign ()
	step ("boot_missing_campaign", dc.boot, config { campaign = "NOSUCH.CHC" })
	step ("boot_after", dc.boot, config ())
end

-- no shutdown or reuse: dropping the engine does not allow another boot, and
-- neither does loading the module again
function cases.no_reuse ()
	local _, engine = step ("boot", dc.boot, config ())
	step ("frames", frames, engine, 50)
	engine = nil
	collectgarbage ()
	collectgarbage ()
	step ("boot_after_collect", dc.boot, config ())
	package.loaded.eech_dc = nil
	local _, reloaded = step ("require_again", require, "eech_dc")
	step ("boot_after_require", reloaded.boot, config ())
end

local case = assert (cases[args.case], "case=boot_twice|bad_arguments|missing_root|missing_map|missing_campaign|no_reuse")
case ()
print ("end")
