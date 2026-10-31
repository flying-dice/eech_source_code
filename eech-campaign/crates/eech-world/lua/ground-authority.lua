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
--   control                    nothing is altered: the healthy same-DLL control. The case weapon's terminal fact is recorded, with the ground questions asked after the launch.
--   below_true                 every armed ground_below is answered { below = true, queried = false }: a VALID changed predicate (the point is reported below the ground without
--                              a terrain read). The weapon's real result must change.
--   below_true_impact_shift    the same, and every armed ground_impact reply has its impact point raised by 37.5 m: a VALID changed impact. The recorded replies are the points
--                              the module was handed; the weapon's terminal position is judged against them.
--   withheld_below             the provider raises on the first armed ground_below. The engine must stop at the named boundary: [terrain-provider:ground_below.callback].
--   bad_below_predicate        ground_below answers { below = 1 } (a number, not a boolean):                       [terrain-provider:ground_below.answer]
--   bad_below_record_present   ground_below answers queried = false WITH a record (an unread terrain has no record): [terrain-provider:ground_below.record]
--   withheld_impact            ground_below as below_true, and the provider raises on the first armed ground_impact: [terrain-provider:ground_impact.callback]
--   bad_impact_type            as below_true, and ground_impact answers a type that is not the record's:            [terrain-provider:ground_impact.type]
--   bad_impact_record          as below_true, and ground_impact answers a record that is not a table:             [terrain-provider:ground_impact.record]
--
-- Nothing is altered before the launch, so a failure is always at an armed ground question (never in setup); the run reports, rather than hides, what happened: the script exits
-- successfully with its evidence in the JSON Lines file whenever the boot and the launch were healthy, and tests/ground_authority.rs judges the modes against the control. A
-- run whose control saw no armed ground question, or whose case weapon never reached a terminal fact, is a setup finding of the run, not a pass.
--
-- host.args: root, mode (default control), warmup (simulated seconds before the launch, default 30), settle (cap on the simulated seconds after it, default 120), frame_ms
--            (default 100), out (JSON Lines path)

io.stdout:setvbuf ("no")

local args = host.args
local root = assert (args.root, "root=<installation root>")
local mode = args.mode or "control"
local warmup = tonumber (args.warmup or "30")
local settle = tonumber (args.settle or "120")
local frame_ms = tonumber (args.frame_ms or "100")
assert (warmup and warmup >= 1 and warmup <= 600 and settle and settle >= 1 and settle <= 1200 and frame_ms and frame_ms >= 10 and frame_ms <= 1000, "warmup, settle or frame_ms is out of range")
local out = assert (io.open (assert (args.out, "out=<JSON Lines path>"), "w"))
local seed = 1

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
	out:write ("{" .. table.concat (parts, ",") .. "}\n")
	out:flush ()
end

-- ---------------------------------------------------------------------------------------------------------------- the modes
local function below_true () return { below = true, queried = false } end
local function impact_shifted (request, real)
	local answer = real (request)
	answer.impact.y = answer.impact.y + 37.5
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
local impact_replies = {} -- the impact points the module was handed after launch, as %.9g text (the form of the fixture's terminal position)
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
			impact_replies [#impact_replies + 1] = string.format ("%.9g,%.9g,%.9g", i.x, i.y, i.z)
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
emit { event = "boot", mode = mode, warmup = warmup, settle = settle, frame_ms = frame_ms, seed = seed }

local clock = engine:clock ()
local frames = 0
local function first_line (text) return (string.match (tostring (text), "[^\r\n]+") or tostring (text)) end
local function step (seconds_more)
	local until_time = clock.elapsed_seconds + seconds_more
	while clock.elapsed_seconds < until_time do
		engine:frame (frame_ms)
		frames = frames + 1
		clock = engine:clock ()
	end
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
	s.position = numbers (s.position)
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
if not launcher then
	emit { event = "setup", ok = false, reason = "no launcher with an unguided ballistic weapon and an enemy was found" }
	out:close ()
	error ("ground-authority: SETUP FAILURE: no launcher with an unguided ballistic weapon and an enemy was found", 0)
end

events () -- everything the natural war logged before the launch is taken first, so what is read after is the launch's own
armed = true
local launched, launch_text = pcall (engine.wpn_case_launch, engine, launcher, target, weapon, 0)
local failure = (not launched) and first_line (launch_text) or nil
local case_ordinals, terminals, all_events = {}, {}, {}
local function take ()
	for _, e in ipairs (events ()) do
		all_events [#all_events + 1] = e
		if e.kind == "birth" and not failure then case_ordinals [e.w] = case_ordinals [e.w] or false end
		if e.kind == "terminal" and case_ordinals [e.w] ~= nil then terminals [#terminals + 1] = e case_ordinals [e.w] = true end
	end
end
-- the births of this launch are exactly the events right after it: take them before any frame runs
if launched then
	for _, e in ipairs (events ()) do
		all_events [#all_events + 1] = e
		if e.kind == "birth" then case_ordinals [e.w] = false end
	end
end
emit { event = "case_launch", ok = launched and true or false, text = first_line (launch_text), births = (function () local n = 0 for _ in pairs (case_ordinals) do n = n + 1 end return n end)() }

-- ---------------------------------------------------------------------------------------------------------------- run until the case weapons end, an error stops the engine, or the cap
local function pending ()
	local n = 0
	for _, done in pairs (case_ordinals) do if not done then n = n + 1 end end
	return n
end
local deadline = clock.elapsed_seconds + settle
while not failure and pending () > 0 and clock.elapsed_seconds < deadline do
	local ok, err = pcall (step, 1)
	if not ok then failure = first_line (err) break end
	take ()
end
if not failure then take () end

for _, e in ipairs (terminals) do emit { event = "terminal", w = e.w, type = e.type, code = e.code, steps = e.steps, pos = e.pos, velocity = e.velocity, lifetime = e.lifetime } end
for i, point in ipairs (impact_replies) do emit { event = "impact_reply", n = i, point = point } end
local ground_after = after.ground_above + after.ground_below + after.ground_impact
emit {
	event = "authority", mode = mode, expected_marker = plan.marker or "", completed = failure == nil, failure = failure or "",
	case_weapons_ended = #terminals, case_weapons_pending = pending (), elapsed = clock.elapsed_seconds, frames = frames,
	ground_above_after = after.ground_above, ground_below_after = after.ground_below, ground_impact_after = after.ground_impact, ground_after = ground_after,
	ground_above_before = before.ground_above, ground_below_before = before.ground_below, ground_impact_before = before.ground_impact,
	point_after = after.point, altered_below = altered.ground_below, altered_impact = altered.ground_impact, impact_replies = #impact_replies,
}
out:close ()
io.stderr:write (string.format ("ground-authority: mode=%s completed=%s ended=%d ground_after=%d altered=%d/%d failure=%s\n", mode, tostring (failure == nil), #terminals, ground_after,
	altered.ground_below, altered.ground_impact, failure or "none"))
