-- Public causal checks for the remaining physical arrival and proximity
-- queries: routed/ship/anti-aircraft/person arrival, helicopter altitude and
-- heading gates, the weapon-vector gate, the waypoint action radius and the
-- recon radius. The host wraps its reference physical predicate.
--
-- mode=census      counts what the campaign asks, per query and mobile kind.
-- mode=control     the same selection and diagnostics, but the reference answer is
--                  never withheld: the no-denial trajectory to compare with.
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
local mode = assert (args.mode, "mode=census|withhold|control")
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
	-- AI helicopter leader with the HEADING criterion: the descriptor carries a heading source
	heading = function (d, answer)
		return d.query == "reached" and d.mobile == "helicopter" and d.is_task_leader and d.is_ai and d.heading_valid
			and d.heading_source ~= nil and answer.gates_passed
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
if mode ~= "census" then
	selector = assert (selectors [select_name], "select=ground|altitude|heading|weapon|action|recon")
end

-- census
local asked, passed, hosts = {}, {}, {}
local features = {}
local function count (table_, key) table_ [key] = (table_ [key] or 0) + 1 end

-- withhold
local selected, selected_at, selected_waypoint, selected_host, selected_query, selected_mobile, selected_subtype
local withheld_calls, positions, released_at, advanced_to, advanced_task, advanced_at = 0, {}, nil, nil, nil, nil
-- what the selected identity looked like at its selected waypoint, for the diagnostics
local watched = { calls = 0, passes = 0, after_calls = 0, after_passes = 0 }
-- every question the selected identity is asked after selection, of any waypoint kind
local seen, seen_order = {}, {}
-- physical losses the host observed for the selected identity after selection
local losses, selected_member, lost_at = {}, nil, nil
local observe_loss = host.observe_loss
local function watch_loss (descriptor)
	if mode ~= "census" and selected_host and (descriptor.host_id == selected_host or descriptor.member_key == selected_member) then
		if descriptor.event == "physical" and descriptor.host_id == selected_host then lost_at = lost_at or elapsed end
		losses [#losses + 1] = string.format ("at=%.1f event=%s host=%s member=%.0f group=%.0f", elapsed, tostring (descriptor.event), descriptor.host_id, descriptor.member_key, descriptor.group_key)
	end
	return observe_loss (descriptor)
end

local function distance (a, b)
	local dx, dy, dz = a [1] - b [1], a [2] - b [2], a [3] - b [3]
	return math.sqrt (dx * dx + dy * dy + dz * dz)
end

local function supply_reached (descriptor)
	local answer = reference (descriptor)
	assert (type (answer) == "table", "the reference host answers with a result table")
	assert (type (answer.gates_passed) == "boolean", "the result carries the physical gates")
	if mode ~= "census" and descriptor.host_id == selected_host then
		local key = string.format ("waypoint=%.0f kind=%s sub_type=%d query=%s", descriptor.waypoint_key, descriptor.waypoint, descriptor.waypoint_sub_type, descriptor.query)
		local entry = seen [key]
		if not entry then entry = { first = elapsed, calls = 0, passes = 0 }; seen [key] = entry; seen_order [#seen_order + 1] = key end
		entry.last, entry.calls = elapsed, entry.calls + 1
		if answer.gates_passed then entry.passes = entry.passes + 1 end
	end
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
			if descriptor.heading_source ~= nil then
				count (features, "heading_source_" .. descriptor.heading_source .. "\t" .. descriptor.mobile)
				if descriptor.heading_target_position ~= nil then count (features, "heading_target_position\t" .. descriptor.mobile) end
			end
			if answer.heading_target_missing then count (features, "heading_target_missing\t" .. descriptor.mobile) end
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
		selected_member = descriptor.member_key
	end
	-- every other member, including the rest of the same task, keeps the reference answer
	if descriptor.host_id ~= selected_host then return answer end

	if descriptor.waypoint_key == selected_waypoint then
		assert (descriptor.task_key == selected, "selected host identity changed task at the same waypoint")
		if descriptor.query == selected_query then
			local after = elapsed - selected_at >= delay_seconds
			watched.calls = watched.calls + 1
			if answer.gates_passed then watched.passes = watched.passes + 1 end
			if after then
				watched.after_calls = watched.after_calls + 1
				if answer.gates_passed then watched.after_passes = watched.after_passes + 1 end
			end
			watched.last, watched.last_at, watched.last_answer = descriptor, elapsed, answer.gates_passed
			watched.first = watched.first or descriptor
		end
		if mode == "withhold" and descriptor.query == selected_query and elapsed - selected_at < delay_seconds then
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
	bind_mobile = host.bind_mobile, move_vehicle = host.move_vehicle,
	observe_loss = watch_loss,
	supply_reached = supply_reached,
	line_of_sight = host.line_of_sight,
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
	local function describe (name, d)
		if not d then return end
		print (string.format ("diag_%s	distance=%.2f radius=%.2f velocity=%.2f landed=%s heading=%.4f heading_criterion=%.4f heading_source=%s waypoint_heading=%.4f altitude=%.2f altitude_criterion=%.2f guide_altitude=%.2f",
			name, d.distance, d.radius, d.velocity, tostring (d.landed), d.heading, d.heading_criterion, tostring (d.heading_source), d.waypoint_heading, d.altitude, d.altitude_criterion, d.guide_altitude))
	end
	print (string.format ("diag_selected	mode=%s host=%s task=%.0f waypoint=%.0f sub_type=%s query=%s mobile=%s selected_at=%.1f delay=%.1f", mode, selected_host, selected, selected_waypoint, tostring (selected_subtype), selected_query, selected_mobile, selected_at, delay_seconds))
	print (string.format ("diag_calls	total=%d passed=%d after_delay=%d after_delay_passed=%d withheld=%d released_at=%s advanced_to=%s advanced_at=%s ended_at=%.1f", watched.calls, watched.passes, watched.after_calls, watched.after_passes, withheld_calls, tostring (released_at), tostring (advanced_to), tostring (advanced_at), elapsed))
	describe ("first", watched.first)
	describe ("last", watched.last)
	for _, key in ipairs (seen_order) do
		local entry = seen [key]
		print (string.format ("diag_seen	%s first=%.1f last=%.1f calls=%d passed=%d", key, entry.first, entry.last, entry.calls, entry.passes))
	end
	for _, line in ipairs (losses) do print ("diag_loss	" .. line) end
	if watched.last then print (string.format ("diag_last_call	at=%.1f reference_gates_passed=%s", watched.last_at, tostring (watched.last_answer))) end
	if mode == "control" then
		print ("result	control")
		return
	end
	assert (released_at, "the withheld arrival was never released")
	-- a released member the world destroys before the campaign advances it is a different trajectory,
	-- not a missing advance; the host's own physical-loss observation is the only accepted explanation
	assert (advanced_to or (lost_at and lost_at >= released_at),
		"the campaign did not advance the task after the arrival was released, and the host observed no loss of the member")
	assert (withheld_calls > 1, "the selected member was asked about its arrival only once")
	local first, last = positions [1], positions [#positions]
	print ("select\t" .. select_name)
	print ("query\t" .. selected_query)
	print ("mobile\t" .. selected_mobile)
	print ("waypoint_sub_type\t" .. tostring (selected_subtype))
	print ("outcome_after_release\t" .. (advanced_to and "advanced" or "lost"))
	print (string.format ("task_key_before_after\t%.0f\t%s", selected, advanced_task and string.format ("%.0f", advanced_task) or "none"))
	print (string.format ("waypoint_key_before_after\t%.0f\t%s", selected_waypoint, advanced_to and string.format ("%.0f", advanced_to) or "none"))
	print ("leader_host_id\t" .. selected_host)
	print (string.format ("withheld_seconds\t%.1f", released_at - selected_at))
	print (string.format ("withheld_calls\t%d", withheld_calls))
	print (string.format ("selected_first_position\t%.3f\t%.3f\t%.3f", first [1], first [2], first [3]))
	print (string.format ("selected_last_position\t%.3f\t%.3f\t%.3f", last [1], last [2], last [3]))
	print (string.format ("selected_displacement\t%.3f", distance (first, last)))
	if advanced_to then print (string.format ("advanced_after_release_seconds\t%.1f", advanced_at - released_at))
	else print (string.format ("lost_after_release_seconds\t%.1f", lost_at - released_at)) end
end
print ("result\tok")
