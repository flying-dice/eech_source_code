-- TEST-ONLY authority controls of the host-owned model data (EEDCS-50, D3; aphavoc/.../mb_hostmodel.h). SOURCE PREPARED, NOT RUN.
--
-- Boots the campaign through require ("eech_dc") with a model provider this script wraps around (or replaces) the reference host's `host.model`, runs frames, and writes one
-- JSON Lines summary. It tests nothing about campaign policy; it shows that the campaign's collision-model answers (the radius, the two bounds and every collision triangle)
-- are the HOST'S, that a missing, declining or malformed host stops the engine with a clear error naming the provider, and that a legitimate absence of a collider is a
-- different thing from an unsupported or missing answer.
--
-- The modes (one per process: the engine is one per process):
--   reference         the reference host's answers unchanged, counted: the healthy control. Its LOS probe fold is the baseline of the changed modes.
--   vanish            every triangle the host would give is answered `end` (the model has no collision triangles): host-authored GEOMETRY change
--   inflate           the first triangle of every model is one enormous triangle and the rest `end`: host-authored GEOMETRY change the other way
--   bounds_zero       every scene's two bounds are answered all zeros: host-authored BOUNDS change
--   missing           boot without a model provider: must fail at boot with `[model-provider:missing]`
--   decline           the first triangle question raises an error: the engine must stop with `[model-provider:triangle.callback]`
--   refused           the first triangle question is answered `refused`: the engine must stop with `[model-provider:triangle.refused]`
--   bad_nil           the first catalogue answer is nil
--   bad_catalogue     the catalogue names one scene fewer than the campaign has
--   bad_identity      the first scene's main object index is off by one (the shape identity cross-check)
--   bad_scene_nan     the first scene's radius is a NaN bit pattern
--   bad_kind          the first triangle answer has no `result`
--   bad_nan           the first triangle is a NaN bit pattern
--   bad_count         the first triangle has eight numbers
--   bad_hex           the first triangle is text that is not nine bit patterns
--   bad_absent        a model the reference host says has triangles is answered `absent` (contradicts the host's own catalogue)
--   bad_end_absent    a model the reference host says is absent is answered `end`
-- A bad_* mode whose precondition the run did not reach (for the triangle ones: no triangle question was asked) is reported `unreached`, never covered.
--
-- With a DLL built with the `model-host-fixture` feature, the script can also (all explicit, all reported):
--   poison=1          overwrite the campaign's own arrays that the ORIGINAL accessors read (radius, both bounds, collision object index) right after boot, so the whole run can
--                     only have used the host's answers; the observations of a poisoned run are compared by the Root with those of the unpoisoned reference run
--   probe_scenes=a,b  after the frames, the real public accessors for those scenes (probe_depths=0,1,2,... depths): radius and bounds twice (same address and bits), and the
--                     triangle step, in the format of the original fixture's rows, with the buffer words written (0 for a FALSE: the buffer is untouched) and the rounding mode
-- and with a DLL built with `los-fixture` as well, a line-of-sight sweep between the first opposed pair of live mobiles: its fold (hash, lookup_hash, lookups, visible) is the
-- campaign observation that a host-authored change must move.
--
-- host.args: root, scenario (lebanon_retail default, georgia_retail), seconds (default 60), frame_ms (default 100), mode (default reference), out (JSON Lines path),
--            poison (0), probe_scenes (none), probe_depths (0,1,2), sweeps (LOS sweeps per probe, default 200), seed (LOS sweep seed, default 7)

io.stdout:setvbuf ("no")

local args = host.args
local root = assert (args.root, "root=<installation root>")
local seconds = tonumber (args.seconds or "60")
local frame_ms = tonumber (args.frame_ms or "100")
local mode = args.mode or "reference"
local poison = args.poison == "1"
local sweeps = tonumber (args.sweeps or "200")
local sweep_seed = tonumber (args.seed or "7")
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

local function list (text, default)
	local items = {}
	for item in string.gmatch (text or default or "", "[^,]+") do items [#items + 1] = tonumber (item) end
	return items
end

-- what the provider was asked and what it answered, by question
local counts = { catalogue = 0, scene = 0, triangle = 0, found = 0, ended = 0, absent = 0 }
local tripped = false
local reference = host.model

local function counted (op, scene, depth)
	counts [op] = (counts [op] or 0) + 1
	local answer = reference (op, scene, depth)
	if op == "triangle" then
		if answer.result == "triangle" then counts.found = counts.found + 1
		elseif answer.result == "end" then counts.ended = counts.ended + 1
		elseif answer.result == "absent" then counts.absent = counts.absent + 1 end
	end
	return answer
end

local ZERO6 = "0x00000000,0x00000000,0x00000000,0x00000000,0x00000000,0x00000000"
local BIG = { -10000, -10000, -10000, 10000, -10000, -10000, 0, 10000, 10000 }

local provider
if mode == "reference" then
	provider = counted
elseif mode == "vanish" then
	provider = function (op, scene, depth)
		local answer = counted (op, scene, depth)
		if op == "triangle" and answer.result == "triangle" then return { result = "end" } end
		return answer
	end
elseif mode == "inflate" then
	provider = function (op, scene, depth)
		local answer = counted (op, scene, depth)
		if op == "triangle" and answer.result == "triangle" then
			if depth == 0 then return { result = "triangle", p = BIG } end
			return { result = "end" }
		end
		return answer
	end
elseif mode == "bounds_zero" then
	provider = function (op, scene, depth)
		local answer = counted (op, scene, depth)
		if op == "scene" then answer = { main = answer.main, collision = answer.collision, radius = answer.radius, bb = ZERO6, bb2 = ZERO6 } end
		return answer
	end
elseif mode == "missing" then
	provider = nil
elseif mode == "decline" then
	provider = function (op, scene, depth)
		if op == "triangle" then tripped = true error ("the host declines on purpose") end
		return counted (op, scene, depth)
	end
elseif mode == "refused" then
	provider = function (op, scene, depth)
		if op == "triangle" then tripped = true return { result = "refused", reason = "declined on purpose" } end
		return counted (op, scene, depth)
	end
elseif mode == "bad_nil" then
	provider = function (op, scene, depth)
		if op == "catalogue" then tripped = true return nil end
		return counted (op, scene, depth)
	end
elseif mode == "bad_catalogue" then
	provider = function (op, scene, depth)
		local answer = counted (op, scene, depth)
		if op == "catalogue" then tripped = true return { scenes = answer.scenes - 1 } end
		return answer
	end
elseif mode == "bad_identity" then
	provider = function (op, scene, depth)
		local answer = counted (op, scene, depth)
		if op == "scene" and scene == 1 then
			tripped = true
			return { main = answer.main + 1, collision = answer.collision, radius = answer.radius, bb = answer.bb, bb2 = answer.bb2 }
		end
		return answer
	end
elseif mode == "bad_scene_nan" then
	provider = function (op, scene, depth)
		local answer = counted (op, scene, depth)
		if op == "scene" and scene == 1 then
			tripped = true
			return { main = answer.main, collision = answer.collision, radius = "0x7fc00000", bb = answer.bb, bb2 = answer.bb2 }
		end
		return answer
	end
elseif mode:sub (1, 4) == "bad_" then
	-- the first triangle question is answered wrongly (or, for bad_absent / bad_end_absent, the first one of the matching kind)
	provider = function (op, scene, depth)
		local answer = counted (op, scene, depth)
		if op ~= "triangle" or tripped then return answer end
		if mode == "bad_kind" then tripped = true return { p = BIG }
		elseif mode == "bad_nan" and answer.result == "triangle" then tripped = true return { result = "triangle", p = "0x7fc00000,0x00000000,0x00000000,0x00000000,0x00000000,0x00000000,0x00000000,0x00000000,0x00000000" }
		elseif mode == "bad_count" and answer.result == "triangle" then tripped = true return { result = "triangle", p = { 1, 2, 3, 4, 5, 6, 7, 8 } }
		elseif mode == "bad_hex" and answer.result == "triangle" then tripped = true return { result = "triangle", p = "1,2,3,4,5,6,7,8,9" }
		elseif mode == "bad_absent" and answer.result ~= "absent" then tripped = true return { result = "absent" }
		elseif mode == "bad_end_absent" and answer.result == "absent" then tripped = true return { result = "end" }
		elseif mode ~= "bad_kind" and mode ~= "bad_nan" and mode ~= "bad_count" and mode ~= "bad_hex" and mode ~= "bad_absent" and mode ~= "bad_end_absent" then
			error ("unknown bad mode " .. mode)
		end
		return answer
	end
else
	error ("unknown mode " .. mode)
end

local dc = require ("eech_dc")
dc.prepare_installation (root)
local config = {
	install_root = root, map = scenario.map, campaign_directory = "camp01", campaign = scenario.campaign, gunship = "apache",
	seed = 1, bind_mobile = host.bind_mobile, move_vehicle = host.move_vehicle, observe_loss = host.observe_loss,
	supply_reached = host.supply_reached, line_of_sight = host.line_of_sight, model = provider, sense = host.sense, aim = host.aim,
}
local ok, engine = pcall (dc.boot, config)
local function first_line (e) return (tostring (e):gsub ("\n.*", "")) end
if not ok then
	emit { event = "boot_failed", mode = mode, error = first_line (engine) }
	out:close ()
	io.stderr:write ("model-authority: boot failed: " .. first_line (engine) .. "\n")
	-- a refusal at boot is the expected result of exactly the modes that are wrong before the first frame
	local expected = { missing = "[model-provider:missing]", bad_nil = "[model-provider:catalogue.", bad_catalogue = "[model-provider:catalogue.mismatch]",
		bad_identity = "[model-provider:catalogue.identity]", bad_scene_nan = "[model-provider:scene.answer]" }
	local marker = expected [mode]
	assert (marker, "boot failed: " .. first_line (engine))
	assert (string.find (first_line (engine), marker, 1, true), "boot failed without the provider marker " .. marker .. ": " .. first_line (engine))
	return
end
assert (mode ~= "missing", "boot succeeded without a model provider")
assert (not ({ bad_nil = true, bad_catalogue = true, bad_identity = true, bad_scene_nan = true }) [mode], "boot accepted a malformed boot-time model answer: mode " .. mode)

-- the boot asked the host for the catalogue and every scene (a host that was never asked is not authoritative)
emit { event = "booted", mode = mode, scenario = args.scenario or "lebanon_retail", catalogue_asked = counts.catalogue, scenes_asked = counts.scene }

local has_fixture = type (engine.model_host_poison) == "function"
if poison then
	assert (has_fixture, "poison=1 needs a DLL built with the model-host-fixture feature: unavailable, not covered")
	engine:model_host_poison ()
	emit { event = "poisoned", note = "radius, both bounds and the collision object index of every campaign-side scene entry overwritten" }
end

local frames = math.floor (seconds * 1000 / frame_ms)
local failure
local ran = 0
for f = 1, frames do
	local frame_ok, err = pcall (engine.frame, engine, frame_ms)
	if not frame_ok then failure = first_line (err) break end
	ran = f
end

-- the campaign observation a host-authored change must move: the LOS fold of a sweep between the first opposed pair of live mobiles
local los
if not failure and type (engine.los_probe) == "function" then
	local by_kind = { ground_vehicle = {}, helicopter = {}, air_defence = {}, fixed_wing = {} }
	for _, o in ipairs (engine:objects ()) do
		local kinds = by_kind [o.kind]
		if kinds and o.alive then kinds [#kinds + 1] = o end
	end
	for _, kinds in pairs (by_kind) do table.sort (kinds, function (a, b) return a.id < b.id end) end
	local pair
	for _, spec in ipairs { { "ground_vehicle", "ground_vehicle" }, { "helicopter", "ground_vehicle" }, { "air_defence", "helicopter" } } do
		for _, a in ipairs (by_kind [spec [1]]) do
			for _, b in ipairs (by_kind [spec [2]]) do
				if not pair and a.id ~= b.id and a.side ~= b.side then pair = { a = a, b = b, label = spec [1] .. "->" .. spec [2] } end
			end
		end
	end
	if pair then
		local trace = engine:los_probe (pair.a.id, pair.b.id, sweep_seed, sweeps)
		los = trace
		emit { event = "los_probe", pair = pair.label, source = pair.a.id, target = pair.b.id, hash = trace.hash, calls = trace.calls, visible = trace.visible,
			skipped = trace.skipped, lookup_hash = trace.lookup_hash, lookups = trace.lookups }
	else
		emit { event = "los_probe_unavailable", reason = "no live opposed pair of supported mobiles" }
	end
end

-- the real public accessors, for the scenes asked for
if not failure and args.probe_scenes then
	assert (type (engine.model_host_probe) == "function", "probe_scenes needs a DLL built with the model-host-fixture feature: unavailable, not covered")
	for _, scene in ipairs (list (args.probe_scenes)) do
		for _, depth in ipairs (list (args.probe_depths, "0,1,2")) do
			local probed, text = pcall (engine.model_host_probe, engine, scene, depth)
			emit { event = "probe", scene = scene, depth = depth, ok = probed, report = probed and text or first_line (text) }
		end
	end
end

if has_fixture and not failure then
	emit { event = "counters", report = engine:model_host_counters () }
end

emit { event = "summary", mode = mode, frames = frames, frames_run = ran, failure = failure or "", poisoned = poison, catalogue = counts.catalogue, scene = counts.scene,
	triangle = counts.triangle, found = counts.found, ended = counts.ended, absent = counts.absent, los_hash = los and los.hash or "", los_lookup_hash = los and los.lookup_hash or "",
	los_visible = los and los.visible or -1, tripped = tripped }
out:close ()
io.stderr:write (string.format ("model-authority: mode=%s frames=%d/%d triangle=%d found=%d ended=%d absent=%d failure=%s\n", mode, ran, frames, counts.triangle, counts.found,
	counts.ended, counts.absent, failure or "none"))

if mode:sub (1, 4) == "bad_" or mode == "decline" or mode == "refused" then
	if not tripped then
		-- the precondition (a triangle question of the matching kind) was not reached in this run: unavailable, never counted as covered
		io.stderr:write ("model-authority: UNREACHED: the " .. mode .. " answer was never given in this run; the control is unavailable, not covered\n")
		error ("unreached: " .. mode)
	end
	assert (failure, "a malformed or declined provider answer was accepted: mode " .. mode)
	local markers = { decline = "[model-provider:triangle.callback]", refused = "[model-provider:triangle.refused]", bad_kind = "[model-provider:triangle.answer]",
		bad_nan = "[model-provider:triangle.answer]", bad_count = "[model-provider:triangle.answer]", bad_hex = "[model-provider:triangle.answer]",
		bad_absent = "[model-provider:triangle.answer]", bad_end_absent = "[model-provider:triangle.answer]" }
	assert (string.find (failure, markers [mode], 1, true), "the engine stopped without the provider marker " .. markers [mode] .. ": " .. failure)
elseif failure then
	error (failure)
end
