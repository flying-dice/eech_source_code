-- Public causal check for host-owned live fixed-wing movement: the host owns where a live
-- fixed-wing aircraft is, so a host that does not move one keeps the campaign from seeing it
-- move, and a host that moves it again lets the campaign advance it.
--
-- mode=census   the reference host moves everything. Counts the members the campaign bound per
--               movement kind, the host steps by mode and (fixed wing) by phase, and the fixed-wing
--               members the host moved at least 10 m. Nothing is asserted. mode=hold prints the same
--               counts, so the hold run carries its own coverage.
-- mode=control  the reference host moves everything. The subject is the first fixed-wing member the
--               host moves at least 10 m: its identity and when it was selected.
-- mode=hold     select=<host id> from the control run. From its first fixed_wing_move step the host
--               declines to move that member until hold_seconds later (it answers with the pose it
--               already holds, calling no step). Every other member is moved by the reference host.
--               The pose the campaign hands the host for that member on each step is the campaign's
--               own view of it: it must not change while the member is held and must change after
--               the host moves it again.
-- mode=missing  the campaign refuses to start without a host move_vehicle callback.

local args = host.args
local root = assert (args.root, "root=<installation root>")
local mode = assert (args.mode, "mode=census|control|hold|missing")
local frame_ms = tonumber (args.frame_ms or "100")
local frames = math.floor (tonumber (args.hours or "1") * 3600000 / frame_ms + 0.5)
local hold_seconds = tonumber (args.hold_seconds or "0")
local subject = args.select
local dc = require ("eech_dc")
dc.prepare_installation (root)
host.use_reference_arithmetic () -- the reference host's arithmetic profile, selected right after the installation is prepared (src/reference_arithmetic.rs)

local kind_names = { [1] = "routed", [2] = "ship", [3] = "person", [4] = "fixed_wing" }

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
	line_of_sight = host.line_of_sight, model = host.model,
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
assert (mode == "census" or mode == "control" or mode == "hold", "mode=census|control|hold|missing")
assert (mode ~= "hold" or subject, "select=<host id>")

local function copy (position) return { position [1], position [2], position [3] } end

local function distance (a, b)
	local dx, dy, dz = a [1] - b [1], a [2] - b [2], a [3] - b [3]
	return math.sqrt (dx * dx + dy * dy + dz * dz)
end

-- census
local steps = {}
local first_pose, moved_far = {}, {}
local function count (name) steps [name] = (steps [name] or 0) + 1 end

-- control and hold
local selected_host, selected_at
local held_steps, moved_steps = 0, 0
local released_at
local first_held, last_held, after_release_first, after_release_last

local function holding () return mode == "hold" and selected_at and elapsed - selected_at < hold_seconds end

local function move_vehicle (step)
	local mover_kind = kind_of [step.host_id]
	if step.mode ~= "place" then count ((mover_kind or "unbound") .. ":" .. step.mode) end
	if mover_kind == "fixed_wing" and step.mode == "fixed_wing_move" then count ("fixed_wing_phase:" .. tostring (step.phase)) end
	local subject_step = mover_kind == "fixed_wing" and step.mode == "fixed_wing_move"
	if mode == "hold" and not selected_at and step.host_id == subject and subject_step then
		selected_host, selected_at = subject, elapsed
	end
	if mode == "hold" and step.host_id == selected_host and subject_step then
		if holding () then
			held_steps = held_steps + 1
			first_held = first_held or copy (step.pose.position)
			last_held = copy (step.pose.position)
			-- the host does not move it: the pose and the motion vector it holds are the answer
			return {
				position = step.pose.position,
				velocity = step.pose.velocity,
				attitude = step.pose.attitude,
				motion_vector = step.motion_vector_in,
				rotated = false,
			}
		end
		released_at = released_at or elapsed
		moved_steps = moved_steps + 1
		after_release_first = after_release_first or copy (step.pose.position)
		after_release_last = copy (step.pose.position)
	end
	local result = reference_move (step)
	if subject_step then
		first_pose [step.host_id] = first_pose [step.host_id] or copy (step.pose.position)
		if distance (first_pose [step.host_id], result.position) >= 10 then moved_far [step.host_id] = true end
		if mode == "control" and not selected_host and moved_far [step.host_id] then
			selected_host, selected_at = step.host_id, elapsed
		end
	end
	return result
end

config.move_vehicle = move_vehicle
local engine = dc.boot (config)

for _ = 1, frames do
	engine:frame (frame_ms)
	elapsed = elapsed + frame_ms / 1000
	if mode == "control" and selected_host then break end
	if mode == "hold" and released_at and moved_steps > 1 and distance (after_release_first, after_release_last) > 0 then break end
end

local function print_census ()
	local bound, far = {}, {}
	for id, name in pairs (kind_of) do
		bound [name] = (bound [name] or 0) + 1
		if moved_far [id] then far [name] = (far [name] or 0) + 1 end
	end
	for _, name in ipairs ({ "routed", "ship", "person", "fixed_wing" }) do
		print (string.format ("census\t%s\tbound=%d\tmoved_10m=%d", name, bound [name] or 0, far [name] or 0))
	end
	local names = {}
	for name in pairs (steps) do names [#names + 1] = name end
	table.sort (names)
	for _, name in ipairs (names) do print (string.format ("steps\t%s\t%d", name, steps [name])) end
	print (string.format ("ended_at\t%.1f", elapsed))
end

if mode == "census" then
	print_census ()
	print ("result\tcensus")
	return
end

if not selected_host then
	print ("selected\tnone")
	print ("result\tunavailable")
	return
end
print (string.format ("diag_selected\tmode=%s host=%s selected_at=%.1f hold_seconds=%.1f", mode, selected_host, selected_at, hold_seconds))
print (string.format ("diag_steps\theld=%d moved=%d released_at=%s ended_at=%.1f", held_steps, moved_steps, tostring (released_at), elapsed))
if mode == "control" then
	print ("subject\t" .. selected_host)
	print (string.format ("selected_at\t%.1f", selected_at))
	print ("result\tcontrol")
	return
end

assert (held_steps > 1, "the selected aircraft was held for only one step")
assert (released_at, "the held aircraft was never released")
assert (distance (first_held, last_held) == 0, "the held aircraft moved in the campaign while the host held it")
assert (moved_steps > 1 and distance (after_release_first, after_release_last) > 0, "the released aircraft did not move")
print_census ()
print ("select\thold")
print ("host_id\t" .. selected_host)
print (string.format ("held_seconds\t%.1f", released_at - selected_at))
print (string.format ("held_steps\t%d", held_steps))
print (string.format ("held_displacement\t%.3f", distance (first_held, last_held)))
print (string.format ("displacement_after_release\t%.3f", distance (after_release_first, after_release_last)))
print ("result\tok")
