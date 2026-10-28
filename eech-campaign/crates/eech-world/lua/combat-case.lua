-- TEST-ONLY controlled combat witness for the ORIGINAL campaign (EEDCS-46). Needs a DLL built with the `weapon-fixture` feature. DESIGN UNDER REVIEW: this script has been
-- parsed but NOT RUN; no build or native slot has been given to it.
--
-- The natural war (lua/combat-characterization.lua) leaves some of the physical combat paths unreached. This script puts those to the campaign's own code, one case at a time,
-- through csrc/eech_cmb_case.c, and the observer (csrc/eech_cmb_fixture.c) records what the original did. Every case is a PRECONDITION chosen here and nothing more: the
-- receiver is "the n-th living entity of a kind, in the entity list's order", the weapon is a real weapon of a type some launcher really carries (found by the Task45
-- read-only query, never invented), the launcher is that launcher, and the modifier, offset, kill code and starting health are numbers written below. What happens next is the
-- original's. Campaign consequences (credit, losses, speech, force notifications) happen as the original does them. A case that cannot be done (no living entity of the kind,
-- no launcher carrying a weapon of the class, a refused launch) is recorded as not done, never replaced by something else.
--
-- Output, JSON Lines (the same writer and parser as the natural run, lua/combat-report.lua):
--   boot, census (before), inventory
--   for every case: `case` (the name, the engine's text, ok, `done`), the observer's whole report immediately BEFORE (scope case:<name>:before) and AFTER (case:<name>:after), the
--                   ordered events the case produced (scope case:<name>:log), and `case_delta`: every observer counter that CHANGED during the case, as key = after - before.
--                   Events the war logged between two cases are kept, not discarded (scope pre:<name>), so the whole run's event log is in the file, in order.
--   census (after), run (the final whole report)
--
-- WHAT `done` AND THE RUN SUMMARY MEAN: `done` says only that the PRECONDITION was met (a receiver of the kind was found and, for a health case, the original critical predicate
-- accepted it).  It is NOT a statement that a physical path ran.  The path ran if and only if the case's `case_delta` shows it: a collision on the kind, a capability draw, a
-- blast call, a timed roll's draws and hits.  The natural-run audit (tools/combat-trace-audit.py) does NOT judge this file; a controlled acceptance reads the deltas.
--
-- Declared source-unreachable: a person's bleeding roll.  The original `vehicle_critically_damaged` returns FALSE for ENTITY_TYPE_PERSON before it looks at the health, so no health
-- reaches it; the case below asks the original predicate once and records the answer.  An anti-aircraft vehicle's fire roll needs an explosive or flammable vehicle type and a
-- health whose integer percentage is strictly below 15; the driver sets the health, asks the original predicate, and puts the health back (and says so) if it is not critical.
--
-- host.args: root, out (JSON Lines path), warmup (simulated seconds before the first case, default 60, the controlled-launch witnesses' warmup), frame_ms (default 100),
--            timed_seconds (simulated seconds the timed-health cases run for, default 20)

io.stdout:setvbuf ("no")

local args = host.args
local root = assert (args.root, "root=<installation root>")
local out = assert (io.open (assert (args.out, "out=<JSON Lines path>"), "w"))
local warmup = tonumber (args.warmup or "60")
local frame_ms = tonumber (args.frame_ms or "100")
local timed_seconds = tonumber (args.timed_seconds or "20")
local seed = 1

local here = (debug.getinfo (1, "S").source:match ("^@?(.*[/\\])") or "")
local writer = dofile (here .. "combat-report.lua") (out)
local emit, emit_report = writer.emit, writer.emit_report
-- the report flattening and the deltas are a file of their own too (loaded before the boot, which changes the working directory), so that the host tests load the same code
local counting = dofile (here .. "combat-counters.lua")
local counters, deltas = counting.counters, counting.deltas

local order = {}
local function bind_mobile (descriptor)
	local id = host.bind_mobile (descriptor)
	order [#order + 1] = id
	return id
end

local dc = require ("eech_dc")
dc.prepare_installation (root)
engine = dc.boot {
	install_root = root, map = "..\\common\\maps\\map5", campaign_directory = "camp01", campaign = "LEBANON.CHC", gunship = "apache",
	seed = seed, bind_mobile = bind_mobile, move_vehicle = host.move_vehicle, observe_loss = host.observe_loss,
	supply_reached = host.supply_reached, line_of_sight = host.line_of_sight, sense = host.sense, aim = host.aim,
}
emit { event = "boot", scenario = "lebanon_retail", case = "controlled_combat", warmup = warmup, frame_ms = frame_ms, timed_seconds = timed_seconds, seed = seed }

local clock = engine:clock ()
local function step (seconds_more)
	local until_time = clock.elapsed_seconds + seconds_more
	local frames = 0
	while clock.elapsed_seconds < until_time do
		if frames > math.ceil (seconds_more * 1000 / frame_ms) + 10 then error ("the clock did not reach " .. tostring (until_time)) end
		engine:frame (frame_ms)
		frames = frames + 1
		clock = engine:clock ()
	end
end

local function fields (text)
	local t = {}
	for k, v in string.gmatch (text, "(%S+)=(%S+)") do t [k] = v end
	return t
end

-- everything the war logged before a case is taken first (and KEPT, under its own scope), so that the events read after it are the case's own and none is lost
local function drain (scope, at)
	local text = engine:cmb_events ()
	if scope then emit_report (scope, at, text) end
end

-- the kinds a case can choose a receiver among
local KINDS = { "ANTI_AIRCRAFT", "CITY_BUILDING", "FIXED_WING", "HELICOPTER", "OBJECT", "PERSON", "PYLON", "ROUTED_VEHICLE", "SCENIC", "SEGMENT", "SHIP_VEHICLE", "SITE", "SITE_UPDATABLE" }
local KILL = { HIT_LAND = 3, HIT_TARGET = 5, OVERSHOT_TARGET = 6, SELF_DESTRUCT = 7 }

local census = {}
local function take_census (scope)
	local text = engine:cmb_case_state (0, "", 0, 0)
	emit_report (scope, clock.elapsed_seconds, text)
	for line in string.gmatch (text, "[^\n]+") do
		local f = fields (line)
		if f.kind then census [(f.kind:gsub ("^ENTITY_TYPE_", ""))] = tonumber (f.live) end
	end
end

local case_count, case_done = 0, 0
local last_delta = {}
-- `precondition_ok` (optional) decides `done` from the engine's fields; the default is that a receiver was found.  `done` is a PRECONDITION, never a physical result.
local function run_case (name, call, after_steps, precondition_ok)
	case_count = case_count + 1
	drain ("pre:" .. name, clock.elapsed_seconds)
	local at = clock.elapsed_seconds
	local before_text = engine:cmb_trace ()
	emit_report ("case:" .. name .. ":before", at, before_text)
	local ok, text = pcall (call)
	local f = ok and fields (tostring (text)) or {}
	local done = ok and ((precondition_ok and precondition_ok (f)) or (not precondition_ok and f.found == "1" and f.placed ~= "0")) or false
	if done then case_done = case_done + 1 end
	if after_steps and done then step (after_steps) end
	emit { event = "case", name = name, ok = ok and true or false, done = done and true or false, text = (tostring (text):gsub ("\n", " ")) }
	local after_text = engine:cmb_trace ()
	emit_report ("case:" .. name .. ":after", clock.elapsed_seconds, after_text)
	drain ("case:" .. name .. ":log", clock.elapsed_seconds)
	local d = deltas (before_text, after_text)
	local row = { event = "case_delta", name = name, done = done and true or false }
	for k, v in pairs (d) do row [k] = v end
	emit (row)
	last_delta = d
	return done, f, d
end

-- ---------------------------------------------------------------------------------------------------------------- warmup, census, inventory (read-only)
step (warmup)
take_census ("census:before")

-- launchers carrying real weapons, from the Task45 read-only query, sorted by their numeric id
local launchers = {}
for _, id in ipairs (order) do
	local ok, text = pcall (engine.wpn_case_query, engine, id, 0)
	if ok then
		local s = fields (text)
		if s.alive == "1" then
			local inventory = {}
			for w = 1, 127 do
				local qok, qtext = pcall (engine.wpn_case_query, engine, id, w)
				if qok then
					local q = fields (qtext)
					if tonumber (q.weapon_count) and tonumber (q.weapon_count) > 0 then inventory [#inventory + 1] = { weapon = w, count = tonumber (q.weapon_count), warhead = tonumber (q.warhead) } end
				end
			end
			if #inventory > 0 then launchers [#launchers + 1] = { id = id, inventory = inventory } end
		end
	end
end
table.sort (launchers, function (a, b) return tonumber (string.match (a.id, "(%d+)$")) < tonumber (string.match (b.id, "(%d+)$")) end)

-- the weapon classes the damage arithmetic distinguishes by warhead: 1 solid shot, 3..6 high explosive (anti tank, dual purpose, plain, anti aircraft), 7 conventional munitions
local function carrier (wanted)
	for _, l in ipairs (launchers) do
		for _, item in ipairs (l.inventory) do
			if wanted (item.warhead) then return l.id, item.weapon, item.warhead end
		end
	end
end
local classes = {
	{ name = "solid_shot", wanted = function (w) return w == 1 end },
	{ name = "he_anti_tank", wanted = function (w) return w == 3 end },
	{ name = "he_dual_purpose", wanted = function (w) return w == 4 end },
	{ name = "he_plain", wanted = function (w) return w == 5 end },
	{ name = "he_anti_aircraft", wanted = function (w) return w == 6 end },
	{ name = "conventional", wanted = function (w) return w == 7 end },
}
local usable = {}
for _, c in ipairs (classes) do
	local launcher, weapon, warhead = carrier (c.wanted)
	emit { event = "weapon_class", class = c.name, launcher = launcher or "none", weapon = weapon or -1, warhead = warhead or -1 }
	if launcher then usable [#usable + 1] = { class = c.name, launcher = launcher, weapon = weapon } end
end
emit { event = "inventory", launchers = #launchers, usable_classes = #usable }

local function collide (kind, modifier, w, weaponless)
	return engine:cmb_case_act (1, kind, 0, w and w.launcher or "", w and w.weapon or 0, modifier, 0, weaponless and true or false)
end
local function blast (kind, offset, kill_code, w)
	return engine:cmb_case_act (2, kind, 0, w.launcher, w.weapon, offset, kill_code, false)
end

-- ---------------------------------------------------------------------------------------------------------------- cases
-- THE SMALLEST SET: the kinds the natural 600-second original run (natural-01, audited) never delivered a collision to, plus one reached kind as a control (the same case path on
-- a kind the war reaches, comparable with the natural events).  `kinds` and `control` can be given as comma lists to change them; the default is the measured unreached set.
local function list (text, default)
	local t = {}
	for item in string.gmatch (text or default, "[^,]+") do t [#t + 1] = item end
	return t
end
local kinds = list (args.kinds, "OBJECT,PERSON,PYLON,SCENIC,SITE,SITE_UPDATABLE")
local control = list (args.control, "ROUTED_VEHICLE")
local selected = {}
for _, k in ipairs (kinds) do selected [#selected + 1] = k end
for _, k in ipairs (control) do selected [#selected + 1] = k end

-- two weapon classes are enough to take both arithmetic branches (armour penetration, high-explosive splash): a high-explosive class and a solid-shot or conventional one
local chosen = {}
for _, wanted in ipairs { { "he_dual_purpose", "he_plain", "he_anti_aircraft", "he_anti_tank" }, { "solid_shot", "conventional" } } do
	for _, name in ipairs (wanted) do
		for _, w in ipairs (usable) do if w.class == name then chosen [#chosen + 1] = w break end end
		if #chosen > 0 and chosen [#chosen].class == name then break end
	end
end
for _, w in ipairs (chosen) do emit { event = "chosen_weapon_class", class = w.class, launcher = w.launcher, weapon = w.weapon } end

-- direct hit (modifier 0.0) and blast-range collision (0.5) by each chosen class on the first living entity of each selected kind: the arithmetic branch, the draw, the health
-- write and the fatal decision of that kind's handler
for _, kind in ipairs (selected) do
	if (census [kind] or 0) > 0 then
		for _, w in ipairs (chosen) do
			run_case ("direct_" .. kind .. "_" .. w.class, function () return collide (kind, 0.0, w, false) end)
			run_case ("blast_modifier_" .. kind .. "_" .. w.class, function () return collide (kind, 0.5, w, false) end)
		end
	else
		emit { event = "case_skipped", kind = kind, reason = "no living entity of the kind in the world" }
	end
end

-- the same entry with no weapon at all (the path a debug event takes): the no-weapon exit of the damage function and the handler's no-aggressor branch
for _, kind in ipairs (selected) do
	if (census [kind] or 0) > 0 then
		run_case ("weaponless_" .. kind, function () return collide (kind, 0.0, nil, true) end)
	end
end

-- the explosion effect: a land hit (which blasts twice), 5 m from the first living entity of each selected kind, so that the blast's sector walk finds the kind in range
local exploder = chosen [1]
if exploder then
	for _, kind in ipairs (selected) do
		if (census [kind] or 0) > 0 then
			run_case ("explosion_HIT_LAND_" .. kind, function () return blast (kind, 5.0, KILL.HIT_LAND, exploder) end)
		end
	end
	-- and one hit on target, which blasts everything but the target
	for _, kind in ipairs (selected) do
		if (census [kind] or 0) > 0 then
			run_case ("explosion_HIT_TARGET_" .. kind, function () return blast (kind, 5.0, KILL.HIT_TARGET, exploder) end)
			break
		end
	end
end

-- the timed health rolls the natural run never fired.  PERSON: the original predicate is FALSE for a person whatever its health, so the bleeding roll is source-unreachable; the
-- case asks the original predicate once and records its answer, and the run says so rather than claiming the roll was missed by chance.  ANTI_AIRCRAFT: the fire roll needs an
-- explosive or flammable vehicle type and an integer health percentage strictly below 15, so up to eight living vehicles are tried in order and the first the ORIGINAL predicate
-- accepts at 14 % of its initial health is run for timed_seconds; every vehicle tried is reported with the reason it was or was not eligible.
if (census.PERSON or 0) > 0 then
	run_case ("timed_health_PERSON_declared_unreachable", function () return engine:cmb_case_state (1, "PERSON", 0, 14) end, nil,
		function (f) return f.found == "1" end)
	emit { event = "declared_unreachable", path = "person_bleeding", reason = "vehicle_critically_damaged is FALSE for ENTITY_TYPE_PERSON in the original source (vehicle.c)" }
end
local fire_ran = false
for ordinal = 0, math.min (7, (census.ANTI_AIRCRAFT or 0) - 1) do
	if not fire_ran then
		local done, f, d = run_case ("timed_health_ANTI_AIRCRAFT_" .. ordinal, function () return engine:cmb_case_state (1, "ANTI_AIRCRAFT", ordinal, 14) end, timed_seconds,
			function (f) return f.found == "1" and f.eligible == "1" end)
		if done then
			fire_ran = true
			-- whether the fire roll ACTUALLY ran is read from the observer, not from `done`
			local draws, hits = d ["cmb-timed.kind=4.draws"] or 0, d ["cmb-timed.kind=4.hits"] or 0
			emit { event = "timed_activity", path = "anti_aircraft_fire", ordinal = ordinal, attempts = d ["cmb-timed.kind=4.attempts"] or 0, draws = draws, hits = hits, fatal = d ["cmb-timed.kind=4.fatal"] or 0,
				rolled = draws > 0, fired = hits > 0 }
		end
	end
end
if not fire_ran then
	emit { event = "timed_activity", path = "anti_aircraft_fire", rolled = false, fired = false, reason = (census.ANTI_AIRCRAFT or 0) > 0 and "no living anti-aircraft vehicle was eligible (explosive or flammable and critical at 14 percent)" or "no living anti-aircraft vehicle" }
end

-- ---------------------------------------------------------------------------------------------------------------- end
take_census ("census:after")
drain ("log", clock.elapsed_seconds)
emit { event = "run", elapsed = clock.elapsed_seconds, cases = case_count, preconditions_met = case_done, usable_classes = #usable, note = "preconditions_met is not physical coverage: read the case_delta rows" }
emit_report ("run", clock.elapsed_seconds, engine:cmb_trace ())
out:close ()
io.stderr:write (string.format ("combat-case: %d cases, %d preconditions met (not coverage: read the deltas), %d usable weapon classes, clock %s\n", case_count, case_done, #usable, tostring (clock.elapsed_seconds)))
