-- Original target-point observer (EEDCS-52, D3). TEST-ONLY: it needs a module built with the `target-point-fixture` feature (the ORIGINAL get_local_entity_target_point of
-- en_maths.c, its callers weapon.c fill_aim_target and the weapon step's target-vector lookup, and the boot census of target_point_offsets[] made by wn_dbase.c, all observed by
-- csrc/eech_tp_fixture.c) and the reference host (EECH_WORLD). It is an OBSERVER: it boots the campaign exactly as lua/campaign.lua does (the same preparation, the reference
-- arithmetic profile, the same boot configuration, seed 1), runs the frames it is asked for and asks the campaign for nothing else. It selects no target, creates no entity, forces
-- no engagement and calls the original function from nowhere: the query calls it reports are only those the campaign's own update loop made.
--
-- WHAT IT REPORTS (all native text kept verbatim in the JSON Lines rows).
--   * the CENSUS of the offset table, read right after the boot and again at the end (the two must be byte-equal): per model index whether the offset came from the model's TARGET_POINT
--     sub-object, from the bounding-box fallback (with the six bounds the original used) or is the invalid index's zero, and the float bits. The census is the original's own boot
--     product; nothing is constructed or measured again. It is required to be WHOLE (it ran once, recorded every index once, and the live table still holds what it recorded).
--   * the QUERIES the campaign made while it booted and ran: per call the caller, the branch, the entity type, whether the host had ALREADY issued an identity for the entity (the
--     observer reads that, it issues none), the semantic model name, the offset and heading it used, the float bits it returned and the instance viewpoint before and after.
--     A short run may reach no query, or only some branches: that is reported as it is (`queries`, `branches_reached`, `natural_queries_reached`), never as coverage.
--
-- THREE KINDS OF OUTCOME, kept apart.
--   * CAPTURE WHOLE: the census is whole, no query was dropped, interrupted, orphaned, unclassified or out of range, and every report is well formed and coherent. Passes, even
--     if NO query was reached; `natural_queries_reached=false` is then stated in the done row, an `unreached` row and the standard error, and nothing is claimed covered.
--   * WELL-FORMED BUT NOT WHOLE: the grammar and counts are coherent but the observer says the capture is not whole (a full event table, a call that never ended ...). The remaining
--     rows are still read for diagnosis, then done complete=false is written and the run FAILS.
--   * MALFORMED: a line the C grammar cannot produce, an out-of-domain or contradictory number, rows that disagree with each other or with the summary, a `whole` claim the counters do
--     not support, a clock that does not advance, a census that changed. The text is already written verbatim; a `malformed` row and done complete=false follow, the script FAILS, and
--     the engine is NOT asked again. An engine call that itself fails stops the questions at once.
--
-- host.args: root (installation root, required), out (the JSON Lines file, required; created exclusively), scenario (lebanon_retail, the default, or georgia_retail),
--            seconds (default 3.6), frame_ms (default 100; seconds*1000/frame_ms must be a whole number of frames), checkpoint_every (simulated seconds between summary samples,
--            default 1; 0 samples only at the start and the end).
--
-- OUTPUT. The file is created with host.create_exclusive: atomic, exclusive, refusing ANY existing target, and every write, flush and close is checked. The owned output is closed
-- on EVERY exit after it was created; a failure to close is reported together with the primary failure, if there is one.
--
-- The caps and names below repeat the C emitter (csrc/eech_tp_fixture.c); tests/target_point_characterization.rs checks that they are the same.

io.stdout:setvbuf ("no")

local here = debug.getinfo (1, "S").source:match ("^@?(.*[/\\])") or ""
local json = dofile (here .. "json-lines.lua")

local args = host.args

-- ---------------------------------------------------------------------------------------------------------------------------------
-- every argument is checked, strictly, BEFORE the output is created, the installation is prepared or anything is booted

local scenarios = {
	lebanon_retail = { map = "..\\common\\maps\\map5", campaign = "LEBANON.CHC" },
	georgia_retail = { map = "..\\common\\maps\\map3", campaign = "GEORGIA.CHC" },
}

local function fail_args (message) error ("target-point-characterization: " .. message, 0) end

local known = { root = true, out = true, scenario = true, seconds = true, frame_ms = true, checkpoint_every = true }
for key in pairs (args) do
	if not known [key] then fail_args ("unknown argument " .. tostring (key)) end
end
local root = args.root
if type (root) ~= "string" or root == "" then fail_args ("root=<installation root> is required") end
local out_path = args.out
if type (out_path) ~= "string" or out_path == "" then fail_args ("out=<JSON Lines file> is required") end
local scenario_name = args.scenario or "lebanon_retail"
local scenario = scenarios [scenario_name]
if not scenario then fail_args ("scenario=lebanon_retail|georgia_retail") end

local MAX_FRAMES = 108000
local function number_arg (name, text, default)
	local value = tonumber (text == nil and default or text)
	if value == nil or value ~= value or value == math.huge or value == -math.huge then fail_args (name .. " is not a number: " .. tostring (text)) end
	return value
end
local seconds = number_arg ("seconds", args.seconds, "3.6")
local frame_ms = number_arg ("frame_ms", args.frame_ms, "100")
local checkpoint_every = number_arg ("checkpoint_every", args.checkpoint_every, "1")
if seconds <= 0 or seconds > 3600 then fail_args ("seconds must be above 0 and at most 3600") end
if frame_ms < 10 or frame_ms > 1000 or frame_ms % 1 ~= 0 then fail_args ("frame_ms must be a whole number of milliseconds from 10 to 1000") end
if checkpoint_every < 0 then fail_args ("checkpoint_every must not be negative") end
local frames = seconds * 1000 / frame_ms
if math.abs (frames - math.floor (frames + 0.5)) > 1e-6 then fail_args ("seconds*1000/frame_ms is not a whole number of frames: " .. tostring (frames)) end
frames = math.floor (frames + 0.5)
if frames < 1 or frames > MAX_FRAMES then fail_args ("the run must be 1 to " .. MAX_FRAMES .. " frames, not " .. tostring (frames)) end
if type (host.create_exclusive) ~= "function" then fail_args ("this host has no create_exclusive: an output that may never overwrite anything cannot be made safely") end

-- ---------------------------------------------------------------------------------------------------------------------------------
-- the native report grammar (csrc/eech_tp_fixture.c), checked line by line

local EVENTS_CAPACITY = 4096
local HOST_ID_MAX = 128
local INT_MAX = 2147483647

local SITES = { [0] = "unattributed", "aim_vectors", "aim_lead", "aim_intercept", "weapon_step_target_vec", "other" }
local SITE_COUNT = 6
local BRANCHES = { "instance_target_point", "instance_position", "fixed_shape_offset", "position_fallback" }
local CASES = {
	[1] = { "target_point_subobject" },
	[2] = { "instance_without_target_point" },
	[3] = { "no_horizontal_offset", "heading_zero_quirk", "rotated_horizontal_offset" },
	[4] = { "not_fixed", "invalid_shape" },
}
local STATES = { none = true, sub_object = true, bbox = true, invalid_zero = true }

local function malformed (message) error ({ malformed = message }, 0) end

-- a decimal integer exactly as C's %d writes it (no plus sign, no leading zero, no "-0"), within lo..hi
local function int (text, lo, hi, what)
	if type (text) ~= "string" or #text > 11 or not (text == "0" or string.match (text, "^%-?[1-9]%d*$")) then malformed (what .. " is not a decimal integer: " .. tostring (text)) end
	local n = tonumber (text)
	if n < lo or n > hi then malformed (string.format ("%s %s is outside %d..%d", what, text, lo, hi)) end
	return n
end

local function flag (text, what) return int (text, 0, 1, what) end

-- a 32-bit word exactly as C's %08x writes it
local HEX8 = "^" .. string.rep ("[0-9a-f]", 8) .. "$"
local function word (text, what)
	if type (text) ~= "string" or not string.match (text, HEX8) then malformed (what .. " is not a lower-case 8-digit word: " .. tostring (text)) end
	return text
end

local function words (text, n, what)
	local list = {}
	for item in string.gmatch (text .. ",", "([^,]*),") do list [#list + 1] = word (item, what) end
	if #list ~= n then malformed (string.format ("%s has %d words, expected %d", what, #list, n)) end
	return list
end

-- the magnitude of a single-precision word (the sign bit dropped) as a number, to ask only whether a float is zero
local function magnitude (w) return tonumber (w, 16) % 2147483648 end

-- a table name as one token of the characters the emitter allows
local function name (text, what)
	if type (text) ~= "string" or #text > 95 or not string.match (text, "^[%w_%.%-]+$") then malformed (what .. " is not a name: " .. tostring (text)) end
	return text
end

-- the text as a list of lines: only complete lines, none empty, no carriage return, no NUL
local function lines_of (text, what)
	if text == "" or string.sub (text, -1) ~= "\n" then malformed (what .. " does not end in a complete line") end
	if string.find (text, "[%z\r]") then malformed (what .. " holds a NUL or a carriage return") end
	local list = {}
	for line in string.gmatch (text, "([^\n]*)\n") do
		if line == "" then malformed (what .. " holds an empty line") end
		list [#list + 1] = line
	end
	return list
end

-- one line of single-space separated tokens: its name, then exactly `keys` in order as key=value; returns the raw values
local function fields (line, line_name, keys, what)
	local tokens = {}
	for token in string.gmatch (line, "[^ ]+") do tokens [#tokens + 1] = token end
	if table.concat (tokens, " ") ~= line then malformed (what .. " has irregular spacing: " .. line) end
	if tokens [1] ~= line_name then malformed (string.format ("%s is '%s', expected '%s'", what, tostring (tokens [1]), line_name)) end
	if #tokens ~= #keys + 1 then malformed (string.format ("%s ('%s') has %d fields, expected %d", what, line_name, #tokens - 1, #keys)) end
	local values = {}
	for i, key in ipairs (keys) do
		local k, v = string.match (tokens [i + 1], "^([%w_]+)=(.+)$")
		if k ~= key then malformed (string.format ("%s ('%s') field %d is '%s', expected '%s'", what, line_name, i, tostring (k), key)) end
		values [key] = v
	end
	return values
end

-- The summary: counters, the branch and caller matrix, the case counts, the census totals and the two verdicts.
local function check_summary (text)
	local list = lines_of (text, "the summary")
	local at = 0
	local function take (line_name, keys)
		at = at + 1
		if not list [at] then malformed ("the summary ends before a '" .. line_name .. "' line") end
		return fields (list [at], line_name, keys, "summary line " .. at)
	end

	local s = take ("tp-summary", { "calls", "events", "event_capacity", "dropped", "stale", "interrupted", "orphans", "unclassified", "open", "out_of_range", "host_id_overlong" })
	local m = {
		calls = int (s.calls, 0, INT_MAX, "calls"), events = int (s.events, 0, EVENTS_CAPACITY, "events"), dropped = int (s.dropped, 0, INT_MAX, "dropped"),
		stale = int (s.stale, 0, INT_MAX, "stale"), interrupted = int (s.interrupted, 0, INT_MAX, "interrupted"), orphans = int (s.orphans, 0, INT_MAX, "orphans"),
		unclassified = int (s.unclassified, 0, INT_MAX, "unclassified"), open = flag (s.open, "open"), out_of_range = int (s.out_of_range, 0, INT_MAX, "out_of_range"),
		overlong = int (s.host_id_overlong, 0, INT_MAX, "host_id_overlong"),
	}
	if int (s.event_capacity, 0, INT_MAX, "event_capacity") ~= EVENTS_CAPACITY then malformed ("event_capacity is not the observer's " .. EVENTS_CAPACITY) end
	-- every begun call is exactly one of: a stored event, a dropped one, one abandoned by the next call, or the one still open
	if m.calls ~= m.events + m.dropped + m.interrupted + m.open then
		malformed (string.format ("calls %d is not events %d + dropped %d + interrupted %d + open %d", m.calls, m.events, m.dropped, m.interrupted, m.open))
	end
	if m.dropped > 0 and m.events ~= EVENTS_CAPACITY then malformed ("calls were dropped while the event table was not full") end
	local finalised = m.events + m.dropped

	m.branch = {}
	local branch_total = 0
	for b = 1, 4 do
		local v = take ("tp-branch", { "branch", "name", "calls" })
		if int (v.branch, 1, 4, "branch") ~= b or v.name ~= BRANCHES [b] then malformed ("branch row " .. b .. " is not branch " .. b .. " " .. BRANCHES [b]) end
		m.branch [b] = int (v.calls, 0, INT_MAX, "branch calls")
		branch_total = branch_total + m.branch [b]
	end
	if branch_total + m.unclassified ~= finalised then malformed (string.format ("the branches (%d) and unclassified (%d) are not the finalised calls (%d)", branch_total, m.unclassified, finalised)) end

	m.site = {}
	local column = { 0, 0, 0, 0 }
	for s_index = 0, SITE_COUNT - 1 do
		local v = take ("tp-site", { "site", "name", "b1", "b2", "b3", "b4" })
		if int (v.site, 0, SITE_COUNT - 1, "site") ~= s_index or v.name ~= SITES [s_index] then malformed ("site row " .. s_index .. " is not site " .. s_index .. " " .. SITES [s_index]) end
		m.site [s_index] = {}
		for b = 1, 4 do
			local n = int (v ["b" .. b], 0, INT_MAX, "site count")
			m.site [s_index] [b] = n
			column [b] = column [b] + n
		end
	end
	for b = 1, 4 do
		if column [b] ~= m.branch [b] then malformed ("the caller rows do not add up to branch " .. b) end
	end

	local c = take ("tp-case", { "target_point", "instance_position", "no_horizontal", "heading_zero", "rotated", "not_fixed", "invalid_shape", "vp_changed", "host_id_present", "host_id_absent" })
	m.case = {}
	for _, key in ipairs { "target_point", "instance_position", "no_horizontal", "heading_zero", "rotated", "not_fixed", "invalid_shape", "vp_changed", "host_id_present", "host_id_absent" } do
		m.case [key] = int (c [key], 0, INT_MAX, "case " .. key)
	end
	if m.case.target_point ~= m.branch [1] or m.case.instance_position ~= m.branch [2] then malformed ("the instance case counts are not their branches' calls") end
	if m.case.no_horizontal + m.case.heading_zero + m.case.rotated ~= m.branch [3] then malformed ("the fixed-shape cases are not branch 3's calls") end
	if m.case.not_fixed + m.case.invalid_shape ~= m.branch [4] then malformed ("the fallback cases are not branch 4's calls") end
	if m.case.vp_changed > m.branch [1] + m.branch [2] + m.unclassified then malformed ("more instance viewpoints changed than there were instance calls") end
	if m.case.host_id_present + m.case.host_id_absent + m.overlong ~= finalised then malformed ("the host identity counts are not the finalised calls") end

	local k = take ("tp-census", { "runs", "ended", "open", "expected", "recorded", "sub_object", "bbox", "invalid_zero", "duplicates", "stray", "live_mismatch" })
	m.census = {
		runs = int (k.runs, 0, INT_MAX, "census runs"), ended = int (k.ended, 0, INT_MAX, "census ended"), open = flag (k.open, "census open"),
		expected = int (k.expected, 1, INT_MAX, "census expected"), recorded = int (k.recorded, 0, INT_MAX, "census recorded"), sub_object = int (k.sub_object, 0, INT_MAX, "census sub_object"),
		bbox = int (k.bbox, 0, INT_MAX, "census bbox"), invalid_zero = int (k.invalid_zero, 0, INT_MAX, "census invalid_zero"), duplicates = int (k.duplicates, 0, INT_MAX, "census duplicates"),
		stray = int (k.stray, 0, INT_MAX, "census stray"), live_mismatch = int (k.live_mismatch, 0, INT_MAX, "census live_mismatch"),
	}
	local census = m.census
	if census.ended > census.runs then malformed ("the census ended more often than it began") end
	if census.sub_object + census.bbox + census.invalid_zero ~= census.recorded or census.recorded > census.expected or census.invalid_zero > 1 or census.live_mismatch > census.recorded then
		malformed ("the census totals are incoherent")
	end

	local v = take ("tp-complete", { "census", "queries_whole" })
	m.census_whole = flag (v.census, "census whole")
	m.queries_whole = flag (v.queries_whole, "queries_whole")
	if list [at + 1] then malformed ("the summary has extra lines after its last expected one: " .. list [at + 1]) end

	-- a verdict is only believed if the counters say the same, both ways
	local census_clean = census.runs == 1 and census.ended == 1 and census.open == 0 and census.recorded == census.expected and census.duplicates == 0 and census.stray == 0 and
		census.live_mismatch == 0 and census.invalid_zero == 1
	if (m.census_whole == 1) ~= census_clean then malformed ("tp-complete census=" .. m.census_whole .. " contradicts the census counters") end
	local queries_clean = m.dropped == 0 and m.stale == 0 and m.interrupted == 0 and m.orphans == 0 and m.unclassified == 0 and m.open == 0 and m.out_of_range == 0 and m.overlong == 0
	if (m.queries_whole == 1) ~= queries_clean then malformed ("tp-complete queries_whole=" .. m.queries_whole .. " contradicts the query counters") end
	return m
end

-- The census rows: one per model index, in order. Returns the counts they hold, joined against the summary by the caller.
local function check_offsets (text, summary)
	local list = lines_of (text, "the census")
	local expected = summary.census.expected
	if #list ~= expected then malformed (string.format ("the census has %d rows, the summary expects %d", #list, expected)) end
	local counts = { none = 0, sub_object = 0, bbox = 0, invalid_zero = 0, live_mismatch = 0 }
	for i = 0, expected - 1 do
		local what = "census row " .. i
		local r = fields (list [i + 1], "tp-offset", { "index", "name", "state", "offset", "bounds", "live" }, what)
		if int (r.index, 0, INT_MAX, "census index") ~= i then malformed (what .. " is out of order") end
		name (r.name, what .. " name")
		if not STATES [r.state] then malformed (what .. " has the state '" .. tostring (r.state) .. "'") end
		counts [r.state] = counts [r.state] + 1
		if r.state == "none" then
			if r.offset ~= "-" or r.bounds ~= "-" or r.live ~= "-" then malformed (what .. " is not recorded but carries values") end
		else
			local offset = words (r.offset, 3, what .. " offset")
			if r.state == "bbox" then words (r.bounds, 6, what .. " bounds") elseif r.bounds ~= "-" then malformed (what .. " has bounds but did not use the bounding box") end
			if flag (r.live, what .. " live") == 0 then counts.live_mismatch = counts.live_mismatch + 1 end
			if r.state == "invalid_zero" then
				if i ~= 0 then malformed (what .. " is the invalid index's zero but is not index 0") end
				for _, w in ipairs (offset) do if magnitude (w) ~= 0 then malformed (what .. " is the invalid index's zero but holds a non-zero word") end end
			elseif i == 0 then
				malformed (what .. " is index 0 but is not the invalid index's zero")
			end
		end
	end
	return counts
end

-- One recorded query. `index` is the number asked for. Returns its facts.
local function check_event (text, index, summary)
	local what = "event " .. index
	local list = lines_of (text, what)
	if #list ~= 1 then malformed (what .. " is not exactly one line") end
	local head, length, rest = string.match (list [1], "^(.-) host_id_len=(%d+)(.*)$")
	if not head then malformed (what .. " has no host_id_len") end
	local e = fields (head, "tp-event", { "index", "seq", "site", "site_name", "detail", "branch", "branch_name", "case", "entity_type", "entity_type_name", "instance", "model", "model_name",
		"fixed", "shape", "horizontal", "heading", "xz_applied", "offset", "result", "vp_before", "vp_after", "vp_changed", "host_id_state" }, what)
	if int (e.index, 0, INT_MAX, "event index") ~= index then malformed (what .. " carries another event's index") end
	local f = {
		seq = int (e.seq, 0, INT_MAX, "seq"), site = int (e.site, 0, SITE_COUNT - 1, "site"), detail = int (e.detail, -INT_MAX, INT_MAX, "detail"), branch = int (e.branch, 0, 4, "branch"),
		entity_type = int (e.entity_type, 0, INT_MAX, "entity_type"), instance = flag (e.instance, "instance"), model = int (e.model, -1, INT_MAX, "model"), fixed = flag (e.fixed, "fixed"),
		shape = int (e.shape, -INT_MAX, INT_MAX, "shape"), horizontal = flag (e.horizontal, "horizontal"), xz_applied = flag (e.xz_applied, "xz_applied"), vp_changed = flag (e.vp_changed, "vp_changed"),
		case = e.case, host_id_state = e.host_id_state,
	}
	if f.seq >= summary.calls then malformed (what .. " has a sequence number beyond the calls the summary counts") end
	if e.site_name ~= SITES [f.site] then malformed (what .. " names site " .. f.site .. " '" .. e.site_name .. "'") end
	if f.site == 0 and f.detail ~= 0 then malformed (what .. " is unattributed but carries a caller detail") end
	if f.site == 4 and f.detail ~= 1 then malformed (what .. " is a weapon step lookup of a question other than the target point") end
	name (e.entity_type_name, what .. " entity_type_name")
	name (e.model_name, what .. " model_name")
	if f.model == -1 and e.model_name ~= "-" then malformed (what .. " has no model but names one") end
	if f.branch == 0 then
		if e.branch_name ~= "-" or f.case ~= "-" then malformed (what .. " reached no branch but names one") end
	else
		if e.branch_name ~= BRANCHES [f.branch] then malformed (what .. " names branch " .. f.branch .. " '" .. e.branch_name .. "'") end
		local allowed = false
		for _, c in ipairs (CASES [f.branch]) do if c == f.case then allowed = true end end
		if not allowed then malformed (what .. " has the case '" .. tostring (f.case) .. "' under branch " .. f.branch) end
	end

	-- the instance viewpoint before and after: known exactly when there was an instance, and "changed" exactly when the words differ
	if f.instance == 1 then
		words (e.vp_before, 12, what .. " vp_before")
		words (e.vp_after, 12, what .. " vp_after")
		if (e.vp_before ~= e.vp_after) ~= (f.vp_changed == 1) then malformed (what .. " vp_changed disagrees with its before and after words") end
		if f.model < 0 then malformed (what .. " has an instance without a model") end
	else
		if e.vp_before ~= "-" or e.vp_after ~= "-" or f.vp_changed ~= 0 then malformed (what .. " has no instance but carries a viewpoint") end
	end
	words (e.result, 3, what .. " result")

	-- the heading and the offset are known only on the paths that read them
	if e.heading ~= "-" then f.heading = word (e.heading, what .. " heading") end
	if e.offset ~= "-" then f.offset = words (e.offset, 3, what .. " offset") end
	if f.branch == 1 or f.branch == 2 then
		if f.instance ~= 1 or f.fixed ~= 0 or f.shape ~= -1 or f.horizontal ~= 0 or f.heading or f.xz_applied ~= 0 or f.offset then malformed (what .. " is an instance branch but carries fixed-object values") end
		if f.branch == 2 and f.vp_changed ~= 0 then malformed (what .. " answered the entity position but its instance viewpoint changed") end
	elseif f.branch == 3 then
		if f.instance ~= 0 or f.fixed ~= 1 or f.shape == 0 then malformed (what .. " is a fixed-shape branch without a fixed, instance-less, non-invalid shape") end
		-- the offset is read only for a shape inside the table; an outside shape is the observer's out_of_range count and not a measurement
		if (f.offset ~= nil) ~= (f.shape >= 1 and f.shape < summary.census.expected) then malformed (what .. " has an offset that does not agree with its shape being inside the table") end
		if f.offset then
			if f.model ~= f.shape then malformed (what .. " has a model other than its shape") end
			local nonzero = magnitude (f.offset [1]) ~= 0 or magnitude (f.offset [3]) ~= 0
			if nonzero ~= (f.horizontal == 1) then malformed (what .. " horizontal disagrees with the offset's x and z") end
		end
		if f.horizontal == 0 then
			if f.heading or f.xz_applied ~= 0 or f.case ~= "no_horizontal_offset" then malformed (what .. " has no horizontal offset but a heading or a rotation") end
		else
			-- the legacy quirk, recorded and not corrected: the x and z offsets are added only for a NON-ZERO heading
			if not f.heading then malformed (what .. " has a horizontal offset but no heading was read") end
			if (magnitude (f.heading) ~= 0) ~= (f.xz_applied == 1) then malformed (what .. " heading and rotation disagree: x/z are added exactly when the heading is non-zero") end
			if f.case ~= (f.xz_applied == 1 and "rotated_horizontal_offset" or "heading_zero_quirk") then malformed (what .. " names the wrong case for its heading") end
		end
	elseif f.branch == 4 then
		if f.instance ~= 0 or f.horizontal ~= 0 or f.heading or f.xz_applied ~= 0 or f.offset then malformed (what .. " is the position fallback but carries offset values") end
		if f.fixed == 1 then
			if f.shape ~= 0 or f.case ~= "invalid_shape" or f.model ~= -1 then malformed (what .. " is a fixed object with the invalid shape but disagrees") end
		else
			if f.shape ~= -1 or f.case ~= "not_fixed" or f.model ~= -1 then malformed (what .. " is not a fixed object but carries a shape") end
		end
	end

	-- the host identity: read, never issued; a present identity runs to the end of the line and has exactly the declared length
	local declared = int (length, 0, HOST_ID_MAX, "host_id_len")
	if f.host_id_state == "present" then
		if declared < 1 or string.sub (rest, 1, 9) ~= " host_id=" or #rest - 9 ~= declared then malformed (what .. " host_id does not have its declared length") end
		f.host_id = string.sub (rest, 10)
	elseif f.host_id_state == "absent" or f.host_id_state == "overlong" then
		if declared ~= 0 or rest ~= "" then malformed (what .. " has no host identity but carries one") end
	else
		malformed (what .. " has the host_id_state '" .. tostring (f.host_id_state) .. "'")
	end
	return f
end

-- ---------------------------------------------------------------------------------------------------------------------------------
-- the run. The output is created here and is closed on every exit.

local out_file, why = host.create_exclusive (out_path)
if not out_file then error ("target-point-characterization: " .. tostring (why), 0) end
local out = json.wrap (out_file, out_path)
local function emit (row) out.emit (row) end

local function first_line (message) return (string.match (tostring (message), "^[^\n]*")) end

-- a protocol error (malformed evidence) is written, the run is closed as not complete, and it fails; any other error (an output failure) is passed on as it is
local function protocol_failure (err)
	if type (err) == "table" and err.malformed then
		emit { event = "malformed", reason = err.malformed }
		emit { event = "done", complete = false, malformed = true }
		error ("target-point-characterization: MALFORMED REPORT: " .. err.malformed .. " (the engine is not asked again)", 0)
	end
	error (err, 0)
end

-- the clock the engine reports: present, numeric and finite; the elapsed seconds never go backwards
local function check_clock (c, when)
	if type (c) ~= "table" then malformed ("the " .. when .. " clock is not a table") end
	for _, key in ipairs { "elapsed_seconds", "time_of_day_seconds", "day" } do
		local v = c [key]
		if type (v) ~= "number" or v ~= v or v == math.huge or v == -math.huge then malformed (string.format ("the %s clock's %s is not a finite number: %s", when, key, tostring (v))) end
	end
	return c
end

local function run ()
	local dc = require ("eech_dc")
	dc.prepare_installation (root)
	-- the reference host's arithmetic profile, selected right after the installation is prepared and before the boot (src/reference_arithmetic.rs)
	host.use_reference_arithmetic ()

	emit { event = "header", scenario = scenario_name, seed = 1, frames = frames, frame_ms = frame_ms, checkpoint_every = checkpoint_every, observer = "target-point-fixture",
		note = "observer only: no target selected, no entity created, the original function is called from nowhere but the campaign" }

	local ok, engine = pcall (dc.boot, {
		install_root = root, map = scenario.map, terrain = host.terrain_provider (root, scenario.map), environment = host.env_provider (), campaign_directory = "camp01", campaign = scenario.campaign, gunship = "apache",
		seed = 1, bind_mobile = host.bind_mobile, move_vehicle = host.move_vehicle, observe_loss = host.observe_loss,
		supply_reached = host.supply_reached, line_of_sight = host.line_of_sight, model = host.model, sense = host.sense, aim = host.aim,
	})
	if not ok then
		local message = first_line (engine)
		emit { event = "boot_failed", error = message }
		error ("target-point-characterization: boot failed: " .. message, 0)
	end
	emit { event = "boot", booted = true }

	-- the fixture entries are methods of the BOOTED engine, so a feature-off module is recognised here, before any question, and reported as NOT RUN: never a zero-success
	if type (engine.target_point_summary) ~= "function" or type (engine.target_point_offsets) ~= "function" or type (engine.target_point_event) ~= "function" or type (engine.clock) ~= "function" or type (engine.frame) ~= "function" then
		emit { event = "not_run", reason = "this module was not built with the target-point-fixture feature" }
		emit { event = "done", complete = false, malformed = false }
		error ("target-point-characterization: NOT RUN: this module was not built with the target-point-fixture feature", 0)
	end

	-- one question to the engine; a failure stops all questions (the engine may be poisoned)
	local function ask (what, call)
		local good, text = pcall (call)
		if not good then
			local message = first_line (text)
			emit { event = "report_failed", what = what, error = message }
			error ("target-point-characterization: " .. what .. " failed: " .. message .. " (the engine is not asked again)", 0)
		end
		if type (text) ~= "string" then
			emit { event = "report_failed", what = what, error = "the report is not a string" }
			error ("target-point-characterization: " .. what .. " returned a " .. type (text), 0)
		end
		return text
	end

	local function sample (when)
		local text = ask ("target_point_summary", function () return engine:target_point_summary () end)
		emit { event = "summary", when = when, text = text }
		local good, summary = pcall (check_summary, text)
		if not good then protocol_failure (summary) end
		return summary, text
	end

	local function read_clock (when)
		local raw = ask ("clock", function () return engine:clock () end)
		local good, clock = pcall (check_clock, raw, when)
		if not good then protocol_failure (clock) end
		emit { event = "clock", when = when, elapsed_seconds = clock.elapsed_seconds, time_of_day_seconds = clock.time_of_day_seconds, day = clock.day }
		return clock
	end

	-- THE CENSUS, straight after the boot: it is the boot's own product and must be whole before any frame is run
	local boot_summary = sample ("boot")
	local census_text = ask ("target_point_offsets", function () return engine:target_point_offsets () end)
	emit { event = "offsets", when = "boot", text = census_text }
	local checked, counts = pcall (check_offsets, census_text, boot_summary)
	if not checked then protocol_failure (counts) end
	local joined, reason = pcall (function ()
		local c = boot_summary.census
		if counts.sub_object ~= c.sub_object or counts.bbox ~= c.bbox or counts.invalid_zero ~= c.invalid_zero or counts.live_mismatch ~= c.live_mismatch or
				counts.none ~= c.expected - c.recorded then
			malformed ("the census rows and the summary's census totals disagree")
		end
	end)
	if not joined then protocol_failure (reason) end
	emit { event = "census", expected = boot_summary.census.expected, sub_object = counts.sub_object, bbox = counts.bbox, invalid_zero = counts.invalid_zero, none = counts.none, whole = boot_summary.census_whole == 1 }
	if boot_summary.census_whole ~= 1 then
		emit { event = "done", complete = false, malformed = false, census_complete = false }
		error ("target-point-characterization: NOT MEASURED: the census of the offset table is not whole (runs " .. boot_summary.census.runs .. ", recorded " .. boot_summary.census.recorded .. " of " .. boot_summary.census.expected .. ")", 0)
	end

	-- THE FRAMES. The campaign decides what is queried; nothing here selects, creates or calls anything.
	local clock_before = read_clock ("before")
	local previous = boot_summary
	local next_sample = checkpoint_every
	for f = 1, frames do
		local stepped, problem = pcall (function () engine:frame (frame_ms) end)
		if not stepped then
			local message = first_line (problem)
			emit { event = "frame_failed", frame = f, error = message }
			error ("target-point-characterization: frame " .. f .. " failed: " .. message .. " (the engine is not asked again)", 0)
		end
		if checkpoint_every > 0 and f < frames and f * frame_ms / 1000 >= next_sample then
			local now = sample ("frame " .. f)
			local monotone, bad = pcall (function ()
				if now.calls < previous.calls or now.events < previous.events or now.dropped < previous.dropped then malformed ("a query counter went backwards between two samples") end
				for _, key in ipairs { "runs", "ended", "expected", "recorded", "sub_object", "bbox", "invalid_zero" } do
					if now.census [key] ~= boot_summary.census [key] then malformed ("the census total '" .. key .. "' changed after the boot") end
				end
			end)
			if not monotone then protocol_failure (bad) end
			previous = now
			next_sample = next_sample + checkpoint_every
		end
	end
	local clock_after = read_clock ("after")
	local advanced, bad_clock = pcall (function ()
		if clock_after.elapsed_seconds <= clock_before.elapsed_seconds then malformed ("the clock did not advance over " .. frames .. " frames") end
	end)
	if not advanced then protocol_failure (bad_clock) end

	-- THE END: the summary, the census again (byte for byte), then every recorded query once
	local final = sample ("end")
	local monotone_end, bad_end = pcall (function ()
		if final.calls < previous.calls or final.events < previous.events or final.dropped < previous.dropped then malformed ("a query counter went backwards at the end") end
	end)
	if not monotone_end then protocol_failure (bad_end) end
	local census_again = ask ("target_point_offsets", function () return engine:target_point_offsets () end)
	emit { event = "offsets", when = "end", text = census_again }
	if census_again ~= census_text then protocol_failure ({ malformed = "the census at the end is not byte-equal to the census after the boot" }) end

	local events, seen = {}, -1
	for index = 0, final.events - 1 do
		local text = ask ("target_point_event " .. index, function () return engine:target_point_event (index) end)
		emit { event = "query", index = index, text = text }
		local good, event = pcall (check_event, text, index, final)
		if not good then protocol_failure (event) end
		if event.seq <= seen then protocol_failure ({ malformed = "event " .. index .. " does not follow the previous event's sequence number" }) end
		seen = event.seq
		events[index] = event
	end

	-- the rows must reproduce the summary's matrix when every finalised call was kept (nothing dropped or abandoned)
	local tally = { branch = { 0, 0, 0, 0 }, present = 0, absent = 0, changed = 0 }
	local per_site = {}
	for s_index = 0, SITE_COUNT - 1 do per_site [s_index] = { 0, 0, 0, 0 } end
	for index = 0, final.events - 1 do
		local e = events [index]
		if e.branch >= 1 then
			tally.branch [e.branch] = tally.branch [e.branch] + 1
			per_site [e.site] [e.branch] = per_site [e.site] [e.branch] + 1
		end
		if e.host_id_state == "present" then tally.present = tally.present + 1 elseif e.host_id_state == "absent" then tally.absent = tally.absent + 1 end
		if e.vp_changed == 1 then tally.changed = tally.changed + 1 end
	end
	local rows_agree, disagreement = pcall (function ()
		if final.dropped == 0 and final.interrupted == 0 then
			for b = 1, 4 do
				if tally.branch [b] ~= final.branch [b] then malformed ("the event rows hold " .. tally.branch [b] .. " calls of branch " .. b .. ", the summary " .. final.branch [b]) end
				for s_index = 0, SITE_COUNT - 1 do
					if per_site [s_index] [b] ~= final.site [s_index] [b] then malformed ("the event rows and the summary disagree on site " .. s_index .. " branch " .. b) end
				end
			end
			if tally.present ~= final.case.host_id_present or tally.absent ~= final.case.host_id_absent or tally.changed ~= final.case.vp_changed then
				malformed ("the event rows and the summary disagree on host identities or changed viewpoints")
			end
		else
			for b = 1, 4 do
				if tally.branch [b] > final.branch [b] then malformed ("the event rows hold more calls of branch " .. b .. " than the summary counts") end
			end
		end
	end)
	if not rows_agree then protocol_failure (disagreement) end

	local reached = {}
	for b = 1, 4 do if final.branch [b] > 0 then reached [#reached + 1] = tostring (b) end end
	local queries = final.calls - boot_summary.calls
	local whole = final.queries_whole == 1 and final.census_whole == 1
	emit { event = "done", complete = whole, census_complete = true, queries_whole = whole, queries = queries, boot_calls = boot_summary.calls, total_calls = final.calls, events = final.events,
		natural_queries_reached = final.calls > 0, branches_reached = #reached > 0 and table.concat (reached, ",") or "none", frames = frames, malformed = false }
	if final.calls == 0 then
		emit { event = "unreached", reason = "no query of the original target-point function was made in " .. frames .. " frames; no branch is covered" }
		io.stderr:write (string.format ("target-point-characterization: census whole; NO natural target-point query was reached in %d frames (nothing is covered)\n", frames))
	else
		io.stderr:write (string.format ("target-point-characterization: census whole; %d queries (%d at boot), branches reached: %s\n", final.calls, boot_summary.calls, #reached > 0 and table.concat (reached, ",") or "none"))
	end
	if not whole then
		error (string.format ("target-point-characterization: NOT COMPLETE: the query capture is not whole (dropped %d, stale %d, interrupted %d, orphans %d, unclassified %d, open %d, out_of_range %d)",
			final.dropped, final.stale, final.interrupted, final.orphans, final.unclassified, final.open, final.out_of_range), 0)
	end
end

local ran, err = pcall (run)
local closed, close_err = pcall (out.close)
if not ran then
	if not closed then err = tostring (err) .. "; additionally the output could not be closed: " .. first_line (close_err) end
	error (err, 0)
end
if not closed then error ("target-point-characterization: the output could not be closed: " .. first_line (close_err), 0) end
