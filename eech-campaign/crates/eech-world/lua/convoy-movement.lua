-- Public causal check for host-owned ground convoy movement: the host owns where a
-- routed vehicle is, so a host that does not move it keeps the campaign from
-- arriving, and a host that moves it again lets the campaign advance.
--
-- mode=control  the reference host moves every vehicle. The first routed vehicle that the
--               reference host moves at least 10 m and then reports arrived at a waypoint
--               is the subject: its identity, when it first moved and when it arrived.
-- mode=hold     select=<host id> from the control run. From the first step the campaign asks
--               the host to move that identity until hold_seconds later, the host declines
--               to move it (it answers with the pose it already holds, calling no step).
--               Every other vehicle is moved by the reference host. The campaign must keep
--               that identity on the same waypoint while it is held and advance it only
--               after the host moves it again (or the host observes its loss).
-- mode=missing  the campaign refuses to start without a host move_vehicle callback.
-- Progression is read only from the waypoint_key the campaign puts on each descriptor.

local args = host.args
local root = assert (args.root, "root=<installation root>")
local mode = assert (args.mode, "mode=control|hold|missing")
local frame_ms = tonumber (args.frame_ms or "100")
local frames = math.floor (tonumber (args.hours or "1") * 3600000 / frame_ms + 0.5)
local hold_seconds = tonumber (args.hold_seconds or "0")
local subject = args.select
local dc = require ("eech_dc")
dc.prepare_installation (root)

local config = {
	install_root = root,
	map = "..\\common\\maps\\map5",
	terrain = host.terrain_provider (root, "..\\common\\maps\\map5"), environment = host.env_provider (),
	campaign_directory = "camp01",
	campaign = "LEBANON.CHC",
	gunship = "apache",
	seed = 1,
	bind_mobile = host.bind_mobile,
	observe_loss = host.observe_loss,
	supply_reached = host.supply_reached,
	line_of_sight = host.line_of_sight,
	sense = host.sense, aim = host.aim,
}

if mode == "missing" then
	local ok, err = pcall (dc.boot, config)
	assert (not ok, "the campaign started without a host move_vehicle callback")
	assert (tostring (err):find ("missing host move_vehicle callback", 1, true), "unexpected refusal: " .. tostring (err))
	print ("refusal\t" .. (tostring (err):gsub ("\n.*", "")))
	print ("result\tok")
	return
end
assert (mode == "control" or mode == "hold", "mode=control|hold|missing")
assert (mode == "control" or subject, "select=<host id>")

local reference_move = host.move_vehicle
local reference_reached = host.supply_reached
local reference_loss = host.observe_loss
local elapsed = 0

local selected_host, selected_at, selected_waypoint
local first_move = {}
local held_steps, moved_steps = 0, 0
local released_at, advanced_to, advanced_at, lost_at
local first_held, last_held, after_release_first, after_release_last

local function copy (position) return { position [1], position [2], position [3] } end

local function distance (a, b)
	local dx, dy, dz = a [1] - b [1], a [2] - b [2], a [3] - b [3]
	return math.sqrt (dx * dx + dy * dy + dz * dz)
end

local function holding () return mode == "hold" and elapsed - selected_at < hold_seconds end

local function move_vehicle (step)
	if step.mode == "move" and step.desired_velocity > 0 and not first_move [step.host_id] then
		first_move [step.host_id] = { at = elapsed, position = copy (step.pose.position) }
		if step.host_id == subject then selected_host, selected_at = subject, elapsed end
	end
	if step.host_id == selected_host and step.mode == "move" then
		if holding () then
			held_steps = held_steps + 1
			-- the host does not move it: the pose it holds is the answer
			return { position = step.pose.position, velocity = step.pose.velocity, attitude = step.pose.attitude, rotated = false }
		end
		released_at = released_at or elapsed
		moved_steps = moved_steps + 1
	end
	return reference_move (step)
end

local function supply_reached (descriptor)
	if mode == "control" and not selected_host and descriptor.query == "reached" and descriptor.mobile == "routed_vehicle" then
		local start = first_move [descriptor.host_id]
		if start and reference_reached (descriptor).gates_passed and distance (start.position, descriptor.position) >= 10 then
			selected_host, selected_at, selected_waypoint = descriptor.host_id, start.at, descriptor.waypoint_key
		end
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
		if selected_waypoint ~= descriptor.waypoint_key and not advanced_to then
			advanced_to, advanced_at = descriptor.waypoint_key, elapsed
		end
	end
	return reference_reached (descriptor)
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
	if (mode == "control" and selected_host) or advanced_to or lost_at then break end
end

if not selected_host then
	print ("selected\tnone")
	print ("result\tunavailable")
	return
end
print (string.format ("diag_selected\tmode=%s host=%s selected_at=%.1f hold_seconds=%.1f waypoint=%s", mode, selected_host, selected_at, hold_seconds, tostring (selected_waypoint)))
print (string.format ("diag_steps\theld=%d moved=%d released_at=%s advanced_to=%s advanced_at=%s lost_at=%s ended_at=%.1f", held_steps, moved_steps, tostring (released_at), tostring (advanced_to), tostring (advanced_at), tostring (lost_at), elapsed))
if mode == "control" then
	print ("subject\t" .. selected_host)
	print (string.format ("first_move_at\t%.1f", selected_at))
	print (string.format ("arrived_at\t%.1f", elapsed))
	print ("result\tcontrol")
	return
end

assert (held_steps > 1, "the selected convoy was held for only one step")
assert (released_at, "the held convoy was never released")
assert (not advanced_to or advanced_at >= released_at, "the campaign advanced the convoy while the host held it")
assert (advanced_to or (lost_at and lost_at >= released_at), "the campaign did not advance the convoy after the host moved it again, and the host observed no loss")
assert (first_held and last_held and distance (first_held, last_held) == 0, "the held convoy moved in the campaign while the host held it")
assert (after_release_first and after_release_last and distance (after_release_first, after_release_last) > 0, "the released convoy did not move")
print ("select\thold")
print ("host_id\t" .. selected_host)
print (string.format ("held_seconds\t%.1f", released_at - selected_at))
print (string.format ("held_steps\t%d", held_steps))
print (string.format ("held_displacement\t%.3f", distance (first_held, last_held)))
print (string.format ("displacement_after_release\t%.3f", distance (after_release_first, after_release_last)))
print ("outcome_after_release\t" .. (advanced_to and "advanced" or "lost"))
print (string.format ("waypoint_key_before_after\t%s\t%s", tostring (selected_waypoint), tostring (advanced_to)))
print (string.format ("advanced_after_release_seconds\t%.1f", (advanced_at or lost_at) - released_at))
print ("result\tok")
