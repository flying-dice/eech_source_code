-- Boot-only witness (outside the repository): does the DLL apply the position the host answers to a
-- create_place step? Host Lua -> eech_dc -> move_vehicle(create_place) -> entity heap -> engine:objects().
-- The first routed member's answer is replaced by the reference answer shifted by (+250, 0, -125) m.
-- The reference host's state is kept consistent through its own public "place" step with the shifted pose.
-- Every other create_place answer is the unmodified reference answer (control). No frame is run.
-- Position is read only from engine:objects(); the host id is joined through engine:members().current_entity_id.

local args = host.args
local root = assert (args.root, "root=<installation root>")
local dc = require ("eech_dc")
dc.prepare_installation (root)

local kind_names = { [1] = "routed", [2] = "ship", [3] = "person" }
local reference_bind, reference_move = host.bind_mobile, host.move_vehicle
local kind_of, answered, input_of = {}, {}, {}
local subject, calls, order = nil, {}, {}

local function bind_mobile (descriptor)
	local id = reference_bind (descriptor)
	if descriptor.movement_kind then kind_of [id] = kind_names [descriptor.movement_kind] end
	return id
end

local function copy3 (p) return { p [1], p [2], p [3] } end

local function move_vehicle (step)
	calls [step.mode] = (calls [step.mode] or 0) + 1
	local result = reference_move (step)
	if step.mode == "create_place" then
		input_of [step.host_id] = copy3 (step.pose.position)
		if not subject and kind_of [step.host_id] == "routed" then
			subject = step.host_id
			local shifted = { result.position [1] + 250, result.position [2], result.position [3] - 125 }
			-- host mutation: the reference host adopts the shifted pose through its public place operation
			reference_move { host_id = step.host_id, mode = "place",
				pose = { position = shifted, velocity = result.velocity, attitude = result.attitude },
				motion_vector_in = { 0, 0, 0 } }
			result.position = shifted
		end
		answered [step.host_id] = copy3 (result.position)
		order [#order + 1] = step.host_id
	end
	return result
end

local engine = dc.boot {
	install_root = root,
	map = "..\common\maps\map5",
	campaign_directory = "camp01",
	campaign = "LEBANON.CHC",
	gunship = "apache",
	seed = 1,
	bind_mobile = bind_mobile,
	observe_loss = host.observe_loss,
	supply_reached = host.supply_reached,
	move_vehicle = move_vehicle,
}

for mode, n in pairs (calls) do print (string.format ("steps\t%s\t%d", mode, n)) end
print ("create_place_answers\t" .. #order)
if not subject then
	print ("subject\tnone")
	print ("result\tunavailable")
	return
end

local entity_of = {}
for _, member in ipairs (engine:members ()) do
	if member.host_id and member.current_entity_id then entity_of [member.host_id] = member.current_entity_id end
end
local object_at = {}
for _, object in ipairs (engine:objects ()) do object_at [object.id] = object end

local function distance (a, b)
	local dx, dy, dz = a [1] - b [1], a [2] - b [2], a [3] - b [3]
	return math.sqrt (dx * dx + dy * dy + dz * dz)
end

local checked, matched, worst = 0, 0, 0
for _, id in ipairs (order) do
	local entity = entity_of [id]
	local object = entity and object_at [entity]
	if object and id ~= subject then
		checked = checked + 1
		local d = distance (object.position, answered [id])
		worst = math.max (worst, d)
		if d < 0.01 then matched = matched + 1 end
	end
end
local entity = entity_of [subject]
local object = entity and object_at [entity]
print ("subject\t" .. subject)
print ("subject_entity\t" .. tostring (entity))
assert (object, "the subject is not observable through engine:objects()")
local input, answer, seen = input_of [subject], answered [subject], object.position
local fmt = "%.3f,%.3f,%.3f"
print ("subject_dll_input\t" .. string.format (fmt, input [1], input [2], input [3]))
print ("subject_host_answer\t" .. string.format (fmt, answer [1], answer [2], answer [3]))
print ("subject_objects_position\t" .. string.format (fmt, seen [1], seen [2], seen [3]))
print (string.format ("subject_answer_vs_input_m\t%.3f", distance (answer, input)))
print (string.format ("subject_objects_vs_answer_m\t%.3f", distance (seen, answer)))
print (string.format ("control\tchecked=%d\tmatched_0.01m=%d\tworst_m=%.4f", checked, matched, worst))
assert (distance (answer, input) > 100, "the shifted answer is not different from the DLL input")
assert (distance (seen, answer) < 0.01, "engine:objects() does not show the position the host answered")
print ("result\tok")
