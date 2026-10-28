-- Public-path checks of the host aiming and weapon-system-readiness provider (EEDCS-44): what the campaign does with the host's physical answers, and how
-- it refuses a missing or malformed provider. Boots the campaign through require ("eech_dc") with a provider this script wraps around (or replaces) the
-- reference host's `host.aim`, runs frames, and prints one summary line. Works with the shipping binaries and with the `aim-fixture` build (which also
-- writes the aiming report of the run beside the summary, so the fixture can be shown to observe what the host answered, not what EECH would have).
--
-- It tests nothing about campaign policy itself: it shows that the campaign's weapon systems act on whatever physical aiming the host reports, and that a
-- bad host is a visible failure. One mode per process (the engine is one per process):
--   mode=reference    the reference host unchanged: the control
--   mode=ready_slow   the reference host, but the weapon-system-ready keyframe advances at a quarter of the rate the campaign sent (weapon systems become ready later)
--   mode=step_slow    the reference host, but the devices step at a tenth of the turn rates the campaign sent (devices reach their offsets later)
--   mode=invalid      the host reports every vectors request invalid (it has no solution): the campaign must not fire on vectors it was told are invalid
--   mode=own_model    a host with a model of its own that is not EECH's: the ready state is instant (closed 0 or open 2, no animation), the devices snap to the
--                     required offsets within their limits, the required offsets are a plain bearing of the intercept vector in the device frame, a guided weapon
--                     intercepts the target's point, and an unguided weapon has no solution (it needs no ballistic table, no draw, no EECH figure)
--   mode=table_drag   the reference host, but every table request carries three times the weapon's drag: the host's tables are its own (EEDCS-45), so its ballistic
--                     and lead answers must differ from the reference's (angle_sum), and so must the weapons the campaign then launches and where they fall
--   mode=table_slow   the same with half the muzzle velocity
--   mode=bad_table_nil, bad_table_state, bad_table_error    a provider whose answer to the first table request is nothing, a state that is not done or need_draw, or an error:
--                     the engine must stop AT BOOT with a clear error
--   mode=weapon_drag  the reference host, but every figures request carries four times the weapon's drag: the host flies its own weapons by the figures it was given
--                     (EEDCS-45), so the weapons the campaign launches fall and end differently from the reference's (kills, steps)
--   mode=weapon_short the same with a tenth of the weapon's cruise time: its weapons are over sooner
--   mode=weapon_fast  the same with twice the muzzle velocity: the weapons the campaign launches leave faster (the host's birth), and fly and end differently
--   mode=release_slow the same with a quarter of the weapon's rate of fire: the launchers let fewer rounds go (the host's rate-of-fire gate)
--   mode=seduce_never the reference host, but a seeker asked about a decoy is told the decoys have no effect: no seeker is drawn to a decoy (the host's seduction)
--   mode=reacquire_never the reference host, but a seeker whose decoy expired is told to drop the lock: it does not go back to the decoy's launcher (the campaign unlinks it)
--   mode=bad_step_*   a provider that answers the first weapon step malformed: the engine must stop with a clear error (bad_step_state, bad_step_lookup_kind, bad_step_action_kind,
--                     bad_step_nonfinite, bad_step_repeat, bad_step_error, bad_step_draw)
--   mode=bad_birth_action, bad_release_burst   the first weapon birth answered with a step's action, the first release with no rounds and no refusal
--   mode=missing      boot without an aim provider: must fail at boot with a clear error
--   mode=bad_*        a provider that answers the first request of a kind malformed: the engine must stop with a clear error (bad_nil, bad_state,
--                     bad_ready_state, bad_phase, bad_nan_vector, bad_draw_for_ballistic, bad_second_draw, bad_negative_range, bad_error)
-- host.args: root, scenario (lebanon_retail default, georgia_retail), seconds (default 300), frame_ms (default 100), mode (default reference),
--            out (JSON Lines summary path), trace (optional, the fixture build's aiming report path: the run-wide report of lua/aim-characterization.lua in
--            the same JSON Lines form, so tools/aim-trace-compare.py --scope run reads it)

io.stdout:setvbuf ("no")

local args = host.args
local root = assert (args.root, "root=<installation root>")
local seconds = tonumber (args.seconds or "300")
local frame_ms = tonumber (args.frame_ms or "100")
local mode = args.mode or "reference"
local out = assert (io.open (assert (args.out, "out=<JSON Lines path>"), "w"))
local scenarios = {
	lebanon_retail = { map = "..\\common\\maps\\map5", campaign = "LEBANON.CHC" },
	georgia_retail = { map = "..\\common\\maps\\map3", campaign = "GEORGIA.CHC" },
}
local scenario = assert (scenarios [args.scenario or "lebanon_retail"], "scenario=lebanon_retail|georgia_retail")

local function emit (t)
	local parts = {}
	for k, v in pairs (t) do
		local value = type (v) == "string" and string.format ("%q", v) or tostring (v)
		parts [#parts + 1] = string.format ("%q:%s", k, value)
	end
	table.sort (parts)
	out:write ("{" .. table.concat (parts, ",") .. "}\n")
	out:flush ()
end

-- one JSON object per report line (as lua/aim-characterization.lua): the first word is the event, `key=value` words are its fields
local string_keys = { physical = true, decision = true }
local function report_lines (scope, at, report)
	local lines = {}
	for line in string.gmatch (report, "[^\n]+") do
		local event, rest = string.match (line, "^(%S+)%s*(.*)$")
		local t = { event = event, scope = scope, at = at }
		if event == "aim-sample" then
			local key, text = string.match (rest, "^key=(%S+)%s+(.*)$")
			t.key = key
			t.text = text
		else
			for k, v in string.gmatch (rest, "(%S+)=(%S+)") do
				t [k] = (not string_keys [k] and tonumber (v)) or v
			end
		end
		local parts = {}
		for k, v in pairs (t) do
			local value = type (v) == "string" and string.format ("%q", v) or tostring (v)
			parts [#parts + 1] = string.format ("%q:%s", k, value)
		end
		table.sort (parts)
		lines [#lines + 1] = "{" .. table.concat (parts, ",") .. "}"
	end
	return lines
end

local reference = host.aim
-- what the campaign asked, by request mode, and what the host answered
local counts = { table = 0, range = 0, figures = 0, weapon_step = 0, weapon_birth = 0, release = 0, animate = 0, weapon_decoy = 0, weapon_seduce = 0, weapon_reacquire = 0, ready_update = 0, ready_command = 0, required = 0, step = 0, vectors = 0, lead = 0, ballistic = 0, intercept = 0 }
local answers = { valid = 0, invalid = 0, need_draw = 0, lead_ok = 0, lead_no = 0, ballistic_ok = 0, ballistic_no = 0, phase_moving = 0, angle_sum = 0 }
-- what the weapon steps came to: the passes the host was asked, the lookups it needed (by kind), the draws it asked for, and the kill codes it left (0 none ... 7)
local steps = { passes = 0, done = 0, need_lookup = 0, need_draw = 0, kills = {}, lookups = {}, births = 0, birth_velocity = 0, releases = 0, release_refused = 0, release_rounds = 0,
	decoy_updates = 0, decoy_restarts = 0, seduce_asked = 0, seduce_locked = 0, seduce_retargeted = 0, reacquire_asked = 0, reacquire_returned = 0 }
for code = 0, 7 do steps.kills [code] = 0 end

local function counted (step, answer)
	counts [step.mode] = counts [step.mode] + 1
	if step.mode == "vectors" then
		if answer.state == "need_draw" then answers.need_draw = answers.need_draw + 1
		elseif answer.valid then answers.valid = answers.valid + 1 else answers.invalid = answers.invalid + 1 end
	elseif step.mode == "lead" then
		if answer.state == "need_draw" then answers.need_draw = answers.need_draw + 1
		elseif answer.ok then answers.lead_ok = answers.lead_ok + 1 answers.angle_sum = answers.angle_sum + answer.angle else answers.lead_no = answers.lead_no + 1 end
	elseif step.mode == "ballistic" then
		if answer.ok then answers.ballistic_ok = answers.ballistic_ok + 1 answers.angle_sum = answers.angle_sum + answer.angle else answers.ballistic_no = answers.ballistic_no + 1 end
	elseif step.mode == "ready_update" and (answer.phase == 1 or answer.phase == 3) then
		answers.phase_moving = answers.phase_moving + 1
	elseif step.mode == "weapon_birth" then
		if answer.state == "done" then
			for _, action in ipairs (answer.actions) do
				if action.kind == 8 then steps.births = steps.births + 1 steps.birth_velocity = steps.birth_velocity + action.f [1] end
			end
		end
	elseif step.mode == "weapon_decoy" then
		if answer.state == "done" then
			steps.decoy_updates = steps.decoy_updates + 1
			if answer.actions [1].a == 1 then steps.decoy_restarts = steps.decoy_restarts + 1 end
		end
	elseif step.mode == "weapon_seduce" then
		if answer.state == "done" then
			steps.seduce_asked = steps.seduce_asked + 1
			steps.seduce_locked = steps.seduce_locked + answer.actions [1].a
			steps.seduce_retargeted = steps.seduce_retargeted + answer.actions [1].b
		end
	elseif step.mode == "weapon_reacquire" then
		if answer.state == "done" then
			steps.reacquire_asked = steps.reacquire_asked + 1
			steps.reacquire_returned = steps.reacquire_returned + answer.actions [1].a
		end
	elseif step.mode == "release" then
		steps.releases = steps.releases + 1
		if answer.refuse then steps.release_refused = steps.release_refused + 1 else steps.release_rounds = steps.release_rounds + answer.burst_size end
	elseif step.mode == "weapon_step" then
		steps.passes = steps.passes + 1
		if answer.state == "need_lookup" then
			steps.need_lookup = steps.need_lookup + 1
			for _, lookup in ipairs (answer.lookups) do steps.lookups [lookup.kind] = (steps.lookups [lookup.kind] or 0) + 1 end
		elseif answer.state == "need_draw" then
			steps.need_draw = steps.need_draw + 1
		else
			steps.done = steps.done + 1
			for _, action in ipairs (answer.actions) do
				-- the last state action of a step is the weapon as it ends the frame
				if action.kind == 1 then steps.last_kill = action.c end
			end
			steps.kills [steps.last_kill or 0] = steps.kills [steps.last_kill or 0] + 1
			steps.last_kill = nil
		end
	end
	return answer
end

local function with (step, changes)
	local copy = {}
	for k, v in pairs (step) do copy [k] = v end
	for k, v in pairs (changes) do copy [k] = v end
	return copy
end

-- the plain model of mode=own_model (and of the controlled launch witness with aim_model=own_model) is a file of its own, next to this one
local here = (debug.getinfo (1, "S").source:match ("^@?(.*[/\\])") or "")
local own = dofile (here .. "own-model.lua")

local provider
if mode == "reference" then
	provider = function (step) return counted (step, reference (step)) end
elseif mode == "ready_slow" then
	provider = function (step)
		if step.mode == "ready_update" then step = with (step, { rate = step.rate * 0.25 }) end
		return counted (step, reference (step))
	end
elseif mode == "step_slow" then
	provider = function (step)
		if step.mode == "step" then step = with (step, { heading_rate = step.heading_rate * 0.1, pitch_rate = step.pitch_rate * 0.1 }) end
		return counted (step, reference (step))
	end
elseif mode == "invalid" then
	provider = function (step)
		local answer = reference (step)
		if step.mode == "vectors" and answer.state == "done" then
			answer = { state = "done", valid = false, weapon_vector = { 0, 0, -1 }, target_vector = { 0, 0, -1 }, intercept_vector = { 0, 0, -1 }, intercept_range = 0 }
		end
		return counted (step, answer)
	end
elseif mode == "own_model" then
	provider = function (step) return counted (step, own [step.mode] (step)) end
elseif mode == "seduce_never" then
	provider = function (step)
		if step.mode == "weapon_seduce" then step = with (step, { effectiveness = { 0, 0, 0 } }) end
		return counted (step, reference (step))
	end
elseif mode == "reacquire_never" then
	provider = function (step)
		local answer = reference (step)
		if step.mode == "weapon_reacquire" and answer.state == "done" then answer.actions [1].a = 0 end
		return counted (step, answer)
	end
elseif mode == "weapon_drag" or mode == "weapon_short" or mode == "weapon_fast" or mode == "release_slow" then
	provider = function (step)
		if step.mode == "figures" then
			local figures = {}
			for k, v in pairs (step.figures) do figures [k] = v end
			if mode == "weapon_drag" then figures.drag_coefficient = figures.drag_coefficient * 4
			elseif mode == "weapon_short" then figures.cruise_time = figures.cruise_time * 0.1
			elseif mode == "weapon_fast" then figures.muzzle_velocity = figures.muzzle_velocity * 2
			else figures.rate_of_fire, figures.inverse_rate_of_fire = figures.rate_of_fire * 0.25, figures.inverse_rate_of_fire * 4 end
			step = with (step, { figures = figures })
		end
		return counted (step, reference (step))
	end
elseif mode == "table_drag" or mode == "table_slow" then
	provider = function (step)
		if step.mode == "table" then
			local figures = {}
			for k, v in pairs (step.figures) do figures [k] = v end
			if mode == "table_drag" then figures.drag_coefficient = figures.drag_coefficient * 3 else figures.muzzle_velocity = figures.muzzle_velocity * 0.5 end
			step = with (step, { figures = figures })
		end
		return counted (step, reference (step))
	end
elseif mode == "bad_table_nil" or mode == "bad_table_state" or mode == "bad_table_error" then
	local armed = true
	provider = function (step)
		local answer = reference (step)
		if armed and step.mode == "table" then
			armed = false
			if mode == "bad_table_nil" then return nil
			elseif mode == "bad_table_state" then return { state = "later" }
			else error ("provider failed on purpose") end
		end
		return counted (step, answer)
	end
elseif mode == "missing" then
	provider = nil
elseif mode:sub (1, 4) == "bad_" then
	-- the reference host answers until the first request of the kind the mode corrupts; that answer is malformed and the engine must stop there
	provider = dofile (here .. "aim-malformed.lua") (mode, reference, counted)
else
	error ("unknown mode " .. mode)
end

local dc = require ("eech_dc")
dc.prepare_installation (root)
local config = {
	install_root = root, map = scenario.map, campaign_directory = "camp01", campaign = scenario.campaign, gunship = "apache",
	seed = 1, bind_mobile = host.bind_mobile, move_vehicle = host.move_vehicle, observe_loss = host.observe_loss,
	supply_reached = host.supply_reached, line_of_sight = host.line_of_sight, sense = host.sense, aim = provider,
}
local ok, engine = pcall (dc.boot, config)
if not ok then
	emit { event = "boot_failed", mode = mode, error = (tostring (engine):gsub ("\n.*", "")) }
	out:close ()
	io.stderr:write ("aim-provider: boot failed: " .. (tostring (engine):gsub ("\n.*", "")) .. "\n")
	assert (mode == "missing" or mode:sub (1, 10) == "bad_table_", "boot failed")
	return
end
assert (mode ~= "missing" and mode:sub (1, 10) ~= "bad_table_", "boot succeeded without a usable aim provider")

local fixture = engine.aim_trace ~= nil
if fixture then engine:aim_trace_reset () end
local frames = math.floor (seconds * 1000 / frame_ms)
local seen, births, lost = {}, 0, 0
local last_alive = {}
local failure
local frames_run = 0
for f = 1, frames + 10 do
	local frame_ok, err = pcall (engine.frame, engine, frame_ms)
	if not frame_ok then failure = (tostring (err):gsub ("\n.*", "")) break end
	frames_run = f
	if f % 10 == 0 then
		local present = {}
		for _, o in ipairs (engine:objects ()) do
			if o.kind == "weapon" then
				present [o.id] = true
				if not seen [o.id] then births = births + 1 end
			elseif o.kind == "ground_vehicle" or o.kind == "air_defence" or o.kind == "helicopter" or o.kind == "fixed_wing" then
				if last_alive [o.id] and not o.alive then lost = lost + 1 end
				last_alive [o.id] = o.alive
			end
		end
		seen = present
	end
	-- the run ends at the first frame after which the simulated clock has reached `seconds` (as the characterisation's checkpoints do), so a report taken
	-- here is comparable with the characterisation's report at the same checkpoint
	if engine:clock ().elapsed_seconds >= seconds then break end
end
if fixture and args.trace and not failure then
	local trace = assert (io.open (args.trace, "wb"))
	local clock = engine:clock ()
	trace:write (string.format ("{\"event\":\"boot\",\"mode\":%q,\"scenario\":%q,\"seconds\":%s}\n", mode, args.scenario or "lebanon_retail", tostring (seconds)))
	trace:write (string.format ("{\"elapsed\":%s,\"event\":\"run\",\"frames\":%d}\n", tostring (clock.elapsed_seconds), frames_run))
	trace:write (table.concat (report_lines ("run", clock.elapsed_seconds, engine:aim_trace ()), "\n") .. "\n")
	trace:close ()
end
emit { event = "summary", mode = mode, frames = frames_run, failure = failure or "", table = counts.table, ready_update = counts.ready_update, ready_command = counts.ready_command,
	required = counts.required, step = counts.step, vectors = counts.vectors, lead = counts.lead, ballistic = counts.ballistic, intercept = counts.intercept,
	vectors_valid = answers.valid, vectors_invalid = answers.invalid, need_draw = answers.need_draw, lead_ok = answers.lead_ok, lead_no = answers.lead_no,
	ballistic_ok = answers.ballistic_ok, ballistic_no = answers.ballistic_no, ready_moving = answers.phase_moving, weapon_births = births, losses = lost, angle_sum = string.format ("%.9g", answers.angle_sum),
	weapon_step_passes = steps.passes, weapon_step_done = steps.done, step_need_lookup = steps.need_lookup, step_need_draw = steps.need_draw,
	kill_exhausted = steps.kills [1], kill_out_of_bounds = steps.kills [2], kill_hit_land = steps.kills [3], kill_hit_water = steps.kills [4], kill_hit_target = steps.kills [5],
	kill_overshot = steps.kills [6], kill_self_destruct = steps.kills [7], births_done = steps.births, birth_velocity_sum = string.format ("%.9g", steps.birth_velocity),
	releases = steps.releases, release_refused = steps.release_refused, release_rounds = steps.release_rounds,
	decoy_updates = steps.decoy_updates, decoy_restarts = steps.decoy_restarts, seduce_asked = steps.seduce_asked, seduce_locked = steps.seduce_locked,
	seduce_retargeted = steps.seduce_retargeted,
	reacquire_asked = steps.reacquire_asked, reacquire_returned = steps.reacquire_returned }
out:close ()
io.stderr:write (string.format ("aim-provider: mode=%s ready_update=%d ready_command=%d required=%d step=%d vectors=%d (valid %d invalid %d) lead=%d ballistic=%d intercept=%d need_draw=%d ready_moving=%d weapon_births=%d losses=%d failure=%s\n",
	mode, counts.ready_update, counts.ready_command, counts.required, counts.step, counts.vectors, answers.valid, answers.invalid, counts.lead, counts.ballistic, counts.intercept,
	answers.need_draw, answers.phase_moving, births, lost, failure or "none"))
if mode:sub (1, 4) == "bad_" then
	assert (failure, "a malformed provider answer was accepted: mode " .. mode)
elseif failure then
	error (failure)
end
