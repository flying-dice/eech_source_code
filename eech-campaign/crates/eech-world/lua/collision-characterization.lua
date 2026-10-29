-- TEST-ONLY object and tree collision characterisation (EEDCS-49, D3 G3a). Needs a DLL built with the `collision-fixture` feature; the shipping
-- module has no `collision_probe` method.
--
-- It boots the campaign through the public path (require ("eech_dc")) with the reference host's callbacks, optionally runs frames, and then calls
-- the one fixture entry, engine:collision_probe (seed, entity_cases, tree_cases), which runs the ORIGINAL get_line_of_sight_collision_entity and
-- get_line_of_sight_collision_tree against the world the campaign's own boot loaded (see csrc/eech_collision_fixture.c). The report is `key=value`
-- text lines; each becomes one JSON Lines event. Raw coordinates appear only in that runtime output (float bit patterns in hex), kept outside Git.
--
-- The probe is repeated (`repeats`, default 2) in the same process with no frames between: the reports must be byte identical. That shows the report is
-- repeatable for the seed and the loaded world; it is not a proof that the probe changes nothing. What is checked about state is bounded: the shared terrain
-- record is restored byte for byte, the game's random stream word is unchanged, and a sector-list fingerprint (entity type, collision flags, position and
-- mobile motion; not city buildings or identities) is equal before and after (the report states all three; each must hold). A difference fails the run;
-- nothing is tuned to pass. The probe also REQUIRES all six floating-point exceptions masked in the x87 control word and the SSE MXCSR (`col-fp`).
--
-- The ORIGINAL getter for the updated, moved and rotated flags is not a read (mb_int.c: `value = array [index] &= mask` clears the other bits of the entity's
-- 32-entity word), and the original moving-target collision calls it, so the original queries really change those bounded arrays. The fixture's own observer reads
-- the raw bit instead, copies the three arrays before anything is observed, reports what the ORIGINAL queries did to them (`col-flags`, LEGACY diagnostics: hashes
-- and counts only) and writes the copy back before the final fingerprint; the driver requires that restoration and that the restored arrays equal the copy. The copy
-- exists for the extra observer only. Nothing else is restored to suppress a fingerprint difference.
--
-- A report that fails validation is kept: `report_invalid` carries the exact text the fixture returned as `raw_report` (JSON-escaped) in the private output, so a rejected
-- measurement is never reduced to its error text. That text is a diagnostic, not a result; a malformed report is never accepted and no physical rule is changed.
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
--   trace_out=<path>  optional: a PRIVATE JSON Lines transcript of every tree query the probe makes (see below). The path must not exist, must differ from out, and the run must
--             probe (probe=yes) with tree_cases at most 8000 (the fixture holds at most 32768 records, at most 4 per random tree case plus about 160 structured). Off by default,
--             and the probe's report is identical either way.
--
-- TREE-QUERY TRANSCRIPT (trace_out): the fixture records, in execution order, each tree query it makes (primary cold and warm, shortened nearer-check cold and warm) with the f32
-- endpoint bits, the caller's initial intercept (zero) and normal (sentinel) bits, the terrain lookups the ORIGINAL made (x, z and the elevation it returned, captured by a passive
-- wrapper at the original's own two lookups; no extra terrain query), and the final hit, intercept and normal bits including a miss's partial writes, plus a facts block (tree record
-- size, offsets and byte dumps that fix the packed height/type bit order, map and tree scales, the forest tree object's collision triangles and the first live records as raw bytes). The
-- driver parses it strictly, checks it against the report it accompanies (record counts per group and stage, hits, warm agreement, miss-time writes, derived-axis rays, tree census), requires
-- no overflow and no unrecorded lookup, requires repeated transcripts to be identical, and writes repeat 1 to trace_out only after every repeat has passed (a trace file without its
-- trace-end line is incomplete). The raw transcript is private runtime output and stays out of Git. It records the original's answers only.

io.stdout:setvbuf ("no")

local args = host.args

local function refuse (message)
	error ("collision-characterization: " .. message, 0)
end

local known = { root = true, out = true, scenario = true, seed = true, entity_cases = true, tree_cases = true, repeats = true, seconds = true,
	after = true, frame_ms = true, probe = true, require_coverage = true, trace_out = true }
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

-- the optional tree-query transcript: every argument checked before anything is opened, required or booted
local TRACE_TREE_CASES_MAX = 8000
local trace_path = args.trace_out
if trace_path ~= nil then
	if type (trace_path) ~= "string" or trace_path == "" then refuse ("trace_out must be a non-empty path") end
	if trace_path == out_path then refuse ("trace_out must differ from out") end
	if probe ~= "yes" then refuse ("trace_out needs probe=yes: the transcript is of the probe's tree queries") end
	if tree_cases > TRACE_TREE_CASES_MAX then
		refuse ("trace_out needs tree_cases at most " .. TRACE_TREE_CASES_MAX .. " (the fixture holds 32768 records, up to 4 per random tree case), got " .. tree_cases)
	end
	local existing = io.open (trace_path, "rb")
	if existing then
		existing:close ()
		refuse ("trace_out '" .. trace_path .. "' already exists: an existing file is never overwritten")
	end
end

local MAX_FRAMES = 72000
local frames_before = math.floor (seconds * 1000 / frame_ms)
local frames_after = math.floor (after * 1000 / frame_ms)
if frames_before + frames_after > MAX_FRAMES then
	refuse ("seconds and after need " .. (frames_before + frames_after) .. " frames of " .. frame_ms .. " ms; the most this driver runs is " .. MAX_FRAMES)
end

-- Only now, with every argument good, is anything opened, required or booted.
local out = assert (io.open (out_path, "w"))
local trace_file = nil
if trace_path ~= nil then
	-- Checked again NOW, after out exists. The string comparison above is not file identity (out=dir/report.jsonl and trace_out=dir/./report.jsonl name one file, as do case or
	-- drive-letter spellings on Windows), and the first check ran before out was created; the operating system resolves an alias here, so an alias of out is found as existing and is
	-- never truncated. This is a check followed by a create, not an exclusive create, and is not claimed race-proof: the run uses fresh unique names.
	local existing = io.open (trace_path, "rb")
	if existing then
		existing:close ()
		pcall (function () out:close () end)
		refuse ("trace_out '" .. trace_path .. "' already exists: an existing file is never overwritten")
	end
	trace_file = io.open (trace_path, "w")
	if trace_file == nil then
		pcall (function () out:close () end)
		refuse ("trace_out '" .. trace_path .. "' cannot be created")
	end
end

-- A JSON string (RFC 8259). Lua's %q writes LUA source, not JSON: it leaves a tab (and every other control byte) raw, which a strict JSON parser refuses, and
-- writes a newline as a backslash and a real newline. So: the quote and the backslash are escaped, every control byte 0x00 to 0x1f becomes its short escape
-- (\b \t \n \f \r) or \u00XX, and every other byte, UTF-8 included, is written unchanged. Nothing is dropped or flattened.
local JSON_SHORT = { ["\b"] = "\\b", ["\t"] = "\\t", ["\n"] = "\\n", ["\f"] = "\\f", ["\r"] = "\\r", ["\""] = "\\\"", ["\\"] = "\\\\" }

local function json_string (text)
	local escaped = string.gsub (text, "[%z\1-\31\"\\]", function (c) return JSON_SHORT [c] or string.format ("\\u%04x", string.byte (c)) end)
	return "\"" .. escaped .. "\""
end

local function encode_line (t)
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
	return "{" .. table.concat (parts, ",") .. "}\n"
end

local function emit_to (file, t)
	file:write (encode_line (t))
	file:flush ()
end

-- The tree-query transcript's writer: every write and flush is CHECKED. A failure is a failure of the run (before trace_written and done), never ignored.
local function trace_write (file, t)
	local ok, err = file:write (encode_line (t))
	if not ok then refuse ("the tree-query transcript could not be written: " .. tostring (err)) end
	ok, err = file:flush ()
	if not ok then refuse ("the tree-query transcript could not be flushed: " .. tostring (err)) end
end

local function emit (t)
	emit_to (out, t)
end

-- THE REPORT SCHEMA. A report is a result only if every row the fixture writes is present exactly once (or, for the keyed rows, once per key) with exactly
-- its keys, every value has its exact type, and the counts agree with what was asked and with each other. Anything else is refused by name; a missing or
-- malformed measurement is never read as zero, as absent-equals-absent, or as a pass.
local DESCRIBE = {
	count = "a decimal count", signed = "a decimal integer", flag = "0 or 1", word = "a word", token = "a token of letters, digits and underscores",
	hexs = "an even number of lowercase hex digits (at most 32)", hex16 = "16 lowercase hex digits", hex8 = "8 lowercase hex digits", hex8x3 = "three 8-digit lowercase hex words joined by commas",
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
	elseif kind == "hexs" then
		if string.match (text, "^[0-9a-f]+$") and #text % 2 == 0 and #text <= 32 then return text end
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
	{ "nearer_exists", "count" }, { "skipped_axis", "count" }, { "derived_axis", "count" } }

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
	-- every candidate of each class in the real sector lists and tree sectors, not only the quota the structured cases take
	["col-census-class"] = counts { "moving_geometry", "moving_box", "stationary_geometry", "stationary_box", "fixed_geometry", "fixed_box", "building_geometry",
		"building_box", "tree_quad", "tree_geometry" },
	-- the floating-point exception masks the probe read (a set bit is a masked exception); both must be all six
	["col-fp"] = { { "arch", "token" }, { "x87_masks", "count" }, { "sse_masks", "count" } },
	-- LEGACY diagnostics of the three bounded update-flag arrays: the full copy taken before observation, what the ORIGINAL queries did to the live arrays, and the arrays
	-- after the copy was written back. Hashes and counts only; no array, pointer or layout leaves the fixture.
	["col-flags"] = { { "restored", "flag" }, { "words", "count" }, { "before", "hex16" }, { "after_queries", "hex16" }, { "after_restore", "hex16" },
		{ "updated_changed_words", "count" }, { "updated_cleared_bits", "count" }, { "moved_changed_words", "count" }, { "moved_cleared_bits", "count" },
		{ "rotated_changed_words", "count" }, { "rotated_cleared_bits", "count" }, { "added_bits", "count" } },
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
	-- the probe's precondition, reported and re-checked here: all six exceptions masked in both the x87 control word and the SSE MXCSR
	need_equal ("col-fp", "x87_masks", rows ["col-fp"].x87_masks, 63)
	need_equal ("col-fp", "sse_masks", rows ["col-fp"].sse_masks, 63)
	-- the update-flag arrays: the copy was written back, and the restored arrays are the copy (the original queries' own effect on them is reported, not hidden)
	local fl = rows ["col-flags"]
	need_equal ("col-flags", "restored", fl.restored, 1)
	if fl.after_restore ~= fl.before then
		refuse ("the update-flag arrays were not restored to the copy taken before the probe: fold " .. fl.before .. " -> " .. fl.after_restore)
	end
	if fl.words < 1 then refuse ("col-flags: words is " .. tostring (fl.words) .. ", expected at least 1") end
	local changed = fl.updated_changed_words + fl.moved_changed_words + fl.rotated_changed_words
	local cleared = fl.updated_cleared_bits + fl.moved_cleared_bits + fl.rotated_cleared_bits
	for _, name in ipairs { "updated_changed_words", "moved_changed_words", "rotated_changed_words" } do
		need_at_most ("col-flags", name, fl [name], fl.words)
	end
	-- a changed word differs in at least one bit, cleared or added
	need_at_most ("col-flags", "changed words", changed, cleared + fl.added_bits)
	-- Feasibility of the counts for 32-bit words (a finite legacy diagnostic, not a host contract). added_bits is ONE aggregate across the three arrays, and the original only
	-- clears, so it is not forced to 0, but the counts must still be realisable:
	--   1. an array's cleared bits fit in its own changed words (at most 32 per word), so no bits at all with no changed word;
	--   2. a changed word with no cleared bit in its own array differs only by an ADDED bit: the changed words an array has beyond its cleared bits, summed over the arrays,
	--      need at least that many added bits (another array's cleared bits cannot account for them);
	--   3. all the differing bits, cleared and added, fit in all the changed words.
	local array_effects = { { "updated", fl.updated_changed_words, fl.updated_cleared_bits }, { "moved", fl.moved_changed_words, fl.moved_cleared_bits },
		{ "rotated", fl.rotated_changed_words, fl.rotated_cleared_bits } }
	local required_additions = 0
	for _, effect in ipairs (array_effects) do
		need_at_most ("col-flags", effect [1] .. "_cleared_bits", effect [3], 32 * effect [2])
		if effect [2] > effect [3] then required_additions = required_additions + (effect [2] - effect [3]) end
	end
	if required_additions > fl.added_bits then
		refuse ("col-flags: changed words without a cleared bit in their own array need at least " .. tostring (required_additions) .. " added bits, added_bits is "
			.. tostring (fl.added_bits))
	end
	need_at_most ("col-flags", "cleared bits + added_bits", cleared + fl.added_bits, 32 * changed)
	-- the hash after the queries equals the copy's exactly when no word changed
	if (changed == 0) ~= (fl.after_queries == fl.before) then
		refuse ("col-flags: " .. tostring (changed) .. " changed words but after_queries " .. fl.after_queries .. " against before " .. fl.before)
	end
	-- the bounded guards: restoration of the terrain record is asserted, not assumed
	need_equal ("col-guard", "record_restored", c.record_restored, 1)
	if c.record_words_before ~= c.record_words_after then
		refuse ("the probe did not restore the shared terrain record: words " .. c.record_words_before .. " -> " .. c.record_words_after)
	end
	need_equal ("col-guard", "rng_unchanged", c.rng_unchanged, 1)
	need_equal ("col-guard", "world_unchanged", c.world_unchanged, 1)
	if c.world_before ~= c.world_after then
		refuse ("the sector-list fingerprint changed during the probe: fold " .. c.world_before .. " -> " .. c.world_after)
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

	-- the class census must agree with the census it refines: the four entity categories, the two model paths, and the trees
	local cc = rows ["col-census-class"]
	need_equal ("col-census-class", "moving_geometry + moving_box", cc.moving_geometry + cc.moving_box, ce.mobile_moving)
	need_equal ("col-census-class", "stationary_geometry + stationary_box", cc.stationary_geometry + cc.stationary_box, ce.mobile_stationary)
	need_equal ("col-census-class", "fixed_geometry + fixed_box", cc.fixed_geometry + cc.fixed_box, ce.fixed_plain)
	need_equal ("col-census-class", "building_geometry + building_box", cc.building_geometry + cc.building_box, ce.buildings)
	need_equal ("col-census-class", "geometry classes", cc.moving_geometry + cc.stationary_geometry + cc.fixed_geometry + cc.building_geometry, ce.geometry_path)
	need_equal ("col-census-class", "box classes", cc.moving_box + cc.stationary_box + cc.fixed_box + cc.building_box, ce.box_path)
	need_equal ("col-census-class", "tree_quad + tree_geometry", cc.tree_quad + cc.tree_geometry, ct.trees_total)
	-- a tree is a geometry tree when it is type 1 and the forest tree object has collision geometry, else a quad-plane tree
	need_equal ("col-census-class", "tree_geometry", cc.tree_geometry, ct.forest_geometry == 1 and ct.trees_type1 or 0)

	-- the structured targets are exactly the first quota of each class in scan order, so exactly min(class census, quota): a class the census found is never
	-- left unselected or under-selected, and a class it did not find has none
	local classes = { "moving_geometry", "moving_box", "stationary_geometry", "stationary_box", "fixed_geometry", "fixed_box", "building_geometry", "building_box",
		"tree_quad", "tree_geometry" }
	for _, name in ipairs (classes) do
		need_at_most ("col-targets", name, tg [name], EMITTER_PER_CLASS)
		need_equal ("col-targets", name, tg [name], math.min (cc [name], EMITTER_PER_CLASS))
	end
	local entity_targets = sum (tg, { "moving_geometry", "moving_box", "stationary_geometry", "stationary_box", "fixed_geometry", "fixed_box", "building_geometry",
		"building_box" })
	local tree_targets = tg.tree_quad + tg.tree_geometry

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
		need_equal ("col-entity-common " .. group, "derived_axis", row.derived_axis, 0)
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

	-- hit entity kinds: the fixture emits EVERY positive bucket, so each group's bucket hits must add up to exactly its hit count; a deleted bucket, or all of
	-- them, is a defective report
	local type_hits = { struct = 0, random = 0 }
	for id, row in pairs (keyed ["col-entity-type"]) do
		if row.group ~= "struct" and row.group ~= "random" then refuse ("col-entity-type " .. id .. ": group must be struct or random") end
		type_hits [row.group] = type_hits [row.group] + row.hits
	end
	need_equal ("col-entity-type", "struct hits", type_hits.struct, ec.struct.hit)
	need_equal ("col-entity-type", "random hits", type_hits.random, ec.random.hit)

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
		-- a derived (shortened nearer-check) ray with a zero x or z extent is queried, never skipped; it can only come from a nearer-check
		need_at_most ("col-tree-common " .. group, "derived_axis", row.derived_axis, row.nearer_checked)
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

-- THE TRANSCRIPT SCHEMA (see above). Each row has exactly its keys, each of its exact type.
local H3 = "hex8x3"
local TRACE_ONCE = {
	["tr-state"] = { { "armed", "flag" }, { "records", "count" }, { "capacity", "count" }, { "overflow", "flag" }, { "extra_lookups", "count" }, { "facts_overflow", "flag" },
		{ "facts_bytes", "count" } },
	["tf-layout"] = { { "record_size", "count" }, { "x_offset", "count" }, { "z_offset", "count" }, { "y_offset", "count" }, { "colour_offset", "count" }, { "probe", "hexs" },
		{ "height_only", "hexs" }, { "type_only", "hexs" } },
	["tf-map"] = { { "terrain_sectors_x", "count" }, { "terrain_sectors_z", "count" }, { "terrain_sector_side", "count" }, { "xz_scale", "hex8" }, { "height_scale", "hex8" },
		{ "height_minimum", "hex8" }, { "max_tree_height", "count" }, { "map_min_x", "hex8" }, { "map_min_z", "hex8" }, { "map_max_x", "hex8" }, { "map_max_z", "hex8" },
		{ "tree_total", "count" }, { "sectors_with_trees", "count" } },
	["tf-forest"] = { { "geometry", "flag" }, { "triangles", "count" }, { "capped", "flag" }, { "fold", "hex16" } },
}
local TRACE_MANY = {
	["tf-triangle"] = { { "i", "count" }, { "p0", H3 }, { "p1", H3 }, { "p2", H3 } },
	["tf-sample"] = { { "sx", "count" }, { "sz", "count" }, { "count", "count" }, { "index", "count" }, { "bytes", "hexs" } },
	["tq"] = { { "ordinal", "count" }, { "group", "count" }, { "kind", "count" }, { "role", "count" }, { "stage", "count" }, { "ignore_edge", "flag" }, { "skip_top", "flag" },
		{ "a", H3 }, { "b", H3 }, { "init_point", H3 }, { "init_normal", H3 }, { "lookups", "count" }, { "extra_lookups", "count" }, { "l0", H3 }, { "l1", H3 }, { "hit", "flag" },
		{ "point", H3 }, { "normal", H3 } },
}
local TRACE_CHUNK = 200
local ZERO3, SENTINEL3 = "00000000,00000000,00000000", "5a5a5a5a,5a5a5a5a,5a5a5a5a"

local function parse_trace_text (text, state)
	for line in string.gmatch (text, "[^\r\n]+") do
		local words = {}
		for word in string.gmatch (line, "%S+") do words [#words + 1] = word end
		local event = words [1]
		local fields = row_fields (event, words)
		local raw = fields
		if TRACE_ONCE [event] then
			if state.once [event] ~= nil then refuse ("the transcript has more than one " .. event .. " row") end
			state.once [event] = typed_row (event, TRACE_ONCE [event], raw)
			state.order [#state.order + 1] = { event = event, typed = state.once [event] }
		elseif TRACE_MANY [event] then
			local typed = typed_row (event, TRACE_MANY [event], raw)
			state.many [event] = state.many [event] or {}
			state.many [event] [#state.many [event] + 1] = typed
			state.order [#state.order + 1] = { event = event, typed = typed }
		else
			refuse ("the transcript has an unknown row '" .. event .. "'")
		end
	end
end

-- The transcript as the fixture returned it: the state row and facts block, then every record in chunks. Returns the parsed rows and the exact text.
local function read_trace (engine)
	local state = { once = {}, many = {}, order = {} }
	local facts_text, total = engine:collision_trace_read (0, 0, 1)
	parse_trace_text (facts_text, state)
	local pieces = { facts_text }
	local offset = 0
	while offset < total do
		local want = math.min (TRACE_CHUNK, total - offset)
		local text, again = engine:collision_trace_read (1, offset, TRACE_CHUNK)
		if again ~= total then refuse ("the transcript changed while it was read: " .. tostring (total) .. " records, then " .. tostring (again)) end
		local _, lines = string.gsub (text, "\n", "")
		if lines ~= want then refuse ("the transcript chunk at " .. offset .. " held " .. lines .. " records, expected " .. want) end
		pieces [#pieces + 1] = text
		parse_trace_text (text, state)
		offset = offset + want
	end
	return state, table.concat (pieces)
end

local function bits_equal_axis (x, y)
	-- the f32 bit words are equal, or both are a zero (the original's test is a float subtraction equal to 0)
	return x == y or (tonumber (x, 16) % 2147483648 == 0 and tonumber (y, 16) % 2147483648 == 0)
end

local function first_word (triple) return string.match (triple, "^(%x+),") end
local function second_word (triple) return string.match (triple, "^%x+,(%x+),") end
local function third_word (triple) return string.match (triple, ",(%x+)$") end

-- The transcript against the report it accompanies and against its own structure. Nothing here re-derives a physical answer: it counts and compares what the fixture recorded.
local function check_trace (trace, rows, keyed)
	local st, once, many = trace.once ["tr-state"], trace.once, trace.many
	if st == nil then refuse ("the transcript has no tr-state row") end
	for _, name in ipairs { "tf-layout", "tf-map", "tf-forest" } do
		if once [name] == nil then refuse ("the transcript has no " .. name .. " row") end
	end
	need_equal ("tr-state", "armed", st.armed, 1)
	need_equal ("tr-state", "overflow", st.overflow, 0)
	need_equal ("tr-state", "extra_lookups", st.extra_lookups, 0)
	need_equal ("tr-state", "facts_overflow", st.facts_overflow, 0)
	local tc = keyed ["col-tree-common"]
	local expected = 2 * (tc.struct.cases + tc.random.cases) + 2 * (tc.struct.nearer_checked + tc.random.nearer_checked)
	local tqs = many ["tq"] or {}
	need_equal ("tr-state", "records", st.records, expected)
	need_equal ("tq", "rows", #tqs, expected)

	-- facts against the report's census
	local layout, map, forest, ct, cm = once ["tf-layout"], once ["tf-map"], once ["tf-forest"], rows ["col-census-tree"], rows ["col-map"]
	local shown = math.min (layout.record_size, 16) * 2
	for _, name in ipairs { "probe", "height_only", "type_only" } do
		need_equal ("tf-layout", name .. " length", #layout [name], shown)
	end
	need_equal ("tf-map", "terrain_sectors_x", map.terrain_sectors_x, cm.terrain_sectors_x)
	need_equal ("tf-map", "terrain_sectors_z", map.terrain_sectors_z, cm.terrain_sectors_z)
	need_equal ("tf-map", "terrain_sector_side", map.terrain_sector_side, cm.terrain_sector_side)
	need_equal ("tf-map", "tree_total", map.tree_total, ct.trees_total)
	need_equal ("tf-map", "sectors_with_trees", map.sectors_with_trees, ct.sectors_with_trees)
	need_equal ("tf-forest", "geometry", forest.geometry, ct.forest_geometry)
	need_equal ("tf-forest", "triangles", forest.triangles, #(many ["tf-triangle"] or {}))
	for i, tri in ipairs (many ["tf-triangle"] or {}) do need_equal ("tf-triangle", "i", tri.i, i - 1) end
	if forest.geometry == 0 then need_equal ("tf-forest", "triangles without geometry", forest.triangles, 0) end
	local samples = many ["tf-sample"] or {}
	if ct.trees_total > 0 then
		if #samples == 0 then refuse ("tf-sample: no sample rows although the census found " .. tostring (ct.trees_total) .. " trees") end
		need_equal ("tf-sample", "rows", #samples, math.min (8, samples [1].count))
	else
		need_equal ("tf-sample", "rows without trees", #samples, 0)
	end
	for i, sample in ipairs (samples) do
		need_equal ("tf-sample", "index", sample.index, i - 1)
		need_equal ("tf-sample", "sector", sample.sx .. "," .. sample.sz, samples [1].sx .. "," .. samples [1].sz)
		need_equal ("tf-sample", "bytes length", #sample.bytes, shown)
	end

	-- the records: contiguous, in range, stage order, and counted against the report
	local seen = { [2] = { s0 = 0, s1 = 0, s2 = 0, s3 = 0, hit = 0, warm_same = 0, miss_point = 0, miss_normal = 0, derived = 0 },
		[3] = { s0 = 0, s1 = 0, s2 = 0, s3 = 0, hit = 0, warm_same = 0, miss_point = 0, miss_normal = 0, derived = 0 } }
	local previous, cold_record, expect_stage = nil, nil, 0
	for i, q in ipairs (tqs) do
		need_equal ("tq", "ordinal", q.ordinal, i - 1)
		if q.group ~= 2 and q.group ~= 3 then refuse ("tq " .. tostring (q.ordinal) .. ": group must be 2 or 3, got " .. tostring (q.group)) end
		if q.kind > 10 then refuse ("tq " .. tostring (q.ordinal) .. ": kind must be 0 to 10, got " .. tostring (q.kind)) end
		if (q.group == 3) ~= (q.kind == 10) then refuse ("tq " .. tostring (q.ordinal) .. ": group " .. tostring (q.group) .. " does not fit kind " .. tostring (q.kind)) end
		if q.stage > 3 then refuse ("tq " .. tostring (q.ordinal) .. ": stage must be 0 to 3, got " .. tostring (q.stage)) end
		if q.lookups > 2 then refuse ("tq " .. tostring (q.ordinal) .. ": lookups must be 0 to 2, got " .. tostring (q.lookups)) end
		need_equal ("tq " .. tostring (q.ordinal), "extra_lookups", q.extra_lookups, 0)
		need_equal ("tq " .. tostring (q.ordinal), "init_point", q.init_point, ZERO3)
		need_equal ("tq " .. tostring (q.ordinal), "init_normal", q.init_normal, SENTINEL3)
		if q.lookups < 2 then need_equal ("tq " .. tostring (q.ordinal), "unused l1", q.l1, ZERO3) end
		if q.lookups < 1 then need_equal ("tq " .. tostring (q.ordinal), "unused l0", q.l0, ZERO3) end
		-- stage order: 0, 1, then for a hit optionally 2, 3; a nearer-check follows only a primary hit
		if q.stage == 0 then
			if expect_stage ~= 0 and expect_stage ~= 2 then refuse ("tq " .. tostring (q.ordinal) .. ": stage 0 where stage " .. tostring (expect_stage) .. " was expected") end
			cold_record = q
			expect_stage = 1
		elseif q.stage == 1 then
			if expect_stage ~= 1 then refuse ("tq " .. tostring (q.ordinal) .. ": stage 1 where stage " .. tostring (expect_stage) .. " was expected") end
			expect_stage = (cold_record.hit == 1) and 2 or 0
		elseif q.stage == 2 then
			if expect_stage ~= 2 then refuse ("tq " .. tostring (q.ordinal) .. ": stage 2 without a primary hit before it") end
			expect_stage = 3
		else
			if expect_stage ~= 3 then refuse ("tq " .. tostring (q.ordinal) .. ": stage 3 where stage " .. tostring (expect_stage) .. " was expected") end
			expect_stage = 0
		end
		local g = seen [q.group]
		g ["s" .. q.stage] = g ["s" .. q.stage] + 1
		if q.stage == 0 then
			if q.hit == 1 then
				g.hit = g.hit + 1
			else
				if q.point ~= ZERO3 then g.miss_point = g.miss_point + 1 end
				if q.normal ~= SENTINEL3 then g.miss_normal = g.miss_normal + 1 end
			end
		elseif q.stage == 1 then
			if q.hit == cold_record.hit and q.point == cold_record.point and q.normal == cold_record.normal then g.warm_same = g.warm_same + 1 end
		elseif q.stage == 2 then
			-- the derived ray: its f32 x or z extent is exactly zero
			if bits_equal_axis (first_word (q.a), first_word (q.b)) or bits_equal_axis (third_word (q.a), third_word (q.b)) then g.derived = g.derived + 1 end
		end
	end
	need_equal ("tq", "stage after the last record", expect_stage == 1 or expect_stage == 3, false)

	for group, name in pairs { [2] = "struct", [3] = "random" } do
		local row, g = tc [name], seen [group]
		local label = "tq group " .. name
		need_equal (label, "stage 0 records", g.s0, row.cases)
		need_equal (label, "stage 1 records", g.s1, row.cases)
		need_equal (label, "stage 2 records", g.s2, row.nearer_checked)
		need_equal (label, "stage 3 records", g.s3, row.nearer_checked)
		need_equal (label, "primary hits", g.hit, row.hit)
		need_equal (label, "warm agreements", g.warm_same, row.warm_same)
		need_equal (label, "miss-time point writes", g.miss_point, row.miss_point_written)
		need_equal (label, "miss-time normal writes", g.miss_normal, row.miss_normal_written)
		need_equal (label, "derived zero-extent rays", g.derived, row.derived_axis)
	end
end

local function write_trace (file, trace, meta)
	trace_write (file, { event = "trace-start", scenario = meta.scenario, seed = meta.seed, entity_cases = meta.entity_cases, tree_cases = meta.tree_cases, seconds = meta.seconds,
		frame_ms = meta.frame_ms })
	for _, item in ipairs (trace.order) do
		local t = { event = item.event }
		for k, v in pairs (item.typed) do t [k] = v end
		trace_write (file, t)
	end
	trace_write (file, { event = "trace-end", records = #(trace.many ["tq"] or {}), rows = #trace.order })
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
local first_trace, first_trace_text
local total_gaps = {}
-- The armed transcript and the open trace file are cleaned up on EVERY failure of the probe/trace path (a probe or read error, a rejected report or transcript, a differing
-- repeat, a write, flush or close error), not only on success. Cleanup is best effort and never replaces the primary failure: a cleanup error is recorded as its own event and the
-- original diagnostic is re-raised unchanged.
local trace_armed = false

local function fail_trace (err)
	pcall (emit, { event = "trace_failed", error = tostring (err) })
	if trace_armed then
		local ok, cleanup_error = pcall (function () engine:collision_trace_arm (false) end)
		if ok then
			trace_armed = false
		else
			pcall (emit, { event = "trace_cleanup_failed", what = "disarm", error = tostring (cleanup_error) })
		end
	end
	if trace_file then
		local ok, closed, cleanup_error = pcall (function () return trace_file:close () end)
		if not ok or not closed then
			pcall (emit, { event = "trace_cleanup_failed", what = "close", error = tostring (ok and cleanup_error or closed) })
		end
		trace_file = nil
	end
	error (err, 0)
end

if probe == "yes" then
	local function run_probes ()
	for repeat_index = 1, repeats do
		if trace_file then
			trace_armed = true
			engine:collision_trace_arm (true)
		end
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
			emit { event = "report_invalid", repeat_index = repeat_index, error = tostring (events), raw_report = report }
			error (tostring (events), 0)
		end
		for _, e in ipairs (events) do emit (e) end
		probed = true
		if trace_file then
			local trace, trace_text = read_trace (engine)
			check_trace (trace, rows, keyed)
			if repeat_index == 1 then
				first_trace, first_trace_text = trace, trace_text
			elseif trace_text ~= first_trace_text then
				emit { event = "trace_differs", repeat_index = repeat_index }
				refuse ("the tree-query transcript of repeat " .. repeat_index .. " differs from repeat 1")
			end
		end
		if repeat_index == 1 then
			first_report, first_rows, first_keyed = report, rows, keyed
		else
			local same = report == first_report
			if same then
				emit { event = "repeat_identical", repeat_index = repeat_index, ok = true }
			else
				-- the differing report is kept whole, as the invalid one is
				emit { event = "repeat_identical", repeat_index = repeat_index, ok = false, raw_report = report }
			end
			if not same then
				refuse ("repeat " .. repeat_index .. " differs from repeat 1: the report is not repeatable for the seed and the loaded world")
			end
		end
	end
	if trace_file then
		engine:collision_trace_arm (false)
		trace_armed = false
		-- every repeat's transcript has passed; only now is the private file written, and its last line says it is complete
		write_trace (trace_file, first_trace, { scenario = scenario_name, seed = seed, entity_cases = entity_cases, tree_cases = tree_cases, seconds = seconds, frame_ms = frame_ms })
		-- the close is checked too: a file is accepted only when this event is in the output, whatever the file's last line says
		local file = trace_file
		trace_file = nil
		local closed, close_error = file:close ()
		if not closed then
			trace_file = nil
			refuse ("the tree-query transcript could not be closed: " .. tostring (close_error))
		end
		emit { event = "trace_written", records = #(first_trace.many ["tq"] or {}), rows = #first_trace.order }
	end
	end

	if trace_file then
		local ok, err = pcall (run_probes)
		if not ok then fail_trace (err) end
	else
		run_probes ()
	end
	total_gaps = coverage_gaps (first_rows, first_keyed)
	emit { event = "coverage", gaps = (#total_gaps == 0) and "none" or table.concat (total_gaps, ","), gap_count = #total_gaps,
		moving_targets = first_rows ["col-targets"].moving_geometry + first_rows ["col-targets"].moving_box,
		structured_entity_hits = first_keyed ["col-entity-kind"] ["struct/through"].hit,
		structured_tree_hits = first_keyed ["col-tree-kind"] ["struct/through"].hit,
		random_entity_hits = first_keyed ["col-entity-kind"] ["random/random"].hit,
		random_tree_hits = first_keyed ["col-tree-kind"] ["random/random"].hit,
		fp_x87_masks = first_rows ["col-fp"].x87_masks, fp_sse_masks = first_rows ["col-fp"].sse_masks, fp_arch = first_rows ["col-fp"].arch,
		skipped_axis_rays = first_keyed ["col-tree-common"].random.skipped_axis,
		derived_axis_rays = first_keyed ["col-tree-common"].struct.derived_axis + first_keyed ["col-tree-common"].random.derived_axis }
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
