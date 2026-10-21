-- Public causal checks for the remaining physical arrival and proximity
-- queries: routed/ship/anti-aircraft/person arrival, helicopter altitude and
-- heading gates, the weapon-vector gate, the waypoint action radius and the
-- recon radius. The host wraps its reference physical predicate.
--
-- mode=census      counts what the campaign asks, per query and mobile kind.
-- mode=withhold    select=<name> follows ONE host identity: from the first
--                  descriptor of that kind the reference host would accept, it
--                  denies that identity's descriptors of that query at that
--                  waypoint for delay_seconds. Every other member keeps the
--                  reference answer. The campaign must keep that identity on
--                  the same waypoint while it is denied and advance it only
--                  after the denial is released. Progression is read only from
--                  the waypoint_key the campaign puts on each descriptor.

local args = host.args
local root = assert (args.root, "root=<installation root>")
local mode = assert (args.mode, "mode=census|withhold")
local select_name = args.select
local frame_ms = tonumber (args.frame_ms or "100")
local frames = math.floor (tonumber (args.hours or "1") * 3600000 / frame_ms + 0.5)
local delay_seconds = tonumber (args.delay_seconds or "10")
local dc = require ("eech_dc")
dc.prepare_installation (root)

local reference = host.supply_reached
local elapsed = 0

local VEHICLE = { routed_vehicle = true, ship_vehicle = true, anti_aircraft = true, person = true }

local selectors = {
	-- routed vehicle, ship, anti-aircraft or person arrival: landed, radius, altitude against the guide
	ground = function (d, answer)
		return d.query == "reached" and VEHICLE [d.mobile] and answer.gates_passed
	end,
	-- helicopter leader with the ALTITUDE criterion
	altitude = function (d, answer)
		return d.query == "reached" and d.mobile == "helicopter" and d.is_task_leader and d.altitude_valid and answer.gates_passed
	end,
	-- helicopter leader with the HEADING criterion and a required heading
	heading = function (d, answer)
		return d.query == "reached" and d.mobile == "helicopter" and d.is_task_leader and d.heading_valid
			and d.required_heading_present and answer.gates_passed
	end,
	-- a member the weapon-vector rule applies to, with both vectors present
	weapon = function (d, answer)
		return d.query == "reached" and d.weapon_gate and d.vectors_valid and answer.gates_passed and answer.launch_angle_error ~= nil
	end,
	action = function (d, answer)
		return d.query == "action" and answer.gates_passed
	end,
	recon = function (d, answer)
		return d.query == "recon" and answer.gates_passed
	end,
}
local selector
if mode == "withhold" then
	selector = assert (selectors [select_name], "select=ground|altitude|heading|weapon|action|recon")
end

-- census
local asked, passed, hosts = {}, {}, {}
local features = {}
local function count (table_, key) table_ [key] = (table_ [key] or 0) + 1 end

-- withhold
local selected, selected_at, selected_waypoint, selected_host, selected_query, selected_mobile, selected_subtype
local withheld_calls, positions, released_at, advanced_to, advanced_task, advanced_at = 0, {}, nil, nil, nil, nil

local function distance (a, b)
	local dx, dy, dz = a [1] - b [1], a [2] - b [2], a [3] - b [3]
	return math.sqrt (dx * dx + dy * dy + dz * dz)
end

local function supply_reached (descriptor)
	local answer = reference (descriptor)
	assert (type (answer) == "table", "the reference host answers with a result table")
	assert (type (answer.gates_passed) == "boolean", "the result carries the physical gates")
	if descriptor.waypoint ~= "radius_arrival" then return answer end

	if mode == "census" then
		local key = descriptor.query .. "\t" .. descriptor.mobile
		count (asked, key)
		if answer.gates_passed then count (passed, key) end
		hosts [key] = hosts [key] or {}
		hosts [key] [descriptor.host_id] = true
		if descriptor.query == "reached" then
			if descriptor.altitude_valid then count (features, "altitude_valid\t" .. descriptor.mobile) end
			if descriptor.heading_valid then count (features, "heading_valid\t" .. descriptor.mobile) end
			if descriptor.heading_valid and descriptor.required_heading_present then count (features, "required_heading\t" .. descriptor.mobile) end
			if descriptor.weapon_gate then count (features, "weapon_gate\t" .. descriptor.mobile) end
			if descriptor.vectors_valid then count (features, "vectors_valid\t" .. descriptor.mobile) end
			if answer.launch_angle_error ~= nil then count (features, "angle_reported\t" .. descriptor.mobile) end
			if not descriptor.is_task_leader then count (features, "non_leader\t" .. descriptor.mobile) end
		end
		return answer
	end

	if not selected_host and selector (descriptor, answer) then
		selected, selected_at = descriptor.task_key, elapsed
		selected_waypoint, selected_host = descriptor.waypoint_key, descriptor.host_id
		selected_query, selected_mobile, selected_subtype = descriptor.query, descriptor.mobile, descriptor.waypoint_sub_type
	end
	-- every other member, including the rest of the same task, keeps the reference answer
	if descriptor.host_id ~= selected_host then return answer end

	if descriptor.waypoint_key == selected_waypoint then
		assert (descriptor.task_key == selected, "selected host identity changed task at the same waypoint")
		if descriptor.query == selected_query and elapsed - selected_at < delay_seconds then
			-- the reference host would have said yes: say no, and keep watching
			withheld_calls = withheld_calls + 1
			positions [#positions + 1] = { descriptor.position [1], descriptor.position [2], descriptor.position [3] }
			return { gates_passed = false }
		end
		if descriptor.query == selected_query then released_at = released_at or elapsed end
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
	bind_mobile = host.bind_mobile,
	observe_loss = host.observe_loss,
	supply_reached = supply_reached,
}

for _ = 1, frames do
	engine:frame (frame_ms)
	elapsed = elapsed + frame_ms / 1000
	if mode == "withhold" and advanced_to then break end
end

local function sorted_keys (table_)
	local keys = {}
	for key in pairs (table_) do keys [#keys + 1] = key end
	table.sort (keys)
	return keys
end

if mode == "census" then
	for _, key in ipairs (sorted_keys (asked)) do
		local identities = 0
		for _ in pairs (hosts [key]) do identities = identities + 1 end
		print (string.format ("asked\t%s\t%d\tpassed=%d\tidentities=%d", key, asked [key], passed [key] or 0, identities))
	end
	for _, key in ipairs (sorted_keys (features)) do print (string.format ("feature\t%s\t%d", key, features [key])) end
else
	-- a selection that never happens is reported, not faked
	if not selected then
		print ("selected\tnone")
		print ("result\tunavailable")
		return
	end
	assert (released_at, "the withheld arrival was never released")
	assert (advanced_to, "the campaign did not advance the task after the arrival was released")
	assert (withheld_calls > 1, "the selected member was asked about its arrival only once")
	local first, last = positions [1], positions [#positions]
	print ("select\t" .. select_name)
	print ("query\t" .. selected_query)
	print ("mobile\t" .. selected_mobile)
	print ("waypoint_sub_type\t" .. tostring (selected_subtype))
	print (string.format ("task_key_before_after\t%.0f\t%.0f", selected, advanced_task))
	print (string.format ("waypoint_key_before_after\t%.0f\t%.0f", selected_waypoint, advanced_to))
	print ("leader_host_id\t" .. selected_host)
	print (string.format ("withheld_seconds\t%.1f", released_at - selected_at))
	print (string.format ("withheld_calls\t%d", withheld_calls))
	print (string.format ("selected_first_position\t%.3f\t%.3f\t%.3f", first [1], first [2], first [3]))
	print (string.format ("selected_last_position\t%.3f\t%.3f\t%.3f", last [1], last [2], last [3]))
	print (string.format ("selected_displacement\t%.3f", distance (first, last)))
	print (string.format ("advanced_after_release_seconds\t%.1f", advanced_at - released_at))
end
print ("result\tok")
