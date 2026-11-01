-- Original route-geometry observer (EEDCS-50, D3). TEST-ONLY: it needs a module built with the `route-fixture` feature (the ORIGINAL airfield route parser and its route-object
-- lookup, observed by csrc/eech_route_fixture.c) and the reference host (EECH_WORLD), and it measures what the DLL's own loader and parser did for ONE natural retail boot.
--
-- It boots the campaign exactly as lua/campaign.lua does (the same preparation, the reference arithmetic profile, the same boot configuration, seed 1) and asks for NOTHING
-- else: no frame is run (the airfield populations, and with them every route, are read while the campaign loads), no entity is created and no setting is changed. After the
-- boot it reads the observer's summary and then every recorded parser call, once each, and writes them as JSON Lines, the native report text verbatim in each row.
--
-- host.args: root (installation root, required), out (the JSON Lines file, required, must not exist), scenario (lebanon_retail, the default, or georgia_retail)
--
-- WHAT A PASS MEANS. The run passes only if the observer says `route-complete value=1` (at least one call was recorded, none was dropped, orphaned, interrupted or left open, every
-- table is within its capacity) AND every call's own report says complete=1 and ends where it began. An empty boot (no parser call), a full table, a cut call or a report that
-- does not fit is written as it is, followed by a `done` row with complete=false, and the script then FAILS: it is never reported as a measured original. A reached route object is
-- not proof that every route object of the installation was parsed, and this script says nothing about the host's model service or about anything but the parser's own tables.
-- After any failure the engine is not asked again.

io.stdout:setvbuf ("no")

local here = debug.getinfo (1, "S").source:match ("^@?(.*[/\\])") or ""
local json = dofile (here .. "json-lines.lua")

local args = host.args

-- ---------------------------------------------------------------------------------------------------------------------------------
-- every argument is checked, strictly, BEFORE the output is opened, the installation is prepared or anything is booted

local scenarios = {
	lebanon_retail = { map = "..\\common\\maps\\map5", campaign = "LEBANON.CHC" },
	georgia_retail = { map = "..\\common\\maps\\map3", campaign = "GEORGIA.CHC" },
}

local function fail_args (message) error ("route-characterization: " .. message, 0) end

local known = { root = true, out = true, scenario = true }
for key in pairs (args) do
	if not known [key] then fail_args ("unknown argument " .. tostring (key)) end
end
local root = args.root
if type (root) ~= "string" or root == "" then fail_args ("root=<installation root> is required") end
local out_path = args.out
if type (out_path) ~= "string" or out_path == "" then fail_args ("out=<JSON Lines file> is required") end
local scenario_name = args.scenario or "lebanon_retail"
local scenario = scenarios [scenario_name]
if not scenario then fail_args ("scenario=lebanon_retail|georgia_retail") end

-- ---------------------------------------------------------------------------------------------------------------------------------
-- parsing of the observer's own text (it is `key=value` lines; nothing here interprets geometry)

local function fields (line)
	local t = {}
	for key, value in string.gmatch (line, "([%w_]+)=(%S+)") do t [key] = value end
	return t
end

-- the single line of `text` that starts with `prefix` followed by a space; nil if there is none, an error if there are several
local function one_line (text, prefix)
	local found
	for line in string.gmatch (text, "[^\n]+") do
		if string.sub (line, 1, #prefix + 1) == prefix .. " " then
			if found then error ("route-characterization: more than one '" .. prefix .. "' line in the report", 0) end
			found = line
		end
	end
	return found
end

local function number_field (t, key, line)
	local n = t [key] and string.match (t [key], "^%d+$") and tonumber (t [key])
	if not n then error ("route-characterization: field " .. key .. " is missing or not a whole number in: " .. tostring (line), 0) end
	return n
end

-- ---------------------------------------------------------------------------------------------------------------------------------
-- the boot: the same preparation, arithmetic profile and configuration as lua/campaign.lua (seed 1; no frame is run)

local out = json.open (out_path)
local function emit (row) out.emit (row) end

local function first_line (message)
	return (string.match (tostring (message), "^[^\n]*"))
end

local dc = require ("eech_dc")
dc.prepare_installation (root)
-- the reference host's arithmetic profile, selected right after the installation is prepared and before the boot (src/reference_arithmetic.rs)
host.use_reference_arithmetic ()

emit { event = "header", scenario = scenario_name, seed = 1, frames = 0, observer = "route-fixture", note = "natural boot only; no frame is run" }

local ok, engine = pcall (dc.boot, {
	install_root = root, map = scenario.map, terrain = host.terrain_provider (root, scenario.map), campaign_directory = "camp01", campaign = scenario.campaign, gunship = "apache",
	seed = 1, bind_mobile = host.bind_mobile, move_vehicle = host.move_vehicle, observe_loss = host.observe_loss,
	supply_reached = host.supply_reached, line_of_sight = host.line_of_sight, model = host.model, sense = host.sense, aim = host.aim,
})
if not ok then
	local message = first_line (engine)
	emit { event = "boot_failed", error = message }
	out.close ()
	error ("route-characterization: boot failed: " .. message, 0)
end
emit { event = "boot", booted = true }

-- ---------------------------------------------------------------------------------------------------------------------------------
-- the summary, then each recorded call once

local function ask (what, call)
	local good, text = pcall (call)
	if not good then
		local message = first_line (text)
		emit { event = "report_failed", what = what, error = message }
		out.close ()
		error ("route-characterization: " .. what .. " failed: " .. message .. " (the engine is not asked again)", 0)
	end
	if type (text) ~= "string" then
		emit { event = "report_failed", what = what, error = "the report is not a string" }
		out.close ()
		error ("route-characterization: " .. what .. " returned a " .. type (text), 0)
	end
	return text
end

local summary = ask ("route_summary", function () return engine:route_summary () end)
emit { event = "route_summary", text = summary }

local function fail_after_rows (message)
	emit { event = "done", complete = false, error = message }
	out.close ()
	error ("route-characterization: " .. message, 0)
end

local summary_line = one_line (summary, "route-summary")
local complete_line = one_line (summary, "route-complete")
if not summary_line or not complete_line then fail_after_rows ("the summary has no route-summary or route-complete line") end
local s = fields (summary_line)
local calls, capacity = number_field (s, "calls", summary_line), number_field (s, "call_capacity", summary_line)
if calls > capacity then fail_after_rows ("the summary reports more calls than its capacity") end
local summary_complete = fields (complete_line).value == "1"

local incomplete = 0
for index = 0, calls - 1 do
	local text = ask ("route_call " .. index, function () return engine:route_call (index) end)
	emit { event = "route_call", index = index, text = text }
	local header = one_line (text, "route-call")
	local tail = one_line (text, "route-end")
	local good = header ~= nil and tail ~= nil and fields (header).index == tostring (index) and fields (header).complete == "1" and fields (tail).index == tostring (index)
	if not good then incomplete = incomplete + 1 end
end

local complete = summary_complete and calls > 0 and incomplete == 0
emit { event = "done", complete = complete, calls = calls, incomplete_calls = incomplete, summary_complete = summary_complete }
out.close ()
if not complete then
	if calls == 0 then error ("route-characterization: NOT MEASURED: the boot recorded no route parser call", 0) end
	error (string.format ("route-characterization: NOT COMPLETE: summary complete=%s, %d of %d recorded calls incomplete", tostring (summary_complete), incomplete, calls), 0)
end
