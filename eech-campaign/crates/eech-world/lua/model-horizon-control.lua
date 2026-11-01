-- Core loader-detachment control (EEDCS-50, D3). TEST-ONLY: it needs a module built with the `model-host-fixture` feature (the hosted engine plus the fixture entries of
-- csrc/eech_model_host_fixture.c) and the reference host (EECH_WORLD). It makes two measurements of ONE natural retail boot and asks for nothing else (no frame is run):
--
--   1. RESIDENCY. After the healthy boot it reads the model-host counters report. Its first row is the closed catalogue / presence counters (the exact C field list below), its
--      `model-core-residency` row counts, over the actual rows of the campaign's own object table, how many still hold each of the four physical buffers (points, faces, face point
--      references, surface point references) next to how many rows carry the file's true point count, and its `model-core-horizon` row counts the horizon images the boot took through
--      the pre-processed `.bin` branch (point colours and fog only, no point read) and through the uncached branch. The report must be EXACTLY those three rows in the C grammar (names,
--      field order, decimal integers as C's %d and %ld write them, within 0..2147483647) and COHERENT (metadata_rows <= rows, each buffer count <= rows, presence_mismatches <=
--      presence_checked). It passes only if rows > 0, metadata_rows > 0, all four buffer counts are 0, cached > 0 and uncached == 0. Anything else - a malformed or incoherent report, or
--      resident buffers, or a boot that did not use the cached images - fails here and the unsupported reader is NOT launched.
--      The integer bound 2147483647 is deliberate: C long is 32 bits on the Windows native control, and Lua numbers are doubles, so no larger value is believed.
--   2. AN UNSUPPORTED READER, CALLED FOR REAL. Only after a passing residency does it call the fixture entry that runs the REAL `add_3d_horizon_image` on the campaign's own horizon
--      environment with a basename no installation has (no retail file or cache is touched, renamed or deleted, and no image is made). That image has no pre-processed `.bin` (the
--      reader asks the file system for it first, then takes its uncached branch), where the hosted engine stops by name BEFORE any TGA I/O or geometry read. It passes only if the entry
--      FAILS with the Engine-owned fatal error whose whole first line is exactly the one the engine writes for this operation (EXPECTED_STOP below: the engine prefix, the named
--      geometry marker, the operation and the fixed reason). A bare or quoted marker, another prefix, a setup, file, TGA or device failure, or a return from the entry is not coverage.
--      The engine is poisoned by that stop and is NOT asked again: the stop is the primary failure and nothing is read after it.
--
-- host.args: root (installation root, required), out (the JSON Lines file, required; created exclusively), scenario (lebanon_retail, the default, or georgia_retail)
--
-- The fixture entries are methods of the BOOTED engine, so a feature-off module (no `model_host_counters` or `model_host_horizon_refusal`) is recognised after the boot, before any
-- question, and reported as NOT RUN: never a zero-success.

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
-- the report grammar (csrc/eech_model_host_fixture.c eech_model_host_counters_run): three rows, in this order, each `name key=value ...` with decimal integers exactly as C's %d
-- and %ld write them. tests/model_horizon_control.rs checks that these names and field lists are the C emitter's.

local INT_MAX = 2147483647

local HOST_KEYS = { "catalogue", "scene", "triangle", "found", "end", "absent", "bounds_reads", "bounds_without_lines_reads", "radius_reads", "presence_checked", "presence_mismatches" }
local RESIDENCY_KEYS = { "rows", "metadata_rows", "points", "faces", "point_refs", "surface_points" }
local HORIZON_KEYS = { "cached", "uncached" }

local function malformed (message) error ({ malformed = message }, 0) end

local function int (text, what)
	if type (text) ~= "string" or #text > 10 or not (text == "0" or string.match (text, "^[1-9]%d*$")) then malformed (what .. " is not a non-negative decimal integer: " .. tostring (text)) end
	local number = tonumber (text)
	if number > INT_MAX then malformed (what .. " is outside the C domain 0.." .. INT_MAX .. ": " .. text) end
	return number
end

-- one row in the exact C grammar: the name, then exactly `keys` in order, one space between words, nothing else; the values are returned as checked integers
local function row (line, name, keys)
	local fields = {}
	local n = 0
	if string.sub (line, 1, #name + 1) ~= name .. " " then malformed ("expected a " .. name .. " row, got: " .. tostring (line)) end
	for key, value in string.gmatch (string.sub (line, #name + 1), "%s+([%w_]+)=(%S+)") do
		n = n + 1
		if keys [n] ~= key then malformed (string.format ("%s field %d is %s, expected %s", name, n, key, tostring (keys [n]))) end
		fields [key] = value
	end
	if n ~= #keys then malformed (string.format ("%s has %d fields, expected %d", name, n, #keys)) end
	local rebuilt = name
	for _, key in ipairs (keys) do rebuilt = rebuilt .. " " .. key .. "=" .. fields [key] end
	if rebuilt ~= line then malformed (name .. " row is not in the exact grammar: " .. line) end
	local values = {}
	for _, key in ipairs (keys) do values [key] = int (fields [key], name .. " " .. key) end
	return values
end

local function parse_counters (text)
	if type (text) ~= "string" then malformed ("the counters report is not a string") end
	local lines = {}
	for line in string.gmatch (text, "([^\n]*)\n") do lines [#lines + 1] = line end
	if #lines ~= 3 or string.sub (text, -1) ~= "\n" or select (2, string.gsub (text, "\n", "")) ~= 3 then malformed (string.format ("the counters report has %d lines, expected 3", #lines)) end
	local counters = row (lines [1], "model-host", HOST_KEYS)
	local residency = row (lines [2], "model-core-residency", RESIDENCY_KEYS)
	local horizon = row (lines [3], "model-core-horizon", HORIZON_KEYS)
	-- coherence: what the rows count cannot exceed what they are counted over
	if counters.presence_mismatches > counters.presence_checked then malformed ("presence_mismatches exceeds presence_checked") end
	if residency.metadata_rows > residency.rows then malformed ("metadata_rows exceeds rows") end
	for _, key in ipairs { "points", "faces", "point_refs", "surface_points" } do
		if residency [key] > residency.rows then malformed (key .. " exceeds rows") end
	end
	return residency, horizon
end

-- The whole first line of the Engine-owned error the stop produces (the engine's own prefix, then the host model's message for this operation): the C text is
-- mb_hostmodel.c campaign_model_geometry_reader's format with the operation of the staged guard LU1 (build/patches.rs); tests/model_horizon_control.rs checks both against it.
local STOP_PREFIX = "runtime error: eech: EECH fatal error: host model: "
local STOP_MARKER = "[model-provider:object.geometry] "
local STOP_OPERATION = "add_3d_horizon_image (an uncached horizon image: no pre-processed .bin of the object's point count for this image)"
local STOP_REASON = " is a legacy visual / display reader of a model's point, face, face-point-reference or surface-point buffer, "
	.. "which the hosted campaign no longer holds and does not serve (no renderer, cockpit display or HUD is supported)"
local EXPECTED_STOP = STOP_PREFIX .. STOP_MARKER .. STOP_OPERATION .. STOP_REASON

-- ---------------------------------------------------------------------------------------------------------------------------------
-- the run. The output is created here and is closed on every exit.

local out_file, why = host.create_exclusive (out_path)
if not out_file then error ("model-horizon-control: " .. tostring (why), 0) end
local out = json.wrap (out_file, out_path)
local function emit (row_) out.emit (row_) end

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
		emit { event = "not_run", reason = "the booted module has no model-host-fixture entries" }
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
	if not (residency_ok and horizon_ok) then
		-- the evidence the stop would be read against is not valid: the unsupported reader is not launched
		emit { event = "done", complete = false, residency = residency_ok, horizon_cache = horizon_ok, refusal = false, malformed = false, reader_launched = false }
		if not residency_ok then error ("model-horizon-control: NOT COMPLETE: the residency is not (rows > 0, metadata_rows > 0, all four buffers 0); the reader was not launched", 0) end
		error ("model-horizon-control: NOT COMPLETE: the boot did not take only the cached horizon branch; the reader was not launched", 0)
	end

	-- 2. the unsupported reader, called for real; the stop is the primary failure and nothing is asked after it
	local stopped, message = pcall (engine.model_host_horizon_refusal, engine)
	local refused, matched = false, false
	if stopped then
		emit { event = "horizon_refusal", refused = false, returned = tostring (message) }
	else
		local line = first_line (message)
		refused = true
		matched = line == EXPECTED_STOP
		emit { event = "horizon_refusal", refused = true, matched = matched, primary = line }
	end

	local complete = refused and matched
	emit { event = "done", complete = complete, residency = true, horizon_cache = true, refusal = complete, malformed = false, reader_launched = true }
	if not refused then error ("model-horizon-control: NOT COMPLETE: the uncached horizon reader did not stop", 0) end
	if not matched then error ("model-horizon-control: NOT COMPLETE: the reader stopped, but not with the Engine-owned named guard at add_3d_horizon_image before image I/O", 0) end
end

local ran, err = pcall (run)
local closed, close_err = pcall (out.close)
if not ran then
	if not closed then err = tostring (err) .. "; additionally the output could not be closed: " .. first_line (close_err) end
	error (err, 0)
end
if not closed then error ("model-horizon-control: the output could not be closed: " .. first_line (close_err), 0) end
