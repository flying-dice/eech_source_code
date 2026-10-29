-- TEST-ONLY ground-contact characterisation (EEDCS-48, D3 G2). Needs a DLL built with the `ground-fixture` feature; the shipping module
-- has no `ground_probe` method.
--
-- It boots the campaign through the public path (require ("eech_dc")) with the reference host's callbacks, optionally runs frames, and
-- then calls the one fixture entry, engine:ground_probe (seed, points, impacts), which runs the ORIGINAL point_above_ground,
-- point_below_ground and get_ground_impact_point against the terrain the campaign's own boot loaded (see csrc/eech_ground_fixture.c).
-- The report is `key=value` text lines; each becomes one JSON Lines event. Raw map coordinates and heights appear only in that
-- runtime output (float bit patterns in hex), which is kept outside Git.
--
-- The probe is repeated (`repeats`, default 2) in the same process with no frames between: the reports must be byte identical, which
-- shows the probe is a pure function of the seed and the loaded map, and that the shared terrain record is left as it was found (the
-- record's value words are reported before and after each probe and must match). A difference fails the run; nothing is tuned to pass.
--
-- A report is accepted only as a whole (read_report): every row once, with exactly its keys, each of its exact type (decimal count, flag, or
-- fixed-width lowercase hex), counts that agree with the request and with each other, and both folds present. `ground-hash` is the LEGACY fold
-- (it includes the original record's private bookkeeping); `ground-physical` is the PHYSICAL fold (inputs, answers and written vectors only). A
-- malformed or incomplete report records `report_invalid` and fails the run; no repeat_identical or state_digest row follows it.
-- `probe=no` is the control: the same frames, no probe call, and the same state digest at the end, so a probe run and a control run can be
-- compared offline. The digest is the object list's identity, kind, side and alive flag in id order after the frames: a coarse witness
-- that the probe did not disturb the campaign, not a proof (see docs/m5-ground-characterization.md).
--
-- host.args (every one checked before anything is booted; an unknown key, a malformed number or an out-of-range value is refused):
--   root=<installation root>  out=<JSON Lines path>                                              required
--   scenario  lebanon_retail (default), georgia_retail or georgia
--   seed      1 to 4294967295 (default 20261029): the private generator's seed; 0 is refused because the generator treats it as 1
--   points    0 to 100000 (default 2000): point cases (point_above_ground and point_below_ground)
--   impacts   0 to 100000 (default 500): impact cases (get_ground_impact_point); points and impacts are not both 0
--   repeats   1 to 5 (default 2): how many times the probe is run in a row
--   seconds   0 to 3600 (default 0): simulated seconds of frames before the probe (0 probes the freshly booted world)
--   after     0 to 3600 (default 0): simulated seconds of frames after the probe, before the digest
--   frame_ms  1 to 1000 (default 100)
--   probe     yes (default) or no (the control)

io.stdout:setvbuf ("no")

local args = host.args

local function refuse (message)
	error ("ground-characterization: " .. message, 0)
end

local known = { root = true, out = true, scenario = true, seed = true, points = true, impacts = true, repeats = true, seconds = true,
	after = true, frame_ms = true, probe = true }
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
local points = whole_arg ("points", args.points, 2000, 0, 100000)
local impacts = whole_arg ("impacts", args.impacts, 500, 0, 100000)
local repeats = whole_arg ("repeats", args.repeats, 2, 1, 5)
local seconds = whole_arg ("seconds", args.seconds, 0, 0, 3600)
local after = whole_arg ("after", args.after, 0, 0, 3600)
local frame_ms = whole_arg ("frame_ms", args.frame_ms, 100, 1, 1000)
local probe = choice_arg ("probe", args.probe, "yes", { "yes", "no" })
if probe == "yes" and points + impacts == 0 then refuse ("points and impacts are both 0: there is nothing to probe") end

local MAX_FRAMES = 72000
local frames_before = math.floor (seconds * 1000 / frame_ms)
local frames_after = math.floor (after * 1000 / frame_ms)
if frames_before + frames_after > MAX_FRAMES then
	refuse ("seconds and after need " .. (frames_before + frames_after) .. " frames of " .. frame_ms .. " ms; the most this driver runs is " .. MAX_FRAMES)
end

local out = assert (io.open (out_path, "w"))

-- A JSON string (RFC 8259). Lua's %q writes LUA source, not JSON: it leaves a tab (and every other control byte) raw, which a strict JSON parser refuses, and writes a
-- newline as a backslash and a real newline. So: the quote and the backslash are escaped, every control byte 0x00 to 0x1f becomes its short escape (\b \t \n \f \r) or
-- \u00XX, and every other byte, UTF-8 included, is written unchanged. Nothing is dropped or flattened: an error text with a traceback's tabs and newlines is recorded exactly.
local JSON_SHORT = { ["\b"] = "\\b", ["\t"] = "\\t", ["\n"] = "\\n", ["\f"] = "\\f", ["\r"] = "\\r", ["\""] = "\\\"", ["\\"] = "\\\\" }

local function json_string (text)
	local escaped = string.gsub (text, "[%z\1-\31\"\\]", function (c) return JSON_SHORT [c] or string.format ("\\u%04x", string.byte (c)) end)
	return "\"" .. escaped .. "\""
end

local function emit (t)
	local parts = {}
	for k, v in pairs (t) do
		local value = type (v) == "string" and json_string (v) or tostring (v)
		parts [#parts + 1] = json_string (tostring (k)) .. ":" .. value
	end
	table.sort (parts)
	out:write ("{" .. table.concat (parts, ",") .. "}\n")
	out:flush ()
end

-- THE REPORT SCHEMA. A report is a result only if every row the fixture writes is present exactly once with exactly its keys, every value has its
-- exact type (a decimal count, a flag, or fixed-width lowercase hex), and the counts agree with what was asked and with each other. Anything else is
-- refused by name; a missing or malformed measurement is never read as zero, as absent-equals-absent, or as a pass.
local DESCRIBE = {
	count = "a decimal count", signed = "a decimal integer", flag = "0 or 1", sflag = "-1, 0 or 1", word = "a word",
	hex16 = "16 lowercase hex digits", hex8 = "8 lowercase hex digits", hex8x3 = "three 8-digit lowercase hex words joined by commas",
}

local function typed_value (kind, text)
	if kind == "count" then
		if string.match (text, "^%d+$") and #text <= 10 then return tonumber (text) end
	elseif kind == "signed" then
		if string.match (text, "^-?%d+$") and #text <= 11 then return tonumber (text) end
	elseif kind == "flag" then
		if text == "0" or text == "1" then return tonumber (text) end
	elseif kind == "sflag" then
		if text == "-1" or text == "0" or text == "1" then return tonumber (text) end
	elseif kind == "word" then
		if string.match (text, "^%a+$") then return text end
	elseif kind == "hex16" then
		if string.match (text, "^[0-9a-f]+$") and #text == 16 then return text end
	elseif kind == "hex8" then
		if string.match (text, "^[0-9a-f]+$") and #text == 8 then return text end
	elseif kind == "hex8x3" then
		if string.match (text, "^%x%x%x%x%x%x%x%x,%x%x%x%x%x%x%x%x,%x%x%x%x%x%x%x%x$") and not string.match (text, "%u") then return text end
	end
	return nil
end

local SCHEMA = {
	["ground-probe"] = { { "seed", "count" }, { "points", "count" }, { "impacts", "count" }, { "samples_per_point_tag", "count" },
		{ "samples_per_impact_type", "count" } },
	-- the campaign's grid (it fixes the map extents) and the physical terrain's grid (which the sector boundaries belong to): two grids, reported apart
	["ground-map"] = { { "world_sectors_x", "count" }, { "world_sectors_z", "count" }, { "world_sector_side", "count" }, { "terrain_sectors_x", "count" },
		{ "terrain_sectors_z", "count" }, { "terrain_sector_side", "count" } },
	["ground-record"] = { { "anchored", "flag" }, { "restored", "flag" }, { "words_before", "hex16" }, { "words_after", "hex16" } },
	["ground-points"] = { { "cases", "count" }, { "in_map", "count" }, { "off_map", "count" }, { "input_mutated", "count" } },
	["ground-above"] = { { "cases", "count" }, { "true", "count" }, { "false", "count" }, { "record_touched", "count" },
		{ "true_record_untouched", "count" }, { "warm_same", "count" }, { "warm_start_intact", "count" } },
	["ground-below"] = { { "cases", "count" }, { "true", "count" }, { "false", "count" }, { "record_touched", "count" },
		{ "true_record_untouched", "count" }, { "warm_same", "count" }, { "warm_start_intact", "count" }, { "off_map_true", "count" },
		{ "off_map_record_touched", "count" } },
	["ground-complement"] = { { "in_map_cases", "count" }, { "complement", "count" }, { "not_complement", "count" }, { "both_true", "count" },
		{ "both_false", "count" } },
	["ground-impact"] = { { "cases", "count" }, { "a_corrected", "count" }, { "b_corrected", "count" }, { "xz_changed", "count" },
		{ "alias_equal", "count" }, { "alias_unequal", "count" }, { "warm_same", "count" }, { "warm_different", "count" },
		{ "record_touched", "count" }, { "record_untouched", "count" }, { "impact_equals_corrected_below", "count" },
		{ "warm_start_intact", "count" } },
	["ground-impact-classes"] = { { "invalid", "count" }, { "water", "count" }, { "forest", "count" }, { "land", "count" }, { "other", "count" } },
	["ground-impact-types"] = { { "t0", "count" }, { "t1", "count" }, { "t2", "count" }, { "t3", "count" }, { "t4", "count" }, { "t5", "count" },
		{ "t6", "count" }, { "t7", "count" } },
	["ground-asserts"] = { { "original_below_endpoint_still_above", "count" }, { "extra_above_endpoint_not_above", "count" } },
	-- the LEGACY fold (it includes the original record's private bookkeeping) and the PHYSICAL fold (inputs, answers, written vectors only)
	["ground-hash"] = { { "combined", "hex16" }, { "above", "hex16" }, { "below", "hex16" }, { "impact", "hex16" } },
	["ground-physical"] = { { "combined", "hex16" }, { "above", "hex16" }, { "below", "hex16" }, { "impact", "hex16" } },
}

local SAMPLE_SCHEMA = {
	point = { { "kind", "word" }, { "n", "count" }, { "tag", "count" }, { "x", "hex8" }, { "y", "hex8" }, { "z", "hex8" }, { "inside", "flag" },
		{ "above", "sflag" }, { "above_touched", "flag" }, { "below", "flag" }, { "below_touched", "flag" }, { "above_elev", "hex8" },
		{ "above_type", "signed" }, { "below_elev", "hex8" }, { "below_type", "signed" } },
	impact = { { "kind", "word" }, { "n", "count" }, { "type", "count" }, { "a0", "hex8x3" }, { "b0", "hex8x3" }, { "class_alias", "signed" },
		{ "pa", "hex8x3" }, { "qa", "hex8x3" }, { "class_separate", "signed" }, { "pb", "hex8x3" }, { "qb", "hex8x3" }, { "rb", "hex8x3" },
		{ "alias_equal", "flag" }, { "warm_same", "flag" }, { "touched", "flag" }, { "elev", "hex8" }, { "type_word", "signed" } },
}

-- the FNV-1a offset basis: the value of a fold that folded nothing
local FOLD_EMPTY = "cbf29ce484222325"

-- The sample quotas are compile-time constants of the fixture (POINT_SAMPLES_PER_TAG and IMPACT_SAMPLES_PER_TYPE in eech_ground_fixture.c), written into the
-- header of every report. The completeness checks below are only as strong as the quota they assume, so a header that names any other quota is refused
-- rather than believed: a changed quota would redefine which sample rows are required or permitted.
local EMITTER_POINT_QUOTA, EMITTER_IMPACT_QUOTA = 2, 4

-- one row's words after the event, as raw `key=value` text: a word that is not key=value, or a key that appears twice, is refused
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

-- the fields of a row against its schema: every key present, of its exact type, and no other key. Returns the typed values (hex stays text).
local function typed_row (event, schema, fields)
	local typed = {}
	for _, spec in ipairs (schema) do
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

-- A report as the fixture writes it, checked in full; returns the events to record (in report order) and the rows by name. Refuses, by name, anything
-- that is not a complete, well-typed result for the seed and counts asked.
local function read_report (report, repeat_index)
	local events, rows, samples = {}, {}, { point = 0, impact = 0 }
	local sample_list = {}
	for line in string.gmatch (report, "[^\r\n]+") do
		local words = {}
		for word in string.gmatch (line, "%S+") do words [#words + 1] = word end
		local event = words [1]
		if event == nil then refuse ("the report has a blank line") end
		if event == "ground-sample" then
			local fields = row_fields (event, words)
			local kind = fields.kind
			local schema = SAMPLE_SCHEMA [kind]
			if schema == nil then refuse ("ground-sample: kind must be point or impact, got '" .. tostring (kind) .. "'") end
			local raw = {}
			for k, v in pairs (fields) do raw [k] = v end
			local typed = typed_row (event, schema, fields)
			samples [kind] = samples [kind] + 1
			sample_list [#sample_list + 1] = { kind = kind, v = typed }
			-- a sample keeps every field as text: its words are bit patterns
			local t = { event = event, repeat_index = repeat_index }
			for k, v in pairs (raw) do t [k] = v end
			events [#events + 1] = t
		else
			local schema = SCHEMA [event]
			if schema == nil then refuse ("the report has an unknown row '" .. event .. "'") end
			if rows [event] ~= nil then refuse ("the report has more than one " .. event .. " row") end
			local typed = typed_row (event, schema, row_fields (event, words))
			rows [event] = typed
			local t = { event = event, repeat_index = repeat_index }
			for k, v in pairs (typed) do t [k] = v end
			events [#events + 1] = t
		end
	end
	for name in pairs (SCHEMA) do
		if rows [name] == nil then refuse ("the report has no " .. name .. " row") end
	end

	-- the report is for the request made
	local header = rows ["ground-probe"]
	if header.seed ~= seed or header.points ~= points or header.impacts ~= impacts then
		refuse (string.format ("the report is for seed %s points %s impacts %s, not the %s %s %s asked", tostring (header.seed), tostring (header.points),
			tostring (header.impacts), tostring (seed), tostring (points), tostring (impacts)))
	end
	need_equal ("ground-probe", "samples_per_point_tag", header.samples_per_point_tag, EMITTER_POINT_QUOTA)
	need_equal ("ground-probe", "samples_per_impact_type", header.samples_per_impact_type, EMITTER_IMPACT_QUOTA)
	local map = rows ["ground-map"]
	for _, key in ipairs { "world_sectors_x", "world_sectors_z", "world_sector_side", "terrain_sectors_x", "terrain_sectors_z", "terrain_sector_side" } do
		if map [key] < 1 then refuse ("ground-map: " .. key .. " is " .. map [key] .. ", the map has no sectors") end
	end
	-- The grids may differ (the campaign's sectors can be larger than the terrain's) but the ground they cover must not: the fixture refuses a probe whose world
	-- extents differ from the terrain's (reason world_map_is_not_the_terrain_extents), so a report that carries two grids covering different extents is not a
	-- report of that probe. The extent is sectors times side, less 1 in both, so the products are compared.
	need_equal ("ground-map", "world extent in x (world_sectors_x * world_sector_side)", map.world_sectors_x * map.world_sector_side,
		map.terrain_sectors_x * map.terrain_sector_side)
	need_equal ("ground-map", "world extent in z (world_sectors_z * world_sector_side)", map.world_sectors_z * map.world_sector_side,
		map.terrain_sectors_z * map.terrain_sector_side)

	-- the shared terrain record is left exactly as it was found
	local record = rows ["ground-record"]
	need_equal ("ground-record", "anchored", record.anchored, 1)
	if record.restored ~= 1 or record.words_before ~= record.words_after then
		refuse ("the probe did not restore the shared terrain record: restored=" .. tostring (record.restored) .. " words " .. tostring (record.words_before) ..
			" -> " .. tostring (record.words_after))
	end

	-- the counts agree with the request and with each other (a family with no cases is valid and reports zeros)
	local pts, above, below, comp = rows ["ground-points"], rows ["ground-above"], rows ["ground-below"], rows ["ground-complement"]
	need_equal ("ground-points", "cases", pts.cases, points)
	need_equal ("ground-points", "in_map + off_map", pts.in_map + pts.off_map, points)
	need_at_most ("ground-points", "input_mutated", pts.input_mutated, points)
	need_equal ("ground-above", "cases", above.cases, pts.in_map)
	need_equal ("ground-above", "true + false", above ["true"] + above ["false"], above.cases)
	need_at_most ("ground-above", "record_touched", above.record_touched, above.cases)
	need_at_most ("ground-above", "true_record_untouched", above.true_record_untouched, above ["true"])
	need_at_most ("ground-above", "warm_same", above.warm_same, above.cases)
	need_equal ("ground-above", "warm_start_intact", above.warm_start_intact, above.cases)
	need_equal ("ground-below", "cases", below.cases, points)
	need_equal ("ground-below", "true + false", below ["true"] + below ["false"], below.cases)
	need_at_most ("ground-below", "record_touched", below.record_touched, below.cases)
	need_at_most ("ground-below", "true_record_untouched", below.true_record_untouched, below ["true"])
	need_at_most ("ground-below", "warm_same", below.warm_same, below.cases)
	need_equal ("ground-below", "warm_start_intact", below.warm_start_intact, below.cases)
	need_at_most ("ground-below", "off_map_true", below.off_map_true, pts.off_map)
	need_at_most ("ground-below", "off_map_record_touched", below.off_map_record_touched, pts.off_map)
	need_equal ("ground-complement", "in_map_cases", comp.in_map_cases, pts.in_map)
	need_equal ("ground-complement", "complement + not_complement", comp.complement + comp.not_complement, comp.in_map_cases)
	need_at_most ("ground-complement", "both_true + both_false", comp.both_true + comp.both_false, comp.in_map_cases)

	local impact, classes, types, asserts = rows ["ground-impact"], rows ["ground-impact-classes"], rows ["ground-impact-types"], rows ["ground-asserts"]
	need_equal ("ground-impact", "cases", impact.cases, impacts)
	need_at_most ("ground-impact", "a_corrected", impact.a_corrected, impact.cases)
	need_at_most ("ground-impact", "b_corrected", impact.b_corrected, impact.cases)
	need_at_most ("ground-impact", "xz_changed", impact.xz_changed, impact.cases)
	need_equal ("ground-impact", "alias_equal + alias_unequal", impact.alias_equal + impact.alias_unequal, impact.cases)
	need_equal ("ground-impact", "warm_same + warm_different", impact.warm_same + impact.warm_different, impact.cases)
	need_equal ("ground-impact", "record_touched + record_untouched", impact.record_touched + impact.record_untouched, impact.cases)
	need_at_most ("ground-impact", "impact_equals_corrected_below", impact.impact_equals_corrected_below, impact.cases)
	need_equal ("ground-impact", "warm_start_intact", impact.warm_start_intact, impact.cases)
	need_equal ("ground-impact-classes", "invalid + water + forest + land + other",
		classes.invalid + classes.water + classes.forest + classes.land + classes.other, impacts)
	need_equal ("ground-impact-types", "t0 .. t7", types.t0 + types.t1 + types.t2 + types.t3 + types.t4 + types.t5 + types.t6 + types.t7, impacts)
	need_at_most ("ground-asserts", "original_below_endpoint_still_above", asserts.original_below_endpoint_still_above, impacts)
	need_at_most ("ground-asserts", "extra_above_endpoint_not_above", asserts.extra_above_endpoint_not_above, impacts)

	-- both folds: a family that ran no case folded nothing, and a family that ran folded something (the above and below folds take every point)
	for _, name in ipairs { "ground-hash", "ground-physical" } do
		local row = rows [name]
		for _, family in ipairs { { "above", points }, { "below", points }, { "impact", impacts } } do
			local key, ran = family [1], family [2]
			if (ran == 0) ~= (row [key] == FOLD_EMPTY) then
				refuse (name .. ": " .. key .. " is " .. row [key] .. " for " .. ran .. " cases")
			end
		end
		if row.combined == FOLD_EMPTY then refuse (name .. ": combined is the empty fold for " .. (points + impacts) .. " cases") end
	end

	-- THE SAMPLES. What must be present follows from how the fixture emits them (eech_ground_fixture.c), not from re-deriving its cases: the cases are
	-- visited in order, every point case has a tag 0 to 5 and every impact case a type 0 to 7, and a case is sampled when fewer than
	-- samples_per_point_tag (of its tag) or samples_per_impact_type (of its type) earlier cases were. So: the point samples come before the impact
	-- samples; within a family the case numbers are distinct, increasing and below the case count; case 0 is always sampled (nothing precedes it);
	-- a tag has at most samples_per_point_tag samples; and, because the impact type tally is reported, a type has EXACTLY
	-- min(its tally, samples_per_impact_type) samples. The per-tag tally of point cases is not reported, so a point tag is bounded only from above.
	local POINT_TAGS, IMPACT_TYPES = 6, 8
	local per_tag_cap, per_type_cap = EMITTER_POINT_QUOTA, EMITTER_IMPACT_QUOTA
	if (points == 0) ~= (samples.point == 0) then refuse ("ground-sample: " .. samples.point .. " point samples for " .. points .. " point cases") end
	if (impacts == 0) ~= (samples.impact == 0) then refuse ("ground-sample: " .. samples.impact .. " impact samples for " .. impacts .. " impact cases") end
	local family = { point = { cases = points, last = nil, by_class = {}, sampled = {} }, impact = { cases = impacts, last = nil, by_class = {}, sampled = {} } }
	local seen_impact = false
	local flags = { inside = { [0] = 0, [1] = 0 }, above = { [0] = 0, [1] = 0 }, below = { [0] = 0, [1] = 0 },
		alias_equal = { [0] = 0, [1] = 0 }, warm_same = { [0] = 0, [1] = 0 } }
	for _, s in ipairs (sample_list) do
		local kind, v = s.kind, s.v
		local f = family [kind]
		if kind == "impact" then seen_impact = true elseif seen_impact then refuse ("ground-sample: a point sample follows an impact sample") end
		if v.n >= f.cases then refuse ("ground-sample: " .. kind .. " sample names case " .. v.n .. " of " .. f.cases) end
		if f.last == nil then
			if v.n ~= 0 then refuse ("ground-sample: the first " .. kind .. " sample names case " .. v.n .. ", not case 0") end
		elseif v.n == f.last then
			refuse ("ground-sample: " .. kind .. " sample repeats case " .. v.n)
		elseif v.n < f.last then
			refuse ("ground-sample: " .. kind .. " sample case " .. v.n .. " follows case " .. f.last)
		end
		f.last = v.n
		local class, domain, cap = v.tag, POINT_TAGS, per_tag_cap
		if kind == "impact" then class, domain, cap = v.type, IMPACT_TYPES, per_type_cap end
		local class_name = kind == "point" and "tag" or "type"
		if class >= domain then refuse ("ground-sample: " .. class_name .. " must be 0 to " .. (domain - 1) .. ", got " .. class) end
		f.by_class [class] = (f.by_class [class] or 0) + 1
		f.sampled [#f.sampled + 1] = { n = v.n, class = class }
		need_at_most ("ground-sample", kind .. " samples of " .. class_name .. " " .. class, f.by_class [class], cap)
		if kind == "point" then
			-- the same values the counters count: whether the point was inside the map area, and what the two macros answered
			if v.inside == 0 and (v.above ~= -1 or v.above_touched ~= 0) then
				refuse ("ground-sample: point sample at case " .. v.n .. " is off the map area but has above=" .. v.above .. " above_touched=" .. v.above_touched)
			end
			if v.inside == 1 and v.above == -1 then refuse ("ground-sample: point sample at case " .. v.n .. " is inside the map area but was not asked above") end
			flags.inside [v.inside] = flags.inside [v.inside] + 1
			if v.above ~= -1 then flags.above [v.above] = flags.above [v.above] + 1 end
			flags.below [v.below] = flags.below [v.below] + 1
		else
			flags.alias_equal [v.alias_equal] = flags.alias_equal [v.alias_equal] + 1
			flags.warm_same [v.warm_same] = flags.warm_same [v.warm_same] + 1
		end
	end
	-- the case numbers above are within the case count and distinct, so each sampled case is a case the counters counted once
	need_at_most ("ground-sample", "point samples inside the map area", flags.inside [1], pts.in_map)
	need_at_most ("ground-sample", "point samples off the map area", flags.inside [0], pts.off_map)
	need_at_most ("ground-sample", "point samples with above=1", flags.above [1], above ["true"])
	need_at_most ("ground-sample", "point samples with above=0", flags.above [0], above ["false"])
	need_at_most ("ground-sample", "point samples with below=1", flags.below [1], below ["true"])
	need_at_most ("ground-sample", "point samples with below=0", flags.below [0], below ["false"])
	need_at_most ("ground-sample", "impact samples with alias_equal=1", flags.alias_equal [1], impact.alias_equal)
	need_at_most ("ground-sample", "impact samples with alias_equal=0", flags.alias_equal [0], impact.alias_unequal)
	need_at_most ("ground-sample", "impact samples with warm_same=1", flags.warm_same [1], impact.warm_same)
	need_at_most ("ground-sample", "impact samples with warm_same=0", flags.warm_same [0], impact.warm_different)
	-- every impact type has exactly the samples the emitter takes for it: the first min(tally, cap) cases of that type
	for k = 0, IMPACT_TYPES - 1 do
		local expected = math.min (types ["t" .. k], per_type_cap)
		local actual = family.impact.by_class [k] or 0
		if actual ~= expected then refuse ("ground-sample: impact samples of type " .. k .. " is " .. actual .. ", expected " .. expected) end
	end

	-- NO CASE MAY BE SKIPPED BEFORE A CLASS IS FULL. The emitter samples a case unless its own tag (or type) already has its quota, and a skipped case's class is not
	-- reported. So a case that is not sampled is explained only if some class has already reached its quota among the samples before it; while no class has,
	-- every consecutive case must be sampled, and a gap (or a tail that stops short) is refused. After a class is full, skipped cases may belong to it, and the
	-- driver does not guess which: it has no per-tag population of the point cases and does not reproduce the generator.
	for _, name in ipairs { "point", "impact" } do
		local f = family [name]
		local quota = name == "point" and per_tag_cap or per_type_cap
		local class_name = name == "point" and "tag" or "type"
		local counts, last, full = {}, -1, false
		for _, item in ipairs (f.sampled) do
			if item.n > last + 1 and not full then
				refuse ("ground-sample: " .. name .. " case " .. (last + 1) .. " is not sampled, but no " .. class_name .. " has reached " .. quota ..
					" samples (the gap is cases " .. (last + 1) .. " to " .. (item.n - 1) .. ")")
			end
			counts [item.class] = (counts [item.class] or 0) + 1
			if counts [item.class] >= quota then full = true end
			last = item.n
		end
		if last < f.cases - 1 and not full then
			refuse ("ground-sample: " .. name .. " samples stop at case " .. last .. " of " .. f.cases .. ", but no " .. class_name .. " has reached " .. quota ..
				" samples (cases " .. (last + 1) .. " to " .. (f.cases - 1) .. " are unsampled)")
		end
	end
	return events, rows
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
emit { event = "boot", scenario = scenario_name, campaign_seed = 1, seed = seed, points = points, impacts = impacts, repeats = repeats, seconds = seconds, after = after,
	frame_ms = frame_ms, probe = probe }

local function run_frames (count)
	for _ = 1, count do engine:frame (frame_ms) end
end

run_frames (frames_before)

local first_report, last_rows
if probe == "yes" then
	for r = 1, repeats do
		local ok, report = pcall (function () return engine:ground_probe (seed, points, impacts) end)
		if not ok then
			-- a refused probe (a precondition not held, a report that does not fit) is an unavailable characterisation, never a quiet success
			emit { event = "probe_refused", repeat_index = r, error = tostring (report) }
			error ("ground-characterization: the probe was refused: " .. tostring (report), 0)
		end
		-- a report is a result only when the whole schema holds (see read_report); otherwise the run fails here, with the reason recorded, and no
		-- success row (repeat_identical, state_digest) is ever written for it
		local valid, events, rows = pcall (read_report, report, r)
		if not valid then
			emit { event = "report_invalid", repeat_index = r, error = tostring (events) }
			error (events, 0)
		end
		for _, t in ipairs (events) do emit (t) end
		if first_report == nil then
			first_report = report
		else
			-- the same seed on the same loaded map and the same record: the same report, byte for byte
			local same = report == first_report
			emit { event = "repeat_identical", repeat_index = r, ok = same }
			if not same then refuse ("repeat " .. r .. " differs from repeat 1: the probe is not a pure function of the seed and the loaded map") end
		end
		last_rows = rows
	end
else
	emit { event = "probe_skipped", reason = "probe=no (the control)" }
end

run_frames (frames_after)

-- the coarse state digest: the object list's identity, kind, side and alive flag in id order (a modular polynomial hash, exact in doubles).
-- The modulus keeps it below 2^31: Lua 5.1 formats %d and %x through a C long, which is 32 bits on Windows.
local objects = engine:objects ()
table.sort (objects, function (a, b) return a.id < b.id end)
local digest, alive = 5381, 0
for _, o in ipairs (objects) do
	local text = tostring (o.id) .. ":" .. tostring (o.kind) .. ":" .. tostring (o.side) .. ":" .. tostring (o.alive) .. ";"
	for i = 1, #text do digest = (digest * 31 + string.byte (text, i)) % 2147483647 end
	if o.alive then alive = alive + 1 end
end
local clock = engine:clock ()
emit { event = "state_digest", objects = #objects, alive = alive, digest = string.format ("%08x", digest), elapsed = clock.elapsed_seconds,
	frames = frames_before + frames_after, probed = probe == "yes" }
out:close ()
local legacy = last_rows and last_rows ["ground-hash"].combined or "none"
local physical = last_rows and last_rows ["ground-physical"].combined or "none"
io.stderr:write (string.format ("ground-characterization: probe %s, %d repeats, legacy hash %s, physical hash %s, state digest %08x over %d objects\n", probe,
	probe == "yes" and repeats or 0, tostring (legacy), tostring (physical), digest, #objects))
