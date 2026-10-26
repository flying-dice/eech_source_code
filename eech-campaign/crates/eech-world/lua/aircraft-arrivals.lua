-- Public causal checks for non-SUPPLY fixed-wing and helicopter radius arrival.
-- The host wraps its reference physical predicate. Campaign progression is
-- read only from the correlation keys the campaign puts on each descriptor:
-- `waypoint_key` names the guide's current waypoint, so a changed key is the
-- campaign having advanced. A task can belong to several guides and groups, so
-- the withhold mode follows one host identity: it denies, samples and
-- correlates only the descriptors of the member it selected, and judges
-- movement only from that member's own positions.

local args = host.args
local root = assert (args.root, "root=<installation root>")
local mode = assert (args.mode, "mode=reference|withhold")
local frame_ms = tonumber (args.frame_ms or "100")
local frames = math.floor (tonumber (args.hours or "1") * 3600000 / frame_ms + 0.5)
local delay_seconds = tonumber (args.delay_seconds or "10")
local dc = require ("eech_dc")
dc.prepare_installation (root)

local ARRIVAL = "radius_arrival"
local MOVING = 20 -- metres per second
local MINIMUM_DISPLACEMENT = 1 -- metres
local elapsed = 0
local reference = host.supply_reached

-- reference mode
local counts, pass_through, landed_reject, leader_cohesion = {}, {}, {}, {}

-- withhold mode
local selected, selected_at, selected_waypoint, selected_host
local withheld_calls, positions, released_at, advanced_to, advanced_task, advanced_at = 0, {}, nil, nil, nil, nil

-- distance between two positions of the selected identity
local function distance (a, b)
	local dx, dy, dz = a[1] - b[1], a[2] - b[2], a[3] - b[3]
	return math.sqrt (dx * dx + dy * dy + dz * dz)
end

local function supply_reached (descriptor)
	local answer = reference (descriptor)
	local reached = answer.gates_passed
	-- this check is about aircraft arrival; the other queries and mobile kinds have their own witnesses
	if descriptor.waypoint ~= ARRIVAL or descriptor.query ~= "reached" then return answer end
	if descriptor.mobile ~= "fixed_wing" and descriptor.mobile ~= "helicopter" then return answer end

	assert (type (descriptor.waypoint_sub_type) == "number", "descriptor carries no campaign waypoint sub-type")
	counts[descriptor.mobile] = (counts[descriptor.mobile] or 0) + 1
	if descriptor.mobile == "fixed_wing" then
		if reached and descriptor.distance > descriptor.radius then pass_through[#pass_through + 1] = descriptor.host_id end
		if descriptor.landed and not reached then landed_reject[#landed_reject + 1] = descriptor.host_id end
	elseif not descriptor.is_task_leader then
		leader_cohesion[#leader_cohesion + 1] = descriptor.host_id
	end

	if mode ~= "withhold" then return answer end

	-- a leader that is already flying, so that its own movement can be told from a hover
	if not selected_host and reached and descriptor.is_task_leader and descriptor.velocity > MOVING then
		selected, selected_at = descriptor.task_key, elapsed
		selected_waypoint, selected_host = descriptor.waypoint_key, descriptor.host_id
	end
	-- every other member, including the rest of the same task, keeps the reference answer
	if descriptor.host_id ~= selected_host then return answer end

	if descriptor.waypoint_key == selected_waypoint then
		assert (descriptor.task_key == selected, "selected host identity changed task at the same waypoint")
		if elapsed - selected_at < delay_seconds then
			-- the reference host would have said yes: say no, and keep watching
			withheld_calls = withheld_calls + 1
			positions[#positions + 1] = { descriptor.position[1], descriptor.position[2], descriptor.position[3] }
			return false
		end
		released_at = released_at or elapsed
		return answer
	end
	-- a different current waypoint for the same identity (possibly the next task's): the campaign advanced
	assert (released_at, "campaign advanced the member while its arrival was withheld")
	advanced_to, advanced_task, advanced_at = descriptor.waypoint_key, descriptor.task_key, advanced_at or elapsed
	return answer
end

local engine = dc.boot {
	install_root = root,
	map = "..\\common\\maps\\map5",
	campaign_directory = "camp01",
	campaign = "LEBANON.CHC",
	gunship = "apache",
	seed = 1,
	bind_mobile = host.bind_mobile, move_vehicle = host.move_vehicle,
	observe_loss = host.observe_loss,
	supply_reached = supply_reached,
	line_of_sight = host.line_of_sight,
	sense = host.sense,
}

for _ = 1, frames do
	engine:frame (frame_ms)
	elapsed = elapsed + frame_ms / 1000
	if mode == "withhold" and advanced_to then break end
end

if mode == "reference" then
	assert ((counts.fixed_wing or 0) > 0, "reference host received no non-SUPPLY fixed-wing arrival")
	assert ((counts.helicopter or 0) > 0, "reference host received no non-SUPPLY helicopter arrival")
	print (string.format ("fixed_wing\t%d", counts.fixed_wing))
	print (string.format ("helicopter\t%d", counts.helicopter))
	print (string.format ("fixed_wing_pass_through\t%d", #pass_through))
	print (string.format ("fixed_wing_landed_reject\t%d", #landed_reject))
	print (string.format ("helicopter_non_leader\t%d", #leader_cohesion))
else
	assert (selected, "withhold mode never selected a non-SUPPLY arrival")
	assert (released_at, "the withheld arrival was never released")
	assert (advanced_to, "the campaign did not advance the task after the arrival was released")
	assert (withheld_calls > 1, "the selected member was asked about its arrival only once")
	local moved_metres = distance (positions[1], positions[#positions])
	assert (moved_metres > MINIMUM_DISPLACEMENT, "the selected host identity did not move while its arrival was withheld")
	local first, last = positions[1], positions[#positions]
	print (string.format ("task_key_before_after\t%.0f\t%.0f", selected, advanced_task))
	print (string.format ("waypoint_key_before_after\t%.0f\t%.0f", selected_waypoint, advanced_to))
	print ("leader_host_id\t" .. selected_host)
	print (string.format ("withheld_seconds\t%.1f", released_at - selected_at))
	print (string.format ("withheld_calls\t%d", withheld_calls))
	print (string.format ("selected_first_position\t%.3f\t%.3f\t%.3f", first[1], first[2], first[3]))
	print (string.format ("selected_last_position\t%.3f\t%.3f\t%.3f", last[1], last[2], last[3]))
	print (string.format ("selected_displacement\t%.3f", moved_metres))
	print (string.format ("advanced_after_release_seconds\t%.1f", advanced_at - released_at))
end
print ("result\tok")
