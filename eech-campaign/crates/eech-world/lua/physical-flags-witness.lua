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
local function whole_fields (record, names, role, what)
	local out = {}
	for _, name in ipairs (names) do
		if whole_number (record [name]) == nil then refuse ("the " .. role .. " report's " .. what .. " record has no whole " .. name) end
		out [name] = record [name]
	end
	return out
end
-- the whole successful report of this driver at `path` for `role`, or a refusal naming what is missing or inconsistent with THIS run's setup
local function read_reference (path, role, expected_mode)
	local file = io.open (path, "rb")
	if file == nil then refuse ("cannot read the " .. role .. " report " .. path) end
	local rec, last, first_name = {}, nil, nil
	for line in file:lines () do
		line = string.gsub (line, "\r$", "")
		local name = string.match (line, "^(%S+)")
		if name == nil then file:close () refuse ("the " .. role .. " report holds an empty line") end
		if REFUSED_RECORD [name] then file:close () refuse ("the " .. role .. " report holds a " .. name .. " record: it is not a successful report") end
		local kind = KNOWN [name]
		if kind == nil then file:close () refuse ("the " .. role .. " report holds the unknown record " .. name) end
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
	if rec ["pf-witness-counts"] == nil then refuse ("the " .. role .. " report has no counts record") end
	return rec
end

local COUNT_FIELDS = { "frames", "attempts_group", "attempts_set", "attempts_read", "attempts_reset", "answered_group", "answered_set", "answered_read", "answered_reset",
	"accepted_group", "accepted_set", "accepted_read", "accepted_reset", "altered", "keys_issued", "reads_consumable", "oracle_mismatches" }
local FIRST_FIELDS = { "ordinal", "frame", "before_group", "before_set", "before_read", "before_reset" }
local reference = {}
if mode == "flags_control" and args.original ~= nil then
	-- the original-profile characterisation: whole, successful, of this setup, and the host never asked for any flag
	local rec = read_reference (args.original, "original", "flags_original")
	local counts = whole_fields (rec ["pf-witness-counts"], COUNT_FIELDS, "original", "counts")
	for _, event in ipairs (EVENTS) do
		if counts ["attempts_" .. event] ~= "0" then refuse ("the original report's host WAS asked a flag " .. event .. ": it is not the original-solver profile") end
	end
	if counts.altered ~= "0" or tonumber (counts.frames) < 1 then refuse ("the original report altered an answer or ran no frame") end
elseif intervening then
	-- the whole PAIRED control BEFORE any output, boot or intervention: its counts, the first event this mode intervenes at, and (a changed read) its consumption record, all present and well formed
	local rec = read_reference (args.control, "control", "flags_control")
	local counts = whole_fields (rec ["pf-witness-counts"], COUNT_FIELDS, "control", "counts")
	if rec ["pf-witness-identity"].paired ~= "1" then refuse ("the control report is not a PAIRED control (an unpaired control makes no parity and unlocks no intervention)") end
	if counts.altered ~= "0" or counts.oracle_mismatches ~= "0" then refuse ("the control report altered an answer or holds a backing difference: it is not an unaltered control") end
	for _, event in ipairs ({ "group", "set", "reset" }) do
		if tonumber (counts ["accepted_" .. event]) < 1 then refuse ("the control report's host never had a flag " .. event .. " accepted: it is not the hosted control") end
	end
	local at
	for _, row in ipairs (rec ["pf-witness-first"] or {}) do
		if row.event == plan.at then
			if at ~= nil then refuse ("the control report holds the first-event record " .. plan.at .. " twice") end
			at = whole_fields (row, FIRST_FIELDS, "control", "first-event " .. plan.at)
		end
	end
	if at == nil then refuse ("the control never reached the event " .. plan.at .. " this mode intervenes at: the case is UNREACHED on this setup, no intervention is made") end
	if plan.changed then
		local c = rec ["pf-witness-consume"]
		if c == nil or c.fact == nil or c.fact ~= c.expected or strict_patterns (c.fact, 1) == nil then refuse ("the control report holds no consumption of a read by its candidate question (the campaign must carry the host's own word first)") end
	end
	reference.first = at
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
}
local function note_reply_problem (what, problem) state.reply_faults [#state.reply_faults + 1] = what .. ": " .. problem end
local function note_request_fault (what) state.request_faults [#state.request_faults + 1] = what end

-- any new host callback shows the engine accepted the unaltered reply before it (an altered one is never counted as a success), and ends every relation that needs "the very next callback"
local function settle ()
	if state.unsettled == "changed" then state.changed_accepted = 1
	elseif state.unsettled ~= nil then state.accepted [state.unsettled] = state.accepted [state.unsettled] + 1 end
	state.unsettled = nil
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
		settle ()
		state.need_updated, state.pending_consume = nil, nil
		return host [name] (...)
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
	if not ok then fatal = { marker = engine_marker (err), line = first_line (err), where = where } end
	return ok, err
end

local function boot_config ()
	return {
		install_root = root,
		map = effective_map,
		terrain = host.terrain_provider (root, effective_map), environment = host.env_provider (),
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
	engine = dc.boot (boot_config ())
	local frames = math.floor (hours * 3600 * 1000 / frame_ms)
	for _ = 1, frames do
		state.frame = state.frame + 1
		local ok = guarded ("frame", function () engine:frame (frame_ms) end)
		state.frames_run = state.frames_run + 1
		if not ok then break end
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
	if fatal == nil then return verdict ("FAILURE", "the engine accepted the single fault made at " .. state.intervention) end
	if fatal.marker ~= plan.marker then return verdict ("FAILURE", "the engine stopped at " .. tostring (fatal.marker) .. ", not " .. plan.marker .. ": " .. fatal.line) end
	return verdict ("PASS", "the engine refused the single fault made at " .. state.intervention .. " at " .. plan.marker)
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
	if state.intervention ~= nil then emit ("pf-witness-intervention at=" .. string.gsub (state.intervention, "%s+", "_")) end
	if fatal ~= nil then emit (string.format ("pf-witness-fatal where=%s marker=%s line=%s", fatal.where, tostring (fatal.marker), string.gsub (fatal.line, "%s+", " "))) end
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
