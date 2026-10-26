-- Public causal check for host-owned ship and infantry movement: the host owns where a
-- ship or a man is, so a host that does not move one keeps the campaign from seeing it
-- move, and a host that moves it again lets the campaign advance.
--
-- mode=census   the reference host moves everything. Counts, per movement kind, the members the
--               campaign bound, the host steps by mode, the members the host moved and the
--               members the campaign saw arrive. Nothing is asserted.
-- mode=control  kind=person|ship. The reference host moves everything. The subject is the first
--               member of that kind the host moves at least 10 m (a man also has to be reported
--               arrived at a waypoint): its identity, when it first moved and when it was selected.
-- mode=hold     kind=person|ship select=<host id> from the control run. From its first step the
--               host declines to move that member until hold_seconds later (it answers with the
--               pose it already holds, calling no step). Every other member is moved by the
--               reference host. The member must not move in the campaign while it is held and
--               must move after the host moves it again (a man must also stay on the same
--               waypoint while held, and advance it only after the release).
-- mode=missing  the campaign refuses to start without a host move_vehicle callback.
-- Progression is read only from the position and waypoint_key the campaign puts on each descriptor.

local args = host.args
local root = assert (args.root, "root=<installation root>")
local mode = assert (args.mode, "mode=census|control|hold|missing")
local frame_ms = tonumber (args.frame_ms or "100")
local frames = math.floor (tonumber (args.hours or "1") * 3600000 / frame_ms + 0.5)
local hold_seconds = tonumber (args.hold_seconds or "0")
local subject = args.select
local kind = args.kind
local dc = require ("eech_dc")
dc.prepare_installation (root)

local kind_names = { [1] = "routed", [2] = "ship", [3] = "person" }
local mobile_names = { person = "person", ship = "ship_vehicle" }
local mobile_kind = { person = "person", ship_vehicle = "ship" }
local move_modes = { ship = "ship_move", person = "person_begin" }

local reference_bind = host.bind_mobile
local reference_move = host.move_vehicle
local reference_reached = host.supply_reached
local reference_loss = host.observe_loss
local elapsed = 0
local kind_of = {}

local function bind_mobile (descriptor)
	local id = reference_bind (descriptor)
	if descriptor.movement_kind then kind_of [id] = kind_names [descriptor.movement_kind] end
	return id
end

local config = {
	install_root = root,
	map = "..\\common\\maps\\map5",
	campaign_directory = "camp01",
	campaign = "LEBANON.CHC",
	gunship = "apache",
	seed = 1,
	bind_mobile = bind_mobile,
	observe_loss = reference_loss,
	supply_reached = reference_reached,
	line_of_sight = host.line_of_sight,
}

if mode == "missing" then
	local ok, err = pcall (dc.boot, config)
	assert (not ok, "the campaign started without a host move_vehicle callback")
	assert (tostring (err):find ("missing host move_vehicle callback", 1, true), "unexpected refusal: " .. tostring (err))
	print ("refusal\t" .. (tostring (err):gsub ("\n.*", "")))
	print ("result\tok")
	return
end
assert (mode == "census" or mode == "control" or mode == "hold", "mode=census|control|hold|missing")
assert (mode == "census" or mobile_names [kind or ""], "kind=person|ship")
assert (mode ~= "hold" or subject, "select=<host id>")

local function copy (position) return { position [1], position [2], position [3] } end

local function distance (a, b)
	local dx, dy, dz = a [1] - b [1], a [2] - b [2], a [3] - b [3]
	return math.sqrt (dx * dx + dy * dy + dz * dz)
end

-- census
local steps = {}
local first_pose, moved_far, arrived = {}, {}, {}
local function count (name) steps [name] = (steps [name] or 0) + 1 end

-- control and hold
local selected_host, selected_at, selected_waypoint
local held_steps, moved_steps = 0, 0
local released_at, advanced_to, advanced_at, lost_at
local first_held, last_held, after_release_first, after_release_last

local function holding () return mode == "hold" and selected_at and elapsed - selected_at < hold_seconds end

local function is_subject_step (step)
	if kind_of [step.host_id] ~= kind then return false end
	return step.mode == move_modes [kind] or (kind == "person" and step.mode == "person_end")
end

local function move_vehicle (step)
	local mover_kind = kind_of [step.host_id]
	if step.mode ~= "place" then count ((mover_kind or "unbound") .. ":" .. step.mode) end
	if mode == "hold" and not selected_at and step.host_id == subject and is_subject_step (step) then
		selected_host, selected_at = subject, elapsed
	end
	if mode == "hold" and step.host_id == selected_host and is_subject_step (step) then
		if holding () then
			held_steps = held_steps + 1
			-- the host does not move it: the pose it holds is the answer; a man's begin still pends for the terrain sample
			return { position = step.pose.position, velocity = step.pose.velocity, attitude = step.pose.attitude, rotated = false,
				pending = step.mode == "person_begin" and step.terrain_follow or false }
		end
		released_at = released_at or elapsed
		moved_steps = moved_steps + 1
	end
	local result = reference_move (step)
	if step.mode == "ship_move" or step.mode == "person_end" or (step.mode == "person_begin" and not result.pending) then
		first_pose [step.host_id] = first_pose [step.host_id] or copy (step.pose.position)
		if distance (first_pose [step.host_id], result.position) >= 10 then moved_far [step.host_id] = true end
	end
	return result
end

local function supply_reached (descriptor)
	local answer = reference_reached (descriptor)
	local mover = mobile_kind [descriptor.mobile]
	if mode == "census" and descriptor.query == "reached" and mover and answer.gates_passed then arrived [descriptor.host_id] = true end
	if mode == "control" and not selected_host and descriptor.host_id and kind_of [descriptor.host_id] == kind
		and moved_far [descriptor.host_id] and (kind == "ship" or (descriptor.query == "reached" and answer.gates_passed)) then
		selected_host, selected_at, selected_waypoint = descriptor.host_id, elapsed, descriptor.waypoint_key
	end
	if mode == "hold" and descriptor.host_id == selected_host then
		selected_waypoint = selected_waypoint or descriptor.waypoint_key
		if holding () then
			first_held = first_held or copy (descriptor.position)
			last_held = copy (descriptor.position)
		elseif released_at then
			after_release_first = after_release_first or copy (descriptor.position)
			after_release_last = copy (descriptor.position)
		end
		if kind == "person" and selected_waypoint ~= descriptor.waypoint_key and not advanced_to then
			advanced_to, advanced_at = descriptor.waypoint_key, elapsed
		end
	end
	return answer
end

local function observe_loss (descriptor)
	if selected_host and descriptor.host_id == selected_host and descriptor.event == "physical" then lost_at = lost_at or elapsed end
	return reference_loss (descriptor)
end

config.move_vehicle, config.supply_reached, config.observe_loss = move_vehicle, supply_reached, observe_loss
local engine = dc.boot (config)

for _ = 1, frames do
	engine:frame (frame_ms)
	elapsed = elapsed + frame_ms / 1000
	if mode == "control" and selected_host then break end
	if mode == "hold" and (advanced_to or lost_at) then break end
	if mode == "hold" and released_at and kind == "ship" and after_release_last and distance (after_release_first, after_release_last) > 0 then break end
end

if mode == "census" then
	local bound, far, reached = {}, {}, {}
	for id, name in pairs (kind_of) do
		bound [name] = (bound [name] or 0) + 1
		if moved_far [id] then far [name] = (far [name] or 0) + 1 end
		if arrived [id] then reached [name] = (reached [name] or 0) + 1 end
	end
	for _, name in ipairs ({ "routed", "ship", "person" }) do
		print (string.format ("census\t%s\tbound=%d\tmoved_10m=%d\tarrived=%d", name, bound [name] or 0, far [name] or 0, reached [name] or 0))
	end
	local names = {}
	for name in pairs (steps) do names [#names + 1] = name end
	table.sort (names)
	for _, name in ipairs (names) do print (string.format ("steps\t%s\t%d", name, steps [name])) end
	print (string.format ("ended_at\t%.1f", elapsed))
	print ("result\tcensus")
	return
end

if not selected_host then
	print ("selected\tnone")
	print ("result\tunavailable")
	return
end
print (string.format ("diag_selected\tmode=%s kind=%s host=%s selected_at=%.1f hold_seconds=%.1f waypoint=%s", mode, kind, selected_host, selected_at, hold_seconds, tostring (selected_waypoint)))
print (string.format ("diag_steps\theld=%d moved=%d released_at=%s advanced_to=%s advanced_at=%s lost_at=%s ended_at=%.1f", held_steps, moved_steps, tostring (released_at), tostring (advanced_to), tostring (advanced_at), tostring (lost_at), elapsed))
if mode == "control" then
	print ("subject\t" .. selected_host)
	print (string.format ("selected_at\t%.1f", selected_at))
	print ("result\tcontrol")
	return
end

assert (held_steps > 1, "the selected member was held for only one step")
assert (released_at, "the held member was never released")
assert (first_held and last_held and distance (first_held, last_held) == 0, "the held member moved in the campaign while the host held it")
assert (not advanced_to or advanced_at >= released_at, "the campaign advanced the member while the host held it")
assert (after_release_first and after_release_last and distance (after_release_first, after_release_last) > 0, "the released member did not move")
if kind == "person" then
	assert (advanced_to or (lost_at and lost_at >= released_at), "the campaign did not advance the man after the host moved him again, and the host observed no loss")
end
print ("select\thold")
print ("host_id\t" .. selected_host)
print (string.format ("held_seconds\t%.1f", released_at - selected_at))
print (string.format ("held_steps\t%d", held_steps))
print (string.format ("held_displacement\t%.3f", distance (first_held, last_held)))
print (string.format ("displacement_after_release\t%.3f", distance (after_release_first, after_release_last)))
print ("outcome_after_release\t" .. (advanced_to and "advanced" or (lost_at and "lost" or "moved")))
print (string.format ("waypoint_key_before_after\t%s\t%s", tostring (selected_waypoint), tostring (advanced_to)))
print ("result\tok")
