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
--
-- THE METADATA CONTROLS (EEDCS-50, D3, L2: the campaign's common / custom model-file loaders no longer parse the model files; the host answers `object_table`, `object` and
-- `animation`, aphavoc/.../mb_hostmodel.h). All are asked while the campaign loads its scene database, so they act at the boot stage:
--   object_withheld     the first question for the MAIN object of scene 1 is answered `absent` (the host's files name no object where the campaign's scene file names one):
--                       the engine must stop on `[model-provider:object.absent]` before any compatibility row is read; nothing falls back to a file of the campaign's own
--   animation_withheld  the first animated-surface reference question (asked only for a scene loaded from an .EES file) is answered `absent`: the engine must stop on
--                       `[model-provider:animation.absent]`
--   animation_unknown   (a CHANGED-host control, healthy) the first reference the host would give is answered as a valid logical name the campaign's animation table does not hold.
--                       That is NOT a refusal: the campaign owns the decision for such a name and keeps the accepted original policy (add_new_animation with the T2 tolerance maps it to
--                       animation 0). The consequence is the scene's FINAL list: its first entry is 0.
--   animation_dropped   (a CHANGED-host control, healthy) the first scene's references are answered as an empty list; the consequence is that scene's FINAL list: empty, where the
--                       reference host's answer had at least one entry. The texture-animation frames of the entity ID digits (en_anim.c set_local_entity_id_number: it sets a digit
--                       only when the instance's scene lists that animation) follow that list; the event records whether the replaced reference is a digit animation.
--   A changed-host control is covered ONLY when (1) the change was actually given (a run never asked an animation question of the kind is UNREACHED and FAILS, never an
--   exit 0), (2) the changed definition is the scene's final one, and (3) the module's own host-profile row (model_host_animations, a model-host-fixture DLL) shows the
--   consequence above. Without that DLL entry the control is unavailable and fails. It chooses no preferred scene: the scene is the first the campaign's own load asked about.
-- THE TARGET-POINT OFFSET CONTROLS (EEDCS-52, D3: initialise_target_point_offsets asks the host for each scene's LOCAL offset, `target_point_offset`, instead of constructing the scene; the
--   campaign copies the three coordinates into its private `target_point_offsets` table, which the direct guidance consumer gd_attck.c calculate_attack_guide_intercept_point reads). All are
--   asked while the campaign initialises its weapons (twice per boot), so they act at the boot stage:
--   offsets=1          (any healthy mode) after the frames, read the HELD offsets of `offset_scenes` (default 1,2,3) with the DLL's model_host_target_offsets and call the REAL direct guidance
--                      consumer once on a live stationary mobile (model_host_attack_consumer). Covered only when the host was asked about each scene, the native question count equals the
--                      provider's, each held word triple is exactly what the provider replied for that scene, and the consumer returned exactly the held words of its target's shape. A DLL
--                      without the fixture, or no stationary mobile with a valid shape and a guided weapon, is UNAVAILABLE and fails the run, never counted as covered.
--   offsets_changed    (a CHANGED-host control, healthy; needs offsets=1) every scene's offset is answered with the constant 1.5, 2.5, -4.5. Covered only when the held words and the consumer's
--                      three coordinates are that constant and differ from what the reference host answered.
--   offsets_withheld   the first offset question is answered `refused`: the engine must stop on `[model-provider:target_point_offset.refused]`, with no fallback to the original construction
--   offsets_bad_nan    the first offset is a NaN bit pattern: `[model-provider:target_point_offset.answer]` ("p: not finite"). The Lua bridge decodes every bit pattern and refuses a non-finite one at the
--                      transport boundary, before the C code sees a value; the C code's own `[model-provider:target_point_offset.value]` check is the backstop for a provider that bypasses the bridge
--   offsets_bad_name   the first answer echoes another scene's name: `[model-provider:target_point_offset.identity]`
--   offsets_bad_end    the first answer is `end` (the question has no cursor): `[model-provider:target_point_offset.answer]`
-- THE TARGET-POINT QUERY CONTROLS (EEDCS-52, D3: get_local_entity_target_point asks the host WHICH sub-object is the target point of a live instance (`target_point_chain`), WHERE it is for
--   the campaign's CURRENT placement (`target_point_world`: the instance's viewpoint and relative scale and the live relative position, heading, pitch, roll and scale of each sub-instance
--   along the chain, as bit patterns in the `key` place) and a fixed object's position (`target_point_fixed`); the old sub-object search, world-viewpoint solver and fixed rotation are not
--   compiled). Asked by the campaign's own update loop in the frames, and for real by the diagnostic below:
--   queries=1          after the frames, call the REAL getter on live entities with the DLL's model_host_target_queries, once as the world has them and once with the leaf sub-instance's live
--                      placement changed (restored after): two vehicles with an instance, a vehicle with its instance pointer NULL for the call, fixed structures with and without a horizontal
--                      cached offset, and a fixed structure with the invalid shape for the call. Covered only when every question the native code asked is the provider's, each result is
--                      EXACTLY the host's reply (and the pose the host received is the live placement the diagnostic read), an instance whose chain the host named wrote its viewpoint and a
--                      refused-to-name one did not, the fallbacks asked the host nothing and returned the entity position bit for bit, and the changed placement moved the host's answer. A
--                      DLL without the fixture, or a case with no live entity, is UNAVAILABLE and fails the run, never counted as covered.
--   queries_changed    (a CHANGED-host control, healthy; needs queries=1) every world answer is 7.5, 8.5, -9.5 and every fixed answer 11.5, 12.5, -13.5. Covered only when the getter returned exactly
--                      those words and they differ from what the reference host answered.
--   queries_chain_withheld / queries_world_withheld / queries_fixed_withheld   the first question of that kind is answered `refused`: the engine must stop on
--                      `[model-provider:target_point_chain|world|fixed.refused]`, with no fallback to the original search, solver or rotation
--   queries_world_bad_nan / queries_fixed_bad_nan   the first position is a NaN bit pattern: `[model-provider:target_point_world|fixed.answer]` (the Lua bridge refuses it at the transport boundary)
--   queries_chain_bad_name / queries_world_bad_name / queries_fixed_bad_name   the first answer echoes another scene's name: `[model-provider:target_point_*.identity]`
--   queries_chain_too_deep   the first chain has nine levels: `[model-provider:target_point_chain.answer]`
--   queries_chain_offset_late   (a FIXTURE-PATH control; healthy otherwise) the first level's sibling position of every chain the host names is answered 4000 (outside any instance) ONLY while
--                      model_host_target_queries(mutate=true) runs, after a coherent unmutated call. The fixture's one bounded walk must refuse that row by name
--                      (`status=refused reason=chain_path_outside_instance`) BEFORE the sentinel, the mutation, the getter or any pose read: no position or fixed question is asked for it, the own
--                      chain question is the only one the provider sees, and the other rows are unaffected. A boot-only bad chain could not reach this phase. Covered only when at least one
--                      row was refused this way; a run in which none was is UNREACHED.
--   THE SIX-ROW WITNESS. The query report must be EXACTLY six lines in the C grammar (fields in the order C prints them, none missing, repeated or unknown, no stray text), named
--   instance_a, instance_b, instance_none, fixed_horizontal, fixed_flat, fixed_invalid, once each and in that order; counts are 0 or 1, the viewpoint-write verdict is the full
--   meaning (position AND attitude written, or neither), `restored` is a measured readback, and every provider event is joined to its row by operation, scene and order. This is a finite
--   six-row witness, not a general parser.
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
	bad_hex = true, bad_absent = true, bad_end_absent = true, bad_found_declared_absent = true, bad_end_declared_absent = true, object_withheld = true, animation_withheld = true,
	animation_unknown = true, animation_dropped = true, offsets_changed = true, offsets_withheld = true, offsets_bad_nan = true, offsets_bad_name = true, offsets_bad_end = true,
	queries_changed = true, queries_chain_withheld = true, queries_world_withheld = true, queries_fixed_withheld = true, queries_world_bad_nan = true, queries_fixed_bad_nan = true,
	queries_chain_bad_name = true, queries_world_bad_name = true, queries_fixed_bad_name = true, queries_chain_too_deep = true, queries_chain_offset_late = true }
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
local offsets_requested = whole (args.offsets or "0", "offsets", 0, 1) == 1
local offset_scenes = whole_list (args.offset_scenes or (offsets_requested and "1,2,3" or nil), "offset_scenes", 1, 2147483646, 64, true)
if args.offset_scenes and not offsets_requested then fail_args ("offset_scenes without offsets=1") end
if mode == "offsets_changed" and not offsets_requested then fail_args ("offsets_changed needs offsets=1: its consequence is read from the held offsets") end
local queries_requested = whole (args.queries or "0", "queries", 0, 1) == 1
if string.sub (mode, 1, 8) == "queries_" and not queries_requested then fail_args (mode .. " needs queries=1: its consequence is read from the getter's results") end
local fpu_roundings = whole_list (args.fpu_rounding or "0,1,2,3", "fpu_rounding", 0, 3, 4, true)
local fpu_scene = whole (args.fpu_scene or "1", "fpu_scene", 1, 2147483646)
local fpu_depth = whole (args.fpu_depth or "0", "fpu_depth", 0, 2147483646)
-- a refusing mode stops the engine at the refused entry, so only one initial rounding can be measured in it: decided now, before anything is written or booted
local healthy_modes = { reference = true, vanish = true, inflate = true, bounds_zero = true, absent_all = true, offsets_changed = true, queries_changed = true, queries_chain_offset_late = true }
local boot_only_modes = { missing = true, bad_nil = true, bad_catalogue = true, bad_name = true, bad_scene_nan = true, bad_collider_unloaded = true, bad_scene0_loaded = true,
	offsets_withheld = true, offsets_bad_nan = true, offsets_bad_name = true, offsets_bad_end = true }
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

local reference_counts = { catalogue = 0, scene = 0, triangle = 0, found = 0, ended = 0, absent = 0, refused = 0, target_point_offset = 0 }
local reference_offset, replied_offset = {}, {}
local CHANGED_OFFSET = "0x3fc00000,0x40200000,0xc0900000"
local CHANGED_WORLD = "0x40f00000,0x41080000,0xc1180000"
local CHANGED_FIXED = "0x41380000,0x41480000,0xc1580000"
-- the target-point questions in the order the provider was asked them, each with the pose text it received, the reference host's answer and the answer it handed back
local LATE_OFFSET = 4000
local late_armed = false
local query_ops = { target_point_chain = true, target_point_world = true, target_point_fixed = true }
local query_log, last_reference = {}, nil
local replied = { catalogue = 0, scene = 0, triangle = 0, found = 0, ended = 0, absent = 0, refused = 0, other = 0 }
local tripped = false
local changed -- the one change a changed-host control made: { scene, definition, replaced }
local reference = host.model

local animation_final_definition = {}
local function counted (op, scene, depth, kind, definition, key)
	reference_counts [op] = (reference_counts [op] or 0) + 1
	if op == "animation" and type (scene) == "number" and type (definition) == "number" then
		animation_final_definition [scene] = math.max (animation_final_definition [scene] or 0, definition)
	end
	local answer = reference (op, scene, depth, kind, definition, key)
	if query_ops [op] then last_reference = answer end
	if op == "target_point_offset" and type (answer) == "table" and answer.result == "offset" then reference_offset [scene] = answer.p end
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
	provider = function (op, scene, depth, kind, definition, key)
		local answer = counted (op, scene, depth, kind, definition, key)
		if op == "triangle" and answer.result == "triangle" then return { result = "end" } end
		return answer
	end
elseif mode == "inflate" then
	provider = function (op, scene, depth, kind, definition, key)
		local answer = counted (op, scene, depth, kind, definition, key)
		if op == "triangle" and answer.result == "triangle" then
			if depth == 0 then return { result = "triangle", p = BIG } end
			return { result = "end" }
		end
		return answer
	end
elseif mode == "bounds_zero" then
	provider = function (op, scene, depth, kind, definition, key)
		local answer = counted (op, scene, depth, kind, definition, key)
		if op == "scene" then return scene_with (answer, { bb = ZERO6, bb2 = ZERO6 }) end
		return answer
	end
elseif mode == "absent_all" or mode == "bad_found_declared_absent" or mode == "bad_end_declared_absent" then
	-- every scene is declared without a collider; the triangle answers follow the mode
	provider = function (op, scene, depth, kind, definition, key)
		local answer = counted (op, scene, depth, kind, definition, key)
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
elseif mode == "object_withheld" then
	provider = function (op, scene, depth, kind, definition, key)
		local answer = counted (op, scene, depth, kind, definition, key)
		if op == "object" and scene == 1 and kind == 0 and not tripped then tripped = true return { result = "absent", name = answer.name } end
		return answer
	end
elseif mode == "animation_withheld" then
	provider = function (op, scene, depth, kind, definition, key)
		local answer = counted (op, scene, depth, kind, definition, key)
		if op == "animation" and not tripped then tripped = true return { result = "absent", name = answer.name } end
		return answer
	end
elseif mode == "animation_unknown" or mode == "animation_dropped" then
	-- the change is made to the first CURSOR 0 reference the reference host would give a custom scene definition; everything else is the reference host's answer
	provider = function (op, scene, depth, kind, definition, key)
		local answer = counted (op, scene, depth, kind, definition, key)
		if op ~= "animation" or tripped or depth ~= 0 or answer.result ~= "name" then return answer end
		tripped = true
		changed = { scene = scene, definition = definition, replaced = answer.animation }
		if mode == "animation_unknown" then return { result = "name", animation = "NO_SUCH_ANIMATION", name = answer.name } end
		return { result = "end", name = answer.name }
	end
elseif mode == "offsets_changed" then
	provider = function (op, scene, depth, kind, definition, key)
		local answer = counted (op, scene, depth, kind, definition, key)
		if op == "target_point_offset" and answer.result == "offset" then tripped = true return { result = "offset", p = CHANGED_OFFSET, name = answer.name } end
		return answer
	end
elseif mode == "offsets_withheld" or mode == "offsets_bad_nan" or mode == "offsets_bad_name" or mode == "offsets_bad_end" then
	-- the first offset question is answered wrongly; everything else is the reference host's answer
	provider = function (op, scene, depth, kind, definition, key)
		local answer = counted (op, scene, depth, kind, definition, key)
		if op ~= "target_point_offset" or tripped then return answer end
		tripped = true
		if mode == "offsets_withheld" then return { result = "refused", reason = "withheld on purpose" }
		elseif mode == "offsets_bad_nan" then return { result = "offset", p = "0x7fc00000,0x00000000,0x00000000", name = answer.name }
		elseif mode == "offsets_bad_name" then return { result = "offset", p = answer.p, name = answer.name .. "_WRONG" }
		end
		return { result = "end", name = answer.name }
	end
elseif string.sub (mode, 1, 8) == "queries_" then
	-- a target-point question is answered as the mode says (the first of its kind for a negative mode); everything else is the reference host's answer
	local negative = {
		queries_chain_withheld = { op = "target_point_chain", answer = function () return { result = "refused", reason = "withheld on purpose" } end },
		queries_world_withheld = { op = "target_point_world", answer = function () return { result = "refused", reason = "withheld on purpose" } end },
		queries_fixed_withheld = { op = "target_point_fixed", answer = function () return { result = "refused", reason = "withheld on purpose" } end },
		queries_world_bad_nan = { op = "target_point_world", answer = function (a) return { result = "world", p = "0x7fc00000,0x00000000,0x00000000", name = a.name } end },
		queries_fixed_bad_nan = { op = "target_point_fixed", answer = function (a) return { result = "fixed", p = "0x7fc00000,0x00000000,0x00000000", name = a.name } end },
		queries_chain_bad_name = { op = "target_point_chain", answer = function (a) return { result = a.result, offsets = a.offsets, name = a.name .. "_WRONG" } end },
		queries_world_bad_name = { op = "target_point_world", answer = function (a) return { result = a.result, p = a.p, name = a.name .. "_WRONG" } end },
		queries_fixed_bad_name = { op = "target_point_fixed", answer = function (a) return { result = a.result, p = a.p, name = a.name .. "_WRONG" } end },
		queries_chain_too_deep = { op = "target_point_chain", answer = function (a) return { result = "chain", offsets = { 0, 0, 0, 0, 0, 0, 0, 0, 0 }, name = a.name } end },
	}
	provider = function (op, scene, depth, kind, definition, key)
		local answer = counted (op, scene, depth, kind, definition, key)
		if not query_ops [op] then return answer end
		if mode == "queries_changed" then
			if op == "target_point_world" and answer.result == "world" then tripped = true return { result = "world", p = CHANGED_WORLD, name = answer.name } end
			if op == "target_point_fixed" and answer.result == "fixed" then tripped = true return { result = "fixed", p = CHANGED_FIXED, name = answer.name } end
			return answer
		end
		if mode == "queries_chain_offset_late" then
			-- armed ONLY while the mutating call of the fixture runs: the first level's sibling position leaves the instance
			if op == "target_point_chain" and late_armed and answer.result == "chain" then
				local offsets = {}
				for i, v in ipairs (answer.offsets) do offsets [i] = v end
				offsets [1] = LATE_OFFSET
				tripped = true
				return { result = "chain", offsets = offsets, name = answer.name }
			end
			return answer
		end
		local spec = negative [mode]
		if op ~= spec.op or tripped then return answer end
		tripped = true
		return spec.answer (answer)
	end
elseif mode == "missing" then
	provider = nil
elseif mode == "decline" then
	provider = function (op, scene, depth, kind, definition, key)
		if op == "triangle" then tripped = true error ("the host declines on purpose") end
		return counted (op, scene, depth, kind, definition, key)
	end
elseif mode == "refused" then
	provider = function (op, scene, depth, kind, definition, key)
		if op == "triangle" then tripped = true return { result = "refused", reason = "declined on purpose" } end
		return counted (op, scene, depth, kind, definition, key)
	end
elseif mode == "bad_nil" then
	provider = function (op, scene, depth, kind, definition, key)
		if op == "catalogue" then tripped = true return nil end
		return counted (op, scene, depth, kind, definition, key)
	end
elseif mode == "bad_catalogue" then
	provider = function (op, scene, depth, kind, definition, key)
		local answer = counted (op, scene, depth, kind, definition, key)
		if op == "catalogue" then tripped = true return { scenes = answer.scenes - 1 } end
		return answer
	end
elseif mode == "bad_name" or mode == "bad_scene_nan" or mode == "bad_collider_unloaded" or mode == "bad_scene0_loaded" then
	provider = function (op, scene, depth, kind, definition, key)
		local answer = counted (op, scene, depth, kind, definition, key)
		if op ~= "scene" then return answer end
		if mode == "bad_name" and scene == 1 then tripped = true return scene_with (answer, { name = answer.name .. "_WRONG" })
		elseif mode == "bad_scene_nan" and scene == 1 then tripped = true return scene_with (answer, { radius = "0x7fc00000" })
		elseif mode == "bad_collider_unloaded" and scene == 1 then tripped = true return scene_with (answer, { loaded = false, collider = true })
		elseif mode == "bad_scene0_loaded" and scene == 0 then tripped = true return scene_with (answer, { loaded = true }) end
		return answer
	end
else
	-- the first triangle question (of the matching kind) is answered wrongly
	provider = function (op, scene, depth, kind, definition, key)
		local answer = counted (op, scene, depth, kind, definition, key)
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
	return function (op, scene, depth, kind, definition, key)
		local answer = inner (op, scene, depth, kind, definition, key)
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
				if op == "target_point_offset" and answer.result == "offset" then replied_offset [scene] = answer.p end
				if query_ops [op] then query_log [#query_log + 1] = { op = op, scene = scene, key = key, reference = last_reference, answer = answer } end
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
local metadata_markers = { object_withheld = "[model-provider:object.absent]", animation_withheld = "[model-provider:animation.absent]" }
local offset_markers = { offsets_withheld = "[model-provider:target_point_offset.refused]", offsets_bad_nan = "[model-provider:target_point_offset.answer]",
	offsets_bad_name = "[model-provider:target_point_offset.identity]", offsets_bad_end = "[model-provider:target_point_offset.answer]" }
local query_markers = { queries_chain_withheld = "[model-provider:target_point_chain.refused]", queries_world_withheld = "[model-provider:target_point_world.refused]",
	queries_fixed_withheld = "[model-provider:target_point_fixed.refused]", queries_world_bad_nan = "[model-provider:target_point_world.answer]",
	queries_fixed_bad_nan = "[model-provider:target_point_fixed.answer]", queries_chain_bad_name = "[model-provider:target_point_chain.identity]",
	queries_world_bad_name = "[model-provider:target_point_world.identity]", queries_fixed_bad_name = "[model-provider:target_point_fixed.identity]",
	queries_chain_too_deep = "[model-provider:target_point_chain.answer]" }
local expected_marker = boot_markers [mode] or triangle_markers [mode] or metadata_markers [mode] or offset_markers [mode] or query_markers [mode]

local function matches (message) return expected_marker ~= nil and string.find (message, expected_marker, 1, true) ~= nil end

-- ---------------------------------------------------------------------------------------------------------------------------------
-- boot

local dc = require ("eech_dc")
dc.prepare_installation (root)
local config = {
	install_root = root, map = scenario.map, terrain = host.terrain_provider (root, scenario.map), environment = host.env_provider (), campaign_directory = "camp01", campaign = scenario.campaign, gunship = "apache",
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
local boot_refused = { bad_nil = true, bad_catalogue = true, bad_name = true, bad_scene_nan = true, bad_collider_unloaded = true, bad_scene0_loaded = true,
	offsets_withheld = true, offsets_bad_nan = true, offsets_bad_name = true, offsets_bad_end = true }
if boot_refused [mode] then
	-- the boot succeeded: either the guarded answer was never asked for (unavailable, not covered) or it was given and ACCEPTED (the guard did not fire)
	if not tripped then error ("UNREACHED: " .. mode .. " booted without its answer ever being given; the control is unavailable, not covered", 0) end
	error ("boot accepted a malformed boot-time model answer: mode " .. mode, 0)
end

-- the boot asked the host for the catalogue and every scene (a host that was never asked is not authoritative)
emit { event = "booted", mode = mode, scenario = args.scenario or "lebanon_retail", catalogue_asked = reference_counts.catalogue, scenes_asked = reference_counts.scene }

local has_fixture = type (engine.model_host_poison) == "function"
if poison then
	assert (has_fixture, "poison=1 needs a DLL built with the model-host-fixture feature: unavailable, not covered")
	engine:model_host_poison ()
	emit { event = "poisoned", note = "radius, both bounds and the collision object index of every campaign-side scene entry overwritten" }
end

-- ---------------------------------------------------------------------------------------------------------------------------------
-- a changed-host control must have been given AND have its consequence in the campaign's own final list. A problem found here is reported by the verdict: never an exit 0.

local changed_problem
if mode == "animation_unknown" or mode == "animation_dropped" then
	if not tripped or not changed then
		changed_problem = "UNREACHED: the " .. mode .. " change was never given (the campaign asked no animation reference of a custom scene definition); the control is unavailable, not covered"
	elseif animation_final_definition [changed.scene] ~= changed.definition then
		changed_problem = string.format ("UNREACHED: the changed definition %d of scene %d is not the scene's final one (%d): its list is not what the campaign keeps", changed.definition, changed.scene,
			animation_final_definition [changed.scene] or -1)
	elseif type (engine.model_host_animations) ~= "function" then
		changed_problem = "UNREACHED: the consequence of " .. mode .. " can only be read from a DLL built with the model-host-fixture feature (model_host_animations); the control is unavailable, not covered"
	else
		local read_ok, page = pcall (engine.model_host_animations, engine, changed.scene, 1)
		if not read_ok then
			changed_problem = mode .. ": the final list of scene " .. changed.scene .. " could not be read: " .. first_line (page)
		else
			local scene, status, count, ids = string.match (tostring (page), "model%-anim scene=(%d+) name=%S+ status=(%S+) source=%S+ succeeded=%d+ count=(%d+) capacity=%d+ filled=%d+ ids=(%S+)\n")
			local first_id = ids and string.match (ids, "^(%d+)")
			local consequence
			if mode == "animation_dropped" then consequence = status == "empty" and tonumber (count) == 0
			else consequence = status == "listed" and first_id == "0" end
			emit { event = "changed_control", mode = mode, scene = changed.scene, definition = changed.definition, replaced = changed.replaced, digit_animation = type (changed.replaced) == "string" and string.find (changed.replaced, "DIGIT", 1, true) ~= nil,
				final_status = status or "", final_count = tonumber (count) or -1, final_ids = ids or "", consequence = consequence == true }
			if tonumber (scene) ~= changed.scene or not consequence then
				changed_problem = string.format ("%s: the change was given but the final list of scene %d does not show it (status %s, count %s, entries %s)", mode, changed.scene, tostring (status), tostring (count), tostring (ids))
			end
		end
	end
	if changed_problem then emit { event = "changed_control_failed", mode = mode, problem = changed_problem } end
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

-- ---------------------------------------------------------------------------------------------------------------------------------
-- the HELD target-point offsets and the direct guidance consumer (offsets=1). The campaign must hold exactly what the host replied for each requested scene (a count alone is not
-- authority), and the consumer's three coordinates are read from that table, so a changed host answer must move both.

local offsets_problem
local offsets_rows, offsets_asked = 0, -1
if offsets_requested and primary then
	offsets_problem = "UNREACHED: the requested target-point offset observation was not made: the run's primary failure at the " .. primary.stage .. " stage came first (" .. primary.message .. "); the control is unavailable, not covered"
elseif offsets_requested then
	if type (engine.model_host_target_offsets) ~= "function" or type (engine.model_host_attack_consumer) ~= "function" then
		offsets_problem = "offsets=1 needs a DLL built with the model-host-fixture feature: unavailable, not covered"
	else
		for _, scene in ipairs (offset_scenes) do
			local read_ok, report = pcall (engine.model_host_target_offsets, engine, scene, 1)
			if not read_ok then
				record_primary ("offsets", first_line (report))
				offsets_problem = offsets_problem or ("the held offset of scene " .. scene .. " could not be read: " .. first_line (report))
				break
			end
			local asked, row_scene, held = string.match (tostring (report), "model%-host%-offsets asked=(%d+)\nmodel%-host%-offset scene=(%d+) cache=(%S+)\n")
			offsets_asked = tonumber (asked) or -1
			local replied_words, reference_words = replied_offset [scene], reference_offset [scene]
			emit { event = "target_offset", scene = scene, held = held or "", replied = replied_words or "", reference = reference_words or "", asked = offsets_asked }
			if tonumber (row_scene) ~= scene or not held then
				offsets_problem = offsets_problem or ("the held-offset report is not for scene " .. scene)
			elseif not replied_words then
				offsets_problem = offsets_problem or ("UNREACHED: the host was never asked for the target-point offset of scene " .. scene .. "; the control is unavailable, not covered")
			elseif held ~= replied_words then
				offsets_problem = offsets_problem or string.format ("scene %d: the campaign holds %s but the host replied %s", scene, held, replied_words)
			elseif mode == "offsets_changed" and (held ~= CHANGED_OFFSET or held == reference_words) then
				offsets_problem = offsets_problem or string.format ("scene %d: the changed host offset %s is not what the campaign holds (%s; the reference host answered %s)", scene, CHANGED_OFFSET, held, tostring (reference_words))
			else
				offsets_rows = offsets_rows + 1
			end
		end
		if not offsets_problem and offsets_asked ~= reference_counts.target_point_offset then
			offsets_problem = string.format ("the native code asked the host %d offset questions and the provider counted %d", offsets_asked, reference_counts.target_point_offset)
		elseif not offsets_problem and offsets_asked < 1 then
			offsets_problem = "UNREACHED: the host was asked no target-point offset question; the control is unavailable, not covered"
		end
		if not primary then
			local called, report = pcall (engine.model_host_attack_consumer, engine)
			emit { event = "attack_consumer", ok = called, report = called and tostring (report) or first_line (report) }
			if not called then
				record_primary ("consumer", first_line (report))
				offsets_problem = offsets_problem or ("the direct guidance consumer failed: " .. first_line (report))
			else
				local status = string.match (report, "status=(%S+)")
				if status ~= "called" then
					offsets_problem = offsets_problem or ("UNREACHED: the direct guidance consumer could not be called (" .. tostring (string.match (report, "reason=(%S+)")) .. "); the control is unavailable, not covered")
				else
					local shape, cache, consumer, equal = string.match (report, "scene=(%d+) entity_type=%d+ weapon=%d+ cache=(%S+) consumer=(%S+) equal=(%d)")
					local replied_words = replied_offset [tonumber (shape)]
					if equal ~= "1" or cache ~= consumer then
						offsets_problem = offsets_problem or string.format ("the direct consumer returned %s but the held offset of its target's shape %s is %s", tostring (consumer), tostring (shape), tostring (cache))
					elseif replied_words and cache ~= replied_words then
						offsets_problem = offsets_problem or string.format ("the consumer's target shape %s holds %s but the host replied %s", tostring (shape), cache, replied_words)
					elseif mode == "offsets_changed" and consumer ~= CHANGED_OFFSET then
						offsets_problem = offsets_problem or string.format ("the changed host offset did not reach the direct consumer: it returned %s", tostring (consumer))
					end
				end
			end
		end
	end
end

-- ---------------------------------------------------------------------------------------------------------------------------------
-- the target-point QUERY (queries=1): the REAL getter on live entities, twice (as the world has them, then with the leaf sub-instance's placement changed). Every row must be the
-- provider's own answer, reached by the questions the provider was asked in the order the native code asked them.

local queries_problem
local queries_rows, queries_found, queries_absent, queries_fixed, queries_fallback, queries_refused = 0, 0, 0, 0, 0, 0
local function same_words (a, b) return type (a) == "string" and type (b) == "string" and string.lower (a) == string.lower (b) end

-- the C grammar of the six rows, field by field in the order C prints them
local EXPECTED_CASES = { "instance_a", "instance_b", "instance_none", "fixed_horizontal", "fixed_flat", "fixed_invalid" }
local KEYS = {
	called = { "case", "status", "type", "scene", "base", "heading", "cache", "result", "chain", "world", "fixed", "vp_written", "levels", "legacy_same", "restored", "mutation", "pose" },
	refused = { "case", "status", "reason", "scene", "level", "offset", "levels", "chain", "world", "fixed", "getter_called", "vp_touched" },
	unavailable = { "case", "status", "reason", "visited" },
}
local MUTATION_TEXT = "x+1,heading+0.25,scale_x*1.5"
local function int (v) if type (v) == "string" and string.match (v, "^%-?%d+$") then return tonumber (v) end end
local function hex_words (v, n)
	if type (v) ~= "string" then return false end
	local pattern = "^0x%x%x%x%x%x%x%x%x"
	for _ = 2, n do pattern = pattern .. ",0x%x%x%x%x%x%x%x%x" end
	return string.match (v, pattern .. "$") ~= nil
end
local function parse_line (line)
	local case, status = string.match (line, "^model%-host%-tp case=(%S+) status=(%S+)")
	if not case then return nil, "the query report has a line that is not a row: " .. string.sub (line, 1, 80) end
	local keys = KEYS [status]
	if not keys then return nil, string.format ("row %s has the unknown status %s", case, status) end
	local row, parts, order = { line = line }, {}, 0
	for key, value in string.gmatch (line, "([%w_]+)=(%S+)") do
		order = order + 1
		if row [key] ~= nil then return nil, string.format ("row %s repeats the field %s", case, key) end
		if keys [order] ~= key then return nil, string.format ("row %s has the field %s where the grammar has %s", case, key, tostring (keys [order])) end
		row [key] = value
		parts [#parts + 1] = key .. "=" .. value
	end
	if order ~= #keys then return nil, string.format ("row %s has %d fields, the grammar has %d", case, order, #keys) end
	if table.concat (parts, " ") ~= string.gsub (line, "^model%-host%-tp ", "") then return nil, string.format ("row %s has stray text", case) end
	row.case, row.status = case, status
	return row
end
local function check_domains (row)
	local function bad (what) return string.format ("row %s: %s", row.case, what) end
	if row.status == "unavailable" then
		if row.reason == "" or int (row.visited) == nil then return bad ("an unavailable row without a reason and a visit count") end
		return
	end
	local function in_set (name, ...)
		local value = int (row [name])
		for _, allowed in ipairs { ... } do if value == allowed then return true end end
		return false, bad (name .. " is outside its domain: " .. tostring (row [name]))
	end
	if row.status == "refused" then
		if row.reason ~= "chain_path_outside_instance" then return bad ("an unknown refusal " .. tostring (row.reason)) end
		for _, name in ipairs { "scene", "level", "offset", "levels" } do if int (row [name]) == nil then return bad (name .. " is not a whole number") end end
		for _, name in ipairs { "chain", "world", "fixed" } do local ok, why = in_set (name, 0, 1) if not ok then return why end end
		if row.getter_called ~= "0" or row.vp_touched ~= "0" then return bad ("a refusal must report that the getter was not called and nothing was touched") end
		return
	end
	for _, name in ipairs { "type", "scene" } do if int (row [name]) == nil then return bad (name .. " is not a whole number") end end
	for _, name in ipairs { "chain", "world", "fixed" } do local ok, why = in_set (name, 0, 1) if not ok then return why end end
	local ok, why = in_set ("vp_written", -1, 0, 1, 2) if not ok then return why end
	ok, why = in_set ("legacy_same", -1, 0, 1) if not ok then return why end
	ok, why = in_set ("restored", -1, 0, 1) if not ok then return why end
	local levels = int (row.levels)
	if levels == nil or levels < 0 or levels > 8 then return bad ("levels is outside 0..8: " .. tostring (row.levels)) end
	if not hex_words (row.base, 3) or not hex_words (row.heading, 1) or not hex_words (row.result, 3) then return bad ("base, heading or result is not exact bit patterns") end
	if row.cache ~= "-" and not hex_words (row.cache, 3) then return bad ("cache is neither - nor three bit patterns") end
	if row.pose ~= "-" and not hex_words (row.pose, 15 + 9 * levels) then return bad ("pose is neither - nor 15 + 9 * levels bit patterns") end
	if row.mutation ~= "-" and row.mutation ~= MUTATION_TEXT then return bad ("mutation is unknown: " .. tostring (row.mutation)) end
end
local function parse_rows (report)
	local lines = {}
	local text = tostring (report)
	if string.sub (text, -1) ~= "\n" then return nil, "the query report does not end with a newline" end
	for line in string.gmatch (text, "([^\n]*)\n") do lines [#lines + 1] = line end
	-- the entity scan itself failed: one row, which is an unavailable observation, not a grammar fault
	if #lines == 1 and string.match (lines [1], "^model%-host%-tp case=scan status=unavailable reason=%S+ visited=%d+$") then
		return nil, "UNREACHED: the query entities could not be scanned (" .. tostring (string.match (lines [1], "reason=(%S+)")) .. "); the control is unavailable, not covered"
	end
	if #lines ~= #EXPECTED_CASES then return nil, string.format ("the query report has %d lines, not exactly %d", #lines, #EXPECTED_CASES) end
	local rows = {}
	for i, line in ipairs (lines) do
		local row, why = parse_line (line)
		if not row then return nil, why end
		if row.case ~= EXPECTED_CASES [i] then return nil, string.format ("row %d is %s where %s is expected once and in order", i, row.case, EXPECTED_CASES [i]) end
		local domain = check_domains (row)
		if domain then return nil, domain end
		rows [i] = row
	end
	return rows
end
local function horizontal (cache)
	local x, z = string.match (cache or "", "^0x(%x+),0x%x+,0x(%x+)$")
	if not x then return nil end
	-- a float is zero if its magnitude bits are (the sign bit is the top bit of the word)
	return tonumber (x, 16) % 0x80000000 ~= 0 or tonumber (z, 16) % 0x80000000 ~= 0
end
local function same_offsets (a, b)
	if type (a) ~= "table" or type (b) ~= "table" then return a == nil and b == nil end
	if #a ~= #b then return false end
	for i = 1, #a do if a [i] ~= b [i] then return false end end
	return true
end
local first_results, first_poses = {}, {}
local mutate_start = 0
local function judge_queries (mutate, report)
	local rows, grammar = parse_rows (report)
	if not rows then return grammar end
	local events = {}
	for i = mutate_start + 1, #query_log do events [#events + 1] = query_log [i] end
	local at = 1
	local function take (op, count, scene)
		local taken = {}
		for _ = 1, count do
			local event = events [at]
			if not event or event.op ~= op then return nil, string.format ("the native code asked a %s question the provider did not see next (the provider saw %s)", op, event and event.op or "nothing") end
			if event.scene ~= scene then return nil, string.format ("the %s question the provider saw is for scene %s, not the row's scene %s", op, tostring (event.scene), tostring (scene)) end
			taken [#taken + 1] = event
			at = at + 1
		end
		return taken
	end
	for _, row in ipairs (rows) do
		emit { event = "target_query", mutate = mutate, case = row.case, status = row.status, report = row.line }
		if row.status == "unavailable" then return string.format ("UNREACHED: query case %s could not be made (%s); the control is unavailable, not covered", row.case, tostring (row.reason)) end
		local scene = int (row.scene)
		local is_instance = row.case == "instance_a" or row.case == "instance_b"
		local own
		if is_instance then
			-- the diagnostic's own chain question, asked before the call to place the change and read the pose
			local taken, why = take ("target_point_chain", 1, scene)
			if not taken then return row.case .. ": " .. why end
			own = taken [1]
		end
		if row.status == "refused" then
			-- only the late-offset witness, only in its mutating call, only for an instance row whose host chain was named, and only for the offset it armed
			if mode ~= "queries_chain_offset_late" or not mutate or not is_instance then return string.format ("%s: the fixture refused the row (%s) where nothing was meant to be refused", row.case, row.reason) end
			if own.answer.result ~= "chain" or int (row.level) ~= 0 or int (row.offset) ~= LATE_OFFSET or own.answer.offsets [1] ~= LATE_OFFSET or int (row.levels) ~= #own.answer.offsets then
				return string.format ("%s: the refusal does not describe the chain the host was made to name (level %s, offset %s, levels %s)", row.case, row.level, row.offset, row.levels)
			end
			if int (row.chain) ~= 0 or int (row.world) ~= 0 or int (row.fixed) ~= 0 then return string.format ("%s: a refused row asked the host something during its (never made) getter call", row.case) end
			queries_refused = queries_refused + 1
			queries_rows = queries_rows + 1
		else
			local chain, world, fixed = int (row.chain), int (row.world), int (row.fixed)
			local chain_events, why = take ("target_point_chain", chain, scene)
			if not chain_events then return row.case .. ": " .. why end
			local world_events, why2 = take ("target_point_world", world, scene)
			if not world_events then return row.case .. ": " .. why2 end
			local fixed_events, why3 = take ("target_point_fixed", fixed, scene)
			if not fixed_events then return row.case .. ": " .. why3 end
			queries_rows = queries_rows + 1
			local levels, vp, restored = int (row.levels), int (row.vp_written), int (row.restored)
			local function must (condition, text) if not condition then return string.format ("%s: %s", row.case, text) end end
			if is_instance then
				if chain ~= 1 or fixed ~= 0 then return string.format ("%s: an instance asked the host %d chain and %d fixed questions", row.case, chain, fixed) end
				-- the diagnostic's own chain question and the getter's own are the same question of the same host: they must have been answered alike
				if own.answer.result ~= chain_events [1].answer.result or not same_offsets (own.answer.offsets, chain_events [1].answer.offsets) then
					return string.format ("%s: the diagnostic's chain and the getter's chain differ, so the placement read is not the placement sent", row.case)
				end
				-- an out-of-range chain armed for this call must have been refused by the fixture's bounded walk, never gone on to the sentinel, the mutation, the getter or the pose read
				if mode == "queries_chain_offset_late" and mutate and own.answer.result == "chain" and own.answer.offsets [1] == LATE_OFFSET then
					return string.format ("%s: the fixture went on to use the out-of-range chain (sibling position %d) instead of refusing the row before touching anything", row.case, LATE_OFFSET)
				end
				-- the original's own search (compiled in the diagnostic build only) reaches the sub-instance the host's chain does, or both find none
				if row.legacy_same ~= "1" then return string.format ("%s: the chain the host named does not reach the sub-instance the original's search finds (legacy_same=%s)", row.case, tostring (row.legacy_same)) end
				if row.cache ~= "-" or row.restored ~= "1" then return string.format ("%s: an instance row must carry no cache and a measured restoration (cache %s, restored %s)", row.case, row.cache, row.restored) end
				if world == 1 then
					local event = world_events [1]
					if own.answer.result ~= "chain" or levels ~= #own.answer.offsets or levels < 1 then return string.format ("%s: a position was asked but the host named %s levels (the row has %d)", row.case, tostring (own.answer.result), levels) end
					if vp == 2 then return string.format ("%s: the instance viewpoint was only PARTLY written (position and attitude must both be)", row.case) end
					if vp ~= 1 then return string.format ("%s: the host named a chain but the instance viewpoint was NOT written (levels %s)", row.case, row.levels) end
					if row.pose == "-" then return row.case .. ": a position was asked but no live placement was read" end
					if not same_words (row.result, event.answer.p) then return string.format ("%s: the getter returned %s but the host replied %s", row.case, row.result, tostring (event.answer.p)) end
					local sent_words = select (2, string.gsub (tostring (event.key), ",", ",")) + 1
					if sent_words ~= 15 + 9 * levels then return string.format ("%s: the host was sent %d floats, not 15 + 9 * %d", row.case, sent_words, levels) end
					if not same_words (row.pose, event.key) then return string.format ("%s: the host was sent %s but the live placement is %s", row.case, tostring (event.key), row.pose) end
					if (row.mutation ~= "-") ~= mutate then return string.format ("%s: the row reports mutation %s in the %s call", row.case, row.mutation, mutate and "mutating" or "unmutated") end
					if mode == "queries_changed" then
						if not same_words (row.result, CHANGED_WORLD) or same_words (row.result, event.reference and event.reference.p) then
							return string.format ("%s: the changed host position %s did not reach the getter (it returned %s; the reference host answered %s)", row.case, CHANGED_WORLD, row.result, tostring (event.reference and event.reference.p))
						end
					elseif mutate then
						if same_words (row.result, first_results [row.case]) or same_words (row.pose, first_poses [row.case]) then
							return string.format ("%s: the changed live placement (%s) did not move the host's answer or the pose it was sent", row.case, row.mutation)
						end
					else
						first_results [row.case], first_poses [row.case] = row.result, row.pose
					end
					queries_found = queries_found + 1
				else
					-- the host named no chain: the entity position, bit for bit, and the instance viewpoint is NOT written (neither position nor attitude)
					if chain_events [1].answer.result ~= "absent" then return string.format ("%s: no position was asked although the host answered %s", row.case, tostring (chain_events [1].answer.result)) end
					if row.result ~= row.base or vp ~= 0 or levels ~= 0 or row.pose ~= "-" or row.mutation ~= "-" then
						return string.format ("%s: with no target point the getter must return the entity position and not write the viewpoint (%s, written %s)", row.case, row.result, row.vp_written)
					end
					queries_absent = queries_absent + 1
				end
			elseif row.case == "instance_none" or row.case == "fixed_invalid" then
				if chain ~= 0 or world ~= 0 or fixed ~= 0 then return string.format ("%s: a fallback asked the host (%d chain, %d world, %d fixed)", row.case, chain, world, fixed) end
				if row.result ~= row.base then return string.format ("%s: the fallback returned %s, not the entity position %s", row.case, row.result, row.base) end
				local bad = must (vp == -1 and levels == 0 and row.legacy_same == "-1" and restored == 1 and row.cache == "-" and row.pose == "-" and row.mutation == "-",
					"a fallback row must carry no viewpoint, chain or placement and a measured restoration of what it changed")
				if bad then return bad end
				queries_fallback = queries_fallback + 1
			else
				if chain ~= 0 or world ~= 0 or fixed ~= 1 then return string.format ("%s: a fixed object asked the host %d chain, %d world and %d fixed questions", row.case, chain, world, fixed) end
				local bad = must (vp == -1 and levels == 0 and row.legacy_same == "-1" and restored == -1 and row.pose == "-" and row.mutation == "-" and row.cache ~= "-",
					"a fixed row changes nothing: no viewpoint, chain, placement or restoration, and it carries the cached offset")
				if bad then return bad end
				local event = fixed_events [1]
				local wants_heading = horizontal (row.cache)
				if wants_heading == nil then return row.case .. ": the cached offset is not readable" end
				if wants_heading ~= (row.case == "fixed_horizontal") then return string.format ("%s: the cached offset is %s a horizontal part", row.case, wants_heading and "with" or "without") end
				local sent = wants_heading and (row.base .. "," .. row.heading) or row.base
				if not same_words (event.key, sent) then return string.format ("%s: the host was sent %s, expected %s (the heading goes only with a horizontal cached offset)", row.case, tostring (event.key), sent) end
				if not same_words (row.result, event.answer.p) then return string.format ("%s: the getter returned %s but the host replied %s", row.case, row.result, tostring (event.answer.p)) end
				if mode == "queries_changed" and (not same_words (row.result, CHANGED_FIXED) or same_words (row.result, event.reference and event.reference.p)) then
					return string.format ("%s: the changed host position %s did not reach the getter (it returned %s)", row.case, CHANGED_FIXED, row.result)
				end
				queries_fixed = queries_fixed + 1
			end
		end
	end
	if at <= #events then return string.format ("the provider saw %d more target-point questions than the native code reported", #events - at + 1) end
end

if queries_requested and primary then
	queries_problem = "UNREACHED: the requested target-point query observation was not made: the run's primary failure at the " .. primary.stage .. " stage came first (" .. primary.message .. "); the control is unavailable, not covered"
elseif queries_requested then
	if type (engine.model_host_target_queries) ~= "function" then
		queries_problem = "queries=1 needs a DLL built with the model-host-fixture feature: unavailable, not covered"
	else
		for _, mutate in ipairs { false, true } do
			mutate_start = #query_log
			-- the late offset is armed ONLY while the mutating call runs, after a coherent unmutated one
			late_armed = mode == "queries_chain_offset_late" and mutate
			local called, report = pcall (engine.model_host_target_queries, engine, mutate)
			late_armed = false
			if not called then
				record_primary ("queries", first_line (report))
				queries_problem = queries_problem or ("the target-point query failed: " .. first_line (report))
				break
			end
			queries_problem = queries_problem or judge_queries (mutate, report)
			if queries_problem then break end
		end
		if not queries_problem and not primary and queries_found < (mode == "queries_chain_offset_late" and 1 or 2) then
			queries_problem = string.format ("UNREACHED: only %d instance queries reached a host-named target point; the control is unavailable, not covered", queries_found)
		end
		if not queries_problem and mode == "queries_chain_offset_late" and queries_refused < 1 then
			queries_problem = "UNREACHED: no row was refused by the fixture's bounded walk although the out-of-range chain was armed; the fixture phase was not exercised, the control is unavailable, not covered"
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
	fpu_problem = fpu_problem or "", offsets_requested = offsets_requested, offsets_rows = offsets_rows, offsets_asked = offsets_asked, offsets_problem = offsets_problem or "",
	queries_requested = queries_requested, queries_rows = queries_rows, queries_found = queries_found, queries_absent = queries_absent, queries_fixed = queries_fixed, queries_fallback = queries_fallback, queries_refused = queries_refused,
	queries_problem = queries_problem or "" }
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
if changed_problem then error (changed_problem, 0) end
if progress_problem then error (progress_problem, 0) end
if los_problem then error (los_problem, 0) end
if probe_problem then error (probe_problem, 0) end
if fpu_problem then error (fpu_problem, 0) end
if offsets_problem then error (offsets_problem, 0) end
if queries_problem then error (queries_problem, 0) end
