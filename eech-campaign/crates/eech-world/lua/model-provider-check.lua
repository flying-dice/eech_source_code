-- Checks of the reference host's model provider (EEDCS-50, D3): host.model (op, scene, depth) and host.model_provider (store), the functions a script passes to
-- eech_dc.boot as `model`. It runs the same questions through both and writes one JSON Lines row per question (strict JSON, every write checked: json-lines.lua): the answer,
-- or the refusal's first line. It boots nothing, needs no DLL and asserts nothing about any model's contents; the test that drives it compares the rows with the store's own
-- answers.
--
-- host.args: root (an installation root with model assets), out (a new JSON Lines path), scenes (comma list, default 0,1,2,3,4,5), depths (comma list, default 0,1,2,3)

local here = debug.getinfo (1, "S").source:match ("^@?(.*[/\\])") or ""
local json = dofile (here .. "json-lines.lua")

local args = host.args
local root = assert (args.root, "root=<installation root>")

local function numbers (text, default)
	local items = {}
	for item in string.gmatch (text or default, "[^,]+") do
		local number = tonumber (item)
		if not number or number ~= math.floor (number) or number < 0 or number > 2147483647 then error ("not a whole number: " .. item, 0) end
		items [#items + 1] = number
	end
	return items
end

local function first_line (e) return (tostring (e):gsub ("\n.*", "")) end

-- the arguments are checked before the output is opened or the store is loaded
local scenes, depths = numbers (args.scenes, "0,1,2,3,4,5"), numbers (args.depths, "0,1,2,3")

local out = json.open (assert (args.out, "out=<JSON Lines path>"))
local emit = out.emit

local providers = { { via = "lazy", model = host.model }, { via = "store", model = host.model_provider (host.model_store (root)) } }

for _, p in ipairs (providers) do
	local ok, answer = pcall (p.model, "catalogue")
	emit { event = "catalogue", via = p.via, ok = ok, scenes = ok and answer.scenes or -1, error = ok and "" or first_line (answer) }
	for _, scene in ipairs (scenes) do
		local ok, a = pcall (p.model, "scene", scene)
		emit { event = "scene", via = p.via, scene = scene, ok = ok, name = ok and a.name or "", loaded = ok and a.loaded or false, collider = ok and a.collider or false,
			radius = ok and a.radius or "", bb = ok and a.bb or "", bb2 = ok and a.bb2 or "", reference_main = ok and a.reference_main or -9,
			reference_collision = ok and a.reference_collision or -9, error = ok and "" or first_line (a) }
		for _, depth in ipairs (depths) do
			local ok, t = pcall (p.model, "triangle", scene, depth)
			emit { event = "triangle", via = p.via, scene = scene, depth = depth, ok = ok, result = ok and t.result or "", p = ok and t.p or "", reason = ok and t.reason or "",
				error = ok and "" or first_line (t) }
		end
	end
	-- requests the campaign never makes are refused by name before the store is asked; a tiny or non-whole number is refused too, not taken for the integer 0
	for _, bad in ipairs {
		{ "catalogue", 1 }, { "catalogue", nil, 1 }, { "scene" }, { "scene", -1 }, { "scene", 1, 1 }, { "scene", 1.5 }, { "scene", "1" }, { "scene", 99999999 },
		{ "triangle" }, { "triangle", 1 }, { "triangle", 1, -1 }, { "triangle", 1, 0.5 }, { "triangle", 0, 0 }, { "triangle", 99999999, 0 }, { "bogus" },
		{ "scene", 1e-20 }, { "scene", -1e-20 }, { "scene", 0 / 0 }, { "scene", 1 / 0 }, { "triangle", 1, 1e-300 }, { "triangle", 1e-20, 0 }, { "scene", true },
	} do
		local ok, e = pcall (p.model, bad [1], bad [2], bad [3])
		emit { event = "bad_request", via = p.via, op = bad [1], scene = tostring (bad [2]), depth = tostring (bad [3]), ok = ok, error = ok and "" or first_line (e) }
	end
end
out.close ()
