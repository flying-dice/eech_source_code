-- Public causal checks for the four SUPPLY arrival predicates.
-- The host wraps its reference physical predicate; campaign state is read only
-- through engine:supply_tasks().

local args = host.args
local root = assert (args.root, "root=<installation root>")
local mode = assert (args.mode, "mode=reference|withhold_pick_up|delay_drop_off")
local frame_ms = tonumber (args.frame_ms or "100")
local frames = math.floor (tonumber (args.hours or "1") * 3600000 / frame_ms + 0.5)
local delay_seconds = tonumber (args.delay_seconds or "10")
local dc = require ("eech_dc")
dc.prepare_installation (root)
host.use_reference_arithmetic () -- the reference host's arithmetic profile, selected right after the installation is prepared (src/reference_arithmetic.rs)

local elapsed = 0
local counts = {}
local selected, selected_before, selected_at
local withheld_calls, positions, allowed = 0, {}, false
local preframe_by_task
local reference = host.supply_reached

local function close (a, b)
	return math.abs (a - b) < 0.001
end

local function snapshot_by_task (engine)
	local by_task = {}
	for _, task in ipairs (engine:supply_tasks ()) do by_task[task.task_key] = task end
	return by_task
end

local function record_position (descriptor)
	local p = descriptor.position
	positions[#positions + 1] = { p[1], p[2], p[3] }
end

local function unchanged (after, before, context)
	assert (after.waypoint == before.waypoint and after.waypoint_key == before.waypoint_key,
		context .. " advanced the task")
	assert (after.task_completed == before.task_completed, context .. " changed task completion")
	assert (after.cargo_on_leader == before.cargo_on_leader, context .. " moved cargo")
	assert (close (after.supplier_ammo, before.supplier_ammo) and close (after.supplier_fuel, before.supplier_fuel),
		context .. " changed the supplier")
	assert (close (after.receiver_ammo, before.receiver_ammo) and close (after.receiver_fuel, before.receiver_fuel),
		context .. " changed the receiver")
end

local function moved ()
	if #positions < 2 then return false end
	local first = positions[1]
	for index = 2, #positions do
		local p = positions[index]
		if math.abs (p[1] - first[1]) + math.abs (p[2] - first[2]) + math.abs (p[3] - first[3]) > 0.01 then return true end
	end
	return false
end

local function supply_reached (descriptor)
	local answer = reference (descriptor)
	local reached = answer.gates_passed
	counts[descriptor.waypoint] = (counts[descriptor.waypoint] or 0) + 1
	if mode == "withhold_pick_up" and descriptor.waypoint == "pick_up" then
		if not selected then
			local before = preframe_by_task[descriptor.task_key]
			if reached and before and before.waypoint == "pick_up" and before.waypoint_key == descriptor.waypoint_key
				and before.leader_host_id == descriptor.host_id
				and before.cargo_on_leader == 0 then
				selected = descriptor.task_key
				selected_before = before
				selected_at = elapsed
			end
		end
		if descriptor.task_key == selected then
			record_position (descriptor)
			if elapsed - selected_at < delay_seconds then
				withheld_calls = withheld_calls + 1
				return false
			end
			allowed = true
		end
	elseif mode == "delay_drop_off" and descriptor.waypoint == "drop_off" then
		if not selected then
			local before = preframe_by_task[descriptor.task_key]
			if reached and before and before.waypoint == "drop_off" and before.waypoint_key == descriptor.waypoint_key
				and before.leader_host_id == descriptor.host_id
				and before.cargo_on_leader > 0 then
				selected = descriptor.task_key
				selected_before = before
				selected_at = elapsed
			end
		end
		if descriptor.task_key == selected then
			record_position (descriptor)
			if elapsed - selected_at < delay_seconds then
				withheld_calls = withheld_calls + 1
				return false
			end
			allowed = true
		end
	end
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
	line_of_sight = host.line_of_sight, model = host.model,
	sense = host.sense, aim = host.aim,
}

local finished
for _ = 1, frames do
	preframe_by_task = snapshot_by_task (engine)
	engine:frame (frame_ms)
	elapsed = elapsed + frame_ms / 1000
	if selected then
		local after_by_task = snapshot_by_task (engine)
		local after = assert (after_by_task[selected], "selected SUPPLY task disappeared before the asserted transition")
		if mode == "withhold_pick_up" then
			if not allowed then
				unchanged (after, selected_before, "withheld PICK_UP")
			elseif after.waypoint_key ~= selected_before.waypoint_key then
				assert (after.leader_host_id == selected_before.leader_host_id, "allowed PICK_UP changed the observed leader")
				assert (after.cargo_on_leader > selected_before.cargo_on_leader, "allowed PICK_UP did not attach cargo to the leader")
				assert (after.supplier_ammo < selected_before.supplier_ammo or after.supplier_fuel < selected_before.supplier_fuel,
					"allowed PICK_UP did not debit the supplier")
				assert (after.task_completed == selected_before.task_completed, "allowed PICK_UP changed task completion")
				assert (withheld_calls > 1 and moved (), "physical member did not continue reporting movement while PICK_UP was withheld")
				finished = after
				break
			end
		elseif mode == "delay_drop_off" then
			if not allowed then
				unchanged (after, selected_before, "delayed DROP_OFF")
			elseif allowed and after.waypoint_key ~= selected_before.waypoint_key then
				assert (after.leader_host_id == selected_before.leader_host_id, "allowed DROP_OFF changed the observed leader")
				assert (after.receiver_ammo > selected_before.receiver_ammo or after.receiver_fuel > selected_before.receiver_fuel,
					"allowed DROP_OFF did not credit the receiver")
				assert (after.task_completed == selected_before.task_completed, "allowed DROP_OFF changed task completion before FINISH_DROP_OFF")
				assert (withheld_calls > 0, "DROP_OFF was not delayed")
				finished = after
				break
			end
		end
	end
end

if mode == "reference" then
	for _, waypoint in ipairs ({ "pick_up", "prepare_for_drop_off", "drop_off", "finish_drop_off" }) do
		assert ((counts[waypoint] or 0) > 0, "reference host did not receive " .. waypoint)
		print (string.format ("%s\t%d", waypoint, counts[waypoint]))
	end
else
	assert (selected and finished, mode .. " did not reach its causal assertion")
	print (string.format ("task_key\t%.0f", selected))
	print (string.format ("waypoint_key\t%.0f", selected_before.waypoint_key))
	print ("leader_host_id\t" .. selected_before.leader_host_id)
	print (string.format ("withheld_seconds\t%.1f", elapsed - selected_at))
	print (string.format ("withheld_calls\t%d", withheld_calls))
	print (string.format ("waypoint_before_after\t%s\t%s", selected_before.waypoint, finished.waypoint))
	print (string.format ("completed_before_after\t%d\t%d", selected_before.task_completed, finished.task_completed))
	print (string.format ("cargo_before_after\t%d\t%d", selected_before.cargo_on_leader, finished.cargo_on_leader))
	print (string.format ("supplier_before_after\t%.3f\t%.3f\t%.3f\t%.3f", selected_before.supplier_ammo,
		selected_before.supplier_fuel, finished.supplier_ammo, finished.supplier_fuel))
	print (string.format ("receiver_before_after\t%.3f\t%.3f\t%.3f\t%.3f", selected_before.receiver_ammo,
		selected_before.receiver_fuel, finished.receiver_ammo, finished.receiver_fuel))
end
print ("result\tok")
