-- Original target-point observer (EEDCS-52, D3). TEST-ONLY: it needs a module built with the `target-point-fixture` feature (the ORIGINAL get_local_entity_target_point of
-- en_maths.c, its callers weapon.c fill_aim_target and the weapon step's target-vector lookup, and the boot census of target_point_offsets[] made by wn_dbase.c, all observed by
-- csrc/eech_tp_fixture.c) and the reference host (EECH_WORLD). It is an OBSERVER: it boots the campaign exactly as lua/campaign.lua does (the same preparation, the reference
-- arithmetic profile, the same boot configuration, seed 1), runs the frames it is asked for and asks the campaign for nothing else. It selects no target, creates no entity, forces
-- no engagement and calls the original function from nowhere: the query calls it reports are only those the campaign's own update loop made.
--
-- WHAT IT REPORTS (all native text kept verbatim in the JSON Lines rows).
--   * the CENSUS of the offset table, one report per ORIGINAL initialiser PASS (generation). One public boot has been seen to run the initialiser twice (full_initialise_game, then
--     reinitialise_entity_system), each pass first running calculate_seeker_limits_from_launch_angle once; no pass count is assumed here, every pass the observer kept is read and
--     retained in full, right after the boot and again at the end: per model index whether the offset came from the model's TARGET_POINT sub-object, from the bounding-box fallback (with
--     the six bounds the original used) or is the invalid index's zero, and the float bits. Each pass must be a complete SNAPSHOT (it ended, recorded every index once, nothing strayed
--     or was lost) and the LATEST pass must be the one the live table holds; an older pass may differ from the table, which is a fact about that pass. Nothing is constructed or
--     measured again.
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
-- THE CONTROLLED PROBE (probe=controlled; absent by default). The natural runs reach only branch 1 (an instance with a TARGET_POINT sub-object). With this argument the observer's probe runs ONCE,
-- after the boot census is whole and before any frame: it finds live entities of the families a target can be for which the ORIGINAL takes branch 2, branch 3 (no horizontal offset, a rotated
-- offset, and the zero-heading quirk) and branch 4 (not fixed, invalid shape), using only reads the original makes, and calls the ORIGINAL get_local_entity_target_point on at most two of each
-- as the controlled caller. This is NOT a campaign target selection and NOT aim or weapon coverage; branches 2 to 4 write nothing (branch 1, the only writer of the instance viewpoint, is not
-- probed), so no state is set or restored. Each controlled query is checked to have taken exactly the case it was picked for, its identity and words are the original's, and the controlled queries
-- are counted apart from the natural ones (`controlled_*` in the done row; `queries`, `branches_reached` and `natural_queries_reached` are the natural run's). A case with no candidate is
-- UNAVAILABLE (`controlled_unavailable`), never a success.
--
-- THE SETUP PROBE (probe=setup; a separate opt-in mode, exclusive with probe=controlled). The read-only probe finds NO live candidate for an instance without a TARGET_POINT, a mobile with no instance
-- and a fixed object with the invalid shape, so this mode gives each a real, explicitly declared physical PRECONDITION on a real live entity (a real aircraft or vehicle's instance replaced by a
-- temporarily constructed instance of a real census bounding-box scene, then NULL; a real fixed structure's shape set to the invalid index), calls the ORIGINAL once and restores it, all before any
-- frame, with the change, the original's answer and the restoration verified inside the engine (a failed restoration is a refusal that stops the run here). Each case is reported as applied/called/
-- restored by semantic identity (families, model and shape indices and names) with the real instance's viewpoint and the construction counters checked; a case with no real candidate is NOT called and
-- says why. Its queries are caller class setup_probe (counted apart, never natural, never aim or weapon coverage, never host authority).
-- Restoration is claimed only on a NORMAL return of the original: a fatal unwind has no guaranteed cleanup and no next frame. Cases 1 and 2 may use different mobiles, and fewer than three entities
-- may be involved. The report's models are joined to the census by index and name, and case 1's scene to the pass that applied when its query ran.
--
-- host.args: root (installation root, required), out (the JSON Lines file, required; created exclusively), probe (controlled, setup, or absent), scenario (lebanon_retail, the default, or georgia_retail),
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

local known = { root = true, out = true, scenario = true, seconds = true, frame_ms = true, checkpoint_every = true, probe = true }
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
-- probe=controlled asks for the CONTROLLED probe: once, after the boot and before any frame, the original is called on real entities picked by the fixture for the cases the natural run does not
-- reach. Those queries are the controlled caller class, are not campaign target selection, and are kept apart from the natural ones in every count below.
-- probe=setup is a SEPARATE, opt-in mode: for the three cases the read-only probe finds no candidate it gives a real live entity an explicit physical precondition, calls the original once,
-- and restores and verifies the precondition before any frame (see the setup section below). It is exclusive with probe=controlled.
if args.probe ~= nil and args.probe ~= "controlled" and args.probe ~= "setup" then fail_args ("probe=controlled|setup (or absent)") end
local probe_requested = args.probe == "controlled"
local setup_requested = args.probe == "setup"

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
local GENERATIONS_CAPACITY = 4
-- the branch-1 chain: nodes the observer keeps, and the original's own walk limit (objects [256] in get_3d_sub_object_world_viewpoint)
local CHAIN_KEPT = 8
local CHAIN_WALK_MAX = 256
-- the CRT-stream marks the observer keeps, and the kinds it writes
local RNG_MARK_CAPACITY = 32
local RNG_KINDS = { seed = true, pass_begin = true, pass_end = true, exposed = true }
local CHAIN_STATES = { complete = true, empty = true, too_deep = true, unterminated = true, wrong_root = true, null_result = true }
local INT_MAX = 2147483647
-- the observer's RNG totals are unsigned: a total that latched its overflow stops at the limit and is reported, not wrapped
local UINT_MAX = 4294967295

local SITES = { [0] = "unattributed", "aim_vectors", "aim_lead", "aim_intercept", "weapon_step_target_vec", "other", "controlled_probe", "setup_probe" }
local SITE_COUNT = 8
local CONTROLLED_SITE = 6
local SETUP_SITE = 7
-- the setup probe's cases (csrc/eech_tp_fixture.c setup_case_name): the event's detail, and the branch and case the original must be seen to take for it
local SETUP_NAMES = { "instance_without_target_point", "not_fixed_no_instance", "fixed_invalid_shape" }
local SETUP_CASE = { { 2, "instance_without_target_point" }, { 4, "not_fixed" }, { 4, "invalid_shape" } }
-- the controlled probe's cases (csrc/eech_tp_fixture.c probe_bucket): the event's detail, the branch and case the original must be seen to take for it
local PROBE_NAMES = { "instance_without_target_point", "fixed_no_horizontal", "fixed_rotated", "fixed_heading_zero", "not_fixed_no_instance", "fixed_invalid_shape" }
local PROBE_CASE = { { 2, "instance_without_target_point" }, { 3, "no_horizontal_offset" }, { 3, "rotated_horizontal_offset" }, { 3, "heading_zero_quirk" }, { 4, "not_fixed" }, { 4, "invalid_shape" } }
local PROBE_PICKS = 2
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

	local k = take ("tp-census", { "expected", "generations", "capacity", "dropped", "open", "interrupted", "stray", "last" })
	m.census = {
		expected = int (k.expected, 1, INT_MAX, "census expected"), generations = int (k.generations, 0, GENERATIONS_CAPACITY, "census generations"),
		dropped = int (k.dropped, 0, INT_MAX, "census dropped"), open = flag (k.open, "census open"), interrupted = int (k.interrupted, 0, INT_MAX, "census interrupted"),
		stray = int (k.stray, 0, INT_MAX, "census stray"), last = int (k.last, -1, GENERATIONS_CAPACITY - 1, "census last"),
	}
	local census = m.census
	if int (k.capacity, 0, INT_MAX, "census capacity") ~= GENERATIONS_CAPACITY then malformed ("the census capacity is not the observer's " .. GENERATIONS_CAPACITY) end

	-- every kept initialiser pass has its own line, in ordinal order: the passes are separate original products, none is merged or overwritten
	m.passes = {}
	local all_complete, latest_ended = true, -1
	for g = 0, census.generations - 1 do
		local p = take ("tp-pass", { "ordinal", "ended", "interrupted", "recorded", "sub_object", "bbox", "invalid_zero", "duplicates", "out_of_range", "live_mismatch", "complete", "rng_draws", "skinned" })
		if int (p.ordinal, 0, GENERATIONS_CAPACITY - 1, "pass ordinal") ~= g then malformed ("the pass rows are not in order at " .. g) end
		local pass = {
			ended = flag (p.ended, "pass ended"), interrupted = flag (p.interrupted, "pass interrupted"), recorded = int (p.recorded, 0, INT_MAX, "pass recorded"),
			sub_object = int (p.sub_object, 0, INT_MAX, "pass sub_object"), bbox = int (p.bbox, 0, INT_MAX, "pass bbox"), invalid_zero = int (p.invalid_zero, 0, INT_MAX, "pass invalid_zero"),
			duplicates = int (p.duplicates, 0, INT_MAX, "pass duplicates"), out_of_range = int (p.out_of_range, 0, INT_MAX, "pass out_of_range"),
			live_mismatch = int (p.live_mismatch, 0, INT_MAX, "pass live_mismatch"), complete = flag (p.complete, "pass complete"),
			rng_draws = int (p.rng_draws, 0, UINT_MAX, "pass rng_draws"), skinned = int (p.skinned, 0, INT_MAX, "pass skinned"),
		}
		-- the skin prediction counts scenes of the table; a pass that never ended has no draws of its own
		if pass.skinned > census.expected - 1 then malformed ("pass " .. g .. " predicts more skinned scenes than the table has beyond the invalid scene 0") end
		if pass.ended == 0 and (pass.rng_draws ~= 0 or pass.skinned ~= 0) then malformed ("pass " .. g .. " never ended but carries draws") end
		if pass.sub_object + pass.bbox + pass.invalid_zero ~= pass.recorded or pass.recorded > census.expected or pass.invalid_zero > 1 or pass.live_mismatch > pass.recorded then
			malformed ("the totals of pass " .. g .. " are incoherent")
		end
		-- a pass is a complete SNAPSHOT if it ended, was not abandoned and recorded every index once inside the table; whether it still equals the current table is live_mismatch, a separate fact
		local snapshot = pass.ended == 1 and pass.interrupted == 0 and pass.recorded == census.expected and pass.duplicates == 0 and pass.out_of_range == 0 and pass.invalid_zero == 1
		if (pass.complete == 1) ~= snapshot then malformed ("pass " .. g .. " claims complete=" .. pass.complete .. " against its own counters") end
		if pass.ended == 1 then latest_ended = g end
		if pass.complete == 0 then all_complete = false end
		m.passes [g] = pass
	end
	if census.last ~= latest_ended then malformed ("the census names pass " .. census.last .. " as the latest ended, the pass rows say " .. latest_ended) end

	-- The CRT stream: the totals, then the ordered marks. What it can and cannot support (the decision rule): a history is COMPLETE only if no mark was dropped and no counter latched an overflow;
	-- a partial or overflowed history is checked as far as it goes and withholds every conclusion about the stream. Even a complete history is evidence only for the captured window: a pass
	-- still pending when the capture ends leaves later exposure open, and a recorded seed before any consumer ran is what erases an earlier advancement.
	local r = take ("tp-rng", { "draws", "seeds", "pass_draws", "outside_draws", "exposed", "marks", "mark_capacity", "marks_dropped", "overflow" })
	m.rng = {
		draws = int (r.draws, 0, UINT_MAX, "rng draws"), seeds = int (r.seeds, 0, UINT_MAX, "rng seeds"), pass_draws = int (r.pass_draws, 0, UINT_MAX, "rng pass_draws"),
		outside = int (r.outside_draws, 0, UINT_MAX, "rng outside_draws"), exposed = int (r.exposed, 0, UINT_MAX, "rng exposed"), count = int (r.marks, 0, RNG_MARK_CAPACITY, "rng marks"),
		dropped = int (r.marks_dropped, 0, UINT_MAX, "rng marks_dropped"), overflow = flag (r.overflow, "rng overflow"), marks = {}, lines = {},
	}
	local rng = m.rng
	if int (r.mark_capacity, 0, INT_MAX, "rng mark_capacity") ~= RNG_MARK_CAPACITY then malformed ("the mark capacity is not the observer's " .. RNG_MARK_CAPACITY) end
	if rng.dropped > 0 and rng.count ~= RNG_MARK_CAPACITY then malformed ("marks were dropped while the mark list was not full") end
	-- a latched overflow means a total stopped at its limit: no relation between the totals can be believed from there, and none is required
	local counted = rng.overflow == 0
	if counted and rng.draws ~= rng.pass_draws + rng.outside then malformed (string.format ("rng draws %d are not pass draws %d + outside draws %d", rng.draws, rng.pass_draws, rng.outside)) end
	-- the draws of the kept passes are part of the draws made inside passes, and are all of them when no pass was lost, abandoned or left open
	local kept_draws = 0
	for g = 0, census.generations - 1 do kept_draws = kept_draws + m.passes [g].rng_draws end
	if counted and kept_draws > rng.pass_draws then malformed ("the passes report more draws than were made inside passes") end
	if counted and census.dropped == 0 and census.interrupted == 0 and census.open == 0 and kept_draws ~= rng.pass_draws then malformed ("the passes' draws are not the draws made inside passes") end
	local seed_marks, exposed_marks = 0, 0
	local next_gen, dropped_begins, interrupted_begins, bracket_total, abandoned_total, early_dropped = 0, 0, 0, 0, 0, false
	local ended_gen, interrupted_gen = {}, {}
	local open_begin, after_pass, previous_draws, last_seed_draws = nil, nil, 0, 0
	for i = 0, rng.count - 1 do
		at = at + 1
		if not list [at] then malformed ("the summary ends before rng mark " .. i) end
		local k = fields (list [at], "tp-rng-mark", { "seq", "kind", "gen", "value", "draws", "since_seed" }, "rng mark " .. i)
		if int (k.seq, 0, RNG_MARK_CAPACITY, "rng mark seq") ~= i then malformed ("the rng marks are not in order at " .. i) end
		if not RNG_KINDS [k.kind] then malformed ("rng mark " .. i .. " has the kind '" .. tostring (k.kind) .. "'") end
		local mark = { kind = k.kind, gen = int (k.gen, -1, GENERATIONS_CAPACITY - 1, "rng mark gen"), draws = int (k.draws, 0, UINT_MAX, "rng mark draws"), since = int (k.since_seed, 0, UINT_MAX, "rng mark since_seed") }
		if k.kind == "seed" then word (k.value, "rng seed value") elseif k.value ~= "-" then malformed ("rng mark " .. i .. " is not a seed but carries a value") end
		if k.kind == "seed" and mark.gen ~= -1 then malformed ("rng seed mark " .. i .. " names a pass") end
		if mark.draws < previous_draws or mark.draws > rng.draws then malformed ("rng mark " .. i .. " has draws that are not ordered within the totals") end
		previous_draws = mark.draws
		if counted then
			-- since_seed is exactly the draws since the preceding seed's total (the seed mark itself records the value BEFORE it resets)
			if mark.since ~= mark.draws - last_seed_draws then
				malformed (string.format ("rng mark %d has since_seed %d, the draws since the preceding seed are %d", i, mark.since, mark.draws - last_seed_draws))
			end
		end
		if k.kind == "seed" then
			seed_marks = seed_marks + 1
			-- a pass end followed by a seed with draws in between must have shown an exposed mark first
			if after_pass and mark.draws ~= after_pass.draws then malformed ("rng seed mark " .. i .. " follows a pass end after outside draws that have no exposed mark") end
			after_pass = nil
			last_seed_draws = mark.draws
		elseif k.kind == "pass_begin" then
			-- one begin per kept pass, in the order the census kept them; a pass beyond the history is the one identity -1
			if mark.gen == -1 then
				dropped_begins = dropped_begins + 1
				if next_gen < GENERATIONS_CAPACITY then early_dropped = true end
			else
				if mark.gen ~= next_gen then malformed ("rng pass_begin mark " .. i .. " names pass " .. mark.gen .. ", the next kept pass is " .. next_gen) end
				next_gen = next_gen + 1
			end
			if open_begin then
				interrupted_begins = interrupted_begins + 1
				abandoned_total = abandoned_total + (mark.draws - open_begin.draws)
				if open_begin.gen >= 0 then interrupted_gen [open_begin.gen] = true end
			end
			if after_pass and mark.draws ~= after_pass.draws then malformed ("rng pass_begin mark " .. i .. " follows a pass end after outside draws that have no exposed mark") end
			open_begin = mark
			after_pass = nil
		elseif k.kind == "pass_end" then
			if not open_begin or open_begin.gen ~= mark.gen then malformed ("rng pass_end mark " .. i .. " does not close the pass that began") end
			bracket_total = bracket_total + (mark.draws - open_begin.draws)
			if mark.gen >= 0 then
				if ended_gen [mark.gen] then malformed ("rng pass " .. mark.gen .. " ended twice") end
				ended_gen [mark.gen] = true
				-- a kept pass that ended: the draws between its two marks are the draws its own line reports
				if m.passes [mark.gen] and m.passes [mark.gen].ended == 1 and counted and mark.draws - open_begin.draws ~= m.passes [mark.gen].rng_draws then
					malformed ("the marks of pass " .. mark.gen .. " bracket " .. (mark.draws - open_begin.draws) .. " draws, its line says " .. m.passes [mark.gen].rng_draws)
				end
			end
			open_begin = nil
			after_pass = mark
		else
			exposed_marks = exposed_marks + 1
			-- the FIRST draw outside any pass after a pass end with no seed between: it follows that end at exactly one more draw
			if after_pass == nil or after_pass.gen ~= mark.gen or mark.draws ~= after_pass.draws + 1 then
				malformed ("rng exposed mark " .. i .. " is not the first outside draw after a pass end (draws pass_end + 1, no seed between)")
			end
			after_pass = nil
		end
		rng.marks [i] = mark
		rng.lines [i] = list [at]
	end
	rng.pending = (after_pass ~= nil) or (open_begin ~= nil)
	if rng.dropped == 0 and counted then
		-- an untruncated history: every total is its marks, every kept pass has exactly one ordered begin and, if it ended, one end, and the passes' flags are the marks' facts
		rng.history = "complete"
		if seed_marks ~= rng.seeds or exposed_marks ~= rng.exposed then malformed ("the rng seed or exposed marks are not the totals") end
		if next_gen ~= census.generations then malformed ("the rng marks begin " .. next_gen .. " kept passes, the census kept " .. census.generations) end
		if early_dropped then malformed ("a pass beyond the history began while kept generations remained") end
		if dropped_begins ~= census.dropped then malformed ("the rng marks begin " .. dropped_begins .. " passes beyond the history, the census counts " .. census.dropped) end
		if interrupted_begins ~= census.interrupted then malformed ("the rng marks abandon " .. interrupted_begins .. " passes, the census counts " .. census.interrupted) end
		for g = 0, census.generations - 1 do
			if (m.passes [g].ended == 1) ~= (ended_gen [g] == true) then malformed ("pass " .. g .. " ended=" .. m.passes [g].ended .. " disagrees with its rng end mark") end
			if (m.passes [g].interrupted == 1) ~= (interrupted_gen [g] == true) then malformed ("pass " .. g .. " interrupted=" .. m.passes [g].interrupted .. " disagrees with its rng marks") end
		end
		if (open_begin ~= nil) ~= (census.open == 1) then malformed ("the rng marks leave a pass open exactly when the census does not say so") end
		-- no draw outside a pass follows the last pass end unrecorded, and the draws inside passes are the brackets of the ended passes plus the open pass's so far
		if after_pass and rng.draws ~= after_pass.draws then malformed ("outside draws follow the last pass end and no exposed mark records the first") end
		if bracket_total + abandoned_total + (open_begin and (rng.draws - open_begin.draws) or 0) ~= rng.pass_draws then malformed ("the draws inside passes are not the brackets of the pass marks") end
		-- a pass abandoned by the next begin drew inside a pass that no pass line reports: the history is explicitly INCOMPLETE (and the census is not whole), never evidence about the stream
		if interrupted_begins > 0 then rng.history = "incomplete" end
	elseif counted then
		rng.history = "partial"
		if seed_marks > rng.seeds or exposed_marks > rng.exposed then malformed ("the rng marks held are more than the totals") end
	else
		rng.history = "overflowed"
	end
	local v = take ("tp-complete", { "census", "queries_whole" })
	m.census_whole = flag (v.census, "census whole")
	m.queries_whole = flag (v.queries_whole, "queries_whole")
	if list [at + 1] then malformed ("the summary has extra lines after its last expected one: " .. list [at + 1]) end

	-- a verdict is only believed if the counters say the same, both ways. The census is whole if at least one pass ran, none was lost, abandoned or left open, nothing strayed, every pass is a complete
	-- snapshot and the LATEST pass is the one the table holds now (an older pass may differ from the table: that is its own fact, not a lost capture)
	local census_clean = census.generations >= 1 and census.dropped == 0 and census.open == 0 and census.interrupted == 0 and census.stray == 0 and all_complete and
		census.last == census.generations - 1 and m.passes [census.last].live_mismatch == 0
	if (m.census_whole == 1) ~= census_clean then malformed ("tp-complete census=" .. m.census_whole .. " contradicts the census counters") end
	local queries_clean = m.dropped == 0 and m.stale == 0 and m.interrupted == 0 and m.orphans == 0 and m.unclassified == 0 and m.open == 0 and m.out_of_range == 0 and m.overlong == 0
	if (m.queries_whole == 1) ~= queries_clean then malformed ("tp-complete queries_whole=" .. m.queries_whole .. " contradicts the query counters") end
	return m
end

-- The census rows of ONE pass (generation `generation`): one per model index, in order. Returns the counts they hold, joined against that pass's summary line by the caller.
local function check_offsets (text, summary, generation)
	local list = lines_of (text, "the census of pass " .. generation)
	local expected = summary.census.expected
	if #list ~= expected then malformed (string.format ("the census of pass %d has %d rows, the summary expects %d", generation, #list, expected)) end
	local counts = { none = 0, sub_object = 0, bbox = 0, invalid_zero = 0, live_mismatch = 0 }
	-- the rows themselves, by model index, for the joins of the queries that name a shape: its state, name, offset words (as the native text) and whether the live table still held them
	local rows = {}
	for i = 0, expected - 1 do
		local what = "census row " .. i .. " of pass " .. generation
		local r = fields (list [i + 1], "tp-offset", { "generation", "index", "name", "state", "offset", "bounds", "live" }, what)
		if int (r.generation, 0, INT_MAX, "census generation") ~= generation then malformed (what .. " belongs to another pass") end
		if int (r.index, 0, INT_MAX, "census index") ~= i then malformed (what .. " is out of order") end
		name (r.name, what .. " name")
		if not STATES [r.state] then malformed (what .. " has the state '" .. tostring (r.state) .. "'") end
		counts [r.state] = counts [r.state] + 1
		rows [i] = { state = r.state, name = r.name, offset = r.offset, live = r.live }
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
	return counts, rows
end

-- The census facts a pass's summary line and its rows must agree on (every total, and the cohort that was not recorded).
local function join_census_rows (counts, pass, expected)
	if counts.sub_object ~= pass.sub_object or counts.bbox ~= pass.bbox or counts.invalid_zero ~= pass.invalid_zero or counts.live_mismatch ~= pass.live_mismatch or counts.none ~= expected - pass.recorded then
		malformed ("the census rows and the summary's totals of the pass disagree")
	end
end

-- Every pass is an original product of the boot and nothing may change it afterwards: each pass the boot summary holds appears in every later summary with the same snapshot facts. A LATER pass
-- is a new original product and is allowed (it is judged by its own line and by the final verdict). While no pass was added nothing wrote the table, so even the live facts and the verdict must
-- be those of the boot; once one was added an older pass may legitimately differ from the table. (The QUERY counters grow; these do not.)
local PASS_FIXED = { "ended", "interrupted", "recorded", "sub_object", "bbox", "invalid_zero", "duplicates", "out_of_range", "complete", "rng_draws", "skinned" }
-- The CRT stream only moves forward between any two samples, and the driver holds it to that between every CONSECUTIVE pair as well as against the boot: its counters never fall, a
-- latched overflow never clears, no mark disappears or changes, and a mark appended after the earlier sample is not older than that sample's draw total (so the draws it had already
-- seen cannot be reassigned backwards). `from` names the earlier sample in the message. This is the stream only: the census passes the boot held are judged separately below.
local function rng_continuity (earlier, later, when, from)
	for _, key in ipairs { "draws", "seeds", "pass_draws", "outside", "exposed", "dropped" } do
		if later.rng [key] < earlier.rng [key] then malformed ("the CRT counter '" .. key .. "' fell after " .. from .. " (" .. when .. ")") end
	end
	if earlier.rng.overflow == 1 and later.rng.overflow ~= 1 then malformed ("the CRT overflow latch cleared after " .. from .. " (" .. when .. ")") end
	if later.rng.count < earlier.rng.count then malformed ("rng marks disappeared after " .. from .. " (" .. when .. ")") end
	for i = earlier.rng.count, later.rng.count - 1 do
		if later.rng.marks [i].draws < earlier.rng.draws then malformed ("rng mark " .. i .. " was appended after the sample yet is older than its " .. earlier.rng.draws .. " draws (" .. when .. ")") end
	end
	for i = 0, earlier.rng.count - 1 do
		if later.rng.lines [i] ~= earlier.rng.lines [i] then malformed ("rng mark " .. i .. " changed after " .. from .. " (" .. when .. ")") end
	end
end

local function same_census (boot, later, when)
	rng_continuity (boot, later, when, "the boot")
	local b, l = boot.census, later.census
	if l.expected ~= b.expected then malformed ("the census size changed after the boot (" .. when .. ")") end
	if l.generations < b.generations then malformed ("the census lost a pass after the boot (" .. when .. ")") end
	for g = 0, b.generations - 1 do
		for _, key in ipairs (PASS_FIXED) do
			if later.passes [g] [key] ~= boot.passes [g] [key] then malformed ("the '" .. key .. "' of pass " .. g .. " changed after the boot (" .. when .. ")") end
		end
		if l.generations == b.generations and later.passes [g].live_mismatch ~= boot.passes [g].live_mismatch then malformed ("the live facts of pass " .. g .. " changed with no pass added (" .. when .. ")") end
	end
	if l.generations == b.generations then
		for _, key in ipairs { "dropped", "open", "interrupted", "stray", "last" } do
			if l [key] ~= b [key] then malformed ("the census total '" .. key .. "' changed with no pass added (" .. when .. ")") end
		end
		if later.census_whole ~= boot.census_whole then malformed ("the census verdict changed with no pass added (" .. when .. ")") end
	end
end

-- The mode values of the host aim operation that fill_aim_target reports as its detail (CAMPAIGN_AIM_VECTORS, _LEAD and _INTERCEPT of aphavoc/source/entity/mobile/mb_hostaim.h), by the
-- caller class the observer's site_class gives them: tests/target_point_characterization.rs checks these numbers against that header.
local AIM_DETAIL = { [1] = 6, [2] = 7, [3] = 9 }

-- One recorded query. `index` is the number asked for. `rows` are the census rows by pass (rows [generation] [model index]). Returns its facts.
local function check_event (text, index, summary, rows)
	local what = "event " .. index
	local list = lines_of (text, what)
	if #list ~= 1 then malformed (what .. " is not exactly one line") end
	local head, length, rest = string.match (list [1], "^(.-) host_id_len=(%d+)(.*)$")
	if not head then malformed (what .. " has no host_id_len") end
	local e = fields (head, "tp-event", { "index", "seq", "gen", "gen_open", "site", "site_name", "detail", "branch", "branch_name", "case", "entity_type", "entity_type_name", "instance", "model", "model_name",
		"fixed", "shape", "horizontal", "heading", "xz_applied", "offset", "result", "vp_before", "vp_after", "base", "main_scale", "chain_state", "chain_depth", "chain", "vp_changed", "host_id_state" }, what)
	if int (e.index, 0, INT_MAX, "event index") ~= index then malformed (what .. " carries another event's index") end
	local f = {
		seq = int (e.seq, 0, INT_MAX, "seq"), gen = int (e.gen, -1, GENERATIONS_CAPACITY - 1, "gen"), gen_open = flag (e.gen_open, "gen_open"), site = int (e.site, 0, SITE_COUNT - 1, "site"), detail = int (e.detail, -INT_MAX, INT_MAX, "detail"), branch = int (e.branch, 0, 4, "branch"),
		entity_type = int (e.entity_type, 0, INT_MAX, "entity_type"), instance = flag (e.instance, "instance"), model = int (e.model, -1, INT_MAX, "model"), fixed = flag (e.fixed, "fixed"),
		shape = int (e.shape, -INT_MAX, INT_MAX, "shape"), horizontal = flag (e.horizontal, "horizontal"), xz_applied = flag (e.xz_applied, "xz_applied"), vp_changed = flag (e.vp_changed, "vp_changed"),
		case = e.case, host_id_state = e.host_id_state, entity_type_name = e.entity_type_name, model_name = e.model_name,
	}
	if f.seq >= summary.calls then malformed (what .. " has a sequence number beyond the calls the summary counts") end
	-- the census pass the query ran after: one of the kept passes, and one that ended
	if f.gen >= summary.census.generations or (f.gen >= 0 and summary.passes [f.gen].ended ~= 1) then malformed (what .. " names a census pass that the summary does not hold as ended") end
	if e.site_name ~= SITES [f.site] then malformed (what .. " names site " .. f.site .. " '" .. e.site_name .. "'") end
	if f.site == 0 and f.detail ~= 0 then malformed (what .. " is unattributed but carries a caller detail") end
	if f.site == 4 and f.detail ~= 1 then malformed (what .. " is a weapon step lookup of a question other than the target point") end
	-- a controlled query names its case in its detail, and the original must be seen to take exactly that case
	if f.site == CONTROLLED_SITE then
		local wanted = PROBE_CASE [f.detail]
		if not wanted then malformed (what .. " is a controlled query of the case " .. f.detail .. ", which the probe does not have") end
		if f.branch ~= wanted [1] or f.case ~= wanted [2] then malformed (string.format ("%s was picked as %s but the original took branch %d case '%s'", what, PROBE_NAMES [f.detail], f.branch, tostring (f.case))) end
	end
	if f.site == SETUP_SITE then
		local wanted = SETUP_CASE [f.detail]
		if not wanted then malformed (what .. " is a setup query of the case " .. f.detail .. ", which the setup does not have") end
		if f.branch ~= wanted [1] or f.case ~= wanted [2] then malformed (string.format ("%s was set up as %s but the original took branch %d case '%s'", what, SETUP_NAMES [f.detail], f.branch, tostring (f.case))) end
	end
	-- the observer's site_class gives an aim caller its class from the aim mode, and only from it
	if AIM_DETAIL [f.site] and f.detail ~= AIM_DETAIL [f.site] then malformed (string.format ("%s is aim caller class %d with the detail %d, not the aim mode %d", what, f.site, f.detail, AIM_DETAIL [f.site])) end
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

	-- the original inputs of the physical query. The BASE position is the entity position the original's own get_local_entity_vec3d copied (branches 2, 3 and 4), read before any physical
	-- addition; branch 1 carries its position and attitude in vp_after instead
	if f.branch == 2 or f.branch == 3 or f.branch == 4 then
		f.base = words (e.base, 3, what .. " base")
	elseif e.base ~= "-" then
		malformed (what .. " carries a base position but its branch did not copy the entity position")
	end
	if f.branch == 2 or f.branch == 4 then
		-- the original answers the entity position unchanged
		if e.result ~= e.base then malformed (what .. " answered the entity position but its result is not the base position") end
	end
	-- branch 1 only: the instance's relative scale and the placement chain the original is about to walk. Anything but a complete chain says why and carries no nodes
	if f.branch == 1 then
		words (e.main_scale, 3, what .. " main_scale")
		if not CHAIN_STATES [e.chain_state] then malformed (what .. " has the chain state '" .. tostring (e.chain_state) .. "'") end
		local depth = int (e.chain_depth, 0, CHAIN_WALK_MAX, "chain_depth")
		if e.chain_state == "complete" then
			if depth < 1 or depth > CHAIN_KEPT then malformed (what .. " has a complete chain of " .. depth .. " nodes, the observer keeps 1 to " .. CHAIN_KEPT) end
			local nodes = 0
			for node in string.gmatch (e.chain .. ";", "([^;]*);") do
				nodes = nodes + 1
				words (node, 9, what .. " chain node " .. nodes)
			end
			if nodes ~= depth then malformed (what .. " declares a chain of " .. depth .. " nodes and holds " .. nodes) end
		else
			if e.chain ~= "-" then malformed (what .. " has the chain state '" .. e.chain_state .. "' but carries nodes") end
			if (e.chain_state == "empty" or e.chain_state == "null_result") and depth ~= 0 then malformed (what .. " has an empty chain with a depth") end
			if e.chain_state == "too_deep" and depth <= CHAIN_KEPT then malformed (what .. " is too deep with only " .. depth .. " nodes") end
			if e.chain_state == "unterminated" and depth ~= CHAIN_WALK_MAX then malformed (what .. " did not end but stopped at " .. depth) end
		end
		f.chain_state = e.chain_state
	else
		if e.main_scale ~= "-" or e.chain_state ~= "-" or e.chain_depth ~= "0" or e.chain ~= "-" then malformed (what .. " is not branch 1 but carries a scale or a chain") end
	end

	-- the heading and the offset are known only on the paths that read them
	if e.heading ~= "-" then f.heading = word (e.heading, what .. " heading") end
	if e.offset ~= "-" then f.offset = words (e.offset, 3, what .. " offset") end
	-- branch 3 added no rotated x/z offset (no horizontal offset, or the heading-zero quirk): x and z are the base words, and only y was changed. Applied only where the offset was read
	if f.branch == 3 and f.offset and f.xz_applied == 0 then
		local answer = words (e.result, 3, what .. " result")
		if answer [1] ~= f.base [1] or answer [3] ~= f.base [3] then malformed (what .. " added no rotated x/z offset but its x or z is not the base position's") end
	end
	if f.branch == 1 or f.branch == 2 then
		if f.instance ~= 1 or f.fixed ~= 0 or f.shape ~= -1 or f.horizontal ~= 0 or f.heading or f.xz_applied ~= 0 or f.offset then malformed (what .. " is an instance branch but carries fixed-object values") end
		if f.branch == 2 and f.vp_changed ~= 0 then malformed (what .. " answered the entity position but its instance viewpoint changed") end
	elseif f.branch == 3 then
		if f.instance ~= 0 or f.fixed ~= 1 or f.shape == 0 then malformed (what .. " is a fixed-shape branch without a fixed, instance-less, non-invalid shape") end
		-- the offset is read only for a shape inside the table; an outside shape is the observer's out_of_range count and not a measurement
		if (f.offset ~= nil) ~= (f.shape >= 1 and f.shape < summary.census.expected) then malformed (what .. " has an offset that does not agree with its shape being inside the table") end
		if f.offset then
			if f.model ~= f.shape then malformed (what .. " has a model other than its shape") end
			-- the offset the original read is the row of that shape in the census pass that was the latest to end when the query ran, word for word (the table is written only by the
			-- initialiser; a query never joins a pass that came later, and none applies to a query made before any pass ended or while one ran)
			if f.gen < 0 or f.gen_open == 1 then malformed (what .. " read the offset table but no finished census pass applies to it") end
			local row = (rows [f.gen] or {}) [f.shape]
			if not row or (row.state ~= "sub_object" and row.state ~= "bbox") then malformed (what .. " names a shape that census pass " .. f.gen .. " did not record from a model") end
			-- the latest pass is the one the table holds now; an older pass is a snapshot and need not be
			if f.gen == summary.census.last and row.live ~= "1" then malformed (what .. " names a shape whose live table entry is no longer the latest census row") end
			if e.offset ~= row.offset then malformed (what .. " has an offset that is not its shape's census row") end
			if e.model_name ~= row.name then malformed (what .. " names a model other than its shape's census row") end
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

-- The controlled probe's report: the entities examined, then one line per case with its candidates and picks. A case with no candidate is UNAVAILABLE, said so and never succeeded.
local function check_probe (text)
	local list = lines_of (text, "the probe report")
	if #list ~= #PROBE_NAMES + 1 then malformed ("the probe report has " .. #list .. " lines, expected " .. (#PROBE_NAMES + 1)) end
	local head = fields (list [1], "tp-probe", { "status", "visited", "entities", "shape_out_of_range" }, "the probe head")
	-- only a COMPLETE scan reports; a refused one raises instead (tp-probe-refused), calls nothing and is never a list of unavailable cases
	if head.status ~= "complete" then malformed ("the probe's scan is not complete: " .. tostring (head.status)) end
	local report = { visited = int (head.visited, 0, INT_MAX, "probe visited"), entities = int (head.entities, 0, INT_MAX, "probe entities"), out_of_range = int (head.shape_out_of_range, 0, INT_MAX, "probe shape_out_of_range"), picked = {}, candidates = {}, picked_total = 0 }
	local candidate_total = 0
	for b = 1, #PROBE_NAMES do
		local v = fields (list [b + 1], "tp-probe-bucket", { "bucket", "name", "candidates", "picked", "available" }, "probe case " .. b)
		if int (v.bucket, 1, #PROBE_NAMES, "probe bucket") ~= b or v.name ~= PROBE_NAMES [b] then malformed ("the probe case rows are not in order at " .. b) end
		local candidates, picked, available = int (v.candidates, 0, INT_MAX, "probe candidates"), int (v.picked, 0, PROBE_PICKS, "probe picked"), flag (v.available, "probe available")
		if picked ~= math.min (candidates, PROBE_PICKS) or (available == 1) ~= (picked > 0) then malformed ("probe case " .. b .. " has picks that are not its candidates bounded by " .. PROBE_PICKS) end
		report.picked [b], report.candidates [b] = picked, candidates
		report.picked_total = report.picked_total + picked
		candidate_total = candidate_total + candidates
	end
	-- the population: the candidates and the shapes outside the table are drawn from the entities of the examined families, which are drawn from the list nodes visited
	if report.entities > report.visited then malformed (string.format ("the probe examined %d entities of only %d list nodes visited", report.entities, report.visited)) end
	if candidate_total + report.out_of_range > report.entities then
		malformed (string.format ("the probe counts %d candidates and %d shapes outside the table among only %d examined entities", candidate_total, report.out_of_range, report.entities))
	end
	return report
end

-- The setup probe's report. Each case is either unavailable (no real candidate, said so with its reason, nothing changed) or a real entity given an explicit physical precondition, observed once
-- through the original and restored. A case that was called must have been applied AND restored, and what the original cannot have written (the real instance's viewpoint) unchanged.
-- What the C setup can say when a case is not called. A missing candidate belongs to ITS case only; a candidate that was found but did not take the precondition has the reasons below.
local SETUP_MISSING = {
	{ no_mobile_with_instance = true, no_census_pass = true, no_bbox_scene_without_skin = true },
	{ no_mobile_with_instance = true },
	{ no_fixed_structure_with_valid_shape = true },
}
local SETUP_NOT_CALLED = {
	{ scene_has_target_point = true, setup_not_applied = true },
	{ setup_not_applied = true },
	{ setup_not_applied = true },
}

-- `rows` are the census rows of the pass that was the latest to end at the boot (the setup runs right after it): the models the report names are joined to them by index and name.
local function check_setup (text, rows)
	local list = lines_of (text, "the setup report")
	if #list ~= #SETUP_NAMES + 1 then malformed ("the setup report has " .. #list .. " lines, expected " .. (#SETUP_NAMES + 1)) end
	local head = fields (list [1], "tp-setup", { "status", "visited", "entities" }, "the setup head")
	if head.status ~= "complete" then malformed ("the setup scan is not complete: " .. tostring (head.status)) end
	local report = { visited = int (head.visited, 0, INT_MAX, "setup visited"), entities = int (head.entities, 0, INT_MAX, "setup entities"), cases = {}, called_total = 0 }
	if report.entities > report.visited then malformed ("the setup examined more entities than list nodes") end
	for b = 1, #SETUP_NAMES do
		local v = fields (list [b + 1], "tp-setup-case", { "case", "name", "available", "reason", "applied", "called", "restored", "entity_type", "host_id_state", "from", "from_name", "to", "to_name", "vp_unchanged", "counters_restored" }, "setup case " .. b)
		if int (v.case, 1, #SETUP_NAMES, "setup case") ~= b or v.name ~= SETUP_NAMES [b] then malformed ("the setup case rows are not in order at " .. b) end
		local c = {
			available = flag (v.available, "setup available"), applied = flag (v.applied, "setup applied"), called = flag (v.called, "setup called"), restored = flag (v.restored, "setup restored"),
			reason = name (v.reason, "setup reason"), entity_type = name (v.entity_type, "setup entity_type"), from = int (v.from, -1, INT_MAX, "setup from"), to = int (v.to, -1, INT_MAX, "setup to"),
			vp_unchanged = int (v.vp_unchanged, -1, 1, "setup vp_unchanged"), counters_restored = int (v.counters_restored, -1, 1, "setup counters_restored"), host_id_state = v.host_id_state,
			from_name = name (v.from_name, "setup from_name"), to_name = name (v.to_name, "setup to_name"),
		}
		local what = "setup case " .. b
		if c.available == 0 then
			if c.reason == "-" or c.applied ~= 0 or c.called ~= 0 or c.restored ~= 0 or c.entity_type ~= "-" or c.host_id_state ~= "-" or c.from ~= -1 or c.to ~= -1 or c.vp_unchanged ~= -1 or c.counters_restored ~= -1 then
				malformed (what .. " is unavailable but carries a reason-less or changed state")
			end
			if not SETUP_MISSING [b] [c.reason] then malformed (what .. " is unavailable for the reason '" .. c.reason .. "', which that case cannot have") end
			if c.from_name ~= "-" or c.to_name ~= "-" then malformed (what .. " is unavailable but names a model") end
		else
			if c.host_id_state ~= "present" and c.host_id_state ~= "absent" then malformed (what .. " has the host identity state '" .. tostring (c.host_id_state) .. "'") end
			if c.entity_type == "-" or c.from < 1 then malformed (what .. " is available but names no entity or original value") end
			-- what was done is restored: a restoration that did not hold is never reported, the engine refuses instead
			if c.restored ~= 1 then malformed (what .. " was given a precondition that was not restored") end
			-- the original is called exactly when the precondition took: applied and called are one fact
			if c.called ~= c.applied then malformed (what .. " was called without being applied, or applied and not called") end
			if c.called == 1 then
				if c.reason ~= "-" then malformed (what .. " was called and still gives a reason") end
			elseif not SETUP_NOT_CALLED [b] [c.reason] then
				malformed (what .. " was not called for the reason '" .. c.reason .. "', which that case cannot have")
			end
			-- the original model (the instance's scene, or the shape) is a model of the census, by index and name; the constructed scene is a bounding-box model of it; the sentinels have no name
			local from_row = rows [c.from]
			if not from_row or c.from_name ~= from_row.name then malformed (what .. " names a model of the census that is not the census's model " .. c.from) end
			if b == 1 then
				local to_row = rows [c.to]
				if not to_row or to_row.state ~= "bbox" or c.to_name ~= to_row.name then malformed (what .. " was given a scene that is not a bounding-box model of the census, or not under its census name") end
			elseif c.to_name ~= "-" then
				malformed (what .. " names a model for a sentinel")
			end
			if b == 1 and (c.vp_unchanged ~= 1 or c.counters_restored ~= 1 or c.to < 1) then malformed (what .. " did not restore the real instance's viewpoint and the construction counters") end
			if b == 2 and (c.vp_unchanged ~= 1 or c.counters_restored ~= -1 or c.to ~= -1) then malformed (what .. " did not restore the real instance's viewpoint") end
			if b == 3 and (c.vp_unchanged ~= -1 or c.counters_restored ~= -1 or c.to ~= 0) then malformed (what .. " is not the shape set to the invalid index") end
		end
		report.cases [b] = c
		report.called_total = report.called_total + c.called
	end
	-- Cases 1 and 2 come from ONE scan of the same mobiles: a mobile with an instance was found exactly when case 2 is available, and case 1's 'no mobile' reason is that same fact (a case 1 that
	-- is available, or short of a scene, needs the mobile case 2 has). They may be the same mobile, so no more than one is needed.
	local c1, c2, c3 = report.cases [1], report.cases [2], report.cases [3]
	if (c1.reason == "no_mobile_with_instance") ~= (c2.available == 0) then
		malformed (string.format ("setup cases 1 and 2 disagree about the mobile with an instance one scan found (case 1: '%s', case 2 available: %d)", c1.reason, c2.available))
	end
	-- The population: a mobile with an instance (case 2 available) and a fixed structure (case 3 available) are different entities of the examined families.
	local needed = c2.available + c3.available
	if report.entities < needed then
		malformed (string.format ("the setup reports %d entities but its available cases need at least %d (a mobile and a fixed structure are different entities)", report.entities, needed))
	end
	return report
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
	if type (engine.target_point_summary) ~= "function" or type (engine.target_point_offsets) ~= "function" or type (engine.target_point_event) ~= "function" or type (engine.clock) ~= "function" or type (engine.frame) ~= "function" or (probe_requested and type (engine.target_point_probe) ~= "function") or (setup_requested and type (engine.target_point_setup) ~= "function") then
		emit { event = "not_run", reason = "this module was not built with the target-point-fixture feature" }
		emit { event = "done", complete = false, malformed = false }
		error ("target-point-characterization: NOT RUN: this module was not built with the target-point-fixture feature", 0)
	end

	-- one checked call of the engine, whatever it returns; a failure stops all questions (the engine may be poisoned)
	local function checked_call (what, call)
		local good, value = pcall (call)
		if not good then
			local message = first_line (value)
			emit { event = "report_failed", what = what, error = message }
			error ("target-point-characterization: " .. what .. " failed: " .. message .. " (the engine is not asked again)", 0)
		end
		return value
	end

	-- one question to the engine whose answer is a TEXT report (the summary, the census, a recorded query); the clock is a table and is read with checked_call
	local function ask (what, call)
		local text = checked_call (what, call)
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
		local raw = checked_call ("clock", function () return engine:clock () end)
		local good, clock = pcall (check_clock, raw, when)
		if not good then protocol_failure (clock) end
		emit { event = "clock", when = when, elapsed_seconds = clock.elapsed_seconds, time_of_day_seconds = clock.time_of_day_seconds, day = clock.day }
		return clock
	end

	-- one initialiser pass's rows, read once, kept verbatim, checked against the grammar and joined to that pass's line of the summary it belongs to
	local function read_pass (when, g, summary)
		local text = ask ("target_point_offsets " .. g, function () return engine:target_point_offsets (g) end)
		emit { event = "offsets", when = when, generation = g, text = text }
		local checked, counts, rows = pcall (check_offsets, text, summary, g)
		if not checked then protocol_failure (counts) end
		local joined, reason = pcall (join_census_rows, counts, summary.passes [g], summary.census.expected)
		if not joined then protocol_failure (reason) end
		return text, rows, counts
	end

	-- THE CENSUS, straight after the boot: every original pass of the initialiser the boot ran is kept as its own generation, read and kept in full BEFORE the verdict is judged, so a
	-- capture that is not whole still retains what it saw. It must be whole before any frame is run.
	local boot_summary = sample ("boot")
	local boot_texts, boot_rows = {}, {}
	for g = 0, boot_summary.census.generations - 1 do
		local text, rows, counts = read_pass ("boot", g, boot_summary)
		boot_texts [g] = text
		boot_rows [g] = rows
		emit { event = "census", generation = g, expected = boot_summary.census.expected, sub_object = counts.sub_object, bbox = counts.bbox, invalid_zero = counts.invalid_zero, none = counts.none,
			live_mismatch = counts.live_mismatch, complete = boot_summary.passes [g].complete == 1 }
	end
	emit { event = "census_summary", generations = boot_summary.census.generations, last = boot_summary.census.last, whole = boot_summary.census_whole == 1 }
	if boot_summary.census_whole ~= 1 then
		emit { event = "done", complete = false, malformed = false, census_complete = false }
		error ("target-point-characterization: NOT MEASURED: the census of the offset table is not whole (" .. boot_summary.census.generations .. " passes kept, latest ended " .. boot_summary.census.last ..
			", dropped " .. boot_summary.census.dropped .. ", open " .. boot_summary.census.open .. ", interrupted " .. boot_summary.census.interrupted .. ", stray " .. boot_summary.census.stray .. ")", 0)
	end

	-- THE CONTROLLED PROBE (only when asked): once, after the boot census is whole and before any frame. It is NOT the natural run: the fixture picks the entities, calls the original on them as the
	-- controlled caller and writes nothing (only branch 1 writes the instance viewpoint, and it is not probed). Its queries are read with the others and joined to this report below.
	local probe_report
	if probe_requested then
		local probe_text = ask ("target_point_probe", function () return engine:target_point_probe () end)
		emit { event = "probe", text = probe_text }
		local probed, report = pcall (check_probe, probe_text)
		if not probed then protocol_failure (report) end
		probe_report = report
	end

	-- THE SETUP PROBE (only when asked, and then INSTEAD of the read-only probe): once, after the boot census and before any frame. It changes real physical preconditions, calls the original
	-- and restores them, case by case, with the restoration verified inside the engine (a failure is a refusal that stops the run here, before any frame). Not natural, not host authority.
	local setup_report
	if setup_requested then
		local setup_text = ask ("target_point_setup", function () return engine:target_point_setup () end)
		emit { event = "setup", text = setup_text }
		local prepared, report = pcall (check_setup, setup_text, boot_rows [boot_summary.census.last])
		if not prepared then protocol_failure (report) end
		setup_report = report
	end

	-- THE FRAMES. The campaign decides what is queried; nothing here selects, creates or calls anything.
	local clock_before = read_clock ("before")
	local previous = boot_summary
	-- The summaries are the only moments the driver knows the observer's state. A query's seq is the call count when it BEGAN and its gen the latest ended census pass then, so against each
	-- sample (calls, last pass ended): a query that began before that call count cannot name a later pass than the sample's, and one that began at or after it cannot name an earlier one.
	-- Nothing is assumed between samples, and the open flag (unknown at a query's moment) is not constrained.
	local boundaries = { { calls = boot_summary.calls, last = boot_summary.census.last, when = "the boot sample" } }
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
				if now.census.last < previous.census.last then malformed ("the latest ended census pass went backwards between two samples") end
				if previous ~= boot_summary then rng_continuity (previous, now, "frame " .. f, "the previous sample") end
				same_census (boot_summary, now, "frame " .. f)
			end)
			if not monotone then protocol_failure (bad) end
			previous = now
			boundaries [#boundaries + 1] = { calls = now.calls, last = now.census.last, when = "the sample at frame " .. f }
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
		if final.census.last < previous.census.last then malformed ("the latest ended census pass went backwards at the end") end
		if previous ~= boot_summary then rng_continuity (previous, final, "end", "the previous sample") end
	end)
	if not monotone_end then protocol_failure (bad_end) end
	boundaries [#boundaries + 1] = { calls = final.calls, last = final.census.last, when = "the end sample" }
	-- the final summary may add passes (a later initialisation is a new original product) but may not change one the boot held
	local same, same_reason = pcall (same_census, boot_summary, final, "end")
	if not same then protocol_failure (same_reason) end
	-- every pass is read again, in full: each is joined to its own line of the final summary; a pass the boot held must still be the same words (its live flags may differ only if a later pass
	-- was added; with none added the whole text must be byte-equal), and a later pass is judged only by its own line and the final verdict
	local end_rows = {}
	local grown = final.census.generations > boot_summary.census.generations
	local function without_live (text) return (string.gsub (text, " live=[01%-]\n", "\n")) end
	for g = 0, final.census.generations - 1 do
		local text, rows = read_pass ("end", g, final)
		end_rows [g] = rows
		if g < boot_summary.census.generations then
			if without_live (text) ~= without_live (boot_texts [g]) then protocol_failure ({ malformed = "the census of pass " .. g .. " at the end is not the census read after the boot" }) end
			if not grown and text ~= boot_texts [g] then protocol_failure ({ malformed = "the census of pass " .. g .. " at the end is not byte-equal to the census after the boot although no pass was added" }) end
		end
	end

	local events, seen, previous_gen = {}, -1, -1
	for index = 0, final.events - 1 do
		local text = ask ("target_point_event " .. index, function () return engine:target_point_event (index) end)
		emit { event = "query", index = index, text = text }
		local good, event = pcall (check_event, text, index, final, end_rows)
		if not good then protocol_failure (event) end
		if event.seq <= seen then protocol_failure ({ malformed = "event " .. index .. " does not follow the previous event's sequence number" }) end
		seen = event.seq
		if event.site == SETUP_SITE and not setup_report then protocol_failure ({ malformed = "event " .. index .. " is a setup query but no setup was asked for" }) end
		if event.site == CONTROLLED_SITE and not probe_report then protocol_failure ({ malformed = "event " .. index .. " is a controlled query but no probe was asked for" }) end
		if event.gen < previous_gen then protocol_failure ({ malformed = "event " .. index .. " names an older census pass than the event before it" }) end
		previous_gen = event.gen
		for _, b in ipairs (boundaries) do
			if event.seq < b.calls and event.gen > b.last then
				protocol_failure ({ malformed = string.format ("event %d began before %s (%d calls) yet names pass %d, a pass that had not ended by then (the latest was %d)", index, b.when, b.calls, event.gen, b.last) })
			elseif event.seq >= b.calls and event.gen < b.last then
				protocol_failure ({ malformed = string.format ("event %d began at or after %s (%d calls) yet names pass %d, earlier than the latest that had ended by then (%d)", index, b.when, b.calls, event.gen, b.last) })
			end
		end
		events[index] = event
	end

	-- the rows must reproduce the summary's matrix when every finalised call was kept (nothing dropped or abandoned)
	-- the event row's case name, by the summary's case-count key
	local CASE_KEY = { target_point_subobject = "target_point", instance_without_target_point = "instance_position", no_horizontal_offset = "no_horizontal", heading_zero_quirk = "heading_zero",
		rotated_horizontal_offset = "rotated", not_fixed = "not_fixed", invalid_shape = "invalid_shape" }
	local tally = { branch = { 0, 0, 0, 0 }, present = 0, absent = 0, changed = 0, case = { target_point = 0, instance_position = 0, no_horizontal = 0, heading_zero = 0, rotated = 0, not_fixed = 0, invalid_shape = 0 } }
	local per_site = {}
	for s_index = 0, SITE_COUNT - 1 do per_site [s_index] = { 0, 0, 0, 0 } end
	for index = 0, final.events - 1 do
		local e = events [index]
		if e.branch >= 1 then
			tally.branch [e.branch] = tally.branch [e.branch] + 1
			per_site [e.site] [e.branch] = per_site [e.site] [e.branch] + 1
			local key = CASE_KEY [e.case]
			tally.case [key] = tally.case [key] + 1
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
			for key, n in pairs (tally.case) do
				if n ~= final.case [key] then malformed ("the event rows hold " .. n .. " calls of the case " .. key .. ", the summary " .. final.case [key]) end
			end
		else
			for b = 1, 4 do
				if tally.branch [b] > final.branch [b] then malformed ("the event rows hold more calls of branch " .. b .. " than the summary counts") end
			end
			-- a capture that kept fewer rows than calls can only be bounded by the summary
			for key, n in pairs (tally.case) do
				if n > final.case [key] then malformed ("the event rows hold more calls of the case " .. key .. " than the summary counts") end
			end
			if tally.present > final.case.host_id_present or tally.absent > final.case.host_id_absent or tally.changed > final.case.vp_changed then
				malformed ("the event rows hold more host identities or changed viewpoints than the summary counts")
			end
		end
	end)
	if not rows_agree then protocol_failure (disagreement) end

	-- the controlled queries are exactly the probe's picks, case by case
	local controlled_total, controlled_cases, unavailable = 0, {}, {}
	if probe_report then
		local seen_cases = {}
		for b = 1, #PROBE_NAMES do seen_cases [b] = 0 end
		for index = 0, final.events - 1 do
			if events [index].site == CONTROLLED_SITE then seen_cases [events [index].detail] = seen_cases [events [index].detail] + 1 end
		end
		local probe_joined, probe_why = pcall (function ()
			for b = 1, #PROBE_NAMES do
				if seen_cases [b] ~= probe_report.picked [b] then malformed (string.format ("the probe picked %d entities for %s but %d controlled queries were recorded", probe_report.picked [b], PROBE_NAMES [b], seen_cases [b])) end
				controlled_total = controlled_total + seen_cases [b]
				if probe_report.picked [b] == 0 then unavailable [#unavailable + 1] = tostring (b) end
				if seen_cases [b] > 0 then controlled_cases [#controlled_cases + 1] = tostring (b) end
			end
		end)
		if not probe_joined then protocol_failure (probe_why) end
		-- The probe ran ONCE, immediately after the boot sample and before any frame, so its queries are exactly the contiguous span of call numbers that starts at the boot sample's call count and is as
		-- long as its picks: no controlled query outside it, no natural query inside it. (Nothing is assumed about the time of the other queries.)
		local first_call, last_call = boot_summary.calls, boot_summary.calls + probe_report.picked_total
		local interval_ok, interval_why = pcall (function ()
			if last_call > final.calls then malformed (string.format ("the probe's %d queries do not fit after the boot sample's %d calls in the %d the observer counts", probe_report.picked_total, first_call, final.calls)) end
			for index = 0, final.events - 1 do
				local e = events [index]
				local inside = e.seq >= first_call and e.seq < last_call
				if e.site == CONTROLLED_SITE and not inside then malformed (string.format ("controlled query event %d (call %d) is outside the interval [%d, %d) that starts at the boot sample", index, e.seq, first_call, last_call)) end
				if e.site ~= CONTROLLED_SITE and inside then malformed (string.format ("natural query event %d (call %d) is inside the controlled interval [%d, %d)", index, e.seq, first_call, last_call)) end
			end
		end)
		if not interval_ok then protocol_failure (interval_why) end
	end
	-- the setup queries are exactly the setup's called cases, in the contiguous span after the boot sample, each one the case it was set up as and the entity it was set up on
	local setup_total, setup_called, setup_unavailable = 0, {}, {}
	if setup_report then
		local seen_setup = { 0, 0, 0 }
		for index = 0, final.events - 1 do
			local e = events [index]
			if e.site == SETUP_SITE then seen_setup [e.detail] = seen_setup [e.detail] + 1 end
		end
		local first_call, last_call = boot_summary.calls, boot_summary.calls + setup_report.called_total
		local setup_joined, setup_why = pcall (function ()
			if last_call > final.calls then malformed ("the setup's queries do not fit after the boot sample in the calls the observer counts") end
			for b = 1, #SETUP_NAMES do
				local c = setup_report.cases [b]
				if seen_setup [b] ~= c.called then malformed (string.format ("the setup called %d times for %s but %d setup queries were recorded", c.called, SETUP_NAMES [b], seen_setup [b])) end
				setup_total = setup_total + seen_setup [b]
				if c.called == 1 then setup_called [#setup_called + 1] = tostring (b) else setup_unavailable [#setup_unavailable + 1] = b .. ":" .. c.reason end
			end
			local last_detail = 0
			for index = 0, final.events - 1 do
				local e = events [index]
				local inside = e.seq >= first_call and e.seq < last_call
				if e.site == SETUP_SITE and not inside then malformed (string.format ("setup query event %d (call %d) is outside the interval [%d, %d) that starts at the boot sample", index, e.seq, first_call, last_call)) end
				if e.site ~= SETUP_SITE and inside then malformed (string.format ("other query event %d (call %d) is inside the setup interval [%d, %d)", index, e.seq, first_call, last_call)) end
				if e.site == SETUP_SITE then
					if e.detail < last_detail then malformed ("the setup queries are not in the order of the cases") end
					last_detail = e.detail
					local c = setup_report.cases [e.detail]
					-- the entity the original was asked about is the one the setup changed: its family, and whether it had a host identity
					if e.entity_type_name ~= c.entity_type or e.host_id_state ~= c.host_id_state then malformed (string.format ("setup query event %d is not about the entity case %d changed", index, e.detail)) end
					if e.detail == 1 then
						-- the constructed instance is a real census bounding-box scene: the model the original saw is that scene, with the census's own name IN THE PASS THAT APPLIED WHEN THE QUERY RAN
						-- (e.gen), not in whatever pass came last: a later pass may arrive during the frames without changing what the setup saw
						local row = (end_rows [e.gen] or {}) [c.to]
						if e.model ~= c.to or not row or row.state ~= "bbox" or e.model_name ~= row.name or e.model_name ~= c.to_name then malformed (string.format ("setup query event %d did not see the bounding-box scene it was set up with", index)) end
					elseif e.detail == 3 then
						if e.shape ~= 0 then malformed (string.format ("setup query event %d did not read the invalid shape", index)) end
					end
				end
			end
		end)
		if not setup_joined then protocol_failure (setup_why) end
	end
	local reached = {}
	for b = 1, 4 do
		local natural = 0
		for s_index = 0, CONTROLLED_SITE - 1 do natural = natural + final.site [s_index] [b] end
		if natural > 0 then reached [#reached + 1] = tostring (b) end
	end
	local natural_calls = final.calls - controlled_total - setup_total
	local queries = final.calls - boot_summary.calls - controlled_total - setup_total
	local whole = final.queries_whole == 1 and final.census_whole == 1
	emit { event = "done", rng_history = final.rng.history or "unknown", rng_pending = final.rng.pending == true, rng_exposed = final.rng.exposed, rng_overflow = final.rng.overflow == 1,
		complete = whole, census_complete = true, queries_whole = whole, queries = queries, boot_calls = boot_summary.calls, total_calls = final.calls, events = final.events, generations = final.census.generations,
		natural_queries_reached = natural_calls > 0, branches_reached = #reached > 0 and table.concat (reached, ",") or "none", frames = frames, malformed = false,
		controlled_probe = probe_requested, controlled_queries = controlled_total, controlled_cases = #controlled_cases > 0 and table.concat (controlled_cases, ",") or "none",
		controlled_unavailable = #unavailable > 0 and table.concat (unavailable, ",") or "none",
		setup_probe = setup_requested, setup_queries = setup_total, setup_cases = #setup_called > 0 and table.concat (setup_called, ",") or "none",
		setup_unavailable = #setup_unavailable > 0 and table.concat (setup_unavailable, ",") or "none" }
	if setup_requested and #setup_unavailable > 0 then
		emit { event = "setup_unavailable", cases = table.concat (setup_unavailable, ","), reason = "no real candidate or the precondition did not apply; nothing is claimed for these cases" }
		io.stderr:write (string.format ("target-point-characterization: setup probe: cases %s were NOT called\n", table.concat (setup_unavailable, ",")))
	end
	if probe_requested and #unavailable > 0 then
		emit { event = "probe_unavailable", cases = table.concat (unavailable, ","), reason = "no live entity of the examined families takes these cases; nothing is claimed for them" }
		io.stderr:write (string.format ("target-point-characterization: controlled probe: cases %s are UNAVAILABLE (no candidate); the other cases were picked\n", table.concat (unavailable, ",")))
	end
	if natural_calls == 0 then
		emit { event = "unreached", reason = "no query of the original target-point function was made in " .. frames .. " frames; no branch is covered" }
		io.stderr:write (string.format ("target-point-characterization: census whole; NO natural target-point query was reached in %d frames (nothing is covered)\n", frames))
	else
		io.stderr:write (string.format ("target-point-characterization: census whole; %d queries (%d at boot), branches reached: %s\n", natural_calls, boot_summary.calls, #reached > 0 and table.concat (reached, ",") or "none"))
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
