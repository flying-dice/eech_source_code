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
-- host.args: root, scenario (lebanon_retail default, georgia_retail, georgia), mode (reference default, generic)
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

local reference = host.terrain_provider (root, scenario.map)
local provider = reference
if mode == "generic" then
	provider = function (request)
		local answer = reference (request)
		if request.op == "point" then
			answer.hint = nil
			answer.stored = nil
		end
		return answer
	end
end

local dc = require ("eech_dc")
dc.prepare_installation (root)
local engine = dc.boot {
	install_root = root, map = scenario.map, terrain = provider, campaign_directory = "camp01", campaign = scenario.campaign, gunship = "apache",
	seed = 1, bind_mobile = host.bind_mobile, move_vehicle = host.move_vehicle, observe_loss = host.observe_loss,
	supply_reached = host.supply_reached,
	line_of_sight = host.line_of_sight,
	sense = host.sense, aim = host.aim,
}

-- the map's centre, from the host's own facts
local facts = reference { op = "map", winter = false }
local x = 0.5 * (facts.min_x + facts.max_x)
local z = 0.5 * (facts.min_z + facts.max_z)
-- the probe takes single-precision positions: pass the nearest single the host will see
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
