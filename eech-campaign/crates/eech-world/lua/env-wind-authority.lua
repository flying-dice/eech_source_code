-- TEST-ONLY host-authority and refusal witness of the host-owned environment WIND (EEDCS-51, D3). SOURCE PREPARED: UNBUILT, UNRUN, UNREVIEWED. Needs a DLL built with the
-- `weapon-fixture` feature (the controlled launch of lua/weapon-case.lua) and an eech-world with `host.terrain_provider` and `host.env_provider`.
--
-- PROVENANCE. The finite shape (one bounded fresh war per process; a real launch through the campaign's own call; the case COHORT frozen from the births of that call; one complete
-- terminal per member; a clock that must advance; a frame budget; checked output) is the closed ground witness's, read from ground-production-source-aafb21775
-- (eech-campaign/crates/eech-world/lua/ground-authority.lua and tests/ground_authority.rs) and re-written here for the environment provider; none of that component's unaccepted
-- production changes is taken. The launch selection (first launcher in roster order with an unguided ballistic weapon in stock and an enemy to aim at, the nearest enemy) is the
-- same deterministic one. This branch keeps its own reference terrain provider (host.terrain_provider) and its own environment provider (host.env_provider ()), and WRAPS only the latter.
--
-- Boundary under test (the product path, not a private test of Rust or C): host-owned Lua -> the public native module -> the host's ENVIRONMENT provider -> back into a REAL weapon
-- launched through the campaign's own launch call, whose hosted flight asks the campaign for the wind at its position (wn_move.c CAMPAIGN_STEP_LOOKUP_WIND ->
-- get_session_wind_velocity_at_point) and whose campaign code answers from the HOST: `wind` is a provider operation and the module has no wind solver of its own behind it.
--
-- Until the case weapon has been launched AND its births read (the cohort frozen) the provider is the host's own, delegated to unchanged and counted. Only then, before any frame, the
-- witness is ARMED, and from then on the chosen mode answers a wind question that asked for a VECTOR (want_vector true: the weapon's) differently. A wind question that asked for
-- no vector (the session's sound update asks the camera's wind speed every frame) is always delegated unchanged: it is counted, never altered.
--
--   control                  nothing is altered: the healthy same-DLL control. The cohort's terminal facts are recorded with the wind questions asked after the launch, and the
--                            first real vector answers.
--   changed_wind             every armed vector question is answered { speed = 64, vector = { 64, 0, 0 } }: a VALID, finite, exact-single wind, about 64 m/s along +x. Each reply
--                            sent is recorded (the first eight as rows). The cohort's real flight must change, in the direction of the sent wind.
--   withheld_wind            the provider raises on the first armed vector question. The engine must stop at the named boundary: [env-provider:wind.callback].
--   malformed_wind_speed     the answer has a WRONG-TYPE speed (a string) alongside a valid vector:                          [env-provider:wind.answer]
--   malformed_wind_vector    the answer has a valid speed and NO vector although one was asked for:                            [env-provider:wind.answer]
--
-- The provider is global to the campaign: after the arm EVERY vector question of the campaign (not only the weapon's) is answered by the mode. That is a bounded alternate-host-world
-- input, not an isolation of the weapon from the rest of the campaign (its policy, its random draws, other weapon families): no claim beyond the cohort's recorded facts is made.
-- The wrapper never changes the request it is handed, calls the host's own provider for every unaltered question, and never calls it for an altered wind question (so nothing of the
-- host's field is touched by a fabricated reply).
--
-- The CASE COHORT is the births of the launch call itself, read immediately after the call and before any frame: frozen then. A weapon the natural war creates later is a `later_birth`
-- row and never a case weapon; a terminal of a weapon outside the cohort is a `foreign_terminal` row. A member has exactly one terminal (a second is a `duplicate_terminal` row).
-- The tests/env_wind_authority.rs judge requires every field of every case terminal and the birth identities of the cohort to be the same across the compared runs.
--
-- EVIDENCE COMPLETENESS (review of 47f3d9460). The events are the fixture's own `wpn-ev seq=<N> kind=<kind> ...` lines (eech_wpn_fixture.c log_event): a line of another form, a sequence with a hole in it
-- (an event missing from the log), a `kind=truncated` event, and a weapon identity that is not a positive ordinal (w=-1 is the fixture's unregistered sentinel) are failures of the witness, never skipped.
-- The frozen cohort must equal the fixture's own launch report (`wpn-case-launch ... births=<N> creates=<M>`), and the fixture's own loss counters (`engine:wpn_trace ()`: the events dropped from its
-- finite log, the births that found no track, the terminals of unregistered weapons) must not have moved since before the launch -- checked right after the launch, BEFORE the arm, and again at the end
-- of a run the engine did not stop (a stopped engine cannot be asked; the authority row says so). The full raw birth line is kept for the comparison of equivalent launches.
--
-- A failure of the ENGINE is only ever at an armed wind question: the script exits successfully with its evidence whenever the boot and the launch were healthy. A failure of the
-- WITNESS itself is never an expected refusal: a clock that does not advance, a frame budget that runs out, a write, flush or close of the evidence that fails, an event with no
-- weapon identity or a constant that is not an exact single all raise an error that begins `env-wind-authority:` (after a best-effort `witness_failure` row), so the process fails.
--
-- host.args: root, mode (default control), warmup (simulated seconds before the launch, default 30), settle (cap on the simulated seconds after it, default 120), frame_ms
--            (default 100), out (JSON Lines path)

io.stdout:setvbuf ("no")

local huge = math.huge
local function finite (v) return type (v) == "number" and v == v and v ~= huge and v ~= -huge end

local args = host.args
local root = assert (args.root, "root=<installation root>")
local mode = args.mode or "control"
local warmup = tonumber (args.warmup or "30")
local settle = tonumber (args.settle or "120")
local frame_ms = tonumber (args.frame_ms or "100")
assert (finite (warmup) and finite (settle) and finite (frame_ms), "warmup, settle and frame_ms must be finite numbers")
assert (warmup >= 1 and warmup <= 600 and settle >= 1 and settle <= 1200 and frame_ms >= 10 and frame_ms <= 1000, "warmup, settle or frame_ms is out of range")
local out_path = assert (args.out, "out=<JSON Lines path>")
local out = assert (io.open (out_path, "w"))
local seed = 1
-- the most frames the whole run can need is far below this: the simulated clock must advance on every frame, and the budget is four times what warmup and settle ask for, plus a margin
local frame_budget = 4 * math.ceil ((warmup + settle) * 1000 / frame_ms) + 200

local escapes = { ['"'] = '\\"', ['\\'] = '\\\\', ['\n'] = '\\n', ['\r'] = '\\r', ['\t'] = '\\t' }
local function json_string (s)
	return '"' .. (string.gsub (s, '[%c"\\\128-\255]', function (c) return escapes [c] or string.format ("\\u%04x", string.byte (c)) end)) .. '"'
end
-- every write and flush of the evidence is checked: a failed one is a failure of the witness, even after the last row
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
	if not ok then error ("env-wind-authority: OUTPUT FAILURE: writing " .. out_path .. " failed: " .. tostring (err), 0) end
	ok, err = out:flush ()
	if not ok then error ("env-wind-authority: OUTPUT FAILURE: flushing " .. out_path .. " failed: " .. tostring (err), 0) end
end
-- a failure of the witness itself: recorded if the evidence can still be written, then raised with the prefix the engine-failure handlers never treat as the engine's
local function hard (message)
	pcall (emit, { event = "witness_failure", reason = message })
	error ("env-wind-authority: " .. message, 0)
end

-- ---------------------------------------------------------------------------------------------------------------- exact singles
-- is `v` exactly a single-precision number (24 significant bits, or a multiple of 2^-149 below 2^-126, within the single range)?
local function is_single (v)
	if not finite (v) then return false end
	if v == 0 then return true end
	if math.abs (v) > 3.4028234663852886e38 then return false end
	local _, e = math.frexp (v)
	local scaled = e < -125 and math.ldexp (v, 149) or math.ldexp (v, 24 - e)
	return scaled == math.floor (scaled)
end

-- ---------------------------------------------------------------------------------------------------------------- the modes
local SENT_SPEED, SENT_X, SENT_Y, SENT_Z = 64, 64, 0, 0
for _, constant in ipairs { SENT_SPEED, SENT_X, SENT_Y, SENT_Z } do
	if not is_single (constant) then hard ("SETUP FAILURE: the changed wind constant " .. tostring (constant) .. " is not an exact single") end
end
local MODES = {
	control = {},
	changed_wind = {
		answer = function () return { speed = SENT_SPEED, vector = { SENT_X, SENT_Y, SENT_Z } } end,
		speed = string.format ("%.17g", SENT_SPEED), vector = string.format ("%.17g,%.17g,%.17g", SENT_X, SENT_Y, SENT_Z),
	},
	withheld_wind = {
		answer = function () error ("the host withholds its wind answer", 0) end,
		marker = "wind.callback", speed = "", vector = "",
	},
	malformed_wind_speed = {
		answer = function () return { speed = "64", vector = { SENT_X, SENT_Y, SENT_Z } } end,
		marker = "wind.answer", speed = "string:64", vector = string.format ("%.17g,%.17g,%.17g", SENT_X, SENT_Y, SENT_Z),
	},
	malformed_wind_vector = {
		answer = function () return { speed = SENT_SPEED } end,
		marker = "wind.answer", speed = string.format ("%.17g", SENT_SPEED), vector = "missing",
	},
}
local plan = MODES [mode]
if plan == nil then error ("env-wind-authority: unknown mode '" .. tostring (mode) .. "'", 0) end

-- ---------------------------------------------------------------------------------------------------------------- the host's provider, counted, altered only when armed
local factory = assert (host.env_provider, "env-wind-authority: this host has no host.env_provider")
local armed = false
local frames = 0
local function counters () return { import = 0, export = 0, init_random = 0, init_fixed = 0, weather = 0, wind = 0, gust = 0, step = 0 } end
local before, after = counters (), counters ()
-- the wind questions: all of them, and those that asked for a vector (the weapon's kind), before and after the arm
local wind_before, wind_after = { total = 0, vector = 0 }, { total = 0, vector = 0 }
local altered = 0
local SENT_LIMIT, SAMPLE_LIMIT = 8, 4
local sent, samples = {}, {}
local real = factory ()
local function provider (request)
	local op = request.op
	local want_vector = request.want_vector == true
	local counts = armed and after or before
	if counts [op] ~= nil then counts [op] = counts [op] + 1 end
	if op == "wind" then
		local w = armed and wind_after or wind_before
		w.total = w.total + 1
		if want_vector then w.vector = w.vector + 1 end
	end
	if armed and op == "wind" and want_vector and plan.answer then
		altered = altered + 1
		if #sent < SENT_LIMIT then
			sent [#sent + 1] = { n = altered, frame = frames, kind = mode, speed = plan.speed or "", vector = plan.vector or "" }
		end
		return plan.answer ()
	end
	local answer = real (request)
	if armed and op == "wind" and want_vector and #samples < SAMPLE_LIMIT and type (answer) == "table" and type (answer.vector) == "table" then
		samples [#samples + 1] = {
			n = wind_after.vector, frame = frames, speed = string.format ("%.9g", answer.speed),
			vector = string.format ("%.9g,%.9g,%.9g", answer.vector [1], answer.vector [2], answer.vector [3]),
		}
	end
	return answer
end

-- ---------------------------------------------------------------------------------------------------------------- boot (the same callbacks as lua/weapon-case.lua)
local descriptors, order = {}, {}
local function bind_mobile (descriptor)
	local id = host.bind_mobile (descriptor)
	descriptors [id] = { kind = descriptor.movement_kind, side = descriptor.side }
	order [#order + 1] = id
	return id
end

local map = "..\\common\\maps\\map5"
local dc = require ("eech_dc")
dc.prepare_installation (root)
local engine = dc.boot {
	install_root = root, map = map, terrain = host.terrain_provider (root, map), environment = provider, campaign_directory = "camp01", campaign = "LEBANON.CHC", gunship = "apache",
	seed = seed, bind_mobile = bind_mobile, move_vehicle = host.move_vehicle, observe_loss = host.observe_loss,
	supply_reached = host.supply_reached, line_of_sight = host.line_of_sight, sense = host.sense, aim = host.aim,
}
emit { event = "boot", mode = mode, warmup = warmup, settle = settle, frame_ms = frame_ms, seed = seed, frame_budget = frame_budget }

local clock = engine:clock ()
local function first_line (text) return (string.match (tostring (text), "[^\r\n]+") or tostring (text)) end
-- one frame, with finite progress: the simulated clock must be a finite number that has moved forward, and the run may not use more frames than its budget
local function frame ()
	if frames >= frame_budget then hard ("FRAME BUDGET: " .. frames .. " frames were run and the simulated clock is at " .. tostring (clock.elapsed_seconds) .. " s") end
	local was = clock.elapsed_seconds
	engine:frame (frame_ms)
	frames = frames + 1
	clock = engine:clock ()
	local now = clock.elapsed_seconds
	if not finite (now) or not (now > was) then hard ("STALLED CLOCK: a frame left the simulated clock at " .. tostring (now) .. " s from " .. tostring (was) .. " s") end
end
local function step (seconds_more)
	local until_time = clock.elapsed_seconds + seconds_more
	while clock.elapsed_seconds < until_time do frame () end
end
-- the engine's own failure, or nil: the witness's own failure is never reported as the engine's
local function run_step (seconds)
	local ok, err = pcall (step, seconds)
	if ok then return nil end
	local text = tostring (err)
	-- the engine wraps a provider's error in its own marker text, so the witness's prefix is looked for anywhere in it
	if string.find (text, "env-wind-authority:", 1, true) then error (text, 0) end
	return first_line (text)
end
step (warmup)
emit { event = "warmed_up", elapsed = clock.elapsed_seconds, frames = frames, bound = #order, wind_before = wind_before.total, wind_vector_before = wind_before.vector }

-- ---------------------------------------------------------------------------------------------------------------- the case launch (preconditions, deterministic: the closed ground witness's selection)
local function fields (line)
	local t = {}
	for k, v in string.gmatch (line, "(%S+)=(%S+)") do t [k] = v end
	return t
end
local function numbers (list)
	local t = {}
	for v in string.gmatch (list, "[^,]+") do t [#t + 1] = tonumber (v) end
	return t
end
local function state (id, weapon)
	local ok, text = pcall (engine.wpn_case_query, engine, id, weapon or 0)
	if not ok then return nil end
	local s = fields (text)
	s.alive, s.weapon_count = tonumber (s.alive), tonumber (s.weapon_count)
	s.warhead, s.guidance, s.aiming, s.decoy = tonumber (s.warhead), tonumber (s.guidance), tonumber (s.aiming), tonumber (s.decoy)
	s.position = numbers (s.position or "")
	return s
end
-- The REAL event grammar (eech_wpn_fixture.c log_event): every line the fixture returns is `wpn-ev seq=<N> ` followed by the event's own `kind=<kind> ...`. The sequence is
-- consecutive over the stored events; an event that does not fit the finite log is counted as overflow and a line that does not fit its buffer is stored as `kind=truncated`. A line
-- of any other form, a sequence that does not follow its predecessor (an event missing in between) and a truncated event are failures of the witness's EVIDENCE, never skipped.
local last_seq
local function events ()
	local list = {}
	for line in string.gmatch (engine:wpn_events (), "[^\n]+") do
		local seq, kind = string.match (line, "^wpn%-ev seq=(%d+) kind=([%w_]+)")
		if not seq then hard ("MALFORMED EVENT: not a `wpn-ev seq=<n> kind=<kind>` line: " .. line) end
		seq = tonumber (seq)
		if seq < 1 or (last_seq and seq ~= last_seq + 1) then
			hard ("EVENT SEQUENCE: event " .. seq .. " follows " .. tostring (last_seq) .. ": an event is missing from the log or is out of order: " .. line)
		end
		last_seq = seq
		if kind == "truncated" then hard ("EVENT TRUNCATED: the fixture marked event " .. seq .. " as not fitting its log buffer: " .. line) end
		local f = fields (line)
		f.raw, f.seq, f.kind = line, seq, kind
		list [#list + 1] = f
	end
	return list
end
-- a registered weapon identity: a positive integer (the fixture's ordinals start at 1; w=-1 is its unregistered sentinel)
local function weapon_identity (e)
	if e.w == nil or not string.match (e.w, "^%d+$") or tonumber (e.w) < 1 then
		hard ("an event with no registered weapon identity (w=" .. tostring (e.w) .. ", -1 is the fixture's unregistered sentinel): " .. e.raw)
	end
	return e.w
end
-- the fixture's own report of what its finite log and its tracks have lost so far (`engine:wpn_trace ()`): the log overflow, the births that found no track, the terminals of unregistered weapons
local function trace_counts ()
	local ok, text = pcall (engine.wpn_trace, engine)
	if not ok then hard ("TRACE UNREADABLE: " .. first_line (text)) end
	local overflow = string.match (text, "wpn%-log events=%d+ stored=%d+ drained=%d+ overflow=(%d+)")
	local track_overflow = string.match (text, "wpn%-counter name=birth_track_overflow value=(%d+)")
	local unregistered = string.match (text, "wpn%-counter name=terminal_unregistered value=(%d+)")
	if not (overflow and track_overflow and unregistered) then hard ("TRACE GRAMMAR: the trace report lacks the log, birth_track_overflow or terminal_unregistered line") end
	return { overflow = tonumber (overflow), track_overflow = tonumber (track_overflow), unregistered = tonumber (unregistered) }
end

local ordered = {}
for _, id in ipairs (order) do ordered [#ordered + 1] = id end
table.sort (ordered, function (a, b) return tonumber (string.match (a, "(%d+)$")) < tonumber (string.match (b, "(%d+)$")) end)
local alive = {}
for _, id in ipairs (ordered) do
	local s = state (id, 0)
	if s and s.alive == 1 then s.side = descriptors [id].side alive [id] = s end
end
local function nearest_enemy (id)
	local s, best, best_d = alive [id], nil, nil
	for _, other in ipairs (ordered) do
		local o = alive [other]
		if o and other ~= id and o.side ~= s.side and (o.side == "blue" or o.side == "red") then
			local dx, dz = o.position [1] - s.position [1], o.position [3] - s.position [3]
			local d = math.sqrt (dx * dx + dz * dz)
			if not best_d or d < best_d then best, best_d = other, d end
		end
	end
	return best
end
-- the first launcher (in roster order) with an unguided ballistic weapon in stock and an enemy to aim at: a weapon that flies a ballistic path to the ground or its target
local launcher, target, weapon
for _, id in ipairs (ordered) do
	if alive [id] then
		local enemy = nearest_enemy (id)
		for w = 1, 127 do
			local q = enemy and state (id, w)
			if q and q.weapon_count and q.weapon_count > 0 and q.guidance == 0 and q.aiming ~= 0 and q.decoy == 0 then launcher, target, weapon = id, enemy, w break end
		end
	end
	if launcher then break end
end
emit { event = "case_selected", launcher = launcher or "none", target = target or "none", weapon = weapon or -1 }
if not launcher then hard ("SETUP FAILURE: no launcher with an unguided ballistic weapon and an enemy was found") end

events () -- everything the natural war logged before the launch is taken first, so what is read after is the launch's own
-- what the fixture has already lost (log overflow, births without a track, unregistered terminals): the launch and the run after it must lose nothing more
local trace_before = trace_counts ()
-- NOT armed: the launch, and the read of its births, are healthy and unaltered, whatever the mode
local launched, launch_text = pcall (engine.wpn_case_launch, engine, launcher, target, weapon, 0)
if not launched and string.find (tostring (launch_text), "env-wind-authority:", 1, true) then error (tostring (launch_text), 0) end
local failure = (not launched) and first_line (launch_text) or nil
local frames_at_launch = frames

-- the cohort: the births of the launch call itself, read before any frame runs, frozen here and never added to
local cohort, members, births = {}, {}, {}
if launched then
	for _, e in ipairs (events ()) do
		if e.kind == "birth" then
			weapon_identity (e)
			if cohort [e.w] then hard ("a weapon born twice in one launch: " .. e.w) end
			cohort [e.w] = true
			members [#members + 1] = e.w
			births [e.w] = e
		end
	end
	table.sort (members, function (a, b) return tonumber (a) < tonumber (b) end)
end
-- The launch REPORT is the fixture's own count, independent of the rows read: `wpn-case-launch ... births=<N> creates=<M> ...` (eech_wpn_fixture.c). The frozen cohort must be exactly that many
-- births, and the fixture must not have dropped, truncated or left untracked any event of the launch, BEFORE anything is armed.
local report_births, report_creates, launch_trace
if launched then
	local report = fields (tostring (launch_text))
	if not string.find (tostring (launch_text), "^wpn%-case%-launch ") then hard ("LAUNCH REPORT: not a `wpn-case-launch` line: " .. first_line (launch_text)) end
	report_births, report_creates = tonumber (report.births), tonumber (report.creates)
	if not report_births or not report_creates or report_births ~= math.floor (report_births) or report_creates ~= math.floor (report_creates) then hard ("LAUNCH REPORT: no whole births and creates counts in: " .. first_line (launch_text)) end
	if report_births < 1 or report_births ~= #members or report_creates ~= #members then
		hard ("COHORT INCOMPLETE: the launch report says births=" .. report_births .. " creates=" .. report_creates .. " but " .. #members .. " births were read")
	end
	launch_trace = trace_counts ()
	if launch_trace.overflow ~= trace_before.overflow then hard ("EVENT LOG OVERFLOW: the fixture dropped " .. (launch_trace.overflow - trace_before.overflow) .. " event(s) by the end of the launch: the cohort's evidence may be incomplete") end
	if launch_trace.track_overflow ~= trace_before.track_overflow then hard ("BIRTH TRACK OVERFLOW: a birth of the launch found no track: the cohort's evidence is incomplete") end
end
emit {
	event = "case_launch", ok = launched and true or false, launcher = launcher, target = target, weapon = weapon, text = first_line (launch_text), births = #members,
	frames = frames_at_launch, report_births = report_births, report_creates = report_creates, launch_trace_checked = launched and true or false,
}
emit { event = "cohort", size = #members, members = table.concat (members, ",") }
for _, w in ipairs (members) do emit { event = "birth", w = w, seq = births [w].seq, raw = births [w].raw } end
-- ARMED: only now, after a successful launch and the frozen cohort, and before any frame
if launched then
	armed = true
	emit { event = "armed", frames = frames, cohort_size = #members }
end

-- ---------------------------------------------------------------------------------------------------------------- run until the cohort has ended, the engine stops, or the cap
local terminal_of, terminals, foreign, duplicates, later_births = {}, {}, {}, {}, {}
local function take ()
	for _, e in ipairs (events ()) do
		if e.kind == "birth" then
			later_births [#later_births + 1] = weapon_identity (e)
		elseif e.kind == "terminal" then
			local w = weapon_identity (e)
			if not cohort [w] then foreign [#foreign + 1] = w
			elseif terminal_of [w] then duplicates [#duplicates + 1] = w
			else terminal_of [w] = e terminals [#terminals + 1] = e end
		end
	end
end
local function pending ()
	local n = 0
	for _, w in ipairs (members) do if not terminal_of [w] then n = n + 1 end end
	return n
end
local deadline = clock.elapsed_seconds + settle
while not failure and pending () > 0 and clock.elapsed_seconds < deadline do
	failure = run_step (1)
	if failure then break end
	take ()
end
if not failure then take () end
-- a completed run is checked once more for what the fixture lost on the way (a run the engine stopped cannot be asked: its trace is not read, and the row says so)
local final_trace
if not failure then
	final_trace = trace_counts ()
	if final_trace.overflow ~= trace_before.overflow then hard ("EVENT LOG OVERFLOW: the fixture dropped " .. (final_trace.overflow - trace_before.overflow) .. " event(s) during the run: terminal facts may be missing") end
	if final_trace.track_overflow ~= trace_before.track_overflow then hard ("BIRTH TRACK OVERFLOW: a birth found no track during the run") end
	if final_trace.unregistered ~= trace_before.unregistered then hard ("UNREGISTERED TERMINAL: " .. (final_trace.unregistered - trace_before.unregistered) .. " terminal fact(s) were for a weapon the fixture had no track for") end
end

for _, w in ipairs (members) do
	local e = terminal_of [w]
	if e then
		emit {
			event = "terminal", w = e.w, type = e.type, code = e.code, steps = e.steps, distance = e.distance, pos = e.pos, velocity = e.velocity, lifetime = e.lifetime,
			max_velocity = e.max_velocity, paths = e.paths, track = e.track,
		}
	end
end
for _, w in ipairs (foreign) do emit { event = "foreign_terminal", w = w } end
for _, w in ipairs (duplicates) do emit { event = "duplicate_terminal", w = w } end
for _, w in ipairs (later_births) do emit { event = "later_birth", w = w } end
for _, s in ipairs (sent) do emit { event = "wind_sent", n = s.n, frame = s.frame, kind = s.kind, speed = s.speed, vector = s.vector } end
for _, s in ipairs (samples) do emit { event = "wind_real", n = s.n, frame = s.frame, speed = s.speed, vector = s.vector } end
emit {
	event = "authority", mode = mode, expected_marker = plan.marker or "", completed = failure == nil, failure = failure or "",
	cohort_size = #members, case_weapons_ended = #terminals, case_weapons_pending = pending (), later_births = #later_births, foreign_terminals = #foreign,
	duplicate_terminals = #duplicates, elapsed = clock.elapsed_seconds, frames = frames, frame_budget = frame_budget,
	wind_before = wind_before.total, wind_vector_before = wind_before.vector, wind_after = wind_after.total, wind_vector_after = wind_after.vector,
	altered_wind = altered, wind_sent_rows = #sent, wind_real_rows = #samples,
	step_after = after.step, weather_after = after.weather, step_before = before.step, weather_before = before.weather,
	final_trace_checked = final_trace ~= nil, log_overflow = (final_trace or launch_trace or trace_before).overflow - trace_before.overflow,
	track_overflow = (final_trace or launch_trace or trace_before).track_overflow - trace_before.track_overflow,
	terminals_unregistered = (final_trace or trace_before).unregistered - trace_before.unregistered,
}
-- the last thing the witness does is close its evidence, and a close that fails is a failure after the success row as much as before it
local closed, close_error = out:close ()
if not closed then error ("env-wind-authority: OUTPUT FAILURE: closing " .. out_path .. " failed: " .. tostring (close_error), 0) end
io.stderr:write (string.format ("env-wind-authority: mode=%s completed=%s cohort=%d ended=%d wind_vector_after=%d altered=%d failure=%s\n", mode, tostring (failure == nil), #members,
	#terminals, wind_after.vector, altered, failure or "none"))
