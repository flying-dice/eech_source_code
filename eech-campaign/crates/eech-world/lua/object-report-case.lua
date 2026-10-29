-- TEST-ONLY controlled witness of a physical fact the host reports about a structure (EEDCS-46, R3): the host-issued identity of a structure and the report, by that identity, that the structure is
-- damaged or destroyed.  It uses only the shipping module (engine:structures, engine:report_object), so it needs no fixture build; it changes no campaign rule and takes no sample of any random stream.
--
-- WHAT IT DOES. From a verified retail boot (Lebanon by default, Georgia with scenario=georgia_retail) and a warm-up it lists the structures (the host is asked for the identity of each, once), chooses the
-- first one of the requested kind that is alive, records it, and then, according to `report`, REPORTS a physical fact about it by its identity: `destroyed` (the host says it is destroyed), `damage` (the host
-- says its damage level is now `level`) or `none` (the host reports nothing: the control that shows what a report changes).  It runs a bounded window of frames and records the structure again, and the
-- structures' counts by kind.  What the campaign did is judged from the state it shows afterwards (alive, damage level) and, for a segment, from the host questions the real handlers asked (the aim
-- provider's bridge counters), never from the report's receipt alone.  The war runs on as it does.
--
-- CONTROLS (the last step of a run, after the window; a refusal poisons the engine, so it is the end of the run): `control` = unknown (an identity nobody issued), duplicate (the destruction reported
-- again: a fact is applied once) or raise (a damage level above the current one: a report cannot repair), or none.  Each control records whether the campaign refused it and the engine's error text.
--
-- ARGUMENTS (key=value). root, out (required); scenario=lebanon_retail|georgia_retail; warmup=<seconds>; frame_ms; window_seconds; kind=segment|site|city_building|pylon|scenic|site_updatable (default site);
-- report=destroyed|damage|none (default destroyed); level=<damage level for report=damage; default half the current level>; index=<which alive structure of the kind, from 0; default 0>;
-- identity=reference|own (which host issues the identities: the reference host's, or the plain model of lua/own-weapon.lua).
local args = host.args

local MAX_FRAMES = 10000

local function refuse (message)
	error ("object-report-case: " .. message, 0)
end

local function number_arg (name, value, default, maximum, integer)
	if value == nil then return default end
	local n = tonumber (value)
	if n == nil or n ~= n or n < 0 or n > maximum or (integer and n ~= math.floor (n)) then
		refuse (name .. " must be a " .. (integer and "whole " or "") .. "number from 0 to " .. maximum .. ", got '" .. tostring (value) .. "'")
	end
	return n
end

local function choice_arg (name, value, default, allowed)
	if value == nil then return default end
	for _, option in ipairs (allowed) do if value == option then return value end end
	refuse (name .. " must be one of " .. table.concat (allowed, ", ") .. ", got '" .. tostring (value) .. "'")
end

-- every argument is checked before anything is booted or reported
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
local frame_ms = number_arg ("frame_ms", args.frame_ms, 100, 1000, true)
if frame_ms < 1 then refuse ("frame_ms must be at least 1, got '" .. tostring (args.frame_ms) .. "'") end
local warmup = number_arg ("warmup", args.warmup, 5, 600, false)
local window_seconds = number_arg ("window_seconds", args.window_seconds, 12, 600, false)
local kind = choice_arg ("kind", args.kind, "site", { "segment", "site", "city_building", "pylon", "scenic", "site_updatable" })
local report = choice_arg ("report", args.report, "destroyed", { "destroyed", "damage", "none" })
local control = choice_arg ("control", args.control, "none", { "none", "unknown", "duplicate", "raise" })
local identity = choice_arg ("identity", args.identity, "reference", { "reference", "own" })
local explicit_level = number_arg ("level", args.level, nil, 1000000, true)
local pick = number_arg ("index", args.index, 0, 1000000, true)
if control == "duplicate" and report ~= "destroyed" then refuse ("control=duplicate needs report=destroyed") end
if control == "raise" and report == "none" then refuse ("control=raise needs a report that is not none") end
local warmup_frames = math.ceil (warmup * 1000 / frame_ms)
local window_frames = math.ceil (window_seconds * 1000 / frame_ms)
if warmup_frames + window_frames > MAX_FRAMES then
	refuse ("warmup and window_seconds need " .. (warmup_frames + window_frames) .. " frames of " .. frame_ms .. " ms; the most this witness records is " .. MAX_FRAMES)
end
local seed = 1

local out = assert (io.open (out_path, "w"))

local here = (debug.getinfo (1, "S").source:match ("^@?(.*[/\\])") or "")
local writer = dofile (here .. "combat-report.lua") (out)
local emit = writer.emit

-- the host that issues the identities: the reference host's own, or a plain model that is not EECH's (only the identify question is the model's)
local aim = host.aim
if identity == "own" then
	local own = dofile (here .. "own-weapon.lua")
	local reference = host.aim
	aim = function (step)
		if step.mode == "identify" then return own.identify (step) end
		return reference (step)
	end
end
-- what the campaign asked the host while the case ran, by request mode (a bridge question by its op): the real handlers of a destroyed segment ask the host to decide the collapse, which is how a
-- report is seen to have reached them
local asked, named, named_chosen = {}, {}, {}
local chosen_id = nil
local issuer = aim
aim = function (step)
	local name = step.mode == "bridge" and ("bridge_" .. tostring (step.op)) or tostring (step.mode)
	asked [name] = (asked [name] or 0) + 1
	-- a question about a physical subject (a hit, a health fact, a weapon's death, a weapon step) carries the identity of the subject it is about: counted by mode, and whether it is the chosen structure's
	if step.target_host_id ~= nil then
		named [name] = (named [name] or 0) + 1
		if step.target_host_id == chosen_id then named_chosen [name] = (named_chosen [name] or 0) + 1 end
	end
	return issuer (step)
end

local dc = require ("eech_dc")
dc.prepare_installation (root)
local engine = dc.boot {
	install_root = root, map = scenario.map, campaign_directory = "camp01", campaign = scenario.campaign, gunship = "apache",
	seed = seed, bind_mobile = host.bind_mobile, move_vehicle = host.move_vehicle, observe_loss = host.observe_loss,
	supply_reached = host.supply_reached, line_of_sight = host.line_of_sight, sense = host.sense, aim = aim,
}
emit { event = "boot", scenario = scenario_name, case = "object_report", warmup = warmup, frame_ms = frame_ms, window_seconds = window_seconds, kind = kind, report = report, control = control,
	identity = identity, seed = seed }

local clock = engine:clock ()
local frames_run = 0
local function step_frame ()
	local before = clock.elapsed_seconds
	frames_run = frames_run + 1
	if frames_run > MAX_FRAMES + 4 then error ("object-report-case: more than " .. MAX_FRAMES .. " frames were run", 0) end
	engine:frame (frame_ms)
	clock = engine:clock ()
	if not (clock.elapsed_seconds > before) then error ("object-report-case: the clock did not advance in a frame of " .. frame_ms .. " ms", 0) end
end

while clock.elapsed_seconds < warmup do step_frame () end

-- the structures, each with the identity the host issued (asked of the host here, once); the counts by kind
local function census ()
	local list = engine:structures ()
	local by_kind, alive = {}, {}
	for _, s in ipairs (list) do
		by_kind [s.kind] = (by_kind [s.kind] or 0) + 1
		if s.alive then alive [s.kind] = (alive [s.kind] or 0) + 1 end
	end
	return list, by_kind, alive
end

local list, by_kind, alive_by_kind = census ()
local ids, duplicates = {}, 0
for _, s in ipairs (list) do
	if ids [s.host_id] then duplicates = duplicates + 1 end
	ids [s.host_id] = true
end
local function flatten (row, counts, prefix)
	for k, v in pairs (counts) do row [prefix .. k] = v end
	return row
end
emit (flatten (flatten ({ event = "object_census", structures = #list, duplicate_identities = duplicates }, by_kind, "n_"), alive_by_kind, "alive_"))

local chosen, seen = nil, 0
for _, s in ipairs (list) do
	if s.kind == kind and s.alive then
		if seen == pick then chosen = s break end
		seen = seen + 1
	end
end
if chosen == nil then
	emit { event = "object_summary", done = false, reason = "no alive structure of kind " .. kind .. " at index " .. pick }
	out:close ()
	print ("OBJECT_CASE_DONE")
	return
end
chosen_id = chosen.host_id
emit { event = "object_before", host_id = chosen.host_id, kind = chosen.kind, sub_type = chosen.sub_type, side = chosen.side, alive = chosen.alive, damage_level = chosen.damage_level,
	initial_damage_level = chosen.initial_damage_level, x = chosen.position [1], y = chosen.position [2], z = chosen.position [3] }

local receipt
if report == "destroyed" then
	receipt = engine:report_object { host_id = chosen.host_id, destroyed = true }
elseif report == "damage" then
	local level = explicit_level or math.floor (chosen.damage_level / 2)
	receipt = engine:report_object { host_id = chosen.host_id, damage_level = level }
end
if receipt ~= nil then
	emit { event = "object_receipt", host_id = chosen.host_id, killed = receipt.killed, rostered = receipt.rostered, entity_type = receipt.entity_type,
		damage_level_before = receipt.damage_level_before, damage_level_after = receipt.damage_level_after }
end

for _ = 1, window_frames do step_frame () end

local after_list, after_by_kind, after_alive = census ()
local after
for _, s in ipairs (after_list) do if s.host_id == chosen.host_id then after = s end end
emit (flatten ({ event = "object_after", host_id = chosen.host_id, present = after ~= nil, alive = after and after.alive, damage_level = after and after.damage_level }, after_alive, "alive_"))
local changed = after ~= nil and (after.alive ~= chosen.alive or after.damage_level ~= chosen.damage_level)
emit { event = "object_summary", done = true, report = report, changed = changed, alive_before = chosen.alive, alive_after = after and after.alive, damage_before = chosen.damage_level,
	damage_after = after and after.damage_level, structures_before = #list, structures_after = #after_list }

local counts = { event = "object_aim_counts" }
for name, count in pairs (asked) do counts ["asked_" .. name] = count end
for name, count in pairs (named) do counts ["subject_id_" .. name] = count end
for name, count in pairs (named_chosen) do counts ["subject_is_chosen_" .. name] = count end
emit (counts)
-- the control: a report the campaign must refuse.  It is the last step: a refusal poisons the engine
if control ~= "none" then
	local request
	if control == "unknown" then request = { host_id = "nobody-issued-this-identity", destroyed = true }
	elseif control == "duplicate" then request = { host_id = chosen.host_id, destroyed = true }
	elseif control == "raise" then request = { host_id = chosen.host_id, damage_level = chosen.initial_damage_level + 1 } end
	local ok, err = pcall (engine.report_object, engine, request)
	emit { event = "object_control", control = control, refused = not ok, error = ok and "" or tostring (err) }
end

out:close ()
print ("OBJECT_CASE_DONE")
