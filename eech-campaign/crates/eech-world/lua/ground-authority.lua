-- TEST-ONLY host-authority and refusal witness of the production ground-contact handoff (EEDCS-48, D3 G2). Needs a DLL built with the `weapon-fixture` feature (the controlled
-- launch of lua/weapon-case.lua) and an eech-world with `host.terrain_provider`.
--
-- Boundary under test (the product path, not a private test of Rust or C): host-owned Lua -> the public native module -> the host's terrain provider -> back into a REAL weapon
-- launched through the campaign's own launch call, whose hosted flight asks the campaign for the ground probe and the ground impact (wn_move.c) and whose campaign code answers
-- from the HOST: point_below_ground and get_ground_impact_point are provider operations (`ground_below`, `ground_impact`) and the module has no solver of its own behind them.
--
-- One bounded fresh war per process (the engine is one per process). The provider is the host's own (host.terrain_provider), delegated to unchanged and counted, until the
-- case weapon is launched ("armed"); from then on the chosen mode may answer a ground question differently:
--
--   control                    nothing is altered: the healthy same-DLL control. The case cohort's terminal facts are recorded, with the ground questions asked after the launch.
--   below_true                 every armed ground_below is answered { below = true, queried = false }: a VALID changed predicate (the point is reported below the ground without
--                              a terrain read). The cohort's real result must change.
--   below_true_impact_shift    the same, and every armed ground_impact reply has its impact point raised by about 37.5 m, to a multiple of 1/16 m: a VALID changed impact whose
--                              every coordinate is an exact single, so the double the provider hands over IS the single the module consumes (a plain +37.5 is not always a single:
--                              255.99998474121094 + 37.5 lies between two). Each recorded reply says whether it is exact; the judge refuses an inexact one.
--   withheld_below             the provider raises on the first armed ground_below. The engine must stop at the named boundary: [terrain-provider:ground_below.callback].
--   bad_below_predicate        ground_below answers { below = 1 } (a number, not a boolean):                       [terrain-provider:ground_below.answer]
--   bad_below_record_present   ground_below answers queried = false WITH a record (an unread terrain has no record): [terrain-provider:ground_below.record]
--   withheld_impact            ground_below as below_true, and the provider raises on the first armed ground_impact: [terrain-provider:ground_impact.callback]
--   bad_impact_type            as below_true, and ground_impact answers a type that is not the record's:            [terrain-provider:ground_impact.type]
--   bad_impact_record          as below_true, and ground_impact answers a record that is not a table:             [terrain-provider:ground_impact.record]
--
-- The CASE COHORT is the set of weapons the launch call itself created: the births in the events read immediately after the call, before any frame runs. It is frozen then; a weapon
-- the natural war creates later is a `later_birth` row and never a case weapon, and a terminal fact of any weapon outside the cohort is a `foreign_terminal` row, never a case
-- terminal. A cohort member has exactly one terminal (a second is a `duplicate_terminal` row); the judge (tests/ground_authority.rs) requires every field of every case terminal.
--
-- Nothing is altered before the launch, so a failure of the ENGINE is always at an armed ground question (never in setup): the script exits successfully with its evidence in the JSON
-- Lines file whenever the boot and the launch were healthy, and tests/ground_authority.rs judges the modes against the control. A failure of the WITNESS itself is different and is
-- never an expected refusal: a clock that does not advance, a frame budget that runs out, a write, flush or close of the evidence that fails, an event with no weapon identity or a
-- shift that cannot be an exact single all raise an error that begins `ground-authority:` (after a best-effort `witness_failure` row), so the process fails.
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
	if not ok then error ("ground-authority: OUTPUT FAILURE: writing " .. out_path .. " failed: " .. tostring (err), 0) end
	ok, err = out:flush ()
	if not ok then error ("ground-authority: OUTPUT FAILURE: flushing " .. out_path .. " failed: " .. tostring (err), 0) end
end
-- a failure of the witness itself: recorded if the evidence can still be written, then raised with the prefix the engine-failure handlers never treat as the engine's
local function hard (message)
	pcall (emit, { event = "witness_failure", reason = message })
	error ("ground-authority: " .. message, 0)
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
local function below_true () return { below = true, queried = false } end
local function impact_shifted (request, real)
	local answer = real (request)
	local y = answer.impact.y
	-- about 37.5 m up, on a 1/16 m grid: a multiple of 2^-4 below 2^20 needs 24 bits at most, so the moved point is an exact single and the provider's double is the consumed single
	local moved = math.floor ((y + 37.5) * 16) / 16
	if not is_single (y) or not is_single (moved) or moved == y then hard ("SETUP FAILURE: the impact point " .. string.format ("%.17g", y) .. " cannot be moved to a different exact single") end
	answer.impact.y = moved
	return answer
end
local MODES = {
	control = { handlers = {} },
	below_true = { handlers = { ground_below = below_true } },
	below_true_impact_shift = { handlers = { ground_below = below_true, ground_impact = impact_shifted } },
	withheld_below = { handlers = { ground_below = function () error ("the host withholds its ground answer", 0) end }, marker = "ground_below.callback" },
	bad_below_predicate = { handlers = { ground_below = function () return { below = 1, queried = false } end }, marker = "ground_below.answer" },
	bad_below_record_present = {
		handlers = { ground_below = function () return { below = true, queried = false, record = { last_elevation = 1, normal = { 0, 1, 0 }, type = 9, stored = true } } end },
		marker = "ground_below.record",
	},
	withheld_impact = { handlers = { ground_below = below_true, ground_impact = function () error ("the host withholds its impact answer", 0) end }, marker = "ground_impact.callback" },
	bad_impact_type = {
		handlers = { ground_below = below_true, ground_impact = function (request, real) local answer = real (request) answer.type = (answer.type + 1) % 62 return answer end },
		marker = "ground_impact.type",
	},
	bad_impact_record = {
		handlers = { ground_below = below_true, ground_impact = function (request, real) local answer = real (request) answer.record = 5 return answer end },
		marker = "ground_impact.record",
	},
}
local plan = MODES [mode]
if plan == nil then error ("ground-authority: unknown mode '" .. tostring (mode) .. "'", 0) end

-- ---------------------------------------------------------------------------------------------------------------- the host's provider, counted, altered only when armed
local factory = assert (host.terrain_provider, "ground-authority: this host has no host.terrain_provider")
local armed = false
local before, after = { point = 0, fast = 0, map = 0, ground_above = 0, ground_below = 0, ground_impact = 0 }, { point = 0, fast = 0, map = 0, ground_above = 0, ground_below = 0, ground_impact = 0 }
local altered = { ground_below = 0, ground_impact = 0 }
-- the impact points the module was handed after launch: the text of the point (the form of the fixture's terminal position), whether every coordinate is an exact single, and whether
-- this reply was altered by the mode
local impact_replies = {}
local map = "..\\common\\maps\\map5"
local real = factory (root, map)
local function provider (request)
	local op = request.op
	local counts = armed and after or before
	if counts [op] ~= nil then counts [op] = counts [op] + 1 end
	local handler = armed and plan.handlers [op]
	local answer
	if handler then
		altered [op] = altered [op] + 1
		answer = handler (request, real)
	else
		answer = real (request)
	end
	if armed and op == "ground_impact" and type (answer) == "table" and type (answer.impact) == "table" then
		local i = answer.impact
		if type (i.x) == "number" and type (i.y) == "number" and type (i.z) == "number" then
			impact_replies [#impact_replies + 1] = {
				point = string.format ("%.9g,%.9g,%.9g", i.x, i.y, i.z), exact = is_single (i.x) and is_single (i.y) and is_single (i.z), altered = handler ~= nil,
			}
		end
	end
	return answer
end

-- ---------------------------------------------------------------------------------------------------------------- boot (the same six callbacks as lua/weapon-case.lua)
local descriptors, order = {}, {}
local function bind_mobile (descriptor)
	local id = host.bind_mobile (descriptor)
	descriptors [id] = { kind = descriptor.movement_kind, side = descriptor.side }
	order [#order + 1] = id
	return id
end

local dc = require ("eech_dc")
dc.prepare_installation (root)
local engine = dc.boot {
	install_root = root, map = map, terrain = provider, campaign_directory = "camp01", campaign = "LEBANON.CHC", gunship = "apache",
	seed = seed, bind_mobile = bind_mobile, move_vehicle = host.move_vehicle, observe_loss = host.observe_loss,
	supply_reached = host.supply_reached, line_of_sight = host.line_of_sight, sense = host.sense, aim = host.aim,
}
emit { event = "boot", mode = mode, warmup = warmup, settle = settle, frame_ms = frame_ms, seed = seed, frame_budget = frame_budget }

local clock = engine:clock ()
local frames = 0
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
	if string.find (text, "ground-authority:", 1, true) then error (text, 0) end
	return first_line (text)
end
step (warmup)
emit { event = "warmed_up", elapsed = clock.elapsed_seconds, frames = frames, bound = #order, ground_before = before.ground_above + before.ground_below + before.ground_impact }

-- ---------------------------------------------------------------------------------------------------------------- the case launch (preconditions, deterministic)
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
local function events ()
	local list = {}
	for line in string.gmatch (engine:wpn_events (), "[^\n]+") do
		local f = fields (line)
		f.raw = line
		list [#list + 1] = f
	end
	return list
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
armed = true
local launched, launch_text = pcall (engine.wpn_case_launch, engine, launcher, target, weapon, 0)
if not launched and string.find (tostring (launch_text), "ground-authority:", 1, true) then error (tostring (launch_text), 0) end
local failure = (not launched) and first_line (launch_text) or nil

-- the cohort: the births of the launch call itself, read before any frame runs, frozen here and never added to
local cohort, members = {}, {}
if launched then
	for _, e in ipairs (events ()) do
		if e.kind == "birth" then
			if e.w == nil or tonumber (e.w) == nil then hard ("an event with no weapon identity: " .. e.raw) end
			if cohort [e.w] then hard ("a weapon born twice in one launch: " .. e.w) end
			cohort [e.w] = true
			members [#members + 1] = e.w
		end
	end
	table.sort (members, function (a, b) return tonumber (a) < tonumber (b) end)
end
emit {
	event = "case_launch", ok = launched and true or false, launcher = launcher, target = target, weapon = weapon, text = first_line (launch_text), births = #members,
}
emit { event = "cohort", size = #members, members = table.concat (members, ",") }

-- ---------------------------------------------------------------------------------------------------------------- run until the cohort has ended, the engine stops, or the cap
local terminal_of, terminals, foreign, duplicates, later_births = {}, {}, {}, {}, {}
local function take ()
	for _, e in ipairs (events ()) do
		if e.kind == "birth" then
			later_births [#later_births + 1] = e.w or "?"
		elseif e.kind == "terminal" then
			local w = e.w
			if w == nil then hard ("a terminal fact with no weapon identity: " .. e.raw) end
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

for _, w in ipairs (members) do
	local e = terminal_of [w]
	if e then emit { event = "terminal", w = e.w, type = e.type, code = e.code, steps = e.steps, pos = e.pos, velocity = e.velocity, lifetime = e.lifetime } end
end
for _, w in ipairs (foreign) do emit { event = "foreign_terminal", w = w } end
for _, w in ipairs (duplicates) do emit { event = "duplicate_terminal", w = w } end
for _, w in ipairs (later_births) do emit { event = "later_birth", w = w } end
for i, reply in ipairs (impact_replies) do emit { event = "impact_reply", n = i, point = reply.point, exact = reply.exact, altered = reply.altered } end
local ground_after = after.ground_above + after.ground_below + after.ground_impact
emit {
	event = "authority", mode = mode, expected_marker = plan.marker or "", completed = failure == nil, failure = failure or "",
	cohort_size = #members, case_weapons_ended = #terminals, case_weapons_pending = pending (), later_births = #later_births, foreign_terminals = #foreign,
	duplicate_terminals = #duplicates, elapsed = clock.elapsed_seconds, frames = frames, frame_budget = frame_budget,
	ground_above_after = after.ground_above, ground_below_after = after.ground_below, ground_impact_after = after.ground_impact, ground_after = ground_after,
	ground_above_before = before.ground_above, ground_below_before = before.ground_below, ground_impact_before = before.ground_impact,
	point_after = after.point, altered_below = altered.ground_below, altered_impact = altered.ground_impact, impact_replies = #impact_replies,
}
-- the last thing the witness does is close its evidence, and a close that fails is a failure after the success row as much as before it
local closed, close_error = out:close ()
if not closed then error ("ground-authority: OUTPUT FAILURE: closing " .. out_path .. " failed: " .. tostring (close_error), 0) end
io.stderr:write (string.format ("ground-authority: mode=%s completed=%s cohort=%d ended=%d ground_after=%d altered=%d/%d failure=%s\n", mode, tostring (failure == nil), #members, #terminals,
	ground_after, altered.ground_below, altered.ground_impact, failure or "none"))
