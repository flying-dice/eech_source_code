-- TEST-ONLY public-boundary check of the G2 handoff (EEDCS-48, D3 G2 HOST PREPARATION). It runs the ESTABLISHED ground characterisation driver (lua/ground-characterization.lua)
-- unchanged, in the actual eech-world host, against a campaign DLL built with the `ground-fixture` feature, and counts what the DLL asks the host's terrain provider, by operation.
-- The driver boots the module through require ("eech_dc") with the host's provider, runs healthy frames, calls the REAL original queries through engine:ground_probe and checks the
-- shared record is restored; this wrapper then judges the boundary (lua/ground-handoff-verdict.lua):
--
--   handoff=control    expected on the UNMOVED DLL: zero ground_below / ground_impact requests, point requests > 0.
--   handoff=required   the boundary the extraction must create: fails with GROUND-HANDOFF-RED on the unmoved DLL (a missing boundary, after a healthy run), passes once the DLL sends them.
--
-- The provider is the host's own, delegated to unchanged (every request and answer passes through; only the operation is counted). If the healthy driver itself fails, this fails with a
-- message that does not contain the marker. The ground_probe seam exercises the original queries, NOT wn_move's launch glue, so a pass on a later DLL is not the weapon-authority
-- witness; and a hybrid of the new host with an old fixture DLL is a test arrangement, never shipping evidence.
--
-- host.args:  handoff=control|required   handoff_out=<JSON summary path>   (this wrapper's; everything else is passed to the driver: root, out, scenario, seed, points, impacts, ...)
--             handoff_driver=<path>      (optional: a driver script other than the established one; the guards use a stand-in)
-- A suggested small case set: scenario=lebanon_retail points=200 impacts=50 repeats=1 seconds=3 after=1 (healthy 100 ms frames before and after the probe).

io.stdout:setvbuf ("no")

local args = host.args
local here = (debug.getinfo (1, "S").source:match ("^@?(.*[/\\])") or "")
local judge = dofile (here .. "ground-handoff-verdict.lua")

local mode = args.handoff
if mode ~= "control" and mode ~= "required" then
	error ("ground-handoff: handoff must be control or required, got '" .. tostring (mode) .. "'", 0)
end
local summary_path = args.handoff_out
if type (summary_path) ~= "string" or summary_path == "" then error ("ground-handoff: handoff_out=<JSON summary path> is required", 0) end
local factory = host.terrain_provider
if factory == nil then
	error ("ground-handoff: this host has no host.terrain_provider, so there is no boundary to observe (the original host, not the host-terrain one)", 0)
end
local driver = args.handoff_driver or (here .. "ground-characterization.lua")

local driver_args = {}
for key, value in pairs (args) do
	if key ~= "handoff" and key ~= "handoff_out" and key ~= "handoff_driver" then driver_args [key] = value end
end

local JSON_SHORT = { ["\b"] = "\\b", ["\t"] = "\\t", ["\n"] = "\\n", ["\f"] = "\\f", ["\r"] = "\\r", ["\""] = "\\\"", ["\\"] = "\\\\" }
local function json_string (text)
	local escaped = string.gsub (text, "[%z\1-\31\"\\]", function (c) return JSON_SHORT [c] or string.format ("\\u%04x", string.byte (c)) end)
	return "\"" .. escaped .. "\""
end

local counts = { point = 0, fast = 0, map = 0, ground_below = 0, ground_impact = 0, other = 0 }
local function summary (healthy, passed, message)
	local file = assert (io.open (summary_path, "w"))
	file:write (string.format ("{\"event\":\"ground_handoff\",\"handoff\":%s,\"healthy\":%s,\"passed\":%s,\"point\":%d,\"fast\":%d,\"map\":%d,\"ground_below\":%d,\"ground_impact\":%d,\"other\":%d,\"message\":%s}\n",
		json_string (mode), healthy and "true" or "false", passed and "true" or "false", counts.point, counts.fast, counts.map, counts.ground_below, counts.ground_impact,
		counts.other, json_string (message)))
	file:close ()
end

host.args = driver_args
host.terrain_provider = function (...)
	local real = factory (...)
	return function (request)
		local op = request.op
		if counts [op] == nil or op == "other" then op = "other" end
		counts [op] = counts [op] + 1
		return real (request)
	end
end

local ok, failure = pcall (dofile, driver)
host.terrain_provider = factory
host.args = args

if not ok then
	local message = "ground-handoff: the healthy driver failed before the boundary could be judged: " .. tostring (failure)
	summary (false, false, message)
	error (message, 0)
end

local passed, message = judge.verdict (mode, counts)
summary (true, passed, message)
if not passed then error (message, 0) end
io.stderr:write (message .. "\n")
