-- TEST-ONLY witness for host loss reports in the five non-helicopter force families (EEDCS-41).
-- Needs a DLL built with the `loss-fixture` feature; the shipping module has no `loss_fixture` method.
--
-- The fixture constructs no entity and replaces no handler. It names one live, hosted member of the booted Lebanon
-- campaign and ends it through the campaign's own kill_client_server_entity, so the genuine kill_local of the family
-- runs, reaches the DLL's loss decision, and calls this script's observe_loss (in the same stack). It stands in for the
-- combat that would end a member at a time a scenario cannot choose; it is not a causal claim about combat (D2).
--
-- Per family (fixed_wing, routed, anti_air, person, ship) the host answers `withhold` for one member A whose shape the
-- bounded case supports (AI, not landed, a non-final group, no group/member task dependents, no landing action but a
-- fixed-wing member's one site reservation, no movement relations). A member whose shape is not supported is answered `forward` by the host (a host policy decision on
-- the descriptor the campaign supplied) and the next candidate is tried; those are listed as `skipped`. Then:
--   * campaign transition: A leaves the campaign group at once, but the group still counts it logically (the withheld
--     loss has no campaign consequence yet);
--   * a FORWARD control B of the same group is ended with the reference host's answer (`forward`, taken from
--     reference_loss, not from this script): the same stack, and B is out of the group at once. B's immediate loss must
--     not disturb the pending A;
--   * wreck disposal is host-owned and differs by family (a routed vehicle, person or fixed-wing wreck is retired after
--     5 min to 3 h of game time, a ship once it has sunk, an anti-aircraft wreck never). The campaign accepts the report once
--     the physical loss is final: the identity is gone, or it is still the incarnation the campaign maps and is dead and
--     unlinked. report=present (default) reports at once, with the wreck still present; report=retired waits for the host to
--     retire it first (the retirement-gated path);
--   * `engine:report_loss(host_id)` then applies A's loss exactly once: the receipt shows the group's loss count +1, its
--     logical members -1 from what B's immediate loss left, the primary task's loss count +1 where the family has one;
--   * a probe C of the same group (group of at least 4 only) shows both campaign decrements (A's report, B's forward) in
--     the member count the campaign supplies to the host.
-- A requested family with no supported member fails the run. Force info and the regen queue are read through the
-- feature-gated, read-only engine:loss_state(side) (total force losses, force-info hardware per category, regen queue
-- ring state and its last-written slot) with no frame between the two readings of a pair: the withheld kill leaves all of it
-- unchanged, the FORWARD control changes it by exactly one loss / one hardware / one regen ring insertion, and the report changes
-- it by exactly the same amount, once, for the phases the family has (a person has neither; ship and anti-air have no regen). The phase order is enforced inside the DLL (strictly ascending phases, fatal otherwise) and FORWARD
-- exactness is covered by the retained regression pack. A rejected report poisons the engine, so no state is read after it.
--
-- Candidates are screened, before any kill, by the feature-gated, read-only engine:loss_shape(entity_id): it runs the guard
-- the bounded WITHHOLD itself runs on the descriptor the host would be shown and returns its refusal text ("" when the loss
-- would be withheld). A screened-out member is not killed, so no member is spent on an unsupported shape. screen=off restores
-- the try-by-kill selection (the host answers `forward` for an unsupported shape and the next candidate is tried).
--
-- release=<families> (optional, witness mode): setup for a family whose members in flight are almost always engaged, which the
--            bounded WITHHOLD refuses by design ("group with task dependents or restricted guides": a task targets the group, or an
--            engage guide holds a per-member valid-member mask). For a candidate refused for that reason, the feature-gated
--            engine:loss_release(entity_id) ends the tasks that target its group through the campaign's own task-completion
--            handlers (what group.c does when it kills a group) and ends its engage guides through the campaign's own engagement
--            teardown (terminate_all_engage_tasks, terminate_entity_current_engage_task), without ending the group; takeoff guides
--            are not touched. The candidate is screened again. It never changes the guard or the physics; each release (count of
--            tasks notified plus guides ended, then the new screen result) is printed as `released`. It is a setup step, not a
--            claim that the campaign would have ended those engagements itself.
--
-- mode=witness   the above (default)
-- mode=shape     read-only census, no kill: per family the members, how many the bounded WITHHOLD would accept, and the
--                refusal histogram of the rest.
-- mode=negative  the unsupported-shape case: withhold the final member of a group of one. The campaign must fail
--                explicitly (a fatal engine error naming the final member); this poisons the engine, so it ends the run.
--
-- reject=duplicate|forwarded|unknown (witness mode, optional): after the last family, one rejected report: the A already
--            reported, the FORWARD control B (no loss is pending for it), or an id the campaign never saw. It must raise.
--            A rejected report is fatal to the engine, so a run carries at most one.
-- host.args: root, mode, report (present|retired), warmup (default 30 frames) of warmup_ms (default frame_ms; a long warmup lets fixed-wing and ships leave the ground/port), frame_ms (default 100), settle_ms (default 1000, the frame step while a wreck ages),
--            settle_limit (default 12000 frames), families (default fixed_wing,routed,anti_air,person,ship),
--            candidates (default 40 tries per family), reject

io.stdout:setvbuf ("no")

local args = host.args
local root = assert (args.root, "root=<installation root>")
local mode = args.mode or "witness"
assert (mode == "witness" or mode == "negative" or mode == "shape", "mode=witness|negative|shape")
local frame_ms = tonumber (args.frame_ms or "100")
local warmup = tonumber (args.warmup or "30")
local warmup_ms = tonumber (args.warmup_ms or args.frame_ms or "100")
local settle_ms = tonumber (args.settle_ms or "1000")
local settle_limit = tonumber (args.settle_limit or "12000")
local max_tries = tonumber (args.candidates or "40")
local screen = args.screen ~= "off"
local release_families, release_limit = {}, tonumber (args.release_limit or "6")
local group_limit = tonumber (args.group_limit or "0") -- mode=shape: print the group shape (loss_group) of the first N refused members per family
for name in string.gmatch (args.release or "", "[^,]+") do release_families [name] = true end
local dc = require ("eech_dc")
dc.prepare_installation (root)

local ENTITY = { fixed_wing = 10, routed = 23, anti_air = 1, person = 19, ship = 28 }
local HAS_PRIMARY_TASK_LOSSES = { fixed_wing = true, routed = true, ship = true }
local order = {}
for name in string.gmatch (args.families or "fixed_wing,routed,anti_air,person,ship", "[^,]+") do
	assert (ENTITY [name], "unknown family " .. name)
	order [#order + 1] = name
end

local reference_bind = host.bind_mobile
local reference_loss = host.observe_loss
local entity_of = {}
local want = {}       -- host id -> true only for the members the host should withhold if the shape is supported
local observed = {}   -- host id -> { descriptor, answer, why, in_fixture }
local in_fixture = false

local function bind_mobile (descriptor)
	local id = reference_bind (descriptor)
	entity_of [id] = descriptor.entity_id
	return id
end

-- the one landing action the bounded WITHHOLD carries: a fixed-wing member's single reservation release
local function fixed_wing_reservation (d)
	local a = d.landing_actions and d.landing_actions [1]
	return d.entity_type == ENTITY.fixed_wing and d.landing_action_count == 1 and a ~= nil and a.action == "unreserve_site" and a.count == 1
end

-- the shapes the campaign's bounded WITHHOLD supports; anything else the host forwards
local function unsupported (d)
	if d.player ~= 0 then return "not an AI member" end
	if d.landed then return "landed" end
	if d.campaign_member_count <= 1 then return "final member" end
	if d.group_task_dependents ~= 0 or d.restricted_guides ~= 0 then return "group task dependents or restricted guides" end
	if d.movement_parent or d.movement_dependents ~= 0 or d.takeoff_queue then return "movement relations" end
	if d.member_task_dependent_count ~= 0 then return "member task dependents" end
	if d.landing_action_count ~= 0 and not fixed_wing_reservation (d) then return "landing actions" end
	return nil
end

local function observe_loss (d)
	local entry = { descriptor = d, in_fixture = in_fixture }
	observed [d.host_id] = entry
	if mode == "witness" and want [d.host_id] then
		local why = unsupported (d)
		if not why then
			entry.answer = "withhold"
			return "withhold"
		end
		entry.answer, entry.why = "forward", why
		return "forward"
	end
	if mode == "negative" and want [d.host_id] then
		-- only the final-member refusal is under test: an unsuitable lone member is forwarded and the next one is tried
		if d.player ~= 0 or d.landed then
			entry.answer, entry.why = "forward", d.player ~= 0 and "not an AI member" or "landed"
			return "forward"
		end
		entry.answer = "withhold"
		return "withhold"
	end
	-- everything not marked is answered by the reference host, not by this script
	entry.answer = reference_loss (d)
	return entry.answer
end

local engine = dc.boot ({
	install_root = root,
	map = "..\\common\\maps\\map5",
	campaign_directory = "camp01",
	campaign = "LEBANON.CHC",
	gunship = "apache",
	seed = 1,
	bind_mobile = bind_mobile,
	move_vehicle = host.move_vehicle,
	observe_loss = observe_loss,
	supply_reached = host.supply_reached,
	line_of_sight = host.line_of_sight,
	sense = host.sense,
})
for _ = 1, warmup do engine:frame (warmup_ms) end
-- The wreck's dead-frame test fires only when (int)(timer + 1) is a multiple of 300 (rv_updt.c), a one second window; a wreck's
-- timer starts at the delta of the frame before the kill.  A settle step that divides 300 s then lands in that window only if
-- that delta is settle_ms - 500 ms, so one aligning frame precedes the kills (the kills follow it with no frame between).
if settle_ms >= 1000 and 300000 % settle_ms == 0 then engine:frame (settle_ms - 500) end

local failures = {}
local function check (name, ok, detail)
	print (string.format ("check\t%s\t%s%s", name, ok and "pass" or "FAIL", detail and ("\t" .. detail) or ""))
	if not ok then failures [#failures + 1] = name end
end

local function finish ()
	if #failures > 0 then
		print ("result\tfail")
		error (#failures .. " check(s) failed: " .. table.concat (failures, ", "), 0)
	end
	print ("result\tok")
end

-- logical group sizes (members still counted in their group) and the live members per family
local function census (family)
	local size, list = {}, {}
	for _, member in ipairs (engine:members ()) do
		if member.in_group and member.physical_present and member.host_id then
			size [member.group_key] = (size [member.group_key] or 0) + 1
		end
	end
	for _, member in ipairs (engine:members ()) do
		if member.in_group and member.physical_present and member.host_id and member.current_entity_id
			and member.entity_type == ENTITY [family] and entity_of [member.host_id] then
			list [#list + 1] = { member = member, size = size [member.group_key] }
		end
	end
	table.sort (list, function (a, b) return a.member.current_entity_id < b.member.current_entity_id end)
	return list
end

local function find_member (host_id)
	for _, member in ipairs (engine:members ()) do
		if member.host_id == host_id then return member end
	end
end

local function raises (f, ...)
	local ok, message = pcall (f, ...)
	return not ok, tostring (message)
end

-- role "withhold" marks the member for the host's withhold policy; any other role is left to the reference host's answer
local SIDE = { blue = 1, red = 2 }   -- entity_sides of the force entities
local snapshots = {}   -- host id -> { before, after }: force state around its kill, read with no frame between
local function kill (candidate, role)
	local id = candidate.member.host_id
	local side = SIDE [candidate.member.side]
	if role == "withhold" then want [id] = true end
	local before = engine:loss_state (side)
	in_fixture = true
	local ok, message = pcall (engine.loss_fixture, engine, candidate.member.current_entity_id)
	in_fixture = false
	if ok then snapshots [id] = { before = before, after = engine:loss_state (side) } end
	return ok, tostring (message), id
end

local function slot_text (slot)
	return slot and string.format ("%d/%d/%d", slot [1], slot [2], slot [3]) or "-"
end

-- what changed between two force states: total losses, the per-category hardware deltas that are not zero, and the regen queues
-- that changed.  A regen queue is a ring (regen_queue_insert in rg_updt.c): an insertion writes the slot at (front + count) % size
-- and either grows the count by one (below capacity) or, at capacity, overwrites the oldest entry, advances the front by one
-- and leaves the count unchanged.  So a queue's change is classified by count, front and the slot written last, never by the count
-- alone: kind "insert" is exactly one of those two ring moves, anything else that changed is kind "other".
local function state_delta (before, after)
	local delta = { losses = after.losses - before.losses, hardware = {}, regen = {} }
	local text = { hardware = {}, regen = {} }
	for index = 1, #before.hardware do
		local change = after.hardware [index] - before.hardware [index]
		if change ~= 0 then
			delta.hardware [#delta.hardware + 1] = change   -- numeric, in category order
			text.hardware [#text.hardware + 1] = string.format ("%d:%+d", index - 1, change)
		end
	end
	for index = 1, #before.regen do
		local count0, count1 = before.regen [index], after.regen [index]
		local front0, front1 = before.regen_front [index], after.regen_front [index]
		local size = before.regen_size [index]
		local last0, last1 = slot_text (before.regen_last [index]), slot_text (after.regen_last [index])
		if count0 ~= count1 or front0 ~= front1 or size ~= after.regen_size [index] or last0 ~= last1 then
			local grew = size == after.regen_size [index] and count1 == count0 + 1 and count0 < size and front1 == front0
			local wrapped = size == after.regen_size [index] and count0 == size and count1 == size and front1 == (front0 + 1) % size
			delta.regen [#delta.regen + 1] = { queue = index - 1, kind = (grew or wrapped) and "insert" or "other", wrapped = wrapped }
			text.regen [#text.regen + 1] = string.format ("%d:%s count %d>%d front %d>%d size %d last %s>%s", index - 1,
				wrapped and "wrapped" or (grew and "grew" or "other"), count0, count1, front0, front1, size, last0, last1)
		end
	end
	delta.text = string.format ("losses%+d hardware[%s] regen[%s]", delta.losses, table.concat (text.hardware, ","), table.concat (text.regen, ";"))
	return delta
end

-- the force-state phases each family's original destroy code ran (docs/m5-physical-loss.md): force info (a force loss and one
-- hardware unit fewer in the member's category) and regen (one ring insertion into one regen queue)
local PHASES = {
	routed    = { force_info = true,  regen = true  },
	fixed_wing = { force_info = true,  regen = true  },
	ship      = { force_info = true,  regen = false },
	anti_air  = { force_info = true,  regen = false },
	person    = { force_info = false, regen = false },
}

-- exactly the family's loss consequence: nothing where the family steps neither phase, else one force loss, one hardware unit
-- fewer in one category, and one ring insertion into one regen queue only where the family has the regen phase
local function is_one_loss (family, delta)
	local phases = assert (PHASES [family], family)
	if not phases.force_info then
		return delta.losses == 0 and #delta.hardware == 0 and #delta.regen == 0
	end
	local regen_ok = phases.regen and (#delta.regen == 1 and delta.regen [1].kind == "insert") or (not phases.regen and #delta.regen == 0)
	return delta.losses == 1 and #delta.hardware == 1 and delta.hardware [1] == -1 and regen_ok
end

local function is_unchanged (delta)
	return delta.losses == 0 and #delta.hardware == 0 and #delta.regen == 0
end

-- the bounded WITHHOLD's own refusal for a member ("" if it would accept), read-only
local function shape_of (candidate)
	return engine:loss_shape (candidate.member.current_entity_id)
end

if mode == "shape" then
	for _, family in ipairs (order) do
		local members, reasons, supported, groups = census (family), {}, {}, {}
		for _, candidate in ipairs (members) do
			local reason = shape_of (candidate)
			if reason == "" then
				supported [#supported + 1] = string.format ("%s(group %d)", candidate.member.host_id, candidate.size)
			else
				reason = string.gsub (reason, "^campaign loss: ", "")
				reasons [reason] = (reasons [reason] or 0) + 1
				if string.find (reason, "task dependents", 1, true) then groups [#groups + 1] = string.format ("%s size=%d %s", candidate.member.host_id, candidate.size, engine:loss_group (candidate.member.current_entity_id)) end
			end
		end
		print (string.format ("shape\t%s\tmembers=%d supported=%d", family, #members, #supported))
		local names = {}
		for reason in pairs (reasons) do names [#names + 1] = reason end
		table.sort (names)
		for _, reason in ipairs (names) do print (string.format ("shape\t%s\trefused %d: %s", family, reasons [reason], reason)) end
		for _, line in ipairs (supported) do print (string.format ("shape\t%s\tsupported %s", family, line)) end
		for i, line in ipairs (groups) do if i <= group_limit then print (string.format ("group\t%s\t%s", family, line)) end end
	end
	finish ()
	return
end

if mode == "negative" then
	-- a member alone in its group: the final member, which the bounded case does not support
	local lones = {}
	for _, family in ipairs (order) do
		for _, candidate in ipairs (census (family)) do
			if candidate.size == 1 then lones [#lones + 1] = { candidate = candidate, family = family } end
		end
	end
	if #lones == 0 then
		print ("note	no single-member group of the selected families in this campaign")
		check ("negative.available", false, "no single-member group of the selected families")
		finish ()
		return
	end
	local lone, ok, message, id, entry
	for _, option in ipairs (lones) do
		lone = option
		ok, message, id = kill (option.candidate, "withhold")
		entry = observed [id]
		if entry and entry.answer == "withhold" then break end
		print (string.format ("note	lone %s member %s answered %s (%s); trying the next", option.family, id, tostring (entry and entry.answer), tostring (entry and entry.why)))
		if not ok then break end
	end
	print (string.format ("negative\tfamily=%s host_id=%s answer=%s message=%s", lone.family, id, tostring (entry and entry.answer), message))
	check ("negative.fails_explicitly", not ok and string.find (message, "final member", 1, true) ~= nil, message)
	finish ()
	return
end

-- squared horizontal distance (m^2) from each live entity to the nearest keysite, by entity id. The host ages a dead wreck 5 minutes
-- within 1000 m of a keysite and up to 3 hours at 10000 m or more (mobile.c), and the report needs the wreck retired, so a member
-- near a keysite is the cheapest to witness. Observation only: it orders candidates, it never changes what is supported.
local function keysite_distances ()
	local sites, distance = {}, {}
	local objects = engine:objects ()
	for _, o in ipairs (objects) do
		if o.kind == "keysite" and o.alive then sites [#sites + 1] = o.position end
	end
	for _, o in ipairs (objects) do
		local best = math.huge
		for _, site in ipairs (sites) do
			local dx, dz = o.position [1] - site [1], o.position [3] - site [3]
			best = math.min (best, dx * dx + dz * dz)
		end
		distance [o.id] = best
	end
	return distance
end

-- candidates of the family in a group of at least `minimum` (nearest a keysite first, then largest groups), skipping members
-- already tried
local function candidates_of (family, minimum, tried)
	local list, near = {}, keysite_distances ()
	for _, candidate in ipairs (census (family)) do
		if candidate.size >= minimum and not tried [candidate.member.host_id] then
			candidate.keysite_m2 = near [candidate.member.current_entity_id] or math.huge
			list [#list + 1] = candidate
		end
	end
	table.sort (list, function (a, b)
		if a.keysite_m2 ~= b.keysite_m2 then return a.keysite_m2 < b.keysite_m2 end
		if a.size ~= b.size then return a.size > b.size end
		return a.member.current_entity_id < b.member.current_entity_id
	end)
	return list
end

local function same_group (family, group_key, tried)
	for _, candidate in ipairs (census (family)) do
		if candidate.member.group_key == group_key and not tried [candidate.member.host_id] then return candidate end
	end
end

local summary, last_reported, last_forwarded = {}, nil, nil
local withheld = {}   -- one entry per family whose member was withheld, in family order

-- stage 1: per family, withhold one member and forward a control of the same group
for _, family in ipairs (order) do
	local skipped, screened, done, tried, tries = {}, {}, nil, {}, 0
	while not done and tries < max_tries do
		local candidate = candidates_of (family, 2, tried) [1] -- 2: a lone member is the final member, which is unsupported
		if candidate and screen then
			-- read-only screen: the first candidate the bounded WITHHOLD would accept; the rest are not killed
			candidate = nil
			for _, option in ipairs (candidates_of (family, 2, tried)) do
				local reason = shape_of (option)
				if release_families [family] and release_limit > 0 and string.find (reason, "task dependents", 1, true) then
					release_limit = release_limit - 1
					local count = engine:loss_release (option.member.current_entity_id)
					reason = shape_of (option)
					print (string.format ("released\t%s\t%s released=%d then=%s", family, option.member.host_id, count, reason == "" and "supported" or reason))
					if reason ~= "" then print (string.format ("residual\t%s\t%s %s", family, option.member.host_id, engine:loss_group (option.member.current_entity_id))) end
				end
				if reason == "" then candidate = option break end
				tried [option.member.host_id] = true
				screened [#screened + 1] = string.format ("%s:%s", option.member.host_id, reason)
			end
		end
		if not candidate then break end
		tries = tries + 1
		tried [candidate.member.host_id] = true
		local ok, message, id = kill (candidate, "withhold")
		local entry = observed [id]
		if not ok then
			check (family .. ".kill_call", false, message)
			break
		end
		if entry and entry.answer == "withhold" then
			done = { candidate = candidate, id = id, entry = entry }
		else
			skipped [#skipped + 1] = string.format ("%s:%s", id, entry and entry.why or "no loss observed")
		end
	end
	for _, line in ipairs (screened) do print (string.format ("screened\t%s\t%s", family, line)) end
	for _, line in ipairs (skipped) do print (string.format ("skipped\t%s\t%s", family, line)) end
	if not done then
		print (string.format ("unavailable\t%s\tno supported member in %d tries, %d screened out", family, tries, #screened))
		check (family .. ".available", false, string.format ("no withhold-supported member in %d tries, %d screened out", tries, #screened))
		summary [#summary + 1] = family .. "=UNAVAILABLE"
	else
		local d, id, candidate = done.entry.descriptor, done.id, done.candidate
		local n, group_key = d.campaign_member_count, candidate.member.group_key
		print (string.format ("withheld\t%s\thost_id=%s entity=%d event=%s group_members=%d primary_task=%s keysite_m=%.0f", family, id, candidate.member.current_entity_id,
			tostring (d.event), n, tostring (d.primary_task_key), math.sqrt (candidate.keysite_m2)))
		check (family .. ".observed_in_same_stack", done.entry.in_fixture, "observe_loss ran inside the kill call")
		check (family .. ".event_physical", d.event == "physical", tostring (d.event))
		check (family .. ".entity_type", d.entity_type == ENTITY [family], tostring (d.entity_type))

		-- campaign transition: out of the group at once; the wreck's physical presence is the host's, reported not asserted
		local record = find_member (id)
		check (family .. ".left_group_at_once", record and not record.in_group,
			record and string.format ("physical_present=%s in_group=%s", tostring (record.physical_present), tostring (record.in_group)) or "no record")
		print (string.format ("wreck\t%s\tphysical_present_at_once=%s", family, tostring (record and record.physical_present)))
		local snapshot = snapshots [id]
		local withheld_delta = snapshot and state_delta (snapshot.before, snapshot.after)
		print (string.format ("force_state\t%s\twithheld kill: %s", family, withheld_delta and withheld_delta.text or "not read"))
		check (family .. ".withheld_force_info_and_regen_unchanged", withheld_delta and is_unchanged (withheld_delta), withheld_delta and withheld_delta.text)

		-- FORWARD control B of the same group, answered by the reference host while A is pending
		local forwarded, control_descriptor = 0, nil
		-- run only when a member survives it: forwarding the second-to-last member while A is pending would free the group
		local control = n >= 3 and same_group (family, group_key, tried) or nil
		if not control then
			print (string.format ("note\t%s\tno forward control (group of %d members at observe)", family, n))
		else
			tried [control.member.host_id] = true
			local ok, message, control_id = kill (control, "control")
			local entry = observed [control_id]
			check (family .. ".forward_call", ok, message)
			check (family .. ".forward_same_stack", entry and entry.in_fixture and entry.answer == "forward",
				entry and tostring (entry.answer) or "no loss observed")
			check (family .. ".forward_withheld_member_still_counted", entry and entry.descriptor.campaign_member_count == n,
				entry and string.format ("count=%d expected=%d", entry.descriptor.campaign_member_count, n) or "no loss observed")
			local control_snapshot = snapshots [control_id]
			local control_delta = control_snapshot and state_delta (control_snapshot.before, control_snapshot.after)
			print (string.format ("force_state\t%s\tforward control kill: %s", family, control_delta and control_delta.text or "not read"))
			check (family .. ".forward_applies_force_info_and_regen_once", control_delta and is_one_loss (family, control_delta), control_delta and control_delta.text)
			local gone = find_member (control_id)
			check (family .. ".forward_left_group_at_once", gone and not gone.in_group,
				gone and string.format ("physical_present=%s in_group=%s", tostring (gone.physical_present), tostring (gone.in_group)) or "no record")
			if ok and entry and entry.answer == "forward" then
				forwarded = 1
				last_forwarded = control_id
				control_descriptor = entry.descriptor
			end
		end
		withheld [#withheld + 1] = { family = family, id = id, candidate = candidate, descriptor = d, n = n, group_key = group_key, tried = tried, forwarded = forwarded,
			control_descriptor = control_descriptor }
	end
end

-- stage 2: with report=retired only, wreck disposal (host-owned, 5 min to 3 h of game time by distance to a keysite): one world
-- run for all the withheld members
local report_mode = args.report or "present"
if report_mode ~= "present" and report_mode ~= "retired" then error ("report=present|retired", 0) end
local waited = 0
local function all_retired ()
	for _, w in ipairs (withheld) do
		local still = find_member (w.id)
		if still and still.physical_present then return false end
	end
	return true
end
while report_mode == "retired" and not all_retired () and waited < settle_limit do
	engine:frame (settle_ms)
	waited = waited + 1
	if waited % 5 == 0 then print (string.format ("waiting	%d frames, cpu %.1f s", waited, os.clock ())) end
end
print (string.format ("settled	%d frames of %d ms (%.1f game minutes)", waited, settle_ms, waited * settle_ms / 60000))

-- stage 3: per family, the report and the probe
local function report_family (w)
	local family, id, candidate, d, n, group_key, tried, forwarded = w.family, w.id, w.candidate, w.descriptor, w.n, w.group_key, w.tried, w.forwarded
	local control_descriptor = w.control_descriptor
	local still = find_member (id)
	print (string.format ("wreck	%s	at_report physical_present=%s in_group=%s after %d frames", family,
		tostring (still and still.physical_present), tostring (still and still.in_group), waited))
	check (family .. ".still_out_of_group_before_report", still and not still.in_group)
	if report_mode == "retired" then
		check (family .. ".wreck_retired", still and not still.physical_present)
		if not (still and not still.physical_present) then
			-- retirement-gated mode reports only a retired wreck
			summary [#summary + 1] = family .. "=WRECK-NOT-RETIRED"
			return
		end
	end

	-- the report: the campaign consequences apply now, once
	local state_before = engine:loss_state (SIDE [candidate.member.side])
	local receipt = engine:report_loss (id)
	local report_delta = state_delta (state_before, engine:loss_state (SIDE [candidate.member.side]))
	print (string.format ("force_state\t%s\treport: %s", family, report_delta.text))
	check (family .. ".report_applies_force_info_and_regen_once", is_one_loss (family, report_delta), report_delta.text)
	print (string.format ("receipt\t%s\tgroup_losses=%d>%d logical=%d>%d physical_before=%d primary_losses=%d>%d landing=%d>%d expire=%d completed=%d",
		family, receipt.group_losses_before, receipt.group_losses_after, receipt.group_logical_members_before, receipt.group_logical_members_after,
		receipt.group_physical_members_before, receipt.primary_task_losses_before, receipt.primary_task_losses_after,
		receipt.landing_reserved_before, receipt.landing_reserved_after, receipt.tasks_retired_expire, receipt.tasks_completed_on_report))
	check (family .. ".group_losses_plus_one", receipt.group_losses_after == receipt.group_losses_before + 1)
	check (family .. ".logical_members_held_until_report", receipt.group_logical_members_before == n - forwarded,
		string.format ("observe=%d forwarded=%d report_before=%d", n, forwarded, receipt.group_logical_members_before))
	check (family .. ".logical_members_minus_one", receipt.group_logical_members_after == receipt.group_logical_members_before - 1)
	check (family .. ".physical_members_already_removed", receipt.group_physical_members_before == n - 1 - forwarded,
		string.format ("physical_before=%d", receipt.group_physical_members_before))
	if HAS_PRIMARY_TASK_LOSSES [family] and d.primary_task_key then
		check (family .. ".primary_task_losses_plus_one", receipt.primary_task_losses_after == receipt.primary_task_losses_before + 1)
	else
		check (family .. ".primary_task_losses_unchanged", receipt.primary_task_losses_after == receipt.primary_task_losses_before)
	end
	if fixed_wing_reservation (d) then
		-- the withheld kill left the reservation in place (this family's own same-stack FORWARD control, which read the reservation
		-- after A was withheld, saw the count A saw), the control released its own once, and the report released A's once.
		-- The control is stored per withheld entry, never the global last_forwarded, which a later family overwrites.
		local a = d.landing_actions [1]
		local c = control_descriptor and control_descriptor.landing_actions and control_descriptor.landing_actions [1]
		local same_landing = c and c.landing_key == a.landing_key
		print (string.format ("landing	%s	observe reserved=%d control_saw=%s report=%d>%d", family, a.reserved_landing_sites,
			same_landing and tostring (c.reserved_landing_sites) or "n/a", receipt.landing_reserved_before, receipt.landing_reserved_after))
		check (family .. ".landing_reservation_released_once", receipt.landing_reserved_before >= 1
			and receipt.landing_reserved_after == receipt.landing_reserved_before - 1,
			string.format ("%d>%d", receipt.landing_reserved_before, receipt.landing_reserved_after))
		-- required causal proof of the preserved reservation, from one of two direct state reads (never skipped, never inferred):
		-- (1) a same-landing FORWARD control read after A was withheld saw the count A saw, and released its own once; or
		-- (2) with no such control (a group too small for the probe), the campaign's own reservation count read inside the report
		-- (receipt.landing_reserved_before) equals the count A saw at observe, with no world frame run between the two reads.
		-- Either way a withheld kill that had released the reservation would make the report-time count one lower.
		local controlled = same_landing and forwarded == 1
		if controlled then
			check (family .. ".withhold_preserved_reservation", c.reserved_landing_sites == a.reserved_landing_sites,
				string.format ("A saw %d, control saw %d", a.reserved_landing_sites, c.reserved_landing_sites))
			check (family .. ".reservation_released_once_per_loss", receipt.landing_reserved_before == c.reserved_landing_sites - 1,
				string.format ("control saw %d, report before %d", c.reserved_landing_sites, receipt.landing_reserved_before))
		else
			check (family .. ".withhold_preserved_reservation", waited == 0 and receipt.landing_reserved_before == a.reserved_landing_sites,
				string.format ("no control; direct read: observe %d, report-time %d, frames between %d", a.reserved_landing_sites,
					receipt.landing_reserved_before, waited))
		end
		check (family .. ".no_task_work", receipt.tasks_retired_expire == 0 and receipt.tasks_completed_on_report == 0)
	else
		check (family .. ".no_landing_or_task_work", receipt.landing_reserved_before == 0 and receipt.landing_reserved_after == 0
			and receipt.tasks_retired_expire == 0 and receipt.tasks_completed_on_report == 0)
	end
	last_reported = id
	for _ = 1, 10 do engine:frame (frame_ms) end

	-- probe C: the campaign's own member count now reflects A's report and B's immediate loss
	local expected = n - 1 - forwarded
	if expected >= 2 then
		local probe = same_group (family, group_key, tried)
		if probe then
			tried [probe.member.host_id] = true
			local ok, message, probe_id = kill (probe, "probe")
			local entry = observed [probe_id]
			check (family .. ".probe_call", ok, message)
			check (family .. ".probe_sees_both_decrements", entry and entry.descriptor.campaign_member_count == expected,
				entry and string.format ("count=%d expected=%d", entry.descriptor.campaign_member_count, expected) or "no loss observed")
		else
			print (string.format ("note\t%s\tno third member of the group for the probe", family))
		end
	else
		print (string.format ("note\t%s\tgroup too small for the probe (%d members at observe)", family, n))
	end
	summary [#summary + 1] = family .. "=withheld+reported(" .. report_mode .. ")"
end
for _, w in ipairs (withheld) do report_family (w) end

-- one rejected report, last: it is fatal to the engine
local reject = args.reject
if reject then
	local id, expect
	if reject == "duplicate" then id, expect = last_reported, "already reported"
	elseif reject == "forwarded" then id, expect = last_forwarded, "no loss is pending"
	elseif reject == "unknown" then id, expect = "host-id-the-campaign-never-saw", "no loss is pending"
	else error ("reject=duplicate|forwarded|unknown", 0) end
	if not id then
		check ("reject." .. reject .. ".available", false, "no such id in this run")
	else
		local failed, message = raises (engine.report_loss, engine, id)
		print (string.format ("rejected\t%s\thost_id=%s message=%s", reject, id, message))
		check ("reject." .. reject .. ".raises", failed and string.find (message, expect, 1, true) ~= nil, message)
		local frame_failed, frame_message = raises (engine.frame, engine, frame_ms)
		print (string.format ("note\tengine after the rejected report: frame %s (%s)", frame_failed and "raises" or "runs", frame_message))
	end
	summary [#summary + 1] = "reject=" .. reject
end
print ("summary\t" .. table.concat (summary, " "))
finish ()
