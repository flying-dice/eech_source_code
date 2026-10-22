-- Public causal check for host-owned live AI helicopter movement: the host owns where a live
-- AI helicopter is, so a host that does not move one keeps the campaign from seeing it move,
-- and a host that moves it again lets the campaign advance it.
--
-- A helicopter step is two host calls around the campaign's attitude guidance: helicopter_begin
-- (the motion vector, the speed and whether it moved) then helicopter_end (the attitude, the
-- position and the floor clamp). This witness covers only that live, alive-state integration. The
-- rotor and the dying, fall and crash movement are covered separately by helicopter-lifecycle.lua;
-- the guidance, the look-ahead, the avoidance and the pilot control are not covered by either and
-- remain DLL-side physical dependencies. Only the AI helicopters the headless campaign updates
-- through the server path are exercised.
--
-- mode=census   the reference host moves everything. Counts the members the campaign bound per
--               movement kind, the host steps by kind and mode, the helicopter placements the
--               campaign announced (setters, death and crash writers) and the helicopters the host
--               moved at least 10 m. Nothing is asserted except that every helicopter begin is
--               followed by its end. mode=hold prints the same counts.
-- mode=control  the reference host moves everything. The subject is the first helicopter the host
--               moves at least 10 m: its identity and when it was selected.
-- mode=hold     select=<host id> from the control run. From its first helicopter_begin step the host
--               declines to move that member until hold_seconds later: begin answers the vector it
--               already holds, not moved, pending; end answers the pose it already holds. Every
--               other member is moved by the reference host. The pose the campaign hands the host for
--               that member on each begin is the campaign's own view of it: it must not change while
--               the member is held and must change after the host moves it again.
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

local kind_names = { [1] = "routed", [2] = "ship", [3] = "person", [4] = "fixed_wing", [5] = "helicopter" }

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
local open_begin = {}
local pairing_errors, paired, moved_begins = 0, 0, 0
local function count (name) steps [name] = (steps [name] or 0) + 1 end

-- control and hold
local selected_host, selected_at
local held_steps, moved_steps = 0, 0
local released_at
local held_pair = false
local first_held, last_held, after_release_first, after_release_last

local function holding () return mode == "hold" and selected_at and elapsed - selected_at < hold_seconds end

local function move_vehicle (step)
	local mover_kind = kind_of [step.host_id]
	count ((mover_kind or "unbound") .. ":" .. step.mode)
	local begin_step = mover_kind == "helicopter" and step.mode == "helicopter_begin"
	local end_step = mover_kind == "helicopter" and step.mode == "helicopter_end"
	if begin_step then
		if open_begin [step.host_id] then pairing_errors = pairing_errors + 1 end
		open_begin [step.host_id] = true
	elseif end_step then
		if open_begin [step.host_id] then paired = paired + 1 else pairing_errors = pairing_errors + 1 end
		open_begin [step.host_id] = nil
	end
	if mode == "hold" and not selected_at and step.host_id == subject and begin_step then
		selected_host, selected_at = subject, elapsed
	end
	if mode == "hold" and step.host_id == selected_host then
		-- the end of a held begin is held with it, so a pair is never split across the release
		if begin_step then held_pair = holding () and true or false end
		if (begin_step or end_step) and held_pair then
			if begin_step then
				held_steps = held_steps + 1
				first_held = first_held or copy (step.pose.position)
				last_held = copy (step.pose.position)
			end
			-- the host does not move it: the pose and the motion vector it holds are the answer
			return {
				position = step.pose.position,
				velocity = step.pose.velocity,
				attitude = step.pose.attitude,
				motion_vector = step.motion_vector_in,
				rotated = false,
				moved = false,
				pending = begin_step,
			}
		end
		if begin_step then
			released_at = released_at or elapsed
			moved_steps = moved_steps + 1
			after_release_first = after_release_first or copy (step.pose.position)
			after_release_last = copy (step.pose.position)
		end
	end
	local result = reference_move (step)
	if begin_step then
		if result.moved then moved_begins = moved_begins + 1 end
		first_pose [step.host_id] = first_pose [step.host_id] or copy (step.pose.position)
	elseif end_step then
		if first_pose [step.host_id] and distance (first_pose [step.host_id], result.position) >= 10 then
			moved_far [step.host_id] = true
		end
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
	for _, name in ipairs ({ "routed", "ship", "person", "fixed_wing", "helicopter" }) do
		print (string.format ("census\t%s\tbound=%d\tmoved_10m=%d", name, bound [name] or 0, far [name] or 0))
	end
	local names = {}
	for name in pairs (steps) do names [#names + 1] = name end
	table.sort (names)
	for _, name in ipairs (names) do print (string.format ("steps\t%s\t%d", name, steps [name])) end
	print (string.format ("pairing_errors\t%d", pairing_errors))
	-- placements the campaign announces (setters, death, crash, pose writers) are the declared transitional writers: counted, never silent
	print (string.format ("helicopter_pairs\t%d\tmoved_begins=%d\tplaces=%d", paired, moved_begins, steps ["helicopter:place"] or 0))
	print (string.format ("ended_at\t%.1f", elapsed))
end

assert (pairing_errors == 0, "a helicopter_begin was not followed by exactly one helicopter_end")
assert ((steps ["helicopter:helicopter_begin"] or 0) - (steps ["helicopter:helicopter_end"] or 0) <= 1, "helicopter begin and end counts diverge")

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

assert (held_steps > 1, "the selected helicopter was held for only one step")
assert (released_at, "the held helicopter was never released")
assert (distance (first_held, last_held) == 0, "the held helicopter moved in the campaign while the host held it")
assert (moved_steps > 1 and distance (after_release_first, after_release_last) > 0, "the released helicopter did not move")
print_census ()
print ("select\thold")
print ("host_id\t" .. selected_host)
print (string.format ("held_seconds\t%.1f", released_at - selected_at))
print (string.format ("held_steps\t%d", held_steps))
print (string.format ("held_displacement\t%.3f", distance (first_held, last_held)))
print (string.format ("displacement_after_release\t%.3f", distance (after_release_first, after_release_last)))
print ("result\tok")
