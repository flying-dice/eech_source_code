-- Public causal check for host-owned helicopter rotor state: the host owns the rotor (rpm, tail rpm, spin-up timer)
-- of a live AI helicopter and integrates it, so a host that declines to spin a dying helicopter's rotor down keeps
-- the campaign from seeing it change, and a host that answers again lets the campaign see the rotor advance.
--
-- The subject is the retained natural loss of the Lebanon reference run: Mi-17 Hip, legacy entity 87568 of group
-- 87567 (Wolfpack), killed at about 1584.66 s (reference/m3-supply-loss/loss-chain.json). The host id is resolved
-- from the binding the campaign announces (descriptor.entity_id), never assumed equal to the legacy index.
--
-- Lifecycle steps are move_vehicle steps of modes rotor_set, rotor_alive, rotor_down, fall, contact, settle,
-- tumble, crash_begin, crash_end, impact, fall_gravity and rotor_idle. The causal claim of this witness is the
-- rotor hold and release only. The fall, contact, settle, tumble and crash phases the helicopter passes through are
-- reported as coverage (which modes ran, in what order, and what the host answered); the witness does not hold or
-- delay them and does not claim a terminal state. The campaign still decides when a helicopter dies and what the
-- death means (group consequences, removal). Smoke, effects, sounds, terrain sampling, the RNG draws and the
-- dying-state clears remain DLL-side physical dependencies that the host does not own; they are not exercised here.
--
-- mode=control  the reference host answers everything. Prints, for the subject, each lifecycle mode's first time and
--               count, the order of phases, what the host answered for the phases that report an outcome (contact
--               touched, settle buried, crash_end stopped), and the modes whose answers changed the rotor.
-- mode=hold     select=<host id> from the control run. From the subject's first rotor_down step the host declines to
--               spin the rotor down until hold_seconds later: it answers the rotor and motion vector it already
--               holds. Every other step of every member is answered by the reference host. The rotor the campaign
--               hands the host on each held rotor_down is the campaign's own view of it: it must not change while
--               the host holds it, and after the host answers again it must change, with the first rotor-changing
--               answer since the release coming from rotor_down itself (not from another phase such as a settle).
--
-- host.args: root, mode, hours (default 0.65), frame_ms (default 100), legacy (default 87568),
--            expected_death (default 1584.66), select, hold_seconds

local args = host.args
local root = assert (args.root, "root=<installation root>")
local mode = assert (args.mode, "mode=control|hold")
assert (mode == "control" or mode == "hold", "mode=control|hold")
local frame_ms = tonumber (args.frame_ms or "100")
local frames = math.floor (tonumber (args.hours or "0.65") * 3600000 / frame_ms + 0.5)
local legacy = tonumber (args.legacy or "87568")
assert (legacy and legacy == math.floor (legacy) and legacy > 0, "legacy=<positive integer entity index>")
local expected_death = tonumber (args.expected_death or "1584.66")
local hold_seconds = tonumber (args.hold_seconds or "0")
local selected = args.select
assert (mode ~= "hold" or selected, "select=<host id>")
local dc = require ("eech_dc")
dc.prepare_installation (root)

local lifecycle_modes = {
	rotor_set = true, rotor_alive = true, rotor_down = true, fall = true, contact = true, settle = true,
	tumble = true, crash_begin = true, crash_end = true, impact = true, fall_gravity = true, rotor_idle = true,
}
local order = { "rotor_set", "rotor_alive", "rotor_idle", "rotor_down", "fall", "fall_gravity", "contact", "settle", "tumble", "impact", "crash_begin", "crash_end" }

local reference_bind = host.bind_mobile
local reference_move = host.move_vehicle
local reference_reached = host.supply_reached
local reference_loss = host.observe_loss
local elapsed = 0
local subject = nil

local function bind_mobile (descriptor)
	local id = reference_bind (descriptor)
	if descriptor.movement_kind == 5 and descriptor.entity_id == legacy then
		assert (not subject, "two helicopters bound for legacy entity " .. legacy)
		subject = id
		print (string.format ("bound\thost_id=%s legacy=%d entity_type=%s sub_type=%s side=%s group_key=%s", id, legacy, tostring (descriptor.entity_type), tostring (descriptor.sub_type), tostring (descriptor.side), tostring (descriptor.group_key)))
	end
	return id
end

local losses = {}
local function observe_loss (descriptor)
	if descriptor.host_id == subject then
		losses [#losses + 1] = string.format ("%.2f\t%s", elapsed, tostring (descriptor.event))
	end
	return reference_loss (descriptor)
end

-- the subject's own record; the fleet counts are context only
local fleet = {}
local first_at, counts = {}, {}
local sequence = {}
local answered = {}      -- phase outcomes the host reported: name -> { true = n, false = n, first_at = t }
local rotor_changers = {} -- modes whose reference answer changed the rotor the campaign handed in

-- hold
local held_rotor_steps, released_steps = 0, 0
local held_rotor, release_at, hold_started
local held_changed = 0
local last_released_rotor
local first_change_mode, first_change_at

local function holding ()
	return mode == "hold" and hold_started and elapsed - hold_started < hold_seconds
end

local function same (a, b)
	if #a ~= #b then return false end
	for i = 1, #a do if a [i] ~= b [i] then return false end end
	return true
end

local function copy (t)
	local c = {}
	for i = 1, #t do c [i] = t [i] end
	return c
end

-- the one outcome each phase reports in `moved`
local outcome_of = { contact = "touched", settle = "buried", crash_end = "stopped" }

local function move_vehicle (step)
	local is_subject = step.host_id == subject and lifecycle_modes [step.mode]
	if lifecycle_modes [step.mode] then fleet [step.mode] = (fleet [step.mode] or 0) + 1 end
	if not is_subject then return reference_move (step) end

	local name = step.mode
	first_at [name] = first_at [name] or elapsed
	counts [name] = (counts [name] or 0) + 1
	if sequence [#sequence] ~= name then sequence [#sequence + 1] = name end
	if name == "rotor_down" then
		if mode == "hold" and not hold_started then
			assert (step.host_id == selected, "the bound subject " .. tostring (step.host_id) .. " is not select=" .. tostring (selected))
			hold_started = elapsed
			held_rotor = copy (step.rotor)
		end
		if holding () then
			-- the host declines: the rotor and the motion vector it holds are the answer
			held_rotor_steps = held_rotor_steps + 1
			if not same (step.rotor, held_rotor) then held_changed = held_changed + 1 end
			return {
				position = step.pose.position,
				velocity = step.pose.velocity,
				attitude = step.pose.attitude,
				motion_vector = step.motion_vector_in,
				rotor = step.rotor,
				altitude = 0.0,
				rotated = false,
				moved = false,
				pending = false,
			}
		elseif hold_started then
			release_at = release_at or elapsed
			released_steps = released_steps + 1
			last_released_rotor = copy (step.rotor)
		end
	end
	local result = reference_move (step)
	if not same (result.rotor, step.rotor) then
		rotor_changers [name] = (rotor_changers [name] or 0) + 1
		if release_at and not first_change_mode then first_change_mode, first_change_at = name, elapsed end
	end
	local outcome = outcome_of [name]
	if outcome then
		local record = answered [name] or { [true] = 0, [false] = 0 }
		local value = result.moved and true or false
		record [value] = record [value] + 1
		record.first_at = record.first_at or elapsed
		answered [name] = record
	end
	return result
end

local config = {
	install_root = root,
	map = "..\\common\\maps\\map5",
	campaign_directory = "camp01",
	campaign = "LEBANON.CHC",
	gunship = "apache",
	seed = 1,
	bind_mobile = bind_mobile,
	move_vehicle = move_vehicle,
	observe_loss = observe_loss,
	supply_reached = reference_reached,
}
local engine = dc.boot (config)

local settled_at
for _ = 1, frames do
	engine:frame (frame_ms)
	elapsed = elapsed + frame_ms / 1000
	-- enough after the first answered outcome (control) or after the release (hold) to see the phases that follow
	if mode == "control" and not settled_at and (answered.settle or answered.crash_end) then settled_at = elapsed end
	if mode == "control" and settled_at and elapsed - settled_at >= 10 then break end
	if mode == "hold" and release_at and elapsed - release_at >= 10 then break end
end

local function fleet_line ()
	for _, name in ipairs (order) do
		print (string.format ("fleet\t%s\t%d", name, fleet [name] or 0))
	end
end

local function subject_lines ()
	print (string.format ("subject\t%s", tostring (subject)))
	for _, name in ipairs (order) do
		if counts [name] then print (string.format ("lifecycle\t%s\tfirst_at=%.2f\tcount=%d", name, first_at [name], counts [name])) end
	end
	print ("sequence\t" .. table.concat (sequence, ">"))
	for _, name in ipairs (order) do
		local record = answered [name]
		if record then
			print (string.format ("answered\t%s\t%s=true:%d false:%d\tfirst_at=%.2f", name, outcome_of [name], record [true], record [false], record.first_at))
		end
	end
	for _, name in ipairs (order) do
		if rotor_changers [name] then print (string.format ("rotor_changed_by\t%s\t%d", name, rotor_changers [name])) end
	end
	for _, line in ipairs (losses) do print ("loss\t" .. line) end
	print (string.format ("ended_at\t%.1f", elapsed))
end

if not subject then
	print (string.format ("bound\tnone for legacy=%d", legacy))
	print ("result\tunavailable")
	return
end
if not first_at.rotor_down then
	subject_lines ()
	print ("result\tunavailable")
	return
end
print (string.format ("death_at\t%.2f\texpected=%.2f", first_at.rotor_down, expected_death))
assert (math.abs (first_at.rotor_down - expected_death) <= 1.0, "the subject did not start dying at the retained time")

if mode == "control" then
	fleet_line ()
	subject_lines ()
	assert (rotor_changers.rotor_down, "the reference host never spun the dying rotor down")
	print ("result\tcontrol")
	return
end

print (string.format ("diag_hold\tstarted=%.2f released_at=%s held_rotor_steps=%d held_changed=%d released_steps=%d", hold_started, tostring (release_at and string.format ("%.2f", release_at)), held_rotor_steps, held_changed, released_steps))
print (string.format ("held_rotor\t%.6f %.6f %.6f", held_rotor [1], held_rotor [2], held_rotor [3]))
if last_released_rotor then print (string.format ("released_rotor_last\t%.6f %.6f %.6f", last_released_rotor [1], last_released_rotor [2], last_released_rotor [3])) end
print (string.format ("first_change_after_release\t%s\tat=%s", tostring (first_change_mode), tostring (first_change_at and string.format ("%.2f", first_change_at))))
fleet_line ()
subject_lines ()
assert (held_rotor_steps > 1, "the dying helicopter's rotor was held for only one step")
assert (held_changed == 0, "the campaign changed the rotor while the host held it")
assert (release_at, "the held rotor was never released")
assert (released_steps > 1, "the rotor was released for only one step")
assert (last_released_rotor and not same (last_released_rotor, held_rotor), "the released rotor did not advance in the campaign")
assert (first_change_mode == "rotor_down", "the first rotor change after the release did not come from rotor_down: " .. tostring (first_change_mode))
print (string.format ("held_seconds\t%.1f", release_at - hold_started))
print ("select\thold")
print ("host_id\t" .. selected)
print ("result\tok")
