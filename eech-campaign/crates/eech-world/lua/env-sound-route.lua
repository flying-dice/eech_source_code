-- TEST-ONLY DRIVER of two more ORIGINAL consumers of the test-owned buffer-absent playback handle record (EEDCS-51, D3). SOURCE PREPARED: UNBUILT, UNRUN, UNREVIEWED. CHARACTERISATION ONLY: it moves, removes and guards
-- nothing, and no production, host-ABI or schema line changed. It is a small SEPARATE file beside lua/env-sound-handle.lua (cases 21 and 22, the valid server expiry) and lua/env-sound-handle-guard.lua; both stay as they are.
--
-- It runs ONE of the fixture's group 14 cases in a FRESH process (the entity cannot be put back; the process is the isolation boundary), through the real public host Lua path (Engine env_case):
--   23  handle_invalid_server_unlink   the REAL pause on the session makes a handle-holding effect invalid and the REAL server routine counts it down: se_updt.c's invalid branch releases the handle with the REAL destroy and
--                                      unlinks the entity from the update list (the valid-expiry status request cannot be made on this route).
--   24  handle_direct_playback         the REAL dispatch (play_local_entity_sound) on a handle-holding looping effect at the session range 0.0 and the main viewpoint: the REAL set volume, get playing and play bodies run on the
--                                      record if the path reaches them. The record starts with three markers (playing = 1, paused = 1, volume = -1) so the report shows how far the path got. The scan and the stop iteration are NOT reached
--                                      (their gates are the sound system status and a static count that is zero here) and are reported unexercised, never as passes.
-- The handle is CONTROLLED TEST STATE (see env-sound-handle.lua): not an original constructor product, not a backend, not a playing buffer and not audio. The DirectSound entry points the record reaches are the headless null COM
-- stubs. A playing buffer is not manufactured; sound_system_initialised is never set.
--
-- WHAT THIS DRIVER JUDGES: only the SHAPE and the BOOKKEEPING of the fixture's report (a closed grammar, the case identity, completeness, the probe's fixed-capacity report against the records, the entity's states against
-- the probe's records of the same instant, the handle readbacks against the cleanup) and a few joins that are the SOURCE ORDER of the original (a volume that changed implies the play body ran in the looping control; a
-- cleared paused marker implies a playing flag that is set afterwards). It computes no lifetime, forces no outcome and defines no oracle: a path that returns early, a muted channel or a destroy is reported as it is.
-- MEASURED (flags read back from the record and the entity) is kept apart from INFERRED (the call order, from the source). The complete report is written to `<out>.case.raw` BEFORE it is validated (PRIVATE: it carries the
-- probe's slot and entity words and the entity's position words and needs a safe projection before any Git publication; no pointer is in it); the JSON file holds only summaries. Both evidence files are created exclusively
-- before boot and BOTH are closed here, every failure reported; a failed close fails the run.
--
-- host.args: root=<installation root> out=<new JSON Lines path> case=<23|24> [scenario=lebanon_retail|georgia_retail]
-- Needs a module built with the env-observer and env-fixture features. One fresh process and root per run.

io.stdout:setvbuf ("no")

local args = host.args
local function refuse (message) error ("env-sound-route: " .. message, 0) end

local root = args.root
if type (root) ~= "string" or root == "" then refuse ("root=<installation root> is required") end
local out_path = args.out
if type (out_path) ~= "string" or out_path == "" then refuse ("out=<JSON Lines path> is required") end
local LABELS = { [23] = "handle_invalid_server_unlink", [24] = "handle_direct_playback" }
local case = tonumber (args.case)
if not case or case ~= math.floor (case) or LABELS [case] == nil then refuse ("case must be 23 or 24, got '" .. tostring (args.case) .. "'") end
local unlink_case = case == 23
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
local files, opened = {}, {}
for _, name in ipairs ({ "out", "case" }) do
	local file, why = create (paths [name])
	if file == nil then
		for _, done in ipairs (opened) do pcall (files [done].close, files [done]) end
		refuse ("OUTPUT FAILURE: creating " .. paths [name] .. " failed: " .. tostring (why) .. " (evidence is created exclusively and never overwritten)")
	end
	files [name] = file
	opened [#opened + 1] = name
end

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
	local ok, err = files.out:write ("{" .. table.concat (parts, ",") .. "}\n")
	if not ok then refuse ("OUTPUT FAILURE: writing " .. out_path .. " failed: " .. tostring (err)) end
	ok, err = files.out:flush ()
	if not ok then refuse ("OUTPUT FAILURE: flushing " .. out_path .. " failed: " .. tostring (err)) end
end
local function hard (message)
	pcall (emit, { event = "witness_failure", reason = message })
	refuse (message)
end
local function first_line (text) return string.match (tostring (text), "[^\r\n]+") or tostring (text) end
-- the raw text of a parsed row lives apart from its map (a map holds exactly the report's own keys), read with raw_of
local RAW = setmetatable ({}, { __mode = "k" })
local function raw_of (map) return RAW [map] or "(no row)" end
local function keep (text)
	local ok, err = files.case:write (text)
	if not ok then refuse ("OUTPUT FAILURE: writing " .. paths.case .. " failed: " .. tostring (err)) end
	ok, err = files.case:flush ()
	if not ok then refuse ("OUTPUT FAILURE: flushing " .. paths.case .. " failed: " .. tostring (err)) end
end

local dc = require ("eech_dc")
dc.prepare_installation (root)
if type (dc.env_observer_arm) ~= "function" then
	emit { event = "refusal", reason = "env_observer_unavailable" }
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
	emit { event = "refusal", reason = "env_case_unavailable" }
	refuse ("the engine has no env_case method: build eech-dc with the env-observer and env-fixture features")
end

-- ---------------------------------------------------------------------------------------------------------------- the closed grammar
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
local LOOP = unlink_case and "0" or "1"
local NEXT = { [0] = { [1] = true }, [1] = { [1] = true }, [5] = {} }

-- ---------------------------------------------------------------------------------------------------------------- the case
local ok, text = pcall (engine.env_case, engine, 14, case)
keep (ok and text or ("env_case failed in the engine: " .. tostring (text) .. "\n"))
if not ok then
	emit { event = "engine_failure", i = case, tag = string.match (tostring (text), "%[[%w_%-]+:[%w_%.]+%]") or "none" }
	refuse ("env_case(14, " .. case .. ") FAILED in the engine (the whole text is in the private raw)")
end
if string.find (text, "env-case-error", 1, true) then hard ("env_case(14, " .. case .. ") refused: " .. first_line (text)) end
if #text >= 32 * 1024 then hard ("the report does not fit the fixture's 32 KiB buffer") end
local rows = parse (text)
local at = 1
local function take (event, spec)
	local row = rows [at]
	local map = check (row, event, spec)
	RAW [map] = row.raw
	at = at + 1
	return map, row
end

local head = take ("env-sound", "kind=control i=" .. case .. " label=" .. LABELS [case] .. " comms:i")
local facts = take ("env-snd-facts", FACTS)
take ("env-snd-unsupported", "item=playing_handle_with_a_device_buffer reason:s")
take ("env-snd-unsupported", "item=handle_made_by_the_original_constructor reason:s")
emit { event = "control", i = case, label = LABELS [case], comms = head.comms, sound_system_initialised = facts.sound_system_initialised }
if rows [at] ~= nil and rows [at].event == "env-sound-end" and rows [at].map.refused ~= nil then
	local end_map = take ("env-sound-end", "i=" .. case .. " refused:s changed=0 process_isolated=required")
	if rows [at] ~= nil then hard ("the report has a row after its refusal end row: " .. rows [at].raw) end
	emit { event = "control_refused", i = case, reason = end_map.refused }
	refuse ("the fixture REFUSED case " .. case .. " (" .. end_map.refused .. "): the control did not run, an unavailable prerequisite and not a result")
end

local descriptor = take ("env-route-input", "tag=descriptor " .. (unlink_case and "controlled=1 controlled_rate=4 controlled_size=20" or "controlled=0 controlled_rate=0 controlled_size=0") .. " retail_rate:i retail_size:i")
if descriptor.retail_rate ~= facts.lightning_rate or descriptor.retail_size ~= facts.lightning_size then hard ("the retail descriptor of the input row is not the one the facts row read before the change") end
local flags = take ("env-route-input", "tag=flags effects_enabled_before:b ui_muted:i channel_muted:i channel_volume:i")
-- case 24 trips assertions on purpose: its prerequisite (the logging assertion mode and the debug log option off) was checked by the fixture before any change and is recorded here
if not unlink_case then take ("env-route-input", "tag=assert mode=logging debug_log=0") end
local initial_playing = unlink_case and "0" or "1"
-- the record as installed (looping as the case names it, no panning, the pitch word 1.0); after the steps: the unlink route changes nothing in it but its use, the playback writes volume, paused, playing, panning and pitch
local READ_BEFORE = "installed=1 used=1 playing=" .. initial_playing .. " paused=1 volume=-1 rate=22050 looping=" .. LOOP .. " panning=0 buffer_null=1 pitch_w=3f800000"
local READ_AFTER = unlink_case and "installed=1 used:b playing=0 paused=1 volume=-1 rate=22050 looping=0 panning=0 buffer_null=1 pitch_w=3f800000"
	or "installed=1 used:b playing:b paused:b volume:i rate=22050 looping=1 panning:i buffer_null=1 pitch_w:w"
take ("env-route-input", "tag=record origin=test_owned_static_record built_by=fixture buffer=absent device=none user_data=none initial_used=1 initial_playing=" .. initial_playing .. " initial_paused=1 initial_volume=-1 rate=22050 looping=" .. (unlink_case and "0" or "1"))
local pause
if unlink_case then
	pause = take ("env-route-input", "tag=pause sub:n delay_w=3fc00000 cohort_before=0 complete_before=1 children_sound:n children_of_type:n complete_after:b called:b returned:i")
	-- a census that failed after the entity was created leaves the pause and the calls unmade: an unavailable prerequisite, never a result
	if pause.complete_after ~= "1" or pause.called ~= "1" then
		emit { event = "control_unavailable", i = case, complete_after = pause.complete_after, called = pause.called }
		refuse ("the cohort census was incomplete or invalid after the entity was created, or the pause was not made: the control did not run, an unavailable prerequisite and not a result")
	end
	take ("env-route-input", "tag=deltas calls=3 delta_w=3f800000")
else
	take ("env-route-input", "tag=play range_w=00000000 viewpoint=main_vp")
end
local before = take ("env-route-record", "tag=before " .. READ_BEFORE)

local states = {}
while rows [at] ~= nil and rows [at].event == "env-sound-state" do
	local map = take ("env-sound-state", STATE)
	states [#states + 1] = map
end
local want_tags = unlink_case and { "0", "99", "1", "2", "3" } or { "0", "1" }
if #states < 2 or #states > #want_tags then hard ("the report holds " .. #states .. " entity states where the case allows 2 to " .. #want_tags) end
for n, s in ipairs (states) do
	if s.ent ~= "0" or s.tag ~= want_tags [n] then hard ("the entity state " .. n .. " is not entity 0 at step " .. want_tags [n] .. ": " .. raw_of (s)) end
	if s.slot ~= states [1].slot then hard ("the entity state " .. n .. " is in slot " .. s.slot .. " where the first is in slot " .. states [1].slot) end
	if s.data == "1" then
		if s.etype ~= states [1].etype or s.sub ~= states [1].sub or s.looping ~= states [1].looping then hard ("the live entity state " .. n .. " changed type, sub type or looping against the first") end
	else
		if n ~= #states then hard ("the entity is dead at state " .. n .. " but later states follow") end
		if s.sub ~= "0" or s.valid ~= "0" or s.looping ~= "0" or s.member ~= "0" or s.handle ~= "0" or s.life_w ~= "00000000" or s.vlife_w ~= "00000000" then hard ("the dead entity state still shows data of the freed entity") end
	end
end
if states [1].data ~= "1" or states [1].valid ~= "1" or states [1].member ~= "1" or states [1].handle ~= "1" then hard ("the entity before the calls is not a valid effect in the update list that holds the handle") end
local last = states [#states]
local alive = last.data == "1"
local first_call_state = unlink_case and 2 or 1
if unlink_case then
	-- the pause names the sub type of the only effect the control created, and its census holds that one effect among the session's sound children
	if pause.sub ~= states [1].sub then hard ("the pause names sub type " .. pause.sub .. " where the created effect has sub type " .. states [1].sub .. ": " .. raw_of (pause)) end
	if pause.children_of_type ~= "1" or tonumber (pause.children_sound) < tonumber (pause.children_of_type) then hard ("the cohort census is not the one created effect among the session's sound children: " .. raw_of (pause)) end
	if tonumber (pause.returned) < 0 or tonumber (pause.returned) > tonumber (pause.children_of_type) then hard ("the pause returned " .. pause.returned .. " where it can have changed 0 or 1 effects: " .. raw_of (pause)) end
end

local after = take ("env-route-record", "tag=after " .. READ_AFTER)
local cleanup = take ("env-route-cleanup", "installed=1 entity_live_after:b detached:b destroyed_by_original:b destroyed_by_fixture:b used_final=0 playing_final=0 paused_final=0 volume_final=0 buffer_null_final=1")
local cleanup_raw = raw_of (cleanup)
if cleanup.entity_live_after ~= (alive and "1" or "0") then hard ("the cleanup says the entity is " .. (cleanup.entity_live_after == "1" and "live" or "gone") .. " where its last state is " .. (alive and "live" or "dead") .. ": " .. cleanup_raw) end
if cleanup.detached ~= (alive and last.handle or "0") then hard ("the cleanup says detached " .. cleanup.detached .. " where the entity's last state shows handle " .. (alive and last.handle or "none (dead)") .. ": " .. cleanup_raw) end
-- the handle was released by the original or by the fixture, never by neither and never by both; the facts come from the snapshot right after the steps
if after.used == "0" then
	if cleanup.destroyed_by_original ~= "1" or cleanup.destroyed_by_fixture ~= "0" then hard ("the handle was no longer in use after the steps but the cleanup does not say the original released it: " .. cleanup_raw) end
	if alive and last.handle ~= "0" then hard ("the original released the handle but the live entity still points at it: " .. cleanup_raw) end
else
	if cleanup.destroyed_by_original ~= "0" or cleanup.destroyed_by_fixture ~= "1" then hard ("the handle was still in use after the steps but the cleanup does not say the fixture released it: " .. cleanup_raw) end
	if not alive or last.handle ~= "1" then hard ("the handle was still in use after the steps but the entity is gone or does not hold it: " .. cleanup_raw) end
end

local report = take ("env-snd-report", "armed=1 capacity=80 stored:n dropped=0 overflow=0 creations=1 updates:n kills:n frames=0 tracked=1 open=0 track_all=0 untracked=0")
local records = {}
while rows [at] ~= nil and rows [at].event == "env-snd" do records [#records + 1] = take ("env-snd", RECORD) end
local planned = unlink_case and 3 or 1
local end_map = take ("env-sound-end", "i=" .. case .. " changed=1 installed=1 entities=1 planned_entities=1 short_of_slots=0 calls:n skipped:n descriptor_restored=1 delta_restored=1 flag_restored=1 handle_released=1 process_isolated=required")
if rows [at] ~= nil then hard ("the report has a row after its end row: " .. rows [at].raw) end
local calls, skipped = tonumber (end_map.calls), tonumber (end_map.skipped)
if calls + skipped ~= planned or calls < 1 or calls ~= #states - first_call_state then hard ("the end row says " .. end_map.calls .. " calls and " .. end_map.skipped .. " skipped where the states show " .. (#states - first_call_state) .. " calls of " .. planned) end

-- ---------------------------------------------------------------------------------------------------------------- the probe's records against the calls and the entity
if tonumber (report.stored) ~= #records then hard ("the probe says it stored " .. report.stored .. " records and the report holds " .. #records) end
local creation = records [1]
if creation == nil or creation.point ~= "0" or creation.upd ~= "0" or creation.handle ~= "0" then hard ("the report holds no creation record first (update 0, no handle yet)") end
-- the constructor's metadata against the declared inputs: the sample descriptor (controlled in case 23, the retail one in case 24), the original rate of the facts, one sample, and the looping the case installed
if creation.rate ~= (unlink_case and "4" or descriptor.retail_rate) or creation.size ~= (unlink_case and "20" or descriptor.retail_size) or creation.orate ~= facts.lightning_original_rate then
	hard ("the creation record shows the descriptor rate " .. creation.rate .. " size " .. creation.size .. " original rate " .. creation.orate .. " where the inputs and the facts give " .. (unlink_case and "4, 20" or (descriptor.retail_rate .. ", " .. descriptor.retail_size)) .. " and " .. facts.lightning_original_rate)
end
if creation.seqcount ~= "1" or creation.sample == "-1" or creation.looping ~= LOOP then hard ("the creation record is not a single-sample effect with a sample and the looping the case installed: " .. raw_of (creation)) end
-- what the constructor and the controlled descriptor fix; validity and update-list membership change on this route (the pause, the unlink) and are joined to the entity states of the same instant instead
local STABLE = { "entity", "etype", "sub", "channel", "locality", "panning", "seqcount", "looping", "session_parent", "sample", "rate", "size", "orate", "amp_w", "pitch_w", "px_w", "py_w", "pz_w" }
local enters = 0
for n, r in ipairs (records) do
	local point = tonumber (r.point)
	if tonumber (r.seq) ~= n - 1 or point > 5 then hard ("the probe record " .. n .. " is out of sequence or of an unknown point") end
	if r.tracked ~= "0" or r.n ~= "0" or r.frame ~= "0" then hard ("the probe record " .. r.seq .. " is not of the one tracked entity in the open-less, frame-less episode: " .. raw_of (r)) end
	if n > 1 then
		if not (NEXT [tonumber (records [n - 1].point)] or {}) [point] then hard ("the probe record " .. r.seq .. ": point " .. point .. " is not an entry of the server routine, which is all the invalid unlink route and the direct playback reach (" .. raw_of (r) .. ")") end
		if r.handle ~= "1" or r.delta_w ~= "3f800000" then hard ("the probe record " .. r.seq .. " shows no handle or another delta word than the control set: " .. raw_of (r)) end
		for _, key in ipairs (STABLE) do
			if r [key] ~= creation [key] then hard ("the probe record " .. r.seq .. " shows " .. key .. " " .. r [key] .. " where the creation shows " .. creation [key]) end
		end
		enters = enters + 1
		if tonumber (r.upd) ~= enters then hard ("the probe record " .. r.seq .. " is update " .. r.upd .. " where it is the " .. enters .. "th entry of an update") end
	end
end
if not unlink_case and #records ~= 1 then hard ("the direct playback calls no update routine, so the probe holds only the creation") end
if tonumber (report.updates) ~= enters or tonumber (report.kills) ~= 0 then hard ("the probe counts " .. report.updates .. " updates and " .. report.kills .. " kills where the records hold " .. enters .. " entries and none") end
if unlink_case and enters ~= calls then hard ("the probe holds " .. enters .. " entries and the end row says " .. calls .. " calls") end
-- the state words are the probe's words of the same instant: the creation is the first state; the entry of update u is the state after the pause (u = 1) or after call u-1
local function same_instant (state, r, what)
	if state.life_w ~= r.life_w or state.vlife_w ~= r.vlife_w or state.valid ~= r.valid or state.member ~= r.update_member or state.handle ~= r.handle or state.looping ~= r.looping or state.sub ~= r.sub then
		hard (what .. " does not agree with the entity state of the same instant: " .. raw_of (r))
	end
end
if states [1].slot ~= creation.entity or states [1].etype ~= creation.etype or states [1].sub ~= creation.sub or states [1].looping ~= creation.looping or states [1].valid ~= creation.valid or states [1].member ~= creation.update_member
	or states [1].life_w ~= creation.life_w or states [1].vlife_w ~= creation.vlife_w then
	hard ("the first entity state is not the creation record's entity: " .. raw_of (creation))
end
if unlink_case then
	for n = 2, #records do same_instant (states [tonumber (records [n].upd) + 1], records [n], "the entry record " .. records [n].seq) end
	-- every call the helper made had its reason: each state from the pause on, but the last, is a live member of the update list, and fewer than three calls mean a dead entity or one that left it
	for n = 2, #states - 1 do
		if states [n].data ~= "1" or states [n].member ~= "1" then hard ("the helper made a call although the entity was not a live member of the update list after state " .. n) end
	end
	if calls < 3 and alive and last.member == "1" then hard ("the helper stopped after " .. calls .. " of 3 calls although the entity is live and still a member of the update list") end
	-- the pause: the valid flag changes exactly when the pause says it changed one effect
	local flipped = states [1].valid ~= states [2].valid
	if (pause.returned == "1") ~= flipped then hard ("the pause returned " .. pause.returned .. " but the entity's validity " .. (flipped and "changed" or "did not change")) end
end

-- ---------------------------------------------------------------------------------------------------------------- what the driver JOINS (labelled derived; never a measurement of anything but the flags it names)
local derived = {
	event = "derived", source = "the driver's join of the records above; not a measurement", case = case, label = LABELS [case], calls = calls,
	original_released_handle = cleanup.destroyed_by_original == "1", fixture_released_handle = cleanup.destroyed_by_fixture == "1", entity_live_after = alive,
	entity_member_after = alive and last.member == "1", entity_handle_after = alive and last.handle == "1",
	record_playing_before = before.playing, record_playing_after = after.playing, record_paused_after = after.paused, record_volume_after = after.volume,
	effects_enabled_before = flags.effects_enabled_before, ui_muted = flags.ui_muted, channel_muted = flags.channel_muted,
	scope = "controlled test state: a buffer-absent record in the static array, not an original product, not a backend and not audio; the DirectSound entry points reached are the headless null stubs; a playing buffer is unsupported; the full report is the private raw sidecar",
}
if unlink_case then
	derived.observed = "the pause's returned count, the entity's validity, update-list membership and handle at each state, the record's use before and after, and who released it; unlinked, handle_cleared_by_original and original_released_handle are computed from them and are true or false as the report says (an entity that stayed linked, with the handle released by the fixture, is as coherent as one that was unlinked)"
	derived.pause_returned = pause.returned
	derived.unlinked = states [2].member == "1" and alive and last.member == "0"
	derived.handle_cleared_by_original = states [2].handle == "1" and alive and last.handle == "0"
else
	local volume_changed = after.volume ~= before.volume
	local play_body = after.paused == "0"
	-- source order of se_snd.c play_local_sound: set volume, then (if the record is not playing) play; the play body is entered only when playing is clear
	if play_body and not volume_changed then hard ("the play body ran (paused cleared) but the set volume body did not (volume unchanged), which the source order excludes: " .. raw_of (after)) end
	if volume_changed and not play_body then hard ("the set volume body ran but the looping record was neither played nor released, which the source order excludes: " .. raw_of (after)) end
	if play_body and after.playing ~= "1" then hard ("the play body ran (paused cleared) but the record is not playing afterwards: " .. raw_of (after)) end
	derived.measured = "the record's volume, paused and playing flags before and after, the entity's handle and the release"
	derived.set_volume_reached = volume_changed
	derived.play_body_reached = play_body
	derived.getter_write_attributed = play_body and before.playing == "1"
	derived.getter_attribution_basis = "measured: paused cleared and playing set; inferred from the source order: the play body is entered only when playing is clear, so the stale marker was cleared by the get playing body first"
	derived.path_returned_before_play = not play_body
	derived.scan_and_stop_unexercised = true
	derived.scan_and_stop_reason = "scan_local_entity_sounds needs get_application_sound_system_status; stop_local_entity_sounds walks get_first_system_sound_effect whose bound is the static count only initialise_sound_system sets (zero here), and the sound system is never initialised"
end
emit (derived)
emit { event = "result", case = case }

-- BOTH evidence files are closed here, the private raw first so that a failure to close it can still be written to the JSON; every close is attempted, every failure is reported and the run then fails closed
local close_errors = {}
for position = #opened, 1, -1 do
	local name = opened [position]
	local closed, close_error = files [name]:close ()
	if not closed then
		close_errors [#close_errors + 1] = "closing " .. paths [name] .. " failed: " .. tostring (close_error)
		if name ~= "out" then pcall (emit, { event = "output_failure", reason = close_errors [#close_errors] }) end
	end
end
if #close_errors > 0 then refuse ("OUTPUT FAILURE: " .. table.concat (close_errors, "; ")) end
io.stderr:write (string.format ("env-sound-route: case=%d calls=%d handle_released_by_original=%s entity_live=%s\n", case, calls, tostring (cleanup.destroyed_by_original == "1"), tostring (alive)))
