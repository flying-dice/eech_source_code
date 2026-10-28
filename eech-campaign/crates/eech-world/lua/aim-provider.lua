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
--   mode=forecast_armoured the reference host, but every damage forecast is made against twice the target's armour level plus one: the host's physical forecast
--                     is what the campaign weighs its packages by (EEDCS-46), so the weapons it selects and fires differ from the reference's
--   mode=damage_armoured the reference host, but every hit is answered against three times the target's armour level plus one; mode=damage_double against twice the weapon's
--                     damage figure; mode=damage_none a hit does nothing (no sample is taken, the player's invulnerability is still honoured): the host's physical damage is what the
--                     campaign applies to health and acts on (EEDCS-46), so the losses, the credit and the kills differ from the reference's while the forecasts do not
--   mode=timed_never  the reference host, but no target is ever eligible for a timed roll (a vehicle is told it cannot burn, an aircraft that it is in no class): nothing burns or
--                     degrades by itself and the six timed rolls take no sample (EEDCS-46); only the timed answers differ
--   mode=timed_fast   the same host with twenty times the frame time in every timed roll: what is eligible is hit twenty times as often (the samples taken are the same)
--   mode=timed_generic the same host, but a host with a model of its own answers the timed roll of a person with health at or under half its initial level, and of an aircraft in no class
--                     at or under half, with a stochastic model that is not EECH's (the host asks for the campaign's one sample and hits when it is under a half): the reference host rolls for neither.
--                     Only the timed answers of those two kinds differ; the counters `timed_generic_*` say how many rolls were its own
--   mode=critical_late the same host with a quarter of the initial level in every classification, assessment, timed roll AND damage question of a vehicle or an aircraft: the critical
--                     classes (the class of a health, what a change crosses, the class a hit leaves) begin at a quarter of the health; the capability arithmetic, the forecasts and every
--                     other answer are the reference's
--   mode=selfhit_allowed the same host, but an AI helicopter's own hit is allowed to hurt it (the helicopter handler asks only for a hit that is its own: if no helicopter hits itself in
--                     the run the counters say so, and this mode proves nothing)
--   mode=forecast_zero the reference host, but every damage forecast is zero: no package is capable of damage, so the campaign selects and fires no weapon
--   mode=seduce_never the reference host, but a seeker asked about a decoy is told the decoys have no effect: no seeker is drawn to a decoy (the host's seduction)
--   mode=reacquire_never the reference host, but a seeker whose decoy expired is told to drop the lock: it does not go back to the decoy's launcher (the campaign unlinks it)
--   mode=bad_forecast_negative, bad_forecast_draw, bad_forecast_fraction   a provider whose answer to the first damage forecast is a negative capability, a request for
--                     a sample of the shared random stream (a forecast takes none), or a capability that is not an integer: the engine must stop with a clear error
--   mode=bad_damage_level, bad_damage_draw_twice, bad_damage_exit, bad_damage_class, bad_damage_unwritten, bad_damage_missing   a provider whose answer to the first hit is
--                     health written below zero, a second request for a sample, a stop by invulnerability of a target that is not invulnerable, a critical class the target
--                     cannot have, a health changed without being written, or an answer with a field missing: the engine must stop with a clear error
--   mode=bad_health_class, bad_health_missing, bad_health_classify_draw, bad_health_second_draw, bad_health_hit_ineligible, bad_health_fatal   a provider whose first answer of the kind is a
--                     class no vehicle can have, a classification with the class missing, a request for the campaign's sample in answer to a classification (a pure question), a second
--                     request for the sample of a timed roll, an ineligible timed answer that nevertheless reports a hit, or a hit whose fatal flag disagrees with the health it leaves: the
--                     engine must stop with a clear error.  The first three are asked in every run (a classification is asked at every aircraft and client update); `bad_health_hit_ineligible`
--                     triggers on the first completed ineligible timed answer (any vehicle update, no hit needed); `bad_health_second_draw` needs an eligible roll and `bad_health_fatal` an
--                     actual timed hit, and may not happen in a short run: the summary's counters say whether the case was reached
--   mode=bad_step_*   a provider that answers the first weapon step malformed: the engine must stop with a clear error (bad_step_state, bad_step_lookup_kind, bad_step_action_kind,
--                     bad_step_nonfinite, bad_step_repeat, bad_step_error, bad_step_draw)
--   mode=bad_birth_action, bad_release_burst   the first weapon birth answered with a step's action, the first release with no rounds and no refusal
--   mode=impact_none  the reference host, but a weapon's death does nothing to the world (EEDCS-46): no crater, no direct hit, no blast, whatever the kill code; mode=impact_own a host
--                     with a model of its own for the dispatch: a land hit blasts ONCE with no crater and never asks for the terrain, and the other deaths as the plain model; only the
--                     dispatch differs, and the counters `impact_denied` (a plan the reference would have acted on, answered with none) and `impact_own_land` say the case was reached
--   mode=blast_small  the reference host, but every blast is asked at a quarter of the weapon's damage capability (half the radius: the host's physical radius is what the campaign
--                     applies, EEDCS-46); mode=blast_none the reference host's radius, but the host reaches no object; mode=blast_own a host with a plain model of its own (a fixed thirty
--                     metres, the explosion's own sector, every placed object whatever its kind); the counters `blast_shrunk`, `blast_denied` and `blast_own` count the blasts each changed
--   mode=bad_impact_*, bad_blast_*  a provider whose first answer of the kind is malformed: bad_impact_crater (a crater on a terrain the campaign has not read), bad_impact_draw (a request for
--                     a sample), bad_impact_blasts (three blasts), bad_blast_handle (an object that is not in the window), bad_blast_radius (a radius that changes within a blast),
--                     bad_blast_draw, bad_blast_sector (a window outside the map), bad_blast_repeat (the same sector's start asked twice), bad_blast_modifier (a modifier above one) and
--                     bad_blast_rocket (a collision-free destruction of an object the campaign does not know as a rocket): the engine must stop with a clear error.  The blast modes need a
--                     blast that reaches an object (`bad_blast_handle`, `bad_blast_modifier`, `bad_blast_rocket`) or a second sector (`bad_blast_repeat`) and may not trigger in a short run
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
local counts = { table = 0, range = 0, figures = 0, weapon_step = 0, weapon_birth = 0, release = 0, animate = 0, weapon_decoy = 0, weapon_seduce = 0, weapon_reacquire = 0, ready_update = 0, ready_command = 0, required = 0, step = 0, vectors = 0, lead = 0, ballistic = 0, intercept = 0, forecast = 0, damage = 0, health = 0, impact = 0, blast = 0 }
local answers = { valid = 0, invalid = 0, need_draw = 0, lead_ok = 0, lead_no = 0, ballistic_ok = 0, ballistic_no = 0, phase_moving = 0, angle_sum = 0 }
-- what the weapon steps came to: the passes the host was asked, the lookups it needed (by kind), the draws it asked for, and the kill codes it left (0 none ... 7)
local steps = { passes = 0, done = 0, need_lookup = 0, need_draw = 0, kills = {}, lookups = {}, births = 0, birth_velocity = 0, releases = 0, release_refused = 0, release_rounds = 0,
	decoy_updates = 0, decoy_restarts = 0, seduce_asked = 0, seduce_locked = 0, seduce_retargeted = 0, reacquire_asked = 0, reacquire_returned = 0,
	forecasts = 0, forecast_sum = 0, forecast_zero = 0,
	damage_draws = 0, damage_hits = 0, damage_sum = 0, damage_exited = 0, damage_written = 0, damage_critical = 0, damage_fatal = 0,
	health_draws = 0, health_classified = 0, health_critical = 0, health_assessed = 0, health_crossed = 0, timed_asked = 0, timed_eligible = 0, timed_rolled = 0, timed_hit = 0, timed_fatal = 0,
	selfhit_asked = 0, selfhit_refused = 0, timed_generic = 0,
	impact_asked = 0, impact_terrain = 0, impact_land = 0, impact_blasts = 0, impact_crater = 0, impact_direct = 0, impact_invalid = 0, impact_denied = 0, impact_own_land = 0,
	blast_begun = 0, blast_zero = 0, blast_windows = 0, blast_collisions = 0, blast_intercepts = 0, blast_ended = 0, blast_shrunk = 0, blast_denied = 0, blast_own = 0 }
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
	elseif step.mode == "forecast" then
		steps.forecasts = steps.forecasts + 1
		steps.forecast_sum = steps.forecast_sum + answer.capability
		if answer.capability == 0 then steps.forecast_zero = steps.forecast_zero + 1 end
	elseif step.mode == "damage" then
		if answer.state == "need_draw" then
			steps.damage_draws = steps.damage_draws + 1
		else
			steps.damage_hits = steps.damage_hits + 1
			steps.damage_sum = steps.damage_sum + answer.capability
			if answer.exited then steps.damage_exited = steps.damage_exited + 1 end
			if answer.written then steps.damage_written = steps.damage_written + 1 end
			if answer.critical_after > answer.critical_before then steps.damage_critical = steps.damage_critical + 1 end
			if answer.written and answer.new_level <= 0 then steps.damage_fatal = steps.damage_fatal + 1 end
		end
	elseif step.mode == "health" then
		if answer.state == "need_draw" then
			steps.health_draws = steps.health_draws + 1
		elseif step.op == "classify" then
			steps.health_classified = steps.health_classified + 1
			if answer.critical_class > 0 then steps.health_critical = steps.health_critical + 1 end
		elseif step.op == "assess" then
			steps.health_assessed = steps.health_assessed + 1
			if answer.crossed then steps.health_crossed = steps.health_crossed + 1 end
		elseif step.op == "timed" then
			steps.timed_asked = steps.timed_asked + 1
			if answer.eligible then steps.timed_eligible = steps.timed_eligible + 1 end
			if answer.rolled then steps.timed_rolled = steps.timed_rolled + 1 end
			if answer.hit then steps.timed_hit = steps.timed_hit + 1 end
			if answer.fatal then steps.timed_fatal = steps.timed_fatal + 1 end
		else
			steps.selfhit_asked = steps.selfhit_asked + 1
			if not answer.eligible then steps.selfhit_refused = steps.selfhit_refused + 1 end
		end
	elseif step.mode == "impact" then
		if answer.state == "need_lookup" then
			steps.impact_terrain = steps.impact_terrain + 1
		else
			steps.impact_asked = steps.impact_asked + 1
			if step.kill_code == 3 then steps.impact_land = steps.impact_land + 1 end
			steps.impact_blasts = steps.impact_blasts + answer.blasts
			if answer.crater then steps.impact_crater = steps.impact_crater + 1 end
			if answer.direct_hit then steps.impact_direct = steps.impact_direct + 1 end
			if answer.invalid then steps.impact_invalid = steps.impact_invalid + 1 end
		end
	elseif step.mode == "blast" then
		if step.window == nil then
			steps.blast_begun = steps.blast_begun + 1
			if answer.radius == 0 then steps.blast_zero = steps.blast_zero + 1 end
		end
		if answer.state == "need_lookup" then steps.blast_windows = steps.blast_windows + 1
		elseif answer.action == "collision" then steps.blast_collisions = steps.blast_collisions + 1
		elseif answer.action == "intercept" then steps.blast_intercepts = steps.blast_intercepts + 1
		else steps.blast_ended = steps.blast_ended + 1 end
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
elseif mode == "forecast_armoured" then
	provider = function (step)
		if step.mode == "forecast" then step = with (step, { armour_level = step.armour_level * 2 + 1 }) end
		return counted (step, reference (step))
	end
elseif mode == "timed_never" or mode == "timed_fast" or mode == "critical_late" or mode == "selfhit_allowed" or mode == "timed_generic" then
	provider = function (step)
		if step.mode == "health" then
			if mode == "timed_generic" and step.op == "timed" and (step.critical_model == 2 or (step.critical_model == 3 and step.aircraft_class == 0)) and step.damage_level > 0 and step.damage_level * 2 <= step.initial_damage_level then
				-- a host's own physical model, not EECH's: it rolls for a hurt person and for a hurt aircraft in no class, takes the campaign's one sample and hits when it is under a half
				if step.draw == nil then return counted (step, { state = "need_draw" }) end
				steps.timed_generic = steps.timed_generic + 1
				local hit = step.draw < 16384
				local level = hit and math.max (0, step.damage_level - 1) or step.damage_level
				return counted (step, { state = "done", critical_class = reference (with (step, { op = "classify", damage_level = level })).critical_class, crossed = false, recovered = false, eject = false,
					eligible = true, rolled = true, hit = hit, new_level = level, fatal = hit and level <= 0 })
			end
			if mode == "timed_generic" then return counted (step, reference (step)) end
			if mode == "timed_never" and step.op == "timed" then step = with (step, { explosive = false, flammable = false, aircraft_class = 0 })
			elseif mode == "timed_fast" and step.op == "timed" then step = with (step, { delta_time = step.delta_time * 20 })
			elseif mode == "critical_late" and step.op ~= "hit_eligible" then step = with (step, { initial_damage_level = math.max (1, math.floor (step.initial_damage_level / 4)) })
			elseif mode == "selfhit_allowed" and step.op == "hit_eligible" then return counted (step, own.health (step)) end
		elseif mode == "critical_late" and step.mode == "damage" and step.critical_model ~= 0 and step.critical_model ~= 2 then
			step = with (step, { initial_damage_level = math.max (1, math.floor (step.initial_damage_level / 4)) })
		end
		return counted (step, reference (step))
	end
elseif mode == "damage_armoured" or mode == "damage_double" then
	provider = function (step)
		if step.mode == "damage" then
			if mode == "damage_armoured" then step = with (step, { armour_level = step.armour_level * 3 + 1 })
			else step = with (step, { damage_capability = step.damage_capability * 2 }) end
		end
		return counted (step, reference (step))
	end
elseif mode == "damage_none" then
	provider = function (step)
		if step.mode == "damage" then return counted (step, own.damage (with (step, { damage_capability = 0 }))) end
		return counted (step, reference (step))
	end
elseif mode == "impact_none" or mode == "impact_own" or mode == "blast_small" or mode == "blast_none" or mode == "blast_own" then
	provider = function (step)
		if step.mode == "impact" then
			if mode == "impact_none" then
				-- the host's physical decision: nothing a weapon's death does reaches the world.  The reference plan (asked with the terrain given, so it never asks for it) says what was denied
				local planned = reference (with (step, { terrain_draws_crater = false }))
				if planned.blasts > 0 or planned.direct_hit then steps.impact_denied = steps.impact_denied + 1 end
				return counted (step, { state = "done", invalid = planned.invalid, crater = false, direct_hit = false, blasts = 0, exclude_target = false })
			elseif mode == "impact_own" then
				if step.kill_code == 3 then steps.impact_own_land = steps.impact_own_land + 1 end
				return counted (step, own.impact (step))
			end
		elseif step.mode == "blast" then
			if mode == "blast_small" then
				if step.window == nil and step.damage_capability >= 4 then steps.blast_shrunk = steps.blast_shrunk + 1 end
				step = with (step, { damage_capability = math.floor (step.damage_capability / 4) })
			elseif mode == "blast_none" then
				local radius = reference (step).radius
				if step.window == nil and radius > 0 then steps.blast_denied = steps.blast_denied + 1 end
				return counted (step, { state = "done", radius = radius, action = "none" })
			elseif mode == "blast_own" then
				if step.window == nil then steps.blast_own = steps.blast_own + 1 end
				return counted (step, own.blast (step))
			end
		end
		return counted (step, reference (step))
	end
elseif mode == "forecast_zero" then
	provider = function (step)
		local answer = reference (step)
		if step.mode == "forecast" then answer = { state = "done", capability = 0 } end
		return counted (step, answer)
	end
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
	reacquire_asked = steps.reacquire_asked, reacquire_returned = steps.reacquire_returned,
	forecasts = steps.forecasts, forecast_sum = steps.forecast_sum, forecast_zero = steps.forecast_zero,
	damage_draws = steps.damage_draws, damage_hits = steps.damage_hits, damage_sum = steps.damage_sum, damage_exited = steps.damage_exited, damage_written = steps.damage_written,
	damage_critical = steps.damage_critical, damage_fatal = steps.damage_fatal,
	health_draws = steps.health_draws, health_classified = steps.health_classified, health_critical = steps.health_critical, health_assessed = steps.health_assessed,
	health_crossed = steps.health_crossed, timed_asked = steps.timed_asked, timed_eligible = steps.timed_eligible, timed_rolled = steps.timed_rolled, timed_hit = steps.timed_hit,
	timed_fatal = steps.timed_fatal, selfhit_asked = steps.selfhit_asked, selfhit_refused = steps.selfhit_refused, timed_generic = steps.timed_generic,
	impact_asked = steps.impact_asked, impact_terrain = steps.impact_terrain, impact_land = steps.impact_land, impact_blasts = steps.impact_blasts, impact_crater = steps.impact_crater,
	impact_direct = steps.impact_direct, impact_invalid = steps.impact_invalid, impact_denied = steps.impact_denied, impact_own_land = steps.impact_own_land,
	blast_begun = steps.blast_begun, blast_zero = steps.blast_zero, blast_windows = steps.blast_windows, blast_collisions = steps.blast_collisions, blast_intercepts = steps.blast_intercepts,
	blast_ended = steps.blast_ended, blast_shrunk = steps.blast_shrunk, blast_denied = steps.blast_denied, blast_own = steps.blast_own }
out:close ()
io.stderr:write (string.format ("aim-provider: mode=%s ready_update=%d ready_command=%d required=%d step=%d vectors=%d (valid %d invalid %d) lead=%d ballistic=%d intercept=%d need_draw=%d ready_moving=%d weapon_births=%d losses=%d failure=%s\n",
	mode, counts.ready_update, counts.ready_command, counts.required, counts.step, counts.vectors, answers.valid, answers.invalid, counts.lead, counts.ballistic, counts.intercept,
	answers.need_draw, answers.phase_moving, births, lost, failure or "none"))
if mode:sub (1, 4) == "bad_" then
	assert (failure, "a malformed provider answer was accepted: mode " .. mode)
elseif failure then
	error (failure)
end
