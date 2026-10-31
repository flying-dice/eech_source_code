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
local LABELS = { "server_branch_ages_then_expires", "client_branch_ages_and_clamps", "playback_prerequisites" }
local case = tonumber (args.case)
if not case or case ~= math.floor (case) or LABELS [case] == nil then refuse ("case must be an integer from 1 to 3, got '" .. tostring (args.case) .. "'") end
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

-- ---------------------------------------------------------------------------------------------------------------- the plan, then the one control
-- THE INDEX-0 REPORT as the C prints it here: the plan, the facts, two unsupported rows, ONE probe report. A process that has run no frame holds no record; the natural run is the other driver's.
local plan = ask (0)
write ("plan", plan)
if #plan ~= 5 or plan [1].event ~= "env-snd-plan" or plan [2].event ~= "env-snd-facts" or plan [3].event ~= "env-snd-unsupported" or plan [4].event ~= "env-snd-unsupported" or plan [5].event ~= "env-snd-report" then
	hard ("the plan is not one plan row, one facts row, two unsupported rows and one report (a process that has run no frame holds no record)")
end
check (plan [1], "version=1 cases:n process_isolated=required", "the plan")
if plan [1].map.cases ~= tostring (#LABELS) then hard ("the plan has " .. plan [1].map.cases .. " cases, not " .. #LABELS) end
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
	-- the points a branch can show: the server routine 1..5, the client routine 6 and 7; each record is of the one tracked entity and may follow the one before it. After a handle that is still playing the
	-- later updates that are only counted leave expiry records that repeat; a handle that stops playing is then idle
	local NEXT = { [0] = { [1] = true, [6] = true }, [1] = { [1] = true, [2] = true, [3] = true, [4] = true }, [2] = { [5] = true }, [3] = { [5] = true }, [4] = { [1] = true, [3] = true, [4] = true }, [5] = {}, [6] = { [7] = true }, [7] = { [6] = true } }
	local lastpoint, lastupd, entity, kills, enters, exits, expiry, creation_n = nil, 0, nil, 0, 0, 0, "none", nil
	for n, r in ipairs (records) do
		local point, upd = tonumber (r.map.point), tonumber (r.map.upd)
		if tonumber (r.map.seq) ~= n - 1 then hard ("the probe record " .. (n - 1) .. " has seq " .. r.map.seq) end
		if r.map.tracked ~= "0" or point > 7 then hard ("the probe record " .. r.map.seq .. " is of an untracked entity or an unknown point: " .. r.raw) end
		if n == 1 then
			if point ~= 0 then hard ("the first probe record is not the creation: " .. r.raw) end
			entity, creation_n = r.map.entity, r.map.n
		else
			if r.map.entity ~= entity then hard ("the probe record " .. r.map.seq .. " is of another entity: " .. r.raw) end
			if r.map.n ~= creation_n then hard ("the probe record " .. r.map.seq .. " has n " .. r.map.n .. " where its creation has " .. creation_n) end
			if not NEXT [lastpoint] [point] then hard ("the probe record " .. r.map.seq .. ": point " .. point .. " cannot follow point " .. lastpoint) end
		end
		if case == 1 and (point == 6 or point == 7) then hard ("the server control shows a client point: " .. r.raw) end
		if case == 2 and point >= 1 and point <= 5 then hard ("the client control shows a server point: " .. r.raw) end
		-- the update counter: a creation has run none; the n-th stored entry of an update is its n-th (the control calls at most twice, so none is left out); an exit, an expiry and a kill carry their update's count
		if point == 0 and upd ~= 0 then hard ("the creation record " .. r.map.seq .. " has the update count " .. upd) end
		if point == 1 or point == 6 then
			enters = enters + 1
			if upd ~= enters then hard ("the probe record " .. r.map.seq .. " is update " .. upd .. " where it is the " .. enters .. "th stored entry of an update") end
		elseif point == 7 then
			exits = exits + 1
			if upd ~= lastupd then hard ("the probe record " .. r.map.seq .. " leaves update " .. upd .. " that began as update " .. lastupd) end
		elseif point >= 2 then
			-- an expiry or a kill belongs to the update that is running: the control calls at most twice and stores every entry, so it carries the CURRENT entry count (not merely one that has not gone down), and a kill
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
	if #records < 1 then hard ("the control stored no probe record at all") end
	-- an operation that began is finished: a lifetime below zero with no or an idle handle is followed by the kill, and an entry into the client routine by its exit
	if lastpoint == 2 or lastpoint == 3 then hard ("the chain ends at an expiry (point " .. lastpoint .. ") with no kill after it: the kill record is missing") end
	if lastpoint == 6 then hard ("the chain ends inside the client routine: its exit record is missing") end
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
	result.second_skipped = last.map.second_skipped
	result.etype_after = last.map.etype_after
	result.data_after = last.map.data_after
end
emit (result)

-- the last thing the witness does is close its evidence, and a close that fails is a failure after the last row as much as before it
local closed, close_error = out:close ()
if not closed then refuse ("OUTPUT FAILURE: closing " .. out_path .. " failed: " .. tostring (close_error)) end
io.stderr:write (string.format ("env-sound-lifetime: case=%d label=%s refused=%s\n", case, LABELS [case], tostring (refused)))
