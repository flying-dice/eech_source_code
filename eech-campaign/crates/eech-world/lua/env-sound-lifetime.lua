-- TEST-ONLY PRIMITIVE CONTROL of the ORIGINAL sound effect entity and its lifetime (EEDCS-51, D3). SOURCE PREPARED: UNBUILT, UNRUN, UNREVIEWED. Needs a DLL built with the `env-observer` feature (the sound
-- entity probe, ss_envfx.h ENV_FXE, lives in csrc/eech_env_observer.c); an older or smaller build refuses.
--
-- The natural run (lua/env-characterization.lua with observer=on sound=on) shows what the ORIGINAL lightning routine created and how the real server update routine aged and expired its two sound entities in
-- the frames. It cannot show the situations the frames never reach: the CLIENT update routine (the headless process is the server), the clamp of an aged lifetime, the facts the playback branches need.
-- This driver runs ONE of the three controls of the fixture's group 14 (csrc/eech_env_fixture.c env_snd_case) in a FRESH process, because an entity cannot be put back (the process is the isolation boundary):
--
--   1 server_branch_ages_then_expires   ONE sound entity is built by the real constructor with the lightning routine's arguments; the real server update routine runs at a small delta and then at a delta far beyond the lifetime
--   2 client_branch_ages_and_clamps     the same entity; the real client update routine runs at the large delta and then at the small one: the lifetime is clamped, the entity is not killed
--   3 playback_prerequisites            nothing is created: the facts of the sound database and the sound system, and the explicit statement that the playback-handle branches are NOT exercised
--
--   4 to 10  CONTROLLED-DATA cases (csrc/eech_env_fixture.c env_snd_data_case), distinct from the unchanged retail baseline whose sample sizes measured 0: the sample DESCRIPTOR numbers are set to named controlled values
--            (restored before any row is printed), the REAL constructor builds the entity, and the REAL update routines of the function table run while it is live and in the update list:
--            4 data_positive_server_ages_and_expires  5 data_positive_client_clamps  6 data_sequence_accumulates  7 data_caps_nonmusic_and_music
--            8 data_looping_server_and_client_not_aged  9 data_invalid_server_countdown_and_unlink  10 data_invalid_client_countdown_and_unlink (made invalid by the REAL pause function)
--            The driver checks shape, restoration, calls, tracked entities and a chain that belongs to the routine; every lifetime, clamp, cap and unlink is OBSERVED, never expected. No handle exists.
--
-- No playback handle is ever created or faked: the branches that need one (a handle that is playing, or is not) are reported as UNSUPPORTED, never as a pass. A process that is not the server, or has no session, REFUSES
-- cases 1 and 2 (the fixture says why) and that refusal is reported here as a refusal. No audible, network or player-side delivery is exercised or claimed.
-- The lifetime is whatever the loaded sample data gives and is never lengthened for the fixture: the server routine may kill the entity at its FIRST call, and then the fixture skips the second call (it would read freed
-- data) and says so with calls=1 second_skipped=1. That one-update early kill is as legitimate a report as the two-update one.
--
-- THIS DRIVER JUDGES NOTHING PHYSICAL. It checks the SHAPE of the fixture's report (the rows in order, the keys in order, every value of its kind, the identity of the case, the end row, a record chain whose points
-- can follow one another) and writes every row as it came, with every value kept as text (a hex word is never turned into a number). What the original did is what the rows say. A report of another shape is a
-- failure of the WITNESS: a `witness_failure` row and an error beginning `env-sound-lifetime:`. If the engine stops inside a control, that is `engine_failure` and the same error.
--
-- host.args: root, out (JSON Lines path), case (1 to 3, required), scenario (lebanon_retail default | georgia_retail)

io.stdout:setvbuf ("no")

local args = host.args
local function refuse (message) error ("env-sound-lifetime: " .. message, 0) end
local root = args.root
if type (root) ~= "string" or root == "" then refuse ("root=<installation root> is required") end
local out_path = args.out
if type (out_path) ~= "string" or out_path == "" then refuse ("out=<JSON Lines path> is required") end
-- cases 1 to 3 are the primitive controls; 4 to 10 are the CONTROLLED-DATA cases (controlled sample descriptors, the real constructor and the real update routines)
local PRIMITIVE_CASES = 3
local LABELS = { "server_branch_ages_then_expires", "client_branch_ages_and_clamps", "playback_prerequisites", "data_positive_server_ages_and_expires", "data_positive_client_clamps", "data_sequence_accumulates",
	"data_caps_nonmusic_and_music", "data_looping_server_and_client_not_aged", "data_invalid_server_countdown_and_unlink", "data_invalid_client_countdown_and_unlink" }
local case = tonumber (args.case)
if not case or case ~= math.floor (case) or LABELS [case] == nil then refuse ("case must be an integer from 1 to 10, got '" .. tostring (args.case) .. "'") end
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
-- every write and flush of the evidence is checked: a failed one is a failure of the witness
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
-- the probe records nothing until it is armed, and it is armed BEFORE boot as the natural run does; a module without the observer has no such function
if type (dc.env_observer_arm) ~= "function" then
	emit { event = "refusal", reason = "env_observer_unavailable", detail = "the module was built without the env-observer feature" }
	refuse ("the module has no env_observer_arm function: build eech-dc with the env-observer feature")
end
dc.env_observer_arm (true)
local engine = dc.boot {
	install_root = root, map = scenario.map, terrain = host.terrain_provider (root, scenario.map), environment = host.env_provider (), campaign_directory = "camp01", campaign = scenario.campaign,
	gunship = "apache", seed = seed, bind_mobile = host.bind_mobile, move_vehicle = host.move_vehicle, observe_loss = host.observe_loss,
	supply_reached = host.supply_reached, line_of_sight = host.line_of_sight, sense = host.sense, aim = host.aim,
}
emit { event = "boot", scenario = scenario_name, seed = seed, case = case, label = LABELS [case], feature = "env-observer", group = 14 }

local present, method = pcall (function () return engine.env_case end)
if not present or type (method) ~= "function" then
	emit { event = "refusal", reason = "env_case_unavailable", detail = "the module was built without the env-fixture feature" }
	refuse ("the engine has no env_case method: build eech-dc with the env-observer feature")
end

-- ---------------------------------------------------------------------------------------------------------------- the report grammar: key=value rows, kept as text
local function parse (text, what)
	if type (text) ~= "string" or text == "" or string.sub (text, -1) ~= "\n" then hard (what .. " is empty or truncated (it does not end with a newline)") end
	if #text >= 32 * 1024 then hard (what .. " does not fit the fixture's 32 KiB buffer") end
	local rows = {}
	for line in string.gmatch (text, "([^\n]*)\n") do
		if line == "" or string.find (line, "[^\32-\126]") then hard (what .. " has an empty line or a byte that is not printable ASCII") end
		local event, rest = string.match (line, "^([%a][%w%-_]*)(.*)$")
		if event == nil or (rest ~= "" and string.sub (rest, 1, 1) ~= " ") then hard (what .. " has a row without an event name: " .. string.sub (line, 1, 60)) end
		local row = { event = event, raw = line, keys = {}, map = {} }
		for token in string.gmatch (rest, "%S+") do
			local k, v = string.match (token, "^([%a_][%w_]*)=([^=]+)$")
			if k == nil or row.map [k] ~= nil then hard (what .. " has a malformed or repeated field '" .. token .. "' in " .. event) end
			row.keys [#row.keys + 1] = k
			row.map [k] = v
		end
		rows [#rows + 1] = row
	end
	return rows
end
local function is_word (v) return type (v) == "string" and #v == 8 and string.find (v, "^[0-9a-f]+$") ~= nil end
local function is_count (v) return type (v) == "string" and #v <= 10 and string.find (v, "^%d+$") ~= nil end
local function is_flag (v) return v == "0" or v == "1" end
local function is_int (v) return type (v) == "string" and #v <= 11 and string.find (v, "^-?%d+$") ~= nil end
local function is_token (v) return type (v) == "string" and #v <= 200 and string.find (v, "^[%w_,]+$") ~= nil end
local KINDS = { w = is_word, n = is_count, b = is_flag, i = is_int, s = is_token }
-- "key:kind" a typed value, "key=literal" one literal; the keys in the order the C prints them
local function check (row, spec, what)
	local expected = {}
	for token in string.gmatch (spec, "%S+") do expected [#expected + 1] = token end
	if #row.keys ~= #expected then hard (what .. " row " .. row.event .. " has " .. #row.keys .. " fields, the protocol has " .. #expected .. ": " .. row.raw) end
	for n, token in ipairs (expected) do
		local k, kind = string.match (token, "^([%w_]+):(%a)$")
		if k ~= nil then
			if row.keys [n] ~= k or not KINDS [kind] (row.map [k]) then hard (what .. " row " .. row.event .. " field " .. n .. " is not '" .. k .. "' as a " .. kind .. ": " .. row.raw) end
		else
			local key, literal = string.match (token, "^([%w_]+)=(.+)$")
			if row.keys [n] ~= key or row.map [key] ~= literal then hard (what .. " row " .. row.event .. " field " .. n .. " is not " .. token .. ": " .. row.raw) end
		end
	end
end
local FACTS = "tag:s comms:i application_sound_system_enabled:i sound_system_initialised:i sound_system_paused:i samples:i lightning_rate:i lightning_size:i lightning_original_rate:i thunder_rate:i thunder_size:i thunder_original_rate:i"
local RECORD = "seq:n point:n n:n tracked:i frame:n upd:n entity:i etype:i sub:i channel:i locality:i valid:i looping:i panning:i seqcount:i handle:i session_parent:b update_member:b sample:i rate:i size:i orate:i life_w:w vlife_w:w amp_w:w pitch_w:w px_w:w py_w:w pz_w:w delta_w:w"
local REPORT = "armed:b capacity:n stored:n dropped:n overflow:b creations:n updates:n kills:n frames:n tracked:i open:n track_all:b untracked:n"
local function ask (index)
	local ok, text = pcall (engine.env_case, engine, 14, index)
	if not ok then
		emit { event = "engine_failure", i = index, reason = tostring (text) }
		refuse ("env_case(14, " .. index .. ") FAILED in the engine: " .. (string.match (tostring (text), "[^\r\n]+") or tostring (text)))
	end
	if string.find (text, "env-case-error", 1, true) then hard ("env_case(14, " .. index .. ") refused: " .. text) end
	return parse (text, "env_case(14, " .. index .. ")")
end
local function write (scope, rows)
	for _, row in ipairs (rows) do
		local t = { event = row.event, scope = scope, raw = row.raw }
		for _, k in ipairs (row.keys) do t [k] = row.map [k] end
		emit (t)
	end
end

-- ---------------------------------------------------------------------------------------------------------------- THE PER-ENTITY CHAIN RULES
-- Shared by the primitive controls (cases 1 and 2) and the controlled-data cases (4 to 10): both call the update routines at most three times, so EVERY entry is stored and the strict form of the rules applies. `recs` are the
-- probe records of ONE tracked entity in time order; `opts.tracked` is its tracked index as text; `opts.routine` is "server", "client" or nil (either), `opts.wrong_server` / `opts.wrong_client` the messages for a point of the other
-- routine. The points a branch can show: the server routine 1..5, the client routine 6 and 7; each record may follow the one before it. After a handle that is still playing the later updates that are only counted leave expiry
-- records that repeat; a handle that stops playing is then idle. Nothing here computes or expects a lifetime.
local NEXT = { [0] = { [1] = true, [6] = true }, [1] = { [1] = true, [2] = true, [3] = true, [4] = true }, [2] = { [5] = true }, [3] = { [5] = true }, [4] = { [1] = true, [3] = true, [4] = true }, [5] = {}, [6] = { [7] = true }, [7] = { [6] = true } }
local function walk_chain (recs, opts)
	local lastpoint, lastupd, entity, kills, enters, exits, expiry, creation_n = nil, 0, nil, 0, 0, 0, "none", nil
	for n, r in ipairs (recs) do
		local point, upd = tonumber (r.map.point), tonumber (r.map.upd)
		if r.map.tracked ~= opts.tracked or point > 7 then hard ("the probe record " .. r.map.seq .. " is of an untracked entity or an unknown point: " .. r.raw) end
		if n == 1 then
			if point ~= 0 then hard ("the first probe record is not the creation: " .. r.raw) end
			entity, creation_n = r.map.entity, r.map.n
		else
			if r.map.entity ~= entity then hard ("the probe record " .. r.map.seq .. " is of another entity: " .. r.raw) end
			if r.map.n ~= creation_n then hard ("the probe record " .. r.map.seq .. " has n " .. r.map.n .. " where its creation has " .. creation_n) end
			if not NEXT [lastpoint] [point] then hard ("the probe record " .. r.map.seq .. ": point " .. point .. " cannot follow point " .. lastpoint) end
		end
		-- the playback handle is what the branch says (none for an expiry with no handle, a handle for the two others) and the control itself never creates one
		if point == 2 and r.map.handle ~= "0" then hard ("the probe record " .. r.map.seq .. " is an expiry with no handle that shows a handle") end
		if (point == 3 or point == 4) and r.map.handle ~= "1" then hard ("the probe record " .. r.map.seq .. " is an expiry with a handle that shows none") end
		if opts.routine == "server" and (point == 6 or point == 7) then hard (opts.wrong_server .. r.raw) end
		if opts.routine == "client" and point >= 1 and point <= 5 then hard (opts.wrong_client .. r.raw) end
		-- the update counter: a creation has run none; the n-th stored entry of an update is its n-th (the control calls at most three times, so none is left out); an exit, an expiry and a kill carry their update's count
		if point == 0 and upd ~= 0 then hard ("the creation record " .. r.map.seq .. " has the update count " .. upd) end
		if point == 1 or point == 6 then
			enters = enters + 1
			if upd ~= enters then hard ("the probe record " .. r.map.seq .. " is update " .. upd .. " where it is the " .. enters .. "th stored entry of an update") end
		elseif point == 7 then
			exits = exits + 1
			if upd ~= lastupd then hard ("the probe record " .. r.map.seq .. " leaves update " .. upd .. " that began as update " .. lastupd) end
		elseif point >= 2 then
			-- an expiry or a kill belongs to the update that is running: every entry is stored, so it carries the CURRENT entry count (not merely one that has not gone down), and a kill
			-- carries the update of the expiry straight before it
			if enters < 1 then hard ("the probe record " .. r.map.seq .. " ends an update that no record began") end
			if point == 5 and upd ~= lastupd then hard ("the probe record " .. r.map.seq .. " is a kill in update " .. upd .. " but the expiry straight before it is in update " .. lastupd) end
			if upd ~= enters then hard ("the probe record " .. r.map.seq .. " is in update " .. upd .. " where the running update is the " .. enters .. "th entry") end
		end
		lastupd = upd
		if point == 5 then kills = kills + 1 end
		if point >= 2 and point <= 4 then expiry = ({ [2] = "no_handle", [3] = "idle_handle", [4] = "playing_handle" }) [point] end
		lastpoint = point
	end
	if #recs < 1 then hard ("the control stored no probe record at all") end
	-- an operation that began is finished: a lifetime below zero with no or an idle handle is followed by the kill, and an entry into the client routine by its exit
	if lastpoint == 2 or lastpoint == 3 then hard ("the chain ends at an expiry (point " .. lastpoint .. ") with no kill after it: the kill record is missing") end
	if lastpoint == 6 then hard ("the chain ends inside the client routine: its exit record is missing") end
	return { lastpoint = lastpoint, kills = kills, enters = enters, exits = exits, expiry = expiry, entity = entity }
end

-- THE DECLARED INPUT DOMAIN of each controlled-data case, as the C control (csrc/eech_env_fixture.c env_snd_data_case) chooses it: the controlled descriptor numbers {rate, size} of the lightning sample (L) and of the thunder
-- sample (T; nil leaves the retail value alone), how many entities it builds, how many calls it plans for EACH, the routine of each, the delta word of a call, the constructor looping flag and, for the invalid cases, the delay
-- word of the real pause. These are the inputs of the control, not outcomes: the typed rows must declare exactly them.
local DATA = {
	[4] = { L = { 4, 20 }, entities = 1, calls = 3, routines = { "server" }, delta = "40000000", looping = "0" },
	[5] = { L = { 4, 20 }, entities = 1, calls = 3, routines = { "client" }, delta = "40000000", looping = "0" },
	[6] = { L = { 4, 6 }, T = { 2, 5 }, entities = 1, seqcount = 2, calls = 0, routines = {}, delta = "00000000", looping = "0" },
	[7] = { L = { 1, 400 }, entities = 2, calls = 0, routines = {}, delta = "00000000", looping = "0", distinct_sub = true },
	[8] = { L = { 4, 20 }, entities = 2, calls = 3, routines = { "server", "client" }, delta = "40000000", looping = "1" },
	[9] = { L = { 4, 20 }, entities = 1, calls = 3, routines = { "server" }, delta = "3f800000", looping = "0", pause = "3fc00000" },
	[10] = { L = { 4, 20 }, entities = 1, calls = 3, routines = { "client" }, delta = "3f800000", looping = "0", pause = "3fc00000" },
}

-- ---------------------------------------------------------------------------------------------------------------- the plan, then the one control
-- THE INDEX-0 REPORT as the C prints it here: the plan, the facts, two unsupported rows, ONE probe report. A process that has run no frame holds no record; the natural run is the other driver's.
local plan = ask (0)
write ("plan", plan)
if #plan ~= 5 or plan [1].event ~= "env-snd-plan" or plan [2].event ~= "env-snd-facts" or plan [3].event ~= "env-snd-unsupported" or plan [4].event ~= "env-snd-unsupported" or plan [5].event ~= "env-snd-report" then
	hard ("the plan is not one plan row, one facts row, two unsupported rows and one report (a process that has run no frame holds no record)")
end
check (plan [1], "version=1 cases:n process_isolated=required", "the plan")
if plan [1].map.cases ~= tostring (PRIMITIVE_CASES) then hard ("the plan has " .. plan [1].map.cases .. " cases, not " .. PRIMITIVE_CASES) end
check (plan [2], FACTS, "the plan")
if plan [2].map.tag ~= "plan" then hard ("the plan's facts row is tagged '" .. plan [2].map.tag .. "'") end
check (plan [3], "item:s reason:s", "the plan")
check (plan [4], "item:s reason:s", "the plan")
check (plan [5], REPORT, "the plan")
if plan [5].map.armed ~= "1" then hard ("the sound entity probe says it is not armed: " .. plan [5].raw) end
if plan [5].map.stored ~= "0" or plan [5].map.creations ~= "0" or plan [5].map.dropped ~= "0" or plan [5].map.track_all ~= "0" or plan [5].map.untracked ~= "0" then
	hard ("the plan's report holds probe records in a process that has run no frame: " .. plan [5].raw)
end

local rows = ask (case)
write ("case", rows)
local head, last = rows [1], rows [#rows]
if head == nil or head.event ~= "env-sound" or last == nil or last.event ~= "env-sound-end" or #rows < 3 then hard ("the control does not run from an env-sound row to an env-sound-end row") end
check (head, "kind=control i:n label:s comms:i", "the control")
if head.map.i ~= tostring (case) or head.map.label ~= LABELS [case] then hard ("the control is not case " .. case .. " '" .. LABELS [case] .. "': " .. head.raw) end
check (rows [2], FACTS, "the control")
if rows [2].map.tag ~= "control" then hard ("the control's facts row is tagged '" .. rows [2].map.tag .. "'") end
-- the facts are those of the plan, in the same process: only the tag differs
for _, k in ipairs (plan [2].keys) do
	if k ~= "tag" and plan [2].map [k] ~= rows [2].map [k] then hard ("the control's facts differ from the plan's in " .. k .. ": " .. rows [2].raw) end
end
if last.map.i ~= tostring (case) then hard ("the end row is of another case: " .. last.raw) end

local refused = last.map.refused ~= nil
local chain_summary = nil
if case == 3 then
	check (last, "i:n changed=0 process_isolated=required", "the control")
	if #rows ~= 5 or rows [3].event ~= "env-snd-unsupported" or rows [4].event ~= "env-snd-unsupported" then hard ("case 3 is not a head, the facts, two unsupported rows and an end row") end
	check (rows [3], "item:s reason:s", "the control")
	check (rows [4], "item:s reason:s", "the control")
elseif case >= 4 then
	-- THE CONTROLLED-DATA CASES (csrc/eech_env_fixture.c env_snd_data_case): the sample descriptor numbers are set to named controlled values, the REAL constructor builds the entity or entities and the REAL update
	-- routines of the function table are called while the entity is live and in the update list. The driver checks the SHAPE and the bookkeeping that makes the evidence complete and coherent: the declared input domain of
	-- the case (the descriptors, the entities, the planned calls and their routine, the delta, the pause delay) is what the typed rows say; a call is skipped only after an entity that is dead or unlinked; the per-entity
	-- chain of the probe obeys the same transition and update-pair rules as the primitive controls (walk_chain); and every independent capture that is the same moment in the same entity agrees (the constructor state with
	-- the CREATED record, each state before a call with that call's ENTER record, each client state after a call with that call's EXIT record). It does NOT compute or expect any lifetime, countdown, clamp, cap or
	-- unlink: those words are what the original did, and an early kill or unlink that skips calls is as legitimate as a late one.
	local spec = DATA [case]
	for at = 3, 5 do
		if rows [at].event ~= "env-snd-unsupported" then hard ("the controlled-data case does not give its three unsupported rows after the facts: " .. rows [at].raw) end
		check (rows [at], "item:s reason:s", "the control")
	end
	if refused then
		check (last, "i:n refused:s changed=0 process_isolated=required", "the control")
		if #rows ~= 6 then hard ("a refused controlled-data case has rows besides its head, its facts, its three unsupported rows and its end: " .. #rows) end
		local reason = last.map.refused
		if reason ~= "the_process_is_not_the_server_so_the_server_kill_cannot_be_observed" and reason ~= "there_is_no_session_entity_to_parent_the_sound_effect" and reason ~= "there_is_no_free_entity_slot_for_the_real_constructor" then
			hard ("the control is refused for a reason the fixture does not give: " .. reason)
		end
	else
		check (last, "i:n changed=1 entities:n planned_entities:n short_of_slots:b calls:n skipped:n descriptor_restored:b delta_restored:b process_isolated=required", "the control")
		if last.map.descriptor_restored ~= "1" then hard ("FIXTURE DEFECT: the sample descriptors were not put back after the control: " .. last.raw) end
		if last.map.delta_restored ~= "1" then hard ("FIXTURE DEFECT: the frame delta was not put back after the control: " .. last.raw) end
		local entities, calls = tonumber (last.map.entities), tonumber (last.map.calls)
		-- the entities the control declares and the ones it built: a control that found itself short of a slot stops where it is and says so (a partial creation is reported, not hidden)
		if tonumber (last.map.planned_entities) ~= spec.entities then hard ("the control declares " .. last.map.planned_entities .. " entities where case " .. case .. " builds " .. spec.entities .. ": " .. last.raw) end
		if entities > spec.entities or (entities < spec.entities) ~= (last.map.short_of_slots == "1") then hard ("the control built " .. entities .. " of " .. spec.entities .. " entities and says short_of_slots=" .. last.map.short_of_slots .. ": " .. last.raw) end
		if entities < 1 then hard ("the control built no entity and did not refuse: " .. last.raw) end
		local at = 6
		-- the declared input domain: the controlled descriptor of the lightning sample, the thunder sample (controlled for case 6, otherwise the retail value left alone), the planned calls and the delta
		local seen_samples = {}
		local first_descriptor
		for k = 1, 2 do
			local row = rows [at]
			if row == nil or row.event ~= "env-sound-data-input" then hard ("the controlled-data case does not give descriptor row " .. k) end
			check (row, "tag=descriptor sample:i controlled_rate:i controlled_size:i retail_rate:i retail_size:i original_rate:i", "the control")
			local want = (k == 1) and spec.L or spec.T
			if want == nil then want = { tonumber (row.map.retail_rate), tonumber (row.map.retail_size) } end
			if tonumber (row.map.controlled_rate) ~= want [1] or tonumber (row.map.controlled_size) ~= want [2] then
				hard ("the descriptor row " .. k .. " does not declare the controlled domain of case " .. case .. " (rate " .. want [1] .. " size " .. want [2] .. "): " .. row.raw)
			end
			if seen_samples [row.map.sample] then hard ("the two descriptor rows are of the same sample: " .. row.raw) end
			seen_samples [row.map.sample] = true
			if k == 1 then first_descriptor = row.map end
			at = at + 1
		end
		if rows [at] == nil or rows [at].event ~= "env-sound-data-input" then hard ("the controlled-data case does not give its deltas row") end
		check (rows [at], "tag=deltas calls:n delta_w:w", "the control")
		if tonumber (rows [at].map.calls) ~= spec.calls or rows [at].map.delta_w ~= spec.delta then
			hard ("the deltas row does not declare the planned " .. spec.calls .. " calls of delta word " .. spec.delta .. " of case " .. case .. ": " .. rows [at].raw)
		end
		at = at + 1
		local pause
		if spec.pause ~= nil then
			pause = rows [at]
			if pause == nil or pause.event ~= "env-sound-pause" then hard ("an invalid-effect case does not give the row of the real pause function") end
			check (pause, "sub:i delay_w:w children_sound:n children_of_type:n returned:i", "the control")
			if pause.map.delay_w ~= spec.pause then hard ("the pause row does not declare the delay word " .. spec.pause .. " of case " .. case .. ": " .. pause.raw) end
			-- honest accounting of what the real pause function can touch. It walks the session's special-effect children and changes ONLY a sound child of the paused sub type that is valid (soundeff.c
			-- pause_local_entity_sound_type filters on `eff.sub_type == type`), so sound children of OTHER sub types (the ambient sounds the session already holds: an actual run showed 11 sound children, 1 of the paused
			-- type) are listed but untouched and are not a lack of isolation. What would be one is a SECOND target of the paused sub type, or a count that contradicts itself.
			local total, matching, returned = tonumber (pause.map.children_sound), tonumber (pause.map.children_of_type), tonumber (pause.map.returned)
			if matching ~= 1 or returned ~= 1 then
				hard ("the real pause function could touch a target other than the control's own entity (sound children " .. pause.map.children_sound .. ", of the paused type " .. pause.map.children_of_type .. ", paused " .. pause.map.returned .. "): the control is not isolated")
			end
			if total < matching then hard ("the pause row counts " .. pause.map.children_sound .. " sound children but " .. pause.map.children_of_type .. " of the paused type: the counts contradict each other: " .. pause.raw) end
			at = at + 1
		end
		-- the state rows: by entity, by tag (0 the constructor's, 99 the state straight after the real pause, k after call k)
		local states, made = {}, {}
		while rows [at] ~= nil and rows [at].event == "env-sound-state" do
			local s = rows [at]
			check (s, "ent:n slot:i tag:n etype:i data:b sub:i valid:b looping:b member:b handle:b life_w:w vlife_w:w", "the control")
			local ent = tonumber (s.map.ent)
			if ent >= entities then hard ("a state row is of entity " .. ent .. " where the control built " .. entities .. ": " .. s.raw) end
			states [ent] = states [ent] or {}
			local list = states [ent]
			if #list == 0 then
				if s.map.tag ~= "0" then hard ("the first state row of entity " .. ent .. " is not its creation state: " .. s.raw) end
				-- the normal constructor flags of the case (inputs, not outcomes)
				if s.map.valid ~= "1" or s.map.looping ~= spec.looping then hard ("FIXTURE DEFECT: the created entity " .. ent .. " has valid " .. s.map.valid .. " looping " .. s.map.looping .. ": " .. s.raw) end
			else
				local prev = list [#list]
				if prev.map.data == "0" then hard ("a state row follows the free slot of entity " .. ent .. ": " .. s.raw) end
				if prev.map.tag ~= "0" and prev.map.tag ~= "99" and prev.map.member == "0" then hard ("a call state follows the unlinked entity " .. ent .. ": " .. s.raw) end
				local want = (prev.map.tag == "0" and spec.pause ~= nil) and "99" or tostring (tonumber (prev.map.tag == "99" and 0 or prev.map.tag) + 1)
				if s.map.tag ~= want then hard ("the state rows of entity " .. ent .. " go from tag " .. prev.map.tag .. " to " .. s.map.tag .. ", not " .. want) end
				if s.map.tag ~= "99" then made [ent] = (made [ent] or 0) + 1 end
				-- the role of the entity is stable: the same slot, and the same sub type while the slot is a sound effect
				if s.map.slot ~= list [1].map.slot then hard ("the state rows of entity " .. ent .. " change slot from " .. list [1].map.slot .. " to " .. s.map.slot .. ": " .. s.raw) end
				if s.map.data == "1" and s.map.sub ~= list [1].map.sub then hard ("the state rows of entity " .. ent .. " change sub type: " .. s.raw) end
			end
			if s.map.data == "0" and s.map.etype ~= "0" then hard ("a state row has no data but is still entity type " .. s.map.etype .. ": " .. s.raw) end
			list [#list + 1] = s
			at = at + 1
		end
		local total, planned_total = 0, spec.entities * spec.calls
		for ent = 0, entities - 1 do
			if states [ent] == nil then hard ("entity " .. ent .. " has no state rows") end
			-- no cohort makes more calls than the case plans for it (the helper stops at the plan, and the probe stores at most three entries): checked BEFORE any aggregate accounting, so an overcall in one
			-- cohort can never cancel a legitimate skip in another in the planned/made/skipped totals
			if (made [ent] or 0) > spec.calls then hard ("entity " .. ent .. " made " .. made [ent] .. " calls where case " .. case .. " plans at most " .. spec.calls .. " for each") end
			total = total + (made [ent] or 0)
			-- the cohort identity: distinct slots; case 7 builds an ordinary and a music sound, so two sub types
			for other = 0, ent - 1 do
				if states [other] [1].map.slot == states [ent] [1].map.slot then hard ("entities " .. other .. " and " .. ent .. " are in the same slot " .. states [ent] [1].map.slot) end
				if spec.distinct_sub and states [other] [1].map.sub == states [ent] [1].map.sub then hard ("entities " .. other .. " and " .. ent .. " have the same sub type where case " .. case .. " builds an ordinary and a music sound") end
			end
			-- a call is skipped only after an entity that is dead or unlinked, never because a scalar says so
			local final = states [ent] [#states [ent]]
			if (made [ent] or 0) < spec.calls and final.map.data ~= "0" and final.map.member ~= "0" then
				hard ("entity " .. ent .. " made " .. (made [ent] or 0) .. " of its " .. spec.calls .. " planned calls but its last state is neither a free slot nor unlinked: " .. final.raw)
			end
		end
		if total ~= calls then hard ("the control says " .. calls .. " calls but its state rows show " .. total) end
		if pause ~= nil and pause.map.sub ~= states [0] [1].map.sub then hard ("the real pause function was given sub type " .. pause.map.sub .. " but the entity it should pause has " .. states [0] [1].map.sub .. ": " .. pause.raw) end
		if tonumber (last.map.skipped) ~= planned_total - calls then hard ("the control says " .. last.map.skipped .. " calls were skipped where " .. planned_total .. " were planned and " .. calls .. " made: " .. last.raw) end
		local report = rows [at]
		if report == nil or report.event ~= "env-snd-report" then hard ("the controlled-data case does not give the probe's report after its states") end
		check (report, REPORT, "the control")
		if report.map.armed ~= "1" then hard ("the sound entity probe says it is not armed: " .. report.raw) end
		if report.map.dropped ~= "0" or report.map.overflow ~= "0" then hard ("the sound entity probe dropped " .. report.map.dropped .. " records: the record is incomplete") end
		if report.map.untracked ~= "0" then hard ("the probe lost track of " .. report.map.untracked .. " creations") end
		if tonumber (report.map.creations) ~= entities or tonumber (report.map.tracked) ~= entities or report.map.open ~= "0" or report.map.track_all ~= "0" then
			hard ("the control built " .. entities .. " entities and switched tracking off again, but the probe says otherwise: " .. report.raw)
		end
		if tonumber (report.map.updates) ~= calls then hard ("the probe counted " .. report.map.updates .. " updates but the control made " .. calls .. " calls") end
		at = at + 1
		local records, groups = {}, {}
		for n = at, #rows - 1 do
			local r = rows [n]
			if r.event ~= "env-snd" then hard ("a row between the probe's report and the end row is not a probe record: " .. r.raw) end
			check (r, RECORD, "the control")
			if tonumber (r.map.seq) ~= #records then hard ("the probe record " .. #records .. " has seq " .. r.map.seq) end
			records [#records + 1] = r
			local tracked = tonumber (r.map.tracked)
			if tracked < 0 or tracked >= entities then hard ("the probe record " .. r.map.seq .. " is of an untracked entity: " .. r.raw) end
			groups [tracked] = groups [tracked] or {}
			groups [tracked] [#groups [tracked] + 1] = r
		end
		if tonumber (report.map.stored) ~= #records then hard ("the probe stored " .. report.map.stored .. " records but the control holds " .. #records) end
		-- two captures of the same moment of the same entity agree in every field they both hold; nothing writes to the entity in between, and nothing is recomputed
		local function same (state, record, what, ent)
			for _, pair in ipairs ({ { "etype", "etype" }, { "sub", "sub" }, { "valid", "valid" }, { "looping", "looping" }, { "member", "update_member" }, { "handle", "handle" }, { "life_w", "life_w" }, { "vlife_w", "vlife_w" }, { "slot", "entity" } }) do
				if state.map [pair [1]] ~= record.map [pair [2]] then
					hard ("the " .. what .. " of entity " .. ent .. " disagrees with its state row in " .. pair [1] .. " (" .. state.map [pair [1]] .. " against " .. record.map [pair [2]] .. "): " .. state.raw .. " | " .. record.raw)
				end
			end
		end
		local kills_total = 0
		for ent = 0, entities - 1 do
			local group = groups [ent]
			if group == nil then hard ("tracked entity " .. ent .. " has no creation record") end
			local routine = spec.routines [ent + 1]
			local w = walk_chain (group, { tracked = tostring (ent), routine = routine,
				wrong_server = "the probe shows a point of the other routine for tracked entity " .. ent .. ": ", wrong_client = "the probe shows a point of the other routine for tracked entity " .. ent .. ": " })
			kills_total = kills_total + w.kills
			local m = made [ent] or 0
			-- every entry is stored (at most three calls): the entries are the calls, and a client call is an entry and its exit
			if w.enters ~= m then hard ("entity " .. ent .. " has " .. m .. " call states but " .. w.enters .. " stored entries") end
			if routine == "client" and w.exits ~= m then hard ("entity " .. ent .. " has " .. m .. " calls but " .. w.exits .. " stored exits") end
			local st = {}
			for _, s in ipairs (states [ent]) do st [tonumber (s.map.tag)] = s end
			local final = states [ent] [#states [ent]]
			if (w.kills == 1) ~= (final.map.data == "0") then hard ("entity " .. ent .. ": the probe's kill record and the final slot state disagree: " .. final.raw) end
			if w.kills > 1 then hard ("tracked entity " .. ent .. " is killed more than once") end
			-- the constructor's state is the CREATED record; each state before a call is that call's ENTER record (after the real pause, the state straight after it); a client state after a call is that call's EXIT record
			same (st [0], group [1], "CREATED record", ent)
			-- the constructor metadata of every record that reads the entity is the declared first descriptor and the case's sequence count: the sample, the controlled rate and size, the observed original rate and
			-- the count of samples (nothing is computed from them; a killed entity's record reads only its slot and carries none)
			for _, r in ipairs (group) do
				if tonumber (r.map.point) ~= 5 then
					local m = r.map
					if m.sample ~= first_descriptor.sample or m.rate ~= first_descriptor.controlled_rate or m.size ~= first_descriptor.controlled_size or m.orate ~= first_descriptor.original_rate or tonumber (m.seqcount) ~= (spec.seqcount or 1) then
						hard ("the probe record " .. m.seq .. " of entity " .. ent .. " carries sample " .. m.sample .. " rate " .. m.rate .. " size " .. m.size .. " original rate " .. m.orate .. " sequence count " .. m.seqcount .. " where the declared constructor input is sample " .. first_descriptor.sample .. " rate " .. first_descriptor.controlled_rate .. " size " .. first_descriptor.controlled_size .. " original rate " .. first_descriptor.original_rate .. " sequence count " .. (spec.seqcount or 1))
					end
				end
			end
			local k_enter, k_exit = 0, 0
			for _, r in ipairs (group) do
				local point = tonumber (r.map.point)
				if point == 1 or point == 6 then
					k_enter = k_enter + 1
					local before = (k_enter == 1) and (st [99] or st [0]) or st [k_enter - 1]
					same (before, r, "ENTER record of call " .. k_enter, ent)
				elseif point == 7 then
					k_exit = k_exit + 1
					same (st [k_exit], r, "EXIT record of call " .. k_exit, ent)
				end
			end
		end
		if kills_total ~= tonumber (report.map.kills) then hard ("the probe counted " .. report.map.kills .. " kills but its records show " .. kills_total) end
		chain_summary = { records = #records, updates = calls, kills = kills_total, expiry = "", calls = calls, entities = entities, skipped = tonumber (last.map.skipped), partial = last.map.short_of_slots == "1" }
	end
elseif refused then
	check (last, "i:n refused:s changed=0 process_isolated=required", "the control")
	if #rows ~= 3 then hard ("a refused control has rows besides its head, its facts and its end: " .. #rows) end
	local reason = last.map.refused
	if reason ~= "the_process_is_not_the_server_so_the_server_kill_cannot_be_observed" and reason ~= "there_is_no_session_entity_to_parent_the_sound_effect" and reason ~= "there_is_no_free_entity_slot_for_the_real_constructor" then
		hard ("the control is refused for a reason the fixture does not give: " .. reason)
	end
else
	check (last, "i:n changed=1 calls:n second_skipped:b etype_after:i data_after:b delta_restored:b process_isolated=required", "the control")
	if last.map.delta_restored ~= "1" then hard ("FIXTURE DEFECT: the frame delta was not put back after the control: " .. last.raw) end
	local input, report = rows [3], rows [4]
	if input == nil or input.event ~= "env-sound-input" or report == nil or report.event ~= "env-snd-report" then hard ("the control does not give its input row and then the probe's report") end
	check (input, "tag=deltas first_w:w second_w:w", "the control")
	check (report, REPORT, "the control")
	if report.map.armed ~= "1" then hard ("the sound entity probe says it is not armed: " .. report.raw) end
	if report.map.dropped ~= "0" or report.map.overflow ~= "0" then hard ("the sound entity probe dropped " .. report.map.dropped .. " records: the record is incomplete") end
	if report.map.creations ~= "1" or report.map.tracked ~= "1" or report.map.open ~= "0" or report.map.track_all ~= "0" then
		hard ("the control built one entity and switched tracking off again, but the probe says otherwise: " .. report.raw)
	end
	if report.map.untracked ~= "0" then hard ("the probe lost track of " .. report.map.untracked .. " creations: the control's one entity is not followed") end
	local records = {}
	for at = 5, #rows - 1 do
		if rows [at].event ~= "env-snd" then hard ("the control has a row between its report and its end that is not a probe record: " .. rows [at].raw) end
		check (rows [at], RECORD, "the control")
		records [#records + 1] = rows [at]
	end
	if tonumber (report.map.stored) ~= #records then hard ("the probe stored " .. report.map.stored .. " records but the control holds " .. #records) end
	for n, r in ipairs (records) do
		if tonumber (r.map.seq) ~= n - 1 then hard ("the probe record " .. (n - 1) .. " has seq " .. r.map.seq) end
	end
	local w = walk_chain (records, { tracked = "0", routine = (case == 1) and "server" or "client", wrong_server = "the server control shows a client point: ", wrong_client = "the client control shows a server point: " })
	local lastpoint, kills, enters, exits, expiry = w.lastpoint, w.kills, w.enters, w.exits, w.expiry
	if enters ~= exits and case == 2 then hard ("the client control shows " .. enters .. " entries and " .. exits .. " exits") end
	-- the end row agrees with the records and with the calls the fixture says it made: the update count is the calls; the second call is skipped only because the first one ended the entity
	local calls = tonumber (last.map.calls)
	if calls < 1 or calls > 2 then hard ("the control says it made " .. last.map.calls .. " update calls, not one or two: " .. last.raw) end
	if enters ~= calls then hard ("the control made " .. calls .. " update calls but the records show " .. enters .. " entries") end
	if tonumber (report.map.updates) ~= calls then hard ("the probe counted " .. report.map.updates .. " updates but the control made " .. calls .. " calls") end
	if kills ~= tonumber (report.map.kills) then hard ("the probe counted " .. report.map.kills .. " kills but its records show " .. kills) end
	if last.map.second_skipped ~= (calls == 1 and "1" or "0") then hard ("the control says second_skipped=" .. last.map.second_skipped .. " after " .. calls .. " calls: " .. last.raw) end
	if lastpoint == 5 then
		if last.map.etype_after ~= "0" or last.map.data_after ~= "0" then hard ("the chain ends in a kill but the entity slot is not free in the end row: " .. last.raw) end
	else
		if calls == 1 then hard ("the second call was skipped but the chain shows no kill that ended the entity: " .. last.raw) end
		local live = records [#records].map.etype
		if last.map.etype_after ~= live or last.map.etype_after == "0" or last.map.data_after ~= "1" then hard ("the chain has no kill but the entity slot is not the live sound entity in the end row: " .. last.raw) end
	end
	chain_summary = { records = #records, updates = calls, kills = kills, expiry = expiry, calls = calls }
end

local result = { event = "result", case = case, label = LABELS [case], refused = refused, refusal = last.map.refused or "", changed = last.map.changed or "0", comms = head.map.comms }
if chain_summary ~= nil then
	result.records = chain_summary.records
	result.updates = chain_summary.updates
	result.kills = chain_summary.kills
	result.expiry = chain_summary.expiry
	result.calls = chain_summary.calls
	result.entities = chain_summary.entities
	result.skipped = chain_summary.skipped
	result.partial = chain_summary.partial
	result.second_skipped = last.map.second_skipped
	result.etype_after = last.map.etype_after
	result.data_after = last.map.data_after
end
emit (result)

-- the last thing the witness does is close its evidence, and a close that fails is a failure after the last row as much as before it
local closed, close_error = out:close ()
if not closed then refuse ("OUTPUT FAILURE: closing " .. out_path .. " failed: " .. tostring (close_error)) end
io.stderr:write (string.format ("env-sound-lifetime: case=%d label=%s refused=%s\n", case, LABELS [case], tostring (refused)))
