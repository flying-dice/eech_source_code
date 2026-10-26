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
local counts = { table = 0, ready_update = 0, ready_command = 0, required = 0, step = 0, vectors = 0, lead = 0, ballistic = 0, intercept = 0 }
local answers = { valid = 0, invalid = 0, need_draw = 0, lead_ok = 0, lead_no = 0, ballistic_ok = 0, ballistic_no = 0, phase_moving = 0 }

local function counted (step, answer)
	counts [step.mode] = counts [step.mode] + 1
	if step.mode == "vectors" then
		if answer.state == "need_draw" then answers.need_draw = answers.need_draw + 1
		elseif answer.valid then answers.valid = answers.valid + 1 else answers.invalid = answers.invalid + 1 end
	elseif step.mode == "lead" then
		if answer.state == "need_draw" then answers.need_draw = answers.need_draw + 1
		elseif answer.ok then answers.lead_ok = answers.lead_ok + 1 else answers.lead_no = answers.lead_no + 1 end
	elseif step.mode == "ballistic" then
		if answer.ok then answers.ballistic_ok = answers.ballistic_ok + 1 else answers.ballistic_no = answers.ballistic_no + 1 end
	elseif step.mode == "ready_update" and (answer.phase == 1 or answer.phase == 3) then
		answers.phase_moving = answers.phase_moving + 1
	end
	return answer
end

local function with (step, changes)
	local copy = {}
	for k, v in pairs (step) do copy [k] = v end
	for k, v in pairs (changes) do copy [k] = v end
	return copy
end

local function normalise (v)
	local length = math.sqrt (v [1] * v [1] + v [2] * v [2] + v [3] * v [3])
	if length == 0 then return { 0, 0, -1 }, 0 end
	return { v [1] / length, v [2] / length, v [3] / length }, length
end

-- the plain model of mode=own_model: instant readiness, snapping devices, a bearing in the device frame, guided weapons intercept the target's point
local own = {}
function own.ready_update (step)
	return { state = "done", ready_state = (step.state >= 1.5 and step.state < 2.5) and 2.0 or (step.state < 0.5 and 0.0 or step.state), phase = (step.state >= 1.5 and step.state < 2.5) and 2 or 0 }
end
function own.ready_command (step) return { state = "done", ready_state = step.open and 2.0 or 0.0 } end
function own.required (step)
	-- the intercept vector in the device frame by the transpose of its attitude (rows are the device axes), then a bearing; a gun aimed at a tracking point
	-- takes the plain direction to it
	local m, v = step.device_attitude, step.intercept_vector
	if step.tracking then
		local d, p = step.tracking.device_position, step.tracking.point
		v = normalise { p [1] - d [1], p [2] - d [2], p [3] - d [3] }
	end
	local x = m [1] * v [1] + m [2] * v [2] + m [3] * v [3]
	local y = m [4] * v [1] + m [5] * v [2] + m [6] * v [3]
	local z = m [7] * v [1] + m [8] * v [2] + m [9] * v [3]
	local heading = math.atan2 (x, z)
	local pitch = math.atan2 (y, math.sqrt (x * x + z * z))
	if not step.unbounded then
		heading = math.max (step.heading_min, math.min (step.heading_max, heading))
		pitch = math.max (step.pitch_min, math.min (step.pitch_max, pitch))
	end
	return { state = "done", required_heading = heading, required_pitch = pitch }
end
function own.step (step)
	local heading = math.max (step.heading_min, math.min (step.heading_max, step.required_heading))
	return { state = "done", delta_heading = heading - step.current_heading, delta_pitch = step.required_pitch - step.current_pitch, new_heading = heading, new_pitch = step.required_pitch }
end
function own.vectors (step)
	local nothing = { 0, 0, -1 }
	if step.tracking_point or step.weapon.aiming_type ~= "intercept_point" then
		-- the model has no solution for an unguided weapon or a tracking point
		return { state = "done", valid = false, weapon_vector = nothing, target_vector = nothing, intercept_vector = nothing, intercept_range = 0 }
	end
	local d, a, p = step.device_position, step.device_attitude, step.target.point
	local to_point, range = normalise { p [1] - d [1], p [2] - d [2], p [3] - d [3] }
	local to_target = normalise { step.target.position [1] - d [1], step.target.position [2] - d [2], step.target.position [3] - d [3] }
	return { state = "done", valid = true, weapon_vector = { a [7], a [8], a [9] }, target_vector = to_target, intercept_vector = to_point, intercept_range = range }
end
function own.lead (step) return { state = "done", ok = false, intercept_point = step.target.point, angle = 0 } end
function own.ballistic () return { state = "done", ok = false } end
function own.intercept (step) return { state = "done", intercept_point = step.target.point } end
function own.table () return { state = "done" } end

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
elseif mode == "missing" then
	provider = nil
elseif mode:sub (1, 4) == "bad_" then
	-- the reference host answers until the first request of the kind the mode corrupts; that answer is malformed and the engine must stop there
	local kinds = { bad_nil = "ready_update", bad_state = "ready_update", bad_ready_state = "ready_update", bad_phase = "ready_update", bad_nan_vector = "vectors",
		bad_draw_for_ballistic = "ballistic", bad_second_draw = "vectors", bad_negative_range = "vectors", bad_error = "ready_command" }
	local kind = assert (kinds [mode], "unknown bad mode " .. mode)
	local armed = true
	provider = function (step)
		local answer = reference (step)
		if armed and step.mode == kind and (mode ~= "bad_second_draw" or step.draw ~= nil) and (mode ~= "bad_negative_range" or answer.state == "done") then
			armed = false
			if mode == "bad_nil" then return nil
			elseif mode == "bad_state" then return { state = "later" }
			elseif mode == "bad_ready_state" then return { state = "done", ready_state = 5.0, phase = 0 }
			elseif mode == "bad_phase" then return { state = "done", ready_state = 1.0, phase = 7 }
			elseif mode == "bad_nan_vector" then answer.weapon_vector = { 0 / 0, 0, 0 } return answer
			elseif mode == "bad_draw_for_ballistic" then return { state = "need_draw" }
			elseif mode == "bad_second_draw" then return { state = "need_draw" }
			elseif mode == "bad_negative_range" then answer.intercept_range = -1 return answer
			elseif mode == "bad_error" then error ("provider failed on purpose")
			end
		end
		return counted (step, answer)
	end
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
	assert (mode == "missing", "boot failed")
	return
end
assert (mode ~= "missing", "boot succeeded without an aim provider")

local fixture = engine.aim_trace ~= nil
if fixture then engine:aim_trace_reset () end
local frames = math.floor (seconds * 1000 / frame_ms)
local seen, births, lost = {}, 0, 0
local last_alive = {}
local failure
for f = 1, frames do
	local frame_ok, err = pcall (engine.frame, engine, frame_ms)
	if not frame_ok then failure = (tostring (err):gsub ("\n.*", "")) break end
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
end
if fixture and args.trace and not failure then
	local trace = assert (io.open (args.trace, "wb"))
	local clock = engine:clock ()
	trace:write (string.format ("{\"event\":\"boot\",\"mode\":%q,\"scenario\":%q,\"seconds\":%s}\n", mode, args.scenario or "lebanon_retail", tostring (seconds)))
	trace:write (string.format ("{\"elapsed\":%s,\"event\":\"run\",\"frames\":%d}\n", tostring (clock.elapsed_seconds), frames))
	trace:write (table.concat (report_lines ("run", clock.elapsed_seconds, engine:aim_trace ()), "\n") .. "\n")
	trace:close ()
end
emit { event = "summary", mode = mode, frames = frames, failure = failure or "", table = counts.table, ready_update = counts.ready_update, ready_command = counts.ready_command,
	required = counts.required, step = counts.step, vectors = counts.vectors, lead = counts.lead, ballistic = counts.ballistic, intercept = counts.intercept,
	vectors_valid = answers.valid, vectors_invalid = answers.invalid, need_draw = answers.need_draw, lead_ok = answers.lead_ok, lead_no = answers.lead_no,
	ballistic_ok = answers.ballistic_ok, ballistic_no = answers.ballistic_no, ready_moving = answers.phase_moving, weapon_births = births, losses = lost }
out:close ()
io.stderr:write (string.format ("aim-provider: mode=%s ready_update=%d ready_command=%d required=%d step=%d vectors=%d (valid %d invalid %d) lead=%d ballistic=%d intercept=%d need_draw=%d ready_moving=%d weapon_births=%d losses=%d failure=%s\n",
	mode, counts.ready_update, counts.ready_command, counts.required, counts.step, counts.vectors, answers.valid, answers.invalid, counts.lead, counts.ballistic, counts.intercept,
	answers.need_draw, answers.phase_moving, births, lost, failure or "none"))
if mode:sub (1, 4) == "bad_" then
	assert (failure, "a malformed provider answer was accepted: mode " .. mode)
elseif failure then
	error (failure)
end
