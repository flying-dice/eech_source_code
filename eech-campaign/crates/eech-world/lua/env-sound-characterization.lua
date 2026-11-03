-- TEST-ONLY characterisation of the ORIGINAL ambient sound routine of the session (EEDCS-51, D3). SOURCE PREPARED: UNBUILT, UNRUN, UNREVIEWED. Needs a DLL built with the `env-fixture` feature.
--
-- update_session_sound_effects (aphavoc/source/entity/special/session/ss_updt.c, UNCHANGED) runs in every server update. For the camera (or the gunship in the cockpit) it computes the rain
-- levels from the session weather, the light and heavy wind levels from the session wind, reads the terrain scratch of the sector under the camera (already host-owned), and sets
-- FLOAT_TYPE_AMPLIFICATION on the ten ambient sound children the campaign attaches to the session at boot; the six terrain children are SMOOTHED (their previous value plus or minus half the
-- frame delta, bounded, scaled by altitude and the cockpit). This driver uses the fixture's group 11 (csrc/eech_env_fixture.c env_sound_case), which sets the controlled inputs the routine reads
-- (the camera position and cloud base, the session weather and wind, the frame delta, the children's initial amplification), calls the REAL routine on the REAL session, camera, terrain and
-- children, prints the amplification of every child as IEEE words, and puts everything back. The driver recomputes NOTHING: it checks only the harness (the plan, the ten children, the setup
-- the fixture says it made, the restoration) and writes what the original answered. The observer's existing records of the same routine inside real frames, and group 10's sector semantics,
-- are REUSED as they are (docs/d3-environment-observer.md, docs/d3-environment-sector-surface.md).
--
-- Boot only, no frame is run. One JSON Lines row per fact. A case the fixture cannot ask (a surface the bounded probe did not find, no gunship for the cockpit branch) is a `refused` row with the
-- reason; the original is then NOT called. The output is private evidence: no assets and no raw coordinates belong in Git. Root runs it on the same retail root BEFORE any physical move of the
-- routine and compares the same command AFTER; nothing is a pass until Root has run and reviewed it.
--
-- host.args: root, out (JSON Lines path), scenario (lebanon_retail default | georgia_retail)
-- Exit: 0 when the plan is complete and every answered case was restored as the fixture says; an error beginning `env-sound:` otherwise (a witness failure, never an expected result).
-- Record grammar (a malformed record is a witness failure whatever it might be read as): every amplitude word is eight lowercase hex digits (`na` only for a declared non-sound child), the two
-- restoration fingerprints are sixteen, each answered case has exactly ONE session row of its own index, and the original's boolean facts (rng_unchanged, session_unchanged_by_the_routine) are 0 or 1;
-- a genuine 0 is an anomaly of the original, counted and reported. The fingerprints are bookkeeping of the fixture's rendered state, NOT a proof of the CPU floating-point state.

io.stdout:setvbuf ("no")

local args = host.args
local function refuse (message) error ("env-sound: " .. message, 0) end
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
host.use_reference_arithmetic () -- the reference host's arithmetic profile, selected right after the installation is prepared (src/reference_arithmetic.rs)
local engine = dc.boot {
	install_root = root, map = scenario.map, terrain = host.terrain_provider (root, scenario.map), environment = host.env_provider (), campaign_directory = "camp01", campaign = scenario.campaign,
	gunship = "apache", seed = seed, bind_mobile = host.bind_mobile, move_vehicle = host.move_vehicle, observe_loss = host.observe_loss,
	supply_reached = host.supply_reached, line_of_sight = host.line_of_sight, model = host.model, sense = host.sense, aim = host.aim,
}
emit { event = "boot", scenario = scenario_name, seed = seed, feature = "env-fixture", group = 11 }

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
local function lines_of (text)
	local lines = {}
	for line in string.gmatch (text, "[^\n]+") do lines [#lines + 1] = line end
	return lines
end
local function ask (index)
	local ok, text = pcall (engine.env_case, engine, 11, index)
	if not ok then hard ("env_case(11, " .. index .. ") failed: " .. tostring (text)) end
	if string.find (text, "env-case-error", 1, true) then hard ("env_case(11, " .. index .. ") refused: " .. text) end
	-- the text as the fixture gave it, without the line ending, and its lines
	local lines = lines_of (text)
	return table.concat (lines, "\n"), lines
end
-- the fixture's serializer prints every IEEE word with %08x and every state fingerprint with %016llx: lowercase hex, an exact number of digits. A shorter, longer or otherwise different
-- token is a malformed witness record, whatever the number it might be read as.
local function hex_exact (value, digits)
	return type (value) == "string" and #value == digits and string.find (value, "^[0-9a-f]+$") ~= nil
end
-- a boolean fact the fixture reports about the original (`0` or `1`, nothing else): the value is the original's, the grammar is the witness's
local function bool_fact (value)
	return value == "0" or value == "1"
end
-- `idx:sub_type:name:word,...` (or `none`): the word is eight hex digits; `na` only for a child the fixture declares NOT a sound child (its class name is `other`)
local function children_of (value, what)
	local list = {}
	if value == nil then hard (what .. " has no children field") end
	if value == "none" then return list end
	for entry in string.gmatch (value, "[^,]+") do
		local idx, sub, name, word = string.match (entry, "^(%d+):(%d+):([%w_]+):(%x+)$")
		if idx then
			if not hex_exact (word, 8) then hard (what .. " has a child amplitude word that is not eight lowercase hex digits in '" .. entry .. "'") end
		else
			idx, sub, name = string.match (entry, "^(%d+):(%d+):([%w_]+):na$")
			word = "na"
			if idx and name ~= "other" then hard (what .. " has no amplitude word for the sound child in '" .. entry .. "': na is only for a declared non-sound child") end
		end
		if not idx then hard (what .. " has a malformed child entry '" .. entry .. "'") end
		list [#list + 1] = { idx = tonumber (idx), sub = tonumber (sub), name = name, word = word }
	end
	return list
end
local function same_order (a, b)
	if #a ~= #b then return false end
	for i = 1, #a do
		if a [i].idx ~= b [i].idx or a [i].sub ~= b [i].sub or a [i].name ~= b [i].name then return false end
	end
	return true
end

-- the plan: one plan line, one line per site, the children in their list order, the unsupported items
local plan_text, plan_lines = ask (0)
local plan = fields (plan_lines [1] or "")
if not string.find (plan_lines [1] or "", "^env%-sound%-plan version=1 ") then hard ("the plan line is not an env-sound-plan version 1 line: " .. tostring (plan_lines [1])) end
local cases, child_count = tonumber (plan.cases), tonumber (plan.children)
if plan.camera ~= "1" or not cases or cases < 1 or cases > 64 or cases ~= math.floor (cases) or not child_count then hard ("the plan does not describe a camera and a finite case list: " .. plan_lines [1]) end
emit { event = "plan", raw = plan_lines [1], cases = cases, children = child_count, complete = plan.complete, gunship = plan.gunship, in_cockpit = plan.in_cockpit, cloud_base_w = plan.cloud_base_w,
	probes = plan.probes, terrain_loaded = plan.terrain_loaded }
local order
for i = 2, #plan_lines do
	local line = plan_lines [i]
	if string.find (line, "^env%-sound%-site ") then
		local f = fields (line)
		emit { event = "site", site = tonumber (f.site), found = f.found, x_w = f.x_w, z_w = f.z_w, sector_x = tonumber (f.sector_x), sector_z = tonumber (f.sector_z), raw = line }
	elseif string.find (line, "^env%-sound%-children ") then
		local f = fields (line)
		order = children_of (f.order, "the plan")
		emit { event = "children_order", count = tonumber (f.count), order = f.order, raw = line }
	elseif string.find (line, "^env%-sound%-unsupported ") then
		local f = fields (line)
		emit { event = "unsupported", item = f.item, reason = f.reason }
	else
		hard ("an unexpected plan line: " .. line)
	end
end
-- the prerequisite: the ten ambient sound children the campaign attaches at boot. Without them the amplitudes cannot be observed; that is reported, never worked around
if plan.complete ~= "1" or not order or #order ~= child_count then
	emit { event = "prerequisite_unavailable", reason = "the session does not hold each of the ten ambient sound children exactly once", children = child_count }
	hard ("PREREQUISITE: the session holds " .. tostring (child_count) .. " special-effect children, not the ten ambient sound children attached at boot (see the plan row)")
end

local SENTINEL_WORD = "3f200000" -- 0.625, the initial amplitude the fixture gives the rain and wind children
local answered, refused_count, anomalies, calls_seen = 0, 0, 0, 0
local reasons = {}
for index = 1, cases do
	local text, lines = ask (index)
	local head = lines [1] or ""
	local f = fields (head)
	if not string.find (head, "^env%-sound kind=original ") or tonumber (f.i) ~= index or not f.label then hard ("case " .. index .. " does not begin with an env-sound original row of that index: " .. head) end
	local last = lines [#lines] or ""
	local e = fields (last)
	if not string.find (last, "^env%-sound%-end ") or tonumber (e.i) ~= index then hard ("case " .. index .. " does not end with its env-sound-end row: " .. last) end
	if f.refused ~= nil then
		-- the original was NOT called: a reason, nothing changed
		if #lines ~= 2 or e.restored ~= "1" or e.nothing_changed ~= "1" then hard ("case " .. index .. " is refused but did not say that nothing changed: " .. text) end
		refused_count = refused_count + 1
		reasons [f.refused] = (reasons [f.refused] or 0) + 1
		emit { event = "case", i = index, label = f.label, refused = f.refused, raw = text }
	else
		local before = children_of (f.children_before, "case " .. index)
		if not same_order (before, order) then hard ("case " .. index .. " lists the children in another order than the plan: " .. head) end
		-- the fixture's setup, checked against what the fixture says it asked for (a fixture defect, never a result): the rain and wind children at the sentinel, the terrain children at amp0
		for _, child in ipairs (before) do
			local klass = child.name
			local expected = (klass == "light_rain" or klass == "heavy_rain" or klass == "light_wind" or klass == "heavy_wind") and SENTINEL_WORD or f.amp0_w
			if child.word ~= expected then hard ("FIXTURE DEFECT: case " .. index .. " child " .. klass .. " started at " .. child.word .. ", not " .. tostring (expected) .. ": " .. head) end
		end
		local ncalls = tonumber (f.calls)
		if not ncalls or ncalls < 1 or ncalls > 8 then hard ("case " .. index .. " has no finite number of calls: " .. head) end
		local call_rows, session_rows = {}, {}
		for i = 2, #lines - 1 do
			if string.find (lines [i], "^env%-sound%-call ") then call_rows [#call_rows + 1] = lines [i]
			elseif string.find (lines [i], "^env%-sound%-session ") then session_rows [#session_rows + 1] = lines [i]
			else hard ("case " .. index .. " has an unexpected row: " .. lines [i]) end
		end
		if #call_rows ~= ncalls or #session_rows ~= 1 then hard ("case " .. index .. " has " .. #call_rows .. " call rows for " .. ncalls .. " calls and " .. #session_rows .. " session rows, not one: " .. text) end
		-- the session row and the restoration record are checked BEFORE any row of the case is written: exactly one session row, of THIS case, with a 0/1 fact of the original; two exact 16-digit fingerprints
		local session_row = session_rows [1]
		local s = fields (session_row)
		if not string.find (s.i or "", "^%d+$") or tonumber (s.i) ~= index then hard ("case " .. index .. " has a session row of another case or without its index: " .. session_row) end
		if not bool_fact (s.session_unchanged_by_the_routine) then hard ("case " .. index .. " has a session row without a 0 or 1 fact: " .. session_row) end
		if not hex_exact (e.fp_before, 16) or not hex_exact (e.fp_after, 16) then hard ("case " .. index .. " has a restoration record without two exact 16-digit state fingerprints: " .. last) end
		for n, row in ipairs (call_rows) do
			local r = fields (row)
			local kids = children_of (r.children, "case " .. index .. " call " .. n)
			if tonumber (r.i) ~= index or tonumber (r.call) ~= n or not same_order (kids, order) or not r.scratch_flags or #r.scratch_flags ~= tonumber (plan.terrain_types) or string.find (r.scratch_flags, "[^01?]") then
				hard ("case " .. index .. " call " .. n .. " is not a call row of that case in the plan's child order with the complete scratch flags: " .. row)
			end
			if not bool_fact (r.rng_unchanged) then hard ("case " .. index .. " call " .. n .. " has no 0 or 1 rng_unchanged fact: " .. row) end
			if r.rng_unchanged ~= "1" then anomalies = anomalies + 1 end
			calls_seen = calls_seen + 1
			emit { event = "call", i = index, call = n, children = r.children, scratch_flags = r.scratch_flags, rng = tonumber (r.rng), rng_unchanged = r.rng_unchanged }
		end
		-- a changed session record (0) is a genuine fact about the original: counted, reported, never repaired
		if s.session_unchanged_by_the_routine ~= "1" then anomalies = anomalies + 1 end
		-- a state that was not put back is a defect of THIS FIXTURE, never a result of the original
		for _, name in ipairs { "restored", "children_restored", "scratch_restored", "camera_restored", "cloud_restored", "in_cockpit_restored" } do
			if e [name] ~= "1" then hard ("FIXTURE DEFECT: " .. name .. " is not 1 after case " .. index .. ": " .. last) end
		end
		if e.fp_before ~= e.fp_after then hard ("FIXTURE DEFECT: the fingerprint of the rendered state changed across case " .. index .. ": " .. last) end
		answered = answered + 1
		emit {
			event = "case", i = index, label = f.label, site = tonumber (f.site), site_found = f.site_found, cockpit = tonumber (f.cockpit), calls = ncalls, want_alt_w = f.want_alt_w, want_zone_w = f.want_zone_w,
			gust_w = f.gust_w, min_w = f.min_w, max_w = f.max_w, amp0_w = f.amp0_w, delta_w = f.delta_w, cloud_mode = tonumber (f.cloud_mode), cloud_base_w = f.cloud_base_w, model = f.model,
			ring_w = f.ring_w, in_cwm = tonumber (f.in_cwm), in_twm = tonumber (f.in_twm), in_ts_w = f.in_ts_w, source = f.source, x_w = f.x_w, y_w = f.y_w, z_w = f.z_w, alt_w = f.alt_w, cwm = tonumber (f.cwm),
			twm = tonumber (f.twm), ts_w = f.ts_w, wind_speed_w = f.wind_speed_w, wind_range_w = f.wind_range_w, children_before = f.children_before, session_unchanged = s.session_unchanged_by_the_routine,
			restored = e.restored, raw = text,
		}
	end
end

local reason_text = {}
for reason, n in pairs (reasons) do reason_text [#reason_text + 1] = reason .. "=" .. n end
table.sort (reason_text)
-- `anomalies` counts facts about the original (a shared-random draw, a change to the session record) reported and never repaired
emit { event = "summary", cases = cases, answered = answered, refused = refused_count, refused_reasons = table.concat (reason_text, ","), calls = calls_seen, anomalies = anomalies }
local closed, close_error = out:close ()
if not closed then refuse ("OUTPUT FAILURE: closing " .. out_path .. " failed: " .. tostring (close_error)) end
io.stderr:write (string.format ("env-sound: scenario=%s cases=%d answered=%d refused=%d calls=%d anomalies=%d\n", scenario_name, cases, answered, refused_count, calls_seen, anomalies))
