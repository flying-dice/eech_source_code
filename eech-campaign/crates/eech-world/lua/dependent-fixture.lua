-- TEST-ONLY controlled fixture for host-owned movement-dependent positioning (EEDCS-40, review R2).
-- Needs a DLL built with the `dependent-fixture` feature; the shipping module has no such method.
--
-- The fixture does not emulate any handler. It re-links *existing, hosted* entities of the booted
-- Lebanon campaign into the original movement-dependent list (`engine:dependent_fixture("link")`),
-- then makes the original C code do the rest: a parent position setter (`"move"`) or the original
-- ship_vehicle_death_movement (`"sink"`) notifies each dependent through ENTITY_MESSAGE_SET_ENTITY_POSITION
-- and the original child handlers send the `dependent_carry` steps recorded here. The reference host
-- answers each one; this script only records the steps and the host's own pose per host id.
--
-- Scenarios (each fails if its expected carry never happens: zero carries is non-acceptance):
--   position    a routed/person/AA child under a parent: carry lands on the parent's new position
--   helicopter  a helicopter child: horizontal and height offset kept, heading = parent heading
--   chain       grandchild under that child: the child's own setter notifies the grandchild while the
--               child is being applied from the host result. A campaign `place` of the child after the
--               grandchild's carry means the apply guard was cleared early (the old boolean guard)
--   sink        original ship_vehicle_death_movement moves the ship from the host (apply route); the
--               dependents are carried against the host's previous/current position
-- After each scenario the host's pose of every involved mobile is compared with the DLL's (objects()).
--
-- warmup=<frames>  frames of the campaign before arranging (default 30)   frame_ms=<ms> (default 100)

-- unbuffered: a native exit (debug_fatal) must not swallow the checks already printed
io.stdout:setvbuf ("no")

local args = host.args
local root = assert (args.root, "root=<installation root>")
local frame_ms = tonumber (args.frame_ms or "100")
local warmup = tonumber (args.warmup or "30")
local dc = require ("eech_dc")
dc.prepare_installation (root)

local kind_names = { [1] = "routed", [2] = "ship", [3] = "person", [4] = "fixed_wing", [5] = "helicopter" }
local reference_bind = host.bind_mobile
local reference_move = host.move_vehicle
local kind_of, host_pose, entity_of = {}, {}, {}
local log, refused = {}, 0

local function copy3 (v) return { v [1], v [2], v [3] } end

local function bind_mobile (descriptor)
	local id = reference_bind (descriptor)
	if descriptor.movement_kind then kind_of [id] = kind_names [descriptor.movement_kind] end
	if descriptor.pose then host_pose [id] = copy3 (descriptor.pose.position) end
	entity_of [id] = descriptor.entity_id
	return id
end

local function move_vehicle (step)
	local entry = { mode = step.mode, id = step.host_id }
	log [#log + 1] = entry
	if step.mode == "dependent_carry" then
		entry.parent = step.dc_parent
		entry.helicopter = step.dc_helicopter
		entry.parent_old, entry.parent_new = copy3 (step.dc_parent_old), copy3 (step.dc_parent_new)
		entry.heading = step.dc_heading
		entry.before = copy3 (step.pose.position)
		local ok, result = pcall (reference_move, step)
		if not ok then
			refused = refused + 1
			entry.refused = tostring (result)
			error (result, 0)
		end
		entry.after = copy3 (result.position)
		host_pose [step.host_id] = copy3 (result.position)
		return result
	end
	local result = reference_move (step)
	if step.mode ~= "retire" and result.position then host_pose [step.host_id] = copy3 (result.position) end
	return result
end

local engine = dc.boot ({
	install_root = root,
	map = "..\\common\\maps\\map5",
	campaign_directory = "camp01",
	campaign = "LEBANON.CHC",
	gunship = "apache",
	seed = 1,
	bind_mobile = bind_mobile,
	move_vehicle = move_vehicle,
	observe_loss = host.observe_loss,
	supply_reached = host.supply_reached,
	line_of_sight = host.line_of_sight,
	sense = host.sense,
})
for _ = 1, warmup do engine:frame (frame_ms) end

local failures, checks = {}, 0
local function check (name, ok, detail)
	checks = checks + 1
	print (string.format ("check\t%s\t%s%s", name, ok and "pass" or "FAIL", detail and ("\t" .. detail) or ""))
	if not ok then failures [#failures + 1] = name end
end

local function horizontal (a, b) return math.sqrt ((a [1] - b [1]) ^ 2 + (a [3] - b [3]) ^ 2) end
local function close3 (a, b, tol) return math.abs (a [1] - b [1]) <= tol and math.abs (a [2] - b [2]) <= tol and math.abs (a [3] - b [3]) <= tol end
local function angle_diff (a, b) local d = (a - b) % (2 * math.pi); if d > math.pi then d = d - 2 * math.pi end; return math.abs (d) end

-- hosted, present, live mobiles: host id -> { entity, kind, host_kind }
local function hosted ()
	local by_entity = {}
	for _, object in ipairs (engine:objects ()) do by_entity [object.id] = object end
	local found = {}
	for _, member in ipairs (engine:members ()) do
		local id = member.host_id
		local object = id and member.physical_present and by_entity [member.current_entity_id]
		if object and object.alive and kind_of [id] and host_pose [id] then
			found [#found + 1] = { host_id = id, entity = object.id, object_kind = object.kind, host_kind = kind_of [id], heading = object.heading }
		end
	end
	table.sort (found, function (a, b) return a.entity < b.entity end)
	return found, by_entity
end

local function dll (entity) for _, object in ipairs (engine:objects ()) do if object.id == entity then return object end end end

local function pick (list, host_kind, used)
	for _, candidate in ipairs (list) do
		if candidate.host_kind == host_kind and not used [candidate.entity] then used [candidate.entity] = true; return candidate end
	end
end

local list = hosted ()
local census = {}
for _, candidate in ipairs (list) do census [candidate.host_kind] = (census [candidate.host_kind] or 0) + 1 end
for kind, n in pairs (census) do print (string.format ("hosted\t%s\t%d", kind, n)) end

local used = {}
local parent = pick (list, "ship", used)
local parent_kind = "ship"
if not parent then
	parent = pick (list, "routed", used)
	parent_kind = "routed"
	print ("note\tno hosted ship: the sink scenario is not exercised")
end
local routed = pick (list, "routed", used) or pick (list, "person", used)
local helicopter = pick (list, "helicopter", used)
local grandchild = pick (list, "person", used) or pick (list, "routed", used)
check ("candidates", parent and routed and helicopter and grandchild,
	string.format ("parent=%s(%s) routed=%s helicopter=%s grandchild=%s", parent and parent.entity, parent_kind, routed and routed.entity, helicopter and helicopter.entity, grandchild and grandchild.entity))
if not (parent and routed and helicopter and grandchild) then
	print ("result\tfail")
	error ("fixture candidates are missing; nothing was arranged", 0)
end

local function carries_in (from)
	local out = {}
	for index = from, #log do if log [index].mode == "dependent_carry" then out [#out + 1] = log [index] end end
	return out
end

-- a campaign `place` of a carried child after its carry in the same operation: the host was asked to
-- vouch for a position it had itself just produced (the early-cleared guard)
local function spurious_places (from)
	local carried, count = {}, 0
	for index = from, #log do
		local entry = log [index]
		if entry.mode == "dependent_carry" then carried [entry.id] = true
		elseif entry.mode == "place" and carried [entry.id] then count = count + 1 end
	end
	return count
end

local function agreement (name, member)
	local object = dll (member.entity)
	local pose = host_pose [member.host_id]
	check (name .. ".host_dll_agree", object and close3 (object.position, pose, 0.01),
		object and string.format ("dll=(%.3f,%.3f,%.3f) host=(%.3f,%.3f,%.3f)", object.position [1], object.position [2], object.position [3], pose [1], pose [2], pose [3]) or "no dll object")
end

local function link (p, c) engine:dependent_fixture ("link", p.entity, c.entity) end

local function move_parent (dx, dz)
	local object = dll (parent.entity)
	local to = { object.position [1] + dx, object.position [2], object.position [3] + dz }
	local from = #log + 1
	engine:dependent_fixture ("move", parent.entity, 0, to [1], to [2], to [3])
	return from, to
end

-- position
do
	link (parent, routed)
	local before = copy3 (host_pose [routed.host_id])
	local from, to = move_parent (50, 30)
	local carries = carries_in (from)
	local mine, others = nil, 0
	for _, carry in ipairs (carries) do
		if carry.id == routed.host_id then mine = carry else others = others + 1 end
	end
	print (string.format ("position.campaign_dependents_also_carried	%d", others))
	check ("position.carried", mine ~= nil and mine.parent == parent.host_id and not mine.helicopter, string.format ("%d carries, %d other", #carries, others))
	if mine then
		check ("position.lands_on_parent_new", close3 (mine.after, mine.parent_new, 0.01) and close3 (mine.parent_new, to, 0.01))
		check ("position.moved", horizontal (before, mine.after) > 1)
	end
	check ("position.no_place_after_carry", spurious_places (from) == 0, tostring (spurious_places (from)))
	agreement ("position.child", routed)
end

-- helicopter
do
	link (parent, helicopter)
	local before = copy3 (host_pose [helicopter.host_id])
	local parent_before = copy3 (host_pose [parent.host_id])
	local from, to = move_parent (-35, 45)
	local carries = carries_in (from)
	local mine
	for _, carry in ipairs (carries) do if carry.id == helicopter.host_id then mine = carry end end
	check ("helicopter.carried", mine ~= nil and mine.helicopter == true and mine.parent == parent.host_id, string.format ("%d carries", #carries))
	if mine then
		local after = mine.after
		local keep_h = math.abs (horizontal (before, parent_before) - horizontal (after, to))
		local keep_y = math.abs ((before [2] - parent_before [2]) - (after [2] - to [2]))
		check ("helicopter.horizontal_offset_kept", keep_h < 0.05, string.format ("%.6f", keep_h))
		check ("helicopter.height_offset_kept", keep_y < 0.01, string.format ("%.6f", keep_y))
		local object, parent_object = dll (helicopter.entity), dll (parent.entity)
		check ("helicopter.heading_is_parent_heading", angle_diff (object.heading, parent_object.heading) < 0.01,
			string.format ("helicopter=%.4f parent=%.4f", object.heading, parent_object.heading))
	end
	check ("helicopter.no_place_after_carry", spurious_places (from) == 0, tostring (spurious_places (from)))
	agreement ("helicopter.child", helicopter)
end

-- helicopter under a parent whose heading differs from the helicopter's: the offset is rotated by the
-- difference, so the check is against the handler's own formula as well as the preserved distance
do
	local second
	for _, candidate in ipairs (list) do
		if candidate.host_kind == "routed" and not used [candidate.entity] and math.abs (candidate.heading) > 0.3 then second = candidate; used [candidate.entity] = true; break end
	end
	check ("rotated.candidate", second ~= nil, second and string.format ("parent %s heading %.3f", second.entity, second.heading) or "no routed vehicle with a heading")
	if second then
		engine:dependent_fixture ("link", second.entity, helicopter.entity)
		local old_child, old_parent = copy3 (host_pose [helicopter.host_id]), copy3 (host_pose [second.host_id])
		local object, parent_object = dll (helicopter.entity), dll (second.entity)
		local angle = parent_object.heading - object.heading
		local to = { old_parent [1] + 25, old_parent [2], old_parent [3] - 40 }
		local from = #log + 1
		engine:dependent_fixture ("move", second.entity, 0, to [1], to [2], to [3])
		local mine
		for _, carry in ipairs (carries_in (from)) do if carry.id == helicopter.host_id then mine = carry end end
		check ("rotated.angle_is_meaningful", math.abs (angle) > 0.1, string.format ("%.4f rad", angle))
		check ("rotated.carried", mine ~= nil and mine.helicopter == true and mine.parent == second.host_id)
		if mine then
			local dx, dy, dz = old_child [1] - old_parent [1], old_child [2] - old_parent [2], old_child [3] - old_parent [3]
			local expect = { to [1] + math.cos (angle) * dx + math.sin (angle) * dz, to [2] + dy, to [3] + math.cos (angle) * dz - math.sin (angle) * dx }
			check ("rotated.matches_handler_formula", close3 (mine.after, expect, 0.05),
				string.format ("host=(%.3f,%.3f,%.3f) formula=(%.3f,%.3f,%.3f)", mine.after [1], mine.after [2], mine.after [3], expect [1], expect [2], expect [3]))
			check ("rotated.distance_kept", math.abs (horizontal (mine.after, to) - horizontal (old_child, old_parent)) < 0.05)
			check ("rotated.offset_was_turned", math.abs (mine.after [1] - to [1] - dx) + math.abs (mine.after [3] - to [3] - dz) > 0.5)
			check ("rotated.heading_is_parent_heading", angle_diff (dll (helicopter.entity).heading, parent_object.heading) < 0.01,
				string.format ("helicopter=%.4f parent=%.4f", dll (helicopter.entity).heading, parent_object.heading))
		end
		check ("rotated.no_place_after_carry", spurious_places (from) == 0, tostring (spurious_places (from)))
		agreement ("rotated.child", helicopter)
		-- back under the ship for the sink scenario
		-- (and brought next to it, so the later rotation acts on a short offset rather than leaving the map)
		engine:dependent_fixture ("link", parent.entity, helicopter.entity)
		local at = host_pose [parent.host_id]
		engine:dependent_fixture ("move", helicopter.entity, 0, at [1] + 40, at [2] + 300, at [3] + 40)
	end
end

-- chain: grandchild under the routed child that is itself carried
do
	link (routed, grandchild)
	local from, to = move_parent (20, -60)
	local carries = carries_in (from)
	local order = {}
	for _, carry in ipairs (carries) do order [#order + 1] = carry.id .. "<-" .. carry.parent end
	print ("chain.order\t" .. table.concat (order, " "))
	local child_carry, grand_carry
	for index, carry in ipairs (carries) do
		if carry.id == routed.host_id then child_carry = child_carry or index end
		if carry.id == grandchild.host_id then grand_carry = grand_carry or index end
	end
	check ("chain.child_then_grandchild", child_carry and grand_carry and child_carry < grand_carry and carries [grand_carry].parent == routed.host_id)
	if grand_carry then check ("chain.grandchild_on_child_new", close3 (carries [grand_carry].parent_new, carries [child_carry].after, 0.01)) end
	check ("chain.no_place_after_carry", spurious_places (from) == 0, tostring (spurious_places (from)))
	agreement ("chain.child", routed)
	agreement ("chain.grandchild", grandchild)
	-- the guard is restored: after the nested application the parent's own campaign place still reaches
	-- the host (a leaked guard would swallow it) and a second move carries the whole chain again
	local again = move_parent (5, 5)
	local placed_parent = false
	for index = again, #log do if log [index].mode == "place" and log [index].id == parent.host_id then placed_parent = true end end
	check ("guard.restored_after_chain", placed_parent and #carries_in (again) >= 2 and spurious_places (again) == 0)
end

-- sink: the original ship_vehicle_death_movement, host-applied
if parent_kind == "ship" then
	local from = #log + 1
	engine:dependent_fixture ("sink", parent.entity, 0)
	local carries = carries_in (from)
	check ("sink.carried", #carries >= 1, string.format ("%d carries", #carries))
	local stray = 0
	for _, carry in ipairs (carries) do
		-- the parent is the ship, or the fixture's routed child (the grandchild's parent)
		if carry.parent ~= parent.host_id and carry.parent ~= routed.host_id then stray = stray + 1 end
	end
	check ("sink.carry_parents_are_ship_or_chain", stray == 0, tostring (stray))
	check ("sink.no_place_after_carry", spurious_places (from) == 0, tostring (spurious_places (from)))
	agreement ("sink.child", routed)
	agreement ("sink.helicopter", helicopter)
	agreement ("sink.grandchild", grandchild)
	agreement ("sink.parent", parent)
end

local total = 0
for _, entry in ipairs (log) do if entry.mode == "dependent_carry" then total = total + 1 end end
print (string.format ("carries\t%d", total))
print (string.format ("refused\t%d", refused))
print (string.format ("checks\t%d\tfailed\t%d", checks, #failures))
assert (total > 0, "no dependent_carry step occurred: not acceptance")
assert (refused == 0, "the host refused a carry the campaign sent")
if #failures > 0 then
	print ("result\tfail\t" .. table.concat (failures, ","))
	error ("fixture checks failed: " .. table.concat (failures, ", "), 0)
end
print ("result\tok")
