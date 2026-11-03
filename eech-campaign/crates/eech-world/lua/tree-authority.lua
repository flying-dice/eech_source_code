-- TEST-ONLY host-authority and refusal witness of the production TREE-collision handoff (EEDCS-49, D3). Needs a DLL built with the `weapon-fixture` feature (the controlled launch of
-- lua/weapon-case.lua) and an eech-world with `host.terrain_provider`. It is NOT built with `collision-fixture`, which keeps the ORIGINAL tree solver on purpose (the original
-- characterisation profile): in that build no tree question reaches the provider and this witness fails its first assertion.
--
-- Boundary under test (the product path, not a private test of Rust or C): host-owned Lua -> the public native module -> the host's terrain provider -> back into a REAL weapon launched
-- through the campaign's own launch call, whose hosted flight asks the campaign for the tree lookup after every entity test (wn_move.c CAMPAIGN_STEP_LOOKUP_TREES) and whose
-- get_line_of_sight_collision_tree answers from the HOST: tree_begin and tree_terrain are provider operations and the module has no tree data, scan or intersection behind them.
--
-- One bounded fresh war per process. The provider is the host's own (host.terrain_provider), delegated to unchanged and counted, until the case weapon is launched ("armed"); from
-- then on the chosen mode may answer a tree question differently:
--
--   control           nothing is altered: the healthy same-DLL control. Self-asserted: the host was asked tree_begin at least once (the route is the host's), every NEED_TERRAIN was
--                     answered by exactly ONE terrain point question and then tree_terrain carrying that point's elevation pattern (the campaign answers from the same host terrain,
--                     in the host's order), the old end was asked before the new end, and the second read carried the hint the first answer returned (the shared record).
--   forced_hit        every armed tree_begin is answered DONE { hit = true } at the segment's OLD position with an up normal, without a terrain question: a VALID changed answer. The
--                     cohort's terminal facts must change against the control (compare the two outputs: the case weapon ends at a tree contact where the control does not).
--   withheld_tree     the provider raises on the first armed tree_begin. The engine must stop at the named boundary: [terrain-provider:tree_begin.callback]
--   bad_tree_state    tree_begin answers state = 'waiting':                                                         [terrain-provider:tree_begin.answer]
--   bad_tree_word     tree_begin answers NEED_TERRAIN with an x that is a Lua number, not a pattern:                [terrain-provider:tree_begin.answer]
--   wrong_end_first   tree_begin answers NEED_TERRAIN for the NEW end first:                                        [terrain-provider:tree.lookups]
--   third_lookup      tree_begin answers NEED_TERRAIN old, and every tree_terrain asks again for the new end:        [terrain-provider:tree.lookups]
--   done_not_boolean  tree_begin answers DONE with hit = 1:                                                          [terrain-provider:tree_begin.answer]
--
-- A mode that alters nothing before the launch can only fail in the ENGINE at an armed tree question (never in setup): the script exits successfully with its evidence in the JSON Lines
-- file when the boot and the launch were healthy. A failure of the WITNESS itself (a clock that does not advance, a frame budget that runs out, a write of the evidence that fails, an
-- unmet self-assertion) raises an error that begins `tree-authority:`, so the process fails; a refusal mode additionally fails unless the engine stopped with the mode's marker.
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
	if not ok then error ("tree-authority: OUTPUT FAILURE: writing " .. out_path .. " failed: " .. tostring (err), 0) end
	ok, err = out:flush ()
	if not ok then error ("tree-authority: OUTPUT FAILURE: flushing " .. out_path .. " failed: " .. tostring (err), 0) end
end
-- a failure of the witness itself: recorded if the evidence can still be written, then raised with the prefix the engine-failure handlers never treat as the engine's
local function hard (message)
	pcall (emit, { event = "witness_failure", reason = message })
	error ("tree-authority: " .. message, 0)
end

-- ---------------------------------------------------------------------------------------------------------------- float patterns
-- the 32-bit pattern of an exact single (the double a Lua provider hands the engine for an elevation is that single); nil for anything else
local function bits_of (v)
	if not finite (v) then return nil end
	if v == 0 then return (1 / v < 0) and 0x80000000 or 0 end
	local sign = 0
	if v < 0 then sign, v = 0x80000000, -v end
	local m, e = math.frexp (v)
	local exponent = e + 126
	local pattern
	if exponent >= 1 then
		pattern = exponent * 8388608 + (m * 2 - 1) * 8388608
	else
		pattern = v * 2 ^ 149
	end
	if pattern ~= math.floor (pattern) or pattern >= 2147483648 then return nil end
	return sign + pattern
end
local function word (pattern) return string.format ("0x%08x", pattern) end

-- ---------------------------------------------------------------------------------------------------------------- the modes
local ZERO = "0x00000000"
local MODES = {
	control = { handlers = {} },
	forced_hit = {
		handlers = { tree_begin = function (request) return { state = "done", hit = true, point = request.old, normal = ZERO .. ",0x3f800000," .. ZERO } end },
	},
	withheld_tree = { handlers = { tree_begin = function () error ("the host withholds its tree answer", 0) end }, marker = "tree_begin.callback" },
	bad_tree_state = { handlers = { tree_begin = function () return { state = "waiting" } end }, marker = "tree_begin.answer" },
	bad_tree_word = { handlers = { tree_begin = function () return { state = "need_terrain", ["end"] = "old", x = 1.5, z = ZERO } end }, marker = "tree_begin.answer" },
	wrong_end_first = { handlers = { tree_begin = function () return { state = "need_terrain", ["end"] = "new", x = ZERO, z = ZERO } end }, marker = "tree.lookups" },
	third_lookup = {
		handlers = {
			tree_begin = function () return { state = "need_terrain", ["end"] = "old", x = ZERO, z = ZERO } end,
			tree_terrain = function () return { state = "need_terrain", ["end"] = "new", x = ZERO, z = ZERO } end,
		},
		marker = "tree.lookups",
	},
	done_not_boolean = {
		handlers = { tree_begin = function () return { state = "done", hit = 1, point = ZERO .. "," .. ZERO .. "," .. ZERO, normal = ZERO .. "," .. ZERO .. "," .. ZERO } end },
		marker = "tree_begin.answer",
	},
}
local plan = MODES [mode]
if plan == nil then error ("tree-authority: unknown mode '" .. tostring (mode) .. "'", 0) end

-- ---------------------------------------------------------------------------------------------------------------- the host's provider, counted, altered only when armed
local factory = assert (host.terrain_provider, "tree-authority: this host has no host.terrain_provider")
local armed = false
local before, after = { point = 0, tree_begin = 0, tree_terrain = 0 }, { point = 0, tree_begin = 0, tree_terrain = 0 }
local altered = { tree_begin = 0, tree_terrain = 0 }
local map = "..\\common\\maps\\map5"
local real = factory (root, map)
-- what the provider SEES after the launch: a tree query is a begin, then per NEED_TERRAIN one point read and one tree_terrain carrying that read's elevation pattern
local pending_end, pending_point_bits, pending_points, last_point_hint = nil, nil, 0, nil
local queries, need_terrain_replies, done_replies, order_faults, pairing_faults, hint_faults = 0, 0, 0, 0, 0, 0
local function note_reply (answer)
	if type (answer) ~= "table" then return end
	if answer.state == "need_terrain" then
		need_terrain_replies = need_terrain_replies + 1
		if answer ["end"] ~= ((pending_end == nil) and "old" or "new") then order_faults = order_faults + 1 end
		pending_end, pending_points, pending_point_bits = answer ["end"], 0, nil
	elseif answer.state == "done" then
		done_replies = done_replies + 1
		pending_end = nil
	end
end
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
	if armed then
		if op == "tree_begin" then
			queries = queries + 1
			pending_end = nil
			note_reply (answer)
		elseif op == "tree_terrain" then
			if pending_points ~= 1 or pending_point_bits == nil or request.elevation ~= word (pending_point_bits) then pairing_faults = pairing_faults + 1 end
			note_reply (answer)
		elseif op == "point" and pending_end ~= nil and type (answer) == "table" then
			pending_points = pending_points + 1
			pending_point_bits = bits_of (answer.elevation)
			-- the second lookup reads the record the first left: its request carries the hint the first answer returned
			if pending_end == "new" and request.hint ~= last_point_hint then hint_faults = hint_faults + 1 end
			last_point_hint = answer.hint
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
host.use_reference_arithmetic () -- the reference host's arithmetic profile, selected right after the installation is prepared and before the boot (src/reference_arithmetic.rs)
local engine = dc.boot {
	install_root = root, map = map, terrain = provider, environment = host.env_provider (), campaign_directory = "camp01", campaign = "LEBANON.CHC", gunship = "apache",
	seed = seed, bind_mobile = bind_mobile, move_vehicle = host.move_vehicle, observe_loss = host.observe_loss,
	supply_reached = host.supply_reached, line_of_sight = host.line_of_sight, model = host.model, sense = host.sense, aim = host.aim,
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
	if string.find (text, "tree-authority:", 1, true) then error (text, 0) end
	return first_line (text)
end
step (warmup)
emit { event = "warmed_up", elapsed = clock.elapsed_seconds, frames = frames, bound = #order, tree_before = before.tree_begin }

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
if not launched and string.find (tostring (launch_text), "tree-authority:", 1, true) then error (tostring (launch_text), 0) end
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
emit {
	event = "authority", mode = mode, expected_marker = plan.marker or "", completed = failure == nil, failure = failure or "",
	cohort_size = #members, case_weapons_ended = #terminals, case_weapons_pending = pending (), later_births = #later_births, foreign_terminals = #foreign,
	duplicate_terminals = #duplicates, elapsed = clock.elapsed_seconds, frames = frames, frame_budget = frame_budget,
	tree_begin_before = before.tree_begin, tree_begin_after = after.tree_begin, tree_terrain_before = before.tree_terrain, tree_terrain_after = after.tree_terrain,
	point_after = after.point, queries = queries, need_terrain_replies = need_terrain_replies, done_replies = done_replies, order_faults = order_faults,
	pairing_faults = pairing_faults, hint_faults = hint_faults, altered_begin = altered.tree_begin, altered_terrain = altered.tree_terrain,
}
-- the last thing the witness does is close its evidence, and a close that fails is a failure after the success row as much as before it
local closed, close_error = out:close ()
if not closed then error ("tree-authority: OUTPUT FAILURE: closing " .. out_path .. " failed: " .. tostring (close_error), 0) end
io.stderr:write (string.format ("tree-authority: mode=%s completed=%s cohort=%d ended=%d tree_begin=%d tree_terrain=%d altered=%d/%d failure=%s\n", mode, tostring (failure == nil), #members,
	#terminals, after.tree_begin, after.tree_terrain, altered.tree_begin, altered.tree_terrain, failure or "none"))

-- ---------------------------------------------------------------------------------------------------------------- the verdict the process exit carries
local function verdict (message) error ("tree-authority: " .. message, 0) end
if plan.marker then
	-- a refusal mode: the engine must have stopped, at this marker, at an armed tree question the provider altered
	if failure == nil then verdict ("mode " .. mode .. " completed: the engine did not stop at " .. plan.marker) end
	if not string.find (failure, "[terrain-provider:" .. plan.marker .. "]", 1, true) then verdict ("mode " .. mode .. " stopped, but not at [terrain-provider:" .. plan.marker .. "]: " .. failure) end
	if altered.tree_begin + altered.tree_terrain < 1 then verdict ("mode " .. mode .. " stopped without the provider having altered a tree answer: " .. failure) end
else
	if failure ~= nil then verdict ("mode " .. mode .. " failed in the engine: " .. failure) end
	-- the route is the host's: the weapon's tree lookups reached the provider
	if after.tree_begin < 1 then verdict ("no tree_begin reached the provider after the launch: the tree route is not the host's (or no tree lookup was made)") end
	if mode == "control" then
		if altered.tree_begin + altered.tree_terrain ~= 0 then verdict ("the control altered a tree answer") end
		if order_faults + pairing_faults + hint_faults ~= 0 then
			verdict (string.format ("the control's tree dialogue is not the original's: order_faults=%d pairing_faults=%d hint_faults=%d", order_faults, pairing_faults, hint_faults))
		end
	elseif altered.tree_begin < 1 then
		verdict ("mode " .. mode .. " altered no tree answer")
	end
end
