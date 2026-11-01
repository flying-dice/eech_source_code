-- TEST-ONLY DRIVER of the ORIGINAL non-session resumed-parent playback (EEDCS-51, D3). SOURCE PREPARED: UNBUILT, UNRUN, UNREVIEWED. CHARACTERISATION ONLY: it moves, removes and guards nothing.
--
-- WHAT IT ASKS. soundeff.c play_resumed_parent_sound, NON-SESSION branch, as the running engine makes it: the range from the main viewpoint to the parent's position (get_approx_3d_range) and then
-- play_local_entity_sound (en, vp, range), which is fn_play_local_entity_sound[type]. Whether that ever reaches anything but the dispatch table's default no-op, for any parent type, and what it computes when it is
-- called, is measured here from the running engine: the playback probe (ss_envfx.h ENV_FXP, csrc/eech_env_observer.c) keeps the table's default entry and a census of the table at the end of its default initialisation
-- and one record for every non-session call, and the fixture (csrc/eech_env_fixture.c, group 14 cases 18, 19 and 20) takes a census of the table at any moment and exercises the REAL collaborators on a real fixed-wing and a real
-- AI helicopter. The SESSION branch is already characterised and is not recorded. Nothing is read from a source file; a reading of the source is not a measurement and a family whose source is not compiled may still be reached
-- through a table entry some other code sets, which the census would show.
--
-- MODES (`mode=`):
--   natural   (default) the probe armed before boot, the census after boot and after every frame (a change in the table is a row of its own), the probe drained after boot and after every frame: the table as it stands
--             and the non-session calls the ordinary frames make. `seconds` of frames (0 to 600, default 0).
--   fixture   as natural, then case 19: pause and resume on a real fixed-wing and a real AI helicopter at three positions of the main viewpoint, everything it sets restored; the probe's phase-1 calls are joined, step by
--             step, to the fixture's own rows (type, entity, dispatch identity and the number of calls). A target that does not exist is REPORTED as absent; nothing is synthesised.
--   overload  as fixture with case 20, in which the fixture itself installs a recording function in the dispatch entries of the fixed-wing and helicopter types for the duration and puts the saved entries back. THE MEANINGFUL
--             RED: the census inside the overload must list both types as non-default, the probe's phase-1 calls must be flagged non-default, the recording function must have been given exactly those calls (type and range
--             word), and the entries must be restored; otherwise the driver cannot identify a live non-default consumer and FAILS.
--
-- WHAT IT DOES NOT. It computes no range and no expected value (a range word is the original's, copied), defines no oracle, hosts nothing, and says nothing about devices, handles, loaders, destroy, network, time, the
-- session branch or whether any sound plays. It computes nothing else either: every number is a count or a word read back.
--
-- EVIDENCE I/O. Three files are created EXCLUSIVELY through host.output_create_new before anything is booted: `out` (JSON Lines), `<out>.play.raw` (every probe report and drained page, exactly as the native wrote them)
-- and `<out>.case.raw` (every fixture report). Raw text is written and flushed BEFORE it is validated; every write, flush and close is checked. The grammar is closed (exact fields); a lost, truncated, unexpected or
-- incomplete record, a probe counter that is not zero (dropped, bad_points), a probe not armed before boot, a defaults census that was not taken, a fixture step that did not restore, and a restored state or
-- viewpoint that differs, are failures of the witness, never results.
--
-- ARGUMENTS (host.args): eech-world <this script> root=<installation root> out=<new JSON Lines path> [scenario=lebanon_retail|georgia_retail] [seconds=<0..600; default 0>] [frame_ms=<10..1000; default 100>]
--                                                 [mode=natural|fixture|overload]
-- Needs a module built with the env-observer and env-fixture features. One fresh process and root per run (the fixture cases mutate and restore; the overload case installs and restores a dispatch entry).

io.stdout:setvbuf ("no")

local args = host.args
local function refuse (message) error ("env-play-resumed-parent: " .. message, 0) end

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
local seconds = args.seconds == nil and 0 or tonumber (args.seconds)
if seconds == nil or seconds ~= math.floor (seconds) or seconds < 0 or seconds > 600 then refuse ("seconds must be an integer from 0 to 600, got '" .. tostring (args.seconds) .. "'") end
local frame_ms = args.frame_ms == nil and 100 or tonumber (args.frame_ms)
if frame_ms == nil or frame_ms ~= math.floor (frame_ms) or frame_ms < 10 or frame_ms > 1000 then refuse ("frame_ms must be an integer from 10 to 1000, got '" .. tostring (args.frame_ms) .. "'") end
local mode = args.mode == nil and "natural" or args.mode
if mode ~= "natural" and mode ~= "fixture" and mode ~= "overload" then refuse ("mode must be natural, fixture or overload, got '" .. tostring (args.mode) .. "'") end
local CASE = { fixture = 19, overload = 20 }

-- every evidence file is created exclusively, here, before anything is booted
local create = host.output_create_new
if type (create) ~= "function" then refuse ("this host has no host.output_create_new: evidence files cannot be created exclusively, and the driver does not fall back to a truncating open") end
local paths = { out = out_path, play = out_path .. ".play.raw", case = out_path .. ".case.raw" }
local files, created = {}, {}
for _, name in ipairs ({ "out", "play", "case" }) do
	local file, why = create (paths [name])
	if file == nil then
		for _, done in ipairs (created) do pcall (files [done].close, files [done]) end
		refuse ("OUTPUT FAILURE: creating " .. paths [name] .. " failed: " .. tostring (why) .. " (evidence is created exclusively and never overwritten; the same file under two names is refused)")
	end
	files [name] = file
	created [#created + 1] = name
end
local out = files.out

local escapes = { ['"'] = '\\"', ['\\'] = '\\\\', ['\n'] = '\\n', ['\r'] = '\\r', ['\t'] = '\\t' }
local function json_string (s)
	return '"' .. (string.gsub (s, '[%c"\\\128-\255]', function (c) return escapes [c] or string.format ("\\u%04x", string.byte (c)) end)) .. '"'
end
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
local function first_line (text) return string.match (tostring (text), "[^\r\n]+") or tostring (text) end

-- the native text, kept whole BEFORE any of it is validated
local function keep (name, text)
	local file = files [name]
	local ok, err = file:write (text)
	if not ok then refuse ("OUTPUT FAILURE: writing " .. paths [name] .. " failed: " .. tostring (err)) end
	ok, err = file:flush ()
	if not ok then refuse ("OUTPUT FAILURE: flushing " .. paths [name] .. " failed: " .. tostring (err)) end
end

local dc = require ("eech_dc")
dc.prepare_installation (root)
if type (dc.env_play_arm) ~= "function" then
	emit { event = "refusal", reason = "play_probe_unavailable", detail = "the module was built without the env-observer feature" }
	refuse ("the module has no env_play_arm function: build eech-dc with the env-observer feature")
end
-- ONLY the playback probe is armed, before boot
dc.env_play_arm (true)
local engine = dc.boot {
	install_root = root, map = scenario.map, terrain = host.terrain_provider (root, scenario.map), environment = host.env_provider (), campaign_directory = "camp01", campaign = scenario.campaign,
	gunship = "apache", seed = 1, bind_mobile = host.bind_mobile, move_vehicle = host.move_vehicle, observe_loss = host.observe_loss,
	supply_reached = host.supply_reached, line_of_sight = host.line_of_sight, sense = host.sense, aim = host.aim,
}
emit { event = "boot", scenario = scenario_name, mode = mode, seconds = seconds, frame_ms = frame_ms, armed_by = "dc.env_play_arm only" }
for _, name in ipairs ({ "env_play_report", "env_play_events", "env_case" }) do
	local present, method = pcall (function () return engine [name] end)
	if not present or type (method) ~= "function" then
		emit { event = "refusal", reason = name .. "_unavailable", detail = "the module was built without the env-observer and env-fixture features" }
		refuse ("the engine has no " .. name .. " method: build eech-dc with the env-observer and env-fixture features")
	end
end

-- ---------------------------------------------------------------------------------------------------------------- the closed grammar
-- a `key=value` row with exactly these keys in this order (a value has no space); the map of its values
local function exact (line, event, keys, what)
	local tokens = {}
	for token in string.gmatch (line, "%S+") do tokens [#tokens + 1] = token end
	if tokens [1] ~= event then hard (what .. ": expected a " .. event .. " row: " .. line) end
	if #tokens ~= #keys + 1 then hard (what .. ": the " .. event .. " row has " .. (#tokens - 1) .. " fields, the protocol has " .. #keys .. ": " .. line) end
	local map = {}
	for position, key in ipairs (keys) do
		local k, v = string.match (tokens [position + 1], "^([%w_]+)=([^=]+)$")
		if k ~= key then hard (what .. ": field " .. position .. " of the " .. event .. " row is not " .. key .. ": " .. line) end
		map [key] = v
	end
	return map
end
local function integer (value, what, line)
	if type (value) ~= "string" or string.find (value, "^%-?%d+$") == nil or #value > 11 then hard (what .. " is not a plain integer (" .. tostring (value) .. "): " .. line) end
	return tonumber (value)
end
local function count (value, what, line)
	local n = integer (value, what, line)
	if n < 0 then hard (what .. " is negative: " .. line) end
	return n
end
local function word (value, what, line)
	if type (value) ~= "string" or string.find (value, "^%x%x%x%x%x%x%x%x$") == nil then hard (what .. " is not an eight digit word (" .. tostring (value) .. "): " .. line) end
	return value
end
local function type_list (value, what, line)
	if value == "-" then return {} end
	local list = {}
	for item in string.gmatch (value .. ",", "([^,]*),") do
		if string.find (item, "^%d+$") == nil then hard (what .. " has an item that is not an entity type (" .. item .. "): " .. line) end
		list [#list + 1] = tonumber (item)
	end
	return list
end
local function lines_of (text, what)
	if type (text) ~= "string" then hard (what .. " is not text") end
	if text ~= "" and string.sub (text, -1) ~= "\n" then hard (what .. " does not end with a newline (a truncated page)") end
	local lines = {}
	for line in string.gmatch (text, "([^\n]*)\n") do
		if line == "" or string.find (line, "[^\32-\126]") then hard (what .. " has an empty line or a byte that is not printable ASCII") end
		lines [#lines + 1] = line
	end
	return lines
end

local REPORT_KEYS = { "version", "armed", "phase", "capacity", "calls_total", "drained", "pending", "dropped", "bad_points", "defaults_taken", "defaults_entries", "defaults_nondefault", "defaults_types", "defaults_over" }
local CALL_KEYS = { "seq", "t", "phase", "ent", "type", "session", "default", "range_w", "rng" }
local CENSUS_KEYS = { "tag", "entries", "default", "nondefault", "types", "over" }
local FENV_ROUND = { [0] = true, [1024] = true, [2048] = true, [3072] = true }

local report_state = {}
local function report (when)
	local ok, text = pcall (engine.env_play_report, engine)
	if not ok then hard ("the play probe's report " .. when .. " FAILED in the engine: " .. first_line (text)) end
	keep ("play", "report " .. when .. "\n" .. text)
	local lines = lines_of (text, "the play probe's report")
	if #lines ~= 1 then hard ("the play probe's report is not one row: " .. tostring (text)) end
	local r = exact (lines [1], "env-play-report", REPORT_KEYS, "the play probe's report")
	local m = {
		version = count (r.version, "version", lines [1]), armed = count (r.armed, "armed", lines [1]), phase = count (r.phase, "phase", lines [1]), capacity = count (r.capacity, "capacity", lines [1]),
		calls_total = count (r.calls_total, "calls_total", lines [1]), drained = count (r.drained, "drained", lines [1]), pending = count (r.pending, "pending", lines [1]), dropped = count (r.dropped, "dropped", lines [1]),
		bad_points = count (r.bad_points, "bad_points", lines [1]), defaults_taken = count (r.defaults_taken, "defaults_taken", lines [1]), defaults_entries = count (r.defaults_entries, "defaults_entries", lines [1]),
		defaults_nondefault = count (r.defaults_nondefault, "defaults_nondefault", lines [1]), defaults_types = type_list (r.defaults_types, "defaults_types", lines [1]), defaults_over = count (r.defaults_over, "defaults_over", lines [1]),
	}
	if m.version ~= 1 then hard ("the play probe's report has another version: " .. lines [1]) end
	if m.armed ~= 1 then hard ("the play probe was not armed " .. when .. ": " .. lines [1]) end
	for _, key in ipairs ({ "dropped", "bad_points", "defaults_over" }) do
		if m [key] ~= 0 then hard ("the play probe's " .. key .. " is " .. m [key] .. " " .. when .. ": the capture is incomplete: " .. lines [1]) end
	end
	if m.defaults_taken ~= 1 then hard ("the dispatch table's defaults census was taken " .. m.defaults_taken .. " times where once is required (a probe armed after the defaults were set, or a second initialisation): " .. lines [1]) end
	if m.defaults_nondefault ~= #m.defaults_types then hard ("the defaults census counts " .. m.defaults_nondefault .. " non-default entries and lists " .. #m.defaults_types .. ": " .. lines [1]) end
	report_state = m
	return m, lines [1]
end

local totals = { calls = 0, phase0 = 0, phase1 = 0, nondefault = 0, nondefault_natural = 0, nondefault_controlled = 0 }
local last_seq, controlled = -1, {}
local natural_by_type = {}
local function drain (when)
	local ok, text = pcall (engine.env_play_events, engine)
	if not ok then hard ("draining the play probe " .. when .. " FAILED in the engine: " .. first_line (text)) end
	keep ("play", "events " .. when .. "\n" .. text)
	local calls = 0
	for _, line in ipairs (lines_of (text, "the play probe's events " .. when)) do
		local c = exact (line, "env-play-call", CALL_KEYS, "a play call " .. when)
		local n = {
			seq = count (c.seq, "seq", line), t = count (c.t, "t", line), phase = count (c.phase, "phase", line), ent = count (c.ent, "ent", line), type = count (c.type, "type", line),
			session = count (c.session, "session", line), default = count (c.default, "default", line), range_w = word (c.range_w, "range_w", line), rng = integer (c.rng, "rng", line),
		}
		if n.seq ~= last_seq + 1 then hard ("the play calls are out of sequence (expected " .. (last_seq + 1) .. "): " .. line) end
		last_seq = n.seq
		if n.session ~= 0 then hard ("a play call is flagged as the session's, which the probe does not record: " .. line) end
		if n.default ~= 0 and n.default ~= 1 then hard ("a play call's default flag is neither 0 nor 1: " .. line) end
		if n.phase ~= 0 and n.phase ~= 1 then hard ("a play call has another phase than 0 (natural) or 1 (the controlled fixture): " .. line) end
		if n.phase == 0 and (when == "after the fixture") then hard ("a natural call arrived in the drain after the fixture: " .. line) end
		calls = calls + 1
		totals.calls = totals.calls + 1
		if n.default == 0 then totals.nondefault = totals.nondefault + 1 end
		if n.phase == 0 then
			totals.phase0 = totals.phase0 + 1
			if n.default == 0 then totals.nondefault_natural = totals.nondefault_natural + 1 end
			natural_by_type [n.type] = (natural_by_type [n.type] or 0) + 1
		else
			totals.phase1 = totals.phase1 + 1
			if n.default == 0 then totals.nondefault_controlled = totals.nondefault_controlled + 1 end
			controlled [#controlled + 1] = n
		end
	end
	return calls
end

-- the census row of case 18, and its bookkeeping: the table now, and every change since the last one
local census_seen, census_now, census_changes = {}, nil, 0
local type_names = {}
local function census (tag)
	local ok, text = pcall (engine.env_case, engine, 14, 18)
	if not ok then
		emit { event = "engine_failure", i = 18, reason = first_line (text) }
		refuse ("env_case (14, 18) FAILED in the engine: " .. first_line (text))
	end
	if type (text) ~= "string" or text == "" or string.find (text, "env-case-error", 1, true) then hard ("env_case (14, 18) did not give a report: " .. first_line (text)) end
	keep ("case", "census " .. tag .. "\n" .. text)
	local lines = lines_of (text, "the census report")
	if #lines ~= 4 then hard ("the census report has " .. #lines .. " rows where 4 are required: " .. tostring (text)) end
	exact (lines [1], "env-play-plan", { "version", "case", "mutating", "natural", "probe", "armed" }, "the census plan")
	local t = exact (lines [2], "env-play-types", { "fixed_wing", "helicopter", "sound_effect", "session", "routed_vehicle", "ship_vehicle", "anti_aircraft", "entries" }, "the census types")
	for key, value in pairs (t) do type_names [key] = count (value, key, lines [2]) end
	local c = exact (lines [3], "env-play-census", CENSUS_KEYS, "the census")
	if c.default == "unknown" or c.nondefault == "unknown" then hard ("the census could not be taken: " .. lines [3]) end
	exact (lines [4], "env-play-end", { "case", "refused", "mutating" }, "the census end")
	local n = { entries = count (c.entries, "entries", lines [3]), default = count (c.default, "default", lines [3]), nondefault = count (c.nondefault, "nondefault", lines [3]), types = type_list (c.types, "types", lines [3]), over = count (c.over, "over", lines [3]) }
	if n.over ~= 0 or n.default + n.nondefault ~= n.entries or n.nondefault ~= #n.types then hard ("the census is not coherent: " .. lines [3]) end
	if n.entries ~= type_names.entries then hard ("the census entries differ from the types row: " .. lines [3]) end
	local key = c.types
	if census_now ~= key then
		census_changes = census_changes + 1
		emit { event = "census", tag = tag, nondefault = n.nondefault, types = key, entries = n.entries, changed = census_now ~= nil }
		census_seen [key] = true
		census_now = key
	end
	return n, key
end

-- ---------------------------------------------------------------------------------------------------------------- boot, the defaults census, the natural frames
local defaults = report ("after boot")
emit { event = "play_defaults", nondefault = defaults.defaults_nondefault, types = table.concat (defaults.defaults_types, ","), entries = defaults.defaults_entries }
local after_boot, after_boot_key = census ("after boot")
local boot_calls = drain ("after boot")
emit { event = "play_frame", frame = 0, calls = boot_calls }
local frames = math.floor (seconds * 1000 / frame_ms)
for n = 1, frames do
	local ok, err = pcall (engine.frame, engine, frame_ms)
	if not ok then
		emit { event = "engine_failure", frame = n, reason = first_line (err) }
		refuse ("frame " .. n .. " FAILED in the engine: " .. first_line (err))
	end
	census ("after frame " .. n)
	local calls = drain ("after frame " .. n)
	if calls > 0 then emit { event = "play_frame", frame = n, calls = calls } end
end
local natural_report = report ("after the natural frames")
if natural_report.calls_total ~= totals.calls or natural_report.drained ~= totals.calls or natural_report.pending ~= 0 then
	hard ("the probe's calls (" .. natural_report.calls_total .. " total, " .. natural_report.drained .. " drained, " .. natural_report.pending .. " pending) are not the " .. totals.calls .. " lines read")
end

-- ---------------------------------------------------------------------------------------------------------------- the controlled fixture
local fixture_facts = { steps = 0, step_calls = 0, targets_present = 0, recorded = 0, installed = 0 }
if mode ~= "natural" then
	local index = CASE [mode]
	local before_fixture_key = census_now
	local ok, text = pcall (engine.env_case, engine, 14, index)
	if not ok then
		emit { event = "engine_failure", i = index, reason = first_line (text) }
		refuse ("env_case (14, " .. index .. ") FAILED in the engine: " .. first_line (text))
	end
	if type (text) ~= "string" or text == "" or string.find (text, "env-case-error", 1, true) then hard ("env_case (14, " .. index .. ") did not give a report: " .. first_line (text)) end
	keep ("case", "case " .. index .. "\n" .. text)
	local rows = lines_of (text, "the fixture report")
	local at = 1
	local function next_row (event, keys, what)
		local line = rows [at]
		if line == nil then hard (what .. ": the report ends before the " .. event .. " row") end
		at = at + 1
		return exact (line, event, keys, what), line
	end
	local plan, plan_line
	if rows [1] ~= nil and string.sub (rows [1], 1, #"env-play-refused ") == "env-play-refused " then
		-- the fixture refused before it changed anything (it says why); the end row follows
		exact (rows [1], "env-play-refused", { "reason" }, "the fixture refusal")
		emit { event = "fixture_refused", reason = rows [1] }
		fixture_facts.refused = true
		at = 2
	else
		plan, plan_line = next_row ("env-play-plan", { "version", "case", "process_isolated", "mutating", "natural", "phase", "restores", "instrumented", "armed", "overload", "unit_offsets_w" }, "the fixture plan")
		if plan.instrumented ~= "1" or plan.armed ~= "1" then hard ("the fixture was not instrumented and armed: " .. plan_line) end
	end
	local targets, steps = {}, {}
	while not fixture_facts.refused and rows [at] and string.sub (rows [at], 1, #"env-play-target ") == "env-play-target " do
		local line = rows [at]
		local t
		if string.find (line, " present=0 ", 1, true) then
			t = exact (line, "env-play-target", { "kind", "present", "reason" }, "an absent fixture target")
			targets [t.kind] = { present = false }
		else
			t = exact (line, "env-play-target", { "kind", "present", "ent", "etype", "sub", "special_effect_children", "sound_children", "matching_children", "second_sub", "second_children", "default_entry" }, "a fixture target")
			targets [t.kind] = { present = true, ent = count (t.ent, "ent", line), etype = count (t.etype, "etype", line), matching = count (t.matching_children, "matching_children", line), second = count (t.second_children, "second_children", line), default_entry = count (t.default_entry, "default_entry", line) }
			fixture_facts.targets_present = fixture_facts.targets_present + 1
		end
		at = at + 1
	end
	if not fixture_facts.refused then
		if rows [at] and string.sub (rows [at], 1, #"env-play-census ") == "env-play-census " then
			-- inside the overload: the census of the dispatch table while the recording function is installed
			local c = exact (rows [at], "env-play-census", CENSUS_KEYS, "the census inside the overload")
			at = at + 1
			fixture_facts.inside_types = type_list (c.types, "types", rows [at - 1])
			fixture_facts.inside_nondefault = count (c.nondefault, "nondefault", rows [at - 1])
		end
		while rows [at] and string.sub (rows [at], 1, #"env-play-step ") == "env-play-step " do
			local line = rows [at]
			local s = exact (line, "env-play-step", { "kind", "ent", "etype", "sub", "pos", "offset_w", "default_entry", "pause_returned", "resume_returned", "calls", "rng_before", "rng_after", "round_before", "round_after", "excepts",
				"digest_before", "digest_paused", "digest_resumed", "digest_restored", "vp_restored", "state_restored", "before", "resumed" }, "a fixture step")
			local step = { kind = s.kind, ent = count (s.ent, "ent", line), etype = count (s.etype, "etype", line), pos = s.pos, default_entry = count (s.default_entry, "default_entry", line), calls = integer (s.calls, "calls", line),
				pause_returned = integer (s.pause_returned, "pause_returned", line), resume_returned = integer (s.resume_returned, "resume_returned", line), line = line }
			if step.calls < 0 then hard ("a fixture step has no probe call count (the build has no observer): " .. line) end
			if not targets [step.kind] or not targets [step.kind].present or targets [step.kind].ent ~= step.ent or targets [step.kind].etype ~= step.etype then hard ("a fixture step is not for a target the report named: " .. line) end
			if s.pos ~= "coincident" and s.pos ~= "middle" and s.pos ~= "outside" then hard ("a fixture step has an unknown position name: " .. line) end
			for _, key in ipairs ({ "round_before", "round_after" }) do
				if not FENV_ROUND [integer (s [key], key, line)] then hard ("a fixture step's " .. key .. " is not a standard fenv rounding mode: " .. line) end
			end
			if s.round_before ~= s.round_after then hard ("the rounding mode changed across a fixture step: " .. line) end
			if string.find (s.excepts, "^%x%x?$") == nil or tonumber (s.excepts, 16) > 63 then hard ("a fixture step's exception flags are not a standard x86 exception flag set: " .. line) end
			for _, key in ipairs ({ "digest_before", "digest_paused", "digest_resumed", "digest_restored" }) do word (s [key], key, line) end
			if s.vp_restored ~= "1" or s.state_restored ~= "1" or s.digest_before ~= s.digest_restored then hard ("a fixture step did not restore the main viewpoint and the children's state (restored vp " .. s.vp_restored .. ", state " .. s.state_restored .. "): " .. line) end
			steps [#steps + 1] = step
			fixture_facts.steps = fixture_facts.steps + 1
			fixture_facts.step_calls = fixture_facts.step_calls + step.calls
			at = at + 1
		end
		while rows [at] and string.sub (rows [at], 1, #"env-play-recorded ") == "env-play-recorded " do
			local line = rows [at]
			local r = exact (line, "env-play-recorded", { "n", "type", "range_w" }, "a recorded overload call")
			fixture_facts.recorded_rows = fixture_facts.recorded_rows or {}
			fixture_facts.recorded_rows [#fixture_facts.recorded_rows + 1] = { type = count (r.type, "type", line), range_w = word (r.range_w, "range_w", line) }
			at = at + 1
		end
		if rows [at] and string.sub (rows [at], 1, #"env-play-overload ") == "env-play-overload " then
			local o = exact (rows [at], "env-play-overload", { "installed", "recorded", "restored" }, "the overload row")
			fixture_facts.installed, fixture_facts.recorded = count (o.installed, "installed", rows [at]), count (o.recorded, "recorded", rows [at])
			if o.restored ~= "1" then hard ("the deliberate overload's dispatch entries were not restored: " .. rows [at]) end
			at = at + 1
		end
	end
	local e = rows [at]
	if e == nil then hard ("the fixture report has no end row") end
	exact (e, "env-play-end", fixture_facts.refused and { "case", "refused", "mutating", "steps" } or { "case", "refused", "mutating", "steps", "rng_start", "rng_end", "entity_index_unchanged", "local_index_unchanged", "stage_ok" }, "the fixture end")
	if not fixture_facts.refused and (string.find (e, " entity_index_unchanged=1 ", 1, true) == nil or string.find (e, " local_index_unchanged=1 ", 1, true) == nil or string.find (e, " stage_ok=1", 1, true) == nil) then
		hard ("the fixture end row does not show the entity indices unchanged and the staged report whole: " .. e)
	end
	if rows [at + 1] ~= nil then hard ("the fixture report has a row after its end row: " .. rows [at + 1]) end
	if mode == "overload" and fixture_facts.refused then hard ("the deliberate overload could not run (" .. rows [1] .. "): no live non-default consumer could be identified, which is a setup failure and not a result") end

	-- the controlled calls: drained after the fixture and joined, step by step, to the fixture's own rows
	local fixture_calls = drain ("after the fixture")
	local after = report ("after the fixture")
	if after.calls_total ~= totals.calls or after.drained ~= totals.calls or after.pending ~= 0 then hard ("the probe's counters do not match the lines read after the fixture: " .. tostring (after.calls_total) .. " / " .. tostring (after.drained)) end
	if #controlled ~= fixture_facts.step_calls then hard ("the fixture steps report " .. fixture_facts.step_calls .. " calls and the probe holds " .. #controlled .. " phase-1 calls") end
	local k = 0
	for _, step in ipairs (steps) do
		for _ = 1, step.calls do
			k = k + 1
			local c = controlled [k]
			if c.type ~= step.etype or c.ent ~= step.ent then hard ("probe call " .. c.seq .. " is for entity " .. c.ent .. " of type " .. c.type .. " where the fixture step has entity " .. step.ent .. " of type " .. step.etype .. ": " .. step.line) end
			if mode == "overload" then
				if c.default ~= 0 then hard ("a call inside the deliberate overload is flagged default (" .. c.seq .. "): the probe did not see the installed consumer") end
			elseif c.default ~= step.default_entry then
				hard ("probe call " .. c.seq .. " default flag " .. c.default .. " is not the fixture's dispatch identity " .. step.default_entry .. ": " .. step.line)
			end
		end
	end
	fixture_facts.controlled = fixture_calls
	if mode == "overload" then
		-- THE RED: a live non-default consumer must be identified from the census, the probe and the recording function together
		local need = { targets.fixed_wing and targets.fixed_wing.present and type_names.fixed_wing, targets.helicopter and targets.helicopter.present and type_names.helicopter }
		local listed = {}
		for _, t in ipairs (fixture_facts.inside_types or {}) do listed [t] = true end
		for _, t in ipairs (need) do
			if t and not listed [t] then hard ("the census inside the deliberate overload does not list entity type " .. t .. " as non-default: the live consumer was not identified") end
		end
		if fixture_facts.step_calls < 1 or totals.nondefault_controlled ~= fixture_facts.step_calls then hard ("the deliberate overload was not seen by the probe on every call (" .. totals.nondefault_controlled .. " non-default of " .. fixture_facts.step_calls .. ")") end
		if fixture_facts.recorded ~= fixture_facts.step_calls then hard ("the recording function was given " .. fixture_facts.recorded .. " calls where the probe saw " .. fixture_facts.step_calls) end
		local given = fixture_facts.recorded_rows or {}
		for i = 1, #given do
			if given [i].type ~= controlled [i].type or given [i].range_w ~= controlled [i].range_w then hard ("the recording function's call " .. i .. " is not the probe's call (type " .. given [i].type .. " range " .. given [i].range_w .. ")") end
		end
	end
	local after_fixture, after_key = census ("after the fixture")
	if after_key ~= before_fixture_key then hard ("the dispatch table after the fixture (" .. after_key .. ") is not as before it (" .. tostring (before_fixture_key) .. "): a restoration failure") end
end

-- ---------------------------------------------------------------------------------------------------------------- what the driver JOINS (labelled derived; never a measurement)
local natural_types = {}
for t, n in pairs (natural_by_type) do natural_types [#natural_types + 1] = t .. ":" .. n end
table.sort (natural_types)
local live = totals.nondefault > 0
emit {
	event = "derived", source = "the driver's join of the records above; not a measurement", mode = mode,
	defaults_nondefault = defaults.defaults_nondefault, defaults_types = table.concat (defaults.defaults_types, ","), census_after_boot_types = after_boot_key, census_final_types = census_now, census_changes = census_changes,
	natural_calls = totals.phase0, natural_calls_nondefault = totals.nondefault_natural, natural_calls_by_type = table.concat (natural_types, ","),
	controlled_steps = fixture_facts.steps, controlled_calls = totals.phase1, controlled_calls_nondefault = totals.nondefault_controlled, targets_present = fixture_facts.targets_present,
	live_non_default_consumer_called = live,
	coverage_note = "natural coverage is only the phase 0 calls; the phase 1 calls are controlled world input and are not natural coverage; a table entry that is non-default but never called is reported by the census and is not a call; the SESSION branch is not recorded; nothing here says any sound plays or that any family source is compiled or dormant",
}
emit { event = "result", mode = mode, calls = totals.calls }

for _, name in ipairs (created) do
	local closed, close_error = files [name]:close ()
	if not closed then refuse ("OUTPUT FAILURE: closing " .. paths [name] .. " failed: " .. tostring (close_error)) end
end
io.stderr:write (string.format ("env-play-resumed-parent: mode=%s calls=%d natural=%d controlled=%d live=%s\n", mode, totals.calls, totals.phase0, totals.phase1, tostring (live)))
