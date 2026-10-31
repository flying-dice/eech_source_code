-- TEST-ONLY control of the ORIGINAL ambient sound routine on a WIDE session child list (EEDCS-51, D3). SOURCE PREPARED: UNBUILT, UNRUN, UNREVIEWED. Needs a DLL built with the `env-fixture` feature.
--
-- update_session_sound_effects walks the session's WHOLE special-effect child list. Nothing in the game limits how many terrain ambient sound children a session holds (create_local_sound_effect_entity
-- attaches any subtype to any parent; the compiled comms receiver creates a received child through the same path). This driver uses the fixture's group 12 (csrc/eech_env_fixture.c env_wide_case), which
-- creates ADDITIONAL terrain ambient children with the REAL constructor until the session holds 16, then 17, then 20 of them (duplicates of every class, each with its own distinct previous amplitude),
-- and calls the REAL routine with the REAL camera at radar altitude 140 outdoors (its height 140 over its cached elevation 0; the echo is read back and must be 430c0000) and a frame delta of 0. The driver
-- recomputes no amplitude. What it checks is the harness and one property of the routine that the setup makes unambiguous and that a skipped child CANNOT satisfy: with delta 0 the smoothing neither adds
-- nor subtracts, every previous value j/128 is inside the bound [0, 0.3] (all are below 0.16), and the altitude factor 1 - 140 / 280 is EXACTLY 0.5 (also the factor of the original's half-altitude cases),
-- so a child that was PROCESSED holds exactly HALF its own previous word, j/256, while a child that was skipped, dropped or given another child's value keeps j/128 or something else. (An identity setup,
-- delta 0 at altitude 0, would leave a skipped child indistinguishable from a processed one.) ALL children, in list order, each with its own previous amplitude: none dropped, reordered or re-valued.
--
-- The cases run in ONE fresh process, in order; the process is the isolation boundary (entities cannot be removed), so the run ends here. If the engine stops in a case (the routine's fatal error) the
-- result is the engine's error, reported as `engine_failure` with its first line, and the driver stops with an error beginning `env-sound-wide:` naming the case. Speaker, network delivery, the player
-- and the renderer are not exercised and not claimed. Nothing is a pass until Root has run and reviewed it.
--
-- host.args: root, out (JSON Lines path), scenario (lebanon_retail default | georgia_retail)

io.stdout:setvbuf ("no")

local args = host.args
local function refuse (message) error ("env-sound-wide: " .. message, 0) end
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
-- every write and flush is checked: a failed one is a failure of the witness
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
emit { event = "boot", scenario = scenario_name, seed = seed, feature = "env-fixture", group = 12 }

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
local function first_line (text) return (string.match (tostring (text), "[^\r\n]+") or tostring (text)) end
local function lines_of (text)
	local lines = {}
	for line in string.gmatch (text, "[^\n]+") do lines [#lines + 1] = line end
	return lines
end
local function hex_exact (value, digits) return type (value) == "string" and #value == digits and string.find (value, "^[0-9a-f]+$") ~= nil end

-- `pos:entity_index:sub_type:name:word,...`
local function list_of (value, what)
	local list = {}
	if value == nil then hard (what .. " has no child list") end
	for entry in string.gmatch (value, "[^,]+") do
		local pos, id, sub, name, word = string.match (entry, "^(%d+):(%d+):(%d+):([%w_]+):(%x+)$")
		if not pos or not hex_exact (word, 8) then hard (what .. " has a malformed child entry '" .. entry .. "' (position:entity:sub type:class:eight hex digits)") end
		list [#list + 1] = { pos = tonumber (pos), id = tonumber (id), sub = tonumber (sub), name = name, word = word }
	end
	return list
end
-- the IEEE word of a positive dyadic float (the fixture sets (j + 1) / 128 for the j-th terrain child)
local function word_of (v)
	local m, e = math.frexp (v)
	local exponent, mantissa = e - 1 + 127, (m * 2 - 1) * 8388608
	return string.format ("%08x", exponent * 8388608 + mantissa)
end
local TERRAIN = { sea = true, jungle = true, amb1 = true, amb2 = true, amb3 = true, amb4 = true }

local function ask (index)
	local ok, text = pcall (engine.env_case, engine, 12, index)
	if not ok then return nil, first_line (text) end
	if string.find (text, "env-case-error", 1, true) then hard ("env_case(12, " .. index .. ") refused: " .. text) end
	return lines_of (text)
end

local plan = ask (0)
if not plan then hard ("env_case(12, 0) failed") end
local p = fields (plan [1] or "")
if not string.find (plan [1] or "", "^env%-wide%-plan version=1 ") or p.camera ~= "1" or p.natural_present ~= "1" or p.process_isolated ~= "required" then
	hard ("the plan is not a version 1 wide-list plan of a session with its camera and its ambient children: " .. tostring (plan [1]))
end
local targets = {}
for value in string.gmatch (p.targets or "", "%d+") do targets [#targets + 1] = tonumber (value) end
local cases = tonumber (p.cases)
if not cases or cases ~= #targets or cases < 1 then hard ("the plan's cases and targets disagree: " .. plan [1]) end
emit { event = "plan", raw = plan [1], cases = cases, targets = p.targets, children = tonumber (p.children), terrain_children = tonumber (p.terrain_children) }
for i = 2, #plan do
	local f = fields (plan [i])
	if not string.find (plan [i], "^env%-wide%-unsupported ") then hard ("an unexpected plan line: " .. plan [i]) end
	emit { event = "unsupported", item = f.item, reason = f.reason }
end

local terrain_before = tonumber (p.terrain_children)
for index = 1, cases do
	local lines, failure = ask (index)
	if not lines then
		-- the engine stopped in the routine: the result is its error, never a repaired or skipped case
		emit { event = "engine_failure", i = index, terrain_target = targets [index], reason = failure }
		refuse ("CASE " .. index .. " (" .. targets [index] .. " terrain ambient children) FAILED in the engine: " .. failure)
	end
	local head = lines [1] or ""
	local h = fields (head)
	if not string.find (head, "^env%-wide kind=control ") or tonumber (h.i) ~= index then hard ("case " .. index .. " does not begin with its control row: " .. head) end
	if h.refused ~= nil then hard ("CONTROL NOT RUN: case " .. index .. " was refused by the fixture: " .. head) end
	local after_row, end_row = lines [2], lines [3]
	if #lines ~= 3 or not string.find (after_row, "^env%-wide%-after ") or not string.find (end_row, "^env%-wide%-end ") then hard ("case " .. index .. " is not a control row, an after row and an end row: " .. table.concat (lines, " | ")) end
	local a, e = fields (after_row), fields (end_row)
	if tonumber (a.i) ~= index or tonumber (e.i) ~= index then hard ("case " .. index .. " has rows of another case: " .. after_row .. " | " .. end_row) end
	-- the isolation is stated, not assumed: entities were created and cannot be put back
	if e.changed ~= "1" or e.process_isolated ~= "required" or e.heap_restored ~= "0" or e.amplitudes_restored ~= "0" or e.session_camera_cloud_cockpit_scratch_restored ~= "1" then
		hard ("case " .. index .. " does not state its isolation and what it restored: " .. end_row)
	end
	local target, created, terrain, children = tonumber (h.target_terrain), tonumber (h.created), tonumber (h.terrain), tonumber (h.children)
	-- the real camera height produced the radar altitude the factor 0.5 needs, and the delta is the zero the smoothing needs
	if h.alt_w ~= "430c0000" or h.delta_w ~= "00000000" then hard ("case " .. index .. " was not run at radar altitude 140 and frame delta 0: " .. head) end
	if target ~= targets [index] or terrain ~= target or created ~= target - terrain_before or not children then hard ("case " .. index .. " counts do not fit the plan: " .. head) end
	local before, after = list_of (h.before, "case " .. index .. " before"), list_of (a.children, "case " .. index .. " after")
	if #before ~= children or #after ~= children then hard ("case " .. index .. " lists " .. #before .. " and " .. #after .. " children for " .. children .. ": " .. head) end
	local ids, terrain_seen, j = {}, 0, 0
	for k = 1, children do
		local b, f = before [k], after [k]
		-- the same child in the same place: nothing dropped, added or reordered by the routine
		if b.pos ~= f.pos or b.id ~= f.id or b.sub ~= f.sub or b.name ~= f.name or b.pos ~= k - 1 then hard ("case " .. index .. " child " .. k .. " is not the same child in the same place before and after: " .. before [k].id .. " / " .. after [k].id) end
		if ids [b.id] then hard ("case " .. index .. " lists entity " .. b.id .. " twice") end
		ids [b.id] = true
		if TERRAIN [b.name] then
			terrain_seen = terrain_seen + 1
			j = j + 1
			-- the setup the fixture says it made: this child's own previous amplification (j + 1) / 128
			if b.word ~= word_of (j / 128) then hard ("FIXTURE DEFECT: case " .. index .. " terrain child " .. j .. " started at " .. b.word .. ", not " .. word_of (j / 128)) end
			-- ALL processed, each with its own value (delta 0, altitude 140, every previous inside the bound): HALF the child's own previous word, j/256; a skipped child keeps j/128
			if f.word ~= word_of (j / 256) then hard ("CASE " .. index .. ": terrain child " .. j .. " (entity " .. b.id .. ") held " .. b.word .. " and holds " .. f.word .. ", not half of it (" .. word_of (j / 256) .. "): it was not processed with its own previous amplitude") end
		end
	end
	if terrain_seen ~= target then hard ("case " .. index .. " lists " .. terrain_seen .. " terrain children, not " .. target) end
	local created_ids = 0
	if h.created_ids ~= "none" then
		for id in string.gmatch (h.created_ids, "%d+") do
			created_ids = created_ids + 1
			if not ids [tonumber (id)] then hard ("case " .. index .. " created entity " .. id .. " is not in the child list") end
		end
	end
	if created_ids ~= created then hard ("case " .. index .. " reports " .. created .. " created children and " .. created_ids .. " ids") end
	if a.rng_unchanged ~= "0" and a.rng_unchanged ~= "1" then hard ("case " .. index .. " has no 0/1 rng_unchanged fact: " .. after_row) end
	emit { event = "case", i = index, terrain_target = target, terrain_before = terrain_before, created = created, created_ids = h.created_ids, children = children, delta_w = h.delta_w, alt_w = h.alt_w,
		rng_unchanged = a.rng_unchanged, before = h.before, after = a.children, raw = table.concat (lines, "\n") }
	terrain_before = target
end

emit { event = "summary", cases = cases, targets = p.targets }
local closed, close_error = out:close ()
if not closed then refuse ("OUTPUT FAILURE: closing " .. out_path .. " failed: " .. tostring (close_error)) end
io.stderr:write (string.format ("env-sound-wide: scenario=%s cases=%d targets=%s\n", scenario_name, cases, p.targets))
