-- TEST-ONLY controlled person target-facing case (EEDCS-44). Needs a DLL built with the `aim-fixture` feature.
--
-- The ordinary 600 s and 900 s witnesses never enter the person target-facing branch (ps_anim.c): no person has a target in them, on Lebanon or Georgia.
-- This is one bounded, fresh original case that does: it boots the campaign through the public path (require ("eech_dc")) with the reference host's six
-- callbacks, steps until persons exist, then picks ONE real host-identified person and ONE real host-identified enemy vehicle by their host ids, gives the
-- person that target through the campaign's own target-link call (a case PRECONDITION: it asserts nothing about the campaign's target selection), and steps
-- normally. ps_anim.c then asks the host to turn the person toward the target (EEDCS-44: the move_vehicle mode person_face; the original asked the host to place the pose the campaign had turned); the driver wraps
-- the host's move_vehicle to record every step of that person with the attitude the campaign sent and the attitude the host answered, and reads the engine's
-- mirror of the person after every frame. Positive assertions, each recorded as a `check` event:
--   * the person branch was entered with a target link and the facing hook fired (the fixture's counters),
--   * the facing heading equals the bearing from the person to the target computed independently here from the positions the hook saw,
--   * the turn from the person's heading before is not trivial,
--   * the engine's mirror attitude after the frame is that heading, and equals the last attitude the host answered for the person (mirror == retained D1 pose),
--   * the host answered the heading matrix of the bearing from ITS record of the person to the target position it was sent, and every later step for the person arrived with the
--     attitude the host last answered (the host refuses a campaign write it did not make; none was refused).
-- case=control runs the same selection and steps WITHOUT the target link: the facing branch must stay at 0 (the witness does not pass by accident).
--
-- host.args: root, scenario (lebanon_retail default, georgia_retail), case (target default, control), warmup (simulated seconds, default 20), max_range (metres, default 12000: the closest pair in the world is chosen),
--            frames (default 20), release_frames (default 20: frames stepped after the link is released), frame_ms (default 100), out (JSON Lines path)

io.stdout:setvbuf ("no")

local args = host.args
local root = assert (args.root, "root=<installation root>")
local case = args.case or "target"
assert (case == "target" or case == "control", "case=target|control")
local warmup = tonumber (args.warmup or "20")
local nframes = tonumber (args.frames or "20")
local release_frames = tonumber (args.release_frames or "20")
local frame_ms = tonumber (args.frame_ms or "100")
local out = assert (io.open (assert (args.out, "out=<JSON Lines path>"), "w"))

local scenarios = {
	lebanon_retail = { map = "..\\common\\maps\\map5", campaign = "LEBANON.CHC" },
	georgia_retail = { map = "..\\common\\maps\\map3", campaign = "GEORGIA.CHC" },
}
local scenario = assert (scenarios [args.scenario or "lebanon_retail"], "scenario=lebanon_retail|georgia_retail")

local function emit (t)
	local parts = {}
	for k, v in pairs (t) do
		local value
		if type (v) == "string" then
			value = string.format ("%q", v)
		elseif type (v) == "table" then
			local items = {}
			for i, x in ipairs (v) do items [i] = type (x) == "string" and string.format ("%q", x) or tostring (x) end
			value = "[" .. table.concat (items, ",") .. "]"
		else
			value = tostring (v)
		end
		parts [#parts + 1] = string.format ("%q:%s", k, value)
	end
	table.sort (parts)
	out:write ("{" .. table.concat (parts, ",") .. "}\n")
	out:flush ()
end

local string_keys = { physical = true, decision = true }

local function emit_report (scope, at, report)
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
		emit (t)
	end
end

local function numbers (list)
	local t = {}
	for v in string.gmatch (list, "[^,]+") do t [#t + 1] = tonumber (v) end
	return t
end

-- one "aim-case-state" line as a table (the lists as number arrays)
local function parse_state (text, label)
	for line in string.gmatch (text, "[^\n]+") do
		if string.match (line, "^aim%-case%-state label=" .. label .. " ") then
			local s = {}
			for k, v in string.gmatch (line, "(%S+)=(%S+)") do s [k] = v end
			s.alive = tonumber (s.alive)
			s.has_target = tonumber (s.has_target)
			s.position = numbers (s.position)
			s.attitude = numbers (s.attitude)
			s.target_position = numbers (s.target_position)
			return s
		end
	end
	error ("no aim-case-state " .. label .. " in: " .. tostring (text))
end

local function heading_of (a) return math.atan2 (a [7], a [9]) end -- the attitude's forward row (m[2][0], m[2][2]): (sin h, cos h)
local function wrap (a) while a > math.pi do a = a - 2 * math.pi end while a < -math.pi do a = a + 2 * math.pi end return a end
local function copy (t) if not t then return nil end local c = {} for i, v in ipairs (t) do c [i] = v end return c end

-- the host: the reference host's callbacks, wrapped to record what the case needs and to forward everything unchanged
local descriptors, order = {}, {}
local watched, steps, frame_now = nil, {}, 0
local function bind_mobile (descriptor)
	local id = host.bind_mobile (descriptor)
	descriptors [id] = { kind = descriptor.movement_kind, side = descriptor.side, pose = descriptor.pose }
	order [#order + 1] = id
	return id
end
local function move_vehicle (step)
	local result = host.move_vehicle (step)
	if watched and step.host_id == watched then
		steps [#steps + 1] = { frame = frame_now, mode = step.mode, attitude_in = copy (step.pose and step.pose.attitude), attitude_out = copy (result.attitude),
			position_in = copy (step.pose and step.pose.position), target = copy (step.pf_target), heading_out = result.angles and result.angles [1] }
	end
	return result
end

local dc = require ("eech_dc")
dc.prepare_installation (root)
local engine = dc.boot {
	install_root = root, map = scenario.map, terrain = host.terrain_provider (root, scenario.map), environment = host.env_provider (), campaign_directory = "camp01", campaign = scenario.campaign, gunship = "apache",
	seed = 1, bind_mobile = bind_mobile, move_vehicle = move_vehicle, observe_loss = host.observe_loss,
	supply_reached = host.supply_reached, line_of_sight = host.line_of_sight, sense = host.sense, aim = host.aim,
}
emit { event = "boot", scenario = args.scenario or "lebanon_retail", case = case, warmup = warmup, frames = nframes, frame_ms = frame_ms }

local frames = math.floor (warmup * 1000 / frame_ms)
for _ = 1, frames do engine:frame (frame_ms) end
emit { event = "warmed_up", elapsed = engine:clock ().elapsed_seconds, bound = #order }

-- Rank real person/enemy ground pairs by binding-time descriptor distance within
-- max_range, then query current state and require a non-trivial turn. This does
-- not select the currently closest pair. The retained case's binding/current
-- separations are 5853.8/5856.5 m; an earlier 11.7 km case lost its link after one
-- frame for an uncharacterized campaign reason. Selection is a case precondition.
local max_range = tonumber (args.max_range or "12000")
local pairs_found = {}
for _, id in ipairs (order) do
	local d = descriptors [id]
	if d.kind == 3 and d.pose and (d.side == "blue" or d.side == "red") then
		local px, pz = d.pose.position [1], d.pose.position [3]
		for _, other in ipairs (order) do
			local o = descriptors [other]
			if other ~= id and (o.kind == 1 or o.kind == 2 or o.kind == 3) and o.pose and (o.side == "blue" or o.side == "red") and o.side ~= d.side then
				local dx, dz = o.pose.position [1] - px, o.pose.position [3] - pz
				local range = math.sqrt (dx * dx + dz * dz)
				if range > 150 and range < max_range then pairs_found [#pairs_found + 1] = { person = id, target = other, range = range } end
			end
		end
	end
end
table.sort (pairs_found, function (a, b) if a.range ~= b.range then return a.range < b.range end if a.person ~= b.person then return a.person < b.person end return a.target < b.target end)
emit { event = "candidates", count = #pairs_found, max_range = max_range }
local chosen
for _, c in ipairs (pairs_found) do
	local ok, text = pcall (engine.aim_person_query, engine, c.person)
	local ok2, text2 = pcall (engine.aim_person_query, engine, c.target)
	if ok and ok2 then
		local p, t = parse_state (text, "query"), parse_state (text2, "query")
		if p.alive == 1 and t.alive == 1 and p.type == "ENTITY_TYPE_PERSON" and p.has_target == 0 then
			local bearing = math.atan2 (t.position [1] - p.position [1], t.position [3] - p.position [3])
			local turn = math.abs (wrap (bearing - heading_of (p.attitude)))
			if turn > 0.5 then chosen = { person = c.person, target = c.target, bearing = bearing, turn = turn, range = c.range, target_type = t.type } break end
		end
	end
end
assert (chosen, "no host-identified person with an enemy ground target inside max_range and a turn of at least 0.5 rad was found")
emit { event = "chosen", person = chosen.person, target = chosen.target, target_type = chosen.target_type, range = chosen.range, bearing = chosen.bearing, turn_needed = chosen.turn }

watched = chosen.person
engine:aim_trace_reset ()
local before, after_setup
if case == "target" then
	local text = engine:aim_person_case (chosen.person, chosen.target)
	before, after_setup = parse_state (text, "before"), parse_state (text, "after_setup")
else
	before = parse_state (engine:aim_person_query (chosen.person), "query")
end
emit { event = "state", label = "before", heading = heading_of (before.attitude), attitude = before.attitude, position = before.position, has_target = before.has_target }
if after_setup then emit { event = "state", label = "after_setup", has_target = after_setup.has_target, target_host_id = after_setup.target_host_id, target_position = after_setup.target_position } end

local mirror = {}
for f = 1, nframes do
	frame_now = f
	local first = #steps + 1
	engine:frame (frame_ms)
	local q = parse_state (engine:aim_person_query (chosen.person), "query")
	local last_place
	for i = first, #steps do if steps [i].mode == "person_face" then last_place = steps [i] end end
	mirror [f] = { heading = heading_of (q.attitude), attitude = q.attitude, alive = q.alive, has_target = q.has_target, place = last_place, steps = #steps - first + 1 }
	emit { event = "frame", frame = f, heading = mirror [f].heading, alive = q.alive, has_target = q.has_target, host_steps = mirror [f].steps,
		face_in = last_place and last_place.attitude_in or "none", face_out = last_place and last_place.attitude_out or "none" }
end

-- release the link through the same campaign call and step on: the person returns to its own movement, whose steps the host compares with its record
local released, release_first
if case == "target" then
	released = parse_state (engine:aim_person_case (chosen.person, ""), "after_release")
	emit { event = "state", label = "after_release", has_target = released.has_target }
	release_first = #steps + 1
	for f = nframes + 1, nframes + release_frames do
		frame_now = f
		engine:frame (frame_ms)
	end
	emit { event = "released", host_steps_after_release = #steps - release_first + 1 }
end

local report = engine:aim_trace ()
emit_report ("run", engine:clock ().elapsed_seconds, report)
local counters = {}
for name, value in string.gmatch (report, "aim%-counter name=(%S+) value=(%d+)") do counters [name] = tonumber (value) end
local facing_hook
for line in string.gmatch (report, "[^\n]+") do
	if string.match (line, "^aim%-sample key=person/facing ") then
		local px, py, pz, tx, ty, tz, h = string.match (line, "position=%(([^,]+),([^,]+),([^)]+)%) target=%(([^,]+),([^,]+),([^)]+)%) heading=(%S+)")
		if px then facing_hook = { px = tonumber (px), pz = tonumber (pz), tx = tonumber (tx), tz = tonumber (tz), heading = tonumber (h) } end
		break
	end
end

local failed = 0
local function check (name, ok, detail)
	emit { event = "check", name = name, ok = ok and true or false, detail = detail or "" }
	if not ok then failed = failed + 1 end
end

local function close (a, b, tolerance)
	if not a or not b then return false end
	for i = 1, #a do if math.abs (a [i] - b [i]) > tolerance then return false end end
	return #a == #b
end

if case == "target" then
	check ("target link set by the campaign's own call", after_setup and after_setup.has_target == 1 and after_setup.target_host_id == chosen.target)
	check ("person branch entered with a target link", (counters.person_branch_entered or 0) >= 1 and (counters.person_branch_with_target_link or 0) >= 1,
		string.format ("entered %s with_target_link %s", tostring (counters.person_branch_entered), tostring (counters.person_branch_with_target_link)))
	check ("facing hook fired", (counters.person_facing or 0) >= 1 and facing_hook ~= nil, "person_facing " .. tostring (counters.person_facing))
	if facing_hook then
		local expected = math.atan2 (facing_hook.tx - facing_hook.px, facing_hook.tz - facing_hook.pz)
		check ("facing heading equals the bearing to the target computed here from the hook's own positions", math.abs (wrap (facing_hook.heading - expected)) < 1e-5,
			string.format ("hook %.9g independent %.9g", facing_hook.heading, expected))
		check ("turn from the heading before is not trivial (> 0.3 rad)", math.abs (wrap (facing_hook.heading - heading_of (before.attitude))) > 0.3,
			string.format ("before %.9g after %.9g", heading_of (before.attitude), facing_hook.heading))
		check ("mirror attitude after the first frame is the hook's heading", math.abs (wrap (mirror [1].heading - facing_hook.heading)) < 1e-5,
			string.format ("mirror %.9g hook %.9g", mirror [1].heading, facing_hook.heading))
	end
	local places, answered_facing, mirror_equals_host = 0, true, true
	for f = 1, nframes do
		local p = mirror [f].place
		if p then
			places = places + 1
			local expected = math.atan2 (p.target [1] - p.position_in [1], p.target [3] - p.position_in [3])
			if math.abs (wrap (heading_of (p.attitude_out) - expected)) > 1e-5 or math.abs (wrap ((p.heading_out or 1e9) - expected)) > 1e-5 then answered_facing = false end
			if not close (mirror [f].attitude, p.attitude_out, 1e-6) then mirror_equals_host = false end
		end
	end
	check ("the host was asked to turn the person (person_face) in most frames", places >= math.floor (nframes / 2), string.format ("%d person_face steps in %d frames", places, nframes))
	check ("the host answered the facing of its own record toward the target (attitude and heading == the independent bearing)", places > 0 and answered_facing)
	check ("mirror attitude after each frame == the attitude the host answered (retained D1 pose)", places > 0 and mirror_equals_host)
	local continuity, compared = true, 0
	local held
	for i = 1, #steps do
		local s = steps [i]
		if held and s.attitude_in and s.mode ~= "person_face" then
			compared = compared + 1
			if not close (s.attitude_in, held, 0) then continuity = false end
		end
		if s.attitude_out then held = s.attitude_out end
	end
	check ("the link was released through the same call", released and released.has_target == 0)
	check ("every non-person_face step arrived with the attitude the host last answered (no write the host did not make); at least one occurred after the release", compared > 0 and continuity,
		string.format ("%d non-person_face steps compared", compared))
else
	check ("no target link in the control", before.has_target == 0 and mirror [nframes].has_target == 0)
	check ("control: person branch not entered", (counters.person_branch_entered or 0) == 0 and (counters.person_facing or 0) == 0,
		string.format ("entered %s facing %s", tostring (counters.person_branch_entered), tostring (counters.person_facing)))
	local turned = math.abs (wrap (mirror [nframes].heading - heading_of (before.attitude)))
	check ("control: the person did not turn toward the target", turned < 0.1, string.format ("turn %.9g rad, bearing needed %.9g", turned, chosen.turn))
end

local modes = {}
for _, s in ipairs (steps) do modes [s.mode] = (modes [s.mode] or 0) + 1 end
local mode_list = {}
for mode, count in pairs (modes) do mode_list [#mode_list + 1] = mode .. "=" .. count end
table.sort (mode_list)
emit { event = "verdict", case = case, failed = failed, frames = nframes, host_steps = #steps, step_modes = mode_list }
out:close ()
io.stderr:write (string.format ("aim-person-case %s: %d failed checks, %d host steps for the person\n", case, failed, #steps))
assert (failed == 0, string.format ("aim-person-case %s: %d checks failed", case, failed))
