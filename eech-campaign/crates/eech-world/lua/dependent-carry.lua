-- Public causal witness for host-owned movement-dependent positioning (EEDCS-40): the real campaign
-- (original C parent setters and child message handlers, reached through the public eech_dc module)
-- moves a hosted parent, and each hosted dependent it notifies is positioned by the host through a
-- `dependent_carry` step. Nothing is emulated: the steps are the ones the DLL's own handlers send.
--
-- The reference host moves everything and answers each carry with the real host arithmetic. This
-- script only observes. It counts the carries per child kind and shape, the carries whose parent
-- had itself been carried earlier (`parent_previously_carried`: history, not proof of call-stack
-- nesting, which a host callback cannot see), whether a carry was directly followed by a campaign
-- `place` of the same child (a write the host would not have vouched for), and checks each answer against
-- the step it answers: a position carry lands on the parent's new position; a helicopter carry
-- keeps its height and horizontal distance from the parent. A refused carry is counted and raised
-- again, never swallowed. A run with no carry at all proves nothing and fails: it is not acceptance.
--
-- hours=<simulated hours>  stop=<n>  stop after n carries of each shape (default: run the hours)

local args = host.args
local root = assert (args.root, "root=<installation root>")
local frame_ms = tonumber (args.frame_ms or "100")
local frames = math.floor (tonumber (args.hours or "0.25") * 3600000 / frame_ms + 0.5)
local stop_after = tonumber (args.stop or "0")
local dc = require ("eech_dc")
dc.prepare_installation (root)

local kind_names = { [1] = "routed", [2] = "ship", [3] = "person", [4] = "fixed_wing", [5] = "helicopter" }
local reference_bind = host.bind_mobile
local reference_move = host.move_vehicle
local kind_of, carried_child = {}, {}
local carries, refused, previously_carried, places_after, trace = {}, 0, 0, 0, {}
local worst_position, worst_horizontal, worst_height = 0, 0, 0
local last_carry_child
local elapsed = 0

local function bind_mobile (descriptor)
	local id = reference_bind (descriptor)
	if descriptor.movement_kind then kind_of [id] = kind_names [descriptor.movement_kind] end
	return id
end

local function count (key) carries [key] = (carries [key] or 0) + 1 end

local function horizontal (a, b) return math.sqrt ((a [1] - b [1]) ^ 2 + (a [3] - b [3]) ^ 2) end

local modes = {}

local function move_vehicle (step)
	local seen = string.format ("%s/%s", step.mode, kind_of [step.host_id] or "unbound")
	modes [seen] = (modes [seen] or 0) + 1
	if step.mode ~= "dependent_carry" then
		if step.mode == "place" and last_carry_child == step.host_id then places_after = places_after + 1 end
		return reference_move (step)
	end
	local child, parent = step.host_id, step.dc_parent
	local shape = step.dc_helicopter and "helicopter" or "position"
	local ok, result = pcall (reference_move, step)
	if not ok then
		refused = refused + 1
		error (result, 0)
	end
	count (string.format ("%s<-%s %s", kind_of [child] or "unbound", kind_of [parent] or "unbound", shape))
	if carried_child [parent] then previously_carried = previously_carried + 1 end
	carried_child [child] = true
	last_carry_child = child
	local before, after = step.pose.position, result.position
	if shape == "position" then
		worst_position = math.max (worst_position, math.abs (after [1] - step.dc_parent_new [1]), math.abs (after [2] - step.dc_parent_new [2]), math.abs (after [3] - step.dc_parent_new [3]))
	else
		worst_horizontal = math.max (worst_horizontal, math.abs (horizontal (before, step.dc_parent_old) - horizontal (after, step.dc_parent_new)))
		worst_height = math.max (worst_height, math.abs ((before [2] - step.dc_parent_old [2]) - (after [2] - step.dc_parent_new [2])))
	end
	if #trace < 12 then
		trace [#trace + 1] = string.format ("t=%.1f child=%s(%s) parent=%s(%s) %s parent_move=%.3f child_move=%.3f heading=%.4f parent_previously_carried=%s",
			elapsed, child, kind_of [child] or "unbound", parent, kind_of [parent] or "unbound", shape,
			horizontal (step.dc_parent_old, step.dc_parent_new), horizontal (before, after), step.dc_heading, tostring (carried_child [parent] == true))
	end
	return result
end

local engine = dc.boot ({
	install_root = root,
	map = "..\\common\\maps\\map5",
	terrain = host.terrain_provider (root, "..\\common\\maps\\map5"), environment = host.env_provider (),
	campaign_directory = "camp01",
	campaign = "LEBANON.CHC",
	gunship = "apache",
	seed = 1,
	bind_mobile = bind_mobile,
	move_vehicle = move_vehicle,
	observe_loss = host.observe_loss,
	supply_reached = host.supply_reached,
	line_of_sight = host.line_of_sight,
	sense = host.sense, aim = host.aim,
})

for _ = 1, frames do
	engine:frame (frame_ms)
	elapsed = elapsed + frame_ms / 1000
	if stop_after > 0 then
		local done = true
		for _, shape in ipairs ({ "position", "helicopter" }) do
			local total = 0
			for key, n in pairs (carries) do if key:sub (-#shape) == shape then total = total + n end end
			if total < stop_after then done = false end
		end
		if done then break end
	end
end

local keys, total = {}, 0
for key, n in pairs (carries) do keys [#keys + 1] = key; total = total + n end
table.sort (keys)
for _, key in ipairs (keys) do print (string.format ("carry\t%s\t%d", key, carries [key])) end
for _, line in ipairs (trace) do print ("trace\t" .. line) end
local mode_keys = {}
for key in pairs (modes) do mode_keys [#mode_keys + 1] = key end
table.sort (mode_keys)
for _, key in ipairs (mode_keys) do print (string.format ("steps\t%s\t%d", key, modes [key])) end
print (string.format ("carries\t%d", total))
print (string.format ("refused\t%d", refused))
print (string.format ("parent_previously_carried\t%d", previously_carried))
print (string.format ("place_after_carry\t%d", places_after))
print (string.format ("worst_position_error\t%.6f", worst_position))
print (string.format ("worst_helicopter_horizontal_error\t%.6f", worst_horizontal))
print (string.format ("worst_helicopter_height_error\t%.6f", worst_height))
print (string.format ("ended_at\t%.1f", elapsed))
assert (refused == 0, "the host refused a carry the campaign sent")
assert (worst_position < 0.01 and worst_horizontal < 0.05 and worst_height < 0.01, "a carry answer disagrees with the step it answers")
if total == 0 then
	print ("result\tfail_no_carry")
	error ("no dependent_carry step occurred: this run is not acceptance evidence", 0)
end
print ("result\tok")
