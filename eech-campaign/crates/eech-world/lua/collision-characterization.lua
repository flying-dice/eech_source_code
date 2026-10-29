-- TEST-ONLY object and tree collision characterisation (EEDCS-49, D3 G3a). Needs a DLL built with the `collision-fixture` feature; the shipping
-- module has no `collision_probe` method.
--
-- It boots the campaign through the public path (require ("eech_dc")) with the reference host's callbacks, optionally runs frames, and then calls
-- the one fixture entry, engine:collision_probe (seed, entity_cases, tree_cases), which runs the ORIGINAL get_line_of_sight_collision_entity and
-- get_line_of_sight_collision_tree against the world the campaign's own boot loaded (see csrc/eech_collision_fixture.c). The report is `key=value`
-- text lines; each becomes one JSON Lines event. Raw coordinates appear only in that runtime output (float bit patterns in hex), kept outside Git.
--
-- The probe is repeated (`repeats`, default 2) in the same process with no frames between: the reports must be byte identical, which shows the probe
-- is a pure function of the seed and the loaded world, and that it leaves the shared terrain record, the game's random stream and the collision
-- candidates' state as it found them (the report states all three; each must hold). A difference fails the run; nothing is tuned to pass.
--
-- A report is accepted only as a whole (read_report): every row once, with exactly its keys, each of its exact type, counts that agree with the request
-- and with each other, and both folds present. A malformed or incomplete report records `report_invalid` and fails the run; no repeat_identical,
-- coverage or state_digest row follows it. `col-physical` is the PHYSICAL fold; `col-legacy` is the LEGACY diagnostic fold (heap slot of a hit and the
-- terrain record's private bookkeeping), never compared with a build that does not keep them.
--
-- WHAT RAN IS STATED, NOT ASSUMED: the `coverage` row names every family the world offered no case for (no moving mobile, no building, no geometry
-- target, no tree data, no hit of a kind, ...) and the final `done` row repeats the list. A run with gaps still exits 0 unless `require_coverage=yes`, in
-- which case any gap fails the run; either way the gap is in the record and the run never reports a bare pass.
--
-- `probe=no` is the control: the same frames, no probe call, and the same state digest at the end (the object list's identity, kind, side and alive flag
-- in id order), a coarse witness that the probe did not disturb the campaign, not a proof.
--
-- host.args (every one checked before anything is opened, required or booted; an unknown key, a malformed number or an out-of-range value is refused):
--   root=<installation root>  out=<JSON Lines path>                                              required
--   scenario  lebanon_retail (default), georgia_retail or georgia
--   seed      1 to 4294967295 (default 20261029): the private generator's seed; 0 is refused because the generator treats it as 1
--   entity_cases  0 to 20000 (default 2000): seeded random entity cases (the structured cases always run)
--   tree_cases    0 to 20000 (default 2000): seeded random tree cases (the structured cases always run)
--   repeats   1 to 5 (default 2): how many times the probe is run in a row
--   seconds   0 to 3600 (default 120): simulated seconds of frames before the probe (0 probes the freshly booted world, where nothing is moving)
--   after     0 to 3600 (default 0): simulated seconds of frames after the probe, before the digest
--   frame_ms  1 to 1000 (default 100)
--   probe     yes (default) or no (the control)
--   require_coverage  no (default) or yes

io.stdout:setvbuf ("no")

local args = host.args

local function refuse (message)
	error ("collision-characterization: " .. message, 0)
end

local known = { root = true, out = true, scenario = true, seed = true, entity_cases = true, tree_cases = true, repeats = true, seconds = true,
	after = true, frame_ms = true, probe = true, require_coverage = true }
for key in pairs (args) do
	if not known [key] then refuse ("unknown argument '" .. tostring (key) .. "'") end
end

local function whole_arg (name, value, default, minimum, maximum)
	if value == nil then return default end
	if type (value) ~= "string" or not string.match (value, "^%d+$") then
		refuse (name .. " must be a whole number from " .. minimum .. " to " .. maximum .. ", got '" .. tostring (value) .. "'")
	end
	local n = tonumber (value)
	if n < minimum or n > maximum then
		refuse (name .. " must be a whole number from " .. minimum .. " to " .. maximum .. ", got '" .. value .. "'")
	end
	return n
end

local function choice_arg (name, value, default, allowed)
	if value == nil then return default end
	for _, option in ipairs (allowed) do if value == option then return value end end
	refuse (name .. " must be one of " .. table.concat (allowed, ", ") .. ", got '" .. tostring (value) .. "'")
end

local root = args.root
if type (root) ~= "string" or root == "" then refuse ("root=<installation root> is required") end
local out_path = args.out
if type (out_path) ~= "string" or out_path == "" then refuse ("out=<JSON Lines path> is required") end

local scenarios = {
	lebanon_retail = { map = "..\\common\\maps\\map5", campaign = "LEBANON.CHC" },
	georgia_retail = { map = "..\\common\\maps\\map3", campaign = "GEORGIA.CHC" },
	georgia = { map = "..\\common\\maps\\map16", campaign = "georgia.chc" },
}
local scenario_name = args.scenario == nil and "lebanon_retail" or args.scenario
local scenario = scenarios [scenario_name]
if scenario == nil then refuse ("scenario must be lebanon_retail, georgia_retail or georgia, got '" .. tostring (scenario_name) .. "'") end

local seed = whole_arg ("seed", args.seed, 20261029, 1, 4294967295)
local entity_cases = whole_arg ("entity_cases", args.entity_cases, 2000, 0, 20000)
local tree_cases = whole_arg ("tree_cases", args.tree_cases, 2000, 0, 20000)
local repeats = whole_arg ("repeats", args.repeats, 2, 1, 5)
local seconds = whole_arg ("seconds", args.seconds, 120, 0, 3600)
local after = whole_arg ("after", args.after, 0, 0, 3600)
local frame_ms = whole_arg ("frame_ms", args.frame_ms, 100, 1, 1000)
local probe = choice_arg ("probe", args.probe, "yes", { "yes", "no" })
local require_coverage = choice_arg ("require_coverage", args.require_coverage, "no", { "no", "yes" })

local MAX_FRAMES = 72000
local frames_before = math.floor (seconds * 1000 / frame_ms)
local frames_after = math.floor (after * 1000 / frame_ms)
if frames_before + frames_after > MAX_FRAMES then
	refuse ("seconds and after need " .. (frames_before + frames_after) .. " frames of " .. frame_ms .. " ms; the most this driver runs is " .. MAX_FRAMES)
end

-- Only now, with every argument good, is anything opened, required or booted.
local out = assert (io.open (out_path, "w"))

-- A JSON string (RFC 8259). Lua's %q writes LUA source, not JSON: it leaves a tab (and every other control byte) raw, which a strict JSON parser refuses, and
-- writes a newline as a backslash and a real newline. So: the quote and the backslash are escaped, every control byte 0x00 to 0x1f becomes its short escape
-- (\b \t \n \f \r) or \u00XX, and every other byte, UTF-8 included, is written unchanged. Nothing is dropped or flattened.
local JSON_SHORT = { ["\b"] = "\\b", ["\t"] = "\\t", ["\n"] = "\\n", ["\f"] = "\\f", ["\r"] = "\\r", ["\""] = "\\\"", ["\\"] = "\\\\" }

local function json_string (text)
	local escaped = string.gsub (text, "[%z\1-\31\"\\]", function (c) return JSON_SHORT [c] or string.format ("\\u%04x", string.byte (c)) end)
	return "\"" .. escaped .. "\""
end

local function emit (t)
	local parts = {}
	for k, v in pairs (t) do
		local value
		if type (v) == "string" then
			value = json_string (v)
		elseif type (v) == "boolean" or type (v) == "number" then
			value = tostring (v)
		else
			error ("emit: field " .. tostring (k) .. " has a value of type " .. type (v), 0)
		end
		parts [#parts + 1] = json_string (tostring (k)) .. ":" .. value
	end
	table.sort (parts)
	out:write ("{" .. table.concat (parts, ",") .. "}\n")
	out:flush ()
end

-- THE REPORT SCHEMA. A report is a result only if every row the fixture writes is present exactly once (or, for the keyed rows, once per key) with exactly
-- its keys, every value has its exact type, and the counts agree with what was asked and with each other. Anything else is refused by name; a missing or
-- malformed measurement is never read as zero, as absent-equals-absent, or as a pass.
local DESCRIBE = {
	count = "a decimal count", signed = "a decimal integer", flag = "0 or 1", word = "a word", token = "a token of letters, digits and underscores",
	hex16 = "16 lowercase hex digits", hex8 = "8 lowercase hex digits", hex8x3 = "three 8-digit lowercase hex words joined by commas",
}

local function typed_value (kind, text)
	if kind == "count" then
		if string.match (text, "^%d+$") and #text <= 10 then return tonumber (text) end
	elseif kind == "signed" then
		if string.match (text, "^-?%d+$") and #text <= 11 then return tonumber (text) end
	elseif kind == "flag" then
		if text == "0" or text == "1" then return tonumber (text) end
	elseif kind == "word" then
		if string.match (text, "^%a+$") then return text end
	elseif kind == "token" then
		if string.match (text, "^[%w_]+$") then return text end
	elseif kind == "hex16" then
		if string.match (text, "^[0-9a-f]+$") and #text == 16 then return text end
	elseif kind == "hex8" then
		if string.match (text, "^[0-9a-f]+$") and #text == 8 then return text end
	elseif kind == "hex8x3" then
		if string.match (text, "^%x%x%x%x%x%x%x%x,%x%x%x%x%x%x%x%x,%x%x%x%x%x%x%x%x$") and not string.match (text, "%u") then return text end
	end
	return nil
end

local function counts (names)
	local spec = {}
	for _, name in ipairs (names) do spec [#spec + 1] = { name, "count" } end
	return spec
end

local COMMON_FIELDS = { { "group", "word" }, { "cases", "count" }, { "hit", "count" }, { "miss", "count" }, { "record_touched", "count" }, { "warm_same", "count" },
	{ "warm_different", "count" }, { "miss_point_written", "count" }, { "miss_normal_written", "count" }, { "input_mutated", "count" }, { "nearer_checked", "count" },
	{ "nearer_exists", "count" }, { "skipped_axis", "count" } }

local SCHEMA = {
	["col-probe"] = counts { "seed", "entity_random", "tree_random", "per_class", "samples_hit", "samples_miss" },
	-- the campaign's grid (it fixes the map extents) and the physical terrain's grid: two grids, reported apart
	["col-map"] = { { "world_sectors_x", "count" }, { "world_sectors_z", "count" }, { "world_sector_side", "count" }, { "terrain_sectors_x", "count" },
		{ "terrain_sectors_z", "count" }, { "terrain_sector_side", "count" }, { "delta_time", "hex8" } },
	["col-guard"] = { { "record_restored", "flag" }, { "record_words_before", "hex16" }, { "record_words_after", "hex16" }, { "rng_unchanged", "flag" },
		{ "world_unchanged", "flag" }, { "world_before", "hex16" }, { "world_after", "hex16" } },
	["col-census-entity"] = counts { "list_entities", "mobile_candidates", "mobile_moving", "mobile_stationary", "fixed_plain", "cities", "cities_with_buildings",
		"buildings", "listed_buildings", "geometry_path", "box_path", "align_terrain", "pool" },
	["col-census-tree"] = { { "tree_data", "flag" }, { "trees_total", "count" }, { "sectors_with_trees", "count" }, { "trees_type0", "count" },
		{ "trees_type1", "count" }, { "forest_geometry", "flag" }, { "pool", "count" } },
	["col-targets"] = counts { "moving_geometry", "moving_box", "stationary_geometry", "stationary_box", "fixed_geometry", "fixed_box", "building_geometry",
		"building_box", "tree_quad", "tree_geometry" },
	["col-entity-exclusion"] = counts { "through_cases", "through_hit_target", "through_hit_other", "base_cases", "src_removed", "tgt_removed", "both_removed",
		"decoy_cases", "decoy_unchanged", "building_base", "building_src_removed", "building_city_src_removed", "building_city_tgt_removed", "chain_cases",
		"chain_depth1", "chain_depth2", "chain_depth3", "chain_stalled", "chain_order_violations" },
	-- the PHYSICAL fold (inputs, roles, answers, hit identity, written vectors) and the LEGACY fold (heap slot, terrain record bookkeeping)
	["col-physical"] = { { "entity_struct", "hex16" }, { "entity_random", "hex16" }, { "tree_struct", "hex16" }, { "tree_random", "hex16" } },
	["col-legacy"] = { { "entity_struct", "hex16" }, { "entity_random", "hex16" }, { "tree_struct", "hex16" }, { "tree_random", "hex16" } },
}

-- rows that appear once per key: the key fields name the row, and the set of keys a report must hold is fixed here (except col-entity-type, whose keys are
-- the entity kinds the world happened to let a ray hit, so only the shape of each row and the uniqueness of its key are checked)
local KEYED = {
	["col-entity-kind"] = { keys = { "group", "kind" }, fields = { { "group", "word" }, { "kind", "word" }, { "cases", "count" }, { "hit", "count" } },
		required = { "struct/through", "struct/reverse", "struct/lateral", "struct/above", "struct/exclusion", "struct/edge", "random/random" } },
	["col-entity-common"] = { keys = { "group" }, fields = COMMON_FIELDS, required = { "struct", "random" } },
	["col-entity-type"] = { keys = { "group", "type" }, fields = { { "group", "word" }, { "type", "count" }, { "name", "token" }, { "hits", "count" } } },
	["col-tree-kind"] = { keys = { "group", "kind" }, fields = { { "group", "word" }, { "kind", "word" }, { "cases", "count" }, { "hit", "count" } },
		required = { "struct/through", "struct/reverse", "struct/over", "struct/under", "struct/lateral", "struct/slant", "struct/high", "struct/vertical",
			"struct/short", "struct/edge", "random/random" } },
	["col-tree-flags"] = { keys = { "class", "ignore_edge", "skip_top" }, fields = { { "class", "word" }, { "ignore_edge", "flag" }, { "skip_top", "flag" },
		{ "cases", "count" }, { "hit", "count" } }, required = { "quad/0/0", "quad/0/1", "quad/1/0", "quad/1/1", "geometry/0/0", "geometry/0/1", "geometry/1/0",
			"geometry/1/1" } },
	["col-tree-common"] = { keys = { "group" }, fields = COMMON_FIELDS, required = { "struct", "random" } },
}

local SAMPLE_FIELDS = { { "kind", "word" }, { "role", "signed" }, { "hit", "flag" }, { "type", "signed" }, { "hosted", "flag" }, { "a", "hex8x3" }, { "b", "hex8x3" },
	{ "point", "hex8x3" }, { "normal", "hex8x3" } }

-- The sample quotas are compile-time constants of the fixture (PER_CLASS, SAMPLES_HIT and SAMPLES_MISS in eech_collision_fixture.c), written into the header
-- of every report. The completeness checks below are only as strong as the quota they assume, so a header that names any other quota is refused rather than
-- believed: a changed quota would redefine which structured targets and sample rows are required or permitted.
local EMITTER_PER_CLASS, EMITTER_SAMPLES_HIT, EMITTER_SAMPLES_MISS = 3, 16, 8

local function row_fields (event, words)
	local fields = {}
	for i = 2, #words do
		local key, text = string.match (words [i], "^([%w_]+)=(.+)$")
		if not key then refuse (event .. ": '" .. words [i] .. "' is not key=value") end
		if fields [key] ~= nil then refuse (event .. ": " .. key .. " appears twice") end
		fields [key] = text
	end
	return fields
end

-- the fields of a row against its field list: every key present, of its exact type, and no other key. Returns the typed values (hex stays text).
local function typed_row (event, spec_list, fields)
	local typed = {}
	for _, spec in ipairs (spec_list) do
		local key, kind = spec [1], spec [2]
		local text = fields [key]
		if text == nil then refuse (event .. ": " .. key .. " is missing") end
		local value = typed_value (kind, text)
		if value == nil then refuse (event .. ": " .. key .. " must be " .. DESCRIBE [kind] .. ", got '" .. text .. "'") end
		typed [key] = value
		fields [key] = nil
	end
	local extra = {}
	for key in pairs (fields) do extra [#extra + 1] = key end
	table.sort (extra)
	if extra [1] then refuse (event .. ": unexpected key " .. extra [1]) end
	return typed
end

local function need_equal (event, key, actual, expected)
	if actual ~= expected then refuse (event .. ": " .. key .. " is " .. tostring (actual) .. ", expected " .. tostring (expected)) end
end

local function need_at_most (event, key, actual, limit)
	if actual > limit then refuse (event .. ": " .. key .. " is " .. tostring (actual) .. ", more than " .. tostring (limit)) end
end

local function sum (t, names)
	local total = 0
	for _, name in ipairs (names) do total = total + t [name] end
	return total
end

-- A report as the fixture writes it, checked in full; returns the events to record (in report order) and the typed rows. Refuses, by name, anything that is
-- not a complete, well-typed result for the seed and counts asked.
local function read_report (report, repeat_index)
	local events, rows, keyed, samples = {}, {}, {}, { entity_hit = 0, entity_miss = 0, tree_hit = 0, tree_miss = 0 }
	for line in string.gmatch (report, "[^\r\n]+") do
		local words = {}
		for word in string.gmatch (line, "%S+") do words [#words + 1] = word end
		local event = words [1]
		if event == nil then refuse ("the report has a blank line") end
		local fields = row_fields (event, words)
		local t = { event = event, repeat_index = repeat_index }
		local typed
		if event == "col-sample" then
			typed = typed_row (event, SAMPLE_FIELDS, fields)
			if typed.kind ~= "entity" and typed.kind ~= "tree" then refuse ("col-sample: kind must be entity or tree, got '" .. typed.kind .. "'") end
			local slot = typed.kind .. (typed.hit == 1 and "_hit" or "_miss")
			samples [slot] = samples [slot] + 1
		elseif KEYED [event] then
			local spec = KEYED [event]
			typed = typed_row (event, spec.fields, fields)
			local parts = {}
			for _, key in ipairs (spec.keys) do parts [#parts + 1] = tostring (typed [key]) end
			local id = table.concat (parts, "/")
			keyed [event] = keyed [event] or {}
			if keyed [event] [id] ~= nil then refuse ("the report has more than one " .. event .. " row for " .. id) end
			keyed [event] [id] = typed
		else
			local schema = SCHEMA [event]
			if event == "col-refused" then refuse ("the probe was refused: " .. line) end
			if schema == nil then refuse ("the report has an unknown row '" .. event .. "'") end
			if rows [event] ~= nil then refuse ("the report has more than one " .. event .. " row") end
			typed = typed_row (event, schema, fields)
			rows [event] = typed
		end
		-- counts, flags and signed integers are recorded as numbers; hex words (bit patterns, folds) and names stay text, never numbers
		for k, v in pairs (typed) do t [k] = v end
		events [#events + 1] = t
	end
	for event in pairs (SCHEMA) do
		if rows [event] == nil then refuse ("the report has no " .. event .. " row") end
	end
	for event, spec in pairs (KEYED) do
		if spec.required then
			local have = keyed [event] or {}
			for _, id in ipairs (spec.required) do
				if have [id] == nil then refuse ("the report has no " .. event .. " row for " .. id) end
			end
			local expected = {}
			for _, id in ipairs (spec.required) do expected [id] = true end
			for id in pairs (have) do
				if not expected [id] then refuse ("the report has an unexpected " .. event .. " row for " .. id) end
			end
		end
	end
	keyed ["col-entity-type"] = keyed ["col-entity-type"] or {}
	return events, rows, keyed, samples
end

local function cross_check (rows, keyed, samples)
	local p, c, ce, ct, tg, ex = rows ["col-probe"], rows ["col-guard"], rows ["col-census-entity"], rows ["col-census-tree"], rows ["col-targets"],
		rows ["col-entity-exclusion"]
	-- the integrity guards: restoration is asserted, not assumed
	need_equal ("col-guard", "record_restored", c.record_restored, 1)
	if c.record_words_before ~= c.record_words_after then
		refuse ("the probe did not restore the shared terrain record: words " .. c.record_words_before .. " -> " .. c.record_words_after)
	end
	need_equal ("col-guard", "rng_unchanged", c.rng_unchanged, 1)
	need_equal ("col-guard", "world_unchanged", c.world_unchanged, 1)
	if c.world_before ~= c.world_after then
		refuse ("the probe changed the collision candidates' state: fold " .. c.world_before .. " -> " .. c.world_after)
	end
	-- the quotas are the ones this driver's completeness checks assume
	need_equal ("col-probe", "per_class", p.per_class, EMITTER_PER_CLASS)
	need_equal ("col-probe", "samples_hit", p.samples_hit, EMITTER_SAMPLES_HIT)
	need_equal ("col-probe", "samples_miss", p.samples_miss, EMITTER_SAMPLES_MISS)

	-- the census
	need_equal ("col-census-entity", "mobile_moving + mobile_stationary", ce.mobile_moving + ce.mobile_stationary, ce.mobile_candidates)
	local candidates = ce.mobile_candidates + ce.fixed_plain + ce.buildings
	need_equal ("col-census-entity", "geometry_path + box_path", ce.geometry_path + ce.box_path, candidates)
	need_equal ("col-census-entity", "pool", ce.pool, math.min (candidates, 1024))
	need_at_most ("col-census-entity", "cities_with_buildings", ce.cities_with_buildings, ce.cities)
	need_at_most ("col-census-entity", "align_terrain", ce.align_terrain, ce.fixed_plain)
	need_equal ("col-census-tree", "trees_type0 + trees_type1", ct.trees_type0 + ct.trees_type1, ct.trees_total)
	need_equal ("col-census-tree", "pool", ct.pool, math.min (ct.trees_total, 1024))
	need_equal ("col-census-tree", "tree_data", ct.tree_data, ct.trees_total > 0 and 1 or 0)
	need_at_most ("col-census-tree", "sectors_with_trees", ct.sectors_with_trees, ct.trees_total)

	-- the structured targets: at most the quota of each class, and none of a class the census did not find
	local classes = { "moving_geometry", "moving_box", "stationary_geometry", "stationary_box", "fixed_geometry", "fixed_box", "building_geometry", "building_box",
		"tree_quad", "tree_geometry" }
	for _, name in ipairs (classes) do need_at_most ("col-targets", name, tg [name], EMITTER_PER_CLASS) end
	need_at_most ("col-targets", "moving_geometry + moving_box", tg.moving_geometry + tg.moving_box, ce.mobile_moving)
	need_at_most ("col-targets", "stationary_geometry + stationary_box", tg.stationary_geometry + tg.stationary_box, ce.mobile_stationary)
	need_at_most ("col-targets", "fixed_geometry + fixed_box", tg.fixed_geometry + tg.fixed_box, ce.fixed_plain)
	need_at_most ("col-targets", "building_geometry + building_box", tg.building_geometry + tg.building_box, ce.buildings)
	local entity_targets = sum (tg, { "moving_geometry", "moving_box", "stationary_geometry", "stationary_box", "fixed_geometry", "fixed_box", "building_geometry",
		"building_box" })
	local tree_targets = tg.tree_quad + tg.tree_geometry
	need_at_most ("col-targets", "tree_quad + tree_geometry", tree_targets, ct.trees_total)
	if ct.tree_data == 0 then need_equal ("col-targets", "tree targets without tree data", tree_targets, 0) end

	-- entity kinds: each structured target has exactly one ray of each kind; the two edge rays are fixed; the random cases are the ones asked (none if there is
	-- no entity to draw from)
	local ek = keyed ["col-entity-kind"]
	for _, kind in ipairs { "through", "reverse", "lateral", "above" } do
		need_equal ("col-entity-kind", "struct/" .. kind .. " cases", ek ["struct/" .. kind].cases, entity_targets)
	end
	need_equal ("col-entity-kind", "struct/edge cases", ek ["struct/edge"].cases, 2)
	need_equal ("col-entity-kind", "random/random cases", ek ["random/random"].cases, ce.pool > 0 and p.entity_random or 0)
	local struct_cases, struct_hits = 0, 0
	for id, row in pairs (ek) do
		need_at_most ("col-entity-kind " .. id, "hit", row.hit, row.cases)
		if string.sub (id, 1, 7) == "struct/" then struct_cases = struct_cases + row.cases; struct_hits = struct_hits + row.hit end
	end
	local ec = keyed ["col-entity-common"]
	for _, group in ipairs { "struct", "random" } do
		local row = ec [group]
		need_equal ("col-entity-common " .. group, "hit + miss", row.hit + row.miss, row.cases)
		need_equal ("col-entity-common " .. group, "warm_same + warm_different", row.warm_same + row.warm_different, row.cases)
		need_at_most ("col-entity-common " .. group, "record_touched", row.record_touched, row.cases)
		need_at_most ("col-entity-common " .. group, "miss_point_written", row.miss_point_written, row.miss)
		need_at_most ("col-entity-common " .. group, "miss_normal_written", row.miss_normal_written, row.miss)
		need_at_most ("col-entity-common " .. group, "nearer_exists", row.nearer_exists, row.nearer_checked)
		need_at_most ("col-entity-common " .. group, "nearer_checked", row.nearer_checked, row.hit)
		need_equal ("col-entity-common " .. group, "skipped_axis", row.skipped_axis, 0)
	end
	need_equal ("col-entity-common struct", "cases", ec.struct.cases, struct_cases)
	need_equal ("col-entity-common struct", "hit", ec.struct.hit, struct_hits)
	need_equal ("col-entity-common random", "cases", ec.random.cases, ek ["random/random"].cases)
	need_equal ("col-entity-common random", "hit", ec.random.hit, ek ["random/random"].hit)

	-- exclusion and ordering counters
	need_equal ("col-entity-exclusion", "through_cases", ex.through_cases, entity_targets)
	need_equal ("col-entity-exclusion", "through_hit_target + through_hit_other", ex.through_hit_target + ex.through_hit_other, ek ["struct/through"].hit)
	need_equal ("col-entity-exclusion", "base_cases", ex.base_cases, ex.through_hit_target)
	for _, name in ipairs { "src_removed", "tgt_removed", "both_removed", "decoy_cases", "building_base" } do
		need_at_most ("col-entity-exclusion", name, ex [name], ex.base_cases)
	end
	need_at_most ("col-entity-exclusion", "decoy_unchanged", ex.decoy_unchanged, ex.decoy_cases)
	for _, name in ipairs { "building_src_removed", "building_city_src_removed", "building_city_tgt_removed" } do
		need_at_most ("col-entity-exclusion", name, ex [name], ex.building_base)
	end
	need_equal ("col-entity-exclusion", "chain_cases", ex.chain_cases, ek ["struct/through"].hit)
	need_equal ("col-entity-exclusion", "chain_depth1", ex.chain_depth1, ex.chain_cases)
	need_at_most ("col-entity-exclusion", "chain_depth2", ex.chain_depth2, ex.chain_depth1)
	need_at_most ("col-entity-exclusion", "chain_depth3", ex.chain_depth3, ex.chain_depth2)
	need_at_most ("col-entity-exclusion", "chain_stalled", ex.chain_stalled, ex.chain_cases)
	need_at_most ("col-entity-exclusion", "chain_order_violations", ex.chain_order_violations, ex.chain_depth2 + ex.chain_depth3)

	-- hit entity kinds: each row's hits fit in its group's hit count, and the groups' rows together do not exceed it
	local type_hits = { struct = 0, random = 0 }
	for id, row in pairs (keyed ["col-entity-type"]) do
		if row.group ~= "struct" and row.group ~= "random" then refuse ("col-entity-type " .. id .. ": group must be struct or random") end
		type_hits [row.group] = type_hits [row.group] + row.hits
	end
	need_at_most ("col-entity-type", "struct hits", type_hits.struct, ec.struct.hit)
	need_at_most ("col-entity-type", "random hits", type_hits.random, ec.random.hit)

	-- tree kinds
	local tk = keyed ["col-tree-kind"]
	need_equal ("col-tree-kind", "struct/through cases", tk ["struct/through"].cases, 4 * tree_targets)
	for _, kind in ipairs { "reverse", "over", "under", "lateral", "slant", "high", "vertical", "short" } do
		need_equal ("col-tree-kind", "struct/" .. kind .. " cases", tk ["struct/" .. kind].cases, tree_targets)
	end
	need_equal ("col-tree-kind", "struct/edge cases", tk ["struct/edge"].cases, 2)
	local tstruct_cases, tstruct_hits = 0, 0
	for id, row in pairs (tk) do
		need_at_most ("col-tree-kind " .. id, "hit", row.hit, row.cases)
		if string.sub (id, 1, 7) == "struct/" then tstruct_cases = tstruct_cases + row.cases; tstruct_hits = tstruct_hits + row.hit end
	end
	local tf = keyed ["col-tree-flags"]
	local flag_hits = 0
	for id, row in pairs (tf) do
		local per_class_targets = (row.class == "quad") and tg.tree_quad or tg.tree_geometry
		need_equal ("col-tree-flags " .. id, "cases", row.cases, per_class_targets)
		need_at_most ("col-tree-flags " .. id, "hit", row.hit, row.cases)
		-- the flags that the compiled consumers pass (ignore_edge 0, skip_top 1) are one of the four; the through kind holds all four
		flag_hits = flag_hits + row.hit
	end
	need_equal ("col-tree-flags", "hits against struct/through hits", flag_hits, tk ["struct/through"].hit)
	local tc = keyed ["col-tree-common"]
	for _, group in ipairs { "struct", "random" } do
		local row = tc [group]
		need_equal ("col-tree-common " .. group, "hit + miss", row.hit + row.miss, row.cases)
		need_equal ("col-tree-common " .. group, "warm_same + warm_different", row.warm_same + row.warm_different, row.cases)
		need_at_most ("col-tree-common " .. group, "record_touched", row.record_touched, row.cases)
		need_at_most ("col-tree-common " .. group, "miss_point_written", row.miss_point_written, row.miss)
		need_at_most ("col-tree-common " .. group, "miss_normal_written", row.miss_normal_written, row.miss)
		need_at_most ("col-tree-common " .. group, "nearer_exists", row.nearer_exists, row.nearer_checked)
		need_at_most ("col-tree-common " .. group, "nearer_checked", row.nearer_checked, row.hit)
	end
	need_equal ("col-tree-common struct", "cases", tc.struct.cases, tstruct_cases)
	need_equal ("col-tree-common struct", "hit", tc.struct.hit, tstruct_hits)
	need_equal ("col-tree-common struct", "skipped_axis", tc.struct.skipped_axis, 0)
	need_equal ("col-tree-common random", "cases", tc.random.cases, tk ["random/random"].cases)
	need_equal ("col-tree-common random", "hit", tc.random.hit, tk ["random/random"].hit)
	need_equal ("col-tree-common random", "cases + skipped_axis", tc.random.cases + tc.random.skipped_axis, ct.pool > 0 and p.tree_random or 0)

	-- samples: exactly the quota, or every case if there were fewer
	need_equal ("col-sample", "entity hit samples", samples.entity_hit, math.min (EMITTER_SAMPLES_HIT, ec.struct.hit + ec.random.hit))
	need_equal ("col-sample", "entity miss samples", samples.entity_miss, math.min (EMITTER_SAMPLES_MISS, ec.struct.miss + ec.random.miss))
	need_equal ("col-sample", "tree hit samples", samples.tree_hit, math.min (EMITTER_SAMPLES_HIT, tc.struct.hit + tc.random.hit))
	need_equal ("col-sample", "tree miss samples", samples.tree_miss, math.min (EMITTER_SAMPLES_MISS, tc.struct.miss + tc.random.miss))
end

-- The families the world gave no case or no hit for: said in the record, never passed over. A gap is a fact about the loaded world and the chosen time,
-- not a failure of the original.
local function coverage_gaps (rows, keyed)
	local ce, ct, tg, ex = rows ["col-census-entity"], rows ["col-census-tree"], rows ["col-targets"], rows ["col-entity-exclusion"]
	local ek, tk, tf = keyed ["col-entity-kind"], keyed ["col-tree-kind"], keyed ["col-tree-flags"]
	local gaps = {}
	local function need (ok, name) if not ok then gaps [#gaps + 1] = name end end
	need (tg.moving_geometry + tg.moving_box > 0, "no_moving_mobile_target")
	need (tg.stationary_geometry + tg.stationary_box > 0, "no_stationary_mobile_target")
	need (tg.fixed_geometry + tg.fixed_box > 0, "no_fixed_object_target")
	need (tg.building_geometry + tg.building_box > 0, "no_city_building_target")
	need (ce.geometry_path > 0, "no_candidate_with_collision_geometry")
	need (ce.box_path > 0, "no_candidate_on_the_bounding_box_path")
	need (ce.align_terrain > 0, "no_terrain_aligned_fixed_object")
	need (ek ["struct/through"].hit > 0, "no_structured_entity_hit")
	need (ex.chain_depth2 > 0, "no_two_object_ray_for_scan_order")
	need (ct.tree_data == 1, "no_tree_data_loaded")
	need (ct.forest_geometry == 1, "forest_tree_object_has_no_collision_geometry")
	need (tg.tree_quad > 0, "no_quad_plane_tree_target")
	need (tg.tree_geometry > 0, "no_geometry_tree_target")
	need (tk ["struct/through"].hit > 0, "no_structured_tree_hit")
	need (tf ["quad/0/1"].hit > 0 or tf ["geometry/0/1"].hit > 0, "no_tree_hit_with_the_consumer_flags")
	return gaps
end

local dc = require ("eech_dc")
dc.prepare_installation (root)
local engine = dc.boot {
	install_root = root, map = scenario.map, campaign_directory = "camp01", campaign = scenario.campaign, gunship = "apache",
	seed = 1, bind_mobile = host.bind_mobile, move_vehicle = host.move_vehicle, observe_loss = host.observe_loss,
	supply_reached = host.supply_reached,
	line_of_sight = host.line_of_sight,
	sense = host.sense, aim = host.aim,
}
emit { event = "boot", scenario = scenario_name, seed = seed, entity_cases = entity_cases, tree_cases = tree_cases, repeats = repeats, seconds = seconds,
	after = after, frame_ms = frame_ms, probe = probe, require_coverage = require_coverage }

local function run_frames (count)
	for _ = 1, count do engine:frame (frame_ms) end
end

-- the coarse witness: the object list's identity, kind, side and alive flag, in id order (a modular polynomial hash, so it needs no bit operations)
local function state_digest ()
	local objects = engine:objects ()
	table.sort (objects, function (a, b) return a.id < b.id end)
	local digest, alive = 5381, 0
	for _, o in ipairs (objects) do
		local text = tostring (o.id) .. ":" .. tostring (o.kind) .. ":" .. tostring (o.side) .. ":" .. tostring (o.alive) .. ";"
		for i = 1, #text do digest = (digest * 31 + string.byte (text, i)) % 2147483647 end
		if o.alive then alive = alive + 1 end
	end
	return string.format ("%08x", digest), #objects, alive
end

run_frames (frames_before)
emit { event = "frames_before", frames = frames_before }

local probed = false
local first_report, first_rows, first_keyed
local total_gaps = {}
if probe == "yes" then
	for repeat_index = 1, repeats do
		local ok, report = pcall (function () return engine:collision_probe (seed, entity_cases, tree_cases) end)
		if not ok then
			emit { event = "probe_refused", repeat_index = repeat_index, error = tostring (report) }
			refuse ("the probe was refused: " .. tostring (report))
		end
		local parsed_ok, events, rows, keyed, samples = pcall (read_report, report, repeat_index)
		if parsed_ok then
			-- the header must be for what was asked, before any count is checked against it
			local p = rows ["col-probe"]
			if p.seed ~= seed or p.entity_random ~= entity_cases or p.tree_random ~= tree_cases then
				parsed_ok = false
				events = "collision-characterization: the report is for seed " .. tostring (p.seed) .. " entity_cases " .. tostring (p.entity_random) .. " tree_cases "
					.. tostring (p.tree_random) .. ", not the " .. tostring (seed) .. " " .. tostring (entity_cases) .. " " .. tostring (tree_cases) .. " asked"
			end
		end
		if parsed_ok then
			local checked_ok, checked = pcall (cross_check, rows, keyed, samples)
			if not checked_ok then parsed_ok, events = false, checked end
		end
		if not parsed_ok then
			emit { event = "report_invalid", repeat_index = repeat_index, error = tostring (events) }
			error (tostring (events), 0)
		end
		for _, e in ipairs (events) do emit (e) end
		probed = true
		if repeat_index == 1 then
			first_report, first_rows, first_keyed = report, rows, keyed
		else
			local same = report == first_report
			emit { event = "repeat_identical", repeat_index = repeat_index, ok = same }
			if not same then
				refuse ("repeat " .. repeat_index .. " differs from repeat 1: the probe is not a pure function of the seed and the loaded world")
			end
		end
	end
	total_gaps = coverage_gaps (first_rows, first_keyed)
	emit { event = "coverage", gaps = (#total_gaps == 0) and "none" or table.concat (total_gaps, ","), gap_count = #total_gaps,
		moving_targets = first_rows ["col-targets"].moving_geometry + first_rows ["col-targets"].moving_box,
		structured_entity_hits = first_keyed ["col-entity-kind"] ["struct/through"].hit,
		structured_tree_hits = first_keyed ["col-tree-kind"] ["struct/through"].hit,
		random_entity_hits = first_keyed ["col-entity-kind"] ["random/random"].hit,
		random_tree_hits = first_keyed ["col-tree-kind"] ["random/random"].hit }
else
	emit { event = "probe_skipped", reason = "probe=no (the control)" }
end

run_frames (frames_after)
emit { event = "frames_after", frames = frames_after }

local digest, objects, alive = state_digest ()
emit { event = "state_digest", digest = digest, objects = objects, alive = alive, probed = probed }

local gap_text = (not probed) and "not_probed" or ((#total_gaps == 0) and "none" or table.concat (total_gaps, ","))
emit { event = "done", probed = probed, coverage_gaps = gap_text }
out:close ()
if probed and #total_gaps > 0 and require_coverage == "yes" then
	refuse ("require_coverage=yes and the world offered no case for: " .. gap_text)
end
print (string.format ("COLLISION_CHARACTERIZATION_DONE probed=%s repeats=%d coverage_gaps=%s", tostring (probed), probed and repeats or 0, gap_text))
