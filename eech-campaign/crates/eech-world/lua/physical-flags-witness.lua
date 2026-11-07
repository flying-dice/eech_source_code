-- TEST-ONLY host-owned witness of the PHYSICAL FLAG STORE (EEDCS-49, D3, Task49): ENV-PHYSICAL-FLAGS-HOST-STORE-NATIVE-ACCEPTANCE-1's public path. One bounded process per mode, a fresh installation root, its report created
-- EXCLUSIVELY first and never overwritten. Contract and plan: docs/d3-physical-flags-contract.md sections 8 to 10. Root runs it natively after the SAME source review; nothing here is a native result, and the scripted tests of this file
-- (tests/physical_flags_witness_driver.rs) are boundary checks of THIS driver over the REAL reference provider, never campaign authority.
--
-- Boundary under test (the product path): this host-owned Lua -> the public native module (`require ("eech_dc")`, `dc.boot`, `engine:frame`) -> the REAL campaign code, which keeps none of the three per-entity words (UPDATED, MOVED,
-- ROTATED) in the hosted profile and asks THIS host for every one of them through the one bridge the ordinary build ships: scan phases 19 GROUP (open a word group, the host issues its key `f<digits>`), 20 SET (a lane), 21 READ
-- (the destructive raw read of a lane) and 22 RESET (an array). The answers are the real provider's (`host.model`, whose `FlagStore` is the store), delegated to UNCHANGED and counted; a mode alters ONE eligible answer and nothing else.
-- Nothing is searched: no actor, ray, seed or window is chosen by what a run reached (the duration is named by the caller BEFORE the launch: by default the established finite plan of the unchanged-default 020 native runs, lebanon_retail,
-- hours=0.001 (35 frames of 100 ms, 3.5 s), seed 1; the regression pack's 3 hours is a longer window a caller may name before a launch); a case that is not reached is UNREACHED, never a pass.
--
-- What is observed. Every flag question and the answer to it, as the exact grammar of the public C bridge (mb_hostentity.h campaign_entity_flag_exchange): sixteen patterns and, for a SET or a READ, one space and the key; three patterns
-- back, a key only for a group. ATTEMPTS (questions the host received), ANSWERED (the provider's reply was whole) and ACCEPTED (the engine went on after an unaltered reply: its next host callback, or the frame returned without a fatal) are
-- counted separately per phase. The BACKING check is an oracle over the OBSERVED traffic only (never a second store the campaign could use): the destructive `&=` semantics of the original's getter, rebuilt from the sets, reads and
-- resets the campaign made. A READ must answer the lane's mask when the lane's bit was set since the last reset of its array and no word-mate's read destroyed it, otherwise 0; a RESET clears the array of the groups opened as
-- eligible and states their number; a group's key is new. Any difference is a reply fault of the HOST under test, not an intervention.
--
--   flags_original            the ORIGINAL-solver profile (a build that keeps the original entity solver: `eech-dc/collision-fixture`): the campaign keeps and consumes its own arrays and the host is never asked for a flag.
--                             FAILURE if it WAS asked (not that profile). The characterisation FIRST.
--   flags_control             hosted, nothing altered. Runs until the first read that follows an UPDATED-asking candidate decision has been consumed (or the duration ends). PASS needs an accepted group, set and reset, no
--                             fault and a backing check without a difference; the reads reached are reported (a read is not required of a duration that never reached one: the read modes then refuse). With `original=<the successful
--                             flags_original report>` it is PAIRED (same setup, the host never asked); an intervention mode accepts nothing else.
--   <event>_withheld          the host raises at the first eligible <event> (group, set, read, reset):         [model-provider:entity_scan.callback]
--   <event>_malformed         two patterns instead of three:                                                   [model-provider:entity_scan.answer]
--   <event>_wrong_state       `height` for a flag question:                                                    [model-provider:entity_scan.state]
--   group_bad_key             a key of 21 bytes (the campaign's identity slot holds 20):                       [model-provider:flag.key]
--   group_duplicate_key       the SECOND group is issued the key of the first, which is live (a wrong identity): [model-provider:flag.key]
--   group_wrong_eligibility   the eligibility stated is echoed the other way:                                  [model-provider:flag.answer]
--   set_wrong_lane / read_wrong_array / reset_wrong_array
--                             the lane (array) echoed is another than the one asked about:                     [model-provider:flag.answer]
--   set_wrong_word / read_wrong_word
--                             a raw word that is neither the lane's mask (nor, for a read, 0):                 [model-provider:flag.word]
--   read_changed              a VALID changed host answer: the first read that follows an UPDATED-asking candidate decision (the campaign's moving-candidate sweep) is answered the other way (0 for the lane's mask, the mask
--                             for 0). PASS needs the campaign to CARRY it: the very next candidate question of the same candidate states UPDATED the way the changed word says and not the way the host's own word said.
--   Each refusal mode needs `control=<the successful PAIRED flags_control report>` of the same setup and is credited ONLY with the one eligible intervention actually made AND the engine stopping at exactly that marker after it; a fatal
--   before it, a run that ends without a fatal, or another marker is a FAILURE; an event never reached is UNREACHED. The intervention is made only at the step whose trajectory (ordinal, frame, attempts of every phase before it) equals the
--   control's: never on another step. A fatal, a request fault or a reply fault always takes precedence over a clean UNREACHED.
--
-- host.args: root, out (the report), mode, scenario (lebanon_retail default, georgia_retail), map, campaign, hours (default 0.001), frame_ms (100), seed (1), original, control.
-- NOTHING IN THIS FILE DIVIDES BY A VALUE THAT CAN BE ZERO (a hardware trap natively, which pcall cannot catch): the one division is the frame count by `frame_ms`, asserted positive.

local NAME = "physical-flags-witness"
local function refuse (message) error (NAME .. ": " .. message, 0) end

local args = host.args
local root = assert (args.root, "root=<installation root>")
local out_path = assert (args.out, "out=<report file>")
local mode = assert (args.mode, "mode=<a mode of the header>")

local ZERO = "0x00000000"
local ONE = "0x3f800000"
local TWO = "0x40000000"
local PAT = "0x%x%x%x%x%x%x%x%x"
local EVENTS = { "group", "set", "read", "reset" }
local EVENT_OF = { ["0x41980000"] = "group", ["0x41a00000"] = "set", ["0x41a80000"] = "read", ["0x41b00000"] = "reset" }
local RESULT_OF = { group = "flag_group", set = "flag_set", read = "flag_read", reset = "flag_reset" }

local MODES = {
	flags_original = { profile = "original-solver" },
	flags_control = { profile = "hosted" },
	read_changed = { profile = "hosted", at = "read_consumable", changed = true },
	group_bad_key = { profile = "hosted", at = "group", fault = "bad_key", marker = "flag.key" },
	group_duplicate_key = { profile = "hosted", at = "group_second", fault = "duplicate_key", marker = "flag.key" },
	group_wrong_eligibility = { profile = "hosted", at = "group", fault = "wrong_eligibility", marker = "flag.answer" },
	set_wrong_lane = { profile = "hosted", at = "set", fault = "wrong_lane", marker = "flag.answer" },
	set_wrong_word = { profile = "hosted", at = "set", fault = "wrong_word", marker = "flag.word" },
	read_wrong_array = { profile = "hosted", at = "read", fault = "wrong_array", marker = "flag.answer" },
	read_wrong_word = { profile = "hosted", at = "read", fault = "wrong_word", marker = "flag.word" },
	reset_wrong_array = { profile = "hosted", at = "reset", fault = "wrong_array", marker = "flag.answer" },
}
for _, event in ipairs (EVENTS) do
	MODES [event .. "_withheld"] = { profile = "hosted", at = event, fault = "withheld", marker = "entity_scan.callback" }
	MODES [event .. "_malformed"] = { profile = "hosted", at = event, fault = "malformed", marker = "entity_scan.answer" }
	MODES [event .. "_wrong_state"] = { profile = "hosted", at = event, fault = "wrong_state", marker = "entity_scan.state" }
end
local plan = MODES [mode]
if plan == nil then refuse ("mode=" .. tostring (mode) .. " is not a mode of this witness") end
local intervening = plan.at ~= nil

local hours = tonumber (args.hours or "0.001")
local frame_ms = tonumber (args.frame_ms or "100")
local seed = tonumber (args.seed or "1")
assert (hours and hours >= 0 and frame_ms and frame_ms > 0 and seed, "hours, frame_ms and seed must be numbers")
local scenarios = {
	georgia_retail = { map = "..\\common\\maps\\map3", campaign = "GEORGIA.CHC" },
	lebanon_retail = { map = "..\\common\\maps\\map5", campaign = "LEBANON.CHC" },
}
local scenario_name = args.scenario or "lebanon_retail"
local scenario = scenarios [scenario_name]
if scenario == nil then refuse ("scenario=georgia_retail|lebanon_retail") end
local effective_campaign = args.campaign or scenario.campaign
local effective_map = args.map or scenario.map
for _, key in ipairs ({ "map", "campaign" }) do
	local value = args [key]
	if value ~= nil and (type (value) ~= "string" or value == "" or string.find (value, "%s")) then refuse (key .. "= must be one non-empty token without white space (it is part of the report's setup identity)") end
end
if args.original ~= nil and mode ~= "flags_control" then refuse ("original=<report> is only for flags_control") end
if args.original ~= nil and (type (args.original) ~= "string" or args.original == "") then refuse ("original=<the flags_original report>") end
if args.control ~= nil and not intervening then refuse ("control=<report> is only for a mode that intervenes") end
if intervening and (type (args.control) ~= "string" or args.control == "") then refuse ("mode " .. mode .. " needs control=<the successful PAIRED flags_control report>") end

-- ---------------------------------------------------------------------------------------------------------------- exact raw grammar (the public C bridge's)
-- a comma list of EXACTLY `count` (when given) patterns 0x%08x and nothing else: an empty segment, an extra or trailing comma, a space or a short token is malformed; the patterns come back lower case
local function strict_patterns (text, count)
	if type (text) ~= "string" or text == "" then return nil end
	local out, position = {}, 1
	while true do
		local token = string.sub (text, position, position + 9)
		if not string.match (token, "^" .. PAT .. "$") then return nil end
		out [#out + 1] = string.lower (token)
		position = position + 10
		local separator = string.sub (text, position, position)
		if separator == "" then break end
		if separator ~= "," then return nil end
		position = position + 1
	end
	if count ~= nil and #out ~= count then return nil end
	return out
end
-- the whole numbers as the single patterns the campaign states them in (computed, never listed) and a lane's mask as a pattern (the hex digit is placed by text: never through a number above 2^31)
-- BEGIN flag patterns
-- the powers of two by repeated doubling, exact in every rounding mode. NEVER `2 ^ n`: libm's pow is one unit low under the reference round-toward-zero profile the drivers select before the boot (2 ^ 1 came out below 2 and the
-- mask of lane 1 printed as 0x00000001); tests/physical_flags_witness_driver.rs runs this block under that profile, and physical-flags-witness.lua holds the SAME text
local POW2 = { [0] = 1 }
for n = 1, 31 do POW2 [n] = POW2 [n - 1] * 2 end
local function whole_pattern (n)
	if n == 0 then return ZERO end
	local e = 0
	while POW2 [e + 1] <= n do e = e + 1 end
	return string.format ("0x%08x", (127 + e) * 8388608 + (n - POW2 [e]) * POW2 [23 - e])
end
-- the lane's mask as a pattern (never through a number above 2^31: the hex digit is placed by text)
local function mask_pattern (lane)
	local digits = { "0", "0", "0", "0", "0", "0", "0", "0" }
	digits [8 - math.floor (lane / 4)] = string.format ("%x", POW2 [lane % 4])
	return "0x" .. table.concat (digits)
end
-- END flag patterns
local ARRAYS, LANES = {}, {}
for n = 0, 2 do ARRAYS [whole_pattern (n)] = n end
for n = 0, 31 do LANES [whole_pattern (n)] = n end
local function is_flag_key (text) return type (text) == "string" and #text <= 20 and string.match (text, "^f[1-9]%d*$") ~= nil end

-- ---------------------------------------------------------------------------------------------------------------- whole reports consumed as references
local function strict_pairs (line)
	local t, first = {}, true
	for token in string.gmatch (line, "%S+") do
		if first then
			first = false
		else
			local k, v = string.match (token, "^([%w_]+)=(%S+)$")
			if not k then return nil, "the token \"" .. token .. "\" is not key=value" end
			if t [k] ~= nil then return nil, "the key " .. k .. " is given twice" end
			t [k] = v
		end
	end
	return t
end
local KNOWN = { ["pf-witness-identity"] = "one", ["pf-witness-counts"] = "one", ["pf-witness-first"] = "many", ["pf-witness-consume"] = "one", ["pf-witness-oracle"] = "one", ["pf-witness-verdict"] = "one" }
local REFUSED_RECORD = { ["pf-witness-fatal"] = true, ["pf-witness-reply-fault"] = true, ["pf-witness-request-fault"] = true }
-- a whole non-negative integer in canonical decimal form (a count, ordinal or frame), or nil
local function whole_number (text)
	if type (text) ~= "string" or not string.match (text, "^%d+$") or #text > 9 then return nil end
	local n = tonumber (text)
	if tostring (n) ~= text then return nil end
	return n
end
-- the COMPLETE schema 1 of each record, in the exact key sets the report emits; a reference with a missing or an extra key, a non-canonical number or a record out of its place is not a positive reference
local RANK = { ["pf-witness-identity"] = 1, ["pf-witness-counts"] = 2, ["pf-witness-oracle"] = 3, ["pf-witness-first"] = 4, ["pf-witness-consume"] = 5, ["pf-witness-verdict"] = 6 }
local IDENTITY_SCHEMA = { "schema", "mode", "profile", "scenario", "seed", "frame_ms", "hours", "campaign", "map", "paired" }
local COUNT_SCHEMA = { "frames", "attempts_group", "attempts_set", "attempts_read", "attempts_reset", "answered_group", "answered_set", "answered_read", "answered_reset", "accepted_group", "accepted_set", "accepted_read", "accepted_reset",
	"altered", "changed_accepted", "keys_issued", "reads_zero", "reads_mask", "reads_consumable", "resets_cleared", "oracle_sets", "oracle_reads", "oracle_resets", "oracle_mismatches" }
local ORACLE_SCHEMA = { "sets", "reads", "resets", "mismatches" }
local FIRST_FIELDS = { "ordinal", "frame", "before_group", "before_set", "before_read", "before_reset" }
local FIRST_SCHEMA = { "event", "ordinal", "frame", "before_group", "before_set", "before_read", "before_reset" }
local CONSUME_SCHEMA = { "fact", "expected", "host", "real_word", "answered_word", "changed" }
local FIRST_EVENTS = { group = true, group_second = true, set = true, read = true, read_consumable = true, reset = true }
-- the words a READ can return: 0 or exactly one lane's mask
local LEGAL_RAW = { [ZERO] = true }
for lane = 0, 31 do LEGAL_RAW [mask_pattern (lane)] = true end

local function exact_keys (record, names, role, what)
	local allowed = {}
	for _, name in ipairs (names) do
		allowed [name] = true
		if record [name] == nil then refuse ("the " .. role .. " report's " .. what .. " record lacks " .. name) end
	end
	for key in pairs (record) do
		if not allowed [key] then refuse ("the " .. role .. " report's " .. what .. " record has the unknown key " .. key) end
	end
end
local function whole_counts (record, names, role, what)
	local n = {}
	for _, name in ipairs (names) do
		n [name] = whole_number (record [name])
		if n [name] == nil then refuse ("the " .. role .. " report's " .. what .. " record has no canonical whole " .. name .. " (" .. tostring (record [name]) .. ")") end
	end
	return n
end

-- the whole successful report of this driver at `path` for `role`, or a refusal naming what is missing, malformed or inconsistent with THIS run's setup. Only the records and their order are read here; the schema is judged by validate_positive
local function read_reference (path, role, expected_mode)
	local file = io.open (path, "rb")
	if file == nil then refuse ("cannot read the " .. role .. " report " .. path) end
	local rec, last, first_name, rank = {}, nil, nil, 0
	for line in file:lines () do
		line = string.gsub (line, "\r$", "")
		local name = string.match (line, "^(%S+)")
		if name == nil then file:close () refuse ("the " .. role .. " report holds an empty line") end
		if REFUSED_RECORD [name] then file:close () refuse ("the " .. role .. " report holds a " .. name .. " record: it is not a successful report") end
		local kind = KNOWN [name]
		if kind == nil then file:close () refuse ("the " .. role .. " report holds the unknown record " .. name) end
		if RANK [name] < rank then file:close () refuse ("the " .. role .. " report's records are out of order at " .. name .. " (identity, counts, oracle, first-event rows, consume, verdict)") end
		rank = RANK [name]
		local fields
		if name == "pf-witness-verdict" then
			local m, s, detail = string.match (line, "^pf%-witness%-verdict mode=(%S+) status=(%S+) detail=(.+)$")
			if m == nil then file:close () refuse ("the " .. role .. " report's verdict line is malformed") end
			fields = { mode = m, status = s, detail = detail }
		else
			local why
			fields, why = strict_pairs (line)
			if fields == nil then file:close () refuse ("the " .. role .. " report's " .. name .. " record is malformed: " .. why) end
		end
		if kind == "one" and rec [name] ~= nil then file:close () refuse ("the " .. role .. " report holds the single record " .. name .. " twice") end
		if kind == "one" then
			rec [name] = fields
		else
			rec [name] = rec [name] or {}
			rec [name] [#rec [name] + 1] = fields
		end
		first_name = first_name or name
		last = name
	end
	file:close ()
	if first_name ~= "pf-witness-identity" or last ~= "pf-witness-verdict" then refuse ("the " .. role .. " report does not begin with its identity and end with its verdict") end
	for _, needed in ipairs ({ "pf-witness-counts", "pf-witness-oracle" }) do
		if rec [needed] == nil then refuse ("the " .. role .. " report has no " .. needed .. " record") end
	end
	local id, verdict_record = rec ["pf-witness-identity"], rec ["pf-witness-verdict"]
	if id.schema ~= "1" or id.mode ~= expected_mode or verdict_record.mode ~= expected_mode or verdict_record.status ~= "PASS" then
		refuse ("the " .. role .. " report is not a successful schema 1 " .. expected_mode .. " report (mode " .. tostring (id.mode) .. ", verdict " .. tostring (verdict_record.status) .. ")")
	end
	if id.profile ~= MODES [expected_mode].profile then refuse ("the " .. role .. " report names profile " .. tostring (id.profile) .. ", not " .. MODES [expected_mode].profile) end
	-- the semantic setup: scenario, seed, frame, duration and the EFFECTIVE campaign and map, never a physical root path
	local setup = { scenario = scenario_name, seed = tostring (seed), frame_ms = tostring (frame_ms), hours = tostring (hours), map = effective_map, campaign = effective_campaign }
	for key, mine in pairs (setup) do
		if id [key] ~= mine then refuse ("the " .. role .. " report is of another setup: " .. key .. "=" .. tostring (id [key]) .. ", this run " .. mine) end
	end
	return rec
end

-- a COMPLETE healthy positive: every record of its role in its exact schema, canonical numbers, and the relationships between the records that an actual run of that role cannot violate. Returns the numbers and the first-event rows by event
local function validate_positive (rec, role)
	local original = role == "original"
	exact_keys (rec ["pf-witness-identity"], IDENTITY_SCHEMA, role, "identity")
	local paired = rec ["pf-witness-identity"].paired
	if paired ~= "0" and paired ~= "1" then refuse ("the " .. role .. " report's identity record has no paired flag of 0 or 1") end
	if original and paired ~= "0" then refuse ("the original report is paired: the characterisation has nothing to pair with") end
	exact_keys (rec ["pf-witness-counts"], COUNT_SCHEMA, role, "counts")
	local n = whole_counts (rec ["pf-witness-counts"], COUNT_SCHEMA, role, "counts")
	if n.frames < 1 then refuse ("the " .. role .. " report ran no frames (frames " .. n.frames .. ")") end
	local total = 0
	for _, event in ipairs (EVENTS) do
		local attempts, answered, accepted = n ["attempts_" .. event], n ["answered_" .. event], n ["accepted_" .. event]
		total = total + attempts
		-- a healthy run has no reply fault and every answered reply was gone on from (the loop settles the last one): the three counts are one number
		if answered ~= attempts then refuse ("the " .. role .. " report's counts: answered_" .. event .. " " .. answered .. " differs from attempts_" .. event .. " " .. attempts) end
		if accepted ~= answered then refuse ("the " .. role .. " report's counts: accepted_" .. event .. " " .. accepted .. " differs from answered_" .. event .. " " .. answered .. ": a healthy report accepts every answered reply") end
	end
	if n.altered ~= 0 then refuse ("the " .. role .. " report's counts: altered " .. n.altered .. ": it is not an unaltered report") end
	if n.changed_accepted ~= 0 then refuse ("the " .. role .. " report's counts: changed_accepted " .. n.changed_accepted .. " in an unaltered report") end
	if n.oracle_mismatches ~= 0 then refuse ("the " .. role .. " report's counts: oracle_mismatches " .. n.oracle_mismatches .. ": the backing check found a difference") end
	if n.keys_issued ~= n.answered_group then refuse ("the " .. role .. " report's counts: keys_issued " .. n.keys_issued .. " differs from answered_group " .. n.answered_group) end
	if n.oracle_sets ~= n.answered_set then refuse ("the " .. role .. " report's counts: oracle_sets " .. n.oracle_sets .. " differs from answered_set " .. n.answered_set) end
	if n.oracle_reads ~= n.answered_read then refuse ("the " .. role .. " report's counts: oracle_reads " .. n.oracle_reads .. " differs from answered_read " .. n.answered_read) end
	if n.oracle_resets ~= n.answered_reset then refuse ("the " .. role .. " report's counts: oracle_resets " .. n.oracle_resets .. " differs from answered_reset " .. n.answered_reset) end
	if n.reads_zero + n.reads_mask ~= n.oracle_reads then refuse ("the " .. role .. " report's counts: reads_zero " .. n.reads_zero .. " plus reads_mask " .. n.reads_mask .. " differs from oracle_reads " .. n.oracle_reads) end
	if n.reads_consumable > n.answered_read then refuse ("the " .. role .. " report's counts: reads_consumable " .. n.reads_consumable .. " exceeds answered_read " .. n.answered_read) end
	if n.resets_cleared > n.oracle_resets * n.keys_issued then refuse ("the " .. role .. " report's counts: resets_cleared " .. n.resets_cleared .. " exceeds what " .. n.oracle_resets .. " resets of " .. n.keys_issued .. " groups can clear") end
	-- the oracle record is the counts' own oracle values
	local oracle = rec ["pf-witness-oracle"]
	exact_keys (oracle, ORACLE_SCHEMA, role, "pf-witness-oracle")
	local o = whole_counts (oracle, ORACLE_SCHEMA, role, "pf-witness-oracle")
	if o.sets ~= n.oracle_sets or o.reads ~= n.oracle_reads or o.resets ~= n.oracle_resets or o.mismatches ~= n.oracle_mismatches then
		refuse ("the " .. role .. " report's pf-witness-oracle record (sets " .. o.sets .. ", reads " .. o.reads .. ", resets " .. o.resets .. ", mismatches " .. o.mismatches .. ") differs from the oracle values of its counts record")
	end
	local rows, consume = rec ["pf-witness-first"] or {}, rec ["pf-witness-consume"]
	if original then
		-- the original-solver profile keeps its own arrays: NOTHING reaches the host, so every traffic, key, oracle and consumption number is zero and no first-event row exists
		for _, name in ipairs (COUNT_SCHEMA) do
			if name ~= "frames" and n [name] ~= 0 then refuse ("the original report's host saw flag traffic: " .. name .. " is " .. n [name] .. ", not 0: it is not the original-solver profile") end
		end
		if #rows > 0 then refuse ("an original report holds first-event rows: the host was never asked") end
		if consume ~= nil then refuse ("an original report holds a pf-witness-consume record: no candidate read reaches the host") end
		return n, {}
	end
	-- the first-event rows: each in its exact schema and bounds, unique, present exactly when the counts say the event happened, in the order they happened, joined to the counts around them
	local by_event, previous = {}, nil
	for _, row in ipairs (rows) do
		exact_keys (row, FIRST_SCHEMA, role, "first-event")
		if not FIRST_EVENTS [row.event] then refuse ("the " .. role .. " report holds a first-event row of the unknown event " .. tostring (row.event)) end
		if by_event [row.event] ~= nil then refuse ("the " .. role .. " report holds the first-event row " .. row.event .. " twice") end
		local f = whole_counts (row, FIRST_FIELDS, role, "first-event " .. row.event)
		local own = (row.event == "group" or row.event == "group_second") and "group" or ((row.event == "read_consumable") and "read" or row.event)
		-- a set or a read names a key the host issued, so a group came before it; the first group has none before it, the second exactly one
		if (row.event == "set" or own == "read") and f.before_group < 1 then refuse ("the " .. role .. " report's first-event row " .. row.event .. " has before_group 0: a set or a read names a key a group opened before") end
		if row.event == "group" and f.before_group ~= 0 then refuse ("the " .. role .. " report's first-event row group has before_group " .. f.before_group .. ", not 0") end
		if row.event == "group_second" and f.before_group ~= 1 then refuse ("the " .. role .. " report's first-event row group_second has before_group " .. f.before_group .. ", not 1") end
		if (row.event == "set" or row.event == "read" or row.event == "reset") and f ["before_" .. row.event] ~= 0 then refuse ("the " .. role .. " report's first-event row " .. row.event .. " has before_" .. row.event .. " " .. f ["before_" .. row.event] .. ", not 0") end
		for _, event in ipairs (EVENTS) do
			local limit = n ["attempts_" .. event] - ((own == event) and 1 or 0)
			if f ["before_" .. event] > limit then refuse ("the " .. role .. " report's first-event row " .. row.event .. " has before_" .. event .. " " .. f ["before_" .. event] .. " beyond the " .. n ["attempts_" .. event] .. " attempts of the whole run") end
		end
		if f.ordinal < 1 or f.ordinal > total then refuse ("the " .. role .. " report's first-event row " .. row.event .. " has ordinal " .. f.ordinal .. " outside the " .. total .. " questions of the run") end
		if f.frame > n.frames then refuse ("the " .. role .. " report's first-event row " .. row.event .. " has frame " .. f.frame .. " beyond the " .. n.frames .. " frames of the run (frame 0 is the engine's boot)") end
		if f.before_group + f.before_set + f.before_read + f.before_reset ~= f.ordinal - 1 then refuse ("the " .. role .. " report's first-event row " .. row.event .. ": the before_ counts do not add up to ordinal - 1") end
		if previous ~= nil then
			if f.ordinal < previous.ordinal or f.frame < previous.frame or f.before_group < previous.before_group or f.before_set < previous.before_set or f.before_read < previous.before_read or f.before_reset < previous.before_reset then
				refuse ("the " .. role .. " report's first-event rows are out of order at " .. row.event .. ": they must follow the order the events happened in")
			end
			if f.ordinal == previous.ordinal then
				-- ONE question is ONE event: distinct base events need strictly increasing ordinals; the only equal ordinal is the ordered first READ followed by the read_consumable tag of that same read, with the identical frame and complete prefix
				if not (previous.event == "read" and row.event == "read_consumable") then
					refuse ("the " .. role .. " report's first-event rows " .. previous.event .. " and " .. row.event .. " share the ordinal " .. f.ordinal .. ": distinct base events need strictly increasing ordinals (only the first read and its read_consumable tag may share one)")
				end
				if f.frame ~= previous.frame or f.before_group ~= previous.before_group or f.before_set ~= previous.before_set or f.before_read ~= previous.before_read or f.before_reset ~= previous.before_reset then
					refuse ("the " .. role .. " report's first-event rows " .. previous.event .. " and " .. row.event .. " share the ordinal " .. f.ordinal .. " but not their frame and complete prefix")
				end
			end
		end
		f.event = row.event
		previous = f
		by_event [row.event] = f
	end
	for _, check in ipairs ({ { "group", "attempts_group", 1 }, { "group_second", "attempts_group", 2 }, { "set", "attempts_set", 1 }, { "read", "attempts_read", 1 }, { "reset", "attempts_reset", 1 } }) do
		local happened = n [check [2]] >= check [3]
		if happened and by_event [check [1]] == nil then refuse ("the " .. role .. " report's counts say " .. check [1] .. " happened (" .. check [2] .. " " .. n [check [2]] .. ") but it holds no " .. check [1] .. " first-event row") end
		if not happened and by_event [check [1]] ~= nil then refuse ("the " .. role .. " report holds a " .. check [1] .. " first-event row but " .. check [2] .. " is " .. n [check [2]]) end
	end
	if (n.reads_consumable >= 1) ~= (by_event.read_consumable ~= nil) then
		refuse ("the " .. role .. " report's reads_consumable " .. n.reads_consumable .. " and its read_consumable first-event row disagree (a consumable read is counted exactly when its row exists)")
	end
	for _, pair in ipairs ({ { "group", "set" }, { "group", "read" }, { "group", "group_second" } }) do
		local a, b = by_event [pair [1]], by_event [pair [2]]
		if a ~= nil and b ~= nil and a.ordinal >= b.ordinal then refuse ("the " .. role .. " report's first-event row " .. pair [1] .. " (ordinal " .. a.ordinal .. ") is not before " .. pair [2] .. " (ordinal " .. b.ordinal .. ")") end
	end
	if consume ~= nil then
		-- the consumption tuple of an UNALTERED control: the raw word the host answered, the UPDATED fact the campaign carried (1.0 not updated, 2.0 updated) and what the host's own word says, all one relationship
		exact_keys (consume, CONSUME_SCHEMA, role, "pf-witness-consume")
		local function updated_fact (word) return word == ZERO and ONE or TWO end
		for _, name in ipairs ({ "fact", "expected", "host" }) do
			if consume [name] ~= ONE and consume [name] ~= TWO then refuse ("the " .. role .. " report's pf-witness-consume record: " .. name .. " " .. tostring (consume [name]) .. " is neither 1.0 (0x3f800000) nor 2.0 (0x40000000)") end
		end
		for _, name in ipairs ({ "real_word", "answered_word" }) do
			if strict_patterns (consume [name], 1) == nil or not LEGAL_RAW [string.lower (consume [name])] then refuse ("the " .. role .. " report's pf-witness-consume record: " .. name .. " " .. tostring (consume [name]) .. " is neither 0 nor one lane's mask") end
		end
		if consume.changed ~= "0" then refuse ("the " .. role .. " report's pf-witness-consume record: changed " .. tostring (consume.changed) .. ": a healthy control alters nothing") end
		if consume.answered_word ~= consume.real_word then refuse ("the " .. role .. " report's pf-witness-consume record: answered_word " .. consume.answered_word .. " differs from real_word " .. consume.real_word .. " in an unaltered control") end
		if consume.host ~= updated_fact (consume.real_word) then refuse ("the " .. role .. " report's pf-witness-consume record: host " .. consume.host .. " is not the UPDATED fact of real_word " .. consume.real_word) end
		if consume.expected ~= updated_fact (consume.answered_word) or consume.fact ~= consume.expected then refuse ("the " .. role .. " report's pf-witness-consume record: fact " .. consume.fact .. " is not the expected " .. consume.expected .. " of answered_word " .. consume.answered_word) end
		if by_event.read_consumable == nil then refuse ("the " .. role .. " report holds a pf-witness-consume record but no read_consumable first-event row") end
	end
	return n, by_event
end

local reference = {}
if mode == "flags_control" and args.original ~= nil then
	-- the original-profile characterisation: a COMPLETE healthy Original report of this setup, the host never asked for anything
	validate_positive (read_reference (args.original, "original", "flags_original"), "original")
elseif intervening then
	-- the COMPLETE PAIRED control BEFORE any output, preparation, boot or intervention; it supports ONLY the cases its own positive reached
	local rec = read_reference (args.control, "control", "flags_control")
	local n, by_event = validate_positive (rec, "control")
	if rec ["pf-witness-identity"].paired ~= "1" then refuse ("the control report is not a PAIRED control (an unpaired control makes no parity and unlocks no intervention)") end
	for _, event in ipairs ({ "group", "set", "reset" }) do
		if n ["accepted_" .. event] < 1 then refuse ("the control report's host never had a flag " .. event .. " accepted: it is not the hosted control") end
	end
	local at = by_event [plan.at]
	if at == nil then refuse ("the control never reached the event " .. plan.at .. " this mode intervenes at: the case is UNREACHED on this setup, no intervention is made") end
	if plan.changed and rec ["pf-witness-consume"] == nil then refuse ("the control report holds no pf-witness-consume record: the campaign's carrying of the host's own word into its candidate question must be seen first") end
	reference.first = { ordinal = tostring (at.ordinal), frame = tostring (at.frame), before_group = tostring (at.before_group), before_set = tostring (at.before_set), before_read = tostring (at.before_read), before_reset = tostring (at.before_reset) }
end

if type (host.create_exclusive) ~= "function" then refuse ("this host has no create_exclusive: an output that may never overwrite anything cannot be made safely") end
local out, why = host.create_exclusive (out_path)
if not out then refuse ("cannot create " .. out_path .. " exclusively: " .. tostring (why)) end
local function emit (line) assert (out:write (line .. "\n")) end

-- ---------------------------------------------------------------------------------------------------------------- observation
local function zeroed () return { group = 0, set = 0, read = 0, reset = 0 } end
local state = {
	frame = 0, frames_run = 0, ordinal = 0,
	attempts = zeroed (), answered = zeroed (), accepted = zeroed (),
	altered = 0, intervened = 0, intervention = nil, abort = nil, unsettled = nil, changed_accepted = 0,
	first = {}, first_order = {},
	need_updated = nil, pending_consume = nil, consumed = nil, reads_consumable = 0, consumable_seen = false,
	keys = {}, key_order = {}, groups = {},
	reads_zero = 0, reads_mask = 0, resets_cleared = 0,
	oracle = { sets = 0, reads = 0, resets = 0, mismatches = 0 },
	reply_faults = {}, request_faults = {},
	entry = "none", refusal = nil,
}
local function note_reply_problem (what, problem) state.reply_faults [#state.reply_faults + 1] = what .. ": " .. problem end
local function note_request_fault (what) state.request_faults [#state.request_faults + 1] = what end

-- any new host callback shows the engine accepted the unaltered reply before it (an altered one is never counted as a success), and ends every relation that needs "the very next callback"
local function settle ()
	if state.unsettled == "changed" then state.changed_accepted = 1
	elseif state.unsettled ~= nil then state.accepted [state.unsettled] = state.accepted [state.unsettled] + 1 end
	state.unsettled = nil
end

-- the REFUSAL LATCH: once an eligible fault is injected, the engine entry that received it must stop at once. ANY later host callback before its fatal ends the refusal credit (the first one is kept and named), even if
-- a later fatal carries the very same marker: that fatal would not show the altered answer was refused
local function note_callback (what)
	local r = state.refusal
	if r ~= nil and r.callback == nil then r.callback = what end
end

-- the flag question as the contract states it (mb_hostentity.h; entity-authority.lua's parser): nil and no problem for any other scan question, else the question or the first problem
local function parse_flag (key)
	local head, rest = string.match (key, "^(%S+)(.*)$")
	if head == nil then return nil end
	local words = strict_patterns (head, 16)
	if words == nil then return nil end
	local event = EVENT_OF [words [12]]
	if event == nil then return nil end
	local identity = ""
	if rest ~= "" then
		identity = string.match (rest, "^ (.+)$")
		if identity == nil then return nil, "a flag " .. event .. " question's identity does not follow the 16 patterns after exactly one space" end
	end
	local stated = event == "group" and { [1] = true, [12] = true } or (event == "reset" and { [12] = true, [13] = true } or { [12] = true, [13] = true, [14] = true })
	for index = 1, 16 do
		if not stated [index] and words [index] ~= ZERO then return nil, "a flag " .. event .. " states nothing but its phase and the facts of its grammar" end
	end
	local parsed = { event = event, words = words, identity = identity }
	if event == "group" then
		if words [1] ~= ZERO and words [1] ~= ONE then return nil, "a flag group states an eligibility that is not 0.0 or 1.0" end
		if identity ~= "" then return nil, "a flag group names no group" end
		parsed.eligible = words [1] == ONE
		return parsed
	end
	parsed.array = ARRAYS [words [13]]
	if parsed.array == nil then return nil, "a flag " .. event .. " names the array " .. words [13] .. ", which is not 0.0, 1.0 or 2.0" end
	if event == "reset" then
		if identity ~= "" then return nil, "a flag reset names no group" end
		return parsed
	end
	parsed.lane = LANES [words [14]]
	if parsed.lane == nil then return nil, "a flag " .. event .. " names the lane " .. words [14] .. ", which is not a whole number from 0 to 31" end
	if not is_flag_key (identity) then return nil, "a flag " .. event .. " names \"" .. identity .. "\", which is not a flag key" end
	return parsed
end

-- the first problem of a REAL reply against the bridge's grammar (and the C bridge's own checks), or nil
local function reply_problem (answer, parsed)
	if type (answer) ~= "table" then return "the reply is not a table" end
	local event = parsed.event
	if answer.result ~= RESULT_OF [event] then return "result " .. tostring (answer.result) .. ", not " .. RESULT_OF [event] end
	local name = answer.name
	if event == "group" then
		if not is_flag_key (name) then return "a flag group was issued the key \"" .. tostring (name) .. "\", which is not a flag key (`f`, a first digit 1 to 9, at most 20 bytes)" end
		if state.keys [name] then return "a flag group was issued the key \"" .. name .. "\", which is already bound to another flag group" end
	elseif name ~= nil and name ~= "" then
		return "a flag " .. event .. " named the key \"" .. tostring (name) .. "\""
	end
	local p = strict_patterns (answer.p, 3)
	if p == nil then return "p is not exactly three 0x%08x patterns" end
	if event == "group" then
		if p [1] ~= parsed.words [1] or p [2] ~= ZERO or p [3] ~= ZERO then return "a flag group answered another eligibility than the one stated, or more than it" end
	elseif event == "reset" then
		if p [1] ~= parsed.words [13] or p [3] ~= ZERO then return "a flag reset answered another array than the one named, or a third word" end
	else
		if p [1] ~= parsed.words [13] or p [2] ~= parsed.words [14] then return "a flag " .. event .. " answered for another flag word than the one asked about" end
		local mask = mask_pattern (parsed.lane)
		if p [3] ~= mask and (event == "set" or p [3] ~= ZERO) then return "a flag " .. event .. " answered the raw word " .. p [3] .. ", which is " .. (event == "set" and "not the lane's mask" or "neither 0 nor the lane's mask") end
	end
	return nil
end

-- the BACKING oracle over the observed traffic: the destructive `&=` semantics of the original's getter, rebuilt from the sets, reads and resets the campaign made. Applied to the REAL reply only; a difference is the HOST's fault.
local function lanes_of (group, array)
	group.lanes [array + 1] = group.lanes [array + 1] or {}
	return group.lanes [array + 1]
end
local function oracle (parsed, real)
	local p = strict_patterns (real.p, 3)
	local o = state.oracle
	if parsed.event == "group" then
		state.keys [real.name] = true
		state.key_order [#state.key_order + 1] = real.name
		state.groups [real.name] = { eligible = parsed.eligible, lanes = {} }
		return
	end
	if parsed.event == "reset" then
		o.resets = o.resets + 1
		local count = 0
		for _, key in ipairs (state.key_order) do
			local group = state.groups [key]
			if group.eligible then
				group.lanes [parsed.array + 1] = {}
				count = count + 1
			end
		end
		state.resets_cleared = state.resets_cleared + count
		if p [2] ~= whole_pattern (count) then
			o.mismatches = o.mismatches + 1
			note_reply_problem ("reset", "the host states " .. p [2] .. " groups cleared, the traffic opened " .. count .. " eligible ones")
		end
		return
	end
	local group = state.groups [parsed.identity]
	if group == nil then
		o.mismatches = o.mismatches + 1
		note_reply_problem (parsed.event, "the campaign named the key \"" .. parsed.identity .. "\", which the host never issued for a group")
		return
	end
	local lanes = lanes_of (group, parsed.array)
	if parsed.event == "set" then
		o.sets = o.sets + 1
		lanes [parsed.lane] = true
		return
	end
	o.reads = o.reads + 1
	local expected = lanes [parsed.lane] and mask_pattern (parsed.lane) or ZERO
	-- the destructive read: every OTHER lane of the shared word is destroyed, the lane's own is kept
	group.lanes [parsed.array + 1] = { [parsed.lane] = lanes [parsed.lane] }
	if p [3] ~= expected then
		o.mismatches = o.mismatches + 1
		note_reply_problem ("read", "the host answered the raw word " .. p [3] .. " for lane " .. parsed.lane .. ", the traffic implies " .. expected)
	end
	if p [3] == ZERO then state.reads_zero = state.reads_zero + 1 else state.reads_mask = state.reads_mask + 1 end
end

local function fault_reply (kind, real, parsed)
	local p = strict_patterns (real.p, 3)
	local function with (index, value)
		local q = { p [1], p [2], p [3] }
		q [index] = value
		return table.concat (q, ",")
	end
	if kind == "withheld" then error ("the host withholds its flag answer", 0) end
	if kind == "malformed" then return { result = real.result, name = real.name, p = p [1] .. "," .. p [2] } end
	if kind == "wrong_state" then return { result = "height", p = real.p } end
	if kind == "bad_key" then return { result = real.result, name = "f" .. string.rep ("1", 20), p = real.p } end
	if kind == "duplicate_key" then return { result = real.result, name = state.key_order [1], p = real.p } end
	if kind == "wrong_eligibility" then return { result = real.result, name = real.name, p = with (1, p [1] == ZERO and ONE or ZERO) } end
	if kind == "wrong_lane" then return { result = real.result, p = with (2, whole_pattern ((parsed.lane + 1) % 32)) } end
	if kind == "wrong_array" then return { result = real.result, p = with (1, whole_pattern ((parsed.array + 1) % 3)) } end
	if kind == "wrong_word" then return { result = real.result, p = with (3, mask_pattern ((parsed.lane + 1) % 32)) } end
	error ("unknown fault kind " .. tostring (kind), 0)
end

local function counts_before ()
	local a = state.attempts
	return { group = a.group, set = a.set, read = a.read, reset = a.reset }
end
local function note_first (name, before)
	state.first [name] = { ordinal = state.ordinal, frame = state.frame, before = before }
	state.first_order [#state.first_order + 1] = name
end
-- the step whose trajectory the intervention may be made at: the control's, field for field
local function trajectory_problem (name, before)
	local t = reference.first
	local mine = { ordinal = tostring (state.ordinal), frame = tostring (state.frame), before_group = tostring (before.group), before_set = tostring (before.set), before_read = tostring (before.read), before_reset = tostring (before.reset) }
	for _, key in ipairs (FIRST_FIELDS) do
		if mine [key] ~= t [key] then
			return string.format ("the trajectory of the first %s differs from the control's (%s: control %s, this run %s): no intervention was made on another step", name, key, t [key], mine [key])
		end
	end
	return nil
end

local function candidate_parts (key)
	if type (key) ~= "string" then return nil end
	local head, rest = string.match (key, "^(%S+)(.*)$")
	local words = head and strict_patterns (head, 44)
	if words == nil then return nil end
	return words, (string.match (rest, "^ (.+)$") or "")
end

local function candidate (scene, depth, kind, definition, key)
	local pending = state.pending_consume
	state.pending_consume = nil
	local words, identity = candidate_parts (key)
	if pending ~= nil and words ~= nil and scene == pending.scene and identity == pending.identity and pending.words [21] == ZERO then
		-- the very next candidate question of the same candidate: all its facts as before, but the UPDATED fact (the 21st pattern: 1.0 not updated, 2.0 updated) the campaign now states
		local same = true
		for index = 1, 44 do
			if index ~= 21 and words [index] ~= pending.words [index] then same = false end
		end
		if same then
			state.consumed = { fact = words [21], expected = pending.answered_updated and TWO or ONE, host = pending.real_updated and TWO or ONE, real_word = pending.real_word, answered_word = pending.answered_word, changed = pending.changed and 1 or 0 }
		end
	end
	local answer = host.model ("entity_candidate", scene, depth, kind, definition, key)
	if type (answer) == "table" and answer.result == "need_updated" and words ~= nil and words [21] == ZERO then
		state.need_updated = { scene = scene, identity = identity, words = words }
	end
	return answer
end

-- the host's model provider: every question is delegated unchanged except the ONE eligible intervention of the mode
local function model (op, scene, depth, kind, definition, key)
	note_callback ("model " .. tostring (op))
	settle ()
	local prior_need = state.need_updated
	state.need_updated = nil
	if op == "entity_candidate" then return candidate (scene, depth, kind, definition, key) end
	state.pending_consume = nil
	if op ~= "entity_scan" or type (key) ~= "string" then return host.model (op, scene, depth, kind, definition, key) end
	local parsed, problem = parse_flag (key)
	if parsed == nil then
		if problem ~= nil then note_request_fault ("entity_scan request: " .. problem) end
		return host.model (op, scene, depth, kind, definition, key)
	end
	local event = parsed.event
	local before = counts_before ()
	state.ordinal = state.ordinal + 1
	local consumable = event == "read" and prior_need ~= nil
	local here = { event }
	if event == "group" and state.attempts.group == 1 then here [#here + 1] = "group_second" end
	if consumable and not state.consumable_seen then
		state.consumable_seen = true
		here [#here + 1] = "read_consumable"
	end
	local eligible = false
	for _, name in ipairs (here) do
		if state.first [name] == nil then note_first (name, before) end
		if name == plan.at and state.intervened == 0 and state.abort == nil then
			local why = trajectory_problem (name, before)
			if why ~= nil then state.abort = why else eligible = true end
		end
	end
	state.attempts [event] = state.attempts [event] + 1
	local real = host.model (op, scene, depth, kind, definition, key)
	local wrong = reply_problem (real, parsed)
	if wrong ~= nil then
		-- a malformed positive is the host's own failure, never the configured single fault
		note_reply_problem (event .. " ordinal " .. state.ordinal, wrong)
		return real
	end
	oracle (parsed, real)
	state.answered [event] = state.answered [event] + 1
	if eligible then
		state.intervened, state.altered = 1, 1
		state.intervention = event .. " ordinal " .. state.ordinal
		if plan.changed then
			local p = strict_patterns (real.p, 3)
			local flipped = p [3] == ZERO and mask_pattern (parsed.lane) or ZERO
			state.pending_consume = { scene = prior_need.scene, identity = prior_need.identity, words = prior_need.words, real_updated = p [3] ~= ZERO, answered_updated = flipped ~= ZERO, real_word = p [3], answered_word = flipped, changed = true }
			state.unsettled = "changed"
			return { result = "flag_read", p = p [1] .. "," .. p [2] .. "," .. flipped }
		end
		state.refusal = { event = event, entry = state.entry, callback = nil, returned = false }
		return fault_reply (plan.fault, real, parsed)
	end
	if consumable then
		state.reads_consumable = state.reads_consumable + 1
		local p = strict_patterns (real.p, 3)
		state.pending_consume = { scene = prior_need.scene, identity = prior_need.identity, words = prior_need.words, real_updated = p [3] ~= ZERO, answered_updated = p [3] ~= ZERO, real_word = p [3], answered_word = p [3], changed = false }
	end
	state.unsettled = event
	return real
end

local function settling (name)
	return function (...)
		note_callback (name)
		settle ()
		state.need_updated, state.pending_consume = nil, nil
		return host [name] (...)
	end
end

-- the ACTUAL callables host.terrain_provider () and host.env_provider () return are genuine host callback surfaces the engine calls (the environment function is also the default sample metadata and sound backend provider): they join the same continuation tracking as every other
-- callback. The wrapper forwards every argument and every result exactly once (a tail call: no extra query, nothing held, no changed error, state or order); a value that is not a function is passed through untouched so the module's own refusal stands
local function continuing (name, callable)
	if type (callable) ~= "function" then return callable end
	return function (...)
		note_callback (name)
		settle ()
		state.need_updated, state.pending_consume = nil, nil
		return callable (...)
	end
end

local function engine_marker (err)
	return string.match (tostring (err), "%[model%-provider:([%w_%.]+)%]")
end
local function first_line (text) return string.match (tostring (text), "[^\r\n]+") or tostring (text) end

local dc = require ("eech_dc")
local engine
local fatal -- the first fatal the engine raised: { marker =, line =, where = }
local function guarded (where, fn)
	local ok, err = pcall (fn)
	if not ok then fatal = { marker = engine_marker (err), line = first_line (err), where = where, entry = state.entry } end
	return ok, err
end

local function boot_config ()
	return {
		install_root = root,
		map = effective_map,
		terrain = continuing ("terrain", host.terrain_provider (root, effective_map)), environment = continuing ("environment", host.env_provider ()),
		campaign_directory = "camp01",
		campaign = effective_campaign,
		gunship = "apache",
		seed = seed,
		bind_mobile = settling ("bind_mobile"),
		move_vehicle = settling ("move_vehicle"),
		observe_loss = settling ("observe_loss"),
		supply_reached = settling ("supply_reached"),
		line_of_sight = settling ("line_of_sight"),
		model = model,
		sense = settling ("sense"),
		aim = settling ("aim"),
	}
end

local status, detail = "FAILURE", "no verdict was reached"
local function verdict (s, d) status, detail = s, d end

local function run ()
	-- the engine's BOOT is an entry like a frame: its fatal is captured here, at the entry (nothing is read, counted or closed through a poisoned engine afterwards), and flag traffic during boot is observed like any other
	state.entry = "boot"
	local booted = guarded ("boot", function () engine = dc.boot (boot_config ()) end)
	if not booted then return end
	settle ()
	if state.refusal ~= nil then
		-- the boot returned normally after an altered answer: it was not refused, and nothing further is run after it
		state.refusal.returned = true
		return
	end
	local frames = math.floor (hours * 3600 * 1000 / frame_ms)
	for _ = 1, frames do
		state.frame = state.frame + 1
		state.entry = "frame:" .. state.frame
		local ok = guarded ("frame", function () engine:frame (frame_ms) end)
		state.frames_run = state.frames_run + 1
		if not ok then break end
		if state.refusal ~= nil then state.refusal.returned = true end
		settle ()
		if state.abort ~= nil then break end
		if state.consumed ~= nil and plan.changed then break end
		if state.consumed ~= nil and mode == "flags_control" and state.accepted.group > 0 and state.accepted.set > 0 and state.accepted.reset > 0 then break end
		if state.intervened == 1 and plan.changed == nil then break end
	end
end

local function totals (t) return t.group + t.set + t.read + t.reset end
local function faults () return #state.reply_faults + #state.request_faults end
local function counts_text (t) return string.format ("group=%d set=%d read=%d reset=%d", t.group, t.set, t.read, t.reset) end

local function decide ()
	if state.abort ~= nil then return verdict ("FAILURE", state.abort) end
	if mode == "flags_original" then
		if fatal ~= nil then return verdict ("FAILURE", "the engine stopped: " .. fatal.line) end
		if totals (state.attempts) > 0 then return verdict ("FAILURE", "the host WAS asked a flag question (" .. counts_text (state.attempts) .. "): this is not the original-solver profile") end
		if faults () > 0 then return verdict ("FAILURE", "a request or reply fault was recorded") end
		if state.frames_run < 1 then return verdict ("UNREACHED", "no frame ran") end
		return verdict ("PASS", "the original-solver profile consumed its own legacy words: no flag question reached the host in " .. state.frames_run .. " frames")
	end
	if fatal ~= nil and (state.intervened == 0 or plan.changed or plan.marker == nil) then return verdict ("FAILURE", (state.intervened == 0 and "a fatal before any intervention: " or "a fatal after the valid changed answer: ") .. fatal.line) end
	if faults () > 0 then return verdict ("FAILURE", "a request or reply fault or a backing difference was recorded (" .. (state.reply_faults [1] or state.request_faults [1]) .. ")") end
	if mode == "flags_control" then
		for _, event in ipairs ({ "group", "set", "reset" }) do
			if state.accepted [event] < 1 then return verdict ("UNREACHED", "no flag " .. event .. " was accepted in " .. state.frames_run .. " frames") end
		end
		if state.consumed ~= nil and state.consumed.fact ~= state.consumed.expected then return verdict ("FAILURE", "the campaign stated UPDATED " .. state.consumed.fact .. " after the host answered " .. state.consumed.real_word .. " (expected " .. state.consumed.expected .. ")") end
		return verdict ("PASS", string.format ("attempts %s; accepted %s; reads_consumable=%d consumed=%d in %d frames", counts_text (state.attempts), counts_text (state.accepted), state.reads_consumable, state.consumed and 1 or 0, state.frames_run))
	end
	if plan.changed then
		if state.intervened == 0 then return verdict ("UNREACHED", "no read followed an UPDATED-asking candidate decision in " .. state.frames_run .. " frames") end
		local c = state.consumed
		if c == nil then return verdict ("UNREACHED", "the changed answer was made and accepted but its candidate question was not observed next") end
		if c.fact ~= c.expected or c.fact == c.host then return verdict ("FAILURE", "the campaign did not carry the changed word: UPDATED " .. c.fact .. ", the changed word says " .. c.expected .. ", the host's own said " .. c.host) end
		return verdict ("PASS", "the campaign carried the host's changed answer (" .. c.real_word .. " -> " .. c.answered_word .. ") into the next candidate question: UPDATED " .. c.fact .. " not " .. c.host)
	end
	if state.intervened == 0 then
		if fatal ~= nil then return verdict ("FAILURE", "a fatal before any intervention: " .. fatal.line) end
		return verdict ("UNREACHED", "the event " .. plan.at .. " was not reached in " .. state.frames_run .. " frames")
	end
	local r = state.refusal
	if fatal == nil then return verdict ("FAILURE", "the engine accepted the single fault made at " .. state.intervention .. (r and r.callback and (" and went on with a host callback (" .. r.callback .. ")") or "") .. (r and r.returned and " and returned normally from its entry" or "")) end
	if fatal.marker ~= plan.marker then return verdict ("FAILURE", "the engine stopped at " .. tostring (fatal.marker) .. ", not " .. plan.marker .. ": " .. fatal.line) end
	-- the causal boundary: the fatal must be the immediate refusal of the altered answer, in the entry that received it, with no host callback in between
	if r == nil then return verdict ("FAILURE", "the refusal latch was not set at " .. state.intervention) end
	if r.callback ~= nil then return verdict ("FAILURE", "a host callback (" .. r.callback .. ") followed the altered answer made at " .. state.intervention .. " before the engine stopped at " .. plan.marker .. ": that fatal does not show an immediate refusal") end
	if fatal.entry ~= r.entry then return verdict ("FAILURE", "the fatal came from the engine entry " .. tostring (fatal.entry) .. ", not the entry " .. tostring (r.entry) .. " that received the altered answer made at " .. state.intervention) end
	return verdict ("PASS", "the engine refused the single fault made at " .. state.intervention .. " at " .. plan.marker .. " immediately, in its entry " .. r.entry)
end

local function report ()
	emit (string.format ("pf-witness-identity schema=1 mode=%s profile=%s scenario=%s seed=%s frame_ms=%s hours=%s campaign=%s map=%s paired=%d", mode, plan.profile, scenario_name, tostring (seed), tostring (frame_ms), tostring (hours), effective_campaign, effective_map, (args.original ~= nil or intervening) and 1 or 0))
	local a, n, k = state.attempts, state.answered, state.accepted
	emit (string.format ("pf-witness-counts frames=%d attempts_group=%d attempts_set=%d attempts_read=%d attempts_reset=%d answered_group=%d answered_set=%d answered_read=%d answered_reset=%d accepted_group=%d accepted_set=%d accepted_read=%d accepted_reset=%d altered=%d changed_accepted=%d keys_issued=%d reads_zero=%d reads_mask=%d reads_consumable=%d resets_cleared=%d oracle_sets=%d oracle_reads=%d oracle_resets=%d oracle_mismatches=%d",
		state.frames_run, a.group, a.set, a.read, a.reset, n.group, n.set, n.read, n.reset, k.group, k.set, k.read, k.reset, state.altered, state.changed_accepted, #state.key_order, state.reads_zero, state.reads_mask, state.reads_consumable, state.resets_cleared,
		state.oracle.sets, state.oracle.reads, state.oracle.resets, state.oracle.mismatches))
	emit (string.format ("pf-witness-oracle sets=%d reads=%d resets=%d mismatches=%d", state.oracle.sets, state.oracle.reads, state.oracle.resets, state.oracle.mismatches))
	for _, name in ipairs (state.first_order) do
		local f = state.first [name]
		emit (string.format ("pf-witness-first event=%s ordinal=%d frame=%d before_group=%d before_set=%d before_read=%d before_reset=%d", name, f.ordinal, f.frame, f.before.group, f.before.set, f.before.read, f.before.reset))
	end
	local c = state.consumed
	if c ~= nil then emit (string.format ("pf-witness-consume fact=%s expected=%s host=%s real_word=%s answered_word=%s changed=%d", c.fact, c.expected, c.host, c.real_word, c.answered_word, c.changed)) end
	if state.intervention ~= nil then emit (string.format ("pf-witness-intervention at=%s entry=%s callback_after=%s", string.gsub (state.intervention, "%s+", "_"), state.refusal and state.refusal.entry or "none", state.refusal and state.refusal.callback and string.gsub (state.refusal.callback, "%s+", "_") or "none")) end
	if fatal ~= nil then emit (string.format ("pf-witness-fatal where=%s marker=%s entry=%s line=%s", fatal.where, tostring (fatal.marker), tostring (fatal.entry), string.gsub (fatal.line, "%s+", " "))) end
	for _, fault in ipairs (state.reply_faults) do emit ("pf-witness-reply-fault " .. string.gsub (fault, "%s+", " ")) end
	for _, fault in ipairs (state.request_faults) do emit ("pf-witness-request-fault " .. string.gsub (fault, "%s+", " ")) end
	emit (string.format ("pf-witness-verdict mode=%s status=%s detail=%s", mode, status, string.gsub (detail, "%s+", " ")))
end

local ran, failure = xpcall (function ()
	dc.prepare_installation (root)
	host.use_reference_arithmetic ()
	run ()
	decide ()
end, function (e) return e end)
if not ran then verdict ("FAILURE", "the witness stopped: " .. first_line (failure)) end
local reported, report_failure = pcall (report)
local closed, close_error = out:close ()
if not reported then error (NAME .. ": the report could not be written: " .. tostring (report_failure), 0) end
if not closed then error (NAME .. ": OUTPUT FAILURE: closing " .. out_path .. " failed: " .. tostring (close_error), 0) end
if status ~= "PASS" then error (NAME .. ": " .. status .. ": " .. detail, 0) end
host.log (NAME .. ": " .. detail)
