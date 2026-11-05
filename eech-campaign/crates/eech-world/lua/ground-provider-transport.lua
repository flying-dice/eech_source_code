-- Direct transport check of the reference host's GROUND operations (EEDCS-48, D3 G2; the campaign DLL sends them: mb_hostterrain.c). It runs in the actual eech-world
-- host against a verified private retail map, through the provider the host exposes to Lua (`host.terrain_open`), whose answers are computed by the reviewed Terrain::ground_below
-- Terrain::ground_above and Terrain::ground_impact (csrc/terrain_ref.c) over the host's own sectors and point solver. Nothing here computes a ground predicate: it sends requests and judges the
-- replies, so it tests the Lua transport (shapes, strict numbers, hint flow, refusals) and that the physical answers have the documented relations. It does not re-derive them.
--
-- Requests:   { op = "ground_above",  world = { min_x, min_z, max_x, max_z }, x, y, z, hint? }
--             { op = "ground_below",  world = { min_x, min_z, max_x, max_z }, x, y, z, hint? }
--             { op = "ground_impact", world = { min_x, min_z, max_x, max_z }, above = { x, y, z }, below = { x, y, z }, hint? }
-- Replies:    ground_above:  { above, queried, record = nil | { last_elevation, normal = { nx, ny, nz }, type, stored, hint? } }
--             ground_below:  { below, queried, record = nil | { last_elevation, normal = { nx, ny, nz }, type, stored, hint? } }
--             ground_impact: { above = { x, y, z }, below = { x, y, z }, impact = { x, y, z }, type, record = { ... } }
--
-- host.args:  terrain_dir=<the map<N>/terrain directory of a verified private installation>   (required)
--             out=<JSON Lines path>                                                          (optional: one row per check)
-- Every input number is an exact single-precision value (integers, or math.ldexp for the tiny ones): arithmetic on a double that is not an exact single would be refused by the
-- strict reader, and that would be the test's fault. A failed check raises an error that names it.

io.stdout:setvbuf ("no")

local args = host.args
local terrain_dir = args.terrain_dir
if type (terrain_dir) ~= "string" or terrain_dir == "" then
	error ("ground-provider-transport: terrain_dir=<map terrain directory> is required", 0)
end
local out = args.out and assert (io.open (args.out, "w")) or nil

-- a JSON string (RFC 8259): the quote and backslash escaped, every control byte as an escape, other bytes unchanged
local JSON_SHORT = { ["\b"] = "\\b", ["\t"] = "\\t", ["\n"] = "\\n", ["\f"] = "\\f", ["\r"] = "\\r", ["\""] = "\\\"", ["\\"] = "\\\\" }
local function json_string (text)
	local escaped = string.gsub (text, "[%z\1-\31\"\\]", function (c) return JSON_SHORT [c] or string.format ("\\u%04x", string.byte (c)) end)
	return "\"" .. escaped .. "\""
end

local checks = 0
local function check (name, condition, detail)
	checks = checks + 1
	if out then
		out:write (string.format ("{\"event\":\"check\",\"n\":%d,\"name\":%s,\"ok\":%s}\n", checks, json_string (name), condition and "true" or "false"))
		out:flush ()
	end
	if not condition then
		error ("ground-provider-transport: FAILED " .. name .. (detail ~= nil and (": " .. tostring (detail)) or ""), 0)
	end
end

local provider = host.terrain_open { terrain_dir = terrain_dir }
local facts = provider { op = "map" }
local world = { min_x = facts.min_x, min_z = facts.min_z, max_x = facts.max_x, max_z = facts.max_z }

local function above (x, y, z, hint, area)
	return provider { op = "ground_above", world = area or world, x = x, y = y, z = z, hint = hint }
end
local function below (x, y, z, hint, area)
	return provider { op = "ground_below", world = area or world, x = x, y = y, z = z, hint = hint }
end
local function impact (above, under, hint, area)
	return provider { op = "ground_impact", world = area or world, above = above, below = under, hint = hint }
end
local function refused (name, request, fragment)
	local ok, err = pcall (provider, request)
	check (name .. " is refused", not ok and string.find (tostring (err), fragment, 1, true) ~= nil, ok and "it was answered" or err)
end

local function is_record (record)
	return type (record) == "table" and type (record.last_elevation) == "number" and type (record.normal) == "table" and #record.normal == 3
		and type (record.normal [1]) == "number" and type (record.normal [2]) == "number" and type (record.normal [3]) == "number"
		and type (record.type) == "number" and record.type == math.floor (record.type) and record.type >= 0 and record.type < 62
		and type (record.stored) == "boolean" and (record.hint == nil or type (record.hint) == "string")
end
local function same_record (a, b)
	return a.last_elevation == b.last_elevation and a.type == b.type and a.stored == b.stored and a.hint == b.hint
		and a.normal [1] == b.normal [1] and a.normal [2] == b.normal [2] and a.normal [3] == b.normal [3]
end

-- a hint of the map that has its last hex digit changed: still well formed, no longer the one the map issued (its tag is the last 8 bytes)
local function altered (hint)
	local last = string.sub (hint, -1)
	return string.sub (hint, 1, -2) .. (last == "0" and "1" or "0")
end

-- ---- a point above the sector maximum: decided without a terrain read; the record is absent so the caller keeps its own ----
local high = below (1000, 60000, 1000)
check ("a high point: below is false", high.below == false)
check ("a high point: queried is false", high.queried == false)
check ("a high point: no record", high.record == nil)

-- ---- far below the ground: both clauses read, the complete record comes back ----
local low = below (1000, -5000, 1000)
check ("a low point: below is true", low.below == true)
check ("a low point: queried is true", low.queried == true)
check ("a low point: the record is complete", is_record (low.record), low.record)
check ("a low point leaves a hint at the reference land point", type (low.record.hint) == "string", "this map left no hint at (1000, 1000)")

-- ---- cold and warm: the same question carrying the hint the cold one left ----
local warm = below (1000, -5000, 1000, low.record.hint)
check ("warm: the same answer", warm.below == low.below and warm.queried == low.queried)
check ("warm: the same record", is_record (warm.record) and same_record (warm.record, low.record))

-- ---- the world area is the caller's: a smaller one makes the same physically-above point outside it, so below, with no read ----
local small = { min_x = 0, min_z = 0, max_x = 500, max_z = 500 }
local clipped = below (1000, 60000, 1000, nil, small)
check ("a smaller world: below is true", clipped.below == true)
check ("a smaller world: no terrain read", clipped.queried == false and clipped.record == nil)
local unclipped = below (1000, 60000, 1000)
check ("the map's own world: the same point is above", unclipped.below == false)

-- ---- a well-formed hint of the wrong tag is used only when the terrain is read ----
local wrong = altered (low.record.hint)
local off_world = below (facts.max_x + 1, 100, 1000, wrong)
check ("outside the world with an altered hint: still decided, no read", off_world.below == true and off_world.queried == false and off_world.record == nil)
local high_wrong = below (1000, 60000, 1000, wrong)
check ("above the sector maximum with an altered hint: still decided, no read", high_wrong.below == false and high_wrong.queried == false)
refused ("a read with an altered hint", { op = "ground_below", world = world, x = 1000, y = -5000, z = 1000, hint = wrong }, "terrain hint:")

-- ---- the above predicate: the sector maximum first, then the elevation; the same record and the same hint flow as the below predicate ----
local over = above (1000, 60000, 1000)
check ("a high point: above is true", over.above == true)
check ("a high point: above read no terrain, so no record", over.queried == false and over.record == nil)
local under = above (1000, -5000, 1000)
check ("a low point: above is false", under.above == false)
check ("a low point: above read the terrain and returns the complete record", under.queried == true and is_record (under.record), under.record)
check ("above and below agree on a high and on a low point", over.above == not high.below and under.above == not low.below)
check ("above and below leave the same record", same_record (under.record, low.record))
local warm_above = above (1000, -5000, 1000, under.record.hint)
check ("above, warm: the same answer and record", warm_above.above == under.above and warm_above.queried == true and same_record (warm_above.record, under.record))
local outside = above (1000, 60000, 1000, nil, small)
check ("above does not compare against the campaign's world area (the same point is below under that area)", outside.above == true and outside.queried == false and clipped.below == true)
local above_wrong = above (1000, 60000, 1000, wrong)
check ("above the sector maximum with an altered hint: still decided, no read", above_wrong.above == true and above_wrong.queried == false)
refused ("an above read with an altered hint", { op = "ground_above", world = world, x = 1000, y = -5000, z = 1000, hint = wrong }, "terrain hint:")

-- ---- the impact: a proper crossing, an above endpoint under the ground, a below endpoint over it ----
local ground_a = provider { op = "point", x = 1000, z = 1000 }.elevation
local ground_b = provider { op = "point", x = 1050, z = 1000 }.elevation
local crossing = impact ({ x = 1000, y = 20000, z = 1000 }, { x = 1050, y = -200, z = 1000 })
check ("a crossing: the replies are keyed positions", type (crossing.above.x) == "number" and type (crossing.below.y) == "number" and type (crossing.impact.z) == "number")
check ("a crossing: x and z of both endpoints are unchanged", crossing.above.x == 1000 and crossing.above.z == 1000 and crossing.below.x == 1050 and crossing.below.z == 1000)
check ("a crossing: the endpoints were already right", crossing.above.y == 20000 and crossing.below.y == -200)
check ("a crossing: a terrain type and a complete record", type (crossing.type) == "number" and is_record (crossing.record) and crossing.record.type == crossing.type, crossing.type)
check ("a crossing: the impact is between the endpoints in x", crossing.impact.x >= 1000 and crossing.impact.x <= 1050)
local sunk = impact ({ x = 1000, y = -8000, z = 1000 }, { x = 1050, y = -200, z = 1000 })
check ("an above endpoint under the ground is corrected to the ground plus 0.001", math.abs (sunk.above.y - (ground_a + 0.001)) < 0.01, sunk.above.y)
local floating = impact ({ x = 1000, y = 20000, z = 1000 }, { x = 1050, y = 20000, z = 1000 })
check ("a below endpoint over the ground is corrected to the ground minus 0.001", math.abs (floating.below.y - (ground_b - 0.001)) < 0.01, floating.below.y)
local warm_impact = impact ({ x = 1000, y = 20000, z = 1000 }, { x = 1050, y = -200, z = 1000 }, crossing.record.hint)
check ("an impact cold and warm: the same endpoints, impact and type",
	warm_impact.type == crossing.type and warm_impact.impact.x == crossing.impact.x and warm_impact.impact.y == crossing.impact.y
	and warm_impact.impact.z == crossing.impact.z and warm_impact.above.y == crossing.above.y and warm_impact.below.y == crossing.below.y)
check ("an impact warm: the same record", is_record (warm_impact.record) and same_record (warm_impact.record, crossing.record))

-- ---- exact numeric edge transport: signed zero and a subnormal go in and come back with their bits ----
local zero = tonumber ("0")
local negative_zero = -zero                  -- made at run time: the constant -0.0 can fold into +0.0
local tiny = math.ldexp (1, -149)            -- the smallest positive single, exactly
local edge = impact ({ x = negative_zero, y = 20000, z = tiny }, { x = 1, y = -200, z = tiny })
check ("a signed zero comes back as a signed zero", edge.above.x == 0 and 1 / edge.above.x == -math.huge, edge.above.x)
check ("a subnormal comes back exactly", edge.above.z == tiny and edge.below.z == tiny and edge.above.z > 0, edge.above.z)

-- ---- the field is looked up ONCE (a stateful getter would show a second read) ----
local reads = 0
local once = setmetatable ({ op = "ground_below", world = world, y = -5000, z = 1000 },
	{ __index = function (_, key) if key == "x" then reads = reads + 1 return 1000 end return nil end })
local answered = provider (once)
check ("a getter is read once and answered", reads == 1 and answered.below == true, reads)

-- ---- refusals: shapes, strict numbers, absent fields, the non-finite policy, a read off the terrain ----
-- Two diagnostics name a type. A field that must be a TABLE is named by the host's Lua binding (mlua LuaValue::type_name), which tells an integral Lua number from any other:
-- "integer" for 5 or 7, "number" for 5.5. A field that must be a NUMBER is named by Lua's own type (nil, string, boolean, table, function). Both are asserted as they are.
refused ("an absent world", { op = "ground_below", x = 1000, y = 0, z = 1000 }, "world must be a table, got nil")
refused ("a world that is not a table", { op = "ground_below", world = 5, x = 1000, y = 0, z = 1000 }, "world must be a table, got integer")
refused ("a world that is a non-integral number", { op = "ground_below", world = 5.5, x = 1000, y = 0, z = 1000 }, "world must be a table, got number")
refused ("a world bound that is a numeric string", { op = "ground_below", world = { min_x = "0", min_z = 0, max_x = 1, max_z = 1 }, x = 1000, y = 0, z = 1000 },
	"world.min_x: min_x must be a number, got string")
refused ("a world with an absent bound", { op = "ground_below", world = { min_x = 0, min_z = 0, max_x = 1 }, x = 1000, y = 0, z = 1000 }, "world.max_z")
refused ("an absent y", { op = "ground_below", world = world, x = 1000, z = 1000 }, "y must be a number, got nil")
refused ("a numeric string x", { op = "ground_below", world = world, x = "1000", y = 0, z = 1000 }, "x must be a number, got string")
refused ("a boolean z", { op = "ground_below", world = world, x = 1000, y = 0, z = true }, "z must be a number, got boolean")
refused ("a double that is not an exact single", { op = "ground_below", world = world, x = 0.1, y = 0, z = 1000 }, "is not an exact single-precision value")
refused ("an above question with an absent world", { op = "ground_above", x = 1000, y = 0, z = 1000 }, "world must be a table, got nil")
refused ("an above question with a numeric string y", { op = "ground_above", world = world, x = 1000, y = "0", z = 1000 }, "y must be a number, got string")
refused ("an above question with a non-finite z", { op = "ground_above", world = world, x = 1000, y = 0, z = 1 / zero }, "not finite")
refused ("an above question with a double that is not an exact single", { op = "ground_above", world = world, x = 0.1, y = 0, z = 1000 }, "is not an exact single-precision value")
refused ("an impact endpoint that is not a table", { op = "ground_impact", world = world, above = 5, below = { x = 1, y = 1, z = 1 } }, "above must be a table, got integer")
refused ("an impact endpoint with an absent coordinate", { op = "ground_impact", world = world, above = { x = 1, y = 1 }, below = { x = 1, y = 1, z = 1 } }, "above.z")
refused ("a hint that is a number", { op = "ground_below", world = world, x = 1000, y = 0, z = 1000, hint = 7 }, "hint must be a string or nil, got integer")
refused ("a hint that is not of this host's form", { op = "ground_below", world = world, x = 1000, y = 0, z = 1000, hint = "zz" }, "terrain hint:")
refused ("a non-finite x", { op = "ground_below", world = world, x = 1 / zero, y = 0, z = 1000 }, "not finite")
refused ("a NaN y", { op = "ground_below", world = world, x = 1000, y = zero / zero, z = 1000 }, "not finite")
refused ("a non-finite world bound", { op = "ground_below", world = { min_x = 0, min_z = 0, max_x = 1 / zero, max_z = 1 }, x = 1000, y = 0, z = 1000 }, "not finite")
local wide = { min_x = -1000000, min_z = -1000000, max_x = 1000000, max_z = 1000000 }
refused ("a sector-maximum read off the terrain", { op = "ground_below", world = wide, x = facts.max_x + 1, y = -100, z = 1000 }, "sector maximum is read off the terrain")
refused ("an above sector-maximum read off the terrain", { op = "ground_above", world = wide, x = facts.max_x + 1, y = -100, z = 1000 }, "sector maximum is read off the terrain")
refused ("an impact endpoint off the terrain", { op = "ground_impact", world = world, above = { x = facts.max_x + 1, y = 100, z = 1000 }, below = { x = 1000, y = -100, z = 1000 } },
	"sector maximum is read off the terrain")

if out then out:close () end
io.stderr:write (string.format ("ground-provider-transport: %d checks passed\n", checks))
