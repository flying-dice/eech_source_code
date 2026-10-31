-- TEST-ONLY characterisation of the ORIGINAL terrain-sector surface enumeration (EEDCS-51, D3). SOURCE PREPARED: UNBUILT, UNRUN, UNREVIEWED. Needs a DLL built with the `env-fixture` feature.
--
-- The sound routine (ss_updt.c update_session_sound_effects) clamps the camera to the map and calls get_terrain_3d_types_in_sector (modules/3d/terrain/terrdata.c) every frame: it selects a
-- sector, clears the shared int array terrain_types_in_sector, flags the sector's first surface and the surface of EVERY surface_change polygon, and returns 1 plus the number of transitions
-- (NOT the number of distinct flags). The observer already records the hash of that scratch inside real frames (docs/d3-environment-observer.md); this driver adds only what it lacks, the surface-
-- enumeration semantics: the return value against the complete flags, the sector the original selected, the reset between calls, the interleave behaviour and the boundary samples. It uses the
-- fixture's group 10 (csrc/eech_env_fixture.c env_sector_case), which calls the ORIGINAL on real loaded sectors and restores the scratch; nothing is recomputed by the driver.
--
-- Boot only, no frame is run (the loaded terrain is all the routine reads). One JSON Lines row per sample, written as the original answered; a sample the original cannot be asked (a sector outside the
-- loaded grid, no surface table, a surface id outside the terrain types) is a `refused` row with the reason. No assets and no raw coordinates belong in Git: the output is private evidence, compared by
-- Root BEFORE and AFTER any host cutover with the same command on the same retail root.
--
-- host.args: root, out (JSON Lines path), scenario (lebanon_retail default | georgia_retail)
-- Exit: 0 when every sample was answered or rejected as described and every scratch was restored; an error beginning `env-sector:` otherwise (a witness failure, never an expected result).

io.stdout:setvbuf ("no")

local args = host.args
local function refuse (message) error ("env-sector: " .. message, 0) end
local root = args.root
if type (root) ~= "string" or root == "" then refuse ("root=<installation root> is required") end
local out_path = args.out
if type (out_path) ~= "string" or out_path == "" then refuse ("out=<JSON Lines path> is required") end
local scenarios = {
	lebanon_retail = { map = "..\\common\\maps\\map5", campaign = "LEBANON.CHC" },
	georgia_retail = { map = "..\\common\\maps\\map3", campaign = "GEORGIA.CHC" },
}
local scenario_name = args.scenario == nil and "lebanon_retail" or args.scenario
local scenario = scenarios [scenario_name]
if scenario == nil then refuse ("scenario must be lebanon_retail or georgia_retail, got '" .. tostring (scenario_name) .. "'") end
local seed = 1
local out = assert (io.open (out_path, "w"))

local escapes = { ['"'] = '\\"', ['\\'] = '\\\\', ['\n'] = '\\n', ['\r'] = '\\r', ['\t'] = '\\t' }
local function json_string (s)
	return '"' .. (string.gsub (s, '[%c"\\\128-\255]', function (c) return escapes [c] or string.format ("\\u%04x", string.byte (c)) end)) .. '"'
end
-- every write and flush is checked: a failed one is a failure of the witness, even after the last row
local function emit (t)
	local parts = {}
	for k, v in pairs (t) do
		local value
		if type (v) == "string" then value = json_string (v)
		elseif type (v) == "boolean" then value = v and "true" or "false"
		else value = tostring (v) end
		parts [#parts + 1] = json_string (k) .. ":" .. value
	end
	table.sort (parts)
	local ok, err = out:write ("{" .. table.concat (parts, ",") .. "}\n")
	if not ok then refuse ("OUTPUT FAILURE: writing " .. out_path .. " failed: " .. tostring (err)) end
	ok, err = out:flush ()
	if not ok then refuse ("OUTPUT FAILURE: flushing " .. out_path .. " failed: " .. tostring (err)) end
end
local function hard (message)
	pcall (emit, { event = "witness_failure", reason = message })
	refuse (message)
end

local dc = require ("eech_dc")
dc.prepare_installation (root)
local engine = dc.boot {
	install_root = root, map = scenario.map, terrain = host.terrain_provider (root, scenario.map), environment = host.env_provider (), campaign_directory = "camp01", campaign = scenario.campaign,
	gunship = "apache", seed = seed, bind_mobile = host.bind_mobile, move_vehicle = host.move_vehicle, observe_loss = host.observe_loss,
	supply_reached = host.supply_reached, line_of_sight = host.line_of_sight, sense = host.sense, aim = host.aim,
}
emit { event = "boot", scenario = scenario_name, seed = seed, feature = "env-fixture", group = 10 }

-- the genuine refusal: a DLL built without the feature has no such method, and nothing is then reported as measured
local present, method = pcall (function () return engine.env_case end)
if not present or type (method) ~= "function" then
	emit { event = "refusal", reason = "env_case_unavailable", detail = "the module was built without the env-fixture feature" }
	refuse ("the engine has no env_case method: build eech-dc with the env-fixture feature")
end

local function fields (line)
	local t = {}
	for k, v in string.gmatch (line, "(%S+)=(%S+)") do t [k] = v end
	return t
end
local function ask (index)
	local ok, text = pcall (engine.env_case, engine, 10, index)
	if not ok then hard ("env_case(10, " .. index .. ") failed: " .. tostring (text)) end
	if string.find (text, "env-case-error", 1, true) then hard ("env_case(10, " .. index .. ") refused: " .. text) end
	local line = string.match (text, "^([^\n]*)\n?$")
	if not line then hard ("env_case(10, " .. index .. ") did not return exactly one line: " .. text) end
	return line
end

-- the plan
local plan_line = ask (0)
local plan = fields (plan_line)
if not string.find (plan_line, "^env%-sector%-plan version=1 ") then hard ("the plan line is not an env-sector-plan version 1 line: " .. plan_line) end
local cases, types, xmax, zmax = tonumber (plan.cases), tonumber (plan.terrain_types), tonumber (plan.sector_x_max), tonumber (plan.sector_z_max)
if plan.terrain_loaded ~= "1" or not cases or cases < 1 or cases > 192 or cases ~= math.floor (cases) or not types or types < 1 or types > 256 then
	hard ("the plan does not describe a loaded terrain and a finite case list: " .. plan_line)
end
emit { event = "plan", raw = plan_line, cases = cases, terrain_types = types, sector_x_max = xmax, sector_z_max = zmax }

-- the samples
local answered, refused_count, defects = 0, 0, 0
local reasons = {}
for index = 1, cases do
	local line = ask (index)
	local f = fields (line)
	if f.kind ~= "original" or tonumber (f.i) ~= index or not f.label or not f.x_w or not f.z_w then hard ("sample " .. index .. " is not an original sample row of that index: " .. line) end
	if f.refused ~= nil then
		-- the original was NOT called: a reason, never a repaired answer
		if f.returned ~= nil or f.flags ~= nil then hard ("sample " .. index .. " is both refused and answered: " .. line) end
		refused_count = refused_count + 1
		reasons [f.refused] = (reasons [f.refused] or 0) + 1
		emit { event = "sample", i = index, label = f.label, x_w = f.x_w, z_w = f.z_w, sector_x = tonumber (f.sector_x), sector_z = tonumber (f.sector_z), polygons = tonumber (f.polygons), refused = f.refused, raw = line }
	else
		local returned, distinct, nonbinary = tonumber (f.returned), tonumber (f.distinct_flags), tonumber (f.nonbinary)
		local flags = f.flags
		if not returned or not distinct or not nonbinary or not flags or #flags ~= types or string.find (flags, "[^01?]") then hard ("sample " .. index .. " has no complete flags of the plan's width: " .. line) end
		local ones, questions = 0, 0
		for c in string.gmatch (flags, ".") do
			if c == "1" then ones = ones + 1 elseif c == "?" then questions = questions + 1 end
		end
		-- the fields count the same scratch: the flags the original left set (1 and any other nonzero), the others
		if distinct ~= ones + questions or nonbinary ~= questions then hard ("sample " .. index .. " counts disagree with its own flags: " .. line) end
		if returned < 1 then hard ("sample " .. index .. " returned " .. returned .. ": the original returns 1 plus the transitions, never less than 1: " .. line) end
		for _, name in ipairs { "repeat_equal", "sentinel_cleared", "restored" } do
			if f [name] ~= "0" and f [name] ~= "1" then hard ("sample " .. index .. " has no 0/1 " .. name .. ": " .. line) end
		end
		if f.interleave ~= "done" and f.interleave ~= "na" then hard ("sample " .. index .. " has no interleave state: " .. line) end
		if (f.interleave == "na") ~= (f.interleave_equal == "-1") then hard ("sample " .. index .. " interleave state and result disagree: " .. line) end
		-- a scratch that was not put back is a defect of THIS FIXTURE, never a result of the original
		if f.restored ~= "1" then hard ("FIXTURE DEFECT: the scratch was not restored after sample " .. index .. ": " .. line) end
		if f.repeat_equal ~= "1" or f.sentinel_cleared ~= "1" or f.interleave_equal == "0" then defects = defects + 1 end
		answered = answered + 1
		emit {
			event = "sample", i = index, label = f.label, x_w = f.x_w, z_w = f.z_w, sector_x = tonumber (f.sector_x), sector_z = tonumber (f.sector_z), polygons = tonumber (f.polygons),
			returned = returned, distinct_flags = distinct, nonbinary = nonbinary, flags = flags, repeat_equal = f.repeat_equal, sentinel_cleared = f.sentinel_cleared,
			interleave = f.interleave, interleave_equal = f.interleave_equal, restored = f.restored, raw = line,
		}
	end
end

local reason_text = {}
for reason, n in pairs (reasons) do reason_text [#reason_text + 1] = reason .. "=" .. n end
table.sort (reason_text)
-- `anomalies` counts answered samples whose repeat, sentinel clearing or interleave differed: facts about the original, reported and never repaired
emit { event = "summary", cases = cases, answered = answered, refused = refused_count, refused_reasons = table.concat (reason_text, ","), anomalies = defects }
local closed, close_error = out:close ()
if not closed then refuse ("OUTPUT FAILURE: closing " .. out_path .. " failed: " .. tostring (close_error)) end
io.stderr:write (string.format ("env-sector: scenario=%s cases=%d answered=%d refused=%d anomalies=%d\n", scenario_name, cases, answered, refused_count, defects))
