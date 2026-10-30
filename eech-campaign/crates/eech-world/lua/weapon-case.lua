-- TEST-ONLY controlled physical-launch witness (EEDCS-45). Needs a DLL built with the `weapon-fixture` feature.
--
-- The natural Lebanon witnesses (lua/weapon-characterization.lua) leave important weapon-execution branches unreached: no launch ever finds the launcher out of ammunition, no
-- launch is made with the launcher's target link empty, no fuze-timer weapon, smoke grenade or lock-on-after-launch weapon is launched, and the fixture's own argument checks
-- (unsupported roster kinds) are never exercised. This is ONE bounded, fresh original war that does: it boots the campaign through the public path (require ("eech_dc")) with
-- the reference host's six callbacks, lets the campaign run for `warmup` simulated seconds, then makes controlled calls into the campaign's OWN physical launch call
-- (launch_client_server_weapon, the call the family fire code makes) for real, host-identified launchers, optionally after setting one real host-identified target link through
-- the campaign's own setter. The launcher, the weapon type and the target are test PRECONDITIONS: nothing here says the campaign would have chosen them. The creation, the
-- birth kinematics, the flight and the terminal facts that follow are the original's and are observed by the same fixture hooks as the natural witnesses; the host answers every
-- aim request through the real callbacks.
--
-- Checks (each a `check` event; the run fails, with a nonzero native exit, if any fails or any expected check is missing):
--   rejects_unsupported_launcher     a roster id that is not one of the six launching families is rejected before any getter or mutation
--   queries_reject_unsupported_subject  the read-only query refuses the same ids
--   rejection_leaves_state_unchanged    the rejection changes nothing (the launcher's link and ammunition, and no event)
--   accepts_site_target              a SITE is a valid static TARGET (target list root, alive, side, position, target point) though it is not a launcher: a real launcher
--                                    is given the site as its target through the campaign's own setter, launches, and gets its previous link back
--   empty_target_establishes_no_link / empty_target_restores_previous_link / empty_target_birth_has_no_target
--                                    an empty target id really leaves the launcher with NO link (set through the campaign's setter), the launch then runs with no target, and
--                                    the launcher's previous link is restored through the same setter
--   no_ammo_outcome / no_ammo_changes_nothing   a launcher asked to launch a type it does not carry reports the no-ammunition outcome and creates nothing
--   targeted_launches_created        at least three targeted launches by real launchers created weapons
--   case_weapons_reach_terminal      every weapon those launches created reached its terminal fact. The wait is computed from the weapons themselves (their lifetime and
--                                    the database cruise time, reported by the fixture): the source ends a weapon at lifetime <= -(cruise_time + error) or on a contact, so a
--                                    crate with a 511 s cruise time needs about 511 s. Optional categories (fuze timer, lock-on-after-launch, ...) are MEASURED, never promised.
--
-- aim_model=own_model runs the same controlled launches under a generic host with a model of its own (lua/own-model.lua) instead of the reference host, and writes the
-- calls the campaign made of it, by request mode, to <out>.model.json: the witness that the host's flight, launch, release and what follows are really asked and answered
-- by a host that is not EECH's.  The default is the reference host and is unchanged.
-- host.args: root, warmup (simulated seconds before the cases, default 60), settle (the CAP, in simulated seconds, on the wait for the case weapons to finish; the wait is computed from the weapons, default 700),
--            frame_ms (default 100), out (JSON Lines path)

io.stdout:setvbuf ("no")

local args = host.args
local root = assert (args.root, "root=<installation root>")
local warmup = tonumber (args.warmup or "60")
local settle = tonumber (args.settle or "700")
local frame_ms = tonumber (args.frame_ms or "100")
local out = assert (io.open (assert (args.out, "out=<JSON Lines path>"), "w"))
local seed = 1

local aim = host.aim
local model_calls
if args.aim_model == "own_model" then
	local here = (debug.getinfo (1, "S").source:match ("^@?(.*[/\\])") or "")
	local own = dofile (here .. "own-model.lua")
	model_calls = {}
	aim = function (step)
		model_calls [step.mode] = (model_calls [step.mode] or 0) + 1
		return own [step.mode] (step)
	end
elseif args.aim_model ~= nil and args.aim_model ~= "reference" then
	error ("aim_model must be reference or own_model, not " .. tostring (args.aim_model))
end

-- a string as a JSON string with every control character, quote, backslash and non-ASCII byte escaped losslessly (a byte >= 0x80 is written as the code point of the same value, i.e.
-- the Latin-1 reading): nothing in the recorded text is replaced or dropped
local escapes = { ['"'] = '\\"', ['\\'] = '\\\\', ['\n'] = '\\n', ['\r'] = '\\r', ['\t'] = '\\t' }
local function json_string (s)
	return '"' .. (string.gsub (s, '[%c"\\\128-\255]', function (c) return escapes [c] or string.format ("\\u%04x", string.byte (c)) end)) .. '"'
end

local function number_or_string (v)
	if v ~= v or v == math.huge or v == -math.huge then return json_string (tostring (v)) end
	return tostring (v)
end

local function emit (t)
	local parts = {}
	for k, v in pairs (t) do
		local value
		if type (v) == "string" then value = json_string (v)
		elseif type (v) == "boolean" then value = v and "true" or "false"
		else value = number_or_string (v) end
		parts [#parts + 1] = json_string (k) .. ":" .. value
	end
	table.sort (parts)
	out:write ("{" .. table.concat (parts, ",") .. "}\n")
	out:flush ()
end

local string_keys = { table = true, launch = true, flight = true, terminal = true, decision = true, hash = true, track = true }

-- one JSON object per report line (see lua/weapon-characterization.lua); returns the parsed events too
local function emit_report (scope, at, report)
	local parsed = {}
	for line in string.gmatch (report, "[^\n]+") do
		local event, rest = string.match (line, "^(%S+)%s*(.*)$")
		local t = { event = event, scope = scope, at = at }
		if event == "wpn-sample" then
			local key, text = string.match (rest, "^key=(%S+)%s+(.*)$")
			t.key = key
			t.text = text
		else
			for k, v in string.gmatch (rest, "(%S+)=(%S+)") do t [k] = (not string_keys [k] and tonumber (v)) or v end
		end
		emit (t)
		parsed [#parsed + 1] = t
	end
	return parsed
end

local checks, failed = {}, 0
local function check (name, ok, detail)
	checks [#checks + 1] = name
	if not ok then failed = failed + 1 end
	emit { event = "check", name = name, ok = ok and true or false, detail = detail or "" }
end

local descriptors, order = {}, {}
local function bind_mobile (descriptor)
	local id = host.bind_mobile (descriptor)
	descriptors [id] = { kind = descriptor.movement_kind, side = descriptor.side }
	order [#order + 1] = id
	return id
end

local dc = require ("eech_dc")
dc.prepare_installation (root)
engine = dc.boot {
	install_root = root, map = "..\\common\\maps\\map5", terrain = host.terrain_provider (root, "..\\common\\maps\\map5"), campaign_directory = "camp01", campaign = "LEBANON.CHC", gunship = "apache",
	seed = seed, bind_mobile = bind_mobile, move_vehicle = host.move_vehicle, observe_loss = host.observe_loss,
	supply_reached = host.supply_reached, line_of_sight = host.line_of_sight, sense = host.sense, aim = aim,
}
emit { event = "boot", scenario = "lebanon_retail", case = "controlled_launch", warmup = warmup, settle = settle, frame_ms = frame_ms, seed = seed }
emit_report ("boot", 0, engine:wpn_trace ())
emit_report ("log", 0, engine:wpn_events ())

local clock = engine:clock ()
local frames = 0
local function step (seconds_more)
	local until_time = clock.elapsed_seconds + seconds_more
	local n = 0
	while clock.elapsed_seconds < until_time do
		engine:frame (frame_ms)
		frames = frames + 1
		n = n + 1
		clock = engine:clock ()
	end
	return n
end
step (warmup)
emit { event = "warmed_up", elapsed = clock.elapsed_seconds, frames = frames, bound = #order }

-- ---------------------------------------------------------------------------------------------------------------- helpers
local function fields (line)
	local t = {}
	for k, v in string.gmatch (line, "(%S+)=(%S+)") do t [k] = v end
	return t
end

local function numbers (list)
	local t = {}
	for v in string.gmatch (list, "[^,]+") do t [#t + 1] = tonumber (v) end
	return t
end

local function state (id, weapon)
	local ok, text = pcall (engine.wpn_case_query, engine, id, weapon or 0)
	if not ok then return nil, tostring (text) end
	local s = fields (text)
	s.alive, s.has_target, s.weapon_count = tonumber (s.alive), tonumber (s.has_target), tonumber (s.weapon_count)
	s.warhead, s.guidance, s.aiming, s.decoy = tonumber (s.warhead), tonumber (s.guidance), tonumber (s.aiming), tonumber (s.decoy)
	s.position = numbers (s.position)
	return s
end

local all_events = {}
local function drain ()
	local parsed = emit_report ("log", clock.elapsed_seconds, engine:wpn_events ())
	for _, e in ipairs (parsed) do all_events [#all_events + 1] = e end
	return parsed
end

local function launch (launcher, target, weapon)
	drain () -- everything the natural war logged before this call is taken first, so the events read after it are the call's own
	local ok, text = pcall (engine.wpn_case_launch, engine, launcher, target, weapon, 0)
	local events = ok and drain () or {}
	local r = { ok = ok, text = tostring (text), events = events }
	if ok then
		r.fields = fields (text)
		for _, k in ipairs { "target_link_before", "target_link", "count_before", "count_after", "births", "creates", "restored" } do r.fields [k] = tonumber (r.fields [k]) end
	end
	emit { event = "case_launch", launcher = launcher, target = target, weapon = weapon, ok = ok and true or false, text = tostring (text):gsub ("\n", " ") }
	return r
end

-- ---------------------------------------------------------------------------------------------------------------- inventory (read-only queries)
local supported = {}
local ids = {}
for _, id in ipairs (order) do ids [#ids + 1] = id end
local n_unsupported_queries = 0
for _, id in ipairs (ids) do
	local s = state (id, 0)
	if s and s.alive == 1 then
		s.side = descriptors [id].side
		s.inventory = {}
		for w = 1, 127 do
			local q = state (id, w)
			if q and q.weapon_count and q.weapon_count > 0 then s.inventory [#s.inventory + 1] = { weapon = w, count = q.weapon_count, warhead = q.warhead, guidance = q.guidance, aiming = q.aiming, decoy = q.decoy } end
		end
		supported [id] = s
	elseif not s then
		n_unsupported_queries = n_unsupported_queries + 1
	end
end
local n_supported = 0
for _ in pairs (supported) do n_supported = n_supported + 1 end
emit { event = "inventory", supported_alive = n_supported, bound_ids = #ids, bound_ids_rejected_by_query = n_unsupported_queries }

-- a roster id that is not a launching family but is a valid target (a site): found by kind, never by an error
local site_id, site_kind
for n = 1, 20000 do
	local id = "eech-world:" .. n
	local ok, text = pcall (engine.wpn_case_kind, engine, id)
	if ok then
		local k = fields (text)
		if k.found == "1" and k.launcher == "0" and k.target == "1" then site_id, site_kind = id, k.kind break end
	end
end
emit { event = "unsupported_roster_id", id = site_id or "none", kind = site_kind or "none" }

-- ---------------------------------------------------------------------------------------------------------------- candidate selection (preconditions, deterministic)
local ordered = {}
for id in pairs (supported) do ordered [#ordered + 1] = id end
table.sort (ordered, function (a, b) return tonumber (string.match (a, "(%d+)$")) < tonumber (string.match (b, "(%d+)$")) end)

local function nearest_enemy (id)
	local s = supported [id]
	local best, best_d
	for _, other in ipairs (ordered) do
		local o = supported [other]
		if other ~= id and o.side ~= s.side and (o.side == "blue" or o.side == "red") then
			local dx, dz = o.position [1] - s.position [1], o.position [3] - s.position [3]
			local d = math.sqrt (dx * dx + dz * dz)
			if not best_d or d < best_d then best, best_d = other, d end
		end
	end
	return best, best_d
end

local function first_weapon (id, wanted)
	for _, w in ipairs (supported [id].inventory) do if wanted (w) then return w end end
end

-- ---------------------------------------------------------------------------------------------------------------- rejections (before anything is mutated)
local linked
for _, id in ipairs (ordered) do
	local s = state (id, 0)
	if s and s.has_target == 1 and #supported [id].inventory > 0 then linked = id break end
end
emit { event = "linked_launcher", id = linked or "none" }
if linked and site_id then
	local w = supported [linked].inventory [1].weapon
	local before = state (linked, w)
	local r1 = launch (site_id, "", w)
	check ("rejects_unsupported_launcher", not r1.ok, r1.text)
	local q = pcall (engine.wpn_case_query, engine, site_id, 0)
	check ("queries_reject_unsupported_subject", not q, site_kind or "")
	local after = state (linked, w)
	check ("rejection_leaves_state_unchanged", before and after and before.has_target == after.has_target and before.target_type == after.target_type
		and before.weapon_count == after.weapon_count and #r1.events == 0, string.format ("before target %s/%s count %s, after %s/%s %s", tostring (before and before.has_target),
		tostring (before and before.target_type), tostring (before and before.weapon_count), tostring (after and after.has_target), tostring (after and after.target_type), tostring (after and after.weapon_count)))
else
	check ("rejects_unsupported_launcher", false, "no unsupported roster id or no linked launcher was found")
	check ("queries_reject_unsupported_subject", false, "no unsupported roster id or no linked launcher was found")
	check ("rejection_leaves_state_unchanged", false, "no unsupported roster id or no linked launcher was found")
end

-- a site is a valid static TARGET: the launcher is given the site through the campaign's own setter, launches (each weapon it carries in turn until one is created), and
-- gets its previous link back. A weapon aimed at a static point is a branch no natural launch reached.
local site_accepted
if linked and site_id then
	local before = state (linked, 0)
	for _, w in ipairs (supported [linked].inventory) do
		if w.decoy == 0 then
			local r = launch (linked, site_id, w.weapon)
			if r.ok and r.fields.creates >= 1 then
				local after = state (linked, 0)
				local birth
				for _, e in ipairs (r.events) do if e.kind == "birth" and not birth then birth = e end end
				site_accepted = r.fields.target_link == 1 and r.fields.restored == 1 and birth ~= nil and birth.target == site_kind and after ~= nil and after.has_target == 1 and after.target_type == before.target_type
				emit { event = "site_target", weapon = w.weapon, site = site_id, site_kind = site_kind, birth_target = birth and birth.target or "none", text = r.text }
				break
			end
		end
	end
end
check ("accepts_site_target", site_accepted == true, site_accepted == nil and "no launch at the site created a weapon" or "")

-- ---------------------------------------------------------------------------------------------------------------- empty target link (a launcher that HAS a link)
local empty_done = false
if linked then
	local before = state (linked, 0)
	for _, w in ipairs (supported [linked].inventory) do
		if w.decoy == 0 then
			local r = launch (linked, "", w.weapon)
			if r.ok and r.fields.creates and r.fields.creates >= 1 then
				local after = state (linked, 0)
				local birth, outcome
				for _, e in ipairs (r.events) do
					if e.kind == "birth" and not birth then birth = e end
					if e.kind == "launch" and not outcome then outcome = e end
				end
				check ("empty_target_establishes_no_link", r.fields.target_link_before == 1 and r.fields.target_link == 0 and outcome ~= nil and outcome.has_target == 0, r.text)
				check ("empty_target_restores_previous_link", r.fields.restored == 1 and after and after.has_target == 1 and after.target_type == before.target_type, r.text)
				check ("empty_target_birth_has_no_target", birth ~= nil and birth.target == "none", birth and tostring (birth.target) or "no birth event")
				empty_done = true
				break
			end
		end
	end
end
if not empty_done then
	check ("empty_target_establishes_no_link", false, "no launch with an empty target created a weapon")
	check ("empty_target_restores_previous_link", false, "no launch with an empty target created a weapon")
	check ("empty_target_birth_has_no_target", false, "no launch with an empty target created a weapon")
end

-- ---------------------------------------------------------------------------------------------------------------- no ammunition
local no_ammo_done = false
for _, id in ipairs (ordered) do
	local have = {}
	for _, w in ipairs (supported [id].inventory) do have [w.weapon] = true end
	for w = 1, 127 do
		local q = state (id, w)
		if q and q.weapon_count == 0 and q.warhead and q.warhead >= 0 and not have [w] and q.decoy == 0 then
			local r = launch (id, "", w)
			local outcome
			for _, e in ipairs (r.events) do if e.kind == "launch" then outcome = e end end
			check ("no_ammo_outcome", r.ok and outcome ~= nil and outcome.outcome == "no_ammo", r.text)
			check ("no_ammo_changes_nothing", r.ok and r.fields.births == 0 and r.fields.count_before == 0 and r.fields.count_after == 0, r.text)
			no_ammo_done = true
			break
		end
	end
	if no_ammo_done then break end
end
if not no_ammo_done then
	check ("no_ammo_outcome", false, "no launcher/weapon pair without ammunition was found")
	check ("no_ammo_changes_nothing", false, "no launcher/weapon pair without ammunition was found")
end

-- ---------------------------------------------------------------------------------------------------------------- targeted launches
-- one (launcher, weapon) per category, in this order; the target is the nearest enemy of the launcher
local categories = {
	{ name = "fuze_timer_gun", want = function (w) return w.guidance == 0 and (w.warhead == 7 or w.warhead == 8) and w.decoy == 0 end },
	{ name = "guided_missile_a", want = function (w) return w.guidance ~= 0 and w.decoy == 0 end },
	{ name = "decoy", want = function (w) return w.decoy ~= 0 end },
	{ name = "smoke_grenade", want = function (w) return w.warhead == 1 end },
	{ name = "unguided_ballistic", want = function (w) return w.guidance == 0 and w.aiming ~= 0 and w.decoy == 0 end },
	{ name = "unguided_other", want = function (w) return w.guidance == 0 and w.aiming == 0 and w.decoy == 0 end },
}
local seen_weapon, used_launcher = {}, {}
local made, created_ordinals, case_type, targeted_created = {}, {}, {}, 0
local function try_category (cat, require_new_guidance)
	for _, id in ipairs (ordered) do
		local w = first_weapon (id, function (x) return cat.want (x) and not seen_weapon [x.weapon] end)
		local target = w and nearest_enemy (id)
		if w and target and not used_launcher [id] then
			seen_weapon [w.weapon] = true
			used_launcher [id] = true
			local r = launch (id, target, w.weapon)
			local created = r.ok and r.fields.creates or 0
			made [#made + 1] = { category = cat.name, launcher = id, target = target, weapon = w.weapon, created = created }
			if created >= 1 then
				targeted_created = targeted_created + 1
				for _, e in ipairs (r.events) do if e.kind == "birth" then created_ordinals [e.w] = false case_type [e.w] = e.type end end
				return true
			end
		end
	end
	return false
end
for _, cat in ipairs (categories) do try_category (cat) end
-- guided missiles of other guidance types (one more per distinct weapon), up to three more
for _ = 1, 3 do try_category (categories [2]) end
emit { event = "targeted_launches", made = #made, created = targeted_created }
for i, m in ipairs (made) do emit { event = "targeted", index = i, category = m.category, launcher = m.launcher, target = m.target, weapon = m.weapon, created = m.created } end
check ("targeted_launches_created", targeted_created >= 3, string.format ("%d of %d targeted launches created weapons", targeted_created, #made))

-- ---------------------------------------------------------------------------------------------------------------- let the case weapons finish
-- weapon_movement ends a weapon when its lifetime reaches -(cruise_time + a draw of at most 0.5 * cruise_time_max_error), or earlier on a ground/water/target/bounds contact; a weapon
-- that survives ground impact (a crate, a grenade) rests until then. The fixture reports each live weapon's own lifetime and the database cruise time (wpn-live), so the longest
-- time any case weapon can still need is known from the weapons themselves, right after the launches. The wait is that horizon plus a margin, capped by `settle`; the check below
-- stays strict: every case weapon must reach its terminal fact.
local function live_weapons ()
	local live = {}
	for line in string.gmatch (engine:wpn_trace (), "[^\n]+") do
		if string.match (line, "^wpn%-live ") then
			local f = fields (line)
			live [tonumber (f.w)] = { type = f.type, on_ground = f.on_ground, lifetime = f.lifetime, cruise = f.cruise_time, error = tonumber (f.cruise_error), expires = tonumber (f.expires_in_at_least), terminal_seen = f.terminal_seen }
		end
	end
	return live
end
local horizon = 0
for w, l in pairs (live_weapons ()) do
	if created_ordinals [w] == false and l.expires then horizon = math.max (horizon, l.expires + 0.5 * (l.error or 0)) end
end
local wait = math.min (settle, horizon + 10)
emit { event = "settle_horizon", horizon = horizon, wait = wait, cap = settle, at = clock.elapsed_seconds }
local pending = 0
for _ in pairs (created_ordinals) do pending = pending + 1 end
local deadline = clock.elapsed_seconds + wait
while pending > 0 and clock.elapsed_seconds < deadline do
	step (2)
	for _, e in ipairs (drain ()) do
		if e.kind == "terminal" and created_ordinals [e.w] == false then created_ordinals [e.w] = true end
	end
	pending = 0
	for _, done in pairs (created_ordinals) do if not done then pending = pending + 1 end end
end
local live = live_weapons ()
local still = {}
for w, done in pairs (created_ordinals) do
	if not done then
		local l = live [w] or {}
		still [#still + 1] = string.format ("%d:%s(on_ground %s, lifetime %s, cruise_time %s)", w, tostring (l.type or case_type [w]), tostring (l.on_ground), tostring (l.lifetime), tostring (l.cruise))
	end
end
check ("case_weapons_reach_terminal", pending == 0 and horizon <= settle, string.format ("%d case weapons without a terminal fact at %s s (horizon %.1f s, cap %s): [%s]", pending,
	tostring (clock.elapsed_seconds), horizon, tostring (settle), table.concat (still, ";")))

drain ()
emit { event = "run", elapsed = clock.elapsed_seconds, frames = frames }
emit_report ("run", clock.elapsed_seconds, engine:wpn_trace ())
emit { event = "summary", checks = #checks, failed = failed }
out:close ()
if model_calls then
	-- the calls the campaign made of the generic host, by request mode (sorted, so the file is the same for the same run)
	local names = {}
	for mode in pairs (model_calls) do names [#names + 1] = mode end
	table.sort (names)
	local parts = {}
	for _, mode in ipairs (names) do parts [#parts + 1] = string.format ("%q:%d", mode, model_calls [mode]) end
	local model_out = assert (io.open (args.out .. ".model.json", "w"))
	model_out:write ("{\"aim_model\":\"own_model\",\"calls\":{" .. table.concat (parts, ",") .. "}}\n")
	model_out:close ()
end
io.stderr:write (string.format ("weapon-case: %d checks, %d failed, %d frames, clock %s\n", #checks, failed, frames, tostring (clock.elapsed_seconds)))
if failed > 0 then error (string.format ("weapon-case: %d checks failed", failed)) end
