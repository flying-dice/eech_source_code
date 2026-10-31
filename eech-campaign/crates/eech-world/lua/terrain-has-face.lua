-- TEST-ONLY (EEDCS-47): the campaign's record predicate through the real client, with a provider that may return no hint. Needs a DLL built with the
-- `terrain-fixture` feature and the host-terrain module (the shipping module has no `terrain_face_probe`; the unmoved module refuses it).
--
-- The fixed-wing movement asks `get_3d_terrain_point_data_has_face` (fw_move.c) where the original asked `terrain_face == NULL`. Whether a face is stored is
-- the host's word (the point answer's `stored`, true when a provider says nothing), never the hint's. This boots the campaign through the public path and
-- asks the fixture to drive the real client queries at the map's centre, on a private and on the shared record, then checks the report:
--
--   mode=reference   the reference host's provider, unchanged (it returns a hint and `stored`)
--   mode=generic     the same provider with the hint and the `stored` field removed from every point answer: a host that keeps no per-record state
--
-- Both must leave a record with a face after a successful point query, and none when fresh and after a clear. Nothing here is tuned to pass.
--
-- host.args: root, scenario (lebanon_retail default, georgia_retail, georgia), mode (reference default, generic), winter (optional: omitted means no override and no
-- assertion, the season is captured from the boot; true selects Winter and asserts it, while false preserves native selection and asserts Summer)
local function refuse (message)
	error ("terrain-has-face: " .. message, 0)
end

local args = host.args
if type (args) ~= "table" then refuse ("host.args is not a table") end
local function text_arg (name, default)
	local value = args [name]
	if value == nil then value = default end
	if type (value) ~= "string" then refuse (name .. " is not a string") end
	return value
end

local root = text_arg ("root", "")
if root == "" then refuse ("root=<installation root> is required and not empty") end
local scenarios = {
	lebanon_retail = { map = "..\\common\\maps\\map5", campaign = "LEBANON.CHC" },
	georgia_retail = { map = "..\\common\\maps\\map3", campaign = "GEORGIA.CHC" },
	georgia = { map = "..\\common\\maps\\map16", campaign = "georgia.chc" },
}
local scenario = scenarios [text_arg ("scenario", "lebanon_retail")]
if scenario == nil then refuse ("scenario is not lebanon_retail, georgia_retail or georgia") end
local mode = text_arg ("mode", "reference")
if mode ~= "reference" and mode ~= "generic" then refuse ("mode is not reference or generic") end
-- Omitted means no override or assertion. True selects Winter; false only asserts Summer after native season selection.
local winter
if args.winter ~= nil then
	if args.winter ~= "true" and args.winter ~= "false" then refuse ("winter is not true or false") end
	winter = args.winter == "true"
end

local reference = host.terrain_provider (root, scenario.map)

-- The engine tells the provider the season it was booted in (`winter` on every request) and asks the map facts at boot. Both are captured here, from the
-- boot's own requests and answers, and used below: nothing asks the provider again after boot, so nothing can force a season (or a map) of this script's own.
local seen_winter, boot_facts
local function provider (request)
	if type (request.winter) ~= "boolean" then refuse ("a terrain request carries no boolean winter") end
	if seen_winter ~= nil and seen_winter ~= request.winter then refuse ("the engine's season changed between terrain requests") end
	seen_winter = request.winter
	local answer = reference (request)
	if request.op == "map" and boot_facts == nil then
		boot_facts = { min_x = answer.min_x, min_z = answer.min_z, max_x = answer.max_x, max_z = answer.max_z }
	end
	if mode == "generic" and request.op == "point" then
		answer.hint = nil
		answer.stored = nil
	end
	return answer
end

local dc = require ("eech_dc")
dc.prepare_installation (root)
local engine = dc.boot {
	install_root = root, map = scenario.map, terrain = provider, environment = host.env_provider (), campaign_directory = "camp01", campaign = scenario.campaign, gunship = "apache",
	seed = 1, winter = winter, bind_mobile = host.bind_mobile, move_vehicle = host.move_vehicle, observe_loss = host.observe_loss,
	supply_reached = host.supply_reached,
	line_of_sight = host.line_of_sight,
	sense = host.sense, aim = host.aim,
}

if seen_winter == nil or boot_facts == nil then refuse ("the boot never asked the provider for the map facts: no season or centre to probe") end
if winter ~= nil and seen_winter ~= winter then
	refuse (string.format ("the engine reported winter=%s but the boot asked for winter=%s", tostring (seen_winter), tostring (winter)))
end

-- the map's centre, from the facts the boot's own map question was answered with
local x = 0.5 * (boot_facts.min_x + boot_facts.max_x)
local z = 0.5 * (boot_facts.min_z + boot_facts.max_z)
local report = engine:terrain_face_probe (x, z)
local fields = {}
for key, value in string.gmatch (report, "(%S+)=(%S+)") do fields [key] = value end

local function expect (key, wanted)
	if tonumber (fields [key]) ~= wanted then
		refuse (string.format ("%s: %s=%s, wanted %d (report: %s)", mode, key, tostring (fields [key]), wanted, report))
	end
end
expect ("fresh", 0)
expect ("after_query", 1)
expect ("after_again", 1)
expect ("after_clear", 0)
expect ("shared_fresh", 0)
expect ("shared_after_query", 1)
expect ("shared_after_clear", 0)
if mode == "generic" then
	-- the point of the generic mode: the record has a face and no hint
	expect ("hint_after_query", 0)
else
	expect ("hint_after_query", 1)
end
io.stderr:write (string.format ("terrain-has-face: %s ok: %s", mode, report))
