-- TEST-ONLY authority controls of the host-owned model data (EEDCS-50, D3; aphavoc/.../mb_hostmodel.h). SOURCE PREPARED, NOT RUN.
--
-- Boots the campaign through require ("eech_dc") with a model provider this script wraps around (or replaces) the reference host's `host.model`, runs frames, and writes one
-- strict JSON Lines summary (json-lines.lua: every control byte escaped, every write, flush and close checked). It tests nothing about campaign policy; it shows that the
-- campaign's collision-model answers (the radius, the two bounds and every collision triangle) are the HOST'S, that a missing, declining or malformed host stops the engine
-- with a clear error naming the provider, and that a legitimate absence of a collider is a different thing from an unsupported or missing answer.
--
-- The modes (one per process: the engine is one per process):
--   reference         the reference host's answers unchanged, counted: the healthy control. Its LOS probe fold is the baseline of the changed modes.
--   vanish            every triangle the host would give is answered `end` (the model has no collision triangles): host-authored GEOMETRY change
--   inflate           the first triangle of every model is one enormous triangle and the rest `end`: host-authored GEOMETRY change the other way
--   bounds_zero       every scene's two bounds are answered all zeros: host-authored BOUNDS change
--   absent_all        every scene is declared without a collider and every triangle question is answered `absent`: a host with no collision geometry (legitimate, no refusal)
--   missing           boot without a model provider: must fail at boot with `[model-provider:missing]`
--   decline           the first triangle question raises an error: the engine must stop with `[model-provider:triangle.callback]`
--   refused           the first triangle question is answered `refused`: the engine must stop with `[model-provider:triangle.refused]`
--   bad_nil           the first catalogue answer is nil
--   bad_catalogue     the catalogue names one scene fewer than the campaign has
--   bad_name          the first scene's name is not the campaign's (the public scene-identity echo)
--   bad_scene_nan     the first scene's radius is a NaN bit pattern
--   bad_collider_unloaded   the first scene is declared not loaded but with a collider
--   bad_scene0_loaded       the invalid scene index 0 is declared loaded
--   bad_kind          the first triangle answer has no `result`
--   bad_nan           the first triangle is a NaN bit pattern
--   bad_count         the first triangle has eight numbers
--   bad_hex           the first triangle is text that is not nine bit patterns
--   bad_absent        a scene the reference host declares a collider for is answered `absent` (contradicts the host's own declaration)
--   bad_end_absent    a scene the reference host declares without a collider is answered `end`
--   bad_found_declared_absent   every scene is declared without a collider and the first triangle question is answered with a finite triangle (FOUND contradicts the declaration)
--   bad_end_declared_absent     every scene is declared without a collider and the first triangle question is answered `end`
-- A bad_* mode whose precondition the run did not reach (for the triangle ones: no triangle question of the matching kind was asked) is reported `UNREACHED` and fails the run,
-- never counted as covered.
--
-- THE STAGE. The first failure of the run is the PRIMARY one and its stage (boot, frame, probe or los) is recorded with its message (`primary_stage`, `primary_message`). A negative
-- mode is covered only when its answer was actually given AND the primary failure, at whatever stage it came (a triangle question may first be asked at boot, in a frame or by a
-- requested probe), is the engine's stop on that mode's provider marker. A failure after the primary one (every later call fails once the engine has stopped), or an unrelated
-- one, is never coverage: the run fails naming the stage and the message. A healthy mode fails on any primary failure and on any unmet requested observation.
--
-- With a DLL built with the `model-host-fixture` feature, the script can also (all explicit, all reported):
--   poison=1          overwrite the campaign's own arrays that the ORIGINAL accessors read (radius, both bounds, collision object index) right after boot, so the whole run can
--                     only have used the host's answers; the observations of a poisoned run are compared by the Root with those of the unpoisoned reference run
--   probe_scenes=a,b  after the frames, the real public accessors for those scenes (probe_depths=0,1,2,... depths): radius and bounds twice (same address and bits), and the
--                     triangle step, in the format of the original fixture's rows, with the buffer words written (0 for a FALSE: the buffer is untouched) and the rounding mode.
--                     EVERY requested probe must succeed and report its row: a refusal, a missing result or a report for another scene or depth fails the run.
-- and with a DLL built with `los-fixture` as well:
--   los=1             a line-of-sight sweep between the first opposed pair of live mobiles: its fold (hash, lookup_hash, lookups, visible) is the campaign observation that a
--                     host-authored change must move. A requested sweep that cannot be made (no fixture, no pair) fails the run.
--
-- COUNTERS. `reference_*` count what the REFERENCE host answered, before any mode above rewrote the answer; `replied_*` count the replies the provider function actually returned
-- to the campaign (after the rewrite). Neither is authority proof: they say what the script handed over. What the native code ACCEPTED is `native_counters` (the diagnostic
-- build's own counters, read from the engine), and the consequence is the LOS fold; together with the poison they are the proof, a count alone is not.
--
-- host.args: root, out (a NEW JSON Lines path), scenario (lebanon_retail default, georgia_retail), seconds (a decimal greater than 0 and at most 86400, default 60),
--            frame_ms (a whole number 1..10000, default 100; at most 1000000 frames in all), mode (default reference), poison (0 or 1, default 0), los (0 or 1, default 0),
--            sweeps (1..10000, default 200), seed (0..2147483647, default 7), probe_scenes (at most 64 distinct whole numbers >= 1), probe_depths (at most 64 whole numbers >= 0,
--            default 0,1,2; at most 1024 probes in all), fpu (0 or 1, default 0), fpu_rounding (distinct 0..3 list, default 0,1,2,3), fpu_scene (default 1), fpu_depth (default 0)
--
-- fpu=1 (model-host-fixture DLL): the COMPLETE outer floating-point state (x87 control word, x87 status word, MXCSR) around REAL public engine entries, measured from the caller's side
-- (eech_engine_model_host_fpu): for each fpu_rounding the caller state is set (that rounding mode in both units, every exception masked, NONZERO exception flags in both) and the
-- entry is called, the counters entry (ENTER/LEAVE alone, no callback) and then the probe entry (the accessors, the host callback, the Lua provider and bridge across the native
-- campaign), and the three words are compared before and after. A word that differs is a measured failure of the shipped entry's restore and fails the run; nothing is repaired
-- here. The registers are the harness's, not fegetround's: a rounding mode alone proves nothing else.
--
-- COMPLETENESS. A requested measurement is covered only when EVERY eligible requested entry was measured: two entries (counters, probe) for each requested rounding. An emitted row is
-- not a measured entry (`fpu_rows` counts rows, `fpu_measured` counts the entries whose three words were read and judged, `fpu_expected_entries` what was asked). In a mode that makes
-- the host refuse, the first probe entry takes the entry's fatal path and that is the refused entry measured; the engine is then stopped (every later entry is POISONED, not
-- eligible), so a refusing mode measures exactly ONE initial rounding per process (fpu_rounding names one; run a fresh process per rounding) and a multi-rounding request is refused
-- before anything is written or booted. fpu=1 is refused in the modes that refuse at boot (nothing can be measured). If the host's refusal comes first, at boot or in a frame, a
-- requested probe, or the line-of-sight sweep, the measurement is never made: the run reports UNREACHED, preserving that primary failure (its stage and message), and fails. A
-- covered refusal never excuses an unmade measurement.

io.stdout:setvbuf ("no")

local here = debug.getinfo (1, "S").source:match ("^@?(.*[/\\])") or ""
local json = dofile (here .. "json-lines.lua")

local args = host.args

-- ---------------------------------------------------------------------------------------------------------------------------------
-- every argument is checked, strictly, BEFORE the output is opened, the installation is prepared or anything is booted

local modes = { reference = true, vanish = true, inflate = true, bounds_zero = true, absent_all = true, missing = true, decline = true, refused = true, bad_nil = true,
	bad_catalogue = true, bad_name = true, bad_scene_nan = true, bad_collider_unloaded = true, bad_scene0_loaded = true, bad_kind = true, bad_nan = true, bad_count = true,
	bad_hex = true, bad_absent = true, bad_end_absent = true, bad_found_declared_absent = true, bad_end_declared_absent = true }
local scenarios = {
	lebanon_retail = { map = "..\\common\\maps\\map5", campaign = "LEBANON.CHC" },
	georgia_retail = { map = "..\\common\\maps\\map3", campaign = "GEORGIA.CHC" },
}

local function fail_args (message) error ("model-authority: " .. message, 0) end

-- a whole number: decimal digits only (no sign, no exponent, no hexadecimal, no spaces), within min..max
local function whole (text, name, min, max)
	if type (text) ~= "string" or not string.match (text, "^%d+$") or #text > 10 then fail_args (name .. " must be a whole number (decimal digits only), got " .. tostring (text)) end
	local number = tonumber (text)
	if number < min or number > max then fail_args (string.format ("%s must be within %d..%d, got %s", name, min, max, text)) end
	return number
end

-- a decimal greater than 0 and at most max
local function decimal (text, name, max)
	if type (text) ~= "string" or not string.match (text, "^%d+%.?%d*$") or #text > 12 then fail_args (name .. " must be a decimal number, got " .. tostring (text)) end
	local number = tonumber (text)
	if not (number > 0) or number > max then fail_args (string.format ("%s must be greater than 0 and at most %d, got %s", name, max, text)) end
	return number
end

local function whole_list (text, name, min, max, limit, distinct)
	if text == nil then return {} end
	if type (text) ~= "string" then fail_args (name .. " must be a comma list") end
	local items, seen = {}, {}
	-- every item between commas, the empty ones too: "1,,2", ",1" and "1," are errors, never skipped
	local position = 1
	while true do
		local comma = string.find (text, ",", position, true)
		local item = string.sub (text, position, (comma or #text + 1) - 1)
		local number = whole (item, name .. " item", min, max)
		if distinct and seen [number] then fail_args (name .. " names " .. item .. " twice") end
		seen [number] = true
		items [#items + 1] = number
		if #items > limit then fail_args (name .. " names more than " .. limit .. " items") end
		if not comma then break end
		position = comma + 1
	end
	return items
end

local root = args.root
if type (root) ~= "string" or root == "" then fail_args ("root=<installation root> is required") end
local mode = args.mode or "reference"
if not modes [mode] then fail_args ("unknown mode " .. tostring (mode)) end
local scenario = scenarios [args.scenario or "lebanon_retail"]
if not scenario then fail_args ("scenario=lebanon_retail|georgia_retail") end
local seconds = decimal (args.seconds or "60", "seconds", 86400)
local frame_ms = whole (args.frame_ms or "100", "frame_ms", 1, 10000)
local frames = math.ceil (seconds * 1000 / frame_ms)
if frames < 1 or frames > 1000000 then fail_args (string.format ("seconds=%s at frame_ms=%d is %d frames; between 1 and 1000000 are allowed", args.seconds or "60", frame_ms, frames)) end
local poison = whole (args.poison or "0", "poison", 0, 1) == 1
local los_requested = whole (args.los or "0", "los", 0, 1) == 1
local sweeps = whole (args.sweeps or "200", "sweeps", 1, 10000)
local sweep_seed = whole (args.seed or "7", "seed", 0, 2147483647)
local probe_scenes = whole_list (args.probe_scenes, "probe_scenes", 1, 2147483646, 64, true)
local probe_depths = whole_list (args.probe_depths or (args.probe_scenes and "0,1,2" or nil), "probe_depths", 0, 2147483646, 64, true)
if #probe_scenes * #probe_depths > 1024 then fail_args ("probe_scenes x probe_depths is more than 1024 probes") end
if args.probe_depths and #probe_scenes == 0 then fail_args ("probe_depths without probe_scenes") end
local probes_requested = #probe_scenes * #probe_depths
local fpu_requested = whole (args.fpu or "0", "fpu", 0, 1) == 1
local fpu_roundings = whole_list (args.fpu_rounding or "0,1,2,3", "fpu_rounding", 0, 3, 4, true)
local fpu_scene = whole (args.fpu_scene or "1", "fpu_scene", 1, 2147483646)
local fpu_depth = whole (args.fpu_depth or "0", "fpu_depth", 0, 2147483646)
-- a refusing mode stops the engine at the refused entry, so only one initial rounding can be measured in it: decided now, before anything is written or booted
local healthy_modes = { reference = true, vanish = true, inflate = true, bounds_zero = true, absent_all = true }
local boot_only_modes = { missing = true, bad_nil = true, bad_catalogue = true, bad_name = true, bad_scene_nan = true, bad_collider_unloaded = true, bad_scene0_loaded = true }
if fpu_requested and not healthy_modes [mode] then
	if boot_only_modes [mode] then
		fail_args ("fpu=1 cannot be measured in mode " .. mode .. ": the host is refused at boot, before any public entry exists to measure")
	end
	if #fpu_roundings ~= 1 or args.fpu_rounding == nil then
		fail_args ("fpu=1 in the refusing mode " .. mode .. " measures ONE initial rounding per process (the refused entry stops the engine): name it with fpu_rounding=N and run a fresh process for each")
	end
end
-- the entries a requested measurement must make: the counters entry and the probe entry for each rounding
local fpu_expected_entries = fpu_requested and 2 * #fpu_roundings or 0

local out = json.open (assert (args.out, "out=<JSON Lines path>"))
local emit = out.emit

local function first_line (e) return (tostring (e):gsub ("\n.*", "")) end

-- ---------------------------------------------------------------------------------------------------------------------------------
-- the provider. `reference_*` count the reference host's answers, `replied_*` what the provider function handed back

local reference_counts = { catalogue = 0, scene = 0, triangle = 0, found = 0, ended = 0, absent = 0, refused = 0 }
local replied = { catalogue = 0, scene = 0, triangle = 0, found = 0, ended = 0, absent = 0, refused = 0, other = 0 }
local tripped = false
local reference = host.model

local function counted (op, scene, depth)
	reference_counts [op] = (reference_counts [op] or 0) + 1
	local answer = reference (op, scene, depth)
	if op == "triangle" then
		if answer.result == "triangle" then reference_counts.found = reference_counts.found + 1
		elseif answer.result == "end" then reference_counts.ended = reference_counts.ended + 1
		elseif answer.result == "absent" then reference_counts.absent = reference_counts.absent + 1
		elseif answer.result == "refused" then reference_counts.refused = reference_counts.refused + 1 end
	end
	return answer
end

local ZERO6 = "0x00000000,0x00000000,0x00000000,0x00000000,0x00000000,0x00000000"
local BIG = { -10000, -10000, -10000, 10000, -10000, -10000, 0, 10000, 10000 }

-- a copy of a reference scene answer with fields replaced
local function scene_with (answer, over)
	local copy = { name = answer.name, loaded = answer.loaded, collider = answer.collider, radius = answer.radius, bb = answer.bb, bb2 = answer.bb2 }
	for k, v in pairs (over) do copy [k] = v end
	return copy
end

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
		if op == "scene" then return scene_with (answer, { bb = ZERO6, bb2 = ZERO6 }) end
		return answer
	end
elseif mode == "absent_all" or mode == "bad_found_declared_absent" or mode == "bad_end_declared_absent" then
	-- every scene is declared without a collider; the triangle answers follow the mode
	provider = function (op, scene, depth)
		local answer = counted (op, scene, depth)
		if op == "scene" then return scene_with (answer, { collider = false }) end
		if op == "triangle" then
			if mode == "absent_all" then return { result = "absent" } end
			if not tripped then
				tripped = true
				if mode == "bad_found_declared_absent" then return { result = "triangle", p = BIG } end
				return { result = "end" }
			end
			return { result = "absent" }
		end
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
elseif mode == "bad_name" or mode == "bad_scene_nan" or mode == "bad_collider_unloaded" or mode == "bad_scene0_loaded" then
	provider = function (op, scene, depth)
		local answer = counted (op, scene, depth)
		if op ~= "scene" then return answer end
		if mode == "bad_name" and scene == 1 then tripped = true return scene_with (answer, { name = answer.name .. "_WRONG" })
		elseif mode == "bad_scene_nan" and scene == 1 then tripped = true return scene_with (answer, { radius = "0x7fc00000" })
		elseif mode == "bad_collider_unloaded" and scene == 1 then tripped = true return scene_with (answer, { loaded = false, collider = true })
		elseif mode == "bad_scene0_loaded" and scene == 0 then tripped = true return scene_with (answer, { loaded = true }) end
		return answer
	end
else
	-- the first triangle question (of the matching kind) is answered wrongly
	provider = function (op, scene, depth)
		local answer = counted (op, scene, depth)
		if op ~= "triangle" or tripped then return answer end
		if mode == "bad_kind" then tripped = true return { p = BIG }
		elseif mode == "bad_nan" and answer.result == "triangle" then
			tripped = true
			return { result = "triangle", p = "0x7fc00000,0x00000000,0x00000000,0x00000000,0x00000000,0x00000000,0x00000000,0x00000000,0x00000000" }
		elseif mode == "bad_count" and answer.result == "triangle" then tripped = true return { result = "triangle", p = { 1, 2, 3, 4, 5, 6, 7, 8 } }
		elseif mode == "bad_hex" and answer.result == "triangle" then tripped = true return { result = "triangle", p = "1,2,3,4,5,6,7,8,9" }
		elseif mode == "bad_absent" and answer.result ~= "absent" then tripped = true return { result = "absent" }
		elseif mode == "bad_end_absent" and answer.result == "absent" then tripped = true return { result = "end" } end
		return answer
	end
end

-- what the provider function actually handed back to the campaign, after the mode's rewrite
local function replying (inner)
	if inner == nil then return nil end
	return function (op, scene, depth)
		local answer = inner (op, scene, depth)
		if type (answer) == "table" then
			if op == "triangle" then
				replied.triangle = replied.triangle + 1
				local result = answer.result
				if result == "triangle" then replied.found = replied.found + 1
				elseif result == "end" then replied.ended = replied.ended + 1
				elseif result == "absent" then replied.absent = replied.absent + 1
				elseif result == "refused" then replied.refused = replied.refused + 1
				else replied.other = replied.other + 1 end
			else
				replied [op] = (replied [op] or 0) + 1
			end
		else
			replied.other = replied.other + 1
		end
		return answer
	end
end

-- The FIRST failure of the run, with the stage it happened at (boot, frame, probe or los) and its message. Everything after it is secondary: once the engine has stopped, every later
-- call fails too, and those later failures are never read as coverage of anything.
local primary
local function record_primary (stage, message)
	if not primary then primary = { stage = stage, message = message } end
end

-- the provider marker an engine stop carries in each negative mode. A boot-time model answer is asked at boot; a triangle question may be asked at boot, in a frame or by a requested
-- probe, whichever the campaign reaches first: the stage is recorded, and the marker decides.
local boot_markers = { missing = "[model-provider:missing]", bad_nil = "[model-provider:catalogue.", bad_catalogue = "[model-provider:catalogue.mismatch]",
	bad_name = "[model-provider:catalogue.identity]", bad_scene_nan = "[model-provider:scene.answer]", bad_collider_unloaded = "[model-provider:scene.answer]",
	bad_scene0_loaded = "[model-provider:scene.answer]" }
local triangle_markers = { decline = "[model-provider:triangle.callback]", refused = "[model-provider:triangle.refused]", bad_kind = "[model-provider:triangle.answer]",
	bad_nan = "[model-provider:triangle.answer]", bad_count = "[model-provider:triangle.answer]", bad_hex = "[model-provider:triangle.answer]",
	bad_absent = "[model-provider:triangle.answer]", bad_end_absent = "[model-provider:triangle.answer]", bad_found_declared_absent = "[model-provider:triangle.answer]",
	bad_end_declared_absent = "[model-provider:triangle.answer]" }
local expected_marker = boot_markers [mode] or triangle_markers [mode]

local function matches (message) return expected_marker ~= nil and string.find (message, expected_marker, 1, true) ~= nil end

-- ---------------------------------------------------------------------------------------------------------------------------------
-- boot

local dc = require ("eech_dc")
dc.prepare_installation (root)
local config = {
	install_root = root, map = scenario.map, campaign_directory = "camp01", campaign = scenario.campaign, gunship = "apache",
	seed = 1, bind_mobile = host.bind_mobile, move_vehicle = host.move_vehicle, observe_loss = host.observe_loss,
	supply_reached = host.supply_reached, line_of_sight = host.line_of_sight, model = replying (provider), sense = host.sense, aim = host.aim,
}
local ok, engine = pcall (dc.boot, config)
if not ok then
	local message = first_line (engine)
	record_primary ("boot", message)
	emit { event = "boot_failed", mode = mode, stage = "boot", error = message, expected_marker = expected_marker or "", matched = matches (message), tripped = tripped, fpu_requested = fpu_requested, fpu_expected_entries = fpu_expected_entries, fpu_measured = 0, fpu_unreached = fpu_requested }
	out.close ()
	io.stderr:write ("model-authority: boot failed: " .. message .. "\n")
	-- the boot failure is the expected result of a negative mode ONLY if that mode's answer was given (`missing` gives none) and the engine stopped on that answer's marker
	if not expected_marker then error ("boot failed in a mode that expects a healthy boot: " .. message, 0) end
	if mode ~= "missing" and not tripped then error ("UNREACHED: " .. mode .. " failed before its answer was given (" .. message .. "); the control is unavailable, not covered", 0) end
	if not matches (message) then error ("boot stopped for another reason than " .. expected_marker .. ": " .. message, 0) end
	-- the refusal is covered, but a requested floating-point measurement was never made: UNREACHED, with the primary failure kept, never success
	if fpu_requested then
		error (string.format ("UNREACHED: the requested floating-point measurement of %d public entries was not made: the host was refused at the boot stage first (%s); the control is unavailable, not covered", fpu_expected_entries, message), 0)
	end
	return
end
assert (mode ~= "missing", "boot succeeded without a model provider")
local boot_refused = { bad_nil = true, bad_catalogue = true, bad_name = true, bad_scene_nan = true, bad_collider_unloaded = true, bad_scene0_loaded = true }
assert (not boot_refused [mode], "boot accepted a malformed boot-time model answer: mode " .. mode)

-- the boot asked the host for the catalogue and every scene (a host that was never asked is not authoritative)
emit { event = "booted", mode = mode, scenario = args.scenario or "lebanon_retail", catalogue_asked = reference_counts.catalogue, scenes_asked = reference_counts.scene }

local has_fixture = type (engine.model_host_poison) == "function"
if poison then
	assert (has_fixture, "poison=1 needs a DLL built with the model-host-fixture feature: unavailable, not covered")
	engine:model_host_poison ()
	emit { event = "poisoned", note = "radius, both bounds and the collision object index of every campaign-side scene entry overwritten" }
end

-- ---------------------------------------------------------------------------------------------------------------------------------
-- the frames: finite, and every one must advance the simulated clock

local failure
local ran = 0
local clock_start = engine:clock ().elapsed_seconds
local clock_last = clock_start
for f = 1, frames do
	local frame_ok, err = pcall (engine.frame, engine, frame_ms)
	if not frame_ok then failure = first_line (err) break end
	local now = engine:clock ().elapsed_seconds
	if not (now > clock_last) then failure = string.format ("no progress: frame %d of %d did not advance the simulated clock (%.6f s before, %.6f s after)", f, frames, clock_last, now) break end
	clock_last = now
	ran = f
end
if failure then record_primary ("frame", failure) end
local progress_problem
if not failure then
	local advanced, expected_advance = clock_last - clock_start, frames * frame_ms / 1000
	if advanced < expected_advance * 0.5 then progress_problem = string.format ("the clock advanced %.3f s over %d frames of %d ms (%.3f s asked)", advanced, frames, frame_ms, expected_advance) end
end

-- ---------------------------------------------------------------------------------------------------------------------------------
-- the campaign observation a host-authored change must move: the LOS fold of a sweep between the first opposed pair of live mobiles

local los, los_problem
if not primary and los_requested then
	if type (engine.los_probe) ~= "function" then
		los_problem = "los=1 needs a DLL built with the los-fixture feature: unavailable, not covered"
	else
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
			local swept, result = pcall (engine.los_probe, engine, pair.a.id, pair.b.id, sweep_seed, sweeps)
			if swept then
				los = result
				emit { event = "los_probe", pair = pair.label, source = pair.a.id, target = pair.b.id, hash = los.hash, calls = los.calls, visible = los.visible, skipped = los.skipped,
					lookup_hash = los.lookup_hash, lookups = los.lookups }
			else
				record_primary ("los", first_line (result))
				los_problem = "the line-of-sight sweep failed: " .. first_line (result)
			end
		else
			los_problem = "UNREACHED: no live opposed pair of supported mobiles for the line-of-sight sweep"
		end
	end
end

-- ---------------------------------------------------------------------------------------------------------------------------------
-- the real public accessors, for the scenes asked for: EVERY requested probe must succeed and report its own row. A probe that RAISES is the engine's own stop (a provider refusal
-- arrives here when the campaign has asked no triangle question before): the first such failure is the primary one of the run; the probes after it fail because the engine has stopped.

local probes_ok, probes_failed = 0, 0
local probe_problem
if probes_requested > 0 and not primary then
	if type (engine.model_host_probe) ~= "function" then
		probe_problem = "probe_scenes needs a DLL built with the model-host-fixture feature: unavailable, not covered"
	else
		for _, scene in ipairs (probe_scenes) do
			for _, depth in ipairs (probe_depths) do
				local probed, text = pcall (engine.model_host_probe, engine, scene, depth)
				if not probed then record_primary ("probe", first_line (text)) end
				local wanted_scene = string.format ("model-host-scene scene=%d ", scene)
				local wanted_tri = string.format ("model-host-tri scene=%d depth=%d ", scene, depth)
				local good = probed and type (text) == "string" and string.find (text, wanted_scene, 1, true) ~= nil and string.find (text, wanted_tri, 1, true) ~= nil
				if good then probes_ok = probes_ok + 1 else probes_failed = probes_failed + 1 end
				emit { event = "probe", scene = scene, depth = depth, ok = good, report = probed and tostring (text) or first_line (text) }
			end
		end
		if probes_failed > 0 or probes_ok ~= probes_requested then
			probe_problem = string.format ("%d of %d requested probes failed or reported another scene or depth", probes_failed, probes_requested)
		end
	end
end

-- ---------------------------------------------------------------------------------------------------------------------------------
-- the complete outer floating-point state around the real public entries (fpu=1)

local fpu_problem
local fpu_rows = 0      -- rows emitted by a harness call that returned
local fpu_measured = 0  -- ENTRIES whose three words were read and judged (the entry was eligible: the engine had not already stopped)
local fpu_unreached = false
if fpu_requested and primary then
	-- the host was refused (or anything else stopped the run) before the measurement could be made: the primary failure is kept, and the requested observation is UNREACHED
	fpu_unreached = true
	fpu_problem = string.format ("UNREACHED: the requested floating-point measurement of %d public entries was not made: the run's primary failure at the %s stage came first (%s); the control is unavailable, not covered",
		fpu_expected_entries, primary.stage, primary.message)
elseif fpu_requested then
	if type (engine.model_host_fpu) ~= "function" then
		fpu_unreached = true
		fpu_problem = "fpu=1 needs a DLL built with the model-host-fixture feature: unavailable, not covered"
	else
		local ENGINE_OK, ENGINE_FATAL = 0, 1
		local engine_stopped = false
		local function same (report, name) return tonumber (string.match (report, "same_" .. name .. "=(%d)")) end
		for _, kind in ipairs { 1, 0 } do
			for _, rounding in ipairs (fpu_roundings) do
				local measured, report = pcall (engine.model_host_fpu, engine, kind, fpu_scene, fpu_depth, rounding)
				if not measured then
					emit { event = "fpu", kind = kind, rounding = rounding, ok = false, report = first_line (report) }
					fpu_problem = fpu_problem or ("the floating-point harness failed (kind " .. kind .. ", rounding " .. rounding .. "): " .. first_line (report))
				else
					fpu_rows = fpu_rows + 1
					local result = tonumber (string.match (report, "result=(%-?%d+)"))
					local x87c, x87s, mx = same (report, "x87_control"), same (report, "x87_status"), same (report, "mxcsr")
					local fatal = string.match (report, "model%-host%-fpu%-fatal ([^\n]*)")
					-- once the engine has stopped (the refused entry), every later entry returns POISONED: reported, not judged
					local judged = not engine_stopped
					emit { event = "fpu", kind = kind, rounding = rounding, scene = fpu_scene, depth = fpu_depth, ok = true, result = result or -1, judged = judged,
						same_x87_control = x87c or -1, same_x87_status = x87s or -1, same_mxcsr = mx or -1, fatal = fatal or "", report = report }
					if judged then
						fpu_measured = fpu_measured + 1
						if result == ENGINE_FATAL then
							engine_stopped = true
							record_primary ("fpu", fatal or "the entry took its fatal path without a message")
						elseif result ~= ENGINE_OK then
							fpu_problem = fpu_problem or string.format ("the public entry returned %s (kind %d, rounding %d)", tostring (result), kind, rounding)
						end
						if x87c ~= 1 or x87s ~= 1 or mx ~= 1 then
							local differing = {}
							if x87c ~= 1 then differing [#differing + 1] = "x87 control word" end
							if x87s ~= 1 then differing [#differing + 1] = "x87 status word" end
							if mx ~= 1 then differing [#differing + 1] = "MXCSR" end
							fpu_problem = fpu_problem or string.format ("the caller's %s changed across the public entry (kind %d, rounding %d)", table.concat (differing, " and "), kind, rounding)
						end
					end
				end
			end
		end
		-- every requested entry must have been MEASURED, not merely reported: an entry after the engine stopped is a row, not a measurement
		if fpu_measured ~= fpu_expected_entries then
			fpu_unreached = true
			fpu_problem = (fpu_problem and (fpu_problem .. "; ") or "") .. string.format ("UNREACHED: only %d of the %d requested public entries were measured (%d rows emitted); the others were not eligible or not made, which is unavailable, not covered",
				fpu_measured, fpu_expected_entries, fpu_rows)
		end
	end
end

local native_counters = ""
if has_fixture and not primary then
	local counters_ok, counters = pcall (engine.model_host_counters, engine)
	native_counters = counters_ok and counters or ("unavailable: " .. first_line (counters))
	emit { event = "native_counters", note = "what the native code ACCEPTED from the host; see the header", report = native_counters }
end

emit { event = "summary", mode = mode, frames = frames, frames_run = ran, failure = failure or "", poisoned = poison, tripped = tripped,
	primary_stage = primary and primary.stage or "", primary_message = primary and primary.message or "", expected_marker = expected_marker or "",
	primary_matched = primary ~= nil and matches (primary.message),
	reference_catalogue = reference_counts.catalogue, reference_scene = reference_counts.scene, reference_triangle = reference_counts.triangle,
	reference_found = reference_counts.found, reference_ended = reference_counts.ended, reference_absent = reference_counts.absent, reference_refused = reference_counts.refused,
	replied_catalogue = replied.catalogue, replied_scene = replied.scene, replied_triangle = replied.triangle, replied_found = replied.found, replied_ended = replied.ended,
	replied_absent = replied.absent, replied_refused = replied.refused, replied_other = replied.other,
	los_requested = los_requested, los_hash = los and los.hash or "", los_lookup_hash = los and los.lookup_hash or "", los_visible = los and los.visible or -1,
	probes_requested = probes_requested, probes_ok = probes_ok, probes_failed = probes_failed,
	progress_problem = progress_problem or "", los_problem = los_problem or "", probe_problem = probe_problem or "", fpu_requested = fpu_requested, fpu_expected_entries = fpu_expected_entries, fpu_rows = fpu_rows, fpu_measured = fpu_measured, fpu_unreached = fpu_unreached,
	fpu_problem = fpu_problem or "" }
out.close ()
io.stderr:write (string.format ("model-authority: mode=%s frames=%d/%d reference triangle=%d found=%d ended=%d absent=%d; replied found=%d ended=%d absent=%d; primary=%s\n", mode, ran, frames,
	reference_counts.triangle, reference_counts.found, reference_counts.ended, reference_counts.absent, replied.found, replied.ended, replied.absent,
	primary and (primary.stage .. ": " .. primary.message) or "none"))

-- ---------------------------------------------------------------------------------------------------------------------------------
-- the verdict. A negative mode is covered ONLY if its answer was given AND the PRIMARY failure of the run, at whatever stage it came, is the engine's stop on that answer's marker.
-- A later or unrelated failure is not coverage; a mode whose answer was never given is UNREACHED; every requested observation of a healthy run must have been made.

if expected_marker then
	if not tripped then
		io.stderr:write ("model-authority: UNREACHED: the " .. mode .. " answer was never given in this run; the control is unavailable, not covered\n")
		error ("UNREACHED: " .. mode, 0)
	end
	if not primary then error ("a malformed or declined provider answer was accepted: mode " .. mode, 0) end
	if not matches (primary.message) then
		error (string.format("mode %s: the run stopped at the %s stage for another reason than %s: %s", mode, primary.stage, expected_marker, primary.message), 0)
	end
	-- the refusal is covered; but a requested floating-point measurement must ALSO be complete (an unmade one is UNREACHED, with the primary failure kept in the message) and a
	-- measured change of the caller's floating-point state across the refused entry fails the run. The covered refusal never excuses an unmade or incomplete measurement.
	if fpu_requested and (fpu_unreached or fpu_measured ~= fpu_expected_entries) then
		fpu_problem = fpu_problem or string.format ("UNREACHED: only %d of the %d requested public entries were measured", fpu_measured, fpu_expected_entries)
	end
	if fpu_problem then
		if fpu_unreached then io.stderr:write ("model-authority: UNREACHED: " .. fpu_problem .. "\n") end
		error (fpu_problem, 0)
	end
	return
end
if primary then
	error (string.format ("the %s stage failed: %s%s%s", primary.stage, primary.message, probe_problem and ("; " .. probe_problem) or "", fpu_problem and ("; " .. fpu_problem) or ""), 0)
end
if progress_problem then error (progress_problem, 0) end
if los_problem then error (los_problem, 0) end
if probe_problem then error (probe_problem, 0) end
if fpu_problem then error (fpu_problem, 0) end
