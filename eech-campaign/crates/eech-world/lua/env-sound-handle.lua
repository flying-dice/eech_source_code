-- TEST-ONLY DRIVER of the ORIGINAL status of a NON-NULL, buffer-absent, idle playback handle at a server expiry (EEDCS-51, D3). SOURCE PREPARED: UNBUILT, UNRUN, UNREVIEWED. CHARACTERISATION ONLY: it moves, removes and
-- guards nothing, and no production, host-ABI or schema line changed.
--
-- WHAT IT ASKS. se_updt.c update_server (hosted block) hands the sound effect's handle to campaign_sound_lifetime_update; for an expired effect that holds a handle the HOST asks whether the handle is playing and the
-- campaign answers with the REAL compiled modules/sound/soundman.c get_system_sound_effect_playing (it clears the record's `playing` flag and answers FALSE when its sound buffer is NULL), and a kill reaches the REAL
-- destroy_system_sound_effect through se_dstry.c. The closed no-handle controls never reach that branch. This driver runs ONE of the two cases of the fixture's group 14 (csrc/eech_env_fixture.c env_snd_handle_case)
-- in a FRESH process (the entity cannot be put back; the process is the isolation boundary):
--   21  handle_idle_buffer_absent_server_expiry                    the handle record starts with playing = 0
--   22  handle_idle_buffer_absent_stale_playing_server_expiry      the handle record starts with playing = 1 (a SENSITIVITY MARKER: the real getter writes the flag)
-- The handle is CONTROLLED TEST STATE: a structurally valid, in-use record in the static array soundman.c owns, with its sound buffer ABSENT. The original constructor never produces such a record in this process (it
-- returns no handle without a device and releases a record whose buffer failed), so this is NOT an original product, NOT a backend and NOT audio. A playing buffer needs a device that does not exist here and is not
-- manufactured: it is reported as UNSUPPORTED, never as a pass.
--
-- WHAT THIS DRIVER JUDGES: only the SHAPE and the BOOKKEEPING of the fixture's report (a closed grammar: the rows in order, the keys in order, every value of its kind and domain), the identity of the case, and that
-- the report is complete and the same event told the same way by every view of it: the probe's fixed-capacity report (one tracked creation, no loss, no untracked creation, no natural frame, no open episode), the number
-- of ENTER records against the calls made, the update number of every record against its ENTER, the kill (or none) against the entity's last state, the entity's slot, type, sub type and controlled metadata (the
-- descriptor numbers, the delta word, the sample sequence) against the creation, the state words against the probe's words of the same instant, and the handle's own readbacks against the cleanup. It computes no
-- lifetime, forces no kill at any step, decides no policy and has no expectation of what the getter, the host or the kill should have done: an early kill, a kill at the last call and an entity that stays alive with no
-- expiry are all legitimate observations. It writes the fixture's report whole to `<out>.case.raw` (PRIVATE: that report carries the probe's entity slot and type words and the sound entity's position words and needs
-- a safe projection before anything is published to Git; no pointer is in it) BEFORE validating any of it, and writes to the JSON file only SUMMARIES (control, unsupported, derived, result and failure rows), not one
-- row per report row.
--
-- WHAT IS MEASURED AND WHAT IS INFERRED ABOUT THE GETTER. The getter is reached through a callback from the host's question, so that it ran at an expiry is INFERRED from the source and from the probe's expiry-with-handle
-- records (points 3 and 4); only a stale `playing` marker that the report shows set before the calls and clear after them is a MEASUREMENT that the real getter wrote the flag. A marker that stays set after an expiry
-- with a handle contradicts the inference and is reported as such, never as a getter reached.
--
-- A report of another shape, or an engine that stops, is a failure of the WITNESS: a `witness_failure` or
-- `engine_failure` row and an error beginning `env-sound-handle:`. A case the fixture REFUSED (not the server, no session, an initialised sound system, a handle record in use, no entity slot) is a refusal, never a pass.
--
-- host.args: root=<installation root> out=<new JSON Lines path> case=<21|22> [scenario=lebanon_retail|georgia_retail]
-- Needs a module built with the env-observer and env-fixture features. One fresh process and root per run.

io.stdout:setvbuf ("no")

local args = host.args
local function refuse (message) error ("env-sound-handle: " .. message, 0) end

local root = args.root
if type (root) ~= "string" or root == "" then refuse ("root=<installation root> is required") end
local out_path = args.out
if type (out_path) ~= "string" or out_path == "" then refuse ("out=<JSON Lines path> is required") end
local LABELS = { [21] = "handle_idle_buffer_absent_server_expiry", [22] = "handle_idle_buffer_absent_stale_playing_server_expiry" }
local case = tonumber (args.case)
if not case or case ~= math.floor (case) or LABELS [case] == nil then refuse ("case must be 21 or 22, got '" .. tostring (args.case) .. "'") end
local scenarios = {
	lebanon_retail = { map = "..\\common\\maps\\map5", campaign = "LEBANON.CHC" },
	georgia_retail = { map = "..\\common\\maps\\map3", campaign = "GEORGIA.CHC" },
}
local scenario_name = args.scenario == nil and "lebanon_retail" or args.scenario
local scenario = scenarios [scenario_name]
if scenario == nil then refuse ("scenario must be lebanon_retail or georgia_retail, got '" .. tostring (scenario_name) .. "'") end

-- every evidence file is created exclusively, here, before anything is booted
local create = host.output_create_new
if type (create) ~= "function" then refuse ("this host has no host.output_create_new: evidence files cannot be created exclusively, and the driver does not fall back to a truncating open") end
local paths = { out = out_path, case = out_path .. ".case.raw" }
local files, created = {}, {}
for _, name in ipairs ({ "out", "case" }) do
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
local function keep (text)
	local ok, err = files.case:write (text)
	if not ok then refuse ("OUTPUT FAILURE: writing " .. paths.case .. " failed: " .. tostring (err)) end
	ok, err = files.case:flush ()
	if not ok then refuse ("OUTPUT FAILURE: flushing " .. paths.case .. " failed: " .. tostring (err)) end
end

local dc = require ("eech_dc")
dc.prepare_installation (root)
-- the tracking probe records nothing until it is armed, and it is armed BEFORE boot as the natural run does
if type (dc.env_observer_arm) ~= "function" then
	emit { event = "refusal", reason = "env_observer_unavailable", detail = "the module was built without the env-observer feature" }
	refuse ("the module has no env_observer_arm function: build eech-dc with the env-observer feature")
end
dc.env_observer_arm (true)
local engine = dc.boot {
	install_root = root, map = scenario.map, terrain = host.terrain_provider (root, scenario.map), environment = host.env_provider (), campaign_directory = "camp01", campaign = scenario.campaign,
	gunship = "apache", seed = 1, bind_mobile = host.bind_mobile, move_vehicle = host.move_vehicle, observe_loss = host.observe_loss,
	supply_reached = host.supply_reached, line_of_sight = host.line_of_sight, sense = host.sense, aim = host.aim,
}
emit { event = "boot", scenario = scenario_name, case = case, label = LABELS [case], group = 14 }
local present, method = pcall (function () return engine.env_case end)
if not present or type (method) ~= "function" then
	emit { event = "refusal", reason = "env_case_unavailable", detail = "the module was built without the env-fixture feature" }
	refuse ("the engine has no env_case method: build eech-dc with the env-observer and env-fixture features")
end

-- ---------------------------------------------------------------------------------------------------------------- the closed grammar: key=value rows, the keys in order, every value of its kind and domain
local function canonical (v, signed, digits)
	if type (v) ~= "string" or #v > digits then return false end
	if v == "0" then return true end
	return string.find (v, signed and "^%-?[1-9]%d*$" or "^[1-9]%d*$") ~= nil
end
local KINDS = {
	w = function (v) return type (v) == "string" and #v == 8 and string.find (v, "^[0-9a-f]+$") ~= nil end,
	n = function (v) return canonical (v, false, 10) and tonumber (v) <= 4294967295 end,
	i = function (v) return canonical (v, true, 11) and tonumber (v) >= -2147483648 and tonumber (v) <= 2147483647 end,
	b = function (v) return v == "0" or v == "1" end,
	s = function (v) return type (v) == "string" and #v >= 1 and #v <= 200 and string.find (v, "^[%w_]+$") ~= nil end,
}
local function parse (text)
	if type (text) ~= "string" or text == "" or string.sub (text, -1) ~= "\n" then hard ("the report is empty or truncated (it does not end with a newline)") end
	local rows = {}
	for line in string.gmatch (text, "([^\n]*)\n") do
		if line == "" or string.find (line, "[^\32-\126]") then hard ("the report has an empty line or a byte that is not printable ASCII") end
		local event, rest = string.match (line, "^([%a][%w%-_]*)(.*)$")
		if event == nil or (rest ~= "" and string.sub (rest, 1, 1) ~= " ") then hard ("the report has a row without an event name: " .. string.sub (line, 1, 60)) end
		local row = { event = event, raw = line, keys = {}, map = {} }
		for token in string.gmatch (rest, "%S+") do
			local k, v = string.match (token, "^([%a_][%w_]*)=([^=]+)$")
			if k == nil or row.map [k] ~= nil then hard ("the report has a malformed or repeated field '" .. token .. "' in " .. event) end
			row.keys [#row.keys + 1] = k
			row.map [k] = v
		end
		rows [#rows + 1] = row
	end
	return rows
end
-- "key:kind" a typed value, "key=literal" one literal, in the order the C prints them
local function check (row, event, spec)
	if row == nil then hard ("the report ends before its " .. event .. " row") end
	if row.event ~= event then hard ("expected a " .. event .. " row, found " .. row.event .. ": " .. row.raw) end
	local expected = {}
	for token in string.gmatch (spec, "%S+") do expected [#expected + 1] = token end
	if #row.keys ~= #expected then hard ("the " .. event .. " row has " .. #row.keys .. " fields, the protocol has " .. #expected .. ": " .. row.raw) end
	for n, token in ipairs (expected) do
		local k, kind = string.match (token, "^([%w_]+):(%a)$")
		if k ~= nil then
			if row.keys [n] ~= k or not KINDS [kind] (row.map [k]) then hard ("field " .. n .. " of the " .. event .. " row is not '" .. k .. "' as a " .. kind .. ": " .. row.raw) end
		else
			local key, literal = string.match (token, "^([%w_]+)=(.+)$")
			if row.keys [n] ~= key or row.map [key] ~= literal then hard ("field " .. n .. " of the " .. event .. " row is not " .. token .. ": " .. row.raw) end
		end
	end
	return row.map
end

local FACTS = "tag=control comms:i application_sound_system_enabled:i sound_system_initialised:i sound_system_paused:i samples:i lightning_rate:i lightning_size:i lightning_original_rate:i thunder_rate:i thunder_size:i thunder_original_rate:i"
local STATE = "ent:i slot:i tag:n etype:i data:b sub:i valid:i looping:i member:b handle:b life_w:w vlife_w:w"
local RECORD = "seq:n point:n n:n tracked:i frame:n upd:n entity:i etype:i sub:i channel:i locality:i valid:i looping:i panning:i seqcount:i handle:i session_parent:b update_member:b sample:i rate:i size:i orate:i life_w:w vlife_w:w amp_w:w pitch_w:w px_w:w py_w:w pz_w:w delta_w:w"
-- the points the original update routine's tracking records may follow one another with (ss_envfx.h ENV_FXE): created 0, server enter 1, expired with no handle 2, expired idle 3, expired playing 4, killed 5
local NEXT = { [0] = { [1] = true }, [1] = { [1] = true, [2] = true, [3] = true, [4] = true }, [2] = { [5] = true }, [3] = { [5] = true }, [4] = { [1] = true }, [5] = {} }

-- ---------------------------------------------------------------------------------------------------------------- the case
local ok, text = pcall (engine.env_case, engine, 14, case)
keep (ok and text or ("env_case failed in the engine: " .. tostring (text) .. "\n"))
if not ok then
	emit { event = "engine_failure", i = case, reason = first_line (text) }
	refuse ("env_case(14, " .. case .. ") FAILED in the engine: " .. first_line (text))
end
if string.find (text, "env-case-error", 1, true) then hard ("env_case(14, " .. case .. ") refused: " .. first_line (text)) end
if #text >= 32 * 1024 then hard ("the report does not fit the fixture's 32 KiB buffer") end
local rows = parse (text)
local at = 1
local function take (event, spec)
	local row = rows [at]
	local map = check (row, event, spec)
	at = at + 1
	return map, row
end

local head = take ("env-sound", "kind=control i=" .. case .. " label=" .. LABELS [case] .. " comms:i")
local facts = take ("env-snd-facts", FACTS)
local item1 = take ("env-snd-unsupported", "item=playing_handle_with_a_device_buffer reason:s")
local item2 = take ("env-snd-unsupported", "item=handle_made_by_the_original_constructor reason:s")
emit { event = "control", i = case, label = LABELS [case], comms = head.comms, sound_system_initialised = facts.sound_system_initialised, application_sound_system_enabled = facts.application_sound_system_enabled }
emit { event = "unsupported", item = "playing_handle_with_a_device_buffer", reason = item1.reason }
emit { event = "unsupported", item = "handle_made_by_the_original_constructor", reason = item2.reason }

-- a refusal ends the report before anything changed: it is reported, and it is not a pass
if rows [at] ~= nil and rows [at].event == "env-sound-end" and rows [at].map.refused ~= nil then
	local end_map = take ("env-sound-end", "i=" .. case .. " refused:s changed=0 process_isolated=required")
	if rows [at] ~= nil then hard ("the report has a row after its refusal end row: " .. rows [at].raw) end
	emit { event = "control_refused", i = case, reason = end_map.refused }
	refuse ("the fixture REFUSED case " .. case .. " (" .. end_map.refused .. "): the control did not run, which is an unavailable prerequisite and not a result")
end

local descriptor = take ("env-handle-input", "tag=descriptor controlled_rate=4 controlled_size=20 retail_rate:i retail_size:i")
if descriptor.retail_rate ~= facts.lightning_rate or descriptor.retail_size ~= facts.lightning_size then
	hard ("the retail descriptor of the input row (" .. descriptor.retail_rate .. ", " .. descriptor.retail_size .. ") is not the one the facts row read before the change (" .. facts.lightning_rate .. ", " .. facts.lightning_size .. ")")
end
local deltas = take ("env-handle-input", "tag=deltas calls=3 delta_w=40000000")
local record = take ("env-handle-input", "tag=record origin=test_owned_static_record built_by=fixture buffer=absent device=none user_data=none initial_used=1 initial_playing:b")
local initial_playing = (case == 22) and "1" or "0"
if record.initial_playing ~= initial_playing then hard ("the handle record starts with playing " .. record.initial_playing .. " where case " .. case .. " names " .. initial_playing) end
local before = take ("env-handle-state", "tag=before installed=1 used=1 playing:b buffer_null=1")
if before.playing ~= initial_playing then hard ("the handle record read back playing " .. before.playing .. " before the calls where it was installed with " .. initial_playing) end

-- the entity's states: before the calls and after each call that was made
local states = {}
while rows [at] ~= nil and rows [at].event == "env-sound-state" do
	local map = take ("env-sound-state", STATE)
	states [#states + 1] = map
	if map.ent ~= "0" or map.tag ~= tostring (#states - 1) then hard ("the entity state " .. #states .. " is not entity 0 at step " .. (#states - 1) .. ": " .. rows [at - 1].raw) end
end
if #states < 1 or #states > 4 then hard ("the report holds " .. #states .. " entity states where 1 to 4 are possible (the entity before the calls and one after each of at most three)") end
for n, s in ipairs (states) do
	if s.slot ~= states [1].slot then hard ("the entity state " .. n .. " is in slot " .. s.slot .. " where the first is in slot " .. states [1].slot) end
	if s.data == "1" then
		if s.etype ~= states [1].etype or s.sub ~= states [1].sub or s.looping ~= states [1].looping then hard ("the live entity state " .. n .. " changed type, sub type or looping against the first") end
	else
		-- a kill frees the entity: the slot keeps its number and the freed slot's type, every other field is zero
		if n ~= #states then hard ("the entity is dead at state " .. n .. " but " .. (#states - n) .. " later states follow") end
		if s.sub ~= "0" or s.valid ~= "0" or s.looping ~= "0" or s.member ~= "0" or s.handle ~= "0" or s.life_w ~= "00000000" or s.vlife_w ~= "00000000" then hard ("the dead entity state still shows data of the freed entity") end
	end
end
if states [1].data ~= "1" or states [1].valid ~= "1" or states [1].member ~= "1" or states [1].handle ~= "1" then
	hard ("the entity before the calls is not a valid effect in the update list that holds the handle")
end
local last = states [#states]
local alive = last.data == "1"

local after = take ("env-handle-state", "tag=after installed=1 used:b playing:b buffer_null=1")
local cleanup = take ("env-handle-cleanup", "installed=1 entity_live_after:b detached:b destroyed_by_original:b destroyed_by_fixture:b used_final=0 playing_final=0 buffer_null_final=1")
local cleanup_raw = rows [at - 1].raw
if cleanup.entity_live_after ~= (alive and "1" or "0") then hard ("the cleanup says the entity is " .. (cleanup.entity_live_after == "1" and "live" or "gone") .. " after the calls where its last state is " .. (alive and "live" or "dead") .. ": " .. cleanup_raw) end
if cleanup.detached ~= (alive and last.handle or "0") then hard ("the cleanup says detached " .. cleanup.detached .. " where the entity's last state shows handle " .. (alive and last.handle or "none (dead)") .. ": " .. cleanup_raw) end
-- the handle was released by the original (its kill) or by the fixture, never by neither and never by both; the facts come from the snapshot taken right after the calls
if after.used == "0" then
	if cleanup.destroyed_by_original ~= "1" or cleanup.destroyed_by_fixture ~= "0" then hard ("the handle was no longer in use after the calls but the cleanup does not say the original released it: " .. cleanup_raw) end
else
	if cleanup.destroyed_by_original ~= "0" or cleanup.destroyed_by_fixture ~= "1" then hard ("the handle was still in use after the calls but the cleanup does not say the fixture released it: " .. cleanup_raw) end
	if not alive or last.handle ~= "1" then hard ("the handle was still in use after the calls but the entity is gone or does not hold it: " .. cleanup_raw) end
end
if not alive and after.used ~= "0" then hard ("the entity was killed but the handle was still in use after the calls: " .. cleanup_raw) end

local report = take ("env-snd-report", "armed=1 capacity=80 stored:n dropped=0 overflow=0 creations=1 updates:n kills:n frames=0 tracked=1 open=0 track_all=0 untracked=0")
local records = {}
while rows [at] ~= nil and rows [at].event == "env-snd" do
	records [#records + 1] = take ("env-snd", RECORD)
end
local end_map = take ("env-sound-end", "i=" .. case .. " changed=1 installed=1 entities=1 planned_entities=1 short_of_slots=0 calls:n skipped:n descriptor_restored=1 delta_restored=1 handle_released=1 process_isolated=required")
if rows [at] ~= nil then hard ("the report has a row after its end row: " .. rows [at].raw) end
local calls, skipped = tonumber (end_map.calls), tonumber (end_map.skipped)
if calls + skipped ~= 3 or calls < 1 or calls ~= #states - 1 then hard ("the end row says " .. end_map.calls .. " calls and " .. end_map.skipped .. " skipped where the states show " .. (#states - 1) .. " calls of 3") end

-- ---------------------------------------------------------------------------------------------------------------- the probe's records against the calls, the entity and the handle
if tonumber (report.stored) ~= #records then hard ("the probe says it stored " .. report.stored .. " records and the report holds " .. #records) end
local created = records [1]
if created == nil or created.point ~= "0" then hard ("the report holds no creation record of the tracked entity first") end
-- what the constructor and the controlled descriptor fix, and what every later record of the same entity must still show
local STABLE = { "entity", "etype", "sub", "channel", "locality", "panning", "seqcount", "valid", "looping", "session_parent", "update_member", "sample", "rate", "size", "orate", "amp_w", "pitch_w", "px_w", "py_w", "pz_w" }
local ZERO_AFTER_KILL = { "sub", "channel", "locality", "valid", "looping", "panning", "seqcount", "handle", "session_parent", "update_member", "rate", "size", "orate" }
local ZERO_WORDS = { "life_w", "vlife_w", "amp_w", "pitch_w", "px_w", "py_w", "pz_w" }
local enters, seen = 0, {}
for n, r in ipairs (records) do
	local point = tonumber (r.point)
	if tonumber (r.seq) ~= n - 1 or point > 5 then hard ("the probe record " .. n .. " is out of sequence or of an unknown point: seq " .. r.seq .. " point " .. r.point) end
	if r.tracked ~= "0" or r.n ~= "0" or r.frame ~= "0" then hard ("the probe record " .. r.seq .. " is not of the one tracked entity in the open-less, frame-less episode: " .. r.raw) end
	if n == 1 then
		if r.upd ~= "0" or r.handle ~= "0" then hard ("the creation record is not update 0 before the handle: " .. r.raw) end
	else
		if point == 0 then hard ("the probe record " .. r.seq .. " is a second creation") end
		if not NEXT [tonumber (records [n - 1].point)] [point] then hard ("the probe record " .. r.seq .. ": point " .. point .. " cannot follow point " .. records [n - 1].point) end
	end
	if point == 1 then
		enters = enters + 1
		if tonumber (r.upd) ~= enters then hard ("the probe record " .. r.seq .. " is update " .. r.upd .. " where it is the " .. enters .. "th entry of an update") end
	elseif point >= 2 and tonumber (r.upd) ~= enters then
		hard ("the probe record " .. r.seq .. " (point " .. point .. ") belongs to update " .. r.upd .. " where the running update is " .. enters)
	end
	if point >= 1 and r.delta_w ~= deltas.delta_w then hard ("the probe record " .. r.seq .. " carries the delta word " .. r.delta_w .. " where the control set " .. deltas.delta_w) end
	if point == 5 then
		-- the kill reads the entity slot only (the sound data was freed with the entity): the raw fields are zero, there is no sample, and the type is the freed slot's
		if n ~= #records then hard ("the kill record is not the last record") end
		for _, key in ipairs (ZERO_AFTER_KILL) do
			if r [key] ~= "0" then hard ("the kill record " .. r.seq .. " shows " .. key .. " " .. r [key] .. " where the freed entity's raw fields are zero: " .. r.raw) end
		end
		for _, key in ipairs (ZERO_WORDS) do
			if r [key] ~= "00000000" then hard ("the kill record " .. r.seq .. " shows " .. key .. " " .. r [key] .. " where the freed entity's raw words are zero: " .. r.raw) end
		end
		if r.sample ~= "-1" then hard ("the kill record " .. r.seq .. " shows a sample where the freed entity has none: " .. r.raw) end
		if r.entity ~= created.entity or r.entity ~= last.slot or r.etype ~= last.etype then hard ("the kill record " .. r.seq .. " is not the dead entity's slot and freed type: " .. r.raw) end
	else
		for _, key in ipairs (STABLE) do
			if r [key] ~= created [key] then hard ("the probe record " .. r.seq .. " shows " .. key .. " " .. r [key] .. " where the creation shows " .. created [key]) end
		end
		-- the handle is what the branch says: an expiry with NO handle cannot be, and an entry or an expiry with a handle shows it
		if point == 2 then hard ("the probe record " .. r.seq .. " is an expiry with NO handle although the control installed one") end
		if (point == 1 or point == 3 or point == 4) and r.handle ~= "1" then hard ("the probe record " .. r.seq .. " (point " .. point .. ") shows no handle although the control installed one") end
	end
	seen [point] = (seen [point] or 0) + 1
end
-- the controlled descriptor and the sample sequence as the records read them from the original's tables
if created.rate ~= "4" or created.size ~= "20" or created.orate ~= facts.lightning_original_rate then
	hard ("the creation record shows the descriptor rate " .. created.rate .. " size " .. created.size .. " original rate " .. created.orate .. " where the control set 4, 20 and the facts read " .. facts.lightning_original_rate)
end
if created.seqcount ~= "1" or created.sample == "-1" then hard ("the creation record is not a single-sample effect with a sample: " .. created.raw) end
-- the counters against the records and the calls
if tonumber (report.updates) ~= enters or enters ~= calls then hard ("the probe counts " .. report.updates .. " updates, holds " .. enters .. " entries and the end row says " .. calls .. " calls") end
if tonumber (report.kills) ~= (seen [5] or 0) then hard ("the probe counts " .. report.kills .. " kills and the records hold " .. (seen [5] or 0)) end
-- the state words are the probe's words of the same instant: the creation is the first state, the entry of update u is the state after call u-1, a playing expiry of update u is the state after call u
local function same_words (state, r, what)
	if state.life_w ~= r.life_w or state.vlife_w ~= r.vlife_w then hard (what .. " shows the words " .. r.life_w .. "/" .. r.vlife_w .. " where the entity state shows " .. state.life_w .. "/" .. state.vlife_w) end
end
if states [1].slot ~= created.entity or states [1].etype ~= created.etype or states [1].sub ~= created.sub or states [1].looping ~= created.looping or states [1].valid ~= created.valid or states [1].member ~= created.update_member then
	hard ("the first entity state is not the creation record's entity: " .. created.raw)
end
same_words (states [1], created, "the creation record")
for _, r in ipairs (records) do
	local point, upd = tonumber (r.point), tonumber (r.upd)
	if point == 1 then
		same_words (states [upd], r, "the entry record " .. r.seq)
	elseif point == 4 then
		if states [upd + 1] == nil or states [upd + 1].data ~= "1" then hard ("the playing expiry record " .. r.seq .. " is followed by a dead or missing entity state") end
		same_words (states [upd + 1], r, "the playing expiry record " .. r.seq)
	elseif point == 3 then
		if upd ~= calls or alive then hard ("the idle expiry record " .. r.seq .. " is not at the last call of an entity that was killed") end
	elseif point == 5 then
		if upd ~= calls or alive then hard ("the kill record " .. r.seq .. " is not at the last call of an entity that is dead") end
	end
end
-- the kill, the entity's last state, the probe and the handle tell one story
local killed = (seen [5] or 0) == 1
if killed == alive then hard ("the probe holds " .. (seen [5] or 0) .. " kill records and the entity's last state is " .. (alive and "live" or "dead")) end
if killed and (seen [3] or 0) ~= 1 then hard ("the kill is not preceded by exactly one idle expiry record (" .. (seen [3] or 0) .. ")") end
if alive and (seen [3] or 0) ~= 0 then hard ("the entity is live but the probe holds an idle expiry record") end

-- ---------------------------------------------------------------------------------------------------------------- the getter: a source-backed inference apart from a measured marker
local expiry_with_handle = ((seen [3] or 0) + (seen [4] or 0)) > 0
local getter_status
if expiry_with_handle then
	if before.playing == "1" then
		getter_status = (after.playing == "0") and "marker_cleared_measured" or "marker_preserved_contradicts_the_inferred_getter_call"
	else
		getter_status = "inferred_from_an_expiry_with_a_handle_no_marker_to_measure"
	end
else
	getter_status = (before.playing == "1" and after.playing == "0") and "marker_cleared_with_no_expiry_unexplained" or "no_expiry_with_a_handle_so_no_getter_call_inferred"
end

-- ---------------------------------------------------------------------------------------------------------------- what the driver JOINS (labelled derived; never a measurement)
emit {
	event = "derived", source = "the driver's join of the records above; not a measurement", case = case, label = LABELS [case],
	handle_playing_before = before.playing, handle_playing_after_calls = after.playing, handle_used_after_calls = after.used,
	getter_status = getter_status,
	getter_call_measured_by_marker = getter_status == "marker_cleared_measured",
	getter_call_inferred_from_source_and_expiry = expiry_with_handle and getter_status ~= "marker_preserved_contradicts_the_inferred_getter_call",
	expiry_with_handle_records = (seen [3] or 0) + (seen [4] or 0), expired_idle = seen [3] or 0, expired_playing = seen [4] or 0, killed = seen [5] or 0,
	original_released_handle = cleanup.destroyed_by_original == "1", fixture_released_handle = cleanup.destroyed_by_fixture == "1", entity_live_after = alive,
	calls = calls, handle_zeroed_playing = "0", handle_zeroed_used = "0",
	scope = "controlled test state: a buffer-absent idle record in the static array, not an original product, not a backend and not audio; a playing buffer and the original constructor's handle are unsupported; the destroy of an invalid effect (the unlink path) and the other callers of the getter are not exercised; the full report is the private raw sidecar",
}
emit { event = "result", case = case }

for _, name in ipairs (created) do
	local closed, close_error = files [name]:close ()
	if not closed then refuse ("OUTPUT FAILURE: closing " .. paths [name] .. " failed: " .. tostring (close_error)) end
end
io.stderr:write (string.format ("env-sound-handle: case=%d calls=%d handle_used_after=%s expired_idle=%d expired_playing=%d killed=%d getter=%s\n", case, calls, after.used, seen [3] or 0, seen [4] or 0, seen [5] or 0, getter_status))
