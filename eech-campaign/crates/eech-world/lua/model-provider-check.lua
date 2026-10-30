-- Checks of the reference host's model provider (EEDCS-50, D3): host.model (op, scene, depth) and host.model_provider (store), the functions a script passes to
-- eech_dc.boot as `model`. It runs the same questions through both and writes one JSON Lines row per question: the answer, or the refusal's first line. It boots nothing,
-- needs no DLL and asserts nothing about any model's contents; the test that drives it compares the rows with the store's own answers.
--
-- host.args: root (an installation root with model assets), out (a new JSON Lines path), scenes (comma list, default 0,1,2,3,4,5), depths (comma list, default 0,1,2,3)

local args = host.args
local root = assert (args.root, "root=<installation root>")
local out = assert (io.open (assert (args.out, "out=<JSON Lines path>"), "w"))

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

local function numbers (text, default)
	local items = {}
	for item in string.gmatch (text or default, "[^,]+") do items [#items + 1] = tonumber (item) end
	return items
end

local function first_line (e) return (tostring (e):gsub ("\n.*", "")) end

local providers = { { via = "lazy", model = host.model }, { via = "store", model = host.model_provider (host.model_store (root)) } }

for _, p in ipairs (providers) do
	local ok, answer = pcall (p.model, "catalogue")
	emit { event = "catalogue", via = p.via, ok = ok, scenes = ok and answer.scenes or -1, error = ok and "" or first_line (answer) }
	for _, scene in ipairs (numbers (args.scenes, "0,1,2,3,4,5")) do
		local ok, a = pcall (p.model, "scene", scene)
		emit { event = "scene", via = p.via, scene = scene, ok = ok, main = ok and a.main or -9, collision = ok and a.collision or -9, radius = ok and a.radius or "",
			bb = ok and a.bb or "", bb2 = ok and a.bb2 or "", error = ok and "" or first_line (a) }
		for _, depth in ipairs (numbers (args.depths, "0,1,2,3")) do
			local ok, t = pcall (p.model, "triangle", scene, depth)
			emit { event = "triangle", via = p.via, scene = scene, depth = depth, ok = ok, result = ok and t.result or "", p = ok and t.p or "", reason = ok and t.reason or "",
				error = ok and "" or first_line (t) }
		end
	end
	-- requests the campaign never makes are refused by name before the store is asked
	for _, bad in ipairs {
		{ "catalogue", 1 }, { "catalogue", nil, 1 }, { "scene" }, { "scene", -1 }, { "scene", 1, 1 }, { "scene", 1.5 }, { "scene", "1" }, { "scene", 99999999 },
		{ "triangle" }, { "triangle", 1 }, { "triangle", 1, -1 }, { "triangle", 1, 0.5 }, { "triangle", 0, 0 }, { "triangle", 99999999, 0 }, { "bogus" },
	} do
		local ok, e = pcall (p.model, bad [1], bad [2], bad [3])
		emit { event = "bad_request", via = p.via, op = bad [1], scene = tostring (bad [2]), depth = tostring (bad [3]), ok = ok, error = ok and "" or first_line (e) }
	end
end
out:close ()
