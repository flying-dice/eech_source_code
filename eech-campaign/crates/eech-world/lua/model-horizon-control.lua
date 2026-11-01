-- Core loader-detachment control (EEDCS-50, D3). TEST-ONLY: it needs a module built with the `model-host-fixture` feature (the hosted engine plus the fixture entries of
-- csrc/eech_model_host_fixture.c) and the reference host (EECH_WORLD). It makes two measurements of ONE natural retail boot and asks for nothing else (no frame is run):
--
--   1. RESIDENCY. After the healthy boot it reads the model-host counters report, whose `model-core-residency` row counts, over the actual rows of the campaign's own object table,
--      how many still hold each of the four physical buffers (points, faces, face point references, surface point references) next to how many rows carry the file's true point
--      count, and whose `model-core-horizon` row counts the horizon images the boot took through the pre-processed `.bin` branch (point colours and fog only, no point read) and
--      through the uncached branch. It passes only if rows > 0, metadata_rows > 0, all four buffer counts are 0, cached > 0 and uncached == 0. Nothing is inferred from the absence
--      of a row: a missing, duplicated or malformed row fails.
--   2. AN UNSUPPORTED READER, CALLED FOR REAL. It then calls the fixture entry that runs the REAL `add_3d_horizon_image` on the campaign's own horizon environment with a basename
--      no installation has (no retail file, no cache is touched, renamed or deleted, and no image is made). That image has no pre-processed `.bin`, so the real reader takes its
--      uncached branch, where the hosted engine stops by name BEFORE any image I/O or geometry read. It passes only if the entry FAILS with `[model-provider:object.geometry]`, names
--      `add_3d_horizon_image` and the uncached image, and names no image or file error. The engine is poisoned by that stop and is NOT asked again: the stop is the primary failure
--      and nothing is read after it. A failure from a missing file or a device, or an entry that returns, is not coverage of this reader.
--
-- host.args: root (installation root, required), out (the JSON Lines file, required; created exclusively), scenario (lebanon_retail, the default, or georgia_retail)
--
-- A feature-off module (no `model_host_counters` or `model_host_horizon_refusal`) is refused before anything is booted into evidence: NOT RUN, never zero-success.

io.stdout:setvbuf ("no")

local here = debug.getinfo (1, "S").source:match ("^@?(.*[/\\])") or ""
local json = dofile (here .. "json-lines.lua")

local args = host.args

local scenarios = {
	lebanon_retail = { map = "..\\common\\maps\\map5", campaign = "LEBANON.CHC" },
	georgia_retail = { map = "..\\common\\maps\\map3", campaign = "GEORGIA.CHC" },
}

local function fail_args (message) error ("model-horizon-control: " .. message, 0) end

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
if type (host.create_exclusive) ~= "function" then fail_args ("this host has no create_exclusive: an output that may never overwrite anything cannot be made safely") end

-- ---------------------------------------------------------------------------------------------------------------------------------
-- the report grammar (csrc/eech_model_host_fixture.c eech_model_host_counters_run): three lines, in this order, each `name key=value ...` with decimal integers exactly as C's %d
-- and %ld write them

local function malformed (message) error ({ malformed = message }, 0) end

local function int (text, what)
	if type (text) ~= "string" or #text > 11 or not (text == "0" or string.match (text, "^[1-9]%d*$")) then malformed (what .. " is not a non-negative decimal integer: " .. tostring (text)) end
	return tonumber (text)
end

local function words (line, name, keys)
	local row = {}
	local n = 0
	local event = string.match (line, "^(%S+)")
	if event ~= name then malformed ("expected a " .. name .. " row, got: " .. tostring (line)) end
	for key, value in string.gmatch (string.sub (line, #name + 1), "%s+([%w_]+)=(%S+)") do
		n = n + 1
		if keys [n] ~= key then malformed (string.format ("%s field %d is %s, expected %s", name, n, key, tostring (keys [n]))) end
		row [key] = value
	end
	if n ~= #keys then malformed (string.format ("%s has %d fields, expected %d", name, n, #keys)) end
	local rebuilt = name
	for _, key in ipairs (keys) do rebuilt = rebuilt .. " " .. key .. "=" .. row [key] end
	if rebuilt ~= line then malformed (name .. " row is not in the exact grammar: " .. line) end
	return row
end

local function parse_counters (text)
	if type (text) ~= "string" then malformed ("the counters report is not a string") end
	local lines = {}
	for line in string.gmatch (text, "([^\n]*)\n") do lines [#lines + 1] = line end
	if #lines ~= 3 or string.sub (text, -1) ~= "\n" then malformed (string.format ("the counters report has %d lines, expected 3", #lines)) end
	if not string.match (lines [1], "^model%-host catalogue=%d+ scene=%d+ ") then malformed ("the first line is not the model-host counters row") end
	local residency = words (lines [2], "model-core-residency", { "rows", "metadata_rows", "points", "faces", "point_refs", "surface_points" })
	local horizon = words (lines [3], "model-core-horizon", { "cached", "uncached" })
	local r, h = {}, {}
	for key, value in pairs (residency) do r [key] = int (value, "residency " .. key) end
	for key, value in pairs (horizon) do h [key] = int (value, "horizon " .. key) end
	return r, h
end

-- ---------------------------------------------------------------------------------------------------------------------------------
-- the run. The output is created here and is closed on every exit.

local out_file, why = host.create_exclusive (out_path)
if not out_file then error ("model-horizon-control: " .. tostring (why), 0) end
local out = json.wrap (out_file, out_path)
local function emit (row) out.emit (row) end

local function first_line (message) return (string.match (tostring (message), "^[^\n]*")) end

local function run ()
	local dc = require ("eech_dc")
	dc.prepare_installation (root)
	-- the reference host's arithmetic profile, selected right after the installation is prepared and before the boot (src/reference_arithmetic.rs)
	host.use_reference_arithmetic ()

	emit { event = "header", scenario = scenario_name, seed = 1, frames = 0, control = "model-host-fixture", note = "natural boot only; no frame is run" }

	local ok, engine = pcall (dc.boot, {
		install_root = root, map = scenario.map, terrain = host.terrain_provider (root, scenario.map), campaign_directory = "camp01", campaign = scenario.campaign, gunship = "apache",
		seed = 1, bind_mobile = host.bind_mobile, move_vehicle = host.move_vehicle, observe_loss = host.observe_loss,
		supply_reached = host.supply_reached, line_of_sight = host.line_of_sight, model = host.model, sense = host.sense, aim = host.aim,
	})
	if not ok then
		local message = first_line (engine)
		emit { event = "boot_failed", error = message }
		error ("model-horizon-control: boot failed: " .. message, 0)
	end
	emit { event = "boot", booted = true }

	if type (engine.model_host_counters) ~= "function" or type (engine.model_host_horizon_refusal) ~= "function" then
		emit { event = "not_run", reason = "the module has no model-host-fixture entries" }
		error ("model-horizon-control: NOT RUN: this module was not built with the model-host-fixture feature", 0)
	end

	-- 1. residency, from the healthy boot
	local good, text = pcall (engine.model_host_counters, engine)
	if not good then
		emit { event = "report_failed", what = "model_host_counters", error = first_line (text) }
		error ("model-horizon-control: model_host_counters failed: " .. first_line (text), 0)
	end
	emit { event = "counters", text = type (text) == "string" and text or tostring (text) }
	local parsed, residency, horizon = pcall (parse_counters, text)
	if not parsed then
		local reason = type (residency) == "table" and residency.malformed or tostring (residency)
		emit { event = "malformed", reason = reason }
		emit { event = "done", complete = false, malformed = true }
		error ("model-horizon-control: MALFORMED REPORT: " .. reason .. " (the engine is not asked again)", 0)
	end
	local residency_ok = residency.rows > 0 and residency.metadata_rows > 0 and residency.points == 0 and residency.faces == 0 and residency.point_refs == 0 and residency.surface_points == 0
	local horizon_ok = horizon.cached > 0 and horizon.uncached == 0
	emit { event = "residency", rows = residency.rows, metadata_rows = residency.metadata_rows, points = residency.points, faces = residency.faces, point_refs = residency.point_refs,
		surface_points = residency.surface_points, pass = residency_ok }
	emit { event = "horizon_cache", cached = horizon.cached, uncached = horizon.uncached, pass = horizon_ok }

	-- 2. the unsupported reader, called for real; the stop is the primary failure and nothing is asked after it
	local stopped, message = pcall (engine.model_host_horizon_refusal, engine)
	local refused, matched = false, false
	if stopped then
		emit { event = "horizon_refusal", refused = false, returned = tostring (message) }
	else
		local line = first_line (message)
		refused = true
		matched = string.find (line, "[model-provider:object.geometry]", 1, true) ~= nil and string.find (line, "add_3d_horizon_image", 1, true) ~= nil
			and string.find (line, "uncached horizon image", 1, true) ~= nil and not string.find (string.lower (line), "tga", 1, true)
			and not string.find (string.lower (line), "no such file", 1, true) and not string.find (string.lower (line), "unable to read", 1, true)
		emit { event = "horizon_refusal", refused = true, matched = matched, primary = line }
	end

	local complete = residency_ok and horizon_ok and refused and matched
	emit { event = "done", complete = complete, residency = residency_ok, horizon_cache = horizon_ok, refusal = refused and matched, malformed = false }
	if not residency_ok then error ("model-horizon-control: NOT COMPLETE: the residency is not (rows > 0, metadata_rows > 0, all four buffers 0)", 0) end
	if not horizon_ok then error ("model-horizon-control: NOT COMPLETE: the boot did not take only the cached horizon branch", 0) end
	if not refused then error ("model-horizon-control: NOT COMPLETE: the uncached horizon reader did not stop", 0) end
	if not matched then error ("model-horizon-control: NOT COMPLETE: the reader stopped, but not at the named guard before image I/O", 0) end
end

local ran, err = pcall (run)
local closed, close_err = pcall (out.close)
if not ran then
	if not closed then err = tostring (err) .. "; additionally the output could not be closed: " .. first_line (close_err) end
	error (err, 0)
end
if not closed then error ("model-horizon-control: the output could not be closed: " .. first_line (close_err), 0) end
