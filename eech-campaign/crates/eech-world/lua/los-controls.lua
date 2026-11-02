-- TEST-ONLY boundary controls for the line-of-sight probe (EEDCS-42). Needs a DLL built with the `los-fixture` feature.
--
-- The probe entry takes two entity indices from Lua. It must refuse, visibly and without poisoning the engine, every index that is
-- not a live mobile: an index outside the allocated entity heap (including one below MAX_NUM_ENTITIES), a negative index, a
-- non-mobile entity (keysite, weapon, free or non-object slots), and the same entity twice. This script boots the campaign,
-- runs a few frames so the world has objects, and tries each case through pcall, printing what it got. After the refusals a valid
-- probe must still work, which shows no refusal poisoned the engine. It uses only the fixture's public methods; it asserts
-- nothing about campaign policy. A case whose precondition the world cannot supply (for example no weapon in flight yet) is
-- reported `unavailable`, never counted as covered.
--
-- host.args: root, scenario (lebanon_retail default), frames (default 20), frame_ms (default 100), out (JSON Lines path)

io.stdout:setvbuf ("no")

local args = host.args
local root = assert (args.root, "root=<installation root>")
local frames = tonumber (args.frames or "20")
local frame_ms = tonumber (args.frame_ms or "100")
local out = assert (io.open (assert (args.out, "out=<JSON Lines path>"), "w"))
local scenarios = {
	lebanon_retail = { map = "..\\common\\maps\\map5", campaign = "LEBANON.CHC" },
	georgia_retail = { map = "..\\common\\maps\\map3", campaign = "GEORGIA.CHC" },
}
local scenario = assert (scenarios [args.scenario or "lebanon_retail"], "scenario=lebanon_retail|georgia_retail")

local function emit (t)
	local parts = {}
	for k, v in pairs (t) do
		local value = type (v) == "string" and string.format ("%q", v) or tostring (v)
		parts [#parts + 1] = string.format ("%q:%s", k, value)
	end
	table.sort (parts)
	out:write ("{" .. table.concat (parts, ",") .. "}\n")
	out:flush ()
end

local dc = require ("eech_dc")
dc.prepare_installation (root)
local engine = dc.boot {
	install_root = root, map = scenario.map, terrain = host.terrain_provider (root, scenario.map), environment = host.env_provider (), campaign_directory = "camp01", campaign = scenario.campaign, gunship = "apache",
	seed = 1, bind_mobile = host.bind_mobile, move_vehicle = host.move_vehicle, observe_loss = host.observe_loss,
	supply_reached = host.supply_reached,
	line_of_sight = host.line_of_sight,
	sense = host.sense, aim = host.aim,
}
for _ = 1, frames do engine:frame (frame_ms) end

local objects = engine:objects ()
local census, by_kind, in_list = {}, {}, {}
for _, o in ipairs (objects) do
	census [o.kind] = (census [o.kind] or 0) + 1
	in_list [o.id] = true
	local list = by_kind [o.kind] or {}
	by_kind [o.kind] = list
	list [#list + 1] = o
end
for _, list in pairs (by_kind) do table.sort (list, function (a, b) return a.id < b.id end) end
local kinds = {}
for k, n in pairs (census) do kinds [#kinds + 1] = k .. "=" .. n end
table.sort (kinds)
emit { event = "census", frames = frames, kinds = table.concat (kinds, " ") }

-- two live opposed mobiles for the valid control and as the second index of each refusal case
local a, b
for _, x in ipairs (by_kind.ground_vehicle or {}) do
	for _, y in ipairs (by_kind.ground_vehicle or {}) do
		if x.id ~= y.id and x.side ~= y.side then a, b = x, y break end
	end
	if a then break end
end
assert (a and b, "los-controls: no opposed ground vehicle pair to anchor the controls")

local refused, accepted_wrongly, unavailable = 0, 0, 0
local function expect_refusal (name, source, target)
	local ok, err = pcall (function () return engine:los_probe (source, target, 1, 8) end)
	if ok then
		accepted_wrongly = accepted_wrongly + 1
		emit { event = "case", case = name, source = source, target = target, result = "ACCEPTED (unexpected)" }
	else
		refused = refused + 1
		emit { event = "case", case = name, source = source, target = target, result = "refused", error = tostring (err):gsub ("\n.*", "") }
	end
end
local function unavailable_case (name, why)
	unavailable = unavailable + 1
	emit { event = "case", case = name, result = "unavailable", why = why }
end

-- the heap: indices at and beyond the allocated number_of_entities, which is less than MAX_NUM_ENTITIES
expect_refusal ("source_index_500000", 500000, b.id)
expect_refusal ("target_index_500000", a.id, 500000)
expect_refusal ("source_index_1000000", 1000000, b.id)
expect_refusal ("source_index_int_max", 2147483647, b.id)
expect_refusal ("source_index_negative", -1, b.id)
expect_refusal ("same_entity_twice", a.id, a.id)

-- entities that are not live mobiles
local keysite = (by_kind.keysite or {}) [1]
if keysite then expect_refusal ("source_keysite", keysite.id, b.id) else unavailable_case ("source_keysite", "no keysite object") end
local weapon = (by_kind.weapon or {}) [1]
if weapon then expect_refusal ("source_weapon", weapon.id, b.id) else unavailable_case ("source_weapon", "no weapon in flight after " .. frames .. " frames") end
-- a low index the object list does not name: sector, session, force or other non-mobile entities share the heap
local other
for index = 0, 4000 do
	if not in_list [index] then other = index break end
end
if other then expect_refusal ("source_not_an_object", other, b.id) else unavailable_case ("source_not_an_object", "no unlisted index below 4000") end

-- after the refusals the engine must still answer a valid probe and a frame
local ok, trace = pcall (function () return engine:los_probe (a.id, b.id, 1, 50) end)
emit { event = "valid_after_refusals", ok = ok, calls = ok and trace.calls or "", hash = ok and trace.hash or tostring (trace) }
assert (ok, "los-controls: a valid probe failed after the refusals (engine poisoned?)")
engine:frame (frame_ms)
emit { event = "summary", refused = refused, accepted_wrongly = accepted_wrongly, unavailable = unavailable }
out:close ()
assert (accepted_wrongly == 0, "los-controls: a case that must be refused was accepted")
io.stderr:write (string.format ("los-controls: %d refused, %d accepted wrongly, %d unavailable\n", refused, accepted_wrongly, unavailable))
